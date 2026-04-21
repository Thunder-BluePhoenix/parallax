// Parallax Local Proxy
// Intercepts HTTP traffic from local apps and streams to the dashboard
package proxy

import (
	"bufio"
	"crypto/tls"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"sync"
	"sync/atomic"
	"time"
)

type TrafficEntry struct {
	ID          string            `json:"id"`
	Timestamp   int64             `json:"timestamp_ms"`
	Method      string            `json:"method"`
	URL         string            `json:"url"`
	RequestHeaders  map[string]string `json:"request_headers"`
	ResponseHeaders map[string]string `json:"response_headers"`
	StatusCode  int               `json:"status_code"`
	LatencyMs   float64           `json:"latency_ms"`
	RequestBodySize  int          `json:"request_body_bytes"`
	ResponseBodySize int          `json:"response_body_bytes"`
	ContentType string            `json:"content_type"`
	Preview     string            `json:"preview"` // First 512 chars of response body
}

type Proxy struct {
	port     int
	server   *http.Server
	mu       sync.RWMutex
	traffic  []TrafficEntry
	onTraffic func(TrafficEntry)
	counter  atomic.Int64
	active   bool
}

func New(port int) *Proxy {
	return &Proxy{
		port:    port,
		traffic: make([]TrafficEntry, 0, 1000),
	}
}

func (p *Proxy) OnTraffic(fn func(TrafficEntry)) {
	p.onTraffic = fn
}

func (p *Proxy) GetTraffic(limit int) []TrafficEntry {
	p.mu.RLock()
	defer p.mu.RUnlock()
	if limit <= 0 || limit > len(p.traffic) {
		limit = len(p.traffic)
	}
	result := make([]TrafficEntry, limit)
	copy(result, p.traffic[len(p.traffic)-limit:])
	return result
}

func (p *Proxy) ClearTraffic() {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.traffic = p.traffic[:0]
}

func (p *Proxy) Start() error {
	mux := http.NewServeMux()
	mux.HandleFunc("/", p.handleHTTP)

	p.server = &http.Server{
		Addr:    fmt.Sprintf("127.0.0.1:%d", p.port),
		Handler: mux,
	}

	p.active = true
	log.Printf("[proxy] Listening on :%d", p.port)
	return p.server.ListenAndServe()
}

func (p *Proxy) Stop() {
	if p.server != nil {
		p.server.Close()
	}
	p.active = false
}

func (p *Proxy) handleHTTP(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodConnect {
		p.handleTunnel(w, r)
		return
	}

	start := time.Now()
	id := fmt.Sprintf("req-%d", p.counter.Add(1))

	// Capture request headers
	reqHeaders := make(map[string]string)
	for k, vv := range r.Header {
		if len(vv) > 0 {
			reqHeaders[k] = vv[0]
		}
	}

	// Forward the request
	outReq := r.Clone(r.Context())
	outReq.RequestURI = ""

	transport := &http.Transport{
		TLSClientConfig: &tls.Config{InsecureSkipVerify: false},
	}
	client := &http.Client{Transport: transport}

	resp, err := client.Do(outReq)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadGateway)
		return
	}
	defer resp.Body.Close()

	latencyMs := float64(time.Since(start).Milliseconds())

	// Read response body with size limit for preview
	respBody, _ := io.ReadAll(io.LimitReader(resp.Body, 1<<20)) // 1MB max

	preview := ""
	if len(respBody) > 0 {
		end := 512
		if end > len(respBody) {
			end = len(respBody)
		}
		preview = string(respBody[:end])
	}

	// Capture response headers
	respHeaders := make(map[string]string)
	for k, vv := range resp.Header {
		if len(vv) > 0 {
			respHeaders[k] = vv[0]
			w.Header().Set(k, vv[0])
		}
	}

	w.WriteHeader(resp.StatusCode)
	w.Write(respBody)

	entry := TrafficEntry{
		ID:               id,
		Timestamp:        time.Now().UnixMilli(),
		Method:           r.Method,
		URL:              r.URL.String(),
		RequestHeaders:   reqHeaders,
		ResponseHeaders:  respHeaders,
		StatusCode:       resp.StatusCode,
		LatencyMs:        latencyMs,
		RequestBodySize:  int(r.ContentLength),
		ResponseBodySize: len(respBody),
		ContentType:      resp.Header.Get("Content-Type"),
		Preview:          preview,
	}

	p.mu.Lock()
	// Keep last 5000 entries
	if len(p.traffic) >= 5000 {
		p.traffic = p.traffic[1:]
	}
	p.traffic = append(p.traffic, entry)
	p.mu.Unlock()

	if p.onTraffic != nil {
		p.onTraffic(entry)
	}
}

func (p *Proxy) handleTunnel(w http.ResponseWriter, r *http.Request) {
	destConn, err := net.DialTimeout("tcp", r.Host, 10*time.Second)
	if err != nil {
		http.Error(w, err.Error(), http.StatusServiceUnavailable)
		return
	}

	w.WriteHeader(http.StatusOK)
	hijacker, ok := w.(http.Hijacker)
	if !ok {
		http.Error(w, "Hijacking not supported", http.StatusInternalServerError)
		return
	}

	clientConn, _, err := hijacker.Hijack()
	if err != nil {
		http.Error(w, err.Error(), http.StatusServiceUnavailable)
		return
	}

	go transfer(destConn, clientConn)
	go transfer(clientConn, destConn)
}

func transfer(dst io.WriteCloser, src io.ReadCloser) {
	defer dst.Close()
	defer src.Close()
	io.Copy(dst, bufio.NewReader(src))
}
