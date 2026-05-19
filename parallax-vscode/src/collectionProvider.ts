import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

interface CollectionInfo {
  name: string;
  path: string;
  requests: string[];
}

export class CollectionRequest extends vscode.TreeItem {
  constructor(
    public readonly requestName: string,
    public readonly collectionPath: string,
    label: string
  ) {
    super(label, vscode.TreeItemCollapsibleState.None);
    this.contextValue = "parallaxRequest";
    this.command = {
      command: "parallax.sendRequest",
      title: "Send Request",
      arguments: [this],
    };
    this.iconPath = new vscode.ThemeIcon("arrow-right");
    this.tooltip = `Send: ${label}`;
  }
}

class CollectionNode extends vscode.TreeItem {
  constructor(
    public readonly info: CollectionInfo
  ) {
    super(info.name, vscode.TreeItemCollapsibleState.Expanded);
    this.contextValue = "parallaxCollection";
    this.iconPath = new vscode.ThemeIcon("folder");
    this.description = `${info.requests.length} requests`;
  }
}

export class CollectionProvider implements vscode.TreeDataProvider<vscode.TreeItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<void>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private collections: CollectionInfo[] = [];

  constructor(private readonly client: LanguageClient) {
    this.load();
  }

  refresh() {
    this.load();
  }

  private async load() {
    try {
      const result = await this.client.sendRequest("workspace/executeCommand", {
        command: "parallax.listCollections",
        arguments: [],
      }) as CollectionInfo[] | null;
      this.collections = result ?? [];
    } catch {
      this.collections = [];
    }
    this._onDidChangeTreeData.fire();
  }

  getTreeItem(element: vscode.TreeItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: vscode.TreeItem): vscode.ProviderResult<vscode.TreeItem[]> {
    if (!element) {
      // Root: list collections
      if (this.collections.length === 0) {
        const empty = new vscode.TreeItem("No collections found");
        empty.description = "Open a workspace with a .parallax folder";
        return [empty];
      }
      return this.collections.map(c => new CollectionNode(c));
    }

    if (element instanceof CollectionNode) {
      return element.info.requests.map(name => {
        const parts = name.split("/");
        const label = parts[parts.length - 1];
        const method = extractMethod(label);
        const item = new CollectionRequest(name, element.info.path, label);
        if (method) {
          item.description = method;
          item.iconPath = new vscode.ThemeIcon(methodIcon(method));
        }
        return item;
      });
    }

    return [];
  }
}

function extractMethod(name: string): string {
  const m = name.match(/^(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\b/i);
  return m ? m[1].toUpperCase() : "";
}

function methodIcon(method: string): string {
  switch (method) {
    case "GET":    return "arrow-right";
    case "POST":   return "add";
    case "PUT":    return "edit";
    case "PATCH":  return "pencil";
    case "DELETE": return "trash";
    default:       return "circle-outline";
  }
}
