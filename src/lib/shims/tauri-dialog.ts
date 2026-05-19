// Browser shim for @tauri-apps/plugin-dialog
export async function open(_opts?: unknown): Promise<null> { return null; }
export async function save(_opts?: unknown): Promise<null> { return null; }
export async function message(_msg: string, _opts?: unknown): Promise<void> {}
export async function ask(_msg: string, _opts?: unknown): Promise<boolean> { return false; }
export async function confirm(_msg: string, _opts?: unknown): Promise<boolean> { return false; }
