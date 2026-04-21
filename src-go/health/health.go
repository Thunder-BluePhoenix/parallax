// Parallax Health Monitor
// Uses goroutines to background-ping services and emit status events
package health

import (
	"context"
	"net/http"
	"sync"
	"time"
)

type ServiceTarget struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	URL         string `json:"url"`
	IntervalSec int    `json:"interval_sec"`
	Timeout     int    `json:"timeout_ms"`
}

type ServiceStatus struct {
	ID          string  `json:"id"`
	Name        string  `json:"name"`
	URL         string  `json:"url"`
	Status      string  `json:"status"` // "up", "down", "slow", "unknown"
	LatencyMs   float64 `json:"latency_ms"`
	StatusCode  int     `json:"status_code"`
	LastChecked int64   `json:"last_checked"`
	ErrorMsg    string  `json:"error_msg,omitempty"`
}

type Monitor struct {
	mu       sync.RWMutex
	targets  map[string]*ServiceTarget
	statuses map[string]*ServiceStatus
	watchers map[string]context.CancelFunc
	onChange func(ServiceStatus)
}

func New() *Monitor {
	return &Monitor{
		targets:  make(map[string]*ServiceTarget),
		statuses: make(map[string]*ServiceStatus),
		watchers: make(map[string]context.CancelFunc),
	}
}

func (m *Monitor) OnChange(fn func(ServiceStatus)) {
	m.onChange = fn
}

func (m *Monitor) AddTarget(target ServiceTarget) {
	m.mu.Lock()
	defer m.mu.Unlock()

	// Cancel existing watcher if any
	if cancel, ok := m.watchers[target.ID]; ok {
		cancel()
	}

	m.targets[target.ID] = &target

	ctx, cancel := context.WithCancel(context.Background())
	m.watchers[target.ID] = cancel

	go m.watch(ctx, &target)
}

func (m *Monitor) RemoveTarget(id string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if cancel, ok := m.watchers[id]; ok {
		cancel()
		delete(m.watchers, id)
	}
	delete(m.targets, id)
	delete(m.statuses, id)
}

func (m *Monitor) GetStatuses() []ServiceStatus {
	m.mu.RLock()
	defer m.mu.RUnlock()

	statuses := make([]ServiceStatus, 0, len(m.statuses))
	for _, s := range m.statuses {
		statuses = append(statuses, *s)
	}
	return statuses
}

func (m *Monitor) Stop() {
	m.mu.Lock()
	defer m.mu.Unlock()
	for _, cancel := range m.watchers {
		cancel()
	}
}

func (m *Monitor) watch(ctx context.Context, target *ServiceTarget) {
	interval := time.Duration(target.IntervalSec) * time.Second
	if interval == 0 {
		interval = 30 * time.Second
	}

	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	// Initial check
	m.check(target)

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			m.check(target)
		}
	}
}

func (m *Monitor) check(target *ServiceTarget) {
	timeout := time.Duration(target.Timeout) * time.Millisecond
	if timeout == 0 {
		timeout = 5 * time.Second
	}

	client := &http.Client{Timeout: timeout}
	start := time.Now()

	resp, err := client.Get(target.URL)
	latencyMs := float64(time.Since(start).Milliseconds())

	status := &ServiceStatus{
		ID:          target.ID,
		Name:        target.Name,
		URL:         target.URL,
		LatencyMs:   latencyMs,
		LastChecked: time.Now().Unix(),
	}

	if err != nil {
		status.Status = "down"
		status.ErrorMsg = err.Error()
	} else {
		defer resp.Body.Close()
		status.StatusCode = resp.StatusCode

		switch {
		case resp.StatusCode >= 500:
			status.Status = "down"
		case resp.StatusCode >= 400:
			status.Status = "down"
		case latencyMs > 2000:
			status.Status = "slow"
		default:
			status.Status = "up"
		}
	}

	m.mu.Lock()
	m.statuses[target.ID] = status
	m.mu.Unlock()

	if m.onChange != nil {
		m.onChange(*status)
	}
}
