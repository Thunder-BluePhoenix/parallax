<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import Logo from "../Common/Logo.svelte";

  interface TrafficEvent {
    id: string;
    timestamp_ms: number;
    method: string;
    url: string;
    status_code: number;
    latency_ms: number;
    content_type: string;
    preview: string;
  }

  let traffic = $state<TrafficEvent[]>([]);
  let unlisten: () => void;

  async function loadInitial() {
    try {
      traffic = await invoke("get_proxy_traffic", { limit: 100 });
    } catch (e) {
      console.error("Failed to load traffic:", e);
    }
  }

  async function clearTraffic() {
    try {
      await invoke("clear_proxy_traffic");
      traffic = [];
    } catch (e) {
      console.error("Failed to clear traffic:", e);
    }
  }

  onMount(async () => {
    await invoke("start_proxy_stream");
    await loadInitial();

    unlisten = await listen<TrafficEvent>("proxy_traffic_event", (event) => {
      // Append to the list, keeping max 500 for UI performance
      traffic.push(event.payload);
      if (traffic.length > 500) {
        traffic.shift();
      }
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });

  function methodColor(m: string) {
    const map: Record<string, string> = {
      GET: "method-get", POST: "method-post", PUT: "method-put",
      PATCH: "method-patch", DELETE: "method-delete", OPTIONS: "method-options"
    };
    return map[m.toUpperCase()] || "method-get";
  }
</script>

<div class="dashboard-section animate-fade-in" style="max-width: 100%;">
  <div class="section-header">
    <h2>Traffic Interceptor</h2>
    <p class="section-desc">Configure your app to use the local proxy on port 8888. Traffic streams in real-time.</p>
  </div>

  <div class="proxy-controls">
    <div class="proxy-status">
      <Logo size={14} />
      <span>Proxy listening on <span class="mono" style="color:var(--accent-secondary)">127.0.0.1:8888</span></span>
    </div>
    <button class="btn-action" onclick={clearTraffic}>Clear Traffic</button>
  </div>

  <div class="traffic-list">
    <div class="traffic-header">
      <span style="width: 80px">Time</span>
      <span style="width: 60px">Method</span>
      <span style="flex: 2">URL</span>
      <span style="width: 60px">Status</span>
      <span style="width: 70px">Latency</span>
      <span style="flex: 1">Type</span>
    </div>
    
    <div class="traffic-body scroll-y">
      {#if traffic.length === 0}
        <div class="traffic-empty">
          <span class="text-muted">Traffic will appear here once your app sends requests through the proxy</span>
        </div>
      {:else}
        {#each traffic.slice().reverse() as t (t.id)}
          <div class="traffic-row">
            <span class="cell time">{new Date(t.timestamp_ms).toLocaleTimeString([], { hour12: false, fractionalSecondDigits: 3 })}</span>
            <span class="cell method"><span class="method-badge {methodColor(t.method)}">{t.method}</span></span>
            <span class="cell url">{t.url}</span>
            <span class="cell status" class:status-err={t.status_code >= 400} class:status-ok={t.status_code < 300}>{t.status_code}</span>
            <span class="cell latency">{Math.round(t.latency_ms)}ms</span>
            <span class="cell type">{t.content_type.split(';')[0] || '-'}</span>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .dashboard-section { display: flex; flex-direction: column; height: 100%; }
  .section-header { margin-bottom: 20px; flex-shrink: 0; }
  .section-header h2 { font-size: 18px; font-weight: 700; margin-bottom: 4px; }
  .section-desc { font-size: 12px; color: var(--text-secondary); }

  .proxy-controls { display: flex; align-items: center; gap: 12px; margin-bottom: 16px; flex-shrink: 0; }
  .proxy-status { display: flex; align-items: center; gap: 8px; font-size: 13px; flex: 1; }
  .btn-action {
    height: 30px; padding: 0 16px; background: var(--bg-elevated); border: 1px solid var(--border-default);
    color: var(--text-primary); font-size: 12px; font-weight: 600; border-radius: var(--radius-md);
    transition: var(--transition-fast); cursor: pointer;
  }
  .btn-action:hover { border-color: var(--accent-primary); color: var(--accent-primary); }

  .traffic-list {
    background: var(--bg-surface); border: 1px solid var(--border-default);
    border-radius: var(--radius-lg); overflow: hidden; display: flex; flex-direction: column;
    flex: 1; min-height: 0;
  }

  .traffic-header {
    display: flex; gap: 12px; padding: 8px 12px; background: var(--bg-elevated);
    border-bottom: 1px solid var(--border-default); font-size: 10px; font-weight: 700;
    color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.06em; flex-shrink: 0;
  }

  .traffic-body { flex: 1; overflow-y: auto; }

  .traffic-row {
    display: flex; gap: 12px; padding: 6px 12px; border-bottom: 1px solid var(--border-subtle);
    font-size: 12px; align-items: center; transition: var(--transition-fast);
  }
  .traffic-row:hover { background: var(--bg-elevated); }

  .cell { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-family: var(--font-mono); }
  .cell.time { width: 80px; font-size: 10px; color: var(--text-muted); }
  .cell.method { width: 60px; }
  .cell.url { flex: 2; font-family: var(--font-sans); }
  .cell.status { width: 60px; font-weight: 600; }
  .cell.latency { width: 70px; color: var(--text-secondary); }
  .cell.type { flex: 1; color: var(--text-muted); font-size: 10px; }

  .status-ok { color: var(--color-success); }
  .status-err { color: var(--color-error); }

  .method-badge { font-size: 9px; padding: 1px 4px; border-radius: 3px; font-weight: 700; }
  .method-get { background: rgba(74, 222, 128, 0.15); color: #4ade80; }
  .method-post { background: rgba(251, 146, 60, 0.15); color: #fb923c; }
  .method-put { background: rgba(96, 165, 250, 0.15); color: #60a5fa; }
  .method-delete { background: rgba(248, 113, 113, 0.15); color: #f87171; }
  .method-patch { background: rgba(250, 204, 21, 0.15); color: #facc15; }
  .method-options { background: rgba(167, 139, 250, 0.15); color: #a78bfa; }

  .traffic-empty { padding: 40px; text-align: center; font-size: 12px; }

  .pulse-dot { display: inline-block; width: 7px; height: 7px; border-radius: 50%; background: var(--text-muted); }
  .pulse-dot.up { background: var(--color-success); animation: pulse-dot 2s infinite; }
</style>
