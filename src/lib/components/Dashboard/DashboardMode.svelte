<script lang="ts">
  import { invoke as tauriInvoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Logo from "../Common/Logo.svelte";
  import HealthHeatmapPanel from "./HealthHeatmapPanel.svelte";
  import LiveTrafficPanel from "./LiveTrafficPanel.svelte";

  const invoke = <T>(cmd: string, args?: Record<string, any>): Promise<T> => {
    const fn = tauriInvoke || (window as any)?.__TAURI__?.core?.invoke;
    return fn ? (fn as any)(cmd, args) : Promise.reject("Tauri invoke not found");
  };

  let activeSection = $state<"health" | "loadtest" | "proxy" | "ecosystem">("health");
  let workerConnected = $state(false);

  const sections = [
    { id: "health", label: "Health Monitor", icon: "❤" },
    { id: "loadtest", label: "Load Test", icon: "⚡" },
    { id: "proxy", label: "Traffic", icon: "🔀" },
    { id: "ecosystem", label: "Ecosystem", icon: "🔌" },
  ] as const;

  async function checkWorkerStatus() {
    try {
      workerConnected = await invoke("ping_worker");
      console.log("[Dashboard] Worker status:", workerConnected);
    } catch (e) {
      console.error("[Dashboard] Worker status check failed:", e);
      workerConnected = false;
    }
  }

  onMount(() => {
    checkWorkerStatus();
    const interval = setInterval(checkWorkerStatus, 5000);
    return () => clearInterval(interval);
  });
</script>

<div class="dashboard-mode">
  <!-- Dashboard nav -->
  <div class="dashboard-nav">
    <div class="nav-header">
      <div style="display:flex; align-items:center; gap:8px;">
        <Logo size={16} />
        <span class="nav-title">Command Center</span>
      </div>
      <span class="nav-subtitle">
        Go-worker: 
        <span style="color: {workerConnected ? 'var(--color-success)' : 'var(--color-error)'}">
          {workerConnected ? 'Connected' : 'Disconnected'}
        </span>
      </span>
    </div>

    {#each sections as section}
      <button
        class="nav-item"
        class:active={activeSection === section.id}
        onclick={() => (activeSection = section.id)}
      >
        <span class="nav-icon">{section.icon}</span>
        <span>{section.label}</span>
        {#if section.id === "health"}
          <span class="status-dot {workerConnected ? 'up' : 'down'}" title={workerConnected ? "Worker running" : "Worker down"}></span>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Dashboard content -->
  <div class="dashboard-content">
    {#if activeSection === "health"}
      <HealthHeatmapPanel />

    {:else if activeSection === "loadtest"}
      <div class="dashboard-section animate-fade-in">
        <div class="section-header">
          <h2>Load Test</h2>
          <p class="section-desc">Stress test your APIs with Go's concurrent goroutines</p>
        </div>

        <div class="loadtest-config">
          <div class="config-row">
            <div class="config-field">
              <label for="lt-url">URL</label>
              <input id="lt-url" type="text" placeholder="https://api.example.com/endpoint" class="form-input" />
            </div>
            <div class="config-field small">
              <label for="lt-method">Method</label>
              <select id="lt-method" class="form-select">
                <option>GET</option><option>POST</option><option>PUT</option>
              </select>
            </div>
          </div>

          <div class="config-row">
            <div class="config-field small">
              <label for="lt-users">Concurrent Users</label>
              <input id="lt-users" type="number" value="50" min="1" max="1000" class="form-input" />
            </div>
            <div class="config-field small">
              <label for="lt-total">Total Requests</label>
              <input id="lt-total" type="number" value="500" class="form-input" />
            </div>
            <div class="config-field small">
              <label for="lt-duration">Duration (sec, 0 = use count)</label>
              <input id="lt-duration" type="number" value="0" class="form-input" />
            </div>
          </div>

          <button class="btn-action danger">▶ Start Load Test</button>
        </div>

        <!-- Results placeholder -->
        <div class="loadtest-results">
          <div class="stat-grid">
            {#each [
              { label: "Req/sec", value: "—" },
              { label: "P50 Latency", value: "—" },
              { label: "P95 Latency", value: "—" },
              { label: "P99 Latency", value: "—" },
              { label: "Success", value: "—" },
              { label: "Errors", value: "—" },
            ] as stat}
              <div class="stat-card">
                <span class="stat-label">{stat.label}</span>
                <span class="stat-value">{stat.value}</span>
              </div>
            {/each}
          </div>

          <div class="histogram-placeholder">
            <span class="text-muted">Latency histogram will appear here after a test run</span>
          </div>
        </div>
      </div>

    {:else if activeSection === "proxy"}
      <LiveTrafficPanel />

    {:else if activeSection === "ecosystem"}
      <div class="dashboard-section animate-fade-in">
        <div class="section-header">
          <h2>Ecosystem Intelligence</h2>
          <p class="section-desc">Auto-detect frameworks, explore schemas, and configure auth providers</p>
        </div>

        <div class="ecosystem-grid">
          {#each [
            { id: "frappe", name: "Frappe / ERPNext", badge: "F", color: "var(--accent-primary)", desc: "DocType JSON crawler, sid+CSRF auth" },
            { id: "django", name: "Django", badge: "D", color: "var(--accent-secondary)", desc: "models.py parser, session auth" },
            { id: "laravel", name: "Laravel", badge: "L", color: "var(--color-warning)", desc: "Sanctum XSRF-TOKEN auth, Eloquent models" },
            { id: "rails", name: "Ruby on Rails", badge: "R", color: "var(--color-error)", desc: "schema.rb crawler, authenticity_token" },
            { id: "openapi", name: "OpenAPI / Swagger", badge: "✓", color: "var(--color-success)", desc: "Full spec import, endpoint generation" },
            { id: "wordpress", name: "WordPress", badge: "W", color: "var(--color-info)", desc: "Application Password, nonce injection" },
          ] as fw}
            <div class="fw-card">
              <div class="fw-badge" style="background: rgba(0,0,0,0.3); color: {fw.color}; border: 1px solid {fw.color}33">
                {fw.badge}
              </div>
              <div class="fw-info">
                <div class="fw-name">{fw.name}</div>
                <div class="fw-desc text-muted">{fw.desc}</div>
              </div>
              <button class="fw-connect">Connect →</button>
            </div>
          {/each}
        </div>

        <div class="schema-explorer">
          <h3>Schema Explorer</h3>
          <p class="text-muted" style="font-size:12px; margin-bottom: 10px;">
            Point to a local project folder — Parallax will auto-detect the framework and map all entities + endpoints.
          </p>
          <div class="explorer-input">
            <input type="text" placeholder="e.g. /Users/you/my-frappe-app" class="form-input flex-1" />
            <button class="btn-action">Explore →</button>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .dashboard-mode {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  /* Nav */
  .dashboard-nav {
    width: 200px;
    background: var(--bg-surface);
    border-right: 1px solid var(--border-default);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    padding: 12px 8px;
    gap: 2px;
  }

  .nav-header {
    padding: 4px 8px 12px;
  }
  .nav-title {
    display: block;
    font-size: 13px;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 2px;
  }
  .nav-subtitle {
    font-size: 10px;
    color: var(--text-muted);
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    text-align: left;
    transition: var(--transition-fast);
  }
  .nav-item:hover { background: var(--bg-elevated); color: var(--text-primary); }
  .nav-item.active { background: var(--accent-primary-dim); color: var(--accent-primary); }

  .nav-icon { font-size: 14px; }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    margin-left: auto;
    background: var(--text-muted);
  }
  .status-dot.up {
    background: var(--color-success);
    animation: pulse-dot 2s infinite;
  }
  .status-dot.down {
    background: var(--color-error);
  }

  /* Content */
  .dashboard-content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    display: flex;
    flex-direction: column;
  }

  /* Forms */
  .config-row, .explorer-input {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
    align-items: flex-end;
  }

  .config-field { display: flex; flex-direction: column; gap: 4px; flex: 1; }
  .config-field label { font-size: 10px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; }
  .config-field.small { flex: 0 0 140px; }

  .form-input {
    height: 34px;
    padding: 0 10px;
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: 13px;
  }
  .form-input.flex-1 { flex: 1; }
  .form-select {
    height: 34px;
    padding: 0 10px;
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: 12px;
    cursor: pointer;
  }

  .btn-action {
    height: 34px;
    padding: 0 16px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--radius-md);
    white-space: nowrap;
    transition: var(--transition-fast);
    cursor: pointer;
  }
  .btn-action:hover { border-color: var(--accent-primary); color: var(--accent-primary); }
  .btn-action.danger:hover { border-color: var(--color-error); color: var(--color-error); }

  /* Load test */
  .loadtest-config {
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    padding: 16px;
    margin-bottom: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .stat-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    margin-bottom: 16px;
  }

  .stat-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .stat-label { font-size: 10px; text-transform: uppercase; color: var(--text-muted); font-weight: 600; }
  .stat-value { font-size: 22px; font-family: var(--font-mono); font-weight: 700; color: var(--text-primary); }

  .histogram-placeholder {
    height: 120px;
    background: var(--bg-surface);
    border: 1px dashed var(--border-default);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
  }

  /* Ecosystem */
  .ecosystem-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 10px;
    margin-bottom: 24px;
  }

  .fw-card {
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    padding: 14px;
    transition: var(--transition-base);
  }
  .fw-card:hover { border-color: var(--border-accent); }

  .fw-badge {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 800;
    font-size: 15px;
    border-radius: var(--radius-md);
    flex-shrink: 0;
    font-family: var(--font-mono);
  }

  .fw-info { flex: 1; }
  .fw-name { font-size: 13px; font-weight: 600; margin-bottom: 2px; }
  .fw-desc { font-size: 11px; }

  .fw-connect {
    background: transparent;
    color: var(--accent-primary);
    font-size: 11px;
    font-weight: 600;
    border-radius: var(--radius-sm);
    padding: 4px 8px;
    transition: var(--transition-fast);
    white-space: nowrap;
  }
  .fw-connect:hover { background: var(--accent-primary-dim); }

  .schema-explorer {
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    padding: 16px;
  }
  .schema-explorer h3 { font-size: 14px; font-weight: 700; margin-bottom: 6px; }

</style>
