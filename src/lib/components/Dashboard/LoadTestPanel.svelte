<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  // Types
  interface LoadTestProgress {
    completed: number;
    total: number;
    current_rps: number;
    done: boolean;
    result?: LoadTestResult;
  }

  interface LoadTestResult {
    total_requests: number;
    successful: number;
    failed: number;
    avg_latency_ms: number;
    p50_latency_ms: number;
    p95_latency_ms: number;
    p99_latency_ms: number;
    min_latency_ms: number;
    max_latency_ms: number;
    reqs_per_sec: number;
    errors: string[];
    histogram: number[];
  }

  // State
  let url = $state("https://www.google.com");
  let method = $state("GET");
  let concurrent = $state(10);
  let totalRequests = $state(100);
  
  let isRunning = $state(false);
  let progress = $state<LoadTestProgress | null>(null);
  let lastResult = $state<LoadTestResult | null>(null);
  let unlisten: () => void;

  async function startTest() {
    if (isRunning) return;
    isRunning = true;
    progress = { completed: 0, total: totalRequests, current_rps: 0, done: false };
    lastResult = null;

    try {
      await invoke("run_load_test", {
        url,
        method,
        concurrent,
        totalRequests
      });
    } catch (e) {
      console.error("Failed to start load test:", e);
      isRunning = false;
    }
  }

  onMount(async () => {
    unlisten = await listen<LoadTestProgress>("loadtest_progress_event", (event) => {
      progress = event.payload;
      if (event.payload.done) {
        isRunning = false;
        if (event.payload.result) {
          lastResult = event.payload.result;
        }
      }
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });

  function getBarHeight(val: number, max: number) {
    if (max === 0) return "0%";
    return `${(val / max) * 100}%`;
  }
</script>

<div class="dashboard-section animate-fade-in">
  <div class="section-header">
    <h2>Load Test</h2>
    <p class="section-desc">Local high-concurrency request engine powered by Go goroutines</p>
  </div>

  <div class="test-config">
    <div class="config-row">
      <div class="config-field flex-1">
        <label>Target URL</label>
        <input type="text" class="form-input" bind:value={url} placeholder="https://api.example.com" />
      </div>
      <div class="config-field" style="width: 80px;">
        <label>Method</label>
        <select class="form-select" bind:value={method}>
          <option>GET</option>
          <option>POST</option>
          <option>PUT</option>
          <option>DELETE</option>
        </select>
      </div>
    </div>
    <div class="config-row">
      <div class="config-field">
        <label>Concurrency</label>
        <input type="number" class="form-input" bind:value={concurrent} min="1" max="100" />
      </div>
      <div class="config-field">
        <label>Total Requests</label>
        <input type="number" class="form-input" bind:value={totalRequests} min="1" max="10000" />
      </div>
      <button class="btn-action" onclick={startTest} disabled={isRunning}>
        {isRunning ? "Running..." : "Start Load Test"}
      </button>
    </div>
  </div>

  {#if isRunning && progress}
    <div class="test-progress-card">
      <div class="progress-stats">
        <div class="p-stat">
          <span class="p-val">{progress.completed} / {progress.total}</span>
          <span class="p-label">Completed</span>
        </div>
        <div class="p-stat">
          <span class="p-val">{progress.current_rps.toFixed(1)}</span>
          <span class="p-label">Reqs / Sec</span>
        </div>
      </div>
      <div class="progress-bar-bg">
        <div class="progress-bar-fill" style="width: {(progress.completed / progress.total) * 100}%"></div>
      </div>
    </div>
  {/if}

  {#if lastResult}
    <div class="test-results-grid">
      <div class="result-card main-stats">
        <h3>Summary</h3>
        <div class="stats-row">
          <div class="stat-item">
            <span class="label">Successful</span>
            <span class="val text-success">{lastResult.successful}</span>
          </div>
          <div class="stat-item">
            <span class="label">Failed</span>
            <span class="val text-error">{lastResult.failed}</span>
          </div>
          <div class="stat-item">
            <span class="label">Avg RPS</span>
            <span class="val">{lastResult.reqs_per_sec.toFixed(1)}</span>
          </div>
        </div>
      </div>

      <div class="result-card latencies">
        <h3>Latencies</h3>
        <div class="stats-grid">
          <div class="stat-item">
            <span class="label">Average</span>
            <span class="val">{lastResult.avg_latency_ms.toFixed(1)}ms</span>
          </div>
          <div class="stat-item">
            <span class="label">P50</span>
            <span class="val">{lastResult.p50_latency_ms.toFixed(1)}ms</span>
          </div>
          <div class="stat-item">
            <span class="label">P95</span>
            <span class="val">{lastResult.p95_latency_ms.toFixed(1)}ms</span>
          </div>
          <div class="stat-item">
            <span class="label">P99</span>
            <span class="val">{lastResult.p99_latency_ms.toFixed(1)}ms</span>
          </div>
        </div>
      </div>

      <div class="result-card histogram-card">
        <h3>Latency Distribution</h3>
        <div class="histogram">
          {#each lastResult.histogram as count}
            <div class="hist-bar" style="height: {getBarHeight(count, Math.max(...lastResult.histogram))}">
              <div class="hist-tooltip">{count} requests</div>
            </div>
          {/each}
        </div>
      </div>
      
      {#if lastResult.errors.length > 0}
        <div class="result-card errors-card">
          <h3>Top Errors</h3>
          <div class="error-list mono">
            {#each lastResult.errors.slice(0, 5) as err}
              <div class="error-item">{err}</div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .test-config {
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    padding: 20px;
    margin-bottom: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .config-row { display: flex; gap: 16px; align-items: flex-end; }
  .config-field { display: flex; flex-direction: column; gap: 6px; }
  .config-field label { font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; }
  .flex-1 { flex: 1; }

  .form-input {
    height: 36px; padding: 0 12px; background: var(--bg-input); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); color: var(--text-primary); font-size: 13px;
  }
  .form-select {
    height: 36px; padding: 0 10px; background: var(--bg-input); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); color: var(--text-primary); font-size: 12px;
  }

  .btn-action {
    height: 36px; padding: 0 20px; background: var(--accent-primary); color: white;
    border-radius: var(--radius-md); font-weight: 600; font-size: 13px; border: none;
    cursor: pointer; transition: var(--transition-fast);
  }
  .btn-action:hover:not(:disabled) { background: #9185ff; }
  .btn-action:disabled { opacity: 0.5; cursor: not-allowed; }

  .test-progress-card {
    background: var(--bg-surface); border: 1px solid var(--accent-primary);
    border-radius: var(--radius-lg); padding: 20px; margin-bottom: 24px;
  }
  .progress-stats { display: flex; gap: 32px; margin-bottom: 16px; }
  .p-stat { display: flex; flex-direction: column; }
  .p-val { font-size: 20px; font-weight: 700; font-family: var(--font-mono); }
  .p-label { font-size: 11px; color: var(--text-muted); text-transform: uppercase; }

  .progress-bar-bg { height: 8px; background: var(--bg-void); border-radius: 4px; overflow: hidden; }
  .progress-bar-fill { height: 100%; background: var(--accent-primary); transition: width 0.3s; }

  .test-results-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
  .result-card { background: var(--bg-surface); border: 1px solid var(--border-default); border-radius: var(--radius-lg); padding: 16px; }
  .result-card h3 { font-size: 12px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); margin-bottom: 16px; }

  .stats-row { display: flex; gap: 24px; }
  .stats-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
  .stat-item { display: flex; flex-direction: column; gap: 2px; }
  .stat-item .label { font-size: 11px; color: var(--text-secondary); }
  .stat-item .val { font-size: 18px; font-weight: 600; font-family: var(--font-mono); }

  .histogram-card { grid-column: span 2; }
  .histogram { display: flex; align-items: flex-end; gap: 4px; height: 120px; padding-top: 20px; }
  .hist-bar { flex: 1; background: var(--accent-primary-dim); border-radius: 2px 2px 0 0; position: relative; min-height: 2px; }
  .hist-bar:hover { background: var(--accent-primary); }
  .hist-tooltip { 
    position: absolute; bottom: 100%; left: 50%; transform: translateX(-50%);
    background: var(--bg-elevated); padding: 4px 8px; border-radius: 4px; font-size: 10px;
    white-space: nowrap; opacity: 0; pointer-events: none; transition: opacity 0.2s;
  }
  .hist-bar:hover .hist-tooltip { opacity: 1; }

  .errors-card { grid-column: span 2; border-color: var(--color-error-dim); }
  .error-list { font-size: 11px; color: var(--color-error); display: flex; flex-direction: column; gap: 4px; }
  .error-item { padding: 4px 8px; background: rgba(255,80,80,0.05); border-radius: 4px; }
</style>
