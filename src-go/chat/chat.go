// Package chat implements a local HTTP+SSE chat hub with P2P forwarding.
// Listens on 0.0.0.0:50152. Messages persisted to .parallax/chat/messages.jsonl.
// Forwards to peer sidecars via HTTP; failures queued in-memory and retried every 15s.
package chat

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/google/uuid"
)

// ── Types ─────────────────────────────────────────────────────

type Message struct {
	ID          string `json:"id"`
	WorkspaceID string `json:"workspace_id"`
	Sender      string `json:"sender"`
	SenderName  string `json:"sender_name"`
	Body        string `json:"body"`
	TS          int64  `json:"ts"`
	Forwarded   bool   `json:"forwarded,omitempty"`
}

type Presence struct {
	Login string `json:"login"`
	Name  string `json:"name"`
	IP    string `json:"ip"`
	Port  int    `json:"port"`
	TS    int64  `json:"ts"`
}

type outboxItem struct {
	Msg     Message `json:"msg"`
	PeerURL string  `json:"peer_url"`
	Retries int     `json:"retries"`
}

// ── Hub ───────────────────────────────────────────────────────

type Hub struct {
	mu       sync.RWMutex
	subs     map[string][]chan Message         // ws → SSE subscribers
	presence map[string]map[string]Presence   // ws → login → Presence

	outboxMu sync.Mutex
	outbox   []outboxItem

	lanIP string
	port  int
}

func NewHub(port int) *Hub {
	h := &Hub{
		subs:     make(map[string][]chan Message),
		presence: make(map[string]map[string]Presence),
		lanIP:    detectLANIP(),
		port:     port,
	}
	go h.evictPresence()
	go h.drainOutbox()
	return h
}

func detectLANIP() string {
	addrs, err := net.InterfaceAddrs()
	if err != nil {
		return "127.0.0.1"
	}
	for _, addr := range addrs {
		if ipnet, ok := addr.(*net.IPNet); ok && !ipnet.IP.IsLoopback() {
			if ipnet.IP.To4() != nil {
				return ipnet.IP.String()
			}
		}
	}
	return "127.0.0.1"
}

// evictPresence removes stale entries (>5 min old) every minute.
func (h *Hub) evictPresence() {
	for range time.Tick(time.Minute) {
		cutoff := time.Now().Add(-5 * time.Minute).Unix()
		h.mu.Lock()
		for _, peers := range h.presence {
			for login, p := range peers {
				if p.TS < cutoff {
					delete(peers, login)
				}
			}
		}
		h.mu.Unlock()
	}
}

// drainOutbox retries failed peer forwards every 15 seconds.
func (h *Hub) drainOutbox() {
	for range time.Tick(15 * time.Second) {
		h.outboxMu.Lock()
		remaining := h.outbox[:0]
		for _, item := range h.outbox {
			if err := h.forwardHTTP(item.Msg, item.PeerURL); err != nil {
				if item.Retries < 20 {
					remaining = append(remaining, outboxItem{item.Msg, item.PeerURL, item.Retries + 1})
				}
			}
		}
		h.outbox = remaining
		h.outboxMu.Unlock()
	}
}

// broadcast pushes a message to all local SSE subscribers of a workspace.
func (h *Hub) broadcast(msg Message) {
	h.mu.RLock()
	channels := h.subs[msg.WorkspaceID]
	h.mu.RUnlock()
	for _, ch := range channels {
		select {
		case ch <- msg:
		default:
		}
	}
}

func (h *Hub) subscribe(ws string) chan Message {
	ch := make(chan Message, 64)
	h.mu.Lock()
	h.subs[ws] = append(h.subs[ws], ch)
	h.mu.Unlock()
	return ch
}

func (h *Hub) unsubscribe(ws string, ch chan Message) {
	h.mu.Lock()
	defer h.mu.Unlock()
	list := h.subs[ws]
	for i, c := range list {
		if c == ch {
			h.subs[ws] = append(list[:i], list[i+1:]...)
			break
		}
	}
	close(ch)
}

// forwardToPeers sends the message to every known peer in the workspace.
func (h *Hub) forwardToPeers(msg Message) {
	h.mu.RLock()
	peers := h.presence[msg.WorkspaceID]
	h.mu.RUnlock()

	myAddr := fmt.Sprintf("%s:%d", h.lanIP, h.port)
	for _, peer := range peers {
		if peer.IP == "" || peer.Port == 0 {
			continue
		}
		peerAddr := fmt.Sprintf("%s:%d", peer.IP, peer.Port)
		if peerAddr == myAddr {
			continue
		}
		url := fmt.Sprintf("http://%s/chat/message", peerAddr)
		if err := h.forwardHTTP(msg, url); err != nil {
			h.outboxMu.Lock()
			h.outbox = append(h.outbox, outboxItem{msg, url, 0})
			h.outboxMu.Unlock()
		}
	}
}

func (h *Hub) forwardHTTP(msg Message, url string) error {
	fwd := msg
	fwd.Forwarded = true
	data, _ := json.Marshal(fwd)
	client := &http.Client{Timeout: 3 * time.Second}
	resp, err := client.Post(url, "application/json", bytes.NewReader(data))
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode >= 400 {
		return fmt.Errorf("peer returned %d", resp.StatusCode)
	}
	return nil
}

// ── HTTP handlers ─────────────────────────────────────────────

func (h *Hub) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.Header().Set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
	w.Header().Set("Access-Control-Allow-Headers", "Content-Type")
	if r.Method == http.MethodOptions {
		w.WriteHeader(http.StatusNoContent)
		return
	}

	switch {
	case r.URL.Path == "/chat/stream" && r.Method == http.MethodGet:
		h.handleStream(w, r)
	case r.URL.Path == "/chat/message" && r.Method == http.MethodPost:
		h.handlePostMessage(w, r)
	case r.URL.Path == "/chat/history" && r.Method == http.MethodGet:
		h.handleHistory(w, r)
	case r.URL.Path == "/chat/presence" && r.Method == http.MethodGet:
		h.handleGetPresence(w, r)
	case r.URL.Path == "/chat/presence" && r.Method == http.MethodPost:
		h.handleSetPresence(w, r)
	case r.URL.Path == "/chat/info" && r.Method == http.MethodGet:
		h.handleInfo(w, r)
	default:
		http.NotFound(w, r)
	}
}

func (h *Hub) handleInfo(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"ip":   h.lanIP,
		"port": h.port,
	})
}

func (h *Hub) handleStream(w http.ResponseWriter, r *http.Request) {
	ws := r.URL.Query().Get("workspace")
	if ws == "" {
		http.Error(w, "workspace required", http.StatusBadRequest)
		return
	}

	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "streaming unsupported", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	w.Header().Set("X-Accel-Buffering", "no")

	ch := h.subscribe(ws)
	defer h.unsubscribe(ws, ch)

	ticker := time.NewTicker(15 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case msg, ok := <-ch:
			if !ok {
				return
			}
			data, _ := json.Marshal(msg)
			fmt.Fprintf(w, "data: %s\n\n", data)
			flusher.Flush()
		case <-ticker.C:
			fmt.Fprintf(w, ": keepalive\n\n")
			flusher.Flush()
		case <-r.Context().Done():
			return
		}
	}
}

func (h *Hub) handlePostMessage(w http.ResponseWriter, r *http.Request) {
	var body struct {
		WorkspaceID string `json:"workspace_id"`
		Sender      string `json:"sender"`
		SenderName  string `json:"sender_name"`
		Body        string `json:"body"`
		Forwarded   bool   `json:"forwarded"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	msg := Message{
		ID:          uuid.New().String(),
		WorkspaceID: body.WorkspaceID,
		Sender:      body.Sender,
		SenderName:  body.SenderName,
		Body:        body.Body,
		TS:          time.Now().Unix(),
	}

	if err := persistMessage(msg); err != nil {
		fmt.Fprintf(os.Stderr, "[chat] persist error: %v\n", err)
	}

	h.broadcast(msg)

	if !body.Forwarded {
		go h.forwardToPeers(msg)
	}

	w.WriteHeader(http.StatusCreated)
}

func (h *Hub) handleHistory(w http.ResponseWriter, r *http.Request) {
	ws := r.URL.Query().Get("workspace")
	messages := loadHistory(ws)
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(messages)
}

func (h *Hub) handleGetPresence(w http.ResponseWriter, r *http.Request) {
	ws := r.URL.Query().Get("workspace")
	cutoff := time.Now().Add(-5 * time.Minute).Unix()

	h.mu.RLock()
	peers := h.presence[ws]
	h.mu.RUnlock()

	var result []Presence
	for _, p := range peers {
		if p.TS >= cutoff {
			result = append(result, p)
		}
	}
	if result == nil {
		result = []Presence{}
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(result)
}

func (h *Hub) handleSetPresence(w http.ResponseWriter, r *http.Request) {
	var body struct {
		WorkspaceID string `json:"workspace_id"`
		Login       string `json:"login"`
		Name        string `json:"name"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	p := Presence{
		Login: body.Login,
		Name:  body.Name,
		IP:    h.lanIP, // server fills in its detected LAN IP automatically
		Port:  h.port,
		TS:    time.Now().Unix(),
	}

	h.mu.Lock()
	if h.presence[body.WorkspaceID] == nil {
		h.presence[body.WorkspaceID] = make(map[string]Presence)
	}
	h.presence[body.WorkspaceID][body.Login] = p
	h.mu.Unlock()

	w.WriteHeader(http.StatusNoContent)
}

// ── Persistence ───────────────────────────────────────────────

func messagesPath(workspaceID string) string {
	return filepath.Join(workspaceID, ".parallax", "chat", "messages.jsonl")
}

func persistMessage(msg Message) error {
	path := messagesPath(msg.WorkspaceID)
	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		return err
	}
	f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return err
	}
	defer f.Close()
	return json.NewEncoder(f).Encode(msg)
}

func loadHistory(workspaceID string) []Message {
	path := messagesPath(workspaceID)
	f, err := os.Open(path)
	if err != nil {
		return []Message{}
	}
	defer f.Close()

	var messages []Message
	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		var msg Message
		if err := json.Unmarshal(scanner.Bytes(), &msg); err == nil {
			messages = append(messages, msg)
		}
	}
	if messages == nil {
		return []Message{}
	}
	return messages
}

// ── Entry point ───────────────────────────────────────────────

// Start launches the chat HTTP+SSE server on all interfaces at the given port.
func Start(port int) {
	hub := NewHub(port)
	addr := fmt.Sprintf("0.0.0.0:%d", port)
	fmt.Printf("[chat] P2P hub on %s (LAN IP: %s)\n", addr, hub.lanIP)
	server := &http.Server{Addr: addr, Handler: hub}
	if err := server.ListenAndServe(); err != nil {
		fmt.Fprintf(os.Stderr, "[chat] server error: %v\n", err)
	}
}
