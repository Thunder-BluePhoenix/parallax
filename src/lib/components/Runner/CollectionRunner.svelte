<script lang="ts">
  import {
    loadedCollections,
    activeEnvironment,
    globalVariables,
    collectionVariables,
    responseHistory,
    testResults
  } from "../../stores/app.svelte";
  import { invoke as tauriInvoke } from "@tauri-apps/api/core";
  const invoke = <T>(cmd: string, args?: Record<string, any>): Promise<T> => {
    const fn = tauriInvoke || (window as any)?.__TAURI__?.core?.invoke;
    return fn ? (fn as any)(cmd, args) : Promise.reject("Tauri invoke not found");
  };
  import { resolveRequestTemplates } from "../../utils/template-tags";
  import { runPreRequestScript, runTestScript } from "../../utils/pm-script-runner";
  import type { CollectionRequest, ResponseState, TestResult } from "../../stores/app.svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { readTextFile } from "@tauri-apps/plugin-fs";
  import Papa from "papaparse";
  import Handlebars from "handlebars";
  import { runnerReportTemplate } from "../../utils/runner-report";
  import { save } from "@tauri-apps/plugin-dialog";
  import { writeTextFile } from "@tauri-apps/plugin-fs";

  let { onClose } = $props<{ onClose: () => void }>();

  let selectedCollectionId = $state<string>("");
  let selectedFolderId     = $state<string>(""); // empty means run whole collection
  let iterations           = $state(1);
  let delayMs              = $state(0);
  let stopOnFailure        = $state(false);
  const uuid = () => Math.random().toString(36).substring(2) + Date.now().toString(36);

  // Data file state
  let dataFileName = $state("");
  let dataRows     = $state<any[]>([]);
  let useDataFile  = $state(false);

  // Runner state
  let isRunning  = $state(false);
  let isFinished = $state(false);
  let runStats   = $state({ passed: 0, failed: 0, timeMs: 0 });
  let runFeed    = $state<{
    id: string;
    reqName: string;
    method: string;
    status: number;
    timeMs: number;
    tests: TestResult[];
    error: string | null;
  }[]>([]);
  async function selectDataFile() {
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [{ name: "Data Files", extensions: ["csv", "json"] }]
      });
      if (selected) {
        const content = await readTextFile(selected as string);
        dataFileName = (selected as string).split("/").pop() || "";
        
        if (dataFileName.endsWith(".json")) {
          dataRows = JSON.parse(content);
          if (!Array.isArray(dataRows)) dataRows = [dataRows];
        } else {
          const results = Papa.parse(content, { header: true, skipEmptyLines: true });
          dataRows = results.data;
        }
        
        useDataFile = true;
        iterations = dataRows.length;
      }
    } catch (e) {
      console.error("Failed to load data file:", e);
      alert("Error loading data file: " + e);
    }
  }

  function clearDataFile() {
    dataFileName = "";
    dataRows = [];
    useDataFile = false;
  }

  async function startRun() {
    if (!selectedCollectionId) return;
    const collection = loadedCollections.find(c => c.name === selectedCollectionId);
    if (!collection) return;

    let targetRequests: CollectionRequest[] = [];
    if (selectedFolderId) {
      const folder = collection.folders.find(f => f.name === selectedFolderId);
      if (folder) targetRequests = [...folder.requests];
    } else {
      targetRequests = [...collection.requests];
      collection.folders.forEach(f => {
        targetRequests.push(...f.requests);
      });
    }

    if (targetRequests.length === 0) return;

    // Initialize run
    isRunning  = true;
    isFinished = false;
    runFeed    = [];
    runStats   = { passed: 0, failed: 0, timeMs: 0 };
    
    const startT0 = Date.now();

    // The execution loop
    const runIterations = useDataFile ? dataRows.length : iterations;

    for (let iter = 0; iter < runIterations; iter++) {
      if (runIterations > 1) {
        runFeed.push({
          id: `iter-${iter}`, reqName: `Iteration ${iter + 1}${useDataFile ? ' (Data Row)' : ''}`,
          method: "INFO", status: 0, timeMs: 0, tests: [], error: null
        });
      }

      const dataVarScope = useDataFile ? dataRows[iter] : {};

      for (const req of targetRequests) {
        if (!isRunning) break;

        const t0 = Date.now();
        let currentError: string | null = null;
        let currentTests: TestResult[] = [];
        let currentStatus = 0;

        try {
          // Scope resolution
          collectionVariables.variables = collection.variables ? { ...collection.variables } : {};
          const mergedEnv = {
            ...globalVariables.variables,
            ...collectionVariables.variables,
            ...activeEnvironment.variables,
            ...dataVarScope, // Data variables have highest priority
          };

          // Load scripts for this request
          let scripts = { pre_request: "", tests: "" };
          try {
            scripts = await invoke("load_scripts", { requestId: req.id });
          } catch (e) {
            console.error("Failed to load scripts for", req.id, e);
          }

          // Pre-request script
          if (scripts.pre_request && scripts.pre_request.trim()) {
            await runPreRequestScript(scripts.pre_request, mergedEnv);
          }

          // Build payload
          let bodyPayload = null;
          if (req.body && req.body.type !== "none") {
            bodyPayload = { type: req.body.type, content: req.body.content, raw: req.body.raw };
          }
          let authPayload = null;
          if (req.auth) {
            authPayload = {
              auth_type: req.auth.auth_type ?? "none",
              token: req.auth.token,
              username: req.auth.username,
              password: req.auth.password,
              api_key_header: req.auth.api_key_header,
              api_key_value: req.auth.api_key_value,
              provider: req.auth.provider,
              provider_session: req.auth.provider_session,
            };
          }

          const rawPayload = {
            id: req.id, name: req.name, method: req.method, url: req.url,
            headers: req.headers ?? {}, params: req.params ?? {},
            body: bodyPayload, auth: authPayload,
            timeout_ms: 30000, follow_redirects: true,
          };

          // Apply templates
          const payload = resolveRequestTemplates(rawPayload, mergedEnv);

          const result = await invoke<any>("send_request", {
            request: payload,
            environment: mergedEnv,
          });

          currentStatus = result.status;
          
          const response: ResponseState = {
            status:     result.status,
            statusText: result.status_text ?? "",
            headers:    result.headers ?? {},
            body: {
              raw:         result.body?.raw ?? "",
              json:        result.body?.json ?? null,
              contentType: result.body?.content_type ?? "",
            },
            timing:    { totalMs: result.timing?.total_ms ?? (Date.now() - t0) },
            sizeBytes: result.size_bytes ?? 0,
            cookies:   result.cookies ?? [],
          };

          // Test scripts
          if (scripts.tests && scripts.tests.trim()) {
            const { tests } = await runTestScript(scripts.tests, response, mergedEnv);
            currentTests = tests;
            for (const t of tests) {
              if (t.passed) runStats.passed++;
              else runStats.failed++;
            }
          } else {
            // Artificial test pass if no scripts
            if (currentStatus < 300) {
              currentTests.push({ name: `Status code is ${currentStatus}`, passed: true });
              runStats.passed++;
            } else {
              currentTests.push({ name: `Status code is ${currentStatus}`, passed: false, error: "Non-2xx status" });
              runStats.failed++;
            }
          }

        } catch (e: any) {
          currentError = e?.toString() ?? "Request failed";
          currentTests.push({ name: "Request execution", passed: false, error: currentError || undefined });
          runStats.failed++;
        }

        const duration = Date.now() - t0;
        
        runFeed.push({
          id: uuid(), reqName: req.name, method: req.method,
          status: currentStatus, timeMs: duration, tests: currentTests, error: currentError
        });

        if (stopOnFailure && runStats.failed > 0) {
          isRunning = false;
          break;
        }

        if (delayMs > 0 && isRunning) {
          await new Promise(r => setTimeout(r, delayMs));
        }
      }
    }

    runStats.timeMs = Date.now() - startT0;
    isRunning = false;
    isFinished = true;
  }

  function stopRun() {
    isRunning = false;
  }

  async function exportReport(format: "html" | "json") {
    try {
      const defaultPath = `parallax-run-${Date.now()}.${format}`;
      const selected = await save({
        defaultPath,
        filters: [{ name: format.toUpperCase() + " Report", extensions: [format] }]
      });

      if (selected) {
        let content = "";
        if (format === "json") {
          content = JSON.stringify({
            collection: selectedCollectionId,
            stats: runStats,
            feed: runFeed,
            timestamp: new Date().toISOString(),
          }, null, 2);
        } else {
          const template = Handlebars.compile(runnerReportTemplate);
          content = template({
            collectionName: selectedCollectionId,
            folderName: selectedFolderId || "Full Collection",
            stats: runStats,
            requests: runFeed.filter(r => r.method !== "INFO"),
            timestamp: new Date().toLocaleString(),
          });
        }
        await writeTextFile(selected, content);
      }
    } catch (e) {
      console.error("Export failed:", e);
      alert("Failed to export report: " + e);
    }
  }
</script>

<div class="runner-container">
  <div class="runner-header">
    <div class="header-left">
      <h2>Collection Runner</h2>
    </div>
    <button class="close-btn" onclick={onClose}>×</button>
  </div>

  <div class="runner-content">
    <div class="runner-config">
      <div class="config-group">
        <label for="collection">Collection</label>
        <select id="collection" bind:value={selectedCollectionId}>
          <option value="" disabled>Select a collection...</option>
          {#each loadedCollections as col}
            <option value={col.name}>{col.name}</option>
          {/each}
        </select>
      </div>

      {#if selectedCollectionId}
        {@const col = loadedCollections.find(c => c.name === selectedCollectionId)}
        {#if col && col.folders.length > 0}
          <div class="config-group">
            <label for="folder">Folder (Optional)</label>
            <select id="folder" bind:value={selectedFolderId}>
              <option value="">-- All Requests --</option>
              {#each col.folders as f}
                <option value={f.name}>{f.name}</option>
              {/each}
            </select>
          </div>
        {/if}
      {/if}

      <div class="config-group">
        <label for="iterations">Iterations</label>
        <input id="iterations" type="number" min="1" max="100" bind:value={iterations} />
      </div>

      <div class="config-group">
        <label for="delay">Delay (ms)</label>
        <input id="delay" type="number" min="0" step="100" bind:value={delayMs} />
      </div>

      <div class="config-checkbox">
        <input id="stopOnFail" type="checkbox" bind:checked={stopOnFailure} />
        <label for="stopOnFail">Stop run if an error occurs</label>
      </div>

      <div class="config-divider"></div>

      <div class="config-group">
        <label>Data File (CSV/JSON)</label>
        {#if !dataFileName}
          <button class="btn-secondary" onclick={selectDataFile}>Select File</button>
        {:else}
          <div class="data-file-info">
            <span class="file-name" title={dataFileName}>{dataFileName}</span>
            <span class="row-count">{dataRows.length} rows</span>
            <button class="btn-icon-sm" onclick={clearDataFile} title="Clear file">×</button>
          </div>
        {/if}
      </div>

      <div class="config-actions">
        {#if isRunning}
          <button class="btn-stop" onclick={stopRun}>Stop Run</button>
        {:else}
          <button class="btn-start" onclick={startRun} disabled={!selectedCollectionId}>
            {isFinished ? "Run Again" : "Run Collection"}
          </button>
        {/if}
      </div>
    </div>

    <div class="runner-results">
      <div class="results-summary">
        <div class="stat-box">
          <span class="stat-value text-success">{runStats.passed}</span>
          <span class="stat-label">Passed</span>
        </div>
        <div class="stat-box">
          <span class="stat-value text-error">{runStats.failed}</span>
          <span class="stat-label">Failed</span>
        </div>
        <div class="stat-box">
          <span class="stat-value">{runStats.timeMs}ms</span>
          <span class="stat-label">Duration</span>
        </div>
        {#if isFinished}
          <div class="summary-actions">
            <button class="btn-secondary" onclick={() => exportReport("html")}>HTML Report</button>
            <button class="btn-secondary" onclick={() => exportReport("json")}>JSON Export</button>
          </div>
        {/if}
      </div>

      <div class="feed-scroll scroll-y">
        {#if runFeed.length === 0}
          <div class="feed-empty">
            <p>Configure and start the run to see results here.</p>
          </div>
        {:else}
          {#each runFeed as item}
            {#if item.method === "INFO"}
              <div class="feed-info">{item.reqName}</div>
            {:else}
              <div class="feed-item">
                <div class="feed-item-header">
                  <span class="feed-method method-{item.method.toLowerCase()}">{item.method}</span>
                  <span class="feed-name">{item.reqName}</span>
                  <span class="feed-status" class:status-2xx={item.status > 0 && item.status < 400} class:status-error={item.status >= 400 || item.error}>
                    {item.status || "ERR"}
                  </span>
                  <span class="feed-time">{item.timeMs}ms</span>
                </div>
                {#if item.tests.length > 0}
                  <div class="feed-tests">
                    {#each item.tests as t}
                      <div class="feed-test" class:test-pass={t.passed} class:test-fail={!t.passed}>
                        <span class="test-icon">{t.passed ? "✓" : "✗"}</span>
                        <span>{t.name}</span>
                        {#if t.error} <span class="test-err">({t.error})</span> {/if}
                      </div>
                    {/each}
                  </div>
                {/if}
                {#if item.error}
                  <div class="feed-error-msg">{item.error}</div>
                {/if}
              </div>
            {/if}
          {/each}
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .runner-container {
    position: absolute;
    top: 0; left: 0; right: 0; bottom: 0;
    background: var(--bg-base);
    z-index: 100;
    display: flex;
    flex-direction: column;
  }

  .runner-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 20px;
    border-bottom: 1px solid var(--border-default);
    background: var(--bg-surface);
  }

  .runner-header h2 { font-size: 16px; font-weight: 600; margin: 0; }

  .close-btn {
    background: transparent;
    font-size: 20px;
    color: var(--text-muted);
    transition: var(--transition-fast);
  }
  .close-btn:hover { color: var(--text-primary); }

  .runner-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .runner-config {
    width: 300px;
    padding: 20px;
    border-right: 1px solid var(--border-default);
    background: var(--bg-surface);
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
  }

  .config-group { display: flex; flex-direction: column; gap: 6px; }
  .config-group label { font-size: 11px; font-weight: 600; text-transform: uppercase; color: var(--text-secondary); }
  .config-group select, .config-group input {
    height: 32px; padding: 0 10px; border-radius: var(--radius-sm); font-size: 13px;
  }

  .config-checkbox {
    display: flex; align-items: center; gap: 8px; font-size: 13px; color: var(--text-primary);
    margin-top: 8px;
  }

  .config-divider { height: 1px; background: var(--border-subtle); margin: 4px 0; }

  .data-file-info {
    display: flex; align-items: center; gap: 8px;
    padding: 6px 10px; background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-sm); font-size: 12px;
  }
  .file-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 500; }
  .row-count { color: var(--text-muted); font-size: 11px; }

  .btn-secondary {
    height: 32px; background: var(--bg-elevated); border: 1px solid var(--border-default);
    color: var(--text-primary); border-radius: var(--radius-sm); font-size: 12px;
    transition: var(--transition-fast);
  }
  .btn-secondary:hover { border-color: var(--accent-primary); }

  .btn-icon-sm {
    background: transparent; border: none; color: var(--text-muted); font-size: 16px;
    cursor: pointer; line-height: 1; padding: 0 4px;
  }
  .btn-icon-sm:hover { color: var(--color-error); }

  .config-actions { margin-top: auto; padding-top: 20px; }
  .btn-start, .btn-stop {
    width: 100%; height: 36px; border-radius: var(--radius-md); font-weight: 600; font-size: 13px;
    transition: var(--transition-fast);
  }
  .btn-start { background: var(--accent-primary); color: white; border: none; }
  .btn-start:hover:not(:disabled) { background: #9185ff; }
  .btn-start:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-stop { background: var(--color-error); color: white; border: none; }
  .btn-stop:hover { background: #ff6b6b; }

  .runner-results {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: var(--bg-base);
  }

  .results-summary {
    display: flex;
    gap: 20px;
    padding: 16px 24px;
    border-bottom: 1px solid var(--border-default);
    background: var(--bg-surface);
  }

  .stat-box { display: flex; flex-direction: column; gap: 4px; }
  .stat-value { font-size: 24px; font-weight: 700; font-family: var(--font-mono); }
  .stat-label { font-size: 11px; color: var(--text-muted); text-transform: uppercase; }
  .text-success { color: var(--color-success); }
  .text-error { color: var(--color-error); }

  .summary-actions { margin-left: auto; display: flex; gap: 8px; align-items: center; }

  .feed-scroll { flex: 1; padding: 12px; }
  .feed-empty { color: var(--text-muted); font-size: 13px; text-align: center; margin-top: 40px; }

  .feed-info {
    padding: 12px 16px; font-size: 13px; font-weight: 600; color: var(--text-secondary);
    background: var(--bg-surface); border-radius: var(--radius-sm); margin-bottom: 8px;
  }

  .feed-item {
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    margin-bottom: 8px;
    padding: 10px 14px;
  }

  .feed-item-header {
    display: flex; align-items: center; gap: 12px; font-size: 13px;
  }

  .feed-method { font-family: var(--font-mono); font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 3px; }
  .method-get { background: rgba(74, 222, 128, 0.15); color: #4ade80; }
  .method-post { background: rgba(251, 146, 60, 0.15); color: #fb923c; }
  .feed-name { flex: 1; font-weight: 500; }
  .feed-status { font-family: var(--font-mono); font-weight: 600; }
  .status-2xx { color: var(--color-success); }
  .status-error { color: var(--color-error); }
  .feed-time { font-family: var(--font-mono); font-size: 11px; color: var(--text-muted); }

  .feed-tests {
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid var(--border-default);
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
  }

  .feed-test { display: flex; gap: 6px; align-items: center; }
  .test-pass { color: var(--text-secondary); }
  .test-fail { color: var(--color-error); }
  .test-icon { font-weight: 700; }
  .test-pass .test-icon { color: var(--color-success); }
  .test-err { font-family: var(--font-mono); font-size: 10px; opacity: 0.8; }

  .feed-error-msg { margin-top: 6px; font-family: var(--font-mono); font-size: 11px; color: var(--color-error); }
</style>
