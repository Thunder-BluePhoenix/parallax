# Parallax — Master Development Plan

## Project Overview

**Parallax** is a standalone desktop API client built with Tauri v2 (Rust) + Go sidecar + Svelte 5.

The goal is not to clone either tool — it is to inherit every must-have feature from both Postman and Insomnia, fill every gap they leave open, and add capabilities neither has considered.

---

## Tech Stack

| Layer | Technology | Role |
|---|---|---|
| Frontend | Svelte 5 + Tailwind CSS | UI — Builder Mode, Dashboard Mode, Design Mode |
| Desktop Host | Tauri v2 | OS integration, window management, Rust-to-UI bridge |
| Request Engine | Rust (`reqwest`, `hyper`) | HTTP/1.1, HTTP/2, HTTP/3, WebSocket, SSE execution |
| Worker Engine | Go (sidecar binary) | Git-watcher, proxy, load tester, health checks, CLI runner |
| IPC Bridge | Local gRPC (Unix socket) | Rust ↔ Go communication |
| Persistence | Local filesystem (YAML/JSON) | `.parallax/` folder — Git-native |
| AI Layer | Pluggable (OpenAI / Anthropic / Ollama) | BYO-AI, MCP server |

---

## Architecture Summary

```
┌──────────────────────────────────────────────────────────────┐
│                        Svelte 5 UI                           │
│   Builder Mode  │  Dashboard Mode  │  Design Mode (OpenAPI)  │
└──────────┬──────────────┬──────────────────────┬────────────┘
           │ Tauri Commands (IPC)                 │
┌──────────▼──────────────────────────────────────▼──────────┐
│                  Rust (Tauri v2 Backend)                    │
│  HTTP Engine │ Script Runner │ Auth Providers │ Cookie Jar  │
│  File I/O    │ Template Tags │ Mock Server    │ AI SDK      │
└──────────────────────────┬──────────────────────────────────┘
                           │ gRPC (Unix socket)
┌──────────────────────────▼──────────────────────────────────┐
│                    Go Sidecar Binary                        │
│  Git-Watcher │ Proxy │ Load-Tester │ Health-Checker │ CLI   │
└──────────────────────────────────────────────────────────────┘
           │
┌──────────▼──────────────────────────────────────────────────┐
│                  .parallax/ (Local Filesystem)               │
│  collections/ │ environments/ │ scripts/ │ mocks/ │ reports/ │
└──────────────────────────────────────────────────────────────┘
```

---

## Feature Inheritance Map

This is the core promise: Parallax ships with EVERY feature from both tools — not a subset.

### From Postman (must-haves)

| Feature | What it is | Parallax implementation |
|---|---|---|
| Pre-request scripts | JS runs before the request fires | Script runner (JS + Python) in Rust |
| Test scripts | JS assertions on responses | Same engine, runs after response |
| Collection runner | Run all requests in a collection in sequence | Built-in, with configurable delays |
| Newman CLI equivalent | Run collections from terminal / CI | `parallax-cli` binary (Go) |
| Variable scoping | Global > Collection > Environment > Local | 4-level scope system in Rust |
| Dynamic variables | `{{$randomEmail}}`, `{{$guid}}`, `{{$timestamp}}` | Template tag engine |
| Response visualization | Custom HTML/chart views rendered from response data | Sandboxed iframe renderer |
| Mock servers | Serve fake responses for an endpoint | Local mock server (Rust, `tiny_http`) |
| Cookie jar | Persist and manage cookies across requests | Rust cookie store |
| Code generation | Generate curl, Python, JS, Rust, Go snippets | Phase 5 — 8 languages |
| API documentation | Auto-generate docs from collections | Phase 3 — static site export |
| Request history | Per-request history of previous sends | Stored in `.parallax/history/` |
| Import from curl | Paste a curl command → get a request | Parser in Rust |
| Import OpenAPI/Swagger | Spec → full collection | Phase 4 |
| Proxy settings | Route requests through a custom proxy | Configurable per-environment |
| GraphQL support | Query editor, schema introspection, variables | First-class — Phase 1 |
| WebSocket support | Connect, send frames, view stream | Phase 1 (Rust `tokio-tungstenite`) |
| gRPC support | Service reflection, unary + streaming calls | Phase 1 |
| Collection folders | Nested folder organization | Built-in to `.parallax/` YAML |

### From Insomnia (must-haves)

| Feature | What it is | Parallax implementation |
|---|---|---|
| Template tags | Dynamic values: `{% now %}`, `{% uuid %}`, `{% response 'body' %}` | Template tag engine — richer than Insomnia's |
| Request chaining | Use a response value in the next request | Template tag `{% response 'body', '$.token' %}` |
| Plugin system | Install/write plugins for custom auth, data gen | Plugin API (Phase 5) |
| Sub-environments | Base environment + overrides per context | Nested env files |
| Scratchpad mode | Open and use with no project, no login | Default state of the app |
| Clean multi-pane UI | Request and response visible at the same time | Core UI — no clicking between tabs |
| OpenAPI design editor | Write/edit API specs inside the tool | Design Mode (Phase 4) |
| Import/export HAR | HTTP Archive format for browser traffic | Phase 2 (proxy capture → HAR) |
| Import Postman collections | Load existing Postman JSON collections | Phase 1 importer |
| SSE support | Server-Sent Events viewer | Phase 1 (Rust async stream) |
| Response history timeline | See all previous responses for a request | `.parallax/history/` with timeline UI |
| Cookie management UI | View, edit, delete cookies per domain | Phase 1 |
| Keyboard-first design | Every action reachable by keyboard | Phase 5 — full shortcut coverage |
| Instant environment switch | Switch environment without restarting | Hot-swap store in Svelte |
| Bearer token display | Decode and preview JWT tokens inline | Phase 5 plugin (parallax-plugin-jwt) |
| gRPC method listing | Show all methods from reflection | Phase 1 |
| GraphQL schema explorer | Browse schema types and fields inline | Phase 1 |

### Parallax-Only (the gap-fillers)

| Feature | Why neither tool has it |
|---|---|
| Git-native `.parallax/` folder | Both use proprietary DBs or cloud |
| Local-first, zero cloud | Postman requires cloud; Insomnia pushes it |
| BYO-AI (OpenAI / Anthropic / Ollama) | Postman has credit wall; Insomnia has nothing |
| MCP server built-in | Neither tool is AI-agent-accessible |
| Live traffic proxy dashboard | Neither has real-time observability |
| Local load tester (Go engine) | Postman uses cloud workers; Insomnia has nothing |
| Health monitor with heatmap | Not in any API client |
| Framework-aware auth providers | No tool handles Frappe/Django/Laravel auth automatically |
| Schema explorer (crawl local code) | No tool reads your source to generate collections |
| Response shape inference | No tool builds a running schema from repeated calls |
| `parallax-cli` (Go) | Newman equivalent but local, no cloud dependency |
| Context-aware env switching | Auto-switch env based on Git branch / folder |
| MCP tools for AI agents | No tool exposes its collections to external AI |

---

## Development Phases

| Phase | Name | Focus | Status |
|---|---|---|---|
| Phase 1 | Architecture & Git-Native Core | Scaffold, HTTP engine, Builder UI, all protocols | 🔄 In Progress |
| Phase 2 | Dashboard & Go Concurrency Engine | Proxy, health, load test, collection runner, CLI | 🔲 Planned |
| Phase 3 | AI Integration & MCP Server | BYO-AI, test gen, request repair, MCP, docs | 🔲 Planned |
| Phase 4 | Ecosystem Intelligence | Auth providers, schema explorer, Design Mode, OpenAPI | 🔲 Planned |
| Phase 5 | Polish, Performance & Release | Plugin system, code gen, shortcuts, builds, release | 🔲 Planned |

---

## Full Feature vs. Tool Comparison

| Feature | Postman | Insomnia | Parallax |
|---|---|---|---|
| **Core** | | | |
| REST requests | ✅ | ✅ | ✅ |
| GraphQL | ✅ | ✅ | ✅ |
| gRPC | ✅ | ✅ | ✅ |
| WebSocket | ✅ | ✅ | ✅ |
| SSE | ✅ | ✅ | ✅ |
| HTTP/2 + HTTP/3 | Partial | No | ✅ |
| **Scripting** | | | |
| Pre-request scripts | ✅ JS | Plugin only | ✅ JS + Python |
| Test assertions | ✅ JS | Plugin only | ✅ JS + Python |
| Dynamic variables | ✅ | ✅ | ✅ + more built-ins |
| Template tags / chaining | Partial | ✅ | ✅ richer |
| **Organization** | | | |
| Collections + folders | ✅ | ✅ | ✅ YAML files |
| Environments | ✅ | ✅ | ✅ nested |
| Request history | ✅ | ✅ | ✅ |
| Cookie jar | ✅ | ✅ | ✅ |
| **Workflow** | | | |
| Collection runner | ✅ | No | ✅ |
| CLI / CI integration | ✅ Newman | No | ✅ parallax-cli |
| Mock servers | ✅ Cloud | No | ✅ Local |
| Code generation | ✅ | Partial | ✅ 8 languages |
| Import curl/OpenAPI | ✅ | ✅ | ✅ |
| Import Postman format | — | ✅ | ✅ |
| OpenAPI design editor | No | ✅ | ✅ Design Mode |
| API documentation | ✅ Cloud | No | ✅ Local static site |
| **Collaboration** | | | |
| Git integration | Plugin/export | No | ✅ Native |
| Cloud sync | Forced | Optional | None — local only |
| **AI** | | | |
| AI features | ✅ Credit wall | No | ✅ BYO keys |
| MCP server | No | No | ✅ |
| **Parallax-Only** | | | |
| Live traffic proxy | No | No | ✅ |
| Local load testing | Cloud workers | No | ✅ Go engine |
| Health heatmap | No | No | ✅ |
| Framework-aware auth | No | No | ✅ |
| Schema explorer | No | No | ✅ |
| Context-aware env switch | No | No | ✅ |
| RAM usage | ~600MB | ~300MB | Target: <100MB |
| Startup time | ~3–5s | ~2s | Target: <800ms |
| **Plugin system** | Limited | ✅ | ✅ |

---

## File Structure

```
parallax/
├── docs/
│   ├── plan.md              ← This file
│   ├── motto.md             ← Identity and philosophy
│   ├── tracker.md           ← Live development tracker
│   └── phases/
│       ├── phase1.md
│       ├── phase2.md
│       ├── phase3.md
│       ├── phase4.md
│       └── phase5.md
├── src-tauri/               ← Rust backend (Tauri v2)
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   └── commands/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src-go/                  ← Go sidecar worker
│   ├── main.go
│   ├── watcher/
│   ├── proxy/
│   ├── loadtest/
│   ├── health/
│   └── cli/                 ← parallax-cli (Newman equivalent)
├── src/                     ← Svelte 5 UI
│   ├── lib/
│   │   ├── components/
│   │   └── stores/
│   └── routes/
├── proto/                   ← gRPC definitions
└── .parallax/               ← Example project collections
    ├── collections/
    ├── environments/
    ├── scripts/
    ├── mocks/
    └── history/
```
