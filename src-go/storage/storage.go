package storage

import (
	"database/sql"
	"fmt"
	"os"
	"path/filepath"

	_ "modernc.org/sqlite"
)

type Storage struct {
	db *sql.DB
}

func New(dbPath string) (*Storage, error) {
	// Ensure directory exists
	dir := filepath.Dir(dbPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return nil, err
	}

	db, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return nil, err
	}

	s := &Storage{db: db}
	if err := s.initSchema(); err != nil {
		return nil, err
	}

	return s, nil
}

func (s *Storage) initSchema() error {
	queries := []string{
		`CREATE TABLE IF NOT EXISTS health_history (
			id TEXT PRIMARY KEY,
			target_id TEXT,
			name TEXT,
			url TEXT,
			status TEXT,
			latency_ms REAL,
			status_code INTEGER,
			last_checked INTEGER,
			error_msg TEXT
		)`,
		`CREATE TABLE IF NOT EXISTS proxy_traffic (
			id TEXT PRIMARY KEY,
			timestamp_ms INTEGER,
			method TEXT,
			url TEXT,
			status_code INTEGER,
			latency_ms REAL,
			content_type TEXT,
			preview TEXT
		)`,
		`CREATE TABLE IF NOT EXISTS mock_rules (
			id TEXT PRIMARY KEY,
			path TEXT,
			method TEXT,
			status_code INTEGER,
			body TEXT,
			headers TEXT,
			content_type TEXT
		)`,
	}

	for _, q := range queries {
		if _, err := s.db.Exec(q); err != nil {
			return fmt.Errorf("schema init error: %v", err)
		}
	}

	return nil
}

func (s *Storage) Close() {
	if s.db != nil {
		s.db.Close()
	}
}

// Health History
func (s *Storage) SaveHealthStatus(id, targetID, name, url, status string, latency float64, code int, checked int64, err string) error {
	_, execErr := s.db.Exec(
		"INSERT OR REPLACE INTO health_history (id, target_id, name, url, status, latency_ms, status_code, last_checked, error_msg) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
		id, targetID, name, url, status, latency, code, checked, err,
	)
	return execErr
}

// Proxy Traffic
func (s *Storage) SaveTraffic(id string, ts int64, method, url string, code int, latency float64, ctype, preview string) error {
	_, execErr := s.db.Exec(
		"INSERT INTO proxy_traffic (id, timestamp_ms, method, url, status_code, latency_ms, content_type, preview) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
		id, ts, method, url, code, latency, ctype, preview,
	)
	return execErr
}

func (s *Storage) GetRecentTraffic(limit int) ([]map[string]interface{}, error) {
	rows, err := s.db.Query("SELECT * FROM proxy_traffic ORDER BY timestamp_ms DESC LIMIT ?", limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var result []map[string]interface{}
	for rows.Next() {
		var id, method, url, ctype, preview string
		var ts int64
		var code int
		var latency float64
		if err := rows.Scan(&id, &ts, &method, &url, &code, &latency, &ctype, &preview); err != nil {
			return nil, err
		}
		result = append(result, map[string]interface{}{
			"id": id, "timestamp_ms": ts, "method": method, "url": url,
			"status_code": code, "latency_ms": latency, "content_type": ctype, "preview": preview,
		})
	}
	return result, nil
}

// Mock Rules
func (s *Storage) SaveMockRule(id, path, method string, code int, body, headers, ctype string) error {
	_, execErr := s.db.Exec(
		"INSERT OR REPLACE INTO mock_rules (id, path, method, status_code, body, headers, content_type) VALUES (?, ?, ?, ?, ?, ?, ?)",
		id, path, method, code, body, headers, ctype,
	)
	return execErr
}

func (s *Storage) DeleteMockRule(id string) error {
	_, execErr := s.db.Exec("DELETE FROM mock_rules WHERE id = ?", id)
	return execErr
}

func (s *Storage) GetAllMockRules() ([]map[string]interface{}, error) {
	rows, err := s.db.Query("SELECT * FROM mock_rules")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var result []map[string]interface{}
	for rows.Next() {
		var id, path, method, body, headers, ctype string
		var code int
		if err := rows.Scan(&id, &path, &method, &code, &body, &headers, &ctype); err != nil {
			return nil, err
		}
		result = append(result, map[string]interface{}{
			"id": id, "path": path, "method": method, "status_code": code,
			"body": body, "headers": headers, "content_type": ctype,
		})
	}
	return result, nil
}
