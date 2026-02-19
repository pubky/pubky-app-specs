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

const patched = content
  // use global TextDecoder TextEncoder
  .replace("require(`util`)", "globalThis")
  // attach to `imports` instead of module.exports
  .replace("= module.exports", "= imports")
  // Export classes
  .replace(/\nclass (.*?) \{/g, "\nexport class $1 {")
  // Export functions (old wasm-bindgen format: module.exports.func = function)
  .replace(/\nmodule.exports.(.*?) = function/g, "\nimports.$1 = $1;\nexport function $1")
  // Add exports to 'imports' (old wasm-bindgen format)
  .replace(/\nmodule\.exports\.(.*?)\s+/g, "\nimports.$1")
  // Remove default export of imports
  .replace(/export default imports$/, '')
  // Handle new wasm-bindgen format: exports.X = X;
  // Uppercase names are classes (already exported via 'export class') or enums (handled below) - remove CJS line
  // Lowercase names are free functions - replace with ESM export
  .replace(/\nexports\.(\w+) = \1;/g, (_, n) => /^[A-Z]/.test(n) ? '' : `\nexport { ${n} };`)
  // Replace inline wasm bytes with __toBinary function and embedded base64 bytes
  // Handles both old wasm-bindgen format (const path/bytes) and new format (const wasmPath/wasmBytes)
  .replace(
    /\nconst (?:wasmPath|path).*\nconst (?:wasmBytes|bytes).*\n/,
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

const wasmBytes = __toBinary(${JSON.stringify(await readFile(path.join(__dirname, `../../pkg/nodejs/${name}_bg.wasm`), "base64"))});
`
  )
  // Ensure old-format reference 'bytes' is updated to 'wasmBytes' for new WebAssembly.Module instantiation
  .replace(/new WebAssembly\.Module\(bytes\)/, 'new WebAssembly.Module(wasmBytes)');

// Collect names already exported as classes to avoid duplicate declarations
const exportedClasses = new Set(
  [...patched.matchAll(/\nexport class (\w+)/g)].map(m => m[1])
);

// Collect names already declared as consts (wasm-bindgen may emit these for enums)
const declaredConsts = new Set(
  [...patched.matchAll(/\nconst (\w+)/g)].map(m => m[1])
);

// Re-export enums so Next.js can statically import them, but only if not already exported as a class
const enumReExports = [
  "PubkyAppPostKind",
  "PubkyAppFeedLayout",
  "PubkyAppFeedReach",
  "PubkyAppFeedSort",
]
  .filter(name => !exportedClasses.has(name))
  .map(name => declaredConsts.has(name)
    ? `export { ${name} };\n`
    : `export const ${name} = imports.${name};\n`)
  .join("");

// Write the patched JavaScript file with additional exports
// This creates the final index.js that will be used by Node.js/browser consumers
await writeFile(path.join(__dirname, `../../pkg/index.js`), patched 
  + "// Re-export enums so Next.js can statically import them\n"
  + enumReExports);

// Move outside of nodejs
await Promise.all([".js", ".d.ts", "_bg.wasm"].map(suffix =>
  rename(
    path.join(__dirname, `../../pkg/nodejs/${name}${suffix}`),
    path.join(__dirname, `../../pkg/${suffix === '.js' ? "index.cjs" : (name + suffix)}`),
  ))
)
