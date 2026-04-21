package grpcworker

import (
	"net"
)

type Server struct{}

func NewServer(watcherSvc, loadtestSvc, healthSvc, proxySvc any) *Server {
	return &Server{}
}

func (s *Server) Serve(lis net.Listener) error {
	// TODO: implement gRPC server
	return nil
}

func (s *Server) GracefulStop() {
	// TODO: implement graceful stop
}
