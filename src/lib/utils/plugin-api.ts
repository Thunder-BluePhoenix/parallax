/**
 * Parallax Plugin API
 * Plugins are sandboxed JS modules that receive a restricted `parallax` context object.
 * They register tools available in pre-request / test scripts and in the UI.
 */

export interface PluginTool {
  name: string;
  description: string;
  fn: (...args: unknown[]) => unknown;
}

export interface PluginMeta {
  id: string;
  name: string;
  version: string;
  description: string;
  author?: string;
  builtin?: boolean;
}

export interface Plugin extends PluginMeta {
  tools: PluginTool[];
  enabled: boolean;
}

// ── Built-in plugin source strings ──────────────────────────────────────────

const BUILTIN_SOURCES: Record<string, string> = {

  "parallax-faker": `
(function(parallax) {
  const words = ["alpha","beta","gamma","delta","epsilon","zeta","eta","theta"];
  const tld   = ["com","io","dev","app","net","org"];
  const rand  = (arr) => arr[Math.floor(Math.random() * arr.length)];
  const randInt = (min, max) => Math.floor(Math.random() * (max - min + 1)) + min;

  parallax.registerTool("faker.word",      () => rand(words));
  parallax.registerTool("faker.email",     () => rand(words) + randInt(10,99) + "@" + rand(words) + "." + rand(tld));
  parallax.registerTool("faker.uuid",      () => "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, c => { const r=Math.random()*16|0; return (c==="x"?r:(r&0x3|0x8)).toString(16); }));
  parallax.registerTool("faker.int",       (min=1, max=1000) => randInt(Number(min), Number(max)));
  parallax.registerTool("faker.name",      () => rand(["Alice","Bob","Carol","Dave","Eve","Frank","Grace","Hank"]) + " " + rand(words).charAt(0).toUpperCase() + rand(words).slice(1));
  parallax.registerTool("faker.url",       () => "https://" + rand(words) + "." + rand(tld));
  parallax.registerTool("faker.phone",     () => "+1" + Array.from({length:10}, () => randInt(0,9)).join(""));
  parallax.registerTool("faker.sentence",  () => words.sort(() => 0.5 - Math.random()).slice(0,5).join(" ") + ".");
})
`,

  "parallax-jwt": `
(function(parallax) {
  function b64url(str) {
    return btoa(str).replace(/=/g,"").replace(/\\+/g,"-").replace(/\\//g,"_");
  }
  parallax.registerTool("jwt.decode", (token) => {
    const parts = String(token).split(".");
    if (parts.length < 2) return null;
    try {
      const pad = s => s + "===".slice((s.length + 3) % 4);
      return {
        header:  JSON.parse(atob(pad(parts[0].replace(/-/g,"+").replace(/_/g,"/")))),
        payload: JSON.parse(atob(pad(parts[1].replace(/-/g,"+").replace(/_/g,"/")))),
      };
    } catch { return null; }
  });
  parallax.registerTool("jwt.isExpired", (token) => {
    const d = parallax.tools["jwt.decode"](token);
    if (!d?.payload?.exp) return false;
    return d.payload.exp < Math.floor(Date.now() / 1000);
  });
  parallax.registerTool("jwt.payload", (token) => {
    const d = parallax.tools["jwt.decode"](token);
    return d?.payload ?? null;
  });
})
`,

  "parallax-base64": `
(function(parallax) {
  parallax.registerTool("base64.encode",     (s) => btoa(String(s)));
  parallax.registerTool("base64.decode",     (s) => atob(String(s)));
  parallax.registerTool("base64.encodeUrl",  (s) => btoa(String(s)).replace(/=/g,"").replace(/\\+/g,"-").replace(/\\//g,"_"));
  parallax.registerTool("base64.decodeUrl",  (s) => atob(String(s).replace(/-/g,"+").replace(/_/g,"/")));
})
`,

  "parallax-xml": `
(function(parallax) {
  parallax.registerTool("xml.parse", (xmlStr) => {
    const parser = new DOMParser();
    const doc = parser.parseFromString(String(xmlStr), "application/xml");
    const err = doc.querySelector("parsererror");
    if (err) throw new Error(err.textContent);
    function nodeToObj(node) {
      if (node.nodeType === 3) return node.textContent?.trim() || undefined;
      const obj = {};
      for (const child of node.childNodes) {
        if (child.nodeType === 3) { obj["_text"] = child.textContent?.trim(); continue; }
        const v = nodeToObj(child);
        if (child.nodeName in obj) {
          if (!Array.isArray(obj[child.nodeName])) obj[child.nodeName] = [obj[child.nodeName]];
          obj[child.nodeName].push(v);
        } else { obj[child.nodeName] = v; }
      }
      return obj;
    }
    return nodeToObj(doc.documentElement);
  });
  parallax.registerTool("xml.toJson", (xmlStr) => {
    return JSON.stringify(parallax.tools["xml.parse"](xmlStr), null, 2);
  });
})
`,

  "parallax-soap": `
(function(parallax) {
  parallax.registerTool("soap.envelope", (body, ns = "http://schemas.xmlsoap.org/soap/envelope/") => {
    return '<?xml version="1.0" encoding="utf-8"?>\\n' +
      '<soap:Envelope xmlns:soap="' + ns + '">\\n' +
      '  <soap:Body>\\n' +
      '    ' + String(body) + '\\n' +
      '  </soap:Body>\\n' +
      '</soap:Envelope>';
  });
  parallax.registerTool("soap.extractBody", (xmlStr) => {
    const m = String(xmlStr).match(/<(?:[^:>]+:)?Body[^>]*>([\\s\\S]*?)<\\/(?:[^:>]+:)?Body>/i);
    return m ? m[1].trim() : null;
  });
})
`,
};

// ── Plugin registry ──────────────────────────────────────────────────────────

const STORAGE_KEY = "parallax_plugins";

function loadEnabled(): Record<string, boolean> {
  try { return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}"); }
  catch { return {}; }
}

class PluginRegistry {
  private _plugins = $state<Plugin[]>([]);
  private _tools: Record<string, (...args: unknown[]) => unknown> = {};

  get plugins() { return this._plugins; }
  get tools() { return this._tools; }

  constructor() {
    const saved = loadEnabled();
    for (const [id, src] of Object.entries(BUILTIN_SOURCES)) {
      const plugin = this._loadPlugin(id, src, true);
      plugin.enabled = saved[id] !== false; // builtins on by default
      this._plugins.push(plugin);
      if (plugin.enabled) this._registerTools(plugin);
    }
  }

  private _loadPlugin(id: string, src: string, builtin: boolean): Plugin {
    const tools: PluginTool[] = [];
    const toolMap: Record<string, (...args: unknown[]) => unknown> = {};

    const ctx = {
      tools: toolMap,
      registerTool: (name: string, fn: (...args: unknown[]) => unknown) => {
        tools.push({ name, description: "", fn });
        toolMap[name] = fn;
      },
    };

    try {
      // eslint-disable-next-line no-new-func
      const factory = new Function(`return ${src.trim()}`)();
      factory(ctx);
    } catch (e) {
      console.warn(`[Plugin] Failed to load ${id}:`, e);
    }

    const label = id.replace("parallax-", "").replace(/-/g, " ");
    return {
      id, builtin, enabled: false,
      name: label.charAt(0).toUpperCase() + label.slice(1),
      version: "1.0.0",
      description: `Built-in ${label} utilities`,
      tools,
    };
  }

  private _registerTools(plugin: Plugin) {
    for (const t of plugin.tools) {
      this._tools[t.name] = t.fn;
    }
  }

  private _unregisterTools(plugin: Plugin) {
    for (const t of plugin.tools) {
      delete this._tools[t.name];
    }
  }

  toggle(id: string) {
    const p = this._plugins.find(p => p.id === id);
    if (!p) return;
    p.enabled = !p.enabled;
    if (p.enabled) this._registerTools(p);
    else this._unregisterTools(p);
    this._saveState();
  }

  /** Install a plugin from a JS source string provided by the user */
  installCustom(id: string, src: string) {
    const existing = this._plugins.findIndex(p => p.id === id);
    const plugin = this._loadPlugin(id, src, false);
    plugin.enabled = true;
    if (existing >= 0) {
      this._unregisterTools(this._plugins[existing]);
      this._plugins[existing] = plugin;
    } else {
      this._plugins.push(plugin);
    }
    this._registerTools(plugin);
    this._saveState();
  }

  private _saveState() {
    const state: Record<string, boolean> = {};
    for (const p of this._plugins) state[p.id] = p.enabled;
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(state)); } catch {}
  }
}

export const pluginRegistry = new PluginRegistry();

/** Call a plugin tool by name, returns undefined if not found */
export function callTool(name: string, ...args: unknown[]): unknown {
  return pluginRegistry.tools[name]?.(...args);
}
