package grpcworker

import (
	"context"
	"net"
	"log"

	"google.golang.org/grpc"
	pb "github.com/bluephoenix/parallax-worker/proto"
)

type Server struct {
	pb.UnimplementedWorkerServiceServer
	grpcServer *grpc.Server
}

func NewServer(watcherSvc, loadtestSvc, healthSvc, proxySvc any) *Server {
	return &Server{
		grpcServer: grpc.NewServer(),
	}
}

func (s *Server) Serve(lis net.Listener) error {
	pb.RegisterWorkerServiceServer(s.grpcServer, s)
	return s.grpcServer.Serve(lis)
}

func (s *Server) GracefulStop() {
	if s.grpcServer != nil {
		s.grpcServer.GracefulStop()
	}
}

func (s *Server) Ping(ctx context.Context, req *pb.GenericRequest) (*pb.GenericResponse, error) {
	log.Println("Ping received from Rust frontend")
	return &pb.GenericResponse{
		Success: true,
		Message: "Pong from Go Sidecar",
	}, nil
}
