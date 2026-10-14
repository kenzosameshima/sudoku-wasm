import { defineConfig } from "vite";

// base "./" so the static build works under any path (e.g. GitHub Pages project sites).
export default defineConfig({ base: "./", build: { target: "es2022" }, server: { fs: { allow: [".."] } } });
