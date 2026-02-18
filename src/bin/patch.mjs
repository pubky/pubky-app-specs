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
const wasmBase64 = await readFile(path.join(__dirname, `../../pkg/nodejs/${name}_bg.wasm`), "base64");

const patched = content
  // use global TextDecoder TextEncoder
  .replace("require(`util`)", "globalThis")
  // attach to `imports` instead of module.exports
  .replace("= module.exports", "= imports")
  // Export classes
  .replace(/\nclass (.*?) \{/g, "\n export class $1 {")
  // Export enums (Object.freeze declarations)
  .replace(/\nconst (\w+) = Object\.freeze/g, "\nexport const $1 = Object.freeze")
  // Export standalone functions that are assigned to exports
  .replace(/\nfunction (\w+)\(/g, (match, name, offset, str) => {
    // Check if there's a corresponding exports.name = name; line
    if (str.includes(`exports.${name} = ${name};`)) {
      return `\nexport function ${name}(`;
    }
    return match;
  })
  // Export functions defined inline on module.exports (for older wasm-bindgen)
  .replace(/\nmodule.exports.(.*?) = function/g, "\nimports.$1 = $1;\nexport function $1")
  // Replace module.exports.X with imports.X (for older wasm-bindgen)
  .replace(/\nmodule\.exports\.(.*?)\s+/g, "\nimports.$1")
  // Replace exports.X = X; with imports.X = X; (for newer wasm-bindgen)
  .replace(/\nexports\.(\w+) = (\w+);/g, "\nimports.$1 = $2;")
  // Remove default export of imports
  .replace(/export default imports$/, '')
  // Replace inline wasm bytes with __toBinary function and embedded base64 bytes
  .replace(
    /\nconst (?:wasm)?[Pp]ath.*\nconst ((?:wasm)?[Bb]ytes).*\n/,
    (match, bytesVar) => `
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

const ${bytesVar} = __toBinary(${JSON.stringify(wasmBase64)});
`
  );

// Write the patched JavaScript file with additional exports
// This creates the final index.js that will be used by Node.js/browser consumers
await writeFile(path.join(__dirname, `../../pkg/index.js`), "const imports = {};\n" + patched 
  + "\nglobalThis['pubky'] = imports\n");  // Make imports available globally as 'pubky'

// Move outside of nodejs
await Promise.all([".js", ".d.ts", "_bg.wasm"].map(suffix =>
  rename(
    path.join(__dirname, `../../pkg/nodejs/${name}${suffix}`),
    path.join(__dirname, `../../pkg/${suffix === '.js' ? "index.cjs" : (name + suffix)}`),
  ))
)

