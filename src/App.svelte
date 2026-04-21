<script lang="ts">
  import Titlebar from "./lib/components/Titlebar.svelte";
  import Sidebar from "./lib/components/Sidebar/Sidebar.svelte";
  import BuilderMode from "./lib/components/RequestBuilder/BuilderMode.svelte";
  import DashboardMode from "./lib/components/Dashboard/DashboardMode.svelte";
  import { appMode, currentWorkspace } from "./lib/stores/app.svelte";

  let isDashboard = $derived(appMode.value === "dashboard");
</script>

<div class="app-shell">
  <Titlebar />

  <div class="app-body">
    <Sidebar />

    <div class="main-content">
      {#if isDashboard}
        <DashboardMode />
      {:else}
        <BuilderMode />
      {/if}
    </div>
  </div>
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
