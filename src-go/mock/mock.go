package mock

import (
	"bytes"
	"fmt"
	"log"
	"net/http"
	"strings"
	"sync"
	"text/template"
	"encoding/json"

	"github.com/bluephoenix/parallax-worker/storage"
)

type MockRule struct {
	ID          string            `json:"id"`
	Path        string            `json:"path"`
	Method      string            `json:"method"`
	StatusCode  int               `json:"status_code"`
	Body        string            `json:"body"`
	Headers     map[string]string `json:"headers"`
	ContentType string            `json:"content_type"`
}

type Server struct {
	port   int
	mu     sync.RWMutex
	rules  map[string]MockRule
	server *http.Server
	store  *storage.Storage
}

func New(port int, store *storage.Storage) *Server {
	s := &Server{
		port:  port,
		rules: make(map[string]MockRule),
		store: store,
	}
	s.loadRules()
	return s
}

func (s *Server) loadRules() {
	if s.store == nil {
		return
	}
	rules, err := s.store.GetAllMockRules()
	if err != nil {
		log.Printf("[mock] Failed to load rules: %v", err)
		return
	}
	for _, r := range rules {
		headers := make(map[string]string)
		json.Unmarshal([]byte(r["headers"].(string)), &headers)
		
		s.rules[r["id"].(string)] = MockRule{
			ID:          r["id"].(string),
			Path:        r["path"].(string),
			Method:      r["method"].(string),
			StatusCode:  r["status_code"].(int),
			Body:        r["body"].(string),
			Headers:     headers,
			ContentType: r["content_type"].(string),
		}
	}
	log.Printf("[mock] Loaded %d rules from persistence", len(s.rules))
}

func (s *Server) GetRules() ([]MockRule, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	
	result := make([]MockRule, 0, len(s.rules))
	for _, r := range s.rules {
		result = append(result, r)
	}
	return result, nil
}

func (s *Server) AddRule(rule MockRule) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.rules[rule.ID] = rule
	
	if s.store != nil {
		h, _ := json.Marshal(rule.Headers)
		s.store.SaveMockRule(rule.ID, rule.Path, rule.Method, rule.StatusCode, rule.Body, string(h), rule.ContentType)
	}
}

func (s *Server) RemoveRule(id string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	delete(s.rules, id)
	
	if s.store != nil {
		s.store.DeleteMockRule(id)
	}
}

func (s *Server) Start() error {
	mux := http.NewServeMux()
	mux.HandleFunc("/", s.handle)

	s.server = &http.Server{
		Addr:    fmt.Sprintf("127.0.0.1:%d", s.port),
		Handler: mux,
	}

	log.Printf("[mock] Listening on :%d", s.port)
	return s.server.ListenAndServe()
}

func (s *Server) Stop() {
	if s.server != nil {
		s.server.Close()
	}
}

func (s *Server) handle(w http.ResponseWriter, r *http.Request) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	// Find matching rule
	for _, rule := range s.rules {
		if rule.Method != r.Method {
			continue
		}

		matched, params := matchPath(rule.Path, r.URL.Path)
		if matched {
			for k, v := range rule.Headers {
				w.Header().Set(k, v)
			}
			if rule.ContentType != "" {
				w.Header().Set("Content-Type", rule.ContentType)
			}
			w.WriteHeader(rule.StatusCode)
			
			// Render template
			w.Write([]byte(renderBody(rule.Body, params, r)))
			return
		}
	}

	// Default 404
	http.Error(w, "No mock rule matched this request", http.StatusNotFound)
}

func matchPath(rulePath, reqPath string) (bool, map[string]string) {
	if rulePath == reqPath {
		return true, nil
	}
	if !strings.Contains(rulePath, ":") && !strings.Contains(rulePath, "*") {
		return false, nil
	}

	segmentsRule := strings.Split(strings.Trim(rulePath, "/"), "/")
	segmentsReq := strings.Split(strings.Trim(reqPath, "/"), "/")

	if len(segmentsRule) > len(segmentsReq) {
		return false, nil
	}

	params := make(map[string]string)
	for i, sr := range segmentsRule {
		if strings.HasPrefix(sr, ":") {
			params[strings.TrimPrefix(sr, ":")] = segmentsReq[i]
		} else if sr == "*" {
			params["wildcard"] = strings.Join(segmentsReq[i:], "/")
			return true, params
		} else if sr != segmentsReq[i] {
			return false, nil
		}
	}
	if len(segmentsRule) != len(segmentsReq) {
		return false, nil
	}
	return true, params
}

func renderBody(bodyTemplate string, params map[string]string, r *http.Request) string {
	if !strings.Contains(bodyTemplate, "{{") {
		return bodyTemplate
	}
	
	query := make(map[string]string)
	for k, v := range r.URL.Query() {
		if len(v) > 0 {
			query[k] = v[0]
		}
	}

	data := map[string]interface{}{
		"Params": params,
		"Query":  query,
	}

	tmpl, err := template.New("mock").Parse(bodyTemplate)
	if err != nil {
		return bodyTemplate
	}

	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, data); err != nil {
		return bodyTemplate
	}
	return buf.String()
}
