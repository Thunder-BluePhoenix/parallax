# Phase 2 — Dashboard, Collection Runner & CLI

**Status:** 🔲 Planned
**Depends on:** Phase 1 complete
**Goal:** Add everything that makes Parallax more than a request-sender. Collection runner, Newman-equivalent CLI, real-time observability, health monitoring, and local load testing.

---

## Objectives

1. Build the **Collection Runner** — run a full collection or folder in sequence (Postman's killer feature).
2. Build `parallax-cli` in Go — the Newman equivalent for CI/CD pipelines.
3. Build the **Dashboard Mode** UI — a full "Command Center" view.
4. Implement the **Go-based local proxy** for live traffic interception.
5. Add **background health checks** for local microservices.
6. Add the **Load Test** tab with real-time latency histogram.
7. Complete the **gRPC streaming bridge** between Rust and Go.
8. Add **Response Visualization** (Postman's "Visualizer" — render HTML from response data).
9. Add **Mock Server** (local — no cloud).

---

## Deliverables

### 1. Collection Runner (Postman's Core Workflow Feature)

The ability to run all requests in a collection or folder in sequence, with scripting, variable passing, and test reporting.

**UI:** Dedicated "Runner" panel, accessible from collection right-click or top nav.

**Controls:**
- Select collection or folder to run
- Select environment
- Number of iterations (run the whole collection N times)
- Delay between requests (ms)
- Stop on first test failure toggle
- Data file (CSV/JSON to drive iterations with different variables per run — Postman "Data Variables")

**Execution:**
- Requests run in order, top to bottom, respecting folder nesting
- Pre-request scripts and test scripts execute for each request
- Variables set by test scripts carry forward to the next request (chaining)
- Each request shows: pass/fail status, response time, test results

**Output:**
- Live run feed — shows each request as it executes with status badge
- Summary panel — total passed, failed, skipped, total time
- Report saved to `.parallax/reports/run-{timestamp}.json`
- Report viewable as HTML summary

**Data-driven testing (Postman "Data Variables"):**
```csv
email,password
user1@test.com,pass1
user2@test.com,pass2
```
Each row drives one iteration of the collection run, injecting `{{email}}` and `{{password}}`.

### 2. `parallax-cli` — Go Binary (Newman Equivalent)

A standalone CLI tool for running Parallax collections from the terminal or CI/CD pipelines, with zero cloud dependency.

Location: `src-go/cli/`

**Commands:**
```bash
# Run a collection
parallax-cli run .parallax/collections/user-api.yaml \
  --env .parallax/environments/staging.json \
  --iterations 3 \
  --delay 200 \
  --reporters json,html,junit \
  --output ./reports

# Validate a collection (check YAML schema, catch broken variable refs)
parallax-cli validate .parallax/collections/user-api.yaml

# List requests in a collection
parallax-cli list .parallax/collections/user-api.yaml

# Import Postman collection
parallax-cli import postman collection.json --output .parallax/collections/

# Export collection as Postman format
parallax-cli export .parallax/collections/user-api.yaml --format postman

# Start mock server from collection
parallax-cli mock .parallax/collections/user-api.yaml --port 3001
```

**Reporters:**
- `console` — colored terminal output (default)
- `json` — machine-readable results
- `html` — beautiful static HTML report
- `junit` — XML for CI systems (Jenkins, GitLab, GitHub Actions)

**Exit codes:**
- `0` — all tests passed
- `1` — one or more tests failed
- `2` — collection could not run (missing env, parse error)

**GitHub Actions example:**
```yaml
- name: Run API Tests
  run: parallax-cli run .parallax/collections/api.yaml --env ci.json --reporters junit
- name: Upload Results
  uses: actions/upload-artifact@v3
  with:
    path: reports/
```

### 3. Dashboard Mode UI (Svelte 5)

A second top-level mode — the "Command Center." Powered by the Go sidecar.

```
┌─────────────────────────────────────────────────────────────┐
│ [Parallax] [Builder] [Dashboard▼] [Design]  [Settings] [AI] │
├──────────────┬──────────────────────┬───────────────────────┤
│              │  LIVE TRAFFIC        │  HEALTH MONITOR       │
│  Services    │  ─────────────────   │  ──────────────────── │
│  ──────────  │  POST /api/login     │  ● Frappe Dev  120ms  │
│  ● Frappe    │  200  │ 45ms │ 1.2KB │  ○ Andromeda   DOWN   │
│  ○ Andromeda │  GET  /api/users     │  ● PostgreSQL  8ms    │
│  ● Postgres  │  200  │ 12ms │ 4KB   │                       │
│              │  POST /api/order     │  LOAD TEST            │
│  [+ Add]     │  422  │ 98ms │ 0.4KB │  ──────────────────── │
│              │                      │  [RPS chart]          │
│  GIT SYNC    │  [Pause] [Clear]     │  [Latency histogram]  │
│  ──────────  │                      │                       │
│  3 modified  │                      │  [Start Test]         │
└──────────────┴──────────────────────┴───────────────────────┘
```

**Panels:**
- **Live Traffic Stream** — real-time feed of intercepted requests with method, URL, status, latency, size. Click any row to inspect full request/response.
- **Health Heatmap** — grid of service tiles: name, URL, last status code, uptime %, avg response time, sparkline. Configurable check intervals.
- **Load Test Results** — real-time chart: requests/sec, p50/p95/p99 latency, error rate, throughput. Histogram of latency distribution.
- **Git Sync Status** — which `.parallax/` files have changed vs. last commit. Quick diff preview.
- **Collection Run History** — log of past `parallax-cli` or in-app runner executions with pass/fail counts.

### 4. Go Local Proxy (`src-go/proxy/`)

A lightweight HTTP/HTTPS proxy that intercepts traffic from any local app configured to use it.

```
Any local app → proxy (localhost:8765) → target server
                    ↓
              TrafficEvent → gRPC → Rust → Svelte dashboard
```

**Features:**
- HTTP proxy (configurable port, default 8765)
- HTTPS via MITM with a generated local CA cert (user installs it once)
- Read-only — no modification of traffic
- Filter by domain, path, method, status code
- Export captured traffic as HAR
- "Replay" — send a captured request through the Parallax engine

```go
type TrafficEvent struct {
    ID           string
    Timestamp    time.Time
    Method       string
    URL          string
    RequestBody  string
    ReqHeaders   map[string]string
    Status       int
    ResponseBody string
    ResHeaders   map[string]string
    Latency      time.Duration
    Size         int64
}
```

### 5. Health Monitor (`src-go/health/`)

Background goroutine pool that pings configured services and streams status updates.

**Config format (`.parallax/health.yaml`):**
```yaml
services:
  - name: Frappe Dev
    url: http://localhost:8000/api/method/ping
    interval: 30s
    timeout: 5s
    expect_status: 200
  - name: Andromeda API
    url: http://localhost:3000/health
    interval: 15s
    expect_body_contains: "ok"
  - name: PostgreSQL
    url: postgres://localhost:5432/mydb
    type: tcp          # just checks if port is open
    interval: 60s
```

**Features:**
- Per-service goroutine with configurable interval
- Uptime % tracked in `.parallax/health.db` (SQLite)
- Desktop notification on status change (up → down or down → up)
- Historical uptime charts in Dashboard
- Alert webhook: POST to a URL on failure (e.g., Slack incoming webhook)

### 6. Load Tester (`src-go/loadtest/`)

Local, cloud-free load testing engine. No Postman cloud workers needed.

**Controls (Dashboard UI):**
- Target: pick any request from a collection
- Concurrent users: 1–1000 (slider)
- Duration: 1–600 seconds
- Ramp-up: add N users per second until target
- Think time: delay between requests per user (simulate real users)

**Metrics streamed to UI every 500ms:**
- Requests/second (RPS)
- Latency: p50, p90, p95, p99, max
- Error rate %
- Throughput (MB/s)
- Active connections

**Output:**
- Real-time chart in Dashboard
- Final report: `.parallax/reports/load-{timestamp}.json`
- HTML report with latency histogram and percentile breakdown

### 7. Response Visualization (Postman "Visualizer")

In Builder Mode — a "Visualize" tab in the response pane.

Users write an HTML template in the test script that receives response data:

```javascript
const data = pm.response.json();
pm.visualizer.set(`
  <table>
    <tr><th>Name</th><th>Email</th></tr>
    {{#each users}}
    <tr><td>{{name}}</td><td>{{email}}</td></tr>
    {{/each}}
  </table>
`, { users: data.data });
```

**Implementation:** Sandboxed iframe renderer in Svelte. Template engine: Handlebars (same as Postman). Data passed from Rust via Tauri event.

### 8. Local Mock Server

Run a fake server from a collection — no cloud required.

**Mock definition (in collection YAML):**
```yaml
mocks:
  - path: /api/users
    method: GET
    status: 200
    delay: 0
    response:
      headers:
        Content-Type: application/json
      body: |
        {"data": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}

  - path: /api/users/:id
    method: GET
    status: 200
    response:
      body: |
        {"id": "{{params.id}}", "name": "Alice"}
```

**Features:**
- Starts on configurable port (default: 3001)
- Path parameters (`:id`) and wildcards
- Configurable response delay (to simulate slow APIs)
- Response templating with request data
- Record mode: proxy real requests and auto-generate mock definitions
- Can be started from `parallax-cli mock ...` for CI

**Implementation:** Rust `tiny_http` or `axum` inside Tauri, managed by Tauri command.

### 9. gRPC Streaming Bridge (complete)

Phase 1 only needed unary request/response. Phase 2 adds **server-side streaming** for all real-time features:

| Stream | Direction | Data |
|---|---|---|
| `WatchFiles` | Go → Rust | `.parallax/` file change events |
| `WatchTraffic` | Go → Rust | Intercepted proxy events |
| `WatchHealth` | Go → Rust | Service status updates |
| `StreamLoadTest` | Go → Rust | Real-time load test metrics (500ms intervals) |
| `StreamRunner` | Go → Rust | Collection runner request-by-request progress |

---

## Success Criteria

- [ ] Collection Runner runs a 5-request collection with variable chaining between requests.
- [ ] Data-driven run works: CSV with 3 rows drives 3 iterations.
- [ ] Test report (JSON + HTML) is generated after a run.
- [ ] `parallax-cli run` works from terminal and exits with code 1 on failed tests.
- [ ] GitHub Actions example runs end-to-end with JUnit XML output.
- [ ] Switching between Builder Mode and Dashboard Mode works smoothly.
- [ ] Local proxy intercepts traffic and shows it in the Live Traffic panel.
- [ ] Health monitor pings two services and shows up/down status with history.
- [ ] Load test fires 100 concurrent users and displays real-time chart.
- [ ] Mock server responds correctly to defined routes.
- [ ] Response Visualizer renders an HTML table from JSON response data.
- [ ] gRPC streaming is stable under sustained data flow for 5+ minutes.

---

## Next Phase

Once the above criteria are met → **Phase 3: AI Integration & MCP Server**
