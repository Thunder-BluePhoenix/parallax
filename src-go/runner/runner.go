package runner

import (
	"bytes"
	"encoding/json"
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
			// Check env
			if val, ok := env[key]; ok {
				return val
			}
			// Check if key is nested e.g. "env.var"
			parts := strings.Split(key, ".")
			if len(parts) > 1 {
				prefix := parts[0]
				actualKey := strings.Join(parts[1:], ".")
				if prefix == "env" || prefix == "environment" {
					return env[actualKey]
				}
			}
			return match
		})
	}

	url := resolve(req.URL)

	// Pre-request script
	vm := goja.New()
	
	// Helper for assertions
	vm.Set("pm", map[string]interface{}{
		"environment": map[string]interface{}{
			"get": func(k string) string { return env[k] },
			"set": func(k, v string) { env[k] = v },
			"has": func(k string) bool { _, ok := env[k]; return ok },
		},
		"globals": map[string]interface{}{
			"get": func(k string) string { return env["global."+k] }, // Mock globals for now
			"set": func(k, v string) { env["global."+k] = v },
		},
		"collectionVariables": map[string]interface{}{
			"get": func(k string) string { return env["col."+k] },
			"set": func(k, v string) { env["col."+k] = v },
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
	var testResults []string
	var testErrors []string
	if req.Scripts != nil && req.Scripts.Tests != "" {
		// Define pm.expect helper
		expectFn := func(val interface{}) interface{} {
			return map[string]interface{}{
				"to": map[string]interface{}{
					"be": map[string]interface{}{
						"a": func(t string) {
							actualType := fmt.Sprintf("%T", val)
							if !strings.Contains(strings.ToLower(actualType), strings.ToLower(t)) {
								panic(fmt.Sprintf("expected type %s but got %s", t, actualType))
							}
						},
					},
					"have": map[string]interface{}{
						"status": func(s int) {
							if resp.StatusCode != s {
								panic(fmt.Sprintf("expected status %d but got %d", s, resp.StatusCode))
							}
						},
						"json": func() {
							var js interface{}
							if err := json.Unmarshal(respBodyBytes, &js); err != nil {
								panic("response body is not valid JSON")
							}
						},
					},
					"equal": func(exp interface{}) {
						if fmt.Sprintf("%v", val) != fmt.Sprintf("%v", exp) {
							panic(fmt.Sprintf("expected %v but got %v", exp, val))
						}
					},
					"include": func(exp interface{}) {
						s, ok1 := val.(string)
						e, ok2 := exp.(string)
						if ok1 && ok2 {
							if !strings.Contains(s, e) {
								panic(fmt.Sprintf("expected string to include %s", e))
							}
						}
					},
				},
			}
		}

		pmObj := vm.Get("pm").Export().(map[string]interface{})
		pmObj["response"] = map[string]interface{}{
			"code": resp.StatusCode,
			"json": func() interface{} {
				var data interface{}
				json.Unmarshal(respBodyBytes, &data)
				return data
			},
			"text": func() string { return respBodyStr },
			"responseTime": duration.Milliseconds(),
		}
		pmObj["test"] = func(name string, fn func()) {
			defer func() {
				if r := recover(); r != nil {
					testErrors = append(testErrors, fmt.Sprintf("%s: %v", name, r))
				}
			}()
			fn()
			testResults = append(testResults, name)
		}
		pmObj["expect"] = expectFn
		
		vm.Set("pm", pmObj)

		if _, err := vm.RunString(req.Scripts.Tests); err != nil {
			fmt.Printf("  Script Error: %v\n", err)
			testErrors = append(testErrors, fmt.Sprintf("Script Error: %v", err))
		}
	}

	passed := len(testErrors) == 0

	if resp.StatusCode < 400 && passed {
		stats.Passed++
		fmt.Printf("✓ %s [%s] - %d (%dms)\n", req.Name, req.Method, resp.StatusCode, duration.Milliseconds())
	} else {
		stats.Failed++
		fmt.Printf("✗ %s [%s] - %d (%dms)\n", req.Name, req.Method, resp.StatusCode, duration.Milliseconds())
	}

	for _, t := range testResults {
		fmt.Printf("  ✓ %s\n", t)
	}
	for _, e := range testErrors {
		fmt.Printf("  ✗ %s\n", e)
	}

	return nil
}
