#!/bin/bash
set -e

echo "🚀 Starting riglr documentation build..."

# Define paths
DOCS_DIR=$(pwd)
REPO_ROOT=$(git rev-parse --show-toplevel)
TOOL_REF_DIR="$DOCS_DIR/src/tool-reference"
JSON_OUTPUT_DIR="$DOCS_DIR/target/doc"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Crates to document
CRATES=(
    "riglr-core"
    "riglr-solana-tools"
    "riglr-evm-tools"
    "riglr-web-tools"
    "riglr-cross-chain-tools"
    "riglr-hyperliquid-tools"
    "riglr-graph-memory"
)

echo "📦 Building rustdoc JSON for tool crates..."

# Check if nightly is installed
if ! rustup toolchain list | grep -q nightly; then
    echo -e "${YELLOW}Installing Rust nightly for rustdoc JSON generation...${NC}"
    rustup toolchain install nightly
fi

# Create output directory
mkdir -p "$JSON_OUTPUT_DIR"

# Generate rustdoc JSON for each crate
cd "$REPO_ROOT"
for CRATE in "${CRATES[@]}"; do
    echo "  - Generating docs for $CRATE..."
    if [ -d "$CRATE" ]; then
        cargo +nightly rustdoc \
            -p "$CRATE" \
            --all-features \
            -- \
            -Z unstable-options \
            --output-format json \
            --output-path "$JSON_OUTPUT_DIR" 2>/dev/null || {
                echo -e "${YELLOW}Warning: Could not generate rustdoc JSON for $CRATE${NC}"
            }
    else
        echo -e "${YELLOW}Warning: Crate directory $CRATE not found${NC}"
    fi
done

echo "🐍 Processing rustdoc JSON to generate tool reference..."

# Create Python script for parsing rustdoc JSON
cat > "$DOCS_DIR/parse_tools.py" << 'EOF'
#!/usr/bin/env python3
"""
Parse rustdoc JSON to extract tool documentation.
This is a simplified version - a production implementation would be more robust.
"""

import json
import sys
import os
from pathlib import Path

def parse_rustdoc_json(json_path, output_dir):
    """Parse rustdoc JSON and generate markdown documentation."""
    
    # This is a placeholder implementation
    # A real implementation would:
    # 1. Parse the JSON structure
    # 2. Find functions with #[tool] attribute
    # 3. Extract doc comments and signatures
    # 4. Generate markdown documentation
    
    print(f"  Processing {json_path.name}...")
    
    # For now, just note that the file exists
    # Real implementation would parse and generate markdown here
    
def main():
    if len(sys.argv) != 3:
        print("Usage: parse_tools.py <json_dir> <output_dir>")
        sys.exit(1)
    
    json_dir = Path(sys.argv[1])
    output_dir = Path(sys.argv[2])
    
    if not json_dir.exists():
        print(f"Error: JSON directory {json_dir} does not exist")
        sys.exit(1)
    
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Process each JSON file
    for json_file in json_dir.glob("*.json"):
        parse_rustdoc_json(json_file, output_dir)
    
    print("✅ Tool reference generation complete (using placeholder)")

if __name__ == "__main__":
    main()
EOF

chmod +x "$DOCS_DIR/parse_tools.py"

# Run the parser (if Python is available)
if command -v python3 &> /dev/null; then
    python3 "$DOCS_DIR/parse_tools.py" "$JSON_OUTPUT_DIR" "$TOOL_REF_DIR" || {
        echo -e "${YELLOW}Note: Tool reference auto-generation is not fully implemented yet${NC}"
    }
else
    echo -e "${YELLOW}Python not found - skipping tool reference generation${NC}"
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
cargo test --doc --workspace || {
    echo -e "${YELLOW}Warning: Some doc tests failed${NC}"
}

echo -e "${GREEN}🎉 Documentation pipeline complete!${NC}"