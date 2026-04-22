# Parallax — Development Tracker

Last updated: 2026-04-23

---

## Overall Progress

```
Phase 1  ███████████████████░  97%  🔄 Near Complete
Phase 2  ████░░░░░░░░░░░░░░░░  20%  🔄 In Progress (Dashboard stub + Proxy + Health wired)
Phase 3  ░░░░░░░░░░░░░░░░░░░░   0%  🔲 Planned
Phase 4  ░░░░░░░░░░░░░░░░░░░░   0%  🔲 Planned
Phase 5  ░░░░░░░░░░░░░░░░░░░░   0%  🔲 Planned
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
| Data file (CSV/JSON) for data-driven runs | 🔲 | |
| Variable chaining between requests in run | ✅ Done | `pm.environment.set()` in scripts persists |
| Live run feed UI | ✅ Done | Per-request pass/fail rows |
| Summary panel (passed/failed/time) | ✅ Done | |
| Report output (JSON + HTML) | 🔲 | |

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
| Load Test Results panel | 🔲 | |
| Git Sync Status panel | 🔲 | |
| Collection Run History panel | 🔲 | |

### Go Local Proxy (Rust commands wired to gRPC)
| Task | Status | Notes |
|---|---|---|
| `start_proxy_stream` / `get_proxy_traffic` / `clear_proxy_traffic` | ✅ Done | Rust ↔ gRPC wired |
| HTTP proxy server in Go (`localhost:8765`) | 🔲 | Go-side not yet implemented |
| HTTPS MITM with local CA cert | 🔲 | |
| Traffic filter by domain/method/status | 🔲 | |
| Export as HAR | 🔲 | |
| Replay captured request | 🔲 | |

### Health Monitor (Rust commands wired to gRPC)
| Task | Status | Notes |
|---|---|---|
| `start_health_stream` / `add/remove_health_target` / `get_health_statuses` | ✅ Done | Rust ↔ gRPC wired |
| Goroutine-per-service health checks in Go | 🔲 | Go-side not yet implemented |
| SQLite uptime history | 🔲 | |
| Desktop notifications on status change | 🔲 | |
| Alert webhook on failure | 🔲 | |

### Load Tester
| Task | Status | Notes |
|---|---|---|
| All load test features | 🔲 | |

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
| All mock server features | 🔲 | |

### gRPC Streaming Bridge
| Task | Status | Notes |
|---|---|---|
| `WatchTraffic` stream | ✅ Done | Rust command → gRPC |
| `WatchHealth` stream | ✅ Done | Rust command → gRPC |
| `WatchFiles` stream | 🔲 | |
| `StreamLoadTest` stream | 🔲 | |
| `StreamRunner` stream | 🔲 | |

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
| Go-side proxy HTTP server not implemented | `LiveTrafficPanel` Rust commands exist but Go not wired | 🟡 P1 |
| Go-side health goroutines not implemented | `HealthHeatmapPanel` Rust commands exist but Go not wired | 🟡 P1 |
| ~~GraphQL schema introspection missing~~ | ~~GraphQL body works; no schema browser~~ | ✅ Resolved |
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
