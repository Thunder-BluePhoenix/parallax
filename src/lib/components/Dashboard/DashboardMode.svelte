<script lang="ts">
  import { invoke as tauriInvoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import Logo from "../Common/Logo.svelte";
  import HealthHeatmapPanel from "./HealthHeatmapPanel.svelte";
  import LiveTrafficPanel from "./LiveTrafficPanel.svelte";
  import LoadTestPanel from "./LoadTestPanel.svelte";
  import MockServerPanel from "./MockServerPanel.svelte";
  import AISettingsPanel from "../AI/AISettingsPanel.svelte";
  import GitPanel from "./GitPanel.svelte";
  import TeamPanel from "./TeamPanel.svelte";
  import ChatPanel from "./ChatPanel.svelte";
  import DocsPanel from "./DocsPanel.svelte";
  import FlowPanel from "./FlowPanel.svelte";
  import { unreadCount } from "../../stores/github.svelte";


  // Schema Explorer state
  let explorerPath = $state("");
  let explorationResult = $state<any>(null);
  let isExploring = $state(false);

  const invoke = <T>(cmd: string, args?: Record<string, any>): Promise<T> => {
    const fn = tauriInvoke || (window as any)?.__TAURI__?.core?.invoke;
    return fn ? (fn as any)(cmd, args) : Promise.reject("Tauri invoke not found");
  };

  let activeSection = $state<"health" | "loadtest" | "proxy" | "mock" | "history" | "ecosystem" | "ai_settings" | "git" | "team" | "chat" | "docs" | "flow">("health");
  let workerConnected = $state(false);

  const sections = [
    { id: "health", label: "Health Monitor", icon: "❤" },
    { id: "loadtest", label: "Load Test", icon: "⚡" },
    { id: "proxy", label: "Traffic", icon: "🔀" },
    { id: "mock", label: "Mock Server", icon: "🎭" },
    { id: "history", label: "Run History", icon: "🕒" },
    { id: "ai_settings", label: "AI Settings", icon: "🧠" },
    { id: "ecosystem", label: "Ecosystem", icon: "🔌" },
    { id: "git", label: "Git", icon: "⎇" },
    { id: "team", label: "Team", icon: "👥" },
    { id: "chat", label: "Chat", icon: "💬", badge: true },
    { id: "docs", label: "API Docs", icon: "📄" },
    { id: "flow", label: "Flow Builder", icon: "🕸" },
  ] as const;

  // Run history stats
  import { responseHistory } from "../../stores/app.svelte";

  async function checkWorkerStatus() {
    try {
      workerConnected = await invoke("ping_worker");
      console.log("[Dashboard] Worker status:", workerConnected);
    } catch (e) {
      console.error("[Dashboard] Worker status check failed:", e);
      workerConnected = false;
    }
  }
  async function selectFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Project Root"
      });
      if (selected) {
        explorerPath = selected as string;
        runExploration();
      }
    } catch (e) {
      console.error("[Dashboard] Folder selection failed:", e);
    }
  }

  async function runExploration() {
    if (!explorerPath) return;
    isExploring = true;
    try {
      explorationResult = await invoke("explore_schema", { rootPath: explorerPath });
      console.log("[Dashboard] Exploration result:", explorationResult);
    } catch (e) {
      console.error("[Dashboard] Exploration failed:", e);
      alert("Failed to explore project: " + e);
    } finally {
      isExploring = false;
    }
  }

  onMount(() => {
    checkWorkerStatus();
    
    // Start background streams
    invoke("start_health_stream").catch(console.error);
    invoke("start_proxy_stream").catch(console.error);

    // Add some default health targets for the demo
    const defaultTargets = [
      { id: "google", name: "Google.com", url: "https://www.google.com", interval_sec: 10, timeout_ms: 2000 },
      { id: "github", name: "GitHub", url: "https://github.com", interval_sec: 30, timeout_ms: 3000 },
      { id: "localhost", name: "Local Dev", url: "http://localhost:1420", interval_sec: 5, timeout_ms: 1000 },
    ];

    defaultTargets.forEach(t => {
      invoke("add_health_target", t).catch(console.error);
    });

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
        {:else if section.id === "chat" && unreadCount.value > 0}
          <span class="nav-badge">{unreadCount.value}</span>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Dashboard content -->
  <div class="dashboard-content">
    {#if activeSection === "health"}
      <HealthHeatmapPanel />

    {:else if activeSection === "loadtest"}
      <LoadTestPanel />

    {:else if activeSection === "proxy"}
      <LiveTrafficPanel />

    {:else if activeSection === "mock"}
      <MockServerPanel />

    {:else if activeSection === "ai_settings"}
      <AISettingsPanel />

    {:else if activeSection === "git"}
      <GitPanel />

    {:else if activeSection === "team"}
      <TeamPanel />

    {:else if activeSection === "chat"}
      <ChatPanel />

    {:else if activeSection === "docs"}
      <DocsPanel />

    {:else if activeSection === "flow"}
      <FlowPanel />

    {:else if activeSection === "history"}
      <div class="dashboard-section animate-fade-in">
        <div class="section-header">
          <h2>Collection Run History</h2>
          <p class="section-desc">View recent test execution history and exports</p>
        </div>
        <div class="history-grid">
          {#if responseHistory.length === 0}
            <div class="traffic-empty">
              <span class="text-muted">No runs have been executed yet.</span>
            </div>
          {:else}
            {#each responseHistory.slice(0, 50) as entry (entry.id)}
              <div class="history-card">
                <div class="history-card-header">
                  <div style="font-weight: 600; font-size: 13px;">{entry.requestName}</div>
                  <div class="text-muted mono" style="font-size: 10px;">{new Date(entry.timestamp).toLocaleString()}</div>
                </div>
                <div style="display:flex; gap: 8px; font-size: 11px; margin-top: 8px;">
                  <span class="status-badge" class:ok={entry.status < 400} class:err={entry.status >= 400}>{entry.status}</span>
                  <span class="text-muted">{entry.durationMs}ms</span>
                  <span class="text-muted">{entry.method} {entry.url}</span>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>

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
            <input 
              type="text" 
              bind:value={explorerPath}
              placeholder="e.g. /Users/you/my-frappe-app" 
              class="form-input flex-1" 
            />
            <button class="btn-action" onclick={selectFolder} disabled={isExploring}>
              {isExploring ? "Scanning..." : "Explore →"}
            </button>
          </div>

          {#if explorationResult}
            <div class="exploration-results">
              <div class="result-header">
                <span class="framework-badge">{explorationResult.framework.toUpperCase()} Detected</span>
                <span class="entity-count">{explorationResult.entities.length} Entities Found</span>
                <button class="btn-send" style="margin-left: auto; padding: 4px 12px; font-size: 11px;" onclick={async () => {
                  try {
                    const collectionName = explorationResult.framework.charAt(0).toUpperCase() + explorationResult.framework.slice(1) + " API";
                    let folders = [];
                    for (const entity of explorationResult.entities) {
                      let entityRequests = [];
                      for (const ep of entity.endpoints) {
                        entityRequests.push({
                          id: "req_" + Math.random().toString(36).substr(2, 9),
                          name: `${ep.method} ${entity.name}`,
                          method: ep.method,
                          url: "{{base_url}}" + ep.path,
                          headers: { "Content-Type": "application/json" },
                          params: {},
                          body: { type: ep.method === "GET" || ep.method === "DELETE" ? "none" : "json", content: "{\n}", raw: "{\n}" },
                          auth: { type: "ecosystem_provider", provider: explorationResult.framework },
                          scripts: null
                        });
                      }
                      folders.push({ name: entity.name, requests: entityRequests });
                    }
                    const newCollection = {
                      name: collectionName,
                      version: "1.0.0",
                      description: `Auto-generated from ${explorationResult.root_path}`,
                      requests: [],
                      folders: folders,
                      variables: { "base_url": "http://localhost:8000" }
                    };
                    await invoke("save_collection", { collection: newCollection });
                    alert("Collection generated! Switch to Builder mode to use it.");
                  } catch (e) {
                    alert("Failed to save collection: " + e);
                  }
                }}>
                  Generate Collection
                </button>
              </div>
              <div class="entity-scroll">
                {#each explorationResult.entities as entity}
                  <div class="entity-item">
                    <div class="entity-name">{entity.name}</div>
                    <div class="entity-meta">{entity.fields.length} fields • {entity.endpoints.length} endpoints</div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
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

  .nav-badge {
    margin-left: auto;
    background: var(--color-error); color: white;
    font-size: 9px; font-weight: 700;
    padding: 1px 5px; border-radius: 10px; min-width: 16px; text-align: center;
  }

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
  /* Ecosystem */

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

  .exploration-results {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid var(--border-default);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .result-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .framework-badge {
    background: var(--accent-primary-dim);
    color: var(--accent-primary);
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 10px;
    font-weight: 700;
  }
  .entity-count {
    font-size: 11px;
    color: var(--text-muted);
  }
  .entity-scroll {
    max-height: 200px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-right: 4px;
  }
  .entity-item {
    padding: 8px 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .entity-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
  }
  .entity-meta {
    font-size: 10px;
    color: var(--text-muted);
  }

  .history-grid { display: flex; flex-direction: column; gap: 8px; }
  .history-card {
    background: var(--bg-surface); border: 1px solid var(--border-default);
    padding: 12px; border-radius: var(--radius-md);
  }
  .history-card-header { display: flex; justify-content: space-between; }
  .status-badge {
    padding: 2px 6px; border-radius: 4px; font-weight: 700; font-family: var(--font-mono);
  }
  .status-badge.ok { background: rgba(63, 185, 80, 0.2); color: var(--color-success); }
  .status-badge.err { background: rgba(248, 81, 73, 0.2); color: var(--color-error); }
  .traffic-empty { padding: 40px; text-align: center; font-size: 12px; }
</style>
