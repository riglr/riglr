# riglr-macros

Procedural macros for the riglr ecosystem, providing code generation for tool definitions and error handling with automatic dependency injection.

[![Crates.io](https://img.shields.io/crates/v/riglr-macros.svg)](https://crates.io/crates/riglr-macros)
[![Documentation](https://docs.rs/riglr-macros/badge.svg)](https://docs.rs/riglr-macros)

## Features

- **`#[tool]` macro**: Automatically implement `riglr_core::Tool` for functions and structs
- **Type-based dependency injection**: Automatic detection and injection of ApplicationContext
- **Automatic error mapping**: Convert function errors to `ToolError` with proper retry classification
- **AI-friendly descriptions**: Generate tool descriptions from doc comments or attributes
- **Type safety**: Generate strongly-typed parameter structs with validation
- **Serde integration**: Automatic serialization/deserialization for tool parameters and results

## The `#[tool]` Macro

The `#[tool]` macro transforms async functions and structs into riglr tools with automatic dependency injection by generating:

- **Args struct** with serde/schemars for parameter validation
- **Tool trait implementation** with proper error mapping to `ToolError`
- **Description extraction** from doc comments or attributes
- **Automatic context injection** based on parameter type signatures

## Type-Based Dependency Injection

The macro uses **type-based detection** to automatically identify and inject dependencies. Functions with `ApplicationContext` parameters are automatically detected and the context is injected at runtime - no attributes required!

### Basic Function Tool with Context

```rust
use riglr_macros::tool;
use riglr_core::{ToolError, provider::ApplicationContext};

/// Checks the SOL balance for a given Solana address
#[tool]
async fn check_sol_balance(
    context: &ApplicationContext,  // Automatically detected and injected
    address: String,
) -> Result<f64, ToolError> {
    // Access blockchain clients through context
    let solana_client = context.solana_client()?;
    
    // Implementation would check actual balance
    let balance = solana_client.get_balance(&address).await?;
    Ok(balance as f64 / 1_000_000_000.0) // Convert lamports to SOL
}
```

The macro generates:

```rust
// Generated parameter struct (only for user parameters)
#[derive(serde::Deserialize, schemars::JsonSchema)]
struct CheckSolBalanceArgs {
    address: String,  // ApplicationContext is excluded from Args struct
}

// Generated Tool implementation with automatic context injection
#[async_trait::async_trait]
impl riglr_core::Tool for CheckSolBalanceTool {
    async fn execute(
        &self, 
        params: serde_json::Value, 
        context: &ApplicationContext  // Context automatically passed here
    ) -> Result<JobResult, ToolError> {
        let args: CheckSolBalanceArgs = serde_json::from_value(params)?;
        // Call original function with injected context + user params
        let result = check_sol_balance(context, args.address).await?;
        Ok(JobResult::Success { 
            value: serde_json::to_value(result)?, 
            tx_hash: None 
        })
    }

    fn name(&self) -> &str {
        "check_sol_balance"
    }

    fn description(&self) -> &str {
        "Checks the SOL balance for a given Solana address"
    }
}
```

## How Type-Based Detection Works

The macro automatically identifies parameters by their type signature:

1. **ApplicationContext parameters**: Any parameter of type `&ApplicationContext`, `&riglr_core::provider::ApplicationContext`, or ending in `::ApplicationContext` is automatically detected
2. **User parameters**: All other parameters become fields in the generated Args struct
3. **Automatic injection**: The context is injected from the Tool trait's execute method
4. **Clean signatures**: Your tool functions have clean, explicit signatures showing exactly what dependencies they need

### Supported ApplicationContext Types

The macro recognizes these type patterns:
- `context: &ApplicationContext`
- `ctx: &riglr_core::provider::ApplicationContext`  
- `app_context: &my_crate::provider::ApplicationContext`

### Function Tool with Custom Description

You can override the description with an attribute:

```rust
use riglr_core::{ToolError, provider::ApplicationContext};

#[tool(description = "Gets current ETH price in USD from external API")]
async fn get_eth_price(
    context: &ApplicationContext,
) -> Result<f64, ToolError> {
    let web_client = context.web_client()?;
    let price_data = web_client.get_eth_price().await?;
    Ok(price_data.usd)
}
```

### Struct Tool Implementation

For more complex tools, implement them as structs. The macro handles context injection automatically:

```rust
use riglr_core::{Tool, JobResult, ToolError, provider::ApplicationContext};
use riglr_macros::tool;
use serde::{Serialize, Deserialize};

/// A comprehensive wallet management tool
#[derive(Serialize, Deserialize, schemars::JsonSchema, Clone)]
#[tool(description = "Manages wallet operations across multiple chains")]
struct WalletManager {
    operation: String,
    amount: Option<f64>,
    address: Option<String>,
}

impl WalletManager {
    /// Execute the wallet operation with the provided context
    pub async fn execute(&self, context: &ApplicationContext) -> Result<String, ToolError> {
        match self.operation.as_str() {
            "balance" => {
                let address = self.address.as_ref()
                    .ok_or_else(|| ToolError::invalid_input_string("Address required for balance check"))?;
                
                if let Ok(solana_client) = context.solana_client() {
                    let balance = self.check_solana_balance(context, address).await?;
                    Ok(format!("Solana balance: {} SOL", balance))
                } else if let Ok(evm_client) = context.evm_client() {
                    let balance = self.check_evm_balance(context, address).await?;
                    Ok(format!("EVM balance: {} ETH", balance))
                } else {
                    Err(ToolError::permanent_string("No supported blockchain client available"))
                }
            }
            "transfer" => {
                let amount = self.amount
                    .ok_or_else(|| ToolError::invalid_input_string("Amount required for transfer"))?;
                let to_address = self.address.as_ref()
                    .ok_or_else(|| ToolError::invalid_input_string("Destination address required"))?;
                
                let tx_hash = self.transfer_funds(context, to_address, amount).await?;
                Ok(format!("Transferred {} - tx: {}", amount, tx_hash))
            }
            _ => Err(ToolError::invalid_input_string(format!("Unknown operation: {}", self.operation)))
        }
    }
    
    async fn check_solana_balance(&self, context: &ApplicationContext, address: &str) -> Result<f64, ToolError> {
        let client = context.solana_client()?;
        // Implementation for Solana balance check using context's client
        Ok(1.5)
    }
    
    async fn check_evm_balance(&self, context: &ApplicationContext, address: &str) -> Result<f64, ToolError> {
        let client = context.evm_client()?;
        // Implementation for EVM balance check using context's client
        Ok(0.25)
    }
    
    async fn transfer_funds(&self, context: &ApplicationContext, to_address: &str, amount: f64) -> Result<String, ToolError> {
        if let Ok(solana_client) = context.solana_client() {
            // Perform Solana transfer
            Ok("solana_tx_hash_123".to_string())
        } else if let Ok(evm_client) = context.evm_client() {
            // Perform EVM transfer
            Ok("0xevm_tx_hash_456".to_string())
        } else {
            Err(ToolError::permanent_string("No supported blockchain client available"))
        }
    }
}
```

### Advanced Error Handling

The `#[tool]` macro automatically maps function errors to `ToolError` types. You can use the enhanced error handling:

```rust
use riglr_core::{ToolError, provider::ApplicationContext};

#[tool]
async fn transfer_tokens(
    context: &ApplicationContext,
    to_address: String, 
    amount: f64,
    token_mint: String,
) -> Result<String, ToolError> {
    // Input validation
    if amount <= 0.0 {
        return Err(ToolError::invalid_input_string("Amount must be positive"));
    }
    
    // Access blockchain clients through context
    let solana_client = context.solana_client()
        .map_err(|_| ToolError::permanent_string("Solana client not available for token transfers"))?;
    
    // Simulate network error that should be retried
    if let Err(e) = perform_transfer(context, &to_address, amount, &token_mint).await {
        return Err(ToolError::retriable_with_source(e, "Failed to submit transaction"));
    }
    
    // Simulate rate limiting with proper retry delay
    if is_rate_limited(context).await {
        return Err(ToolError::rate_limited_string_with_delay(
            "API rate limit exceeded",
            Some(std::time::Duration::from_secs(60))
        ));
    }
    
    Ok("transaction_hash_123".to_string())
}

async fn perform_transfer(
    context: &ApplicationContext,
    to: &str, 
    amount: f64, 
    token: &str
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = context.solana_client()?;
    // Implementation would perform actual transfer using context's client
    Ok(())
}

async fn is_rate_limited(context: &ApplicationContext) -> bool {
    // Check if we're being rate limited via context's rate limiter
    false
}
```

### Working with ApplicationContext

Tools automatically have access to blockchain clients and other services through ApplicationContext:

```rust
use riglr_core::{ToolError, provider::ApplicationContext};

#[tool]
async fn multi_chain_balance(
    context: &ApplicationContext,
    address: String,
) -> Result<serde_json::Value, ToolError> {
    let mut balances = serde_json::Map::new();
    
    // Try to get Solana balance
    if let Ok(solana_client) = context.solana_client() {
        match get_solana_balance(context, &address).await {
            Ok(sol_balance) => {
                balances.insert("solana".to_string(), serde_json::json!({
                    "balance": sol_balance,
                    "currency": "SOL"
                }));
            }
            Err(e) => {
                // Log error but continue to other chains
                eprintln!("Failed to get Solana balance: {}", e);
            }
        }
    }
    
    // Try to get EVM balance
    if let Ok(evm_client) = context.evm_client() {
        match get_ethereum_balance(context, &address).await {
            Ok(eth_balance) => {
                balances.insert("ethereum".to_string(), serde_json::json!({
                    "balance": eth_balance,
                    "currency": "ETH"
                }));
            }
            Err(e) => {
                eprintln!("Failed to get Ethereum balance: {}", e);
            }
        }
    }
    
    if balances.is_empty() {
        return Err(ToolError::permanent_string("No supported blockchain clients available"));
    }
    
    Ok(serde_json::Value::Object(balances))
}

async fn get_solana_balance(context: &ApplicationContext, address: &str) -> Result<f64, ToolError> {
    let client = context.solana_client()?;
    // Implementation using the Solana client from context
    Ok(1.5)
}

async fn get_ethereum_balance(context: &ApplicationContext, address: &str) -> Result<f64, ToolError> {
    let client = context.evm_client()?;
    // Implementation using the EVM client from context
    Ok(0.25)
}
```

## Benefits of Type-Based Dependency Injection

The new architecture provides several advantages over the previous `#[context]` attribute approach:

### 1. **Clean, Explicit Signatures**
- Function signatures clearly show what dependencies are needed
- No hidden dependencies - everything is explicit in the function signature
- Easy to understand for both humans and AI assistants

### 2. **Automatic Detection** 
- No need to remember special attributes like `#[context]`
- The macro automatically detects ApplicationContext by type
- Reduces boilerplate and potential for errors

### 3. **Better IDE Support**
- IDEs can provide better autocomplete and error checking
- Type information is preserved throughout the process
- Easier refactoring when dependency types change

### 4. **rig-core Compatibility**
- Generated tools work seamlessly with the rig framework
- Standard Tool trait implementation with proper context passing
- Easy integration into existing rig-based applications

### 5. **Simplified Migration**
- Old `_with_context` patterns are no longer needed
- Cleaner, more maintainable code
- Consistent pattern across all tools

## Description Extraction

The macro extracts descriptions in priority order:

1. **Attribute `description = "..."`**: Explicit description override
2. **Rust doc comments**: First line of doc comments on the function/struct
3. **Empty string**: Fallback if no description is found

```rust
/// This is the primary description from doc comments
/// Additional documentation here is ignored for the tool description
#[tool]
async fn documented_tool() -> Result<String, ToolError> {
    Ok("result".to_string())
}

#[tool(description = "This explicit description overrides doc comments")]
/// This doc comment will be ignored for tool description
async fn explicit_description_tool() -> Result<String, ToolError> {
    Ok("result".to_string())
}
```

## Integration with riglr-core

Tools generated by the macro integrate seamlessly with `riglr-core` and the rig framework:

```rust
use riglr_core::{ToolWorker, ExecutionConfig, Job, provider::ApplicationContext};
use riglr_macros::tool;
use std::sync::Arc;

#[tool]
async fn example_tool(
    context: &ApplicationContext,
    param: String,
) -> Result<String, riglr_core::ToolError> {
    // Use context to access services
    let web_client = context.web_client()?;
    Ok(format!("Processed: {}", param))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create application context with necessary clients
    let context = ApplicationContext::builder()
        .with_web_client()
        .build()?;
    
    // Create worker with context
    let worker = ToolWorker::new(ExecutionConfig::default(), context);
    
    // Register the generated tool using the factory function
    worker.register_tool(example_tool_tool()).await;
    
    // Create and process a job - context is automatically injected
    let job = Job::new(
        "example_tool",
        &serde_json::json!({"param": "test data"}),
        3
    )?;
    
    let result = worker.process_job(job).await?;
    println!("Result: {:?}", result);
    
    Ok(())
}
```

## Parameter Validation

The generated parameter structs support full serde validation. Parameters are automatically validated when the tool is executed:

```rust
use serde::{Deserialize, Serialize};
use riglr_core::{ToolError, provider::ApplicationContext};
use riglr_macros::tool;

#[tool]
async fn transfer_with_validation(
    context: &ApplicationContext,
    #[serde(alias = "to")]
    recipient_address: String,
    
    #[serde(deserialize_with = "validate_positive_amount")]
    amount: f64,
    
    #[serde(default = "default_slippage")]
    slippage_bps: u16,
) -> Result<String, ToolError> {
    // Parameters are already validated by serde before this function is called
    let solana_client = context.solana_client()?;
    
    // Perform transfer with validated parameters
    let tx_hash = solana_client.transfer(
        &recipient_address, 
        amount, 
        slippage_bps
    ).await?;
    
    Ok(format!(
        "Transferred {} to {} with {}bps slippage - tx: {}", 
        amount, 
        recipient_address,
        slippage_bps,
        tx_hash
    ))
}

fn validate_positive_amount<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let amount = f64::deserialize(deserializer)?;
    if amount <= 0.0 {
        return Err(serde::de::Error::custom("Amount must be positive"));
    }
    Ok(amount)
}

fn default_slippage() -> u16 {
    100 // 1% default slippage
}
```

## Best Practices

### 1. Function Design
- Always include `ApplicationContext` as the first parameter for tools that need external services
- Use descriptive parameter names that clearly indicate their purpose
- Provide comprehensive doc comments for each parameter
- Use appropriate default values with `#[serde(default)]` where applicable

### 2. Error Handling

Use the structured error types for better retry logic:

```rust
use riglr_core::{ToolError, provider::ApplicationContext};

#[tool]
async fn robust_tool(
    context: &ApplicationContext,
    param: String,
) -> Result<String, ToolError> {
    // Validate input
    if param.is_empty() {
        return Err(ToolError::invalid_input_string("Parameter cannot be empty"));
    }
    
    // Handle permanent errors (don't retry)
    if !has_required_permissions(context).await {
        return Err(ToolError::permanent_string("Insufficient permissions"));
    }
    
    // Handle retriable errors (retry with backoff)
    match make_network_call(context, &param).await {
        Ok(result) => Ok(result),
        Err(e) if is_network_error(&e) => {
            Err(ToolError::retriable_with_source(e, "Network call failed"))
        }
        Err(e) if is_rate_limited(&e) => {
            Err(ToolError::rate_limited_string_with_delay(
                "API rate limited",
                Some(std::time::Duration::from_secs(30))
            ))
        }
        Err(e) => Err(ToolError::permanent_with_source(e, "Unexpected error"))
    }
}

async fn has_required_permissions(context: &ApplicationContext) -> bool { 
    // Check permissions using context services
    true 
}

async fn make_network_call(
    context: &ApplicationContext, 
    param: &str
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> { 
    let client = context.web_client()?;
    // Make network call using context's HTTP client
    Ok(param.to_string()) 
}

fn is_network_error(_e: &dyn std::error::Error) -> bool { false }
fn is_rate_limited(_e: &dyn std::error::Error) -> bool { false }
```

### 3. ApplicationContext Usage

Always check client availability and handle graceful fallbacks:

```rust
#[tool]
async fn chain_specific_tool(
    context: &ApplicationContext,
    operation: String,
) -> Result<String, ToolError> {
    match operation.as_str() {
        "solana_op" => {
            let solana_client = context.solana_client()
                .map_err(|_| ToolError::permanent_string("Solana client not available"))?;
            
            // Perform Solana operation
            let result = solana_client.get_latest_blockhash().await?;
            Ok(format!("Solana operation completed: {}", result))
        }
        "evm_op" => {
            let evm_client = context.evm_client()
                .map_err(|_| ToolError::permanent_string("EVM client not available"))?;
            
            // Perform EVM operation  
            let block_number = evm_client.get_block_number().await?;
            Ok(format!("EVM operation completed at block: {}", block_number))
        }
        "web_op" => {
            let web_client = context.web_client()
                .map_err(|_| ToolError::permanent_string("Web client not available"))?;
                
            // Perform web operation
            let response = web_client.get("https://api.example.com").await?;
            Ok(format!("Web operation completed: {}", response.status()))
        }
        _ => Err(ToolError::invalid_input_string("Unknown operation"))
    }
}
```

### 4. Context-Aware Design

Design tools that gracefully adapt to available services:

```rust
#[tool]
async fn adaptive_balance_check(
    context: &ApplicationContext,
    address: String,
) -> Result<serde_json::Value, ToolError> {
    let mut results = serde_json::Map::new();
    
    // Try each available blockchain client
    if let Ok(solana_client) = context.solana_client() {
        match solana_client.get_balance(&address).await {
            Ok(balance) => {
                results.insert("solana".to_string(), serde_json::json!({
                    "balance": balance,
                    "status": "success"
                }));
            }
            Err(e) => {
                results.insert("solana".to_string(), serde_json::json!({
                    "error": e.to_string(),
                    "status": "error"
                }));
            }
        }
    }
    
    if let Ok(evm_client) = context.evm_client() {
        match evm_client.get_balance(&address).await {
            Ok(balance) => {
                results.insert("ethereum".to_string(), serde_json::json!({
                    "balance": balance.to_string(),
                    "status": "success"
                }));
            }
            Err(e) => {
                results.insert("ethereum".to_string(), serde_json::json!({
                    "error": e.to_string(),
                    "status": "error"
                }));
            }
        }
    }
    
    if results.is_empty() {
        return Err(ToolError::permanent_string("No blockchain clients available"));
    }
    
    Ok(serde_json::Value::Object(results))
}
```

## Migration from Previous Versions

If you're upgrading from a previous version that used `#[context]` attributes or `SignerContext`, here's how to migrate:

### Before (Old Architecture)
```rust
#[tool]
async fn old_transfer(
    #[context] _ctx: &SignerContext,  // ❌ Old way
    to_address: String,
    amount: f64,
) -> Result<String, ToolError> {
    let signer = SignerContext::current().await?;  // ❌ Old way
    // ... implementation
}
```

### After (New Architecture)  
```rust
#[tool]
async fn new_transfer(
    context: &ApplicationContext,  // ✅ New way - automatically detected
    to_address: String,
    amount: f64,
) -> Result<String, ToolError> {
    let client = context.solana_client()?;  // ✅ New way - use context directly
    // ... implementation
}
```

### Key Changes
1. Remove `#[context]` attributes - they're no longer needed
2. Replace `SignerContext::current().await?` with direct context usage  
3. Add `ApplicationContext` parameter to function signatures
4. Access clients through `context.solana_client()`, `context.evm_client()`, etc.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
riglr-macros = "0.2.0"
riglr-core = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-trait = "0.1"
```

## Quick Start

```rust
use riglr_macros::tool;
use riglr_core::{ToolError, provider::ApplicationContext};

#[tool]
async fn hello_world(
    context: &ApplicationContext,
    name: String,
) -> Result<String, ToolError> {
    Ok(format!("Hello, {}!", name))
}

fn main() {
    // Your tool is ready to use!
    let tool = hello_world_tool();
    println!("Created tool: {}", tool.name());
}
```

## Examples

See the `examples/` directory in the riglr-core crate for complete working examples using the `#[tool]` macro with the new ApplicationContext architecture.

## Requirements

- **Rust 1.70+**: For async trait support and modern language features
- **Function Requirements**: Tools must be async functions returning `Result<T, E>` where `E: Into<ToolError>`
- **Context Requirements**: Exactly one `ApplicationContext` parameter is required for dependency injection
- **Parameter Requirements**: All user parameters must implement `Serialize + Deserialize + JsonSchema`

## Testing

```bash
cargo test --workspace
```

## License

MIT License - see LICENSE file for details
