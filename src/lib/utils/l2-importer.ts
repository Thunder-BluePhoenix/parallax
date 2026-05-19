// Lama2 .l2 file / folder → Parallax Collection importer
import type { Collection, CollectionRequest } from "../stores/app.svelte";

function generateId(): string {
  return Math.random().toString(36).slice(2, 10);
}

// Convert ${VAR} → {{VAR}}
function convertVars(s: string): string {
  return s.replace(/\$\{([^}]+)\}/g, "{{$1}}");
}

interface L2Block {
  preJs: string;   // JS before first ---
  request: string; // raw request block between first and second ---
  postJs: string;  // JS after second ---
}

function splitBlocks(src: string): L2Block {
  const parts = src.split(/^---\s*$/m);
  if (parts.length === 1) {
    // No separators — treat whole file as request block
    return { preJs: "", request: parts[0], postJs: "" };
  }
  if (parts.length === 2) {
    // One separator: could be pre-JS --- request or request --- post-JS
    // Heuristic: if first part looks like a request (starts with HTTP verb), treat as request
    const first = parts[0].trim();
    const httpVerbs = /^(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\s/i;
    if (httpVerbs.test(first)) {
      return { preJs: "", request: parts[0], postJs: parts[1] };
    }
    return { preJs: parts[0], request: parts[1], postJs: "" };
  }
  // 3+ parts: preJs --- request --- postJs
  return { preJs: parts[0], request: parts[1], postJs: parts.slice(2).join("\n---\n") };
}

function parseRequestBlock(block: string): Partial<CollectionRequest> | null {
  const lines = block.split("\n").map(l => l.trimEnd());
  let i = 0;

  // Skip blank lines
  while (i < lines.length && !lines[i].trim()) i++;
  if (i >= lines.length) return null;

  // METHOD
  const methodMatch = lines[i].trim().match(/^(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\b/i);
  if (!methodMatch) return null;
  const method = methodMatch[1].toUpperCase();
  i++;

  // Skip blank
  while (i < lines.length && !lines[i].trim()) i++;
  if (i >= lines.length) return null;

  // URL
  const url = convertVars(lines[i].trim());
  i++;

  // Headers — lines matching "Key: value" before the first blank line or body
  const headers: Record<string, string> = {};
  while (i < lines.length && lines[i].trim() && !lines[i].trim().startsWith("{") && !lines[i].trim().startsWith("[")) {
    const headerMatch = lines[i].match(/^([A-Za-z0-9_\-]+)\s*:\s*(.+)$/);
    if (headerMatch) {
      headers[headerMatch[1]] = convertVars(headerMatch[2].replace(/^["']|["']$/g, ""));
    }
    i++;
  }

  // Skip blank
  while (i < lines.length && !lines[i].trim()) i++;

  // Body — everything remaining
  const bodyLines = lines.slice(i);
  let bodyContent = convertVars(bodyLines.join("\n").trim());
  let bodyType: CollectionRequest["body"] = null;

  if (bodyContent) {
    // Detect varjson: lines of key=value (no braces)
    const isVarJson = bodyContent.split("\n").every(l => !l.trim() || /^[\w-]+=/.test(l.trim()));
    if (isVarJson && !bodyContent.startsWith("{") && !bodyContent.startsWith("[")) {
      // Convert varjson key=value pairs to JSON
      const obj: Record<string, string> = {};
      for (const line of bodyContent.split("\n")) {
        const m = line.trim().match(/^([\w-]+)=(.*)$/);
        if (m) obj[m[1]] = m[2];
      }
      const jsonStr = JSON.stringify(obj, null, 2);
      bodyType = { type: "json", content: obj, raw: jsonStr };
    } else {
      try {
        const parsed = JSON.parse(bodyContent);
        bodyType = { type: "json", content: parsed, raw: bodyContent };
      } catch {
        bodyType = { type: "raw", content: null, raw: bodyContent };
      }
    }
  }

  return {
    id: generateId(),
    name: `${method} ${url.replace(/https?:\/\/[^/]+/, "") || "/"}`,
    method,
    url,
    headers,
    params: {},
    body: bodyType,
    auth: { type: "none" },
  };
}

// Convert post-JS block to a Parallax test script (best-effort translation)
function convertPostJs(js: string): string {
  if (!js.trim()) return "";
  // Replace result["json"]["field"] with pm.response.json().field
  let out = js
    .replace(/result\["json"\]\["([^"]+)"\]/g, 'pm.response.json().$1')
    .replace(/result\["([^"]+)"\]/g, 'pm.response.json().$1');
  // Wrap variable captures: let token = ... → pm.environment.set("token", ...)
  out = out.replace(/^let\s+(\w+)\s*=\s*(.+)$/gm, (_, name, val) =>
    `pm.environment.set("${name}", ${val.replace(/;$/, "")});`
  );
  return `// Converted from Lama2 post-JS block\n${out}`;
}

// Convert pre-JS block to a pre-request script (best-effort)
function convertPreJs(js: string): string {
  if (!js.trim()) return "";
  let out = js
    .replace(/let\s+(\w+)\s*=\s*(.+);?/g, (_, name, val) =>
      `pm.environment.set("${name}", ${val.replace(/;$/, "")});`
    );
  return `// Converted from Lama2 pre-JS block\n${out}`;
}

export function importL2File(src: string, name: string): CollectionRequest | null {
  const { preJs, request, postJs } = splitBlocks(src);
  const req = parseRequestBlock(request);
  if (!req) return null;

  return {
    id: generateId(),
    name: name.replace(/\.l2$/, "") || req.name || "Imported request",
    method: req.method ?? "GET",
    url: req.url ?? "",
    headers: req.headers ?? {},
    params: req.params ?? {},
    body: req.body ?? null,
    auth: req.auth ?? { type: "none" },
    preRequestScript: convertPreJs(preJs),
    testScript: convertPostJs(postJs),
  } as CollectionRequest;
}

// Parse a simple KEY=VALUE env file (l2.env / l2config.env)
export function importL2Env(src: string): Record<string, string> {
  const env: Record<string, string> = {};
  for (const line of src.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    // Skip backtick substitution lines (Phase 7)
    if (trimmed.includes("`")) continue;
    const match = trimmed.replace(/^export\s+/, "").match(/^([\w]+)=(.*)$/);
    if (match) {
      env[match[1]] = match[2].replace(/^["']|["']$/g, "");
    }
  }
  return env;
}

export function importL2Collection(files: { name: string; content: string }[], colName: string): Collection {
  const requests: CollectionRequest[] = [];
  const folders: Record<string, CollectionRequest[]> = {};

  for (const file of files) {
    if (!file.name.endsWith(".l2")) continue;
    const parts = file.name.split("/");
    const req = importL2File(file.content, parts[parts.length - 1]);
    if (!req) continue;

    if (parts.length > 1) {
      const folder = parts[parts.length - 2];
      if (!folders[folder]) folders[folder] = [];
      folders[folder].push(req);
    } else {
      requests.push(req);
    }
  }

  const folderList = Object.entries(folders).map(([name, reqs]) => ({
    id: generateId(),
    name,
    requests: reqs,
  }));

  return {
    name: colName,
    version: "1",
    folders: folderList,
    requests,
    variables: {},
  };
}
