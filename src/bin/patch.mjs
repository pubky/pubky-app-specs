// Turns the wasm-bindgen nodejs output into the package's two private glue modules and
// generates the CommonJS twin of the hand-written entry.
//
// The glue instantiates the wasm at module load; both copies get a `__wbg_init()` instead, so
// nothing runs until the entry's `init()` asks. The ESM copy carries the wasm inline as base64
// (browsers and bundlers load one file), the CJS copy reads the `.wasm` next to it.

import { readFile, writeFile, rename, unlink } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path, { dirname } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const pkg = path.join(__dirname, "../../pkg");
const cargoToml = await readFile(path.join(__dirname, "../../Cargo.toml"), "utf8");
const name = /\[package\]\nname = "(.*?)"/.exec(cargoToml)[1].replace(/-/g, "_");

const glue = await readFile(path.join(pkg, `nodejs/${name}.js`), "utf8");

// The generated tail: read the bytes, compile, instantiate, start. Fail loudly if a
// wasm-bindgen upgrade changes it, rather than ship a glue that loads at import time.
const tail =
  /\nconst wasmPath = `\$\{__dirname\}\/\w+_bg\.wasm`;\nconst wasmBytes = require\('fs'\)\.readFileSync\(wasmPath\);\nconst wasmModule = new WebAssembly\.Module\(wasmBytes\);\nlet wasm = new WebAssembly\.Instance\(wasmModule, __wbg_get_imports\(\)\)\.exports;\nwasm\.__wbindgen_start\(\);\n?$/;
if (!tail.test(glue)) {
  throw new Error("patch.mjs: the wasm-bindgen glue tail changed; update the loader patch");
}

const loader = (bytes) => `
let wasm;
async function __wbg_init() {
  const { instance } = await WebAssembly.instantiate(${bytes}, __wbg_get_imports());
  wasm = instance.exports;
  wasm.__wbindgen_start();
}
`;

const cjs =
  glue.replace(tail, () => loader(`require("fs").readFileSync(\`\${__dirname}/${name}_bg.wasm\`)`)) +
  "exports.__wbg_init = __wbg_init;\n";

const base64 = await readFile(path.join(pkg, `nodejs/${name}_bg.wasm`), "base64");
let esm = glue
  .replace(/^exports\.(\w+) = (\w+);$/gm, "export { $2 as $1 };")
  .replace(tail, () => loader(`__toBinary(${JSON.stringify(base64)})`));
esm += `
function __toBinary(base64) {
  const table = new Uint8Array(128);
  for (let i = 0; i < 64; i++) table[i < 26 ? i + 65 : i < 52 ? i + 71 : i < 62 ? i - 4 : i * 4 - 205] = i;
  const n = base64.length;
  const bytes = new Uint8Array(((n - (base64[n - 1] == "=") - (base64[n - 2] == "=")) * 3) / 4 | 0);
  for (let i = 0, j = 0; i < n; ) {
    const c0 = table[base64.charCodeAt(i++)], c1 = table[base64.charCodeAt(i++)];
    const c2 = table[base64.charCodeAt(i++)], c3 = table[base64.charCodeAt(i++)];
    bytes[j++] = (c0 << 2) | (c1 >> 4);
    bytes[j++] = (c1 << 4) | (c2 >> 2);
    bytes[j++] = (c2 << 6) | c3;
  }
  return bytes;
}
export { __wbg_init };
`;
if (/\bexports\.|\brequire\(|\bmodule\.exports\b/.test(esm)) {
  throw new Error("patch.mjs: CommonJS left in the ESM glue");
}

await writeFile(path.join(pkg, `${name}.cjs`), cjs);
await writeFile(path.join(pkg, `${name}.js`), esm);
await rename(path.join(pkg, `nodejs/${name}_bg.wasm`), path.join(pkg, `${name}_bg.wasm`));
// The exported class declares [Symbol.dispose], which the default lib of a consumer lacks
const dts = await readFile(path.join(pkg, `nodejs/${name}.d.ts`), "utf8");
await writeFile(path.join(pkg, `${name}.d.ts`), `/// <reference lib="esnext.disposable" />\n${dts}`);
await unlink(path.join(pkg, `nodejs/${name}.d.ts`));

// index.cjs from index.js: relative ESM imports become requires of the .cjs twin, and the one
// closing `export { ... };` becomes module.exports.
const entry = await readFile(path.join(pkg, "index.js"), "utf8");
const cjsEntry = entry
  .replace(
    /^import \* as (\w+) from "(\.\/[\w.]+)\.js";$/gm,
    (_m, local, file) => `const ${local} = require("${file}.cjs");`,
  )
  .replace(
    /^import \{([^}]*)\} from "(\.\/[\w.]+)\.js";$/gm,
    (_m, names, file) => `const {${names}} = require("${file}.cjs");`,
  )
  .replace(/^export \{([^}]*)\};$/m, (_m, names) => `module.exports = {${names}};`);
if (/^\s*(import|export)\b/m.test(cjsEntry)) {
  throw new Error("patch.mjs: index.js has an import or export index.cjs cannot carry");
}
await writeFile(path.join(pkg, "index.cjs"), `"use strict";\n${cjsEntry}`);
