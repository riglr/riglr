# riglr-solana-tools

{{#include ../../../riglr-solana-tools/README.md}}

## API Reference

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Functions](#functions)
- [Type Aliases](#type-aliases)
- [Constants](#constants)

### Structs

> Core data structures and types.

#### `ApiClients`

Collection of all external API clients

---

#### `Args`

Arguments structure for the tool

---

#### `BalanceResult`

Result structure for balance queries

Contains balance information for a Solana address including both raw lamports
and human-readable SOL amounts.

---

#### `CreateMintResult`

Result of creating a new SPL token mint

---

#### `JupiterClient`

HTTP client for Jupiter aggregator API

---

#### `LocalSolanaSigner`

Local Solana signer with keypair management

---

#### `PriceInfo`

Token price information

---

#### `PriorityFeeConfig`

Solana priority fee configuration

---

#### `PumpClient`

HTTP client for Pump.fun API

---

#### `PumpTokenInfo`

Information about a Pump.fun token

---

#### `PumpTradeAnalysis`

Result of analyzing a Pump.fun trade transaction

---

#### `PumpTradeResult`

Result of a Pump.fun trade operation

---

#### `RoutePlanStep`

Route plan step in Jupiter quote

---

#### `SolanaAccount`

Common Solana account metadata

---

#### `SolanaAddress`

Type-safe wrapper for Solana addresses

---

#### `SolanaConfig`

Shared configuration for Solana operations

---

#### `SolanaSignature`

Type-safe wrapper for Solana signatures

---

#### `SolanaTransactionData`

Solana transaction metadata shared between crates

---

#### `SwapInfo`

Swap information for a route step

---

#### `SwapQuote`

Result of a swap quote from Jupiter

---

#### `SwapResult`

Result of a swap execution

---

#### `TokenBalanceResult`

Result structure for SPL token balance queries

Contains balance information for a specific SPL token including both raw amounts
and decimal-adjusted values.

---

#### `TokenTransferResult`

Result of an SPL token transfer transaction

---

#### `Tool`

Tool implementation structure

---

#### `TransactionResult`

Result of a SOL transfer transaction

---

#### `TransactionSubmissionResult`

Result of a transaction submission

---

### Enums

> Enumeration types for representing variants.

#### `PermanentError`

Permanent errors that should not be retried

**Variants:**

- `InsufficientFunds`
  - Insufficient funds for transaction
- `InvalidSignature`
  - Invalid signature provided
- `InvalidAccount`
  - Invalid account referenced
- `InstructionError`
  - Program execution error
- `InvalidTransaction`
  - Invalid transaction structure
- `DuplicateTransaction`
  - Duplicate transaction

---

#### `PumpTradeType`

Type of trade on Pump.fun

**Variants:**

- `Buy`
  - Buy tokens with SOL
- `Sell`
  - Sell tokens for SOL

---

#### `RateLimitError`

Rate limiting errors with special handling

**Variants:**

- `RpcRateLimit`
  - Standard RPC rate limiting
- `TooManyRequests`
  - Too many requests error

---

#### `RetryableError`

Errors that can be retried with appropriate backoff

**Variants:**

- `NetworkConnectivity`
  - Network connectivity issues
- `TemporaryRpcFailure`
  - RPC service temporary unavailability
- `NetworkCongestion`
  - Blockchain congestion
- `TransactionPoolFull`
  - Transaction pool full

---

#### `SolanaCommonError`

Error types for Solana operations shared across crates

**Variants:**

- `InvalidPubkey`
  - Invalid Solana public key format or encoding
- `ClientError`
  - Solana RPC client communication error
- `ParseError`
  - Failed to parse Solana-related data

---

#### `SolanaToolError`

Main error type for Solana tool operations.

**Variants:**

- `ToolError`
  - Core tool error - passthrough
- `SignerError`
  - Signer context error - configuration issue
- `Rpc`
  - RPC client error - network issues are typically retriable
- `SolanaClient`
  - Solana client error - classification depends on inner error
- `InvalidAddress`
  - Invalid address format - user input error
- `InvalidKey`
  - Invalid key format - user input error
- `InvalidSignature`
  - Invalid signature format - user input error
- `Transaction`
  - Transaction failed - may be retriable depending on message
- `InsufficientFunds`
  - Insufficient funds for operation - permanent error
- `InvalidTokenMint`
  - Invalid token mint - user input error
- `Serialization`
  - Serialization error - data corruption/format issue
- `Http`
  - HTTP request error - network issues are typically retriable
- `Core`
  - Core riglr error - typically retriable
- `Generic`
  - Generic error - default to retriable

---

#### `TransactionErrorType`

Structured classification of transaction errors for intelligent retry logic

**Variants:**

- `Retryable`
  - Errors that can be retried with appropriate backoff
- `Permanent`
  - Errors that represent permanent failures and should not be retried
- `RateLimited`
  - Rate limiting errors that require special handling with delays
- `Unknown`
  - Unknown error types that don't fit other categories

---

#### `TransactionStatus`

Transaction status

**Variants:**

- `Pending`
  - Transaction is pending confirmation
- `Confirmed`
  - Transaction is confirmed
- `Finalized`
  - Transaction is finalized
- `Failed`
  - Transaction failed

---

### Functions

> Standalone functions and utilities.

#### `analyze_pump_transaction`

Analyze a Pump.fun trade transaction to extract actual amounts and price

This tool parses a completed Pump.fun transaction to determine the actual
token amounts, SOL amounts, and price per token that were executed.
Separate from action tools following riglr separation of concerns pattern.

## Security Features
- Enhanced input validation for all parameters
- Rate limiting via SignerContext
- Signature format validation
- Address length validation

---

#### `analyze_pump_transaction_tool`

Factory function to create a new instance of the tool

---

#### `buy_pump_token`

Buy tokens on Pump.fun

This tool executes a buy order for a specific token on Pump.fun
with configurable slippage protection.

---

#### `buy_pump_token_tool`

Factory function to create a new instance of the tool

---

#### `classify_transaction_error`

Classify a Solana ClientError into a structured transaction error type

This function provides intelligent error classification based on the actual
error types from the Solana client, rather than brittle string matching.
It handles the most common error scenarios and provides appropriate
retry guidance.

---

#### `create_associated_token_account_idempotent_v3`

Create associated token account idempotent instruction with type conversion

---

#### `create_shared_solana_client`

Create a shared Solana RPC client

---

#### `create_solana_client`

Create a Solana RPC client with the given configuration

---

#### `create_spl_token_mint`

Create a new SPL token mint

This tool creates a new SPL token mint account on the Solana blockchain with specified
configuration parameters. The mint authority is set to the current signer, enabling
future token supply management operations.

This function creates the mint account, sets up the mint and freeze authorities
(if applicable), and optionally mints an initial supply of tokens to the creator's
associated token account.

# Arguments

* `decimals` - Number of decimal places for the token (0-9, commonly 6 or 9)
* `initial_supply` - Initial number of tokens to mint (in smallest unit)
* `freezable` - Whether the token accounts can be frozen by the freeze authority

# Returns

Returns `CreateMintResult` containing:
- `mint_address`: The newly created token mint address
- `transaction_signature`: Transaction signature of the mint creation
- `initial_supply`: Number of tokens initially minted
- `decimals`: Decimal places configuration
- `authority`: The mint authority address (signer address)

# Errors

Returns `ToolError` in the following cases:
* `ToolError::Permanent` - Invalid decimals value (must be 0-9)
* `ToolError::Retriable` - Network connection issues, insufficient SOL balance
* `ToolError::Permanent` - Signer context not available

# Examples

```rust,ignore
use riglr_solana_tools::transaction::create_spl_token_mint;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// This would create a USDC-like token with 6 decimals
let result = create_spl_token_mint(
    6,             // 6 decimal places
    1_000_000_000, // 1,000 tokens initial supply (1000 * 10^6)
    true,          // Allow freezing accounts
).await?;

println!("Created token mint: {}", result.mint_address);
println!("Initial supply: {} tokens", result.initial_supply);
println!("Transaction: {}", result.transaction_signature);
# Ok(())
# }
```

---

#### `create_spl_token_mint_tool`

Factory function to create a new instance of the tool

---

#### `create_token_with_mint_keypair`

Creates properly signed Solana transaction with mint keypair

This function creates a transaction with the given instructions and signs it
using both the payer from signer context and the provided mint keypair.

---

#### `default_solana_config`

Get the default Solana configuration from environment or defaults

---

#### `deploy_pump_token`

Deploy a new token on Pump.fun

This tool creates and deploys a new meme token on the Pump.fun platform.
Optionally performs an initial buy to bootstrap liquidity.

---

#### `deploy_pump_token_tool`

Factory function to create a new instance of the tool

---

#### `execute_solana_transaction`

Higher-order function to execute Solana transactions

Abstracts signer context retrieval and transaction signing, following the established
riglr pattern of using SignerContext for multi-tenant operation.

# Arguments

* `tx_creator` - Function that creates the transaction given a pubkey and RPC client

# Returns

Returns the transaction signature on success

# Examples

```rust,ignore
use riglr_solana_tools::utils::transaction::execute_solana_transaction;
use solana_sdk::transaction::Transaction;
use solana_system_interface::instruction as system_instruction;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let signature = execute_solana_transaction(|pubkey, client| async move {
    let to = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".parse()?;
    let instruction = system_instruction::transfer(&pubkey, &to, 1000000);

    let recent_blockhash = client.get_latest_blockhash()?;
    let mut tx = Transaction::new_with_payer(&[instruction], Some(&pubkey));
    tx.sign(&[], recent_blockhash);

    Ok(tx)
}).await?;

println!("Transaction sent: {}", signature);
# Ok(())
# }
```

---

#### `format_balance`

Format a balance for display with appropriate units

---

#### `format_solana_address`

Format a Solana address for display

---

#### `from_spl_pubkey`

Convert from SPL-compatible Pubkey to Solana SDK v3 Pubkey

---

#### `from_spl_token_pubkey`

Convert from SPL token Pubkey to Solana SDK v3 Pubkey

---

#### `generate_mint_keypair`

Generates new mint keypair for token creation

Returns a fresh keypair that can be used as the mint address for a new token.

---

#### `get_associated_token_address_v3`

Get associated token address with type conversion

---

#### `get_associated_token_address_with_program_id_v3`

Get associated token address with program ID and type conversion

---

#### `get_block_height`

Get the current block height from the Solana blockchain using an RPC client

This tool queries the Solana network to retrieve the most recent block height,
which represents the number of blocks that have been processed by the network.
Essential for checking network activity and determining transaction finality.
This is a read-only operation that uses ApplicationContext extensions instead of requiring transaction signing.

# Arguments

* `context` - The ApplicationContext containing the RPC client

# Returns

Returns the current block height as a `u64` representing the total number
of blocks processed by the Solana network since genesis.

# Errors

* `ToolError::Retriable` - When network connection issues occur or RPC timeouts

# Examples

```rust,ignore
use riglr_solana_tools::network::get_block_height;
use riglr_core::provider::ApplicationContext;
use riglr_config::Config;
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let config = Config::from_env();
let context = ApplicationContext::from_config(&config);

// Add Solana RPC client as an extension
let solana_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com"));
context.set_extension(solana_client);

let height = get_block_height(&context).await?;
println!("Current block height: {}", height);

// Use block height for transaction confirmation checks
if height > 150_000_000 {
    println!("Network has processed over 150M blocks");
}
# Ok(())
# }
```
Get the current block height from the Solana blockchain

This tool queries the Solana network to retrieve the most recent block height.

---

#### `get_block_height_tool`

Factory function to create a new instance of the tool

---

#### `get_jupiter_quote`

Get a quote from Jupiter for swapping tokens

This tool queries the Jupiter aggregator to find the best swap route between two SPL tokens
and returns detailed pricing information without executing any transaction. Jupiter aggregates
liquidity from multiple DEXs to provide optimal pricing.

# Arguments

* `input_mint` - Source token mint address to swap from
* `output_mint` - Destination token mint address to swap to
* `amount` - Input amount in token's smallest unit (e.g., lamports for SOL)
* `slippage_bps` - Maximum acceptable slippage in basis points (e.g., 50 = 0.5%)
* `only_direct_routes` - If true, only consider direct swap routes (no intermediate tokens)
* `jupiter_api_url` - Optional custom Jupiter API endpoint URL

# Returns

Returns `SwapQuote` containing:
- `input_mint` and `output_mint`: Token addresses
- `in_amount` and `out_amount`: Expected input and output amounts
- `other_amount_threshold`: Minimum output after slippage
- `price_impact_pct`: Price impact as percentage
- `route_plan`: Detailed routing through DEXs
- `context_slot` and `time_taken`: Quote freshness metadata

# Errors

* `ToolError::Permanent` - When token addresses are invalid or no routes exist
* `ToolError::Retriable` - When Jupiter API is temporarily unavailable

# Examples

```rust,ignore
use riglr_solana_tools::swap::get_jupiter_quote;
use riglr_core::provider::ApplicationContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let context = ApplicationContext::from_env();
// Get quote for swapping 1 SOL to USDC
let quote = get_jupiter_quote(
    "So11111111111111111111111111111111111111112".to_string(), // SOL mint
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
    1_000_000_000, // 1 SOL in lamports
    50, // 0.5% slippage
    false, // Allow multi-hop routes
    None, // Use default Jupiter API
    &context,
).await?;

println!("Quote: {} SOL -> {} USDC", quote.in_amount, quote.out_amount);
println!("Price impact: {:.2}%", quote.price_impact_pct);
# Ok(())
# }
```

---

#### `get_jupiter_quote_tool`

Factory function to create a new instance of the tool

---

#### `get_multiple_balances`

Get SOL balances for multiple addresses

This tool queries the Solana blockchain to retrieve SOL balances for multiple wallet addresses
in a batch operation. Each address is processed individually and results are collected.

# Arguments

* `addresses` - Vector of Solana wallet addresses to check (base58 encoded public keys)

# Returns

Returns `Vec<BalanceResult>` with balance information for each address that was successfully queried.
Each result contains the same fields as `get_sol_balance`.

# Errors

* `ToolError::Permanent` - When any individual address is invalid or signer context unavailable
* `ToolError::Retriable` - When network issues occur during any balance query

Note: This function fails fast - if any address query fails, the entire operation returns an error.

# Examples

```rust,ignore
use riglr_solana_tools::balance::get_multiple_balances;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let addresses = vec![
    "So11111111111111111111111111111111111111112".to_string(),
    "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
];

let balances = get_multiple_balances(&context, addresses).await?;
for balance in balances {
    println!("{}: {} SOL", balance.address, balance.sol);
}
# Ok(())
# }
```

---

#### `get_multiple_balances_tool`

Factory function to create a new instance of the tool

---

#### `get_pump_token_info`

Get token information from Pump.fun

This tool fetches current token information, price, and market data
for a specific token on the Pump.fun platform.

---

#### `get_pump_token_info_tool`

Factory function to create a new instance of the tool

---

#### `get_sol_balance`

Get SOL balance for a given address using an RPC client

This tool queries the Solana blockchain to retrieve the SOL balance for any wallet address.
The balance is returned in both lamports (smallest unit) and SOL (human-readable format).
This is a read-only operation that uses ApplicationContext extensions instead of requiring transaction signing.

# Arguments

* `address` - The Solana wallet address to check (base58 encoded public key)
* `context` - The ApplicationContext containing RPC client and configuration

# Returns

Returns `BalanceResult` containing:
- `address`: The queried wallet address
- `lamports`: Balance in lamports (1 SOL = 1,000,000,000 lamports)
- `sol`: Balance in SOL units as a floating-point number
- `formatted`: Human-readable balance string with 9 decimal places

# Errors

* `ToolError::Permanent` - When the address format is invalid or parsing fails
* `ToolError::Retriable` - When network connection issues occur (timeouts, connection errors)

# Examples

```rust,ignore
use riglr_solana_tools::balance::get_sol_balance;
use riglr_core::provider::ApplicationContext;
use riglr_config::Config;
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Set up ApplicationContext with Solana RPC client
let config = Config::from_env();
let context = ApplicationContext::from_config(&config);

// Add Solana RPC client as an extension
let solana_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com"));
context.set_extension(solana_client);

// Check SOL balance for a wallet using the tool directly
let balance = get_sol_balance(
    "So11111111111111111111111111111111111111112".to_string(),
    &context
).await?;

println!("Address: {}", balance.address);
println!("Balance: {} SOL ({} lamports)", balance.sol, balance.lamports);
println!("Formatted: {}", balance.formatted);
# Ok(())
# }
```

---

#### `get_sol_balance_tool`

Factory function to create a new instance of the tool

---

#### `get_spl_token_balance`

Get SPL token balance for a given owner and mint using an RPC client

This tool queries the Solana blockchain to retrieve the balance of a specific SPL token
for a given wallet address. It automatically finds the Associated Token Account (ATA)
and returns both raw and UI-adjusted amounts. This is a read-only operation that uses
ApplicationContext extensions instead of requiring transaction signing.

# Arguments

* `owner_address` - The wallet address that owns the tokens (base58 encoded)
* `mint_address` - The SPL token mint address (contract address)
* `context` - The ApplicationContext containing the RPC client

# Returns

Returns `TokenBalanceResult` containing:
- `owner_address`: The wallet address queried
- `mint_address`: The token mint address
- `raw_amount`: Balance in token's smallest unit (before decimal adjustment)
- `ui_amount`: Balance adjusted for token decimals
- `decimals`: Number of decimal places for the token
- `formatted`: Human-readable balance string

# Errors

* `ToolError::Permanent` - When addresses are invalid
* `ToolError::Retriable` - When network issues occur during balance retrieval

# Examples

```rust,ignore
use riglr_solana_tools::balance::get_spl_token_balance;
use riglr_core::provider::ApplicationContext;
use riglr_config::Config;
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let config = Config::from_env();
let context = ApplicationContext::from_config(&config);

// Add Solana RPC client as an extension
let solana_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com"));
context.set_extension(solana_client);

// Check USDC balance for a wallet
let balance = get_spl_token_balance(
    "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
    &context
).await?;

println!("Token balance: {} {}", balance.ui_amount, balance.mint_address);
println!("Raw amount: {} (decimals: {})", balance.raw_amount, balance.decimals);
# Ok(())
# }
```
Get SPL token balance for a given owner and mint

This tool queries the Solana blockchain to retrieve the balance of a specific SPL token.
This version uses dependency injection to get the RPC client from ApplicationContext.

---

#### `get_spl_token_balance_tool`

Factory function to create a new instance of the tool

---

#### `get_token_price`

Get the current price of a token pair

This tool fetches the current exchange rate between two SPL tokens by requesting
a small test quote from Jupiter. This provides real-time pricing without executing trades.

# Arguments

* `base_mint` - Token address to price (the token being quoted)
* `quote_mint` - Token address to price against (usually USDC or SOL)
* `jupiter_api_url` - Optional custom Jupiter API endpoint URL

# Returns

Returns `PriceInfo` containing:
- `base_mint` and `quote_mint`: Token addresses used
- `price`: Exchange rate (how much quote_mint per 1 base_mint)
- `price_impact_pct`: Price impact for small test trade

# Errors

* `ToolError::Permanent` - When tokens are invalid or no liquidity exists
* `ToolError::Retriable` - When Jupiter API is temporarily unavailable

# Examples

```rust,ignore
use riglr_solana_tools::swap::get_token_price;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Get SOL price in USDC
let price_info = get_token_price(
    "So11111111111111111111111111111111111111112".to_string(), // SOL mint
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
    None, // Use default Jupiter API
).await?;

println!("1 SOL = {} USDC", price_info.price);
println!("Price impact: {:.3}%", price_info.price_impact_pct);
# Ok(())
# }
```

---

#### `get_token_price_tool`

Factory function to create a new instance of the tool

---

#### `get_transaction_status`

Get transaction status by signature using an RPC client

This tool queries the Solana network to check the confirmation status of a transaction
using its signature. Essential for monitoring transaction progress and ensuring operations
have been confirmed by the network before proceeding with dependent actions.
This is a read-only operation that uses ApplicationContext extensions instead of requiring transaction signing.

# Arguments

* `signature` - The transaction signature to check (base58-encoded string)
* `context` - The ApplicationContext containing the RPC client

# Returns

Returns a `String` indicating the transaction status:
- `"finalized"` - Transaction is finalized and cannot be rolled back
- `"confirmed"` - Transaction is confirmed by supermajority of cluster
- `"processed"` - Transaction has been processed but may not be confirmed
- `"failed"` - Transaction failed due to an error
- `"not_found"` - Transaction signature not found (may not exist or be too old)

# Errors

* `ToolError::Permanent` - When signature format is invalid
* `ToolError::Retriable` - When network issues occur during status lookup

# Examples

```rust,ignore
use riglr_solana_tools::network::get_transaction_status;
use riglr_core::provider::ApplicationContext;
use riglr_config::Config;
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let config = Config::from_env();
let context = ApplicationContext::from_config(&config);

// Add Solana RPC client as an extension
let solana_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com"));
context.set_extension(solana_client);

let status = get_transaction_status(
    "5j7s88CkzQeE6EN5HiV7CqkYsL3x6PbJmSjYpJjm1J2v3z4x8K7b".to_string(),
    &context
).await?;

match status.as_str() {
    "finalized" => println!("✅ Transaction is finalized"),
    "confirmed" => println!("🔄 Transaction is confirmed"),
    "processed" => println!("⏳ Transaction is processed, awaiting confirmation"),
    "failed" => println!("❌ Transaction failed"),
    "not_found" => println!("🔍 Transaction not found"),
    _ => println!("Unknown status: {}", status),
}
# Ok(())
# }
```
Get transaction status by signature

This tool queries the Solana network to check the confirmation status of a transaction.

---

#### `get_transaction_status_tool`

Factory function to create a new instance of the tool

---

#### `get_trending_pump_tokens`

Get trending tokens on Pump.fun

This tool fetches the currently trending tokens on the Pump.fun platform.

---

#### `get_trending_pump_tokens_tool`

Factory function to create a new instance of the tool

---

#### `initialize_mint2_v3`

Initialize mint instruction with type conversion

---

#### `lamports_to_sol`

Convert lamports to SOL for display

---

#### `mint_to_v3`

Mint to instruction with type conversion

---

#### `parse_commitment`

Parse a commitment level string

---

#### `perform_jupiter_swap`

Execute a token swap using Jupiter

This tool executes an actual token swap using the Jupiter aggregator. It automatically
gets a fresh quote, constructs the swap transaction, signs it with the current signer context,
and submits it to the Solana network. The swap uses optimal routing across multiple DEXs.

# Arguments

* `input_mint` - Source token mint address to swap from
* `output_mint` - Destination token mint address to swap to
* `amount` - Input amount in token's smallest unit
* `slippage_bps` - Maximum acceptable slippage in basis points (e.g., 50 = 0.5%)
* `jupiter_api_url` - Optional custom Jupiter API endpoint URL
* `use_versioned_transaction` - Whether to use versioned transactions (recommended for lower fees)

# Returns

Returns `SwapResult` containing:
- `signature`: Transaction signature for tracking
- `input_mint` and `output_mint`: Token addresses involved
- `in_amount` and `out_amount`: Actual swap amounts
- `price_impact_pct`: Price impact percentage experienced
- `status`: Current transaction status (initially Pending)

# Errors

* `ToolError::Permanent` - When addresses invalid, signer unavailable, or swap construction fails
* `ToolError::Retriable` - When Jupiter API unavailable or network issues occur

# Examples

```rust,ignore
use riglr_solana_tools::swap::perform_jupiter_swap;
use riglr_core::SignerContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Swap 0.1 SOL to USDC
let result = perform_jupiter_swap(
    "So11111111111111111111111111111111111111112".to_string(), // SOL mint
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
    100_000_000, // 0.1 SOL in lamports
    100, // 1% slippage tolerance
    None, // Use default Jupiter API
    true, // Use versioned transactions
).await?;

println!("Swap executed! Signature: {}", result.signature);
println!("Swapped {} for {} tokens", result.in_amount, result.out_amount);
# Ok(())
# }
```

---

#### `perform_jupiter_swap_tool`

Factory function to create a new instance of the tool

---

#### `sell_pump_token`

Sell tokens on Pump.fun

This tool executes a sell order for a specific token on Pump.fun
with configurable slippage protection.

---

#### `sell_pump_token_tool`

Factory function to create a new instance of the tool

---

#### `send_transaction`

Send a transaction with default retry configuration

Convenience function that uses the default `TransactionConfig` for standard
retry behavior. Suitable for most transaction sending scenarios.

# Arguments

* `transaction` - The transaction to send
* `operation_name` - Human-readable operation name for logging

# Returns

Returns the transaction signature on success

# Examples

```rust,ignore
use riglr_solana_tools::utils::transaction::send_transaction;
use solana_sdk::transaction::Transaction;
use solana_system_interface::instruction as system_instruction;
use riglr_core::SignerContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let signer = SignerContext::current().await?;
let from = signer.pubkey().unwrap().parse()?;
let to = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".parse()?;

let instruction = system_instruction::transfer(&from, &to, 1000000);
let mut tx = Transaction::new_with_payer(&[instruction], Some(&from));

let signature = send_transaction(&mut tx, "SOL Transfer").await?;
println!("Transaction sent: {}", signature);
# Ok(())
# }
```

---

#### `send_transaction_with_retry`

Send a Solana transaction with retry logic and exponential backoff

This function centralizes all Solana transaction sending logic with robust
error handling, retry logic, and comprehensive logging. It automatically
classifies errors and applies appropriate retry strategies.

Uses SignerContext for secure multi-tenant operation.

# Arguments

* `transaction` - The transaction to send (will be mutably borrowed for signing)
* `config` - Configuration for retry behavior
* `operation_name` - Human-readable operation name for logging

# Returns

Returns `TransactionSubmissionResult` containing signature and attempt metadata

# Error Handling

Automatically retries on:
- Network timeouts and connection issues
- RPC rate limiting (with longer backoff)
- Temporary blockchain congestion

Does NOT retry on:
- Insufficient funds
- Invalid signatures or accounts
- Program execution errors
- Invalid transaction structure

# Examples

```rust,ignore
use riglr_solana_tools::utils::transaction::{send_transaction_with_retry, TransactionConfig};
use solana_sdk::transaction::Transaction;
use solana_system_interface::instruction as system_instruction;
use riglr_core::SignerContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let signer = SignerContext::current().await?;
let from = signer.pubkey().unwrap().parse()?;
let to = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".parse()?;

let instruction = system_instruction::transfer(&from, &to, 1000000);
let mut tx = Transaction::new_with_payer(&[instruction], Some(&from));

let config = TransactionConfig::default();
let result = send_transaction_with_retry(
    &mut tx,
    &config,
    "SOL Transfer"
).await?;

println!("Transaction sent: {} (attempts: {})",
         result.signature, result.attempts);
# Ok(())
# }
```

---

#### `sol_to_lamports`

Convert SOL to lamports

---

#### `spl_token_transfer_v3`

Create SPL token transfer instruction with type conversion

---

#### `string_to_pubkey`

Convert a string to a Solana Pubkey with better error handling

---

#### `to_spl_pubkey`

Convert from Solana SDK v3 Pubkey to SPL-compatible Pubkey

SPL libraries (spl-token v8, spl-associated-token-account v7) are built against
Solana SDK v2 and have their own bundled Pubkey type. This function converts
between them by serializing to string and parsing back.

---

#### `transfer_sol`

Transfer SOL from one account to another

This tool creates, signs, and executes a SOL transfer transaction using the current signer context.
The transaction includes optional memo and priority fee support for faster confirmation.

# Arguments

* `to_address` - Recipient wallet address (base58 encoded public key)
* `amount_sol` - Amount to transfer in SOL (e.g., 0.001 for 1,000,000 lamports)
* `memo` - Optional memo to include in the transaction for record keeping
* `priority_fee` - Optional priority fee in microlamports for faster processing

# Returns

Returns `TransactionResult` containing:
- `signature`: Transaction signature hash
- `from`: Sender address from signer context
- `to`: Recipient address
- `amount`: Transfer amount in lamports
- `amount_display`: Human-readable amount in SOL
- `status`: Transaction confirmation status
- `memo`: Included memo (if any)

# Errors

* `ToolError::Permanent` - When amount is non-positive, addresses are invalid, or signer unavailable
* `ToolError::Permanent` - When transaction signing or submission fails

# Examples

```rust,ignore
use riglr_solana_tools::transaction::transfer_sol;
use riglr_core::SignerContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Transfer 0.001 SOL with a memo
let result = transfer_sol(
    "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
    0.001, // 0.001 SOL
    Some("Payment for services".to_string()),
    Some(5000), // 5000 microlamports priority fee
).await?;

println!("Transfer completed! Signature: {}", result.signature);
println!("Sent {} from {} to {}", result.amount_display, result.from, result.to);
# Ok(())
# }
```

---

#### `transfer_sol_tool`

Factory function to create a new instance of the tool

---

#### `transfer_spl_token`

Transfer SPL tokens from one account to another

This tool creates, signs, and executes an SPL token transfer transaction. It automatically
handles Associated Token Account (ATA) creation if needed and supports any SPL token.

# Arguments

* `to_address` - Recipient wallet address (base58 encoded public key)
* `mint_address` - SPL token mint address (contract address)
* `amount` - Amount to transfer in token's smallest unit (before decimal adjustment)
* `decimals` - Number of decimal places for the token (e.g., 6 for USDC, 9 for most tokens)
* `create_ata_if_needed` - Whether to create recipient's ATA if it doesn't exist

# Returns

Returns `TokenTransferResult` containing transaction details and amount information
in both raw and UI-adjusted formats.

# Errors

* `ToolError::Permanent` - When addresses are invalid, signer unavailable, or transaction fails

# Examples

```rust,ignore
use riglr_solana_tools::transaction::transfer_spl_token;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Transfer 1 USDC (6 decimals)
let result = transfer_spl_token(
    "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
    1_000_000, // 1 USDC in microunits
    6, // USDC has 6 decimals
    true, // Create recipient ATA if needed
).await?;

println!("Token transfer completed! Signature: {}", result.signature);
println!("Transferred {} tokens", result.ui_amount);
# Ok(())
# }
```

---

#### `transfer_spl_token_tool`

Factory function to create a new instance of the tool

---

#### `validate_address`

Check if a Solana address is valid

Validates that the provided string is a valid base58-encoded Solana public key.

# Arguments

* `address` - The address string to validate

# Returns

Returns the parsed `Pubkey` if valid, or a `SolanaToolError::InvalidAddress` if invalid.

# Examples

```rust,ignore
use riglr_solana_tools::utils::validation::validate_address;

// Valid address
let pubkey = validate_address("11111111111111111111111111111111")?;

// Invalid address will return error
assert!(validate_address("invalid").is_err());
```

---

#### `validate_rpc_url`

Validate that an RPC URL is reachable

---

#### `validate_solana_address`

Helper function to validate Solana addresses

---

### Type Aliases

#### `Result`

Result type alias for Solana tool operations.

**Type:** `<T, >`

---

#### `TransactionConfig`

Configuration for transaction retry behavior
This is now a simple wrapper around riglr_core::retry::RetryConfig

**Type:** ``

---

### Constants

#### `JUPITER_API_URL`

Environment variable for Jupiter API URL

**Type:** `&str`

---

#### `VERSION`

Current version of riglr-solana-tools

**Type:** `&str`

---
