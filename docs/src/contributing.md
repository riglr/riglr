# Contributing to riglr

Thank you for your interest in contributing to riglr! This guide will help you get started.

## Ways to Contribute

- **Report bugs**: Open issues for any bugs you encounter
- **Suggest features**: Propose new tools or improvements
- **Submit PRs**: Fix bugs or implement new features
- **Improve docs**: Fix typos, clarify explanations, or add examples
- **Write tutorials**: Share your experience building with riglr

## Development Setup

1. Fork and clone the repository
2. Install Rust (1.75+)
3. Install development dependencies:
   ```bash
   cargo install cargo-watch cargo-edit cargo-expand
   ```
4. Set up pre-commit hooks:
   ```bash
   ./scripts/setup-hooks.sh
   ```

## Code Guidelines

### Rust Code Style

- Follow standard Rust conventions
- Use `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Add tests for new functionality
- Document public APIs with doc comments

### Tool Development

When creating new tools:

1. Use the `#[tool]` macro
2. Return `Result<T, ToolError>`
3. Classify errors as retriable or permanent
4. Add comprehensive doc comments
5. Include usage examples
6. Write unit and integration tests

### Commit Messages

Follow conventional commits:

```
feat: Add new Jupiter V6 swap tool
fix: Handle slippage correctly in Raydium swaps
docs: Update SignerContext examples
test: Add integration tests for cross-chain bridges
```

## Testing

### Run All Tests

```bash
cargo test --workspace
```

### Run Specific Tests

```bash
cargo test -p riglr-solana-tools
cargo test test_swap_tokens
```

### Integration Tests

```bash
cargo test --workspace -- --ignored
```

## Documentation

### Building Docs Locally

```bash
cd docs
mdbook serve --open
```

### Adding Documentation

- Update relevant markdown files
- Include code examples
- Add to SUMMARY.md if new page
- Test locally before submitting

## Pull Request Process

1. Create a feature branch
2. Make your changes
3. Add tests
4. Update documentation
5. Run tests and linting
6. Submit PR with clear description
7. Address review feedback

## Release Process

Releases follow semantic versioning:

- **Patch**: Bug fixes (0.1.x)
- **Minor**: New features (0.x.0)
- **Major**: Breaking changes (x.0.0)

## Getting Help

- Discord: [Join our community](https://discord.gg/riglr)
- GitHub Discussions: Ask questions
- Documentation: [docs.riglr.dev](https://docs.riglr.dev)

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT/Apache 2.0).