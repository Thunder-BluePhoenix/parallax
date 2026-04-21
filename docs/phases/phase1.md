# Phase 1 — Architecture & Git-Native Core

**Status:** 🔄 In Progress
**Goal:** Establish the foundation. Every core feature that both Postman and Insomnia ship as a baseline must be wired up or stubbed by the end of this phase. Builder Mode must be fully functional.

---

## Objectives

1. Scaffold full project structure (Tauri v2 + Svelte 5 + Go sidecar).
2. Define and implement the `.parallax/` folder schema.
3. Build a working HTTP engine in Rust: REST, GraphQL, gRPC, WebSocket, SSE.
4. Implement the template tag engine (Insomnia-style dynamic variables + chaining).
5. Implement variable scoping (Postman-style: Global > Collection > Environment > Local).
6. Implement pre-request and test scripts (JS runtime in Rust via QuickJS or Deno).
7. Implement cookie jar.
8. Build Builder Mode UI with multi-pane layout (request + response always visible).
9. Implement response history per request.
10. Set up Go sidecar and confirm gRPC IPC is functional.

---

## Deliverables

### 1. Project Scaffold
- [x] Tauri v2 project initialized
- [x] Svelte 5 frontend wired to Tauri
- [x] Go sidecar directory (`src-go/`) created with watcher, proxy, loadtest, health stubs
- [x] gRPC proto definitions (`proto/parallax.proto`)
- [x] `.parallax/` example folder with sample YAML collection

### 2. `.parallax/` Schema (Git-Native Persistence)

Every API project stores its data as human-readable, diff-friendly files committed alongside code.

```
.parallax/
├── collections/
│   └── user-api.yaml          ← Requests and folders, YAML
├── environments/
│   ├── dev.json               ← Committed — non-sensitive values
│   └── prod.json              ← Gitignored by default — secrets
├── scripts/
│   └── setup-auth.js          ← Shared pre-request scripts
├── mocks/
│   └── users-mock.yaml        ← Local mock server definitions
├── history/
│   └── get-users/             ← Per-request history of past responses
│       ├── 2026-04-21T10:00.json
│       └── 2026-04-21T11:30.json
└── cookies/
    └── jar.json               ← Cookie store (gitignored by default)
```

**Full collection YAML format:**
```yaml
name: User API
description: User management endpoints
variables:
  base_path: /api/v1
folders:
  - name: Auth
    requests:
      - id: login
        name: Login
        method: POST
        url: "{{base_url}}/auth/login"
        body:
          type: json
          content: |
            {"email": "{{email}}", "password": "{{password}}"}
        scripts:
          pre_request: |
            console.log("Sending login request");
          test: |
            pm.test("Status 200", () => pm.response.to.have.status(200));
            pm.environment.set("token", pm.response.json().token);
requests:
  - id: get-users
    name: Get All Users
    method: GET
    url: "{{base_url}}{{base_path}}/users"
    headers:
      Authorization: "Bearer {{token}}"
      X-Request-ID: "{% uuid %}"
    tests:
      - assert: response.status == 200
      - assert: response.body.data != null
```

### 3. Rust HTTP Engine

Location: `src-tauri/src/commands/`

#### Protocols
- [x] REST — all methods (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD)
- [ ] GraphQL — query editor, variables panel, schema introspection (`__schema` query)
- [ ] gRPC — service reflection, unary calls, server/client/bidi streaming
- [ ] WebSocket — connect, send text/binary frames, view incoming stream
- [ ] SSE — connect and stream events in real time

#### Request Features
- [x] `send_request` Tauri command via `reqwest`
- [ ] HTTP/2 support (reqwest default with `rustls`)
- [ ] HTTP/3 support (via `hyper` + `quiche` or `h3`)
- [ ] Proxy settings per environment (HTTP proxy, SOCKS5)
- [x] Auth provider module — CSRF, Bearer, API key injection
- [ ] Cookie jar — persist cookies across requests per domain
- [ ] Follow redirects (configurable, 0–20 hops)
- [ ] SSL/TLS — verify on by default, toggle off per request
- [ ] Client certificates per environment
- [ ] Request timeout (configurable)

#### Template Tag Engine
Insomnia-style dynamic values. These resolve at send-time.

| Tag | Output |
|---|---|
| `{% uuid %}` | A random UUID v4 |
| `{% timestamp %}` | Unix timestamp (ms) |
| `{% now 'iso' %}` | ISO 8601 datetime string |
| `{% randomInt 1 100 %}` | Random integer |
| `{% randomEmail %}` | Fake email address |
| `{% randomName %}` | Fake full name |
| `{% randomPhone %}` | Fake phone number |
| `{% base64 'value' %}` | Base64-encode a string |
| `{% hash 'sha256' 'value' %}` | Hash a string |
| `{% response 'body', '$.token' %}` | Extract JSONPath from a previous response |
| `{% response 'header', 'X-Auth-Token', 'login' %}` | Get header from named request |
| `{% env 'BASE_URL' %}` | Read OS environment variable |
| `{% file '/path/to/file' %}` | Read file contents |
| `{% prompt 'Enter OTP' %}` | Ask user at send-time |

#### Variable Scoping (Postman-style 4 levels)
Priority (highest → lowest): Local > Environment > Collection > Global

```
Global vars      — shared across all projects
Collection vars  — set in the collection YAML header
Environment vars — from .parallax/environments/*.json
Local vars       — set by test scripts, ephemeral
```

#### Script Runner (Pre-request + Test)
- [ ] JavaScript runtime: embed `deno_core` or `quickjs-rs` in Tauri
- [ ] `pm` compatibility object — Postman-compatible API surface:
  - `pm.environment.get/set/unset`
  - `pm.globals.get/set/unset`
  - `pm.collectionVariables.get/set`
  - `pm.request` — access and modify request before send
  - `pm.response` — access response in test scripts
  - `pm.test(name, fn)` — register a named assertion
  - `pm.expect` — Chai-style assertions
  - `pm.sendRequest(options, callback)` — fire a sub-request
- [ ] Python runtime: PyO3 (Phase 2 addition — stubbed here)
- [ ] Script timeout: 10s default, configurable

#### Dynamic Variables (Postman `{{$...}}` style)
Available as template tags AND as `{{$randomEmail}}` syntax for Postman compatibility:
`$randomEmail`, `$guid`, `$timestamp`, `$isoTimestamp`, `$randomInt`, `$randomBoolean`, `$randomWord`, `$randomLoremIpsum`

#### Persistence Commands
- [x] `load_collection` — reads `.yaml` from disk
- [x] `save_collection` — writes `.yaml` to disk
- [x] `list_environments` — lists env files
- [ ] `load_environment` — parses and resolves env with variable scoping
- [ ] `save_environment` — writes env changes back
- [ ] `import_postman` — convert Postman Collection v2.1 JSON → Parallax YAML
- [ ] `import_curl` — parse a curl command → request definition
- [ ] `import_openapi` — parse OpenAPI 3.x YAML/JSON → collection stubs
- [ ] `import_har` — parse HAR file → requests
- [ ] `save_history_entry` — write response snapshot to `.parallax/history/`

### 4. Svelte 5 Builder Mode UI

The core experience. No tabs between request config areas. Everything visible at once.

#### Layout
```
┌─────────────────────────────────────────────────────────────┐
│ [Parallax] [Builder▼] [Dashboard] [Design]   [Settings] [AI]│
├──────────────────┬──────────────────────────────────────────┤
│                  │  METHOD ▼  │  URL bar              [Send]│
│  Collection      ├──────────────────────────────────────────┤
│  Sidebar         │  Params │ Headers │ Auth │ Body │ Scripts│
│                  │─────────────────────────────────────────│
│  ▶ Auth          │  (request config pane)                   │
│    login         ├──────────────────────────────────────────┤
│    refresh       │  200 OK  │ 142ms  │ 1.2KB  [History ▼]  │
│  ▶ Users         │─────────────────────────────────────────│
│    get-users     │  Body │ Headers │ Cookies │ Tests │ Vis  │
│    create-user   │  (response pane — always visible)        │
│                  │                                          │
│  [+ New Request] │                                          │
└──────────────────┴──────────────────────────────────────────┘
```

#### Request Builder Pane
- [x] Method selector (GET/POST/PUT/PATCH/DELETE/OPTIONS/HEAD/custom)
- [x] URL bar with template tag support and autocomplete for env vars
- [ ] Params tab — key/value pairs with enable/disable toggles
- [x] Headers tab — key/value with bulk edit (raw text) mode
- [ ] Auth tab — type selector: None / Bearer / Basic / API Key / OAuth2 / Digest / AWS Sig v4 / custom
- [x] Body tab — Raw (JSON/XML/text), Form-data, URL-encoded, Binary file, GraphQL
- [ ] Scripts tab — Pre-request and Test script editors with syntax highlighting
- [ ] Settings tab — timeout, redirects, proxy, SSL, client cert
- [ ] GraphQL tab (appears when type = GraphQL) — query editor, variables, schema explorer

#### Response Viewer Pane
- [x] Status code + reason phrase (color-coded by range)
- [x] Response time + size
- [x] Body viewer — JSON (pretty-print + collapsible tree), XML, HTML preview, raw text, binary hex dump
- [x] Headers tab
- [ ] Cookies tab — shows cookies set by response, links to jar
- [ ] Tests tab — pass/fail list for each `pm.test()` assertion
- [ ] Visualize tab — render custom HTML/chart from response data (Postman's "Visualizer" feature)
- [ ] History dropdown — quick-switch to a previous response

#### Collection Sidebar
- [x] Tree view of collections → folders → requests
- [ ] Drag-and-drop reordering
- [ ] Right-click context menu: Rename, Duplicate, Move, Delete, Run Folder, Add to mock
- [ ] Request color-coded by method (GET=blue, POST=green, PUT=orange, DELETE=red)
- [ ] Search/filter requests by name or URL
- [ ] Badge showing last response status code on each request

#### Environment Selector
- [x] Dropdown to switch active environment
- [ ] Quick-edit overlay — edit env vars inline without leaving Builder Mode
- [ ] Env diff view — compare two environments side by side
- [ ] Secret masking — values marked as `secret: true` are hidden in UI (shown as `••••`)

### 5. Additional Phase 1 Features

#### Cookie Jar
- [ ] Cookies stored per domain in `.parallax/cookies/jar.json`
- [ ] Cookie manager UI: view, add, edit, delete cookies
- [ ] Per-request opt-in/opt-out of cookie jar
- [ ] Session cookies expire on app restart unless pinned

#### Import / Export
- [ ] Import from Postman Collection v2.1 JSON
- [ ] Import from curl (paste a curl command → request auto-filled)
- [ ] Import from OpenAPI 3.x (stub; full support in Phase 4)
- [ ] Import from HAR file
- [ ] Export collection as Postman JSON (for teams still on Postman)

#### Response History
- [ ] Every sent request saves response to `.parallax/history/{request-id}/`
- [ ] Timeline UI in response pane — pick any past response
- [ ] Diff view: compare two historical responses

### 6. Go Sidecar (Phase 1 Stubs)
- [x] `src-go/main.go` — starts gRPC server on Unix socket
- [x] `watcher/` — file system watcher (notifies Rust of `.parallax/` changes)
- [x] Health check ping (proves IPC is working)
- [ ] Go binary compiled and bundled as Tauri sidecar

---

## Known Issues / Blockers

- `cargo check` macro expansion error from `generate_context!()` — tracking fix.
- Go binary not yet compiled — `externalBin` temporarily removed from `tauri.conf.json`.
- Icon files are placeholder PNGs (valid format, not final design).
- Script runner not yet chosen (Deno core vs. QuickJS — decision pending).

---

## Success Criteria

- [ ] `cargo tauri dev` launches the app without errors.
- [ ] User can send REST, GraphQL, and WebSocket requests and see the response.
- [ ] Template tags resolve at send-time (at minimum: `{% uuid %}`, `{% timestamp %}`, `{% response %}`)
- [ ] Pre-request and test scripts execute with `pm` API.
- [ ] Variable scoping resolves correctly across all 4 levels.
- [ ] Cookie jar persists between requests.
- [ ] Collections load from and save to `.parallax/collections/*.yaml`.
- [ ] Import a Postman Collection JSON — requests appear in sidebar.
- [ ] Import a curl command — request auto-fills.
- [ ] Response history saves and timeline UI shows past responses.
- [ ] Go sidecar starts and responds to a ping from Rust.

---

## Next Phase

Once the above criteria are met → **Phase 2: Dashboard, Collection Runner & CLI**
