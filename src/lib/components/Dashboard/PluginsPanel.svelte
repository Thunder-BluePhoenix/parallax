<script lang="ts">
  import { pluginRegistry } from "../../utils/plugin-api.svelte";

  let customId  = $state("");
  let customSrc = $state("");
  let installError = $state("");

  function togglePlugin(id: string) {
    pluginRegistry.toggle(id);
  }

  function installCustom() {
    installError = "";
    if (!customId.trim() || !customSrc.trim()) {
      installError = "Plugin ID and source are required.";
      return;
    }
    try {
      pluginRegistry.installCustom(customId.trim(), customSrc.trim());
      customId = ""; customSrc = "";
    } catch (e: any) {
      installError = e?.message ?? "Install failed";
    }
  }
</script>

<div class="plugins-panel">
  <div class="panel-header">
    <h2>Plugins</h2>
    <p class="section-desc">Extend Parallax with built-in and custom JS plugins. Enabled plugins expose tools usable in pre-request and test scripts via <code>parallax.tools["name"]()</code>.</p>
  </div>

  <div class="plugins-list">
    {#each pluginRegistry.plugins as plugin}
      <div class="plugin-card" class:enabled={plugin.enabled}>
        <div class="plugin-info">
          <div class="plugin-name">
            {plugin.name}
            {#if plugin.builtin}<span class="builtin-badge">built-in</span>{/if}
          </div>
          <div class="plugin-desc">{plugin.description}</div>
          {#if plugin.tools.length > 0}
            <div class="plugin-tools">
              {#each plugin.tools as t}
                <span class="tool-tag mono">{t.name}</span>
              {/each}
            </div>
          {/if}
        </div>
        <button
          class="toggle-btn"
          class:on={plugin.enabled}
          onclick={() => togglePlugin(plugin.id)}
          title={plugin.enabled ? "Disable" : "Enable"}
        >
          {plugin.enabled ? "ON" : "OFF"}
        </button>
      </div>
    {/each}
  </div>

  <div class="custom-install">
    <div class="install-title">Install Custom Plugin</div>
    <input
      class="install-input mono"
      type="text"
      placeholder="plugin-id (e.g. my-utils)"
      bind:value={customId}
    />
    <textarea
      class="install-src mono"
      placeholder="(function(parallax) &#123; parallax.registerTool('myTool', () => 42); &#125;)"
      bind:value={customSrc}
      rows="5"
      spellcheck="false"
    ></textarea>
    {#if installError}
      <div class="install-error">{installError}</div>
    {/if}
    <button class="btn-install" onclick={installCustom}>Install Plugin</button>
  </div>
</div>

<style>
  .plugins-panel { display: flex; flex-direction: column; gap: 20px; padding: 24px; overflow-y: auto; height: 100%; }
  .panel-header h2 { font-size: 18px; font-weight: 700; color: var(--text-primary); margin-bottom: 6px; }
  .section-desc { font-size: 12px; color: var(--text-muted); line-height: 1.6; }
  .section-desc code { color: var(--accent-secondary); font-family: var(--font-mono); }

  .plugins-list { display: flex; flex-direction: column; gap: 8px; }

  .plugin-card {
    display: flex; align-items: flex-start; gap: 12px;
    padding: 12px 14px; border-radius: var(--radius-md);
    border: 1px solid var(--border-default); background: var(--bg-surface);
    transition: var(--transition-fast);
  }
  .plugin-card.enabled { border-color: rgba(124,110,255,0.3); background: var(--bg-elevated); }

  .plugin-info { flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .plugin-name { font-size: 13px; font-weight: 600; color: var(--text-primary); display: flex; align-items: center; gap: 6px; }
  .builtin-badge {
    font-size: 9px; font-weight: 700; padding: 1px 5px;
    background: var(--accent-primary-dim); color: var(--accent-primary);
    border-radius: 8px; text-transform: uppercase;
  }
  .plugin-desc { font-size: 11px; color: var(--text-muted); }
  .plugin-tools { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 4px; }
  .tool-tag {
    font-size: 9px; padding: 1px 6px; border-radius: 4px;
    background: var(--bg-overlay); color: var(--accent-secondary);
    border: 1px solid var(--border-subtle);
  }

  .toggle-btn {
    padding: 3px 10px; font-size: 10px; font-weight: 800; border-radius: 20px;
    border: 1px solid var(--border-default); color: var(--text-muted);
    background: var(--bg-base); flex-shrink: 0;
    transition: var(--transition-fast); letter-spacing: 0.05em;
  }
  .toggle-btn.on {
    background: var(--accent-primary-dim); color: var(--accent-primary);
    border-color: rgba(124,110,255,0.4);
  }
  .toggle-btn:hover { opacity: 0.8; }

  .custom-install {
    display: flex; flex-direction: column; gap: 8px;
    padding: 16px; background: var(--bg-surface);
    border: 1px solid var(--border-default); border-radius: var(--radius-md);
  }
  .install-title { font-size: 12px; font-weight: 700; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.05em; }
  .install-input, .install-src {
    padding: 7px 10px; font-size: 11px;
    background: var(--bg-base); border: 1px solid var(--border-default);
    border-radius: var(--radius-sm); color: var(--text-primary);
  }
  .install-input:focus, .install-src:focus { border-color: var(--accent-primary); outline: none; }
  .install-src { resize: vertical; min-height: 80px; }
  .install-error { font-size: 11px; color: var(--color-error); }
  .btn-install {
    align-self: flex-start; padding: 6px 16px; font-size: 12px; font-weight: 600;
    background: var(--accent-primary); color: white; border-radius: var(--radius-sm);
  }
  .btn-install:hover { opacity: 0.85; }
</style>
