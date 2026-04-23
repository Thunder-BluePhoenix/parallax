<script lang="ts">
  let { spec, error } = $props<{ spec: any; error: string }>();

  // Helper to extract paths array from the OpenAPI spec object
  let paths = $derived.by(() => {
    if (!spec || !spec.paths) return [];
    return Object.entries(spec.paths).flatMap(([path, methods]: [string, any]) => {
      return Object.entries(methods).map(([method, details]: [string, any]) => ({
        path,
        method: method.toUpperCase(),
        summary: details.summary || "",
        description: details.description || "",
      }));
    });
  });

  function getMethodColor(method: string) {
    switch (method) {
      case "GET": return "var(--color-success)";
      case "POST": return "var(--color-warning)";
      case "PUT": return "var(--color-info)";
      case "DELETE": return "var(--color-error)";
      case "PATCH": return "var(--color-warning)";
      default: return "var(--text-muted)";
    }
  }
</script>

<div class="api-preview">
  {#if error}
    <div class="error-panel">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      {error}
    </div>
  {:else if spec}
    <div class="spec-header">
      <h1 class="spec-title">{spec.info?.title || "Untitled API"}</h1>
      <div class="spec-meta">
        <span class="spec-version">v{spec.info?.version || "1.0.0"}</span>
        {#if spec.openapi}
          <span class="spec-badge">OAS {spec.openapi}</span>
        {/if}
      </div>
      {#if spec.info?.description}
        <p class="spec-desc">{spec.info.description}</p>
      {/if}
    </div>

    {#if spec.servers && spec.servers.length > 0}
      <div class="servers-section">
        <h3 class="section-title">Servers</h3>
        <div class="server-list">
          {#each spec.servers as s}
            <div class="server-item">
              <span class="mono">{s.url}</span>
              {#if s.description}
                <span class="server-desc text-muted">{s.description}</span>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <div class="paths-section">
      <h3 class="section-title">Endpoints</h3>
      <div class="endpoint-list">
        {#each paths as p}
          <div class="endpoint-card">
            <div class="endpoint-header">
              <span class="method-badge" style="color: {getMethodColor(p.method)}">{p.method}</span>
              <span class="path-text mono">{p.path}</span>
            </div>
            {#if p.summary || p.description}
              <div class="endpoint-desc text-muted">
                <strong>{p.summary}</strong> {p.description ? '— ' + p.description : ''}
              </div>
            {/if}
          </div>
        {/each}
        {#if paths.length === 0}
          <div class="text-muted" style="font-size: 12px;">No endpoints defined in paths.</div>
        {/if}
      </div>
    </div>
  {:else}
    <div class="empty-state text-muted">
      Start typing YAML on the left to see the API preview.
    </div>
  {/if}
</div>

<style>
  .api-preview {
    padding: 24px;
    height: 100%;
    overflow-y: auto;
    background: var(--bg-surface);
  }

  .error-panel {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 12px;
    background: var(--color-error-dim);
    color: var(--color-error);
    border-radius: var(--radius-sm);
    font-size: 12px;
    line-height: 1.5;
    font-family: var(--font-mono);
  }

  .spec-header {
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .spec-title {
    margin: 0 0 8px 0;
    font-size: 24px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .spec-meta {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-bottom: 12px;
  }

  .spec-version {
    font-size: 11px;
    font-weight: 700;
    background: var(--bg-elevated);
    padding: 2px 6px;
    border-radius: 10px;
    color: var(--text-secondary);
  }

  .spec-badge {
    font-size: 10px;
    font-weight: 700;
    background: rgba(99, 85, 255, 0.15);
    color: var(--accent-primary);
    padding: 2px 6px;
    border-radius: 10px;
  }

  .spec-desc {
    font-size: 13px;
    line-height: 1.6;
    color: var(--text-secondary);
    margin: 0;
  }

  .section-title {
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    margin: 0 0 12px 0;
  }

  .servers-section {
    margin-bottom: 24px;
  }

  .server-item {
    font-size: 12px;
    padding: 8px 12px;
    background: var(--bg-base);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 8px;
  }

  .endpoint-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .endpoint-card {
    background: var(--bg-base);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 12px;
  }

  .endpoint-header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .method-badge {
    font-size: 11px;
    font-weight: 800;
    min-width: 50px;
  }

  .path-text {
    font-size: 13px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .endpoint-desc {
    font-size: 12px;
    margin-top: 8px;
    line-height: 1.5;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 13px;
  }
</style>
