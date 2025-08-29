#!/bin/bash
echo "🦀 Testing WebAssembly package..."
wasm-pack test --headless --firefox

echo "🦀 Building WebAssembly package..."
cargo run --bin bundle_specs_npm

echo "📋 Copying package.json and Readme files to /dist..."
cp bindings/js/* dist/

echo "✨ Building and testing completed!"