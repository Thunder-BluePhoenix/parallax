<script lang="ts">
  import { appMode, currentWorkspace } from "../../stores/app.svelte";

  function toggleMode() {
    appMode.value = appMode.value === "builder" ? "dashboard" : "builder";
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="titlebar-left">
    <div class="logo">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="3" fill="var(--accent-primary)" />
        <circle cx="5" cy="5" r="2" fill="var(--accent-secondary)" opacity="0.7" />
        <circle cx="19" cy="5" r="1.5" fill="var(--accent-primary)" opacity="0.5" />
        <circle cx="5" cy="19" r="1.5" fill="var(--accent-secondary)" opacity="0.4" />
        <circle cx="19" cy="19" r="2" fill="var(--accent-primary)" opacity="0.6" />
        <line x1="12" y1="12" x2="5" y2="5" stroke="var(--accent-primary)" stroke-width="0.5" opacity="0.3" />
        <line x1="12" y1="12" x2="19" y2="5" stroke="var(--accent-secondary)" stroke-width="0.5" opacity="0.3" />
        <line x1="12" y1="12" x2="5" y2="19" stroke="var(--accent-secondary)" stroke-width="0.5" opacity="0.3" />
        <line x1="12" y1="12" x2="19" y2="19" stroke="var(--accent-primary)" stroke-width="0.5" opacity="0.3" />
      </svg>
      <span class="logo-text">Parallax</span>
    </div>

    {#if currentWorkspace.path}
      <div class="workspace-pill">
        <span class="workspace-name">{currentWorkspace.name}</span>
        {#if currentWorkspace.gitBranch}
          <span class="git-branch">
            <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor">
              <path d="M11.75 2.5a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0zm.75 2.25a2.25 2.25 0 1 1-1.5-2.121V6A2.5 2.5 0 0 1 8.5 8.5h-3a1 1 0 0 0-1 1v1.379a2.251 2.251 0 1 1-1.5 0V9.5a2.5 2.5 0 0 1 2.5-2.5h3a1 1 0 0 0 1-1V4.629A2.251 2.251 0 0 1 12.5 4.75z"/>
            </svg>
            {currentWorkspace.gitBranch}
          </span>
        {/if}
      </div>
    {/if}
  </div>

  <div class="titlebar-center">
    <div class="mode-toggle">
      <button
        class="mode-btn"
        class:active={appMode.value === "builder"}
        onclick={() => (appMode.value = "builder")}
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="16 18 22 12 16 6"/>
          <polyline points="8 6 2 12 8 18"/>
        </svg>
        Builder
      </button>
      <button
        class="mode-btn"
        class:active={appMode.value === "dashboard"}
        onclick={() => (appMode.value = "dashboard")}
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="7" height="7"/>
          <rect x="14" y="3" width="7" height="7"/>
          <rect x="14" y="14" width="7" height="7"/>
          <rect x="3" y="14" width="7" height="7"/>
        </svg>
        Dashboard
      </button>
    </div>
  </div>

  <div class="titlebar-right">
    <button class="titlebar-action" title="Settings">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/>
      </svg>
    </button>
    <button class="titlebar-action" title="AI Assistant">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
        <polyline points="3.27 6.96 12 12.01 20.73 6.96"/>
        <line x1="12" y1="22.08" x2="12" y2="12"/>
      </svg>
    </button>
  </div>
</header>

<style>
  .titlebar {
    height: var(--header-height);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
    -webkit-app-region: drag;
    gap: 12px;
  }

  .titlebar-left, .titlebar-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    -webkit-app-region: no-drag;
  }

  .titlebar-center {
    display: flex;
    align-items: center;
    -webkit-app-region: no-drag;
  }

  .titlebar-right {
    justify-content: flex-end;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 7px;
  }

  .logo-text {
    font-size: 14px;
    font-weight: 700;
    letter-spacing: -0.3px;
    background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .workspace-pill {
    display: flex;
    align-items: center;
    gap: 5px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    border-radius: 20px;
    padding: 3px 10px;
    font-size: 11px;
  }

  .workspace-name {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .git-branch {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--accent-secondary);
    font-family: var(--font-mono);
    font-size: 10px;
  }

  .mode-toggle {
    display: flex;
    background: var(--bg-void);
    border: 1px solid var(--border-default);
    border-radius: 6px;
    padding: 2px;
    gap: 1px;
  }

  .mode-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 12px;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    background: transparent;
    border-radius: 4px;
    transition: var(--transition-fast);
  }

  .mode-btn:hover { color: var(--text-primary); background: var(--bg-elevated); }

  .mode-btn.active {
    color: var(--accent-primary);
    background: var(--accent-primary-dim);
  }

  .titlebar-action {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    color: var(--text-secondary);
    background: transparent;
    border-radius: var(--radius-sm);
    transition: var(--transition-fast);
  }
  .titlebar-action:hover { background: var(--bg-elevated); color: var(--text-primary); }
</style>
