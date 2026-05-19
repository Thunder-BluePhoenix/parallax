// Parallax Platform Abstraction Layer
//
// Routes core operations to the right backend depending on the runtime:
//   - Desktop (Tauri): Rust commands via invoke()
//   - Browser (WASM):  Go WASM engine loaded from /parallax.wasm
//
// All call sites should import from this module instead of
// reaching for @tauri-apps/api/core directly.

const IS_TAURI = typeof window !== "undefined" && !!(window as any).__TAURI__;

// ── WASM loader ───────────────────────────────────────────────────────────────

let wasmReady: Promise<void> | null = null;

function ensureWasm(): Promise<void> {
  if (wasmReady) return wasmReady;
  wasmReady = (async () => {
    if (typeof (window as any).parallaxSendRequest === "function") return;

    // Load the Go WASM support script (sets up Go runtime globals)
    await new Promise<void>((resolve, reject) => {
      const script = document.createElement("script");
      script.src = "/wasm_exec.js";
      script.onload = () => resolve();
      script.onerror = reject;
      document.head.appendChild(script);
    });

    const go = new (window as any).Go();
    const result = await WebAssembly.instantiateStreaming(fetch("/parallax.wasm"), go.importObject);
    go.run(result.instance); // does not resolve — WASM goroutine runs forever
  })();
  return wasmReady;
}

// ── sendRequest ───────────────────────────────────────────────────────────────

export async function sendRequest(
  request: Record<string, any>,
  environment: Record<string, string>,
): Promise<any> {
  if (IS_TAURI) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<any>("send_request", { request, environment });
  }

  // Browser: delegate to WASM engine
  await ensureWasm();
  const jsonStr: string = await (window as any).parallaxSendRequest(
    JSON.stringify(request),
    JSON.stringify(environment),
  );
  return JSON.parse(jsonStr);
}

// ── cancelRequest ─────────────────────────────────────────────────────────────

export async function cancelRequest(requestId: string): Promise<void> {
  if (!IS_TAURI) return; // WASM requests can't be cancelled mid-flight yet
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("cancel_request", { requestId });
}

// ── evalShellTemplate ─────────────────────────────────────────────────────────

export async function evalShellTemplate(cmd: string): Promise<string> {
  if (!IS_TAURI) {
    // Shell execution is not available in the browser; return empty string so
    // the {% shell %} tag resolves to "" rather than throwing.
    console.warn("parallax: {% shell %} tags are not supported in the browser");
    return "";
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<string>("eval_shell_template", { cmd });
}

// ── readFileForTemplate ───────────────────────────────────────────────────────

export async function readFileForTemplate(path: string): Promise<string> {
  if (!IS_TAURI) {
    console.warn("parallax: {% file %} tags are not supported in the browser");
    return "";
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<string>("read_file_for_template", { path });
}

// ── convertToCode (WASM only extra; Tauri has a Rust impl) ───────────────────

export async function convertToCode(
  request: Record<string, any>,
  lang: string,
): Promise<string> {
  if (IS_TAURI) {
    // Tauri path: Rust code-gen command (fallback to WASM if not available)
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke<string>("convert_to_code", { request, lang });
    } catch {
      // fall through to WASM
    }
  }
  await ensureWasm();
  return (window as any).parallaxConvertToCode(JSON.stringify(request), lang) as Promise<string>;
}

export { IS_TAURI };
