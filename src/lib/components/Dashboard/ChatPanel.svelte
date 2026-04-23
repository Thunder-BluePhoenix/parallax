<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy, tick } from "svelte";
  import { currentWorkspace } from "../../stores/app.svelte";
  import {
    githubIdentity,
    chatMessages,
    chatPresence,
    chatEnabled,
    unreadCount,
    enableChat,
    disableChat,
    appendChatMessage,
    clearUnread,
    type ChatMessage,
  } from "../../stores/github.svelte";

  let messageInput = $state("");
  let sending = $state(false);
  let listEl = $state<HTMLDivElement | null>(null);
  let unlisten: (() => void) | null = null;

  // Relay / polling state
  let relayMode = $state(false);
  let lastSyncTime = $state<Date | null>(null);
  let syncing = $state(false);
  let pollInterval: ReturnType<typeof setInterval> | null = null;
  let lastActivityTs = $state(Date.now());

  const ws = $derived(currentWorkspace.path);
  const isLoggedIn = $derived(!!githubIdentity.value);

  async function scrollToBottom() {
    await tick();
    if (listEl) listEl.scrollTop = listEl.scrollHeight;
  }

  async function sendMessage() {
    if (!ws || !messageInput.trim() || !githubIdentity.value || sending) return;
    sending = true;
    const body = messageInput.trim();
    messageInput = "";
    try {
      await invoke("chat_send_message", {
        workspace: ws,
        sender: githubIdentity.value.login,
        senderName: githubIdentity.value.name ?? githubIdentity.value.login,
        body,
      });
    } catch {
      // Optimistically add for local display (picked up on next sync)
      appendChatMessage({
        id: crypto.randomUUID(),
        workspace_id: ws,
        sender: githubIdentity.value.login,
        sender_name: githubIdentity.value.name ?? githubIdentity.value.login,
        body,
        ts: Math.floor(Date.now() / 1000),
      });
    } finally {
      sending = false;
      scrollToBottom();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  // ── Polling fallback (dedup by ID) ────────────────────────
  async function pollHistory() {
    if (!ws) return;
    try {
      const history: ChatMessage[] = await invoke("chat_get_history", { workspace: ws });
      const known = new Set(chatMessages.map((m) => m.id));
      let added = false;
      for (const msg of history) {
        if (!known.has(msg.id)) {
          appendChatMessage(msg);
          added = true;
        }
      }
      if (added) scrollToBottom();
      lastActivityTs = Date.now();
    } catch { /* chat service may be down */ }
  }

  function startPoll() {
    if (pollInterval) return;
    pollInterval = setInterval(pollHistory, 30_000);
  }

  function stopPoll() {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  }

  // ── Git relay sync ────────────────────────────────────────
  async function gitRelaySync() {
    if (!ws || !githubIdentity.value) return;
    syncing = true;
    try {
      // Pull latest from remote to get teammates' messages
      await invoke("git_pull", {
        path: ws,
        remoteName: "origin",
        branch: "main",
        token: githubIdentity.value.token || null,
      });
    } catch { /* remote may not be configured */ }

    // Reload history after pull
    await pollHistory();

    // Auto-commit and push any new local messages
    try {
      const name = githubIdentity.value.name ?? githubIdentity.value.login;
      const email = githubIdentity.value.email ?? `${githubIdentity.value.login}@users.noreply.github.com`;
      await invoke("git_commit", {
        path: ws,
        message: "chat: sync messages",
        authorName: name,
        authorEmail: email,
      });
      await invoke("git_push", {
        path: ws,
        remoteName: "origin",
        branch: "main",
        token: githubIdentity.value.token || null,
      });
    } catch { /* no changes or remote not configured */ }

    lastSyncTime = new Date();
    syncing = false;
  }

  // ── Presence + relay mode detection ──────────────────────
  $effect(() => {
    if (!chatEnabled.value) return;
    // If no SSE activity for 45s, show relay mode indicator
    const check = setInterval(() => {
      relayMode = Date.now() - lastActivityTs > 45_000;
    }, 10_000);
    return () => clearInterval(check);
  });

  function formatTime(ts: number) {
    return new Date(ts * 1000).toLocaleTimeString([], {
      hour12: false, hour: "2-digit", minute: "2-digit",
    });
  }

  function initials(name: string) {
    return name.split(" ").map((w) => w[0]).slice(0, 2).join("").toUpperCase();
  }

  function isOnline(login: string) {
    const cutoff = Math.floor(Date.now() / 1000) - 300;
    return chatPresence.some((p) => p.login === login && p.ts > cutoff);
  }

  onMount(async () => {
    unlisten = await listen<ChatMessage>("chat_message", (event) => {
      appendChatMessage(event.payload);
      lastActivityTs = Date.now();
      scrollToBottom();
    });
    startPoll();
  });

  onDestroy(() => {
    if (unlisten) unlisten();
    stopPoll();
  });

  $effect(() => {
    if (chatMessages.length > 0) scrollToBottom();
  });

  $effect(() => {
    if (chatEnabled.value) clearUnread();
  });
</script>

<div class="dashboard-section animate-fade-in">
  <div class="section-header">
    <h2>
      Team Chat
      {#if unreadCount.value > 0}
        <span class="unread-badge">{unreadCount.value}</span>
      {/if}
    </h2>
    <p class="section-desc">
      P2P on the same network · git-relay fallback for remote teams · messages in
      <code>.parallax/chat/messages.jsonl</code>
    </p>
  </div>

  {#if !isLoggedIn}
    <div class="empty-prompt">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.4">
        <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/>
      </svg>
      <p>Sign in with GitHub (Team panel) to use team chat.</p>
    </div>
  {:else if !chatEnabled.value}
    <div class="empty-prompt">
      <p>Chat is disabled for this workspace.</p>
      <button class="btn-primary" onclick={() => ws && enableChat(ws)}>Enable Chat</button>
    </div>
  {:else}
    <div class="chat-layout">
      <!-- Presence sidebar -->
      <div class="presence-panel">
        <div class="presence-title">Online</div>
        {#each chatPresence as p}
          <div class="presence-user">
            <div class="avatar sm" class:online={isOnline(p.login)}>{initials(p.name || p.login)}</div>
            <span class="presence-name">{p.name || p.login}</span>
          </div>
        {/each}
        {#if chatPresence.length === 0}
          <div class="presence-empty">Just you</div>
        {/if}

        <div class="presence-sep"></div>

        <!-- Relay status + sync -->
        {#if relayMode}
          <div class="relay-badge">Git Relay</div>
        {/if}
        <button
          class="sync-btn"
          onclick={gitRelaySync}
          disabled={syncing}
          title="Pull + push chat via git"
        >
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:spin={syncing}>
            <polyline points="1 4 1 10 7 10"/>
            <polyline points="23 20 23 14 17 14"/>
            <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15"/>
          </svg>
          {syncing ? "Syncing…" : "Sync via Git"}
        </button>
        {#if lastSyncTime}
          <div class="sync-time">{lastSyncTime.toLocaleTimeString([], { hour12: false, hour: "2-digit", minute: "2-digit" })}</div>
        {/if}

        <button class="btn-disable" onclick={disableChat}>Disable</button>
      </div>

      <!-- Messages -->
      <div class="chat-main">
        <div class="message-list" bind:this={listEl}>
          {#if chatMessages.length === 0}
            <div class="chat-empty">No messages yet. Say hello!</div>
          {:else}
            {#each chatMessages as msg (msg.id)}
              {@const isMine = msg.sender === githubIdentity.value?.login}
              <div class="message-row" class:mine={isMine}>
                {#if !isMine}
                  <div class="avatar" class:online={isOnline(msg.sender)}>
                    {initials(msg.sender_name || msg.sender)}
                  </div>
                {/if}
                <div class="message-content" class:mine={isMine}>
                  {#if !isMine}
                    <div class="message-meta">
                      <span class="sender">{msg.sender_name || msg.sender}</span>
                      <span class="time">{formatTime(msg.ts)}</span>
                    </div>
                  {/if}
                  <div class="bubble" class:mine={isMine}>{msg.body}</div>
                  {#if isMine}
                    <div class="time mine">{formatTime(msg.ts)}</div>
                  {/if}
                </div>
              </div>
            {/each}
          {/if}
        </div>

        <div class="input-row">
          <textarea
            class="message-input"
            placeholder="Message your team… (Enter to send)"
            bind:value={messageInput}
            onkeydown={handleKeydown}
            rows="2"
            disabled={sending}
          ></textarea>
          <button
            class="send-btn"
            onclick={sendMessage}
            disabled={sending || !messageInput.trim()}
            aria-label="Send message"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <line x1="22" y1="2" x2="11" y2="13"/>
              <polygon points="22 2 15 22 11 13 2 9 22 2"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .dashboard-section { display: flex; flex-direction: column; height: 100%; }
  .section-header { margin-bottom: 16px; flex-shrink: 0; }
  .section-header h2 { font-size: 18px; font-weight: 700; margin-bottom: 4px; display: flex; align-items: center; gap: 8px; }
  .section-desc { font-size: 12px; color: var(--text-secondary); }
  .section-desc code { color: var(--accent-primary); font-size: 11px; }

  .unread-badge {
    background: var(--color-error); color: white; font-size: 10px; font-weight: 700;
    padding: 1px 6px; border-radius: 10px; min-width: 18px; text-align: center;
  }

  .empty-prompt {
    display: flex; flex-direction: column; align-items: center; gap: 16px;
    padding: 48px 24px; text-align: center; color: var(--text-muted);
    border: 1px dashed var(--border-default); border-radius: var(--radius-md);
    font-size: 13px;
  }

  .btn-primary {
    height: 34px; padding: 0 20px; background: var(--accent-primary); color: white;
    border: none; border-radius: var(--radius-md); font-weight: 600; font-size: 13px;
    cursor: pointer;
  }
  .btn-primary:hover { filter: brightness(1.1); }

  .chat-layout {
    display: flex; flex: 1; min-height: 0;
    border: 1px solid var(--border-default); border-radius: var(--radius-lg); overflow: hidden;
  }

  /* Presence sidebar */
  .presence-panel {
    width: 140px; flex-shrink: 0; background: var(--bg-surface);
    border-right: 1px solid var(--border-default); padding: 12px 8px;
    display: flex; flex-direction: column; gap: 6px;
  }
  .presence-title { font-size: 10px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: .06em; margin-bottom: 4px; }
  .presence-user { display: flex; align-items: center; gap: 6px; }
  .presence-name { font-size: 11px; color: var(--text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .presence-empty { font-size: 11px; color: var(--text-muted); }
  .presence-sep { border-top: 1px solid var(--border-subtle); margin: 6px 0; }

  .relay-badge {
    font-size: 9px; font-weight: 700; color: var(--color-warning);
    background: rgba(210, 153, 34, 0.15); border: 1px solid rgba(210,153,34,0.3);
    padding: 2px 6px; border-radius: 10px; text-align: center;
  }
  .sync-btn {
    display: flex; align-items: center; gap: 5px; font-size: 10px; font-weight: 600;
    color: var(--text-muted); background: none; border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm); cursor: pointer; padding: 4px 6px;
    transition: var(--transition-fast);
  }
  .sync-btn:hover:not(:disabled) { border-color: var(--accent-primary); color: var(--accent-primary); }
  .sync-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .sync-time { font-size: 9px; color: var(--text-muted); }

  @keyframes spin { to { transform: rotate(360deg); } }
  .spin { animation: spin 1s linear infinite; }

  .btn-disable {
    margin-top: auto; font-size: 10px; color: var(--text-muted); background: none;
    border: none; cursor: pointer; text-align: left; padding: 4px 0;
    transition: var(--transition-fast);
  }
  .btn-disable:hover { color: var(--color-error); }

  /* Avatar */
  .avatar {
    width: 26px; height: 26px; border-radius: 50%; background: var(--accent-primary-dim);
    color: var(--accent-primary); font-size: 10px; font-weight: 700;
    display: flex; align-items: center; justify-content: center; flex-shrink: 0;
    position: relative;
  }
  .avatar.sm { width: 22px; height: 22px; font-size: 9px; }
  .avatar.online::after {
    content: ""; position: absolute; bottom: 0; right: 0;
    width: 7px; height: 7px; border-radius: 50%;
    background: var(--color-success); border: 1.5px solid var(--bg-surface);
  }

  /* Messages */
  .chat-main { flex: 1; display: flex; flex-direction: column; min-width: 0; }
  .message-list {
    flex: 1; overflow-y: auto; padding: 16px 12px;
    display: flex; flex-direction: column; gap: 12px;
  }
  .chat-empty { text-align: center; color: var(--text-muted); font-size: 12px; margin: auto; }

  .message-row { display: flex; gap: 8px; align-items: flex-end; }
  .message-row.mine { flex-direction: row-reverse; }
  .message-content { display: flex; flex-direction: column; gap: 3px; max-width: 70%; }
  .message-content.mine { align-items: flex-end; }
  .message-meta { display: flex; align-items: baseline; gap: 6px; }
  .sender { font-size: 11px; font-weight: 600; color: var(--text-secondary); }
  .time { font-size: 10px; color: var(--text-muted); }
  .time.mine { font-size: 10px; color: var(--text-muted); }

  .bubble {
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: 12px 12px 12px 3px; padding: 7px 12px;
    font-size: 13px; color: var(--text-primary); line-height: 1.4; word-break: break-word;
  }
  .bubble.mine {
    background: var(--accent-primary-dim); border-color: var(--accent-primary);
    border-radius: 12px 12px 3px 12px;
  }

  /* Input */
  .input-row {
    display: flex; align-items: flex-end; gap: 8px;
    padding: 8px 12px; border-top: 1px solid var(--border-default);
    background: var(--bg-surface); flex-shrink: 0;
  }
  .message-input {
    flex: 1; background: var(--bg-input); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); color: var(--text-primary); font-size: 13px;
    padding: 7px 12px; resize: none; font-family: var(--font-sans); line-height: 1.4;
  }
  .message-input:focus { border-color: var(--accent-primary); outline: none; }

  .send-btn {
    width: 34px; height: 34px; background: var(--accent-primary); color: white;
    border: none; border-radius: var(--radius-md); cursor: pointer;
    display: flex; align-items: center; justify-content: center; flex-shrink: 0;
    transition: var(--transition-fast);
  }
  .send-btn:hover:not(:disabled) { filter: brightness(1.1); }
  .send-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
