# End-to-End Integration Tests for Riglr

This document describes the comprehensive E2E integration test suite for the riglr blockchain automation framework.

## Overview

The E2E test suite validates the entire riglr ecosystem including:
- Core agent workflows with blockchain interactions
- Multi-agent coordination patterns
- Real-time data processing pipelines
- Web service authentication and APIs
- Agent memory and knowledge graphs
- Application scaffolding tools

## Test Environment

### Prerequisites

1. **Docker & Docker Compose**: For running test infrastructure
2. **Rust & Cargo**: For building and running tests
3. **Test Wallets**: Pre-funded wallets on public testnets (Solana Devnet, Ethereum Sepolia)
4. **API Keys**: Gemini API key for LLM operations

### Infrastructure Services

The test suite uses Docker Compose to manage:
- **Redis**: Distributed registry and job queues
- **PostgreSQL**: Event indexer database
- **Neo4j**: Graph memory storage
- **Wiremock**: Mock external API endpoints

## Test Suites

### Suite 1: Core Agent Workflow (riglr-agents)

Tests fundamental agent operations with blockchain tools:

- **Test 1.1**: Basic read-only tool execution
  - Queries SOL balance from Solana Devnet
  - Validates ApplicationContext and RPC client integration
  
- **Test 1.2**: Secure transaction execution
  - Executes signed SOL transfer on Devnet
  - Validates SignerContext propagation
  
- **Test 1.3**: Multi-tool, multi-chain execution
  - Combines tools from different blockchains
  - Tests cross-chain workflow coordination

### Suite 2: Multi-Agent Coordination (riglr-agents)

Tests agent collaboration patterns:

- **Test 2.1**: Sequential workflow with local registry
  - Agent chaining with data passing
  - Local registry operations
  
- **Test 2.2**: Distributed workflow with Redis
  - Redis-backed agent registry
  - Distributed agent coordination
  
- **Test 2.3**: Parallel agent execution
  - Concurrent agent operations
  - Load balancing and task distribution

### Suite 3: Real-time Data Pipeline (riglr-streams)

Tests streaming data ingestion:

- **Test 3.1**: Solana Devnet to indexer pipeline
  - Geyser stream simulation
  - Event parsing and storage
  
- **Test 3.2**: Event parsing and enrichment
  - Transaction log parsing
  - Metadata enrichment
  
- **Test 3.3**: Stream error handling
  - Reconnection logic
  - Error recovery patterns
  
- **Test 3.4**: Concurrent stream processing
  - Multiple stream sources
  - Parallel event processing

### Suite 4: Web Service & Authentication (riglr-server)

Tests HTTP API layer:

- **Test 4.1**: Authenticated API requests
  - Bearer token authentication
  - Transaction execution via API
  
- **Test 4.2**: Rate limiting and throttling
  - Request rate limits
  - Backpressure handling
  
- **Test 4.3**: CORS and security headers
  - Cross-origin requests
  - Security header validation
  
- **Test 4.4**: WebSocket support
  - Real-time connections
  - Streaming updates

### Suite 5: Agent Memory & Knowledge Graph (riglr-graph-memory)

Tests persistent agent memory:

- **Test 5.1**: RAG agent workflow
  - Knowledge storage and retrieval
  - Graph-based memory
  
- **Test 5.2**: Knowledge graph traversal
  - Multi-hop relationships
  - Graph queries
  
- **Test 5.3**: Memory context switching
  - Agent-specific contexts
  - Context isolation
  
- **Test 5.4**: Memory persistence
  - Data durability
  - Cross-session memory

### Suite 6: Application Scaffolding (create-riglr-app)

Tests project generation:

- **Test 6.1**: Generate, build, and test
  - Project structure validation
  - Compilation checks
  
- **Test 6.2**: Template customization
  - Different template types
  - Feature selection
  
- **Test 6.3**: Configuration generation
  - Environment files
  - Testnet configuration
  
- **Test 6.4**: Example code generation
  - Working examples
  - Documentation

## Running the Tests

### Quick Start

```bash
# Run all E2E tests
./scripts/run_e2e_tests.sh

# Run specific suite
./scripts/run_e2e_tests.sh --suite 1

# Keep services running after tests
./scripts/run_e2e_tests.sh --keep-running

# Verbose output
./scripts/run_e2e_tests.sh --verbose
```

### Manual Setup

1. **Start Docker services**:
```bash
docker-compose -f docker-compose.test.yml up -d
```

2. **Set environment variables**:
```bash
cp .env.test .env.test.local
# Edit .env.test.local with your API keys and wallet keys
source .env.test.local
```

3. **Run specific test suite**:
```bash
# Suite 1: Core Agent Workflow
cargo test --package riglr-agents --test e2e_core_agent_workflow

# Suite 2: Multi-Agent Coordination
cargo test --package riglr-agents --test e2e_multi_agent_coordination

# Suite 3: Data Pipeline
cargo test --package riglr-streams --test e2e_data_pipeline

# Suite 4: Web Service
cargo test --package riglr-server --test e2e_web_service

# Suite 5: Agent Memory
cargo test --package riglr-graph-memory --test e2e_agent_memory

# Suite 6: Scaffolding
cargo test --package create-riglr-app --test e2e_scaffolding
```

4. **Clean up**:
```bash
docker-compose -f docker-compose.test.yml down -v
```

## Environment Configuration

### Required Environment Variables

```bash
# LLM API Keys
GEMINI_API_KEY=your-gemini-api-key

# Blockchain RPC Endpoints
SOLANA_RPC_URL=https://api.devnet.solana.com
RPC_URL_11155111=https://ethereum-sepolia-rpc.publicnode.com

# Test Wallets (NEVER use mainnet keys!)
SOLANA_PRIVATE_KEY=base58-encoded-private-key
EVM_PRIVATE_KEY=0x-prefixed-hex-private-key

# Database Configuration
DATABASE_URL=postgresql://riglr_test:test_password@localhost:5432/riglr_indexer_test
REDIS_URL=redis://localhost:6379
NEO4J_URL=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=test_password
```

### Funding Test Wallets

Before running transaction tests, ensure test wallets are funded:

**Solana Devnet**:
```bash
solana airdrop 2 YOUR_WALLET_ADDRESS --url devnet
```

**Ethereum Sepolia**:
- Use Sepolia faucets: https://sepoliafaucet.com/

## Mock API Configuration

Wiremock mappings are provided in `test/wiremock/mappings/`:
- `dexscreener-api.json`: DexScreener API mocks
- `jupiter-api.json`: Jupiter DEX mocks
- `pump-fun-api.json`: Pump.fun API mocks
- `general-api.json`: Generic API endpoints

## Troubleshooting

### Common Issues

1. **Docker services not starting**:
   - Check Docker daemon is running
   - Ensure ports are not already in use
   - Verify Docker Compose version compatibility

2. **Test wallet insufficient funds**:
   - Request testnet tokens from faucets
   - Check wallet address is correct
   - Verify network connectivity to testnets

3. **API rate limits**:
   - Use mock APIs via Wiremock
   - Implement retry logic with backoff
   - Use different API keys for testing

4. **Database connection errors**:
   - Ensure PostgreSQL/Neo4j containers are healthy
   - Check connection strings in .env.test.local
   - Verify network connectivity between containers

### Debug Mode

Enable detailed logging:
```bash
export RUST_LOG=debug
export RUST_BACKTRACE=1
./scripts/run_e2e_tests.sh --verbose
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: E2E Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  e2e-tests:
    runs-on: ubuntu-latest
    services:
      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_USER: riglr_test
          POSTGRES_PASSWORD: test_password
          POSTGRES_DB: riglr_indexer_test
        ports:
          - 5432:5432
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Setup test environment
        run: |
          cp .env.test .env.test.local
          echo "GEMINI_API_KEY=${{ secrets.GEMINI_API_KEY }}" >> .env.test.local
          
      - name: Run E2E tests
        run: ./scripts/run_e2e_tests.sh
        env:
          SOLANA_PRIVATE_KEY: ${{ secrets.TEST_SOLANA_PRIVATE_KEY }}
```

## Best Practices

1. **Isolation**: Each test should be independent and not rely on others
2. **Cleanup**: Always clean up resources after tests
3. **Timeouts**: Set appropriate timeouts for blockchain operations
4. **Retries**: Implement retry logic for network operations
5. **Mocking**: Use mocks for external APIs to ensure consistency
6. **Security**: Never commit real private keys or API credentials

## Contributing

When adding new E2E tests:

1. Choose the appropriate test suite based on functionality
2. Follow the existing test structure and naming conventions
3. Update this README with test descriptions
4. Ensure tests work with both real testnets and mocked services
5. Add necessary Wiremock mappings for external APIs
6. Update CI/CD configuration if new dependencies are added

## License

These tests are part of the riglr project and follow the same MIT license.