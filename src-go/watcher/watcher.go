// Copyright 2026 Parallax Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Parallax Watcher — Git-native filesystem watcher
// Watches .parallax/ directories and emits change events
package watcher

import (
	"log"
	"path/filepath"
	"sync"

	"github.com/fsnotify/fsnotify"
)

type ChangeEvent struct {
	Path      string `json:"path"`
	Operation string `json:"operation"` // "create", "write", "remove", "rename"
	IsCollection bool `json:"is_collection"`
	IsEnvironment bool `json:"is_environment"`
}

type Watcher struct {
	mu       sync.Mutex
	fsw      *fsnotify.Watcher
	watched  map[string]bool
	onChange func(ChangeEvent)
	quit     chan struct{}
}

func New() *Watcher {
	fsw, err := fsnotify.NewWatcher()
	if err != nil {
		log.Printf("[watcher] Failed to create fsnotify watcher: %v", err)
		return nil
	}

	w := &Watcher{
		fsw:     fsw,
		watched: make(map[string]bool),
		quit:    make(chan struct{}),
	}

	go w.loop()
	return w
}

func (w *Watcher) OnChange(fn func(ChangeEvent)) {
	w.onChange = fn
}

func (w *Watcher) WatchWorkspace(workspacePath string) error {
	w.mu.Lock()
	defer w.mu.Unlock()

	parallaxDir := filepath.Join(workspacePath, ".parallax")
	dirs := []string{
		parallaxDir,
		filepath.Join(parallaxDir, "collections"),
		filepath.Join(parallaxDir, "environments"),
		filepath.Join(parallaxDir, "scripts"),
	}

	for _, dir := range dirs {
		if !w.watched[dir] {
			if err := w.fsw.Add(dir); err != nil {
				log.Printf("[watcher] Could not watch %s: %v", dir, err)
				continue
			}
			w.watched[dir] = true
			log.Printf("[watcher] Watching: %s", dir)
		}
	}

	return nil
}

func (w *Watcher) UnwatchWorkspace(workspacePath string) {
	w.mu.Lock()
	defer w.mu.Unlock()

	parallaxDir := filepath.Join(workspacePath, ".parallax")
	dirs := []string{
		parallaxDir,
		filepath.Join(parallaxDir, "collections"),
		filepath.Join(parallaxDir, "environments"),
	}

	for _, dir := range dirs {
		if w.watched[dir] {
			w.fsw.Remove(dir)
			delete(w.watched, dir)
		}
	}
}

func (w *Watcher) Stop() {
	close(w.quit)
	w.fsw.Close()
}

func (w *Watcher) loop() {
	for {
		select {
		case <-w.quit:
			return
		case event, ok := <-w.fsw.Events:
			if !ok {
				return
			}
			if w.onChange != nil {
				change := ChangeEvent{
					Path:         event.Name,
					Operation:    opToString(event.Op),
					IsCollection: filepath.Ext(event.Name) == ".yaml",
					IsEnvironment: filepath.Ext(event.Name) == ".json",
				}
				w.onChange(change)
			}
		case err, ok := <-w.fsw.Errors:
			if !ok {
				return
			}
			log.Printf("[watcher] Error: %v", err)
		}
	}
}

func opToString(op fsnotify.Op) string {
	switch {
	case op&fsnotify.Create != 0:
		return "create"
	case op&fsnotify.Write != 0:
		return "write"
	case op&fsnotify.Remove != 0:
		return "remove"
	case op&fsnotify.Rename != 0:
		return "rename"
	default:
		return "unknown"
	}
}
