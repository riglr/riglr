#!/bin/bash

# Script to update Cargo.toml files from path dependencies to version dependencies
# This is required before publishing to crates.io

set -e

# Color output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Default version from workspace
DEFAULT_VERSION="0.3.0"

# Parse arguments
ACTION="prepare"  # prepare or restore
VERSION="$DEFAULT_VERSION"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --restore) ACTION="restore" ;;
        --version) VERSION="$2"; shift ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --restore       Restore path dependencies from backups"
            echo "  --version VER   Version to use for dependencies (default: $DEFAULT_VERSION)"
            echo "  --help          Show this help message"
            echo ""
            echo "This script converts path dependencies to version dependencies for publishing."
            echo "Backup files are created as Cargo.toml.bak"
            exit 0
            ;;
        *) print_error "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

# Get all crate directories
CRATES=($(find . -maxdepth 1 -type d -name "riglr-*" -o -name "create-riglr-app" | sort))

if [ "$ACTION" = "restore" ]; then
    print_info "Restoring original Cargo.toml files from backups..."
    
    for crate_dir in "${CRATES[@]}"; do
        crate_name=$(basename "$crate_dir")
        
        if [ -f "$crate_dir/Cargo.toml.bak" ]; then
            print_info "Restoring $crate_name/Cargo.toml"
            mv "$crate_dir/Cargo.toml.bak" "$crate_dir/Cargo.toml"
        else
            print_warn "No backup found for $crate_name"
        fi
    done
    
    print_info "Restoration complete!"
    exit 0
fi

print_info "Preparing Cargo.toml files for publishing..."
print_info "Using version: $VERSION"

# Create a temporary sed script for replacements
cat > /tmp/update_deps.sed << EOF
# Update riglr dependencies from path to version
s/riglr-core = { path = "\.\.\/riglr-core"[^}]*}/riglr-core = { version = "$VERSION" }/g
s/riglr-config = { path = "\.\.\/riglr-config"[^}]*}/riglr-config = { version = "$VERSION" }/g
s/riglr-macros = { path = "\.\.\/riglr-macros"[^}]*}/riglr-macros = { version = "$VERSION" }/g
s/riglr-solana-tools = { path = "\.\.\/riglr-solana-tools"[^}]*}/riglr-solana-tools = { version = "$VERSION" }/g
s/riglr-evm-tools = { path = "\.\.\/riglr-evm-tools"[^}]*}/riglr-evm-tools = { version = "$VERSION" }/g
s/riglr-evm-common = { path = "\.\.\/riglr-evm-common"[^}]*}/riglr-evm-common = { version = "$VERSION" }/g
s/riglr-web-tools = { path = "\.\.\/riglr-web-tools"[^}]*}/riglr-web-tools = { version = "$VERSION" }/g
s/riglr-web-adapters = { path = "\.\.\/riglr-web-adapters"[^}]*}/riglr-web-adapters = { version = "$VERSION" }/g
s/riglr-graph-memory = { path = "\.\.\/riglr-graph-memory"[^}]*}/riglr-graph-memory = { version = "$VERSION" }/g
s/riglr-auth = { path = "\.\.\/riglr-auth"[^}]*}/riglr-auth = { version = "$VERSION" }/g
s/riglr-events-core = { path = "\.\.\/riglr-events-core"[^}]*}/riglr-events-core = { version = "$VERSION" }/g
s/riglr-solana-events = { path = "\.\.\/riglr-solana-events"[^}]*}/riglr-solana-events = { version = "$VERSION" }/g
s/riglr-streams = { path = "\.\.\/riglr-streams"[^}]*}/riglr-streams = { version = "$VERSION" }/g
s/riglr-indexer = { path = "\.\.\/riglr-indexer"[^}]*}/riglr-indexer = { version = "$VERSION" }/g
s/riglr-agents = { path = "\.\.\/riglr-agents"[^}]*}/riglr-agents = { version = "$VERSION" }/g
s/riglr-hyperliquid-tools = { path = "\.\.\/riglr-hyperliquid-tools"[^}]*}/riglr-hyperliquid-tools = { version = "$VERSION" }/g
s/riglr-cross-chain-tools = { path = "\.\.\/riglr-cross-chain-tools"[^}]*}/riglr-cross-chain-tools = { version = "$VERSION" }/g
s/riglr-showcase = { path = "\.\.\/riglr-showcase"[^}]*}/riglr-showcase = { version = "$VERSION" }/g
s/riglr-server = { path = "\.\.\/riglr-server"[^}]*}/riglr-server = { version = "$VERSION" }/g

# Handle workspace dependencies (these should already use workspace = true, but just in case)
s/riglr-\([a-z-]*\) = { workspace = true }/riglr-\1 = { version = "$VERSION" }/g
EOF

# Process each crate
for crate_dir in "${CRATES[@]}"; do
    crate_name=$(basename "$crate_dir")
    cargo_file="$crate_dir/Cargo.toml"
    
    if [ -f "$cargo_file" ]; then
        print_info "Processing $crate_name..."
        
        # Create backup
        cp "$cargo_file" "$cargo_file.bak"
        
        # Count path dependencies before
        path_deps_before=$(grep -c 'path = "../' "$cargo_file" || true)
        
        # Apply sed script
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS sed requires different syntax
            sed -i '' -f /tmp/update_deps.sed "$cargo_file"
        else
            # Linux sed
            sed -i -f /tmp/update_deps.sed "$cargo_file"
        fi
        
        # Count path dependencies after
        path_deps_after=$(grep -c 'path = "../' "$cargo_file" || true)
        
        if [ "$path_deps_before" -gt 0 ]; then
            converted=$((path_deps_before - path_deps_after))
            print_info "  Converted $converted path dependencies to version dependencies"
            if [ "$path_deps_after" -gt 0 ]; then
                print_warn "  Still has $path_deps_after non-riglr path dependencies"
            fi
        else
            print_info "  No path dependencies to convert"
        fi
    else
        print_warn "No Cargo.toml found in $crate_dir"
    fi
done

# Clean up
rm -f /tmp/update_deps.sed

print_info "Update complete!"
print_info ""
print_info "Next steps:"
print_info "1. Review the changes: git diff"
print_info "2. Run verification: ./scripts/verify-publish.sh"
print_info "3. Publish crates: ./scripts/publish.sh"
print_info "4. After publishing, restore path dependencies: $0 --restore"
print_info ""
print_warn "Note: Backup files created as *.bak"
print_warn "To restore original files, run: $0 --restore"