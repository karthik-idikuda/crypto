#!/bin/bash
# NEXARA Build Script
set -euo pipefail

echo "=============================="
echo "  NEXARA Build Script"
echo "=============================="

# Check Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Install from https://rustup.rs"
    exit 1
fi

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo ""

# Build in release mode
echo "Building all crates..."
cargo build --release 2>&1

echo ""
echo "Build complete!"
echo "Binary: target/release/nexara"
echo ""

# Run tests
echo "Running tests..."
cargo test --workspace 2>&1

echo ""
echo "All tests passed!"
