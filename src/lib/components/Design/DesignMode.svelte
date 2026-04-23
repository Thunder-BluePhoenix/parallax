<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import YamlEditor from "./YamlEditor.svelte";
  import ApiPreview from "./ApiPreview.svelte";
  import yaml from "js-yaml";

  const DEFAULT_SPEC = `openapi: 3.0.0
info:
  title: Sample API
  version: 1.0.0
  description: A sample API to demonstrate Design Mode
servers:
  - url: http://localhost:8000
    description: Local dev server
paths:
  /hello:
    get:
      summary: Say Hello
      description: Returns a simple greeting
      responses:
        '200':
          description: Successful response
`;

  let specYaml = $state(DEFAULT_SPEC);
  let parsedSpec = $state<any>(null);
  let parseError = $state("");

  // Parse YAML whenever specYaml changes
  $effect(() => {
    try {
      parsedSpec = yaml.load(specYaml);
      parseError = "";
    } catch (e: any) {
      parseError = e.message || "Failed to parse YAML";
    }
  });

  async function generateCollection() {
    if (!parsedSpec) {
      alert("Spec is not valid.");
      return;
    }
    
    try {
      const collectionName = parsedSpec.info?.title || "OpenAPI Generated Collection";
      let folders: any[] = [];
      let rootRequests: any[] = [];

      // A simple implementation of OpenAPI to Parallax Collection
      if (parsedSpec.paths) {
        for (const [path, methods] of Object.entries(parsedSpec.paths)) {
          let reqs = [];
          for (const [method, details] of Object.entries<any>(methods as any)) {
            const m = method.toUpperCase();
            reqs.push({
              id: "req_" + Math.random().toString(36).substr(2, 9),
              name: details.summary || `${m} ${path}`,
              method: m,
              url: "{{base_url}}" + path,
              headers: { "Content-Type": "application/json" },
              params: {},
              body: { type: m === "GET" || m === "DELETE" ? "none" : "json", content: "{\n}", raw: "{\n}" },
              auth: { type: "none" },
              scripts: null
            });
          }
          
          // Use the first tag as folder name, or root if none
          const tag = (methods as any)[Object.keys(methods as any)[0]]?.tags?.[0];
          if (tag) {
            let folder: any = folders.find(f => f.name === tag);
            if (!folder) {
              folder = { name: tag, requests: [] };
              folders.push(folder);
            }
            folder.requests.push(...reqs);
          } else {
            rootRequests.push(...reqs);
          }
        }
      }

      const newCollection = {
        name: collectionName,
        version: "1.0.0",
        description: parsedSpec.info?.description || "Generated from Design Mode",
        requests: rootRequests,
        folders: folders,
        variables: { "base_url": parsedSpec.servers?.[0]?.url || "http://localhost:8000" }
      };

      await invoke("save_collection", { collection: newCollection });
      alert("Collection generated successfully! Switch to Builder mode to see it.");
    } catch (e) {
      alert("Failed to generate collection: " + e);
    }
  }

  async function saveSpec() {
    try {
      // Assuming a tauri command to save the spec file
      await invoke("save_design_spec", { name: parsedSpec?.info?.title || "untitled", yaml: specYaml });
      // notification / toast could go here
    } catch (e) {
      alert("Failed to save spec: " + e);
    }
  }
</script>

<div class="design-mode">
  <div class="toolbar">
    <div class="toolbar-left">
      <span class="mode-title">Design Mode</span>
      <span class="file-name mono">.parallax/design/{parsedSpec?.info?.title ? parsedSpec.info.title.toLowerCase().replace(/\\s+/g, '-') : 'untitled'}.openapi.yaml</span>
    </div>
    <div class="toolbar-right">
      <button class="btn" onclick={saveSpec}>Save Spec</button>
      <button class="btn-send" onclick={generateCollection} disabled={!!parseError}>Generate Collection →</button>
    </div>
  </div>

  <div class="split-pane">
    <div class="pane left-pane">
      <YamlEditor bind:value={specYaml} onChange={(val) => specYaml = val} />
    </div>
    <div class="pane right-pane">
      <ApiPreview spec={parsedSpec} error={parseError} />
    </div>
  </div>
</div>

<style>
  .design-mode {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-base);
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-default);
    flex-shrink: 0;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .mode-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .file-name {
    font-size: 11px;
    color: var(--text-muted);
    background: var(--bg-elevated);
    padding: 3px 8px;
    border-radius: var(--radius-sm);
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .split-pane {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .pane {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .left-pane {
    border-right: 1px solid var(--border-default);
  }
</style>
