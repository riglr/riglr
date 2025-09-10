#!/bin/bash

# Riglr Crates Publishing Script
# This script publishes all riglr crates to crates.io in the correct dependency order

set -e  # Exit on any error

# Color output for better readability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "riglr-core" ]; then
    print_error "This script must be run from the riglr workspace root directory"
    exit 1
fi

# Check for dry-run flag
DRY_RUN=false
SKIP_VERIFY=false
FORCE=false
DELAY=30  # Delay between publishes in seconds

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --dry-run) DRY_RUN=true ;;
        --skip-verify) SKIP_VERIFY=true ;;
        --force) FORCE=true ;;
        --delay) DELAY="$2"; shift ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dry-run       Run in dry-run mode (don't actually publish)"
            echo "  --skip-verify   Skip verification step"
            echo "  --force         Force publish even if crate already exists"
            echo "  --delay SECS    Delay between publishes (default: 30)"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *) print_error "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

# Define the publishing order based on dependencies
# Each tier can only depend on crates from previous tiers
declare -a TIER_1=(
    "riglr-config"
)

declare -a TIER_2=(
    "riglr-core"
    "riglr-evm-common"
)

declare -a TIER_2B=(
    "riglr-macros"  # Depends on riglr-core
)

declare -a TIER_3=(
    "riglr-events-core"
    "riglr-solana-tools"
    "riglr-evm-tools"
    "riglr-web-tools"
    "riglr-web-adapters"
)

declare -a TIER_4=(
    "riglr-solana-events"
    "riglr-hyperliquid-tools"
    "riglr-auth"
    "riglr-streams"
    "riglr-cross-chain-tools"
)

declare -a TIER_5=(
    "riglr-agents"
    "riglr-server"
    "riglr-graph-memory"
)

declare -a TIER_6=(
    "riglr-showcase"
    "riglr-indexer"
)

# Optional: The CLI tool
declare -a TIER_7=(
    "create-riglr-app"
)

# Function to check if a crate is already published
is_published() {
    local crate=$1
    local version=$(grep "^version" "$crate/Cargo.toml" | head -1 | cut -d'"' -f2)
    
    if cargo search "$crate" --limit 1 | grep -q "^$crate = \"$version\""; then
        return 0
    else
        return 1
    fi
}

# Function to publish a single crate
publish_crate() {
    local crate=$1
    
    print_info "Processing $crate..."
    
    # Check if already published (unless forced)
    if [ "$FORCE" != true ] && is_published "$crate"; then
        print_warn "$crate is already published with this version, skipping..."
        return 0
    fi
    
    # Change to crate directory
    cd "$crate"
    
    # Run tests if not skipped
    if [ "$SKIP_VERIFY" != true ]; then
        print_info "Running tests for $crate..."
        cargo test --quiet || {
            print_error "Tests failed for $crate"
            cd ..
            return 1
        }
    fi
    
    # Publish the crate
    if [ "$DRY_RUN" = true ]; then
        print_info "[DRY RUN] Would publish $crate"
        cargo publish --dry-run --allow-dirty || {
            print_error "Dry run failed for $crate"
            cd ..
            return 1
        }
    else
        print_info "Publishing $crate to crates.io..."
        cargo publish || {
            print_error "Failed to publish $crate"
            cd ..
            return 1
        }
        print_info "Successfully published $crate"
        
        # Wait for crates.io to index the crate
        print_info "Waiting ${DELAY} seconds for crates.io to index..."
        sleep "$DELAY"
    fi
    
    cd ..
}

# Function to publish a tier of crates
publish_tier() {
    local tier_name=$1
    shift
    local crates=("$@")
    
    print_info "===== Publishing Tier: $tier_name ====="
    
    for crate in "${crates[@]}"; do
        if [ -d "$crate" ]; then
            publish_crate "$crate" || {
                print_error "Failed to publish $crate. Stopping."
                exit 1
            }
        else
            print_warn "Crate directory $crate not found, skipping..."
        fi
    done
    
    print_info "Completed tier: $tier_name"
    echo ""
}

# Main execution
main() {
    print_info "Starting Riglr crates publishing process"
    
    if [ "$DRY_RUN" = true ]; then
        print_warn "Running in DRY RUN mode - no crates will actually be published"
    fi
    
    # Check for CARGO_REGISTRY_TOKEN
    if [ "$DRY_RUN" != true ] && [ -z "$CARGO_REGISTRY_TOKEN" ]; then
        print_warn "CARGO_REGISTRY_TOKEN not set. You may be prompted to login."
        print_info "Get your token from: https://crates.io/me"
        echo ""
    fi
    
    # Verify workspace builds before starting
    if [ "$SKIP_VERIFY" != true ]; then
        print_info "Verifying workspace builds..."
        cargo check --all || {
            print_error "Workspace check failed. Fix errors before publishing."
            exit 1
        }
    fi
    
    # Publish each tier in order
    publish_tier "1 - Base crates (no dependencies)" "${TIER_1[@]}"
    publish_tier "2 - Core functionality" "${TIER_2[@]}"
    publish_tier "2B - Macros (depends on core)" "${TIER_2B[@]}"
    publish_tier "3 - Tool implementations" "${TIER_3[@]}"
    publish_tier "4 - Advanced tools and integrations" "${TIER_4[@]}"
    publish_tier "5 - High-level abstractions" "${TIER_5[@]}"
    publish_tier "6 - Demo and showcase crates" "${TIER_6[@]}"
    
    # Ask about publishing the CLI tool
    if [ "$DRY_RUN" != true ]; then
        read -p "Do you want to publish create-riglr-app CLI tool? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            publish_tier "7 - CLI tool" "${TIER_7[@]}"
        fi
    else
        publish_tier "7 - CLI tool (optional)" "${TIER_7[@]}"
    fi
    
    print_info "===== Publishing Complete! ====="
    print_info "All crates have been successfully published to crates.io"
    print_info "Documentation will be automatically built at docs.rs"
    echo ""
    print_info "Next steps:"
    print_info "1. Verify all crates on https://crates.io/search?q=riglr"
    print_info "2. Check documentation at https://docs.rs/riglr-core"
    print_info "3. Create a GitHub release with changelog"
    print_info "4. Update the website documentation links"
}

# Run main function
main