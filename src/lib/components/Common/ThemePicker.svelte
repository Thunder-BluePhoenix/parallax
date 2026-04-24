<script lang="ts">
  import { THEMES, activeTheme, applyTheme, type ThemeId } from "../../stores/theme.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let customCss = $state(activeTheme.customCss);
  let showCustom = $state(activeTheme.id === "custom");

  function pick(id: ThemeId) {
    showCustom = id === "custom";
    applyTheme(id, customCss);
  }

  function applyCustom() {
    applyTheme("custom", customCss);
  }
</script>

<div class="theme-panel animate-fade-in">
  <div class="theme-header">
    <span class="theme-title">Theme</span>
    <button class="close-x" onclick={onclose}>×</button>
  </div>

  <div class="theme-grid">
    {#each THEMES as t}
      <button
        class="theme-swatch"
        class:active={activeTheme.id === t.id}
        onclick={() => pick(t.id)}
        style="background: {t.vars['--bg-base']}; border-color: {activeTheme.id === t.id ? t.vars['--accent-primary'] : t.vars['--border-default']}"
      >
        <div class="swatch-bar" style="background: {t.vars['--accent-primary']}"></div>
        <div class="swatch-bar secondary" style="background: {t.vars['--accent-secondary']}"></div>
        <span class="swatch-label" style="color: {t.vars['--text-primary']}">{t.label}</span>
      </button>
    {/each}
    <button
      class="theme-swatch custom-swatch"
      class:active={activeTheme.id === "custom"}
      onclick={() => pick("custom")}
    >
      <div class="swatch-bar" style="background: linear-gradient(90deg,#ff6b6b,#ffd93d,#6bcb77,#4d96ff)"></div>
      <span class="swatch-label">Custom CSS</span>
    </button>
  </div>

  {#if showCustom || activeTheme.id === "custom"}
    <div class="custom-css-section">
      <div class="custom-label">CSS Variables &amp; Overrides</div>
      <textarea
        class="custom-css-input mono"
        bind:value={customCss}
        placeholder=":root &#123; --accent-primary: #ff6b6b; &#125;"
        spellcheck="false"
        rows="6"
      ></textarea>
      <button class="btn-apply-css" onclick={applyCustom}>Apply CSS</button>
    </div>
  {/if}
</div>

<style>
  .theme-panel {
    position: absolute; bottom: 44px; left: 8px; right: 8px;
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-lg); box-shadow: var(--shadow-lg);
    z-index: 100; overflow: hidden;
  }
  .theme-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 12px; border-bottom: 1px solid var(--border-subtle);
  }
  .theme-title {
    font-size: 11px; font-weight: 700; text-transform: uppercase;
    letter-spacing: 0.07em; color: var(--text-secondary);
  }
  .close-x {
    background: transparent; color: var(--text-muted);
    font-size: 16px; width: 20px; height: 20px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 3px;
  }
  .close-x:hover { background: var(--bg-overlay); color: var(--text-primary); }

  .theme-grid {
    display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px;
    padding: 10px 12px;
  }
  .theme-swatch {
    display: flex; flex-direction: column; align-items: flex-start;
    padding: 8px; border-radius: var(--radius-md); border: 1px solid transparent;
    cursor: pointer; transition: var(--transition-fast); gap: 4px;
  }
  .theme-swatch.active { box-shadow: 0 0 0 2px var(--accent-primary); }
  .theme-swatch:hover { opacity: 0.9; }
  .custom-swatch { background: var(--bg-overlay) !important; border-color: var(--border-default) !important; }
  .custom-swatch.active { box-shadow: 0 0 0 2px var(--accent-secondary); }
  .swatch-bar { width: 100%; height: 3px; border-radius: 2px; }
  .swatch-bar.secondary { opacity: 0.7; }
  .swatch-label { font-size: 9px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; margin-top: 2px; }

  .custom-css-section {
    padding: 8px 12px 12px; border-top: 1px solid var(--border-subtle);
    display: flex; flex-direction: column; gap: 6px;
  }
  .custom-label { font-size: 10px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; }
  .custom-css-input {
    resize: vertical; font-size: 11px; padding: 6px 8px;
    background: var(--bg-base); border: 1px solid var(--border-default);
    border-radius: var(--radius-sm); color: var(--text-primary); min-height: 80px;
  }
  .custom-css-input:focus { border-color: var(--accent-primary); outline: none; }
  .btn-apply-css {
    align-self: flex-end; padding: 4px 14px; font-size: 11px; font-weight: 600;
    background: var(--accent-primary); color: white; border-radius: var(--radius-sm);
  }
</style>
