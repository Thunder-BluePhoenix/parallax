// Parallax LSP server — JSON-RPC 2.0 over stdin/stdout (standard LSP transport).
// Started with: parallax --lsp
//
// Capabilities:
//   - textDocument/completion  : {{VAR}} from active env + {% tag %} snippets
//   - textDocument/hover       : resolved value for {{VAR}} under cursor
//   - workspace/executeCommand : parallax.sendRequest, parallax.listCollections
//   - textDocument/publishDiagnostics: unresolved {{VAR}} not found in any env
package lsp

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"

	"github.com/bluephoenix/parallax-worker/models"
	"github.com/bluephoenix/parallax-worker/runner"
	"gopkg.in/yaml.v3"
)

// ── JSON-RPC 2.0 types ───────────────────────────────────────────────────────

type Request struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      interface{}     `json:"id,omitempty"`
	Method  string          `json:"method"`
	Params  json.RawMessage `json:"params,omitempty"`
}

type Response struct {
	JSONRPC string      `json:"jsonrpc"`
	ID      interface{} `json:"id"`
	Result  interface{} `json:"result,omitempty"`
	Error   *RPCError   `json:"error,omitempty"`
}

type Notification struct {
	JSONRPC string      `json:"jsonrpc"`
	Method  string      `json:"method"`
	Params  interface{} `json:"params,omitempty"`
}

type RPCError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

// ── LSP protocol types (minimal subset) ─────────────────────────────────────

type InitializeParams struct {
	RootURI string `json:"rootUri"`
}

type Position struct {
	Line      int `json:"line"`
	Character int `json:"character"`
}

type Range struct {
	Start Position `json:"start"`
	End   Position `json:"end"`
}

type TextDocumentIdentifier struct {
	URI string `json:"uri"`
}

type TextDocumentPositionParams struct {
	TextDocument TextDocumentIdentifier `json:"textDocument"`
	Position     Position               `json:"position"`
}

type CompletionItem struct {
	Label         string `json:"label"`
	Kind          int    `json:"kind"` // 6 = Variable, 15 = Snippet
	Detail        string `json:"detail,omitempty"`
	Documentation string `json:"documentation,omitempty"`
	InsertText    string `json:"insertText,omitempty"`
}

type Hover struct {
	Contents string `json:"contents"`
}

type Diagnostic struct {
	Range    Range  `json:"range"`
	Severity int    `json:"severity"` // 1=Error, 2=Warning, 3=Info, 4=Hint
	Message  string `json:"message"`
	Source   string `json:"source"`
}

type PublishDiagnosticsParams struct {
	URI         string       `json:"uri"`
	Diagnostics []Diagnostic `json:"diagnostics"`
}

type ExecuteCommandParams struct {
	Command   string            `json:"command"`
	Arguments []json.RawMessage `json:"arguments,omitempty"`
}

// ── Server state ─────────────────────────────────────────────────────────────

type Server struct {
	rootPath  string
	openDocs  map[string]string // uri → content
	envVars   map[string]string // merged env from .parallax/
	reader    *bufio.Reader
	writer    io.Writer
}

func New() *Server {
	return &Server{
		openDocs: make(map[string]string),
		envVars:  make(map[string]string),
		reader:   bufio.NewReader(os.Stdin),
		writer:   os.Stdout,
	}
}

// Run is the main loop — reads JSON-RPC messages until stdin closes.
func (s *Server) Run() {
	log.SetOutput(os.Stderr)
	log.Println("[lsp] Parallax LSP server started")

	for {
		msg, err := s.readMessage()
		if err != nil {
			if err == io.EOF {
				return
			}
			log.Printf("[lsp] read error: %v", err)
			continue
		}

		var req Request
		if err := json.Unmarshal(msg, &req); err != nil {
			log.Printf("[lsp] parse error: %v", err)
			continue
		}

		s.dispatch(req)
	}
}

// ── Message framing (Content-Length header) ──────────────────────────────────

func (s *Server) readMessage() ([]byte, error) {
	var contentLength int
	for {
		line, err := s.reader.ReadString('\n')
		if err != nil {
			return nil, err
		}
		line = strings.TrimSpace(line)
		if line == "" {
			break
		}
		if strings.HasPrefix(line, "Content-Length:") {
			n, _ := strconv.Atoi(strings.TrimSpace(strings.TrimPrefix(line, "Content-Length:")))
			contentLength = n
		}
	}
	if contentLength == 0 {
		return nil, fmt.Errorf("missing Content-Length")
	}
	buf := make([]byte, contentLength)
	_, err := io.ReadFull(s.reader, buf)
	return buf, err
}

func (s *Server) send(v interface{}) {
	data, err := json.Marshal(v)
	if err != nil {
		log.Printf("[lsp] marshal error: %v", err)
		return
	}
	fmt.Fprintf(s.writer, "Content-Length: %d\r\n\r\n%s", len(data), data)
}

func (s *Server) reply(id interface{}, result interface{}) {
	s.send(Response{JSONRPC: "2.0", ID: id, Result: result})
}

func (s *Server) replyError(id interface{}, code int, msg string) {
	s.send(Response{JSONRPC: "2.0", ID: id, Error: &RPCError{Code: code, Message: msg}})
}

func (s *Server) notify(method string, params interface{}) {
	s.send(Notification{JSONRPC: "2.0", Method: method, Params: params})
}

// ── Dispatch ─────────────────────────────────────────────────────────────────

func (s *Server) dispatch(req Request) {
	switch req.Method {
	case "initialize":
		s.handleInitialize(req)
	case "initialized":
		// no-op
	case "shutdown":
		s.reply(req.ID, nil)
	case "exit":
		os.Exit(0)
	case "textDocument/didOpen":
		s.handleDidOpen(req)
	case "textDocument/didChange":
		s.handleDidChange(req)
	case "textDocument/completion":
		s.handleCompletion(req)
	case "textDocument/hover":
		s.handleHover(req)
	case "workspace/executeCommand":
		s.handleExecuteCommand(req)
	default:
		if req.ID != nil {
			s.replyError(req.ID, -32601, "method not found: "+req.Method)
		}
	}
}

// ── initialize ───────────────────────────────────────────────────────────────

func (s *Server) handleInitialize(req Request) {
	var params InitializeParams
	json.Unmarshal(req.Params, &params)

	if params.RootURI != "" {
		s.rootPath = uriToPath(params.RootURI)
	} else if cwd, err := os.Getwd(); err == nil {
		s.rootPath = cwd
	}
	s.loadEnvVars()

	s.reply(req.ID, map[string]interface{}{
		"capabilities": map[string]interface{}{
			"textDocumentSync": 1, // Full sync
			"completionProvider": map[string]interface{}{
				"triggerCharacters": []string{"{", "%"},
			},
			"hoverProvider": true,
			"executeCommandProvider": map[string]interface{}{
				"commands": []string{"parallax.sendRequest", "parallax.listCollections"},
			},
		},
		"serverInfo": map[string]string{
			"name":    "parallax-lsp",
			"version": "1.0.0",
		},
	})
}

// ── textDocument/didOpen + didChange ─────────────────────────────────────────

func (s *Server) handleDidOpen(req Request) {
	var p struct {
		TextDocument struct {
			URI  string `json:"uri"`
			Text string `json:"text"`
		} `json:"textDocument"`
	}
	json.Unmarshal(req.Params, &p)
	s.openDocs[p.TextDocument.URI] = p.TextDocument.Text
	s.publishDiagnostics(p.TextDocument.URI, p.TextDocument.Text)
}

func (s *Server) handleDidChange(req Request) {
	var p struct {
		TextDocument   TextDocumentIdentifier `json:"textDocument"`
		ContentChanges []struct {
			Text string `json:"text"`
		} `json:"contentChanges"`
	}
	json.Unmarshal(req.Params, &p)
	if len(p.ContentChanges) > 0 {
		s.openDocs[p.TextDocument.URI] = p.ContentChanges[len(p.ContentChanges)-1].Text
		s.publishDiagnostics(p.TextDocument.URI, s.openDocs[p.TextDocument.URI])
	}
}

// ── textDocument/completion ───────────────────────────────────────────────────

var templateTags = []CompletionItem{
	{Label: "{% uuid %}", Kind: 15, Detail: "Random UUID", InsertText: "{% uuid %}"},
	{Label: "{% timestamp %}", Kind: 15, Detail: "Unix timestamp (ms)", InsertText: "{% timestamp %}"},
	{Label: "{% now 'iso' %}", Kind: 15, Detail: "ISO 8601 timestamp", InsertText: "{% now 'iso' %}"},
	{Label: "{% randomInt min max %}", Kind: 15, Detail: "Random integer", InsertText: "{% randomInt 1 100 %}"},
	{Label: "{% randomEmail %}", Kind: 15, Detail: "Random email address", InsertText: "{% randomEmail %}"},
	{Label: "{% randomName %}", Kind: 15, Detail: "Random full name", InsertText: "{% randomName %}"},
	{Label: "{% base64 encode val %}", Kind: 15, Detail: "Base64 encode a value", InsertText: "{% base64 encode '${1:value}' %}"},
	{Label: "{% file '/path' %}", Kind: 15, Detail: "Read file contents", InsertText: "{% file '${1:/path/to/file}' %}"},
	{Label: "{% prompt 'label' %}", Kind: 15, Detail: "Prompt user at send-time", InsertText: "{% prompt '${1:Enter value}' %}"},
	{Label: "{% shell 'cmd' %}", Kind: 15, Detail: "Run shell command, inject stdout", InsertText: "{% shell '${1:command}' %}"},
	{Label: "{% response 'body' '$.path' %}", Kind: 15, Detail: "Chain response value from another request", InsertText: "{% response 'body' '${1:\\$.field}' %}"},
}

func (s *Server) handleCompletion(req Request) {
	var params TextDocumentPositionParams
	json.Unmarshal(req.Params, &params)

	s.loadEnvVars()
	items := make([]CompletionItem, 0, len(s.envVars)+len(templateTags))

	// Env var completions as {{VAR}}
	for k, v := range s.envVars {
		preview := v
		if len(preview) > 40 {
			preview = preview[:40] + "…"
		}
		items = append(items, CompletionItem{
			Label:      "{{" + k + "}}",
			Kind:       6,
			Detail:     preview,
			InsertText: "{{" + k + "}}",
		})
	}

	items = append(items, templateTags...)
	s.reply(req.ID, items)
}

// ── textDocument/hover ────────────────────────────────────────────────────────

var varRefRE = regexp.MustCompile(`\{\{([^}$][^}]*)\}\}`)

func (s *Server) handleHover(req Request) {
	var params TextDocumentPositionParams
	json.Unmarshal(req.Params, &params)

	content, ok := s.openDocs[params.TextDocument.URI]
	if !ok {
		s.reply(req.ID, nil)
		return
	}

	lines := strings.Split(content, "\n")
	if params.Position.Line >= len(lines) {
		s.reply(req.ID, nil)
		return
	}
	line := lines[params.Position.Line]
	col := params.Position.Character

	for _, m := range varRefRE.FindAllStringSubmatchIndex(line, -1) {
		start, end := m[0], m[1]
		if col >= start && col <= end {
			varName := strings.TrimSpace(line[m[2]:m[3]])
			s.loadEnvVars()
			if val, ok := s.envVars[varName]; ok {
				masked := val
				if isSecret(varName) {
					masked = strings.Repeat("•", len(val))
				}
				s.reply(req.ID, Hover{
					Contents: fmt.Sprintf("**%s** = `%s`", varName, masked),
				})
				return
			}
			s.reply(req.ID, Hover{Contents: fmt.Sprintf("**%s** — not found in active environment", varName)})
			return
		}
	}
	s.reply(req.ID, nil)
}

// ── workspace/executeCommand ─────────────────────────────────────────────────

func (s *Server) handleExecuteCommand(req Request) {
	var params ExecuteCommandParams
	json.Unmarshal(req.Params, &params)

	switch params.Command {
	case "parallax.listCollections":
		s.reply(req.ID, s.listCollections())

	case "parallax.sendRequest":
		if len(params.Arguments) < 2 {
			s.replyError(req.ID, -32602, "usage: parallax.sendRequest(collectionPath, requestName)")
			return
		}
		var colPath, reqName string
		json.Unmarshal(params.Arguments[0], &colPath)
		json.Unmarshal(params.Arguments[1], &reqName)

		data, err := os.ReadFile(colPath)
		if err != nil {
			s.replyError(req.ID, -32603, fmt.Sprintf("cannot read collection: %v", err))
			return
		}
		var col models.Collection
		if err := yaml.Unmarshal(data, &col); err != nil {
			s.replyError(req.ID, -32603, fmt.Sprintf("cannot parse collection: %v", err))
			return
		}
		s.loadEnvVars()
		result, err := runner.RunRequest(col, reqName, s.envVars)
		if err != nil {
			s.replyError(req.ID, -32603, err.Error())
			return
		}
		s.reply(req.ID, result)

	default:
		s.replyError(req.ID, -32601, "unknown command: "+params.Command)
	}
}

// ── Diagnostics ───────────────────────────────────────────────────────────────

func (s *Server) publishDiagnostics(uri, content string) {
	s.loadEnvVars()
	diags := []Diagnostic{}

	lines := strings.Split(content, "\n")
	for lineIdx, line := range lines {
		for _, m := range varRefRE.FindAllStringSubmatchIndex(line, -1) {
			varName := strings.TrimSpace(line[m[2]:m[3]])
			if _, ok := s.envVars[varName]; !ok {
				diags = append(diags, Diagnostic{
					Range: Range{
						Start: Position{Line: lineIdx, Character: m[0]},
						End:   Position{Line: lineIdx, Character: m[1]},
					},
					Severity: 2, // Warning
					Message:  fmt.Sprintf("'%s' is not defined in the active environment", varName),
					Source:   "parallax",
				})
			}
		}
	}

	s.notify("textDocument/publishDiagnostics", PublishDiagnosticsParams{
		URI:         uri,
		Diagnostics: diags,
	})
}

// ── Helpers ───────────────────────────────────────────────────────────────────

func (s *Server) loadEnvVars() {
	if s.rootPath == "" {
		return
	}
	envDir := filepath.Join(s.rootPath, ".parallax", "environments")
	entries, err := os.ReadDir(envDir)
	if err != nil {
		return
	}
	merged := make(map[string]string)
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".json") {
			continue
		}
		data, err := os.ReadFile(filepath.Join(envDir, e.Name()))
		if err != nil {
			continue
		}
		var env struct {
			Variables map[string]string `json:"variables"`
		}
		if json.Unmarshal(data, &env) == nil {
			for k, v := range env.Variables {
				merged[k] = v
			}
		}
	}
	s.envVars = merged
}

type CollectionInfo struct {
	Name     string   `json:"name"`
	Path     string   `json:"path"`
	Requests []string `json:"requests"`
}

func (s *Server) listCollections() []CollectionInfo {
	if s.rootPath == "" {
		return nil
	}
	colDir := filepath.Join(s.rootPath, ".parallax", "collections")
	entries, err := os.ReadDir(colDir)
	if err != nil {
		return nil
	}
	var result []CollectionInfo
	for _, e := range entries {
		if e.IsDir() {
			continue
		}
		path := filepath.Join(colDir, e.Name())
		data, err := os.ReadFile(path)
		if err != nil {
			continue
		}
		var col models.Collection
		if err := yaml.Unmarshal(data, &col); err != nil {
			continue
		}
		var reqNames []string
		for _, r := range col.Requests {
			reqNames = append(reqNames, r.Name)
		}
		for _, f := range col.Folders {
			for _, r := range f.Requests {
				reqNames = append(reqNames, f.Name+"/"+r.Name)
			}
		}
		result = append(result, CollectionInfo{
			Name:     col.Name,
			Path:     path,
			Requests: reqNames,
		})
	}
	return result
}

func uriToPath(uri string) string {
	path := strings.TrimPrefix(uri, "file://")
	return path
}

func isSecret(name string) bool {
	lower := strings.ToLower(name)
	return strings.Contains(lower, "token") ||
		strings.Contains(lower, "key") ||
		strings.Contains(lower, "secret") ||
		strings.Contains(lower, "password") ||
		strings.Contains(lower, "pass")
}
