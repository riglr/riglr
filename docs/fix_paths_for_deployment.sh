#!/bin/bash

# Fix paths in mdBook HTML files for deployment to /docs/ subdirectory
echo "Fixing paths for /docs/ deployment..."

# Navigate to the book directory
cd book || exit 1

# Fix all HTML files to use absolute paths with /docs/ prefix
find . -name "*.html" -exec sed -i \
    -e 's|href="css/|href="/docs/css/|g' \
    -e 's|href="FontAwesome/|href="/docs/FontAwesome/|g' \
    -e 's|href="fonts/|href="/docs/fonts/|g' \
    -e 's|href="highlight\.css"|href="/docs/highlight.css"|g' \
    -e 's|href="tomorrow-night\.css"|href="/docs/tomorrow-night.css"|g' \
    -e 's|href="ayu-highlight\.css"|href="/docs/ayu-highlight.css"|g' \
    -e 's|href="src/css/|href="/docs/src/css/|g' \
    -e 's|href="theme/|href="/docs/theme/|g' \
    -e 's|href="favicon\.|href="/docs/favicon.|g' \
    -e 's|src="toc\.js"|src="/docs/toc.js"|g' \
    -e 's|src="mark\.min\.js"|src="/docs/mark.min.js"|g' \
    -e 's|src="elasticlunr\.min\.js"|src="/docs/elasticlunr.min.js"|g' \
    -e 's|src="searcher\.js"|src="/docs/searcher.js"|g' \
    -e 's|src="clipboard\.min\.js"|src="/docs/clipboard.min.js"|g' \
    -e 's|src="highlight\.js"|src="/docs/highlight.js"|g' \
    -e 's|src="book\.js"|src="/docs/book.js"|g' \
    -e 's|path_to_root = "";|path_to_root = "/docs/";|g' \
    -e 's|window.path_to_searchindex_js = "searchindex\.js"|window.path_to_searchindex_js = "/docs/searchindex.js"|g' \
    {} +

# Also fix paths in JavaScript files
find . -name "*.js" -exec sed -i \
    -e 's|"css/|"/docs/css/|g' \
    -e 's|"FontAwesome/|"/docs/FontAwesome/|g' \
    -e 's|"fonts/|"/docs/fonts/|g' \
    -e 's|"theme/|"/docs/theme/|g' \
    -e 's|"searchindex\.js"|"/docs/searchindex.js"|g' \
    {} + 2>/dev/null || true

echo "✓ Path fixes completed"