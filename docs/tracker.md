# Parallax — Development Tracker

Last updated: 2026-04-22

---

## Overall Progress

```
Phase 1  ████████████░░░░░░░░  60%  🔄 In Progress
Phase 2  ░░░░░░░░░░░░░░░░░░░░   0%  🔲 Planned
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
| Svelte 5 frontend wired | ✅ Done | `src/` with full component tree |
| Go sidecar directory created | ✅ Done | `src-go/` with gRPC stubs |
| gRPC proto definitions | ✅ Done | `proto/parallax.proto` |
| `.parallax/` example folder | ✅ Done | Sample YAML collection included |
| npm install passing | ✅ Done | `--legacy-peer-deps` |
| Tauri CLI v2 installed | ✅ Done | v2.10.1 |
| `cargo check` passing | ✅ Done | Zero errors (warnings only) |
| `cargo tauri dev` launching | ✅ Done | App runs; Go sidecar on gRPC :50151 |
| Go binary compiled and bundled | ✅ Done | `parallax-worker-aarch64-apple-darwin` |
| Tauri shell plugin config fixed | ✅ Done | Removed invalid `all`/`execute`/`sidecar` fields |
| `.gitignore` with `.claude` exclusion | ✅ Done | Sensitive `.parallax/` paths also gitignored |

### Protocols (HTTP Engine — Rust)
| Task | Status | Notes |
|---|---|---|
| REST — all methods (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS) | ✅ Done | Via `reqwest` |
| HTTP/2 support | ✅ Done | `reqwest` default with `rustls` |
| HTTP/3 support | 🔲 | Via `hyper` + `quiche` |
| GraphQL — query + variables | ✅ Done | Body type wired; schema introspection pending |
| GraphQL schema introspection | 🔲 | |
| gRPC — unary calls | 🔲 | |
| gRPC — streaming | 🔲 | |
| WebSocket — connect, send frames, stream | 🔲 | `tokio-tungstenite` |
| SSE — Server-Sent Events streaming | 🔲 | |
| Proxy settings per environment | 🔲 | |
| SSL/TLS toggle per request | 🔲 | |
| Client certificates | 🔲 | |
| Follow redirects (configurable) | ✅ Done | `follow_redirects: true` in payload |
| Request timeout (configurable) | ✅ Done | `timeout_ms: 30000` in payload |

### Template Tag Engine (Insomnia-style)
| Task | Status | Notes |
|---|---|---|
| `{% uuid %}` / `{% guid %}` | ✅ Done | `template-tags.ts` |
| `{% timestamp %}` | ✅ Done | |
| `{% now 'iso' %}` / `'unix'` / `'ms'` | ✅ Done | |
| `{% randomInt min max %}` | ✅ Done | |
| `{% randomEmail %}` | ✅ Done | |
| `{% randomName %}` | ✅ Done | |
| `{% randomPhone %}` | ✅ Done | |
| `{% randomWord %}` | ✅ Done | |
| `{% randomBoolean %}` | ✅ Done | |
| `{% randomLoremIpsum %}` | ✅ Done | |
| `{% base64 encode/decode val %}` | ✅ Done | |
| `{% env 'VAR' %}` | ✅ Done | |
| `{% response 'body', '$.path' %}` — request chaining | 🔲 | Key Insomnia feature |
| `{% file '/path' %}` | 🔲 | |
| `{% prompt 'label' %}` | 🔲 | |
| `{{$guid}}`, `{{$timestamp}}`, `{{$isoTimestamp}}` — Postman `$` compat | ✅ Done | `resolvePostmanDynamic()` |
| `{{$randomEmail}}`, `{{$randomBoolean}}`, `{{$randomFullName}}` | ✅ Done | |
| `{{$randomPhoneNumber}}`, `{{$randomWord}}` | ✅ Done | |
| `{{var}}` — environment variable substitution | ✅ Done | |
| Template resolution wired into `sendRequest()` | ✅ Done | `resolveRequestTemplates()` called on full payload |

### Variable Scoping (Postman 4-level system)
| Task | Status | Notes |
|---|---|---|
| Environment variables (`.parallax/environments/*.json`) | ✅ Done | Load/save/edit working |
| Local variables (ephemeral, set by scripts) | ✅ Done | `pm.environment.set()` in script runner |
| Global variables | 🔲 | Shared across all projects |
| Collection variables | 🔲 | Per-collection YAML header |
| Resolution priority: Local > Env > Collection > Global | 🔲 | Full 4-level pending |

### Script Runner (Postman `pm.*` API)
| Task | Status | Notes |
|---|---|---|
| JS runtime — sandboxed `new Function()` | ✅ Done | No native dep needed; safe for local tool |
| `pm.environment.get/set/unset/has` | ✅ Done | `pm-script-runner.ts` |
| `pm.globals.get/set/unset` | ✅ Done | Aliased to environment for now |
| `pm.response.code/status/responseTime/responseSize` | ✅ Done | |
| `pm.response.json()` / `pm.response.text()` | ✅ Done | |
| `pm.response.headers.get(name)` | ✅ Done | |
| `pm.test(name, fn)` — named assertions | ✅ Done | |
| `pm.expect(val).to.equal/eql` | ✅ Done | |
| `pm.expect(val).to.be.ok/above/below/a/string/number` | ✅ Done | |
| `pm.expect(val).to.include/have.property/have.length` | ✅ Done | |
| `pm.expect(val).to.have.status/header` | ✅ Done | |
| `pm.expect(val).not.equal/include` | ✅ Done | |
| Pre-request script runs before HTTP call | ✅ Done | |
| Test script runs after response received | ✅ Done | |
| `pm.sendRequest()` | 🔲 | |
| Python runtime (PyO3) | 🔲 | Phase 2 |

### Persistence & Import/Export
| Task | Status | Notes |
|---|---|---|
| `list_collections` — YAML | ✅ Done | |
| `load_collection` — YAML | ✅ Done | |
| `save_collection` — YAML | ✅ Done | |
| `delete_collection` | ✅ Done | |
| `list_environments` | ✅ Done | |
| `load_environment` | ✅ Done | |
| `save_environment` | ✅ Done | |
| `save_history_entry` — JSON to `.parallax/history/` | ✅ Done | Per-request timestamped files |
| Response history in-memory (capped 200) | ✅ Done | `responseHistory` store |
| Import from Postman Collection v2.1 JSON | ✅ Done | `postman-importer.ts` — requests, folders, auth, body |
| Import from Insomnia v4 export JSON | ✅ Done | `importInsomniaExport()` |
| Import from curl command (URL bar paste) | ✅ Done | `parseCurl()` — method, headers, body, Bearer |
| Import from OpenAPI 3.x | 🔲 | Phase 4 |
| Import from HAR file | 🔲 | |
| Export as Postman JSON | 🔲 | |

### Cookie Jar
| Task | Status | Notes |
|---|---|---|
| Cookie store (`reqwest` `cookie_store(true)`) | ✅ Done | Rust engine has it enabled |
| Cookie manager UI | 🔲 | |
| Per-request opt-in/opt-out | 🔲 | |
| Session cookies expire on restart | 🔲 | |

### Auth Providers (Phase 1 — basic set)
| Task | Status | Notes |
|---|---|---|
| Bearer token | ✅ Done | |
| Basic auth | ✅ Done | |
| API key (header) | ✅ Done | |
| API key (query param) | 🔲 | |
| Ecosystem provider selector UI | ✅ Done | Frappe, Django, Laravel, Rails, WordPress, FastAPI |
| Frappe sid + CSRF (stub) | ✅ Done | Full in Phase 4 |
| Django CSRF (stub) | ✅ Done | Full in Phase 4 |
| OAuth2 (full) | 🔲 | Phase 4 |
| AWS Signature v4 | 🔲 | Phase 4 |

### Svelte 5 Builder Mode UI
| Task | Status | Notes |
|---|---|---|
| App shell / layout (Builder + Dashboard + Design modes) | ✅ Done | |
| Method selector | ✅ Done | |
| URL bar | ✅ Done | |
| Params tab (key-value editor) | ✅ Done | |
| Headers tab (key-value editor) | ✅ Done | |
| Auth tab with all provider sub-forms | ✅ Done | Bearer/Basic/API Key/Ecosystem |
| Body tab — JSON, form-data, URL-encoded, raw, GraphQL | ✅ Done | |
| Scripts tab — Pre-request editor | ✅ Done | Real textarea with syntax hint |
| Scripts tab — Tests editor | ✅ Done | Real textarea with syntax hint |
| Response pane — status, time, size | ✅ Done | |
| Response body — JSON tree (colorized), raw | ✅ Done | |
| Response headers tab | ✅ Done | |
| Response tests tab (pass/fail per `pm.test()`) | ✅ Done | |
| Response history tab (scrollable, click to view) | ✅ Done | |
| Response cookies tab | 🔲 | |
| Response visualize tab (Postman Visualizer) | 🔲 | Phase 2 |
| Collection sidebar — tree view (collections → folders → requests) | ✅ Done | |
| Sidebar — search/filter | ✅ Done | Real-time `$derived.by()` filter |
| Sidebar — method badges (color-coded) | ✅ Done | |
| Sidebar — import button (Postman / Insomnia) | ✅ Done | File picker, auto-detects format |
| Sidebar — git branch chip | ✅ Done | Shows current branch from workspace |
| Sidebar drag-and-drop reordering | 🔲 | |
| Sidebar right-click context menu | 🔲 | |
| Environment quick-edit overlay | ✅ Done | `EnvironmentPanel.svelte` |
| Environment variable count badge | ✅ Done | |
| Secret masking in env editor | ✅ Done | Per-variable eye toggle |
| Environment diff view | 🔲 | |
| Multi-tab UI | ✅ Done | Tabs store, create/switch/close |
| Tabs persist across restarts | 🔲 | |
| Split view | 🔲 | |

### Go Sidecar
| Task | Status | Notes |
|---|---|---|
| `src-go/main.go` gRPC server stub | ✅ Done | |
| File watcher stub | ✅ Done | |
| Health check ping endpoint | ✅ Done | |
| Go binary compiled and bundled | ✅ Done | Running on :50151 |
| IPC ping from Rust working | 🔲 | sidecar starts but ping not wired yet |

### Phase 1 Success Criteria
| Criteria | Status |
|---|---|
| `cargo tauri dev` launches without errors | ✅ |
| REST requests work end-to-end | ✅ |
| Template tags resolve at send-time | ✅ |
| Pre-request and test scripts execute with `pm` API | ✅ |
| Variable scoping (env + local via scripts) | ✅ |
| Cookie jar active in reqwest | ✅ (no UI yet) |
| Collections load from and save to `.parallax/` | ✅ |
| Import a Postman Collection JSON | ✅ |
| Import from Insomnia export | ✅ |
| Import a curl command | ✅ |
| Response history saves in-memory + to disk | ✅ |
| GraphQL, WebSocket requests work | ❌ (GraphQL body only; no WS/SSE yet) |
| Variable scoping resolves across all 4 levels | ❌ (2/4 levels done) |
| Cookie jar has management UI | ❌ |
| Go sidecar responds to ping | ❌ |

---

## Phase 2 — Dashboard, Collection Runner & CLI

> Not started. Waiting on Phase 1 completion.

### Collection Runner (Postman core feature)
| Task | Status |
|---|---|
| Collection / folder selection | 🔲 |
| Environment selection for run | 🔲 |
| Iteration count | 🔲 |
| Delay between requests | 🔲 |
| Stop on first failure toggle | 🔲 |
| Data file (CSV/JSON) for data-driven runs | 🔲 |
| Variable chaining between requests in run | 🔲 |
| Live run feed UI | 🔲 |
| Summary panel (passed/failed/time) | 🔲 |
| Report output (JSON + HTML) | 🔲 |

### `parallax-cli` (Newman equivalent, Go)
| Task | Status |
|---|---|
| `parallax-cli run` command | 🔲 |
| `parallax-cli validate` command | 🔲 |
| `parallax-cli list` command | 🔲 |
| `parallax-cli import` command | 🔲 |
| `parallax-cli export` command | 🔲 |
| `parallax-cli mock` command | 🔲 |
| Console reporter | 🔲 |
| JSON reporter | 🔲 |
| HTML reporter | 🔲 |
| JUnit XML reporter (for CI) | 🔲 |
| Exit codes (0=pass, 1=fail, 2=error) | 🔲 |

### Dashboard Mode
| Task | Status |
|---|---|
| Dashboard Mode UI shell | ✅ Done (stub) |
| Live Traffic Stream panel | 🔲 |
| Health Heatmap panel | 🔲 |
| Load Test Results panel | 🔲 |
| Git Sync Status panel | 🔲 |
| Collection Run History panel | 🔲 |

### Go Local Proxy
| Task | Status |
|---|---|
| HTTP proxy server (`localhost:8765`) | 🔲 |
| HTTPS MITM with local CA cert | 🔲 |
| Traffic stream to Rust via gRPC | 🔲 |
| Filter by domain/path/method/status | 🔲 |
| Export captured traffic as HAR | 🔲 |
| Replay captured request | 🔲 |

### Health Monitor
| Task | Status |
|---|---|
| Health config (`.parallax/health.yaml`) | 🔲 |
| Goroutine-per-service background checks | 🔲 |
| SQLite uptime history (`.parallax/health.db`) | 🔲 |
| Desktop notifications on status change | 🔲 |
| Alert webhook on failure | 🔲 |
| Historical uptime charts | 🔲 |
| TCP port-only check type | 🔲 |

### Load Tester
| Task | Status |
|---|---|
| Go goroutine load engine | 🔲 |
| Concurrent users control (1–1000) | 🔲 |
| Duration + ramp-up settings | 🔲 |
| Think time between requests | 🔲 |
| Real-time metrics stream (500ms) | 🔲 |
| p50/p90/p95/p99 latency calculation | 🔲 |
| Load test report JSON + HTML | 🔲 |
| Real-time chart in Dashboard | 🔲 |

### Response Visualization (Postman Visualizer)
| Task | Status |
|---|---|
| Visualize tab in response pane | 🔲 |
| Handlebars template renderer | 🔲 |
| Sandboxed iframe | 🔲 |
| `pm.visualizer.set(template, data)` API | 🔲 |

### Mock Server (Local — no cloud)
| Task | Status |
|---|---|
| Mock definition format in collection YAML | 🔲 |
| Rust HTTP mock server (`axum` or `tiny_http`) | 🔲 |
| Path parameters + wildcards | 🔲 |
| Configurable response delay | 🔲 |
| Response templating with request data | 🔲 |
| Record mode (proxy → auto-generate mocks) | 🔲 |
| `parallax-cli mock` command | 🔲 |

### gRPC Streaming Bridge
| Task | Status |
|---|---|
| `WatchFiles` stream | 🔲 |
| `WatchTraffic` stream | 🔲 |
| `WatchHealth` stream | 🔲 |
| `StreamLoadTest` stream | 🔲 |
| `StreamRunner` stream | 🔲 |

---

## Phase 3 — AI Integration & MCP Server

> Not started. Waiting on Phase 2 completion.

### BYO-AI Providers
| Task | Status |
|---|---|
| OpenAI provider | 🔲 |
| Anthropic (Claude) provider | 🔲 |
| Ollama (local) provider | 🔲 |
| Custom OpenAI-compatible provider | 🔲 |
| Google Gemini provider | 🔲 |
| AI settings UI + `ai.json` config | 🔲 |
| Air-gap mode (disable all AI) | 🔲 |
| Usage transparency (provider/model/tokens shown) | 🔲 |

### AI Features
| Task | Status |
|---|---|
| AI test generator (response → assertions) | 🔲 |
| `pm.test()` format output from AI | 🔲 |
| AI request repair (4xx/5xx → fix suggestions) | 🔲 |
| AI collection creator (natural language → collection) | 🔲 |
| AI script assistant (autocomplete + explain + fix) | 🔲 |
| AI env variable suggestion (detect missing vars) | 🔲 |

### MCP Server
| Task | Status |
|---|---|
| MCP HTTP server (`localhost:7676`) | 🔲 |
| `parallax.list_collections()` | 🔲 |
| `parallax.get_collection(name)` | 🔲 |
| `parallax.get_request(collection, id)` | 🔲 |
| `parallax.execute_request(...)` | 🔲 |
| `parallax.run_collection(...)` | 🔲 |
| `parallax.create_request(...)` | 🔲 |
| `parallax.list_environments()` | 🔲 |
| `parallax.set_env_variable(...)` | 🔲 |
| `parallax.generate_tests(...)` | 🔲 |
| `parallax.start_mock(...)` | 🔲 |
| Local auth token for MCP clients | 🔲 |

### Documentation Generator
| Task | Status |
|---|---|
| Static HTML doc site generator | 🔲 |
| Markdown export | 🔲 |
| OpenAPI 3.x reverse-generation from collection | 🔲 |
| Searchable endpoint list | 🔲 |
| `parallax-cli serve-docs` command | 🔲 |

---

## Phase 4 — Ecosystem Intelligence

> Not started. Waiting on Phase 3 completion.

### Auth Providers (Full Set)
| Task | Status |
|---|---|
| Auth provider trait (Rust) | 🔲 |
| Frappe / ERPNext (full) | 🔲 |
| Django (CSRF + DRF Token + Session) | 🔲 |
| Laravel / Sanctum | 🔲 |
| Ruby on Rails / Devise | 🔲 |
| WordPress REST API | 🔲 |
| Next.js / NextAuth | 🔲 |
| FastAPI / Starlette | 🔲 |
| ASP.NET Core | 🔲 |
| Generic OAuth2 (code + PKCE + client_credentials + password) | 🔲 |
| AWS Signature v4 | 🔲 |
| Digest Auth | 🔲 |
| NTLLM / Negotiate | 🔲 |
| mTLS (client certificate) | 🔲 |

### Schema Explorer (Go)
| Task | Status |
|---|---|
| Frappe DocType explorer | 🔲 |
| Frappe `@whitelist()` method scanner | 🔲 |
| Django URL + DRF ViewSet explorer | 🔲 |
| Laravel route explorer | 🔲 |
| Rails routes.rb + schema.rb explorer | 🔲 |
| FastAPI decorator scanner | 🔲 |
| Express.js / Fastify route scanner | 🔲 |
| OpenAPI 3.x full importer | 🔲 |
| Framework auto-detection on folder open | 🔲 |

### Design Mode (OpenAPI Editor)
| Task | Status |
|---|---|
| Design Mode UI shell | 🔲 |
| YAML editor with syntax highlighting | 🔲 |
| Real-time OpenAPI validation + inline errors | 🔲 |
| Rendered docs preview (right pane) | 🔲 |
| OpenAPI keyword autocomplete | 🔲 |
| Schema builder UI (form-based) | 🔲 |
| "Try it out" — execute from spec | 🔲 |
| Sync spec → collection | 🔲 |
| Sync collection → spec | 🔲 |
| Spec lint (style rules) | 🔲 |

### Response Intelligence
| Task | Status |
|---|---|
| Response shape inference engine | 🔲 |
| Export as JSON Schema | 🔲 |
| Export as TypeScript interface | 🔲 |
| Export as Pydantic model | 🔲 |
| Export as Rust struct | 🔲 |
| Export as Go struct | 🔲 |

### Visual Flow Builder
| Task | Status |
|---|---|
| Canvas-based editor UI | 🔲 |
| Request node | 🔲 |
| Condition node | 🔲 |
| Transform node | 🔲 |
| Loop node | 🔲 |
| Delay node | 🔲 |
| Variable node | 🔲 |
| Flow execution via Collection Runner | 🔲 |

### Enhanced Protocol Support
| Task | Status |
|---|---|
| gRPC service reflection | 🔲 |
| GraphQL schema explorer (full type browser) | 🔲 |
| GraphQL field autocomplete | 🔲 |
| GraphQL query builder | 🔲 |
| GraphQL subscription support | 🔲 |
| GraphQL schema diff | 🔲 |

---

## Phase 5 — Polish, Performance & Release

> Not started. Waiting on Phase 4 completion.

### Performance
| Task | Status |
|---|---|
| Startup time < 800ms | 🔲 |
| RAM idle < 80MB | 🔲 |
| RAM under load test < 250MB | 🔲 |
| Request overhead < 5ms vs curl | 🔲 |
| Collection load 1000 requests < 200ms | 🔲 |
| Lazy-load Dashboard + Design mode | 🔲 |
| reqwest connection pooling per env | 🔲 |

### Plugin System
| Task | Status |
|---|---|
| Plugin API (`onRequest`, `onResponse`, etc.) | 🔲 |
| Plugin loader (from `~/.config/parallax/plugins/`) | 🔲 |
| Plugin sandbox | 🔲 |
| `parallax-cli plugin install` | 🔲 |
| Plugin registry site | 🔲 |
| parallax-plugin-faker | 🔲 |
| parallax-plugin-jwt | 🔲 |
| parallax-plugin-aws-sigv4 | 🔲 |
| parallax-plugin-base64 | 🔲 |
| parallax-plugin-xml | 🔲 |
| parallax-plugin-soap | 🔲 |

### Code Generation (8+ languages)
| Task | Status |
|---|---|
| curl | 🔲 |
| Python httpx | 🔲 |
| Python requests | 🔲 |
| JavaScript fetch | 🔲 |
| JavaScript axios | 🔲 |
| Rust reqwest | 🔲 |
| Go net/http | 🔲 |
| PHP Guzzle | 🔲 |
| Ruby Net::HTTP | 🔲 |
| Java OkHttp | 🔲 |
| C# HttpClient | 🔲 |
| Swift URLSession | 🔲 |
| Kotlin OkHttp | 🔲 |

### Keyboard & Command Palette
| Task | Status |
|---|---|
| Command palette (Cmd+K / Ctrl+K) | 🔲 |
| All actions discoverable via palette | 🔲 |
| Full shortcut table implemented | 🔲 |
| Shortcut customization | 🔲 |

### Multi-window & Tabs
| Task | Status |
|---|---|
| Multi-tab UI | ✅ Done (Phase 1) |
| Tabs persist across restarts | 🔲 |
| Split view (two requests side-by-side) | 🔲 |
| Detach tab to separate window | 🔲 |
| Pin tab | 🔲 |

### Themes
| Task | Status |
|---|---|
| Parallax Dark (default) | ✅ Done (Phase 1) |
| Parallax Light | 🔲 |
| High Contrast Dark/Light | 🔲 |
| Monokai | 🔲 |
| Solarized Dark/Light | 🔲 |
| Custom CSS override (`theme.css`) | 🔲 |

### Distribution
| Task | Status |
|---|---|
| macOS `.dmg` (arm64 + x86_64 universal) | 🔲 |
| Windows `.msi` + `.exe` | 🔲 |
| Linux `.AppImage` | 🔲 |
| Linux `.deb` | 🔲 |
| Linux `.rpm` | 🔲 |
| macOS code signing + notarization | 🔲 |
| Windows code signing | 🔲 |
| GitHub Actions CI/CD pipeline | 🔲 |
| Auto-updater end-to-end | 🔲 |

### Documentation
| Task | Status |
|---|---|
| Documentation site live | 🔲 |
| Getting Started / quickstart | 🔲 |
| Builder Mode guide | 🔲 |
| Dashboard Mode guide | 🔲 |
| Design Mode guide | 🔲 |
| Collections YAML schema reference | 🔲 |
| Scripts / `pm.*` API reference | 🔲 |
| Auth Providers reference | 🔲 |
| Schema Explorer guide | 🔲 |
| AI Integration guide | 🔲 |
| Plugin API reference | 🔲 |
| `parallax-cli` full command reference | 🔲 |
| Keyboard shortcuts cheat sheet | 🔲 |
| README with 5-min quickstart | 🔲 |

---

## Current Blockers

| Blocker | Impact | Priority |
|---|---|---|
| WebSocket / SSE not implemented in Rust engine | Blocks WS/SSE request types | 🟡 P1 |
| Cookie manager UI missing | Cookies work in reqwest but not inspectable | 🟡 P1 |
| Global + collection variable scopes not implemented | Only env + local vars work | 🟡 P1 |
| Go sidecar IPC ping not wired from Rust | Sidecar starts but no handshake | 🟢 P2 |
| Icon files are placeholder PNGs | Cosmetic only | 🟢 P2 |

---

## Decision Log

| Date | Decision | Reason |
|---|---|---|
| 2026-04-21 | Manual scaffold instead of `npm create tauri-app` | CLI was hanging; manual gives full control |
| 2026-04-21 | Frappe auth → Framework-agnostic auth provider system | All users need auth handling, not just Frappe users |
| 2026-04-21 | `--legacy-peer-deps` for npm install | Peer dep conflict between Svelte 5 and some dev tools |
| 2026-04-21 | Include ALL Postman + Insomnia must-haves, not just gaps | Parallax is both tools unified, not a "better Postman" |
| 2026-04-21 | Tauri shell plugin `all`/`execute`/`sidecar` fields removed | Tauri v2 only supports `open` in `plugins.shell` |
| 2026-04-21 | Go sidecar non-fatal on startup failure | Allows `cargo tauri dev` to work before binary is compiled |
| 2026-04-22 | Script runner uses `new Function()` sandbox | No native dep; sufficient for local trusted tool; revisit if multi-user |
| 2026-04-22 | Postman + Insomnia import in TypeScript (not Rust) | Runs in renderer, no IPC round-trip needed for parsing |
| 2026-04-22 | Response history capped at 200 in-memory entries | Prevents memory growth; disk persistence via `save_history_entry` |

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
| 🔴 P0 | Critical — blocks everything |
