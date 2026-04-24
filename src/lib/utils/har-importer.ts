// HAR 1.2 importer — converts a browser/proxy HTTP Archive into a Parallax collection
import type { Collection, CollectionRequest } from "../stores/app.svelte";

interface HarEntry {
  request: {
    method: string;
    url: string;
    headers: { name: string; value: string }[];
    queryString: { name: string; value: string }[];
    postData?: { mimeType: string; text?: string };
  };
  response?: { status: number };
  comment?: string;
}

interface Har {
  log: {
    entries: HarEntry[];
    pages?: { title?: string }[];
  };
}

const SKIP_HEADERS = new Set([
  "host", "content-length", "connection", "keep-alive",
  "transfer-encoding", "upgrade-insecure-requests",
]);

function bodyTypeFromMime(mime: string): string {
  if (mime.includes("application/json")) return "json";
  if (mime.includes("x-www-form-urlencoded")) return "urlencoded";
  if (mime.includes("multipart/form-data")) return "form";
  return "raw";
}

export function importHar(raw: string, collectionName?: string): Collection {
  const har: Har = JSON.parse(raw);
  const entries = har.log?.entries ?? [];
  const name = collectionName
    ?? har.log?.pages?.[0]?.title
    ?? "HAR Import";

  const requests: CollectionRequest[] = entries.map((entry, idx) => {
    const req = entry.request;
    const url = new URL(req.url);

    const headers: Record<string, string> = {};
    for (const h of req.headers ?? []) {
      if (!SKIP_HEADERS.has(h.name.toLowerCase())) {
        headers[h.name] = h.value;
      }
    }

    const params: Record<string, string> = {};
    for (const q of req.queryString ?? []) {
      params[q.name] = q.value;
    }
    // Strip query string from URL — params go into the params object
    url.search = "";
    const cleanUrl = url.toString();

    let body: CollectionRequest["body"] = null;
    if (req.postData?.text) {
      const mime = req.postData.mimeType ?? "";
      const btype = bodyTypeFromMime(mime);
      body = { type: btype, content: req.postData.text, raw: req.postData.text };
    }

    const id = `har_${Date.now()}_${idx}`;
    const pathLabel = url.pathname.split("/").filter(Boolean).pop() ?? url.pathname;
    const reqName = entry.comment || `${req.method} ${pathLabel}`;

    return {
      id,
      name: reqName,
      method: req.method,
      url: cleanUrl,
      headers,
      params: Object.keys(params).length > 0 ? params : undefined,
      body,
      auth: null,
      scripts: null,
    };
  });

  return {
    name,
    version: "1.0.0",
    description: `Imported from HAR — ${entries.length} requests`,
    requests,
    folders: [],
    variables: {},
  };
}
