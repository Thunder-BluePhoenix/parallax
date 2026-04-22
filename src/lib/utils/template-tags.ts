// Parallax Template Tag Engine
// Resolves both {% tag %} (Insomnia-style) and {{$var}} (Postman-style dynamic vars)

const ADJECTIVES = ["quick","brave","silent","cosmic","neon","golden","swift","dark","bright","wild"];
const NOUNS      = ["phoenix","nebula","comet","quasar","pulse","echo","prism","vortex","cipher","node"];
const DOMAINS    = ["example.com","test.io","mock.dev","api.local","parallax.dev"];
const FIRST      = ["Alice","Bob","Carol","Dave","Eve","Frank","Grace","Hana","Ivan","Jade"];
const LAST       = ["Smith","Jones","Chen","Patel","Kim","Müller","Rossi","Silva","Tanaka","Osei"];

function uuid(): string { return crypto.randomUUID(); }

function randomInt(min = 0, max = 100): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function randomEmail(): string {
  return `${FIRST[randomInt(0,9)].toLowerCase()}.${LAST[randomInt(0,9)].toLowerCase()}${randomInt(1,99)}@${DOMAINS[randomInt(0,4)]}`;
}
function randomName():  string { return `${FIRST[randomInt(0,9)]} ${LAST[randomInt(0,9)]}`; }
function randomWord():  string { return `${ADJECTIVES[randomInt(0,9)]}-${NOUNS[randomInt(0,9)]}`; }
function randomPhone(): string { return `+1-${randomInt(200,999)}-${randomInt(100,999)}-${randomInt(1000,9999)}`; }

function base64Encode(value: string): string {
  const bytes  = new TextEncoder().encode(value);
  const binStr = Array.from(bytes).map(b => String.fromCharCode(b)).join("");
  return btoa(binStr);
}

function base64Decode(value: string): string {
  try {
    const binStr = atob(value);
    const bytes  = Uint8Array.from(binStr, c => c.charCodeAt(0));
    return new TextDecoder().decode(bytes);
  } catch { return value; }
}

// ── Simple JSONPath-like extractor ($.key.nested[0]) ─────────────────────────
function jsonPath(obj: any, path: string): string {
  const segs = path.replace(/^\$\.?/, "").split(/\.|\[(\d+)\]/).filter(Boolean);
  let cur = obj;
  for (const seg of segs) {
    if (cur == null) return "";
    const idx = parseInt(seg, 10);
    cur = isNaN(idx) ? cur[seg] : cur[idx];
  }
  return cur == null ? "" : String(cur);
}

// ── Resolve a single {% ... %} tag ───────────────────────────────────────────
function resolveTag(
  inner:   string,
  env:     Record<string, string>,
  cache:   Record<string, any>,
): string {
  // Parse parts, respecting quoted strings
  const parts: string[] = [];
  const re = /[^\s"']+|"([^"]*)"|'([^']*)'/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(inner.trim())) !== null) {
    parts.push((m[1] ?? m[2] ?? m[0]).replace(/^['"]|['"]$/g, ""));
  }
  const tag = parts[0];

  switch (tag) {
    case "uuid":
    case "guid":        return uuid();
    case "timestamp":   return String(Date.now());
    case "now": {
      const fmt = parts[1] ?? "iso";
      if (fmt === "unix") return String(Math.floor(Date.now() / 1000));
      if (fmt === "ms")   return String(Date.now());
      return new Date().toISOString();
    }
    case "randomInt": {
      const lo = parseInt(parts[1] ?? "0", 10);
      const hi = parseInt(parts[2] ?? "100", 10);
      return String(randomInt(lo, hi));
    }
    case "randomEmail":      return randomEmail();
    case "randomName":       return randomName();
    case "randomWord":       return randomWord();
    case "randomPhone":      return randomPhone();
    case "randomBoolean":    return String(Math.random() < 0.5);
    case "randomLoremIpsum": return "Lorem ipsum dolor sit amet consectetur adipiscing elit.";
    case "base64": {
      const op  = parts[1] ?? "encode";
      const val = parts[2] ?? "";
      return op === "decode" ? base64Decode(val) : base64Encode(val);
    }
    case "env": {
      const key = parts[1] ?? "";
      return env[key] ?? `{% env '${key}' %}`;
    }
    // ── Request chaining (Insomnia's killer feature) ──────────────────────────
    // {% response 'body', '$.path.to.value', 'Request Name' %}
    // {% response 'header', 'Content-Type', 'Request Name' %}
    // {% response 'status', '', 'Request Name' %}
    case "response": {
      const field   = parts[1] ?? "body";   // body | header | status
      const path    = parts[2] ?? "$";
      const reqName = parts[3] ?? "";
      const cached  = reqName ? cache[reqName] : Object.values(cache)[0];
      if (!cached) return `{% response '${field}' '${path}' '${reqName}' %}`;

      if (field === "status") return String(cached.status ?? "");
      if (field === "header") {
        const h = cached.headers ?? {};
        const val = Object.entries(h).find(([k]) => k.toLowerCase() === path.toLowerCase())?.[1];
        return val ? String(val) : "";
      }
      // body — JSONPath extraction
      const json = cached.body?.json ?? (() => {
        try { return JSON.parse(cached.body?.raw ?? ""); } catch { return null; }
      })();
      if (!json) return cached.body?.raw ?? "";
      return jsonPath(json, path);
    }
    // ── Prompt user at send-time ───────────────────────────────────────────────
    case "prompt": {
      const label   = parts[1] ?? "Enter value";
      const defVal  = parts[2] ?? "";
      const result  = window.prompt(label, defVal);
      return result ?? defVal;
    }
    default:
      return `{% ${inner} %}`; // unknown tag — leave as-is
  }
}

// ── Resolve Postman $-variable syntax ────────────────────────────────────────
function resolvePostmanDynamic(value: string): string {
  return value.replace(/\{\{\$([a-zA-Z]+)\}\}/g, (_, name) => {
    switch (name) {
      case "guid":              return uuid();
      case "timestamp":         return String(Math.floor(Date.now() / 1000));
      case "isoTimestamp":      return new Date().toISOString();
      case "randomInt":         return String(randomInt(0, 1000));
      case "randomEmail":       return randomEmail();
      case "randomBoolean":     return String(Math.random() < 0.5);
      case "randomWord":        return randomWord();
      case "randomFullName":    return randomName();
      case "randomLoremIpsum":  return "Lorem ipsum dolor sit amet.";
      case "randomPhoneNumber": return randomPhone();
      default:                  return `{{$${name}}}`;
    }
  });
}

// ── Main resolver ─────────────────────────────────────────────────────────────
export function resolveTemplate(
  template: string,
  env:      Record<string, string>,
  cache:    Record<string, any> = {},
): string {
  if (!template) return template;
  let result = template;
  result = result.replace(/\{%([^%]+)%\}/g, (_, inner) => resolveTag(inner, env, cache));
  result = resolvePostmanDynamic(result);
  result = result.replace(/\{\{([^}$][^}]*)\}\}/g, (match, key) => {
    const k = key.trim();
    return Object.prototype.hasOwnProperty.call(env, k) ? env[k] : match;
  });
  return result;
}

// ── Resolve all fields of a request payload ───────────────────────────────────
export function resolveRequestTemplates(
  payload: Record<string, any>,
  env:     Record<string, string>,
  cache:   Record<string, any> = {},
): Record<string, any> {
  const resolve = (v: any): any => {
    if (typeof v === "string") return resolveTemplate(v, env, cache);
    if (Array.isArray(v))     return v.map(resolve);
    if (v && typeof v === "object")
      return Object.fromEntries(Object.entries(v).map(([k, val]) => [k, resolve(val)]));
    return v;
  };
  return resolve(payload);
}
