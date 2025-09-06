# riglr-evm-tools

Production-grade EVM blockchain tools for riglr agents, providing comprehensive Ethereum and EVM-compatible chain interactions.

## Features

- 🔐 **Secure Transaction Management**: Thread-safe SignerContext pattern for multi-tenant key management
- 💰 **Balance Operations**: Check ETH and ERC20 token balances with ApplicationContext
- 📤 **Token Transfers**: Send ETH and ERC20 tokens with SignerContext and automatic gas estimation
- 🔄 **DeFi Integration**: Uniswap V3 support for token swaps and liquidity operations
- 🌐 **Multi-Chain Support**: Works with Ethereum, Polygon, Arbitrum, Optimism, Base, and more
- ⚡ **High Performance**: Async/await with connection pooling and retry logic
- 🛡️ **Error Handling**: Type-safe ToolError for distinguishing retriable and permanent failures
- 🔒 **Multi-Tenant Safe**: Automatic isolation between concurrent user requests
- 📖 **Clear Separation**: ApplicationContext for read operations, SignerContext for write operations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
riglr-evm-tools = "0.1.0"
```

## Quick Start

### Using LocalEvmSigner

The `LocalEvmSigner` is the primary concrete implementation for EVM transaction signing:

```rust
use riglr_evm_tools::LocalEvmSigner;
use riglr_config::EvmNetworkConfig;
use riglr_core::{ApplicationContext, UnifiedSigner};
use std::sync::Arc;

// Create network config
let config = EvmNetworkConfig::new(
    "mainnet",
    1,
    "https://eth-mainnet.g.alchemy.com/v2/your-key".to_string()
);

// Create signer from private key
let signer = LocalEvmSigner::new(
    "0xYOUR_PRIVATE_KEY_HEX",
    config
)?;

// Set up application context
let app_context = ApplicationContext::new()?;
let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(signer);
app_context.set_signer(unified_signer).await?;

// Now you can use any EVM tool
use riglr_evm_tools::transaction::send_eth;
let tx_hash = send_eth("0xRECIPIENT_ADDRESS", U256::from(1_000_000_000_000_000_000u64)).await?;
```

### Read Operations with ApplicationContext

Read-only operations (like checking balances) use the ApplicationContext pattern:

```rust
use riglr_core::provider::ApplicationContext;
use riglr_evm_tools::balance::{get_eth_balance, get_erc20_balance};
use alloy::providers::{Provider, ProviderBuilder};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create application context with provider
    let app_context = ApplicationContext::from_env();
    
    // Add a Provider to the context for EVM operations
    let provider = ProviderBuilder::new()
        .on_http("https://eth.llamarpc.com".parse()?)
        .boxed();
    app_context.add_extension(Arc::new(provider) as Arc<dyn Provider>);
    
    // Check ETH balance
    let balance = get_eth_balance(
        "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(),
        Some(1), // Ethereum mainnet
        &app_context
    ).await?;
    
    println!("Balance: {}", balance.balance_formatted);
    Ok(())
}
```

### Write Operations with SignerContext

Write operations (like transfers) require a SignerContext with proper signing capabilities:

```rust
use riglr_core::signer::{SignerContext, UnifiedSigner};
use riglr_evm_tools::signer::LocalEvmSigner;
use riglr_evm_tools::transaction::send_eth;
use alloy::primitives::{Address, U256};
use std::sync::Arc;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create configuration for the network
    let config = riglr_config::EvmNetworkConfig::new(
        "mainnet",
        1,
        "https://eth.llamarpc.com".to_string()
    );
    
    // Create a signer with private key
    let signer = Arc::new(LocalEvmSigner::new(
        std::env::var("EVM_PRIVATE_KEY")?,
        config
    )?) as Arc<dyn UnifiedSigner>;
    
    // Execute transfer within SignerContext
    let tx_hash = SignerContext::with_signer(signer, async {
        let to = Address::from_str("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb5")?;
        let amount = U256::from(1_000_000_000_000_000u64); // 0.001 ETH in wei
        
        let hash = send_eth(to, amount).await?;
        Ok::<_, anyhow::Error>(hash)
    }).await?;
    
    println!("Transaction sent: {}", tx_hash);
    Ok(())
}
```

### DeFi Operations (Uniswap)

```rust
use riglr_core::signer::{SignerContext, UnifiedSigner};
use riglr_evm_tools::signer::LocalEvmSigner;
use riglr_evm_tools::swap::{get_uniswap_quote, perform_uniswap_swap};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create signer for swap operations
    let config = riglr_config::EvmNetworkConfig::new(
        "mainnet",
        1,
        "https://eth.llamarpc.com".to_string()
    );
    
    let signer = Arc::new(LocalEvmSigner::new(
        std::env::var("EVM_PRIVATE_KEY")?,
        config
    )?) as Arc<dyn UnifiedSigner>;
    
    // Execute swap within SignerContext
    SignerContext::with_signer(signer, async {
        // Get a swap quote (read operation)
        let quote = get_uniswap_quote(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(), // WETH
            "1000".to_string(),       // Amount in
            6,                        // USDC decimals
            18,                       // WETH decimals
            Some(3000),              // 0.3% fee tier
            Some(50),                // 0.5% slippage
        ).await?;
        
        println!("Expected output: {} WETH", quote.amount_out);
        
        // Execute the swap (write operation)
        let swap = perform_uniswap_swap(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(), // WETH
            "1000".to_string(),
            6,
            quote.amount_out_minimum,
            Some(3000),
            Some(300), // 5 minute deadline
        ).await?;
        
        println!("Swap executed: {}", swap.tx_hash);
        Ok(())
    }).await?;
    
    Ok(())
}
```

## Available Tools

### Balance Tools
- `get_eth_balance` - Get ETH balance for an address
- `get_erc20_balance` - Get ERC20 token balance

### Transaction Tools
- `transfer_eth` - Transfer ETH to another address
- `transfer_erc20` - Transfer ERC20 tokens
- `get_transaction_receipt` - Get receipt for a transaction

### DeFi Tools
- `get_uniswap_quote` - Get swap quote from Uniswap V3
- `perform_uniswap_swap` - Execute a token swap on Uniswap

## Configuration

### Environment Variables

```bash
# Convention-based RPC endpoints (RPC_URL_{CHAIN_ID})
RPC_URL_1=https://eth.llamarpc.com        # Ethereum (Chain ID: 1)
RPC_URL_137=https://polygon-rpc.com       # Polygon (Chain ID: 137)
RPC_URL_42161=https://arb1.arbitrum.io/rpc # Arbitrum (Chain ID: 42161)
RPC_URL_8453=https://base.llamarpc.com    # Base (Chain ID: 8453)

# Add any EVM chain by setting RPC_URL_{CHAIN_ID}
# Example: RPC_URL_10=https://optimism.io  # Optimism (Chain ID: 10)

# Private key for transactions (required for signing)
EVM_PRIVATE_KEY=0x...
```

### Extensible Chain Support

The EVM tools now support adding new chains without code changes. Simply set an environment variable following the `RPC_URL_{CHAIN_ID}` convention:

```rust
// Any chain with RPC_URL_{ID} set is automatically supported
let supported_chains = riglr_evm_tools::util::get_supported_chains();
println!("Supported chains: {:?}", supported_chains);

// Check if a specific chain is supported
if riglr_evm_tools::util::is_chain_supported(10) {
    println!("Optimism is configured!");
}
```

### Supported Networks

- Ethereum Mainnet
- Polygon
- Arbitrum One
- Optimism
- Base
- BNB Smart Chain
- Avalanche C-Chain
- Any EVM-compatible chain with custom RPC

## Architecture Patterns

### ApplicationContext for Read Operations

Read-only operations use ApplicationContext with a Provider extension:

```rust
use riglr_core::provider::ApplicationContext;
use alloy::providers::Provider;

// Setup once in your application
let app_context = ApplicationContext::from_env();
let provider = /* create provider */;
app_context.add_extension(Arc::new(provider) as Arc<dyn Provider>);

// Tools use the provider from context
let balance = get_eth_balance(address, chain_id, &app_context).await?;
```

### SignerContext for Write Operations

Write operations require SignerContext with proper signing capabilities:

```rust
use riglr_core::signer::{SignerContext, UnifiedSigner};

// Create signer with private key
let signer = Arc::new(LocalEvmSigner::new(key, config)?) as Arc<dyn UnifiedSigner>;

// Execute within context
SignerContext::with_signer(signer, async {
    // Operations here have access to signing capabilities
    let tx_hash = send_eth(to, amount).await?;
    Ok(tx_hash)
}).await?;
```

### Key Benefits

1. **Clear Separation**: Read vs write operations are clearly distinguished
2. **Thread Safety**: Each async task has its own isolated context
3. **Multi-Tenant Support**: Multiple users can execute operations concurrently
4. **Type Safety**: Operations are type-checked at compile time
5. **Automatic Context Propagation**: Tools automatically access the current context

### Type-Safe Chain Access

```rust
use riglr_core::signer::SignerContext;

// Get EVM-specific signer (fails if current signer doesn't support EVM)
let evm_signer = SignerContext::current_as_evm().await?;
let address = evm_signer.address();
let chain_id = evm_signer.chain_id();

// Check capabilities before operations
let signer = SignerContext::current().await?;
if signer.supports_evm() {
    // Perform EVM operations
}
```

## Error Handling

All tools use the `ToolError` pattern to distinguish between retriable and permanent failures:

```rust
use riglr_core::ToolError;
use riglr_evm_tools::balance::get_eth_balance;

match get_eth_balance(address, chain_id, &app_context).await {
    Ok(balance) => println!("Success: {}", balance.balance_formatted),
    Err(e) => match e {
        ToolError::Permanent { message, .. } => {
            // Invalid input, don't retry
            println!("Permanent error: {}", message);
        }
        ToolError::Retriable { message, .. } => {
            // Network issues, can retry
            println!("Temporary error: {}", message);
        }
        _ => println!("Other error: {:?}", e),
    }
}
```

## Integration with rig

All tools are compatible with the rig framework:

```rust
use rig::agents::Agent;
use riglr_evm_tools::get_eth_balance_tool;

let agent = Agent::builder()
    .preamble("You are an EVM blockchain analyst.")
    .tool(get_eth_balance_tool())
    .build();
```

## Safety and Security

- Private keys are managed per-client instance, not globally
- All addresses are validated before use
- Automatic nonce management prevents transaction conflicts
- Gas estimation with configurable limits
- Slippage protection for DeFi operations
- Thread-safe client instances can be cloned and shared

## License

MIT

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit PRs to our GitHub repository.

## Support

For issues and questions, please open an issue on GitHub or reach out to the maintainers.