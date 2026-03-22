#!/bin/bash
# NEXARA Benchmark Script
set -euo pipefail

echo "=============================="
echo "  NEXARA Benchmarks"
echo "=============================="

echo ""
echo "Running crypto benchmarks..."
cargo bench --bench bench_crypto 2>&1

echo ""
echo "Running VM benchmarks..."
cargo bench --bench bench_vm 2>&1

echo ""
echo "Benchmarks complete!"
