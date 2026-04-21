<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { tabs, activeRequest, currentWorkspace, appMode } from "../../stores/app.svelte";

  let collections = $state<string[]>([]);
  let expanded = $state<Record<string, boolean>>({ "Collections": true });

  async function openWorkspace() {
    const selected = await open({ directory: true, multiple: false, title: "Open Workspace" });
    if (!selected) return;
    const info: any = await invoke("open_workspace", { path: selected });
    currentWorkspace.path = info.root;
    currentWorkspace.name = info.name;
    currentWorkspace.gitBranch = info.git_branch;
    currentWorkspace.hasParallax = info.has_parallax;

    collections = await invoke("list_collections", { workspace: selected });
  }

  function newRequest() {
    const id = crypto.randomUUID();
    tabs.list.push({ id, name: "New Request", method: "GET", modified: false });
    tabs.activeId = id;
    activeRequest.id = id;
    activeRequest.name = "New Request";
    activeRequest.method = "GET";
    activeRequest.url = "";
  }

  function toggle(section: string) {
    expanded[section] = !expanded[section];
  }

  const navItems = [
    { icon: "builder", label: "Builder", mode: "builder" },
    { icon: "dashboard", label: "Dashboard", mode: "dashboard" },
  ] as const;
</script>

<aside class="sidebar">
  <!-- Workspace header -->
  <div class="sidebar-header">
    <button class="workspace-btn" onclick={openWorkspace} title="Open Workspace">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>
      <span>{currentWorkspace.name || "Open Workspace"}</span>
    </button>

    <button class="icon-btn" onclick={newRequest} title="New Request">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19"></line>
        <line x1="5" y1="12" x2="19" y2="12"></line>
      </svg>
    </button>
  </div>

  <!-- Search -->
  <div class="search-box">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="search-icon">
      <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
    </svg>
    <input type="text" placeholder="Search requests..." class="search-input" />
  </div>

  <!-- Collections tree -->
  <div class="scroll-y sidebar-tree">
    <div class="tree-section">
      <button class="tree-section-header" onclick={() => toggle("Collections")}>
        <svg class="chevron" class:open={expanded["Collections"]} width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="9 18 15 12 9 6"/>
        </svg>
        <span>Collections</span>
        <span class="count">{collections.length}</span>
      </button>

      {#if expanded["Collections"]}
        {#if collections.length === 0}
          <div class="empty-tree">
            <span>No collections yet</span>
            <button class="link-btn" onclick={newRequest}>Create one →</button>
          </div>
        {:else}
          {#each collections as col}
            <button class="tree-item">
              <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
              </svg>
              {col}
            </button>
          {/each}
        {/if}
      {/if}
    </div>

    <div class="tree-section">
      <button class="tree-section-header" onclick={() => toggle("Ecosystem")}>
        <svg class="chevron" class:open={expanded["Ecosystem"]} width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="9 18 15 12 9 6"/>
        </svg>
        <span>Ecosystem</span>
      </button>

      {#if expanded["Ecosystem"]}
        <button class="tree-item ecosystem-item" onclick={() => appMode.value = "dashboard"}>
          <span class="eco-badge frappe">F</span>
          Frappe Auth
        </button>
        <button class="tree-item ecosystem-item">
          <span class="eco-badge django">D</span>
          Django Auth
        </button>
        <button class="tree-item ecosystem-item">
          <span class="eco-badge openapi">API</span>
          Schema Explorer
        </button>
      {/if}
    </div>
  </div>

  <!-- Bottom nav -->
  <div class="sidebar-footer">
    <button class="footer-btn" title="Environments">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/>
      </svg>
      Environments
    </button>
    <button class="footer-btn" title="History">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="12 8 12 12 14 14"/>
        <path d="M3.05 11a9 9 0 1 0 .5-4.5"/>
        <polyline points="3 3 3 11 11 11"/>
      </svg>
      History
    </button>
  </div>
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
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 8px 10px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .workspace-btn {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 7px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    text-align: left;
    overflow: hidden;
    transition: var(--transition-fast);
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .workspace-btn:hover { background: var(--bg-elevated); color: var(--text-primary); }

  .search-box {
    display: flex;
    align-items: center;
    gap: 7px;
    margin: 8px 10px;
    padding: 6px 10px;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .search-icon { color: var(--text-muted); flex-shrink: 0; }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-size: 12px;
    padding: 0;
  }
  .search-input::placeholder { color: var(--text-muted); }

  .sidebar-tree {
    flex: 1;
    padding: 4px 0;
  }

  .tree-section {
    margin-bottom: 4px;
  }

  .tree-section-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 12px;
    background: transparent;
    color: var(--text-muted);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    transition: var(--transition-fast);
  }
  .tree-section-header:hover { color: var(--text-secondary); }

  .chevron {
    transition: transform var(--transition-fast);
    flex-shrink: 0;
  }
  .chevron.open { transform: rotate(90deg); }

  .count {
    margin-left: auto;
    background: var(--bg-overlay);
    color: var(--text-muted);
    font-size: 9px;
    padding: 1px 5px;
    border-radius: 10px;
  }

  .tree-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 5px 14px 5px 24px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    text-align: left;
    transition: var(--transition-fast);
    border-radius: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .tree-item:hover { background: var(--bg-elevated); color: var(--text-primary); }

  .ecosystem-item { padding-left: 16px; }

  .eco-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 16px;
    border-radius: 3px;
    font-size: 9px;
    font-weight: 700;
    flex-shrink: 0;
  }
  .frappe { background: rgba(124, 110, 255, 0.2); color: var(--accent-primary); }
  .django { background: rgba(54, 217, 196, 0.2); color: var(--accent-secondary); }
  .openapi { background: rgba(63, 185, 80, 0.2); color: var(--color-success); }

  .empty-tree {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 16px;
    color: var(--text-muted);
    font-size: 11px;
  }

  .link-btn {
    background: transparent;
    color: var(--accent-primary);
    font-size: 11px;
  }

  .sidebar-footer {
    border-top: 1px solid var(--border-subtle);
    padding: 6px;
    display: flex;
    gap: 2px;
  }

  .footer-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    padding: 6px;
    background: transparent;
    color: var(--text-muted);
    font-size: 11px;
    border-radius: var(--radius-sm);
    transition: var(--transition-fast);
  }
  .footer-btn:hover { background: var(--bg-elevated); color: var(--text-secondary); }
</style>
