#!/bin/bash
# NEXARA Test Script
set -euo pipefail

echo "=============================="
echo "  NEXARA Test Suite"
echo "=============================="

echo ""
echo "[1/4] Running unit tests..."
cargo test --workspace --lib 2>&1

echo ""
echo "[2/4] Running doc tests..."
cargo test --workspace --doc 2>&1

echo ""
echo "[3/4] Running integration tests..."
cargo test --workspace --test '*' 2>&1

echo ""
echo "[4/4] Running clippy..."
cargo clippy --workspace --all-targets -- -D warnings 2>&1

echo ""
echo "=============================="
echo "  All checks passed!"
echo "=============================="
