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
	"github.com/bluephoenix/parallax-worker/ai"
	"path/filepath"
	"strings"
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
	aiSvc := ai.New()

	// Start gRPC server
	grpcServer := grpcworker.NewServer(watcherSvc, loadtestSvc, healthSvc, proxySvc, mockSvc, aiSvc)

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
	globalsPath := fs.String("g", "", "Global variables file path")
	iterations := fs.Int("i", 1, "Number of iterations")
	delay := fs.Int("d", 0, "Delay between requests in ms")
	verbose := fs.Bool("v", false, "Verbose output")
	fs.Parse(args)

	if fs.NArg() < 1 {
		fmt.Println("Usage: parallax run <collection.yaml> [options]")
		fmt.Println("Options:")
		fs.PrintDefaults()
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
	if strings.HasSuffix(colPath, ".yaml") || strings.HasSuffix(colPath, ".yml") {
		if err := yaml.Unmarshal(colData, &col); err != nil {
			fmt.Printf("Error parsing YAML collection: %v\n", err)
			os.Exit(1)
		}
	} else {
		if err := json.Unmarshal(colData, &col); err != nil {
			fmt.Printf("Error parsing JSON collection: %v\n", err)
			os.Exit(1)
		}
	}

	// Load environment
	env := make(map[string]string)
	loadEnvFile := func(path string, prefix string) {
		data, err := os.ReadFile(path)
		if err != nil {
			return
		}
		var envObj struct {
			Values []struct {
				Key   string      `json:"key"`
				Value interface{} `json:"value"`
			} `json:"values"`
			Variables []struct { // Some formats use "variables"
				Key   string      `json:"key"`
				Value interface{} `json:"value"`
			} `json:"variables"`
		}
		json.Unmarshal(data, &envObj)
		for _, v := range envObj.Values {
			env[prefix+v.Key] = fmt.Sprintf("%v", v.Value)
		}
		for _, v := range envObj.Variables {
			env[prefix+v.Key] = fmt.Sprintf("%v", v.Value)
		}
	}

	if *envPath != "" {
		loadEnvFile(*envPath, "")
	}
	if *globalsPath != "" {
		loadEnvFile(*globalsPath, "global.")
	}

	fmt.Printf("\n🚀 Running collection: %s\n", col.Name)
	fmt.Printf("   Iterations: %d | Delay: %dms\n", *iterations, *delay)
	fmt.Println("--------------------------------------------------")

	var totalPassed, totalFailed int
	var totalDuration int64

	for iter := 1; iter <= *iterations; iter++ {
		if *iterations > 1 {
			fmt.Printf("\n[Iteration %d]\n", iter)
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

	fmt.Println("\n--------------------------------------------------")
	fmt.Printf("🏁 Run Summary:\n")
	fmt.Printf("   ✅ Passed: %d\n", totalPassed)
	fmt.Printf("   ❌ Failed: %d\n", totalFailed)
	fmt.Printf("   ⏱️  Total Duration: %dms\n", totalDuration)

	if totalFailed > 0 {
		fmt.Printf("\n❌ Build FAILED (%d failed assertions)\n", totalFailed)
		os.Exit(1)
	} else {
		fmt.Printf("\n✅ Build PASSED\n")
	}

	if *verbose {
		// Log detailed environment state if requested
		fmt.Printf("\nFinal Environment State:\n")
		for k, v := range env {
			fmt.Printf("  %s: %s\n", k, v)
		}
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
