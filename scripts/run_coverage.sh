#!/bin/bash
# Script to run comprehensive test coverage for riglr-core

set -e

echo "Installing cargo-tarpaulin for coverage analysis..."
cargo install cargo-tarpaulin --locked

echo "Running comprehensive unit tests with coverage for riglr-core..."

# Run tests with coverage using tarpaulin
# --workspace: Run for all workspace members
# --exclude-files: Exclude test files and generated code
# --out: Output format (Html for detailed report)
# --timeout: Timeout per test in seconds
# --skip-clean: Don't clean build artifacts before running
cargo tarpaulin \
    --packages riglr-core \
    --exclude-files "*/tests/*" \
    --exclude-files "*/target/*" \
    --exclude-files "*/macros/*" \
    --out Html \
    --out Stdout \
    --timeout 120 \
    --skip-clean \
    --all-features \
    --verbose

echo "Coverage report generated in tarpaulin-report.html"

# Also run with llvm-cov for additional insights
echo "Installing cargo-llvm-cov..."
cargo install cargo-llvm-cov --locked

echo "Running with llvm-cov for line-by-line coverage..."
cargo llvm-cov \
    --package riglr-core \
    --all-features \
    --html \
    --open

echo "LLVM coverage report generated and opened in browser"

# Summary of coverage
echo "=== Coverage Summary ==="
cargo llvm-cov \
    --package riglr-core \
    --all-features \
    --summary-only

echo "Coverage analysis complete!"