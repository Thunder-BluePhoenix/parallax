// Parallax MCP Server
// Implements the Model Context Protocol over HTTP+SSE, exposing Parallax tools
// to AI agents such as Claude Desktop and custom automation workflows.
//
// Default port: 7676 (configurable)
// Endpoints:
//   GET  /mcp/sse      – SSE stream for MCP messages
//   POST /mcp/message  – Client messages to server
//   GET  /mcp/health   – Liveness probe

package mcp

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"sync"
	"sync/atomic"
	"time"
)

// ─── Protocol types ───────────────────────────────────────────────────────────

type MCPMessage struct {
	JSONRPC string      `json:"jsonrpc"`
	ID      interface{} `json:"id,omitempty"`
	Method  string      `json:"method,omitempty"`
	Params  interface{} `json:"params,omitempty"`
	Result  interface{} `json:"result,omitempty"`
	Error   *MCPError   `json:"error,omitempty"`
}

type MCPError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

type ToolDef struct {
	Name        string      `json:"name"`
	Description string      `json:"description"`
	InputSchema interface{} `json:"inputSchema"`
}

// ─── Service dependencies (injected) ─────────────────────────────────────────

type CollectionLister func() ([]map[string]interface{}, error)
type RequestRunner     func(ctx context.Context, colPath, reqID string, env map[string]string) (map[string]interface{}, error)
type TrafficGetter     func(limit int) []map[string]interface{}

// ─── Server ───────────────────────────────────────────────────────────────────

type Server struct {
	port       int
	token      string
	httpServer *http.Server
	mu         sync.RWMutex
	clients    map[uint64]chan string
	counter    atomic.Uint64
	enabled    bool

	// injected service hooks
	listCollections CollectionLister
	runRequest      RequestRunner
	getTraffic      TrafficGetter
}

func New(port int, token string) *Server {
	return &Server{
		port:    port,
		token:   token,
		clients: make(map[uint64]chan string),
	}
}

func (s *Server) SetCollectionLister(fn CollectionLister) { s.listCollections = fn }
func (s *Server) SetRequestRunner(fn RequestRunner)       { s.runRequest = fn }
func (s *Server) SetTrafficGetter(fn TrafficGetter)       { s.getTraffic = fn }

func (s *Server) Start() error {
	mux := http.NewServeMux()
	mux.HandleFunc("/mcp/health", s.handleHealth)
	mux.HandleFunc("/mcp/sse", s.handleSSE)
	mux.HandleFunc("/mcp/message", s.handleMessage)

	s.httpServer = &http.Server{
		Addr:    fmt.Sprintf("127.0.0.1:%d", s.port),
		Handler: mux,
	}

	s.enabled = true
	log.Printf("[mcp] Server listening on localhost:%d", s.port)
	if err := s.httpServer.ListenAndServe(); err != http.ErrServerClosed {
		return err
	}
	return nil
}

func (s *Server) Stop() {
	s.enabled = false
	if s.httpServer != nil {
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		s.httpServer.Shutdown(ctx)
	}
}

// ─── HTTP Handlers ────────────────────────────────────────────────────────────

func (s *Server) handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status":  "ok",
		"version": "1.0.0",
		"port":    s.port,
	})
}

func (s *Server) handleSSE(w http.ResponseWriter, r *http.Request) {
	// Auth check
	if s.token != "" && r.Header.Get("Authorization") != "Bearer "+s.token {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	w.Header().Set("Access-Control-Allow-Origin", "*")

	clientID := s.counter.Add(1)
	ch := make(chan string, 32)

	s.mu.Lock()
	s.clients[clientID] = ch
	s.mu.Unlock()

	defer func() {
		s.mu.Lock()
		delete(s.clients, clientID)
		s.mu.Unlock()
	}()

	// Send capabilities on connect
	caps := s.buildCapabilities()
	if data, err := json.Marshal(caps); err == nil {
		fmt.Fprintf(w, "data: %s\n\n", data)
		if f, ok := w.(http.Flusher); ok {
			f.Flush()
		}
	}

	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "SSE not supported", http.StatusInternalServerError)
		return
	}

	ticker := time.NewTicker(15 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-r.Context().Done():
			return
		case msg := <-ch:
			fmt.Fprintf(w, "data: %s\n\n", msg)
			flusher.Flush()
		case <-ticker.C:
			// heartbeat
			fmt.Fprintf(w, ": ping\n\n")
			flusher.Flush()
		}
	}
}

func (s *Server) handleMessage(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	if s.token != "" && r.Header.Get("Authorization") != "Bearer "+s.token {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	var msg MCPMessage
	if err := json.NewDecoder(r.Body).Decode(&msg); err != nil {
		http.Error(w, "Bad request", http.StatusBadRequest)
		return
	}

	resp := s.dispatch(r.Context(), msg)
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}

// ─── MCP Protocol ─────────────────────────────────────────────────────────────

func (s *Server) buildCapabilities() MCPMessage {
	tools := []ToolDef{
		{
			Name:        "parallax.list_collections",
			Description: "List all API collections in the current workspace.",
			InputSchema: map[string]interface{}{"type": "object", "properties": map[string]interface{}{}},
		},
		{
			Name:        "parallax.get_traffic",
			Description: "Fetch recent proxy traffic entries.",
			InputSchema: map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"limit": map[string]interface{}{"type": "integer", "description": "Max entries to return (default 50)"},
				},
			},
		},
		{
			Name:        "parallax.execute_request",
			Description: "Execute a specific request from a collection and return the response.",
			InputSchema: map[string]interface{}{
				"type": "object",
				"required": []string{"collection_path", "request_id"},
				"properties": map[string]interface{}{
					"collection_path": map[string]interface{}{"type": "string"},
					"request_id":      map[string]interface{}{"type": "string"},
					"env":             map[string]interface{}{"type": "object", "description": "Override environment variables"},
				},
			},
		},
	}

	return MCPMessage{
		JSONRPC: "2.0",
		Method:  "notifications/initialized",
		Params: map[string]interface{}{
			"protocolVersion": "2024-11-05",
			"serverInfo": map[string]interface{}{
				"name":    "Parallax",
				"version": "0.1.0",
			},
			"capabilities": map[string]interface{}{
				"tools": map[string]interface{}{
					"listChanged": false,
				},
			},
			"tools": tools,
		},
	}
}

func (s *Server) dispatch(ctx context.Context, msg MCPMessage) MCPMessage {
	base := MCPMessage{JSONRPC: "2.0", ID: msg.ID}

	params, _ := msg.Params.(map[string]interface{})

	switch msg.Method {
	case "initialize":
		base.Result = map[string]interface{}{
			"protocolVersion": "2024-11-05",
			"serverInfo": map[string]interface{}{
				"name":    "Parallax",
				"version": "0.1.0",
			},
			"capabilities": map[string]interface{}{
				"tools": map[string]interface{}{"listChanged": false},
			},
		}

	case "tools/list":
		caps := s.buildCapabilities()
		toolList := caps.Params.(map[string]interface{})["tools"]
		base.Result = map[string]interface{}{"tools": toolList}

	case "tools/call":
		toolName, _ := params["name"].(string)
		toolInput, _ := params["arguments"].(map[string]interface{})
		base.Result = s.callTool(ctx, toolName, toolInput)

	default:
		base.Error = &MCPError{Code: -32601, Message: fmt.Sprintf("method not found: %s", msg.Method)}
	}

	return base
}

func (s *Server) callTool(ctx context.Context, name string, args map[string]interface{}) interface{} {
	type contentBlock struct {
		Type string `json:"type"`
		Text string `json:"text"`
	}
	makeResult := func(data interface{}) map[string]interface{} {
		b, _ := json.MarshalIndent(data, "", "  ")
		return map[string]interface{}{
			"content": []contentBlock{{Type: "text", Text: string(b)}},
			"isError": false,
		}
	}
	makeError := func(msg string) map[string]interface{} {
		return map[string]interface{}{
			"content": []contentBlock{{Type: "text", Text: msg}},
			"isError": true,
		}
	}

	switch name {
	case "parallax.list_collections":
		if s.listCollections == nil {
			return makeError("collection lister not available")
		}
		cols, err := s.listCollections()
		if err != nil {
			return makeError(err.Error())
		}
		return makeResult(cols)

	case "parallax.get_traffic":
		if s.getTraffic == nil {
			return makeError("traffic getter not available")
		}
		limit := 50
		if v, ok := args["limit"].(float64); ok {
			limit = int(v)
		}
		return makeResult(s.getTraffic(limit))

	case "parallax.execute_request":
		if s.runRequest == nil {
			return makeError("request runner not available")
		}
		colPath, _ := args["collection_path"].(string)
		reqID, _ := args["request_id"].(string)
		env := map[string]string{}
		if e, ok := args["env"].(map[string]interface{}); ok {
			for k, v := range e {
				env[k] = fmt.Sprintf("%v", v)
			}
		}
		result, err := s.runRequest(ctx, colPath, reqID, env)
		if err != nil {
			return makeError(err.Error())
		}
		return makeResult(result)

	default:
		return map[string]interface{}{
			"content": []map[string]interface{}{{"type": "text", "text": fmt.Sprintf("unknown tool: %s", name)}},
			"isError": true,
		}
	}
}

// broadcast sends a message to all connected SSE clients.
func (s *Server) broadcast(msg MCPMessage) {
	data, err := json.Marshal(msg)
	if err != nil {
		return
	}
	s.mu.RLock()
	defer s.mu.RUnlock()
	for _, ch := range s.clients {
		select {
		case ch <- string(data):
		default:
		}
	}
}
