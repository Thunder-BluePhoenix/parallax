<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy } from "svelte";

  interface GrpcResponse {
    grpc_status: number;
    grpc_message: string;
    http_status: number;
    message_json: string;
    headers: Record<string, string>;
    trailers: Record<string, string>;
    duration_ms: number;
  }

  let serverAddr = $state("http://localhost:50051");
  let methodPath = $state("");
  let requestJson = $state("{}");
  let tlsSkipVerify = $state(false);
  let callType = $state<"unary" | "stream">("unary");
  let metadataRaw = $state("");

  let response = $state<GrpcResponse | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let activeTab = $state<"response" | "headers" | "trailers">("response");

  // Streaming state
  let streamMessages = $state<string[]>([]);
  let streamDone = $state(false);
  let streamId = $state("");
  let unlistenMsg: UnlistenFn | null = null;
  let unlistenEnd: UnlistenFn | null = null;

  // Reflection state
  let reflectedMethods = $state<string[]>([]);
  let reflecting = $state(false);
  let reflectError = $state<string | null>(null);

  async function reflect() {
    if (!serverAddr) return;
    reflecting = true; reflectError = null; reflectedMethods = [];
    try {
      reflectedMethods = await invoke<string[]>("grpc_reflect", { serverAddr, tlsSkipVerify });
      if (reflectedMethods.length === 0) reflectError = "No services found (server may not support reflection)";
    } catch (e: any) {
      reflectError = String(e);
    } finally {
      reflecting = false;
    }
  }

  function pickMethod(m: string) { methodPath = m; }


  function parseMetadata(): Record<string, string> {
    const out: Record<string, string> = {};
    for (const line of metadataRaw.split("\n")) {
      const ci = line.indexOf(":");
      if (ci > 0) out[line.slice(0, ci).trim()] = line.slice(ci + 1).trim();
    }
    return out;
  }

  async function send() {
    if (!serverAddr || !methodPath) return;
    if (callType === "stream") { await startStream(); return; }
    loading = true; error = null; response = null;
    try {
      response = await invoke<GrpcResponse>("grpc_unary", {
        serverAddr, methodPath, requestJson,
        metadata: parseMetadata(), tlsSkipVerify,
      });
    } catch (e: any) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function startStream() {
    stopStream();
    streamMessages = []; streamDone = false; error = null;
    streamId = Math.random().toString(36).slice(2);
    loading = true;

    unlistenMsg = await listen<string>(`grpc_stream_message_${streamId}`, (evt) => {
      try {
        const pretty = JSON.stringify(JSON.parse(evt.payload), null, 2);
        streamMessages.push(pretty);
      } catch {
        streamMessages.push(evt.payload);
      }
    });

    unlistenEnd = await listen<any>(`grpc_stream_end_${streamId}`, (evt) => {
      loading = false; streamDone = true;
      if (evt.payload?.error) error = evt.payload.error;
      stopStream();
    });

    invoke("grpc_server_stream", {
      serverAddr, methodPath, requestJson,
      metadata: parseMetadata(), streamId, tlsSkipVerify,
    }).catch((e: any) => {
      error = String(e); loading = false; streamDone = true; stopStream();
    });
  }

  function stopStream() {
    unlistenMsg?.(); unlistenMsg = null;
    unlistenEnd?.(); unlistenEnd = null;
  }

  onDestroy(stopStream);

  function grpcStatusName(code: number): string {
    const names: Record<number, string> = {
      1: "CANCELLED", 2: "UNKNOWN", 3: "INVALID_ARGUMENT", 4: "DEADLINE_EXCEEDED",
      5: "NOT_FOUND", 6: "ALREADY_EXISTS", 7: "PERMISSION_DENIED", 8: "RESOURCE_EXHAUSTED",
      9: "FAILED_PRECONDITION", 10: "ABORTED", 11: "OUT_OF_RANGE", 12: "UNIMPLEMENTED",
      13: "INTERNAL", 14: "UNAVAILABLE", 15: "DATA_LOSS", 16: "UNAUTHENTICATED",
    };
    return names[code] ?? `STATUS_${code}`;
  }

  const statusLabel = $derived(
    response === null ? "" :
    response.grpc_status === 0 ? "OK" : grpcStatusName(response.grpc_status)
  );

  let prettyJson = $derived.by(() => {
    if (!response?.message_json) return "";
    try { return JSON.stringify(JSON.parse(response.message_json), null, 2); }
    catch { return response.message_json; }
  });
</script>

<div class="grpc-pane">
  <!-- Connection bar -->
  <div class="grpc-bar">
    <input
      class="grpc-input mono"
      type="text"
      bind:value={serverAddr}
      placeholder="http://localhost:50051"
      title="gRPC server address"
    />
    <span class="grpc-sep">/</span>
    <input
      class="grpc-input grpc-method mono"
      type="text"
      bind:value={methodPath}
      placeholder="package.Service/Method"
      title="Fully-qualified method path"
    />
    <button class="btn-reflect" onclick={reflect} disabled={reflecting || !serverAddr} title="Discover methods via server reflection">
      {reflecting ? "…" : "Reflect"}
    </button>
    <button class="btn-invoke" onclick={send} disabled={loading || !serverAddr || !methodPath}>
      {loading ? "Invoking…" : "Invoke"}
    </button>
  </div>

  <!-- Reflection method picker -->
  {#if reflectedMethods.length > 0}
    <div class="reflect-list">
      {#each reflectedMethods as m}
        <button class="reflect-method mono" class:active={methodPath === m} onclick={() => pickMethod(m)}>{m}</button>
      {/each}
    </div>
  {/if}
  {#if reflectError}
    <div class="reflect-error">{reflectError}</div>
  {/if}

  <!-- Options row -->
  <div class="grpc-options">
    <label class="opt-toggle">
      <input type="radio" bind:group={callType} value="unary" />
      <span>Unary</span>
    </label>
    <label class="opt-toggle">
      <input type="radio" bind:group={callType} value="stream" />
      <span>Server stream</span>
    </label>
    <span class="opt-sep"></span>
    <label class="opt-toggle">
      <input type="checkbox" bind:checked={tlsSkipVerify} />
      <span>Skip TLS verify</span>
    </label>
    {#if callType === "stream" && loading}
      <button class="btn-stop" onclick={stopStream}>Stop</button>
    {/if}
  </div>

  <div class="grpc-body">
    <!-- Left: request editor -->
    <div class="grpc-request">
      <div class="pane-label">Request JSON</div>
      <textarea
        class="grpc-textarea mono"
        bind:value={requestJson}
        spellcheck="false"
        placeholder="&#123;&#125;"
      ></textarea>
      <div class="pane-label" style="margin-top:8px">Metadata</div>
      <textarea
        class="grpc-textarea grpc-metadata mono"
        bind:value={metadataRaw}
        spellcheck="false"
        placeholder="authorization: Bearer token&#10;x-request-id: abc123"
      ></textarea>
    </div>

    <!-- Right: response -->
    <div class="grpc-response">
      {#if error}
        <div class="grpc-error">{error}</div>
      {/if}

      {#if callType === "stream"}
        <!-- Streaming response feed -->
        <div class="stream-header">
          <span class="pane-label">Stream messages</span>
          <span class="stream-count">{streamMessages.length} received</span>
          {#if streamDone}<span class="stream-done">✓ done</span>{/if}
          {#if loading}<span class="stream-live">● live</span>{/if}
        </div>
        <div class="stream-feed mono">
          {#each streamMessages as msg, i}
            <div class="stream-msg">
              <span class="stream-idx">{i + 1}</span>
              <pre class="stream-pre">{msg}</pre>
            </div>
          {/each}
          {#if streamMessages.length === 0 && !loading}
            <div class="grpc-empty">No messages yet. Click Invoke to start streaming.</div>
          {/if}
        </div>

      {:else if response}
        <div class="grpc-status-bar">
          <span class="grpc-badge" class:ok={response.grpc_status === 0} class:err={response.grpc_status !== 0}>
            {response.grpc_status} {statusLabel}
          </span>
          <span class="grpc-meta">HTTP {response.http_status} · {response.duration_ms}ms</span>
          {#if response.grpc_message}
            <span class="grpc-msg">{response.grpc_message}</span>
          {/if}
        </div>

        <div class="resp-tabs">
          {#each (["response", "headers", "trailers"] as const) as t}
            <button class="resp-tab" class:active={activeTab === t} onclick={() => activeTab = t}>{t}</button>
          {/each}
        </div>

        {#if activeTab === "response"}
          <pre class="grpc-body-pre mono">{prettyJson || "(empty response)"}</pre>
        {:else if activeTab === "headers"}
          <div class="kv-list">
            {#each Object.entries(response.headers) as [k, v]}
              <div class="kv-row"><span class="kv-key mono">{k}</span><span class="kv-val mono">{v}</span></div>
            {/each}
          </div>
        {:else}
          <div class="kv-list">
            {#each Object.entries(response.trailers) as [k, v]}
              <div class="kv-row"><span class="kv-key mono">{k}</span><span class="kv-val mono">{v}</span></div>
            {/each}
            {#if Object.keys(response.trailers).length === 0}
              <div class="grpc-empty">No trailers</div>
            {/if}
          </div>
        {/if}

      {:else if !error}
        <div class="grpc-empty">
          Enter a server address and method path, then click <strong>Invoke</strong>.
          <br /><br />
          <span class="grpc-hint">
            Requires a server supporting <code>application/grpc+json</code>
            (grpc-gateway, Connect protocol, or grpc-web with JSON).
          </span>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .grpc-pane {
    display: flex; flex-direction: column; height: 100%; overflow: hidden;
    background: var(--bg-base);
  }

  .grpc-bar {
    display: flex; align-items: center; gap: 6px;
    padding: 10px 16px; border-bottom: 1px solid var(--border-default);
    background: var(--bg-surface);
  }
  .grpc-input {
    flex: 1; height: 32px; padding: 0 10px;
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); color: var(--text-primary); font-size: 12px;
  }
  .grpc-input:focus { border-color: var(--accent-primary); outline: none; }
  .grpc-method { flex: 1.2; }
  .grpc-sep { color: var(--text-muted); font-size: 18px; }

  .btn-reflect {
    height: 32px; padding: 0 12px;
    background: transparent; color: var(--accent-secondary);
    border: 1px solid var(--accent-secondary); opacity: 0.8;
    font-size: 11px; font-weight: 700; border-radius: var(--radius-md);
    white-space: nowrap;
  }
  .btn-reflect:hover:not(:disabled) { opacity: 1; background: rgba(100,160,255,0.08); }
  .btn-reflect:disabled { opacity: 0.35; }

  .btn-invoke {
    height: 32px; padding: 0 18px;
    background: var(--accent-primary); color: white;
    font-size: 12px; font-weight: 700; border-radius: var(--radius-md);
    white-space: nowrap;
  }
  .btn-invoke:disabled { opacity: 0.5; }

  .reflect-list {
    display: flex; flex-wrap: wrap; gap: 4px;
    padding: 6px 16px; background: var(--bg-surface);
    border-bottom: 1px solid var(--border-default);
  }
  .reflect-method {
    padding: 2px 8px; font-size: 10px;
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-sm); color: var(--text-secondary);
  }
  .reflect-method:hover { border-color: var(--accent-primary); color: var(--accent-primary); }
  .reflect-method.active { background: var(--accent-primary-dim); color: var(--accent-primary); border-color: var(--accent-primary); }
  .reflect-error { font-size: 11px; color: var(--color-warning); padding: 4px 16px; background: var(--bg-surface); border-bottom: 1px solid var(--border-default); }

  .grpc-options {
    display: flex; gap: 12px; align-items: center;
    padding: 6px 16px; background: var(--bg-surface);
    border-bottom: 1px solid var(--border-default);
    font-size: 11px;
  }
  .opt-toggle { display: flex; align-items: center; gap: 4px; cursor: pointer; color: var(--text-secondary); }
  .opt-toggle input { cursor: pointer; }

  .grpc-body {
    flex: 1; display: grid; grid-template-columns: 1fr 1fr;
    overflow: hidden; gap: 0;
  }

  .grpc-request {
    display: flex; flex-direction: column; gap: 4px;
    padding: 12px; border-right: 1px solid var(--border-default);
    overflow: hidden;
  }
  .pane-label { font-size: 10px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.06em; }
  .grpc-textarea {
    flex: 1; resize: none; padding: 8px 10px; font-size: 12px;
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); color: var(--text-primary);
    min-height: 0;
  }
  .grpc-textarea:focus { border-color: var(--accent-primary); outline: none; }
  .grpc-metadata { flex: none; height: 80px; }

  .grpc-response {
    display: flex; flex-direction: column; overflow: hidden; padding: 12px;
  }

  .grpc-status-bar {
    display: flex; align-items: center; gap: 10px; margin-bottom: 8px;
    flex-wrap: wrap;
  }
  .grpc-badge {
    padding: 2px 10px; border-radius: 20px; font-size: 11px; font-weight: 700; font-family: var(--font-mono);
  }
  .grpc-badge.ok  { background: rgba(63,185,80,0.15); color: var(--color-success); }
  .grpc-badge.err { background: rgba(248,81,73,0.15);  color: var(--color-error); }
  .grpc-meta { font-size: 11px; color: var(--text-muted); }
  .grpc-msg  { font-size: 11px; color: var(--color-error); }

  .resp-tabs { display: flex; gap: 2px; margin-bottom: 8px; }
  .resp-tab {
    padding: 4px 10px; font-size: 11px; font-weight: 600;
    border-radius: var(--radius-sm); color: var(--text-secondary); background: transparent;
    text-transform: capitalize;
  }
  .resp-tab:hover { background: var(--bg-elevated); }
  .resp-tab.active { background: var(--accent-primary-dim); color: var(--accent-primary); }

  .grpc-body-pre {
    flex: 1; overflow: auto; font-size: 12px; color: var(--text-primary);
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); padding: 10px; white-space: pre;
    margin: 0;
  }

  .kv-list { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 2px; }
  .kv-row { display: flex; gap: 8px; font-size: 11px; padding: 4px 8px; background: var(--bg-elevated); border-radius: var(--radius-sm); }
  .kv-key { color: var(--accent-secondary); min-width: 120px; }
  .kv-val { color: var(--text-primary); word-break: break-all; }

  .grpc-error { color: var(--color-error); font-size: 12px; padding: 12px; background: rgba(248,81,73,0.08); border-radius: var(--radius-md); margin-bottom: 8px; }
  .grpc-empty { color: var(--text-muted); font-size: 12px; padding: 24px 0; }
  .grpc-hint { font-size: 11px; line-height: 1.6; }
  .grpc-hint code { color: var(--accent-secondary); font-family: var(--font-mono); }

  .opt-sep { flex: 1; }
  .btn-stop {
    height: 22px; padding: 0 10px; font-size: 10px; font-weight: 700;
    background: rgba(248,81,73,0.15); color: var(--color-error);
    border: 1px solid rgba(248,81,73,0.3); border-radius: var(--radius-sm);
  }
  .btn-stop:hover { background: rgba(248,81,73,0.25); }

  /* Streaming */
  .stream-header {
    display: flex; align-items: center; gap: 8px;
    margin-bottom: 6px; flex-shrink: 0;
  }
  .stream-count { font-size: 10px; color: var(--text-muted); }
  .stream-done  { font-size: 10px; color: var(--color-success); font-weight: 700; }
  .stream-live  { font-size: 10px; color: var(--color-error); font-weight: 700; animation: pulse-dot 1.2s infinite; }

  .stream-feed {
    flex: 1; overflow-y: auto;
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); padding: 6px;
    display: flex; flex-direction: column; gap: 4px;
  }
  .stream-msg {
    display: flex; gap: 6px; align-items: flex-start;
    padding: 4px 6px; background: var(--bg-surface);
    border-radius: var(--radius-sm); border-left: 2px solid var(--accent-primary);
  }
  .stream-idx  { font-size: 9px; color: var(--text-muted); min-width: 18px; padding-top: 2px; text-align: right; }
  .stream-pre  { margin: 0; font-size: 11px; color: var(--text-primary); white-space: pre-wrap; word-break: break-all; }
</style>
