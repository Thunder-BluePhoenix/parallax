// Export a Parallax Collection to Postman Collection v2.1 JSON
import type { Collection, CollectionRequest } from "../stores/app.svelte";

function buildAuth(auth: CollectionRequest["auth"]): any {
  if (!auth) return { type: "noauth" };
  switch (auth.auth_type) {
    case "bearer":
      return { type: "bearer", bearer: [{ key: "token", value: auth.token ?? "", type: "string" }] };
    case "basic":
      return { type: "basic", basic: [
        { key: "username", value: auth.username ?? "", type: "string" },
        { key: "password", value: auth.password ?? "", type: "string" },
      ]};
    case "api_key":
      return { type: "apikey", apikey: [
        { key: "key",   value: auth.api_key_header ?? "", type: "string" },
        { key: "value", value: auth.api_key_value  ?? "", type: "string" },
        { key: "in",    value: "header",                  type: "string" },
      ]};
    default:
      return { type: "noauth" };
  }
}

function buildBody(req: CollectionRequest): any {
  if (!req.body || req.body.type === "none") return undefined;
  switch (req.body.type) {
    case "json":
      return { mode: "raw", raw: req.body.raw ?? JSON.stringify(req.body.content, null, 2),
               options: { raw: { language: "json" } } };
    case "urlencoded":
      return { mode: "urlencoded", urlencoded: Object.entries(req.body.content ?? {}).map(([k, v]) =>
        ({ key: k, value: String(v), disabled: false })) };
    case "form":
      return { mode: "formdata", formdata: Object.entries(req.body.content ?? {}).map(([k, v]) =>
        ({ key: k, value: String(v), type: "text", disabled: false })) };
    case "graphql":
      return { mode: "graphql", graphql: {
        query: req.body.content?.query ?? req.body.raw ?? "",
        variables: req.body.content?.variables ?? "",
      }};
    default:
      return { mode: "raw", raw: req.body.raw ?? "" };
  }
}

function convertRequest(req: CollectionRequest): any {
  return {
    name: req.name,
    request: {
      method: req.method,
      header: Object.entries(req.headers ?? {}).map(([key, value]) => ({ key, value, disabled: false })),
      url: {
        raw:   req.url,
        protocol: req.url.split("://")[0] ?? "https",
        host: (req.url.split("://")[1] ?? "").split("/")[0]?.split(".") ?? [],
        path: (req.url.split("://")[1] ?? "").split("/").slice(1),
        query: Object.entries(req.params ?? {}).map(([key, value]) => ({ key, value, disabled: false })),
      },
      auth: buildAuth(req.auth),
      body: buildBody(req),
    },
    response: [],
  };
}

export function exportPostmanCollection(col: Collection): string {
  const items: any[] = [];

  for (const req of col.requests ?? []) {
    items.push(convertRequest(req));
  }

  for (const folder of col.folders ?? []) {
    items.push({
      name: folder.name,
      item: folder.requests.map(convertRequest),
    });
  }

  const postman = {
    info: {
      _postman_id: crypto.randomUUID(),
      name: col.name,
      description: col.description ?? "",
      schema: "https://schema.getpostman.com/json/collection/v2.1.0/collection.json",
    },
    item: items,
    variable: Object.entries(col.variables ?? {}).map(([key, value]) => ({ key, value })),
  };

  return JSON.stringify(postman, null, 2);
}
