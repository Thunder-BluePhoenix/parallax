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
	"time"
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
		case "init":
			handleCLIInit(os.Args[2:])
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
	mockSvc := mock.New(*mockPort, store)

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
	fs := flag.NewFlagSet("run", flag.ExitOnError)
	envPath := fs.String("e", "", "Environment file path")
	iterations := fs.Int("i", 1, "Number of iterations")
	delay := fs.Int("d", 0, "Delay between requests in ms")
	fs.Parse(args)

	if fs.NArg() < 1 {
		fmt.Println("Usage: parallax run <collection.yaml> [-e <environment.json>] [-i <iterations>] [-d <delay_ms>]")
		os.Exit(1)
	}

	colPath := fs.Arg(0)

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
	if *envPath != "" {
		envData, err := os.ReadFile(*envPath)
		if err == nil {
			var envObj struct {
				Variables []struct {
					Key   string `json:"key"`
					Value string `json:"value"`
				} `json:"variables"`
			}
			json.Unmarshal(envData, &envObj)
			for _, v := range envObj.Variables {
				env[v.Key] = v.Value
			}
		}
	}

	fmt.Printf("Running collection: %s (%d iterations)\n", col.Name, *iterations)
	fmt.Println("--------------------------------------------------")

	var totalPassed, totalFailed int
	var totalDuration int64

	for iter := 1; iter <= *iterations; iter++ {
		if *iterations > 1 {
			fmt.Printf("\nIteration %d:\n", iter)
		}
		stats, err := runner.RunCollection(col, env)
		if err != nil {
			fmt.Printf("Run failed: %v\n", err)
			os.Exit(1)
		}
		totalPassed += stats.Passed
		totalFailed += stats.Failed
		totalDuration += stats.DurationMs
		
		if *delay > 0 && iter < *iterations {
			time.Sleep(time.Duration(*delay) * time.Millisecond)
		}
	}

	fmt.Println("--------------------------------------------------")
	fmt.Printf("Run Summary: %d Passed, %d Failed, %dms Total\n", totalPassed, totalFailed, totalDuration)

	if totalFailed > 0 {
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
	
	mockSvc := mock.New(port, nil)
	// Optionally load rules from a file here...
	
	if err := mockSvc.Start(); err != nil {
		fmt.Printf("Mock server error: %v\n", err)
		os.Exit(1)
	}
}

func handleCLIInit(args []string) {
	fmt.Println("Initializing Parallax workspace...")
	
	dirs := []string{
		".parallax",
		".parallax/collections",
		".parallax/environments",
		".parallax/history",
		".parallax/reports",
	}

	for _, dir := range dirs {
		if err := os.MkdirAll(dir, 0755); err != nil {
			fmt.Printf("Error creating directory %s: %v\n", dir, err)
			os.Exit(1)
		}
	}

	// Create a dummy environment
	dummyEnv := map[string]interface{}{
		"name": "Default",
		"variables": []map[string]string{
			{"key": "baseUrl", "value": "http://localhost:8080"},
		},
	}
	envData, _ := json.MarshalIndent(dummyEnv, "", "  ")
	os.WriteFile(".parallax/environments/default.json", envData, 0644)

	fmt.Println("✓ Workspace initialized. You can now use 'parallax run'.")
}
