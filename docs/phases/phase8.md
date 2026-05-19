# Phase 8 — Web / WASM

**Status:** 🔲 Planned
**Depends on:** Phase 7 complete
**Inspired by:** `l2.wasm.go` compiles Go HTTP client to WebAssembly for browser use
**Goal:** Make Parallax accessible from a browser with no install. A hosted web app backed by a WASM HTTP engine and a lightweight cloud sync layer so teams can use Parallax on any device.

---

## Objectives

1. Compile the Go HTTP runner to WebAssembly — expose `sendRequest` and `convertToCode` as browser-callable functions.
2. Build a web app shell (Svelte, same component tree) that loads the WASM engine.
3. Add a sync backend (minimal — just collection/env storage) so web users share a workspace with desktop users via the existing GitHub-backed git flow.
4. Handle browser CORS constraints gracefully — proxy mode for cross-origin requests.

---

## Architecture Overview

```
Browser
  └── Parallax Web App (Svelte, same components)
        ├── parallax-worker.wasm  (Go HTTP engine compiled to WASM)
        │     ├── sendRequest(requestJson) → responseJson
        │     └── convertToCode(requestJson, language) → string
        └── Sync layer
              └── GitHub API (collections / envs stored as repo files, same as desktop)
```

Desktop and web share the same `.parallax/` git repository. The web app reads/writes via the GitHub REST API directly from the browser (no Parallax cloud server).

---

## Task Breakdown

### Go WASM Build

| Task | Status | Notes |
|---|---|---|
| Create `src-go/wasm/wasm.go` entry point | 🔲 | `//go:build js,wasm` build tag |
| Expose `sendRequest(inputJSON string) Promise<string>` via `syscall/js` | 🔲 | Parses request JSON, runs via `runner.RunRequest()`, returns response JSON |
| Expose `convertToCode(inputJSON string, language string) Promise<string>` via `syscall/js` | 🔲 | Reuses Go code-gen logic |
| Compile with `GOOS=js GOARCH=wasm go build -o parallax.wasm` | 🔲 | Add to Makefile / CI |
| Bundle `wasm_exec.js` (Go WASM glue) alongside the binary | 🔲 | Copied from `$(go env GOROOT)/misc/wasm/` |
| Handle CORS in WASM mode: detect `Access-Control-Allow-Origin` failures and surface a clear message | 🔲 | Browser fetch API rejects cross-origin without CORS headers |
| Proxy mode: when CORS is blocked, tunnel request through a lightweight server-side proxy | 🔲 | See "Proxy Mode" section below |
| Publish `parallax.wasm` as a GitHub release asset alongside desktop binaries | 🔲 | `release.yml` update |

**`wasm.go` structure (mirrors **'s `l2.wasm.go`):**
```go
//go:build js,wasm

package main

import (
    "syscall/js"
    "github.com/bluephoenix/parallax-worker/runner"
)

func main() {
    js.Global().Set("parallaxSendRequest", js.FuncOf(sendRequest))
    js.Global().Set("parallaxConvertToCode", js.FuncOf(convertToCode))
    <-make(chan struct{}) // keep alive
}
```

---

### Web App Shell

The web app reuses the existing Svelte component tree. Feature flags gate desktop-only capabilities (Tauri `invoke` calls are replaced with WASM calls).

| Task | Status | Notes |
|---|---|---|
| Add `src/lib/platform.ts` — abstraction layer over Tauri invoke vs WASM | 🔲 | `isDesktop()` guard; `sendRequest()` routes to correct backend |
| Replace all `invoke("send_request", ...)` calls with `platform.sendRequest()` | 🔲 | Audit all invoke calls; ~15 call sites |
| Replace `invoke("save_collection", ...)` with GitHub API write | 🔲 | `platform.saveCollection()` → GitHub REST |
| Replace `invoke("load_collection", ...)` with GitHub API read | 🔲 | `platform.loadCollection()` → GitHub REST |
| Web build target: `npm run build:web` — outputs to `dist/web/` | 🔲 | Separate Vite config; no Tauri, loads `parallax.wasm` |
| Auth: GitHub OAuth App flow (browser redirect) | 🔲 | Desktop uses Device Flow; browser needs redirect flow |
| Host on GitHub Pages or Cloudflare Pages | 🔲 | Auto-deploy from `release.yml` on tag push |
| Feature flags: disable proxy, health monitor, load tester, Go sidecar features in web build | 🔲 | These require the native Go sidecar |
| Responsive layout — web app works at 1024px+ | 🔲 | Sidebar collapse, panel resize |

**Capability matrix:**

| Feature | Desktop | Web |
|---|---|---|
| REST / GraphQL requests | ✅ | ✅ (WASM) |
| WebSocket / SSE / gRPC | ✅ | ❌ (browser restrictions) |
| Mock server | ✅ | ❌ |
| Proxy / traffic capture | ✅ | ❌ |
| Health monitor | ✅ | ❌ |
| Load tester | ✅ | ❌ |
| Git operations | ✅ native | ✅ via GitHub API |
| Team chat | ✅ P2P | ✅ git-relay only |
| AI features | ✅ | ✅ |
| Code generation | ✅ | ✅ (WASM) |
| Collections / environments | ✅ local | ✅ GitHub-backed |

---

### CORS Proxy Mode

Browsers block cross-origin requests without CORS headers. Parallax web needs a workaround for APIs that don't set `Access-Control-Allow-Origin`.

| Task | Status | Notes |
|---|---|---|
| Lightweight CORS proxy endpoint — `POST /proxy` accepts `{url, method, headers, body}`, forwards, returns response | 🔲 | Deploy as a Cloudflare Worker (zero cold start, free tier) |
| In WASM `sendRequest()`, catch CORS errors and retry via proxy with user confirmation | 🔲 | "This request was blocked by CORS — retry via Parallax proxy?" |
| Allow users to configure a self-hosted proxy URL | 🔲 | Settings → Web → Proxy URL |
| Privacy notice: proxy sees request/response data | 🔲 | Shown once on first CORS-proxy use |

---

### Sync Backend

No Parallax cloud server. Collections and environments live in a GitHub repo (same as the desktop git flow). The web app reads/writes via the GitHub REST API.

| Task | Status | Notes |
|---|---|---|
| `github-sync.ts` — read collection YAML from repo via `GET /repos/{owner}/{repo}/contents/{path}` | 🔲 | Returns base64-decoded content |
| `github-sync.ts` — write collection YAML via `PUT /repos/{owner}/{repo}/contents/{path}` | 🔲 | SHA-based upsert (same pattern as `github_publish_docs`) |
| Conflict detection: if remote SHA differs from last-read SHA, show diff before overwrite | 🔲 | |
| Offline mode: cache collections in `localStorage`, sync on next load | 🔲 | |
| Auto-sync: debounce writes — 2s after last change, push to GitHub | 🔲 | Same UX as Google Docs auto-save |

---

## Success Criteria

| Criteria | Status |
|---|---|
| `GOOS=js GOARCH=wasm go build` produces a working `parallax.wasm` | 🔲 |
| Web app loads in browser, sends a REST request via WASM | 🔲 |
| Collections load from and save to a GitHub repo | 🔲 |
| CORS-blocked requests surface a clear message + proxy retry option | 🔲 |
| Web app deployed to GitHub Pages on tag push | 🔲 |
| Desktop and web users can share the same workspace via git | 🔲 |
| Web app works on Chrome, Firefox, and Safari | 🔲 |
