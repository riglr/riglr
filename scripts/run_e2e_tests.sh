#!/bin/bash
set -e

# Script to run end-to-end integration tests for riglr
# This script sets up the test environment, runs tests, and cleans up

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Function to cleanup resources
cleanup() {
    print_info "Cleaning up test environment..."
    cd "$PROJECT_ROOT"
    docker-compose -f docker-compose.test.yml down -v
    rm -f .env.test.local
    print_info "Cleanup complete"
}

# Trap to ensure cleanup on exit
trap cleanup EXIT INT TERM

# Parse command line arguments
SUITE=""
VERBOSE=false
KEEP_RUNNING=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --suite)
            SUITE="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --keep-running)
            KEEP_RUNNING=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --suite SUITE      Run only a specific test suite (1-6)"
            echo "  --verbose          Enable verbose output"
            echo "  --keep-running     Keep services running after tests"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check prerequisites
print_info "Checking prerequisites..."

if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker first."
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    print_error "Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo is not installed. Please install Rust first."
    exit 1
fi

# Setup test environment
print_info "Setting up test environment..."

cd "$PROJECT_ROOT"

# Copy .env.test to .env.test.local for local modifications
if [ ! -f .env.test.local ]; then
    cp .env.test .env.test.local
    print_info "Created .env.test.local from .env.test"
fi

# Add PostgreSQL configuration to .env.test.local
cat >> .env.test.local << EOF

# PostgreSQL configuration for indexer tests
DATABASE_URL=postgresql://riglr_test:test_password@localhost:5432/riglr_indexer_test
EOF

# Start Docker services
print_info "Starting Docker services..."
docker-compose -f docker-compose.test.yml up -d

# Wait for services to be healthy
print_info "Waiting for services to be healthy..."

# Wait for Redis
until docker-compose -f docker-compose.test.yml exec -T redis redis-cli ping &> /dev/null; do
    print_warning "Waiting for Redis..."
    sleep 2
done
print_info "Redis is ready"

# Wait for PostgreSQL
until docker-compose -f docker-compose.test.yml exec -T postgres pg_isready -U riglr_test &> /dev/null; do
    print_warning "Waiting for PostgreSQL..."
    sleep 2
done
print_info "PostgreSQL is ready"

# Wait for Neo4j
max_attempts=30
attempt=0
until curl -s http://localhost:7474 &> /dev/null; do
    if [ $attempt -ge $max_attempts ]; then
        print_error "Neo4j failed to start after $max_attempts attempts"
        exit 1
    fi
    print_warning "Waiting for Neo4j... (attempt $((attempt+1))/$max_attempts)"
    sleep 5
    attempt=$((attempt+1))
done
print_info "Neo4j is ready"

# Wait for Wiremock
until curl -s http://localhost:8080/__admin/health &> /dev/null; do
    print_warning "Waiting for Wiremock..."
    sleep 2
done
print_info "Wiremock is ready"

# Create Wiremock mappings directory if it doesn't exist
mkdir -p "$PROJECT_ROOT/test/wiremock/mappings"
mkdir -p "$PROJECT_ROOT/test/wiremock/__files"

# Build the project
print_info "Building the project..."
cargo build --workspace --all-features

# Run database migrations for indexer
print_info "Running database migrations..."
if [ -d "$PROJECT_ROOT/riglr-indexer/migrations" ]; then
    cd "$PROJECT_ROOT/riglr-indexer"
    # Run migrations if sqlx is available
    if command -v sqlx &> /dev/null; then
        DATABASE_URL=postgresql://riglr_test:test_password@localhost:5432/riglr_indexer_test sqlx migrate run
    else
        print_warning "sqlx-cli not installed, skipping migrations"
    fi
    cd "$PROJECT_ROOT"
fi

# Function to run a specific test suite
run_suite() {
    local suite_num=$1
    local suite_name=$2
    
    print_info "Running Suite $suite_num: $suite_name"
    
    # Set environment variables for the test
    export RUST_BACKTRACE=1
    export RUST_LOG=debug
    
    # Source the test environment
    set -a
    source .env.test.local
    set +a
    
    # Run the specific test suite
    case $suite_num in
        1)
            cargo test --package riglr-agents --test e2e_core_agent_workflow -- --nocapture
            ;;
        2)
            cargo test --package riglr-agents --test e2e_multi_agent_coordination -- --nocapture
            ;;
        3)
            cargo test --package riglr-streams --test e2e_data_pipeline -- --nocapture
            ;;
        4)
            cargo test --package riglr-server --test e2e_web_service -- --nocapture
            ;;
        5)
            cargo test --package riglr-graph-memory --test e2e_agent_memory -- --nocapture
            ;;
        6)
            cargo test --package create-riglr-app --test e2e_scaffolding -- --nocapture
            ;;
        *)
            print_error "Unknown suite: $suite_num"
            return 1
            ;;
    esac
    
    if [ $? -eq 0 ]; then
        print_info "Suite $suite_num passed ✓"
    else
        print_error "Suite $suite_num failed ✗"
        return 1
    fi
}

# Run tests
print_info "Running E2E integration tests..."

if [ -n "$SUITE" ]; then
    # Run specific suite
    case $SUITE in
        1) run_suite 1 "Core Agent Workflow" ;;
        2) run_suite 2 "Multi-Agent Coordination" ;;
        3) run_suite 3 "Real-time Data Processing Pipeline" ;;
        4) run_suite 4 "Web Service & Authentication" ;;
        5) run_suite 5 "Agent Memory & Knowledge Graph" ;;
        6) run_suite 6 "Application Scaffolding" ;;
        *) 
            print_error "Invalid suite number: $SUITE. Must be 1-6."
            exit 1
            ;;
    esac
else
    # Run all suites
    failed_suites=()
    
    for i in {1..6}; do
        case $i in
            1) suite_name="Core Agent Workflow" ;;
            2) suite_name="Multi-Agent Coordination" ;;
            3) suite_name="Real-time Data Processing Pipeline" ;;
            4) suite_name="Web Service & Authentication" ;;
            5) suite_name="Agent Memory & Knowledge Graph" ;;
            6) suite_name="Application Scaffolding" ;;
        esac
        
        if ! run_suite $i "$suite_name"; then
            failed_suites+=($i)
        fi
    done
    
    # Report results
    echo ""
    print_info "========================================="
    print_info "E2E Integration Test Results"
    print_info "========================================="
    
    if [ ${#failed_suites[@]} -eq 0 ]; then
        print_info "All test suites passed! ✓"
        exit_code=0
    else
        print_error "Failed suites: ${failed_suites[*]}"
        exit_code=1
    fi
fi

# Keep services running if requested
if [ "$KEEP_RUNNING" = true ]; then
    print_info "Keeping services running. Use 'docker-compose -f docker-compose.test.yml down' to stop them."
    trap - EXIT
else
    cleanup
fi

exit ${exit_code:-0}