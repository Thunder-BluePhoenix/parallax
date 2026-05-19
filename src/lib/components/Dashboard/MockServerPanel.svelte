<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Logo from "../Common/Logo.svelte";

  interface MockRule {
    id: string;
    path: string;
    method: string;
    status_code: number;
    body: string;
    headers: Record<string, string>;
    content_type: string;
  }

  let rules = $state<MockRule[]>([]);
  let newPath = $state("/");
  let newMethod = $state("GET");
  let newStatus = $state(200);
  let newBody = $state('{"message": "Hello from Parallax Mock"}');
  let newContentType = $state("application/json");
  let newDelayMs = $state(0);

  // Record mode
  let recordLimit = $state(20);
  let recordLoading = $state(false);
  let recordResult = $state<{ count: number; paths: string[] } | null>(null);
  let recordError = $state<string | null>(null);

  async function loadRules() {
    try {
       rules = await invoke("list_mock_rules");
    } catch (e) {
       console.error("Failed to load mock rules:", e);
    }
  }

  async function addRule() {
    const id = "mock-" + Date.now();
    const headers: Record<string, string> = {};
    if (newDelayMs > 0) headers["x-parallax-delay-ms"] = String(newDelayMs);
    try {
      await invoke("add_mock_rule", {
        id,
        path: newPath,
        method: newMethod,
        statusCode: newStatus,
        body: newBody,
        headers,
        contentType: newContentType
      });
      rules.push({
        id, path: newPath, method: newMethod, status_code: newStatus,
        body: newBody, headers, content_type: newContentType
      });
      newPath = "/";
      newBody = '{"message": "success"}';
      newDelayMs = 0;
    } catch (e) {
      console.error("Failed to add mock rule:", e);
    }
  }

  async function removeRule(id: string) {
    try {
      await invoke("remove_mock_rule", { id });
      rules = rules.filter(r => r.id !== id);
    } catch (e) {
      console.error("Failed to remove mock rule:", e);
    }
  }

  async function importFromTraffic() {
    recordLoading = true;
    recordResult = null;
    recordError = null;
    try {
      const imported: MockRule[] = await invoke("mock_import_from_traffic", { limit: recordLimit });
      // Merge into local rules list (skip duplicates by id)
      const existingIds = new Set(rules.map(r => r.id));
      const fresh = imported.filter(r => !existingIds.has(r.id));
      rules = [...rules, ...fresh];
      recordResult = { count: fresh.length, paths: fresh.map(r => `${r.method} ${r.path}`) };
    } catch (e) {
      recordError = String(e);
    } finally {
      recordLoading = false;
    }
  }

  onMount(loadRules);
</script>

<div class="dashboard-section animate-fade-in">
  <div class="section-header">
    <h2>Mock Server</h2>
    <p class="section-desc">Create virtual endpoints that return templated responses. Supports path params (e.g. /users/:id) and wildcards (*).</p>
  </div>

  <div class="mock-form">
    <div class="form-row">
      <div class="config-field" style="flex: 0 0 100px;">
        <label for="mock-method">Method</label>
        <select id="mock-method" class="form-select" bind:value={newMethod}>
          <option>GET</option>
          <option>POST</option>
          <option>PUT</option>
          <option>PATCH</option>
          <option>DELETE</option>
        </select>
      </div>
      <div class="config-field flex-1">
        <label for="mock-path">Path</label>
        <input id="mock-path" type="text" class="form-input" bind:value={newPath} placeholder="/api/v1/users/:id" />
      </div>
      <div class="config-field" style="flex: 0 0 100px;">
        <label for="mock-status">Status</label>
        <input id="mock-status" type="number" class="form-input" bind:value={newStatus} />
      </div>
    </div>

    <div class="form-row">
      <div class="config-field flex-1">
        <label for="mock-body">Response Body (Supports Template Tags)</label>
        <textarea id="mock-body" class="form-textarea" bind:value={newBody} placeholder="Raw response body..."></textarea>
        <div class="help-text">Use <code>{`{{.Params.id}}`}</code> or <code>{`{{.Query.token}}`}</code> to inject dynamic data.</div>
      </div>
    </div>

    <div class="form-row">
      <div class="config-field flex-1">
        <label for="mock-content-type">Content Type</label>
        <input id="mock-content-type" type="text" class="form-input" bind:value={newContentType} />
      </div>
      <div class="config-field" style="flex: 0 0 120px;">
        <label for="mock-delay">Delay (ms)</label>
        <input id="mock-delay" type="number" min="0" class="form-input" bind:value={newDelayMs} placeholder="0" />
      </div>
      <button class="btn-primary" onclick={addRule}>Create Mock Endpoint</button>
    </div>
  </div>

  <div class="record-section">
    <div class="record-header">
      <span class="record-title">Record from Proxy Traffic</span>
      <span class="record-desc">Convert captured proxy entries into mock rules automatically.</span>
    </div>
    <div class="record-row">
      <div class="config-field" style="flex: 0 0 120px;">
        <label for="record-limit">Max entries</label>
        <input id="record-limit" type="number" min="1" max="200" class="form-input" bind:value={recordLimit} />
      </div>
      <button class="btn-record" onclick={importFromTraffic} disabled={recordLoading}>
        {recordLoading ? "Capturing…" : "⏺ Capture from Proxy"}
      </button>
    </div>
    {#if recordError}
      <div class="record-error">{recordError}</div>
    {/if}
    {#if recordResult}
      <div class="record-success">
        <strong>{recordResult.count}</strong> rule{recordResult.count !== 1 ? "s" : ""} imported.
        {#if recordResult.paths.length > 0}
          <ul class="record-paths">
            {#each recordResult.paths.slice(0, 8) as p}
              <li class="mono">{p}</li>
            {/each}
            {#if recordResult.paths.length > 8}
              <li class="text-muted">…and {recordResult.paths.length - 8} more</li>
            {/if}
          </ul>
        {/if}
      </div>
    {/if}
  </div>

  <div class="rules-list">
    {#if rules.length === 0}
      <div class="empty-state">No mock endpoints defined.</div>
    {:else}
      {#each rules as rule (rule.id)}
        <div class="rule-card">
          <div class="rule-header">
            <div class="rule-title">
              <span class="method-badge {rule.method.toLowerCase()}">{rule.method}</span>
              <span class="rule-path">{rule.path}</span>
            </div>
            <div class="rule-actions">
              <span class="status-badge">{rule.status_code}</span>
              {#if rule.headers["x-parallax-delay-ms"]}
                <span class="delay-badge">{rule.headers["x-parallax-delay-ms"]}ms</span>
              {/if}
              <button class="btn-icon-remove" onclick={() => removeRule(rule.id)}>×</button>
            </div>
          </div>
          <div class="rule-body-preview text-muted mono">
            {rule.body.length > 100 ? rule.body.slice(0, 100) + "..." : rule.body}
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .dashboard-section { max-width: 800px; }
  .section-header { margin-bottom: 24px; }
  .section-header h2 { font-size: 18px; font-weight: 700; margin-bottom: 4px; }
  .section-desc { font-size: 12px; color: var(--text-secondary); }

  .mock-form {
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    padding: 20px;
    margin-bottom: 30px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .form-row { display: flex; gap: 12px; align-items: flex-end; }
  .config-field { display: flex; flex-direction: column; gap: 6px; }
  .config-field label { font-size: 10px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; }
  .flex-1 { flex: 1; }

  .form-input, .form-select, .form-textarea {
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: 13px;
    padding: 8px 12px;
  }
  .form-input:focus, .form-select:focus, .form-textarea:focus {
    border-color: var(--accent-primary);
    outline: none;
  }
  .form-textarea { height: 80px; resize: vertical; font-family: var(--font-mono); }
  
  .help-text { font-size: 10px; color: var(--text-muted); margin-top: 4px; }
  .help-text code { color: var(--accent-primary); }

  .btn-primary {
    background: var(--accent-primary);
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: var(--radius-md);
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
    transition: var(--transition-fast);
  }
  .btn-primary:hover { filter: brightness(1.1); }

  .rules-list { display: flex; flex-direction: column; gap: 10px; }
  .rule-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: 12px 16px;
  }
  .rule-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
  .rule-title { display: flex; align-items: center; gap: 10px; }
  .rule-path { font-family: var(--font-mono); font-size: 13px; font-weight: 600; }
  
  .method-badge {
    font-size: 10px; font-weight: 800; padding: 2px 6px; border-radius: 4px;
    text-transform: uppercase;
  }
  .method-badge.get { background: rgba(97, 175, 239, 0.2); color: #61afef; }
  .method-badge.post { background: rgba(152, 195, 121, 0.2); color: #98c379; }
  .method-badge.put { background: rgba(209, 154, 102, 0.2); color: #d19a66; }
  .method-badge.delete { background: rgba(224, 108, 117, 0.2); color: #e06c75; }

  .rule-actions { display: flex; align-items: center; gap: 12px; }
  .status-badge { font-size: 11px; font-family: var(--font-mono); font-weight: 700; color: var(--text-secondary); }
  .delay-badge { font-size: 10px; font-family: var(--font-mono); color: var(--text-muted); background: var(--bg-elevated); border: 1px solid var(--border-default); border-radius: 4px; padding: 1px 5px; }
  
  .btn-icon-remove {
    background: transparent; border: none; color: var(--text-muted);
    font-size: 18px; cursor: pointer; padding: 0; line-height: 1;
  }
  .btn-icon-remove:hover { color: var(--color-error); }

  .rule-body-preview { font-size: 11px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .empty-state { text-align: center; padding: 40px; color: var(--text-muted); font-size: 13px; border: 1px dashed var(--border-default); border-radius: var(--radius-md); }

  .record-section {
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    padding: 16px 20px;
    margin-bottom: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .record-header { display: flex; align-items: baseline; gap: 12px; }
  .record-title { font-size: 13px; font-weight: 700; color: var(--text-primary); }
  .record-desc { font-size: 11px; color: var(--text-muted); }
  .record-row { display: flex; gap: 12px; align-items: flex-end; }
  .btn-record {
    background: transparent;
    border: 1px solid var(--accent-primary);
    color: var(--accent-primary);
    padding: 8px 16px;
    border-radius: var(--radius-md);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: var(--transition-fast);
    white-space: nowrap;
  }
  .btn-record:hover:not(:disabled) { background: var(--accent-primary); color: white; }
  .btn-record:disabled { opacity: 0.5; cursor: not-allowed; }
  .record-error { font-size: 12px; color: var(--color-error); }
  .record-success { font-size: 12px; color: var(--color-success, #98c379); }
  .record-paths { margin: 6px 0 0 16px; padding: 0; list-style: disc; display: flex; flex-direction: column; gap: 2px; }
  .record-paths li { font-size: 11px; color: var(--text-secondary); }
</style>
