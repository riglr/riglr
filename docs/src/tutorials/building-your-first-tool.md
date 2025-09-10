## Step-by-Step: Building Your First Custom Tool

Let's create a complete custom tool from scratch - a portfolio analyzer:

### Step 1: Create the Tool File

Create `src/tools/portfolio_analyzer.rs`:

```rust
use riglr_macros::tool;
use riglr_core::{ToolError, ApplicationContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioAnalysis {
    pub total_value_usd: f64,
    pub assets: Vec<AssetInfo>,
    pub top_performer: String,
    pub worst_performer: String,
    pub risk_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetInfo {
    pub symbol: String,
    pub amount: f64,
    pub value_usd: f64,
    pub allocation_percent: f64,
    pub change_24h: f64,
}

#[tool]
/// Analyzes a wallet's portfolio composition and performance
async fn analyze_portfolio(
    /// The wallet address to analyze
    wallet_address: String,
    /// Include detailed breakdown (default: false)
    detailed: Option<bool>,
    context: &ApplicationContext,
) -> Result<PortfolioAnalysis, ToolError> {
    // Step 1: Validate the wallet address
    if !is_valid_address(&wallet_address) {
        return Err(ToolError::invalid_input_string(
            format!("Invalid wallet address: {}", wallet_address)
        ));
    }
    
    // Step 2: Get the blockchain client from context
    let client = context.get_extension::<Arc<SolanaRpcClient>>()
        .map_err(|_| ToolError::permanent_string(
            "Solana RPC client not configured"
        ))?;
    
    // Step 3: Fetch wallet balances with retry on network errors
    let balances = fetch_wallet_balances(&client, &wallet_address)
        .await
        .map_err(|e| {
            // Classify the error for intelligent retry
            if e.to_string().contains("timeout") {
                ToolError::retriable_string(format!("Network timeout: {}", e))
            } else if e.to_string().contains("429") {
                ToolError::rate_limited_string("RPC rate limit exceeded")
            } else {
                ToolError::permanent_string(format!("Failed to fetch balances: {}", e))
            }
        })?;
    
    // Step 4: Get current prices for all assets
    let prices = fetch_asset_prices(&balances)
        .await
        .map_err(|e| ToolError::retriable_string(e.to_string()))?;
    
    // Step 5: Calculate portfolio metrics
    let mut assets = Vec::new();
    let mut total_value = 0.0;
    
    for (token, amount) in balances {
        let price = prices.get(&token).copied().unwrap_or(0.0);
        let value = amount * price;
        total_value += value;
        
        assets.push(AssetInfo {
            symbol: token.clone(),
            amount,
            value_usd: value,
            allocation_percent: 0.0, // Will calculate after total
            change_24h: get_price_change(&token).await.unwrap_or(0.0),
        });
    }
    
    // Calculate allocation percentages
    for asset in &mut assets {
        asset.allocation_percent = (asset.value_usd / total_value) * 100.0;
    }
    
    // Find best and worst performers
    let top_performer = assets
        .iter()
        .max_by(|a, b| a.change_24h.partial_cmp(&b.change_24h).unwrap())
        .map(|a| a.symbol.clone())
        .unwrap_or_default();
    
    let worst_performer = assets
        .iter()
        .min_by(|a, b| a.change_24h.partial_cmp(&b.change_24h).unwrap())
        .map(|a| a.symbol.clone())
        .unwrap_or_default();
    
    // Calculate risk score based on concentration
    let risk_score = calculate_risk_score(&assets);
    
    // Return detailed or summary based on parameter
    let final_assets = if detailed.unwrap_or(false) {
        assets
    } else {
        assets.into_iter().take(5).collect() // Top 5 holdings only
    };
    
    Ok(PortfolioAnalysis {
        total_value_usd: total_value,
        assets: final_assets,
        top_performer,
        worst_performer,
        risk_score,
    })
}

// Helper functions
fn is_valid_address(address: &str) -> bool {
    // Implement address validation
    address.len() >= 32 && address.len() <= 44
}

fn calculate_risk_score(assets: &[AssetInfo]) -> f64 {
    // Higher concentration = higher risk
    let max_allocation = assets
        .iter()
        .map(|a| a.allocation_percent)
        .fold(0.0, f64::max);
    
    // Risk score from 0-10
    (max_allocation / 10.0).min(10.0)
}
```

### Step 2: Register the Tool

Add to `src/tools/mod.rs`:

```rust
pub mod portfolio_analyzer;
pub use portfolio_analyzer::*;
```

### Step 3: Add the Tool to Your Agent

Update `src/agents/mod.rs`:

```rust
let agent = AgentBuilder::new(model)
    .tool(tools::analyze_portfolio_tool())  // Your new tool!
    .tool(tools::get_token_price_tool())
    // ... other tools
    .build();
```

### Step 4: Test Your Tool

Create `tests/test_portfolio.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use riglr_core::provider::ApplicationContext;
    
    #[tokio::test]
    async fn test_portfolio_analysis() {
        // Setup test context
        let context = ApplicationContext::from_env();
        
        // Mock RPC client for testing
        let mock_client = Arc::new(MockSolanaClient::new());
        context.set_extension(mock_client);
        
        // Test the tool
        let result = analyze_portfolio(
            "test_wallet_address".to_string(),
            Some(true),
            &context,
        ).await;
        
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.total_value_usd > 0.0);
    }
}
```

## Integrating with SignerContext for Write Operations

When your tools need to sign transactions, use the SignerContext pattern:

### Creating a Trading Tool with Signing

```rust
#[tool]
/// Executes a token swap on the blockchain
async fn execute_swap(
    /// Token to sell (symbol or address)
    from_token: String,
    /// Token to buy (symbol or address)
    to_token: String,
    /// Amount to swap
    amount: f64,
    /// Maximum slippage percentage (default: 0.5%)
    slippage: Option<f64>,
    context: &ApplicationContext,
) -> Result<SwapResult, ToolError> {
    // Get the RPC client from ApplicationContext
    let client = context.get_extension::<Arc<SolanaRpcClient>>()
        .map_err(|_| ToolError::permanent_string("RPC client not configured"))?;
    
    // Get the signer from SignerContext (thread-local)
    let signer = SignerContext::current_as_solana()
        .await
        .map_err(|e| ToolError::permanent_string(
            format!("No signer available: {}", e)
        ))?;
    
    // Build the swap transaction
    let swap_params = SwapParams {
        from: from_token,
        to: to_token,
        amount,
        slippage: slippage.unwrap_or(0.5),
        wallet: signer.address(),
    };
    
    let transaction = build_swap_transaction(&client, swap_params)
        .await
        .map_err(|e| ToolError::retriable_string(e.to_string()))?;
    
    // Sign and send the transaction
    let signature = signer
        .sign_and_send_transaction(transaction)
        .await
        .map_err(|e| {
            // Classify blockchain errors
            if e.to_string().contains("insufficient funds") {
                ToolError::permanent_string("Insufficient funds for swap")
            } else if e.to_string().contains("blockhash") {
                ToolError::retriable_string("Blockhash expired, retrying...")
            } else {
                ToolError::permanent_string(format!("Transaction failed: {}", e))
            }
        })?;
    
    Ok(SwapResult {
        signature,
        from: from_token,
        to: to_token,
        amount_in: amount,
        amount_out: calculate_output_amount(&swap_params).await?,
        executed_at: chrono::Utc::now(),
    })
}
```

### Using Your Trading Tool in main.rs

```rust
// main.rs - Setting up signer context for user operations
async fn run_interactive(app_context: ApplicationContext, config: Config) -> Result<()> {
    // Load user's private key (NEVER hardcode!)
    let private_key = load_private_key_from_file("~/.riglr/keys/trading.key")?;
    
    // Create a signer
    let signer = Arc::new(LocalSolanaSigner::new(
        private_key,
        app_context.get_extension::<Arc<SolanaRpcClient>>()?,
    ));
    
    // Wrap all agent operations in SignerContext
    SignerContext::with_signer(signer, async move {
        let agent = TradingAgent::new(
            &config,
            Arc::new(app_context),
        )?;
        
        // Now the agent can execute swaps!
        agent.run_interactive().await
    }).await
}
```

## Running and Testing Your Agent

### Setting Up Your Environment

First, copy the example environment file and configure it:

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```env
# Database (Redis is required)
REDIS_URL=redis://localhost:6379

# Network - Solana
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com

# Network - EVM chains (use RPC_URL_{CHAIN_ID} pattern)
RPC_URL_1=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY  # Ethereum
RPC_URL_137=https://polygon-rpc.com                       # Polygon
RPC_URL_42161=https://arb1.arbitrum.io/rpc               # Arbitrum

# Providers
OPENAI_API_KEY=sk-...

# Application settings (optional)
PORT=8080
ENVIRONMENT=development
LOG_LEVEL=info
```

### Running Different Modes

#### Interactive Mode (CLI Chat)

```bash
cargo run -- --interactive

# Example session:
# You: What's the current price of SOL?
# Agent: Checking the price of SOL...
# Agent: SOL is currently trading at $125.43, up 5.2% in the last 24 hours.
```

#### API Server Mode

```bash
cargo run -- --server --port 8080

# Test with curl:
curl -X POST http://localhost:8080/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Analyze my portfolio: wallet123.sol"}'
```

#### Job Worker Mode (for production)

```bash
# Start a worker that processes jobs from a queue
cargo run -- --worker --queue redis://localhost:6379

# The worker will:
# - Connect to the job queue
# - Process tool execution requests
# - Handle retries automatically
# - Report results back to the queue
```

### Testing Your Tools

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_portfolio_analysis

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

## Best Practices and Tips

### 1. Error Classification

Always classify errors correctly for optimal retry behavior:

```rust
// Network issues - will retry with exponential backoff
ToolError::retriable_string("Connection timeout")

// Rate limiting - will retry with longer delays
ToolError::rate_limited_string("API rate limit exceeded")

// Bad input - won't retry
ToolError::invalid_input_string("Invalid wallet address")

// Permanent failures - won't retry
ToolError::permanent_string("Insufficient funds")
```

### 2. Tool Documentation

Write clear doc comments - they become prompts for the AI:

```rust
#[tool]
/// Analyzes market sentiment for a specific token.
/// Returns bullish/bearish indicators and a confidence score.
/// Use this before making large trades to gauge market conditions.
async fn analyze_market_sentiment(
    /// Token symbol (e.g., "SOL", "ETH")
    token: String,
    /// Time window: "1h", "24h", "7d" (default: "24h")
    timeframe: Option<String>,
) -> Result<SentimentAnalysis, ToolError> {
    // The AI will understand exactly when and how to use this tool
}
```

### 3. Testing Tools in Isolation

Create a test harness for individual tools:

```rust
// tests/tools_test.rs
#[tokio::test]
async fn test_tool_directly() {
    let context = create_test_context();
    
    // Test success case
    let result = get_token_price(
        "SOL".to_string(),
        "mainnet".to_string(),
        &context,
    ).await;
    
    assert!(result.is_ok());
    
    // Test error handling
    let error_result = get_token_price(
        "INVALID".to_string(),
        "mainnet".to_string(),
        &context,
    ).await;
    
    assert!(matches!(
        error_result,
        Err(ToolError::InvalidInput(_))
    ));
}
```

### 4. Monitoring and Logging

Add structured logging to your tools:

```rust
use tracing::{info, warn, error, instrument};

#[tool]
#[instrument(skip(context))]  // Automatic tracing
async fn execute_trade(
    params: TradeParams,
    context: &ApplicationContext,
) -> Result<TradeResult, ToolError> {
    info!("Starting trade execution for {}", params.token);
    
    // Log important decisions
    if params.amount > 1000.0 {
        warn!("Large trade detected: ${}", params.amount);
    }
    
    // Log errors with context
    match execute_internal(&params).await {
        Ok(result) => {
            info!("Trade successful: {}", result.tx_hash);
            Ok(result)
        }
        Err(e) => {
            error!("Trade failed: {}", e);
            Err(e)
        }
    }
}
```

## Next Steps

- **Deep Dive**: Learn about [The SignerContext Pattern](../concepts/signer-context.md) for secure transaction signing
- **Error Handling**: Understand [Error Classification](../concepts/error-handling.md) for resilient tools
- **Advanced Examples**: Check out our [Tutorials](../tutorials/index.md):
  - [Building a DeFi Arbitrage Bot](../tutorials/arbitrage-bot.md)
  - [Multi-Chain Portfolio Manager](../tutorials/cross-chain-portfolio.md)
  - [Real-time Event Monitoring](../tutorials/event-monitoring.md)
- **Production**: Learn about [Deployment Strategies](../deployment/index.md)