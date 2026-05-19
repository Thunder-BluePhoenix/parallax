<script lang="ts">
  import { activeEnvironment, currentWorkspace, activeRequest } from "../../stores/app.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let { onclose }: { onclose: () => void } = $props();

  let activeTab = $state<"edit" | "diff">("edit");

  let envName = $state(activeEnvironment.name);
  let pairs = $state(
    Object.entries(activeEnvironment.variables).map(([k, v]) => ({ k, v, secret: false }))
  );

  function addRow() { pairs.push({ k: "", v: "", secret: false }); }
  function removeRow(i: number) { pairs.splice(i, 1); }

  // ── Smart variable extraction from active request ────────────
  function suggestFromRequest() {
    const req = activeRequest;
    if (!req) return;
    const suggestions: { k: string; v: string }[] = [];
    const existing = new Set(pairs.map(p => p.k));

    // Extract base URL
    try {
      const u = new URL(req.url);
      const baseKey = "base_url";
      if (!existing.has(baseKey)) {
        suggestions.push({ k: baseKey, v: `${u.protocol}//${u.host}` });
        existing.add(baseKey);
      }
    } catch {}

    // Extract auth header tokens
    const authHeader = req.headers?.["Authorization"] ?? req.headers?.["authorization"] ?? "";
    if (authHeader.startsWith("Bearer ")) {
      const key = "auth_token";
      if (!existing.has(key)) suggestions.push({ k: key, v: authHeader.slice(7) });
    } else if (authHeader.startsWith("Basic ")) {
      const key = "basic_credentials";
      if (!existing.has(key)) suggestions.push({ k: key, v: authHeader.slice(6) });
    }

    // Extract API key headers
    for (const [k, v] of Object.entries(req.headers ?? {})) {
      if (/api[-_]?key|x-api-key/i.test(k)) {
        const varName = k.toLowerCase().replace(/[-\s]/g, "_");
        if (!existing.has(varName)) {
          suggestions.push({ k: varName, v: String(v) });
          existing.add(varName);
        }
      }
    }

    for (const s of suggestions) {
      pairs.push({ ...s, secret: /token|key|secret|password/i.test(s.k) });
    }
  }

  function apply() {
    const vars: Record<string, string> = {};
    for (const { k, v } of pairs) { if (k.trim()) vars[k.trim()] = v; }
    activeEnvironment.name      = envName;
    activeEnvironment.variables = vars;

    if (currentWorkspace.path) {
      invoke("save_environment", {
        workspace: currentWorkspace.path,
        env: { name: envName, variables: vars },
      }).catch(() => {});
    }
    onclose();
  }

  // ── Diff view ───────────────────────────────────────────────
  let savedEnvs = $state<{ name: string; variables: Record<string, string> }[]>([]);
  let compareEnvName = $state("");

  async function loadEnvsForDiff() {
    if (!currentWorkspace.path) return;
    try {
      const names: string[] = await invoke("list_environments", { workspace: currentWorkspace.path });
      const envs = await Promise.all(
        names.map(n => invoke<{ name: string; variables: Record<string, string> }>(
          "load_environment", { workspace: currentWorkspace.path, name: n }
        ).catch(() => null))
      );
      savedEnvs = envs.filter(Boolean) as typeof savedEnvs;
      if (savedEnvs.length > 0 && !compareEnvName) compareEnvName = savedEnvs[0].name;
    } catch { savedEnvs = []; }
  }

  $effect(() => { if (activeTab === "diff") loadEnvsForDiff(); });

  // ── Shell tag security ──────────────────────────────────────
  const hasShellTags = $derived(pairs.some(p => p.v.includes("{% shell")));
  let shellWarningDismissed = $state(false);

  const compareEnv = $derived(savedEnvs.find(e => e.name === compareEnvName));

  interface DiffRow {
    key: string;
    left: string | undefined;   // active env
    right: string | undefined;  // compare env
    status: "same" | "changed" | "added" | "removed";
  }

  const diffRows = $derived.by((): DiffRow[] => {
    const leftVars = activeEnvironment.variables;
    const rightVars = compareEnv?.variables ?? {};
    const allKeys = new Set([...Object.keys(leftVars), ...Object.keys(rightVars)]);
    return [...allKeys].sort().map(key => {
      const l = leftVars[key];
      const r = rightVars[key];
      const status =
        l === undefined ? "added" :
        r === undefined ? "removed" :
        l !== r         ? "changed" : "same";
      return { key, left: l, right: r, status };
    });
  });
</script>

<div class="env-panel animate-fade-in">
  <div class="env-header">
    <span class="env-title">Environment</span>
    <div class="tab-pills">
      <button class="tab-pill" class:active={activeTab === "edit"} onclick={() => activeTab = "edit"}>Edit</button>
      <button class="tab-pill" class:active={activeTab === "diff"} onclick={() => activeTab = "diff"}>Diff</button>
    </div>
    <button class="close-x" onclick={onclose}>×</button>
  </div>

  {#if activeTab === "edit"}
    <div class="env-name-row">
      <label for="env-name" class="field-label">Name</label>
      <input id="env-name" type="text" bind:value={envName} placeholder="dev" />
    </div>

    {#if hasShellTags && !shellWarningDismissed}
      <div class="shell-warning">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
          <line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
        </svg>
        <span>This environment runs shell commands at request send-time. Only activate environments you trust.</span>
        <button class="shell-warn-dismiss" onclick={() => (shellWarningDismissed = true)}>Got it</button>
      </div>
    {/if}

    <div class="pairs-header">
      <span>Key</span>
      <span>Value</span>
    </div>

    <div class="pairs-scroll scroll-y">
      {#each pairs as row, i}
        <div class="pair-row">
          <input class="mono" type="text" placeholder="variable" bind:value={row.k} />
          <input class="mono" type={row.secret ? "password" : "text"} placeholder="value" bind:value={row.v} />
          <button class="secret-toggle" title="Toggle secret" onclick={() => (row.secret = !row.secret)}>
            {#if row.secret}
              <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94"/>
                <path d="M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19"/>
                <line x1="1" y1="1" x2="23" y2="23"/>
              </svg>
            {:else}
              <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
            {/if}
          </button>
          <button class="remove-btn" onclick={() => removeRow(i)}>×</button>
        </div>
      {/each}
    </div>

    <div class="add-row">
      <button class="add-pair-btn" onclick={addRow}>+ Add variable</button>
      <button class="add-pair-btn suggest-btn" onclick={suggestFromRequest} title="Extract variables from active request">⚡ Suggest from request</button>
    </div>

    <div class="env-actions">
      <button class="btn-cancel" onclick={onclose}>Cancel</button>
      <button class="btn-apply" onclick={apply}>Apply</button>
    </div>

  {:else}
    <!-- Diff view -->
    <div class="diff-bar">
      <span class="field-label">Compare with:</span>
      <select class="diff-select mono" bind:value={compareEnvName}>
        {#each savedEnvs as env}
          <option value={env.name}>{env.name}</option>
        {/each}
        {#if savedEnvs.length === 0}
          <option value="">No saved environments</option>
        {/if}
      </select>
    </div>

    <div class="diff-legend">
      <span class="diff-badge same">= same</span>
      <span class="diff-badge changed">~ changed</span>
      <span class="diff-badge removed">− only here</span>
      <span class="diff-badge added">+ only there</span>
    </div>

    <div class="diff-scroll scroll-y">
      {#if diffRows.length === 0}
        <div class="diff-empty">No variables to compare.</div>
      {:else}
        {#each diffRows as row}
          {#if row.status !== "same"}
            <div class="diff-row diff-{row.status}">
              <span class="diff-key mono">{row.key}</span>
              <span class="diff-val mono">{row.left ?? "—"}</span>
              <span class="diff-arrow">→</span>
              <span class="diff-val mono">{row.right ?? "—"}</span>
            </div>
          {/if}
        {/each}
        {#if diffRows.every(r => r.status === "same")}
          <div class="diff-empty">Environments are identical.</div>
        {/if}
      {/if}
    </div>

    <div class="env-actions">
      <button class="btn-cancel" onclick={onclose}>Close</button>
    </div>
  {/if}
</div>

<style>
  .env-panel {
    position: absolute;
    bottom: 44px;
    left: 8px;
    right: 8px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
    z-index: 100;
    overflow: hidden;
    max-height: 380px;
  }

  .env-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .tab-pills { display: flex; gap: 2px; flex: 1; }
  .tab-pill {
    padding: 2px 10px; font-size: 10px; font-weight: 700;
    border-radius: 10px; background: transparent; color: var(--text-muted);
    transition: var(--transition-fast);
  }
  .tab-pill:hover { color: var(--text-primary); }
  .tab-pill.active { background: var(--accent-primary-dim); color: var(--accent-primary); }

  /* Diff view */
  .diff-bar {
    display: flex; align-items: center; gap: 8px;
    padding: 8px 12px; border-bottom: 1px solid var(--border-subtle); flex-shrink: 0;
  }
  .diff-select {
    flex: 1; height: 24px; padding: 0 6px; font-size: 11px;
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-sm); color: var(--text-primary);
  }
  .diff-legend {
    display: flex; gap: 6px; padding: 5px 12px; flex-shrink: 0; flex-wrap: wrap;
  }
  .diff-badge {
    font-size: 9px; font-weight: 700; padding: 1px 6px;
    border-radius: 8px; font-family: var(--font-mono);
  }
  .diff-badge.same    { background: rgba(139,148,158,0.15); color: var(--text-muted); }
  .diff-badge.changed { background: rgba(227,179,65,0.15);  color: var(--color-warning); }
  .diff-badge.removed { background: rgba(248,81,73,0.15);   color: var(--color-error); }
  .diff-badge.added   { background: rgba(63,185,80,0.15);   color: var(--color-success); }

  .diff-scroll { flex: 1; padding: 4px 8px; overflow-y: auto; min-height: 60px; }
  .diff-row {
    display: grid; grid-template-columns: 1fr 1fr 16px 1fr;
    gap: 4px; align-items: center;
    padding: 4px 6px; border-radius: var(--radius-sm);
    margin-bottom: 2px; font-size: 10px;
  }
  .diff-row.diff-changed { background: rgba(227,179,65,0.08); border-left: 2px solid var(--color-warning); }
  .diff-row.diff-removed { background: rgba(248,81,73,0.08);  border-left: 2px solid var(--color-error); }
  .diff-row.diff-added   { background: rgba(63,185,80,0.08);  border-left: 2px solid var(--color-success); }
  .diff-key   { color: var(--text-secondary); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .diff-val   { color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .diff-arrow { color: var(--text-muted); text-align: center; }
  .diff-empty { padding: 16px; text-align: center; font-size: 11px; color: var(--text-muted); }

  .env-title {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--text-secondary);
  }

  .close-x {
    background: transparent;
    color: var(--text-muted);
    font-size: 16px;
    line-height: 1;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
    transition: var(--transition-fast);
  }
  .close-x:hover { background: var(--bg-overlay); color: var(--text-primary); }

  .env-name-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .field-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .env-name-row input {
    flex: 1;
    height: 26px;
    padding: 0 8px;
    font-size: 12px;
    border-radius: var(--radius-sm);
  }

  .pairs-header {
    display: grid;
    grid-template-columns: 1fr 1fr 20px 20px;
    gap: 4px;
    padding: 4px 12px;
    font-size: 10px;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    flex-shrink: 0;
  }

  .pairs-scroll {
    flex: 1;
    padding: 2px 8px;
    min-height: 60px;
  }

  .pair-row {
    display: grid;
    grid-template-columns: 1fr 1fr 20px 20px;
    gap: 4px;
    margin-bottom: 3px;
    align-items: center;
  }

  .pair-row input {
    height: 26px;
    padding: 0 6px;
    font-size: 11px;
    border-radius: var(--radius-sm);
  }

  .secret-toggle, .remove-btn {
    width: 20px;
    height: 20px;
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
    transition: var(--transition-fast);
  }

  .secret-toggle { color: var(--text-muted); }
  .secret-toggle:hover { background: var(--bg-overlay); color: var(--accent-secondary); }

  .remove-btn { color: var(--text-muted); font-size: 13px; }
  .remove-btn:hover { background: var(--color-error-dim); color: var(--color-error); }

  .add-pair-btn {
    background: transparent;
    color: var(--accent-primary);
    font-size: 11px;
    padding: 6px 12px;
    text-align: left;
    flex-shrink: 0;
    transition: var(--transition-fast);
  }
  .add-pair-btn:hover { color: #9185ff; }

  .add-row { display: flex; gap: 4px; flex-shrink: 0; }
  .suggest-btn { color: var(--accent-secondary); font-size: 10px; }

  .env-actions {
    display: flex;
    gap: 6px;
    padding: 8px 12px;
    border-top: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .btn-cancel, .btn-apply {
    flex: 1;
    height: 28px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 500;
    transition: var(--transition-fast);
  }

  .btn-cancel {
    background: transparent;
    color: var(--text-secondary);
    border: 1px solid var(--border-default);
  }
  .btn-cancel:hover { background: var(--bg-overlay); }

  .btn-apply {
    background: var(--accent-primary);
    color: white;
  }
  .btn-apply:hover { background: #9185ff; }

  .shell-warning {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    background: rgba(224, 108, 117, 0.12);
    border: 1px solid rgba(224, 108, 117, 0.4);
    border-radius: var(--radius-md);
    padding: 10px 12px;
    margin-bottom: 10px;
    font-size: 11px;
    color: #e06c75;
    line-height: 1.5;
  }
  .shell-warning svg { flex-shrink: 0; margin-top: 1px; }
  .shell-warning span { flex: 1; }
  .shell-warn-dismiss {
    flex-shrink: 0;
    background: transparent;
    border: 1px solid rgba(224, 108, 117, 0.5);
    color: #e06c75;
    border-radius: var(--radius-sm);
    font-size: 10px;
    padding: 2px 8px;
    cursor: pointer;
    white-space: nowrap;
  }
  .shell-warn-dismiss:hover { background: rgba(224, 108, 117, 0.15); }
</style>
