# Contributing to {{project-name}}

Thank you for your interest in contributing to {{project-name}}! This document outlines the guidelines and best practices for contributing to this riglr-powered AI agent project.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ installed via [rustup](https://rustup.rs/)
- Git for version control
- Basic understanding of the [riglr framework](https://docs.riglr.dev)
- Familiarity with blockchain development ({% if primary-chain == "both" %}Solana and Ethereum{% else %}{{primary-chain | title_case}}{% endif %})

### Setting Up Development Environment

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/your-username/{{project-name}}.git
   cd {{project-name}}
   ```

2. **Set up your environment:**
   ```bash
   cp .env.example .env
   # Configure your .env with development API keys
   ```

3. **Install dependencies and run tests:**
   ```bash
   cargo build
   cargo test
   ```

## 📝 Code Style and Standards

### Rust Code Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for consistent formatting:
  ```bash
  cargo fmt
  ```
- Run `clippy` to catch common mistakes:
  ```bash
  cargo clippy -- -D warnings
  ```

### Code Organization

- **Tools**: Place new tools in appropriate modules (`src/tools/` if created)
- **Utilities**: Common utilities go in `src/utils/` (if created)
- **Configurations**: Agent configurations in `src/config/` (if created)
- **Tests**: Unit tests alongside code, integration tests in `tests/`

### Documentation

- Document all public functions and modules using Rust doc comments (`///`)
- Include examples in documentation where helpful
- Update README.md if adding new features or changing setup procedures

## 🔧 Development Workflow

### Branch Naming

Use descriptive branch names:
- `feature/add-arbitrage-tools` - New features
- `fix/balance-check-error` - Bug fixes
- `docs/update-api-examples` - Documentation updates
- `refactor/optimize-trading-logic` - Code refactoring

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add arbitrage opportunity detection tool

- Implements cross-DEX price comparison
- Adds profit calculation with gas estimation
- Includes risk assessment for arbitrage trades

Closes #123
```

Types:
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code refactoring
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks

### Pull Request Process

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes:**
   - Write clean, well-documented code
   - Add or update tests as needed
   - Update documentation

3. **Test thoroughly:**
   ```bash
   # Run tests
   cargo test
   
   # Check formatting
   cargo fmt --check
   
   # Check for common issues
   cargo clippy -- -D warnings
   
   # Test with testnet if applicable
   TEST_MODE=true cargo test -- --ignored
   ```

4. **Submit a pull request:**
   - Clear title describing the change
   - Detailed description of what was changed and why
   - Reference any related issues
   - Include testing instructions

## 🧪 Testing Guidelines

### Test Categories

- **Unit Tests**: Test individual functions and methods
- **Integration Tests**: Test complete workflows and agent interactions
- **End-to-End Tests**: Test full trading or analysis scenarios

### Writing Good Tests

```rust
#[tokio::test]
async fn test_portfolio_balance_calculation() {
    // Arrange
    let mut agent = create_test_agent().await;
    
    // Act
    let balance = agent.calculate_portfolio_value().await.unwrap();
    
    // Assert
    assert!(balance > 0.0);
    assert!(balance < 1_000_000.0); // Reasonable upper bound
}

#[tokio::test]
#[ignore] // For tests requiring live API keys
async fn test_live_price_feed() {
    // Integration test with real APIs
}
```

### Test Configuration

- Use `TEST_MODE=true` for safe testing without real transactions
- Mock external API calls in unit tests
- Use integration tests for full end-to-end scenarios
- Test both success and error scenarios

## 🚨 Security Considerations

### API Keys and Secrets

- **Never commit real API keys or private keys**
- Use `.env` files that are `.gitignore`d
- Use test/development keys for testing
- Rotate keys regularly in production

### Trading Safety

- Always test trading logic in paper trading mode first
- Include proper input validation and bounds checking
- Implement circuit breakers for unexpected behavior
- Log all trading decisions and outcomes

### Code Review Focus Areas

- Input validation and sanitization
- Error handling and graceful degradation
- Resource cleanup (database connections, file handles)
- Rate limiting and API quota management

## 🐛 Reporting Issues

When reporting bugs, please include:

1. **Environment information:**
   - Rust version (`rustc --version`)
   - Operating system
   - Relevant dependency versions

2. **Steps to reproduce:**
   - Clear, numbered steps
   - Expected vs. actual behavior
   - Any error messages or logs

3. **Minimal example:**
   - Simplest possible code that demonstrates the issue
   - Remove sensitive information (API keys, addresses)

4. **Additional context:**
   - Screenshots if relevant
   - Related issues or discussions
   - Potential solutions you've tried

## 💡 Feature Requests

For new features:

1. **Search existing issues** to avoid duplicates
2. **Describe the use case** - why is this needed?
3. **Provide implementation details** if you have ideas
4. **Consider backwards compatibility**
5. **Estimate complexity** and breaking changes

### Good Feature Request Example

```markdown
## Feature: Automated Portfolio Rebalancing

### Problem
Currently, the agent doesn't automatically rebalance portfolios based on 
target allocations, leading to drift over time.

### Proposed Solution
Add a `rebalance_portfolio()` tool that:
- Accepts target allocations (e.g., 50% SOL, 30% ETH, 20% stablecoins)
- Calculates current deviations from targets
- Executes trades to restore balance within tolerance thresholds

### Implementation Ideas
- New module: `src/tools/rebalancing.rs`
- Configuration in `.env`: `REBALANCE_THRESHOLD=5.0`
- Integration with existing trading tools

### Benefits
- Maintains desired risk profile
- Reduces manual intervention
- Improves long-term returns through systematic rebalancing
```

## 🤝 Community Guidelines

### Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help newcomers learn and contribute
- Maintain professional communication

### Getting Help

- **Documentation**: Check [riglr docs](https://docs.riglr.dev) first
- **Discussions**: Use GitHub Discussions for questions
- **Discord**: Join the [riglr Discord](https://discord.gg/riglr) for real-time chat
- **Issues**: Use GitHub Issues for bugs and feature requests

### Review Process

All contributions go through review:

1. **Automated checks**: CI runs tests, formatting, and linting
2. **Code review**: Maintainers review for quality, security, and design
3. **Testing**: Contributors and reviewers test the changes
4. **Documentation**: Ensure docs are updated appropriately

## 🏆 Recognition

Contributors are recognized through:

- GitHub contributor graphs
- Release notes acknowledgments
- Community showcase features
- Maintainer nominations for significant contributions

## 📚 Resources

### riglr Framework
- [riglr Documentation](https://docs.riglr.dev)
- [riglr Examples](https://github.com/riglr-project/examples)
- [API Reference](https://docs.riglr.dev/api/)

### Blockchain Development
{% if primary-chain == "solana" or primary-chain == "both" -%}
- [Solana Development Guide](https://docs.solana.com/)
- [Jupiter API Docs](https://station.jup.ag/docs/)
{% endif %}
{% if primary-chain == "ethereum" or primary-chain == "both" -%}
- [Ethereum Development](https://ethereum.org/developers/)
- [Uniswap Integration](https://docs.uniswap.org/)
{% endif %}

### AI & LLM Integration
- [rig-core Documentation](https://docs.rig.rs/)
- [OpenAI API](https://platform.openai.com/docs/)
- [Anthropic Claude](https://docs.anthropic.com/)

---

Thank you for contributing to {{project-name}}! Your help makes the riglr ecosystem stronger and more useful for everyone. 🚀