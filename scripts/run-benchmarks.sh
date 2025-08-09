#!/bin/bash

# Run benchmarks for all riglr crates
# Usage: ./scripts/run-benchmarks.sh [crate] [--save] [--compare base_file]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BENCHMARK_DIR="benchmark-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Parse arguments
CRATE=""
SAVE_RESULTS=false
COMPARE_WITH=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --save)
            SAVE_RESULTS=true
            shift
            ;;
        --compare)
            COMPARE_WITH="$2"
            shift 2
            ;;
        *)
            if [ -z "$CRATE" ]; then
                CRATE="$1"
            fi
            shift
            ;;
    esac
done

# Function to run benchmarks for a specific crate
run_crate_benchmarks() {
    local crate_name=$1
    echo -e "${BLUE}Running benchmarks for $crate_name...${NC}"
    
    cd "$crate_name"
    
    # Run benchmarks and capture output
    if cargo bench --no-default-features 2>&1 | tee "../${BENCHMARK_DIR}/${crate_name}_${TIMESTAMP}.txt"; then
        echo -e "${GREEN}✓ $crate_name benchmarks completed${NC}"
    else
        echo -e "${RED}✗ $crate_name benchmarks failed${NC}"
    fi
    
    cd ..
}

# Function to run all benchmarks
run_all_benchmarks() {
    local crates=(
        "riglr-core"
        "riglr-evm-tools"
        "riglr-solana-tools"
        "riglr-web-tools"
        "riglr-graph-memory"
    )
    
    for crate in "${crates[@]}"; do
        if [ -d "$crate" ]; then
            run_crate_benchmarks "$crate"
        else
            echo -e "${YELLOW}Warning: $crate directory not found, skipping...${NC}"
        fi
    done
}

# Function to compare benchmark results
compare_benchmarks() {
    local base_file=$1
    local new_file=$2
    
    echo -e "${BLUE}Comparing benchmark results...${NC}"
    echo "Base: $base_file"
    echo "New: $new_file"
    echo ""
    
    # Simple comparison - in production you'd want a more sophisticated tool
    diff -u "$base_file" "$new_file" || true
}

# Main execution
main() {
    echo -e "${GREEN}=== Riglr Benchmark Suite ===${NC}"
    echo ""
    
    # Create benchmark directory if it doesn't exist
    mkdir -p "$BENCHMARK_DIR"
    
    # Run benchmarks
    if [ -z "$CRATE" ] || [ "$CRATE" == "all" ]; then
        echo "Running benchmarks for all crates..."
        run_all_benchmarks
    else
        if [ -d "$CRATE" ]; then
            run_crate_benchmarks "$CRATE"
        else
            echo -e "${RED}Error: Crate '$CRATE' not found${NC}"
            exit 1
        fi
    fi
    
    # Save results if requested
    if [ "$SAVE_RESULTS" = true ]; then
        echo -e "${BLUE}Benchmark results saved to ${BENCHMARK_DIR}/${NC}"
    fi
    
    # Compare with baseline if provided
    if [ -n "$COMPARE_WITH" ]; then
        if [ -f "$COMPARE_WITH" ]; then
            latest_results=$(ls -t ${BENCHMARK_DIR}/*.txt | head -1)
            compare_benchmarks "$COMPARE_WITH" "$latest_results"
        else
            echo -e "${RED}Error: Comparison file '$COMPARE_WITH' not found${NC}"
        fi
    fi
    
    echo ""
    echo -e "${GREEN}Benchmark suite completed!${NC}"
    
    # Generate summary
    echo ""
    echo "Summary:"
    echo "--------"
    for result_file in ${BENCHMARK_DIR}/*_${TIMESTAMP}.txt; do
        if [ -f "$result_file" ]; then
            crate_name=$(basename "$result_file" | cut -d'_' -f1)
            bench_count=$(grep -c "bench:" "$result_file" || echo "0")
            echo "  • $crate_name: $bench_count benchmarks"
        fi
    done
}

# Run main function
main