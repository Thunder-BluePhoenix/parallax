# Phase 3 — AI Integration & MCP Server

**Status:** 🔲 Planned
**Depends on:** Phase 2 complete
**Goal:** Eliminate the AI Credit Wall. Let users bring their own LLM. Make Parallax an MCP server so AI agents can drive API collections directly. Generate living documentation from collections.

---

## Objectives

1. Build the **BYO-AI settings panel** — plug in OpenAI, Anthropic, or Ollama keys.
2. Implement **AI-powered test generation** from response shapes.
3. Implement **AI-powered request repair** (fix broken headers, auth, bodies).
4. Implement **AI-powered collection creation** from natural language.
5. Build Parallax as an **MCP (Model Context Protocol) server**.
6. Add **API documentation generation** — static site from collections (Postman's cloud docs, but local).
7. Add **AI script assistant** — help write pre-request and test scripts.
8. Add **AI environment variable suggestion** — detect and suggest missing vars.

---

## Deliverables

### 1. BYO-AI Settings Panel (Svelte 5)

Location: `Settings → AI`

**Providers:**
| Provider | Config | Local? |
|---|---|---|
| OpenAI | API key + model (gpt-4o, gpt-4-turbo, etc.) | No |
| Anthropic (Claude) | API key + model (claude-opus-4, claude-sonnet-4-6, etc.) | No |
| Ollama | Base URL (`http://localhost:11434`) + model name | Yes — fully offline |
| Custom OpenAI-compatible | Base URL + optional API key + model name | Flexible |
| Google Gemini | API key + model | No |

**Config stored:** `.parallax/ai.json` (gitignored by default)

```json
{
  "provider": "anthropic",
  "model": "claude-sonnet-4-6",
  "api_key": "sk-ant-...",
  "temperature": 0.3,
  "air_gap_mode": false
}
```

**Air-gap mode:** Toggle that disables all AI features entirely. Zero data leaves the machine. For environments where even Ollama is not permitted.

**Usage transparency:** Every AI action shows: provider, model, token count (if returned by API), estimated cost.

**Data policy:** No request body, URL, or response data is sent to any AI provider without the user explicitly triggering an AI action. Each AI action shows exactly what data will be sent before confirming.

### 2. AI Test Generator

**Trigger:** After receiving any response → "Generate Tests" button appears in the Tests tab.

**What it sends to the LLM:**
- Request method + URL (path only, no domain unless user approves)
- Response status code
- Response body (anonymized / truncated if large)
- Response headers (content-type, etc.)
- Existing tests (to avoid duplication)

**What it returns:** Test assertions in Parallax YAML format, ready to save.

**Output example (from a user list endpoint):**
```yaml
tests:
  - assert: response.status == 200
  - assert: response.body.data != null
  - assert: response.body.data.length > 0
  - assert: response.body.data[0].id != null
  - assert: response.body.data[0].email matches /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  - assert: response.headers["content-type"] contains "application/json"
  - assert: response.time < 2000
```

Also generates `pm.test()` format for users who prefer Postman-compatible scripting:
```javascript
pm.test("Status is 200", () => pm.response.to.have.status(200));
pm.test("Data array is not empty", () => {
    const body = pm.response.json();
    pm.expect(body.data).to.be.an('array').that.is.not.empty;
});
```

**User flow:** AI suggestion shown as a diff — user reviews, accepts or edits, then saves.

### 3. AI Request Repair

**Trigger:** When a request returns 4xx or 5xx — a "Diagnose with AI" button appears.

**What it analyzes:**
- Full request definition (method, URL, headers, body)
- Response status + body
- Current environment variables (key names only, not values for secrets)
- Any recent changes to the request (diff from history)

**What it suggests:**
- Missing or malformed headers
- Auth token expiry (suggests re-running a login request)
- Body schema mismatch
- URL parameter errors
- CSRF token issues

**Output:** Inline diff of the request — user accepts specific suggestions.

**Example:** A 422 response with `{"error": "Missing field: email"}`:
- AI detects the body is missing `email`
- Suggests adding `"email": "{{email}}"` to the body
- User accepts → request body updated

### 4. AI Collection Creator (Natural Language → Collection)

**Trigger:** `Cmd+K` → "Create collection from description..." (or via AI sidebar)

**User input:**
```
"Create a collection for a typical blog REST API with posts, 
comments, and user auth using JWT Bearer tokens. Include 
create, read, update, delete for each resource."
```

**What Parallax generates:**
- Full collection YAML with folders (Auth, Posts, Comments, Users)
- Realistic endpoint names and URLs with `{{base_url}}` placeholder
- Pre-request script that injects the token from a login response
- Test assertions for each request
- Matching environment template with required variables

**Use case:** Starting a new project's API collection from scratch in seconds.

### 5. AI Script Assistant

**Where:** Script editor tab (pre-request or test scripts)

**Features:**
- Inline autocomplete for `pm.*` API surface
- "Explain this script" — plain-English explanation of what the script does
- "Generate script for: [description]" — type a goal, get working code
- "Fix script error" — paste an error message, get corrected code
- Syntax highlighting + error markers in the editor

### 6. AI Environment Variable Suggestion

**Trigger:** Automatic — runs in background as user edits requests.

**What it detects:**
- `{{variable_name}}` references in any request that don't exist in the active environment
- Suggests adding missing variables with likely example values
- Groups suggestions: "These 3 variables are used but not defined in your environment"

### 7. MCP Server (Model Context Protocol)

**What it enables:** External AI agents (Claude Desktop, custom agents, automated workflows) can connect to Parallax and drive API collections as a tool.

**Implementation:**
- MCP-compatible HTTP server on `localhost:7676`
- Toggle: `Settings → AI → MCP Server → Enable`
- Auth: local token shown in settings (user copies it into their AI client)
- Runs only when explicitly enabled — not on by default

**MCP Tools exposed to AI agents:**
```
parallax.list_collections()
  → [{name, path, description, request_count, folder_count}]

parallax.get_collection(name)
  → {name, requests: [{id, name, method, url, ...}]}

parallax.get_request(collection, id)
  → {method, url, headers, body, scripts, tests}

parallax.execute_request(collection, id, env?, variables?)
  → {status, headers, body, latency, test_results}

parallax.run_collection(collection, env?, iterations?, data_file?)
  → {summary: {passed, failed, total}, results: [{id, status, tests}]}

parallax.create_request(collection, definition)
  → {id, saved: true}

parallax.list_environments()
  → [{name, path, variable_names}]

parallax.set_env_variable(env, key, value)
  → {updated: true}

parallax.generate_tests(collection, request_id)
  → [{assertion, confidence}]

parallax.start_mock(collection)
  → {url: "http://localhost:3001", active: true}
```

**Example agent workflow:**
> "Run the auth collection against staging, check all tests pass, and if any fail, use the response to diagnose and suggest a fix."

The agent calls:
1. `parallax.run_collection("auth", "staging")` → gets failures
2. `parallax.get_request("auth", "login-request")` → gets request definition
3. Analyzes failures → suggests fix → calls `parallax.create_request(...)` with patch

### 8. API Documentation Generator (Postman Cloud Docs — but local)

**Trigger:** Right-click collection → "Generate Documentation"

**Output:** Static HTML site, saved to `.parallax/docs/`

**Content per endpoint:**
- Method badge + full URL
- Description (from collection YAML `description` field)
- Request parameters, headers, body schema
- Example request (from saved history or mock)
- Example response (from saved history)
- Test assertions listed as "Expected Behavior"
- Copy-as-curl button

**Export options:**
- Static HTML folder (shareable as a site)
- Markdown (for GitHub wikis)
- OpenAPI 3.x YAML (reverse-generate spec from collection)

**Features:**
- Searchable — fuzzy search across all endpoints
- Dark/light mode (uses system preference)
- Hosted locally by `parallax-cli serve-docs --port 8080` for team sharing
- Can be published to GitHub Pages or any static host

---

## Success Criteria

- [ ] User enters Anthropic API key → "Generate Tests" produces valid assertions for a 200 response.
- [ ] "Generate Tests" also works with Ollama using a local model (no internet).
- [ ] AI Request Repair suggests the correct fix for a 422 response with a missing field.
- [ ] AI Collection Creator generates a 3-folder collection from a one-paragraph description.
- [ ] MCP server starts on `localhost:7676`.
- [ ] Claude Desktop can call `parallax.list_collections()` and `parallax.execute_request()`.
- [ ] An AI agent can run a full collection through MCP and receive pass/fail results.
- [ ] Documentation generator produces a valid HTML site from a 10-request collection.
- [ ] Generated OpenAPI YAML validates against the OpenAPI 3.x spec.
- [ ] Air-gap mode fully disables all AI features — no network calls from AI module.

---

## Next Phase

Once the above criteria are met → **Phase 4: Ecosystem Intelligence**
