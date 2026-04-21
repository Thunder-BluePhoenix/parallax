package grpcworker

import (
	"context"
	"log"
	"net"

	"github.com/bluephoenix/parallax-worker/health"
	"github.com/bluephoenix/parallax-worker/proxy"
	pb "github.com/bluephoenix/parallax-worker/proto"
	"google.golang.org/grpc"
)

type Server struct {
	pb.UnimplementedWorkerServiceServer
	pb.UnimplementedProxyServiceServer
	pb.UnimplementedHealthServiceServer

	grpcServer *grpc.Server
	proxySvc   *proxy.Proxy
	healthSvc  *health.Monitor
}

func NewServer(watcherSvc any, loadtestSvc any, healthSvc *health.Monitor, proxySvc *proxy.Proxy) *Server {
	return &Server{
		grpcServer: grpc.NewServer(),
		proxySvc:   proxySvc,
		healthSvc:  healthSvc,
	}
}

func (s *Server) Serve(lis net.Listener) error {
	pb.RegisterWorkerServiceServer(s.grpcServer, s)
	pb.RegisterProxyServiceServer(s.grpcServer, s)
	pb.RegisterHealthServiceServer(s.grpcServer, s)
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
