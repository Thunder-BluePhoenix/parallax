<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { githubIdentity, chatPresence, startGitHubLogin, pollGitHubLogin, type DeviceCodeInfo } from "../../stores/github.svelte";

  interface Collaborator { login: string; avatar_url: string; role: string; }

  let repoOwner = $state("");
  let repoName = $state("");
  let collaborators = $state<Collaborator[]>([]);
  let inviteUser = $state("");
  let clientId = $state(localStorage.getItem("parallax:github_client_id") ?? "");
  let loadError = $state("");
  let busy = $state(false);
  let statusMsg = $state("");

  // GitHub Device Auth state
  let deviceInfo = $state<DeviceCodeInfo | null>(null);
  let polling = $state(false);
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  const isLoggedIn = $derived(!!githubIdentity.value);
  const token = $derived(githubIdentity.value?.token ?? "");

  function isOnline(login: string) {
    const cutoff = Date.now() / 1000 - 300;
    return chatPresence.some((p) => p.login === login && p.ts > cutoff);
  }

  async function loadCollaborators() {
    if (!repoOwner || !repoName || !token) return;
    loadError = "";
    try {
      collaborators = await invoke("github_list_collaborators", {
        owner: repoOwner, repo: repoName, token,
      });
    } catch (e: any) {
      loadError = String(e);
    }
  }

  async function invite() {
    if (!inviteUser.trim() || !repoOwner || !repoName || !token) return;
    busy = true;
    try {
      await invoke("github_invite_collaborator", {
        owner: repoOwner, repo: repoName, username: inviteUser.trim(), token,
      });
      statusMsg = `Invitation sent to ${inviteUser.trim()}`;
      inviteUser = "";
      await loadCollaborators();
    } catch (e: any) {
      statusMsg = `Invite failed: ${e}`;
    } finally {
      busy = false;
    }
  }

  async function remove(username: string) {
    if (!repoOwner || !repoName || !token) return;
    try {
      await invoke("github_remove_collaborator", {
        owner: repoOwner, repo: repoName, username, token,
      });
      collaborators = collaborators.filter((c) => c.login !== username);
      statusMsg = `Removed ${username}`;
    } catch (e: any) {
      statusMsg = `Remove failed: ${e}`;
    }
  }

  // ── GitHub OAuth Device Flow ───────────────────────────────

  function saveClientId() {
    localStorage.setItem("parallax:github_client_id", clientId);
  }

  async function startLogin() {
    saveClientId();
    try {
      deviceInfo = await startGitHubLogin(clientId);
      beginPolling();
    } catch (e: any) {
      statusMsg = String(e);
    }
  }

  function beginPolling() {
    if (!deviceInfo) return;
    polling = true;
    const interval = (deviceInfo.interval + 1) * 1000;
    pollTimer = setInterval(async () => {
      if (!deviceInfo) return stopPolling();
      try {
        const id = await pollGitHubLogin(deviceInfo.device_code, clientId);
        if (id) {
          stopPolling();
          deviceInfo = null;
          statusMsg = `Signed in as @${id.login}`;
        }
      } catch (e: any) {
        stopPolling();
        statusMsg = String(e);
      }
    }, interval);
  }

  function stopPolling() {
    polling = false;
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  async function signOut() {
    await invoke("github_sign_out");
    githubIdentity.value = null;
  }

  function openVerificationUrl() {
    if (deviceInfo) {
      window.open(deviceInfo.verification_uri, "_blank");
    }
  }

  onMount(() => {
    // Try to auto-detect repo from workspace git remote (best-effort)
    return () => stopPolling();
  });
</script>

<div class="dashboard-section animate-fade-in">
  <div class="section-header">
    <h2>Team</h2>
    <p class="section-desc">Manage collaborators via GitHub. Your team = your repo's collaborators.</p>
  </div>

  <!-- GitHub Account -->
  <div class="card">
    <div class="card-title">GitHub Account</div>
    {#if isLoggedIn}
      <div class="identity-row">
        <img src={githubIdentity.value!.avatar_url} alt="avatar" class="gh-avatar" />
        <div class="identity-info">
          <div class="identity-login">@{githubIdentity.value!.login}</div>
          {#if githubIdentity.value!.name}
            <div class="identity-name">{githubIdentity.value!.name}</div>
          {/if}
        </div>
        <button class="btn-danger" onclick={signOut}>Sign Out</button>
      </div>
    {:else if deviceInfo}
      <div class="device-flow">
        <p class="device-desc">Open the URL below and enter the code to authorize Parallax:</p>
        <div class="device-code-box">{deviceInfo.user_code}</div>
        <button class="btn-primary" onclick={openVerificationUrl}>
          Open {deviceInfo.verification_uri}
        </button>
        {#if polling}
          <p class="polling-hint">Waiting for authorization…</p>
        {/if}
        <button class="btn-text" onclick={() => { deviceInfo = null; stopPolling(); }}>Cancel</button>
      </div>
    {:else}
      <div class="login-section">
        <div class="client-id-row">
          <input
            class="form-input flex-1"
            placeholder="GitHub OAuth App Client ID (PARALLAX_GITHUB_CLIENT_ID)"
            bind:value={clientId}
            onchange={saveClientId}
          />
        </div>
        <button class="btn-primary" onclick={startLogin} disabled={!clientId.trim()}>
          Sign in with GitHub
        </button>
        <p class="hint">
          Create a GitHub OAuth App at <code>github.com/settings/developers</code>.
          Enable Device Flow. Copy the Client ID here.
        </p>
      </div>
    {/if}
  </div>

  <!-- Repo settings -->
  {#if isLoggedIn}
    <div class="card">
      <div class="card-title">Repository</div>
      <div class="repo-row">
        <span class="repo-label">github.com /</span>
        <input class="form-input" style="width:130px" bind:value={repoOwner} placeholder="owner" />
        <span class="repo-slash">/</span>
        <input class="form-input" style="width:180px" bind:value={repoName} placeholder="repo-name" />
        <button class="btn-action" onclick={loadCollaborators} disabled={!repoOwner || !repoName}>
          Load Team
        </button>
      </div>
    </div>

    {#if loadError}
      <div class="error-msg">{loadError}</div>
    {/if}

    <!-- Collaborators -->
    {#if collaborators.length > 0}
      <div class="card">
        <div class="card-title">Collaborators</div>
        <div class="collab-list">
          {#each collaborators as c (c.login)}
            <div class="collab-row">
              <div class="collab-avatar-wrap">
                <img src={c.avatar_url} alt={c.login} class="gh-avatar sm" />
                {#if isOnline(c.login)}
                  <span class="online-dot"></span>
                {/if}
              </div>
              <div class="collab-info">
                <span class="collab-login">@{c.login}</span>
                <span class="collab-role">{c.role}</span>
              </div>
              {#if c.login !== githubIdentity.value?.login}
                <button class="btn-remove" onclick={() => remove(c.login)}>×</button>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Invite -->
    <div class="invite-row">
      <input
        class="form-input flex-1"
        placeholder="GitHub username to invite"
        bind:value={inviteUser}
        onkeydown={(e) => e.key === "Enter" && invite()}
      />
      <button class="btn-primary" onclick={invite} disabled={busy || !inviteUser.trim() || !repoOwner || !repoName}>
        {busy ? "Sending…" : "Invite"}
      </button>
    </div>
  {/if}

  {#if statusMsg}
    <div class="status-msg">{statusMsg}</div>
  {/if}
</div>

<style>
  .dashboard-section { max-width: 700px; display: flex; flex-direction: column; gap: 16px; }
  .section-header { margin-bottom: 4px; }
  .section-header h2 { font-size: 18px; font-weight: 700; margin-bottom: 4px; }
  .section-desc { font-size: 12px; color: var(--text-secondary); }

  .card {
    background: var(--bg-surface); border: 1px solid var(--border-default);
    border-radius: var(--radius-lg); padding: 16px;
  }
  .card-title { font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: .06em; margin-bottom: 12px; }

  /* Identity */
  .identity-row { display: flex; align-items: center; gap: 12px; }
  .gh-avatar { width: 36px; height: 36px; border-radius: 50%; flex-shrink: 0; }
  .gh-avatar.sm { width: 28px; height: 28px; }
  .identity-info { flex: 1; }
  .identity-login { font-size: 13px; font-weight: 600; color: var(--text-primary); }
  .identity-name { font-size: 11px; color: var(--text-muted); }
  .btn-danger {
    height: 28px; padding: 0 12px; background: transparent;
    border: 1px solid var(--color-error); color: var(--color-error);
    font-size: 11px; font-weight: 600; border-radius: var(--radius-sm); cursor: pointer;
    transition: var(--transition-fast);
  }
  .btn-danger:hover { background: var(--color-error-dim); }

  /* Device flow */
  .device-flow { display: flex; flex-direction: column; align-items: flex-start; gap: 12px; }
  .device-desc { font-size: 12px; color: var(--text-secondary); margin: 0; }
  .device-code-box {
    font-size: 28px; font-weight: 800; letter-spacing: .2em;
    font-family: var(--font-mono); color: var(--accent-primary);
    background: var(--accent-primary-dim); padding: 10px 24px;
    border-radius: var(--radius-md); border: 1px solid var(--accent-primary);
    align-self: flex-start;
  }
  .polling-hint { font-size: 11px; color: var(--text-muted); margin: 0; }
  .btn-text { font-size: 11px; color: var(--text-muted); background: none; border: none; cursor: pointer; }
  .btn-text:hover { color: var(--text-primary); }

  /* Login section */
  .login-section { display: flex; flex-direction: column; gap: 10px; }
  .client-id-row { display: flex; gap: 8px; }
  .hint { font-size: 10px; color: var(--text-muted); margin: 0; line-height: 1.5; }
  .hint code { color: var(--accent-primary); }

  /* Form */
  .form-input {
    height: 30px; padding: 0 10px; background: var(--bg-input);
    border: 1px solid var(--border-default); border-radius: var(--radius-md);
    color: var(--text-primary); font-size: 12px;
  }
  .form-input:focus { border-color: var(--accent-primary); outline: none; }
  .flex-1 { flex: 1; }

  .btn-primary {
    height: 30px; padding: 0 16px; background: var(--accent-primary); color: white;
    border: none; border-radius: var(--radius-md); font-weight: 600; font-size: 12px;
    cursor: pointer; transition: var(--transition-fast); white-space: nowrap;
  }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn-action {
    height: 30px; padding: 0 14px; background: var(--bg-elevated);
    border: 1px solid var(--border-default); color: var(--text-secondary);
    font-size: 12px; font-weight: 600; border-radius: var(--radius-md);
    cursor: pointer; transition: var(--transition-fast); white-space: nowrap;
  }
  .btn-action:hover:not(:disabled) { border-color: var(--accent-primary); color: var(--accent-primary); }
  .btn-action:disabled { opacity: 0.5; cursor: not-allowed; }

  .repo-row { display: flex; align-items: center; gap: 8px; }
  .repo-label { font-size: 12px; color: var(--text-muted); white-space: nowrap; }
  .repo-slash { font-size: 16px; color: var(--text-muted); }

  /* Collaborators */
  .collab-list { display: flex; flex-direction: column; gap: 8px; }
  .collab-row { display: flex; align-items: center; gap: 10px; }
  .collab-avatar-wrap { position: relative; flex-shrink: 0; }
  .online-dot {
    position: absolute; bottom: 0; right: 0;
    width: 8px; height: 8px; border-radius: 50%;
    background: var(--color-success); border: 1.5px solid var(--bg-surface);
  }
  .collab-info { flex: 1; display: flex; align-items: center; gap: 8px; }
  .collab-login { font-size: 13px; font-weight: 600; color: var(--text-primary); }
  .collab-role { font-size: 10px; color: var(--text-muted); background: var(--bg-elevated); padding: 1px 6px; border-radius: 10px; }
  .btn-remove {
    background: transparent; border: none; color: var(--text-muted); font-size: 18px;
    cursor: pointer; line-height: 1; padding: 0 4px;
  }
  .btn-remove:hover { color: var(--color-error); }

  .invite-row { display: flex; gap: 8px; align-items: center; }
  .error-msg { font-size: 11px; color: var(--color-error); }
  .status-msg { font-size: 11px; color: var(--accent-secondary); font-family: var(--font-mono); }
</style>
