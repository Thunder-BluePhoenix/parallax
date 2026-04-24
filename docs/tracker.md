# Parallax — Development Tracker

Last updated: 2026-04-24 (Phase 1 complete — all gaps closed)

---

## Overall Progress

```
Phase 1    ████████████████████ 100%  ✅ Complete
Phase 2    ████████████████████ 100%  ✅ Complete
Phase 2.5  ████████████████████ 100%  ✅ Complete (Git Collaboration + Chat)
Phase 3    ████████████████████ 100%  ✅ Complete (AI providers, MCP server, Design Mode, script assistant, OpenAPI export)
Phase 4    ████████████████████ 100%  ✅ Complete (auth providers, schema crawlers, response intelligence, flow builder)
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
| HTTP/3 | 🔲 | Requires unstable Rust feature flag — deferred |
| GraphQL — query + variables body | ✅ Done | Body type wired |
| GraphQL schema introspection | ✅ Done | Fetch Schema button; type browser panel | |
| gRPC unary calls | ✅ Done | `grpc_unary` Rust command + `GRPCPane.svelte` — HTTP/2 + grpc+json framing |
| gRPC streaming | ✅ Done | `grpc_server_stream` Tauri command + Tauri events per frame; stream feed UI in `GRPCPane.svelte` |
| WebSocket — connect/send/disconnect/stream | ✅ Done | `tokio-tungstenite`; Tauri event bridge |
| SSE — Server-Sent Events streaming | ✅ Done | `reqwest` bytes_stream; Tauri event bridge |
| Follow redirects (configurable) | ✅ Done | |
| Request timeout (configurable) | ✅ Done | |
| Proxy settings per environment | ✅ Done | `proxy_url` field on `ParallaxRequest`; `HttpEngine::build_for_request()` |
| SSL/TLS toggle per request | ✅ Done | `tls_skip_verify` field; `danger_accept_invalid_certs` per-request client |
| Client certificates | ✅ Done | `client_cert_pem` + `client_key_pem` fields; `reqwest::Identity::from_pem` |

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
| `{% file '/path' %}` | ✅ Done | `read_file_for_template` Tauri command; async pre-processing in `sendRequest()` before template engine |
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
| Import from OpenAPI 3.x | ✅ Done | `openapi-importer.ts` — JSON spec → Collection; toolbar button in Sidebar |
| Import from HAR file | ✅ Done | `har-importer.ts` + HAR button in BuilderMode tab bar |
| Export as Postman JSON | ✅ Done | postman-exporter.ts + sidebar context menu | |

### Cookie Jar
| Task | Status | Notes |
|---|---|---|
| Cookie store in `reqwest` (`cookie_store(true)`) | ✅ Done | Rust engine |
| Cookie manager UI | ✅ Done | Cookies tab in response pane | |
| Per-request opt-in/opt-out | ✅ Done | `disable_cookies` field on `ParallaxRequest`; per-request client skips cookie jar |

### Auth Providers
| Task | Status | Notes |
|---|---|---|
| Bearer token | ✅ Done | |
| Basic auth | ✅ Done | |
| API key (header) | ✅ Done | |
| API key (query param) | ✅ Done | api_key_location field + location selector UI | |
| Ecosystem provider selector UI (Frappe, Django, Laravel, Rails, WordPress, FastAPI) | ✅ Done | |
| Frappe / Django stubs | ✅ Done | Full in Phase 4 |
| OAuth2 / PKCE | ✅ Done | Phase 4 — `auth_oauth2_pkce()` in `auth_providers.rs` |
| AWS SigV4 | ✅ Done | HMAC-SHA256 signing chain in `http_engine.rs`; `AwsSigV4` AuthType |
| Digest auth | ✅ Done | RFC 7617 MD5 challenge-response in `http_engine.rs`; `Digest` AuthType |
| mTLS (client certs) | ✅ Done | `client_cert_pem` / `client_key_pem` fields on `ParallaxRequest` |
| NTLM | ⚠️ Deferred | Windows-specific; no cross-platform Rust crate available |

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
| Sidebar drag-and-drop reordering | ✅ Done | HTML5 drag-and-drop on req-items in `Sidebar.svelte`; reorders within collection/folder |
| Sidebar right-click context menu | ✅ Done | Duplicate / Rename / Delete on requests, folders, collections |
| Environment quick-edit overlay with secret masking | ✅ Done | |
| Environment variable count badge | ✅ Done | |
| Environment diff view | ✅ Done | Diff tab in `EnvironmentPanel.svelte` — changed/added/removed comparison against any saved env |
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
| gRPC unary calls work (`application/grpc+json`) | ✅ |
| Template tags resolve at send-time (incl. `{% file %}`) | ✅ |
| Pre-request and test scripts execute with `pm` API | ✅ |
| Variable scoping resolves across all 4 levels | ✅ |
| Collections load/save to `.parallax/` | ✅ |
| Import Postman + Insomnia + HAR collections | ✅ |
| Import curl commands | ✅ |
| Response history saves in-memory + disk | ✅ |
| Response visualizer (Handlebars) works | ✅ |
| Collection runner with iterations + delay | ✅ |
| Go sidecar responds to ping | ✅ |
| Cookie jar management UI | ✅ |
| GraphQL schema introspection | ✅ |
| Request cancellation (abort in-flight requests) | ✅ |
| SSL skip-verify / proxy / client cert per request | ✅ |
| Sidebar right-click context menu | ✅ |

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
| `parallax run <collection>` | ✅ Done | Flags: `-e` env, `-g` globals, `-i` iterations, `-d` delay, `-v` verbose, `--data`, `--reporter` |
| `parallax mock <port>` | ✅ Done | Standalone mock server; no DB persistence in CLI mode |
| `parallax init` | ✅ Done | Scaffolds `.parallax/` dirs + `default.json` environment |
| `parallax run --data <csv/json>` | ✅ Done | CSV header row + data rows; JSON array of objects — each row overrides env per iteration |
| `parallax run --reporter html` | ✅ Done | Self-contained HTML report saved to `.parallax/reports/run-{ts}.html` |

### Dashboard Mode
| Task | Status | Notes |
|---|---|---|
| Dashboard Mode UI shell | ✅ Done | `DashboardMode.svelte` |
| Live Traffic Stream panel | ✅ Done | `LiveTrafficPanel.svelte` — streams from proxy gRPC |
| Health Heatmap panel | ✅ Done | `HealthHeatmapPanel.svelte` — streams from health gRPC |
| Load Test Results panel | ✅ Done | `LoadTestPanel.svelte` — RPS chart + latency histogram |
| Git Sync Status panel | ✅ Done | `GitPanel.svelte` in Dashboard — git status, commit, push, pull, branch management |
| Collection Run History panel | ✅ Done | "Run History" section in `DashboardMode.svelte` — `responseHistory` cards with status/timing |

### Go Local Proxy (Rust commands wired to gRPC)
| Task | Status | Notes |
|---|---|---|
| `start_proxy_stream` / `get_proxy_traffic` / `clear_proxy_traffic` | ✅ Done | Rust ↔ gRPC wired |
| HTTP proxy server in Go | ✅ Done | `src-go/proxy/proxy.go` (428 lines) — real HTTP/HTTPS intercept; capped 5000 entries |
| HTTPS MITM with local CA cert | ✅ Done | `src-go/proxy/ca.go` (100 lines) — CA cert gen; `/parallax/ca.crt` download endpoint |
| Traffic filter by domain/method/status | ✅ Done | Go `SetFilter()` + gRPC + Rust `set_proxy_filter` + UI domain include/exclude fields |
| Export as HAR | ✅ Done | Client-side HAR 1.2 + Go `ExportHAR()`; "Export HAR" button in LiveTrafficPanel |
| Replay captured request | ✅ Done | "Replay" button in `LiveTrafficPanel.svelte` — calls `loadRequestIntoTab` + switches to builder |

### Health Monitor (Rust commands wired to gRPC)
| Task | Status | Notes |
|---|---|---|
| `start_health_stream` / `add/remove_health_target` / `get_health_statuses` | ✅ Done | Rust ↔ gRPC wired; `alert_webhook` field passed through |
| Goroutine-per-service health checks in Go | ✅ Done | `src-go/health/health.go` (194 lines) — goroutine-per-service |
| SQLite uptime history | ✅ Done | `storage.go` `SaveHealthStatus()` called on every check |
| Desktop notifications on status change | ✅ Done | `HealthHeatmapPanel.svelte` — `sendNotification` on down/restored transitions |
| Alert webhook on failure | ✅ Done | `health.go` fires POST to `AlertWebhook` when status transitions to "down" |

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
| Go mock server — start/stop + AddRule/RemoveRule/ListRules | ✅ Done | `src-go/mock/mock.go` (210 lines); Rust `mock.rs` + SQLite persistence |
| Path parameters (`:id`) and wildcards (`*`) | ✅ Done | `mock.go` `matchPath()` — `:param` captured into `{{.Params.id}}` |
| Response templating with request data | ✅ Done | `mock.go` `renderBody()` — Go `text/template`; `{{.Params.*}}` and `{{.Query.*}}` |
| Configurable response delay | ✅ Done | `x-parallax-delay-ms` header convention; UI delay field in MockServerPanel; Go sleeps before responding |
| Record mode (proxy + auto-generate rules) | 🔲 | Deferred — complex; not blocking Phase 2.5 |
| `parallax-cli mock` command | ✅ Done | `main.go` `handleCLIMock` — standalone mock server on any port |

### gRPC Streaming Bridge
| Task | Status | Notes |
|---|---|---|
| `WatchTraffic` stream | ✅ Done | Rust command → gRPC |
| `WatchHealth` stream (`WatchStatuses`) | ✅ Done | Rust command → gRPC |
| `StreamLoadTest` (`RunLoadTest`) stream | ✅ Done | `grpc/server.go` — full progress + result |
| `WatchFiles` stream (`WatchWorkspace`) | ✅ Done | `watcher.go` fsnotify + gRPC + Rust `watch_workspace`/`unwatch_workspace` + `workspace_file_changed` event |
| `StreamRunner` stream (`RunCollection`) | ✅ Done | `server.go` loads collection from path, resolves env, streams per-request events via `RunCollectionStream` |

---

## Phase 2.5 — Git Collaboration & Team Chat

> Complete. Architecture: Go sidecar as HTTP+SSE chat hub on `:50152`, git2 Rust crate for all git ops, GitHub Device Flow for identity, GitHub API for team management. No Parallax cloud required.

### Git-Native Workspace

| Task | Status | Notes |
|---|---|---|
| Workspace as a git repo (`git init` on create) | ✅ Done | `git2` crate; `create_workspace` calls `Repository::init()` |
| `git_status` — branch + uncommitted changes | ✅ Done | `spawn_blocking` + git2 |
| `git_commit` — stage all + commit with author | ✅ Done | Signature from GitHub identity name/email |
| `git_push` — push to remote | ✅ Done | git2 remote push |
| `git_pull` — fetch + fast-forward | ✅ Done | Returns error on merge conflicts |
| `git_stash` / `git_stash_pop` | ✅ Done | `&mut repo` pattern |
| `git_create_branch` / `git_switch_branch` / `git_delete_branch` | ✅ Done | |
| `git_log` — commit history with author + message | ✅ Done | `GitCommit` struct |
| `git_branches` — local + remote branch list | ✅ Done | |
| `git_diff` — unstaged diff as string | ✅ Done | |
| Git panel in Dashboard | ✅ Done | `GitPanel.svelte` — status, log, branches, diff, commit UI |

### GitHub OAuth Identity

| Task | Status | Notes |
|---|---|---|
| GitHub Device Authorization Grant flow | ✅ Done | No client_secret; POST device/code → poll access_token |
| Store GitHub token + identity in `~/.parallax/github_auth.json` | ✅ Done | `parallax_home()` via `HOME`/`USERPROFILE` env vars |
| GitHub ID as universal identity (commits, presence, chat) | ✅ Done | `githubIdentity` store wired everywhere |
| Sign-out / revoke token | ✅ Done | Deletes `~/.parallax/github_auth.json` |
| GitHub avatar + username in titlebar | ✅ Done | Titlebar shows avatar, `@login`, unread badge |
| Auto-load identity on app start | ✅ Done | `loadGitHubIdentity` called in `onMount` |

### Publish API Docs to GitHub

| Task | Status | Notes |
|---|---|---|
| Client-side HTML generator from collection | ✅ Done | `DocsPanel.svelte` — walks folders/requests, method badges, sidebar nav |
| Push generated `index.html` to `gh-pages` branch | ✅ Done | `github_publish_docs` Rust command; creates branch from default if missing |
| Upsert (SHA-based) for existing `index.html` | ✅ Done | Checks existing file SHA before PUT |
| Live URL shown after publish | ✅ Done | `https://{owner}.github.io/{repo}` link with copy button |
| HTML preview before publish | ✅ Done | Sandboxed iframe in `DocsPanel.svelte` |

### Team Workspaces

| Task | Status | Notes |
|---|---|---|
| Invite teammate by GitHub username | ✅ Done | `github_invite_collaborator` → GitHub REST API |
| List repo collaborators | ✅ Done | `github_list_collaborators` |
| Remove teammate | ✅ Done | `github_remove_collaborator` |
| Online presence dots on collaborator list | ✅ Done | `chatPresence` store cross-referenced in `TeamPanel.svelte` |
| Team panel in Dashboard | ✅ Done | `TeamPanel.svelte` — login, repo picker, collaborator list, invite |

### Real-Time Chat (Go Sidecar)

| Task | Status | Notes |
|---|---|---|
| HTTP+SSE chat hub in Go sidecar on `:50152` | ✅ Done | `src-go/chat/chat.go` — `Hub` struct, SSE stream, broadcast |
| Message persistence — append-only JSONL | ✅ Done | `.parallax/chat/messages.jsonl` per workspace |
| P2P message forwarding to peer sidecars | ✅ Done | `forwardToPeers()` — skips self by LAN IP:port comparison |
| LAN IP auto-detection | ✅ Done | `detectLANIP()` via `net.InterfaceAddrs()` |
| Offline outbox with 15s retry, max 20 attempts | ✅ Done | `drainOutbox()` goroutine; `outboxItem.Retries` |
| Anti-loop `forwarded` flag | ✅ Done | Server skips `forwardToPeers` when `forwarded: true` |
| In-memory presence with 5-min expiry | ✅ Done | `evictPresence()` goroutine; server fills `IP:Port` on set |
| `/chat/stream` SSE endpoint | ✅ Done | 15s keepalive ticker |
| `/chat/message` POST endpoint | ✅ Done | Creates UUID, persists, broadcasts, forwards |
| `/chat/history` GET endpoint | ✅ Done | Loads from JSONL |
| `/chat/presence` GET+POST endpoints | ✅ Done | |
| `/chat/info` endpoint | ✅ Done | Returns `{ip, port}` |
| Rust `chat_start_stream` — SSE → Tauri events | ✅ Done | `percent_encoding` URL-encode; emits `chat_message` events |
| Rust `chat_post_message` / `chat_get_history` | ✅ Done | HTTP → Go hub |
| Rust `chat_set_presence` / `chat_get_presence` | ✅ Done | |
| Git-relay fallback (poll history every 30s) | ✅ Done | `ChatPanel.svelte` `pollHistory()` with ID-set dedup |
| "Sync via Git" button | ✅ Done | `git_pull` → reload → `git_commit` + `git_push` |
| Relay mode badge (shown after 45s no SSE activity) | ✅ Done | `relayMode` derived in `ChatPanel.svelte` |
| Unread message badge in titlebar + nav | ✅ Done | `unreadCount` store; badge on Chat nav item |
| Chat panel in Dashboard | ✅ Done | `ChatPanel.svelte` — message list, composer, presence sidebar, relay UI |

### Phase 2.5 Success Criteria

| Criteria | Status |
|---|---|
| User can commit/push/pull workspace changes from within Parallax | ✅ |
| GitHub login gives identity used for all git ops + chat | ✅ |
| Team invited by GitHub username can clone + join workspace | ✅ |
| API docs published to GitHub Pages in one click | ✅ |
| Chat works P2P on same network with no external server | ✅ |
| Chat falls back to git-relay when P2P unavailable | ✅ |
| Chat can be fully disabled per user with no side effects | ✅ |

---

## Phase 3 — AI Integration & MCP Server

> Complete. All 5 AI providers, 4 AI actions, MCP server with all 3 tools wired, Design Mode (OpenAPI editor), AI script assistant in scripts tab, OpenAPI 3.0 export from DocsPanel.

### BYO-AI Providers
| Task | Status | Notes |
|---|---|---|
| OpenAI provider | ✅ Done | `ai.go` `callOpenAI()` — JSON mode; chat completions |
| Ollama provider | ✅ Done | `ai.go` `callOllama()` — local model, JSON format |
| Anthropic (Claude) provider | ✅ Done | `ai.go` `callAnthropic()` — `x-api-key` + `anthropic-version` headers |
| Google Gemini provider | ✅ Done | `ai.go` `callGemini()` — `responseMimeType: application/json` |
| Custom (OpenAI-compatible) provider | ✅ Done | Reuses `callOpenAI()` with configurable `baseUrl` |
| AI settings UI + `ai.svelte.ts` store | ✅ Done | `AISettingsPanel.svelte` + `ai.svelte.ts` — 4 store actions |
| Air-gap mode (Ollama-only) | ✅ Done | Guard in every store action; cloud providers rejected |

### AI Features
| Task | Status | Notes |
|---|---|---|
| AI test generator | ✅ Done | Response panel "Generate Tests" → `ai_generate_tests` → Go gRPC |
| AI request repair (4xx/5xx) | ✅ Done | Response panel "Diagnose with AI" → `ai_repair_request` → diagnosis + fix list |
| AI collection creator | ✅ Done | `AISettingsPanel` textarea → `ai_create_collection` → YAML output + copy |
| AI script assistant | ✅ Done | Scripts tab prompt bar → `generateScript` → inserts into pre-request or test editor |
| AI env variable suggestion | ✅ Done | "⚡ Suggest from request" button in `EnvironmentPanel.svelte` — extracts base_url, auth tokens, API key headers |

### MCP Server
| Task | Status | Notes |
|---|---|---|
| MCP HTTP+SSE server (`localhost:7676`) | ✅ Done | `src-go/mcp/mcp.go` — JSON-RPC 2.0 over HTTP; `--mcp` flag enables |
| `parallax.list_collections` tool | ✅ Done | Reads `.parallax/collections/` dir; returns name + path + request count |
| `parallax.get_traffic` tool | ✅ Done | Returns last N proxy entries from live proxy service |
| `parallax.execute_request` tool | ✅ Done | Loads collection YAML, finds request by ID/name, runs via `runner.RunRequest()` |
| Bearer token auth | ✅ Done | `--mcp-token` flag; `Authorization: Bearer` checked on SSE + message endpoints |
| MCP toggle + URL in AI settings UI | ✅ Done | `AISettingsPanel` toggle shows `http://localhost:7676/mcp/sse` + copy button |

### Documentation Generator
| Task | Status | Notes |
|---|---|---|
| Static single-page HTML export | ✅ Done | `DocsPanel.svelte` `generateHTML()` — sidebar nav, method badges, headers/params/body/scripts |
| Publish HTML to GitHub Pages | ✅ Done | `github_publish_docs` Rust command → `gh-pages` branch upsert |
| OpenAPI 3.0 JSON export | ✅ Done | `DocsPanel.svelte` `generateOpenAPI()` — paths, parameters, requestBody, security schemes, tags |
| OpenAPI download button | ✅ Done | Client-side Blob download as `{collection-name}-openapi.json` |
| Design Mode — OpenAPI YAML editor | ✅ Done | `DesignMode.svelte` + `YamlEditor.svelte` + `ApiPreview.svelte` — live parse + endpoint preview |

---

## Phase 4 — Ecosystem Intelligence

> Complete. Full auth provider set, schema crawlers for 7 frameworks, response intelligence (5 schema export formats), OAuth2 PKCE, and visual SVG flow builder.

### Auth Providers (Full Set)
| Task | Status | Notes |
|---|---|---|
| Frappe / ERPNext (full) | ✅ Done | `auth_providers.rs` — sid+CSRF token flow |
| Django | ✅ Done | Session cookie + CSRF header |
| Laravel | ✅ Done | Sanctum XSRF-TOKEN cookie |
| Rails | ✅ Done | `authenticity_token` form injection |
| WordPress | ✅ Done | Application Password / nonce |
| FastAPI | ✅ Done | Bearer token + optional basic |
| OAuth2 / PKCE (RFC 7636) | ✅ Done | `auth_oauth2_pkce()` — SHA-256 code challenge, token endpoint POST |
| AWS Signature v4 | ✅ Done | `AwsSigV4` AuthType in `http_engine.rs` — full HMAC-SHA256 signing chain |
| Digest auth | ✅ Done | `Digest` AuthType — RFC 7617 MD5 challenge-response |
| mTLS | ✅ Done | `client_cert_pem` / `client_key_pem` on `ParallaxRequest` |
| NTLM | ⚠️ Deferred | Windows-only; no cross-platform Rust crate |

### Schema Explorer (Rust)
| Task | Status | Notes |
|---|---|---|
| Frappe DocType JSON crawler | ✅ Done | `explore_frappe()` in `schema_explorer.rs` |
| Django `models.py` parser | ✅ Done | `explore_django()` |
| Laravel `$fillable` PHP parser | ✅ Done | `explore_laravel()` |
| Rails `schema.rb` crawler | ✅ Done | `explore_rails()` |
| OpenAPI 3.x YAML/JSON importer | ✅ Done | `explore_openapi()` — full path/entity extraction |
| FastAPI Pydantic BaseModel crawler | ✅ Done | `explore_fastapi()` — class + `@app.route` decorators |
| Express router route crawler | ✅ Done | `explore_express()` — JS/TS router patterns |
| Framework auto-detection on folder open | ✅ Done | `has_*_structure()` detection per framework |
| Generate collection from schema | ✅ Done | "Generate Collection" button in Ecosystem panel |

### Response Intelligence
| Task | Status | Notes |
|---|---|---|
| Export as JSON Schema | ✅ Done | `schema-inference.ts` `inferJsonSchema()` |
| Export as TypeScript interface | ✅ Done | `inferTypeScript()` |
| Export as Pydantic v2 BaseModel | ✅ Done | `inferPydantic()` — recursive nested classes |
| Export as Rust struct | ✅ Done | `inferRustStruct()` — `#[derive(Debug, Clone, Serialize, Deserialize)]` |
| Export as Go struct | ✅ Done | `inferGoStruct()` — `json:"..."` tags |
| Schema tab in response pane | ✅ Done | 5 format toggles in `ResponsePanel.svelte` |

### Visual Flow Builder
| Task | Status | Notes |
|---|---|---|
| SVG request dependency graph | ✅ Done | `FlowPanel.svelte` — cubic bezier edges, arrowheads |
| Variable dependency analysis | ✅ Done | `setsVars()` + `extractVarRefs()` — detects `pm.environment.set()` chains |
| Method-colored node cards | ✅ Done | Color-coded accent strip + badge per HTTP method |
| Folder grouping bands | ✅ Done | Shaded rect per folder group |
| Collection picker + stats bar | ✅ Done | |
| Wired into Dashboard | ✅ Done | "Flow Builder" nav item in `DashboardMode.svelte` |

### Enhanced Protocol Support
| Task | Status | Notes |
|---|---|---|
| gRPC unary + server streaming | ✅ Done | `GRPCPane.svelte` + `commands/grpc.rs` — HTTP/2 grpc+json framing |
| gRPC service reflection (proto descriptors) | ⚠️ Deferred | Requires protobuf descriptor pool; deferred |
| GraphQL schema explorer + autocomplete + builder | ✅ Done | Phase 1 — introspection + type browser |

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
| ~~Go-side proxy HTTP server not implemented~~ | ~~`LiveTrafficPanel` Rust commands exist but Go not wired~~ | ✅ Resolved — `proxy.go` + `ca.go` |
| ~~Go-side health goroutines not implemented~~ | ~~`HealthHeatmapPanel` Rust commands exist but Go not wired~~ | ✅ Resolved — `health.go` goroutines |
| ~~GraphQL schema introspection missing~~ | ~~GraphQL body works; no schema browser~~ | ✅ Resolved |
| ~~`parallax-cli` not started~~ | ~~No Newman equivalent for CI/CD pipelines~~ | ✅ Resolved — `run` / `mock` / `init` subcommands done |
| ~~Go runner missing template engine + test scripts~~ | ~~CLI runner can't resolve `{{vars}}` or run `pm.test()`~~ | ✅ Resolved — `runner.go` has `resolve()` + `pm.test()` via goja |
| ~~Mock server path params + templating not implemented~~ | ~~Basic rules only; `:id` routes don't work~~ | ✅ Resolved — `matchPath()` + `renderBody()` done |
| ~~Traffic filter + HAR export not wired to Tauri~~ | ~~Go+gRPC done; need Rust commands + UI~~ | ✅ Resolved — `set_proxy_filter` + domain filter UI |
| ~~`WatchFiles` gRPC stream not wired to Tauri~~ | ~~need Rust `watch_workspace` command~~ | ✅ Resolved — `watcher.rs` done |
| ~~`StreamRunner` not implemented~~ | ~~`RunCollection` gRPC is a stub~~ | ✅ Resolved — full streaming runner |
| ~~`parallax-cli --reporter html` and `--data` flags~~ | ~~CLI run works but no HTML report~~ | ✅ Resolved — both done |
| Mock record mode | Would auto-generate rules from proxy traffic | 🟢 P2 — deferred to post-2.5 |
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
| 2026-04-23 | Go runner uses `goja` JS engine for `pm.test()` in CLI | Same JS runtime as browser-side script runner; consistent test semantics across UI + CLI |
| 2026-04-23 | Mock response templating uses Go `text/template` not Handlebars | Zero deps; `{{.Params.id}}` and `{{.Query.key}}` pattern is simpler for server-side |
| 2026-04-23 | AI service in Go sidecar (not Rust) to keep heavy HTTP + JSON parsing off main thread | Go goroutines handle 60s AI timeouts gracefully; Rust stays low-latency for UI |
| 2026-04-23 | Mock delay stored as `x-parallax-delay-ms` header convention (not a proto field) | Avoids proto regeneration; Go strips it before sending response; UI treats it specially |
| 2026-04-23 | CLI `--data` uses simple split-on-comma CSV (no quoting support) | Sufficient for typical env-override data files; proper CSV parser can be added later if needed |
| 2026-04-23 | `RunCollectionStream` accepts `emit func(StreamEvent)` — nil for CLI, channel-based for gRPC | Single implementation serves both UI streaming (gRPC) and CLI text output without duplication |

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
