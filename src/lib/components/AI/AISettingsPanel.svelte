<script lang="ts">
  import { aiConfig, saveAIConfig, createCollectionFromDescription, aiStatus, type AIProvider } from "../../stores/ai.svelte";
  import Logo from "../Common/Logo.svelte";

  const providers: { id: AIProvider; name: string; local: boolean }[] = [
    { id: "ollama",    name: "Ollama",                     local: true  },
    { id: "openai",   name: "OpenAI",                     local: false },
    { id: "anthropic",name: "Anthropic (Claude)",          local: false },
    { id: "gemini",   name: "Google Gemini",               local: false },
    { id: "custom",   name: "Custom (OpenAI-Compatible)",  local: false },
  ];

  // MCP state (stored in localStorage)
  let mcpEnabled = $state(localStorage.getItem("parallax:mcp_enabled") === "true");
  let mcpToken   = $state(localStorage.getItem("parallax:mcp_token") ?? "");

  // Collection creator
  let collectionDesc    = $state("");
  let collectionResult  = $state("");
  let collectionError   = $state("");

  function handleSave() {
    saveAIConfig();
    localStorage.setItem("parallax:mcp_enabled", String(mcpEnabled));
    localStorage.setItem("parallax:mcp_token", mcpToken);
    alert("AI settings saved.");
  }

  async function handleCreateCollection() {
    if (!collectionDesc.trim()) return;
    collectionResult = "";
    collectionError  = "";
    try {
      collectionResult = await createCollectionFromDescription(collectionDesc.trim());
    } catch (e: any) {
      collectionError = e.toString();
    }
  }

  function copyMcpUrl() {
    navigator.clipboard.writeText(`http://localhost:7676/mcp/sse`);
  }
</script>

<div class="ai-settings animate-fade-in">
  <div class="settings-header">
    <div style="display:flex; align-items:center; gap:10px;">
      <Logo size={24} />
      <h2>BYO-AI Integration</h2>
    </div>
    <p class="desc">Eliminate the credit wall. Plug in your own API keys or run fully local models via Ollama.</p>
  </div>

  <div class="settings-grid">
    <!-- Provider Configuration -->
    <div class="config-section">
      <h3>Provider Configuration</h3>

      <div class="field">
        <label for="ai-provider">Active Provider</label>
        <select id="ai-provider" class="form-select" bind:value={aiConfig.provider}>
          {#each providers as p}
            <option value={p.id}>{p.name} {p.local ? '(Local)' : ''}</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label for="ai-model">Model Name</label>
        <input id="ai-model" type="text" class="form-input" bind:value={aiConfig.model}
          placeholder={aiConfig.provider === 'ollama' ? 'llama3' : aiConfig.provider === 'anthropic' ? 'claude-3-5-sonnet-20241022' : aiConfig.provider === 'gemini' ? 'gemini-2.0-flash' : 'gpt-4o'} />
      </div>

      {#if aiConfig.provider !== 'ollama'}
        <div class="field">
          <label for="ai-key">API Key</label>
          <input id="ai-key" type="password" class="form-input" bind:value={aiConfig.apiKey} placeholder="sk-..." />
        </div>
      {/if}

      {#if aiConfig.provider === 'ollama' || aiConfig.provider === 'custom'}
        <div class="field">
          <label for="ai-base-url">Base URL</label>
          <input id="ai-base-url" type="text" class="form-input" bind:value={aiConfig.baseUrl} placeholder="http://localhost:11434" />
        </div>
      {/if}
    </div>

    <!-- Safety & Advanced -->
    <div class="config-section">
      <h3>Safety & Advanced</h3>

      <div class="field checkbox">
        <input type="checkbox" id="air-gap" bind:checked={aiConfig.airGapMode} />
        <label for="air-gap">
          <strong>Air-Gap Mode</strong>
          <span>Fully disables all cloud AI. Only Ollama (local) will work.</span>
        </label>
      </div>

      <div class="field">
        <label for="ai-temp">Temperature ({aiConfig.temperature})</label>
        <input id="ai-temp" type="range" min="0" max="1" step="0.1" bind:value={aiConfig.temperature} />
        <div class="range-labels"><span>Precise</span><span>Creative</span></div>
      </div>

      <div class="info-box">
        <h4>Data Privacy</h4>
        <p>Parallax never sends request/response data to any AI provider without an explicit user action. No background telemetry.</p>
      </div>
    </div>
  </div>

  <!-- MCP Server Section -->
  <div class="mcp-section">
    <div class="mcp-header">
      <div>
        <h3>MCP Server <span class="badge-new">NEW</span></h3>
        <p class="mcp-desc">Let Claude Desktop, Cursor, or custom agents drive your API collections as tools.</p>
      </div>
      <label class="toggle" title="Enable MCP Server">
        <input type="checkbox" bind:checked={mcpEnabled} />
        <span class="toggle-track"><span class="toggle-thumb"></span></span>
      </label>
    </div>

    {#if mcpEnabled}
      <div class="mcp-config">
        <div class="mcp-url-row">
          <code class="mcp-url">http://localhost:7676/mcp/sse</code>
          <button class="btn-copy" onclick={copyMcpUrl}>Copy URL</button>
        </div>
        <div class="field" style="margin-top:12px;">
          <label for="mcp-token">Auth Token <span class="optional">(optional)</span></label>
          <input id="mcp-token" type="password" class="form-input" bind:value={mcpToken} placeholder="Leave empty to disable auth" />
        </div>
        <div class="mcp-tools-list">
          <p class="tools-label">Exposed Tools:</p>
          <code>parallax.list_collections</code>
          <code>parallax.get_traffic</code>
          <code>parallax.execute_request</code>
        </div>
        <p class="mcp-note">⚠ Restart the app or run with <code>--mcp --mcp-token &lt;token&gt;</code> flags for changes to take effect.</p>
      </div>
    {/if}
  </div>

  <!-- AI Collection Creator -->
  <div class="creator-section">
    <h3>AI Collection Creator</h3>
    <p class="creator-desc">Describe an API in plain English and Parallax generates a full collection with folders, auth, and test assertions.</p>
    <textarea id="collection-desc" class="form-textarea" rows="3"
      bind:value={collectionDesc}
      placeholder="e.g. Create a REST API collection for a blog with posts, comments, and JWT auth. Include CRUD for each resource."
    ></textarea>
    <div class="creator-actions">
      <button class="btn-create" onclick={handleCreateCollection} disabled={aiStatus.busy || !collectionDesc.trim()}>
        {aiStatus.busy ? 'Generating…' : '✨ Generate Collection'}
      </button>
    </div>
    {#if collectionError}
      <div class="creator-error">{collectionError}</div>
    {/if}
    {#if collectionResult}
      <div class="creator-result">
        <div class="result-header">
          <span>Generated YAML — copy and save as a <code>.yaml</code> collection file.</span>
          <button class="btn-copy" onclick={() => navigator.clipboard.writeText(collectionResult)}>Copy</button>
        </div>
        <pre class="result-code">{collectionResult}</pre>
      </div>
    {/if}
  </div>

  <div class="actions">
    <button class="btn-save" onclick={handleSave}>Save AI Configuration</button>
  </div>
</div>

<style>
  .ai-settings { padding: 24px; max-width: 860px; margin: 0 auto; }
  .settings-header { margin-bottom: 32px; border-bottom: 1px solid var(--border-default); padding-bottom: 20px; }
  .settings-header h2 { font-size: 20px; font-weight: 700; margin-bottom: 6px; }
  .desc { font-size: 13px; color: var(--text-secondary); }

  .settings-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 40px; margin-bottom: 32px; }
  .config-section h3 { font-size: 12px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); margin-bottom: 16px; letter-spacing: 0.05em; }

  .field { display: flex; flex-direction: column; gap: 8px; margin-bottom: 20px; }
  .field label { font-size: 12px; font-weight: 600; color: var(--text-primary); }

  .form-input, .form-select, .form-textarea {
    background: var(--bg-input); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); color: var(--text-primary); font-size: 13px;
  }
  .form-input, .form-select { height: 36px; padding: 0 12px; }
  .form-input:focus, .form-select:focus { border-color: var(--accent-primary); outline: none; }
  .form-textarea { padding: 8px 12px; resize: vertical; font-family: var(--font-mono); font-size: 12px; }
  .form-textarea:focus { border-color: var(--accent-primary); outline: none; }

  .field.checkbox { flex-direction: row; align-items: flex-start; gap: 12px; padding: 12px; background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: var(--radius-lg); }
  .field.checkbox input { width: 16px; height: 16px; margin-top: 2px; }
  .field.checkbox label { display: flex; flex-direction: column; gap: 2px; font-weight: normal; }
  .field.checkbox label span { font-size: 11px; color: var(--text-secondary); line-height: 1.4; }

  .range-labels { display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); }

  .info-box { margin-top: 24px; padding: 16px; background: rgba(56,139,253,0.05); border: 1px solid rgba(56,139,253,0.2); border-radius: var(--radius-lg); }
  .info-box h4 { font-size: 12px; font-weight: 700; color: var(--accent-primary); margin-bottom: 8px; }
  .info-box p { font-size: 11px; color: var(--text-secondary); line-height: 1.5; }

  /* MCP Section */
  .mcp-section {
    margin-bottom: 32px; padding: 20px;
    background: var(--bg-surface); border: 1px solid var(--border-default); border-radius: var(--radius-lg);
  }
  .mcp-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 12px; }
  .mcp-header h3 { font-size: 14px; font-weight: 700; margin-bottom: 4px; display: flex; align-items: center; gap: 8px; }
  .badge-new { font-size: 9px; font-weight: 800; background: var(--accent-primary); color: white; padding: 2px 5px; border-radius: 4px; letter-spacing: 0.05em; }
  .mcp-desc { font-size: 12px; color: var(--text-secondary); }

  /* Toggle */
  .toggle { cursor: pointer; }
  .toggle input { display: none; }
  .toggle-track { display: flex; width: 40px; height: 22px; border-radius: 11px; background: var(--bg-overlay); border: 1px solid var(--border-default); position: relative; transition: var(--transition-fast); }
  .toggle input:checked ~ .toggle-track { background: var(--accent-primary); border-color: var(--accent-primary); }
  .toggle-thumb { width: 16px; height: 16px; border-radius: 50%; background: white; position: absolute; top: 2px; left: 2px; transition: var(--transition-fast); }
  .toggle input:checked ~ .toggle-track .toggle-thumb { left: 20px; }

  .mcp-config { padding-top: 12px; border-top: 1px solid var(--border-subtle); }
  .mcp-url-row { display: flex; align-items: center; gap: 10px; }
  .mcp-url { font-size: 12px; background: var(--bg-overlay); padding: 5px 10px; border-radius: var(--radius-sm); color: var(--accent-primary); }
  .mcp-tools-list { margin-top: 14px; display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }
  .mcp-tools-list p { font-size: 11px; font-weight: 600; color: var(--text-muted); margin-right: 4px; }
  .mcp-tools-list code { font-size: 10px; background: var(--bg-overlay); padding: 2px 7px; border-radius: 3px; color: var(--text-secondary); }
  .mcp-note { margin-top: 12px; font-size: 11px; color: var(--text-muted); }
  .mcp-note code { color: var(--accent-primary); }

  .optional { font-size: 10px; color: var(--text-muted); font-weight: normal; }

  /* Collection Creator */
  .creator-section {
    margin-bottom: 32px; padding: 20px;
    background: var(--bg-surface); border: 1px solid var(--border-default); border-radius: var(--radius-lg);
  }
  .creator-section h3 { font-size: 14px; font-weight: 700; margin-bottom: 6px; }
  .creator-desc { font-size: 12px; color: var(--text-secondary); margin-bottom: 14px; }
  .creator-actions { display: flex; justify-content: flex-end; margin-top: 10px; }
  .creator-error { margin-top: 10px; padding: 10px 12px; background: var(--color-error-dim); color: var(--color-error); font-size: 12px; border-radius: var(--radius-md); }

  .creator-result { margin-top: 14px; border: 1px solid var(--border-default); border-radius: var(--radius-md); overflow: hidden; }
  .result-header { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: var(--bg-elevated); font-size: 11px; color: var(--text-secondary); border-bottom: 1px solid var(--border-subtle); }
  .result-header code { color: var(--accent-primary); }
  .result-code { padding: 12px; font-size: 11px; font-family: var(--font-mono); color: var(--text-secondary); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; margin: 0; }

  /* Shared */
  .btn-copy { font-size: 11px; padding: 3px 10px; background: var(--bg-elevated); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); color: var(--text-secondary); cursor: pointer; transition: var(--transition-fast); }
  .btn-copy:hover { color: var(--text-primary); }
  .btn-create { padding: 8px 20px; background: var(--accent-primary); color: white; border: none; border-radius: var(--radius-md); font-weight: 600; font-size: 13px; cursor: pointer; transition: var(--transition-fast); }
  .btn-create:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-create:disabled { opacity: 0.5; cursor: not-allowed; }

  .actions { display: flex; justify-content: flex-end; padding-top: 20px; border-top: 1px solid var(--border-default); }
  .btn-save { background: var(--accent-primary); color: white; border: none; padding: 10px 24px; border-radius: var(--radius-md); font-weight: 600; font-size: 13px; cursor: pointer; transition: var(--transition-fast); }
  .btn-save:hover { filter: brightness(1.1); }
</style>


<div class="ai-settings animate-fade-in">
  <div class="settings-header">
    <div style="display:flex; align-items:center; gap:10px;">
      <Logo size={24} />
      <h2>BYO-AI Integration</h2>
    </div>
    <p class="desc">Eliminate the credit wall. Plug in your own API keys or run fully local models via Ollama.</p>
  </div>

  <div class="settings-grid">
    <div class="config-section">
      <h3>Provider Configuration</h3>
      
      <div class="field">
        <label for="ai-provider">Active Provider</label>
        <select id="ai-provider" class="form-select" bind:value={aiConfig.provider}>
          {#each providers as p}
            <option value={p.id}>{p.name} {p.local ? '(Local)' : ''}</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label for="ai-model">Model Name</label>
        <input id="ai-model" type="text" class="form-input" bind:value={aiConfig.model} placeholder={aiConfig.provider === 'ollama' ? 'llama3' : 'gpt-4o'} />
      </div>

      {#if aiConfig.provider !== 'ollama'}
        <div class="field">
          <label for="ai-apikey">API Key</label>
          <input id="ai-apikey" type="password" class="form-input" bind:value={aiConfig.apiKey} placeholder="sk-..." />
        </div>
      {/if}

      {#if aiConfig.provider === 'ollama' || aiConfig.provider === 'custom'}
        <div class="field">
          <label for="ai-baseurl">Base URL</label>
          <input id="ai-baseurl" type="text" class="form-input" bind:value={aiConfig.baseUrl} placeholder="http://localhost:11434" />
        </div>
      {/if}
    </div>

    <div class="config-section">
      <h3>Safety & Advanced</h3>
      
      <div class="field checkbox">
        <input type="checkbox" id="air-gap" bind:checked={aiConfig.airGapMode} />
        <label for="air-gap">
          <strong>Air-Gap Mode</strong>
          <span>Fully disables all network-based AI features. Only local providers (Ollama) will work.</span>
        </label>
      </div>

      <div class="field">
        <label for="ai-temp">Temperature ({aiConfig.temperature})</label>
        <input id="ai-temp" type="range" min="0" max="1" step="0.1" bind:value={aiConfig.temperature} />
        <div class="range-labels">
          <span>Precise</span>
          <span>Creative</span>
        </div>
      </div>

      <div class="info-box">
        <h4>Data Privacy Note</h4>
        <p>Parallax never sends your request/response data to an AI provider unless you explicitly click an AI action button. No background telemetry or training.</p>
      </div>
    </div>
  </div>
  </div>
