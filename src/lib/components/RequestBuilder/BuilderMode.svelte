<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { tabs, activeRequest, responseState, sendRequest, cancelRequest, persistTabs, loadedCollections } from "../../stores/app.svelte";
  import RequestPanel from "./RequestPanel.svelte";
  import ResponsePanel from "./ResponsePanel.svelte";
  import WebSocketPane from "./WebSocketPane.svelte";
  import SSEPane from "./SSEPane.svelte";
  import GRPCPane from "./GRPCPane.svelte";

  let activeTab = $state("params");
  const uuid = () => Math.random().toString(36).substring(2) + Date.now().toString(36);

  const METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "WS", "SSE", "GRPC"] as const;

  // ── Context menu ────────────────────────────────────────────
  interface CtxTarget { type: "collection" | "folder" | "request"; colIdx: number; folderIdx?: number; reqIdx: number }
  let ctxMenu = $state<{ x: number; y: number; target: CtxTarget } | null>(null);

  function showCtxMenu(e: MouseEvent, target: CtxTarget) {
    e.preventDefault();
    e.stopPropagation();
    ctxMenu = { x: e.clientX, y: e.clientY, target };
  }

  function hideCtxMenu() { ctxMenu = null; }

  function ctxDuplicate() {
    if (!ctxMenu) return;
    const { colIdx, folderIdx, reqIdx, type } = ctxMenu.target;
    if (type === "request") {
      const col = loadedCollections[colIdx];
      if (!col) return;
      if (folderIdx !== undefined) {
        const req = col.folders[folderIdx]?.requests[reqIdx];
        if (req) {
          const clone = { ...req, id: uuid(), name: req.name + " (copy)" };
          col.folders[folderIdx].requests.splice(reqIdx + 1, 0, clone);
        }
      } else {
        const req = col.requests[reqIdx];
        if (req) {
          const clone = { ...req, id: uuid(), name: req.name + " (copy)" };
          col.requests.splice(reqIdx + 1, 0, clone);
        }
      }
    }
    hideCtxMenu();
  }

  function ctxRename() {
    if (!ctxMenu) return;
    const { colIdx, folderIdx, reqIdx, type } = ctxMenu.target;
    const col = loadedCollections[colIdx];
    if (!col) { hideCtxMenu(); return; }
    if (type === "request") {
      const req = folderIdx !== undefined
        ? col.folders[folderIdx]?.requests[reqIdx]
        : col.requests[reqIdx];
      if (req) {
        const name = window.prompt("Rename request:", req.name);
        if (name && name.trim()) req.name = name.trim();
      }
    } else if (type === "folder" && folderIdx !== undefined) {
      const folder = col.folders[folderIdx];
      if (folder) {
        const name = window.prompt("Rename folder:", folder.name);
        if (name && name.trim()) folder.name = name.trim();
      }
    } else if (type === "collection") {
      const name = window.prompt("Rename collection:", col.name);
      if (name && name.trim()) col.name = name.trim();
    }
    hideCtxMenu();
  }

  function ctxDelete() {
    if (!ctxMenu) return;
    const { colIdx, folderIdx, reqIdx, type } = ctxMenu.target;
    const col = loadedCollections[colIdx];
    if (!col) { hideCtxMenu(); return; }
    if (type === "request") {
      if (folderIdx !== undefined) {
        col.folders[folderIdx]?.requests.splice(reqIdx, 1);
      } else {
        col.requests.splice(reqIdx, 1);
      }
    } else if (type === "folder" && folderIdx !== undefined) {
      if (window.confirm(`Delete folder "${col.folders[folderIdx]?.name}" and all its requests?`)) {
        col.folders.splice(folderIdx, 1);
      }
    }
    hideCtxMenu();
  }

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
      const col = importHar(raw);
      loadedCollections.push(col);
      alert(`Imported "${col.name}" — ${col.requests.length} requests`);
    } catch (e) {
      alert("HAR import failed: " + e);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") sendRequest();
    if (e.key === "Escape" && ctxMenu) hideCtxMenu();
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

<svelte:window on:keydown={onKeydown} on:click={hideCtxMenu} />

<!-- Context menu overlay -->
{#if ctxMenu}
  <div
    class="ctx-menu"
    style="left:{ctxMenu.x}px; top:{ctxMenu.y}px"
    role="menu"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { if (e.key === "Escape") hideCtxMenu(); }}
  >
    {#if ctxMenu.target.type === "request"}
      <button class="ctx-item" onclick={ctxDuplicate}>Duplicate</button>
    {/if}
    <button class="ctx-item" onclick={ctxRename}>Rename…</button>
    {#if ctxMenu.target.type !== "collection"}
      <div class="ctx-sep"></div>
      <button class="ctx-item ctx-danger" onclick={ctxDelete}>Delete</button>
    {/if}
  </div>
{/if}

<div class="builder-mode">
  <!-- Tab Bar -->
  <div class="tab-row">
    <div class="tab-scroll scroll-y">
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

    <!-- HAR import button -->
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

  <!-- Collections sidebar + panels -->
  <div class="panels">
    <!-- Sidebar: loaded collections tree -->
    <div class="sidebar">
      <div class="sidebar-header">Collections</div>
      {#if loadedCollections.length === 0}
        <div class="sidebar-empty">No collections loaded.</div>
      {:else}
        {#each loadedCollections as col, ci}
          <details class="col-tree" open>
            <summary
              class="col-name"
              oncontextmenu={(e) => showCtxMenu(e, { type: "collection", colIdx: ci, reqIdx: -1 })}
            >{col.name}</summary>

            {#each col.requests as req, ri}
              <div
                class="tree-req"
                oncontextmenu={(e) => showCtxMenu(e, { type: "request", colIdx: ci, reqIdx: ri })}
                onclick={() => { const { loadRequestIntoTab } = require("../../stores/app.svelte"); loadRequestIntoTab(req); }}
                role="button" tabindex="0"
                onkeydown={(e) => e.key === "Enter" && (async () => { const m = await import("../../stores/app.svelte"); m.loadRequestIntoTab(req); })()}
              >
                <span class="badge method-{req.method}">{req.method}</span>
                <span class="req-name">{req.name}</span>
              </div>
            {/each}

            {#each col.folders as folder, fi}
              <details class="folder-tree" open>
                <summary
                  class="folder-name"
                  oncontextmenu={(e) => showCtxMenu(e, { type: "folder", colIdx: ci, folderIdx: fi, reqIdx: -1 })}
                >📁 {folder.name}</summary>
                {#each folder.requests as req, ri}
                  <div
                    class="tree-req tree-req--nested"
                    oncontextmenu={(e) => showCtxMenu(e, { type: "request", colIdx: ci, folderIdx: fi, reqIdx: ri })}
                    onclick={async () => { const m = await import("../../stores/app.svelte"); m.loadRequestIntoTab(req); }}
                    role="button" tabindex="0"
                    onkeydown={async (e) => { if (e.key === "Enter") { const m = await import("../../stores/app.svelte"); m.loadRequestIntoTab(req); } }}
                  >
                    <span class="badge method-{req.method}">{req.method}</span>
                    <span class="req-name">{req.name}</span>
                  </div>
                {/each}
              </details>
            {/each}
          </details>
        {/each}
      {/if}
    </div>

    <div class="main-panels">
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

<style>
  .builder-mode {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* Context menu */
  .ctx-menu {
    position: fixed;
    z-index: 9999;
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0,0,0,0.4);
    min-width: 140px;
    padding: 4px 0;
    display: flex;
    flex-direction: column;
  }
  .ctx-item {
    padding: 7px 14px;
    font-size: 12px;
    text-align: left;
    background: transparent;
    color: var(--text-primary);
    transition: var(--transition-fast);
  }
  .ctx-item:hover { background: var(--bg-elevated); }
  .ctx-danger { color: var(--color-error); }
  .ctx-sep { height: 1px; background: var(--border-default); margin: 3px 0; }

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
    color: var(--text-muted); font-size: 18px; line-height: 1;
    flex-shrink: 0; transition: var(--transition-fast);
  }
  .new-tab-btn:hover { color: var(--text-primary); background: var(--bg-elevated); }

  .har-btn {
    height: 24px; padding: 0 10px; margin-right: 8px;
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

  /* Sidebar */
  .sidebar {
    width: 220px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-default);
    background: var(--bg-surface);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }
  .sidebar-header {
    padding: 8px 12px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border-default);
    flex-shrink: 0;
  }
  .sidebar-empty {
    padding: 16px 12px;
    font-size: 11px;
    color: var(--text-muted);
  }
  .col-tree {
    border-bottom: 1px solid var(--border-subtle);
  }
  .col-name {
    padding: 8px 12px;
    font-size: 12px;
    font-weight: 700;
    color: var(--text-primary);
    cursor: pointer;
    list-style: none;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .col-name:hover { background: var(--bg-elevated); }
  .folder-tree { margin-left: 8px; }
  .folder-name {
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    cursor: pointer;
    list-style: none;
  }
  .folder-name:hover { color: var(--text-primary); }
  .tree-req {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    font-size: 11px;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: 0;
    transition: var(--transition-fast);
  }
  .tree-req:hover { background: var(--bg-elevated); color: var(--text-primary); }
  .tree-req--nested { padding-left: 24px; }
  .req-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .badge {
    font-size: 8px; font-weight: 800; font-family: var(--font-mono);
    padding: 1px 4px; border-radius: 3px; flex-shrink: 0;
  }
  .badge.method-GET    { background: rgba(63,185,80,0.2);   color: var(--method-get); }
  .badge.method-POST   { background: rgba(124,110,255,0.2); color: var(--method-post); }
  .badge.method-PUT    { background: rgba(227,179,65,0.2);  color: var(--method-put); }
  .badge.method-PATCH  { background: rgba(54,217,196,0.2);  color: var(--method-patch); }
  .badge.method-DELETE { background: rgba(248,81,73,0.2);   color: var(--method-delete); }
  .badge.method-WS     { background: rgba(0,255,255,0.15);  color: #00ffff; }
  .badge.method-SSE    { background: rgba(255,0,255,0.15);  color: #ff00ff; }
  .badge.method-GRPC   { background: rgba(251,188,4,0.15);  color: #fbbc04; }

  .main-panels {
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
    padding: 1px 4px; border-radius: 3px; opacity: 0.7;
  }
  .method-GET    { color: var(--method-get); }
  .method-POST   { color: var(--method-post); }
  .method-PUT    { color: var(--method-put); }
  .method-PATCH  { color: var(--method-patch); }
  .method-DELETE { color: var(--method-delete); }
  .method-HEAD, .method-OPTIONS { color: var(--text-muted); }
  .method-WS     { color: #00ffff; }
  .method-SSE    { color: #ff00ff; }
  .method-GRPC   { color: #fbbc04; }
</style>
