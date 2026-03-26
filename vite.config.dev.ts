import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import wasmPackWatchPlugin from "vite-plugin-wasm-pack-watcher";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
    build: {
        watch: {
            include: ["src/**/*.ts", "src/**/*.rs"],
        },
    },
    plugins: [react(), wasm(), topLevelAwait(), wasmPackWatchPlugin({})],
});
