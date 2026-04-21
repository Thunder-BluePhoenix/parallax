<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { activeRequest } from "../../stores/app.svelte";

  type FrameType = "connected" | "text" | "binary" | "ping" | "pong" | "close" | "error";

  interface WsFrame {
    connectionId: string;
    frameType:    FrameType;
    data:         string | null;
    timestampMs:  number;
  }

  let connectionId = $state<string | null>(null);
  let frames       = $state<WsFrame[]>([]);
  let message      = $state("");
  let connecting   = $state(false);
  let unlisten     = $state<UnlistenFn | null>(null);
  let framesEl     = $state<HTMLDivElement | null>(null);

  let connected = $derived(connectionId !== null);

  async function connect() {
    if (!activeRequest.url) return;
    connecting = true;

    const ul = await listen<WsFrame>("ws_frame", (ev) => {
      if (ev.payload.connectionId !== connectionId && connectionId !== null) return;
      frames.push(ev.payload);
      if (connectionId === null && ev.payload.frameType === "connected") {
        connectionId = ev.payload.connectionId;
      }
      if (ev.payload.frameType === "close" || ev.payload.frameType === "error") {
        connectionId = null;
      }
      // auto-scroll
      setTimeout(() => { framesEl?.scrollTo({ top: framesEl.scrollHeight, behavior: "smooth" }); }, 0);
    });
    unlisten = ul;

    try {
      const id: string = await invoke("ws_connect", {
        req: {
          url:     activeRequest.url,
          headers: { ...activeRequest.headers },
        },
      });
      connectionId = id;
    } catch (e: any) {
      frames.push({
        connectionId: "—",
        frameType: "error",
        data: e?.toString() ?? "Connection failed",
        timestampMs: Date.now(),
      });
      unlisten?.();
      unlisten = null;
    } finally {
      connecting = false;
    }
  }

  async function disconnect() {
    if (!connectionId) return;
    await invoke("ws_disconnect", { connectionId });
    unlisten?.();
    unlisten = null;
    connectionId = null;
  }

  async function sendMessage() {
    if (!connectionId || !message.trim()) return;
    const text = message.trim();
    frames.push({
      connectionId,
      frameType: "text",
      data: text,
      timestampMs: Date.now(),
    } as WsFrame & { outgoing?: boolean });
    message = "";
    await invoke("ws_send", { connectionId, message: text });
  }

  function clearFrames() { frames = []; }

  function formatTime(ms: number) {
    return new Date(ms).toLocaleTimeString(undefined, { hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit", fractionalSecondDigits: 3 });
  }

  function frameClass(type: FrameType) {
    const map: Record<FrameType, string> = {
      connected: "frame-connected",
      text:      "frame-text",
      binary:    "frame-binary",
      ping:      "frame-ping",
      pong:      "frame-ping",
      close:     "frame-close",
      error:     "frame-error",
    };
    return map[type] ?? "";
  }
</script>

<div class="ws-pane">
  <!-- Status bar -->
  <div class="ws-statusbar">
    <div class="ws-status-dot" class:connected class:connecting></div>
    <span class="ws-status-label">
      {#if connecting}Connecting…{:else if connected}Connected · {connectionId?.slice(0,8)}{:else}Disconnected{/if}
    </span>
    <div class="ws-actions">
      {#if connected}
        <button class="ws-btn ws-btn-disconnect" onclick={disconnect}>Disconnect</button>
      {:else}
        <button class="ws-btn ws-btn-connect" onclick={connect} disabled={connecting}>
          {connecting ? "Connecting…" : "Connect"}
        </button>
      {/if}
      <button class="ws-btn" onclick={clearFrames} title="Clear frames">Clear</button>
    </div>
  </div>

  <!-- Frame stream -->
  <div class="frames-scroll scroll-y" bind:this={framesEl}>
    {#if frames.length === 0}
      <div class="frames-empty">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" opacity="0.3">
          <path d="M18 8h1a4 4 0 0 1 0 8h-1"/><path d="M2 8h16v9a4 4 0 0 1-4 4H6a4 4 0 0 1-4-4V8z"/>
          <line x1="6" y1="1" x2="6" y2="4"/><line x1="10" y1="1" x2="10" y2="4"/><line x1="14" y1="1" x2="14" y2="4"/>
        </svg>
        <p>No frames yet — connect to a WebSocket endpoint</p>
      </div>
    {:else}
      {#each frames as frame, i}
        <div class="frame-row {frameClass(frame.frameType)}">
          <span class="frame-time mono">{formatTime(frame.timestampMs)}</span>
          <span class="frame-type">{frame.frameType.toUpperCase()}</span>
          <span class="frame-data mono">{frame.data ?? ""}</span>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Send bar -->
  <div class="ws-send-bar">
    <input
      class="ws-message-input mono"
      type="text"
      placeholder={connected ? 'Send a message…' : 'Connect first'}
      bind:value={message}
      disabled={!connected}
      onkeydown={(e) => { if (e.key === "Enter") sendMessage(); }}
    />
    <button class="ws-btn ws-btn-send" onclick={sendMessage} disabled={!connected || !message.trim()}>
      Send
    </button>
  </div>
</div>

<style>
  .ws-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* Status bar */
  .ws-statusbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 12px;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    flex-shrink: 0;
  }

  .ws-status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-muted);
    flex-shrink: 0;
    transition: background 0.3s;
  }
  .ws-status-dot.connecting { background: var(--color-warning); animation: pulse 1s infinite; }
  .ws-status-dot.connected  { background: var(--color-success); }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.3; }
  }

  .ws-status-label {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    flex: 1;
  }

  .ws-actions { display: flex; gap: 4px; }

  .ws-btn {
    padding: 3px 10px;
    font-size: 11px;
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    color: var(--text-secondary);
    border: 1px solid var(--border-default);
    transition: var(--transition-fast);
  }
  .ws-btn:hover:not(:disabled) { color: var(--text-primary); background: var(--bg-overlay); }
  .ws-btn:disabled { opacity: 0.4; }

  .ws-btn-connect    { background: var(--accent-primary-dim); color: var(--accent-primary); border-color: var(--accent-primary); }
  .ws-btn-connect:hover:not(:disabled) { background: var(--accent-primary); color: white; }

  .ws-btn-disconnect { background: var(--color-error-dim); color: var(--color-error); border-color: var(--color-error); }
  .ws-btn-disconnect:hover { background: var(--color-error); color: white; }

  .ws-btn-send { background: var(--accent-primary); color: white; border-color: var(--accent-primary); }
  .ws-btn-send:hover:not(:disabled) { background: #9185ff; }

  /* Frame stream */
  .frames-scroll {
    flex: 1;
    font-size: 12px;
  }

  .frames-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 10px;
    color: var(--text-muted);
    font-size: 12px;
    text-align: center;
    padding: 24px;
  }

  .frame-row {
    display: grid;
    grid-template-columns: 100px 64px 1fr;
    gap: 10px;
    align-items: start;
    padding: 5px 12px;
    border-bottom: 1px solid var(--border-subtle);
    line-height: 1.5;
  }

  .frame-time { color: var(--text-muted); font-size: 10px; }
  .frame-type { font-size: 10px; font-weight: 700; letter-spacing: 0.04em; }
  .frame-data { color: var(--text-secondary); word-break: break-all; white-space: pre-wrap; }

  .frame-connected .frame-type { color: var(--color-success); }
  .frame-text      .frame-type { color: var(--accent-primary); }
  .frame-binary    .frame-type { color: var(--color-warning); }
  .frame-ping      .frame-type { color: var(--text-muted); }
  .frame-close     .frame-type { color: var(--color-error); }
  .frame-row.frame-error,
  .frame-row.frame-error .frame-type { color: var(--color-error); }
  .frame-row.frame-error { background: var(--color-error-dim); }

  /* Send bar */
  .ws-send-bar {
    display: flex;
    gap: 6px;
    padding: 8px 10px;
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    flex-shrink: 0;
  }

  .ws-message-input {
    flex: 1;
    height: 32px;
    padding: 0 10px;
    font-size: 12px;
    border-radius: var(--radius-sm);
  }
</style>
