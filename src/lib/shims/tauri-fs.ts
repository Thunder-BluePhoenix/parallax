// Browser shim for @tauri-apps/plugin-fs
export async function readTextFile(_path: string): Promise<string> { return ""; }
export async function writeTextFile(_path: string, _contents: string): Promise<void> {}
export async function exists(_path: string): Promise<boolean> { return false; }
export async function readDir(_path: string): Promise<unknown[]> { return []; }
export async function createDir(_path: string, _opts?: unknown): Promise<void> {}
export async function removeFile(_path: string): Promise<void> {}
