# riglr Documentation

This directory contains the source for the official riglr documentation site, built with [mdBook](https://rust-lang.github.io/mdBook/).

## 📚 Viewing Documentation

### Online
Visit [riglr.com/docs](https://riglr.com/docs) for the latest published documentation.

### Local Development
To view and edit the documentation locally:

```bash
# Install mdbook (if not already installed)
cargo install mdbook

# Serve the documentation with live reload
mdbook serve --open

# The docs will be available at http://localhost:3000
```

## 🏗️ Building Documentation

### Quick Build
```bash
mdbook build
```

This creates the static site in the `book/` directory.

### Full Build with Auto-Generation
```bash
./build.sh
```

This script:
1. Generates rustdoc JSON for all crates
2. Extracts tool documentation from source code
3. Builds the complete mdBook site
4. Runs doc tests

## 📁 Structure

```
docs/
├── book.toml              # mdBook configuration
├── src/                   # Documentation source files
│   ├── SUMMARY.md         # Table of contents
│   ├── introduction.md    # Home page
│   ├── getting-started/   # Quick start guides
│   ├── tutorials/         # Step-by-step tutorials
│   ├── concepts/          # Core concepts
│   ├── tool-reference/    # Auto-generated tool docs
│   └── deployment/        # Deployment guides
├── build.sh              # Build script with auto-generation
└── book/                 # Generated static site (git ignored)
```

## ✍️ Contributing

### Adding New Content

1. Create/edit markdown files in `src/`
2. Add new pages to `src/SUMMARY.md`
3. Test locally with `mdbook serve`
4. Submit PR with your changes

### Writing Guidelines

- Use clear, concise language
- Include code examples
- Add links to related content
- Test all code snippets
- Follow the existing structure

### Code Examples

Always test code examples:

```rust
// Use complete, runnable examples
use riglr_core::signer::SignerContext;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Your code here
    Ok(())
}
```

## 🚀 Deployment

The documentation is automatically deployed when changes are pushed to the main branch:

1. GitHub Actions runs `build.sh`
2. Static site is generated in `book/`
3. Site is deployed to GitHub Pages / Netlify / Vercel
4. Available at riglr.com/docs

### Manual Deployment

```bash
# Build the documentation
./build.sh

# Deploy to GitHub Pages
ghp-import -n -p -f book

# Or deploy to Netlify
netlify deploy --dir=book --prod
```

## 🔧 Tool Reference Auto-Generation

The tool reference is automatically generated from source code:

1. `rustdoc` generates JSON documentation
2. Python script parses for `#[tool]` functions
3. Markdown files are generated in `src/tool-reference/`
4. Integrated into the final documentation

To regenerate tool docs manually:
```bash
cd docs
./build.sh
```

## 📦 Dependencies

- **mdBook**: Static site generator
- **Rust nightly**: For rustdoc JSON generation (optional)
- **Python 3**: For parsing rustdoc JSON (optional)

## 🐛 Troubleshooting

### mdbook command not found
```bash
cargo install mdbook
```

### Build script fails
```bash
# Make sure you're in the docs directory
cd docs

# Make script executable
chmod +x build.sh

# Run with debug output
bash -x build.sh
```

### Port already in use
```bash
# Use a different port
mdbook serve --port 3001
```

## 📄 License

The documentation is licensed under the same terms as the riglr project (MIT/Apache 2.0).

## 🤝 Getting Help

- Open an issue for documentation problems
- Join our Discord for questions
- See [Contributing Guide](src/contributing.md) for more details