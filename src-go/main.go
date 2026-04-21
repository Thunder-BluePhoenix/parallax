// Parallax Worker — Go Sidecar
// Handles: Git watching, load testing, health monitoring, local proxy
package main

import (
	"flag"
	"fmt"
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"

	"github.com/bluephoenix/parallax-worker/health"
	"github.com/bluephoenix/parallax-worker/loadtest"
	"github.com/bluephoenix/parallax-worker/proxy"
	"github.com/bluephoenix/parallax-worker/watcher"
	"github.com/bluephoenix/parallax-worker/grpc"
)

var (
	grpcPort  = flag.Int("grpc-port", 50151, "gRPC server port")
	proxyPort = flag.Int("proxy-port", 8888, "Local HTTP proxy port")
)

func main() {
	flag.Parse()

	log.SetPrefix("[parallax-worker] ")
	log.SetFlags(log.LstdFlags | log.Lshortfile)

	log.Printf("Starting Parallax Worker (gRPC :%d, Proxy :%d)", *grpcPort, *proxyPort)

	// Start gRPC server
	lis, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", *grpcPort))
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	// Initialize subsystems
	watcherSvc := watcher.New()
	loadtestSvc := loadtest.New()
	healthSvc := health.New()
	proxySvc := proxy.New(*proxyPort)

	// Start gRPC server
	grpcServer := grpcworker.NewServer(watcherSvc, loadtestSvc, healthSvc, proxySvc)

	go func() {
		log.Printf("gRPC server listening on :%d", *grpcPort)
		if err := grpcServer.Serve(lis); err != nil {
			log.Fatalf("gRPC server error: %v", err)
		}
	}()

	// Start local proxy
	go func() {
		log.Printf("Starting local proxy on :%d", *proxyPort)
		if err := proxySvc.Start(); err != nil {
			log.Printf("Proxy error: %v", err)
		}
	}()

	// Graceful shutdown
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	log.Println("Shutting down Parallax Worker...")
	grpcServer.GracefulStop()
	watcherSvc.Stop()
	healthSvc.Stop()
}
