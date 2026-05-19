import * as vscode from "vscode";
import * as path from "path";
import * as cp from "child_process";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import { CollectionProvider, CollectionRequest } from "./collectionProvider";
import { ResponsePanel } from "./responsePanel";

let client: LanguageClient | undefined;
let statusBarItem: vscode.StatusBarItem;

export async function activate(context: vscode.ExtensionContext) {
  // ── Find the parallax binary ────────────────────────────────────────────────
  const binary = findParallaxBinary();
  if (!binary) {
    vscode.window.showWarningMessage(
      "Parallax: could not find the `parallax` binary. Install it and ensure it is on your PATH."
    );
    return;
  }

  // ── Start LSP client ────────────────────────────────────────────────────────
  const serverOptions: ServerOptions = {
    command: binary,
    args: ["--lsp"],
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", language: "yaml", pattern: "**/.parallax/collections/**" },
      { scheme: "file", language: "yaml", pattern: "**/.parallax/environments/**" },
      { scheme: "file", language: "json", pattern: "**/.parallax/environments/**" },
    ],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/.parallax/**"),
    },
  };

  client = new LanguageClient("parallax", "Parallax LSP", serverOptions, clientOptions);
  await client.start();
  context.subscriptions.push(client);

  // ── Sidebar collection tree ─────────────────────────────────────────────────
  const provider = new CollectionProvider(client);
  vscode.window.registerTreeDataProvider("parallaxCollections", provider);

  // ── Status bar — active environment ────────────────────────────────────────
  statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  statusBarItem.command = "parallax.switchEnvironment";
  statusBarItem.text = "$(server) Parallax: default";
  statusBarItem.tooltip = "Click to switch Parallax environment";
  statusBarItem.show();
  context.subscriptions.push(statusBarItem);

  // ── Commands ────────────────────────────────────────────────────────────────
  context.subscriptions.push(
    vscode.commands.registerCommand("parallax.refreshCollections", () => {
      provider.refresh();
    }),

    vscode.commands.registerCommand(
      "parallax.sendRequest",
      async (item?: CollectionRequest) => {
        if (!item || !client) return;
        try {
          const result = await client.sendRequest("workspace/executeCommand", {
            command: "parallax.sendRequest",
            arguments: [item.collectionPath, item.requestName],
          });
          ResponsePanel.show(context, item.requestName, result as any);
        } catch (e) {
          vscode.window.showErrorMessage(`Parallax: request failed — ${e}`);
        }
      }
    ),

    vscode.commands.registerCommand("parallax.switchEnvironment", async () => {
      const envs = await discoverEnvironments();
      if (!envs.length) {
        vscode.window.showInformationMessage("No Parallax environments found in this workspace.");
        return;
      }
      const picked = await vscode.window.showQuickPick(envs, {
        placeHolder: "Select active environment",
      });
      if (picked) {
        statusBarItem.text = `$(server) Parallax: ${picked}`;
      }
    })
  );
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}

// ── Helpers ──────────────────────────────────────────────────────────────────

function findParallaxBinary(): string | undefined {
  // Check common install locations
  const candidates = ["parallax", "parallax-worker"];
  for (const name of candidates) {
    try {
      cp.execFileSync(name, ["--version"], { stdio: "ignore" });
      return name;
    } catch {}
  }
  return undefined;
}

async function discoverEnvironments(): Promise<string[]> {
  const folders = vscode.workspace.workspaceFolders;
  if (!folders) return [];
  const envDir = path.join(folders[0].uri.fsPath, ".parallax", "environments");
  try {
    const files = await vscode.workspace.fs.readDirectory(vscode.Uri.file(envDir));
    return files
      .filter(([name, type]) => type === vscode.FileType.File && name.endsWith(".json"))
      .map(([name]) => name.replace(".json", ""));
  } catch {
    return ["default"];
  }
}
