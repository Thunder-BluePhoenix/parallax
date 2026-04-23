<script lang="ts">
  import { aiConfig, saveAIConfig, type AIProvider } from "../../stores/ai.svelte";
  import Logo from "../Common/Logo.svelte";

  const providers: { id: AIProvider; name: string; local: boolean }[] = [
    { id: "ollama", name: "Ollama", local: true },
    { id: "openai", name: "OpenAI", local: false },
    { id: "anthropic", name: "Anthropic", local: false },
    { id: "gemini", name: "Google Gemini", local: false },
    { id: "custom", name: "Custom (OpenAI-Compatible)", local: false },
  ];

  function handleSave() {
    saveAIConfig();
    alert("AI settings saved locally.");
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
    <div class="config-section">
      <h3>Provider Configuration</h3>
      
      <div class="field">
        <label>Active Provider</label>
        <select class="form-select" bind:value={aiConfig.provider}>
          {#each providers as p}
            <option value={p.id}>{p.name} {p.local ? '(Local)' : ''}</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label>Model Name</label>
        <input type="text" class="form-input" bind:value={aiConfig.model} placeholder={aiConfig.provider === 'ollama' ? 'llama3' : 'gpt-4o'} />
      </div>

      {#if aiConfig.provider !== 'ollama'}
        <div class="field">
          <label>API Key</label>
          <input type="password" class="form-input" bind:value={aiConfig.apiKey} placeholder="sk-..." />
        </div>
      {/if}

      {#if aiConfig.provider === 'ollama' || aiConfig.provider === 'custom'}
        <div class="field">
          <label>Base URL</label>
          <input type="text" class="form-input" bind:value={aiConfig.baseUrl} placeholder="http://localhost:11434" />
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
        <label>Temperature ({aiConfig.temperature})</label>
        <input type="range" min="0" max="1" step="0.1" bind:value={aiConfig.temperature} />
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

  <div class="actions">
    <button class="btn-save" onclick={handleSave}>Save AI Configuration</button>
  </div>
</div>

<style>
  .ai-settings { padding: 24px; max-width: 800px; margin: 0 auto; }
  .settings-header { margin-bottom: 32px; border-bottom: 1px solid var(--border-default); padding-bottom: 20px; }
  .settings-header h2 { font-size: 20px; font-weight: 700; margin-bottom: 6px; }
  .desc { font-size: 13px; color: var(--text-secondary); }

  .settings-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 40px; margin-bottom: 32px; }
  .config-section h3 { font-size: 12px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); margin-bottom: 16px; letter-spacing: 0.05em; }

  .field { display: flex; flex-direction: column; gap: 8px; margin-bottom: 20px; }
  .field label { font-size: 12px; font-weight: 600; color: var(--text-primary); }
  
  .form-input, .form-select {
    height: 36px; padding: 0 12px; background: var(--bg-input); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); color: var(--text-primary); font-size: 13px;
  }
  .form-input:focus, .form-select:focus { border-color: var(--accent-primary); outline: none; }

  .field.checkbox { flex-direction: row; align-items: flex-start; gap: 12px; padding: 12px; background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: var(--radius-lg); }
  .field.checkbox input { width: 16px; height: 16px; margin-top: 2px; }
  .field.checkbox label { display: flex; flex-direction: column; gap: 2px; font-weight: normal; }
  .field.checkbox label span { font-size: 11px; color: var(--text-secondary); line-height: 1.4; }

  .range-labels { display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); }

  .info-box {
    margin-top: 24px; padding: 16px; background: rgba(56, 139, 253, 0.05); border: 1px solid rgba(56, 139, 253, 0.2); border-radius: var(--radius-lg);
  }
  .info-box h4 { font-size: 12px; font-weight: 700; color: var(--accent-primary); margin-bottom: 8px; }
  .info-box p { font-size: 11px; color: var(--text-secondary); line-height: 1.5; }

  .actions { display: flex; justify-content: flex-end; padding-top: 20px; border-top: 1px solid var(--border-default); }
  .btn-save {
    background: var(--accent-primary); color: white; border: none; padding: 10px 24px; border-radius: var(--radius-md); font-weight: 600; font-size: 13px; cursor: pointer; transition: var(--transition-fast);
  }
  .btn-save:hover { filter: brightness(1.1); }
</style>
