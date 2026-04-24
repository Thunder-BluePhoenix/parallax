<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { tabs, activeRequest, responseState, sendRequest, cancelRequest, persistTabs } from "../../stores/app.svelte";
  import RequestPanel from "./RequestPanel.svelte";
  import ResponsePanel from "./ResponsePanel.svelte";
  import WebSocketPane from "./WebSocketPane.svelte";
  import SSEPane from "./SSEPane.svelte";
  import GRPCPane from "./GRPCPane.svelte";

  let activeTab = $state("params");
  const uuid = () => Math.random().toString(36).substring(2) + Date.now().toString(36);

  const METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "WS", "SSE", "GRPC"] as const;

  // ── HAR Import ──────────────────────────────────────────────
  async function importHAR() {
    try {
      const filePath = await open({
        filters: [{ name: "HAR Archive", extensions: ["har", "json"] }],
        multiple: false,
        title: "Import HAR file",
      }) as string | null;
      if (!filePath) return;
      const raw = await invoke<string>("read_file_for_template", { path: filePath });
      const { importHar } = await import("../../utils/har-importer");
      const { loadedCollections } = await import("../../stores/app.svelte");
      const col = importHar(raw);
      loadedCollections.push(col);
      alert(`Imported "${col.name}" — ${col.requests.length} requests`);
    } catch (e) {
      alert("HAR import failed: " + e);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") sendRequest();
  }

  function closeTab(id: string, e: MouseEvent) {
    e.stopPropagation();
    const idx = tabs.list.findIndex((t) => t.id === id);
    tabs.list = tabs.list.filter((t) => t.id !== id);
    if (tabs.activeId === id && tabs.list.length > 0) {
      tabs.activeId = tabs.list[Math.max(0, idx - 1)].id;
    }
    persistTabs();
  }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="builder-mode">
  <!-- Tab Bar -->
  <div class="tab-row">
    <div class="tab-scroll">
      {#each tabs.list as tab}
        <div
          class="req-tab"
          class:active={tabs.activeId === tab.id}
          role="tab"
          tabindex="0"
          aria-selected={tabs.activeId === tab.id}
          onclick={() => { tabs.activeId = tab.id; persistTabs(); }}
          onkeydown={(e) => { if (e.key === "Enter") { tabs.activeId = tab.id; persistTabs(); } }}
        >
          <span class="method-badge method-{tab.method}">{tab.method}</span>
          <span class="tab-name">{tab.name}</span>
          {#if tab.modified}
            <span class="modified-dot" title="Unsaved"></span>
          {/if}
          <button class="close-btn" onclick={(e) => closeTab(tab.id, e)}>×</button>
        </div>
      {/each}
    </div>

    <button class="new-tab-btn" onclick={() => {
      const id = uuid();
      tabs.list.push({ id, name: "New Request", method: "GET", modified: false });
      tabs.activeId = id;
      persistTabs();
    }}>+</button>

    <button class="har-btn" onclick={importHAR} title="Import HAR file">HAR</button>
  </div>

  <!-- URL Bar -->
  <div class="url-bar">
    <select
      class="method-select method-{activeRequest.method}"
      bind:value={activeRequest.method}
    >
      {#each METHODS as m}
        <option value={m}>{m}</option>
      {/each}
    </select>

    <input
      class="url-input mono"
      type="text"
      placeholder="Enter request URL or paste a cURL command…"
      bind:value={activeRequest.url}
    />

    {#if activeRequest.method !== "WS" && activeRequest.method !== "SSE" && activeRequest.method !== "GRPC"}
      {#if responseState.loading}
        <button class="btn-cancel" onclick={cancelRequest}>Cancel</button>
      {:else}
        <button class="btn-send" onclick={sendRequest}>Send</button>
      {/if}
    {/if}
  </div>

  <!-- Body: request left, response right -->
  <div class="panels">
    <RequestPanel bind:activeTab />
    <div class="panel-resize-handle"></div>
    {#if activeRequest.method === "WS"}
      <WebSocketPane />
    {:else if activeRequest.method === "SSE"}
      <SSEPane />
    {:else if activeRequest.method === "GRPC"}
      <GRPCPane />
    {:else}
      <ResponsePanel />
    {/if}
  </div>
</div>

<style>
  .builder-mode {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* Tabs */
  .tab-row {
    display: flex;
    align-items: center;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-default);
    flex-shrink: 0;
    overflow: hidden;
  }

  .tab-scroll {
    display: flex;
    overflow-x: auto;
    overflow-y: hidden;
    flex: 1;
    scrollbar-width: none;
  }
  .tab-scroll::-webkit-scrollbar { display: none; }

  .req-tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 12px;
    height: 36px;
    background: transparent;
    border-right: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    font-size: 12px;
    white-space: nowrap;
    min-width: 120px;
    max-width: 200px;
    transition: var(--transition-fast);
    cursor: pointer;
  }
  .req-tab:hover { background: var(--bg-elevated); color: var(--text-primary); }
  .req-tab.active { background: var(--bg-base); color: var(--text-primary); }
  .req-tab.active .method-badge { opacity: 1; }

  .tab-name { flex: 1; overflow: hidden; text-overflow: ellipsis; }
  .modified-dot { width: 5px; height: 5px; border-radius: 50%; background: var(--color-warning); flex-shrink: 0; }

  .close-btn {
    background: transparent; color: var(--text-muted); font-size: 14px; line-height: 1;
    padding: 0 2px; border-radius: 2px; opacity: 0; transition: var(--transition-fast);
  }
  .req-tab:hover .close-btn { opacity: 1; }
  .close-btn:hover { background: var(--bg-overlay); color: var(--text-primary); }

  .new-tab-btn {
    padding: 0 14px; height: 36px; background: transparent;
    color: var(--text-muted); font-size: 18px; flex-shrink: 0; transition: var(--transition-fast);
  }
  .new-tab-btn:hover { color: var(--text-primary); background: var(--bg-elevated); }

  .har-btn {
    height: 22px; padding: 0 9px; margin: 0 8px;
    font-size: 10px; font-weight: 700;
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-sm); color: var(--text-muted); flex-shrink: 0;
    transition: var(--transition-fast);
  }
  .har-btn:hover { border-color: var(--accent-primary); color: var(--accent-primary); }

  /* URL Bar */
  .url-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--bg-base);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .method-select {
    height: 34px; padding: 0 10px; border-radius: var(--radius-md);
    font-size: 11px; font-weight: 700; font-family: var(--font-mono);
    letter-spacing: 0.04em; cursor: pointer; flex-shrink: 0;
    appearance: none; -webkit-appearance: none;
  }

  .method-select.method-GET    { background: rgba(63,185,80,0.12);   color: var(--method-get);     border-color: rgba(63,185,80,0.3); }
  .method-select.method-POST   { background: rgba(124,110,255,0.12); color: var(--method-post);    border-color: rgba(124,110,255,0.3); }
  .method-select.method-PUT    { background: rgba(227,179,65,0.12);  color: var(--method-put);     border-color: rgba(227,179,65,0.3); }
  .method-select.method-PATCH  { background: rgba(54,217,196,0.12);  color: var(--method-patch);   border-color: rgba(54,217,196,0.3); }
  .method-select.method-DELETE { background: rgba(248,81,73,0.12);   color: var(--method-delete);  border-color: rgba(248,81,73,0.3); }
  .method-select.method-HEAD   { background: rgba(88,166,255,0.12);  color: var(--method-head);    border-color: rgba(88,166,255,0.3); }
  .method-select.method-OPTIONS{ background: rgba(188,140,255,0.12); color: var(--method-options); border-color: rgba(188,140,255,0.3); }
  .method-select.method-WS     { background: rgba(0,255,255,0.12);   color: #00ffff;               border-color: rgba(0,255,255,0.3); }
  .method-select.method-SSE    { background: rgba(255,0,255,0.12);   color: #ff00ff;               border-color: rgba(255,0,255,0.3); }
  .method-select.method-GRPC   { background: rgba(251,188,4,0.12);   color: #fbbc04;               border-color: rgba(251,188,4,0.3); }

  .url-input { flex: 1; height: 34px; padding: 0 12px; font-size: 13px; border-radius: var(--radius-md); }

  .btn-send {
    height: 34px; padding: 0 20px; background: var(--accent-primary); color: white;
    font-size: 13px; font-weight: 700; border-radius: var(--radius-md); flex-shrink: 0;
  }
  .btn-send:hover { opacity: 0.9; }

  .btn-cancel {
    height: 34px; padding: 0 20px;
    background: rgba(248,81,73,0.15); color: var(--color-error);
    border: 1px solid rgba(248,81,73,0.3);
    font-size: 13px; font-weight: 700; border-radius: var(--radius-md); flex-shrink: 0;
  }
  .btn-cancel:hover { background: rgba(248,81,73,0.25); }

  /* Panels */
  .panels {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .panel-resize-handle {
    width: 4px;
    background: var(--border-default);
    cursor: col-resize;
    flex-shrink: 0;
    transition: background var(--transition-fast);
  }
  .panel-resize-handle:hover { background: var(--accent-primary); }

  /* Method badges in tabs */
  .method-badge {
    font-size: 8px; font-weight: 800; font-family: var(--font-mono);
    padding: 1px 4px; border-radius: 3px; opacity: 0.7; flex-shrink: 0;
  }
  .method-GET    { color: var(--method-get); }
  .method-POST   { color: var(--method-post); }
  .method-PUT    { color: var(--method-put); }
  .method-PATCH  { color: var(--method-patch); }
  .method-DELETE { color: var(--method-delete); }
  .method-HEAD, .method-OPTIONS { color: var(--text-muted); }
  .method-WS  { color: #00ffff; }
  .method-SSE { color: #ff00ff; }
  .method-GRPC{ color: #fbbc04; }
</style>
