#!/bin/bash

# Script to add docs.rs metadata configuration to all crates
# This ensures proper documentation building on docs.rs

set -e

# Color output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Get all crate directories
CRATES=($(find . -maxdepth 1 -type d -name "riglr-*" -o -name "create-riglr-app" | sort))

print_info "Configuring docs.rs metadata for ${#CRATES[@]} crates"

# Function to add docs.rs metadata to a Cargo.toml
add_docs_metadata() {
    local cargo_file=$1
    local crate_name=$2
    
    # Check if docs.rs metadata already exists
    if grep -q '\[package\.metadata\.docs\.rs\]' "$cargo_file"; then
        print_info "  docs.rs metadata already exists"
        return 0
    fi
    
    # Determine which features to document based on the crate
    local features_line=""
    
    case "$crate_name" in
        riglr-auth)
            features_line='all-features = false
features = ["web3auth", "privy"]'
            ;;
        riglr-agents)
            features_line='all-features = false
features = ["distributed", "solana", "evm"]'
            ;;
        riglr-showcase)
            features_line='all-features = false
features = ["cross-chain", "hyperliquid", "web-adapters"]'
            ;;
        riglr-core)
            features_line='all-features = false
features = ["redis"]'
            ;;
        *)
            # For most crates, document all features
            features_line='all-features = true'
            ;;
    esac
    
    # Add the metadata section at the end of the file
    cat >> "$cargo_file" << EOF

[package.metadata.docs.rs]
$features_line
rustdoc-args = ["--cfg", "docsrs"]
targets = ["x86_64-unknown-linux-gnu"]
EOF
    
    print_success "  Added docs.rs metadata"
}

# Function to update documentation link
update_doc_link() {
    local cargo_file=$1
    local crate_name=$2
    
    # Check if documentation field exists
    if grep -q '^documentation = ' "$cargo_file"; then
        # Update to ensure it points to docs.rs
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s|^documentation = .*|documentation = \"https://docs.rs/$crate_name\"|" "$cargo_file"
        else
            sed -i "s|^documentation = .*|documentation = \"https://docs.rs/$crate_name\"|" "$cargo_file"
        fi
        print_success "  Updated documentation URL"
    else
        # Add documentation field after repository field if it exists
        if grep -q '^repository = ' "$cargo_file"; then
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "/^repository = /a\\
documentation = \"https://docs.rs/$crate_name\"" "$cargo_file"
            else
                sed -i "/^repository = /a\documentation = \"https://docs.rs/$crate_name\"" "$cargo_file"
            fi
            print_success "  Added documentation URL"
        fi
    fi
}

# Function to add #![cfg_attr(docsrs, feature(doc_cfg))] to lib.rs
add_docsrs_cfg() {
    local lib_file=$1
    
    if [ ! -f "$lib_file" ]; then
        return 0
    fi
    
    # Check if the attribute already exists
    if grep -q '#!\[cfg_attr(docsrs' "$lib_file"; then
        print_info "  docsrs cfg attribute already exists in lib.rs"
        return 0
    fi
    
    # Add the attribute at the beginning of the file
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS sed
        sed -i '' '1i\
#![cfg_attr(docsrs, feature(doc_cfg))]\
' "$lib_file"
    else
        # Linux sed
        sed -i '1i#![cfg_attr(docsrs, feature(doc_cfg))]\n' "$lib_file"
    fi
    
    print_success "  Added docsrs cfg attribute to lib.rs"
}

# Process each crate
for crate_dir in "${CRATES[@]}"; do
    crate_name=$(basename "$crate_dir")
    cargo_file="$crate_dir/Cargo.toml"
    lib_file="$crate_dir/src/lib.rs"
    
    print_info "Processing $crate_name..."
    
    if [ -f "$cargo_file" ]; then
        # Add docs.rs metadata
        add_docs_metadata "$cargo_file" "$crate_name"
        
        # Update documentation link
        update_doc_link "$cargo_file" "$crate_name"
        
        # Add docsrs cfg to lib.rs
        add_docsrs_cfg "$lib_file"
    else
        print_warn "No Cargo.toml found in $crate_dir"
    fi
done

print_info ""
print_success "docs.rs configuration complete!"
print_info ""
print_info "The following has been configured:"
print_info "1. Added [package.metadata.docs.rs] sections to control documentation building"
print_info "2. Updated documentation URLs to point to docs.rs"
print_info "3. Added docsrs cfg attributes for conditional compilation"
print_info ""
print_info "When crates are published, docs.rs will automatically:"
print_info "- Build documentation with specified features"
print_info "- Use stable Rust compiler"
print_info "- Generate cross-references between crates"
print_info ""
print_warn "Note: Documentation will be built automatically after publishing to crates.io"