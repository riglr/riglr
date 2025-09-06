# riglr-config

Unified, hierarchical configuration management for RIGLR applications.

## Overview

This crate consolidates all configuration concerns into a single, well-structured system that:
- Loads from environment variables with fail-fast validation
- Supports the `RPC_URL_{CHAIN_ID}` convention for dynamic chain support
- Provides hierarchical organization (app, database, network, providers, features)
- Includes comprehensive validation and error handling

## Architecture

riglr-config provides a unified configuration system for the entire riglr ecosystem, designed to be the single source of truth for all application settings.

### Design Principles

- **Chain-Agnostic Core**: Configuration loading and validation is independent of blockchain SDKs
- **Fail-Fast Validation**: Invalid configuration is caught at startup, not runtime
- **Hierarchical Structure**: Related settings are grouped into logical sections
- **Convention over Configuration**: Uses patterns like `RPC_URL_{CHAIN_ID}` for flexibility
- **Pluggable Validation**: Address validation is decoupled via trait-based validators
- **Arc-Wrapped Sharing**: Configs are Arc-wrapped for efficient multi-threaded access

### Configuration Hierarchy

```
Config (root)
├── app: ApplicationConfig
│   ├── Server settings (port, environment)
│   ├── Transaction settings (gas, slippage)
│   └── Retry configuration
├── database: DatabaseConfig
│   ├── Redis (required)
│   ├── Neo4j (optional)
│   └── ClickHouse (optional)
├── network: NetworkConfig
│   ├── Solana RPC URLs
│   ├── EVM RPC URLs (dynamic)
│   └── Chain contracts (from chains.toml)
├── providers: ProvidersConfig
│   ├── AI providers
│   ├── Blockchain data providers
│   └── Market data providers
└── features: FeaturesConfig
    └── Feature flags
```

### Validation Strategy

1. **Environment Loading**: Reads from env vars with optional .env file
2. **Structural Validation**: Ensures required fields are present
3. **Format Validation**: Validates URLs, numbers, enums
4. **Address Validation**: Optional blockchain-specific validation via traits
5. **Post-Load Validation**: Custom business rule validation

## Usage

### Basic Usage

```rust
use riglr_config::Config;

// Load configuration from environment (fail-fast)
let config = Config::from_env();

// NEW: Explicitly validate blockchain addresses if needed
// This step is now required for EVM address validation after architectural decoupling
#[cfg(feature = "evm")]
{
    use riglr_evm_common::validation::EvmAddressValidator;
    config.network.validate_config(Some(&EvmAddressValidator))
        .expect("Invalid EVM addresses in configuration");
}
#[cfg(not(feature = "evm"))]
{
    // Skip address validation when EVM support is not available
    config.network.validate_config(None)
        .expect("Configuration validation failed");
}

// Access configuration values
println!("Redis URL: {}", config.database.redis_url);
println!("Environment: {:?}", config.app.environment);

// Get RPC URL for a specific chain using chain ID
if let Some(rpc_url) = config.network.get_rpc_url("1") {
    println!("Ethereum RPC: {}", rpc_url);
}

// Or using network name (new feature)
if let Some(rpc_url) = config.network.get_rpc_url("ethereum") {
    println!("Ethereum RPC: {}", rpc_url);
}

// For backward compatibility with numeric chain IDs
if let Some(rpc_url) = config.network.get_rpc_url_by_id(137) {
    println!("Polygon RPC: {}", rpc_url);
}
```

### Library Usage

For library usage where you want to handle configuration errors gracefully:

```rust
use riglr_config::Config;
use std::sync::Arc;

// Use try_from_env() for Result-based error handling
match Config::try_from_env() {
    Ok(config) => {
        // Config loaded successfully as Arc<Config>
        println!("Redis URL: {}", config.database.redis_url);
    }
    Err(e) => {
        // Handle configuration error gracefully
        eprintln!("Failed to load config: {}", e);
        // Use defaults or alternative configuration
    }
}
```

The `try_from_env()` method:
- Returns `Result<Arc<Config>, ConfigError>` for library-friendly error handling
- Automatically loads `.env` file if present
- Returns an Arc-wrapped Config for efficient sharing
- Validates all required fields

### Migrating from Existing Configuration

If you're currently using configuration in `riglr-showcase`, `create-riglr-app`, or other crates:

1. **Add dependency**:
```toml
[dependencies]
riglr-config = { workspace = true }
```

2. **Replace custom config with unified config**:
```rust
// Before (riglr-showcase/src/config.rs)
use crate::config::Config;
let config = Config::from_env()?;

// After
use riglr_config::Config;
let config = Config::from_env(); // Fail-fast, no Result
```

3. **Update field access**:
```rust
// Before
config.redis_url

// After
config.database.redis_url

// Before
config.enable_trading

// After
config.features.enable_trading
```

## Address Validation

**Important**: Starting in v0.3.0, address validation has been decoupled from the core configuration system to maintain architectural purity. This is a **breaking change**.

### Design Rationale

The `AddressValidator` trait represents a deliberate architectural choice to maintain the chain-agnostic purity of `riglr-config`. This design pattern provides several critical benefits:

1. **Dependency Inversion**: The config crate defines the validation interface but doesn't depend on any blockchain-specific implementations. This prevents circular dependencies and keeps the dependency graph clean.

2. **Chain Agnosticism**: `riglr-config` remains completely independent of any blockchain SDKs (Solana, Ethereum, etc.), enforcing the project's unidirectional dependency flow where core crates never depend on chain-specific ones.

3. **Single Source of Truth**: The final application binary becomes the single source of truth for validation logic, allowing it to compose validators based on its specific needs without forcing unnecessary dependencies on other applications.

4. **Extensibility**: New blockchain networks can be supported by simply implementing the `AddressValidator` trait, without modifying the configuration crate itself.

5. **Optional Validation**: Applications that don't need address validation (e.g., off-chain tools) aren't forced to include blockchain dependencies, keeping their dependency tree minimal.

### New Pattern

The configuration system now uses a trait-based approach for blockchain address validation:

```rust
use riglr_config::{Config, AddressValidator};
use riglr_evm_common::validation::EvmAddressValidator;

let config = Config::from_env();

// Explicit validation is now required for address validation
config.network.validate_config(Some(&EvmAddressValidator))?;
```

### Custom Validators

You can implement custom address validators:

```rust
use riglr_config::{AddressValidator, ConfigResult, ConfigError};

struct MyCustomValidator;

impl AddressValidator for MyCustomValidator {
    fn validate(&self, address: &str, contract_name: &str) -> ConfigResult<()> {
        // Your validation logic here
        if address.len() < 10 {
            return Err(ConfigError::validation(format!(
                "Invalid {} address: too short", contract_name
            )));
        }
        Ok(())
    }
}

// Use your custom validator
config.network.validate_config(Some(&MyCustomValidator))?;
```

### Migration Guide

**Before v0.3.0:**
```rust
let config = Config::from_env(); // Address validation was automatic
```

**After v0.3.0:**
```rust
let config = Config::from_env();

// Address validation is now explicit and optional
use riglr_evm_common::validation::EvmAddressValidator;
config.network.validate_config(Some(&EvmAddressValidator))?;

// Or skip validation entirely
config.network.validate_config(None)?;
```

## Configuration Structure

The configuration is organized into logical sections:

### App Configuration
- Server settings (port, environment, log level)
- Transaction settings (gas prices, slippage)
- Retry configuration

### Database Configuration
- Redis URL and settings
- Optional Neo4j for graph memory
- Optional ClickHouse for analytics
- Connection pool settings

### Network Configuration
- Solana RPC URLs
- EVM RPC URLs (via `RPC_URL_{CHAIN_ID}` or `RPC_URL_{NETWORK_NAME}` convention)
- Chain-specific contracts (loaded from chains.toml)
- Network timeouts

### Providers Configuration
- AI providers (Anthropic, OpenAI, etc.)
- Blockchain data providers (Alchemy, Infura, etc.)
- Market data providers (DexScreener, CoinGecko, etc.)
- Social data providers (Twitter, etc.)

### Features Configuration
- Feature flags for enabling/disabling functionality
- Custom feature flags support

## Environment Variables

### Required Variables
```bash
# Database
REDIS_URL=redis://localhost:6379

# Network
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com

# At least one EVM RPC
RPC_URL_1=https://eth-mainnet.alchemyapi.io/v2/your-key
```

### Optional Variables
```bash
# App settings
PORT=8080
ENVIRONMENT=production
LOG_LEVEL=info
USE_TESTNET=false

# Transaction settings
MAX_GAS_PRICE_GWEI=100
PRIORITY_FEE_GWEI=2
SLIPPAGE_TOLERANCE_PERCENT=0.5

# Retry settings
MAX_RETRY_ATTEMPTS=3
RETRY_DELAY_MS=1000
RETRY_BACKOFF_MULTIPLIER=2.0

# Additional databases
NEO4J_URL=neo4j://localhost:7687
CLICKHOUSE_URL=http://localhost:8123

# Provider API keys
ANTHROPIC_API_KEY=sk-...
OPENAI_API_KEY=sk-...
ALCHEMY_API_KEY=...
LIFI_API_KEY=...

# Feature flags
ENABLE_TRADING=true
ENABLE_BRIDGING=true
ENABLE_SOCIAL_MONITORING=false
ENABLE_GRAPH_MEMORY=false
```

## Chain Configuration

Place a `chains.toml` file in your project root (or set `RIGLR_CHAINS_CONFIG` to its path):

```toml
[chains.ethereum]
id = 1
name = "Ethereum Mainnet"
router = "0xE592427A0AEce92De3Edee1F18E0157C05861564"
quoter = "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6"
factory = "0x1F98431c8aD98523631AE4a59f267346ea31F984"

[chains.polygon]
id = 137
name = "Polygon"
router = "0xE592427A0AEce92De3Edee1F18E0157C05861564"
quoter = "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6"
factory = "0x1F98431c8aD98523631AE4a59f267346ea31F984"
```

## Dynamic Chain Support

Add support for new chains without code changes using either chain IDs or network names:

```bash
# Add Optimism support using chain ID
export RPC_URL_10=https://optimism-mainnet.alchemyapi.io/v2/your-key

# OR using network name (more readable)
export RPC_URL_OPTIMISM=https://optimism-mainnet.alchemyapi.io/v2/your-key

# Other examples with network names
export RPC_URL_ETHEREUM=https://eth-mainnet.alchemyapi.io/v2/your-key
export RPC_URL_POLYGON=https://polygon-mainnet.alchemyapi.io/v2/your-key
export RPC_URL_ARBITRUM=https://arb1.arbitrum.io/rpc

# Override contract addresses (still uses chain ID)
export ROUTER_10=0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45
export QUOTER_10=0x61fFE014bA17989E743c5F6cB21bF9697530B21e
```

### Supported Network Aliases

Common network names are automatically resolved to their chain IDs:
- **Ethereum**: ethereum, mainnet, eth → 1
- **Polygon**: polygon, matic → 137
- **Arbitrum**: arbitrum, arb → 42161
- **Optimism**: optimism, op → 10
- **Base**: base → 8453
- **BSC**: bsc, binance, bnb → 56
- **Avalanche**: avalanche, avax → 43114

See `network.rs` for the complete list of supported aliases.

## Migration Guide

### From riglr-showcase

```rust
// Old
use crate::config::{Config, ConfigError};
let config = Config::from_env()?;
config.validate()?;

// New
use riglr_config::Config;
let config = Config::from_env(); // Validation is automatic
```

### From create-riglr-app

```rust
// Old
use crate::config::Config;
let config = Config::from_env(); // Panics on error

// New
use riglr_config::Config;
let config = Config::from_env(); // Same behavior, better structure
```

### From riglr-core

```rust
// Old
use riglr_core::config::RpcConfig;
let rpc_config = RpcConfig::default().with_env_overrides();

// New
use riglr_config::Config;
let config = Config::from_env();
// RPC URLs are in config.network.rpc_urls
```

## Benefits

1. **Single source of truth**: No more configuration sprawl
2. **Hierarchical organization**: Logical grouping of related settings
3. **Fail-fast validation**: Catch errors at startup, not runtime
4. **Convention-based**: RPC_URL_{CHAIN_ID} pattern just works
5. **Type-safe**: Strongly typed with serde
6. **Extensible**: Easy to add new configuration sections
7. **Well-documented**: Clear structure and validation rules