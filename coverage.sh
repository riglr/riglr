#!/bin/bash
# Script to run code coverage with tarpaulin, avoiding timeout issues

echo "Running code coverage with tarpaulin..."
echo "Note: Skipping slow tests (timeout tests, stress tests, Redis tests)"

# Set environment variable to skip slow tests if your tests check for it
export SKIP_SLOW_TESTS=1
export TARPAULIN_RUN=1

# Run tarpaulin with optimized settings
cargo tarpaulin \
  --config .tarpaulin.toml \
  --timeout 600 \
  --skip-clean \
  --implicit-test-threads 1 \
  --exclude-files "riglr-showcase/*" \
  --exclude-files "*/tests/*" \
  --ignore-panics \
  --verbose \
  -- --skip test_tool_worker_process_job_timeout \
     --skip test_queue_stress_test \
     --skip test_concurrent

echo "Coverage report generated in ./coverage/"