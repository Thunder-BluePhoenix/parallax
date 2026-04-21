<script lang="ts">
  import { responseState } from "../../stores/app.svelte";

  let viewMode = $state<"pretty" | "raw" | "headers">("pretty");

  function statusClass(code: number) {
    if (code >= 500) return "status-5xx";
    if (code >= 400) return "status-4xx";
    if (code >= 300) return "status-3xx";
    return "status-2xx";
  }

  function formatSize(bytes: number) {
    if (bytes < 1024) return `${bytes} B`;
    return `${(bytes / 1024).toFixed(1)} KB`;
  }

  function prettyJson(val: any) {
    return JSON.stringify(val, null, 2);
  }

  let jsonLines = $derived.by(() => {
    if (!responseState.response?.body?.json) return [];
    return prettyJson(responseState.response.body.json).split("\n");
  });
</script>

<div class="response-panel pane">
  {#if responseState.loading}
    <div class="response-loading">
      <div class="loading-spinner"></div>
      <span>Sending request…</span>
    </div>

  {:else if responseState.error}
    <div class="response-error animate-fade-in">
      <div class="error-icon">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
      </div>
      <p class="error-title">Request Failed</p>
      <p class="error-msg">{responseState.error}</p>
    </div>

  {:else if responseState.response}
    {@const res = responseState.response}
    <!-- Status Bar -->
    <div class="response-statusbar animate-fade-in">
      <div class="status-info">
        <span class="status-code {statusClass(res.status)}">{res.status}</span>
        <span class="status-text">{res.statusText}</span>
        <span class="status-divider">·</span>
        <span class="timing">{res.timing.totalMs}ms</span>
        <span class="status-divider">·</span>
        <span class="size">{formatSize(res.sizeBytes)}</span>
      </div>

      <div class="view-modes">
        {#each ["pretty", "raw", "headers"] as mode}
          <button
            class="view-mode-btn"
            class:active={viewMode === mode}
            onclick={() => (viewMode = mode as any)}
          >
            {mode.charAt(0).toUpperCase() + mode.slice(1)}
          </button>
        {/each}
      </div>
    </div>

    <!-- Response body -->
    <div class="response-body scroll-y">
      {#if viewMode === "pretty"}
        {#if res.body.json}
          <div class="json-viewer mono">
            {#each jsonLines as line, i}
              <div class="json-line">
                <span class="line-num">{i + 1}</span>
                <span class="line-content">{@html colorizeJsonLine(line)}</span>
              </div>
            {/each}
          </div>
        {:else}
          <pre class="raw-body">{res.body.raw}</pre>
        {/if}

      {:else if viewMode === "raw"}
        <pre class="raw-body mono">{res.body.raw}</pre>

      {:else if viewMode === "headers"}
        <div class="headers-viewer">
          {#each Object.entries(res.headers) as [key, val]}
            <div class="header-row">
              <span class="header-key mono">{key}</span>
              <span class="header-val mono">{val}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>

  {:else}
    <div class="response-empty">
      <div class="empty-icon">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" opacity="0.3">
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
        </svg>
      </div>
      <p>Hit <kbd>Send</kbd> or press <kbd>⌘ Enter</kbd></p>
    </div>
  {/if}
</div>

<script lang="ts" module>
  function colorizeJsonLine(line: string): string {
    return line
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"([^"]+)":/g, '<span class="json-key">"$1"</span>:')
      .replace(/: "(.*?)"/g, ': <span class="json-str">"$1"</span>')
      .replace(/: (\d+\.?\d*)/g, ': <span class="json-num">$1</span>')
      .replace(/: (true|false)/g, ': <span class="json-bool">$1</span>')
      .replace(/: (null)/g, ': <span class="json-null">$1</span>');
  }
</script>

<style>
  .response-panel {
    flex: 1;
    min-width: 0;
    background: var(--bg-base);
  }

  /* Loading */
  .response-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .loading-spinner {
    width: 28px;
    height: 28px;
    border: 2px solid var(--border-default);
    border-top-color: var(--accent-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  /* Error */
  .response-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
    padding: 24px;
    text-align: center;
  }

  .error-icon { color: var(--color-error); opacity: 0.7; margin-bottom: 4px; }
  .error-title { font-weight: 600; font-size: 14px; color: var(--color-error); }
  .error-msg { font-size: 12px; color: var(--text-secondary); font-family: var(--font-mono); max-width: 400px; word-break: break-all; }

  /* Status bar */
  .response-statusbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .status-info {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }

  .status-code {
    font-family: var(--font-mono);
    font-weight: 700;
    font-size: 14px;
  }
  .status-text { color: var(--text-secondary); }
  .status-divider { color: var(--text-muted); }
  .timing { color: var(--text-secondary); font-family: var(--font-mono); }
  .size { color: var(--text-muted); font-family: var(--font-mono); font-size: 11px; }

  .view-modes {
    display: flex;
    gap: 1px;
    background: var(--bg-void);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: 2px;
  }

  .view-mode-btn {
    padding: 2px 10px;
    font-size: 11px;
    color: var(--text-muted);
    border-radius: 3px;
    background: transparent;
    transition: var(--transition-fast);
  }
  .view-mode-btn:hover { color: var(--text-primary); background: var(--bg-elevated); }
  .view-mode-btn.active { color: var(--accent-primary); background: var(--accent-primary-dim); }

  /* Body */
  .response-body { flex: 1; padding: 0; }

  .json-viewer {
    padding: 12px 0;
    font-size: 12px;
    line-height: 1.7;
  }

  .json-line {
    display: flex;
    gap: 0;
    padding: 0 12px;
    min-height: 20px;
  }

  .json-line:hover { background: var(--bg-elevated); }

  .line-num {
    min-width: 36px;
    color: var(--text-muted);
    font-size: 11px;
    text-align: right;
    padding-right: 16px;
    user-select: none;
    flex-shrink: 0;
  }

  .line-content { flex: 1; white-space: pre; }

  :global(.json-key) { color: var(--color-info); }
  :global(.json-str) { color: var(--color-success); }
  :global(.json-num) { color: var(--color-warning); }
  :global(.json-bool) { color: var(--accent-secondary); }
  :global(.json-null) { color: var(--text-muted); }

  .raw-body {
    padding: 12px;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.7;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--text-secondary);
  }

  /* Headers */
  .headers-viewer { padding: 8px 12px; }

  .header-row {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: 12px;
    padding: 6px 0;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 12px;
  }

  .header-key { color: var(--color-info); font-size: 11px; }
  .header-val { color: var(--text-secondary); word-break: break-all; }

  /* Empty */
  .response-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
    color: var(--text-muted);
    font-size: 13px;
  }

  .empty-icon { margin-bottom: 4px; }

  kbd {
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: 4px;
    padding: 1px 6px;
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }
</style>
