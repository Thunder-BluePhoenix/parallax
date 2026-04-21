// Parallax Global App State — Svelte 5 Runes
import { invoke } from "@tauri-apps/api/core";

// ============================================================
// App Mode
// ============================================================
export const appMode = $state<{ value: "builder" | "dashboard" }>({
  value: "builder",
});

// ============================================================
// Current Workspace
// ============================================================
export const currentWorkspace = $state<{
  path: string;
  name: string;
  gitBranch: string | null;
  hasParallax: boolean;
}>({
  path: "",
  name: "No Workspace",
  gitBranch: null,
  hasParallax: false,
});

// ============================================================
// Active Request (for Builder Mode)
// ============================================================
export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS";

export interface RequestState {
  id: string;
  name: string;
  method: HttpMethod;
  url: string;
  headers: Record<string, string>;
  params: Record<string, string>;
  bodyType: "none" | "json" | "form" | "urlencoded" | "raw" | "graphql";
  bodyContent: string;
  auth: AuthState;
}

export interface AuthState {
  type: "none" | "bearer" | "basic" | "api_key" | "ecosystem_provider";
  token: string;
  username: string;
  password: string;
  apiKeyHeader: string;
  apiKeyValue: string;
  provider: string;
  providerSession: any | null;
}

export const activeRequest = $state<RequestState>({
  id: crypto.randomUUID(),
  name: "New Request",
  method: "GET",
  url: "",
  headers: {},
  params: {},
  bodyType: "none",
  bodyContent: "",
  auth: {
    type: "none",
    token: "",
    username: "",
    password: "",
    apiKeyHeader: "X-API-Key",
    apiKeyValue: "",
    provider: "frappe",
    providerSession: null,
  },
});

// ============================================================
// Response State
// ============================================================
export interface ResponseState {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: {
    raw: string;
    json: any | null;
    contentType: string;
  };
  timing: {
    totalMs: number;
  };
  sizeBytes: number;
}

export const responseState = $state<{
  loading: boolean;
  response: ResponseState | null;
  error: string | null;
}>({
  loading: false,
  response: null,
  error: null,
});

// ============================================================
// Active Environment
// ============================================================
export const activeEnvironment = $state<{
  name: string;
  variables: Record<string, string>;
}>({
  name: "dev",
  variables: {},
});

// ============================================================
// Tabs (open requests)
// ============================================================
export interface RequestTab {
  id: string;
  name: string;
  method: HttpMethod;
  modified: boolean;
}

export const tabs = $state<{ list: RequestTab[]; activeId: string }>({
  list: [
    { id: "default", name: "New Request", method: "GET", modified: false },
  ],
  activeId: "default",
});

// ============================================================
// Ecosystem Auth Sessions
// ============================================================
export const authSessions = $state<Record<string, any>>({});

// ============================================================
// Actions
// ============================================================
export async function sendRequest() {
  if (!activeRequest.url) return;

  responseState.loading = true;
  responseState.error = null;
  responseState.response = null;

  try {
    const req = buildRequestPayload();
    const env = { ...activeEnvironment.variables };

    const result: ResponseState = await invoke("send_request", {
      request: req,
      environment: env,
    });

    responseState.response = result;
  } catch (err: any) {
    responseState.error = err?.toString() ?? "Unknown error";
  } finally {
    responseState.loading = false;
  }
}

function buildRequestPayload() {
  return {
    id: activeRequest.id,
    name: activeRequest.name,
    method: activeRequest.method,
    url: activeRequest.url,
    headers: activeRequest.headers,
    params: activeRequest.params,
    body: buildBody(),
    auth: buildAuth(),
    timeout_ms: 30000,
    follow_redirects: true,
  };
}

function buildBody() {
  if (activeRequest.bodyType === "none") return null;
  return {
    type: activeRequest.bodyType,
    content: tryParse(activeRequest.bodyContent),
    raw: activeRequest.bodyContent,
  };
}

function buildAuth() {
  const a = activeRequest.auth;
  if (a.type === "none") return null;
  return {
    auth_type: a.type,
    token: a.token || null,
    username: a.username || null,
    password: a.password || null,
    api_key_header: a.apiKeyHeader || null,
    api_key_value: a.apiKeyValue || null,
    provider: a.provider || null,
    provider_session: a.providerSession || null,
  };
}

function tryParse(s: string) {
  try { return JSON.parse(s); } catch { return s; }
}
