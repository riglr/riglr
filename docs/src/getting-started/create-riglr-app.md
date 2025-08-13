# Create Your First Agent

This guide walks you through creating a custom riglr agent from scratch, explaining each component and how they work together.

## Project Structure

After running `create-riglr-app`, your project will have the following structure:

```
my-trading-bot/
├── Cargo.toml
├── .env.example
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── tools/
│   │   └── mod.rs
│   └── agents/
│       └── mod.rs
└── chains.toml
```

## Understanding the Components

### 1. Configuration (`config.rs`)

The configuration module handles all environment variables and validates them at startup:

```rust
use riglr_core::Config;

let config = Config::from_env();
config.validate()?;
```

### 2. Tools

Tools are the building blocks of your agent. They define what actions your agent can perform:

```rust
use riglr_macros::tool;
use riglr_core::ToolError;

#[tool]
/// Gets the current price of a token
async fn get_token_price(
    /// The token address
    token: String,
) -> Result<f64, ToolError> {
    // Implementation here
}
```

### 3. Agents

Agents combine tools with an AI model to create intelligent behavior:

```rust
use rig::agent::Agent;
use rig::tool::ToolSet;

let agent = Agent::builder()
    .tools(vec![get_token_price, swap_tokens])
    .model(model)
    .build();
```

## Building Your First Custom Tool

Let's create a tool that monitors token prices:

```rust
use riglr_macros::tool;
use riglr_core::ToolError;
use riglr_web_tools::dexscreener::get_token_info;

#[tool]
/// Monitors a token and alerts when price changes significantly
async fn monitor_token_price(
    /// Token address to monitor
    token_address: String,
    /// Percentage change threshold (e.g., 5.0 for 5%)
    threshold: f64,
) -> Result<String, ToolError> {
    let info = get_token_info(&token_address).await
        .map_err(|e| ToolError::retriable(e.to_string()))?;
    
    let price_change = info.price_change_24h;
    
    if price_change.abs() > threshold {
        Ok(format!("Alert: {} changed by {:.2}%", 
            info.symbol, price_change))
    } else {
        Ok(format!("{} is stable at ${}", 
            info.symbol, info.price_usd))
    }
}
```

## Integrating with SignerContext

For blockchain operations, use the SignerContext pattern:

```rust
use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;

let signer = Arc::new(LocalSolanaSigner::new(
    keypair,
    config.solana_rpc_url.clone()
));

SignerContext::with_signer(signer, async {
    // Your agent operations here
    let result = agent.prompt("Swap 1 SOL for USDC").await?;
    Ok(result)
}).await?;
```

## Running Your Agent

### Interactive Mode

```bash
cargo run -- --interactive
```

### API Server Mode

```bash
cargo run -- --server --port 8080
```

### Job Worker Mode

```bash
cargo run -- --worker
```

## Next Steps

- Learn about [The SignerContext Pattern](../concepts/signer-context.md)
- Explore [Error Handling Philosophy](../concepts/error-handling.md)
- Check out our [Tutorials](../tutorials/index.md) for more complex examples