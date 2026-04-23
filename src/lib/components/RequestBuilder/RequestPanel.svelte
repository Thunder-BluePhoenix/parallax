<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { activeRequest, activeScripts } from "../../stores/app.svelte";
  import { generateScript, aiStatus } from "../../stores/ai.svelte";

  let aiPrompt = $state("");
  let aiScriptError = $state("");

  async function handleAIScript() {
    if (!aiPrompt.trim()) return;
    aiScriptError = "";
    try {
      const type = scriptTab === "pre" ? "pre-request" : "test";
      const result = await generateScript(type, aiPrompt);
      if (scriptTab === "pre") {
        activeScripts.preRequest = result;
      } else {
        activeScripts.tests = result;
      }
      aiPrompt = "";
    } catch (e: any) {
      aiScriptError = e.toString();
    }
  }

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

  // ── GraphQL schema introspection ──────────────────────────────
  let schemaTypes = $state<Array<{ name: string; kind: string }>>([]);
  let schemaLoading = $state(false);
  let schemaError = $state("");
  let schemaVisible = $state(false);

  const INTROSPECTION_QUERY = `{
  __schema {
    queryType { name }
    types {
      name
      kind
      description
      fields { name type { name kind ofType { name kind } } }
    }
  }
}`;

  async function fetchSchema() {
    if (!activeRequest.url) return;
    schemaLoading = true;
    schemaError = "";
    try {
      const result = await invoke<any>("send_request", {
        request: {
          id: "gql-introspect",
          name: "GraphQL Introspection",
          method: "POST",
          url: activeRequest.url,
          headers: { ...activeRequest.headers, "Content-Type": "application/json" },
          params: null,
          body: {
            type: "json",
            content: { query: INTROSPECTION_QUERY },
            raw: JSON.stringify({ query: INTROSPECTION_QUERY }),
          },
          auth: null,
          timeout_ms: 15000,
          follow_redirects: true,
        },
        environment: {},
      });
      const data = result.body?.json ?? JSON.parse(result.body?.raw ?? "{}");
      const types: any[] = data?.data?.__schema?.types ?? [];
      schemaTypes = types.filter((t) => !t.name.startsWith("__"));
      schemaVisible = schemaTypes.length > 0;
      if (!schemaTypes.length) schemaError = "No types returned. Is this a GraphQL endpoint?";
    } catch (e: any) {
      schemaError = String(e);
    } finally {
      schemaLoading = false;
    }
  }
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

        {#if activeRequest.bodyType === "none"}
          <div class="no-body">
            <span>This request has no body</span>
          </div>
        {:else if activeRequest.bodyType === "graphql"}
          <div class="gql-toolbar">
            <span class="gql-hint mono">Format: {"{"}  "query": "...", "variables": {"{}"} {"}"}</span>
            <button
              class="gql-schema-btn"
              onclick={fetchSchema}
              disabled={schemaLoading || !activeRequest.url}
            >
              {#if schemaLoading}
                <span class="spinner-xs"></span> Loading…
              {:else}
                Fetch Schema
              {/if}
            </button>
          </div>
          <textarea
            class="body-textarea mono"
            placeholder={'{\n  "query": "{ __typename }",\n  "variables": {}\n}'}
            bind:value={activeRequest.bodyContent}
          ></textarea>
          {#if schemaError}
            <div class="gql-error">{schemaError}</div>
          {/if}
          {#if schemaVisible && schemaTypes.length > 0}
            <div class="gql-schema-panel">
              <div class="gql-schema-header">
                Schema — {schemaTypes.length} types
                <button class="gql-close" onclick={() => (schemaVisible = false)}>×</button>
              </div>
              <div class="gql-schema-list scroll-y">
                {#each schemaTypes as t}
                  <div class="gql-type-row">
                    <span class="gql-kind gql-kind-{t.kind.toLowerCase()}">{t.kind}</span>
                    <span class="gql-type-name mono">{t.name}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        {:else}
          <textarea
            class="body-textarea mono"
            placeholder={activeRequest.bodyType === "json" ? '{\n  "key": "value"\n}' : "Request body..."}
            bind:value={activeRequest.bodyContent}
          ></textarea>
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
            <label for="auth-key-loc" class="field-label">Add to</label>
            <select id="auth-key-loc" bind:value={activeRequest.auth.apiKeyLocation}>
              <option value="header">Header</option>
              <option value="query">Query Param</option>
            </select>
            <label for="auth-key-header" class="field-label mt">
              {activeRequest.auth.apiKeyLocation === "query" ? "Param Name" : "Header Name"}
            </label>
            <input
              id="auth-key-header"
              type="text"
              placeholder={activeRequest.auth.apiKeyLocation === "query" ? "api_key" : "X-API-Key"}
              bind:value={activeRequest.auth.apiKeyHeader}
            />
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
            
            <label for="auth-base-url" class="field-label mt">Base/Login URL</label>
            <input id="auth-base-url" class="mono" type="text" placeholder="http://localhost:8000" bind:value={activeRequest.auth.authUrl} />
            
            <label for="auth-ecosystem-user" class="field-label mt">Username/Email</label>
            <input id="auth-ecosystem-user" type="text" placeholder="user@example.com" bind:value={activeRequest.auth.username} />
            
            <label for="auth-ecosystem-pass" class="field-label mt">Password/Secret</label>
            <input id="auth-ecosystem-pass" type="password" placeholder="Password" bind:value={activeRequest.auth.password} />

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
              <button class="btn" style="margin-top: 8px; width: 100%; justify-content: center;" onclick={() => activeRequest.auth.providerSession = null}>
                Clear Session
              </button>
            {:else}
              <button 
                class="btn-send" 
                style="margin-top: 12px; width: 100%; justify-content: center;"
                onclick={async () => {
                  try {
                    const session = await invoke("perform_auth", {
                      input: {
                        provider: activeRequest.auth.provider,
                        credentials: {
                          base_url: activeRequest.auth.authUrl || "http://localhost:8000",
                          username: activeRequest.auth.username,
                          password: activeRequest.auth.password,
                        }
                      }
                    });
                    activeRequest.auth.providerSession = session;
                  } catch (e) {
                    alert("Authentication failed: " + e);
                  }
                }}
              >
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

        <div class="ai-script-bar">
          <input
            class="ai-script-input"
            placeholder="Describe what the script should do… (AI)"
            bind:value={aiPrompt}
            onkeydown={(e) => e.key === "Enter" && handleAIScript()}
          />
          <button
            class="ai-script-btn"
            onclick={handleAIScript}
            disabled={aiStatus.busy || !aiPrompt.trim()}
            title="Generate script with AI"
          >
            {aiStatus.busy ? "…" : "✨ AI"}
          </button>
        </div>
        {#if aiScriptError}
          <div class="ai-script-error">{aiScriptError}</div>
        {/if}

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

  .ai-script-bar {
    display: flex; gap: 6px; padding: 6px 10px;
    background: var(--bg-surface); border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }
  .ai-script-input {
    flex: 1; height: 26px; padding: 0 8px; font-size: 11px;
    background: var(--bg-input); border: 1px solid var(--border-default);
    border-radius: var(--radius-sm); color: var(--text-primary);
  }
  .ai-script-input:focus { border-color: var(--accent-primary); outline: none; }
  .ai-script-btn {
    height: 26px; padding: 0 10px; font-size: 11px; font-weight: 600;
    background: var(--accent-primary-dim); color: var(--accent-primary);
    border: 1px solid var(--accent-primary); border-radius: var(--radius-sm);
    cursor: pointer; white-space: nowrap; transition: var(--transition-fast);
  }
  .ai-script-btn:hover:not(:disabled) { background: var(--accent-primary); color: white; }
  .ai-script-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .ai-script-error {
    padding: 4px 10px; font-size: 10px; color: var(--color-error);
    background: var(--color-error-dim); flex-shrink: 0;
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

  /* GraphQL */
  .gql-toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
    background: var(--bg-surface);
  }
  .gql-hint {
    flex: 1;
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .gql-schema-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    font-size: 11px;
    background: var(--accent-primary-dim);
    color: var(--accent-primary);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
    transition: var(--transition-fast);
  }
  .gql-schema-btn:hover:not(:disabled) { background: rgba(99,85,255,0.25); }
  .gql-schema-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .spinner-xs {
    width: 10px; height: 10px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    display: inline-block;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  .gql-error {
    padding: 6px 12px;
    font-size: 11px;
    color: var(--color-error);
    background: var(--color-error-dim);
    flex-shrink: 0;
  }
  .gql-schema-panel {
    border-top: 1px solid var(--border-default);
    background: var(--bg-surface);
    flex-shrink: 0;
    max-height: 220px;
    display: flex;
    flex-direction: column;
  }
  .gql-schema-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }
  .gql-close {
    background: transparent;
    color: var(--text-muted);
    font-size: 14px;
    line-height: 1;
    padding: 0 2px;
  }
  .gql-close:hover { color: var(--text-primary); }
  .gql-schema-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .gql-type-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 12px;
    font-size: 11px;
  }
  .gql-type-row:hover { background: var(--bg-elevated); }
  .gql-kind {
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.05em;
    padding: 1px 5px;
    border-radius: 3px;
    min-width: 60px;
    text-align: center;
    flex-shrink: 0;
  }
  .gql-kind-object    { background: rgba(63,185,80,0.15);  color: #3fb950; }
  .gql-kind-scalar    { background: rgba(88,166,255,0.15); color: #58a6ff; }
  .gql-kind-enum      { background: rgba(227,179,65,0.15); color: #e3b341; }
  .gql-kind-input_object { background: rgba(124,110,255,0.15); color: #7c6eff; }
  .gql-kind-interface { background: rgba(54,217,196,0.15); color: #36d9c4; }
  .gql-kind-union     { background: rgba(255,127,80,0.15); color: #ff7f50; }
  .gql-type-name { color: var(--text-primary); }
</style>
