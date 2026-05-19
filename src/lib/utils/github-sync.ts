// Parallax GitHub Sync
//
// Reads and writes .parallax/ collections and environments via the GitHub
// Contents API (https://docs.github.com/en/rest/repos/contents).
//
// Auth flow:
//   Desktop (Tauri)  → GitHub Device Flow  (no redirect needed)
//   Browser (Web)    → GitHub OAuth redirect (VITE_GITHUB_CLIENT_ID must be set)
//
// Collections are stored as YAML files under:
//   <repo>/.parallax/collections/<name>.yaml
// Environments as JSON under:
//   <repo>/.parallax/environments/<name>.json

const GH_API = "https://api.github.com";
const DEVICE_CODE_URL = "https://github.com/login/device/code";
const TOKEN_URL = "https://github.com/login/oauth/access_token";
const OAUTH_REDIRECT_URL = "https://github.com/login/oauth/authorize";

const SCOPES = "repo";

// ── Token storage (localStorage, scoped per origin) ───────────────────────────

const TOKEN_KEY = "parallax:github_token";
const REPO_KEY  = "parallax:github_repo";  // "owner/repo"

export function getStoredToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function setStoredToken(token: string) {
  localStorage.setItem(TOKEN_KEY, token);
}

export function clearStoredToken() {
  localStorage.removeItem(TOKEN_KEY);
}

export function getStoredRepo(): string | null {
  return localStorage.getItem(REPO_KEY);
}

export function setStoredRepo(repo: string) {
  localStorage.setItem(REPO_KEY, repo);
}

// ── Auth: Device Flow (Desktop / CLI) ─────────────────────────────────────────

export interface DeviceFlowStart {
  userCode: string;       // "XXXX-XXXX" shown to user
  verificationUri: string;
  interval: number;       // seconds between polls
  deviceCode: string;     // used internally for polling
}

export async function startDeviceFlow(clientId: string): Promise<DeviceFlowStart> {
  const res = await fetch(DEVICE_CODE_URL, {
    method: "POST",
    headers: { Accept: "application/json", "Content-Type": "application/json" },
    body: JSON.stringify({ client_id: clientId, scope: SCOPES }),
  });
  if (!res.ok) throw new Error(`Device flow start failed: ${res.status}`);
  const data = await res.json();
  return {
    userCode:        data.user_code,
    verificationUri: data.verification_uri,
    interval:        data.interval ?? 5,
    deviceCode:      data.device_code,
  };
}

export async function pollDeviceFlow(
  clientId: string,
  deviceCode: string,
  interval: number,
): Promise<string> {
  // Polls until the user authorises or the code expires (max 15 min)
  const deadline = Date.now() + 15 * 60 * 1000;
  while (Date.now() < deadline) {
    await new Promise(r => setTimeout(r, interval * 1000));
    const res = await fetch(TOKEN_URL, {
      method: "POST",
      headers: { Accept: "application/json", "Content-Type": "application/json" },
      body: JSON.stringify({
        client_id:   clientId,
        device_code: deviceCode,
        grant_type:  "urn:ietf:params:oauth:grant-type:device_code",
      }),
    });
    const data = await res.json();
    if (data.access_token) return data.access_token as string;
    if (data.error === "access_denied") throw new Error("User denied access");
    // "authorization_pending" or "slow_down" → keep polling
    if (data.error === "slow_down") interval = (data.interval ?? interval) + 5;
  }
  throw new Error("Device flow timed out");
}

// ── Auth: OAuth redirect (Browser) ───────────────────────────────────────────

export function startOAuthRedirect(clientId: string, redirectUri: string) {
  const state = crypto.randomUUID();
  sessionStorage.setItem("parallax:oauth_state", state);
  const url = new URL(OAUTH_REDIRECT_URL);
  url.searchParams.set("client_id", clientId);
  url.searchParams.set("redirect_uri", redirectUri);
  url.searchParams.set("scope", SCOPES);
  url.searchParams.set("state", state);
  window.location.href = url.toString();
}

// Called after GitHub redirects back to ?code=...&state=...
// The code must be exchanged server-side (or via a CORS proxy) because
// the OAuth token endpoint does not set Access-Control-Allow-Origin.
export async function finishOAuthRedirect(
  proxyExchangeUrl: string,
): Promise<string | null> {
  const params = new URLSearchParams(window.location.search);
  const code  = params.get("code");
  const state = params.get("state");
  if (!code || !state) return null;

  const storedState = sessionStorage.getItem("parallax:oauth_state");
  if (state !== storedState) throw new Error("OAuth state mismatch");
  sessionStorage.removeItem("parallax:oauth_state");

  const res = await fetch(proxyExchangeUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ code }),
  });
  if (!res.ok) throw new Error(`Token exchange failed: ${res.status}`);
  const data = await res.json();
  if (!data.access_token) throw new Error("No access_token in exchange response");
  return data.access_token as string;
}

// ── GitHub Contents API helpers ───────────────────────────────────────────────

async function ghFetch(path: string, token: string, opts?: RequestInit): Promise<Response> {
  const res = await fetch(`${GH_API}${path}`, {
    ...opts,
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
      ...(opts?.headers ?? {}),
    },
  });
  return res;
}

interface GhFileContent {
  name: string;
  path: string;
  sha: string;
  content: string; // base64
  encoding: string;
}

export async function readGhFile(
  repo: string,
  filePath: string,
  token: string,
  ref?: string,
): Promise<{ content: string; sha: string }> {
  const qs = ref ? `?ref=${encodeURIComponent(ref)}` : "";
  const res = await ghFetch(`/repos/${repo}/contents/${filePath}${qs}`, token);
  if (res.status === 404) throw new Error(`File not found: ${filePath}`);
  if (!res.ok) throw new Error(`GitHub read error ${res.status}: ${await res.text()}`);
  const data: GhFileContent = await res.json();
  const content = atob(data.content.replace(/\n/g, ""));
  return { content, sha: data.sha };
}

export async function writeGhFile(
  repo: string,
  filePath: string,
  content: string,
  message: string,
  token: string,
  sha?: string,   // required for updates, omit for creates
): Promise<string> {
  const body: Record<string, any> = {
    message,
    content: btoa(unescape(encodeURIComponent(content))),
  };
  if (sha) body.sha = sha;

  const res = await ghFetch(`/repos/${repo}/contents/${filePath}`, token, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`GitHub write error ${res.status}: ${await res.text()}`);
  const data = await res.json();
  return data.content.sha as string;
}

export async function listGhDir(
  repo: string,
  dirPath: string,
  token: string,
): Promise<{ name: string; path: string; sha: string; type: "file" | "dir" }[]> {
  const res = await ghFetch(`/repos/${repo}/contents/${dirPath}`, token);
  if (res.status === 404) return [];
  if (!res.ok) throw new Error(`GitHub list error ${res.status}: ${await res.text()}`);
  const data = await res.json();
  return Array.isArray(data) ? data : [];
}

// ── High-level sync operations ────────────────────────────────────────────────

export interface SyncedCollection {
  name: string;
  yaml: string;
  sha: string;
}

export interface SyncedEnvironment {
  name: string;
  json: string;
  sha: string;
}

/** Pull all collections from the GitHub repo. */
export async function pullCollections(
  repo: string,
  token: string,
): Promise<SyncedCollection[]> {
  const entries = await listGhDir(repo, ".parallax/collections", token);
  const yamls = entries.filter(e => e.type === "file" && e.name.endsWith(".yaml"));
  return Promise.all(
    yamls.map(async e => {
      const { content, sha } = await readGhFile(repo, e.path, token);
      return { name: e.name.replace(/\.yaml$/, ""), yaml: content, sha };
    }),
  );
}

/** Push a single collection YAML to GitHub (creates or updates). */
export async function pushCollection(
  repo: string,
  name: string,
  yaml: string,
  token: string,
  existingSha?: string,
): Promise<string> {
  const path = `.parallax/collections/${name}.yaml`;
  const msg  = existingSha ? `chore: update collection ${name}` : `chore: add collection ${name}`;
  return writeGhFile(repo, path, yaml, msg, token, existingSha);
}

/** Pull all environments from the GitHub repo. */
export async function pullEnvironments(
  repo: string,
  token: string,
): Promise<SyncedEnvironment[]> {
  const entries = await listGhDir(repo, ".parallax/environments", token);
  const jsons = entries.filter(e => e.type === "file" && e.name.endsWith(".json"));
  return Promise.all(
    jsons.map(async e => {
      const { content, sha } = await readGhFile(repo, e.path, token);
      return { name: e.name.replace(/\.json$/, ""), json: content, sha };
    }),
  );
}

/** Push a single environment JSON to GitHub (creates or updates). */
export async function pushEnvironment(
  repo: string,
  name: string,
  json: string,
  token: string,
  existingSha?: string,
): Promise<string> {
  const path = `.parallax/environments/${name}.json`;
  const msg  = existingSha ? `chore: update env ${name}` : `chore: add env ${name}`;
  return writeGhFile(repo, path, json, msg, token, existingSha);
}
