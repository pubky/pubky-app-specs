// Convert wasm-bindgen CJS output into a self-contained ESM bundle with inline WASM.
// Based on hacks from https://github.com/rustwasm/wasm-pack/issues/1334
// Aligned with https://github.com/pubky/pubky-core/blob/main/pubky-sdk/bindings/js/scripts/patch.mjs

import { readFile, writeFile, rename } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path, { dirname } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const cargoTomlContent = await readFile(path.join(__dirname, "../../Cargo.toml"), "utf8");
const name = /\[package\]\nname = "(.*?)"/.exec(cargoTomlContent)[1].replace(/-/g, "_");

const content = await readFile(path.join(__dirname, `../../pkg/nodejs/${name}.js`), "utf8");

const needsNamedExport = new Set();
const hasModuleExports = content.includes("= module.exports");

let patched = content
  .replace("require(`util`)", "globalThis")
  .replace("= module.exports", "= imports")
  .replace(/\nclass (.*?) \{/g, "\n export class $1 {")
  .replace(
    /\n(?:module\.exports|exports)\.(\w+)\s*=\s*function/g,
    (_match, fn) => {
      needsNamedExport.delete(fn);
      return `\nimports.${fn} = ${fn};\nexport function ${fn}`;
    },
  )
  .replace(
    /\n(?:module\.exports|exports)\.(\w+)\s*=\s*([^;\n]+)(;?)/g,
    (_match, name, value, suffix) => {
      const trimmed = value.trim();
      if (trimmed === name) {
        needsNamedExport.add(name);
      }
      return `\nimports.${name} = ${trimmed}${suffix}`;
    },
  )
  .replace(/= exports\./g, "= imports.");

if (!hasModuleExports) {
  patched = "const imports = {};\n" + patched;
}

for (const name of needsNamedExport) {
  if (
    name !== "default" &&
    !new RegExp(`export (?:class|function|const|let|var) ${name}\\b`).test(patched)
  ) {
    patched += `\nexport { ${name} };`;
  }
}

patched += "\nexport default imports";
patched = patched
  .replace(
    /\nconst (?:path.*\nconst bytes.*|wasmPath.*\nconst wasmBytes.*)\nconst wasmModule.*\n/,
    `
var __toBinary = /* @__PURE__ */ (() => {
  var table = new Uint8Array(128);
  for (var i = 0; i < 64; i++)
    table[i < 26 ? i + 65 : i < 52 ? i + 71 : i < 62 ? i - 4 : i * 4 - 205] = i;
  return (base64) => {
    var n = base64.length, bytes = new Uint8Array((n - (base64[n - 1] == "=") - (base64[n - 2] == "=")) * 3 / 4 | 0);
    for (var i2 = 0, j = 0; i2 < n; ) {
      var c0 = table[base64.charCodeAt(i2++)], c1 = table[base64.charCodeAt(i2++)];
      var c2 = table[base64.charCodeAt(i2++)], c3 = table[base64.charCodeAt(i2++)];
      bytes[j++] = c0 << 2 | c1 >> 4;
      bytes[j++] = c1 << 4 | c2 >> 2;
      bytes[j++] = c2 << 6 | c3;
    }
    return bytes;
  };
})();

const bytes = __toBinary(${JSON.stringify(
      await readFile(path.join(__dirname, `../../pkg/nodejs/${name}_bg.wasm`), "base64"),
    )});
const wasmModule = new WebAssembly.Module(bytes);
`,
  );

await writeFile(
  path.join(__dirname, `../../pkg/index.js`),
  patched + "\nglobalThis['pubky'] = imports\n",
);

// Move CJS output outside of nodejs/
await Promise.all(
  [".js", ".d.ts", "_bg.wasm"].map((suffix) =>
    rename(
      path.join(__dirname, `../../pkg/nodejs/${name}${suffix}`),
      path.join(__dirname, `../../pkg/${suffix === ".js" ? "index.cjs" : name + suffix}`),
    ),
  ),
);
