// This script is used to generate isomorphic code for web and nodejs
//
// Based on hacks from [this issue](https://github.com/rustwasm/wasm-pack/issues/1334)

import { readFile, writeFile, rename } from "node:fs/promises";
import { fileURLToPath } from 'node:url';
import path, { dirname } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const cargoTomlContent = await readFile(path.join(__dirname, "../../Cargo.toml"), "utf8");
const cargoPackageName = /\[package\]\nname = "(.*?)"/.exec(cargoTomlContent)[1]
const name = cargoPackageName.replace(/-/g, '_')

const content = await readFile(path.join(__dirname, `../../pkg/dist/nodejs/${name}.js`), "utf8");

const patched = content
  // use global TextDecoder TextEncoder
  .replace("require(`util`)", "globalThis")
  // attach to `imports` instead of module.exports
  .replace("= module.exports", "= imports")
  // Export classes
  .replace(/\nclass (.*?) \{/g, "\n export class $1 {")
  // Export functions
  .replace(/\nmodule.exports.(.*?) = function/g, "\nimports.$1 = $1;\nexport function $1")
  // Add exports to 'imports'
  .replace(/\nmodule\.exports\.(.*?)\s+/g, "\nimports.$1")
  // Remove default export of imports (will be replaced with init function)
  .replace(/export default imports$/, '')
  // Remove inline wasm bytes - will use separate file
  .replace(
    /\nconst path.*\nconst bytes.*\n/,
    '\n// WASM bytes removed - use separate .wasm file\n'
  );

// Create async init function that works in both Node.js and browser
const asyncInitFunction = `
let wasmInitialized = false;

export default async function init() {
  if (wasmInitialized) return wasm;
  
  let wasmBytes;
  
  // Detect environment and load WASM accordingly
  if (typeof window !== 'undefined') {
    // Browser environment - use fetch with relative path
    const response = await fetch('./pubky_app_specs_bg.wasm');
    if (!response.ok) {
      throw new Error(\`Failed to fetch WASM: \${response.status} \${response.statusText}\`);
    }
    wasmBytes = await response.arrayBuffer();
  } else {
    // Node.js environment - read file directly
    const fs = await import('fs');
    const path = await import('path');
    const url = await import('url');
    
    // Get current directory of this module
    const currentDir = path.dirname(url.fileURLToPath(import.meta.url));
    const wasmPath = path.join(currentDir, 'pubky_app_specs_bg.wasm');
    
    wasmBytes = fs.readFileSync(wasmPath);
  }
  
  const wasmModule = new WebAssembly.Module(wasmBytes);
  const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
  
  wasm = wasmInstance.exports;
  imports.__wasm = wasm;
  wasm.__wbindgen_start();
  wasmInitialized = true;
  
  return wasm;
}

export { init };
`;

// Replace synchronous initialization with async version
const patchedWithAsync = patched
  // Remove synchronous WASM instantiation
  .replace(/const wasmModule = new WebAssembly\.Module\(bytes\);\nconst wasmInstance = new WebAssembly\.Instance\(wasmModule, imports\);\nwasm = wasmInstance\.exports;\nimports\.__wasm= wasm;\n\nwasm\.__wbindgen_start\(\);/, '')
  // Add the async init function
  + asyncInitFunction;

// Write the patched JavaScript file with additional exports
// This creates the final index.js that will be used by Node.js/browser consumers
await writeFile(path.join(__dirname, `../../pkg/index.js`), patchedWithAsync 
  + "\nglobalThis['pubky'] = imports\n"  // Make imports available globally as 'pubky'
  + "// Re-export enums so Next.js can statically import them\n"
  + "export const PubkyAppPostKind  = imports.PubkyAppPostKind;\n"   // Export enum for named imports
  + "export const PubkyAppFeedLayout = imports.PubkyAppFeedLayout;\n" // Export enum for named imports  
  + "export const PubkyAppFeedReach  = imports.PubkyAppFeedReach;\n"  // Export enum for named imports
  + "export const PubkyAppFeedSort   = imports.PubkyAppFeedSort;\n"); // Export enum for named imports

// Move outside of nodejs
await Promise.all([".js", ".d.ts", "_bg.wasm"].map(suffix =>
  rename(
    path.join(__dirname, `../../pkg/dist/nodejs/${name}${suffix}`),
    path.join(__dirname, `../../pkg/${suffix === '.js' ? "index.cjs" : (name + suffix)}`),
  ))
)

// Add index.cjs headers

const indexcjsPath = path.join(__dirname, `../../pkg/index.cjs`);
const indexcjsContent = await readFile(indexcjsPath, 'utf8');

await writeFile(indexcjsPath, indexcjsContent, 'utf8')


