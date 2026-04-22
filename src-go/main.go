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
	"encoding/json"

	"github.com/bluephoenix/parallax-worker/health"
	"github.com/bluephoenix/parallax-worker/loadtest"
	"github.com/bluephoenix/parallax-worker/proxy"
	"github.com/bluephoenix/parallax-worker/watcher"
	"github.com/bluephoenix/parallax-worker/grpc"
	"github.com/bluephoenix/parallax-worker/storage"
	"github.com/bluephoenix/parallax-worker/mock"
	"github.com/bluephoenix/parallax-worker/models"
	"github.com/bluephoenix/parallax-worker/runner"
	"path/filepath"
	"gopkg.in/yaml.v3"
)

var (
	grpcPort  = flag.Int("grpc-port", 50151, "gRPC server port")
	proxyPort = flag.Int("proxy-port", 8888, "Local HTTP proxy port")
	mockPort  = flag.Int("mock-port", 9999, "Local Mock server port")
)

func main() {
	flag.Parse()

	// Check for CLI subcommands
	if len(os.Args) > 1 {
		switch os.Args[1] {
		case "run":
			handleCLIRun(os.Args[2:])
			return
		case "mock":
			handleCLIMock(os.Args[2:])
			return
		}
	}

	log.SetPrefix("[parallax-worker] ")
	log.SetFlags(log.LstdFlags | log.Lshortfile)

	log.Printf("Starting Parallax Worker (gRPC :%d, Proxy :%d)", *grpcPort, *proxyPort)

	// Start gRPC server
	lis, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", *grpcPort))
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	// Initialize persistence
	dbPath := filepath.Join(".", ".parallax", "worker.db")
	store, err := storage.New(dbPath)
	if err != nil {
		log.Printf("Warning: Failed to init storage: %v", err)
	} else {
		defer store.Close()
	}

	// Initialize subsystems
	watcherSvc := watcher.New()
	loadtestSvc := loadtest.New()
	healthSvc := health.New(store)
	proxySvc := proxy.New(*proxyPort, store)
	mockSvc := mock.New(*mockPort)

	// Start gRPC server
	grpcServer := grpcworker.NewServer(watcherSvc, loadtestSvc, healthSvc, proxySvc, mockSvc)

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

	// Start local mock server
	go func() {
		log.Printf("Starting mock server on :%d", *mockPort)
		if err := mockSvc.Start(); err != nil {
			log.Printf("Mock server error: %v", err)
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

func handleCLIRun(args []string) {
	if len(args) < 1 {
		fmt.Println("Usage: parallax run <collection.yaml> [-e <environment.json>]")
		os.Exit(1)
	}

	colPath := args[0]
	envPath := ""
	for i := 1; i < len(args); i++ {
		if args[i] == "-e" && i+1 < len(args) {
			envPath = args[i+1]
			break
		}
	}

	// Load collection
	colData, err := os.ReadFile(colPath)
	if err != nil {
		fmt.Printf("Error reading collection: %v\n", err)
		os.Exit(1)
	}

	var col models.Collection
	if err := yaml.Unmarshal(colData, &col); err != nil {
		fmt.Printf("Error parsing collection: %v\n", err)
		os.Exit(1)
	}

	// Load environment
	env := make(map[string]string)
	if envPath != "" {
		envData, err := os.ReadFile(envPath)
		if err == nil {
			var envObj struct {
				Variables map[string]string `json:"variables"`
			}
			json.Unmarshal(envData, &envObj)
			env = envObj.Variables
		}
	}

	fmt.Printf("Running collection: %s\n", col.Name)
	fmt.Println("--------------------------------------------------")

	stats, err := runner.RunCollection(col, env)
	if err != nil {
		fmt.Printf("Run failed: %v\n", err)
		os.Exit(1)
	}

	fmt.Println("--------------------------------------------------")
	fmt.Printf("Run Summary: %d Passed, %d Failed, %dms Total\n", stats.Passed, stats.Failed, stats.DurationMs)

	if stats.Failed > 0 {
		os.Exit(1)
	}
}

func handleCLIMock(args []string) {
	if len(args) < 1 {
		fmt.Println("Usage: parallax mock <port>")
		os.Exit(1)
	}
	port := 9999
	fmt.Sscanf(args[0], "%d", &port)
	fmt.Printf("Starting standalone mock server on port %d...\n", port)
	
	mockSvc := mock.New(port)
	// Optionally load rules from a file here...
	
	if err := mockSvc.Start(); err != nil {
		fmt.Printf("Mock server error: %v\n", err)
		os.Exit(1)
	}
}
