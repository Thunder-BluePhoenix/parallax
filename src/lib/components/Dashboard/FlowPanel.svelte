<script lang="ts">
  import { loadedCollections, type Collection, type CollectionRequest, type CollectionFolder } from "../../stores/app.svelte";

  let selectedColIdx = $state(0);
  let svgEl = $state<SVGSVGElement | null>(null);

  const collection = $derived(loadedCollections[selectedColIdx] ?? null);

  // ── Graph builder ─────────────────────────────────────────────

  interface Node {
    id: string;
    label: string;
    method: string;
    url: string;
    x: number;
    y: number;
    folder?: string;
  }

  interface Edge {
    from: string;
    to: string;
    label: string;
  }

  function methodColor(m: string): string {
    const map: Record<string, string> = {
      GET: "#3fb950", POST: "#e3b341", PUT: "#79c0ff",
      PATCH: "#d2a8ff", DELETE: "#f85149", HEAD: "#8b949e",
    };
    return map[m.toUpperCase()] ?? "#8b949e";
  }

  // Extract {{var}} and {% response %} references from a string
  function extractVarRefs(s: string): string[] {
    const refs: string[] = [];
    const varRe = /\{\{([^}]+)\}\}/g;
    let m: RegExpExecArray | null;
    while ((m = varRe.exec(s)) !== null) refs.push(m[1].trim());
    return refs;
  }

  function collectText(req: CollectionRequest): string {
    return [
      req.url,
      JSON.stringify(req.headers ?? {}),
      JSON.stringify(req.params ?? {}),
      req.body?.raw ?? "",
      req.scripts?.preRequest ?? "",
      req.scripts?.tests ?? "",
    ].join(" ");
  }

  // Vars SET by a request (pm.environment.set calls in test/pre-request scripts)
  function setsVars(req: CollectionRequest): string[] {
    const sets: string[] = [];
    const setRe = /pm\.environment\.set\s*\(\s*["']([^"']+)["']/g;
    const text = (req.scripts?.preRequest ?? "") + " " + (req.scripts?.tests ?? "");
    let m: RegExpExecArray | null;
    while ((m = setRe.exec(text)) !== null) sets.push(m[1]);
    return sets;
  }

  const graph = $derived.by(() => {
    if (!collection) return { nodes: [], edges: [] };

    const allRequests: Array<{ req: CollectionRequest; folder?: string }> = [
      ...collection.requests.map(r => ({ req: r })),
      ...collection.folders.flatMap(f => f.requests.map(r => ({ req: r, folder: f.name }))),
    ];

    // Layout: columns by folder, rows within folder
    const folderGroups = new Map<string, typeof allRequests>();
    for (const item of allRequests) {
      const key = item.folder ?? "__root__";
      if (!folderGroups.has(key)) folderGroups.set(key, []);
      folderGroups.get(key)!.push(item);
    }

    const NODE_W = 180;
    const NODE_H = 54;
    const COL_GAP = 60;
    const ROW_GAP = 24;
    const PAD = 40;

    const nodes: Node[] = [];
    let colX = PAD;

    for (const [folderKey, items] of folderGroups) {
      items.forEach((item, rowIdx) => {
        nodes.push({
          id: item.req.id || item.req.name,
          label: item.req.name,
          method: item.req.method,
          url: item.req.url,
          x: colX,
          y: PAD + rowIdx * (NODE_H + ROW_GAP),
          folder: folderKey === "__root__" ? undefined : folderKey,
        });
      });
      colX += NODE_W + COL_GAP;
    }

    // Build variable set map: varName → nodeId
    const varSetBy = new Map<string, string>();
    for (const node of nodes) {
      const req = allRequests.find(r => (r.req.id || r.req.name) === node.id)?.req;
      if (!req) continue;
      for (const v of setsVars(req)) varSetBy.set(v, node.id);
    }

    // Build edges: if req B references {{var}} set by req A → edge A→B
    const edges: Edge[] = [];
    const seenEdges = new Set<string>();
    for (const node of nodes) {
      const req = allRequests.find(r => (r.req.id || r.req.name) === node.id)?.req;
      if (!req) continue;
      const refs = extractVarRefs(collectText(req));
      for (const varName of refs) {
        const sourceId = varSetBy.get(varName);
        if (sourceId && sourceId !== node.id) {
          const key = `${sourceId}→${node.id}`;
          if (!seenEdges.has(key)) {
            seenEdges.add(key);
            edges.push({ from: sourceId, to: node.id, label: `{{${varName}}}` });
          }
        }
      }
    }

    return { nodes, edges };
  });

  const svgWidth  = $derived(graph.nodes.length === 0 ? 600 : Math.max(...graph.nodes.map(n => n.x + 220)));
  const svgHeight = $derived(graph.nodes.length === 0 ? 300 : Math.max(...graph.nodes.map(n => n.y + 80)));

  // ── Edge path helper ──────────────────────────────────────────

  const NODE_W = 180;
  const NODE_H = 54;

  function edgePath(from: Node, to: Node): string {
    const x1 = from.x + NODE_W;
    const y1 = from.y + NODE_H / 2;
    const x2 = to.x;
    const y2 = to.y + NODE_H / 2;
    const cx = (x1 + x2) / 2;
    return `M ${x1} ${y1} C ${cx} ${y1} ${cx} ${y2} ${x2} ${y2}`;
  }

  function findNode(id: string) {
    return graph.nodes.find(n => n.id === id);
  }

  function edgeMidpoint(from: Node, to: Node): { x: number; y: number } {
    return {
      x: (from.x + NODE_W + to.x) / 2,
      y: (from.y + to.y + NODE_H) / 2,
    };
  }
</script>

<div class="dashboard-section animate-fade-in">
  <div class="section-header">
    <h2>Request Flow</h2>
    <p class="section-desc">
      Visual dependency graph of your collection — arrows show variable chains between requests.
    </p>
  </div>

  <!-- Collection picker -->
  <div class="col-bar">
    {#if loadedCollections.length === 0}
      <p class="hint">No collections loaded. Open a workspace and import a collection first.</p>
    {:else}
      {#each loadedCollections as col, i}
        <button
          class="col-pill"
          class:active={selectedColIdx === i}
          onclick={() => selectedColIdx = i}
        >{col.name}</button>
      {/each}
    {/if}
  </div>

  {#if collection}
    <div class="legend-row">
      {#each [["GET","#3fb950"],["POST","#e3b341"],["PUT","#79c0ff"],["PATCH","#d2a8ff"],["DELETE","#f85149"]] as [m, c]}
        <span class="legend-item">
          <span class="legend-dot" style="background:{c}"></span>{m}
        </span>
      {/each}
      <span class="legend-sep"></span>
      <span class="legend-edge">— — →</span>
      <span class="legend-edge-label">variable dependency</span>
    </div>

    {#if graph.nodes.length === 0}
      <div class="empty-state">No requests in this collection.</div>
    {:else}
      <div class="flow-scroll">
        <svg
          bind:this={svgEl}
          width={svgWidth}
          height={svgHeight}
          class="flow-svg"
          xmlns="http://www.w3.org/2000/svg"
        >
          <defs>
            <marker id="arrow" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
              <path d="M0,0 L0,6 L8,3 z" fill="#58a6ff" opacity="0.7" />
            </marker>
          </defs>

          <!-- Folder background bands -->
          {#each [...new Set(graph.nodes.filter(n => n.folder).map(n => n.folder))] as folder}
            {@const folderNodes = graph.nodes.filter(n => n.folder === folder)}
            {@const minY = Math.min(...folderNodes.map(n => n.y)) - 12}
            {@const maxY = Math.max(...folderNodes.map(n => n.y)) + NODE_H + 12}
            {@const x = folderNodes[0].x - 12}
            <rect
              x={x} y={minY}
              width={NODE_W + 24}
              height={maxY - minY}
              rx="10"
              fill="rgba(56,139,253,0.06)"
              stroke="rgba(56,139,253,0.15)"
              stroke-width="1"
            />
            <text x={x + 10} y={minY + 14} font-size="10" fill="#8b949e" font-family="sans-serif">
              {folder}
            </text>
          {/each}

          <!-- Edges -->
          {#each graph.edges as edge}
            {@const fromNode = findNode(edge.from)}
            {@const toNode = findNode(edge.to)}
            {#if fromNode && toNode}
              {@const mid = edgeMidpoint(fromNode, toNode)}
              <path
                d={edgePath(fromNode, toNode)}
                fill="none"
                stroke="#58a6ff"
                stroke-width="1.5"
                stroke-dasharray="5,3"
                opacity="0.6"
                marker-end="url(#arrow)"
              />
              <rect
                x={mid.x - 30} y={mid.y - 9}
                width="60" height="16"
                rx="4"
                fill="#0d1117"
                stroke="#30363d"
                stroke-width="1"
              />
              <text
                x={mid.x} y={mid.y + 4}
                text-anchor="middle"
                font-size="9"
                fill="#79c0ff"
                font-family="monospace"
              >{edge.label.length > 12 ? edge.label.slice(0, 11) + "…" : edge.label}</text>
            {/if}
          {/each}

          <!-- Nodes -->
          {#each graph.nodes as node}
            {@const color = methodColor(node.method)}
            <g class="node-group" transform="translate({node.x},{node.y})">
              <!-- Card shadow -->
              <rect x="2" y="2" width={NODE_W} height={NODE_H} rx="8" fill="rgba(0,0,0,0.3)" />
              <!-- Card body -->
              <rect width={NODE_W} height={NODE_H} rx="8"
                fill="#161b22" stroke="#30363d" stroke-width="1.5" />
              <!-- Method accent strip -->
              <rect width="5" height={NODE_H} rx="8"
                fill={color} opacity="0.9" />
              <rect x="5" width="3" height={NODE_H} fill={color} opacity="0.9" />
              <!-- Method badge -->
              <rect x="12" y="10" width="42" height="16" rx="4"
                fill={color + "22"} />
              <text x="33" y="22" text-anchor="middle" font-size="9"
                font-weight="700" fill={color} font-family="monospace"
                letter-spacing="0.04em">{node.method}</text>
              <!-- Request name -->
              <text x="62" y="22" font-size="11" font-weight="600"
                fill="#e6edf3" font-family="sans-serif">
                {node.label.length > 14 ? node.label.slice(0, 13) + "…" : node.label}
              </text>
              <!-- URL -->
              <text x="12" y="42" font-size="9" fill="#8b949e" font-family="monospace">
                {node.url.replace(/https?:\/\/[^/]+/, "").slice(0, 26) || "/"}
              </text>
            </g>
          {/each}
        </svg>
      </div>

      <!-- Stats bar -->
      <div class="stats-bar">
        <span class="stat"><strong>{graph.nodes.length}</strong> requests</span>
        <span class="stat-sep">·</span>
        <span class="stat"><strong>{collection.folders.length}</strong> folders</span>
        <span class="stat-sep">·</span>
        <span class="stat"><strong>{graph.edges.length}</strong> variable {graph.edges.length === 1 ? "link" : "links"}</span>
        {#if graph.edges.length === 0}
          <span class="stat-hint">
            — Add <code>pm.environment.set()</code> in test scripts to see variable chains
          </span>
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .dashboard-section { max-width: 100%; display: flex; flex-direction: column; gap: 16px; }
  .section-header { margin-bottom: 4px; }
  .section-header h2 { font-size: 18px; font-weight: 700; margin-bottom: 4px; }
  .section-desc { font-size: 12px; color: var(--text-secondary); }

  .hint { font-size: 12px; color: var(--text-muted); }

  /* Collection picker */
  .col-bar { display: flex; gap: 6px; flex-wrap: wrap; }
  .col-pill {
    height: 26px; padding: 0 12px; font-size: 11px; font-weight: 600;
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: 13px; color: var(--text-secondary); cursor: pointer;
    transition: var(--transition-fast);
  }
  .col-pill:hover { border-color: var(--accent-primary); color: var(--text-primary); }
  .col-pill.active { background: var(--accent-primary-dim); border-color: var(--accent-primary); color: var(--accent-primary); }

  /* Legend */
  .legend-row {
    display: flex; align-items: center; gap: 10px; flex-wrap: wrap;
    padding: 8px 12px; background: var(--bg-surface);
    border: 1px solid var(--border-default); border-radius: var(--radius-md);
  }
  .legend-item { display: flex; align-items: center; gap: 4px; font-size: 10px; font-weight: 700; color: var(--text-muted); }
  .legend-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
  .legend-sep { flex: 1; }
  .legend-edge { font-size: 10px; color: #58a6ff; letter-spacing: 2px; }
  .legend-edge-label { font-size: 10px; color: var(--text-muted); }

  /* Flow canvas */
  .flow-scroll {
    overflow: auto;
    background: #0d1117;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    padding: 16px;
    min-height: 200px;
  }
  .flow-svg { display: block; }
  .node-group { cursor: default; }
  .node-group:hover rect:nth-child(2) { stroke: #58a6ff; }

  /* Stats */
  .stats-bar {
    display: flex; align-items: center; gap: 8px;
    padding: 8px 12px; background: var(--bg-surface);
    border: 1px solid var(--border-default); border-radius: var(--radius-md);
    font-size: 12px; color: var(--text-secondary);
  }
  .stats-bar strong { color: var(--text-primary); }
  .stat-sep { color: var(--text-muted); }
  .stat-hint { font-size: 11px; color: var(--text-muted); }
  .stat-hint code { color: var(--accent-secondary); font-family: var(--font-mono); font-size: 10px; }

  .empty-state {
    padding: 60px; text-align: center; font-size: 13px; color: var(--text-muted);
    background: var(--bg-surface); border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
  }
</style>
