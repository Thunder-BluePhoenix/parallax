<script lang="ts">
  import Titlebar from "./lib/components/Titlebar.svelte";
  import Sidebar from "./lib/components/Sidebar/Sidebar.svelte";
  import BuilderMode from "./lib/components/RequestBuilder/BuilderMode.svelte";
  import DashboardMode from "./lib/components/Dashboard/DashboardMode.svelte";
  import DesignMode from "./lib/components/Design/DesignMode.svelte";
  import CollectionRunner from "./lib/components/Runner/CollectionRunner.svelte";
  import CommandPalette from "./lib/components/Common/CommandPalette.svelte";
  import { onMount } from "svelte";
  import { appMode, showRunner } from "./lib/stores/app.svelte";
  import { activeTheme, applyTheme } from "./lib/stores/theme.svelte";

  let showPalette = $state(false);

  onMount(() => {
    console.log("[App] Parallax initializing...");
    const isTauri = !!(window as any).__TAURI__;
    console.log("[App] Tauri detected:", isTauri);

    // Restore saved theme on boot
    applyTheme(activeTheme.id, activeTheme.customCss);

    function onKey(e: KeyboardEvent) {
      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;
      if (e.key === "k") { e.preventDefault(); showPalette = !showPalette; }
      else if (e.key === "1") { e.preventDefault(); appMode.value = "builder"; }
      else if (e.key === "2") { e.preventDefault(); appMode.value = "dashboard"; }
      else if (e.key === "3") { e.preventDefault(); appMode.value = "design"; }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });
</script>

<div class="app-shell">
  <Titlebar />

  <div class="app-body">
    <Sidebar />

    <div class="main-content">
      {#if appMode.value === "dashboard"}
        <DashboardMode />
      {:else if appMode.value === "design"}
        <DesignMode />
      {:else}
        <BuilderMode />
      {/if}
    </div>
  </div>

  {#if showRunner.value}
    <CollectionRunner onClose={() => (showRunner.value = false)} />
  {/if}

  {#if showPalette}
    <CommandPalette onclose={() => (showPalette = false)} />
  {/if}
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: var(--bg-base);
  }

  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
