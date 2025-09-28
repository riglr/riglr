# Riglr Agents Test Suite

This document describes the comprehensive testing strategy for the riglr-agents crate, implementing a three-layer testing approach designed for reliability, performance, and production readiness.

## Overview

The riglr-agents test suite is organized into three distinct layers:

1. **Unit Tests** - Fast, isolated tests for individual components
2. **Integration Tests** - Tests for component interactions and workflows
3. **End-to-End (E2E) Tests** - Full system tests with real blockchain connectivity

## Test Architecture

### Layer 1: Unit Tests
Located in `src/` files alongside the code they test.

**Purpose**: Verify individual functions, methods, and isolated component behavior.

**Characteristics**:
- Execute in milliseconds
- No external dependencies
- Use mocks and stubs
- High code coverage focus
- Run on every commit

**Coverage Goals**: >90% line coverage for core logic

### Layer 2: Integration Tests
Located in `tests/integration/` directory.

**Purpose**: Test component interactions, message passing, and multi-agent coordination.

**Characteristics**:
- Execute in seconds
- Use test harnesses and controlled environments
- Test cross-component workflows
- Validate error propagation and recovery
- Run on every commit

**Key Test Areas**:
- Agent-to-agent communication
- Dispatcher routing logic
- Registry operations
- Error handling chains
- Message serialization/deserialization

### Layer 3: E2E Tests
Located in `tests/e2e/` directory.

**Purpose**: Validate complete workflows with real blockchain connectivity.

**Characteristics**:
- Execute in minutes
- Require external blockchain access
- Test realistic scenarios
- Performance and stress testing
- Run selectively (main branch, PR labels)

**Key Test Areas**:
- Blockchain transaction execution
- Multi-chain coordination
- Real signer integration
- Performance under load
- Network resilience

## Test Features

The test suite uses Cargo features to control execution scope:

```toml
[features]
default = ["redis"]
redis = ["riglr-core/redis"]

# Test-specific features
test-utils = []                    # Basic test utilities
blockchain-tests = ["test-utils"]  # E2E blockchain tests
performance-tests = ["test-utils"] # Performance benchmarks
```

## Running Tests

### Quick Development Testing
```bash
# Run unit tests only (fastest)
cargo test --lib

# Run unit tests with test utilities
cargo test --lib --features test-utils
```

### Integration Testing
```bash
# Run integration tests
cargo test --test '*' --features test-utils

# Run specific integration test
cargo test --test agent_coordination --features test-utils
```

### E2E Testing
```bash
# Run E2E tests (requires blockchain connectivity)
cargo test --features blockchain-tests --test '*' -- --ignored

# Run specific E2E test
cargo test --test blockchain_integration --features blockchain-tests -- --ignored
```

### Performance Testing
```bash
# Run benchmarks
cargo bench --features performance-tests

# Run performance tests
cargo test --features performance-tests --release -- perf_
```

### Complete Test Suite
```bash
# Use the provided test runner
./run_tests.sh

# Or run manually
cargo test --all-features -- --test-threads=1
```

## Test Organization

```
tests/
├── README.md              # This documentation
├── lib.rs                 # Test crate root
├── common/                # Shared test utilities
│   ├── mod.rs             # Common module exports
│   ├── mock_agents.rs     # Mock agent implementations
│   ├── test_fixtures.rs   # Test data and fixtures
│   ├── rig_mocks.rs       # Rig framework mocks
│   ├── blockchain_harness.rs # Blockchain test setup
│   ├── scenario_builders.rs  # Test scenario builders
│   └── signer_integration.rs # Signer test utilities
├── integration/           # Layer 2: Integration tests
│   ├── mod.rs
│   ├── agent_coordination.rs    # Multi-agent workflows
│   ├── dispatcher_integration.rs # Dispatcher functionality
│   ├── message_passing.rs       # Communication tests
│   └── error_handling.rs        # Error propagation tests
└── e2e/                   # Layer 3: End-to-end tests
    ├── mod.rs
    ├── blockchain_integration.rs # Real blockchain tests
    ├── performance.rs            # Performance benchmarks
    ├── rig_integration.rs        # Rig framework integration
    └── signer_context.rs         # Signer integration tests
```

## Performance Requirements

### Unit Tests
- **Target**: <100ms total execution time
- **Individual Test**: <10ms per test
- **Memory Usage**: <50MB peak

### Integration Tests  
- **Target**: <5 seconds total execution time
- **Individual Test**: <500ms per test
- **Memory Usage**: <200MB peak

### E2E Tests
- **Target**: <30 minutes total execution time
- **Individual Test**: <5 minutes per test
- **Memory Usage**: <500MB peak
- **Network Timeout**: 30 seconds per blockchain call

## Test Coverage Goals

| Component | Unit Test Coverage | Integration Coverage | E2E Coverage |
|-----------|-------------------|---------------------|--------------|
| Core Agent Logic | >90% | >80% | >60% |
| Communication | >85% | >90% | >70% |
| Dispatcher | >90% | >85% | >50% |
| Registry | >90% | >80% | >40% |
| Error Handling | >95% | >90% | >60% |

## Environment Setup

### Required Environment Variables

```bash
# For basic testing
export ANTHROPIC_API_KEY="test_key"
export REDIS_URL="redis://localhost:6379"

# For E2E testing
export ETHEREUM_RPC_URL="https://eth.llamarpc.com"
export ARBITRUM_RPC_URL="https://arb1.arbitrum.io/rpc" 
export POLYGON_RPC_URL="https://polygon-rpc.com"
export BASE_RPC_URL="https://mainnet.base.org"
export SOLANA_RPC_URL="https://api.devnet.solana.com"
export NEO4J_URL="bolt://localhost:7687"
```

### Local Development Setup

```bash
# Install required tools
cargo install cargo-llvm-cov  # For coverage
cargo install criterion       # For benchmarks

# Start local services (optional)
docker run -d -p 6379:6379 redis:alpine
docker run -d -p 7687:7687 neo4j:latest
```

## CI/CD Integration

### GitHub Actions Workflow

The test suite integrates with GitHub Actions via `.github/workflows/riglr-agents-tests.yml`:

- **Unit Tests**: Run on every commit
- **Integration Tests**: Run on every commit  
- **E2E Tests**: Run on main branch pushes or PR label `test:e2e`
- **Benchmarks**: Run on main branch pushes only
- **Coverage**: Generated on main branch pushes

### Test Execution Strategy

1. **Fast Fail**: Unit tests run first, fail fast on format/lint issues
2. **Parallel Execution**: Independent test suites run in parallel
3. **Resource Management**: Memory-efficient execution with controlled concurrency
4. **Selective E2E**: E2E tests only run when necessary to save resources

### Coverage Reporting

Coverage reports are generated using `cargo-llvm-cov` and uploaded to Codecov:

```bash
# Generate coverage locally
cargo llvm-cov --package riglr-agents \
  --features test-utils \
  --lcov --output-path coverage.lcov
```

## Debugging Tests

### Common Issues

1. **Test Timeouts**: Increase timeout or reduce test scope
2. **Resource Contention**: Use `--test-threads=1` for problematic tests
3. **Environment Issues**: Verify required environment variables
4. **Network Connectivity**: Check RPC endpoints for E2E tests

### Debug Commands

```bash
# Run with detailed output
cargo test -- --nocapture --test-threads=1

# Run specific test with logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Run with backtraces
RUST_BACKTRACE=1 cargo test

# Profile test execution
cargo test --release -- --profile-time
```

## Adding New Tests

### Unit Tests
Add directly to the module being tested in `src/`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // Test implementation
    }
}
```

### Integration Tests
Create or extend files in `tests/integration/`:

```rust
use riglr_agents::*;
use crate::common::*;

#[tokio::test]
async fn test_integration_scenario() {
    // Integration test implementation
}
```

### E2E Tests
Create or extend files in `tests/e2e/`:

```rust
use riglr_agents::*;
use crate::common::*;

#[tokio::test]
#[ignore] // Mark as ignored for selective execution
async fn test_e2e_scenario() {
    // E2E test implementation
}
```

## Test Utilities

The `tests/common/` directory provides shared utilities:

- **MockAgents**: Configurable agent implementations for testing
- **TestFixtures**: Pre-built test data and scenarios
- **RigMocks**: Mock implementations of Rig framework components
- **BlockchainHarness**: Setup and teardown for blockchain tests
- **ScenarioBuilders**: Fluent builders for complex test scenarios

Example usage:
```rust
use crate::common::{MockAgent, TestFixtures, ScenarioBuilder};

let scenario = ScenarioBuilder::new()
    .with_agents(3)
    .with_blockchain_connectivity()
    .build();
```

## Maintenance

### Regular Tasks
- Review and update performance benchmarks monthly
- Verify E2E test stability with RPC endpoint changes
- Update test documentation with new features
- Monitor test execution times and optimize slow tests

### Quality Gates
- All tests must pass before merge
- Coverage must not decrease below thresholds
- New features require corresponding tests
- Performance regressions trigger review