<script lang="ts">
  import { responseState, responseHistory, testResults, visualizerData } from "../../stores/app.svelte";
  import VisualizerIframe from "./VisualizerIframe.svelte";
  import { generateTests, repairRequest, aiStatus } from "../../stores/ai.svelte";
  import { currentRequestId } from "../../stores/app.svelte";

  let showRepairPanel = $state(false);
  let repairError     = $state("");

  const isFailure = $derived(
    !!responseState.response && responseState.response.status >= 400
  );

  let viewMode   = $state<"pretty" | "raw" | "headers" | "cookies" | "tests" | "history" | "visualize">("pretty");
  let historyIdx = $state(0);

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

  async function handleGenerateTests() {
    if (!responseState.response) return;
    try {
      const result = await generateTests(currentRequestId.value, responseState.response);
      console.log("AI Tests:", result);
      alert("AI tests generated! Check the Tests tab.");
    } catch (e: any) {
      console.error(e);
      alert("AI Error: " + e.toString());
    }
  }

  async function handleRepairRequest() {
    if (!responseState.response) return;
    repairError = "";
    showRepairPanel = false;
    try {
      await repairRequest(responseState.response);
      showRepairPanel = true;
    } catch (e: any) {
      repairError = e.toString();
      showRepairPanel = true;
    }
  }
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
        {#each (["pretty", "raw", "headers"] as const) as mode}
          <button
            class="view-mode-btn"
            class:active={viewMode === mode}
            onclick={() => (viewMode = mode)}
          >
            {mode.charAt(0).toUpperCase() + mode.slice(1)}
          </button>
        {/each}
        <button
          class="view-mode-btn"
          class:active={viewMode === "cookies"}
          onclick={() => (viewMode = "cookies")}
        >
          Cookies
          {#if (responseState.response?.cookies?.length ?? 0) > 0}
            <span class="test-badge">{responseState.response!.cookies.length}</span>
          {/if}
        </button>
        <button
          class="view-mode-btn"
          class:active={viewMode === "tests"}
          onclick={() => (viewMode = "tests")}
        >
          Tests
          {#if testResults.ran}
            {@const passed = testResults.results.filter(r => r.passed).length}
            {@const total  = testResults.results.length}
            <span class="test-badge" class:fail={passed < total}>{passed}/{total}</span>
          {/if}
        </button>
        <button
          class="view-mode-btn"
          class:active={viewMode === "history"}
          onclick={() => { viewMode = "history"; historyIdx = 0; }}
        >
          History
          {#if responseHistory.length > 0}
            <span class="test-badge">{responseHistory.length}</span>
          {/if}
        </button>
        {#if visualizerData.template}
          <button
            class="view-mode-btn visualizer-btn"
            class:active={viewMode === "visualize"}
            onclick={() => (viewMode = "visualize")}
          >
            Visualize
          </button>
        {/if}
      </div>
    </div>

    <!-- AI Action Bar: appears when response is loaded -->
    <div class="ai-action-bar">
      <button class="ai-btn" onclick={handleGenerateTests} disabled={aiStatus.busy}>
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/></svg>
        {aiStatus.busy ? 'Thinking…' : 'Generate Tests'}
      </button>
      {#if isFailure}
        <button class="ai-btn ai-repair-btn" onclick={handleRepairRequest} disabled={aiStatus.busy}>
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>
          {aiStatus.busy ? 'Analysing…' : 'Diagnose with AI'}
        </button>
      {/if}
    </div>

    <!-- AI Repair Result -->
    {#if showRepairPanel}
      <div class="ai-repair-panel animate-fade-in">
        {#if repairError}
          <div class="repair-error">
            <strong>AI Error:</strong> {repairError}
          </div>
        {:else if aiStatus.repairResult}
          {@const r = aiStatus.repairResult}
          <div class="repair-header">
            <span class="repair-priority priority-{r.priority}">{r.priority.toUpperCase()}</span>
            <span class="repair-diagnosis">{r.diagnosis}</span>
            <button class="repair-close" onclick={() => (showRepairPanel = false)}>×</button>
          </div>
          {#if r.fixes?.length > 0}
            <div class="repair-fixes">
              {#each r.fixes as fix}
                <div class="fix-item">
                  <span class="fix-type">{fix.type}</span>
                  <span class="fix-desc">{fix.description}</span>
                </div>
              {/each}
            </div>
          {/if}
        {/if}
      </div>
    {/if}

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

      {:else if viewMode === "cookies"}
        <div class="cookies-viewer">
          {#if !res.cookies?.length}
            <p class="tests-empty">No cookies returned by this response.</p>
          {:else}
            <table class="cookie-table">
              <thead>
                <tr>
                  <th>Name</th><th>Value</th><th>Domain</th><th>Path</th><th>Secure</th><th>HttpOnly</th>
                </tr>
              </thead>
              <tbody>
                {#each res.cookies as c}
                  <tr>
                    <td class="mono">{c.name}</td>
                    <td class="mono cookie-value">{c.value}</td>
                    <td class="mono">{c.domain ?? "—"}</td>
                    <td class="mono">{c.path ?? "/"}</td>
                    <td class="cookie-flag" class:flag-on={c.secure}>{c.secure ? "✓" : "—"}</td>
                    <td class="cookie-flag" class:flag-on={c.httpOnly}>{c.httpOnly ? "✓" : "—"}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </div>

      {:else if viewMode === "tests"}
        <div class="tests-viewer">
          <div class="tests-toolbar">
            <button class="btn-ai-tests" disabled={aiStatus.busy} onclick={handleGenerateTests}>
              {#if aiStatus.busy}
                <span class="spinner-small"></span> Generating...
              {:else}
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
                Generate Tests with AI
              {/if}
            </button>
            {#if aiStatus.lastError}
              <span class="ai-error">{aiStatus.lastError}</span>
            {/if}
          </div>

          {#if !testResults.ran}
            <p class="tests-empty">No tests ran — add scripts in the Tests tab and send the request.</p>
          {:else if testResults.results.length === 0}
            <p class="tests-empty">Script ran but contained no <code>pm.test()</code> calls.</p>
          {:else}
            {#each testResults.results as result}
              <div class="test-row" class:test-pass={result.passed} class:test-fail={!result.passed}>
                <span class="test-icon">{result.passed ? "✓" : "✗"}</span>
                <span class="test-name">{result.name}</span>
                {#if result.error}
                  <span class="test-error">{result.error}</span>
                {/if}
              </div>
            {/each}
          {/if}
        </div>

      {:else if viewMode === "history"}
        <div class="history-viewer">
          {#if responseHistory.length === 0}
            <p class="tests-empty">No history yet — send a request to record it here.</p>
          {:else}
            <div class="history-list">
              {#each responseHistory as entry, i}
                <button
                  class="history-entry"
                  class:active={historyIdx === i}
                  onclick={() => (historyIdx = i)}
                >
                  <span class="history-method method-{entry.method.toLowerCase()}">{entry.method}</span>
                  <span class="history-url">{entry.url}</span>
                  <span class="history-status" class:status-2xx={entry.status < 300} class:status-4xx={entry.status >= 400}>{entry.status}</span>
                  <span class="history-time">{entry.durationMs}ms</span>
                </button>
              {/each}
            </div>
            {#if responseHistory[historyIdx]}
              {@const hres = responseHistory[historyIdx].response}
              <div class="history-body scroll-y">
                <pre class="raw-body mono">{hres.body.raw}</pre>
              </div>
            {/if}
          {/if}
        </div>
      {:else if viewMode === "visualize"}
        <div class="visualizer-wrapper">
          <VisualizerIframe />
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

  /* Test badge on view-mode button */
  .visualizer-btn {
    color: var(--accent-primary);
  }
  .test-badge {
    display: inline-block;
    margin-left: 4px;
    padding: 0 5px;
    border-radius: 10px;
    font-size: 9px;
    background: var(--color-success-dim, rgba(0,255,156,0.15));
    color: var(--color-success);
  }
  .test-badge.fail {
    background: var(--color-error-dim);
    color: var(--color-error);
  }

  /* Cookies panel */
  .cookies-viewer { padding: 8px 12px; overflow-x: auto; }

  .cookie-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
  }
  .cookie-table th {
    text-align: left;
    padding: 4px 8px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border-subtle);
  }
  .cookie-table td {
    padding: 5px 8px;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    vertical-align: top;
  }
  .cookie-value { max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .cookie-flag { text-align: center; color: var(--text-muted); }
  .cookie-flag.flag-on { color: var(--color-success); font-weight: 700; }

  /* Tests panel */
  .tests-viewer { padding: 8px 12px; display: flex; flex-direction: column; gap: 2px; }
  .tests-empty  { font-size: 12px; color: var(--text-muted); padding: 16px 0; }
  .tests-empty code { font-family: var(--font-mono); color: var(--accent-primary); }

  .test-row {
    display: grid;
    grid-template-columns: 16px 1fr;
    gap: 8px;
    align-items: start;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    font-size: 12px;
  }
  .test-pass { background: rgba(0,255,156,0.06); }
  .test-fail { background: rgba(255,80,80,0.08); }

  .test-icon { font-size: 11px; font-weight: 700; line-height: 1.6; }
  .test-pass .test-icon { color: var(--color-success); }
  .test-fail .test-icon { color: var(--color-error); }

  .test-name { color: var(--text-primary); line-height: 1.6; }
  .test-error {
    grid-column: 2;
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--color-error);
    opacity: 0.8;
    word-break: break-all;
  }

  .tests-toolbar { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
  .btn-ai-tests {
    display: flex; align-items: center; gap: 8px; background: var(--bg-elevated); border: 1px solid var(--border-default);
    color: var(--accent-primary); font-size: 11px; font-weight: 600; padding: 4px 10px; border-radius: 4px; cursor: pointer;
    transition: var(--transition-fast);
  }
  .btn-ai-tests:hover:not(:disabled) { background: var(--accent-primary-dim); border-color: var(--accent-primary); }
  .btn-ai-tests:disabled { opacity: 0.5; cursor: not-allowed; }

  .ai-error { color: var(--color-error); font-size: 10px; }
  .spinner-small { width: 10px; height: 10px; border: 1px solid var(--accent-primary); border-top-color: transparent; border-radius: 50%; animation: spin 0.8s linear infinite; }

  /* History panel */
  .history-viewer { display: flex; flex-direction: column; height: 100%; }

  .history-list {
    flex-shrink: 0;
    max-height: 180px;
    overflow-y: auto;
    border-bottom: 1px solid var(--border-subtle);
  }

  .history-entry {
    display: grid;
    grid-template-columns: 52px 1fr 46px 52px;
    gap: 8px;
    align-items: center;
    padding: 5px 12px;
    font-size: 11px;
    background: transparent;
    color: var(--text-secondary);
    text-align: left;
    width: 100%;
    border-radius: 0;
    transition: var(--transition-fast);
  }
  .history-entry:hover { background: var(--bg-elevated); }
  .history-entry.active { background: var(--accent-primary-dim); }

  .history-method {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    text-align: center;
    border-radius: 3px;
    padding: 1px 4px;
  }
  .method-get    { color: #4ade80; }
  .method-post   { color: #fb923c; }
  .method-put    { color: #60a5fa; }
  .method-patch  { color: #a78bfa; }
  .method-delete { color: #f87171; }

  .history-url {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
    font-size: 10px;
  }

  .history-status { font-family: var(--font-mono); font-weight: 600; font-size: 11px; }
  .history-status.status-2xx { color: var(--color-success); }
  .history-status.status-4xx { color: var(--color-error); }

  .history-time { font-family: var(--font-mono); font-size: 10px; color: var(--text-muted); text-align: right; }

  .history-body { flex: 1; }
  
  .visualizer-wrapper { flex: 1; height: 100%; display: flex; flex-direction: column; }

  /* ── AI Action Bar ───────────────────────────────────── */
  .ai-action-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-void);
    flex-shrink: 0;
  }

  .ai-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
    cursor: pointer;
    transition: var(--transition-fast);
  }
  .ai-btn:hover:not(:disabled) {
    background: var(--accent-primary-dim);
    color: var(--accent-primary);
    border-color: var(--accent-primary);
  }
  .ai-btn:disabled { opacity: 0.45; cursor: not-allowed; }

  .ai-repair-btn { color: var(--color-warning, #f59e0b); }
  .ai-repair-btn:hover:not(:disabled) {
    background: rgba(245, 158, 11, 0.12);
    color: #f59e0b;
    border-color: rgba(245, 158, 11, 0.4);
  }

  /* ── Repair Panel ────────────────────────────────────── */
  .ai-repair-panel {
    margin: 8px 12px;
    border-radius: var(--radius-md);
    border: 1px solid rgba(245, 158, 11, 0.3);
    background: rgba(245, 158, 11, 0.05);
    overflow: hidden;
    flex-shrink: 0;
  }

  .repair-header {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 10px 12px;
  }

  .repair-priority {
    font-size: 9px;
    font-weight: 800;
    letter-spacing: 0.05em;
    padding: 2px 6px;
    border-radius: 4px;
    flex-shrink: 0;
    margin-top: 1px;
  }
  .priority-high   { background: rgba(239,68,68,0.2);   color: #ef4444; }
  .priority-medium { background: rgba(245,158,11,0.2);  color: #f59e0b; }
  .priority-low    { background: rgba(74,222,128,0.15); color: #4ade80; }

  .repair-diagnosis { flex: 1; font-size: 12px; color: var(--text-primary); line-height: 1.5; }

  .repair-close {
    background: transparent;
    color: var(--text-muted);
    font-size: 16px;
    line-height: 1;
    padding: 0 2px;
    flex-shrink: 0;
    cursor: pointer;
  }
  .repair-close:hover { color: var(--text-primary); }

  .repair-fixes {
    border-top: 1px solid rgba(245,158,11,0.2);
    padding: 8px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .fix-item {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    font-size: 11px;
  }

  .fix-type {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 1px 5px;
    border-radius: 3px;
    background: var(--bg-overlay);
    color: var(--text-muted);
    flex-shrink: 0;
    margin-top: 1px;
  }

  .fix-desc { color: var(--text-secondary); line-height: 1.4; }

  .repair-error {
    padding: 10px 12px;
    font-size: 11px;
    color: var(--color-error);
  }
</style>
