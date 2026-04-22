<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import { sendNotification } from "@tauri-apps/plugin-notification";
  import Logo from "../Common/Logo.svelte";

  // Types
  interface HealthEvent {
    id: string;
    name: string;
    url: string;
    status: "up" | "down" | "slow" | "unknown";
    latency_ms: number;
    status_code: number;
    last_checked: number;
    error_msg?: string;
  }

  // State
  let statuses = $state<Record<string, HealthEvent>>({});
  let newName = $state("");
  let newUrl = $state("");
  let newInterval = $state(30);
  let newWebhook = $state("");

  let unlisten: () => void;

  async function loadInitial() {
    try {
      const result: HealthEvent[] = await invoke("get_health_statuses");
      for (const st of result) {
        statuses[st.id] = st;
      }
    } catch (e) {
      console.error("Failed to get initial health statuses:", e);
    }
  }

  async function addTarget() {
    if (!newName || !newUrl) return;
    const id = "target-" + Date.now();
    try {
      await invoke("add_health_target", {
        id,
        name: newName,
        url: newUrl,
        intervalSec: newInterval,
        timeoutMs: 5000,
        alertWebhook: newWebhook
      });
      newName = "";
      newUrl = "";
      newWebhook = "";
      // The Go sidecar will do an initial check and fire an event back
    } catch (e) {
      console.error("Failed to add target:", e);
    }
  }

  async function removeTarget(id: string) {
    try {
      await invoke("remove_health_target", { id });
      delete statuses[id];
    } catch (e) {
      console.error("Failed to remove target:", e);
    }
  }

  onMount(async () => {
    // Start the gRPC stream
    await invoke("start_health_stream");
    // Load existing
    await loadInitial();

    // Listen to events from Rust
    unlisten = await listen<HealthEvent>("health_status_event", (event) => {
      const prev = statuses[event.payload.id];
      statuses[event.payload.id] = event.payload;

      // Desktop Notification on state change
      if (prev && prev.status !== event.payload.status) {
        if (event.payload.status === "down") {
          sendNotification({ title: `Service Down: ${event.payload.name}`, body: event.payload.error_msg || "Health check failed." });
        } else if (event.payload.status === "up" && prev.status === "down") {
          sendNotification({ title: `Service Restored: ${event.payload.name}`, body: "Service is back online." });
        }
      }
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });
</script>

<div class="dashboard-section animate-fade-in">
  <div class="section-header">
    <h2>Health Monitor</h2>
    <p class="section-desc">Real-time background health checks via Go goroutines streaming over gRPC</p>
  </div>

  <div class="add-target-form">
    <div class="config-field">
      <label for="h-name">Service Name</label>
      <input id="h-name" type="text" placeholder="e.g. Auth Service" class="form-input" bind:value={newName} />
    </div>
    <div class="config-field flex-1">
      <label for="h-url">URL</label>
      <input id="h-url" type="text" placeholder="https://api.example.com/health" class="form-input" bind:value={newUrl} />
    </div>
    <div class="config-field" style="flex: 0 0 100px;">
      <label for="h-interval">Interval</label>
      <select id="h-interval" class="form-select" bind:value={newInterval}>
        <option value={10}>10s</option>
        <option value={30}>30s</option>
        <option value={60}>1m</option>
        <option value={300}>5m</option>
      </select>
    </div>
    <div class="config-field">
      <label for="h-webhook">Webhook (Optional)</label>
      <input id="h-webhook" type="text" placeholder="https://alerts.my..." class="form-input" bind:value={newWebhook} />
    </div>
    <button class="btn-action" onclick={addTarget} disabled={!newName || !newUrl}>Add Monitor</button>
  </div>

  <div class="health-grid">
    {#each Object.values(statuses) as svc (svc.id)}
      <div class="health-card status-{svc.status}">
        <div class="health-card-header">
          <div style="display:flex; align-items:center; gap:6px;">
            <Logo size={12} />
            <span class="health-name">{svc.name}</span>
          </div>
          <div class="header-actions">
            <span class="health-status {svc.status === 'up' ? 'status-2xx' : 'status-5xx'}">
              <span class="pulse-dot" class:up={svc.status === 'up'}></span>
              {svc.status.toUpperCase()}
            </span>
            <button class="btn-remove" onclick={() => removeTarget(svc.id)} title="Remove target">×</button>
          </div>
        </div>
        <div class="health-url text-muted mono">{svc.url}</div>
        
        {#if svc.status === "up" || svc.status === "slow"}
          <div class="health-latency">
            <span class="latency-bar-wrap">
              <span class="latency-bar" class:slow={svc.status === "slow"} style="width: {Math.min(svc.latency_ms / 2, 100)}%"></span>
            </span>
            <span class="latency-val">{Math.round(svc.latency_ms)}ms</span>
            {#if svc.status_code}
              <span class="status-code">{svc.status_code}</span>
            {/if}
          </div>
        {:else}
          <div class="health-error text-muted">{svc.error_msg || `Status Code: ${svc.status_code}`}</div>
        {/if}
        
        <div class="health-footer">
          Last checked: {new Date(svc.last_checked * 1000).toLocaleTimeString()}
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .dashboard-section { max-width: 900px; }
  .section-header { margin-bottom: 20px; }
  .section-header h2 { font-size: 18px; font-weight: 700; margin-bottom: 4px; }
  .section-desc { font-size: 12px; color: var(--text-secondary); }

  .add-target-form {
    display: flex;
    gap: 8px;
    margin-bottom: 24px;
    align-items: flex-end;
  }
  .config-field { display: flex; flex-direction: column; gap: 4px; }
  .config-field label { font-size: 10px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; }
  .flex-1 { flex: 1; }
  .form-input {
    height: 34px; padding: 0 10px; background: var(--bg-input); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); color: var(--text-primary); font-size: 13px;
  }
  .form-select {
    height: 34px; padding: 0 10px; background: var(--bg-input); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); color: var(--text-primary); font-size: 12px; cursor: pointer;
  }
  .btn-action {
    height: 34px; padding: 0 16px; background: var(--bg-elevated); border: 1px solid var(--border-default);
    color: var(--text-primary); font-size: 12px; font-weight: 600; border-radius: var(--radius-md);
    transition: var(--transition-fast); cursor: pointer;
  }
  .btn-action:hover:not(:disabled) { border-color: var(--accent-primary); color: var(--accent-primary); }
  .btn-action:disabled { opacity: 0.5; cursor: not-allowed; }

  .health-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 12px; }

  .health-card {
    background: var(--bg-surface); border: 1px solid var(--border-default);
    border-radius: var(--radius-lg); padding: 14px; transition: var(--transition-base);
    position: relative;
  }
  .health-card.status-up { border-color: rgba(63, 185, 80, 0.2); }
  .health-card.status-slow { border-color: rgba(210, 153, 34, 0.3); }
  .health-card.status-down { border-color: rgba(248, 81, 73, 0.2); }

  .health-card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; }
  .health-name { font-weight: 600; font-size: 13px; }
  
  .header-actions { display: flex; align-items: center; gap: 8px; }
  .health-status { display: flex; align-items: center; gap: 5px; font-size: 10px; font-weight: 700; font-family: var(--font-mono); }
  .status-2xx { color: var(--color-success); }
  .status-5xx { color: var(--color-error); }
  
  .btn-remove {
    background: transparent; color: var(--text-muted); font-size: 16px; line-height: 1;
    border: none; cursor: pointer; padding: 0 2px;
  }
  .btn-remove:hover { color: var(--color-error); }

  .health-url { font-size: 11px; margin-bottom: 10px; word-break: break-all; }

  .latency-bar-wrap { flex: 1; height: 3px; background: var(--bg-overlay); border-radius: 2px; overflow: hidden; display: block; }
  .latency-bar { height: 100%; background: var(--color-success); display: block; transition: width 0.3s; }
  .latency-bar.slow { background: var(--color-warning); }
  
  .health-latency { display: flex; align-items: center; gap: 8px; }
  .latency-val { font-size: 11px; font-family: var(--font-mono); color: var(--text-secondary); }
  .status-code { font-size: 10px; font-family: var(--font-mono); padding: 2px 4px; background: var(--bg-overlay); border-radius: 3px; }
  
  .health-error { font-size: 11px; margin-top: 6px; color: var(--color-error); }

  .health-footer { font-size: 10px; color: var(--text-muted); margin-top: 10px; text-align: right; font-family: var(--font-mono); }

  .pulse-dot { display: inline-block; width: 7px; height: 7px; border-radius: 50%; background: var(--text-muted); }
  .pulse-dot.up { background: var(--color-success); animation: pulse-dot 2s infinite; }
</style>
