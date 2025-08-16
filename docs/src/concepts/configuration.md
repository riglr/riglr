# Unified Configuration

The `riglr-config` crate provides a unified, fail-fast configuration system that ensures your applications are configured correctly before they start. It standardizes configuration across all riglr components while maintaining flexibility for custom needs.

## Core Principles

1. **Fail-Fast**: Configuration errors are caught at startup, not runtime
2. **Type-Safe**: Strongly typed configuration with compile-time guarantees
3. **Environment-Aware**: Seamless transition from development to production
4. **Standardized**: Consistent patterns across all riglr crates

## Configuration Sources

Configuration is loaded from multiple sources in priority order:

```
1. Command-line arguments (highest priority)
2. Environment variables
3. Configuration files (TOML/YAML/JSON)
4. Default values (lowest priority)
```

## Basic Usage

### Loading Configuration

```rust
use riglr_config::{Config, ConfigBuilder};

#[derive(Config)]
struct AppConfig {
    #[config(env = "DATABASE_URL")]
    database_url: String,
    
    #[config(env = "PORT", default = 8080)]
    port: u16,
    
    #[config(env = "LOG_LEVEL", default = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = AppConfig::load()?;
    
    // Use configuration
    println!("Starting on port {}", config.port);
    
    Ok(())
}
```

### Configuration Files

Create a `riglr.toml` file in your project root:

```toml
# riglr.toml
[app]
name = "my-trading-bot"
version = "1.0.0"

[chains]
# Ethereum Mainnet
[chains.1]
name = "ethereum"
rpc_url = "${RPC_URL_1}"
chain_id = 1

# Arbitrum
[chains.42161]
name = "arbitrum"
rpc_url = "${RPC_URL_42161}"
chain_id = 42161

[solana]
rpc_url = "${SOLANA_RPC_URL}"
ws_url = "${SOLANA_WS_URL}"
commitment = "confirmed"

[redis]
url = "${REDIS_URL}"
max_connections = 100

[api_keys]
anthropic = "${ANTHROPIC_API_KEY}"
openai = "${OPENAI_API_KEY}"
coingecko = "${COINGECKO_API_KEY}"
```

## Chain Configuration

### RPC URL Convention

riglr uses a standardized convention for EVM chain RPC URLs:

```bash
# Environment variables
RPC_URL_1=https://eth-mainnet.example.com     # Ethereum
RPC_URL_56=https://bsc.example.com            # BSC
RPC_URL_137=https://polygon.example.com       # Polygon
RPC_URL_42161=https://arbitrum.example.com    # Arbitrum
```

### chains.toml

Define all your chains in a centralized `chains.toml` file:

```toml
# chains.toml
[ethereum]
chain_id = 1
name = "Ethereum Mainnet"
rpc_url = "${RPC_URL_1}"
explorer = "https://etherscan.io"
native_token = "ETH"
decimals = 18

[bsc]
chain_id = 56
name = "BNB Smart Chain"
rpc_url = "${RPC_URL_56}"
explorer = "https://bscscan.com"
native_token = "BNB"
decimals = 18

[polygon]
chain_id = 137
name = "Polygon"
rpc_url = "${RPC_URL_137}"
explorer = "https://polygonscan.com"
native_token = "MATIC"
decimals = 18

[arbitrum]
chain_id = 42161
name = "Arbitrum One"
rpc_url = "${RPC_URL_42161}"
explorer = "https://arbiscan.io"
native_token = "ETH"
decimals = 18

[solana]
name = "Solana"
rpc_url = "${SOLANA_RPC_URL}"
ws_url = "${SOLANA_WS_URL}"
explorer = "https://solscan.io"
native_token = "SOL"
decimals = 9
```

### Accessing Chain Configuration

```rust
use riglr_config::{ChainConfig, load_chains};

// Load all chains
let chains = load_chains()?;

// Get specific chain
let ethereum = chains.get_chain(1)?;
println!("Ethereum RPC: {}", ethereum.rpc_url);

// Get by name
let polygon = chains.get_by_name("polygon")?;

// Iterate all chains
for chain in chains.all() {
    println!("{}: {}", chain.name, chain.rpc_url);
}
```

## Environment-Based Configuration

### Development vs Production

Use different configuration files for different environments:

```rust
use riglr_config::{Config, Environment};

let config = Config::builder()
    .environment(Environment::from_env()?)
    .build()?;

// Automatically loads:
// - riglr.toml (base config)
// - riglr.development.toml (in development)
// - riglr.production.toml (in production)
```

### Environment Variables

Override any configuration value with environment variables:

```toml
# riglr.toml
[database]
host = "localhost"  # Can be overridden by DATABASE_HOST
port = 5432        # Can be overridden by DATABASE_PORT
```

```bash
# Override in production
export DATABASE_HOST=prod-db.example.com
export DATABASE_PORT=5433
```

## Validation

### Fail-Fast Validation

Configuration is validated at startup:

```rust
#[derive(Config, Validate)]
struct ApiConfig {
    #[validate(url)]
    endpoint: String,
    
    #[validate(range(min = 1, max = 65535))]
    port: u16,
    
    #[validate(email)]
    admin_email: String,
    
    #[validate(length(min = 32))]
    api_key: String,
}

// Validation happens automatically
let config = ApiConfig::load()?; // Fails if validation fails
```

### Custom Validators

Implement custom validation logic:

```rust
use riglr_config::{Config, Validator};

#[derive(Config)]
struct TradingConfig {
    #[validate(custom = "validate_slippage")]
    max_slippage: f64,
    
    #[validate(custom = "validate_gas_price")]
    max_gas_price: u64,
}

fn validate_slippage(value: &f64) -> Result<(), String> {
    if *value < 0.0 || *value > 0.5 {
        return Err("Slippage must be between 0 and 50%".into());
    }
    Ok(())
}

fn validate_gas_price(value: &u64) -> Result<(), String> {
    if *value > 1000 * 10u64.pow(9) { // 1000 gwei
        return Err("Gas price too high".into());
    }
    Ok(())
}
```

## Secrets Management

### Environment Variables

Store secrets in environment variables:

```bash
# .env file (never commit this!)
PRIVATE_KEY=0x1234567890abcdef...
API_KEY=sk-1234567890abcdef...
DATABASE_PASSWORD=super-secret-password
```

### Secret Providers

Integrate with secret management services:

```rust
use riglr_config::{Config, SecretProvider};

// AWS Secrets Manager
let provider = AwsSecretsProvider::new("us-east-1");

// HashiCorp Vault
let provider = VaultProvider::new("https://vault.example.com");

// Load config with secrets
let config = Config::builder()
    .secret_provider(provider)
    .build()?;
```

### Secure Defaults

Never include secrets in default values:

```rust
#[derive(Config)]
struct SecureConfig {
    // ❌ Bad: Default private key
    #[config(default = "0x1234...")]
    private_key: String,
    
    // ✅ Good: Required without default
    #[config(env = "PRIVATE_KEY")]
    private_key: String,
    
    // ✅ Good: Safe default for non-secret
    #[config(default = "mainnet")]
    network: String,
}
```

## Advanced Configuration

### Nested Configuration

Support complex nested structures:

```rust
#[derive(Config)]
struct AppConfig {
    server: ServerConfig,
    database: DatabaseConfig,
    chains: HashMap<u32, ChainConfig>,
    features: FeatureFlags,
}

#[derive(Config)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
}

#[derive(Config)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
    timeout: Duration,
}
```

### Dynamic Reloading

Support configuration hot-reloading:

```rust
use riglr_config::{Config, Watcher};

let (config, mut watcher) = Config::load_watched()?;

// Listen for changes
tokio::spawn(async move {
    while let Some(event) = watcher.next().await {
        match event {
            ConfigEvent::Updated(new_config) => {
                println!("Configuration updated");
                // Apply new configuration
            }
            ConfigEvent::Error(e) => {
                eprintln!("Configuration error: {}", e);
            }
        }
    }
});
```

### Feature Flags

Manage feature toggles:

```toml
# riglr.toml
[features]
enable_trading = true
enable_analytics = false
max_position_size = 1000.0
allowed_tokens = ["SOL", "ETH", "USDC"]
```

```rust
#[derive(Config)]
struct FeatureFlags {
    enable_trading: bool,
    enable_analytics: bool,
    max_position_size: f64,
    allowed_tokens: Vec<String>,
}

// Use in code
if config.features.enable_trading {
    execute_trade().await?;
}
```

## Testing Configuration

### Test Configurations

Create test-specific configurations:

```rust
#[cfg(test)]
mod tests {
    use riglr_config::TestConfig;
    
    #[test]
    fn test_with_config() {
        let config = TestConfig::builder()
            .set("database.url", "sqlite::memory:")
            .set("redis.url", "redis://localhost:6379/1")
            .build();
        
        // Test with mock configuration
        run_app(config);
    }
}
```

### Configuration Fixtures

Load configuration from fixtures:

```rust
#[test]
fn test_production_config() {
    let config = Config::from_file("fixtures/production.toml")?;
    
    // Validate production configuration
    assert!(config.validate().is_ok());
    assert_eq!(config.chains.len(), 5);
}
```

## Deployment Patterns

### Docker

Configure via environment variables in Docker:

```dockerfile
# Dockerfile
FROM rust:1.75
WORKDIR /app
COPY . .
RUN cargo build --release

# Use environment variables for configuration
ENV RUST_LOG=info
ENV DATABASE_URL=postgresql://user:pass@db:5432/app
ENV REDIS_URL=redis://redis:6379

CMD ["./target/release/app"]
```

```yaml
# docker-compose.yml
version: '3.8'
services:
  app:
    build: .
    env_file: .env
    environment:
      - DATABASE_URL=postgresql://user:pass@db:5432/app
      - REDIS_URL=redis://redis:6379
```

### Kubernetes

Use ConfigMaps and Secrets:

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  riglr.toml: |
    [server]
    port = 8080
    
    [features]
    enable_trading = true

---
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
data:
  private-key: <base64-encoded-key>
  api-key: <base64-encoded-key>

---
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: trading-bot
spec:
  template:
    spec:
      containers:
      - name: app
        image: trading-bot:latest
        volumeMounts:
        - name: config
          mountPath: /config
        envFrom:
        - secretRef:
            name: app-secrets
      volumes:
      - name: config
        configMap:
          name: app-config
```

## Best Practices

1. **Never Commit Secrets**: Use `.env` files locally and secret managers in production
2. **Validate Early**: Use fail-fast validation to catch errors at startup
3. **Use Defaults Wisely**: Provide sensible defaults for non-critical settings
4. **Environment Parity**: Keep development and production configs as similar as possible
5. **Document Configuration**: Include example configuration files in your repository

## Migration Guide

### From Manual Configuration

Before (manual):
```rust
let rpc_url = std::env::var("RPC_URL").expect("RPC_URL not set");
let port = std::env::var("PORT")
    .unwrap_or_else(|_| "8080".to_string())
    .parse()
    .expect("Invalid port");
```

After (riglr-config):
```rust
#[derive(Config)]
struct AppConfig {
    #[config(env = "RPC_URL")]
    rpc_url: String,
    
    #[config(env = "PORT", default = 8080)]
    port: u16,
}

let config = AppConfig::load()?; // Automatic validation
```

## Troubleshooting

### Common Issues

**Missing Environment Variable**
```
Error: Configuration error: Missing required field 'database_url'
Hint: Set the DATABASE_URL environment variable
```

**Invalid Configuration Value**
```
Error: Validation failed for 'max_slippage': Value 1.5 exceeds maximum of 0.5
```

**File Not Found**
```
Error: Configuration file not found: riglr.toml
Hint: Create a riglr.toml file or set CONFIG_PATH
```

### Debug Mode

Enable debug logging for configuration:

```bash
RUST_LOG=riglr_config=debug cargo run
```

## Next Steps

- Explore [Agent Coordination](agents.md) for multi-agent configuration
- Learn about [Real-Time Streaming](streams.md) for stream configuration
- Understand the [Indexer](indexer.md) for indexer configuration