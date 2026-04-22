// Postman Collection v2.1 → Parallax Collection converter
import type { Collection, CollectionFolder, CollectionRequest } from "../stores/app.svelte";

interface PostmanItem {
  name?: string;
  request?: {
    method?: string;
    url?: { raw?: string } | string;
    header?: { key: string; value: string; disabled?: boolean }[];
    body?: {
      mode?: string;
      raw?: string;
      urlencoded?: { key: string; value: string; disabled?: boolean }[];
      formdata?: { key: string; value: string; type?: string; disabled?: boolean }[];
      graphql?: { query?: string; variables?: string };
    };
    auth?: {
      type?: string;
      bearer?: { key: string; value: string }[];
      basic?: { key: string; value: string }[];
      apikey?: { key: string; value: string }[];
    };
  };
  item?: PostmanItem[]; // folder
}

interface PostmanCollection {
  info?: { name?: string; description?: string };
  item?: PostmanItem[];
  variable?: { key: string; value: string }[];
}

function extractUrl(url: any): string {
  if (!url) return "";
  if (typeof url === "string") return url;
  return url.raw ?? "";
}

function extractHeaders(headers?: { key: string; value: string; disabled?: boolean }[]): Record<string, string> {
  const result: Record<string, string> = {};
  for (const h of headers ?? []) {
    if (!h.disabled && h.key) result[h.key] = h.value ?? "";
  }
  return result;
}

function extractBody(body?: any): CollectionRequest["body"] {
  if (!body) return null;
  switch (body.mode) {
    case "raw":
      return { type: "json", content: null, raw: body.raw ?? "" };
    case "urlencoded": {
      const pairs = Object.fromEntries(
        (body.urlencoded ?? []).filter((p: any) => !p.disabled).map((p: any) => [p.key, p.value])
      );
      return { type: "urlencoded", content: pairs, raw: JSON.stringify(pairs) };
    }
    case "formdata": {
      const pairs = Object.fromEntries(
        (body.formdata ?? []).filter((p: any) => !p.disabled).map((p: any) => [p.key, p.value])
      );
      return { type: "form", content: pairs, raw: JSON.stringify(pairs) };
    }
    case "graphql":
      return {
        type: "graphql",
        content: { query: body.graphql?.query ?? "", variables: body.graphql?.variables ?? "" },
        raw: body.graphql?.query ?? "",
      };
    default:
      return null;
  }
}

function extractAuth(auth?: any): CollectionRequest["auth"] {
  if (!auth || auth.type === "noauth") return null;
  const get = (arr: { key: string; value: string }[] | undefined, k: string) =>
    arr?.find(x => x.key === k)?.value ?? "";

  switch (auth.type) {
    case "bearer":
      return { auth_type: "bearer", token: get(auth.bearer, "token") };
    case "basic":
      return { auth_type: "basic", username: get(auth.basic, "username"), password: get(auth.basic, "password") };
    case "apikey":
      return { auth_type: "api_key", api_key_header: get(auth.apikey, "key"), api_key_value: get(auth.apikey, "value") };
    default:
      return null;
  }
}

function convertItem(item: PostmanItem): CollectionRequest | null {
  if (!item.request) return null;
  const req = item.request;
  return {
    id:      crypto.randomUUID(),
    name:    item.name ?? "Untitled",
    method:  (req.method ?? "GET").toUpperCase(),
    url:     extractUrl(req.url),
    headers: extractHeaders(req.header),
    body:    extractBody(req.body),
    auth:    extractAuth(req.auth),
  };
}

export function importPostmanCollection(json: string): Collection {
  let raw: PostmanCollection;
  try { raw = JSON.parse(json); }
  catch { throw new Error("Invalid JSON — not a Postman collection"); }

  if (!raw.item) throw new Error("Not a valid Postman collection (missing 'item' array)");

  const requests: CollectionRequest[] = [];
  const folders: CollectionFolder[] = [];

  for (const item of raw.item ?? []) {
    if (item.item) {
      // Folder
      const folderRequests: CollectionRequest[] = [];
      for (const child of item.item) {
        const converted = convertItem(child);
        if (converted) folderRequests.push(converted);
      }
      folders.push({ name: item.name ?? "Folder", requests: folderRequests });
    } else {
      const converted = convertItem(item);
      if (converted) requests.push(converted);
    }
  }

  // Extract collection-level variables as an environment hint
  const envHint: Record<string, string> = {};
  for (const v of raw.variable ?? []) {
    if (v.key) envHint[v.key] = v.value ?? "";
  }

  return {
    name:        (raw.info?.name ?? "Imported Collection").toLowerCase().replace(/\s+/g, "-"),
    version:     "1.0.0",
    description: raw.info?.description ?? undefined,
    requests,
    folders,
    _envHint:    envHint, // caller can use this to seed an environment
  } as any;
}

export function importInsomniaExport(json: string): Collection {
  let raw: any;
  try { raw = JSON.parse(json); }
  catch { throw new Error("Invalid JSON — not an Insomnia export"); }

  const resources: any[] = raw.resources ?? raw.__export_format === 4 ? raw.resources : [];
  if (!resources.length) throw new Error("Not a valid Insomnia export");

  const workspaceResource = resources.find((r: any) => r._type === "workspace");
  const groups = resources.filter((r: any) => r._type === "request_group");
  const reqs   = resources.filter((r: any) => r._type === "request");

  function convertInsomnia(r: any): CollectionRequest {
    const headers: Record<string, string> = {};
    for (const h of r.headers ?? []) {
      if (!h.disabled && h.name) headers[h.name] = h.value ?? "";
    }

    let body: CollectionRequest["body"] = null;
    if (r.body?.mimeType?.includes("json") || r.body?.mimeType === "") {
      body = { type: "json", content: null, raw: r.body?.text ?? "" };
    } else if (r.body?.mimeType?.includes("x-www-form-urlencoded")) {
      const pairs = Object.fromEntries((r.body?.params ?? []).map((p: any) => [p.name, p.value]));
      body = { type: "urlencoded", content: pairs, raw: JSON.stringify(pairs) };
    } else if (r.body?.mimeType?.includes("graphql")) {
      body = { type: "graphql", content: { query: r.body?.text ?? "", variables: r.body?.graphqlVariables ?? "" }, raw: r.body?.text ?? "" };
    }

    return {
      id:      r._id ?? crypto.randomUUID(),
      name:    r.name ?? "Request",
      method:  r.method ?? "GET",
      url:     r.url ?? "",
      headers,
      body,
      auth:    null,
    };
  }

  const rootRequests = reqs.filter((r: any) => !groups.find((g: any) => g._id === r.parentId));
  const folders: CollectionFolder[] = groups.map((g: any) => ({
    name:     g.name ?? "Folder",
    requests: reqs.filter((r: any) => r.parentId === g._id).map(convertInsomnia),
  }));

  return {
    name:        (workspaceResource?.name ?? "Imported from Insomnia").toLowerCase().replace(/\s+/g, "-"),
    version:     "1.0.0",
    requests:    rootRequests.map(convertInsomnia),
    folders,
  };
}
