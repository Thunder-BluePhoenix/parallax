// Browser shim: @tauri-apps/api/core is never called at runtime when IS_TAURI=false,
// but some bundlers resolve imports statically. This stub prevents build errors.
export async function invoke<T = unknown>(_cmd: string, _args?: Record<string, unknown>): Promise<T> {
  throw new Error("Tauri invoke is not available in the browser");
}
