<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { currentWorkspace } from "../../stores/app.svelte";
  import { githubIdentity } from "../../stores/github.svelte";

  interface GitChange { path: string; status: string; }
  interface GitCommit { hash: string; author: string; email: string; message: string; timestamp: number; }
  interface GitBranch { name: string; is_current: boolean; is_remote: boolean; }

  let branch = $state<string | null>(null);
  let changes = $state<GitChange[]>([]);
  let commits = $state<GitCommit[]>([]);
  let branches = $state<GitBranch[]>([]);
  let commitMessage = $state("");
  let newBranchName = $state("");
  let remoteUrl = $state("");
  let showBranchMenu = $state(false);
  let showNewBranch = $state(false);
  let showRemoteForm = $state(false);
  let diffText = $state("");
  let showDiff = $state(false);
  let busy = $state(false);
  let statusMsg = $state("");

  const ws = $derived(currentWorkspace.path);

  async function refresh() {
    if (!ws) return;
    try {
      const s: any = await invoke("git_status", { path: ws });
      branch = s.branch ?? null;
      changes = s.changes ?? [];
    } catch { changes = []; }

    try {
      commits = await invoke("git_log", { path: ws, limit: 30 });
    } catch { commits = []; }

    try {
      branches = await invoke("git_branches", { path: ws });
    } catch { branches = []; }
  }

  async function commit() {
    if (!ws || !commitMessage.trim()) return;
    busy = true;
    const name = githubIdentity.value?.name ?? githubIdentity.value?.login ?? "Parallax User";
    const email = githubIdentity.value?.email ?? "user@parallax.local";
    try {
      const hash: string = await invoke("git_commit", {
        path: ws, message: commitMessage.trim(),
        authorName: name, authorEmail: email,
      });
      statusMsg = `Committed ${hash}`;
      commitMessage = "";
      await refresh();
    } catch (e: any) {
      statusMsg = `Commit failed: ${e}`;
    } finally {
      busy = false;
    }
  }

  async function push() {
    if (!ws) return;
    busy = true;
    try {
      await invoke("git_push", {
        path: ws, remoteName: "origin",
        branch: branch ?? "main",
        token: githubIdentity.value?.token ?? null,
      });
      statusMsg = "Pushed to origin";
    } catch (e: any) {
      statusMsg = `Push failed: ${e}`;
    } finally {
      busy = false;
    }
  }

  async function pull() {
    if (!ws) return;
    busy = true;
    try {
      await invoke("git_pull", {
        path: ws, remoteName: "origin",
        branch: branch ?? "main",
        token: githubIdentity.value?.token ?? null,
      });
      statusMsg = "Pulled from origin";
      await refresh();
    } catch (e: any) {
      statusMsg = `Pull failed: ${e}`;
    } finally {
      busy = false;
    }
  }

  async function stash() {
    if (!ws) return;
    await invoke("git_stash", { path: ws, message: null });
    statusMsg = "Stashed";
    await refresh();
  }

  async function stashPop() {
    if (!ws) return;
    try {
      await invoke("git_stash_pop", { path: ws });
      statusMsg = "Stash popped";
      await refresh();
    } catch (e: any) {
      statusMsg = `Stash pop failed: ${e}`;
    }
  }

  async function checkoutBranch(name: string) {
    if (!ws) return;
    try {
      await invoke("git_checkout_branch", { path: ws, branchName: name });
      showBranchMenu = false;
      await refresh();
    } catch (e: any) {
      statusMsg = `Checkout failed: ${e}`;
    }
  }

  async function createBranch() {
    if (!ws || !newBranchName.trim()) return;
    try {
      await invoke("git_create_branch", { path: ws, branchName: newBranchName.trim() });
      await checkoutBranch(newBranchName.trim());
      newBranchName = "";
      showNewBranch = false;
    } catch (e: any) {
      statusMsg = `Branch failed: ${e}`;
    }
  }

  async function setRemote() {
    if (!ws || !remoteUrl.trim()) return;
    try {
      await invoke("git_set_remote", { path: ws, url: remoteUrl.trim() });
      statusMsg = "Remote URL updated";
      showRemoteForm = false;
    } catch (e: any) {
      statusMsg = `Remote failed: ${e}`;
    }
  }

  async function viewDiff() {
    if (!ws) return;
    try {
      diffText = await invoke("git_diff", { path: ws });
      showDiff = true;
    } catch { diffText = ""; }
  }

  function statusColor(s: string) {
    if (s === "added") return "var(--color-success)";
    if (s === "deleted") return "var(--color-error)";
    return "var(--accent-secondary)";
  }

  onMount(refresh);

  $effect(() => {
    if (ws) refresh();
  });
</script>

<div class="dashboard-section animate-fade-in">
  <div class="section-header">
    <h2>Git Workspace</h2>
    <p class="section-desc">Commit, push, pull, and manage branches — all from within Parallax.</p>
  </div>

  {#if !ws}
    <div class="empty-state">Open a workspace to use git operations.</div>
  {:else}
    <!-- Branch bar -->
    <div class="branch-bar">
      <button class="branch-btn" onclick={() => (showBranchMenu = !showBranchMenu)}>
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
          <path d="M11.75 2.5a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0zm.75 2.25a2.25 2.25 0 1 1-1.5-2.121V6A2.5 2.5 0 0 1 8.5 8.5h-3a1 1 0 0 0-1 1v1.379a2.251 2.251 0 1 1-1.5 0V9.5a2.5 2.5 0 0 1 2.5-2.5h3a1 1 0 0 0 1-1V4.629A2.251 2.251 0 0 1 12.5 4.75z"/>
        </svg>
        {branch ?? "no branch"}
        <svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="6 9 12 15 18 9"/>
        </svg>
      </button>
      <span class="change-count" class:has-changes={changes.length > 0}>{changes.length} change{changes.length !== 1 ? "s" : ""}</span>
      <div class="spacer"></div>
      <button class="btn-sm" onclick={viewDiff} title="View diff">Diff</button>
      <button class="btn-sm" onclick={stash} title="Stash changes">Stash</button>
      <button class="btn-sm" onclick={stashPop} title="Pop stash">Pop</button>
      <button class="btn-sm" onclick={() => (showRemoteForm = !showRemoteForm)}>Remote</button>
      <button class="btn-sm" onclick={refresh}>Refresh</button>
    </div>

    {#if showBranchMenu}
      <div class="branch-menu">
        {#each branches.filter(b => !b.is_remote) as b}
          <button
            class="branch-item"
            class:current={b.is_current}
            onclick={() => checkoutBranch(b.name)}
          >
            <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor" style="flex-shrink:0">
              <path d="M11.75 2.5a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0zm.75 2.25a2.25 2.25 0 1 1-1.5-2.121V6A2.5 2.5 0 0 1 8.5 8.5h-3a1 1 0 0 0-1 1v1.379a2.251 2.251 0 1 1-1.5 0V9.5a2.5 2.5 0 0 1 2.5-2.5h3a1 1 0 0 0 1-1V4.629A2.251 2.251 0 0 1 12.5 4.75z"/>
            </svg>
            {b.name}
            {#if b.is_current}<span class="current-dot">●</span>{/if}
          </button>
        {/each}
        <button class="branch-item new-branch-btn" onclick={() => { showNewBranch = !showNewBranch; }}>
          + New Branch
        </button>
        {#if showNewBranch}
          <div class="new-branch-row">
            <input class="mini-input" bind:value={newBranchName} placeholder="branch-name" onkeydown={(e) => e.key === "Enter" && createBranch()} />
            <button class="btn-sm accent" onclick={createBranch}>Create</button>
          </div>
        {/if}
      </div>
    {/if}

    {#if showRemoteForm}
      <div class="remote-form">
        <input class="form-input flex-1" bind:value={remoteUrl} placeholder="https://github.com/you/repo.git" />
        <button class="btn-sm accent" onclick={setRemote}>Save Remote</button>
      </div>
    {/if}

    <!-- Changes list -->
    {#if changes.length > 0}
      <div class="changes-list">
        {#each changes as c}
          <div class="change-row">
            <span class="change-status" style="color:{statusColor(c.status)}">{c.status[0].toUpperCase()}</span>
            <span class="change-path">{c.path}</span>
          </div>
        {/each}
      </div>
    {:else}
      <div class="no-changes">Working tree clean</div>
    {/if}

    <!-- Commit form -->
    <div class="commit-form">
      <textarea
        class="commit-input"
        placeholder="Commit message…"
        bind:value={commitMessage}
        rows="2"
      ></textarea>
      <div class="commit-actions">
        <button class="btn-primary" onclick={commit} disabled={busy || !commitMessage.trim()}>
          {busy ? "…" : "Commit"}
        </button>
        <button class="btn-action" onclick={push} disabled={busy}>Push</button>
        <button class="btn-action" onclick={pull} disabled={busy}>Pull</button>
      </div>
    </div>

    {#if statusMsg}
      <div class="status-msg">{statusMsg}</div>
    {/if}

    <!-- Diff viewer -->
    {#if showDiff && diffText}
      <div class="diff-header">
        <span>Diff</span>
        <button class="btn-sm" onclick={() => (showDiff = false)}>Close</button>
      </div>
      <pre class="diff-view">{diffText}</pre>
    {/if}

    <!-- Commit log -->
    {#if commits.length > 0}
      <div class="log-header">Commit History</div>
      <div class="log-list">
        {#each commits as c (c.hash)}
          <div class="log-row">
            <span class="log-hash">{c.hash}</span>
            <span class="log-message">{c.message}</span>
            <span class="log-author">{c.author}</span>
            <span class="log-time">{new Date(c.timestamp * 1000).toLocaleDateString()}</span>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .dashboard-section { max-width: 800px; display: flex; flex-direction: column; gap: 12px; }
  .section-header { margin-bottom: 4px; }
  .section-header h2 { font-size: 18px; font-weight: 700; margin-bottom: 4px; }
  .section-desc { font-size: 12px; color: var(--text-secondary); }

  .branch-bar {
    display: flex; align-items: center; gap: 8px;
    background: var(--bg-surface); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); padding: 8px 12px;
  }
  .branch-btn {
    display: flex; align-items: center; gap: 5px; font-size: 12px;
    font-family: var(--font-mono); color: var(--accent-secondary);
    background: none; border: none; cursor: pointer; padding: 0;
  }
  .branch-btn:hover { color: var(--text-primary); }
  .change-count { font-size: 11px; color: var(--text-muted); }
  .change-count.has-changes { color: var(--color-warning); }
  .spacer { flex: 1; }

  .btn-sm {
    height: 26px; padding: 0 10px; background: var(--bg-elevated);
    border: 1px solid var(--border-default); color: var(--text-secondary);
    font-size: 11px; font-weight: 600; border-radius: var(--radius-sm);
    cursor: pointer; transition: var(--transition-fast);
  }
  .btn-sm:hover { border-color: var(--accent-primary); color: var(--accent-primary); }
  .btn-sm.accent { background: var(--accent-primary); border-color: var(--accent-primary); color: white; }
  .btn-sm.accent:hover { filter: brightness(1.1); }

  .branch-menu {
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); padding: 4px 0;
  }
  .branch-item {
    display: flex; align-items: center; gap: 8px; width: 100%;
    padding: 6px 12px; font-size: 12px; color: var(--text-secondary);
    background: none; text-align: left; cursor: pointer;
    transition: var(--transition-fast);
  }
  .branch-item:hover { background: var(--bg-overlay); color: var(--text-primary); }
  .branch-item.current { color: var(--accent-primary); }
  .current-dot { margin-left: auto; font-size: 8px; color: var(--accent-primary); }
  .new-branch-btn { color: var(--text-muted); border-top: 1px solid var(--border-subtle); margin-top: 4px; padding-top: 8px; }
  .new-branch-row { display: flex; gap: 8px; padding: 6px 12px; }
  .mini-input {
    flex: 1; height: 26px; padding: 0 8px; font-size: 11px;
    background: var(--bg-input); border: 1px solid var(--border-default);
    border-radius: var(--radius-sm); color: var(--text-primary);
  }

  .remote-form { display: flex; gap: 8px; align-items: center; }
  .form-input {
    height: 30px; padding: 0 10px; background: var(--bg-input);
    border: 1px solid var(--border-default); border-radius: var(--radius-md);
    color: var(--text-primary); font-size: 12px;
  }
  .flex-1 { flex: 1; }

  .changes-list {
    background: var(--bg-surface); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); max-height: 160px; overflow-y: auto;
  }
  .change-row {
    display: flex; gap: 10px; align-items: center;
    padding: 4px 12px; font-size: 11px; border-bottom: 1px solid var(--border-subtle);
  }
  .change-row:last-child { border-bottom: none; }
  .change-status { font-weight: 700; font-family: var(--font-mono); font-size: 10px; width: 12px; }
  .change-path { font-family: var(--font-mono); color: var(--text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .no-changes { font-size: 12px; color: var(--color-success); padding: 8px 0; }

  .commit-form {
    background: var(--bg-surface); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); padding: 12px; display: flex; flex-direction: column; gap: 8px;
  }
  .commit-input {
    background: var(--bg-input); border: 1px solid var(--border-default);
    border-radius: var(--radius-sm); color: var(--text-primary); font-size: 12px;
    padding: 8px; resize: none; font-family: var(--font-sans);
  }
  .commit-input:focus { border-color: var(--accent-primary); outline: none; }
  .commit-actions { display: flex; gap: 8px; }

  .btn-primary {
    height: 30px; padding: 0 16px; background: var(--accent-primary); color: white;
    border: none; border-radius: var(--radius-md); font-weight: 600; font-size: 12px;
    cursor: pointer; transition: var(--transition-fast);
  }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn-action {
    height: 30px; padding: 0 14px; background: var(--bg-elevated);
    border: 1px solid var(--border-default); color: var(--text-secondary);
    font-size: 12px; font-weight: 600; border-radius: var(--radius-md);
    cursor: pointer; transition: var(--transition-fast);
  }
  .btn-action:hover:not(:disabled) { border-color: var(--accent-primary); color: var(--accent-primary); }
  .btn-action:disabled { opacity: 0.5; cursor: not-allowed; }

  .status-msg {
    font-size: 11px; color: var(--accent-secondary); padding: 4px 0;
    font-family: var(--font-mono);
  }

  .diff-header {
    display: flex; justify-content: space-between; align-items: center;
    font-size: 11px; font-weight: 700; color: var(--text-muted);
    text-transform: uppercase; letter-spacing: .06em; padding: 4px 0;
  }
  .diff-view {
    background: var(--bg-surface); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); padding: 12px; font-size: 11px;
    font-family: var(--font-mono); max-height: 260px; overflow: auto;
    white-space: pre; color: var(--text-secondary);
  }

  .log-header {
    font-size: 10px; font-weight: 700; color: var(--text-muted);
    text-transform: uppercase; letter-spacing: .06em; padding: 4px 0;
  }
  .log-list {
    background: var(--bg-surface); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); overflow: hidden;
  }
  .log-row {
    display: flex; gap: 12px; padding: 6px 12px; border-bottom: 1px solid var(--border-subtle);
    font-size: 11px; align-items: center;
  }
  .log-row:last-child { border-bottom: none; }
  .log-hash { font-family: var(--font-mono); color: var(--accent-secondary); width: 50px; flex-shrink: 0; }
  .log-message { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-primary); }
  .log-author { color: var(--text-muted); font-size: 10px; width: 100px; flex-shrink: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .log-time { color: var(--text-muted); font-size: 10px; width: 70px; flex-shrink: 0; }

  .empty-state { text-align: center; padding: 40px; color: var(--text-muted); font-size: 13px; border: 1px dashed var(--border-default); border-radius: var(--radius-md); }
</style>
