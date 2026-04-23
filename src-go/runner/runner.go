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

// StreamEvent is emitted per-request and per-test during a collection run.
type StreamEvent struct {
	Type       string // request_start | request_end | test_result | summary
	Name       string
	StatusCode int
	DurationMs int64
	Passed     bool
	Error      string
}

const (
	EventRequestStart = "request_start"
	EventRequestEnd   = "request_end"
	EventTestResult   = "test_result"
	EventSummary      = "summary"
)

// RunCollection is the synchronous CLI version — no streaming.
func RunCollection(col models.Collection, env map[string]string) (*RunStats, error) {
	return RunCollectionStream(col, env, nil)
}

// RunCollectionStream runs a collection and calls emit() for every event.
// Pass nil for emit to suppress streaming (CLI text mode).
func RunCollectionStream(col models.Collection, env map[string]string, emit func(StreamEvent)) (*RunStats, error) {
	stats := &RunStats{}
	start := time.Now()

	runGroup := func(reqs []models.CollectionRequest) {
		for _, req := range reqs {
			if emit != nil {
				emit(StreamEvent{Type: EventRequestStart, Name: req.Name})
			}
			if err := runRequest(req, env, stats, emit); err != nil {
				fmt.Printf("Error running %s: %v\n", req.Name, err)
			}
		}
	}

	runGroup(col.Requests)
	for _, folder := range col.Folders {
		runGroup(folder.Requests)
	}

	stats.Total = stats.Passed + stats.Failed
	stats.DurationMs = time.Since(start).Milliseconds()

	if emit != nil {
		emit(StreamEvent{
			Type:       EventSummary,
			Name:       fmt.Sprintf("passed:%d failed:%d", stats.Passed, stats.Failed),
			StatusCode: stats.Failed, // 0 = all passed
			DurationMs: stats.DurationMs,
			Passed:     stats.Failed == 0,
		})
	}

	return stats, nil
}

func runRequest(req models.CollectionRequest, env map[string]string, stats *RunStats, emit func(StreamEvent)) error {
	client := &http.Client{Timeout: 30 * time.Second}

	resolve := func(s string) string {
		re := regexp.MustCompile(`\{\{([^}]+)\}\}`)
		return re.ReplaceAllStringFunc(s, func(match string) string {
			key := strings.TrimSpace(match[2 : len(match)-2])
			if val, ok := env[key]; ok {
				return val
			}
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

	vm := goja.New()
	vm.Set("pm", map[string]interface{}{
		"environment": map[string]interface{}{
			"get": func(k string) string { return env[k] },
			"set": func(k, v string) { env[k] = v },
			"has": func(k string) bool { _, ok := env[k]; return ok },
		},
		"globals": map[string]interface{}{
			"get": func(k string) string { return env["global."+k] },
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
		if emit != nil {
			emit(StreamEvent{Type: EventRequestEnd, Name: req.Name, Passed: false, Error: err.Error(), DurationMs: duration.Milliseconds()})
		}
		return err
	}
	defer resp.Body.Close()
	respBodyBytes, _ := io.ReadAll(resp.Body)
	respBodyStr := string(respBodyBytes)

	var testResults []string
	var testErrors []string

	if req.Scripts != nil && req.Scripts.Tests != "" {
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
						if ok1 && ok2 && !strings.Contains(s, e) {
							panic(fmt.Sprintf("expected string to include %s", e))
						}
					},
				},
			}
		}

		pmObj := vm.Get("pm").Export().(map[string]interface{})
		pmObj["response"] = map[string]interface{}{
			"code":         resp.StatusCode,
			"json":         func() interface{} { var d interface{}; json.Unmarshal(respBodyBytes, &d); return d },
			"text":         func() string { return respBodyStr },
			"responseTime": duration.Milliseconds(),
		}
		pmObj["test"] = func(name string, fn func()) {
			var testErr string
			func() {
				defer func() {
					if r := recover(); r != nil {
						testErr = fmt.Sprintf("%v", r)
					}
				}()
				fn()
			}()
			passed := testErr == ""
			if passed {
				testResults = append(testResults, name)
			} else {
				testErrors = append(testErrors, fmt.Sprintf("%s: %v", name, testErr))
			}
			if emit != nil {
				emit(StreamEvent{Type: EventTestResult, Name: name, Passed: passed, Error: testErr})
			}
		}
		pmObj["expect"] = expectFn
		vm.Set("pm", pmObj)

		if _, err := vm.RunString(req.Scripts.Tests); err != nil {
			errMsg := fmt.Sprintf("Script Error: %v", err)
			testErrors = append(testErrors, errMsg)
			if emit != nil {
				emit(StreamEvent{Type: EventTestResult, Name: "script", Passed: false, Error: errMsg})
			}
		}
	}

	overallPassed := len(testErrors) == 0 && resp.StatusCode < 400

	if overallPassed {
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

	if emit != nil {
		errStr := ""
		if len(testErrors) > 0 {
			errStr = strings.Join(testErrors, "; ")
		}
		emit(StreamEvent{
			Type:       EventRequestEnd,
			Name:       req.Name,
			StatusCode: resp.StatusCode,
			DurationMs: duration.Milliseconds(),
			Passed:     overallPassed,
			Error:      errStr,
		})
	}

	return nil
}
