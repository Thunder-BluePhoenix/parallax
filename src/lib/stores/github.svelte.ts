// GitHub auth + chat state — Svelte 5 runes
import { invoke } from "@tauri-apps/api/core";

export interface GitHubIdentity {
  login: string;
  name: string | null;
  email: string | null;
  avatar_url: string;
  token: string;
}

export interface ChatMessage {
  id: string;
  workspace_id: string;
  sender: string;
  sender_name: string;
  body: string;
  ts: number;
}

export interface ChatPresence {
  login: string;
  name: string;
  ts: number;
}

// ── Identity ──────────────────────────────────────────────────
export const githubIdentity = $state<{ value: GitHubIdentity | null }>({ value: null });

export async function loadGitHubIdentity() {
  try {
    const id: GitHubIdentity | null = await invoke("github_get_identity");
    githubIdentity.value = id;
  } catch {
    githubIdentity.value = null;
  }
}

export async function signOutGitHub() {
  await invoke("github_sign_out");
  githubIdentity.value = null;
}

// ── Device OAuth flow ─────────────────────────────────────────
export interface DeviceCodeInfo {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

export async function startGitHubLogin(clientId: string): Promise<DeviceCodeInfo> {
  return invoke("github_device_auth_start", { clientId: clientId || undefined });
}

export async function pollGitHubLogin(
  deviceCode: string,
  clientId: string
): Promise<GitHubIdentity | null> {
  const id: GitHubIdentity | null = await invoke("github_device_auth_poll", {
    deviceCode,
    clientId: clientId || undefined,
  });
  if (id) githubIdentity.value = id;
  return id;
}

// ── Chat ──────────────────────────────────────────────────────
export const chatMessages = $state<ChatMessage[]>([]);
export const chatPresence = $state<ChatPresence[]>([]);
export const chatEnabled = $state<{ value: boolean }>({ value: false });
export const unreadCount = $state<{ value: number }>({ value: 0 });

let presenceInterval: ReturnType<typeof setInterval> | null = null;

export async function enableChat(workspace: string) {
  if (!githubIdentity.value) return;
  chatEnabled.value = true;
  unreadCount.value = 0;

  // Load history
  try {
    const history: ChatMessage[] = await invoke("chat_get_history", { workspace });
    chatMessages.splice(0, chatMessages.length, ...history);
  } catch { /* chat service may not be running yet */ }

  // Start SSE stream
  try {
    await invoke("chat_start_stream", { workspace });
  } catch { /* non-fatal */ }

  // Announce presence and refresh every 90s
  const announcePresence = async () => {
    if (!githubIdentity.value) return;
    try {
      await invoke("chat_set_presence", {
        workspace,
        login: githubIdentity.value.login,
        name: githubIdentity.value.name ?? githubIdentity.value.login,
      });
      const presence: ChatPresence[] = await invoke("chat_get_presence", { workspace });
      chatPresence.splice(0, chatPresence.length, ...presence);
    } catch { /* non-fatal */ }
  };
  announcePresence();
  presenceInterval = setInterval(announcePresence, 90_000);
}

export function disableChat() {
  chatEnabled.value = false;
  if (presenceInterval) {
    clearInterval(presenceInterval);
    presenceInterval = null;
  }
  chatMessages.splice(0, chatMessages.length);
  chatPresence.splice(0, chatPresence.length);
}

export function appendChatMessage(msg: ChatMessage) {
  chatMessages.push(msg);
  if (!document.hasFocus()) {
    unreadCount.value++;
  }
}

export function clearUnread() {
  unreadCount.value = 0;
}
