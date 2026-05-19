# Phase 6 — Quick Wins

**Status:** 🔲 Planned
**Depends on:** Phase 5 complete
**Inspired by:**  plain-text API client
**Goal:** Absorb four high-value, low-effort ideas from ** that improve Parallax's ergonomics and interoperability with no architectural risk.

---

## Objectives

1. Import `.l2` files as Parallax collections — complete the importer set and give ** users a migration path.
2. `varjson` body type — `key=value` shorthand that serialises to JSON automatically.
3. Lenient JSON body parser — accept single quotes, trailing commas, and minor formatting errors before send.
4. CLI `--output` flag — write a single-request response to a structured JSON file for scripting and CI pipelines.

---

## Task Breakdown

### `.l2` File Importer

| Task | Status | Notes |
|---|---|---|
| Parse `.l2` `---` separator blocks | 🔲 | Extract pre-JS, request block, post-JS per file |
| Extract method + URL from request block | 🔲 | First two non-blank lines after separators |
| Extract headers (`Key: value` lines) | 🔲 | Stop at blank line before body |
| Extract body — JSON, varjson, multipart | 🔲 | Detect by content |
| Convert inline JS blocks to Parallax pre-request / test scripts | 🔲 | Pre-block → pre-request; post-block → test script |
| Variable substitution `${VAR}` → `{{VAR}}` | 🔲 | Regex replace across all fields |
| Import folder of `.l2` files as a collection | 🔲 | Walk directory; map subdirs to folders |
| Add `.l2` button to Sidebar import menu | 🔲 | Alongside existing Postman / Insomnia / OpenAPI buttons |
| Import `.l2.env` / `l2config.env` as Parallax environment | 🔲 | Parse `KEY=VALUE` pairs; skip backtick lines (Phase 7) |

**Implementation file:** `src/lib/importers/l2-importer.ts`

**`.l2` block anatomy:**
```
[optional pre-JS block]
---
METHOD
URL
Header-Name: value

{ "body": "json" }
---
[optional post-JS block]
---
```

**Variable mapping:**

| ** | Parallax |
|---|---|
| `${VAR}` | `{{VAR}}` |
| `result["json"]["field"]` | `pm.response.json().field` (post-JS → test script) |
| `let token = result["jwt"]` | `pm.environment.set("token", pm.response.json().jwt)` |

---

### `varjson` Body Type

| Task | Status | Notes |
|---|---|---|
| Add `varjson` tab option in body tab selector | 🔲 | Between `form` and `raw` |
| `key=value` editor (same key-value table as params/headers) | 🔲 | Reuse `KeyValueEditor.svelte` |
| `varjsonToJson()` serialiser in `sendRequest()` | 🔲 | Runs before template resolution; outputs `application/json` body |
| Auto-set `Content-Type: application/json` | 🔲 | When varjson body is non-empty |
| Export varjson body in Postman exporter | 🔲 | Serialise to `raw` JSON mode in v2.1 format |

**Serialiser logic (TypeScript):**
```typescript
function varjsonToJson(pairs: KeyValue[]): string {
  const obj = Object.fromEntries(pairs.filter(p => p.enabled).map(p => [p.key, p.value]));
  return JSON.stringify(obj);
}
```

---

### Lenient JSON Body Parser

| Task | Status | Notes |
|---|---|---|
| Add `json5` npm dependency | 🔲 | `npm install json5` |
| In `sendRequest()`, when body content-type is `application/json`, attempt `JSON5.parse()` then `JSON.stringify()` before dispatch | 🔲 | Normalises single-quotes, trailing commas, unquoted keys, comments |
| Show a subtle "JSON normalised" indicator in the response pane status bar | 🔲 | Small warning icon + tooltip |
| Skip normalisation if body contains template tags (resolve first) | 🔲 | Check for `{{` before parsing |

**What json5 fixes:**
```js
// These all parse correctly with json5:
{'key': 'value',}        // trailing comma + single quotes
{key: "value"}           // unquoted key
{"a": 1, /* comment */}  // comments
```

---

### CLI `--output FILE`

| Task | Status | Notes |
|---|---|---|
| Add `-o / --output <file>` flag to `parallax run` in Go CLI | 🔲 | Write structured JSON to file after single-request run |
| Output schema: `{ status, headers, body, latency_ms, timestamp }` | 🔲 | |
| Works alongside existing `--reporter html` — orthogonal flags | 🔲 | |
| When `--output -`, print JSON to stdout (pipe-friendly) | 🔲 | Enables `parallax run req.yaml | jq '.body.token'` |
| Document in README CLI section | 🔲 | |

**Output schema:**
```json
{
  "status": 200,
  "status_text": "OK",
  "headers": { "content-type": "application/json" },
  "body": { ... },
  "body_raw": "...",
  "latency_ms": 42.3,
  "timestamp": "2026-05-19T10:00:00Z",
  "request": {
    "method": "POST",
    "url": "https://api.example.com/token"
  }
}
```

---

## Success Criteria

| Criteria | Status |
|---|---|
| Can import a folder of `.l2` files as a Parallax collection | 🔲 |
| `varjson` body serialises to valid JSON and sends correctly | 🔲 |
| JSON body with single quotes / trailing commas sends without 400 errors | 🔲 |
| `parallax run req.yaml --output result.json` writes structured JSON file | 🔲 |
| `parallax run req.yaml --output -` pipes JSON to stdout | 🔲 |
| ** env vars (`${VAR}`) resolve correctly after import | 🔲 |
