#!/bin/bash

# Riglr Crates Pre-Publishing Verification Script
# This script verifies all crates are ready for publishing to crates.io

set -e  # Exit on any error

# Color output for better readability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters for summary
TOTAL_CHECKS=0
PASSED_CHECKS=0
WARNINGS=0

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
    ((PASSED_CHECKS++))
    ((TOTAL_CHECKS++))
}

print_warn() {
    echo -e "${YELLOW}[⚠]${NC} $1"
    ((WARNINGS++))
    ((TOTAL_CHECKS++))
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
    ((TOTAL_CHECKS++))
}

print_header() {
    echo ""
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}======================================${NC}"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "riglr-core" ]; then
    print_error "This script must be run from the riglr workspace root directory"
    exit 1
fi

# Get all crate directories
CRATES=($(find . -maxdepth 1 -type d -name "riglr-*" -o -name "create-riglr-app" | sort))

print_header "Riglr Pre-Publishing Verification"
print_info "Found ${#CRATES[@]} crates to verify"

# Check workspace configuration
print_header "Workspace Configuration"

if grep -q '^\[workspace\]' Cargo.toml; then
    print_success "Workspace configuration found"
else
    print_error "No workspace configuration in root Cargo.toml"
fi

if grep -q '^version = ' Cargo.toml; then
    WORKSPACE_VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
    print_info "Workspace version: $WORKSPACE_VERSION"
fi

# Check each crate
for crate_dir in "${CRATES[@]}"; do
    crate_name=$(basename "$crate_dir")
    print_header "Verifying $crate_name"
    
    cd "$crate_dir"
    
    # Check Cargo.toml exists
    if [ -f "Cargo.toml" ]; then
        print_success "Cargo.toml exists"
        
        # Check required fields
        if grep -q '^name = ' Cargo.toml; then
            print_success "Package name defined"
        else
            print_error "Package name missing"
        fi
        
        if grep -q '^version = ' Cargo.toml; then
            VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
            print_success "Version defined: $VERSION"
        else
            print_error "Version missing"
        fi
        
        if grep -q '^description = ' Cargo.toml; then
            print_success "Description defined"
        else
            print_error "Description missing (required for crates.io)"
        fi
        
        if grep -q '^license = ' Cargo.toml; then
            print_success "License defined"
        else
            print_error "License missing (required for crates.io)"
        fi
        
        if grep -q '^repository = ' Cargo.toml; then
            print_success "Repository URL defined"
        else
            print_warn "Repository URL missing (recommended)"
        fi
        
        if grep -q '^documentation = ' Cargo.toml; then
            print_success "Documentation URL defined"
        else
            print_warn "Documentation URL missing (will default to docs.rs)"
        fi
        
        if grep -q '^keywords = ' Cargo.toml; then
            print_success "Keywords defined"
        else
            print_warn "Keywords missing (recommended for discoverability)"
        fi
        
        if grep -q '^categories = ' Cargo.toml; then
            print_success "Categories defined"
        else
            print_warn "Categories missing (recommended for discoverability)"
        fi
    else
        print_error "Cargo.toml missing"
    fi
    
    # Check for README
    if [ -f "README.md" ]; then
        print_success "README.md exists"
        
        # Check README is not empty
        if [ -s "README.md" ]; then
            LINE_COUNT=$(wc -l < README.md)
            if [ "$LINE_COUNT" -gt 5 ]; then
                print_success "README.md has content ($LINE_COUNT lines)"
            else
                print_warn "README.md is very short ($LINE_COUNT lines)"
            fi
        else
            print_error "README.md is empty"
        fi
    else
        print_warn "README.md missing (recommended)"
    fi
    
    # Check for LICENSE file or license field
    if [ -f "../LICENSE" ] || [ -f "LICENSE" ]; then
        print_success "LICENSE file found"
    elif grep -q '^license = "MIT"' Cargo.toml; then
        print_success "MIT license specified in Cargo.toml"
    else
        print_warn "No LICENSE file (relies on workspace license)"
    fi
    
    # Check if crate builds
    print_info "Checking if crate builds..."
    if cargo check --quiet 2>/dev/null; then
        print_success "Crate builds successfully"
    else
        print_error "Crate fails to build"
    fi
    
    # Check for path dependencies (these need to be updated before publishing)
    if grep -q 'path = "../' Cargo.toml; then
        PATH_DEPS=$(grep 'path = "../' Cargo.toml | wc -l)
        print_warn "Found $PATH_DEPS path dependencies (need to be updated for publishing)"
    else
        print_success "No local path dependencies"
    fi
    
    # Check if already published
    if cargo search "$crate_name" --limit 1 2>/dev/null | grep -q "^$crate_name = "; then
        PUBLISHED_VERSION=$(cargo search "$crate_name" --limit 1 | head -1 | cut -d'"' -f2)
        if [ "$VERSION" = "$PUBLISHED_VERSION" ]; then
            print_warn "Already published with version $VERSION (needs version bump)"
        else
            print_info "Published version: $PUBLISHED_VERSION, local version: $VERSION"
        fi
    else
        print_info "Not yet published to crates.io"
    fi
    
    cd ..
done

# Check for API token
print_header "Publishing Requirements"

if [ -n "$CARGO_REGISTRY_TOKEN" ]; then
    print_success "CARGO_REGISTRY_TOKEN is set"
else
    print_warn "CARGO_REGISTRY_TOKEN not set (will need to login manually)"
    print_info "Get your token from: https://crates.io/me"
fi

# Run workspace-wide tests
print_header "Workspace Tests"

print_info "Running workspace tests..."
if cargo test --all --quiet 2>/dev/null; then
    print_success "All tests pass"
else
    print_error "Some tests fail"
fi

# Summary
print_header "Verification Summary"

FAILED_CHECKS=$((TOTAL_CHECKS - PASSED_CHECKS - WARNINGS))

echo ""
echo -e "Total checks: ${TOTAL_CHECKS}"
echo -e "${GREEN}Passed: ${PASSED_CHECKS}${NC}"
echo -e "${YELLOW}Warnings: ${WARNINGS}${NC}"
echo -e "${RED}Failed: ${FAILED_CHECKS}${NC}"

echo ""
if [ "$FAILED_CHECKS" -eq 0 ]; then
    if [ "$WARNINGS" -gt 0 ]; then
        print_warn "Ready to publish with $WARNINGS warnings"
        print_info "Review warnings above and fix if needed"
    else
        print_success "All checks passed! Ready to publish to crates.io"
    fi
    print_info "Run './scripts/publish.sh --dry-run' to do a dry run"
    print_info "Run './scripts/publish.sh' to publish all crates"
else
    print_error "Found $FAILED_CHECKS errors that must be fixed before publishing"
    print_info "Fix the errors above and run this script again"
    exit 1
fi