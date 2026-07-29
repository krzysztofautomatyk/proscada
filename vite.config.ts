import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "node:path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      $lib: path.resolve("./src/lib"),
    },
  },
  clearScreen: false,
  server: {
    port: 1450,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1451 }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari15",
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    // Third-party code changes far less often than the application, so it is
    // split out to keep the entry chunk small and cacheable. Application code
    // is intentionally left in one chunk: the Designer and the Toolbox
    // renderers reference each other, and splitting them produces circular
    // chunks rather than smaller downloads.
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (!id.includes("node_modules")) return undefined;
          if (id.includes("qrcode")) return "vendor-qrcode";
          if (id.includes("@tauri-apps")) return "vendor-tauri";
          return "vendor";
        },
      },
    },
  },
});
