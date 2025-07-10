# CI Integration Tests Setup

This document describes how to configure the GitHub Actions integration tests for the riglr project.

## Required Secrets

The integration tests require the following secrets to be configured in your GitHub repository settings:

### Blockchain Test Keys

These should be test-only private keys with minimal funds on testnets:

- `SOLANA_TEST_PRIVATE_KEY`: Base58-encoded Solana private key for devnet testing
- `EVM_TEST_PRIVATE_KEY`: Hex-encoded Ethereum private key for Sepolia testnet
- `HYPERLIQUID_TEST_PRIVATE_KEY`: Private key for Hyperliquid testnet (if applicable)

### API Keys

These are for external service integrations:

- `DEXSCREENER_API_KEY`: API key for DexScreener service
- `LUNARCRUSH_API_KEY`: API key for LunarCrush social analytics
- `CROSS_CHAIN_TEST_KEYS`: JSON object with bridge service API keys

## Setting Up Secrets

1. Go to your repository's Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Add each secret with the appropriate name and value
4. Ensure test keys only have minimal testnet funds

## Test Workflow

The integration tests run in several scenarios:

1. **Scheduled**: Daily at 2 AM UTC to catch external API changes
2. **Manual Trigger**: Via workflow_dispatch for debugging
3. **Push to main/develop**: For repository owners only
4. **Pull Requests**: From the main repository (not forks)

For pull requests from forks, mock integration tests run instead to ensure code quality without exposing secrets.

## Local Testing

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

## Security Considerations

- **Never use production keys**: All keys should be for testnets only
- **Minimal funds**: Test accounts should have only enough funds for testing
- **Rotate regularly**: Change test keys periodically for security
- **Monitor usage**: Check test account activity regularly
- **Fork safety**: Integration tests with real services don't run on fork PRs

## Troubleshooting

### Tests Failing Due to Rate Limits

If tests fail due to rate limiting:
1. Reduce test frequency
2. Implement retry logic with exponential backoff
3. Consider using mock services for most tests

### Network Issues

For network-related failures:
1. Check if the RPC endpoints are operational
2. Verify network connectivity in CI environment
3. Consider using fallback RPC endpoints

### Secret Not Available

If secrets aren't available:
1. Verify secret names match exactly
2. Check repository settings for secret visibility
3. Ensure workflow has permission to access secrets