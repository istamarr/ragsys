#!/bin/bash

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found, installing..."
    cargo install wasm-pack
fi

# Build the project targeting web
echo "Building project with wasm-pack..."
wasm-pack build --target web

# Build the project as C and RUST
echo "Building project with Lvl wasm C and Rust..."
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir ./out target/wasm32-unknown-unknown/release/lib_presets.wasm

# (Optional) Optimize with wasm-opt
# wasm-opt -O3 -o ./out/lib_presets_bg.wasm ./out/lib_presets_bg.wasm