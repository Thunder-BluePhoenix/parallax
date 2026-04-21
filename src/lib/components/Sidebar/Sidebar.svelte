<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    tabs, activeRequest, currentWorkspace, appMode,
    loadedCollections, activeEnvironment, loadRequestIntoTab,
    type Collection, defaultAuth,
  } from "../../stores/app.svelte";
  import EnvironmentPanel from "./EnvironmentPanel.svelte";
  import { importPostmanCollection, importInsomniaExport } from "../../utils/postman-importer";

  let searchQuery   = $state("");
  let showEnvPanel  = $state(false);
  let importError   = $state("");
  let expandedCols  = $state<Record<string, boolean>>({});
  let expandedFolders = $state<Record<string, boolean>>({});

  // ── Filtered view of requests across all collections ──────
  let filteredCollections = $derived.by(() => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return loadedCollections;
    return loadedCollections.map((col) => ({
      ...col,
      requests: col.requests.filter(
        (r) => r.name.toLowerCase().includes(q) || r.url.toLowerCase().includes(q)
      ),
      folders: col.folders.map((f) => ({
        ...f,
        requests: f.requests.filter(
          (r) => r.name.toLowerCase().includes(q) || r.url.toLowerCase().includes(q)
        ),
      })).filter((f) => f.requests.length > 0),
    })).filter((c) => c.requests.length > 0 || c.folders.length > 0);
  });

  // ── Open workspace folder ──────────────────────────────────
  async function openWorkspace() {
    const selected = await open({ directory: true, multiple: false, title: "Open Workspace" });
    if (!selected || typeof selected !== "string") return;

    const info: any = await invoke("open_workspace", { path: selected });
    currentWorkspace.path      = info.root;
    currentWorkspace.name      = info.name;
    currentWorkspace.gitBranch = info.git_branch;
    currentWorkspace.hasParallax = info.has_parallax;

    await loadCollections(selected);
  }

  async function loadCollections(workspace: string) {
    const names: string[] = await invoke("list_collections", { workspace });
    const collections: Collection[] = [];
    for (const name of names) {
      try {
        const col: Collection = await invoke("load_collection", { workspace, name });
        collections.push(col);
        expandedCols[col.name] = true; // auto-expand first load
      } catch { /* skip malformed */ }
    }
    loadedCollections.splice(0, loadedCollections.length, ...collections);
  }

  // ── New blank request ──────────────────────────────────────
  function newRequest() {
    const id = crypto.randomUUID();
    tabs.list.push({ id, name: "New Request", method: "GET", modified: false });
    tabs.activeId = id;
    activeRequest.id          = id;
    activeRequest.name        = "New Request";
    activeRequest.method      = "GET";
    activeRequest.url         = "";
    activeRequest.headers     = {};
    activeRequest.params      = {};
    activeRequest.bodyType    = "none";
    activeRequest.bodyContent = "";
    activeRequest.auth        = defaultAuth();
  }

  async function importCollection(e: Event) {
    importError = "";
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const text = await file.text();
    try {
      let col;
      if (text.includes('"__export_format"') || text.includes('"_type":"workspace"')) {
        col = importInsomniaExport(text);
      } else {
        col = importPostmanCollection(text);
      }
      if (currentWorkspace.path) {
        await invoke("save_collection", { workspace: currentWorkspace.path, collection: col });
        await loadCollections(currentWorkspace.path);
      } else {
        loadedCollections.push(col);
        expandedCols[col.name] = true;
      }
    } catch (err: any) {
      importError = err?.message ?? "Import failed";
    }
    (e.target as HTMLInputElement).value = "";
  }

  function toggleCol(name: string) {
    expandedCols[name] = !expandedCols[name];
  }
  function toggleFolder(key: string) {
    expandedFolders[key] = !expandedFolders[key];
  }

  function methodColor(m: string) {
    const map: Record<string, string> = {
      GET: "method-GET", POST: "method-POST", PUT: "method-PUT",
      PATCH: "method-PATCH", DELETE: "method-DELETE",
      HEAD: "method-HEAD", OPTIONS: "method-OPTIONS",
    };
    return map[m.toUpperCase()] ?? "method-GET";
  }
</script>

<aside class="sidebar">
  <!-- Workspace header -->
  <div class="sidebar-header">
    <button class="workspace-btn" onclick={openWorkspace} title="Open Workspace">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>
      <span class="ws-name">{currentWorkspace.name || "Open Workspace"}</span>
      {#if currentWorkspace.gitBranch}
        <span class="git-chip">
          <svg width="9" height="9" viewBox="0 0 16 16" fill="currentColor">
            <path d="M11.75 2.5a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0zm.75 2.25a2.25 2.25 0 1 1-1.5-2.121V6A2.5 2.5 0 0 1 8.5 8.5h-3a1 1 0 0 0-1 1v1.379a2.251 2.251 0 1 1-1.5 0V9.5a2.5 2.5 0 0 1 2.5-2.5h3a1 1 0 0 0 1-1V4.629A2.251 2.251 0 0 1 12.5 4.75z"/>
          </svg>
          {currentWorkspace.gitBranch}
        </span>
      {/if}
    </button>
    <button class="icon-btn" onclick={newRequest} title="New Request (Ctrl+N)">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
    </button>
    <label class="icon-btn" title="Import Postman / Insomnia collection">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
        <polyline points="7 10 12 15 17 10"/>
        <line x1="12" y1="15" x2="12" y2="3"/>
      </svg>
      <input type="file" accept=".json" style="display:none" onchange={importCollection} />
    </label>
  </div>
  {#if importError}
    <div class="import-error">{importError}</div>
  {/if}

  <!-- Search -->
  <div class="search-box">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="search-icon">
      <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
    </svg>
    <input type="text" placeholder="Search requests…" class="search-input" bind:value={searchQuery} />
    {#if searchQuery}
      <button class="clear-search" onclick={() => (searchQuery = "")}>×</button>
    {/if}
  </div>

  <!-- Collections tree -->
  <div class="scroll-y sidebar-tree">
    {#if loadedCollections.length === 0}
      <div class="empty-state">
        {#if currentWorkspace.path}
          <p>No collections in this workspace.</p>
          <button class="link-btn" onclick={newRequest}>Create a request →</button>
        {:else}
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" opacity="0.3">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
          <p>Open a workspace to see collections.</p>
          <button class="link-btn" onclick={openWorkspace}>Open folder →</button>
        {/if}
      </div>
    {:else}
      {#each filteredCollections as col}
        <!-- Collection header -->
        <button
          class="col-header"
          onclick={() => toggleCol(col.name)}
          title={col.description ?? col.name}
        >
          <svg
            class="chevron"
            class:open={expandedCols[col.name]}
            width="10" height="10" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2.5"
          >
            <polyline points="9 18 15 12 9 6"/>
          </svg>
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="col-icon">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
          </svg>
          <span class="col-name">{col.name}</span>
          <span class="count">{col.requests.length + col.folders.reduce((n,f)=>n+f.requests.length,0)}</span>
        </button>

        {#if expandedCols[col.name]}
          <!-- Top-level requests -->
          {#each col.requests as req}
            <button
              class="req-item"
              class:active={tabs.activeId === req.id}
              onclick={() => { loadRequestIntoTab(req); appMode.value = "builder"; }}
            >
              <span class="method-badge {methodColor(req.method)}">{req.method}</span>
              <span class="req-name">{req.name}</span>
            </button>
          {/each}

          <!-- Folders -->
          {#each col.folders as folder}
            {@const fkey = col.name + "/" + folder.name}
            <button class="folder-header" onclick={() => toggleFolder(fkey)}>
              <svg class="chevron sm" class:open={expandedFolders[fkey]}
                width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="9 18 15 12 9 6"/>
              </svg>
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
              <span class="folder-name">{folder.name}</span>
              <span class="count sm">{folder.requests.length}</span>
            </button>
            {#if expandedFolders[fkey]}
              {#each folder.requests as req}
                <button
                  class="req-item nested"
                  class:active={tabs.activeId === req.id}
                  onclick={() => { loadRequestIntoTab(req); appMode.value = "builder"; }}
                >
                  <span class="method-badge {methodColor(req.method)}">{req.method}</span>
                  <span class="req-name">{req.name}</span>
                </button>
              {/each}
            {/if}
          {/each}
        {/if}
      {/each}
    {/if}

    <!-- Ecosystem section -->
    <div class="section-sep"></div>
    <button class="col-header eco" onclick={() => (appMode.value = "dashboard")}>
      <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
        <rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>
      </svg>
      <span class="col-name">Dashboard</span>
    </button>
  </div>

  <!-- Env indicator + footer -->
  <div class="sidebar-footer">
    <button
      class="env-btn"
      class:active={showEnvPanel}
      onclick={() => (showEnvPanel = !showEnvPanel)}
      title="Environment variables"
    >
      <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/>
      </svg>
      <span class="env-name">{activeEnvironment.name}</span>
      {#if Object.keys(activeEnvironment.variables).length > 0}
        <span class="env-count">{Object.keys(activeEnvironment.variables).length}</span>
      {/if}
    </button>
  </div>

  <!-- Environment panel overlay -->
  {#if showEnvPanel}
    <EnvironmentPanel onclose={() => (showEnvPanel = false)} />
  {/if}
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    background: var(--bg-surface);
    border-right: 1px solid var(--border-default);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    overflow: hidden;
    position: relative;
  }

  /* Header */
  .sidebar-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 8px 8px 8px 10px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .workspace-btn {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    text-align: left;
    overflow: hidden;
    transition: var(--transition-fast);
  }
  .workspace-btn:hover { background: var(--bg-elevated); color: var(--text-primary); }

  .ws-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .git-chip {
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--accent-secondary);
    background: var(--accent-secondary-dim);
    padding: 1px 5px;
    border-radius: 10px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .import-error {
    font-size: 10px;
    color: var(--color-error);
    padding: 3px 10px;
    background: var(--color-error-dim);
    border-bottom: 1px solid var(--border-subtle);
  }

  /* Search */
  .search-box {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 6px 8px;
    padding: 5px 8px;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    flex-shrink: 0;
  }
  .search-icon { color: var(--text-muted); flex-shrink: 0; }
  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-size: 12px;
    padding: 0;
    box-shadow: none;
  }
  .search-input::placeholder { color: var(--text-muted); }
  .clear-search {
    background: transparent;
    color: var(--text-muted);
    font-size: 14px;
    line-height: 1;
    border-radius: 3px;
    padding: 0 2px;
    transition: var(--transition-fast);
  }
  .clear-search:hover { color: var(--text-primary); }

  /* Tree */
  .sidebar-tree { flex: 1; padding: 4px 0; }

  .col-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 10px 5px 8px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.02em;
    text-align: left;
    transition: var(--transition-fast);
  }
  .col-header:hover { background: var(--bg-elevated); color: var(--text-primary); }
  .col-header.eco { color: var(--text-muted); margin-top: 2px; }

  .col-icon { color: var(--text-muted); flex-shrink: 0; }
  .col-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .folder-header {
    display: flex;
    align-items: center;
    gap: 5px;
    width: 100%;
    padding: 4px 8px 4px 20px;
    background: transparent;
    color: var(--text-muted);
    font-size: 11px;
    text-align: left;
    transition: var(--transition-fast);
  }
  .folder-header:hover { background: var(--bg-elevated); color: var(--text-secondary); }
  .folder-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .req-item {
    display: flex;
    align-items: center;
    gap: 7px;
    width: 100%;
    padding: 4px 8px 4px 26px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    text-align: left;
    transition: var(--transition-fast);
    border-left: 2px solid transparent;
  }
  .req-item.nested { padding-left: 36px; }
  .req-item:hover { background: var(--bg-elevated); color: var(--text-primary); }
  .req-item.active {
    background: var(--accent-primary-dim);
    color: var(--text-primary);
    border-left-color: var(--accent-primary);
  }
  .req-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Method badge override for sidebar (smaller) */
  .req-item .method-badge {
    font-size: 9px;
    padding: 1px 4px;
    min-width: 44px;
    flex-shrink: 0;
  }

  .chevron {
    flex-shrink: 0;
    transition: transform var(--transition-fast);
    color: var(--text-muted);
  }
  .chevron.open { transform: rotate(90deg); }

  .count {
    margin-left: auto;
    background: var(--bg-overlay);
    color: var(--text-muted);
    font-size: 9px;
    padding: 1px 5px;
    border-radius: 10px;
    flex-shrink: 0;
  }
  .section-sep {
    height: 1px;
    background: var(--border-subtle);
    margin: 6px 10px;
  }

  /* Empty state */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 32px 16px;
    color: var(--text-muted);
    font-size: 11px;
    text-align: center;
    line-height: 1.5;
  }
  .link-btn {
    background: transparent;
    color: var(--accent-primary);
    font-size: 11px;
    transition: var(--transition-fast);
  }
  .link-btn:hover { color: #9185ff; }

  /* Footer */
  .sidebar-footer {
    border-top: 1px solid var(--border-subtle);
    padding: 5px 8px;
    flex-shrink: 0;
  }

  .env-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 8px;
    background: transparent;
    color: var(--text-muted);
    font-size: 11px;
    border-radius: var(--radius-sm);
    transition: var(--transition-fast);
    text-align: left;
  }
  .env-btn:hover { background: var(--bg-elevated); color: var(--text-secondary); }
  .env-btn.active { background: var(--accent-primary-dim); color: var(--accent-primary); }

  .env-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .env-count {
    background: var(--accent-primary-dim);
    color: var(--accent-primary);
    font-size: 9px;
    padding: 1px 5px;
    border-radius: 10px;
    flex-shrink: 0;
  }
</style>
