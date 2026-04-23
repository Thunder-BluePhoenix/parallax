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

	"github.com/bluephoenix/parallax-worker/ai"
	"github.com/bluephoenix/parallax-worker/chat"
	"github.com/bluephoenix/parallax-worker/health"
	"github.com/bluephoenix/parallax-worker/loadtest"
	"github.com/bluephoenix/parallax-worker/mcp"
	"github.com/bluephoenix/parallax-worker/mock"
	"github.com/bluephoenix/parallax-worker/models"
	"github.com/bluephoenix/parallax-worker/proxy"
	"github.com/bluephoenix/parallax-worker/runner"
	"github.com/bluephoenix/parallax-worker/storage"
	"github.com/bluephoenix/parallax-worker/watcher"
	grpcworker "github.com/bluephoenix/parallax-worker/grpc"
	"path/filepath"
	"strings"
	"gopkg.in/yaml.v3"
	"time"
)

var (
	grpcPort  = flag.Int("grpc-port", 50151, "gRPC server port")
	proxyPort = flag.Int("proxy-port", 8888, "Local HTTP proxy port")
	mockPort  = flag.Int("mock-port", 9999, "Local Mock server port")
	mcpPort   = flag.Int("mcp-port", 7676, "MCP server port (localhost only)")
	mcpToken  = flag.String("mcp-token", "", "Bearer token for MCP auth (empty = no auth)")
	mcpEnable = flag.Bool("mcp", false, "Enable MCP server")
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

	// Start team chat HTTP+SSE server
	go chat.Start(50152)

	// Start MCP server (optional)
	var mcpSrv *mcp.Server
	if *mcpEnable {
		mcpSrv = mcp.New(*mcpPort, *mcpToken)
		// Wire service hooks so MCP tools can call into live subsystems
		mcpSrv.SetTrafficGetter(func(limit int) []map[string]interface{} {
			entries := proxySvc.GetTraffic(limit)
			result := make([]map[string]interface{}, 0, len(entries))
			for _, e := range entries {
				result = append(result, map[string]interface{}{
					"id": e.ID, "method": e.Method, "url": e.URL,
					"status": e.StatusCode, "latency_ms": e.LatencyMs,
					"timestamp_ms": e.Timestamp, "preview": e.Preview,
				})
			}
			return result
		})
		go func() {
			log.Printf("MCP server listening on localhost:%d", *mcpPort)
			if err := mcpSrv.Start(); err != nil {
				log.Printf("MCP server error: %v", err)
			}
		}()
	}

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
	delay := fs.Int("d", 0, "Delay between iterations in ms")
	verbose := fs.Bool("v", false, "Verbose output — print final env state")
	dataFile := fs.String("data", "", "Data file path (CSV or JSON array) for data-driven runs")
	reporter := fs.String("reporter", "text", "Reporter: text | html")
	fs.Parse(args)

	if fs.NArg() < 1 {
		fmt.Println("Usage: parallax run <collection.yaml> [options]")
		fmt.Println("Options:")
		fs.PrintDefaults()
		os.Exit(1)
	}

	colPath := fs.Arg(0)

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

	// Base environment
	baseEnv := make(map[string]string)
	loadEnvFile := func(path string, prefix string) {
		data, err := os.ReadFile(path)
		if err != nil {
			return
		}
		var envObj struct {
			Values    []struct{ Key string; Value interface{} } `json:"values"`
			Variables []struct{ Key string; Value interface{} } `json:"variables"`
		}
		json.Unmarshal(data, &envObj)
		for _, v := range envObj.Values {
			baseEnv[prefix+v.Key] = fmt.Sprintf("%v", v.Value)
		}
		for _, v := range envObj.Variables {
			baseEnv[prefix+v.Key] = fmt.Sprintf("%v", v.Value)
		}
	}
	if *envPath != "" {
		loadEnvFile(*envPath, "")
	}
	if *globalsPath != "" {
		loadEnvFile(*globalsPath, "global.")
	}

	// Load data rows for data-driven runs
	dataRows := loadDataFile(*dataFile)
	if len(dataRows) > 0 {
		*iterations = len(dataRows)
	}

	fmt.Printf("\n Running collection: %s\n", col.Name)
	fmt.Printf("   Iterations: %d | Delay: %dms", *iterations, *delay)
	if *dataFile != "" {
		fmt.Printf(" | Data: %s (%d rows)", *dataFile, len(dataRows))
	}
	fmt.Println()
	fmt.Println("--------------------------------------------------")

	var totalPassed, totalFailed int
	var totalDuration int64

	for iter := 1; iter <= *iterations; iter++ {
		if *iterations > 1 {
			fmt.Printf("\n[Iteration %d]\n", iter)
		}

		// Merge base env + data row override
		env := make(map[string]string, len(baseEnv))
		for k, v := range baseEnv {
			env[k] = v
		}
		if iter-1 < len(dataRows) {
			for k, v := range dataRows[iter-1] {
				env[k] = v
			}
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
	fmt.Printf("Run Summary:\n")
	fmt.Printf("   Passed:   %d\n", totalPassed)
	fmt.Printf("   Failed:   %d\n", totalFailed)
	fmt.Printf("   Duration: %dms\n", totalDuration)

	if *reporter == "html" {
		writeHTMLReport(col.Name, totalPassed, totalFailed, totalDuration)
	}

	if *verbose {
		fmt.Printf("\nFinal Environment State:\n")
		for k, v := range baseEnv {
			fmt.Printf("  %s: %s\n", k, v)
		}
	}

	if totalFailed > 0 {
		fmt.Printf("\nBuild FAILED (%d failed)\n", totalFailed)
		os.Exit(1)
	}
	fmt.Printf("\nBuild PASSED\n")
}

// loadDataFile parses a CSV or JSON array data file into a slice of row maps.
func loadDataFile(path string) []map[string]string {
	if path == "" {
		return nil
	}
	data, err := os.ReadFile(path)
	if err != nil {
		fmt.Printf("Warning: could not read data file: %v\n", err)
		return nil
	}

	// JSON array: [{...}, {...}]
	if strings.HasSuffix(path, ".json") {
		var rows []map[string]interface{}
		if err := json.Unmarshal(data, &rows); err != nil {
			fmt.Printf("Warning: could not parse JSON data file: %v\n", err)
			return nil
		}
		result := make([]map[string]string, len(rows))
		for i, row := range rows {
			m := make(map[string]string, len(row))
			for k, v := range row {
				m[k] = fmt.Sprintf("%v", v)
			}
			result[i] = m
		}
		return result
	}

	// CSV: header row + data rows
	lines := strings.Split(strings.ReplaceAll(string(data), "\r\n", "\n"), "\n")
	if len(lines) < 2 {
		return nil
	}
	headers := strings.Split(lines[0], ",")
	var result []map[string]string
	for _, line := range lines[1:] {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		cells := strings.Split(line, ",")
		row := make(map[string]string, len(headers))
		for i, h := range headers {
			if i < len(cells) {
				row[strings.TrimSpace(h)] = strings.TrimSpace(cells[i])
			}
		}
		result = append(result, row)
	}
	return result
}

// writeHTMLReport writes a self-contained HTML report to .parallax/reports/.
func writeHTMLReport(name string, passed, failed int, durationMs int64) {
	os.MkdirAll(".parallax/reports", 0755)
	total := passed + failed
	pct := 0
	if total > 0 {
		pct = passed * 100 / total
	}
	status := "PASSED"
	statusColor := "#3fb950"
	if failed > 0 {
		status = "FAILED"
		statusColor = "#f85149"
	}
	html := fmt.Sprintf(`<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><title>Parallax Run Report — %s</title>
<style>
  body{font-family:system-ui,sans-serif;background:#0d1117;color:#c9d1d9;margin:0;padding:32px}
  h1{font-size:22px;margin-bottom:4px}
  .meta{font-size:13px;color:#8b949e;margin-bottom:32px}
  .summary{display:flex;gap:24px;margin-bottom:32px}
  .card{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:20px 28px}
  .card-label{font-size:11px;text-transform:uppercase;letter-spacing:.06em;color:#8b949e;margin-bottom:6px}
  .card-value{font-size:32px;font-weight:700;font-family:monospace}
  .status{display:inline-block;padding:6px 20px;border-radius:20px;font-weight:700;font-size:14px;background:%s22;color:%s;border:1px solid %s44}
</style>
</head>
<body>
<h1>%s</h1>
<p class="meta">Generated by parallax-cli • Duration: %dms</p>
<div class="summary">
  <div class="card"><div class="card-label">Status</div><div><span class="status">%s</span></div></div>
  <div class="card"><div class="card-label">Passed</div><div class="card-value" style="color:#3fb950">%d</div></div>
  <div class="card"><div class="card-label">Failed</div><div class="card-value" style="color:#f85149">%d</div></div>
  <div class="card"><div class="card-label">Total</div><div class="card-value">%d</div></div>
  <div class="card"><div class="card-label">Pass Rate</div><div class="card-value">%d%%</div></div>
</div>
</body></html>`,
		name,
		statusColor, statusColor, statusColor,
		name, durationMs,
		status,
		passed, failed, total, pct,
	)
	reportPath := fmt.Sprintf(".parallax/reports/run-%d.html", time.Now().UnixMilli())
	if err := os.WriteFile(reportPath, []byte(html), 0644); err == nil {
		fmt.Printf("   Report: %s\n", reportPath)
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
