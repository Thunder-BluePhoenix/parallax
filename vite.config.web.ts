// Vite config for the browser / WASM build target.
// Usage:
//   npm run build:web          →  builds to dist-web/
//   npm run dev:web            →  dev server on :1421 (no Tauri)
//
// Key differences from the Tauri build (vite.config.ts):
//   - No TAURI_ENV_* env vars; targets modern evergreen browsers
//   - @tauri-apps/* packages stubbed out (never imported at runtime because
//     IS_TAURI = false in platform.ts)
//   - Copies parallax.wasm + wasm_exec.js into public/ so they are served
//   - Strips the Tauri optimizeDeps hints

import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "path";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,

  define: {
    // Prevent Tauri runtime checks from failing at build time
    "window.__TAURI__": "undefined",
  },

  server: {
    port: 1421,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**", "**/src-go/**"],
    },
  },

  build: {
    outDir: "dist-web",
    target: "es2022",
    minify: "esbuild",
    sourcemap: false,
    rollupOptions: {
      // Tauri packages should never end up in the browser bundle.
      // If they are somehow imported at build-time, replace with empty stubs.
      external: [
        "@tauri-apps/api",
        "@tauri-apps/api/core",
        "@tauri-apps/plugin-dialog",
        "@tauri-apps/plugin-fs",
      ],
      output: {
        globals: {
          "@tauri-apps/api/core": "{}",
          "@tauri-apps/plugin-dialog": "{}",
          "@tauri-apps/plugin-fs": "{}",
        },
      },
    },
  },

  resolve: {
    alias: {
      // Point any accidental direct Tauri imports to a no-op shim
      "@tauri-apps/api/core": path.resolve(__dirname, "src/lib/shims/tauri-core.ts"),
      "@tauri-apps/plugin-dialog": path.resolve(__dirname, "src/lib/shims/tauri-dialog.ts"),
      "@tauri-apps/plugin-fs": path.resolve(__dirname, "src/lib/shims/tauri-fs.ts"),
    },
  },

  envPrefix: ["VITE_"],
});
