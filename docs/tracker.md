# Parallax — Development Tracker

Last updated: 2026-04-23

---

## Overall Progress

```
Phase 1    ███████████████████░  97%  🔄 Near Complete (HTTP/3 + gRPC calls remain)
Phase 2    █████████████░░░░░░░  65%  🔄 In Progress (proxy/health/loadtest done; CLI + filter + SQLite + mock full remain)
Phase 2.5  ░░░░░░░░░░░░░░░░░░░░   0%  🔲 Planned (Git Collaboration + Chat)
Phase 3    ░░░░░░░░░░░░░░░░░░░░   0%  🔲 Planned
Phase 4    ░░░░░░░░░░░░░░░░░░░░   0%  🔲 Planned
Phase 5    ░░░░░░░░░░░░░░░░░░░░   0%  🔲 Planned
```

---

## Phase 1 — Architecture & Git-Native Core

### Infrastructure
| Task | Status | Notes |
|---|---|---|
| Tauri v2 project initialized | ✅ Done | Manual scaffold |
| Svelte 5 frontend wired | ✅ Done | Full component tree |
| Go sidecar compiled & bundled | ✅ Done | `parallax-worker-aarch64-apple-darwin` on :50151 |
| Sidecar stdout/stderr piped to terminal | ✅ Done | `tauri::async_runtime::spawn` reader |
| gRPC proto definitions | ✅ Done | `proto/parallax.proto` |
| `.parallax/` folder structure | ✅ Done | collections, environments, history, scripts dirs |
| `cargo check` passing | ✅ Done | Warnings only |
| `cargo tauri dev` launching | ✅ Done | App runs; Rust + Go sidecar |
| Tauri shell plugin config fixed | ✅ Done | |
| `.gitignore` with `.claude` and sensitive `.parallax/` paths | ✅ Done | |

### Protocols (HTTP Engine — Rust)
| Task | Status | Notes |
|---|---|---|
| REST — all methods | ✅ Done | Via `reqwest` |
| HTTP/2 | ✅ Done | `rustls` + `reqwest` |
| HTTP/3 | 🔲 | |
| GraphQL — query + variables body | ✅ Done | Body type wired |
| GraphQL schema introspection | ✅ Done | Fetch Schema button; type browser panel | |
| gRPC unary calls | 🔲 | |
| gRPC streaming | 🔲 | |
| WebSocket — connect/send/disconnect/stream | ✅ Done | `tokio-tungstenite`; Tauri event bridge |
| SSE — Server-Sent Events streaming | ✅ Done | `reqwest` bytes_stream; Tauri event bridge |
| Follow redirects (configurable) | ✅ Done | |
| Request timeout (configurable) | ✅ Done | |
| Proxy settings per environment | 🔲 | |
| SSL/TLS toggle per request | 🔲 | |
| Client certificates | 🔲 | |

### Template Tag Engine
| Task | Status | Notes |
|---|---|---|
| `{% uuid %}` / `{% guid %}` | ✅ Done | |
| `{% timestamp %}` | ✅ Done | |
| `{% now 'iso'/'unix'/'ms' %}` | ✅ Done | |
| `{% randomInt min max %}` | ✅ Done | |
| `{% randomEmail/Name/Phone/Word/Boolean/LoremIpsum %}` | ✅ Done | |
| `{% base64 encode/decode val %}` | ✅ Done | |
| `{% env 'VAR' %}` | ✅ Done | |
| `{{$guid}}`, `{{$timestamp}}`, `{{$isoTimestamp}}` | ✅ Done | Postman $ compat |
| `{{$randomEmail/Boolean/FullName/PhoneNumber/Word}}` | ✅ Done | |
| `{{var}}` — environment variable substitution | ✅ Done | |
| Template resolution wired into `sendRequest()` | ✅ Done | `resolveRequestTemplates()` on full payload |
| \`{% response 'body', '\$.path' %}\` — request chaining | ✅ Done | Template tag + JSONPath extractor |
| `{% file '/path' %}` | 🔲 | |
| `{% prompt 'label' %}` | ✅ Done | Uses window.prompt() | |

### Variable Scoping (4-level system)
| Task | Status | Notes |
|---|---|---|
| Global variables (`globals.json`) | ✅ Done | `globalVariables` store; `load_globals`/`save_globals` Rust commands |
| Collection variables | ✅ Done | `collectionVariables` store; loaded on request open |
| Environment variables | ✅ Done | `activeEnvironment`; load/save/edit working |
| Local variables (ephemeral, set by scripts) | ✅ Done | `pm.environment.set()` mutates merged env |
| Resolution priority: Global → Collection → Env → Local | ✅ Done | Merged in `sendRequest()` |

### Script Runner (`pm.*` API)
| Task | Status | Notes |
|---|---|---|
| JS runtime — sandboxed `new Function()` | ✅ Done | |
| `pm.environment.get/set/unset/has/toObject` | ✅ Done | |
| `pm.globals` / `pm.variables` | ✅ Done | Aliased to merged env |
| `pm.response.code/status/responseTime/responseSize` | ✅ Done | |
| `pm.response.json()` / `pm.response.text()` | ✅ Done | |
| `pm.response.headers.get(name)` | ✅ Done | |
| `pm.test(name, fn)` | ✅ Done | |
| `pm.expect(val)` — full Chai-style chain | ✅ Done | |
| `pm.visualizer.set(template, data)` | ✅ Done | Sets `visualizerData` store |
| Pre-request script runs before HTTP call | ✅ Done | |
| Test script runs after response | ✅ Done | |
| `pm.sendRequest()` | ✅ Done | Fires sub-request via Tauri invoke | |
| Python runtime (PyO3) | 🔲 | Phase 2 |

### Persistence & Import/Export
| Task | Status | Notes |
|---|---|---|
| `list/load/save/delete_collection` | ✅ Done | YAML |
| `list/load/save_environment` | ✅ Done | JSON |
| `load_globals` / `save_globals` | ✅ Done | `globals.json` |
| `save_history_entry` — `.parallax/history/` | ✅ Done | Timestamped JSON per request |
| `create_workspace` — scaffolds `.parallax/` dirs | ✅ Done | |
| Response history in-memory (capped 200) | ✅ Done | |
| Import from Postman Collection v2.1 JSON | ✅ Done | Requests, folders, auth, body |
| Import from Insomnia v4 export JSON | ✅ Done | |
| Import from curl command (URL bar paste) | ✅ Done | |
| Import from OpenAPI 3.x | 🔲 | Phase 4 |
| Import from HAR file | 🔲 | |
| Export as Postman JSON | ✅ Done | postman-exporter.ts + sidebar context menu | |

### Cookie Jar
| Task | Status | Notes |
|---|---|---|
| Cookie store in `reqwest` (`cookie_store(true)`) | ✅ Done | Rust engine |
| Cookie manager UI | ✅ Done | Cookies tab in response pane | |
| Per-request opt-in/opt-out | 🔲 | |

### Auth Providers
| Task | Status | Notes |
|---|---|---|
| Bearer token | ✅ Done | |
| Basic auth | ✅ Done | |
| API key (header) | ✅ Done | |
| API key (query param) | ✅ Done | api_key_location field + location selector UI | |
| Ecosystem provider selector UI (Frappe, Django, Laravel, Rails, WordPress, FastAPI) | ✅ Done | |
| Frappe / Django stubs | ✅ Done | Full in Phase 4 |
| OAuth2, AWS SigV4, Digest, mTLS | 🔲 | Phase 4 |

### Svelte 5 Builder Mode UI
| Task | Status | Notes |
|---|---|---|
| App shell (Builder + Dashboard + Design modes) | ✅ Done | |
| Method selector + URL bar | ✅ Done | |
| Params tab | ✅ Done | |
| Headers tab | ✅ Done | |
| Auth tab — all provider sub-forms | ✅ Done | |
| Body tab — JSON / form / URL-encoded / raw / GraphQL | ✅ Done | |
| Scripts tab — Pre-request editor | ✅ Done | |
| Scripts tab — Tests editor | ✅ Done | |
| WebSocket pane — connect/send/disconnect/frame stream | ✅ Done | `WebSocketPane.svelte` |
| SSE pane — connect/stream/close | ✅ Done | `SSEPane.svelte` |
| Response pane — status / time / size | ✅ Done | |
| Response body — JSON tree (colorized) / raw | ✅ Done | |
| Response headers tab | ✅ Done | |
| Response tests tab (pass/fail per `pm.test()`) | ✅ Done | |
| Response history tab (scrollable list + body preview) | ✅ Done | |
| Response visualizer tab (Handlebars + iframe) | ✅ Done | `VisualizerIframe.svelte` |
| Response cookies tab | ✅ Done | Cookie table in ResponsePanel | |
| Collection sidebar — tree (collections → folders → requests) | ✅ Done | |
| Sidebar search/filter | ✅ Done | |
| Sidebar method badges | ✅ Done | |
| Sidebar import button (Postman / Insomnia) | ✅ Done | |
| Sidebar git branch chip | ✅ Done | |
| Sidebar drag-and-drop reordering | 🔲 | |
| Sidebar right-click context menu | 🔲 | |
| Environment quick-edit overlay with secret masking | ✅ Done | |
| Environment variable count badge | ✅ Done | |
| Environment diff view | 🔲 | |
| Multi-tab UI | ✅ Done | |
| Tabs persist across restarts | ✅ Done | localStorage via persistTabs() | |
| Collection Runner UI | ✅ Done | `CollectionRunner.svelte` — iterations, delay, stop-on-failure, live feed |
| Framework logos | ✅ Done | `FrameworkLogo.svelte` |

### Go Sidecar
| Task | Status | Notes |
|---|---|---|
| `src-go/main.go` gRPC server | ✅ Done | |
| File watcher stub | ✅ Done | |
| Health check ping | ✅ Done | |
| Go binary compiled and bundled | ✅ Done | Running on :50151 |
| `ping_worker` Tauri command | ✅ Done | gRPC handshake to sidecar |
| Sidecar stdout/stderr piped to Tauri terminal | ✅ Done | |
| `WatchTraffic` gRPC stream | ✅ Done | Proxy service wired |
| `WatchHealth` gRPC stream | ✅ Done | Health service wired |

### Phase 1 Success Criteria
| Criteria | Status |
|---|---|
| `cargo tauri dev` launches without errors | ✅ |
| REST requests work end-to-end | ✅ |
| WebSocket connections work | ✅ |
| SSE streams work | ✅ |
| Template tags resolve at send-time | ✅ |
| Pre-request and test scripts execute with `pm` API | ✅ |
| Variable scoping resolves across all 4 levels | ✅ |
| Collections load/save to `.parallax/` | ✅ |
| Import Postman + Insomnia collections | ✅ |
| Import curl commands | ✅ |
| Response history saves in-memory + disk | ✅ |
| Response visualizer (Handlebars) works | ✅ |
| Collection runner with iterations + delay | ✅ |
| Go sidecar responds to ping | ✅ |
| Cookie jar management UI | ✅ |
| GraphQL schema introspection | ✅ |

---

## Phase 2 — Dashboard, Collection Runner & CLI

### Collection Runner (Postman core feature)
| Task | Status | Notes |
|---|---|---|
| Collection / folder selection | ✅ Done | `CollectionRunner.svelte` |
| Environment selection for run | ✅ Done | Uses active env + global/collection vars |
| Iteration count | ✅ Done | |
| Delay between requests | ✅ Done | |
| Stop on first failure toggle | ✅ Done | |
| Data file (CSV/JSON) for data-driven runs | ✅ Done | UI file picker + CSV/JSON loader in `CollectionRunner.svelte` |
| Variable chaining between requests in run | ✅ Done | `pm.environment.set()` in scripts persists |
| Live run feed UI | ✅ Done | Per-request pass/fail rows |
| Summary panel (passed/failed/time) | ✅ Done | |
| Report output (JSON + HTML) | ✅ Done | `runner-report.ts` — HTML template; saved to `.parallax/reports/` |

### `parallax-cli` (Newman equivalent, Go)
| Task | Status | Notes |
|---|---|---|
| All CLI commands | 🔲 | |

### Dashboard Mode
| Task | Status | Notes |
|---|---|---|
| Dashboard Mode UI shell | ✅ Done | `DashboardMode.svelte` |
| Live Traffic Stream panel | ✅ Done | `LiveTrafficPanel.svelte` — streams from proxy gRPC |
| Health Heatmap panel | ✅ Done | `HealthHeatmapPanel.svelte` — streams from health gRPC |
| Load Test Results panel | ✅ Done | `LoadTestPanel.svelte` — RPS chart + latency histogram |
| Git Sync Status panel | 🔲 | Phase 2.5 |
| Collection Run History panel | 🔲 | |

### Go Local Proxy (Rust commands wired to gRPC)
| Task | Status | Notes |
|---|---|---|
| `start_proxy_stream` / `get_proxy_traffic` / `clear_proxy_traffic` | ✅ Done | Rust ↔ gRPC wired |
| HTTP proxy server in Go (`localhost:8765`) | ✅ Done | `src-go/proxy/proxy.go` (274 lines) — real HTTP/HTTPS intercept |
| HTTPS MITM with local CA cert | ✅ Done | `src-go/proxy/ca.go` (99 lines) — CA cert generation |
| Traffic filter by domain/method/status | 🔲 | |
| Export as HAR | 🔲 | |
| Replay captured request | 🔲 | |

### Health Monitor (Rust commands wired to gRPC)
| Task | Status | Notes |
|---|---|---|
| `start_health_stream` / `add/remove_health_target` / `get_health_statuses` | ✅ Done | Rust ↔ gRPC wired |
| Goroutine-per-service health checks in Go | ✅ Done | `src-go/health/health.go` (180 lines) — goroutine-per-service |
| SQLite uptime history | 🔲 | |
| Desktop notifications on status change | 🔲 | |
| Alert webhook on failure | 🔲 | |

### Load Tester
| Task | Status | Notes |
|---|---|---|
| Go concurrent engine | ✅ Done | Histogram + RPS calc |
| Rust gRPC bridge | ✅ Done | `run_load_test` command |
| UI Panel | ✅ Done | `LoadTestPanel.svelte` with histogram |
| StreamLoadTest stream | ✅ Done | |

### Response Visualization (Postman Visualizer)
| Task | Status | Notes |
|---|---|---|
| Visualize tab in response pane | ✅ Done | |
| Handlebars template renderer | ✅ Done | `handlebars` npm package |
| Sandboxed iframe | ✅ Done | `VisualizerIframe.svelte` |
| `pm.visualizer.set(template, data)` API | ✅ Done | Sets `visualizerData` store |

### Mock Server
| Task | Status | Notes |
|---|---|---|
| Go mock server — start/stop + AddRule/RemoveRule | ✅ Done | `src-go/mock/mock.go` (86 lines); Rust commands in `mock.rs` |
| Path parameters (`:id`) and wildcards | 🔲 | |
| Response templating with request data | 🔲 | |
| Configurable response delay | 🔲 | |
| Record mode (proxy + auto-generate rules) | 🔲 | |
| `parallax-cli mock` command | 🔲 | |

### gRPC Streaming Bridge
| Task | Status | Notes |
|---|---|---|
| `WatchTraffic` stream | ✅ Done | Rust command → gRPC |
| `WatchHealth` stream (`WatchStatuses`) | ✅ Done | Rust command → gRPC |
| `StreamLoadTest` (`RunLoadTest`) stream | ✅ Done | `grpc/server.go` line 208 |
| `WatchFiles` stream | 🔲 | |
| `StreamRunner` stream | 🔲 | |

---

## Phase 2.5 — Git Collaboration & Team Chat

> Not started. Architecture: Go sidecar as WebSocket chat broker, git repo as signaling + persistence layer. No Parallax cloud ever required.

### Git-Native Workspace

| Task | Status | Notes |
|---|---|---|
| Workspace as a git repo (`git init` / `git clone` on create) | 🔲 | `git2` Rust crate |
| `commit` command — stage + commit `.parallax/` changes | 🔲 | Message from UI; author = GitHub identity |
| `push` command — push to remote | 🔲 | |
| `pull` command — fetch + merge remote changes | 🔲 | |
| `stash` / `stash pop` commands | 🔲 | |
| Branch create / switch / delete | 🔲 | Extend existing git branch chip in sidebar |
| Conflict detection + diff view on pull | 🔲 | Show conflicting files; user resolves |
| Commit history panel | 🔲 | Log of commits with author + message |
| Git status badge on sidebar (uncommitted changes count) | 🔲 | |

### GitHub OAuth Identity

| Task | Status | Notes |
|---|---|---|
| GitHub OAuth2 PKCE flow via Tauri | 🔲 | Opens browser → redirect back to app |
| Store GitHub token + user info in local keychain | 🔲 | `tauri-plugin-keychain` or OS keyring |
| GitHub ID as universal user identity in Parallax | 🔲 | Used for commits, presence, chat |
| Sign-out / revoke token | 🔲 | |
| Display GitHub avatar + username in titlebar | 🔲 | |

### Publish API Docs to GitHub

| Task | Status | Notes |
|---|---|---|
| Generate static HTML docs from collection | 🔲 | Reuse Phase 3 doc generator |
| Push generated docs to `gh-pages` branch | 🔲 | Auto-commit + push |
| Publish settings: public repo / private repo toggle | 🔲 | |
| Custom doc site title + description | 🔲 | |
| "View live docs" button (opens GitHub Pages URL) | 🔲 | |

### Team Workspaces

| Task | Status | Notes |
|---|---|---|
| Invite teammate by GitHub username (adds as repo collaborator via GitHub API) | 🔲 | |
| List team members in workspace sidebar | 🔲 | Pulled from GitHub repo collaborators |
| Remove teammate (revoke collaborator access) | 🔲 | |
| Each workspace has its own independent team | 🔲 | Natural: each workspace = separate repo |
| Workspace visibility badge (public / private repo) | 🔲 | |

### Real-Time Chat (Go Sidecar)

| Task | Status | Notes |
|---|---|---|
| `Chat` gRPC service in Go sidecar | 🔲 | `ConnectPeer`, `SendMessage`, `GetHistory`, `SetPresence` |
| Per-user enable/disable toggle (local config, not committed) | 🔲 | When disabled: sidecar skips chat listener, user appears offline |
| Peer discovery via `.parallax/team/presence.json` (committed to repo) | 🔲 | `{github-id}:{ip}:{port}` written on connect, pulled by peers |
| Direct P2P WebSocket between sidecars (same network / VPN) | 🔲 | Default mode |
| Git-relay fallback (messages as `.parallax/chat/{workspace-id}/messages.jsonl`, polling 15s) | 🔲 | Works across internet without any relay server |
| Custom relay URL (optional, user-configured) | 🔲 | Self-hosted WebSocket relay for remote teams |
| Chat persistence — append-only JSONL, git-tracked | 🔲 | Full history versioned with the workspace |
| Offline message queue — flush on next push | 🔲 | |
| Chat UI panel in Dashboard / workspace view | 🔲 | Threaded by workspace; GitHub avatar + username per message |
| Online presence indicators (green dot on teammate avatar) | 🔲 | From `presence.json` + direct heartbeat |
| Unread message badge | 🔲 | |

### Phase 2.5 Success Criteria

| Criteria | Status |
|---|---|
| User can commit/push/pull workspace changes from within Parallax | 🔲 |
| GitHub login gives identity used for all git ops + chat | 🔲 |
| Team invited by GitHub username can clone + join workspace | 🔲 |
| API docs published to GitHub Pages in one click | 🔲 |
| Chat works P2P on same network with no external server | 🔲 |
| Chat falls back to git-relay when P2P unavailable | 🔲 |
| Chat can be fully disabled per user with no side effects | 🔲 |

---

## Phase 3 — AI Integration & MCP Server

> Not started.

### BYO-AI Providers
| Task | Status |
|---|---|
| OpenAI / Anthropic / Ollama / Gemini / Custom providers | 🔲 |
| AI settings UI + `ai.json` config | 🔲 |
| Air-gap mode | 🔲 |

### AI Features
| Task | Status |
|---|---|
| AI test generator | 🔲 |
| AI request repair (4xx/5xx) | 🔲 |
| AI collection creator | 🔲 |
| AI script assistant | 🔲 |
| AI env variable suggestion | 🔲 |

### MCP Server
| Task | Status |
|---|---|
| MCP HTTP server (`localhost:7676`) | 🔲 |
| All `parallax.*` tool endpoints | 🔲 |

### Documentation Generator
| Task | Status |
|---|---|
| Static HTML / Markdown export | 🔲 |
| OpenAPI reverse-generation from collection | 🔲 |

---

## Phase 4 — Ecosystem Intelligence

> Not started.

### Auth Providers (Full Set)
| Task | Status |
|---|---|
| Frappe / ERPNext (full) | 🔲 |
| Django / Laravel / Rails / WordPress / Next.js / FastAPI / ASP.NET | 🔲 |
| OAuth2 (code + PKCE + client_credentials + password) | 🔲 |
| AWS Signature v4 / Digest / NTLM / mTLS | 🔲 |

### Schema Explorer (Go)
| Task | Status |
|---|---|
| Frappe / Django / Laravel / Rails / FastAPI / Express explorers | 🔲 |
| OpenAPI 3.x full importer | 🔲 |
| Framework auto-detection on folder open | 🔲 |

### Design Mode (OpenAPI Editor)
| Task | Status |
|---|---|
| All Design Mode features | 🔲 |

### Response Intelligence
| Task | Status |
|---|---|
| Export as JSON Schema / TS interface / Pydantic / Rust / Go struct | 🔲 |

### Visual Flow Builder
| Task | Status |
|---|---|
| Canvas editor + all node types | 🔲 |

### Enhanced Protocol Support
| Task | Status |
|---|---|
| gRPC service reflection + unary calls | 🔲 |
| GraphQL schema explorer + autocomplete + builder | 🔲 |

---

## Phase 5 — Polish, Performance & Release

> Not started.

### Performance targets
| Target | Status |
|---|---|
| Startup < 800ms / RAM idle < 80MB / Request overhead < 5ms | 🔲 |

### Code Generation (13 languages)
| Task | Status |
|---|---|
| curl / Python / JS / Rust / Go / PHP / Ruby / Java / C# / Swift / Kotlin | 🔲 |

### Plugin System
| Task | Status |
|---|---|
| Plugin API + loader + sandbox + registry | 🔲 |
| parallax-plugin-faker/jwt/aws-sigv4/base64/xml/soap | 🔲 |

### Keyboard & Command Palette
| Task | Status |
|---|---|
| Cmd+K command palette | 🔲 |
| Full shortcut table | 🔲 |

### Themes
| Task | Status |
|---|---|
| Parallax Dark (default) | ✅ Done (Phase 1) |
| Light / High Contrast / Monokai / Solarized / Custom CSS | 🔲 |

### Distribution
| Task | Status |
|---|---|
| macOS `.dmg` / Windows `.msi` / Linux `.AppImage/.deb/.rpm` | 🔲 |
| Code signing + notarization | 🔲 |
| GitHub Actions CI/CD + auto-updater | 🔲 |

### Documentation
| Task | Status |
|---|---|
| Docs site / guides / API reference / README | 🔲 |

---

## Current Blockers

| Blocker | Impact | Priority |
|---|---|---|
| ~~Cookie manager UI missing~~ | ~~Cookies work in reqwest but not inspectable~~ | ✅ Resolved |
| ~~Go-side proxy HTTP server not implemented~~ | ~~`LiveTrafficPanel` Rust commands exist but Go not wired~~ | ✅ Resolved — `proxy.go` + `ca.go` implemented |
| ~~Go-side health goroutines not implemented~~ | ~~`HealthHeatmapPanel` Rust commands exist but Go not wired~~ | ✅ Resolved — `health.go` goroutines implemented |
| ~~GraphQL schema introspection missing~~ | ~~GraphQL body works; no schema browser~~ | ✅ Resolved |
| `parallax-cli` not started | No Newman equivalent for CI/CD pipelines | 🟡 P1 |
| Go runner missing template engine + test scripts | CLI runner executes requests but can't resolve `{{vars}}` or run `pm.test()` | 🟡 P1 |
| Mock server path params + templating not implemented | Basic rules only; `:id` routes don't work | 🟢 P2 |
| Sidebar drag-and-drop reordering | Collections are static order | 🟢 P2 |

---

## Decision Log

| Date | Decision | Reason |
|---|---|---|
| 2026-04-21 | Manual scaffold instead of `npm create tauri-app` | CLI was hanging |
| 2026-04-21 | Frappe auth → Framework-agnostic auth provider system | All users need it |
| 2026-04-21 | Include ALL Postman + Insomnia must-haves, not just gaps | Parallax is both tools unified |
| 2026-04-21 | Tauri shell plugin `all`/`execute`/`sidecar` fields removed | Tauri v2 only supports `open` |
| 2026-04-22 | Script runner uses `new Function()` sandbox | No native dep; sufficient for local trusted tool |
| 2026-04-22 | Postman + Insomnia import in TypeScript (not Rust) | Runs in renderer, no IPC round-trip needed |
| 2026-04-22 | Response history capped at 200 in-memory entries | Prevents memory growth; disk persistence via `save_history_entry` |
| 2026-04-22 | Handlebars for Visualizer (not custom renderer) | Postman-compatible, well-tested, 4kB gzip |
| 2026-04-22 | WS/SSE use Tauri `app.emit()` events bridge | Avoids polling; frontend listens with `listen()` |
| 2026-04-23 | API key location (header vs query) via `api_key_location` field | Complete parity with Postman API key auth |
| 2026-04-23 | GraphQL schema browser uses inline panel in body tab | Avoids modal; keeps context while building queries |
| 2026-04-23 | Tab persistence via `persistTabs()` on every mutation | Tabs survive app restarts without needing IPC |
| 2026-04-23 | Sidebar export uses `postman-exporter.ts` canonical utility | Single source of truth for v2.1 format |
| 2026-04-23 | Phase 2.5: Go sidecar is the chat WebSocket broker, not a Parallax cloud server | Local-first; no infra cost; fits existing sidecar architecture |
| 2026-04-23 | Chat is per-user opt-in toggle (local config, never committed) | Privacy control; user appears offline when disabled |
| 2026-04-23 | Peer discovery via `presence.json` committed to git repo | Git repo is the signaling layer; no external discovery server |
| 2026-04-23 | Git-relay fallback stores chat as `.parallax/chat/*.jsonl` (polled every 15s) | Works cross-internet with zero relay infra |
| 2026-04-23 | GitHub ID is universal identity (git author, presence, chat, team invites) | Eliminates separate user database; leverages existing GitHub social graph |
| 2026-04-23 | Team = GitHub repo collaborators; invite by GitHub username via GitHub API | No Parallax user management needed; permissions enforced by GitHub |

---

## Legend

| Symbol | Meaning |
|---|---|
| ✅ Done | Complete and working |
| 🔄 In Progress | Currently being worked on |
| ❌ Blocked | Blocked by an issue listed above |
| 🔲 Planned | Not started yet |
| 🟢 P2 | Low priority |
| 🟡 P1 | Medium priority |
| 🔴 P0 | Critical |
