<script lang="ts">
  import { appMode, currentWorkspace } from "../stores/app.svelte";
  import { githubIdentity, loadGitHubIdentity, unreadCount } from "../stores/github.svelte";
  import Logo from "./Common/Logo.svelte";
  import { onMount } from "svelte";

  onMount(loadGitHubIdentity);
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="titlebar-left">
    <div class="logo">
      <Logo size={20} hasBackground={true} />
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
    {#if githubIdentity.value}
      <button
        class="titlebar-action gh-identity"
        title="@{githubIdentity.value.login}"
        onclick={() => { appMode.value = "dashboard"; }}
      >
        <img src={githubIdentity.value.avatar_url} alt={githubIdentity.value.login} class="gh-avatar" />
        <span class="gh-login">@{githubIdentity.value.login}</span>
        {#if unreadCount.value > 0}
          <span class="unread-dot">{unreadCount.value}</span>
        {/if}
      </button>
    {:else}
      <button
        class="titlebar-action"
        title="Sign in with GitHub"
        onclick={() => { appMode.value = "dashboard"; }}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/>
        </svg>
        <span style="font-size:11px">GitHub</span>
      </button>
    {/if}
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

  .gh-identity {
    display: flex; align-items: center; gap: 6px;
    width: auto; padding: 0 8px; border-radius: var(--radius-sm);
  }
  .gh-avatar { width: 20px; height: 20px; border-radius: 50%; }
  .gh-login { font-size: 11px; font-weight: 600; color: var(--text-secondary); }
  .unread-dot {
    background: var(--color-error); color: white; font-size: 9px; font-weight: 700;
    padding: 1px 5px; border-radius: 10px; min-width: 16px; text-align: center;
  }
</style>
