# Parallax — Development Tracker

Last updated: 2026-04-21

---

## Overall Progress

```
Phase 1  ████░░░░░░░░░░░░░░░░  20%  🔄 In Progress  (scaffold done, core features pending)
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
| Tauri v2 project initialized | ✅ Done | Manual scaffold — npm create was hanging |
| Svelte 5 frontend wired | ✅ Done | `src/` directory with components |
| Go sidecar directory created | ✅ Done | `src-go/` with watcher, proxy, loadtest, health stubs |
| gRPC proto definitions | ✅ Done | `proto/parallax.proto` |
| `.parallax/` example folder | ✅ Done | Sample YAML collection included |
| npm install passing | ✅ Done | `--legacy-peer-deps` flag needed |
| Tauri CLI v2 installed | ✅ Done | v2.10.1 |
| `cargo check` passing | ❌ Blocked | `generate_context!()` macro expansion error |
| `cargo tauri dev` launching | ❌ Blocked | Depends on cargo check |

### Protocols (HTTP Engine — Rust)
| Task | Status | Notes |
|---|---|---|
| REST — all methods (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS) | ✅ Done | Via `reqwest` |
| HTTP/2 support | 🔲 | `reqwest` default with `rustls` |
| HTTP/3 support | 🔲 | Via `hyper` + `quiche` or `h3` |
| GraphQL — query + variables + schema introspection | 🔲 | |
| gRPC — unary calls + service reflection | 🔲 | |
| gRPC — server/client/bidi streaming | 🔲 | |
| WebSocket — connect, send frames, stream events | 🔲 | `tokio-tungstenite` |
| SSE — Server-Sent Events streaming | 🔲 | |
| Proxy settings per environment | 🔲 | HTTP + SOCKS5 |
| SSL/TLS toggle per request | 🔲 | |
| Client certificates | 🔲 | |
| Follow redirects (configurable) | 🔲 | |
| Request timeout (configurable) | 🔲 | |

### Template Tag Engine (Insomnia-style)
| Task | Status | Notes |
|---|---|---|
| `{% uuid %}` | 🔲 | |
| `{% timestamp %}` | 🔲 | |
| `{% now 'iso' %}` | 🔲 | |
| `{% randomInt %}` | 🔲 | |
| `{% randomEmail %}`, `{% randomName %}`, `{% randomPhone %}` | 🔲 | |
| `{% base64 %}`, `{% hash %}` | 🔲 | |
| `{% response 'body', '$.path' %}` — request chaining | 🔲 | Key Insomnia feature |
| `{% response 'header', 'X-Token', 'request-name' %}` | 🔲 | |
| `{% env 'VAR' %}` — OS env var | 🔲 | |
| `{% file '/path' %}` | 🔲 | |
| `{% prompt 'label' %}` — ask user at send-time | 🔲 | |
| `{{$randomEmail}}` etc. — Postman `$` syntax compat | 🔲 | Postman compatibility |

### Variable Scoping (Postman 4-level system)
| Task | Status | Notes |
|---|---|---|
| Global variables | 🔲 | Shared across all projects |
| Collection variables | 🔲 | Per-collection YAML header |
| Environment variables | 🔲 | `.parallax/environments/*.json` |
| Local variables (ephemeral, set by scripts) | 🔲 | |
| Resolution priority: Local > Env > Collection > Global | 🔲 | |

### Script Runner (Postman `pm.*` API)
| Task | Status | Notes |
|---|---|---|
| Choose JS runtime (Deno core vs. QuickJS) | 🔲 | Decision pending |
| `pm.environment.get/set/unset` | 🔲 | |
| `pm.globals.get/set/unset` | 🔲 | |
| `pm.collectionVariables.get/set` | 🔲 | |
| `pm.request` — access/modify pre-send | 🔲 | |
| `pm.response` — access in test scripts | 🔲 | |
| `pm.test(name, fn)` — named assertions | 🔲 | |
| `pm.expect` — Chai-style assertions | 🔲 | |
| `pm.sendRequest(options, callback)` | 🔲 | |
| Script timeout (10s default) | 🔲 | |
| Python runtime stub (PyO3) | 🔲 | Full in Phase 2 |

### Persistence & Import/Export
| Task | Status | Notes |
|---|---|---|
| `load_collection` — reads YAML | ✅ Done | |
| `save_collection` — writes YAML | ✅ Done | |
| `list_environments` | ✅ Done | |
| `load_environment` with variable scoping | 🔲 | |
| `save_environment` | 🔲 | |
| Import from Postman Collection v2.1 JSON | 🔲 | Must-have (Insomnia has this) |
| Import from curl command | 🔲 | Must-have (both tools have this) |
| Import from OpenAPI 3.x (stub) | 🔲 | Full in Phase 4 |
| Import from HAR file | 🔲 | |
| Export as Postman JSON | 🔲 | For teams still on Postman |
| Response history — save to `.parallax/history/` | 🔲 | |

### Cookie Jar
| Task | Status | Notes |
|---|---|---|
| Cookie store (`.parallax/cookies/jar.json`) | 🔲 | |
| Cookie manager UI | 🔲 | |
| Per-request opt-in/opt-out | 🔲 | |
| Session cookies expire on restart | 🔲 | |

### Auth Providers (Phase 1 — basic set)
| Task | Status | Notes |
|---|---|---|
| Bearer token | ✅ Done | |
| Basic auth | 🔲 | |
| API key (header / query param) | 🔲 | |
| Frappe sid + CSRF (stub) | ✅ Done | Full in Phase 4 |
| Django CSRF (stub) | ✅ Done | Full in Phase 4 |

### Svelte 5 Builder Mode UI
| Task | Status | Notes |
|---|---|---|
| App shell / layout (Builder + Dashboard + Design modes) | ✅ Done | |
| Method selector | ✅ Done | |
| URL bar with template tag autocomplete | ✅ Done | |
| Params tab | 🔲 | |
| Headers tab (key-value + bulk edit) | ✅ Done | |
| Auth tab with provider selector | 🔲 | |
| Body tab — JSON, XML, form-data, URL-encoded, binary, GraphQL | ✅ Done | |
| Scripts tab — pre-request + test editor | 🔲 | |
| Settings tab — timeout, redirects, proxy, SSL | 🔲 | |
| GraphQL pane (query editor, variables, schema explorer) | 🔲 | |
| Response pane — status, time, size | ✅ Done | |
| Response body — JSON tree, XML, HTML, raw, hex | ✅ Done | |
| Response headers tab | ✅ Done | |
| Response cookies tab | 🔲 | |
| Response tests tab (pass/fail list) | 🔲 | |
| Response visualize tab (Postman Visualizer) | 🔲 | Phase 2 |
| Response history dropdown | 🔲 | |
| Collection sidebar tree view | ✅ Done | |
| Sidebar drag-and-drop reordering | 🔲 | |
| Sidebar right-click context menu | 🔲 | |
| Sidebar search/filter | 🔲 | |
| Sidebar method badges | 🔲 | |
| Environment selector dropdown | ✅ Done | |
| Environment quick-edit overlay | 🔲 | |
| Environment diff view | 🔲 | |
| Secret masking in env viewer | 🔲 | |

### Go Sidecar
| Task | Status | Notes |
|---|---|---|
| `src-go/main.go` gRPC server stub | ✅ Done | |
| File watcher stub | ✅ Done | |
| Health check ping endpoint | ✅ Done | |
| Go binary compiled and bundled | ❌ Blocked | Not compiled yet |
| IPC ping from Rust working | ❌ Blocked | Depends on binary |

### Phase 1 Success Criteria
| Criteria | Status |
|---|---|
| `cargo tauri dev` launches without errors | ❌ |
| REST, GraphQL, WebSocket requests work | ❌ |
| Template tags resolve at send-time | ❌ |
| Pre-request and test scripts execute with `pm` API | ❌ |
| Variable scoping resolves across 4 levels | ❌ |
| Cookie jar persists between requests | ❌ |
| Collections load from and save to `.parallax/` | ❌ |
| Import a Postman Collection JSON | ❌ |
| Import a curl command | ❌ |
| Response history saves and timeline works | ❌ |
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
| Dashboard Mode UI shell | 🔲 |
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

### Documentation Generator (Postman Cloud Docs — local)
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
| NTLM / Negotiate | 🔲 |
| mTLS (client certificate) | 🔲 |

### Schema Explorer (Go)
| Task | Status |
|---|---|
| Frappe DocType explorer | 🔲 |
| Frappe `@whitelist()` method scanner | 🔲 |
| Django URL + DRF ViewSet explorer | 🔲 |
| Laravel route explorer (`artisan route:list` + file parse) | 🔲 |
| Rails routes.rb + schema.rb explorer | 🔲 |
| FastAPI decorator scanner | 🔲 |
| Express.js / Fastify route scanner | 🔲 |
| OpenAPI 3.x full importer | 🔲 |
| Framework auto-detection on folder open | 🔲 |

### Design Mode (Insomnia OpenAPI editor — improved)
| Task | Status |
|---|---|
| Design Mode UI shell | 🔲 |
| YAML editor with syntax highlighting | 🔲 |
| Real-time OpenAPI validation + inline errors | 🔲 |
| Rendered docs preview (right pane) | 🔲 |
| OpenAPI keyword autocomplete | 🔲 |
| Schema builder UI (form-based) | 🔲 |
| "Try it out" — execute from spec preview | 🔲 |
| Sync spec → collection | 🔲 |
| Sync collection → spec | 🔲 |
| Spec lint (style rules) | 🔲 |
| Save as `.parallax/design/*.openapi.yaml` | 🔲 |

### Response Intelligence
| Task | Status |
|---|---|
| Response shape inference engine | 🔲 |
| Schema confidence tracking | 🔲 |
| Export as JSON Schema | 🔲 |
| Export as TypeScript interface | 🔲 |
| Export as Pydantic model | 🔲 |
| Export as Rust struct | 🔲 |
| Export as Go struct | 🔲 |
| Schema stored in `.parallax/schemas/` | 🔲 |

### Visual Flow Builder (Postman Flows equivalent)
| Task | Status |
|---|---|
| Canvas-based editor UI | 🔲 |
| Request node | 🔲 |
| Condition node (branch) | 🔲 |
| Transform node (extract/reshape) | 🔲 |
| Loop node (iterate over array) | 🔲 |
| Delay node | 🔲 |
| Variable node | 🔲 |
| Flow execution via Collection Runner | 🔲 |
| Save as `.parallax/flows/*.yaml` | 🔲 |

### Enhanced Protocol Support
| Task | Status |
|---|---|
| gRPC service reflection (auto-discover methods) | 🔲 |
| GraphQL schema explorer (full type browser) | 🔲 |
| GraphQL field autocomplete | 🔲 |
| GraphQL query builder (click to build) | 🔲 |
| GraphQL subscription support | 🔲 |
| GraphQL schema diff | 🔲 |
| GraphQL persisted queries | 🔲 |

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
| Plugin sandbox (no direct fs/network access) | 🔲 |
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
| Multi-tab UI | 🔲 |
| Tabs persist across restarts | 🔲 |
| Split view (two requests side-by-side) | 🔲 |
| Detach tab to separate window | 🔲 |
| Pin tab | 🔲 |

### Themes
| Task | Status |
|---|---|
| Parallax Dark (default) finalized | 🔲 |
| Parallax Light | 🔲 |
| High Contrast Dark | 🔲 |
| High Contrast Light | 🔲 |
| Monokai | 🔲 |
| Solarized Dark/Light | 🔲 |
| Custom CSS override (`theme.css`) | 🔲 |
| Full CSS custom property exposure | 🔲 |

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
| `cargo check` fails — `generate_context!()` macro expansion error | Blocks app launch | 🔴 P0 |
| Go binary not compiled | Blocks IPC, sidecar, CLI features | 🟡 P1 |
| JS runtime not chosen (Deno core vs. QuickJS) | Blocks script runner, template tags | 🟡 P1 |
| Icon files are placeholder PNGs | Cosmetic only | 🟢 P2 |

---

## Decision Log

| Date | Decision | Reason |
|---|---|---|
| 2026-04-21 | Manual scaffold instead of `npm create tauri-app` | CLI command was hanging; manual gives full control |
| 2026-04-21 | `externalBin` removed from tauri.conf.json during dev | Go binary not compiled yet — re-add when sidecar is ready |
| 2026-04-21 | Frappe auth → Framework-agnostic auth provider system | All users need auth handling, not just Frappe users |
| 2026-04-21 | `--legacy-peer-deps` for npm install | Peer dep conflict between Svelte 5 and some dev tools |
| 2026-04-21 | Include ALL Postman + Insomnia must-haves, not just gaps | Parallax is not a "better Postman" — it is both tools unified |

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
