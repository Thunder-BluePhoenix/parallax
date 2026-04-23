<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { loadedCollections, currentWorkspace, type Collection, type CollectionRequest, type CollectionFolder } from "../../stores/app.svelte";
  import { githubIdentity } from "../../stores/github.svelte";

  let selectedColIdx = $state(0);
  let repoOwner = $state(localStorage.getItem("parallax:docs_owner") ?? "");
  let repoName = $state(localStorage.getItem("parallax:docs_repo") ?? "");
  let publishing = $state(false);
  let previewing = $state(false);
  let publishedUrl = $state(localStorage.getItem("parallax:docs_url") ?? "");
  let statusMsg = $state("");
  let previewHtml = $state("");
  let showPreview = $state(false);

  const token = $derived(githubIdentity.value?.token ?? "");
  const collection = $derived(loadedCollections[selectedColIdx] ?? null);

  function saveRepo() {
    localStorage.setItem("parallax:docs_owner", repoOwner);
    localStorage.setItem("parallax:docs_repo", repoName);
  }

  // ── HTML generator ────────────────────────────────────────────

  function methodColor(m: string): string {
    const colors: Record<string, string> = {
      GET: "#3fb950", POST: "#e3b341", PUT: "#79c0ff",
      PATCH: "#d2a8ff", DELETE: "#f85149", HEAD: "#8b949e", OPTIONS: "#8b949e",
    };
    return colors[m.toUpperCase()] ?? "#8b949e";
  }

  function renderRequest(req: CollectionRequest): string {
    const color = methodColor(req.method);
    const headersHtml = Object.entries(req.headers ?? {}).map(([k, v]) =>
      `<tr><td class="key">${esc(k)}</td><td>${esc(v)}</td></tr>`
    ).join("");
    const paramsHtml = Object.entries(req.params ?? {}).map(([k, v]) =>
      `<tr><td class="key">${esc(k)}</td><td>${esc(v)}</td></tr>`
    ).join("");
    const bodyHtml = req.body?.raw
      ? `<pre class="body-pre">${esc(req.body.raw)}</pre>`
      : req.body?.content && typeof req.body.content === "object"
        ? `<pre class="body-pre">${esc(JSON.stringify(req.body.content, null, 2))}</pre>`
        : "";
    const preReq = req.scripts?.preRequest?.trim() ?? "";
    const tests = req.scripts?.tests?.trim() ?? "";

    return `
<div class="req-card" id="req-${esc(req.id)}">
  <div class="req-header">
    <span class="method-badge" style="background:${color}22;color:${color};border:1px solid ${color}44">${esc(req.method)}</span>
    <span class="req-name">${esc(req.name)}</span>
  </div>
  <div class="req-url">${esc(req.url)}</div>
  ${headersHtml ? `<h4>Headers</h4><table class="kv-table">${headersHtml}</table>` : ""}
  ${paramsHtml ? `<h4>Query Parameters</h4><table class="kv-table">${paramsHtml}</table>` : ""}
  ${bodyHtml ? `<h4>Body</h4>${bodyHtml}` : ""}
  ${preReq ? `<h4>Pre-request Script</h4><pre class="body-pre">${esc(preReq)}</pre>` : ""}
  ${tests ? `<h4>Tests</h4><pre class="body-pre">${esc(tests)}</pre>` : ""}
</div>`;
  }

  function renderFolder(folder: CollectionFolder): string {
    return `
<section class="folder-section">
  <h3 class="folder-title">${esc(folder.name)}</h3>
  ${folder.requests.map(renderRequest).join("")}
</section>`;
  }

  function buildNavItems(col: Collection): string {
    const rootItems = col.requests.map(r =>
      `<li><a href="#req-${esc(r.id)}">${esc(r.name)}</a></li>`
    ).join("");
    const folderItems = col.folders.map(f => `
<li class="nav-folder">
  <span class="nav-folder-name">${esc(f.name)}</span>
  <ul>${f.requests.map(r => `<li><a href="#req-${esc(r.id)}">${esc(r.name)}</a></li>`).join("")}</ul>
</li>`).join("");
    return rootItems + folderItems;
  }

  function generateHTML(col: Collection): string {
    const rootRequests = col.requests.map(renderRequest).join("");
    const folders = col.folders.map(renderFolder).join("");
    const navItems = buildNavItems(col);
    const desc = col.description ? `<p class="col-desc">${esc(col.description)}</p>` : "";

    return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>${esc(col.name)} — API Reference</title>
<style>
  *{box-sizing:border-box;margin:0;padding:0}
  body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;background:#0d1117;color:#c9d1d9;display:flex;min-height:100vh}
  a{color:#58a6ff;text-decoration:none}a:hover{text-decoration:underline}
  nav{width:240px;min-width:240px;background:#161b22;border-right:1px solid #30363d;padding:24px 0;position:sticky;top:0;height:100vh;overflow-y:auto}
  .nav-title{font-size:14px;font-weight:700;padding:0 20px 16px;color:#e6edf3;border-bottom:1px solid #30363d;margin-bottom:12px}
  nav ul{list-style:none;padding:0 12px}
  nav li{margin:2px 0}
  nav a{display:block;padding:6px 8px;border-radius:6px;font-size:12px;color:#8b949e;transition:background .15s,color .15s}
  nav a:hover{background:#21262d;color:#c9d1d9;text-decoration:none}
  .nav-folder-name{display:block;padding:8px 8px 4px;font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:.06em;color:#8b949e}
  .nav-folder>ul{padding-left:8px}
  main{flex:1;padding:40px;max-width:900px}
  .col-header{margin-bottom:32px;padding-bottom:24px;border-bottom:1px solid #30363d}
  .col-header h1{font-size:28px;font-weight:800;color:#e6edf3;margin-bottom:6px}
  .col-desc{font-size:14px;color:#8b949e;line-height:1.6}
  .folder-section{margin-bottom:40px}
  .folder-title{font-size:16px;font-weight:700;color:#e6edf3;margin:32px 0 16px;padding-bottom:8px;border-bottom:1px solid #21262d}
  .req-card{background:#161b22;border:1px solid #30363d;border-radius:10px;padding:20px;margin-bottom:16px}
  .req-header{display:flex;align-items:center;gap:10px;margin-bottom:8px}
  .method-badge{font-size:11px;font-weight:800;font-family:monospace;padding:3px 8px;border-radius:4px;letter-spacing:.04em}
  .req-name{font-size:15px;font-weight:700;color:#e6edf3}
  .req-url{font-family:monospace;font-size:12px;color:#8b949e;background:#0d1117;padding:8px 12px;border-radius:6px;margin-bottom:14px;word-break:break-all}
  h4{font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.06em;color:#8b949e;margin:14px 0 6px}
  .kv-table{width:100%;border-collapse:collapse;font-size:12px}
  .kv-table td{padding:5px 8px;border-bottom:1px solid #21262d}
  .kv-table .key{font-family:monospace;color:#79c0ff;width:35%}
  .body-pre{background:#0d1117;border:1px solid #21262d;border-radius:6px;padding:12px;font-size:12px;font-family:monospace;white-space:pre-wrap;word-break:break-all;color:#c9d1d9;overflow-x:auto}
  .footer{margin-top:60px;padding-top:20px;border-top:1px solid #30363d;font-size:11px;color:#8b949e}
  @media(max-width:700px){body{flex-direction:column}nav{width:100%;min-width:0;height:auto;position:static;border-right:none;border-bottom:1px solid #30363d}}
</style>
</head>
<body>
<nav>
  <div class="nav-title">${esc(col.name)}</div>
  <ul>${navItems}</ul>
</nav>
<main>
  <div class="col-header">
    <h1>${esc(col.name)}</h1>
    ${desc}
  </div>
  ${rootRequests}
  ${folders}
  <div class="footer">Generated by <strong>Parallax</strong> · ${new Date().toLocaleDateString()}</div>
</main>
</body>
</html>`;
  }

  function esc(s: string): string {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  // ── Actions ───────────────────────────────────────────────────

  function preview() {
    if (!collection) return;
    previewHtml = generateHTML(collection);
    showPreview = true;
    previewing = false;
  }

  async function publish() {
    if (!collection || !repoOwner || !repoName || !token) return;
    saveRepo();
    publishing = true;
    statusMsg = "";
    publishedUrl = "";
    localStorage.removeItem("parallax:docs_url");
    try {
      const htmlContent = generateHTML(collection);
      const url = await invoke<string>("github_publish_docs", {
        workspacePath: currentWorkspace.path || ".",
        repoOwner,
        repoName,
        token,
        htmlContent,
      });
      publishedUrl = url;
      localStorage.setItem("parallax:docs_url", url);
      statusMsg = "Published successfully!";
    } catch (e: any) {
      statusMsg = `Publish failed: ${e}`;
    } finally {
      publishing = false;
    }
  }

  function totalRequestCount(col: Collection): number {
    return col.requests.length + col.folders.reduce((s, f) => s + f.requests.length, 0);
  }
</script>

<div class="dashboard-section animate-fade-in">
  <div class="section-header">
    <h2>API Docs Publisher</h2>
    <p class="section-desc">Generate a single-page API reference from any collection and publish to GitHub Pages.</p>
  </div>

  <!-- Collection picker -->
  <div class="card">
    <div class="card-title">Collection</div>
    {#if loadedCollections.length === 0}
      <p class="hint">No collections loaded. Open a workspace and import or create a collection first.</p>
    {:else}
      <div class="col-picker">
        {#each loadedCollections as col, i}
          <button
            class="col-option"
            class:selected={selectedColIdx === i}
            onclick={() => { selectedColIdx = i; showPreview = false; }}
          >
            <span class="col-option-name">{col.name}</span>
            <span class="col-option-meta">{totalRequestCount(col)} requests</span>
          </button>
        {/each}
      </div>

      {#if collection}
        <div class="col-summary">
          <span class="col-ver">v{collection.version}</span>
          {#if collection.description}
            <span class="col-desc-text">{collection.description}</span>
          {/if}
          <span class="col-counts">
            {collection.requests.length} root · {collection.folders.length} folders
          </span>
        </div>
      {/if}
    {/if}
  </div>

  <!-- GitHub repo -->
  <div class="card">
    <div class="card-title">Publish Target</div>
    {#if !githubIdentity.value}
      <p class="hint">Sign in with GitHub (via the Team panel) to publish docs.</p>
    {:else}
      <div class="repo-row">
        <span class="repo-label">github.com /</span>
        <input
          class="form-input"
          style="width:130px"
          bind:value={repoOwner}
          placeholder="owner"
          onchange={saveRepo}
        />
        <span class="repo-slash">/</span>
        <input
          class="form-input"
          style="width:180px"
          bind:value={repoName}
          placeholder="repo-name"
          onchange={saveRepo}
        />
      </div>
      <p class="hint" style="margin-top:8px">
        Publishes <code>index.html</code> to the <code>gh-pages</code> branch.
        GitHub Pages must be enabled on that branch in your repo settings.
      </p>
    {/if}
  </div>

  <!-- Actions -->
  {#if collection}
    <div class="action-row">
      <button class="btn-action" onclick={preview} disabled={previewing}>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
        Preview HTML
      </button>

      {#if githubIdentity.value}
        <button
          class="btn-primary"
          onclick={publish}
          disabled={publishing || !repoOwner || !repoName}
        >
          {#if publishing}
            <svg class="spin" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/></svg>
            Publishing…
          {:else}
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
            Publish to GitHub Pages
          {/if}
        </button>
      {/if}
    </div>
  {/if}

  {#if statusMsg}
    <div class="status-msg" class:error={statusMsg.startsWith("Publish failed")}>{statusMsg}</div>
  {/if}

  {#if publishedUrl}
    <div class="live-url-card">
      <span class="live-label">Live URL</span>
      <a href={publishedUrl} target="_blank" rel="noreferrer" class="live-url">{publishedUrl}</a>
      <button class="btn-copy" onclick={() => navigator.clipboard.writeText(publishedUrl)} title="Copy URL">
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
      </button>
    </div>
  {/if}

  <!-- Preview pane -->
  {#if showPreview && previewHtml}
    <div class="preview-pane">
      <div class="preview-header">
        <span class="preview-title">HTML Preview</span>
        <button class="btn-text" onclick={() => showPreview = false}>Close ×</button>
      </div>
      <iframe
        title="API Docs Preview"
        class="preview-iframe"
        srcdoc={previewHtml}
        sandbox="allow-same-origin"
      ></iframe>
    </div>
  {/if}
</div>

<style>
  .dashboard-section { max-width: 760px; display: flex; flex-direction: column; gap: 16px; }
  .section-header { margin-bottom: 4px; }
  .section-header h2 { font-size: 18px; font-weight: 700; margin-bottom: 4px; }
  .section-desc { font-size: 12px; color: var(--text-secondary); }

  .card {
    background: var(--bg-surface); border: 1px solid var(--border-default);
    border-radius: var(--radius-lg); padding: 16px;
  }
  .card-title {
    font-size: 11px; font-weight: 700; color: var(--text-muted);
    text-transform: uppercase; letter-spacing: .06em; margin-bottom: 12px;
  }

  .hint { font-size: 11px; color: var(--text-muted); line-height: 1.5; }
  .hint code { color: var(--accent-primary); font-family: var(--font-mono); }

  /* Collection picker */
  .col-picker { display: flex; flex-direction: column; gap: 4px; }
  .col-option {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 12px; border-radius: var(--radius-md);
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    cursor: pointer; text-align: left; transition: var(--transition-fast);
  }
  .col-option:hover { border-color: var(--accent-primary); }
  .col-option.selected { border-color: var(--accent-primary); background: var(--accent-primary-dim); }
  .col-option-name { font-size: 13px; font-weight: 600; color: var(--text-primary); }
  .col-option-meta { font-size: 10px; color: var(--text-muted); }

  .col-summary {
    display: flex; align-items: center; gap: 8px; margin-top: 10px;
    padding-top: 10px; border-top: 1px solid var(--border-subtle);
  }
  .col-ver {
    font-size: 10px; font-weight: 700; font-family: var(--font-mono);
    color: var(--accent-secondary); background: var(--bg-elevated);
    padding: 1px 6px; border-radius: 10px;
  }
  .col-desc-text { font-size: 11px; color: var(--text-muted); flex: 1; }
  .col-counts { font-size: 10px; color: var(--text-muted); white-space: nowrap; }

  /* Repo row */
  .repo-row { display: flex; align-items: center; gap: 8px; }
  .repo-label { font-size: 12px; color: var(--text-muted); white-space: nowrap; }
  .repo-slash { font-size: 16px; color: var(--text-muted); }

  .form-input {
    height: 30px; padding: 0 10px; background: var(--bg-input);
    border: 1px solid var(--border-default); border-radius: var(--radius-md);
    color: var(--text-primary); font-size: 12px;
  }
  .form-input:focus { border-color: var(--accent-primary); outline: none; }

  /* Actions */
  .action-row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }

  .btn-action {
    display: flex; align-items: center; gap: 6px;
    height: 30px; padding: 0 14px; background: var(--bg-elevated);
    border: 1px solid var(--border-default); color: var(--text-secondary);
    font-size: 12px; font-weight: 600; border-radius: var(--radius-md);
    cursor: pointer; transition: var(--transition-fast); white-space: nowrap;
  }
  .btn-action:hover:not(:disabled) { border-color: var(--accent-primary); color: var(--accent-primary); }
  .btn-action:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn-primary {
    display: flex; align-items: center; gap: 6px;
    height: 30px; padding: 0 16px; background: var(--accent-primary); color: white;
    border: none; border-radius: var(--radius-md); font-weight: 600; font-size: 12px;
    cursor: pointer; transition: var(--transition-fast); white-space: nowrap;
  }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn-text { font-size: 11px; color: var(--text-muted); background: none; border: none; cursor: pointer; }
  .btn-text:hover { color: var(--text-primary); }

  .spin { animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* Status */
  .status-msg { font-size: 11px; color: var(--accent-secondary); font-family: var(--font-mono); }
  .status-msg.error { color: var(--color-error); }

  /* Live URL */
  .live-url-card {
    display: flex; align-items: center; gap: 8px;
    background: var(--bg-surface); border: 1px solid var(--color-success);
    border-radius: var(--radius-md); padding: 10px 14px;
  }
  .live-label {
    font-size: 10px; font-weight: 700; color: var(--color-success);
    text-transform: uppercase; letter-spacing: .06em; white-space: nowrap;
  }
  .live-url { font-size: 12px; font-family: var(--font-mono); color: var(--accent-primary); flex: 1; word-break: break-all; }
  .btn-copy {
    background: transparent; border: none; color: var(--text-muted);
    cursor: pointer; padding: 2px; flex-shrink: 0;
  }
  .btn-copy:hover { color: var(--text-primary); }

  /* Preview */
  .preview-pane {
    border: 1px solid var(--border-default); border-radius: var(--radius-lg); overflow: hidden;
  }
  .preview-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 14px; background: var(--bg-elevated);
    border-bottom: 1px solid var(--border-default);
  }
  .preview-title { font-size: 11px; font-weight: 700; color: var(--text-secondary); text-transform: uppercase; letter-spacing: .06em; }
  .preview-iframe { width: 100%; height: 520px; border: none; display: block; background: #0d1117; }
</style>
