# riglr-solana-tools

A production-grade Rust library for building on-chain AI agents on Solana. Part of the riglr ecosystem, this crate provides high-level, secure, and efficient tools for interacting with the Solana blockchain.

## Features

- **Balance Queries**: Check SOL and SPL token balances
- **Network State**: Query blockchain state and transaction status
- **Secure Transactions**: Transfer SOL and SPL tokens with built-in security patterns
- **DeFi Integration**: Swap tokens via Jupiter aggregator
- **Production Ready**: Battle-tested patterns with comprehensive error handling
- **Developer Friendly**: Intuitive API with extensive documentation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
riglr-solana-tools = "0.1.0"
```

## Quick Start

### Basic Setup

```rust
use riglr_solana_tools::client::SolanaClient;
use riglr_solana_tools::balance::get_sol_balance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client for mainnet
    let client = SolanaClient::mainnet();
    
    // Query SOL balance
    let address = "11111111111111111111111111111111".to_string();
    let balance = get_sol_balance(&client, address).await?;
    
    println!("Balance: {} SOL", balance.sol);
    Ok(())
}
```

### Secure Transaction Signing

The library uses a secure `SignerContext` pattern to protect private keys:

```rust
use riglr_solana_tools::transaction::{SignerContext, init_signer_context, transfer_sol};
use solana_sdk::signature::Keypair;

// Initialize signer context (do this once at startup)
let mut context = SignerContext::new();
let keypair = Keypair::new(); // In production, load from secure storage
context.add_signer("main", keypair)?;
init_signer_context(context);

// Transfer SOL
let client = SolanaClient::mainnet();
let result = transfer_sol(
    &client,
    "recipient_address".to_string(),
    1.0, // Amount in SOL
    Some("main".to_string()), // Signer name
    Some("Payment for services".to_string()), // Optional memo
    None, // Optional priority fee
).await?;

println!("Transaction signature: {}", result.signature);
```

## Available Tools

### Balance Tools

#### `get_sol_balance`
Query SOL balance for an address.

```rust
let balance = get_sol_balance(&client, address).await?;
```

#### `get_spl_token_balance`
Query SPL token balance for an address and mint.

```rust
let balance = get_spl_token_balance(
    &client,
    owner_address,
    mint_address,
).await?;
```

### Network Tools

#### `get_block_height`
Get the current block height.

```rust
let height = get_block_height(&client).await?;
```

#### `get_transaction_status`
Check the status of a transaction.

```rust
let status = get_transaction_status(&client, signature).await?;
```

### Transaction Tools

#### `transfer_sol`
Transfer SOL between accounts.

```rust
let result = transfer_sol(
    &client,
    to_address,
    amount_sol,
    from_signer,
    memo,
    priority_fee,
).await?;
```

#### `transfer_spl_token`
Transfer SPL tokens between accounts.

```rust
let result = transfer_spl_token(
    &client,
    to_address,
    mint_address,
    amount,
    decimals,
    from_signer,
    create_ata_if_needed,
).await?;
```

### DeFi Tools (Jupiter Integration)

#### `get_jupiter_quote`
Get a swap quote from Jupiter aggregator.

```rust
let quote = get_jupiter_quote(
    input_mint,
    output_mint,
    amount,
    slippage_bps,
    only_direct_routes,
    jupiter_api_url,
).await?;
```

#### `perform_jupiter_swap`
Execute a token swap via Jupiter.

```rust
let result = perform_jupiter_swap(
    &client,
    input_mint,
    output_mint,
    amount,
    slippage_bps,
    signer_name,
    jupiter_api_url,
    use_versioned_transaction,
).await?;
```

#### `get_token_price`
Get current price for a token pair.

```rust
let price = get_token_price(
    base_mint,
    quote_mint,
    jupiter_api_url,
).await?;
```

## Security Considerations

### Private Key Management
- **Never** expose private keys in code
- Use the `SignerContext` pattern to isolate key management
- Load keys from secure storage (environment variables, HSM, etc.)

### Transaction Safety
- All transactions require explicit signer authorization
- Built-in idempotency support prevents duplicate transactions
- Comprehensive error handling for network issues

### Best Practices
1. Initialize `SignerContext` once at application startup
2. Use appropriate commitment levels for your use case
3. Implement proper error handling and retries
4. Monitor transaction status after submission

## Network Configuration

### Using Different Networks

```rust
// Mainnet (default)
let client = SolanaClient::mainnet();

// Devnet
let client = SolanaClient::devnet();

// Testnet
let client = SolanaClient::testnet();

// Custom RPC
let client = SolanaClient::with_rpc_url("https://your-rpc-endpoint.com");
```

### Commitment Levels

```rust
use solana_sdk::commitment_config::CommitmentLevel;

let client = SolanaClient::mainnet()
    .with_commitment(CommitmentLevel::Finalized);
```

## Error Handling

The library uses a custom error type `SolanaToolError` with detailed error variants:

```rust
use riglr_solana_tools::error::SolanaToolError;

match transfer_sol(...).await {
    Ok(result) => println!("Success: {}", result.signature),
    Err(SolanaToolError::InvalidAddress(msg)) => println!("Invalid address: {}", msg),
    Err(SolanaToolError::InsufficientBalance) => println!("Insufficient balance"),
    Err(SolanaToolError::Transaction(msg)) => println!("Transaction failed: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

## Examples

See the `examples/` directory for complete working examples:

- `balance_checker.rs` - Query balances across multiple addresses
- `simple_swapper.rs` - Perform token swaps via Jupiter

Run examples with:

```bash
cargo run --example balance_checker
cargo run --example simple_swapper
```

## Testing

Run the test suite:

```bash
# Unit tests
cargo test --lib

# Integration tests (requires network connection)
cargo test --test '*'

# All tests
cargo test --workspace
```

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass
2. Code follows Rust idioms
3. Documentation is updated
4. Changes are covered by tests

## License

MIT License - see LICENSE file for details

## Support

For issues and questions:
- GitHub Issues: [riglr/issues](https://github.com/riglr/riglr/issues)
- Documentation: [riglr.dev](https://riglr.dev)

## Acknowledgments

Built with battle-tested patterns from production Solana applications. Special thanks to the Solana and Jupiter teams for their excellent APIs and documentation.