# riglr-evm-tools

Production-grade EVM blockchain tools for riglr agents, providing comprehensive Ethereum and EVM-compatible chain interactions.

## Features

- 🔐 **Secure Transaction Management**: Built-in signer context for safe key management
- 💰 **Balance Operations**: Check ETH and ERC20 token balances
- 📤 **Token Transfers**: Send ETH and ERC20 tokens with automatic gas estimation
- 🔄 **DeFi Integration**: Uniswap V3 support for token swaps and liquidity operations
- 🌐 **Multi-Chain Support**: Works with Ethereum, Polygon, Arbitrum, Optimism, Base, and more
- ⚡ **High Performance**: Async/await with connection pooling and retry logic
- 🛡️ **Error Handling**: Distinguishes between retriable and permanent failures

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
riglr-evm-tools = "0.1.0"
```

## Quick Start

### Setting up the Client

```rust
use riglr_evm_tools::EvmClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a client for Ethereum mainnet
    let client = EvmClient::mainnet().await?;
    
    // Or use a custom RPC endpoint
    let client = EvmClient::new("https://your-rpc-endpoint.com".to_string()).await?;
    
    // Add a signer for transactions (optional, only needed for sending transactions)
    let client_with_signer = client.with_signer("YOUR_PRIVATE_KEY");
    
    Ok(())
}
```

### Checking Balances

```rust
use riglr_evm_tools::{EvmClient, get_eth_balance};

// Create client
let client = EvmClient::mainnet().await?;

// Get ETH balance
let balance = get_eth_balance(
    &client,
    "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(),
    None, // Latest block
).await?;

println!("Balance: {} ETH", balance.balance_formatted);
```

### Transferring Tokens

```rust
use riglr_evm_tools::{EvmClient, transfer_eth, transfer_erc20};

// Create client with signer for transactions
let client = EvmClient::mainnet().await?
    .with_signer("YOUR_PRIVATE_KEY");

// Transfer ETH
let tx = transfer_eth(
    &client,
    "0xRecipientAddress".to_string(),
    0.1, // Amount in ETH
    None, // Auto gas price
    None, // Auto nonce
).await?;

println!("Transaction hash: {}", tx.tx_hash);

// Transfer ERC20 tokens
let tx = transfer_erc20(
    &client,
    "0xUSDC_CONTRACT".to_string(),
    "0xRecipientAddress".to_string(),
    "100".to_string(), // Amount
    6, // USDC has 6 decimals
    None, // Auto gas price
).await?;
```

### DeFi Operations (Uniswap)

```rust
use riglr_evm_tools::{EvmClient, get_uniswap_quote, perform_uniswap_swap};

// Create client with signer for swaps
let client = EvmClient::mainnet().await?
    .with_signer("YOUR_PRIVATE_KEY");

// Get a swap quote
let quote = get_uniswap_quote(
    &client,
    "0xUSDC".to_string(),     // Token in
    "0xWETH".to_string(),     // Token out
    "1000".to_string(),       // Amount in
    6,                        // USDC decimals
    18,                       // WETH decimals
    Some(3000),              // 0.3% fee tier
    Some(50),                // 0.5% slippage
).await?;

println!("Expected output: {} WETH", quote.amount_out);

// Execute the swap
let swap = perform_uniswap_swap(
    &client,
    "0xUSDC".to_string(),
    "0xWETH".to_string(),
    "1000".to_string(),
    6,
    quote.amount_out_minimum,
    Some(3000),
    Some(300), // 5 minute deadline
).await?;
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
# Optional: Default RPC endpoints
ETH_RPC_URL=https://eth.llamarpc.com
POLYGON_RPC_URL=https://polygon-rpc.com
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc

# Private key for transactions (required for signing)
EVM_PRIVATE_KEY=0x...
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

## Error Handling

All tools use the `ToolError` pattern to distinguish between retriable and permanent failures:

```rust
let client = EvmClient::mainnet().await?;

match get_eth_balance(&client, address, None).await {
    Ok(balance) => println!("Success: {}", balance.balance_formatted),
    Err(ToolError::Retriable(msg)) => {
        // Network issues, can retry
        println!("Temporary error: {}", msg);
    }
    Err(ToolError::Permanent(msg)) => {
        // Invalid input, don't retry
        println!("Permanent error: {}", msg);
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