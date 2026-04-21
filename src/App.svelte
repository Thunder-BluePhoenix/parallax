<script lang="ts">
  import Titlebar from "./lib/components/Titlebar.svelte";
  import Sidebar from "./lib/components/Sidebar/Sidebar.svelte";
  import BuilderMode from "./lib/components/RequestBuilder/BuilderMode.svelte";
  import DashboardMode from "./lib/components/Dashboard/DashboardMode.svelte";
  import CollectionRunner from "./lib/components/Runner/CollectionRunner.svelte";
  import { onMount } from "svelte";
  import { appMode, showRunner } from "./lib/stores/app.svelte";

  let isDashboard = $derived(appMode.value === "dashboard");

  onMount(() => {
    console.log("[App] Parallax initializing...");
    const isTauri = !!(window as any).__TAURI__;
    console.log("[App] Tauri detected:", isTauri);
  });
</script>

<div class="app-shell">
  <Titlebar />

  <div class="app-body">
    <Sidebar />

    <div class="main-content">
      {#if isDashboard}
        <div style="display:none">{console.log("[App] Rendering DashboardMode")}</div>
        <DashboardMode />
      {:else}
        <div style="display:none">{console.log("[App] Rendering BuilderMode")}</div>
        <BuilderMode />
      {/if}
    </div>
  </div>

  {#if showRunner.value}
    <div style="display:none">{console.log("[App] Rendering CollectionRunner")}</div>
    <CollectionRunner onClose={() => (showRunner.value = false)} />
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
