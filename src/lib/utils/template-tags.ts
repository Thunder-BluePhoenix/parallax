// Parallax Template Tag Engine
// Resolves both {% tag %} (Insomnia-style) and {{$var}} (Postman-style dynamic vars)

const ADJECTIVES = ["quick","brave","silent","cosmic","neon","golden","swift","dark","bright","wild"];
const NOUNS      = ["phoenix","nebula","comet","quasar","pulse","echo","prism","vortex","cipher","node"];
const DOMAINS    = ["example.com","test.io","mock.dev","api.local","parallax.dev"];
const FIRST      = ["Alice","Bob","Carol","Dave","Eve","Frank","Grace","Hana","Ivan","Jade"];
const LAST       = ["Smith","Jones","Chen","Patel","Kim","Müller","Rossi","Silva","Tanaka","Osei"];

function uuid(): string {
  return crypto.randomUUID();
}

function randomInt(min = 0, max = 100): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function randomEmail(): string {
  return `${FIRST[randomInt(0,9)].toLowerCase()}.${LAST[randomInt(0,9)].toLowerCase()}${randomInt(1,99)}@${DOMAINS[randomInt(0,4)]}`;
}

function randomName(): string {
  return `${FIRST[randomInt(0,9)]} ${LAST[randomInt(0,9)]}`;
}

function randomWord(): string {
  return `${ADJECTIVES[randomInt(0,9)]}-${NOUNS[randomInt(0,9)]}`;
}

function randomPhone(): string {
  return `+1-${randomInt(200,999)}-${randomInt(100,999)}-${randomInt(1000,9999)}`;
}

function base64Encode(value: string): string {
  return btoa(unescape(encodeURIComponent(value)));
}

function base64Decode(value: string): string {
  try { return decodeURIComponent(escape(atob(value))); } catch { return value; }
}

// ── Resolve a single {% ... %} tag ───────────────────────────────────────────
function resolveTag(inner: string, env: Record<string, string>): string {
  const parts = inner.trim().split(/\s+/);
  const tag   = parts[0];

  switch (tag) {
    case "uuid":
    case "guid":
      return uuid();

    case "timestamp":
      return String(Date.now());

    case "now": {
      const fmt = (parts[1] ?? "iso").replace(/['"]/g, "");
      if (fmt === "iso") return new Date().toISOString();
      if (fmt === "unix") return String(Math.floor(Date.now() / 1000));
      if (fmt === "ms")   return String(Date.now());
      return new Date().toISOString();
    }

    case "randomInt": {
      const lo = parseInt(parts[1] ?? "0", 10);
      const hi = parseInt(parts[2] ?? "100", 10);
      return String(randomInt(lo, hi));
    }

    case "randomEmail":
      return randomEmail();

    case "randomName":
      return randomName();

    case "randomWord":
      return randomWord();

    case "randomPhone":
      return randomPhone();

    case "randomBoolean":
      return String(Math.random() < 0.5);

    case "randomLoremIpsum":
      return "Lorem ipsum dolor sit amet consectetur adipiscing elit.";

    case "base64": {
      const op  = (parts[1] ?? "encode").replace(/['"]/g, "");
      const val = (parts[2] ?? "").replace(/['"]/g, "");
      return op === "decode" ? base64Decode(val) : base64Encode(val);
    }

    case "env": {
      const key = (parts[1] ?? "").replace(/['"]/g, "");
      return env[key] ?? `{% env '${key}' %}`;
    }

    default:
      return `{% ${inner} %}`; // unknown tag — leave as-is
  }
}

// ── Resolve Postman $-variable syntax ────────────────────────────────────────
function resolvePostmanDynamic(value: string): string {
  return value.replace(/\{\{\$([a-zA-Z]+)\}\}/g, (_, name) => {
    switch (name) {
      case "guid":           return uuid();
      case "timestamp":      return String(Math.floor(Date.now() / 1000));
      case "isoTimestamp":   return new Date().toISOString();
      case "randomInt":      return String(randomInt(0, 1000));
      case "randomEmail":    return randomEmail();
      case "randomBoolean":  return String(Math.random() < 0.5);
      case "randomWord":     return randomWord();
      case "randomFullName": return randomName();
      case "randomLoremIpsum": return "Lorem ipsum dolor sit amet.";
      case "randomPhoneNumber": return randomPhone();
      default:               return `{{$${name}}}`;
    }
  });
}

// ── Main resolver ─────────────────────────────────────────────────────────────
export function resolveTemplate(
  template: string,
  env: Record<string, string>,
): string {
  if (!template) return template;

  let result = template;

  // 1. {% tag args %} — Insomnia-style
  result = result.replace(/\{%([^%]+)%\}/g, (_, inner) => resolveTag(inner, env));

  // 2. {{$var}} — Postman dynamic variables
  result = resolvePostmanDynamic(result);

  // 3. {{var}} — environment variables
  result = result.replace(/\{\{([^}]+)\}\}/g, (match, key) => {
    const k = key.trim();
    return env.hasOwnProperty(k) ? env[k] : match;
  });

  return result;
}

// ── Resolve all fields of a request payload ───────────────────────────────────
export function resolveRequestTemplates(
  payload: Record<string, any>,
  env: Record<string, string>,
): Record<string, any> {
  const resolve = (v: any): any => {
    if (typeof v === "string") return resolveTemplate(v, env);
    if (Array.isArray(v))     return v.map(resolve);
    if (v && typeof v === "object") {
      return Object.fromEntries(Object.entries(v).map(([k, val]) => [k, resolve(val)]));
    }
    return v;
  };
  return resolve(payload);
}
