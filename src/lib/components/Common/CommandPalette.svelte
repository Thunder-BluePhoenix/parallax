<script lang="ts">
  import { appMode, showRunner, loadedCollections, loadRequestIntoTab, activeRequest } from "../../stores/app.svelte";
  import { onDestroy } from "svelte";

  let { onclose }: { onclose: () => void } = $props();

  let query = $state("");
  let selected = $state(0);
  let inputEl: HTMLInputElement;

  interface Command {
    id: string;
    label: string;
    description?: string;
    icon: string;
    action: () => void;
    tags?: string[];
  }

  const staticCommands: Command[] = [
    {
      id: "mode-builder",
      label: "Switch to Builder",
      description: "Open the API request builder",
      icon: "⚡",
      action: () => { appMode.value = "builder"; onclose(); },
      tags: ["request", "api", "http"],
    },
    {
      id: "mode-dashboard",
      label: "Switch to Dashboard",
      description: "Open the monitoring dashboard",
      icon: "📊",
      action: () => { appMode.value = "dashboard"; onclose(); },
      tags: ["dashboard", "monitor", "traffic"],
    },
    {
      id: "mode-design",
      label: "Switch to Design",
      description: "Open the API design & spec editor",
      icon: "✏️",
      action: () => { appMode.value = "design"; onclose(); },
      tags: ["design", "openapi", "spec"],
    },
    {
      id: "runner",
      label: "Open Collection Runner",
      description: "Run a collection of requests",
      icon: "▶",
      action: () => { showRunner.value = true; onclose(); },
      tags: ["run", "test", "collection"],
    },
    {
      id: "new-request",
      label: "New Request",
      description: "Start a blank request",
      icon: "+",
      action: () => {
        appMode.value = "builder";
        loadRequestIntoTab({
          id: "new-" + Date.now(),
          name: "New Request",
          method: "GET",
          url: "",
          headers: {},
          params: {},
          auth: { type: "none" },
        });
        onclose();
      },
      tags: ["new", "create", "blank"],
    },
  ];

  // Dynamically add requests from loaded collections
  const requestCommands = $derived.by((): Command[] => {
    const cmds: Command[] = [];
    for (const col of loadedCollections) {
      for (const req of col.requests ?? []) {
        cmds.push({
          id: `req-${col.name}-${req.id}`,
          label: req.name,
          description: `${req.method} · ${col.name}`,
          icon: methodIcon(req.method),
          action: () => { loadRequestIntoTab(req); appMode.value = "builder"; onclose(); },
          tags: [req.method.toLowerCase(), col.name.toLowerCase(), req.url],
        });
      }
      for (const folder of col.folders ?? []) {
        for (const req of folder.requests ?? []) {
          cmds.push({
            id: `req-${col.name}-${folder.name}-${req.id}`,
            label: req.name,
            description: `${req.method} · ${col.name} / ${folder.name}`,
            icon: methodIcon(req.method),
            action: () => { loadRequestIntoTab(req); appMode.value = "builder"; onclose(); },
            tags: [req.method.toLowerCase(), col.name.toLowerCase(), req.url],
          });
        }
      }
    }
    return cmds;
  });

  const allCommands = $derived([...staticCommands, ...requestCommands]);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return allCommands.slice(0, 12);
    return allCommands.filter(c =>
      c.label.toLowerCase().includes(q) ||
      c.description?.toLowerCase().includes(q) ||
      c.tags?.some(t => t.includes(q))
    ).slice(0, 12);
  });

  $effect(() => {
    // reset selection when results change
    selected = 0;
  });

  function methodIcon(m: string) {
    const map: Record<string, string> = {
      GET: "●", POST: "●", PUT: "●", PATCH: "●", DELETE: "●",
    };
    return map[m.toUpperCase()] ?? "●";
  }

  function methodColor(desc?: string): string {
    if (!desc) return "var(--text-muted)";
    if (desc.startsWith("GET"))    return "var(--method-get)";
    if (desc.startsWith("POST"))   return "var(--method-post)";
    if (desc.startsWith("PUT"))    return "var(--method-put)";
    if (desc.startsWith("PATCH"))  return "var(--method-patch)";
    if (desc.startsWith("DELETE")) return "var(--method-delete)";
    return "var(--accent-primary)";
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "ArrowDown") { e.preventDefault(); selected = Math.min(selected + 1, filtered.length - 1); }
    else if (e.key === "ArrowUp") { e.preventDefault(); selected = Math.max(selected - 1, 0); }
    else if (e.key === "Enter") { e.preventDefault(); filtered[selected]?.action(); }
    else if (e.key === "Escape") onclose();
  }
</script>

<div class="palette-backdrop" onclick={onclose} role="presentation">
  <div class="palette" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="Command palette">
    <div class="palette-input-row">
      <svg class="palette-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      <input
        bind:this={inputEl}
        class="palette-input mono"
        type="text"
        placeholder="Search commands or requests…"
        bind:value={query}
        onkeydown={onKeyDown}
        autofocus
      />
      <kbd class="palette-esc">Esc</kbd>
    </div>

    {#if filtered.length > 0}
      <div class="palette-results" role="listbox">
        {#each filtered as cmd, i}
          <button
            class="palette-item"
            class:active={i === selected}
            onclick={cmd.action}
            onmouseenter={() => selected = i}
            role="option"
            aria-selected={i === selected}
          >
            <span class="palette-item-icon" style="color: {methodColor(cmd.description)}">{cmd.icon}</span>
            <span class="palette-item-label">{cmd.label}</span>
            {#if cmd.description}
              <span class="palette-item-desc">{cmd.description}</span>
            {/if}
          </button>
        {/each}
      </div>
    {:else}
      <div class="palette-empty">No results for "<strong>{query}</strong>"</div>
    {/if}

    <div class="palette-footer">
      <span><kbd>↑↓</kbd> navigate</span>
      <span><kbd>↵</kbd> select</span>
      <span><kbd>Esc</kbd> close</span>
    </div>
  </div>
</div>

<style>
  .palette-backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.55); backdrop-filter: blur(4px);
    z-index: 9999; display: flex; align-items: flex-start;
    justify-content: center; padding-top: 15vh;
  }

  .palette {
    width: 560px; max-width: 95vw;
    background: var(--bg-elevated);
    border: 1px solid var(--border-accent);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg), 0 0 40px rgba(124,110,255,0.15);
    overflow: hidden;
    animation: palette-in 120ms ease;
  }

  @keyframes palette-in {
    from { opacity: 0; transform: scale(0.96) translateY(-8px); }
    to   { opacity: 1; transform: scale(1) translateY(0); }
  }

  .palette-input-row {
    display: flex; align-items: center; gap: 8px;
    padding: 12px 16px; border-bottom: 1px solid var(--border-default);
  }
  .palette-icon { color: var(--text-muted); flex-shrink: 0; }
  .palette-input {
    flex: 1; background: transparent; border: none; outline: none;
    color: var(--text-primary); font-size: 14px;
    caret-color: var(--accent-primary);
  }
  .palette-input::placeholder { color: var(--text-muted); }
  .palette-esc {
    font-size: 9px; padding: 2px 5px;
    background: var(--bg-overlay); border: 1px solid var(--border-default);
    border-radius: 3px; color: var(--text-muted);
  }

  .palette-results { max-height: 360px; overflow-y: auto; }
  .palette-item {
    width: 100%; display: flex; align-items: center; gap: 10px;
    padding: 9px 16px; text-align: left;
    border-bottom: 1px solid var(--border-subtle);
    transition: background var(--transition-fast);
    cursor: pointer; background: transparent;
  }
  .palette-item:last-child { border-bottom: none; }
  .palette-item.active, .palette-item:hover { background: var(--accent-primary-dim); }
  .palette-item-icon { font-size: 10px; flex-shrink: 0; width: 14px; text-align: center; }
  .palette-item-label { font-size: 13px; color: var(--text-primary); flex: 1; }
  .palette-item-desc { font-size: 11px; color: var(--text-muted); white-space: nowrap; }

  .palette-empty {
    padding: 24px; text-align: center;
    font-size: 12px; color: var(--text-muted);
  }
  .palette-empty strong { color: var(--text-secondary); }

  .palette-footer {
    display: flex; gap: 16px; padding: 6px 16px;
    border-top: 1px solid var(--border-subtle);
    font-size: 10px; color: var(--text-muted);
    background: var(--bg-surface);
  }
  .palette-footer kbd {
    padding: 1px 4px; background: var(--bg-elevated);
    border: 1px solid var(--border-default); border-radius: 3px;
    font-family: var(--font-mono); font-size: 9px;
  }
</style>
