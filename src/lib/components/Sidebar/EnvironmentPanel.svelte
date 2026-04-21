<script lang="ts">
  import { activeEnvironment, currentWorkspace } from "../../stores/app.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let { onclose }: { onclose: () => void } = $props();

  let envName = $state(activeEnvironment.name);
  let pairs = $state(
    Object.entries(activeEnvironment.variables).map(([k, v]) => ({ k, v, secret: false }))
  );

  function addRow() { pairs.push({ k: "", v: "", secret: false }); }
  function removeRow(i: number) { pairs.splice(i, 1); }

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
</script>

<div class="env-panel animate-fade-in">
  <div class="env-header">
    <span class="env-title">Environment</span>
    <button class="close-x" onclick={onclose}>×</button>
  </div>

  <div class="env-name-row">
    <label for="env-name" class="field-label">Name</label>
    <input id="env-name" type="text" bind:value={envName} placeholder="dev" />
  </div>

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

  <button class="add-pair-btn" onclick={addRow}>+ Add variable</button>

  <div class="env-actions">
    <button class="btn-cancel" onclick={onclose}>Cancel</button>
    <button class="btn-apply" onclick={apply}>Apply</button>
  </div>
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
    justify-content: space-between;
    padding: 10px 12px 8px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

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
</style>
