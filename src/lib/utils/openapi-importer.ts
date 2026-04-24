import type { Collection, CollectionRequest } from "../stores/app.svelte";

interface OASParam {
  name: string;
  in: string;
  required?: boolean;
  schema?: { type?: string; example?: unknown };
  example?: unknown;
}

interface OASOperation {
  operationId?: string;
  summary?: string;
  tags?: string[];
  parameters?: OASParam[];
  requestBody?: {
    content?: Record<string, { schema?: { example?: unknown; properties?: Record<string, unknown> } }>;
  };
  security?: unknown[];
}

interface OASSpec {
  openapi?: string;
  swagger?: string;
  info?: { title?: string; version?: string };
  paths?: Record<string, Record<string, OASOperation>>;
  servers?: { url: string }[];
  components?: {
    securitySchemes?: Record<string, { type: string; scheme?: string; name?: string; in?: string }>;
  };
}

const HTTP_METHODS = ["get", "post", "put", "patch", "delete", "head", "options"];

function guessBaseUrl(spec: OASSpec): string {
  if (spec.servers?.length) return spec.servers[0].url.replace(/\/$/, "");
  return "{{base_url}}";
}

function buildExampleFromSchema(schema: unknown): unknown {
  if (!schema || typeof schema !== "object") return null;
  const s = schema as Record<string, unknown>;
  if (s.example !== undefined) return s.example;
  if (s.type === "object" && s.properties) {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(s.properties as Record<string, unknown>)) {
      out[k] = buildExampleFromSchema(v);
    }
    return out;
  }
  if (s.type === "array" && s.items) return [buildExampleFromSchema(s.items)];
  if (s.type === "string") return s.enum ? (s.enum as string[])[0] : "string";
  if (s.type === "integer" || s.type === "number") return 0;
  if (s.type === "boolean") return false;
  return null;
}

export function importOpenAPI(raw: string, overrideName?: string): Collection {
  let spec: OASSpec;
  try {
    spec = JSON.parse(raw);
  } catch {
    // Try YAML-like: basic key:value (no real YAML parser, just JSON fallback)
    throw new Error("OpenAPI import requires JSON format. Convert YAML to JSON first.");
  }

  const version = spec.openapi ?? spec.swagger ?? "?";
  if (!version.startsWith("3") && !version.startsWith("2")) {
    throw new Error(`Unsupported OpenAPI version: ${version}`);
  }

  const colName = overrideName ?? spec.info?.title ?? "OpenAPI Import";
  const baseUrl = guessBaseUrl(spec);

  // Group operations by tag → folders
  const byTag: Record<string, CollectionRequest[]> = { __untagged__: [] };

  for (const [pathStr, pathItem] of Object.entries(spec.paths ?? {})) {
    for (const method of HTTP_METHODS) {
      const op = pathItem[method] as OASOperation | undefined;
      if (!op) continue;

      // Build URL with path params as template vars
      const url = `${baseUrl}${pathStr.replace(/\{(\w+)\}/g, "{{$1}}")}`;

      // Query params
      const queryParams: Record<string, string> = {};
      const headers: Record<string, string> = {};

      for (const p of op.parameters ?? []) {
        if (p.in === "query") {
          const ex = p.example ?? p.schema?.example ?? "";
          queryParams[p.name] = String(ex);
        } else if (p.in === "header") {
          headers[p.name] = String(p.example ?? "");
        }
      }

      // Body
      let body: CollectionRequest["body"] = undefined;
      if (op.requestBody?.content) {
        const ct = Object.keys(op.requestBody.content)[0] ?? "";
        const schema = op.requestBody.content[ct]?.schema;
        const example = schema?.example ?? buildExampleFromSchema(schema);
        if (ct.includes("json")) {
          body = { type: "json", content: example ?? {}, raw: JSON.stringify(example ?? {}, null, 2) };
        } else if (ct.includes("form")) {
          body = { type: "urlencoded", content: example ?? {}, raw: "" };
        } else {
          body = { type: "raw", content: {}, raw: "" };
        }
      }

      const req: CollectionRequest = {
        id: `oas-${method}-${pathStr.replace(/\W+/g, "-")}`,
        name: op.summary ?? op.operationId ?? `${method.toUpperCase()} ${pathStr}`,
        method: method.toUpperCase(),
        url,
        headers,
        params: queryParams,
        body,
        auth: { type: "none" },
      };

      const tag = op.tags?.[0] ?? "__untagged__";
      if (!byTag[tag]) byTag[tag] = [];
      byTag[tag].push(req);
    }
  }

  // Build folders
  const folders: Collection["folders"] = [];
  const rootRequests: CollectionRequest[] = [];

  for (const [tag, reqs] of Object.entries(byTag)) {
    if (tag === "__untagged__") {
      rootRequests.push(...reqs);
    } else {
      folders.push({ id: `folder-${tag}`, name: tag, requests: reqs });
    }
  }

  return {
    id: `oas-${Date.now()}`,
    name: colName,
    folders,
    requests: rootRequests,
    environments: [],
  };
}
