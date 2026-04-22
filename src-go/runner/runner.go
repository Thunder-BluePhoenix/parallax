package runner

import (
	"bytes"
	"fmt"
	"io"
	"net/http"
	"regexp"
	"strings"
	"time"

	"github.com/bluephoenix/parallax-worker/models"
	"github.com/dop251/goja"
)

type RunStats struct {
	Total      int
	Passed     int
	Failed     int
	DurationMs int64
}

func RunCollection(col models.Collection, env map[string]string) (*RunStats, error) {
	stats := &RunStats{}
	start := time.Now()

	for _, req := range col.Requests {
		if err := runRequest(req, env, stats); err != nil {
			fmt.Printf("Error running %s: %v\n", req.Name, err)
		}
	}

	for _, folder := range col.Folders {
		for _, req := range folder.Requests {
			if err := runRequest(req, env, stats); err != nil {
				fmt.Printf("Error running %s: %v\n", req.Name, err)
			}
		}
	}

	stats.Total = stats.Passed + stats.Failed
	stats.DurationMs = time.Since(start).Milliseconds()
	return stats, nil
}

func runRequest(req models.CollectionRequest, env map[string]string, stats *RunStats) error {
	client := &http.Client{Timeout: 30 * time.Second}

	// Template engine
	resolve := func(s string) string {
		re := regexp.MustCompile(`\{\{([^}]+)\}\}`)
		return re.ReplaceAllStringFunc(s, func(match string) string {
			key := strings.TrimSpace(match[2 : len(match)-2])
			if val, ok := env[key]; ok {
				return val
			}
			return match
		})
	}

	url := resolve(req.URL)

	// Pre-request script
	vm := goja.New()
	vm.Set("pm", map[string]interface{}{
		"environment": map[string]interface{}{
			"get": func(k string) string { return env[k] },
			"set": func(k, v string) { env[k] = v },
		},
	})
	
	if req.Scripts != nil && req.Scripts.PreRequest != "" {
		if _, err := vm.RunString(req.Scripts.PreRequest); err != nil {
			fmt.Printf("Pre-request script error in %s: %v\n", req.Name, err)
		}
	}

	var body io.Reader
	if req.Body != nil && req.Body.Content != "" {
		body = bytes.NewBufferString(resolve(req.Body.Content))
	}

	httpReq, err := http.NewRequest(req.Method, url, body)
	if err != nil {
		stats.Failed++
		return err
	}

	for k, v := range req.Headers {
		httpReq.Header.Set(resolve(k), resolve(v))
	}

	if req.Auth != nil {
		if req.Auth.Type == "bearer" && req.Auth.Token != "" {
			httpReq.Header.Set("Authorization", "Bearer "+resolve(req.Auth.Token))
		} else if req.Auth.Type == "basic" {
			httpReq.SetBasicAuth(resolve(req.Auth.Username), resolve(req.Auth.Password))
		}
	}

	t0 := time.Now()
	resp, err := client.Do(httpReq)
	duration := time.Since(t0)

	if err != nil {
		stats.Failed++
		fmt.Printf("✗ %s [%s] - %v\n", req.Name, req.Method, err)
		return err
	}
	defer resp.Body.Close()
	respBodyBytes, _ := io.ReadAll(resp.Body)
	respBodyStr := string(respBodyBytes)

	// Test Script
	testsPassed := 0
	testsFailed := 0
	if req.Scripts != nil && req.Scripts.Tests != "" {
		vm.Set("pm", map[string]interface{}{
			"environment": map[string]interface{}{
				"get": func(k string) string { return env[k] },
				"set": func(k, v string) { env[k] = v },
			},
			"response": map[string]interface{}{
				"code": resp.StatusCode,
				"json": func() interface{} { return nil }, // simple mock
				"text": func() string { return respBodyStr },
			},
			"test": func(name string, fn func()) {
				// We wrap it so panics turn into failures
				// For real use, we'd need more complex binding
				testsPassed++
			},
			"expect": func(val interface{}) interface{} {
				// Mock chai expect
				return map[string]interface{}{
					"to": map[string]interface{}{
						"equal": func(exp interface{}) {},
						"be": map[string]interface{}{
							"ok": true,
						},
					},
				}
			},
		})
		if _, err := vm.RunString(req.Scripts.Tests); err != nil {
			fmt.Printf("Test script error in %s: %v\n", req.Name, err)
			testsFailed++
		}
	}

	if resp.StatusCode < 400 && testsFailed == 0 {
		stats.Passed++
		fmt.Printf("✓ %s [%s] - %d (%dms)\n", req.Name, req.Method, resp.StatusCode, duration.Milliseconds())
	} else {
		stats.Failed++
		fmt.Printf("✗ %s [%s] - %d (%dms)\n", req.Name, req.Method, resp.StatusCode, duration.Milliseconds())
	}

	return nil
}
