//go:build js,wasm

// Package wasm is the WebAssembly entry point for the Parallax HTTP engine.
// It exposes two JS globals:
//   - parallaxSendRequest(requestJSON, envJSON) → Promise<string> (JSON response)
//   - parallaxConvertToCode(requestJSON, lang)  → Promise<string> (code snippet)
//
// Build:
//
//	GOOS=js GOARCH=wasm go build -o parallax.wasm ./wasm
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"syscall/js"
	"time"
)

// ── Minimal request/response models (no sqlite/gRPC dependencies) ──────────────

type wasmRequest struct {
	ID              string            `json:"id"`
	Name            string            `json:"name"`
	Method          string            `json:"method"`
	URL             string            `json:"url"`
	Headers         map[string]string `json:"headers"`
	Params          map[string]string `json:"params"`
	Body            *wasmBody         `json:"body"`
	TimeoutMs       int               `json:"timeout_ms"`
	FollowRedirects bool              `json:"follow_redirects"`
}

type wasmBody struct {
	Type    string `json:"type"`
	Raw     string `json:"raw"`
	Content any    `json:"content"`
}

type wasmResponse struct {
	Status     int               `json:"status"`
	StatusText string            `json:"status_text"`
	Headers    map[string]string `json:"headers"`
	Body       wasmRespBody      `json:"body"`
	Timing     wasmTiming        `json:"timing"`
	SizeBytes  int               `json:"size_bytes"`
}

type wasmRespBody struct {
	Raw         string `json:"raw"`
	JSON        any    `json:"json"`
	ContentType string `json:"content_type"`
}

type wasmTiming struct {
	TotalMs int64 `json:"total_ms"`
}

// ── Template variable resolution: {{VAR}} → env lookup ───────────────────────

func resolveVars(s string, env map[string]string) string {
	for k, v := range env {
		s = strings.ReplaceAll(s, "{{"+k+"}}", v)
	}
	return s
}

func resolveHeaders(h map[string]string, env map[string]string) map[string]string {
	out := make(map[string]string, len(h))
	for k, v := range h {
		out[resolveVars(k, env)] = resolveVars(v, env)
	}
	return out
}

// ── Core HTTP executor ────────────────────────────────────────────────────────

func executeRequest(req wasmRequest, env map[string]string) (*wasmResponse, error) {
	timeout := time.Duration(req.TimeoutMs) * time.Millisecond
	if timeout <= 0 {
		timeout = 30 * time.Second
	}
	client := &http.Client{Timeout: timeout}
	if !req.FollowRedirects {
		client.CheckRedirect = func(*http.Request, []*http.Request) error {
			return http.ErrUseLastResponse
		}
	}

	// Resolve URL and query params
	rawURL := resolveVars(req.URL, env)
	if req.Params != nil && len(req.Params) > 0 {
		sep := "?"
		if strings.Contains(rawURL, "?") {
			sep = "&"
		}
		parts := make([]string, 0, len(req.Params))
		for k, v := range req.Params {
			parts = append(parts, resolveVars(k, env)+"="+resolveVars(v, env))
		}
		rawURL += sep + strings.Join(parts, "&")
	}

	// Build body
	var bodyReader io.Reader
	if req.Body != nil && req.Body.Raw != "" {
		bodyReader = bytes.NewBufferString(resolveVars(req.Body.Raw, env))
	}

	httpReq, err := http.NewRequest(strings.ToUpper(req.Method), rawURL, bodyReader)
	if err != nil {
		return nil, fmt.Errorf("build request: %w", err)
	}

	// Apply headers
	for k, v := range resolveHeaders(req.Headers, env) {
		httpReq.Header.Set(k, v)
	}

	// Infer Content-Type if body present but no header set
	if req.Body != nil && httpReq.Header.Get("Content-Type") == "" {
		switch req.Body.Type {
		case "json":
			httpReq.Header.Set("Content-Type", "application/json")
		case "form":
			httpReq.Header.Set("Content-Type", "multipart/form-data")
		case "urlencoded":
			httpReq.Header.Set("Content-Type", "application/x-www-form-urlencoded")
		}
	}

	t0 := time.Now()
	resp, err := client.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("http: %w", err)
	}
	defer resp.Body.Close()
	elapsed := time.Since(t0).Milliseconds()

	rawBody, _ := io.ReadAll(resp.Body)
	bodyStr := string(rawBody)

	respHeaders := make(map[string]string, len(resp.Header))
	for k := range resp.Header {
		respHeaders[k] = resp.Header.Get(k)
	}

	ct := resp.Header.Get("Content-Type")
	var jsonBody any
	if strings.Contains(ct, "application/json") {
		_ = json.Unmarshal(rawBody, &jsonBody)
	}

	return &wasmResponse{
		Status:     resp.StatusCode,
		StatusText: resp.Status,
		Headers:    respHeaders,
		Body: wasmRespBody{
			Raw:         bodyStr,
			JSON:        jsonBody,
			ContentType: ct,
		},
		Timing:    wasmTiming{TotalMs: elapsed},
		SizeBytes: len(rawBody),
	}, nil
}

// ── Code-snippet generators ───────────────────────────────────────────────────

func toCodeSnippet(req wasmRequest, env map[string]string, lang string) string {
	url := resolveVars(req.URL, env)
	method := strings.ToUpper(req.Method)
	headers := resolveHeaders(req.Headers, env)
	body := ""
	if req.Body != nil {
		body = resolveVars(req.Body.Raw, env)
	}

	switch lang {
	case "curl":
		var sb strings.Builder
		sb.WriteString(fmt.Sprintf("curl -X %s '%s'", method, url))
		for k, v := range headers {
			sb.WriteString(fmt.Sprintf(" \\\n  -H '%s: %s'", k, v))
		}
		if body != "" {
			sb.WriteString(fmt.Sprintf(" \\\n  -d '%s'", body))
		}
		return sb.String()

	case "python":
		var sb strings.Builder
		sb.WriteString("import requests\n\n")
		if body != "" {
			sb.WriteString(fmt.Sprintf("payload = %s\n\n", body))
		}
		sb.WriteString(fmt.Sprintf("response = requests.%s(\n    '%s',\n", strings.ToLower(method), url))
		if len(headers) > 0 {
			sb.WriteString("    headers={\n")
			for k, v := range headers {
				sb.WriteString(fmt.Sprintf("        '%s': '%s',\n", k, v))
			}
			sb.WriteString("    },\n")
		}
		if body != "" {
			sb.WriteString("    data=payload,\n")
		}
		sb.WriteString(")\nprint(response.json())\n")
		return sb.String()

	case "javascript", "js":
		var sb strings.Builder
		sb.WriteString(fmt.Sprintf("const response = await fetch('%s', {\n", url))
		sb.WriteString(fmt.Sprintf("  method: '%s',\n", method))
		if len(headers) > 0 {
			sb.WriteString("  headers: {\n")
			for k, v := range headers {
				sb.WriteString(fmt.Sprintf("    '%s': '%s',\n", k, v))
			}
			sb.WriteString("  },\n")
		}
		if body != "" {
			sb.WriteString(fmt.Sprintf("  body: JSON.stringify(%s),\n", body))
		}
		sb.WriteString("});\nconst data = await response.json();\nconsole.log(data);\n")
		return sb.String()

	default:
		return fmt.Sprintf("// Language '%s' not supported", lang)
	}
}

// ── JS-facing promise helpers ─────────────────────────────────────────────────

func promiseOf(fn func() (any, error)) js.Value {
	handler := js.FuncOf(func(_ js.Value, args []js.Value) any {
		resolve := args[0]
		reject := args[1]
		go func() {
			result, err := fn()
			if err != nil {
				reject.Invoke(js.ValueOf(err.Error()))
				return
			}
			resolve.Invoke(js.ValueOf(result))
		}()
		return nil
	})
	promise := js.Global().Get("Promise").New(handler)
	return promise
}

// ── Exported JS functions ─────────────────────────────────────────────────────

func jsSendRequest(_ js.Value, args []js.Value) any {
	if len(args) < 2 {
		return js.ValueOf("error: need (requestJSON, envJSON)")
	}
	reqJSON := args[0].String()
	envJSON := args[1].String()

	return promiseOf(func() (any, error) {
		var req wasmRequest
		if err := json.Unmarshal([]byte(reqJSON), &req); err != nil {
			return nil, fmt.Errorf("parse request: %w", err)
		}
		var env map[string]string
		if err := json.Unmarshal([]byte(envJSON), &env); err != nil {
			env = map[string]string{}
		}
		resp, err := executeRequest(req, env)
		if err != nil {
			return nil, err
		}
		out, err := json.Marshal(resp)
		if err != nil {
			return nil, err
		}
		return string(out), nil
	})
}

func jsConvertToCode(_ js.Value, args []js.Value) any {
	if len(args) < 2 {
		return js.ValueOf("error: need (requestJSON, lang)")
	}
	reqJSON := args[0].String()
	lang := args[1].String()

	return promiseOf(func() (any, error) {
		var req wasmRequest
		if err := json.Unmarshal([]byte(reqJSON), &req); err != nil {
			return nil, fmt.Errorf("parse request: %w", err)
		}
		return toCodeSnippet(req, map[string]string{}, lang), nil
	})
}

func main() {
	js.Global().Set("parallaxSendRequest", js.FuncOf(jsSendRequest))
	js.Global().Set("parallaxConvertToCode", js.FuncOf(jsConvertToCode))

	// Keep the Go runtime alive until the page unloads
	<-make(chan struct{})
}
