<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    tabs, activeRequest, responseState, sendRequest, cancelRequest, persistTabs,
    loadedCollections, loadRequestIntoTab, saveTabSnapshot, restoreTabSnapshot, defaultAuth,
    type RequestTab,
  } from "../../stores/app.svelte";
  import RequestPanel from "./RequestPanel.svelte";
  import ResponsePanel from "./ResponsePanel.svelte";
  import WebSocketPane from "./WebSocketPane.svelte";
  import SSEPane from "./SSEPane.svelte";
  import GRPCPane from "./GRPCPane.svelte";

  let activeTab = $state("params");
  let showRequestList = $state(true);
  let expandedFolders = $state<Record<string, boolean>>({});
  const uuid = () => Math.random().toString(36).substring(2) + Date.now().toString(36);

  const METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "WS", "SSE", "GRPC"] as const;

  function switchTab(id: string) {
    if (id === tabs.activeId) return;
    saveTabSnapshot(tabs.activeId);
    tabs.activeId = id;
    restoreTabSnapshot(id);
    // sync method on tab metadata
    const t = tabs.list.find(t => t.id === id);
    if (t) t.method = activeRequest.method;
    persistTabs();
  }

  function newTab() {
    saveTabSnapshot(tabs.activeId);
    const id = uuid();
    tabs.list.push({ id, name: "New Request", method: "GET", modified: false });
    tabs.activeId = id;
    restoreTabSnapshot(id);
    persistTabs();
  }

  function closeTab(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (tabs.activeId === id) saveTabSnapshot(id);
    const idx = tabs.list.findIndex(t => t.id === id);
    tabs.list = tabs.list.filter(t => t.id !== id);
    if (tabs.activeId === id && tabs.list.length > 0) {
      const next = tabs.list[Math.max(0, idx - 1)].id;
      tabs.activeId = next;
      restoreTabSnapshot(next);
    }
    persistTabs();
  }

  // Sync method badge whenever activeRequest.method changes
  $effect(() => {
    const m = activeRequest.method;
    const t = tabs.list.find(t => t.id === tabs.activeId);
    if (t && t.method !== m) { t.method = m; persistTabs(); }
  });

  // Sync name badge whenever activeRequest.name changes
  $effect(() => {
    const n = activeRequest.name;
    const t = tabs.list.find(t => t.id === tabs.activeId);
    if (t && n && t.name !== n) { t.name = n; persistTabs(); }
  });

  async function importHAR() {
    try {
      const filePath = await open({
        filters: [{ name: "HAR Archive", extensions: ["har", "json"] }],
        multiple: false, title: "Import HAR file",
      }) as string | null;
      if (!filePath) return;
      const raw = await invoke<string>("read_file_for_template", { path: filePath });
      const { importHar } = await import("../../utils/har-importer");
      const col = importHar(raw);
      loadedCollections.push(col);
      alert(`Imported "${col.name}" — ${col.requests.length} requests`);
    } catch (e) { alert("HAR import failed: " + e); }
  }

  function onKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") sendRequest();
  }

  function toggleFolder(id: string) {
    expandedFolders[id] = !expandedFolders[id];
  }

  function openRequest(req: any) {
    saveTabSnapshot(tabs.activeId);
    loadRequestIntoTab(req);
  }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="builder-mode">
  <!-- Tab Bar -->
  <div class="tab-row">
    <button class="list-toggle" class:active={showRequestList} onclick={() => showRequestList = !showRequestList} title="Toggle request list">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/>
        <line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/>
      </svg>
    </button>

    <div class="tab-scroll">
      {#each tabs.list as tab (tab.id)}
        <div
          class="req-tab"
          class:active={tabs.activeId === tab.id}
          role="tab" tabindex="0"
          aria-selected={tabs.activeId === tab.id}
          onclick={() => switchTab(tab.id)}
          onkeydown={(e) => e.key === "Enter" && switchTab(tab.id)}
        >
          <span class="method-badge method-{tab.method}">{tab.method}</span>
          <span class="tab-name">{tab.name}</span>
          {#if tab.modified}<span class="modified-dot" title="Unsaved"></span>{/if}
          <button class="close-btn" onclick={(e) => closeTab(tab.id, e)}>×</button>
        </div>
      {/each}
    </div>

    <button class="new-tab-btn" onclick={newTab} title="New request">+</button>
    <button class="har-btn" onclick={importHAR} title="Import HAR file">HAR</button>
  </div>

  <div class="builder-body">
    <!-- Request List Panel -->
    {#if showRequestList}
      <div class="request-list">
        <div class="rl-header">
          <span class="rl-title">Requests</span>
          <button class="rl-new" onclick={newTab} title="New request">+</button>
        </div>
        <div class="rl-scroll">
          <!-- Open tabs section -->
          <div class="rl-collection">
            <div class="rl-col-name">Open</div>
            {#each tabs.list as tab (tab.id)}
              <div
                class="rl-request"
                class:active={tabs.activeId === tab.id}
                role="button" tabindex="0"
                onclick={() => switchTab(tab.id)}
                onkeydown={(e) => e.key === "Enter" && switchTab(tab.id)}
              >
                <span class="rl-method method-{tab.method}">{tab.method}</span>
                <span class="rl-req-name">{tab.name}</span>
                <button class="rl-close" onclick={(e) => { e.stopPropagation(); closeTab(tab.id, e as MouseEvent); }}>×</button>
              </div>
            {/each}
          </div>

          {#if loadedCollections.length > 0}<div class="rl-divider"></div>{/if}

          {#each loadedCollections as col}
            <div class="rl-collection">
              <div class="rl-col-name" title={col.name}>{col.name}</div>

              {#each (col.folders ?? []) as folder}
                <div class="rl-folder">
                  <button class="rl-folder-btn" onclick={() => toggleFolder(folder.id)}>
                    <span class="rl-chevron" class:open={expandedFolders[folder.id]}>▶</span>
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M10 4H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-8l-2-2z"/></svg>
                    <span class="rl-folder-name">{folder.name}</span>
                    <span class="rl-count">{folder.requests?.length ?? 0}</span>
                  </button>
                  {#if expandedFolders[folder.id]}
                    {#each (folder.requests ?? []) as req}
                      <button
                        class="rl-request"
                        class:active={tabs.activeId === req.id}
                        onclick={() => openRequest(req)}
                      >
                        <span class="rl-method method-{req.method}">{req.method}</span>
                        <span class="rl-req-name">{req.name}</span>
                      </button>
                    {/each}
                  {/if}
                </div>
              {/each}

              {#each (col.requests ?? []) as req}
                <button
                  class="rl-request"
                  class:active={tabs.activeId === req.id}
                  onclick={() => openRequest(req)}
                >
                  <span class="rl-method method-{req.method}">{req.method}</span>
                  <span class="rl-req-name">{req.name}</span>
                </button>
              {/each}
            </div>
          {/each}

          {#if loadedCollections.length === 0}
            <div class="rl-empty">Open a collection from the sidebar to see requests here.</div>
          {/if}
        </div>
      </div>
      <div class="rl-resize-handle"></div>
    {/if}

    <!-- Main editor area -->
    <div class="editor-area">
      <!-- URL Bar -->
      <div class="url-bar">
        <select class="method-select method-{activeRequest.method}" bind:value={activeRequest.method}>
          {#each METHODS as m}
            <option value={m}>{m}</option>
          {/each}
        </select>

        <input
          class="url-input mono" type="text"
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

      <!-- Request / Response panels -->
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
  </div>
</div>

<style>
  .builder-mode {
    display: flex; flex-direction: column; height: 100%; overflow: hidden;
  }

  /* Tab row */
  .tab-row {
    display: flex; align-items: center;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-default);
    flex-shrink: 0; overflow: hidden;
  }

  .list-toggle {
    padding: 0 10px; height: 36px; background: transparent;
    color: var(--text-muted); flex-shrink: 0; transition: var(--transition-fast);
    display: flex; align-items: center;
  }
  .list-toggle:hover, .list-toggle.active { color: var(--accent-primary); }

  .tab-scroll {
    display: flex; overflow-x: auto; overflow-y: hidden;
    flex: 1; scrollbar-width: none;
  }
  .tab-scroll::-webkit-scrollbar { display: none; }

  .req-tab {
    display: flex; align-items: center; gap: 6px;
    padding: 0 12px; height: 36px;
    background: transparent; border-right: 1px solid var(--border-subtle);
    color: var(--text-secondary); font-size: 12px;
    white-space: nowrap; min-width: 120px; max-width: 200px;
    transition: var(--transition-fast); cursor: pointer;
  }
  .req-tab:hover { background: var(--bg-elevated); color: var(--text-primary); }
  .req-tab.active { background: var(--bg-base); color: var(--text-primary); }
  .tab-name { flex: 1; overflow: hidden; text-overflow: ellipsis; }
  .modified-dot { width: 5px; height: 5px; border-radius: 50%; background: var(--color-warning); flex-shrink: 0; }
  .close-btn {
    background: transparent; color: var(--text-muted); font-size: 14px;
    padding: 0 2px; border-radius: 2px; opacity: 0; transition: var(--transition-fast);
  }
  .req-tab:hover .close-btn { opacity: 1; }
  .close-btn:hover { background: var(--bg-overlay); color: var(--text-primary); }

  .new-tab-btn {
    padding: 0 14px; height: 36px; background: transparent;
    color: var(--text-muted); font-size: 18px; flex-shrink: 0;
    transition: var(--transition-fast);
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

  /* Builder body: list panel + editor */
  .builder-body {
    display: flex; flex: 1; overflow: hidden;
  }

  /* Request list */
  .request-list {
    width: 220px; flex-shrink: 0;
    background: var(--bg-surface);
    border-right: 1px solid var(--border-default);
    display: flex; flex-direction: column; overflow: hidden;
  }
  .rl-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 10px; border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }
  .rl-title { font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.06em; }
  .rl-new {
    width: 20px; height: 20px; border-radius: var(--radius-sm);
    background: var(--bg-elevated); color: var(--text-secondary);
    font-size: 16px; display: flex; align-items: center; justify-content: center;
    transition: var(--transition-fast);
  }
  .rl-new:hover { background: var(--accent-primary-dim); color: var(--accent-primary); }

  .rl-scroll { flex: 1; overflow-y: auto; overflow-x: hidden; }
  .rl-empty { padding: 16px 10px; font-size: 11px; color: var(--text-muted); text-align: center; }

  .rl-collection { padding-bottom: 8px; }
  .rl-col-name {
    padding: 8px 10px 4px;
    font-size: 10px; font-weight: 700; color: var(--text-muted);
    text-transform: uppercase; letter-spacing: 0.06em;
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }

  .rl-folder { }
  .rl-folder-btn {
    display: flex; align-items: center; gap: 5px;
    width: 100%; padding: 4px 10px;
    background: transparent; color: var(--text-secondary); font-size: 12px;
    text-align: left; transition: var(--transition-fast);
  }
  .rl-folder-btn:hover { background: var(--bg-elevated); color: var(--text-primary); }
  .rl-chevron { font-size: 8px; transition: transform var(--transition-fast); color: var(--text-muted); }
  .rl-chevron.open { transform: rotate(90deg); }
  .rl-folder-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .rl-count { font-size: 10px; color: var(--text-muted); background: var(--bg-elevated); padding: 0 4px; border-radius: 10px; }

  .rl-request {
    display: flex; align-items: center; gap: 6px;
    width: 100%; padding: 5px 10px 5px 22px;
    background: transparent; color: var(--text-secondary); font-size: 12px;
    text-align: left; transition: var(--transition-fast); border-left: 2px solid transparent;
  }
  .rl-request:hover { background: var(--bg-elevated); color: var(--text-primary); }
  .rl-request.active { background: var(--accent-primary-dim); border-left-color: var(--accent-primary); color: var(--text-primary); }
  .rl-method {
    font-size: 8px; font-weight: 800; font-family: var(--font-mono);
    flex-shrink: 0; min-width: 28px;
  }
  .rl-req-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .rl-close {
    opacity: 0; background: transparent; color: var(--text-muted);
    font-size: 13px; padding: 0 2px; border-radius: 2px; flex-shrink: 0;
    transition: var(--transition-fast);
  }
  .rl-request:hover .rl-close { opacity: 1; }
  .rl-close:hover { color: var(--color-error); }
  .rl-divider { height: 1px; background: var(--border-subtle); margin: 6px 0; }

  .rl-resize-handle {
    width: 3px; background: var(--border-default); cursor: col-resize; flex-shrink: 0;
    transition: background var(--transition-fast);
  }
  .rl-resize-handle:hover { background: var(--accent-primary); }

  /* Editor area */
  .editor-area { display: flex; flex-direction: column; flex: 1; overflow: hidden; }

  /* URL Bar */
  .url-bar {
    display: flex; align-items: center; gap: 8px;
    padding: 8px 12px;
    background: var(--bg-base); border-bottom: 1px solid var(--border-subtle);
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
  .panels { display: flex; flex: 1; overflow: hidden; }
  .panel-resize-handle {
    width: 4px; background: var(--border-default); cursor: col-resize; flex-shrink: 0;
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
