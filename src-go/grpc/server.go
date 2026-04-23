package grpcworker

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net"
	"os"
	"strings"

	"github.com/bluephoenix/parallax-worker/ai"
	"github.com/bluephoenix/parallax-worker/health"
	"github.com/bluephoenix/parallax-worker/loadtest"
	"github.com/bluephoenix/parallax-worker/mock"
	"github.com/bluephoenix/parallax-worker/models"
	pb "github.com/bluephoenix/parallax-worker/proto"
	"github.com/bluephoenix/parallax-worker/proxy"
	"github.com/bluephoenix/parallax-worker/runner"
	"github.com/bluephoenix/parallax-worker/watcher"
	"google.golang.org/grpc"
	"gopkg.in/yaml.v3"
)

type Server struct {
	pb.UnimplementedWorkerServiceServer
	pb.UnimplementedProxyServiceServer
	pb.UnimplementedHealthServiceServer
	pb.UnimplementedLoadTestServiceServer
	pb.UnimplementedMockServiceServer
	pb.UnimplementedWatcherServiceServer
	pb.UnimplementedAIServiceServer
	pb.UnimplementedRunnerServiceServer

	grpcServer  *grpc.Server
	proxySvc    *proxy.Proxy
	healthSvc   *health.Monitor
	loadtestSvc *loadtest.Service
	mockSvc     *mock.Server
	watcherSvc  *watcher.Watcher
	aiSvc       *ai.AIService
}

func NewServer(watcherSvc *watcher.Watcher, loadtestSvc *loadtest.Service, healthSvc *health.Monitor, proxySvc *proxy.Proxy, mockSvc *mock.Server, aiSvc *ai.AIService) *Server {
	return &Server{
		grpcServer:  grpc.NewServer(),
		proxySvc:    proxySvc,
		healthSvc:   healthSvc,
		loadtestSvc: loadtestSvc,
		mockSvc:     mockSvc,
		watcherSvc:  watcherSvc,
		aiSvc:       aiSvc,
	}
}

func (s *Server) Serve(lis net.Listener) error {
	pb.RegisterWorkerServiceServer(s.grpcServer, s)
	pb.RegisterProxyServiceServer(s.grpcServer, s)
	pb.RegisterHealthServiceServer(s.grpcServer, s)
	pb.RegisterLoadTestServiceServer(s.grpcServer, s)
	pb.RegisterMockServiceServer(s.grpcServer, s)
	pb.RegisterWatcherServiceServer(s.grpcServer, s)
	pb.RegisterAIServiceServer(s.grpcServer, s)
	pb.RegisterRunnerServiceServer(s.grpcServer, s)
	return s.grpcServer.Serve(lis)
}

func (s *Server) GracefulStop() {
	if s.grpcServer != nil {
		s.grpcServer.GracefulStop()
	}
}

// -----------------------------------------------------------------------------
// WorkerService
// -----------------------------------------------------------------------------
func (s *Server) Ping(ctx context.Context, req *pb.GenericRequest) (*pb.GenericResponse, error) {
	log.Println("Ping received from Rust frontend")
	return &pb.GenericResponse{
		Success: true,
		Message: "Pong from Go Sidecar",
	}, nil
}

// -----------------------------------------------------------------------------
// ProxyService
// -----------------------------------------------------------------------------
func (s *Server) GetTraffic(ctx context.Context, req *pb.TrafficRequest) (*pb.TrafficList, error) {
	entries := s.proxySvc.GetTraffic(int(req.Limit))
	res := &pb.TrafficList{Entries: make([]*pb.TrafficEntry, 0, len(entries))}
	for _, e := range entries {
		res.Entries = append(res.Entries, &pb.TrafficEntry{
			Id:                 e.ID,
			TimestampMs:        e.Timestamp,
			Method:             e.Method,
			Url:                e.URL,
			RequestHeaders:     e.RequestHeaders,
			ResponseHeaders:    e.ResponseHeaders,
			StatusCode:         int32(e.StatusCode),
			LatencyMs:          e.LatencyMs,
			RequestBodyBytes:   int32(e.RequestBodySize),
			ResponseBodyBytes:  int32(e.ResponseBodySize),
			ContentType:        e.ContentType,
			Preview:            e.Preview,
		})
	}
	return res, nil
}

func (s *Server) ClearTraffic(ctx context.Context, req *pb.GenericRequest) (*pb.GenericResponse, error) {
	s.proxySvc.ClearTraffic()
	return &pb.GenericResponse{Success: true}, nil
}

func (s *Server) SetFilter(ctx context.Context, req *pb.TrafficFilter) (*pb.GenericResponse, error) {
	s.proxySvc.SetFilter(proxy.TrafficFilter{
		IncludeDomains: req.IncludeDomains,
		ExcludeDomains: req.ExcludeDomains,
		OnlyMethods:    req.OnlyMethods,
		MinStatus:      int(req.MinStatus),
	})
	return &pb.GenericResponse{Success: true}, nil
}

func (s *Server) WatchTraffic(req *pb.GenericRequest, stream pb.ProxyService_WatchTrafficServer) error {
	ch := make(chan proxy.TrafficEntry, 100)
	
	s.proxySvc.OnTraffic(func(e proxy.TrafficEntry) {
		select {
		case ch <- e:
		default: // drop if channel is full
		}
	})

	defer s.proxySvc.OnTraffic(nil) // Note: this removes the callback for all, might need multiple listeners support in proxy if needed, but fine for 1 UI client

	for {
		select {
		case <-stream.Context().Done():
			return nil
		case e := <-ch:
			err := stream.Send(&pb.TrafficEntry{
				Id:                 e.ID,
				TimestampMs:        e.Timestamp,
				Method:             e.Method,
				Url:                e.URL,
				RequestHeaders:     e.RequestHeaders,
				ResponseHeaders:    e.ResponseHeaders,
				StatusCode:         int32(e.StatusCode),
				LatencyMs:          e.LatencyMs,
				RequestBodyBytes:   int32(e.RequestBodySize),
				ResponseBodyBytes:  int32(e.ResponseBodySize),
				ContentType:        e.ContentType,
				Preview:            e.Preview,
			})
			if err != nil {
				return err
			}
		}
	}
}

// -----------------------------------------------------------------------------
// HealthService
// -----------------------------------------------------------------------------
func (s *Server) AddTarget(ctx context.Context, req *pb.ServiceTarget) (*pb.GenericResponse, error) {
	s.healthSvc.AddTarget(health.ServiceTarget{
		ID:          req.Id,
		Name:        req.Name,
		URL:         req.Url,
		IntervalSec: int(req.IntervalSec),
		Timeout:     int(req.TimeoutMs),
	})
	return &pb.GenericResponse{Success: true}, nil
}

func (s *Server) RemoveTarget(ctx context.Context, req *pb.TargetIDRequest) (*pb.GenericResponse, error) {
	s.healthSvc.RemoveTarget(req.Id)
	return &pb.GenericResponse{Success: true}, nil
}

func (s *Server) GetStatuses(ctx context.Context, req *pb.GenericRequest) (*pb.HealthStatusList, error) {
	statuses := s.healthSvc.GetStatuses()
	res := &pb.HealthStatusList{Statuses: make([]*pb.ServiceStatus, 0, len(statuses))}
	for _, st := range statuses {
		res.Statuses = append(res.Statuses, &pb.ServiceStatus{
			Id:          st.ID,
			Name:        st.Name,
			Url:         st.URL,
			Status:      st.Status,
			LatencyMs:   st.LatencyMs,
			StatusCode:  int32(st.StatusCode),
			LastChecked: st.LastChecked,
			ErrorMsg:    st.ErrorMsg,
		})
	}
	return res, nil
}

func (s *Server) WatchStatuses(req *pb.GenericRequest, stream pb.HealthService_WatchStatusesServer) error {
	ch := make(chan health.ServiceStatus, 100)
	
	s.healthSvc.OnChange(func(st health.ServiceStatus) {
		select {
		case ch <- st:
		default: // drop if full
		}
	})

	defer s.healthSvc.OnChange(nil)

	for {
		select {
		case <-stream.Context().Done():
			return nil
		case st := <-ch:
			err := stream.Send(&pb.ServiceStatus{
				Id:          st.ID,
				Name:        st.Name,
				Url:         st.URL,
				Status:      st.Status,
				LatencyMs:   st.LatencyMs,
				StatusCode:  int32(st.StatusCode),
				LastChecked: st.LastChecked,
				ErrorMsg:    st.ErrorMsg,
			})
			if err != nil {
				return err
			}
		}
	}
}

// -----------------------------------------------------------------------------
// LoadTestService
// -----------------------------------------------------------------------------
func (s *Server) RunLoadTest(req *pb.LoadTestConfig, stream pb.LoadTestService_RunLoadTestServer) error {
	progressCh := make(chan loadtest.ProgressEvent, 100)
	
	cfg := loadtest.Config{
		URL:         req.Url,
		Method:      req.Method,
		Headers:     req.Headers,
		Body:        req.Body,
		Concurrent:  int(req.Concurrent),
		TotalReqs:   int(req.TotalRequests),
		DurationSec: int(req.DurationSec),
	}

	var result *loadtest.Result
	var runErr error
	done := make(chan bool)

	go func() {
		result, runErr = s.loadtestSvc.Run(cfg, progressCh)
		close(done)
	}()

	for {
		select {
		case <-stream.Context().Done():
			s.loadtestSvc.Stop()
			return nil
		case p, ok := <-progressCh:
			if !ok {
				continue
			}
			err := stream.Send(&pb.LoadTestProgress{
				Completed:  p.Completed,
				Total:      p.Total,
				CurrentRps: p.CurrentRPS,
				Done:       false,
			})
			if err != nil {
				return err
			}
		case <-done:
			if runErr != nil {
				return runErr
			}
			if result == nil {
				return nil
			}
			// Send final result
			return stream.Send(&pb.LoadTestProgress{
				Done: true,
				Result: &pb.LoadTestResult{
					TotalRequests:  result.TotalRequests,
					Successful:     result.Successful,
					Failed:         result.Failed,
					AvgLatencyMs:   result.AvgLatencyMs,
					P50LatencyMs:   result.P50LatencyMs,
					P95LatencyMs:   result.P95LatencyMs,
					P99LatencyMs:   result.P99LatencyMs,
					MinLatencyMs:   result.MinLatencyMs,
					MaxLatencyMs:   result.MaxLatencyMs,
					ReqsPerSec:     result.ReqsPerSec,
					Errors:         result.Errors,
					Histogram:      result.Histogram,
				},
			})
		}
	}
}

func (s *Server) StopLoadTest(ctx context.Context, req *pb.StopRequest) (*pb.GenericResponse, error) {
	s.loadtestSvc.Stop()
	return &pb.GenericResponse{Success: true}, nil
}

// -----------------------------------------------------------------------------
// MockService
// -----------------------------------------------------------------------------
func (s *Server) ListRules(ctx context.Context, req *pb.GenericRequest) (*pb.MockRuleList, error) {
	rules, err := s.mockSvc.GetRules()
	if err != nil {
		return nil, err
	}

	pbRules := make([]*pb.MockRule, 0, len(rules))
	for _, r := range rules {
		pbRules = append(pbRules, &pb.MockRule{
			Id:          r.ID,
			Path:        r.Path,
			Method:      r.Method,
			StatusCode:  int32(r.StatusCode),
			Body:        r.Body,
			Headers:     r.Headers,
			ContentType: r.ContentType,
		})
	}
	return &pb.MockRuleList{Rules: pbRules}, nil
}

func (s *Server) AddRule(ctx context.Context, req *pb.MockRule) (*pb.GenericResponse, error) {
	s.mockSvc.AddRule(mock.MockRule{
		ID:          req.Id,
		Path:        req.Path,
		Method:      req.Method,
		StatusCode:  int(req.StatusCode),
		Body:        req.Body,
		Headers:     req.Headers,
		ContentType: req.ContentType,
	})
	return &pb.GenericResponse{Success: true}, nil
}

func (s *Server) RemoveRule(ctx context.Context, req *pb.TargetIDRequest) (*pb.GenericResponse, error) {
	s.mockSvc.RemoveRule(req.Id)
	return &pb.GenericResponse{Success: true}, nil
}

func (s *Server) GenerateTests(ctx context.Context, req *pb.AITestRequest) (*pb.AITestResponse, error) {
	js, yaml, err := s.aiSvc.GenerateTests(ctx, req.Model, req.Provider, req.ApiKey, req.BaseUrl, req.Method, req.Url, req.ResponseBody, int(req.ResponseStatus), req.ResponseHeaders)
	if err != nil {
		return nil, err
	}
	return &pb.AITestResponse{TestsJs: js, TestsYaml: yaml}, nil
}

func (s *Server) RepairRequest(ctx context.Context, req *pb.AIRepairRequest) (*pb.AIRepairResponse, error) {
	cfg := req.Config
	diagnosis, err := s.aiSvc.RepairRequest(ctx,
		cfg.Model, cfg.Provider, cfg.ApiKey, cfg.BaseUrl,
		req.Method, req.Url, req.RequestHeaders, req.RequestBody,
		int(req.ResponseStatus), req.ResponseBody, req.EnvKeys,
	)
	if err != nil {
		return nil, err
	}
	// Parse out fields from the JSON diagnosis string
	var parsed struct {
		Diagnosis string `json:"diagnosis"`
		Priority  string `json:"priority"`
		Fixes     []interface{} `json:"fixes"`
	}
	fixesJSON := "[]"
	priority := "medium"
	if err := json.Unmarshal([]byte(diagnosis), &parsed); err == nil {
		if b, e := json.Marshal(parsed.Fixes); e == nil {
			fixesJSON = string(b)
		}
		if parsed.Priority != "" {
			priority = parsed.Priority
		}
		if parsed.Diagnosis != "" {
			diagnosis = parsed.Diagnosis
		}
	}
	return &pb.AIRepairResponse{Diagnosis: diagnosis, FixesJson: fixesJSON, Priority: priority}, nil
}

func (s *Server) GenerateScript(ctx context.Context, req *pb.AIScriptRequest) (*pb.AIScriptResponse, error) {
	cfg := req.Config
	script, err := s.aiSvc.GenerateScript(ctx,
		cfg.Model, cfg.Provider, cfg.ApiKey, cfg.BaseUrl,
		req.ScriptType, req.UserPrompt, req.Method, req.Url,
	)
	if err != nil {
		return nil, err
	}
	return &pb.AIScriptResponse{Script: script}, nil
}

func (s *Server) CreateCollection(ctx context.Context, req *pb.AICollectionRequest) (*pb.AICollectionResponse, error) {
	cfg := req.Config
	yaml, err := s.aiSvc.CreateCollection(ctx,
		cfg.Model, cfg.Provider, cfg.ApiKey, cfg.BaseUrl,
		req.Description,
	)
	if err != nil {
		return nil, err
	}
	return &pb.AICollectionResponse{CollectionYaml: yaml}, nil
}

// -----------------------------------------------------------------------------
// WatcherService
// -----------------------------------------------------------------------------
func (s *Server) WatchWorkspace(req *pb.WatchRequest, stream pb.WatcherService_WatchWorkspaceServer) error {
	ch := make(chan watcher.ChangeEvent, 100)
	s.watcherSvc.OnChange(func(e watcher.ChangeEvent) {
		ch <- e
	})
	s.watcherSvc.WatchWorkspace(req.WorkspacePath)
	defer s.watcherSvc.UnwatchWorkspace(req.WorkspacePath)

	for {
		select {
		case <-stream.Context().Done():
			return nil
		case e := <-ch:
			err := stream.Send(&pb.FileChangeEvent{
				Path:          e.Path,
				Operation:     e.Operation,
				IsCollection:  e.IsCollection,
				IsEnvironment: e.IsEnvironment,
			})
			if err != nil {
				return err
			}
		}
	}
}

func (s *Server) UnwatchWorkspace(ctx context.Context, req *pb.WatchRequest) (*pb.GenericResponse, error) {
	s.watcherSvc.UnwatchWorkspace(req.WorkspacePath)
	return &pb.GenericResponse{Success: true}, nil
}

// -----------------------------------------------------------------------------
// RunnerService
// -----------------------------------------------------------------------------
func (s *Server) RunCollection(req *pb.RunRequest, stream pb.RunnerService_RunCollectionServer) error {
	colData, err := os.ReadFile(req.CollectionPath)
	if err != nil {
		return fmt.Errorf("collection not found: %w", err)
	}

	var col models.Collection
	if strings.HasSuffix(req.CollectionPath, ".yaml") || strings.HasSuffix(req.CollectionPath, ".yml") {
		if err := yaml.Unmarshal(colData, &col); err != nil {
			return fmt.Errorf("parse collection: %w", err)
		}
	} else {
		if err := json.Unmarshal(colData, &col); err != nil {
			return fmt.Errorf("parse collection: %w", err)
		}
	}

	env := make(map[string]string)
	if req.EnvironmentPath != "" {
		if envData, err := os.ReadFile(req.EnvironmentPath); err == nil {
			var envObj struct {
				Values    []struct{ Key string; Value interface{} } `json:"values"`
				Variables []struct{ Key string; Value interface{} } `json:"variables"`
			}
			json.Unmarshal(envData, &envObj)
			for _, v := range envObj.Values {
				env[v.Key] = fmt.Sprintf("%v", v.Value)
			}
			for _, v := range envObj.Variables {
				env[v.Key] = fmt.Sprintf("%v", v.Value)
			}
		}
	}

	emit := func(e runner.StreamEvent) {
		stream.Send(&pb.RunEvent{ //nolint
			Type:       e.Type,
			Name:       e.Name,
			StatusCode: int32(e.StatusCode),
			DurationMs: e.DurationMs,
			Error:      e.Error,
		})
	}

	if _, err := runner.RunCollectionStream(col, env, emit); err != nil {
		return err
	}
	return nil
}
