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

## 📋 Test Requirements and Known Issues

### Build and Compilation Notes

Some tests may have longer compilation times due to dependencies:
- **riglr-agents**: Many dependencies, longer compilation time
- **riglr-events-core**: Heavy async test setup
- **riglr-evm-tools**: Requires actual RPC endpoints for network tests

### Required Test Environment

For integration tests to pass, you may need:
- **API Keys**: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `ALCHEMY_API_KEY`
- **Local Services**: Redis (localhost:6379), PostgreSQL, Neo4j
- **Network Access**: For blockchain RPC calls and API providers
- **Test Wallets**: Pre-funded wallets on testnets (never use mainnet keys!)

### Known Test Issues

- **Timeouts**: Some blockchain RPC tests may timeout on slow connections
- **Rate Limiting**: Free tier API keys may hit limits during full test suite
- **Ignored Tests**: Tests marked `#[ignore]` may require specific setup or paid services
- **CI Environments**: Some tests may fail in CI without proper secrets setup

### System Requirements

- **Rust**: 1.75+ with cargo
- **Memory**: 8GB+ RAM for parallel test execution
- **Disk**: ~5GB for build artifacts
- **OS**: Linux/macOS/WSL2 recommended

## 🧪 End-to-End Integration Tests

The riglr project includes a comprehensive E2E test suite that validates the entire ecosystem in realistic scenarios.

**📚 For detailed E2E testing documentation, see [tests/E2E_TESTS_README.md](tests/E2E_TESTS_README.md)**

### Quick Start

```bash
# Run all E2E tests
./scripts/run_e2e_tests.sh

# Run specific suite
./scripts/run_e2e_tests.sh --suite 1

# Keep services running after tests
./scripts/run_e2e_tests.sh --keep-running
```

### Test Suites Overview

1. **Core Agent Workflow** (`riglr-agents`): Tests fundamental agent operations with blockchain tools
2. **Multi-Agent Coordination** (`riglr-agents`): Tests agent collaboration patterns
3. **Real-time Data Pipeline** (`riglr-streams`): Tests streaming data ingestion
4. **Web Service & Authentication** (`riglr-server`): Tests HTTP API layer
5. **Agent Memory & Knowledge Graph** (`riglr-graph-memory`): Tests persistent agent memory
6. **Application Scaffolding** (`create-riglr-app`): Tests project generation

### Prerequisites

- Docker & Docker Compose for test infrastructure
- Rust & Cargo for building and running tests
- Test wallets on public testnets (Solana Devnet, Ethereum Sepolia)
- API keys (Gemini for LLM operations)

For detailed setup instructions, test descriptions, troubleshooting, and CI/CD integration, please refer to the [E2E Tests Documentation](tests/E2E_TESTS_README.md).

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

## 🔧 CI Integration Tests Setup

### Required Secrets

The integration tests require the following secrets to be configured in your GitHub repository settings:

#### Blockchain Test Keys

These should be test-only private keys with minimal funds on testnets:

- `SOLANA_TEST_PRIVATE_KEY`: Base58-encoded Solana private key for devnet testing
- `EVM_TEST_PRIVATE_KEY`: Hex-encoded Ethereum private key for Sepolia testnet
- `HYPERLIQUID_TEST_PRIVATE_KEY`: Private key for Hyperliquid testnet (if applicable)

#### API Keys

These are for external service integrations:

- `DEXSCREENER_API_KEY`: API key for DexScreener service
- `LUNARCRUSH_API_KEY`: API key for LunarCrush social analytics
- `CROSS_CHAIN_TEST_KEYS`: JSON object with bridge service API keys

### Setting Up Secrets

1. Go to your repository's Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Add each secret with the appropriate name and value
4. Ensure test keys only have minimal testnet funds

### Test Workflow

The integration tests run in several scenarios:

1. **Scheduled**: Daily at 2 AM UTC to catch external API changes
2. **Manual Trigger**: Via workflow_dispatch for debugging
3. **Push to main/develop**: For repository owners only
4. **Pull Requests**: From the main repository (not forks)

For pull requests from forks, mock integration tests run instead to ensure code quality without exposing secrets.

### Local Testing

To run integration tests locally:

```bash
# Set up environment variables
export SOLANA_TEST_PRIVATE_KEY="your-test-key"
export EVM_TEST_PRIVATE_KEY="your-test-key"
export SOLANA_RPC_URL="https://api.devnet.solana.com"
export ETHEREUM_RPC_URL="https://ethereum-sepolia-rpc.publicnode.com"

# Run specific integration tests
cargo test --package riglr-solana-tools --test integration
cargo test --package riglr-evm-tools --test integration
cargo test --package riglr-showcase --test showcase_e2e
```

### Security Considerations

- **Never use production keys**: All keys should be for testnets only
- **Minimal funds**: Test accounts should have only enough funds for testing
- **Rotate regularly**: Change test keys periodically for security
- **Monitor usage**: Check test account activity regularly
- **Fork safety**: Integration tests with real services don't run on fork PRs

### Troubleshooting

#### Tests Failing Due to Rate Limits

If tests fail due to rate limiting:
1. Reduce test frequency
2. Implement retry logic with exponential backoff
3. Consider using mock services for most tests

#### Network Issues

For network-related failures:
1. Check if the RPC endpoints are operational
2. Verify network connectivity in CI environment
3. Consider using fallback RPC endpoints

#### Secret Not Available

If secrets aren't available:
1. Verify secret names match exactly
2. Check repository settings for secret visibility
3. Ensure workflow has permission to access secrets

## 📄 License

By contributing to riglr, you agree that your contributions will be licensed under the [MIT License](LICENSE).

---

Thank you for helping make riglr better! 🦀✨