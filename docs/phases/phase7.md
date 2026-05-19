# Phase 7 — Shell Substitution & Editor Integration

**Status:** 🔲 Planned
**Depends on:** Phase 6 complete
**Inspired by:** LSP server + backtick env substitution
**Goal:** Bring Parallax into the developer's editor (VSCode first) and add shell command substitution in environments so power users can pull dynamic secrets from keychains, files, and CLIs without leaving their workflow.

---

## Objectives

1. Shell command substitution in environment files — `{% shell 'cmd' %}` template tag evaluates a shell command and injects its stdout as a variable value.
2. LSP server in the Go sidecar — JSON-RPC 2.0 server that exposes Parallax collections and environments to any LSP-compatible editor.
3. VSCode extension — browse collections, send requests, and view responses without leaving VS Code.

---

## Task Breakdown

### Shell Command Substitution

**Use case:** Dynamic secrets that live outside Parallax — AWS session tokens, base64-encoded files, JWT values from a system keychain CLI, etc.

```
# .parallax/environments/staging.json
{
  "AWS_TOKEN": "{% shell 'aws sts get-session-token --query Credentials.SessionToken --output text' %}",
  "PHOTO_B64": "{% shell 'base64 ~/images/avatar.png' %}",
  "DB_PASS":   "{% shell 'security find-generic-password -a parallax -w' %}"
}
```

| Task | Status | Notes |
|---|---|---|
| Add `{% shell 'cmd' %}` to the template tag engine | 🔲 | New tag type in `template-engine.ts` |
| Execute via Tauri `Command::new("sh").args(["-c", cmd])` in a new Rust command `eval_shell_template` | 🔲 | `src-tauri/src/commands/templates.rs` |
| Resolve shell tags in `resolveRequestTemplates()` before other tag resolution | 🔲 | First pass so output can itself contain `{{vars}}` |
| Timeout: kill shell command after 10s, surface error in response pane | 🔲 | Prevents hangs on broken commands |
| Security warning banner in Environment editor when a shell tag is detected | 🔲 | "This environment runs shell commands at send time" |
| Show resolved value preview in env editor (masked like secrets) | 🔲 | `eval_shell_template` called on blur |
| Support in CLI runner: resolve shell tags via `os/exec` in Go runner | 🔲 | `src-go/runner/runner.go` |
| Document in README — security implications, examples | 🔲 | |

**Supported shells:** `sh -c` on macOS/Linux; `cmd /C` on Windows (auto-detected by OS).

**Security model:** Shell tags only execute when a request is sent — never on load, never in collection browser. The security banner is non-dismissible the first time a shell-tagged env is activated.

---

### LSP Server (Go Sidecar)

** implements a minimal LSP server for `.l2` files. Parallax's LSP server targets `.parallax/` workspace files (YAML collections, JSON environments) and any editor where users write URL paths, headers, or body JSON with variable references.

**Protocol:** JSON-RPC 2.0 over stdin/stdout (standard LSP transport). Started with `parallax --lsp`.

| Task | Status | Notes |
|---|---|---|
| Add `--lsp` flag to Go binary / CLI entrypoint | 🔲 | `src-go/main.go` — starts LSP loop instead of gRPC server |
| Implement `initialize` / `initialized` / `shutdown` lifecycle | 🔲 | `src-go/lsp/server.go` |
| `textDocument/didOpen` + `textDocument/didChange` — track open `.yaml` / `.json` files | 🔲 | |
| `textDocument/completion` — suggest `{{VAR}}` from active environment | 🔲 | Trie or prefix scan over merged env keys |
| `textDocument/completion` — suggest template tags (`{% uuid %}`, `{% timestamp %}`, etc.) | 🔲 | Static list + description |
| `textDocument/hover` — show resolved value for `{{VAR}}` under cursor | 🔲 | Masked for secret vars |
| `workspace/executeCommand` → `parallax.sendRequest` — execute current request YAML, return response JSON | 🔲 | Reuses `runner.RunRequest()` |
| `workspace/executeCommand` → `parallax.listCollections` — return collection tree as JSON | 🔲 | For extension sidebar |
| Diagnostics: highlight unresolved `{{VAR}}` not found in any env | 🔲 | `textDocument/publishDiagnostics` |
| Diagnostics: highlight malformed template tags | 🔲 | |

**LSP capabilities summary:**

```json
{
  "completionProvider": { "triggerCharacters": ["{", "%"] },
  "hoverProvider": true,
  "executeCommandProvider": {
    "commands": ["parallax.sendRequest", "parallax.listCollections"]
  }
}
```

---

### VSCode Extension

A companion extension (`parallax-vscode`) that uses the LSP server for language features and adds a dedicated sidebar panel.

| Task | Status | Notes |
|---|---|---|
| Scaffold extension with `yo code` | 🔲 | TypeScript, Language Client target |
| Wire LSP client → `parallax --lsp` server process | 🔲 | `vscode-languageclient` package |
| Register `.yaml` / `.json` document selectors for LSP activation | 🔲 | Activate only inside `.parallax/` workspace |
| Sidebar tree view — collections + folders + requests | 🔲 | `TreeDataProvider` backed by `parallax.listCollections` LSP command |
| "Send Request" CodeLens above each request YAML block | 🔲 | Shows method + URL; click triggers `parallax.sendRequest` |
| Response panel (WebviewPanel) — renders status / headers / body | 🔲 | JSON syntax-highlighted via `highlight.js` |
| Environment picker status bar item | 🔲 | Shows active env; click opens quick-pick |
| `{{VAR}}` hover: show resolved value (masked for secrets) | 🔲 | Via LSP hover provider |
| Autocomplete `{{` → env var suggestions | 🔲 | Via LSP completion provider |
| Unresolved variable diagnostics (squiggly underline) | 🔲 | Via LSP diagnostics |
| Publish to VS Code Marketplace | 🔲 | `vsce package` + marketplace publish |
| Document in README | 🔲 | Installation + activation steps |

**Extension layout:**
```
parallax-vscode/
  src/
    extension.ts       — activate(), LSP client init, sidebar registration
    collectionProvider.ts — TreeDataProvider
    responsePanel.ts   — WebviewPanel for response display
  package.json         — contributes: commands, views, languages
```

---

## Success Criteria

| Criteria | Status |
|---|---|
| `{% shell 'cmd' %}` in env resolves at send-time with stdout value | 🔲 |
| Shell command timeout (10s) surfaces a clear error | 🔲 |
| `parallax --lsp` starts and responds to `initialize` | 🔲 |
| VSCode extension sidebar shows collection tree | 🔲 |
| Clicking a request in VSCode sidebar sends it and shows response | 🔲 |
| `{{VAR}}` autocomplete works in YAML collection files in VSCode | 🔲 |
| Unresolved variable shows a diagnostic squiggly | 🔲 |
| Extension published on VS Code Marketplace | 🔲 |
