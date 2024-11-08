import * as esbuild from "npm:esbuild";
import { denoPlugins } from "jsr:@luca/esbuild-deno-loader";

const result = await esbuild.build({
  plugins: [...denoPlugins()],
  entryPoints: ["./src/lib.ts"],
  outfile: "./dist/deno_bindgen_utils.esm.js",
  bundle: true,
  format: "esm",
});

esbuild.stop();
