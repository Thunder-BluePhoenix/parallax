# Parallax — Motto & Identity

## Tagline
> **"See your APIs from every angle."**

## Extended Tagline
> *Everything Postman has. Everything Insomnia has. Everything neither thought of.*
> *Local-first. Git-native. AI-ready. Built for the next decade of development.*

---

## What "Parallax" Means

Parallax is the optical effect where an object's position appears to shift depending on your vantage point.

That's exactly what this tool does — it lets you see the same API from multiple perspectives simultaneously:
- **The Builder** sees clean request/response flows with scripting, chaining, and test assertions.
- **The Operator** sees live traffic, health status, and load curves.
- **The AI Agent** sees structured collections it can read, execute, and modify.
- **The Team** sees version-controlled YAML that lives inside the Git repo.
- **The CI pipeline** sees a CLI runner that needs no cloud account.

---

## The Problem Parallax Solves

Postman started as a simple Chrome extension. It grew into a massive platform that now requires a cloud account for basic collaboration, gates AI features behind credits, and consumes 600MB of RAM to send a GET request.

Insomnia started clean and elegant. Kong acquired it, pushed cloud sync, removed the local-only option, and gradually eroded what made it good. Its scripting story never matched Postman's. Its load testing story doesn't exist.

Both tools left gaps that developers fill with scripts, workarounds, and a second tool running alongside the first.

**Parallax is not a replacement for one of them. It is the tool you'd build if you took the best of both, threw away everything broken, and then kept going.**

---

## The Feature Promise

Parallax ships with:
- Every must-have from Postman (scripting, collection runner, Newman CLI, mock servers, cookie jar, dynamic variables, response visualization, code generation, variable scoping, GraphQL, gRPC, WebSocket, SSE).
- Every must-have from Insomnia (template tags with request chaining, plugin system, scratchpad mode, multi-pane UI, OpenAPI design editor, clean environment management, Postman collection importer).
- Features neither tool offers (Git-native storage, local load testing, live proxy dashboard, BYO-AI, MCP server, framework-aware auth providers, schema explorer, health heatmap, context-aware environment switching).

---

## Core Philosophy

| Principle | What it means in practice |
|---|---|
| **Local-First** | No cloud account required. Your data never leaves your machine unless you push it. |
| **Git-Native** | Every request, environment, script, and mock is a plain-text file you can `git diff`. |
| **Complete, Not Curated** | Every feature Postman and Insomnia ship must exist in Parallax. No artificial gaps. |
| **Hybrid-Engine** | Rust for raw speed. Go for concurrency. Right tool for each job. |
| **Open AI** | Bring your own keys (OpenAI, Anthropic, Ollama). No credit walls. No subscriptions. |
| **Protocol-Agnostic** | REST, GraphQL, gRPC, WebSocket, SSE — one unified interface, not five separate modes. |
| **Framework-Aware** | Auth flows and schema explorers for Frappe, Django, Laravel, Rails, WordPress, and more. |
| **Keyboard-First** | Every action must be reachable without a mouse. Command palette at the center. |
| **Scriptable** | JS and Python pre-request and test scripts. Not locked to one language. |

---

## What Parallax Is NOT

- Not a Postman clone. Not an Insomnia clone. It is the tool that learns from both.
- Not cloud-dependent. You own your data.
- Not Electron. Tauri + Rust means it starts in under a second and uses <100MB RAM at idle.
- Not locked to one framework, one AI provider, or one team's workflow.
- Not a platform. It is a tool. Fast, focused, and yours.

---

## Audience

Parallax is built for developers who:
- Are tired of cloud lock-in and subscription paywalls for features that should be free.
- Want their API collections tracked in Git alongside their code — not in some SaaS database.
- Run multiple local services and need one command center with real-time observability.
- Want AI assistance without surrendering their keys to a third-party platform.
- Need scripting power (Postman-level) with the elegance of Insomnia's UI.
- Want the best features of both tools in one app that actually starts fast.

---

## Visual Identity Direction

- **Color palette**: Deep space — indigo (`#7C6EFF`), void black (`#0A0A0F`), star white (`#E8E8F0`), signal green (`#00FF9C`), alert amber (`#FFB800`).
- **Typography**: Clean mono for code and URLs, geometric sans for UI labels.
- **UI philosophy**: Multi-pane by default. Request and response visible simultaneously. No forced tab-switching for common tasks. Get out of the way.
- **Mode switching**: Builder (the editor), Dashboard (the command center), Design (the spec editor) — three modes, one consistent shell.
