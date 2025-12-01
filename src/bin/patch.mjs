// This script is used to generate isomorphic code for web and nodejs
//
// Based on hacks from [this issue](https://github.com/rustwasm/wasm-pack/issues/1334)

import { readFile, writeFile, rename } from "node:fs/promises";
import { fileURLToPath } from 'node:url';
import path, { dirname } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));

const cargoTomlContent = await readFile(path.join(__dirname, "../../Cargo.toml"), "utf8");
const cargoPackageName = /\[package\]\nname = "(.*?)"/.exec(cargoTomlContent)[1]
const name = cargoPackageName.replace(/-/g, '_')

const content = await readFile(path.join(__dirname, `../../pkg/nodejs/${name}.js`), "utf8");

// First, find all functions that are exported via exports.X = X; pattern
const exportedFunctions = new Set();
const exportPattern = /\nexports\.(\w+) = \1;/g;
let match;
while ((match = exportPattern.exec(content)) !== null) {
  exportedFunctions.add(match[1]);
}

let patched = content
  // use global TextDecoder TextEncoder
  .replace("require(`util`)", "globalThis")
  // attach to `imports` instead of module.exports
  .replace("= module.exports", "= imports")
  // Export classes
  .replace(/\nclass (.*?) \{/g, "\n export class $1 {")
  // Export enums (Object.freeze pattern used by wasm-bindgen)
  .replace(/\nconst (\w+) = Object\.freeze\(\{/g, "\nexport const $1 = Object.freeze({")
  // Export functions that have module.exports.X = function pattern
  .replace(/\nmodule.exports.(.*?) = function/g, "\nimports.$1 = $1;\nexport function $1")
  // Add exports to 'imports' (old module.exports pattern)
  .replace(/\nmodule\.exports\.(.*?)\s+/g, "\nimports.$1")
  // Transform exports.X = X; to imports.X = X; (new wasm-bindgen pattern)
  .replace(/\nexports\.(\w+) = (\w+);/g, "\nimports.$1 = $2;")
  // Transform exports.__wbg_* = ... to imports.__wbg_* = ... (wasm-bindgen internals)
  .replace(/\nexports\.(__\w+) = /g, "\nimports.$1 = ")
  // Remove default export of imports
  .replace(/export default imports$/, '')
  // Replace wasm loading code with base64 embedded version
  // Handle both old pattern (const path, const bytes) and new pattern (wasmPath, wasmBytes, etc.)
  .replace(
    /\nconst wasmPath[^\n]*\nconst wasmBytes[^\n]*\nconst wasmModule[^\n]*\nconst wasm = exports\.__wasm[^\n]*/,
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
const bytes = __toBinary(${JSON.stringify(await readFile(path.join(__dirname, `../../pkg/nodejs/${name}_bg.wasm`), "base64"))});
const wasmModule = new WebAssembly.Module(bytes);
const wasm = imports.__wasm = new WebAssembly.Instance(wasmModule, imports).exports`
  )
  // Also handle old pattern for backwards compatibility
  .replace(
    /\nconst path.*\nconst bytes.*\n/,
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

const bytes = __toBinary(${JSON.stringify(await readFile(path.join(__dirname, `../../pkg/nodejs/${name}_bg.wasm`), "base64"))});
`
  );

// Add 'export' to function definitions that are exported via exports.X = X;
for (const funcName of exportedFunctions) {
  // Match function definitions that are not already exported
  const funcPattern = new RegExp(`\\nfunction ${funcName}\\(`, 'g');
  patched = patched.replace(funcPattern, `\nexport function ${funcName}(`);
}

// Write the patched JavaScript file with additional exports
// This creates the final index.js that will be used by Node.js/browser consumers
await writeFile(path.join(__dirname, `../../pkg/index.js`), patched 
  + "\nglobalThis['pubky'] = imports\n");  // Make imports available globally as 'pubky'

// Move outside of nodejs
await Promise.all([".js", ".d.ts", "_bg.wasm"].map(suffix =>
  rename(
    path.join(__dirname, `../../pkg/nodejs/${name}${suffix}`),
    path.join(__dirname, `../../pkg/${suffix === '.js' ? "index.cjs" : (name + suffix)}`),
  ))
)

