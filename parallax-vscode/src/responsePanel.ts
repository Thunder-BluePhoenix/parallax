import * as vscode from "vscode";

interface RequestResult {
  status:     number;
  body:       string;
  headers:    Record<string, string>;
  duration_ms: number;
}

export class ResponsePanel {
  private static current: ResponsePanel | undefined;
  private readonly panel: vscode.WebviewPanel;

  private constructor(context: vscode.ExtensionContext, title: string) {
    this.panel = vscode.window.createWebviewPanel(
      "parallaxResponse",
      `Parallax — ${title}`,
      vscode.ViewColumn.Beside,
      { enableScripts: true, retainContextWhenHidden: true }
    );
    this.panel.onDidDispose(() => { ResponsePanel.current = undefined; });
  }

  static show(context: vscode.ExtensionContext, title: string, result: RequestResult) {
    if (!ResponsePanel.current) {
      ResponsePanel.current = new ResponsePanel(context, title);
    } else {
      ResponsePanel.current.panel.title = `Parallax — ${title}`;
      ResponsePanel.current.panel.reveal(vscode.ViewColumn.Beside, true);
    }
    ResponsePanel.current.render(result);
  }

  private render(result: RequestResult) {
    const statusClass = result.status < 300 ? "ok" : result.status < 400 ? "redirect" : "error";
    const headersHtml = Object.entries(result.headers ?? {})
      .map(([k, v]) => `<tr><td class="hk">${esc(k)}</td><td class="hv">${esc(v)}</td></tr>`)
      .join("");

    let bodyHtml: string;
    try {
      const pretty = JSON.stringify(JSON.parse(result.body), null, 2);
      bodyHtml = `<pre class="body json">${esc(pretty)}</pre>`;
    } catch {
      bodyHtml = `<pre class="body raw">${esc(result.body ?? "")}</pre>`;
    }

    this.panel.webview.html = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
  body { font-family: var(--vscode-editor-font-family, monospace); font-size: 13px;
         background: var(--vscode-editor-background); color: var(--vscode-editor-foreground);
         margin: 0; padding: 16px; }
  .statusbar { display: flex; align-items: center; gap: 12px; margin-bottom: 16px;
               padding: 8px 12px; border-radius: 6px; background: var(--vscode-sideBar-background); }
  .status { font-weight: 700; font-size: 16px; }
  .status.ok       { color: #98c379; }
  .status.redirect { color: #d19a66; }
  .status.error    { color: #e06c75; }
  .timing { font-size: 11px; color: var(--vscode-descriptionForeground); }
  h3 { font-size: 11px; text-transform: uppercase; letter-spacing: .06em;
       color: var(--vscode-descriptionForeground); margin: 16px 0 6px; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  td { padding: 3px 6px; border-bottom: 1px solid var(--vscode-panel-border, #333); }
  .hk { color: var(--vscode-symbolIcon-propertyForeground, #9cdcfe); width: 40%; }
  .hv { color: var(--vscode-editor-foreground); word-break: break-all; }
  pre.body { margin: 0; padding: 12px; border-radius: 6px; overflow: auto;
             background: var(--vscode-sideBar-background); white-space: pre-wrap;
             word-break: break-all; max-height: 60vh; }
  pre.json { color: #abb2bf; }
</style>
</head>
<body>
  <div class="statusbar">
    <span class="status ${statusClass}">${result.status}</span>
    <span class="timing">${result.duration_ms}ms</span>
  </div>

  <h3>Headers</h3>
  <table>${headersHtml || "<tr><td colspan=2>No headers</td></tr>"}</table>

  <h3>Body</h3>
  ${bodyHtml}
</body>
</html>`;
  }
}

function esc(s: string): string {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
