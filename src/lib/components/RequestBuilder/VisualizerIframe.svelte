<script lang="ts">
  import Handlebars from "handlebars";
  import { visualizerData } from "../../stores/app.svelte";

  let iframeContent = $derived.by(() => {
    if (!visualizerData.template) return "";
    try {
      const template = Handlebars.compile(visualizerData.template);
      const renderedHtml = template(visualizerData.data ?? {});
      // Wrap it in a basic HTML structure so it renders nicely inside the iframe
      return `
        <!DOCTYPE html>
        <html>
          <head>
            <meta charset="utf-8">
            <style>
              body {
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                margin: 0;
                padding: 12px;
                color: #24292f;
                background: #ffffff;
                font-size: 13px;
                line-height: 1.5;
              }
              /* Dark mode support if we want it */
              @media (prefers-color-scheme: dark) {
                body {
                  color: #c9d1d9;
                  background: #0d1117;
                }
              }
            </style>
          </head>
          <body>
            ${renderedHtml}
          </body>
        </html>
      `;
    } catch (err: any) {
      return `
        <div style="color: #cf222e; font-family: monospace; padding: 12px;">
          <strong>Visualizer Error:</strong><br/>
          ${err?.message ?? String(err)}
        </div>
      `;
    }
  });
</script>

<div class="visualizer-container">
  {#if visualizerData.template}
    <iframe
      class="visualizer-iframe"
      title="Response Visualizer"
      sandbox="allow-scripts"
      srcdoc={iframeContent}
    ></iframe>
  {:else}
    <div class="visualizer-empty">
      <p>No visualizer template provided.</p>
    </div>
  {/if}
</div>

<style>
  .visualizer-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    background: var(--bg-surface);
  }

  .visualizer-iframe {
    flex: 1;
    width: 100%;
    height: 100%;
    border: none;
    background: transparent;
  }

  .visualizer-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: 12px;
  }
</style>
