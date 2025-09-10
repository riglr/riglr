#!/bin/bash
set -e

echo "🚀 Starting riglr documentation build..."

# Note: This script requires a recent nightly Rust toolchain (Sept 2025 or later)
# to avoid SQLX compilation issues. Run 'rustup update nightly' if you encounter
# hanging or recursion overflow errors during rustdoc generation.

# Define paths
DOCS_DIR=$(pwd)
REPO_ROOT=$(git rev-parse --show-toplevel)

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "📦 Generating comprehensive API documentation..."

# Run the documentation generator
if command -v python3 &> /dev/null; then
    echo "  Generating rustdoc JSON for all workspace crates..."
    cd "$REPO_ROOT"
    # Generate fresh rustdoc JSON files for each crate in the workspace
    # Using SQLX_OFFLINE=true to prevent database connections and avoid compilation hangs
    for crate_dir in riglr-*; do
        if [ -d "$crate_dir" ]; then
            echo "    Processing $crate_dir..."
            cd "$REPO_ROOT/$crate_dir"
            SQLX_OFFLINE=true cargo +nightly rustdoc --lib --all-features -- -Z unstable-options --output-format json 2>/dev/null || {
                echo "      Warning: Failed to generate rustdoc JSON for $crate_dir"
                true
            }
        fi
    done
    echo "  ✅ Generated rustdoc JSON files"
    cd "$DOCS_DIR"
    
    echo "  Generating API documentation..."
    mkdir -p "$DOCS_DIR/src/api-reference"
    # Generate API documentation with detailed output
    # Use SQLX_OFFLINE to prevent database connections in the Python script
    if SQLX_OFFLINE=true python3 "$DOCS_DIR/generate_api_docs.py" "$REPO_ROOT" "$DOCS_DIR/src/api-reference"; then
        echo "  ✅ Generated API docs with rustdoc JSON"
    else
        echo -e "${YELLOW}Warning: Some API docs may be incomplete${NC}"
    fi
    echo "  Generating dependency graph..."
    if SQLX_OFFLINE=true python3 "$DOCS_DIR/generate_dependency_graph.py" "$REPO_ROOT" "$DOCS_DIR/src/concepts/dependency-graph.md"; then
        echo "  ✅ Generated dependency graph"
    else
        echo -e "${YELLOW}Failed to generate dependency graph${NC}"
    fi
else
    echo -e "${YELLOW}Python not found - skipping documentation generation${NC}"
fi

echo "📚 Building mdbook..."

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo -e "${YELLOW}Installing mdbook...${NC}"
    cargo install mdbook
fi

# Build the book
cd "$DOCS_DIR"
mdbook build

echo -e "${GREEN}✅ Documentation build complete!${NC}"
echo ""
echo "📖 To view the documentation locally, run:"
echo "   cd docs && mdbook serve --open"
echo ""
echo "🌐 To deploy to production:"
echo "   1. The 'book' directory contains the static site"
echo "   2. Deploy to your hosting service (GitHub Pages, Netlify, etc.)"
echo ""

# Optional: Run tests on code examples in documentation
echo "🧪 Testing code examples in documentation..."
cd "$REPO_ROOT"
SQLX_OFFLINE=true cargo test --doc --workspace || {
    echo -e "${YELLOW}Warning: Some doc tests failed${NC}"
}

echo -e "${GREEN}🎉 Documentation pipeline complete!${NC}"