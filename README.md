# Parallax

> **Local-first API super-client** — a Postman + Insomnia replacement built on Tauri v2, Svelte 5, and Rust.

![CI](https://github.com/Thunder-BluePhoenix/parallax/actions/workflows/ci.yml/badge.svg)

---

## Features

| Category | Highlights |
|---|---|
| **Request Builder** | HTTP/1.1, HTTP/2, WebSocket, SSE, gRPC (unary + streaming + reflection) |
| **Auth** | Bearer, Basic, API Key, OAuth2/PKCE, AWS SigV4, Digest, NTLMv2, mTLS, 7 ecosystem providers |
| **Protocols** | GraphQL (introspection + builder), gRPC (JSON framing + reflection), WebSocket, SSE |
| **Collections** | Import Postman v2.1, Insomnia v4, OpenAPI 3.x, HAR · Export Postman JSON |
| **Environments** | Variable resolution (`{{var}}`), diff viewer, smart extraction from requests |
| **Scripting** | Pre-request & test scripts (Postman-compatible `pm.*` API), AI generation |
| **Dashboard** | Live traffic proxy, health heatmap, load tester, mock server, git sync |
| **AI** | Test generator, request repair, collection creator, script assistant (OpenAI / Claude / Gemini / Ollama) |
| **Design** | OpenAPI 3.0 spec editor, preview, collection generator, GitHub Pages publisher |
| **Plugins** | Sandboxed JS plugin system · Built-ins: faker, jwt, base64, xml, soap |
| **Themes** | Dark, Light, High Contrast, Monokai, Solarized Dark, Custom CSS |
| **Code Gen** | 13 languages: curl, Python, JS/TS, Rust, Go, PHP, Ruby, Java, C#, Swift, Kotlin, PowerShell |
| **Cmd+K** | Global command palette — navigate, search requests, switch modes |

---

## Installation

### Pre-built binaries (recommended)

Download the latest release for your platform from the [Releases](../../releases) page:

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `Parallax_*.aarch64.dmg` |
| macOS (Intel) | `Parallax_*.x64.dmg` |
| Linux | `Parallax_*.AppImage` or `parallax_*.deb` |
| Windows | `Parallax_*.msi` |

### Build from source

**Prerequisites:** Node 20+, Rust 1.77+, Go 1.22+

```bash
# 1. Clone
git clone https://github.com/Thunder-BluePhoenix/parallax
cd parallax

# 2. Build Go sidecar
cd src-go && go build -o ../src-tauri/binaries/parallax-worker . && cd ..

# 3. Install frontend deps
npm install

# 4. Dev mode
npm run tauri dev

# 5. Production build
npm run tauri build
```

---

## Architecture

```
parallax/
├── src/                    # Svelte 5 frontend (TypeScript)
│   └── lib/
│       ├── components/     # UI components (Builder, Dashboard, Design, Sidebar…)
│       ├── stores/         # Svelte 5 $state stores (app, theme, ai, github…)
│       └── utils/          # Code gen, importers, exporters, plugin API
├── src-tauri/              # Tauri v2 + Rust backend
│   └── src/
│       ├── commands/       # Tauri commands (http, grpc, auth, collections, git…)
│       ├── http_engine.rs  # reqwest engine with TLS/proxy/auth
│       └── auth_providers/ # Ecosystem auth (Frappe, Django, Laravel, Rails…)
└── src-go/                 # Go sidecar (parallax-worker)
    ├── grpc/               # gRPC server (proxy, health, load test, mock, AI, MCP)
    ├── proxy/              # HTTP/HTTPS intercept proxy with MITM CA
    ├── mock/               # Mock server with path params + templating
    ├── runner/             # Collection runner (pm.* API via goja)
    └── mcp/                # MCP server (JSON-RPC 2.0 over HTTP+SSE)
```

---

## Plugin API

Plugins are sandboxed JS modules loaded at runtime. Enable them in **Dashboard → Plugins**.

```javascript
// Custom plugin example
(function(parallax) {
  parallax.registerTool("myPlugin.greet", (name) => `Hello, ${name}!`);
})
```

Use in scripts:
```javascript
// Pre-request or test script
const greeting = parallax.tools["myPlugin.greet"]("World");
pm.environment.set("greeting", greeting);
```

Built-in plugins: `parallax-faker`, `parallax-jwt`, `parallax-base64`, `parallax-xml`, `parallax-soap`

---

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `⌘K` / `Ctrl+K` | Open command palette |
| `⌘Enter` / `Ctrl+Enter` | Send request |
| `⌘N` / `Ctrl+N` | New request |
| `⌘S` / `Ctrl+S` | Save to collection |
| `⌘1` | Switch to Builder mode |
| `⌘2` | Switch to Dashboard mode |
| `⌘3` | Switch to Design mode |

---

## MCP Server

Parallax exposes a local MCP server for AI agent integration:

```bash
# Start with MCP enabled
parallax-worker --mcp --mcp-token mytoken

# Endpoint
http://localhost:7676/mcp/sse
```

Tools: `parallax.list_collections`, `parallax.get_traffic`, `parallax.execute_request`

---

## License

MIT © Thunder-BluePhoenix
