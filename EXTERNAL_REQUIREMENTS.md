# External Requirements for Test Fixes

This file tracks external dependencies and requirements needed for tests to pass.

## Test Compilation Issues

Some tests may fail to compile or run due to slow build times. These are not actual test failures but build system issues:
- riglr-agents: Long compilation time due to many dependencies
- riglr-events-core: Heavy async test setup
- riglr-evm-tools: Network connectivity tests require actual RPC endpoints

## Environment Variables Required

### API Keys (for integration tests)
- `ANTHROPIC_API_KEY` - Required for AI agent tests
- `OPENAI_API_KEY` - Optional for OpenAI integration tests
- `ALCHEMY_API_KEY` - Required for Ethereum RPC tests
- `INFURA_API_KEY` - Alternative Ethereum RPC provider
- `TWITTER_BEARER_TOKEN` - Required for social monitoring tests

### RPC Endpoints
- `SOLANA_RPC_URL` - Solana RPC endpoint (default: https://api.mainnet-beta.solana.com)
- `ETH_RPC_URL` - Ethereum RPC endpoint
- `BSC_RPC_URL` - Binance Smart Chain RPC endpoint
- `POLYGON_RPC_URL` - Polygon RPC endpoint

### Database URLs
- `REDIS_URL` - Redis connection string (default: redis://localhost:6379)
- `DATABASE_URL` - PostgreSQL connection string
- `NEO4J_URL` - Neo4j connection for graph memory

## External Services Required

### Local Services
- Redis server running on localhost:6379 (for cache tests)
- PostgreSQL database (for persistence tests)
- Neo4j database (for graph memory tests)

### Network Requirements
- Internet connection for:
  - Blockchain RPC calls
  - API provider calls
  - Package downloads

## Test Data Files Required

### Solana Test Wallets
- Test keypairs in `~/.config/solana/` (for Solana transaction tests)

### Contract Addresses
- Valid contract addresses on testnets for interaction tests

## System Requirements

### Build Tools
- Rust 1.75+ with cargo
- C compiler for native dependencies
- pkg-config for system libraries

### Operating System
- Linux/macOS/WSL2 (Windows native may have path issues)
- Sufficient disk space for build artifacts (~5GB)
- Sufficient RAM for parallel test execution (8GB+)

## Known Issues

### Timeout Issues
- Some blockchain RPC tests may timeout on slow connections
- Increase timeout values in test configuration if needed

### Rate Limiting
- Free tier API keys may hit rate limits during full test suite
- Consider using paid tiers or running tests in smaller batches

## Notes

- Tests marked as `#[ignore]` may require specific setup or paid services
- Integration tests require actual blockchain connections
- Some tests may fail in CI environments without proper secrets setup