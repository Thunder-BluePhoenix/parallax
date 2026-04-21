# Phase 5 — Polish, Performance & Release

**Status:** 🔲 Planned
**Depends on:** Phase 4 complete
**Goal:** Make Parallax production-ready for daily use. Hit performance targets, complete the plugin system, ship code generation, add full keyboard coverage, and release v1.0 across macOS, Windows, and Linux.

---

## Objectives

1. Performance audit — hit RAM, startup, and latency targets.
2. Plugin system — open the tool to community extensions (Insomnia-style, improved).
3. Complete code generation — 8 languages, every request.
4. Full keyboard shortcut coverage + command palette.
5. Complete theme system (dark, light, high-contrast, custom CSS).
6. Multi-window and split-tab support.
7. CI/CD build pipeline across all three platforms.
8. Auto-update system.
9. Documentation site.
10. Beta testing + bug bash.

---

## Performance Targets

| Metric | Target | Stretch |
|---|---|---|
| Cold startup time | < 800ms | < 500ms |
| RAM at idle | < 80MB | < 60MB |
| RAM under load test (500 users) | < 250MB | < 180MB |
| First request overhead vs. curl | < 5ms | < 2ms |
| Collection load (1000 requests) | < 200ms | < 100ms |
| Dashboard refresh rate | 60fps | 60fps |
| Go sidecar startup | < 300ms | < 150ms |

**Planned optimizations:**
- Lazy-load Design Mode and Dashboard Mode components — Builder Mode loads first
- `reqwest` connection pool per environment (keep-alive across requests)
- YAML parser: use `serde_yaml` with typed structs, not dynamic `Value`
- Go goroutine pool caps: max 1000 concurrent for load test, graceful backpressure
- Svelte stores: fine-grained reactivity — only re-render changed response pane
- Response body: stream large bodies in chunks, don't hold in memory
- History writes: async via Go watcher — non-blocking to UI

---

## Plugin System (Insomnia-style, improved)

Plugins are TypeScript/JavaScript modules loaded from `~/.config/parallax/plugins/` or a local project `.parallax/plugins/` folder.

### Plugin API

```typescript
interface ParallaxPlugin {
  name: string;
  version: string;
  description: string;

  // Hook: runs before a request is sent
  onRequest?(context: RequestContext): Promise<RequestContext>;

  // Hook: runs after a response is received
  onResponse?(context: ResponseContext): Promise<ResponseContext>;

  // Hook: runs in the script sandbox alongside pm.* API
  scriptHelpers?: Record<string, Function>;

  // Provide a custom auth provider
  authProvider?: AuthProviderDefinition;

  // Provide a custom schema explorer for a framework
  schemaExplorer?: SchemaExplorerDefinition;

  // Add a panel to the sidebar or response pane
  uiPanel?: UIPanelDefinition;

  // Add a custom template tag
  templateTags?: TemplateTagDefinition[];

  // Add CLI subcommands to parallax-cli
  cliCommands?: CLICommandDefinition[];
}
```

### Built-in Plugins (Shipped with Parallax)

| Plugin | What it does | Inspired by |
|---|---|---|
| `parallax-plugin-faker` | Template tags for fake data: `{% faker 'name' %}`, `{% faker 'address.city' %}` | Insomnia Generator |
| `parallax-plugin-jwt` | Decode JWT inline in response body, show header/payload/signature panel | Community Insomnia plugin |
| `parallax-plugin-aws-sigv4` | AWS Signature v4 signing (also built into Auth Providers) | Postman built-in |
| `parallax-plugin-graphql-schema` | Fetch and display GraphQL schema from endpoint | Insomnia |
| `parallax-plugin-base64` | Template tag: `{% base64 'encode' 'value' %}` | Community |
| `parallax-plugin-aes` | Encrypt/decrypt request body fields | Security use-case |
| `parallax-plugin-hashing` | Template tags: `{% hash 'sha256' 'value' %}` | Community |
| `parallax-plugin-date` | Rich date template tags: `{% date 'add' 7 'days' 'iso' %}` | Community |
| `parallax-plugin-xml` | XML body prettifier + XPath response query | Not in Insomnia |
| `parallax-plugin-soap` | SOAP envelope builder, WSDL importer | Not in Insomnia |

### Plugin Registry

- Plugins discoverable at `plugins.parallax.dev` (static GitHub Pages site)
- Install from UI: `Settings → Plugins → Browse Registry`
- Install from CLI: `parallax-cli plugin install parallax-plugin-faker`
- Plugins are npm packages — any npm package can be a Parallax plugin
- Sandboxed execution — plugins cannot access filesystem or network directly (only via Parallax APIs)

---

## Code Generation

Accessible from: Response pane → "Generate Code" button → language picker.
Also available as: right-click any request in sidebar → "Copy as..."

| Language | Library | Notes |
|---|---|---|
| curl | — | With all headers, auth, body |
| Python | `httpx` | Async-first |
| Python | `requests` | Sync alternative |
| JavaScript / TypeScript | `fetch` | Native, with async/await |
| JavaScript / TypeScript | `axios` | |
| Rust | `reqwest` | With `tokio` async |
| Go | `net/http` | Standard library |
| PHP | `Guzzle` | |
| Ruby | `Net::HTTP` | |
| Java | `OkHttp` | |
| C# | `HttpClient` | |
| Swift | `URLSession` | For iOS developers |
| Kotlin | `OkHttp` | For Android developers |

**Features:**
- Environment variable references preserved as named constants in output
- Auth handling included (Bearer header, Basic auth encoding, etc.)
- Copy to clipboard or save to file
- Copy all requests in a folder as a code file

---

## Keyboard Shortcuts & Command Palette

### Command Palette (`Cmd+K` / `Ctrl+K`)

Fuzzy-searches across:
- All requests by name, URL, method
- All collections
- All environments
- All settings pages
- All recent responses
- All actions (run collection, start mock, toggle proxy, etc.)

Every action in the app must be triggerable from the command palette. The palette is the single source of truth for keyboard-first users.

### Full Shortcut Table

| Action | Mac | Windows / Linux |
|---|---|---|
| **Navigation** | | |
| Open command palette | `Cmd+K` | `Ctrl+K` |
| Switch to Builder Mode | `Cmd+1` | `Ctrl+1` |
| Switch to Dashboard Mode | `Cmd+2` | `Ctrl+2` |
| Switch to Design Mode | `Cmd+3` | `Ctrl+3` |
| Toggle sidebar | `Cmd+B` | `Ctrl+B` |
| Open settings | `Cmd+,` | `Ctrl+,` |
| **Requests** | | |
| Send request | `Cmd+Enter` | `Ctrl+Enter` |
| Cancel request | `Esc` | `Esc` |
| New request | `Cmd+N` | `Ctrl+N` |
| New folder | `Cmd+Shift+N` | `Ctrl+Shift+N` |
| Duplicate request | `Cmd+D` | `Ctrl+D` |
| Delete request | `Cmd+Backspace` | `Ctrl+Delete` |
| Focus URL bar | `Cmd+L` | `Ctrl+L` |
| **Tabs** | | |
| New tab | `Cmd+T` | `Ctrl+T` |
| Close tab | `Cmd+W` | `Ctrl+W` |
| Next tab | `Cmd+]` | `Ctrl+Tab` |
| Prev tab | `Cmd+[` | `Ctrl+Shift+Tab` |
| **Response** | | |
| Copy response body | `Cmd+Shift+C` | `Ctrl+Shift+C` |
| Open response in new pane | `Cmd+Shift+O` | `Ctrl+Shift+O` |
| View response history | `Cmd+H` | `Ctrl+H` |
| **Collections** | | |
| Run collection | `Cmd+Shift+R` | `Ctrl+Shift+R` |
| Save collection | `Cmd+S` | `Ctrl+S` |
| Import | `Cmd+I` | `Ctrl+I` |
| Export | `Cmd+Shift+E` | `Ctrl+Shift+E` |
| **Environment** | | |
| Switch environment | `Cmd+E` | `Ctrl+E` |
| Edit environment | `Cmd+Shift+E` | `Ctrl+Shift+E` |

---

## Multi-Window & Tab Support

- Multiple requests open as tabs simultaneously (Insomnia and Postman both have this)
- Tabs persist across app restarts
- Split view: two requests side-by-side in Builder Mode
- Detach tab to separate window (`Cmd+Shift+D`)
- "Pin" a tab so it doesn't get replaced by clicking sidebar items

---

## Theme System

**Built-in themes:**
- **Parallax Dark** (default) — deep space palette
- **Parallax Light** — clean white with indigo accents
- **High Contrast Dark** — accessibility-focused
- **High Contrast Light** — accessibility-focused
- **Monokai** — classic for code-heavy users
- **Solarized Dark / Light** — popular developer palette

**Custom themes:**
- Override any theme via `~/.config/parallax/theme.css`
- Full CSS custom property exposure:
  ```css
  :root {
    --px-bg-primary: #0A0A0F;
    --px-bg-secondary: #14141E;
    --px-accent: #7C6EFF;
    --px-text: #E8E8F0;
    --px-border: #2A2A3A;
    --px-success: #00FF9C;
    --px-error: #FF4D6D;
    --px-warning: #FFB800;
  }
  ```

---

## Distribution & CI/CD

### Build Targets

| Platform | Output | Architecture |
|---|---|---|
| macOS | `.dmg` | Universal (arm64 + x86_64) |
| Windows | `.msi` + `.exe` | x64 |
| Linux | `.AppImage` | x64 |
| Linux | `.deb` | x64 |
| Linux | `.rpm` | x64 |

### GitHub Actions Pipeline

Triggered on: push of a version tag (`v*.*.*`)

```
build-macos    → sign + notarize → upload artifact
build-windows  → sign → upload artifact
build-linux    → upload artifact
create-release → download all artifacts → create GitHub release
update-json    → update releases.parallax.dev/latest.json
```

**Go sidecar build** is included in the pipeline — compiles for each target OS and embeds into the Tauri bundle.

### Auto-Update

- Tauri's built-in updater — checks `releases.parallax.dev/latest.json` on launch
- Update shown as a non-intrusive banner — user chooses when to apply
- Delta updates where Tauri supports it
- Signature verification on all update packages

---

## Documentation Site

Static site generated from Markdown, hosted on GitHub Pages at `docs.parallax.dev`.

**Sections:**
- **Getting Started** — 5-minute quickstart, install, first request
- **Builder Mode** — requests, scripting, template tags, environments, cookies, history
- **Dashboard Mode** — proxy, health monitor, load testing, collection runner
- **Design Mode** — OpenAPI editor, spec sync, Design-First workflow
- **Collections** — YAML schema reference, folder structure, import/export
- **Environments** — variable scoping, secret management, sub-environments
- **Scripts** — pre-request + test scripts, `pm.*` API reference, Python scripts
- **Auth Providers** — full reference for every provider
- **Mock Server** — definition format, record mode, CLI usage
- **Schema Explorer** — framework-by-framework setup guide
- **AI Integration** — BYO-AI setup, test generation, MCP server, air-gap mode
- **Plugin System** — writing a plugin, API reference, publishing to registry
- **parallax-cli** — full command reference, CI/CD integration, GitHub Actions examples
- **Keyboard Shortcuts** — full cheat sheet
- **FAQ / Troubleshooting**

---

## v1.0 Release Checklist

### Feature Completeness
- [ ] All Phase 1–4 success criteria met
- [ ] Every Postman must-have feature implemented (see plan.md Feature Inheritance Map)
- [ ] Every Insomnia must-have feature implemented (see plan.md Feature Inheritance Map)
- [ ] All Parallax-only features implemented and stable

### Quality
- [ ] No P0 or P1 bugs open
- [ ] Performance targets met on macOS M-series and Windows x64
- [ ] Memory does not grow unboundedly over a 2-hour session
- [ ] All imports (Postman, curl, OpenAPI, HAR) tested with real-world files
- [ ] Script runner tested with real Postman-style test suites
- [ ] MCP server tested with Claude Desktop and a custom agent

### UX
- [ ] Keyboard shortcut coverage for all core actions
- [ ] Command palette covers every action
- [ ] Dark and Light themes complete and polished
- [ ] All error states have user-facing messages (not raw Rust panics)
- [ ] Empty states (no collections, no requests, new install) are designed and implemented

### Distribution
- [ ] macOS, Windows, Linux builds passing in CI
- [ ] Auto-update working end-to-end (test the full update flow)
- [ ] Code signing and notarization on macOS
- [ ] Code signing on Windows
- [ ] Install/uninstall clean on all platforms

### Documentation
- [ ] Documentation site live at `docs.parallax.dev`
- [ ] README with 5-min quickstart
- [ ] `parallax-cli` man page

---

## Post-v1.0 Roadmap (Future Consideration)

| Feature | Notes |
|---|---|
| WebSocket client improvements | Multi-frame history, binary inspector |
| gRPC bi-directional streaming | Full client-streaming support |
| Team sync via self-hosted Git | No Parallax cloud — just your own remote |
| VS Code extension | Run `.parallax/` requests without leaving the editor |
| JetBrains plugin | Same as VS Code extension |
| Mobile companion | View health dashboards on iOS/Android |
| SOAP / WSDL support | Via plugin initially, built-in later |
| Kafka / AMQP / MQTT | Message queue request support |
| Database query runner | Run SQL alongside API requests — inspect DB state |
| Parallax Cloud (opt-in) | Paid — team sharing without self-hosting Git. Local-first remains free forever. |
