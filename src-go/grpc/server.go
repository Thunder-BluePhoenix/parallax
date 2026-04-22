package grpcworker

import (
	"context"
	"log"
	"net"

	"github.com/bluephoenix/parallax-worker/health"
	"github.com/bluephoenix/parallax-worker/loadtest"
	"github.com/bluephoenix/parallax-worker/mock"
	"github.com/bluephoenix/parallax-worker/proxy"
	pb "github.com/bluephoenix/parallax-worker/proto"
	"github.com/bluephoenix/parallax-worker/watcher"
	"google.golang.org/grpc"
)

type Server struct {
	pb.UnimplementedWorkerServiceServer
	pb.UnimplementedProxyServiceServer
	pb.UnimplementedHealthServiceServer
	pb.UnimplementedLoadTestServiceServer
	pb.UnimplementedMockServiceServer
	pb.UnimplementedWatcherServiceServer

	grpcServer  *grpc.Server
	proxySvc    *proxy.Proxy
	healthSvc   *health.Monitor
	loadtestSvc *loadtest.Service
	mockSvc     *mock.Server
	watcherSvc  *watcher.Watcher
}

func NewServer(watcherSvc *watcher.Watcher, loadtestSvc *loadtest.Service, healthSvc *health.Monitor, proxySvc *proxy.Proxy, mockSvc *mock.Server) *Server {
	return &Server{
		grpcServer:  grpc.NewServer(),
		proxySvc:    proxySvc,
		healthSvc:   healthSvc,
		loadtestSvc: loadtestSvc,
		mockSvc:     mockSvc,
		watcherSvc:  watcherSvc,
	}
}

func (s *Server) Serve(lis net.Listener) error {
	pb.RegisterWorkerServiceServer(s.grpcServer, s)
	pb.RegisterProxyServiceServer(s.grpcServer, s)
	pb.RegisterHealthServiceServer(s.grpcServer, s)
	pb.RegisterLoadTestServiceServer(s.grpcServer, s)
	pb.RegisterMockServiceServer(s.grpcServer, s)
	pb.RegisterWatcherServiceServer(s.grpcServer, s)
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
