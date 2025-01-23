#!/bin/bash


echo "🦀 Testing WebAssembly package..."
wasm-pack test --headless --firefox

echo "🦀 Building WebAssembly package..."
wasm-pack build --target web --out-dir dist

echo "📋 Copying package.json and Readme files to /dist..."
cp pkg/* dist/

# echo "📦 Installing npm dependencies..."
# cd dist && npm install

# echo "🧪 Running JavaScript tests..."
# npm test

echo "✨ Building and testing completed!"