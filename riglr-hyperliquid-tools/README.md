# riglr-hyperliquid-tools

A comprehensive suite of rig-compatible tools for interacting with Hyperliquid perpetual futures.

## Features

- **Trading Tools**: Place, cancel, and manage perpetual futures orders
- **Position Management**: Monitor and close trading positions  
- **Account Management**: Query account information and set leverage
- **Risk Analysis**: Calculate portfolio risk metrics and exposure
- **Production Ready**: Built-in retry logic, error handling, and safety checks
- **Type Safe**: Full Rust type safety with comprehensive documentation

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
riglr-hyperliquid-tools = { path = "../riglr-hyperliquid-tools" }
rig-core = "0.17"
```

### Using with rig agents

```rust
use rig::agent::AgentBuilder;
use riglr_hyperliquid_tools::{
    place_hyperliquid_order,
    get_hyperliquid_positions,
    get_hyperliquid_account_info,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = AgentBuilder::new("gpt-4")
        .preamble("You are a derivatives trading assistant specialized in Hyperliquid.")
        .tool(place_hyperliquid_order)
        .tool(get_hyperliquid_positions)
        .tool(get_hyperliquid_account_info)
        .build();

    let response = agent.prompt("Check my current positions").await?;
    println!("Response: {}", response);
    
    Ok(())
}
```

### Direct tool usage

```rust
use riglr_core::{SignerContext, signer::LocalSolanaSigner};
use riglr_hyperliquid_tools::place_hyperliquid_order;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up signer context
    let keypair = solana_sdk::signature::Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(
        keypair, 
        "https://api.devnet.solana.com".to_string()
    ));
    
    // Execute trading operations
    let result = SignerContext::with_signer(signer, async {
        place_hyperliquid_order(
            "ETH-PERP".to_string(),
            "buy".to_string(),
            "0.1".to_string(),
            "limit".to_string(),
            Some("2000.0".to_string()),
            None,
            None,
        ).await
    }).await?;
    
    println!("Order result: {:?}", result);
    Ok(())
}
```

## Available Tools

### Trading Tools
- `place_hyperliquid_order` - Place market or limit orders
- `cancel_hyperliquid_order` - Cancel existing orders
- `set_leverage` - Set leverage for trading pairs

### Position Management
- `get_hyperliquid_positions` - Get current positions
- `close_hyperliquid_position` - Close positions
- `get_hyperliquid_position_details` - Get detailed position info

### Account Management
- `get_hyperliquid_account_info` - Get account balance and margin info
- `get_hyperliquid_portfolio_risk` - Calculate risk metrics

## Order Types

### Market Orders
```rust
place_hyperliquid_order(
    "BTC-PERP".to_string(),
    "buy".to_string(),
    "0.01".to_string(),
    "market".to_string(),
    None, // No price needed
    None,
    None,
).await
```

### Limit Orders
```rust
place_hyperliquid_order(
    "ETH-PERP".to_string(),
    "sell".to_string(),
    "0.5".to_string(),
    "limit".to_string(),
    Some("2100.0".to_string()),
    Some(false), // Not reduce-only
    Some("gtc".to_string()), // Good till cancel
).await
```

## Risk Management

The tools include built-in risk management features:

- **Leverage Limits**: Validation of leverage settings (1-100x)
- **Position Sizing**: Validation of order sizes and position limits
- **Risk Metrics**: Portfolio-wide risk analysis
- **Error Handling**: Comprehensive error types for different failure scenarios

## Examples

See the `examples/` directory for complete usage examples:

- `perpetual_trading.rs` - Comprehensive trading example
- Integration with rig agents
- Error handling patterns
- Risk management examples

## Development Status

⚠️ **Note**: This is a development/simulation implementation. The actual order placement and management functions are currently simulated. For production use, you would need to:

1. Implement actual Hyperliquid transaction signing
2. Add proper L1 blockchain interaction
3. Handle Hyperliquid-specific cryptographic operations
4. Add production error handling and retry logic

## Architecture

- **Stateless Tools**: All tools use the SignerContext pattern for thread-safe operation
- **Error Classification**: Retriable vs permanent errors for robust error handling
- **Type Safety**: Strong typing throughout with serde support
- **Async First**: Non-blocking operations using tokio

## License

This project is licensed under the MIT License.