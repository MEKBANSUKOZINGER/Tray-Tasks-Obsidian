import { defineConfig } from "vite";
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**", "**/artifacts/**", "**/.smoke-*/**"],
    },
  },
  build: { target: "es2022" },
});
