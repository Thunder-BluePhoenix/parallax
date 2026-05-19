// Parallax Global App State — Svelte 5 Runes
console.log("[Parallax] Store module loading...");
import { sendRequest as platformSendRequest, cancelRequest as platformCancelRequest, readFileForTemplate } from "../platform";
import { resolveRequestTemplates, resolveShellTagsInObj } from "../utils/template-tags";

const uuid = () => Math.random().toString(36).substring(2) + Date.now().toString(36);

// ============================================================
// App Mode & Overlays
// ============================================================
export const appMode = $state<{ value: "builder" | "dashboard" | "design" }>({ value: "builder" });
export const showRunner = $state<{ value: boolean }>({ value: false });

// ============================================================
// Current Workspace
// ============================================================
export const currentWorkspace = $state<{
  path: string; name: string; gitBranch: string | null; hasParallax: boolean;
}>({ path: "", name: "No Workspace", gitBranch: null, hasParallax: false });

// ============================================================
// Collection types (mirrors Rust structs)
// ============================================================
export interface CollectionRequest {
  id: string; name: string; method: string; url: string;
  headers: Record<string, string>;
  params?: Record<string, string>;
  body?: { type: string; content: any; raw?: string } | null;
  auth?: any | null;
  scripts?: { preRequest?: string; tests?: string } | null;
}
export interface CollectionFolder { name: string; requests: CollectionRequest[]; }
export interface Collection {
  name: string; version: string; description?: string;
  requests: CollectionRequest[]; folders: CollectionFolder[];
  variables?: Record<string, string>;
}
export const loadedCollections = $state<Collection[]>([]);

// ============================================================
// Request state
// ============================================================
export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS" | "WS" | "SSE" | "GRPC";
export interface AuthState {
  type: "none" | "bearer" | "basic" | "api_key" | "ecosystem_provider";
  token: string; username: string; password: string;
  apiKeyHeader: string; apiKeyValue: string;
  apiKeyLocation: "header" | "query";
  provider: string; authUrl: string; providerSession: any | null;
}
export interface RequestState {
  id: string; name: string; method: HttpMethod; url: string;
  headers: Record<string, string>; params: Record<string, string>;
  bodyType: "none" | "json" | "form" | "urlencoded" | "raw" | "graphql" | "varjson";
  bodyContent: string; auth: AuthState;
}

export function defaultAuth(): AuthState {
  return { type: "none", token: "", username: "", password: "",
           apiKeyHeader: "X-API-Key", apiKeyValue: "", apiKeyLocation: "header",
           provider: "frappe", authUrl: "", providerSession: null };
}

export const activeRequest = $state<RequestState>({
  id: uuid(), name: "New Request", method: "GET", url: "",
  headers: {}, params: {}, bodyType: "none", bodyContent: "", auth: defaultAuth(),
});

export const currentRequestId = $state<{ value: string }>({ value: activeRequest.id });

// ============================================================
// Response state
// ============================================================
export interface CookieInfo {
  name: string; value: string; domain?: string | null;
  path?: string | null; secure: boolean; httpOnly: boolean;
}
export interface ResponseState {
  status: number; statusText: string; headers: Record<string, string>;
  body: { raw: string; json: any | null; contentType: string };
  timing: { totalMs: number }; sizeBytes: number;
  cookies: CookieInfo[];
}
// Cache of last response per request name — used by {% response %} tag
export const responseCache = $state<Record<string, ResponseState>>({});
export const responseState = $state<{ loading: boolean; response: ResponseState | null; error: string | null }>({
  loading: false, response: null, error: null,
});

// ============================================================
// Visualizer state
// ============================================================
export const visualizerData = $state<{ template: string | null; data: any | null }>({
  template: null,
  data: null,
});

// ============================================================
// History
// ============================================================
export interface HistoryEntry {
  id: string; requestId: string; requestName: string;
  method: string; url: string; timestamp: number;
  status: number; durationMs: number; sizeBytes: number; response: ResponseState;
}
export const responseHistory = $state<HistoryEntry[]>([]);

// ============================================================
// Environment & Global Scopes
// ============================================================
export const activeEnvironment = $state<{ name: string; variables: Record<string, string> }>({
  name: "No Environment", variables: {},
});

export const globalVariables = $state<{ variables: Record<string, string> }>({
  variables: {},
});

export const collectionVariables = $state<{ variables: Record<string, string> }>({
  variables: {},
});

// ============================================================
// Tabs
// ============================================================
export interface RequestTab { id: string; name: string; method: HttpMethod; modified: boolean; }

function loadPersistedTabs(): { list: RequestTab[]; activeId: string } {
  try {
    const raw = localStorage.getItem("parallax:tabs");
    if (raw) return JSON.parse(raw);
  } catch { /* ignore */ }
  return { list: [{ id: "default", name: "New Request", method: "GET", modified: false }], activeId: "default" };
}

export const tabs = $state<{ list: RequestTab[]; activeId: string }>(loadPersistedTabs());

export function persistTabs() {
  try { localStorage.setItem("parallax:tabs", JSON.stringify({ list: tabs.list, activeId: tabs.activeId })); }
  catch { /* ignore */ }
}

// Per-tab request snapshots — keyed by tab id
const tabSnapshots = $state<Record<string, Partial<RequestState>>>({});

function snapshotDefault(): Partial<RequestState> {
  return { name: "New Request", method: "GET", url: "", headers: {}, params: {},
           bodyType: "none", bodyContent: "", auth: defaultAuth() };
}

/** Save current activeRequest into the snapshot for tabId */
export function saveTabSnapshot(tabId: string) {
  tabSnapshots[tabId] = {
    id: activeRequest.id, name: activeRequest.name, method: activeRequest.method,
    url: activeRequest.url, headers: { ...activeRequest.headers },
    params: { ...activeRequest.params }, bodyType: activeRequest.bodyType,
    bodyContent: activeRequest.bodyContent, auth: { ...activeRequest.auth },
  };
}

/** Restore activeRequest from snapshot for tabId (or blank if none) */
export function restoreTabSnapshot(tabId: string) {
  const snap = tabSnapshots[tabId] ?? snapshotDefault();
  activeRequest.id          = snap.id ?? tabId;
  activeRequest.name        = snap.name ?? "New Request";
  activeRequest.method      = snap.method ?? "GET";
  activeRequest.url         = snap.url ?? "";
  activeRequest.headers     = { ...(snap.headers ?? {}) };
  activeRequest.params      = { ...(snap.params ?? {}) };
  activeRequest.bodyType    = snap.bodyType ?? "none";
  activeRequest.bodyContent = snap.bodyContent ?? "";
  activeRequest.auth        = snap.auth ? { ...snap.auth } : defaultAuth();
  responseState.response    = null;
  responseState.error       = null;
}

export const authSessions = $state<Record<string, any>>({});

// ============================================================
// Scripts
// ============================================================
export interface ScriptState {
  preRequest: string;
  tests: string;
}
export const activeScripts = $state<ScriptState>({ preRequest: "", tests: "" });

export interface TestResult {
  name: string; passed: boolean; error?: string;
}
export const testResults = $state<{ results: TestResult[]; ran: boolean }>({ results: [], ran: false });
export const jsonNormalised = $state<{ value: boolean }>({ value: false });

// ============================================================
// Load a collection request into the active tab
// ============================================================
export function loadRequestIntoTab(req: CollectionRequest) {
  activeRequest.id          = req.id;
  activeRequest.name        = req.name;
  activeRequest.method      = req.method as HttpMethod;
  activeRequest.url         = req.url;
  activeRequest.headers     = { ...(req.headers ?? {}) };
  activeRequest.params      = { ...(req.params ?? {}) };
  activeRequest.auth        = defaultAuth();
  if (req.body && req.body.type && req.body.type !== "none") {
    activeRequest.bodyType    = req.body.type as any;
    activeRequest.bodyContent = req.body.raw ?? (req.body.content ? JSON.stringify(req.body.content, null, 2) : "");
  } else {
    activeRequest.bodyType = "none";
    activeRequest.bodyContent = "";
  }
  if (req.auth) {
    const a = req.auth;
    activeRequest.auth.type         = a.auth_type ?? "none";
    activeRequest.auth.token        = a.token ?? "";
    activeRequest.auth.username     = a.username ?? "";
    activeRequest.auth.password     = a.password ?? "";
    activeRequest.auth.apiKeyHeader   = a.api_key_header ?? "X-API-Key";
    activeRequest.auth.apiKeyValue    = a.api_key_value ?? "";
    activeRequest.auth.apiKeyLocation = (a.api_key_location === "query") ? "query" : "header";
    activeRequest.auth.provider     = a.provider ?? "frappe";
  }
  if (req.scripts) {
    activeScripts.preRequest = req.scripts.preRequest ?? "";
    activeScripts.tests = req.scripts.tests ?? "";
  } else {
    activeScripts.preRequest = "";
    activeScripts.tests = "";
  }

  // Load collection variables if we can find the parent collection
  const parentCol = loadedCollections.find(c =>
    c.requests.some(r => r.id === req.id) ||
    c.folders.some(f => f.requests.some(r => r.id === req.id))
  );
  if (parentCol && parentCol.variables) {
    collectionVariables.variables = { ...parentCol.variables };
  } else {
    collectionVariables.variables = {};
  }
  const existing = tabs.list.find((t) => t.id === req.id);
  if (existing) {
    tabs.activeId = req.id;
  } else {
    tabs.list.push({ id: req.id, name: req.name, method: req.method as HttpMethod, modified: false });
    tabs.activeId = req.id;
  }
  responseState.response = null;
  responseState.error    = null;
}

// ============================================================
// Curl parser
// ============================================================
export function parseCurl(raw: string): Partial<RequestState> | null {
  const s = raw.trim();
  if (!s.toLowerCase().startsWith("curl")) return null;
  const result: any = { method: "GET", headers: {}, params: {}, bodyType: "none", bodyContent: "" };
  const line = s.replace(/\\\s*\n/g, " ");

  const urlMatch = line.match(/curl\s+(?:[^\s]*\s+)*['"]?(https?:\/\/[^\s'"]+)['"]?/)
                ?? line.match(/['"]?(https?:\/\/[^\s'"]+)['"]?/);
  if (urlMatch) result.url = urlMatch[1];

  const methodMatch = line.match(/-X\s+['"]?([A-Z]+)['"]?/);
  if (methodMatch) result.method = methodMatch[1];

  const headerRe = /-H\s+['"]([^'"]+)['"]/g;
  let m: RegExpExecArray | null;
  while ((m = headerRe.exec(line)) !== null) {
    const ci = m[1].indexOf(":");
    if (ci > 0) {
      const k = m[1].slice(0, ci).trim();
      const v = m[1].slice(ci + 1).trim();
      result.headers[k] = v;
      if (k.toLowerCase() === "authorization" && v.toLowerCase().startsWith("bearer ")) {
        result.auth = { ...defaultAuth(), type: "bearer", token: v.slice(7) };
      }
    }
  }

  const bodyMatch = line.match(/(?:--data-raw|--data|-d)\s+['"]([^'"]*)['"]/s)
                 ?? line.match(/(?:--data-raw|--data|-d)\s+\$?['"]([^'"]*)['"]/s);
  if (bodyMatch) {
    if (result.method === "GET") result.method = "POST";
    const body = bodyMatch[1];
    try { JSON.parse(body); result.bodyType = "json"; result.bodyContent = body; }
    catch {
      const ct = Object.entries(result.headers as Record<string,string>)
        .find(([k]) => k.toLowerCase() === "content-type")?.[1] ?? "";
      result.bodyType    = ct.includes("x-www-form-urlencoded") ? "urlencoded" : "raw";
      result.bodyContent = body;
    }
  }
  return result.url ? result : null;
}

// ============================================================
// Send Request
// ============================================================
// ============================================================
// File-tag pre-processor (handles {% file '/path' %} before template engine)
// ============================================================
async function resolveFileTags(text: string): Promise<string> {
  const re = /\{%\s*file\s+['"]([^'"]+)['"]\s*%\}/g;
  const matches = [...text.matchAll(re)];
  let result = text;
  for (const m of matches) {
    try {
      const content = await readFileForTemplate(m[1]);
      result = result.replace(m[0], content);
    } catch { /* leave the tag as-is if file unreadable */ }
  }
  return result;
}

async function resolveFileTagsInObj(obj: any): Promise<any> {
  if (typeof obj === "string") return resolveFileTags(obj);
  if (Array.isArray(obj)) return Promise.all(obj.map(resolveFileTagsInObj));
  if (obj && typeof obj === "object") {
    const out: any = {};
    for (const [k, v] of Object.entries(obj)) out[k] = await resolveFileTagsInObj(v);
    return out;
  }
  return obj;
}

// ============================================================
// Cancel active request
// ============================================================
export async function cancelRequest() {
  const id = activeRequest.id;
  try { await platformCancelRequest(id); } catch { /* ignore */ }
  responseState.loading = false;
  responseState.error   = "Request cancelled";
}

export async function sendRequest() {
  if (!activeRequest.url) return;
  const raw = activeRequest.url.trim();

  // Detect pasted curl command
  if (raw.toLowerCase().startsWith("curl ")) {
    const parsed = parseCurl(raw);
    if (parsed) {
      if (parsed.method)      activeRequest.method      = parsed.method;
      if (parsed.url)         activeRequest.url         = parsed.url;
      if (parsed.headers)     activeRequest.headers     = parsed.headers;
      if (parsed.params)      activeRequest.params      = parsed.params;
      if (parsed.bodyType)    activeRequest.bodyType    = parsed.bodyType;
      if (parsed.bodyContent !== undefined) activeRequest.bodyContent = parsed.bodyContent;
      if (parsed.auth)        Object.assign(activeRequest.auth, parsed.auth);
      const tab = tabs.list.find((t) => t.id === tabs.activeId);
      if (tab && parsed.method) tab.method = parsed.method;
      return; // show populated fields, don't fire the request yet
    }
  }

  responseState.loading  = true;
  responseState.error    = null;
  responseState.response = null;
  testResults.results     = [];
  testResults.ran         = false;
  visualizerData.template = null;
  visualizerData.data     = null;
  jsonNormalised.value    = false;
  const t0 = Date.now();

  try {
    // Merge scopes: Global -> Collection -> Environment
    // Local script variables will mutate this merged object during pre-request
    const rawEnv = {
      ...globalVariables.variables,
      ...collectionVariables.variables,
      ...activeEnvironment.variables,
    };
    // Resolve {% shell 'cmd' %} tags in env values first (output may contain {{vars}})
    const mergedEnv = await resolveShellTagsInObj(rawEnv) as Record<string, string>;

    // Run pre-request script
    if (activeScripts.preRequest.trim()) {
      const { runPreRequestScript } = await import("../utils/pm-script-runner");
      await runPreRequestScript(activeScripts.preRequest, mergedEnv);
    }

    // Resolve shell tags in payload before file tags and template resolution
    const rawPayload = await resolveShellTagsInObj(await resolveFileTagsInObj(buildPayload()));
    // Ensure varjson requests carry the correct content-type header
    if (activeRequest.bodyType === "varjson" && !rawPayload.headers?.["Content-Type"]) {
      rawPayload.headers = { ...rawPayload.headers, "Content-Type": "application/json" };
    }
    const payload = resolveRequestTemplates(rawPayload, mergedEnv, { ...responseCache });

    // Lenient JSON normalisation: fix single-quotes, trailing commas, etc. before dispatch
    if (payload.body?.type === "json" && payload.body?.raw && !payload.body.raw.includes("{{")) {
      try {
        const JSON5 = (await import("json5")).default;
        const normalised = JSON.stringify(JSON5.parse(payload.body.raw));
        if (normalised !== payload.body.raw) {
          payload.body.raw = normalised;
          payload.body.content = JSON.parse(normalised);
          jsonNormalised.value = true;
        }
      } catch {
        // Body isn't parseable even with json5 — send as-is, server will reject
      }
    } else {
      jsonNormalised.value = false;
    }

    const result = await platformSendRequest(payload, mergedEnv);

    const response: ResponseState = {
      status:     result.status,
      statusText: result.status_text ?? "",
      headers:    result.headers ?? {},
      body: {
        raw:         result.body?.raw ?? "",
        json:        result.body?.json ?? null,
        contentType: result.body?.content_type ?? "",
      },
      timing:    { totalMs: result.timing?.total_ms ?? (Date.now() - t0) },
      sizeBytes: result.size_bytes ?? 0,
      cookies:   (result.cookies ?? []).map((c: any) => ({
        name: c.name, value: c.value, domain: c.domain ?? null,
        path: c.path ?? null, secure: c.secure ?? false, httpOnly: c.http_only ?? false,
      })),
    };

    responseState.response = response;
    // Cache response by request name for {% response %} tag chaining
    responseCache[activeRequest.name] = response;

    // Run test script
    if (activeScripts.tests.trim()) {
      const { runTestScript } = await import("../utils/pm-script-runner");
      const { tests, visualizer } = await runTestScript(activeScripts.tests, response, mergedEnv);
      testResults.results = tests;
      testResults.ran     = true;
      if (visualizer.template) {
        visualizerData.template = visualizer.template;
        visualizerData.data = visualizer.data;
      }
    }

    responseHistory.unshift({
      id:          uuid(),
      requestId:   activeRequest.id,
      requestName: activeRequest.name,
      method:      activeRequest.method,
      url:         activeRequest.url,
      timestamp:   Date.now(),
      status:      response.status,
      durationMs:  response.timing.totalMs,
      sizeBytes:   response.sizeBytes,
      response,
    });
    if (responseHistory.length > 200) responseHistory.splice(200);

  } catch (err: any) {
    responseState.error = err?.toString() ?? "Unknown error";
  } finally {
    responseState.loading = false;
  }
}

function buildPayload() {
  return {
    id: activeRequest.id, name: activeRequest.name, method: activeRequest.method,
    url: activeRequest.url, headers: activeRequest.headers, params: activeRequest.params,
    body: buildBody(), auth: buildAuth(), timeout_ms: 30000, follow_redirects: true,
    scripts: { preRequest: activeScripts.preRequest, tests: activeScripts.tests }
  };
}
function varjsonToJson(raw: string): string {
  const obj: Record<string, string> = {};
  for (const line of raw.split("\n")) {
    const m = line.trim().match(/^([\w-]+)=(.*)$/);
    if (m) obj[m[1]] = m[2];
  }
  return JSON.stringify(obj);
}

function buildBody() {
  if (activeRequest.bodyType === "none") return null;
  if (activeRequest.bodyType === "varjson") {
    const jsonStr = varjsonToJson(activeRequest.bodyContent);
    return { type: "json", content: tryParse(jsonStr), raw: jsonStr };
  }
  return { type: activeRequest.bodyType, content: tryParse(activeRequest.bodyContent), raw: activeRequest.bodyContent };
}
function buildAuth() {
  const a = activeRequest.auth;
  if (a.type === "none") return null;
  return {
    auth_type: a.type, token: a.token || null, username: a.username || null,
    password: a.password || null, api_key_header: a.apiKeyHeader || null,
    api_key_value: a.apiKeyValue || null, api_key_location: a.apiKeyLocation || "header",
    provider: a.provider || null,
    provider_session: a.providerSession || null,
  };
}
function tryParse(s: string) { try { return JSON.parse(s); } catch { return s; } }
