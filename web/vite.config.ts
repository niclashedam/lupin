import { readFileSync } from "node:fs";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

const version = readFileSync(new URL("../Cargo.toml", import.meta.url), "utf8").match(/^version = "(.+)"/m)![1];

export default defineConfig({
  plugins: [tailwindcss()],
  define: { __LUPIN_VERSION__: JSON.stringify(version) },
  // The sample files live in ../examples
  server: { fs: { allow: [".."] } },
});
