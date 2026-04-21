# Phase 4 — Ecosystem Intelligence

**Status:** 🔲 Planned
**Depends on:** Phase 3 complete
**Goal:** Make Parallax framework-aware and spec-aware. Smart auth providers for every popular framework, schema explorers that crawl local code, an OpenAPI Design Mode editor, and response intelligence — features no API client has shipped before.

---

## Objectives

1. Build a **pluggable Auth Provider system** for every major framework.
2. Build a **Schema Explorer** — crawl local code, generate collection stubs.
3. Ship **Design Mode** — an OpenAPI 3.x spec editor inside Parallax (Insomnia's best feature, improved).
4. Build a full **OpenAPI 3.x importer** — spec → complete collection with tests.
5. Add **response shape inference** — build a running schema from repeated responses.
6. Add **Postman Flows equivalent** — visual request chaining for complex workflows.
7. Add **gRPC service reflection** — auto-discover all methods from a running gRPC server.
8. Add **GraphQL schema explorer** improvements — full type browser, subscription support.

---

## Deliverables

### 1. Auth Provider System (Rust)

Location: `src-tauri/src/auth/`

The Auth tab in the request builder is powered by this system. Each provider handles a complete auth lifecycle: initial auth, token injection, and automatic refresh.

```rust
#[async_trait]
trait AuthProvider: Send + Sync {
    fn name(&self) -> &str;
    fn detect(base_url: &str) -> bool;
    async fn authenticate(&self, config: AuthConfig) -> Result<AuthState>;
    async fn inject(&self, request: &mut PreparedRequest, state: &AuthState);
    async fn refresh_if_needed(&self, state: &mut AuthState) -> Result<bool>;
    fn ui_fields(&self) -> Vec<SettingsField>;  // What to show in the Auth tab UI
}
```

#### All Auth Providers

**Frappe / ERPNext**
- POST to `/api/method/login` with `usr` + `pwd`
- Extracts `sid` cookie + `X-Frappe-CSRF-Token` from subsequent GET
- Injects both into all mutation requests automatically
- Refreshes on 403 (session expired) without prompting user
- UI fields: Username, Password, Site URL

**Django**
- GET any page to receive `csrftoken` cookie
- Injects as `X-CSRFToken` header on POST/PUT/PATCH/DELETE
- Handles Django REST Framework `TokenAuthentication` and `SessionAuthentication`
- UI fields: Login URL, Username, Password (or static token)

**Laravel / Sanctum**
- GET `/sanctum/csrf-cookie` to receive `XSRF-TOKEN` cookie
- Injects as `X-XSRF-TOKEN` header (URL-decoded)
- Handles Passport Bearer tokens
- UI fields: Base URL, Email, Password

**Ruby on Rails**
- Fetches `authenticity_token` from login page HTML via CSS selector
- Handles Devise session auth
- Handles API token auth (custom header)
- UI fields: Login URL, Token field selector, Credentials

**WordPress REST API**
- Handles Application Passwords (Basic Auth)
- Handles WP nonces from page source
- Handles JWT for WP plugins (e.g., JWT Auth for WP)
- UI fields: Site URL, Username, Application Password

**Next.js / NextAuth**
- Handles NextAuth session cookies
- CSRF token from `/api/auth/csrf`
- UI fields: Callback URL, Provider, Credentials

**FastAPI / Starlette**
- Bearer token injection
- OAuth2 password flow
- UI fields: Token URL, Client ID, Client Secret, Scope

**ASP.NET Core**
- Handles Antiforgery tokens (`__RequestVerificationToken`)
- Bearer JWT with configurable header name
- UI fields: Login URL, Token endpoint, Credentials

**Generic OAuth2 (all providers)**
- Authorization Code flow with PKCE
- Client Credentials flow
- Password flow
- Token refresh on expiry
- Token storage in keychain / OS credential store
- UI fields: Auth URL, Token URL, Client ID, Client Secret, Scope, Redirect URI

**Generic Bearer**
- Static token injection
- Auto-prefix: `Bearer `, `Token `, or custom
- UI fields: Token value, prefix

**AWS Signature v4**
- Signs requests with AWS SigV4
- Supports all AWS services
- UI fields: Access Key, Secret Key, Region, Service

**API Key**
- Header injection: `X-API-Key: {value}`
- Query param injection: `?api_key={value}`
- Cookie injection
- UI fields: Key name, Key value, Placement

**Digest Auth**
- RFC 7616 Digest Access Authentication
- UI fields: Username, Password

**NTLM / Negotiate**
- Windows Integrated Authentication
- UI fields: Username, Password, Domain

**Certificate Auth (mTLS)**
- Client certificate + private key
- UI fields: Certificate file, Key file, CA bundle

### 2. Schema Explorer (Go engine)

Location: `src-go/schema/`

A Go-based crawler that reads local project source code and generates Parallax collection stubs. Saves hours of manually typing out endpoints.

#### Frappe / ERPNext Explorer
- Scans `apps/*/doctype/**/*.json` for DocType definitions
- Generates full CRUD collection: `GET/POST /api/resource/{DocType}`, `GET/PUT/DELETE /api/resource/{DocType}/{name}`
- Reads field definitions → suggests request body JSON schema
- Scans Python files for `@frappe.whitelist()` decorators → generates `POST /api/method/{module}.{method}` stubs
- Reads Form DocType JSON for child table relationships
- Output: fully-formed `.parallax/collections/frappe-{app-name}.yaml`

#### Django Explorer
- Parses `urls.py` files recursively to discover all registered routes
- Reads DRF `Serializer` classes to infer request/response field shapes
- Reads DRF `ViewSet` classes to generate full CRUD collections
- Generates: URL pattern → method → request body schema → test stub
- Supports `router.register()` patterns

#### Laravel Explorer
- Runs `php artisan route:list --json` if PHP is available
- Falls back to parsing `routes/api.php` and `routes/web.php` directly
- Reads migration files and Eloquent model `$fillable` for request body schemas
- Generates: Route → controller method → request body → test stub

#### Rails Explorer
- Parses `config/routes.rb` for resource routes (`resources :users`)
- Reads `db/schema.rb` for model column definitions
- Generates standard RESTful collection: index, show, create, update, destroy

#### FastAPI Explorer
- Reads Python files for FastAPI `@app.get/post/put/delete/patch` decorators
- Extracts Pydantic model schemas for request/response bodies
- Generates: endpoint → method → body schema → response model → tests

#### Express.js / Fastify Explorer
- Parses JS/TS route definitions (`app.get(...)`, `router.post(...)`)
- Reads TypeScript interfaces or Zod schemas for body types
- Generates: route → method → body stub

#### OpenAPI / Swagger Importer (full)
- Accepts OpenAPI 3.x and Swagger 2.0 `.yaml` or `.json`
- Converts ALL paths, operations, parameters → Parallax collection
- Preserves: descriptions, examples, security schemes → auth provider config
- Generates test assertions from response schemas (e.g., required fields → `assert field != null`)
- Maps security schemes → appropriate Parallax auth provider
- Drag-and-drop into Parallax OR `parallax-cli import openapi spec.yaml`

#### Framework Auto-Detection

When a user opens a folder or `.parallax/` is initialized:

```
Detected: Frappe application (found apps.txt, frappe in Python path)
→ Auto-configure: Frappe Auth Provider
→ Suggest: "Scan DocTypes and generate collection?"

Detected: Django application (found manage.py + django in requirements.txt)
→ Auto-configure: Django CSRF Provider
→ Suggest: "Scan URL patterns and generate collection?"

Detected: Laravel (found artisan + composer.json with laravel/framework)
→ Auto-configure: Laravel Sanctum Provider
→ Suggest: "Run route:list and generate collection?"

Detected: Rails (found Gemfile with 'rails')
→ Suggest: "Scan routes.rb and generate collection?"

Detected: OpenAPI spec (found openapi.yaml or swagger.json)
→ Suggest: "Import spec and generate collection?"
```

### 3. Design Mode — OpenAPI Spec Editor (Insomnia's best feature, upgraded)

A third top-level mode for API-design-first workflows.

**What it is:** Write and edit OpenAPI 3.x specs inside Parallax, with instant preview of what the generated collection looks like.

**Features:**
- Split-pane: YAML editor on left, rendered docs preview on right
- Real-time validation with inline error markers
- Autocomplete for OpenAPI keywords, HTTP methods, media types
- Schema builder UI — define models with a form instead of writing YAML by hand
- "Try it out" — execute a request directly from the spec preview
- Sync to collection: changes in Design Mode auto-update the collection
- Sync from collection: record requests in Builder Mode → push back to spec
- Export: generate OpenAPI 3.x from an existing `.parallax/` collection
- Import: paste a spec → Parallax creates Design Mode document + collection simultaneously
- Lint: check spec against OpenAPI style rules (missing descriptions, unused schemas, etc.)

**File:** Spec saved as `.parallax/design/{api-name}.openapi.yaml` (committed to Git)

### 4. Response Shape Inference

After N requests to the same endpoint, Parallax builds a running schema automatically.

**Features:**
- Field names, types, nullable status tracked per field
- Confidence score: "this field was present in 23/25 responses"
- Displayed in a "Schema" panel next to the response body
- Export as:
  - JSON Schema
  - TypeScript interface
  - Pydantic model (Python)
  - Rust struct (with `serde`)
  - Go struct
- Used by AI Test Generator (Phase 3) to write stronger, schema-aware assertions

**Storage:** `.parallax/schemas/{request-id}.json` — committed to Git, evolves over time

### 5. Visual Flow Builder (Postman Flows equivalent)

A canvas-based visual editor for building complex, multi-step API workflows.

**What it is:** Drag request nodes onto a canvas, connect them with arrows, define data mapping between outputs and inputs.

**Node types:**
- **Request node** — a single API call
- **Condition node** — branch on response value (if `status == 200`, go to A, else go to B)
- **Transform node** — extract, reshape, or compute a value (e.g., `body.data[0].id`)
- **Loop node** — iterate over an array in a response
- **Delay node** — wait N milliseconds
- **Webhook node** — trigger an external URL
- **Variable node** — set an environment variable from a value

**Use cases:**
- Login → extract token → use token in 5 parallel requests → aggregate results
- Paginate through a list API, collecting all items across pages
- Create a resource, get its ID, use that ID in follow-up requests

**Storage:** Flow saved as `.parallax/flows/{flow-name}.yaml`

**Execution:** Runs via the Collection Runner engine (Phase 2) with flow graph instead of linear order.

### 6. gRPC Service Reflection

Auto-discover all services and methods from a running gRPC server. No proto files needed.

**How:** Uses gRPC Server Reflection Protocol (if enabled on target server):
1. Connect to server URL
2. List all services via `grpc.reflection.v1alpha.ServerReflection`
3. Describe each service's methods, request types, response types
4. Auto-populate the gRPC request builder with all methods and example bodies

**Also supports:** Manually providing `.proto` files for servers without reflection.

### 7. Enhanced GraphQL Support

- **Schema explorer:** Browse all types, queries, mutations, subscriptions in a sidebar tree
- **Field autocomplete:** IntelliSense-style completion as you type a query
- **Query builder:** Click fields to build queries visually (no typing)
- **Subscription support:** Long-lived WebSocket connection, stream events to response pane
- **Multiple schemas:** Different schemas per environment (dev vs. staging schema)
- **Schema diff:** Compare schema versions to see what changed
- **Persisted queries:** Send query hash instead of full query text

---

## Success Criteria

- [ ] Frappe auth provider: login once → all requests have valid `sid` + CSRF automatically.
- [ ] Django auth provider: CSRF token injected on POST without user action.
- [ ] OAuth2 provider: completes authorization code flow, stores and refreshes token.
- [ ] AWS SigV4 signs a request to S3 correctly.
- [ ] Schema Explorer scans a Frappe app and generates a collection with 15+ endpoints.
- [ ] Django Explorer generates a collection from `urls.py` + DRF ViewSets.
- [ ] OpenAPI importer converts a 50-endpoint spec into a full Parallax collection with tests.
- [ ] Framework auto-detection correctly identifies Frappe, Django, and Laravel projects.
- [ ] Design Mode editor validates a spec and shows errors inline.
- [ ] Design Mode "Try it out" sends a request and shows response.
- [ ] Response shape inference builds a schema after 5 requests and exports as TypeScript.
- [ ] Visual Flow Builder runs a 3-node flow: Login → Extract Token → Call Protected Endpoint.
- [ ] gRPC reflection lists all methods from a running gRPC server with reflection enabled.
- [ ] GraphQL subscription streams events from a live endpoint.

---

## Next Phase

Once the above criteria are met → **Phase 5: Polish, Performance & Release**
