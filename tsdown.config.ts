import { defineConfig } from "tsdown";

export default defineConfig([
  { entry: ["src/mod.ts"] },
  {
    entry: ["src/ts-plugin/index.ts"],
    format: "cjs",
    dts: false,
  },
]);
