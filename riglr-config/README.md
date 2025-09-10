# riglr-config

Unified, hierarchical configuration management for RIGLR applications.

## Overview

`riglr-config` provides a fail-fast, environment-driven configuration system that ensures your riglr applications are configured correctly before they start. It consolidates all configuration concerns into a single, well-structured system with strong typing and comprehensive validation.

### Key Features

- **Fail-Fast Validation**: Configuration errors crash at startup, not runtime
- **Environment-Driven**: All configuration loaded from environment variables
- **Multi-Chain Support**: Dynamic chain configuration via `RPC_URL_{CHAIN_ID}` convention
- **Type-Safe**: Strongly typed configuration with compile-time guarantees
- **Hierarchical Structure**: Logical organization of related settings

📚 **[View Full Documentation](https://riglr.dev/concepts/configuration)** for detailed usage guides and examples.

## Quick Start

```rust
use riglr_config::Config;

// Load configuration from environment (fail-fast)
let config = Config::from_env();

// Access configuration values
println!("Redis URL: {}", config.database.redis_url);
println!("Environment: {:?}", config.app.environment);

// Get RPC URL for any chain
if let Some(rpc_url) = config.network.get_rpc_url("ethereum") {
    println!("Ethereum RPC: {}", rpc_url);
}
```


## Address Validation (v0.3.0+)

Address validation is decoupled from the core configuration to maintain chain-agnostic architecture:

```rust
use riglr_config::{Config, AddressValidator};
use riglr_evm_common::validation::EvmAddressValidator;

let config = Config::from_env();

// Optional: Validate blockchain addresses
config.network.validate_config(Some(&EvmAddressValidator))?;
```


## Configuration Structure

- **app**: Server settings, transaction defaults, retry configuration
- **database**: Redis (required), Neo4j, ClickHouse
- **network**: RPC URLs for all chains, contract addresses
- **providers**: API keys for AI, blockchain, and market data providers
- **features**: Feature flags for enabling/disabling functionality

## Required Environment Variables

```bash
REDIS_URL=redis://localhost:6379
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
RPC_URL_1=https://eth-mainnet.alchemyapi.io/v2/your-key  # Or any EVM chain
```

## Dynamic Chain Support

Add new chains without code changes:

```bash
# Using chain ID
export RPC_URL_137=https://polygon-rpc.com

# Using network name
export RPC_URL_POLYGON=https://polygon-rpc.com
```

📚 **[View Full Documentation](https://riglr.dev/concepts/configuration)** for complete environment variable reference, chain configuration, and migration guides.