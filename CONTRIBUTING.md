# Contributing to riglr

Thank you for your interest in contributing to riglr! This document outlines the process for contributing to the project and helps ensure a smooth experience for everyone involved.

## 🎯 Project Vision

riglr aims to be the premier Rust ecosystem for building high-performance, resilient, and developer-friendly on-chain AI agents. We value:

- **Modularity**: Independent, focused crates that can be used separately or together
- **Developer Experience**: Ease of use through great APIs, comprehensive documentation, and helpful error messages
- **Performance & Resilience**: Production-ready code with proper error handling, retries, and timeouts
- **rig-native**: Seamless integration with the rig framework
- **Community-driven**: Welcoming environment for contributors of all skill levels

## 🚀 Getting Started

### Prerequisites

- **Rust**: Install the latest stable version via [rustup](https://rustup.rs/)
- **Git**: For version control
- **IDE/Editor**: We recommend VS Code with rust-analyzer, but any editor works

### Setting Up Your Development Environment

1. **Fork and clone the repository**:
   ```bash
   git clone https://github.com/your-username/riglr.git
   cd riglr
   ```

2. **Build the workspace**:
   ```bash
   cargo build --workspace
   ```

3. **Run tests**:
   ```bash
   cargo test --workspace
   ```

4. **Check code formatting and linting**:
   ```bash
   cargo fmt --check
   cargo clippy --workspace -- -D warnings
   ```

## 📝 Development Workflow

### 1. Creating Issues

Before starting work, please:

- **Search existing issues** to avoid duplicates
- **Use issue templates** when creating new issues
- **Provide detailed context** including use cases and expected behavior
- **Label appropriately** (bug, feature, documentation, etc.)

### 2. Working on Code

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**:
   - Follow the existing code style and patterns
   - Add tests for new functionality
   - Update documentation as needed
   - Ensure all tests pass

3. **Commit your changes**:
   ```bash
   git commit -m "feat: add new balance checking tool for Solana"
   ```

   Use [conventional commits](https://conventionalcommits.org/):
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation changes
   - `test:` for test additions/changes
   - `refactor:` for code refactoring
   - `chore:` for maintenance tasks

### 3. Pull Requests

1. **Push your branch**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a pull request**:
   - Use the PR template
   - Link to related issues
   - Provide a clear description of changes
   - Include examples of usage if applicable

3. **Respond to feedback**:
   - Address review comments promptly
   - Update tests and documentation as requested
   - Keep the PR focused and atomic

## 🏗️ Code Standards

### Rust Guidelines

- **Follow standard Rust conventions** from the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- **Use `cargo fmt`** for consistent formatting
- **Address `cargo clippy` warnings**
- **Write comprehensive documentation** with examples
- **Include unit and integration tests**

### Error Handling

- **Use `Result<T, E>` types** for all fallible operations
- **Avoid `.unwrap()` and `.expect()`** in production code
- **Create specific error types** using `thiserror`
- **Provide helpful error messages** that guide users toward solutions

### Async Code

- **Use `tokio` as the async runtime**
- **Implement `Send + Sync`** for all async types
- **Use `async-trait` when needed**
- **Handle cancellation gracefully**

### Documentation

- **Document all public APIs** with doc comments
- **Include examples** in doc comments when helpful
- **Write crate-level documentation** explaining purpose and usage
- **Update README files** when adding new features

### Testing

- **Write unit tests** for individual functions and methods
- **Write integration tests** for crate-level functionality  
- **Use meaningful test names** that describe what's being tested
- **Mock external dependencies** in tests
- **Test error conditions** as well as happy paths

## 🧪 Testing Guidelines

### Testing Strategy

This project uses a two-tiered testing approach:

#### Unit Tests (Default)
Run locally with: `cargo test`
- Executes quickly (2-5 minutes)
- Provides quick feedback during development
- Runs in main CI for every PR
- Does not require external dependencies

#### Integration Tests
Integration tests require Docker and are controlled by a feature flag.

**Prerequisites**: Docker must be running

Run integration tests with:
```bash
# Run integration tests for a specific package
cargo test --package riglr-core --features integration-tests
cargo test --package riglr-evm-tools --features integration-tests
cargo test --package riglr-solana-tools --features integration-tests

# Run all tests including integration tests
cargo test --all-features
```

Integration tests automatically:
- Spin up Redis instances for testing queues and idempotency stores
- Launch Ethereum nodes (Anvil) for EVM integration tests
- Start Solana validators for Solana integration tests

#### All Tests
Run locally with: `cargo test --all-features`
- Executes all tests (unit + integration)
- Use before submitting significant changes
- Requires Docker to be running

#### Adding New Tests
- Use `#[cfg(feature = "integration-tests")]` for tests requiring Docker/external services
- Do NOT use `#[ignore]` for new integration tests
- Follow existing patterns in the codebase

### Unit Tests

Place unit tests in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let job = Job::new("test_tool", serde_json::json!({}));
        assert_eq!(job.tool_name, "test_tool");
        assert_eq!(job.max_retries, 3); // default value
    }

    #[tokio::test]
    async fn test_async_function() {
        // Test async functionality
    }
}
```

### Integration Tests

Place integration tests in the `tests/` directory:

```rust
// tests/integration_test.rs
use riglr_core::{JobQueue, InMemoryJobQueue};

#[tokio::test]
async fn test_job_queue_integration() {
    let queue = InMemoryJobQueue::new();
    // Test full workflow
}
```

## 🎨 Crate Structure

When adding new functionality:

### New Tools

1. **Add to appropriate crate** (`riglr-solana-tools`, `riglr-evm-tools`, etc.)
2. **Use the `#[tool]` macro** for automatic rig integration
3. **Include comprehensive error handling**
4. **Add examples** in the `examples/` directory

### New Crates

1. **Discuss the need** in an issue first
2. **Follow naming convention**: `riglr-{functionality}`
3. **Include proper `Cargo.toml` metadata**
4. **Add to workspace members**
5. **Create comprehensive documentation**

## 📞 Getting Help

- **GitHub Discussions**: For questions about usage or development
- **GitHub Issues**: For bug reports and feature requests
- **Code Review**: Don't hesitate to ask for help in PRs

## 🎉 Recognition

All contributors will be:

- **Listed in CONTRIBUTORS.md**
- **Mentioned in release notes** for significant contributions
- **Invited to join** the riglr organization (for regular contributors)

## 📄 License

By contributing to riglr, you agree that your contributions will be licensed under the [MIT License](LICENSE).

---

Thank you for helping make riglr better! 🦀✨