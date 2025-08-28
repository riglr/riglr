# riglr-solana-tools

Production-grade Solana blockchain tools for riglr agents, providing comprehensive Solana network interactions.

[![Crates.io](https://img.shields.io/crates/v/riglr-solana-tools.svg)](https://crates.io/crates/riglr-solana-tools)
[![Documentation](https://docs.rs/riglr-solana-tools/badge.svg)](https://docs.rs/riglr-solana-tools)

## Features

- 🔐 **Secure Transaction Management**: Thread-safe ApplicationContext with automatic client injection
- 💰 **Balance Operations**: Check SOL and SPL token balances with clean #[tool] functions
- 📤 **Token Transfers**: Send SOL and SPL tokens with automatic signer context management
- 🔄 **DeFi Integration**: Jupiter aggregator for token swaps and liquidity operations
- 🚀 **Pump.fun Integration**: Deploy and trade meme tokens on Solana's latest platform
- ⚡ **High Performance**: Async/await with connection pooling and retry logic
- 🛡️ **Error Handling**: Type-safe ToolError for distinguishing retriable and permanent failures
- 🔒 **Multi-Tenant Safe**: Automatic isolation between concurrent user requests
- 📖 **Clean Architecture**: Single #[tool] functions with automatic context injection

## Architecture

riglr-solana-tools uses the modern ApplicationContext pattern from riglr-core with automatic dependency injection:

### Clean Tool Functions

All tools use the same clean pattern with automatic context injection:

```rust
use riglr_core::provider::ApplicationContext;
use riglr_solana_tools::balance::get_sol_balance;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup ApplicationContext with Solana client
    let context = ApplicationContext::from_env();
    
    // Tools automatically use context.solana_client()
    let balance = get_sol_balance(
        "So11111111111111111111111111111111111111112".to_string(),
        &context,
    ).await?;
    
    println!("Balance: {} SOL", balance.sol);
    Ok(())
}
```

### Transaction Operations

Transaction tools work with SignerContext automatically detected from ApplicationContext:

```rust
use riglr_core::provider::ApplicationContext;
use riglr_solana_tools::transaction::transfer_sol;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = ApplicationContext::from_env();
    
    // Tool automatically uses signer context for transactions
    let result = transfer_sol(
        "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
        0.1, // 0.1 SOL
        Some("Payment".to_string()),
        None,
        &context,
    ).await?;
    
    println!("Transaction signature: {}", result.signature);
    Ok(())
}
```

## Tool Integration

All tools follow the `#[tool]` macro pattern and integrate seamlessly with ToolWorker:

```rust
use riglr_core::{ToolWorker, ExecutionConfig, Job, JobResult};
use riglr_core::provider::ApplicationContext;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = ApplicationContext::from_env();
    
    let worker = ToolWorker::new(
        ExecutionConfig::default(),
        context
    );

    // Execute tools via job system
    let job = Job::new("get_sol_balance", &json!({
        "address": "So11111111111111111111111111111111111111112"
    }), 3)?;

    let result: JobResult = worker.process_job(job).await?;
    println!("Job result: {:?}", result);
    Ok(())
}
```

## Tool Implementation Pattern

All tools follow a consistent pattern with ApplicationContext automatic injection:

```rust
use riglr_core::provider::ApplicationContext;
use riglr_core::ToolError;
use riglr_macros::tool;
use std::sync::Arc;

#[tool]
pub async fn my_solana_tool(
    param: String,
    context: &ApplicationContext,
) -> Result<MyResult, ToolError> {
    // Get Solana RPC client from context extensions
    let rpc_client = context
        .get_extension::<Arc<solana_client::rpc_client::RpcClient>>()
        .ok_or_else(|| {
            ToolError::permanent_string("Solana RpcClient not found in context".to_string())
        })?;
    
    // Use the client for operations
    let result = rpc_client.get_account(&pubkey)
        .map_err(|e| ToolError::retriable_string(format!("Network error: {}", e)))?;
    
    Ok(MyResult { data: result })
}
```

**Key Points:**
- All tools use `context: &ApplicationContext` as the last parameter
- Solana RPC clients are injected via `context.get_extension()`
- Transaction tools automatically access `SignerContext` when needed
- Error handling distinguishes between retriable and permanent failures

## Available Tools

All tools use the new clean signature pattern:

### Balance Operations
```rust
#[tool]
pub async fn get_sol_balance(
    address: String,
    context: &ApplicationContext,
) -> Result<BalanceResult, ToolError>

#[tool] 
pub async fn get_spl_token_balance(
    owner_address: String,
    mint_address: String,
    context: &ApplicationContext,
) -> Result<TokenBalanceResult, ToolError>

#[tool]
pub async fn get_multiple_balances(
    addresses: Vec<String>,
    context: &ApplicationContext,
) -> Result<Vec<BalanceResult>, ToolError>
```

### Transaction Operations
```rust
#[tool]
pub async fn transfer_sol(
    to_address: String,
    amount_sol: f64,
    memo: Option<String>,
    priority_fee: Option<u64>,
    context: &ApplicationContext,
) -> Result<TransactionResult, ToolError>

#[tool]
pub async fn transfer_spl_token(
    mint_address: String,
    to_address: String,
    amount: u64,
    memo: Option<String>,
    context: &ApplicationContext,
) -> Result<TransactionResult, ToolError>
```

### Trading Operations
```rust
#[tool]
pub async fn get_jupiter_quote(
    input_mint: String,
    output_mint: String,
    amount: u64,
    slippage_bps: u16,
    only_direct_routes: bool,
    jupiter_api_url: Option<String>,
    context: &ApplicationContext,
) -> Result<SwapQuote, ToolError>

#[tool]
pub async fn perform_jupiter_swap(
    input_mint: String,
    output_mint: String,
    amount: u64,
    slippage_bps: u16,
    jupiter_api_url: Option<String>,
    use_versioned_transaction: bool,
    context: &ApplicationContext,
) -> Result<SwapResult, ToolError>

#[tool]
pub async fn get_token_price(
    base_mint: String,
    quote_mint: String,
    context: &ApplicationContext,
) -> Result<PriceResult, ToolError>
```

### Pump.fun Operations
```rust
#[tool]
pub async fn deploy_pump_token(
    name: String,
    symbol: String,
    description: String,
    image_url: Option<String>,
    initial_buy_sol: Option<f64>,
    context: &ApplicationContext,
) -> Result<PumpTokenResult, ToolError>

#[tool]
pub async fn buy_pump_token(
    token_mint: String,
    sol_amount: f64,
    slippage_bps: u16,
    max_sol_cost: Option<f64>,
    context: &ApplicationContext,
) -> Result<PumpTradeResult, ToolError>

#[tool]
pub async fn sell_pump_token(
    token_mint: String,
    token_amount: u64,
    slippage_bps: u16,
    min_sol_output: Option<f64>,
    context: &ApplicationContext,
) -> Result<PumpTradeResult, ToolError>

#[tool]
pub async fn get_pump_token_info(
    token_mint: String,
    context: &ApplicationContext,
) -> Result<PumpTokenInfo, ToolError>
```

### Network Operations
```rust
#[tool]
pub async fn get_block_height(
    context: &ApplicationContext,
) -> Result<u64, ToolError>

#[tool]
pub async fn get_transaction_status(
    signature: String,
    context: &ApplicationContext,
) -> Result<TransactionStatusResult, ToolError>
```

## Error Handling

All tools use structured error handling with retry classification and preserve the original error context for downcasting:

### Basic Error Handling

```rust
use riglr_core::{ToolError, provider::ApplicationContext};
use riglr_solana_tools::balance::get_sol_balance;

async fn handle_balance_check(context: &ApplicationContext, address: String) {
    match get_sol_balance(address, context).await {
        Ok(balance) => println!("Balance: {} SOL", balance.sol),
        Err(ToolError::Retriable { message, .. }) => {
            // Network errors - safe to retry
            eprintln!("Temporary error: {}", message);
        },
        Err(ToolError::Permanent { message, .. }) => {
            // Invalid input or permanent failures  
            eprintln!("Permanent error: {}", message);
        },
    }
}
```

### Structured Error Context Preservation

riglr-solana-tools preserves the original `SolanaToolError` as the source when converting to `ToolError`, enabling downcasting for detailed error handling:

```rust
use riglr_core::ToolError;
use riglr_solana_tools::error::SolanaToolError;
use riglr_solana_tools::balance::get_sol_balance;

async fn handle_with_downcasting(context: &ApplicationContext, address: String) {
    match get_sol_balance(address, context).await {
        Ok(balance) => println!("Balance: {} SOL", balance.sol),
        Err(tool_error) => {
            // Access the structured error context via downcasting
            if let Some(source) = tool_error.source() {
                if let Some(solana_error) = source.downcast_ref::<SolanaToolError>() {
                    match solana_error {
                        SolanaToolError::InvalidAddress(addr) => {
                            eprintln!("Invalid Solana address format: {}", addr);
                        },
                        SolanaToolError::InsufficientBalance { required, available } => {
                            eprintln!("Need {} SOL but only have {} SOL", required, available);
                        },
                        SolanaToolError::TransactionFailed(msg) => {
                            eprintln!("Transaction failed: {}", msg);
                        },
                        _ => eprintln!("Solana error: {}", solana_error),
                    }
                }
            }
        },
    }
}
```

This pattern enables:
- **Retry Logic**: Use ToolError's classification for retry decisions
- **Detailed Diagnostics**: Downcast to SolanaToolError for specific error details
- **Backwards Compatibility**: Code using basic error handling continues to work
- **Type Safety**: Compile-time guarantees for error handling

## Configuration

Configure Solana endpoints and API keys via environment variables or riglr-config:

```bash
# Solana RPC
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com

# Jupiter API (optional)
JUPITER_API_URL=https://quote-api.jup.ag

# Pump.fun API (optional)  
PUMP_API_URL=https://pumpapi.fun/api
```

ApplicationContext automatically loads these configurations and injects the appropriate clients into tools.

See [riglr-config](../riglr-config) for complete configuration options.

## Migration from Legacy Patterns

If you're upgrading from the dual-API pattern:

### Before (Legacy)
```rust
// OLD - dual API pattern
get_sol_balance_with_context(address, &app_context).await?;
get_sol_balance(address, rpc_client).await?;
```

### After (Current)  
```rust
// NEW - clean single API
get_sol_balance(address, &context).await?;
```

The new pattern:
- ✅ Single function per tool 
- ✅ Automatic context injection
- ✅ Cleaner signatures
- ✅ Better error handling
- ✅ Consistent across all tools