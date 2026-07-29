/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "node:path";

// 모바일 dev 는 기기/에뮬레이터가 호스트 PC 를 바라봐야 하므로 0.0.0.0 바인딩.
// TAURI_DEV_HOST 는 `tauri android dev` / `tauri ios dev` 가 주입.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: { "@": path.resolve(__dirname, "src") },
  },
  clearScreen: false,
  server: {
    host: host || "0.0.0.0",
    port: 1420,
    strictPort: true,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  test: {
    environment: "jsdom",
    globals: true,
  },
});
