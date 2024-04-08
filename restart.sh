#!/bin/bash

# Build Rust WebAssembly
echo "Building Rust WebAssembly..."
rm -r pkg/
wasm-pack build --target web

# Navigate to frontend directory
cd ../frontend-rimpinterpreter

# Install dependencies
echo "Installing frontend dependencies..."
npm install

# Start the frontend
echo "Starting the frontend..."
npm start

