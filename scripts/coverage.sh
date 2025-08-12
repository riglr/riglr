#!/bin/bash
# Script to run code coverage with tarpaulin

echo "Running code coverage with tarpaulin..."
echo "Note: Skipping slow tests marked with #[ignore]"

# Run tarpaulin with optimized settings
cargo tarpaulin --config .tarpaulin.toml

echo "Coverage report generated in ./coverage/"