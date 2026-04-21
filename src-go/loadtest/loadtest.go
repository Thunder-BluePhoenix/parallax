// Parallax Load Test Engine
// Uses goroutines to fire N concurrent requests and collect latency stats
package loadtest

import (
	"crypto/tls"
	"fmt"
	"net/http"
	"sync"
	"sync/atomic"
	"time"
)

type Config struct {
	URL         string            `json:"url"`
	Method      string            `json:"method"`
	Headers     map[string]string `json:"headers"`
	Body        string            `json:"body"`
	Concurrent  int               `json:"concurrent"`
	TotalReqs   int               `json:"total_requests"`
	DurationSec int               `json:"duration_sec"` // 0 = use TotalReqs
}

type Result struct {
	TotalRequests  int64   `json:"total_requests"`
	Successful     int64   `json:"successful"`
	Failed         int64   `json:"failed"`
	TotalDurationMs int64  `json:"total_duration_ms"`
	AvgLatencyMs   float64 `json:"avg_latency_ms"`
	P50LatencyMs   float64 `json:"p50_latency_ms"`
	P95LatencyMs   float64 `json:"p95_latency_ms"`
	P99LatencyMs   float64 `json:"p99_latency_ms"`
	MinLatencyMs   float64 `json:"min_latency_ms"`
	MaxLatencyMs   float64 `json:"max_latency_ms"`
	ReqsPerSec     float64 `json:"reqs_per_sec"`
	Errors         []string `json:"errors"`
	Histogram      []int64  `json:"histogram"` // latency buckets in ms
}

type ProgressEvent struct {
	Completed int64   `json:"completed"`
	Total     int64   `json:"total"`
	CurrentRPS float64 `json:"current_rps"`
}

type Service struct {
	mu      sync.Mutex
	running bool
	cancel  chan struct{}
}

func New() *Service {
	return &Service{
		cancel: make(chan struct{}),
	}
}

func (s *Service) Run(cfg Config, progressCh chan<- ProgressEvent) (*Result, error) {
	s.mu.Lock()
	if s.running {
		s.mu.Unlock()
		return nil, fmt.Errorf("load test already running")
	}
	s.running = true
	s.cancel = make(chan struct{})
	s.mu.Unlock()

	defer func() {
		s.mu.Lock()
		s.running = false
		s.mu.Unlock()
	}()

	client := &http.Client{
		Timeout: 30 * time.Second,
		Transport: &http.Transport{
			TLSClientConfig:   &tls.Config{InsecureSkipVerify: false},
			MaxIdleConns:      cfg.Concurrent * 2,
			IdleConnTimeout:   90 * time.Second,
		},
	}

	latencies := make([]float64, 0, cfg.TotalReqs)
	var mu sync.Mutex
	var successful, failed atomic.Int64
	errors := make([]string, 0)

	sem := make(chan struct{}, cfg.Concurrent)
	var wg sync.WaitGroup

	startTime := time.Now()
	total := int64(cfg.TotalReqs)
	var completed atomic.Int64

	// Progress reporter goroutine
	go func() {
		ticker := time.NewTicker(500 * time.Millisecond)
		defer ticker.Stop()
		var lastCompleted int64
		lastTime := time.Now()

		for {
			select {
			case <-ticker.C:
				now := time.Now()
				c := completed.Load()
				elapsed := now.Sub(lastTime).Seconds()
				rps := float64(c-lastCompleted) / elapsed
				lastCompleted = c
				lastTime = now

				if progressCh != nil {
					progressCh <- ProgressEvent{
						Completed:  c,
						Total:      total,
						CurrentRPS: rps,
					}
				}
				if c >= total {
					return
				}
			case <-s.cancel:
				return
			}
		}
	}()

	for i := 0; i < cfg.TotalReqs; i++ {
		select {
		case <-s.cancel:
			break
		default:
		}

		sem <- struct{}{}
		wg.Add(1)

		go func() {
			defer func() {
				<-sem
				wg.Done()
				completed.Add(1)
			}()

			req, err := http.NewRequest(cfg.Method, cfg.URL, nil)
			if err != nil {
				failed.Add(1)
				mu.Lock()
				errors = append(errors, err.Error())
				mu.Unlock()
				return
			}

			for k, v := range cfg.Headers {
				req.Header.Set(k, v)
			}

			reqStart := time.Now()
			resp, err := client.Do(req)
			latencyMs := float64(time.Since(reqStart).Milliseconds())

			if err != nil {
				failed.Add(1)
				mu.Lock()
				errors = append(errors, err.Error())
				mu.Unlock()
				return
			}
			defer resp.Body.Close()

			if resp.StatusCode >= 400 {
				failed.Add(1)
			} else {
				successful.Add(1)
			}

			mu.Lock()
			latencies = append(latencies, latencyMs)
			mu.Unlock()
		}()
	}

	wg.Wait()
	totalDuration := time.Since(startTime)

	result := computeStats(latencies, successful.Load(), failed.Load(), totalDuration, errors)
	return result, nil
}

func (s *Service) Stop() {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.running {
		close(s.cancel)
	}
}

func computeStats(latencies []float64, successful, failed int64, duration time.Duration, errors []string) *Result {
	if len(latencies) == 0 {
		return &Result{
			Successful: successful,
			Failed:     failed,
			Errors:     errors,
		}
	}

	// Sort latencies
	sorted := make([]float64, len(latencies))
	copy(sorted, latencies)
	sortFloat64(sorted)

	n := len(sorted)
	var sum float64
	for _, v := range sorted {
		sum += v
	}

	durationMs := duration.Milliseconds()
	reqs := float64(successful + failed)
	rps := reqs / duration.Seconds()

	// Build 20-bucket histogram
	min := sorted[0]
	max := sorted[n-1]
	bucketSize := (max - min) / 20.0
	histogram := make([]int64, 20)
	for _, v := range sorted {
		idx := int((v - min) / bucketSize)
		if idx >= 20 {
			idx = 19
		}
		histogram[idx]++
	}

	return &Result{
		TotalRequests:   successful + failed,
		Successful:      successful,
		Failed:          failed,
		TotalDurationMs: durationMs,
		AvgLatencyMs:    sum / float64(n),
		P50LatencyMs:    percentile(sorted, 50),
		P95LatencyMs:    percentile(sorted, 95),
		P99LatencyMs:    percentile(sorted, 99),
		MinLatencyMs:    min,
		MaxLatencyMs:    max,
		ReqsPerSec:      rps,
		Errors:          errors,
		Histogram:       histogram,
	}
}

func percentile(sorted []float64, p float64) float64 {
	idx := int(float64(len(sorted)) * p / 100.0)
	if idx >= len(sorted) {
		idx = len(sorted) - 1
	}
	return sorted[idx]
}

func sortFloat64(s []float64) {
	// Insertion sort (fine for < 100k elements)
	for i := 1; i < len(s); i++ {
		v := s[i]
		j := i - 1
		for j >= 0 && s[j] > v {
			s[j+1] = s[j]
			j--
		}
		s[j+1] = v
	}
}
