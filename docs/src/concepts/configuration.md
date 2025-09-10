# Configuration

The `riglr-config` crate provides a unified, environment-driven configuration system that ensures your applications are configured correctly before they start. It follows a hierarchical structure with fail-fast validation.

## Core Principles

1. **Environment-Driven**: Configuration loaded from environment variables
2. **Fail-Fast**: Configuration errors crash at startup, not runtime
3. **Type-Safe**: Strongly typed configuration with compile-time guarantees
4. **Hierarchical**: Organized into logical sections for clarity
5. **Convention-Based**: Uses patterns like `RPC_URL_{CHAIN_ID}` for flexibility
6. **Chain-Agnostic**: Core configuration is independent of blockchain SDKs

## The Config Structure

riglr uses a hierarchical configuration structure:

```rust
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

## Loading Configuration

Configuration is loaded from environment variables using `Config::from_env()`:

```rust
use riglr_config::Config;

// Load configuration from environment (fail-fast)
let config = Config::from_env();

// Access configuration values
println!("Redis URL: {}", config.database.redis_url);
println!("Environment: {:?}", config.app.environment);

// Get RPC URL for a specific chain using chain ID
if let Some(rpc_url) = config.network.get_rpc_url("1") {
    println!("Ethereum RPC: {}", rpc_url);
}

// Or using network name
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

## Address Validation

Starting in v0.3.0, address validation has been decoupled from the core configuration system to maintain architectural purity. This is a **breaking change**.

### Why Decoupled?

1. **Dependency Inversion**: Config defines the interface, not implementations
2. **Chain Agnosticism**: No blockchain SDK dependencies in core config
3. **Extensibility**: New chains supported by implementing `AddressValidator`
4. **Optional Validation**: Off-chain tools don't need blockchain dependencies

### Usage

```rust
use riglr_config::{Config, AddressValidator};

let config = Config::from_env();

// Explicit validation is now required for address validation
#[cfg(feature = "evm")]
{
    use riglr_evm_common::validation::EvmAddressValidator;
    config.network.validate_config(Some(&EvmAddressValidator))
        .expect("Invalid EVM addresses in configuration");
}

// Or skip validation entirely for off-chain tools
#[cfg(not(feature = "evm"))]
{
    config.network.validate_config(None)
        .expect("Configuration validation failed");
}
```

## Environment Variable Convention

### Required Variables

```bash
# Database (Redis is required)
REDIS_URL=redis://localhost:6379

# Network
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com

# At least one EVM RPC
RPC_URL_1=https://eth-mainnet.alchemyapi.io/v2/your-key
```

### Application Configuration

```bash
# Server settings
PORT=8080
ENVIRONMENT=production  # or development, staging
LOG_LEVEL=info

# Transaction settings
MAX_GAS_PRICE_GWEI=100
PRIORITY_FEE_GWEI=2
SLIPPAGE_TOLERANCE_PERCENT=0.5

# Retry settings
MAX_RETRY_ATTEMPTS=3
RETRY_DELAY_MS=1000
RETRY_BACKOFF_MULTIPLIER=2.0
```

### Database Configuration

```bash
# Required
REDIS_URL=redis://localhost:6379

# Optional databases
NEO4J_URL=neo4j://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=password

CLICKHOUSE_URL=http://localhost:8123
CLICKHOUSE_USER=default
CLICKHOUSE_PASSWORD=password
```

### Network Configuration

```bash
# Solana
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_WS_URL=wss://api.mainnet-beta.solana.com

# EVM Chains - Two formats supported:

# 1. Using Chain IDs
RPC_URL_1=https://eth-mainnet.alchemyapi.io/v2/your-key      # Ethereum
RPC_URL_56=https://bsc-dataseed.binance.org                  # BSC
RPC_URL_137=https://polygon-rpc.com                          # Polygon
RPC_URL_42161=https://arb1.arbitrum.io/rpc                   # Arbitrum

# 2. Using Network Names (more readable)
RPC_URL_ETHEREUM=https://eth-mainnet.alchemyapi.io/v2/your-key
RPC_URL_POLYGON=https://polygon-mainnet.alchemyapi.io/v2/your-key
RPC_URL_ARBITRUM=https://arb1.arbitrum.io/rpc
RPC_URL_OPTIMISM=https://mainnet.optimism.io
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

### Provider Configuration

```bash
# AI Providers
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...

# Blockchain Data Providers
ALCHEMY_API_KEY=...
INFURA_API_KEY=...
LIFI_API_KEY=...

# Market Data Providers
COINGECKO_API_KEY=...
DEXSCREENER_API_KEY=...

# Social/Web APIs
TWITTER_BEARER_TOKEN=...
EXA_API_KEY=...
```

### Feature Flags

```bash
# Feature toggles
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

Add support for new chains without code changes:

```bash
# Add Optimism support using chain ID
export RPC_URL_10=https://optimism-mainnet.alchemyapi.io/v2/your-key

# OR using network name (more readable)
export RPC_URL_OPTIMISM=https://optimism-mainnet.alchemyapi.io/v2/your-key

# Override contract addresses
export ROUTER_10=0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45
export QUOTER_10=0x61fFE014bA17989E743c5F6cB21bF9697530B21e
```

## Migrating from Other Configurations

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

### Field Access Changes

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

## Security Best Practices

### Never Commit Secrets

Use `.env` files for local development (add to `.gitignore`):

```bash
# .env (local development only - never commit!)
PRIVATE_KEY=0x1234567890abcdef...
ANTHROPIC_API_KEY=sk-ant-...
DATABASE_PASSWORD=super-secret
```

Load with dotenv in development:

```rust
#[cfg(debug_assertions)]
fn load_dev_env() {
    dotenv::dotenv().ok();
}

fn main() {
    #[cfg(debug_assertions)]
    load_dev_env();
    
    let config = Config::from_env(); // Panics on error
}
```

### Production Deployment

In production, use proper secret management:

```yaml
# Kubernetes Secret
apiVersion: v1
kind: Secret
metadata:
  name: riglr-secrets
type: Opaque
data:
  PRIVATE_KEY: <base64-encoded>
  ANTHROPIC_API_KEY: <base64-encoded>
```

```bash
# Docker Compose with env file
docker run --env-file .env.production riglr-bot

# Systemd with environment file
[Service]
EnvironmentFile=/etc/riglr/env
ExecStart=/usr/bin/riglr-bot
```

## Testing with Configuration

### Mock Configuration for Tests

```rust
#[cfg(test)]
mod tests {
    use riglr_config::Config;
    use std::env;
    
    fn setup_test_env() {
        env::set_var("REDIS_URL", "redis://localhost:6379");
        env::set_var("SOLANA_RPC_URL", "https://api.testnet.solana.com");
    }
    
    #[test]
    fn test_config_loading() {
        setup_test_env();
        
        let config = Config::from_env();
        assert!(config.database.redis_url.contains("redis"));
    }
}
```

## Common Patterns

### Centralized Configuration

Create a configuration module in your application:

```rust
// src/config.rs
use riglr_config::Config;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| {
    Config::from_env() // Returns Arc<Config>
});

// Usage anywhere in your app
use crate::config::CONFIG;

fn some_function() {
    if CONFIG.features.enable_trading {
        // Trading logic
    }
}
```

### Configuration with ApplicationContext

Integrate with ApplicationContext pattern:

```rust
use riglr_config::Config;
use riglr_core::provider::ApplicationContext;

async fn setup_application() -> Result<ApplicationContext, Box<dyn std::error::Error>> {
    // Load configuration (returns Arc<Config>)
    let config = Config::from_env();
    
    // Build ApplicationContext from config
    let context = ApplicationContext::builder()
        .with_redis(&config.database.redis_url)?
        .with_solana_rpc(&config.network.solana_rpc_url)?
        .build()?;
    
    Ok(context)
}
```

## Troubleshooting

### Common Issues

**Missing Required Environment Variable**
```
thread 'main' panicked at 'Failed to load config: Environment variable REDIS_URL not found'
Solution: export REDIS_URL="redis://localhost:6379"
```

**No RPC URL for Chain**
```
No RPC URL configured for chain ethereum
Solution: export RPC_URL_ETHEREUM="https://eth-mainnet.g.alchemy.com/v2/your-key"
```

### Debug Configuration Loading

Enable debug logging:

```bash
RUST_LOG=riglr_config=debug cargo run
```

Print loaded configuration (development only):

```rust
#[cfg(debug_assertions)]
fn debug_config(config: &Config) {
    println!("Loaded configuration:");
    println!("  Redis: {}", config.database.redis_url);
    println!("  Environment: {:?}", config.app.environment);
    // ... print other values
}
```

## Best Practices

1. **Load Once**: Load configuration at application startup and pass it down
2. **Fail Fast**: Let the application crash early if configuration is invalid
3. **Use Arc**: Config is Arc-wrapped for efficient multi-threaded sharing
4. **Validate Addresses**: Explicitly validate blockchain addresses when needed
5. **Document Variables**: Maintain a `.env.example` file with all variables
6. **Separate Secrets**: Keep secrets in secure storage, not in code

## Summary

The riglr configuration system provides:
- Simple environment-based configuration with `Config::from_env()`
- Arc-wrapped configs for efficient sharing across threads
- Standardized RPC URL convention (`RPC_URL_{CHAIN_ID}` or `RPC_URL_{NAME}`)
- Hierarchical structure for organization
- Type safety and fail-fast validation
- Decoupled address validation for architectural purity
- Production-ready patterns for deployment

This approach ensures configuration is explicit, validated, and secure across all environments.