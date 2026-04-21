<script lang="ts">
  import { activeRequest, activeScripts } from "../../stores/app.svelte";

  let { activeTab = $bindable() } = $props<{ activeTab: string }>();

  const TABS = ["Params", "Headers", "Body", "Auth", "Scripts"];

  function addHeader() {
    activeRequest.headers = { ...activeRequest.headers, "": "" };
  }

  function removeHeader(key: string) {
    const { [key]: _, ...rest } = activeRequest.headers;
    activeRequest.headers = rest;
  }

  let scriptTab = $state<"pre" | "tests">("pre");

  const BODY_TYPES = [
    { value: "none", label: "None" },
    { value: "json", label: "JSON" },
    { value: "form", label: "Form Data" },
    { value: "urlencoded", label: "URL Encoded" },
    { value: "raw", label: "Raw" },
    { value: "graphql", label: "GraphQL" },
  ];
</script>

<div class="request-panel pane">
  <!-- Tab Bar -->
  <div class="tab-bar">
    {#each TABS as tab}
      <button
        class="tab"
        class:active={activeTab.toLowerCase() === tab.toLowerCase()}
        onclick={() => (activeTab = tab.toLowerCase())}
      >
        {tab}
        {#if tab === "Headers"}
          {@const count = Object.keys(activeRequest.headers).filter(k => k).length}
          {#if count > 0}
            <span class="tab-count">{count}</span>
          {/if}
        {/if}
      </button>
    {/each}
  </div>

  <!-- Content area -->
  <div class="panel-content scroll-y">
    {#if activeTab === "params"}
      <div class="kv-editor">
        <div class="kv-header">
          <span>Key</span>
          <span>Value</span>
        </div>
        {#each Object.entries(activeRequest.params) as [key, val]}
          <div class="kv-row">
            <input class="kv-input mono" type="text" value={key} placeholder="key" />
            <input class="kv-input mono" type="text" value={val} placeholder="value" />
            <button class="kv-remove" onclick={() => {
              const { [key]: _, ...rest } = activeRequest.params;
              activeRequest.params = rest;
            }}>×</button>
          </div>
        {/each}
        <button class="add-row-btn" onclick={() => {
          activeRequest.params = { ...activeRequest.params, "": "" };
        }}>
          + Add parameter
        </button>
      </div>

    {:else if activeTab === "headers"}
      <div class="kv-editor">
        <div class="kv-header">
          <span>Key</span>
          <span>Value</span>
        </div>
        {#each Object.entries(activeRequest.headers) as [key, val]}
          <div class="kv-row">
            <input class="kv-input mono" type="text" value={key} placeholder="header name" />
            <input class="kv-input mono" type="text" value={val} placeholder="value" />
            <button class="kv-remove" onclick={() => removeHeader(key)}>×</button>
          </div>
        {/each}
        <button class="add-row-btn" onclick={addHeader}>
          + Add header
        </button>
      </div>

    {:else if activeTab === "body"}
      <div class="body-editor">
        <div class="body-type-selector">
          {#each BODY_TYPES as type}
            <button
              class="body-type-btn"
              class:active={activeRequest.bodyType === type.value}
              onclick={() => (activeRequest.bodyType = type.value as any)}
            >
              {type.label}
            </button>
          {/each}
        </div>

        {#if activeRequest.bodyType !== "none"}
          <textarea
            class="body-textarea mono"
            placeholder={activeRequest.bodyType === "json" ? '{\n  "key": "value"\n}' : "Request body..."}
            bind:value={activeRequest.bodyContent}
          ></textarea>
        {:else}
          <div class="no-body">
            <span>This request has no body</span>
          </div>
        {/if}
      </div>

    {:else if activeTab === "auth"}
      <div class="auth-section">
        <div class="auth-type-select">
          <label for="auth-type" class="field-label">Auth Type</label>
          <select id="auth-type" bind:value={activeRequest.auth.type}>
            <option value="none">None</option>
            <option value="bearer">Bearer Token</option>
            <option value="basic">Basic Auth</option>
            <option value="api_key">API Key</option>
            <option value="ecosystem_provider">Ecosystem Provider</option>
          </select>
        </div>

        {#if activeRequest.auth.type === "bearer"}
          <div class="auth-fields">
            <label for="auth-token" class="field-label">Token</label>
            <input id="auth-token" class="mono" type="text" placeholder="Bearer token..." bind:value={activeRequest.auth.token} />
          </div>

        {:else if activeRequest.auth.type === "basic"}
          <div class="auth-fields">
            <label for="auth-username" class="field-label">Username</label>
            <input id="auth-username" type="text" placeholder="Username" bind:value={activeRequest.auth.username} />
            <label for="auth-password" class="field-label mt">Password</label>
            <input id="auth-password" type="password" placeholder="Password" bind:value={activeRequest.auth.password} />
          </div>

        {:else if activeRequest.auth.type === "api_key"}
          <div class="auth-fields">
            <label for="auth-key-header" class="field-label">Header Name</label>
            <input id="auth-key-header" type="text" placeholder="X-API-Key" bind:value={activeRequest.auth.apiKeyHeader} />
            <label for="auth-key-value" class="field-label mt">Value</label>
            <input id="auth-key-value" class="mono" type="text" placeholder="API key value" bind:value={activeRequest.auth.apiKeyValue} />
          </div>

        {:else if activeRequest.auth.type === "ecosystem_provider"}
          <div class="auth-fields">
            <label for="auth-provider" class="field-label">Framework</label>
            <select id="auth-provider" bind:value={activeRequest.auth.provider}>
              <option value="frappe">Frappe / ERPNext</option>
              <option value="django">Django</option>
              <option value="laravel">Laravel</option>
              <option value="rails">Ruby on Rails</option>
              <option value="wordpress">WordPress</option>
              <option value="fastapi">FastAPI (OAuth2)</option>
              <option value="generic">Generic Bearer</option>
            </select>
            <p class="helper-text">
              Parallax will auto-handle CSRF tokens, session cookies, and auth headers for the selected framework.
            </p>
            {#if activeRequest.auth.providerSession}
              <div class="session-active">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <polyline points="20 6 9 17 4 12"/>
                </svg>
                Session active
              </div>
            {:else}
              <button class="btn-send" style="margin-top: 12px; width: 100%; justify-content: center;">
                Authenticate
              </button>
            {/if}
          </div>
        {/if}
      </div>

    {:else if activeTab === "scripts"}
      <div class="scripts-section">
        <div class="scripts-tabs">
          <button
            class="script-tab"
            class:active={scriptTab === "pre"}
            onclick={() => (scriptTab = "pre")}
          >Pre-request</button>
          <button
            class="script-tab"
            class:active={scriptTab === "tests"}
            onclick={() => (scriptTab = "tests")}
          >Tests</button>
        </div>

        {#if scriptTab === "pre"}
          <div class="script-editor-wrap">
            <div class="script-hint mono">// pm.environment.set("token", pm.response.json().token);</div>
            <textarea
              class="script-textarea mono"
              placeholder={"// Pre-request script\n// Runs before the request is sent\n// pm.environment.set('key', 'value');"}
              bind:value={activeScripts.preRequest}
            ></textarea>
          </div>
        {:else}
          <div class="script-editor-wrap">
            <div class="script-hint mono">// pm.test("Status is 200", () =&gt; pm.expect(pm.response.code).to.equal(200));</div>
            <textarea
              class="script-textarea mono"
              placeholder={"// Test script\n// Runs after the response is received\npm.test('Status is 200', () => {\n  pm.expect(pm.response.code).to.equal(200);\n});"}
              bind:value={activeScripts.tests}
            ></textarea>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .request-panel {
    flex: 1;
    min-width: 0;
  }

  .panel-content {
    flex: 1;
    padding: 0;
  }

  .tab-count {
    background: var(--accent-primary-dim);
    color: var(--accent-primary);
    font-size: 9px;
    padding: 0 4px;
    border-radius: 10px;
    margin-left: 3px;
  }

  /* KV Editor */
  .kv-editor { padding: 8px 12px; }

  .kv-header {
    display: grid;
    grid-template-columns: 1fr 1fr 20px;
    gap: 6px;
    padding: 4px 6px;
    font-size: 10px;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin-bottom: 4px;
  }

  .kv-row {
    display: grid;
    grid-template-columns: 1fr 1fr 20px;
    gap: 6px;
    margin-bottom: 4px;
    align-items: center;
  }

  .kv-input {
    height: 30px;
    padding: 0 8px;
    font-size: 12px;
    border-radius: var(--radius-sm);
  }

  .kv-remove {
    background: transparent;
    color: var(--text-muted);
    font-size: 14px;
    width: 20px;
    height: 20px;
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: var(--transition-fast);
  }
  .kv-remove:hover { background: var(--color-error-dim); color: var(--color-error); }

  .add-row-btn {
    background: transparent;
    color: var(--accent-primary);
    font-size: 12px;
    padding: 6px;
    text-align: left;
    transition: var(--transition-fast);
  }
  .add-row-btn:hover { color: #9185ff; }

  /* Body Editor */
  .body-editor { display: flex; flex-direction: column; height: 100%; }

  .body-type-selector {
    display: flex;
    gap: 2px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .body-type-btn {
    padding: 4px 10px;
    font-size: 11px;
    color: var(--text-secondary);
    background: transparent;
    border-radius: var(--radius-sm);
    transition: var(--transition-fast);
  }
  .body-type-btn:hover { background: var(--bg-elevated); color: var(--text-primary); }
  .body-type-btn.active { background: var(--accent-primary-dim); color: var(--accent-primary); }

  .body-textarea {
    flex: 1;
    resize: none;
    border: none;
    border-radius: 0;
    padding: 12px;
    font-size: 12px;
    line-height: 1.7;
    background: var(--bg-base);
    color: var(--text-primary);
    height: 100%;
  }

  .no-body {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--text-muted);
    font-size: 12px;
  }

  /* Auth */
  .auth-section { padding: 16px; display: flex; flex-direction: column; gap: 10px; }
  .auth-type-select, .auth-fields { display: flex; flex-direction: column; gap: 6px; }

  .auth-fields { margin-top: 6px; }

  .auth-fields input, .auth-fields select {
    height: 32px;
    padding: 0 10px;
    width: 100%;
  }

  .auth-section select {
    height: 32px;
    padding: 0 10px;
    width: 100%;
  }

  .field-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .mt { margin-top: 8px; }

  .helper-text {
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.5;
    margin-top: 4px;
  }

  .session-active {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 10px;
    font-size: 12px;
    color: var(--color-success);
    font-weight: 500;
  }

  .scripts-section { display: flex; flex-direction: column; height: 100%; }

  .scripts-tabs {
    display: flex;
    gap: 2px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .script-tab {
    padding: 3px 12px;
    font-size: 11px;
    color: var(--text-secondary);
    background: transparent;
    border-radius: var(--radius-sm);
    transition: var(--transition-fast);
  }
  .script-tab:hover { background: var(--bg-elevated); color: var(--text-primary); }
  .script-tab.active { background: var(--accent-primary-dim); color: var(--accent-primary); }

  .script-editor-wrap { display: flex; flex-direction: column; flex: 1; }

  .script-hint {
    padding: 6px 12px;
    font-size: 10px;
    color: var(--text-muted);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .script-textarea {
    flex: 1;
    resize: none;
    border: none;
    border-radius: 0;
    padding: 10px 12px;
    font-size: 12px;
    line-height: 1.7;
    background: var(--bg-base);
    color: var(--text-primary);
    height: 100%;
  }
</style>
