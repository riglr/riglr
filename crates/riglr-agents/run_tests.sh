#!/bin/bash

# Riglr Agents Test Runner
# Comprehensive test execution script for the riglr-agents crate

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target}"
TEST_THREADS="${TEST_THREADS:-1}"
COVERAGE_OUTPUT="${COVERAGE_OUTPUT:-coverage.lcov}"
RUN_E2E="${RUN_E2E:-false}"
RUN_BENCHMARKS="${RUN_BENCHMARKS:-false}"
RUN_COVERAGE="${RUN_COVERAGE:-false}"
VERBOSE="${VERBOSE:-false}"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_usage() {
    cat << EOF
Riglr Agents Test Runner

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -h, --help         Show this help message
    -e, --e2e          Run E2E tests (requires blockchain connectivity)
    -b, --benchmarks   Run performance benchmarks
    -c, --coverage     Generate coverage report
    -v, --verbose      Enable verbose output
    -t, --threads N    Set number of test threads (default: 1)
    -a, --all          Run all tests including E2E and benchmarks

EXAMPLES:
    $0                 # Run unit and integration tests only
    $0 -e              # Include E2E tests
    $0 -c              # Generate coverage report
    $0 -a              # Run everything
    $0 -e -c -v        # E2E tests with coverage and verbose output

ENVIRONMENT VARIABLES:
    CARGO_TARGET_DIR   Target directory for builds (default: target)
    TEST_THREADS       Number of test threads (default: 1)
    COVERAGE_OUTPUT    Coverage report file (default: coverage.lcov)
    RUN_E2E            Run E2E tests (default: false)
    RUN_BENCHMARKS     Run benchmarks (default: false) 
    RUN_COVERAGE       Generate coverage (default: false)
    VERBOSE            Enable verbose output (default: false)
EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if we're in the right directory
    if [[ ! -f "Cargo.toml" ]] || [[ ! -d "src" ]]; then
        log_error "Not in riglr-agents directory. Please run from riglr-agents root."
        exit 1
    fi
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust and Cargo."
        exit 1
    fi
    
    # Check for coverage tool if needed
    if [[ "$RUN_COVERAGE" == "true" ]]; then
        if ! command -v cargo-llvm-cov &> /dev/null; then
            log_warning "cargo-llvm-cov not found. Installing..."
            cargo install --locked cargo-llvm-cov || {
                log_error "Failed to install cargo-llvm-cov"
                exit 1
            }
        fi
    fi
    
    log_success "Prerequisites check passed"
}

run_format_check() {
    log_info "Checking code formatting..."
    if cargo fmt --check; then
        log_success "Code formatting is correct"
    else
        log_error "Code formatting issues found. Run 'cargo fmt' to fix."
        exit 1
    fi
}

run_clippy() {
    log_info "Running Clippy lints..."
    if cargo clippy --all-features --all-targets -- -D warnings; then
        log_success "Clippy checks passed"
    else
        log_error "Clippy found issues"
        exit 1
    fi
}

run_unit_tests() {
    log_info "Running unit tests..."
    
    local verbose_flag=""
    if [[ "$VERBOSE" == "true" ]]; then
        verbose_flag="-- --nocapture"
    fi
    
    if cargo test --lib --all-features --test-threads="$TEST_THREADS" $verbose_flag; then
        log_success "Unit tests passed"
    else
        log_error "Unit tests failed"
        exit 1
    fi
}

run_integration_tests() {
    log_info "Running integration tests..."
    
    local verbose_flag=""
    if [[ "$VERBOSE" == "true" ]]; then
        verbose_flag="-- --nocapture"
    fi
    
    if cargo test --test '*' --features test-utils --test-threads="$TEST_THREADS" $verbose_flag; then
        log_success "Integration tests passed"
    else
        log_error "Integration tests failed"
        exit 1
    fi
}

run_e2e_tests() {
    if [[ "$RUN_E2E" != "true" ]]; then
        log_info "Skipping E2E tests (use -e to enable)"
        return 0
    fi
    
    log_info "Running E2E tests (this may take several minutes)..."
    
    # Check for required environment variables
    local missing_vars=()
    
    if [[ -z "$ETHEREUM_RPC_URL" ]]; then missing_vars+=("ETHEREUM_RPC_URL"); fi
    if [[ -z "$SOLANA_RPC_URL" ]]; then missing_vars+=("SOLANA_RPC_URL"); fi
    
    if [[ ${#missing_vars[@]} -gt 0 ]]; then
        log_warning "E2E tests require environment variables: ${missing_vars[*]}"
        log_warning "Setting default test values..."
        export ETHEREUM_RPC_URL="${ETHEREUM_RPC_URL:-https://eth.llamarpc.com}"
        export SOLANA_RPC_URL="${SOLANA_RPC_URL:-https://api.devnet.solana.com}"
        export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-test_key}"
        export REDIS_URL="${REDIS_URL:-redis://localhost:6379}"
    fi
    
    local verbose_flag=""
    if [[ "$VERBOSE" == "true" ]]; then
        verbose_flag="-- --nocapture --ignored"
    else
        verbose_flag="-- --ignored"
    fi
    
    # Set timeout for E2E tests
    if timeout 1800 cargo test --features blockchain-tests --test '*' --test-threads="$TEST_THREADS" $verbose_flag; then
        log_success "E2E tests passed"
    else
        log_error "E2E tests failed or timed out"
        exit 1
    fi
}

run_benchmarks() {
    if [[ "$RUN_BENCHMARKS" != "true" ]]; then
        log_info "Skipping benchmarks (use -b to enable)"
        return 0
    fi
    
    log_info "Running performance benchmarks..."
    
    if cargo bench --features performance-tests; then
        log_success "Benchmarks completed"
        log_info "Benchmark results saved to target/criterion/"
    else
        log_error "Benchmarks failed"
        exit 1
    fi
}

generate_coverage() {
    if [[ "$RUN_COVERAGE" != "true" ]]; then
        log_info "Skipping coverage report (use -c to enable)"
        return 0
    fi
    
    log_info "Generating coverage report..."
    
    if cargo llvm-cov --package riglr-agents \
        --features test-utils \
        --lcov --output-path "$COVERAGE_OUTPUT" \
        -- --test-threads="$TEST_THREADS"; then
        log_success "Coverage report generated: $COVERAGE_OUTPUT"
        
        # Extract coverage percentage if possible
        if command -v lcov &> /dev/null; then
            local coverage_percent=$(lcov --summary "$COVERAGE_OUTPUT" 2>/dev/null | grep "lines" | awk '{print $2}' | sed 's/%//' || echo "unknown")
            log_info "Line coverage: ${coverage_percent}%"
        fi
    else
        log_error "Coverage generation failed"
        exit 1
    fi
}

print_summary() {
    echo
    log_info "=== Test Summary ==="
    echo "✅ Code formatting check"
    echo "✅ Clippy lints"
    echo "✅ Unit tests"
    echo "✅ Integration tests"
    
    if [[ "$RUN_E2E" == "true" ]]; then
        echo "✅ E2E tests"
    else
        echo "⏭️  E2E tests (skipped)"
    fi
    
    if [[ "$RUN_BENCHMARKS" == "true" ]]; then
        echo "✅ Performance benchmarks"
    else
        echo "⏭️  Performance benchmarks (skipped)"
    fi
    
    if [[ "$RUN_COVERAGE" == "true" ]]; then
        echo "✅ Coverage report: $COVERAGE_OUTPUT"
    else
        echo "⏭️  Coverage report (skipped)"
    fi
    
    echo
    log_success "All enabled tests passed!"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            print_usage
            exit 0
            ;;
        -e|--e2e)
            RUN_E2E="true"
            shift
            ;;
        -b|--benchmarks)
            RUN_BENCHMARKS="true"
            shift
            ;;
        -c|--coverage)
            RUN_COVERAGE="true"
            shift
            ;;
        -v|--verbose)
            VERBOSE="true"
            shift
            ;;
        -t|--threads)
            TEST_THREADS="$2"
            shift 2
            ;;
        -a|--all)
            RUN_E2E="true"
            RUN_BENCHMARKS="true"
            RUN_COVERAGE="true"
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    log_info "Starting Riglr Agents test suite..."
    log_info "Configuration:"
    log_info "  E2E Tests: $RUN_E2E"
    log_info "  Benchmarks: $RUN_BENCHMARKS"
    log_info "  Coverage: $RUN_COVERAGE"
    log_info "  Test Threads: $TEST_THREADS"
    log_info "  Verbose: $VERBOSE"
    echo
    
    check_prerequisites
    run_format_check
    run_clippy
    run_unit_tests
    run_integration_tests
    run_e2e_tests
    run_benchmarks
    generate_coverage
    print_summary
}

# Trap to ensure cleanup on script exit
cleanup() {
    if [[ $? -ne 0 ]]; then
        log_error "Test suite failed. Check output above for details."
    fi
}
trap cleanup EXIT

# Run main function
main "$@"