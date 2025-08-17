# riglr-solana-tools API Reference

Comprehensive API documentation for the `riglr-solana-tools` crate.

## Table of Contents

### Tools

- [`analyze_pump_transaction`](#analyze_pump_transaction)
- [`buy_pump_token`](#buy_pump_token)
- [`create_spl_token_mint`](#create_spl_token_mint)
- [`deploy_pump_token`](#deploy_pump_token)
- [`get_block_height`](#get_block_height)
- [`get_jupiter_quote`](#get_jupiter_quote)
- [`get_multiple_balances`](#get_multiple_balances)
- [`get_pump_token_info`](#get_pump_token_info)
- [`get_sol_balance`](#get_sol_balance)
- [`get_spl_token_balance`](#get_spl_token_balance)
- [`get_token_price`](#get_token_price)
- [`get_transaction_status`](#get_transaction_status)
- [`get_trending_pump_tokens`](#get_trending_pump_tokens)
- [`perform_jupiter_swap`](#perform_jupiter_swap)
- [`sell_pump_token`](#sell_pump_token)
- [`transfer_sol`](#transfer_sol)
- [`transfer_spl_token`](#transfer_spl_token)

### Constants

- [`VERSION`](#version)

### Structs

- [`BalanceResult`](#balanceresult)
- [`CreateMintResult`](#createmintresult)
- [`JupiterConfig`](#jupiterconfig)
- [`LocalSolanaSigner`](#localsolanasigner)
- [`PriceInfo`](#priceinfo)
- [`PumpConfig`](#pumpconfig)
- [`PumpTokenInfo`](#pumptokeninfo)
- [`PumpTradeAnalysis`](#pumptradeanalysis)
- [`PumpTradeResult`](#pumptraderesult)
- [`RoutePlanStep`](#routeplanstep)
- [`SolanaClient`](#solanaclient)
- [`SolanaConfig`](#solanaconfig)
- [`SwapInfo`](#swapinfo)
- [`SwapQuote`](#swapquote)
- [`SwapResult`](#swapresult)
- [`TokenBalanceResult`](#tokenbalanceresult)
- [`TokenTransferResult`](#tokentransferresult)
- [`TransactionConfig`](#transactionconfig)
- [`TransactionResult`](#transactionresult)
- [`TransactionSubmissionResult`](#transactionsubmissionresult)

### Enums

- [`PermanentError`](#permanenterror)
- [`PumpTradeType`](#pumptradetype)
- [`RateLimitError`](#ratelimiterror)
- [`RetryableError`](#retryableerror)
- [`SolanaToolError`](#solanatoolerror)
- [`TransactionErrorType`](#transactionerrortype)
- [`TransactionStatus`](#transactionstatus)

### Functions (error)

- [`classify_transaction_error`](#classify_transaction_error)
- [`is_rate_limited`](#is_rate_limited)
- [`is_rate_limited`](#is_rate_limited)
- [`is_retriable`](#is_retriable)
- [`is_retryable`](#is_retryable)
- [`retry_delay`](#retry_delay)

### Functions (pump)

- [`create_token_with_mint_keypair`](#create_token_with_mint_keypair)
- [`generate_mint_keypair`](#generate_mint_keypair)

### Functions (client)

- [`call_rpc`](#call_rpc)
- [`devnet`](#devnet)
- [`from_signer`](#from_signer)
- [`get_balance`](#get_balance)
- [`get_block_height`](#get_block_height)
- [`get_cluster_info`](#get_cluster_info)
- [`get_latest_blockhash`](#get_latest_blockhash)
- [`get_recent_transactions_for_token`](#get_recent_transactions_for_token)
- [`get_signature_statuses`](#get_signature_statuses)
- [`get_token_account_balance`](#get_token_account_balance)
- [`get_token_accounts_by_owner`](#get_token_accounts_by_owner)
- [`get_token_balance`](#get_token_balance)
- [`get_transaction`](#get_transaction)
- [`get_transaction_with_meta`](#get_transaction_with_meta)
- [`has_signer`](#has_signer)
- [`is_connected`](#is_connected)
- [`mainnet`](#mainnet)
- [`new`](#new)
- [`require_signer`](#require_signer)
- [`send_and_confirm_transaction`](#send_and_confirm_transaction)
- [`send_transaction`](#send_transaction)
- [`signer`](#signer)
- [`testnet`](#testnet)
- [`with_commitment`](#with_commitment)
- [`with_rpc_url`](#with_rpc_url)
- [`with_signer`](#with_signer)
- [`with_signer_from_bytes`](#with_signer_from_bytes)

### Functions (local)

- [`from_seed_phrase`](#from_seed_phrase)
- [`keypair`](#keypair)
- [`new`](#new)
- [`rpc_url`](#rpc_url)

### Functions (keypair)

- [`generate_mint_keypair`](#generate_mint_keypair)

### Functions (transaction)

- [`create_token_with_mint_keypair`](#create_token_with_mint_keypair)
- [`execute_solana_transaction`](#execute_solana_transaction)
- [`send_transaction`](#send_transaction)
- [`send_transaction_with_retry`](#send_transaction_with_retry)

### Functions (validation)

- [`validate_address`](#validate_address)

### Functions (config)

- [`get_rpc_url`](#get_rpc_url)

## Tools

### analyze_pump_transaction

**Source**: `src/pump.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn analyze_pump_transaction( signature: String, user_address: String, token_mint: String, ) -> Result<PumpTradeAnalysis, ToolError>
```

Analyze a Pump.fun trade transaction to extract actual amounts and price

This tool parses a completed Pump.fun transaction to determine the actual
token amounts, SOL amounts, and price per token that were executed.
Separate from action tools following riglr separation of concerns pattern.

---

### buy_pump_token

**Source**: `src/pump.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn buy_pump_token( token_mint: String, sol_amount: f64, slippage_percent: Option<f64>, ) -> Result<PumpTradeResult, ToolError>
```

Buy tokens on Pump.fun

This tool executes a buy order for a specific token on Pump.fun
with configurable slippage protection.

---

### create_spl_token_mint

**Source**: `src/transaction.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn create_spl_token_mint( decimals: u8, initial_supply: u64, freezable: bool, ) -> Result<CreateMintResult, ToolError>
```

Create a new SPL token mint

This tool creates a new SPL token mint account on the Solana blockchain with specified
configuration parameters. The mint authority is set to the current signer, enabling
future token supply management operations.

**Note**: This function is currently a placeholder and will return an implementation
pending error. A full implementation would create the mint account, set authorities,
and optionally mint initial tokens to the creator.

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

* `ToolError::Permanent` - Currently returns "Implementation pending" error
* Future implementation would include:
- Invalid decimals value (must be 0-9)
- Insufficient SOL balance for rent and fees
- Network connection issues

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

### deploy_pump_token

**Source**: `src/pump.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn deploy_pump_token( name: String, symbol: String, description: String, image_url: Option<String>, initial_buy_sol: Option<f64>, ) -> Result<PumpTokenInfo, ToolError>
```

Deploy a new token on Pump.fun

This tool creates and deploys a new meme token on the Pump.fun platform.
Optionally performs an initial buy to bootstrap liquidity.

---

### get_block_height

**Source**: `src/network.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_block_height() -> Result<u64, ToolError>
```

Get the current block height from the Solana blockchain

This tool queries the Solana network to retrieve the most recent block height,
which represents the number of blocks that have been processed by the network.
Essential for checking network activity and determining transaction finality.

# Returns

Returns the current block height as a `u64` representing the total number
of blocks processed by the Solana network since genesis.

# Errors

* `ToolError::Permanent` - When no signer context is available
* `ToolError::Retriable` - When network connection issues occur or RPC timeouts

# Examples

```rust,ignore
use riglr_solana_tools::network::get_block_height;
use riglr_core::SignerContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let height = get_block_height().await?;
println!("Current block height: {}", height);

// Use block height for transaction confirmation checks
if height > 150_000_000 {
println!("Network has processed over 150M blocks");
}
# Ok(())
# }
```

---

### get_jupiter_quote

**Source**: `src/swap.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_jupiter_quote( input_mint: String, output_mint: String, amount: u64, slippage_bps: u16, only_direct_routes: bool, jupiter_api_url: Option<String>, ) -> Result<SwapQuote, ToolError>
```

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

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Get quote for swapping 1 SOL to USDC
let quote = get_jupiter_quote(
"So11111111111111111111111111111111111111112".to_string(), // SOL mint
"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
1_000_000_000, // 1 SOL in lamports
50, // 0.5% slippage
false, // Allow multi-hop routes
None, // Use default Jupiter API
).await?;

println!("Quote: {} SOL -> {} USDC", quote.in_amount, quote.out_amount);
println!("Price impact: {:.2}%", quote.price_impact_pct);
# Ok(())
# }
```

---

### get_multiple_balances

**Source**: `src/balance.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_multiple_balances( addresses: Vec<String>, ) -> Result<Vec<BalanceResult>, ToolError>
```

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

let balances = get_multiple_balances(addresses).await?;
for balance in balances {
println!("{}: {} SOL", balance.address, balance.sol);
}
# Ok(())
# }
```

---

### get_pump_token_info

**Source**: `src/pump.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_pump_token_info(token_mint: String) -> Result<PumpTokenInfo, ToolError>
```

Get token information from Pump.fun

This tool fetches current token information, price, and market data
for a specific token on the Pump.fun platform.

---

### get_sol_balance

**Source**: `src/balance.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_sol_balance(address: String) -> Result<BalanceResult, ToolError>
```

Get SOL balance for a given address

This tool queries the Solana blockchain to retrieve the SOL balance for any wallet address.
The balance is returned in both lamports (smallest unit) and SOL (human-readable format).

# Arguments

* `address` - The Solana wallet address to check (base58 encoded public key)

# Returns

Returns `BalanceResult` containing:
- `address`: The queried wallet address
- `lamports`: Balance in lamports (1 SOL = 1,000,000,000 lamports)
- `sol`: Balance in SOL units as a floating-point number
- `formatted`: Human-readable balance string with 9 decimal places

# Errors

* `ToolError::Permanent` - When the address format is invalid or parsing fails
* `ToolError::Retriable` - When network connection issues occur (timeouts, connection errors)
* `ToolError::Permanent` - When no signer context is available

# Examples

```rust,ignore
use riglr_solana_tools::balance::get_sol_balance;
use riglr_core::SignerContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Check SOL balance for a wallet
let balance = get_sol_balance(
"So11111111111111111111111111111111111111112".to_string()
).await?;

println!("Address: {}", balance.address);
println!("Balance: {} SOL ({} lamports)", balance.sol, balance.lamports);
println!("Formatted: {}", balance.formatted);
# Ok(())
# }
```

---

### get_spl_token_balance

**Source**: `src/balance.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_spl_token_balance( owner_address: String, mint_address: String, ) -> Result<TokenBalanceResult, ToolError>
```

Get SPL token balance for a given owner and mint

This tool queries the Solana blockchain to retrieve the balance of a specific SPL token
for a given wallet address. It automatically finds the Associated Token Account (ATA)
and returns both raw and UI-adjusted amounts.

# Arguments

* `owner_address` - The wallet address that owns the tokens (base58 encoded)
* `mint_address` - The SPL token mint address (contract address)

# Returns

Returns `TokenBalanceResult` containing:
- `owner_address`: The wallet address queried
- `mint_address`: The token mint address
- `raw_amount`: Balance in token's smallest unit (before decimal adjustment)
- `ui_amount`: Balance adjusted for token decimals
- `decimals`: Number of decimal places for the token
- `formatted`: Human-readable balance string

# Errors

* `ToolError::Permanent` - When addresses are invalid or context unavailable
* `ToolError::Retriable` - When network issues occur during balance retrieval

# Examples

```rust,ignore
use riglr_solana_tools::balance::get_spl_token_balance;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Check USDC balance for a wallet
let balance = get_spl_token_balance(
"9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
).await?;

println!("Token balance: {} {}", balance.ui_amount, balance.mint_address);
println!("Raw amount: {} (decimals: {})", balance.raw_amount, balance.decimals);
# Ok(())
# }
```

---

### get_token_price

**Source**: `src/swap.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_token_price( base_mint: String, quote_mint: String, jupiter_api_url: Option<String>, ) -> Result<PriceInfo, ToolError>
```

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

### get_transaction_status

**Source**: `src/network.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_transaction_status(signature: String) -> Result<String, ToolError>
```

Get transaction status by signature

This tool queries the Solana network to check the confirmation status of a transaction
using its signature. Essential for monitoring transaction progress and ensuring operations
have been confirmed by the network before proceeding with dependent actions.

# Arguments

* `signature` - The transaction signature to check (base58-encoded string)

# Returns

Returns a `String` indicating the transaction status:
- `"finalized"` - Transaction is finalized and cannot be rolled back
- `"confirmed"` - Transaction is confirmed by supermajority of cluster
- `"processed"` - Transaction has been processed but may not be confirmed
- `"failed"` - Transaction failed due to an error
- `"not_found"` - Transaction signature not found (may not exist or be too old)

# Errors

* `ToolError::Permanent` - When signature format is invalid or signer context unavailable
* `ToolError::Retriable` - When network issues occur during status lookup

# Examples

```rust,ignore
use riglr_solana_tools::network::get_transaction_status;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let status = get_transaction_status(
"5j7s88CkzQeE6EN5HiV7CqkYsL3x6PbJmSjYpJjm1J2v3z4x8K7b".to_string()
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

---

### get_trending_pump_tokens

**Source**: `src/pump.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_trending_pump_tokens(limit: Option<u32>) -> Result<Vec<PumpTokenInfo>, ToolError>
```

Get trending tokens on Pump.fun

This tool fetches the currently trending tokens on the Pump.fun platform.

---

### perform_jupiter_swap

**Source**: `src/swap.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn perform_jupiter_swap( input_mint: String, output_mint: String, amount: u64, slippage_bps: u16, jupiter_api_url: Option<String>, use_versioned_transaction: bool, ) -> Result<SwapResult, ToolError>
```

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

### sell_pump_token

**Source**: `src/pump.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn sell_pump_token( token_mint: String, token_amount: u64, slippage_percent: Option<f64>, ) -> Result<PumpTradeResult, ToolError>
```

Sell tokens on Pump.fun

This tool executes a sell order for a specific token on Pump.fun
with configurable slippage protection.

---

### transfer_sol

**Source**: `src/transaction.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn transfer_sol( to_address: String, amount_sol: f64, memo: Option<String>, priority_fee: Option<u64>, ) -> Result<TransactionResult, ToolError>
```

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

### transfer_spl_token

**Source**: `src/transaction.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn transfer_spl_token( to_address: String, mint_address: String, amount: u64, decimals: u8, create_ata_if_needed: bool, ) -> Result<TokenTransferResult, ToolError>
```

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

## Constants

### VERSION

**Source**: `src/lib.rs`

```rust
const VERSION: &str
```

Current version of riglr-solana-tools

---

## Structs

### BalanceResult

**Source**: `src/balance.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct BalanceResult { /// The Solana wallet address that was queried pub address: String, /// Balance in lamports (smallest unit)
```

Result structure for balance queries

Contains balance information for a Solana address including both raw lamports
and human-readable SOL amounts.

---

### CreateMintResult

**Source**: `src/transaction.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct CreateMintResult { /// Transaction signature pub signature: String, /// Address of the newly created token mint pub mint_address: String, /// Mint authority address pub authority: String, /// Number of decimal places for the token pub decimals: u8, /// Initial supply of tokens minted pub initial_supply: u64, /// Whether token accounts can be frozen pub freezable: bool, }
```

Result of creating a new SPL token mint

---

### JupiterConfig

**Source**: `src/swap.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct JupiterConfig { /// Jupiter API base URL pub api_url: String, /// Maximum acceptable slippage in basis points (e.g., 50 = 0.5%)
```

Jupiter API configuration

---

### LocalSolanaSigner

**Source**: `signer/local.rs`

```rust
pub struct LocalSolanaSigner { keypair: solana_sdk::signature::Keypair, rpc_url: String, client: Arc<RpcClient>, }
```

A local Solana signer implementation that holds a keypair and RPC client.
This implementation is suitable for development, testing, and scenarios where
private keys can be safely managed in memory.

---

### PriceInfo

**Source**: `src/swap.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct PriceInfo { /// Token being priced (the asset)
```

Token price information

---

### PumpConfig

**Source**: `src/pump.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct PumpConfig { /// Pump.fun API base URL pub api_url: String, /// Default slippage tolerance for trades (in basis points)
```

Pump.fun API configuration

---

### PumpTokenInfo

**Source**: `src/pump.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct PumpTokenInfo { /// Token mint address pub mint_address: String, /// Token name pub name: String, /// Token symbol pub symbol: String, /// Token description pub description: String, /// Optional image URL pub image_url: Option<String>, /// Current market cap in lamports pub market_cap: Option<u64>, /// Current price in SOL pub price_sol: Option<f64>, /// Transaction signature for token creation pub creation_signature: Option<String>, /// Creator's public key pub creator: String, /// Transaction signature for initial buy (if any)
```

Information about a Pump.fun token

---

### PumpTradeAnalysis

**Source**: `src/pump.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct PumpTradeAnalysis { /// Transaction signature that was analyzed pub signature: String, /// User address involved in the trade pub user_address: String, /// Token mint address pub token_mint: String, /// Actual token amount involved (in raw units)
```

Result of analyzing a Pump.fun trade transaction

---

### PumpTradeResult

**Source**: `src/pump.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct PumpTradeResult { /// Transaction signature pub signature: String, /// Token mint address pub token_mint: String, /// SOL amount involved in the trade pub sol_amount: f64, /// Token amount (for sells or when known)
```

Result of a Pump.fun trade operation

---

### RoutePlanStep

**Source**: `src/swap.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct RoutePlanStep { /// Detailed swap information for this step pub swap_info: SwapInfo, /// Percentage of input amount for this route step pub percent: u8, }
```

Route plan step in Jupiter quote

---

### SolanaClient

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct SolanaClient { /// Native Solana RPC client pub rpc_client: Arc<RpcClient>, /// HTTP client for custom requests pub http_client: Client, /// Configuration pub config: SolanaConfig, /// Optional signer for transactions pub signer: Option<Arc<Keypair>>, }
```

A client for interacting with the Solana blockchain

---

### SolanaConfig

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct SolanaConfig { /// RPC URL for the Solana cluster pub rpc_url: String, /// Commitment level for transactions pub commitment: CommitmentLevel, /// Request timeout pub timeout: Duration, /// Whether to skip preflight checks pub skip_preflight: bool, }
```

Configuration for Solana RPC client

---

### SwapInfo

**Source**: `src/swap.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SwapInfo { /// AMM program public key performing the swap pub amm_key: String, /// Human-readable DEX name (e.g., "Raydium", "Orca")
```

Swap information for a route step

---

### SwapQuote

**Source**: `src/swap.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SwapQuote { /// Source token mint address pub input_mint: String, /// Destination token mint address pub output_mint: String, /// Input amount pub in_amount: u64, /// Expected output amount pub out_amount: u64, /// Minimum output amount after slippage pub other_amount_threshold: u64, /// Price impact percentage pub price_impact_pct: f64, /// Detailed routing plan pub route_plan: Vec<RoutePlanStep>, /// Context slot for the quote pub context_slot: Option<u64>, /// Time taken to compute quote pub time_taken: Option<f64>, }
```

Result of a swap quote from Jupiter

---

### SwapResult

**Source**: `src/swap.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SwapResult { /// Transaction signature pub signature: String, /// Source token mint address pub input_mint: String, /// Destination token mint address pub output_mint: String, /// Input amount pub in_amount: u64, /// Expected output amount pub out_amount: u64, /// Price impact percentage pub price_impact_pct: f64, /// Transaction status pub status: TransactionStatus, /// Idempotency key if provided pub idempotency_key: Option<String>, }
```

Result of a swap execution

---

### TokenBalanceResult

**Source**: `src/balance.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TokenBalanceResult { /// The wallet address that owns the tokens pub owner_address: String, /// The SPL token mint address (contract address)
```

Result structure for SPL token balance queries

Contains balance information for a specific SPL token including both raw amounts
and decimal-adjusted values.

---

### TokenTransferResult

**Source**: `src/transaction.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TokenTransferResult { /// Transaction signature pub signature: String, /// Sender address pub from: String, /// Recipient address pub to: String, /// SPL token mint address pub mint: String, /// Raw amount transferred pub amount: u64, /// UI-formatted amount (adjusted for decimals)
```

Result of an SPL token transfer transaction

---

### TransactionConfig

**Source**: `utils/transaction.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct TransactionConfig { /// Maximum number of retry attempts pub max_retries: u32, /// Initial retry delay in milliseconds pub base_delay_ms: u64, /// Maximum retry delay in milliseconds pub max_delay_ms: u64, /// Multiplier for exponential backoff pub backoff_multiplier: f64, /// Whether to use jitter to avoid thundering herd pub use_jitter: bool, }
```

Configuration for transaction retry behavior

---

### TransactionResult

**Source**: `src/transaction.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TransactionResult { /// Transaction signature pub signature: String, /// Sender address pub from: String, /// Recipient address pub to: String, /// Amount transferred in lamports pub amount: u64, /// Human-readable amount display pub amount_display: String, /// Transaction status pub status: TransactionStatus, /// Optional memo included with the transaction pub memo: Option<String>, /// Idempotency key if provided pub idempotency_key: Option<String>, }
```

Result of a SOL transfer transaction

---

### TransactionSubmissionResult

**Source**: `utils/transaction.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct TransactionSubmissionResult { /// Transaction signature pub signature: String, /// Number of attempts made pub attempts: u32, /// Total time taken for all attempts pub total_duration_ms: u64, /// Whether transaction was confirmed (false for non-blocking sending)
```

Result of a transaction submission

---

## Enums

### PermanentError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq)]
```

```rust
pub enum PermanentError { /// Insufficient funds for transaction InsufficientFunds, /// Invalid signature provided InvalidSignature, /// Invalid account referenced InvalidAccount, /// Program execution error InstructionError, /// Invalid transaction structure InvalidTransaction, /// Duplicate transaction DuplicateTransaction, }
```

Permanent errors that should not be retried

**Variants**:

- `InsufficientFunds`
- `InvalidSignature`
- `InvalidAccount`
- `InstructionError`
- `InvalidTransaction`
- `DuplicateTransaction`

---

### PumpTradeType

**Source**: `src/pump.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub enum PumpTradeType { /// Buy tokens with SOL Buy, /// Sell tokens for SOL Sell, }
```

Type of trade on Pump.fun

**Variants**:

- `Buy`
- `Sell`

---

### RateLimitError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq)]
```

```rust
pub enum RateLimitError { /// Standard RPC rate limiting RpcRateLimit, /// Too many requests error TooManyRequests, }
```

Rate limiting errors with special handling

**Variants**:

- `RpcRateLimit`
- `TooManyRequests`

---

### RetryableError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq)]
```

```rust
pub enum RetryableError { /// Network connectivity issues NetworkConnectivity, /// RPC service temporary unavailability TemporaryRpcFailure, /// Blockchain congestion NetworkCongestion, /// Transaction pool full TransactionPoolFull, }
```

Errors that can be retried with appropriate backoff

**Variants**:

- `NetworkConnectivity`
- `TemporaryRpcFailure`
- `NetworkCongestion`
- `TransactionPoolFull`

---

### SolanaToolError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
#[allow(clippy::result_large_err)]
#[allow(clippy::large_enum_variant)]
```

```rust
pub enum SolanaToolError { /// Core tool error #[error("Core tool error: {0}")] ToolError(#[from] ToolError), /// Signer context error #[error("Signer context error: {0}")] SignerError(#[from] SignerError), /// RPC client error #[error("RPC error: {0}")] Rpc(String), /// Solana client error #[error("Solana client error: {0}")] SolanaClient(Box<ClientError>), /// Invalid address format #[error("Invalid address: {0}")] InvalidAddress(String), /// Invalid key format #[error("Invalid key: {0}")] InvalidKey(String), /// Invalid signature format #[error("Invalid signature: {0}")] InvalidSignature(String), /// Transaction failed #[error("Transaction error: {0}")] Transaction(String), /// Insufficient funds for operation #[error("Insufficient funds for operation")] InsufficientFunds, /// Invalid token mint #[error("Invalid token mint: {0}")] InvalidTokenMint(String), /// Serialization error #[error("Serialization error: {0}")] Serialization(#[from] serde_json::Error), /// HTTP request error #[error("HTTP error: {0}")] Http(#[from] reqwest::Error), /// Core riglr error #[error("Core error: {0}")] Core(#[from] riglr_core::CoreError), /// Generic error #[error("Solana tool error: {0}")] Generic(String), }
```

Main error type for Solana tool operations.

**Variants**:

- `ToolError(#[from] ToolError)`
- `SignerError(#[from] SignerError)`
- `Rpc(String)`
- `SolanaClient(Box<ClientError>)`
- `InvalidAddress(String)`
- `InvalidKey(String)`
- `InvalidSignature(String)`
- `Transaction(String)`
- `InsufficientFunds`
- `InvalidTokenMint(String)`
- `Serialization(#[from] serde_json::Error)`
- `Http(#[from] reqwest::Error)`
- `Core(#[from] riglr_core::CoreError)`
- `Generic(String)`

---

### TransactionErrorType

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq)]
```

```rust
pub enum TransactionErrorType { /// Errors that can be retried with appropriate backoff Retryable(RetryableError), /// Errors that represent permanent failures and should not be retried Permanent(PermanentError), /// Rate limiting errors that require special handling with delays RateLimited(RateLimitError), /// Unknown error types that don't fit other categories Unknown(String), }
```

Structured classification of transaction errors for intelligent retry logic

**Variants**:

- `Retryable(RetryableError)`
- `Permanent(PermanentError)`
- `RateLimited(RateLimitError)`
- `Unknown(String)`

---

### TransactionStatus

**Source**: `src/transaction.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub enum TransactionStatus { /// Transaction is pending confirmation Pending, /// Transaction is confirmed Confirmed, /// Transaction is finalized Finalized, /// Transaction failed Failed(String), }
```

Transaction status

**Variants**:

- `Pending`
- `Confirmed`
- `Finalized`
- `Failed(String)`

---

## Functions (error)

### classify_transaction_error

**Source**: `src/error.rs`

```rust
pub fn classify_transaction_error(error: &ClientError) -> TransactionErrorType
```

Classify a Solana ClientError into a structured transaction error type

This function provides intelligent error classification based on the actual
error types from the Solana client, rather than brittle string matching.
It handles the most common error scenarios and provides appropriate
retry guidance.

---

### is_rate_limited

**Source**: `src/error.rs`

```rust
pub fn is_rate_limited(&self) -> bool
```

Check if this error is rate-limited.

---

### is_rate_limited

**Source**: `src/error.rs`

```rust
pub fn is_rate_limited(&self) -> bool
```

Check if this is a rate limiting error (special case of retryable)

---

### is_retriable

**Source**: `src/error.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if this error is retriable.

---

### is_retryable

**Source**: `src/error.rs`

```rust
pub fn is_retryable(&self) -> bool
```

Check if this error type is retryable

---

### retry_delay

**Source**: `src/error.rs`

```rust
pub fn retry_delay(&self) -> Option<std::time::Duration>
```

Get appropriate retry delay for rate-limited errors.

---

## Functions (pump)

### create_token_with_mint_keypair

**Source**: `src/pump.rs`

```rust
pub async fn create_token_with_mint_keypair( instructions: Vec<Instruction>, _mint_keypair: &Keypair, ) -> Result<String, ToolError>
```

Creates properly signed Solana transaction with mint keypair

This function creates a transaction with the given instructions and signs it
using both the payer from signer context and the provided mint keypair.

---

### generate_mint_keypair

**Source**: `src/pump.rs`

```rust
pub fn generate_mint_keypair() -> Keypair
```

Generates new mint keypair for token creation

Returns a fresh keypair that can be used as the mint address for a new token.

---

## Functions (client)

### call_rpc

**Source**: `src/client.rs`

```rust
pub async fn call_rpc( &self, method: &str, params: serde_json::Value, ) -> Result<serde_json::Value>
```

Make a custom RPC call

---

### devnet

**Source**: `src/client.rs`

```rust
pub fn devnet() -> Self
```

Create a new Solana client with devnet configuration

---

### from_signer

**Source**: `src/client.rs`

```rust
pub fn from_signer(signer: &dyn riglr_core::signer::TransactionSigner) -> Result<Self>
```

Create a SolanaClient from a TransactionSigner

---

### get_balance

**Source**: `src/client.rs`

```rust
pub async fn get_balance(&self, address: &str) -> Result<u64>
```

Get SOL balance for an address

---

### get_block_height

**Source**: `src/client.rs`

```rust
pub async fn get_block_height(&self) -> Result<u64>
```

Get the current block height

---

### get_cluster_info

**Source**: `src/client.rs`

```rust
pub async fn get_cluster_info(&self) -> Result<serde_json::Value>
```

Get cluster info

---

### get_latest_blockhash

**Source**: `src/client.rs`

```rust
pub async fn get_latest_blockhash(&self) -> Result<String>
```

Get latest blockhash

---

### get_recent_transactions_for_token

**Source**: `src/client.rs`

```rust
pub async fn get_recent_transactions_for_token( &self, _token_address: &str, limit: usize, ) -> Result<Vec<EncodedTransactionWithStatusMeta>>
```

Get recent transactions for a token (simplified implementation)

---

### get_signature_statuses

**Source**: `src/client.rs`

```rust
pub async fn get_signature_statuses( &self, signatures: &[String], ) -> Result<Vec<Option<solana_transaction_status::TransactionStatus>>>
```

Get transaction status by signature

---

### get_token_account_balance

**Source**: `src/client.rs`

```rust
pub async fn get_token_account_balance( &self, token_account: &str, ) -> Result<solana_account_decoder::parse_token::UiTokenAmount>
```

Get token account balance

---

### get_token_accounts_by_owner

**Source**: `src/client.rs`

```rust
pub async fn get_token_accounts_by_owner( &self, owner: &str, mint: Option<&str>, ) -> Result<Vec<solana_client::rpc_response::RpcKeyedAccount>>
```

Get token accounts owned by the given address

---

### get_token_balance

**Source**: `src/client.rs`

```rust
pub async fn get_token_balance(&self, address: &str, mint: &str) -> Result<u64>
```

Get SPL token balance for a specific token mint owned by an address

---

### get_transaction

**Source**: `src/client.rs`

```rust
pub async fn get_transaction(&self, signature: &str) -> Result<serde_json::Value>
```

Get transaction details

---

### get_transaction_with_meta

**Source**: `src/client.rs`

```rust
pub async fn get_transaction_with_meta( &self, signature: &str, ) -> Result<EncodedConfirmedTransactionWithStatusMeta>
```

Get transaction with full metadata for event parsing

---

### has_signer

**Source**: `src/client.rs`

```rust
pub fn has_signer(&self) -> bool
```

Check if client has a signer configured

---

### is_connected

**Source**: `src/client.rs`

```rust
pub async fn is_connected(&self) -> bool
```

Check if the client is connected

---

### mainnet

**Source**: `src/client.rs`

```rust
pub fn mainnet() -> Self
```

Create a new Solana client with default mainnet configuration

---

### new

**Source**: `src/client.rs`

```rust
pub fn new(config: SolanaConfig) -> Self
```

Create a new Solana client with the given configuration

---

### require_signer

**Source**: `src/client.rs`

```rust
pub fn require_signer(&self) -> Result<&Arc<Keypair>>
```

Get signer or return error if not configured

---

### send_and_confirm_transaction

**Source**: `src/client.rs`

```rust
pub async fn send_and_confirm_transaction( &self, transaction: &Transaction, ) -> Result<Signature>
```

Send and confirm a transaction with retries

---

### send_transaction

**Source**: `src/client.rs`

```rust
pub async fn send_transaction(&self, transaction: Transaction) -> Result<String>
```

Send a transaction

---

### signer

**Source**: `src/client.rs`

```rust
pub fn signer(&self) -> Option<&Arc<Keypair>>
```

Get reference to the signer if configured

---

### testnet

**Source**: `src/client.rs`

```rust
pub fn testnet() -> Self
```

Create a new Solana client with testnet configuration

---

### with_commitment

**Source**: `src/client.rs`

```rust
pub fn with_commitment(mut self, commitment: CommitmentLevel) -> Self
```

Set commitment level

---

### with_rpc_url

**Source**: `src/client.rs`

```rust
pub fn with_rpc_url(rpc_url: impl Into<String>) -> Self
```

Create a new Solana client with custom RPC URL

---

### with_signer

**Source**: `src/client.rs`

```rust
pub fn with_signer(mut self, keypair: Keypair) -> Self
```

Configure client with a keypair signer for transactions

---

### with_signer_from_bytes

**Source**: `src/client.rs`

```rust
pub fn with_signer_from_bytes(self, private_key_bytes: &[u8]) -> Result<Self>
```

Configure client with a signer from private key bytes

---

## Functions (local)

### from_seed_phrase

**Source**: `signer/local.rs`

```rust
pub fn from_seed_phrase(seed_phrase: &str, rpc_url: String) -> Result<Self, SignerError>
```

Create a new LocalSolanaSigner from a seed phrase

---

### keypair

**Source**: `signer/local.rs`

```rust
pub fn keypair(&self) -> &solana_sdk::signature::Keypair
```

Get the keypair (for advanced use cases)

---

### new

**Source**: `signer/local.rs`

```rust
pub fn new(keypair: solana_sdk::signature::Keypair, rpc_url: String) -> Self
```

Create a new LocalSolanaSigner with the given keypair and RPC URL

---

### rpc_url

**Source**: `signer/local.rs`

```rust
pub fn rpc_url(&self) -> &str
```

Get the RPC URL

---

## Functions (keypair)

### generate_mint_keypair

**Source**: `utils/keypair.rs`

```rust
pub fn generate_mint_keypair() -> Keypair
```

Generates new mint keypair for token creation

Creates a new randomly generated keypair suitable for use as a mint account
in SPL token creation operations.

# Returns

Returns a new `Keypair` with a randomly generated public/private key pair.

# Examples

```rust,ignore
use riglr_solana_tools::utils::keypair::generate_mint_keypair;

let mint_keypair = generate_mint_keypair();
println!("New mint pubkey: {}", mint_keypair.pubkey());
```

# Security Notes

- Each call generates a completely new keypair
- The private key should be handled securely
- For production use, consider proper key management practices

---

## Functions (transaction)

### create_token_with_mint_keypair

**Source**: `utils/transaction.rs`

```rust
pub async fn create_token_with_mint_keypair( instructions: Vec<Instruction>, mint_keypair: &Keypair, ) -> std::result::Result<String, SolanaToolError>
```

Creates properly signed Solana transaction with mint keypair

This function handles the complex case where a transaction needs to be signed by both
the signer context (for fees) and a mint keypair (for token creation).

# Arguments

* `instructions` - The instructions to include in the transaction
* `mint_keypair` - The keypair for the mint account (must sign the transaction)

# Returns

Returns the transaction signature on success

# Examples

```rust,ignore
use riglr_solana_tools::utils::transaction::create_token_with_mint_keypair;
use solana_sdk::{instruction::Instruction, signature::Keypair};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let mint_keypair = Keypair::new();
let instructions = vec![
// Token creation instructions here
];

let signature = create_token_with_mint_keypair(instructions, &mint_keypair).await?;
println!("Token created with signature: {}", signature);
# Ok(())
# }
```

---

### execute_solana_transaction

**Source**: `utils/transaction.rs`

```rust
pub async fn execute_solana_transaction<F, Fut>( tx_creator: F, ) -> std::result::Result<String, SolanaToolError> where F: FnOnce(Pubkey, Arc<RpcClient>) -> Fut + Send + 'static, Fut: Future<Output = std::result::Result<Transaction, SolanaToolError>> + Send + 'static,
```

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
use solana_sdk::{transaction::Transaction, system_instruction};

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

### send_transaction

**Source**: `utils/transaction.rs`

```rust
pub async fn send_transaction( transaction: &mut Transaction, operation_name: &str, ) -> std::result::Result<String, ToolError>
```

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
use solana_sdk::{transaction::Transaction, system_instruction};
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

### send_transaction_with_retry

**Source**: `utils/transaction.rs`

```rust
pub async fn send_transaction_with_retry( transaction: &mut Transaction, config: &TransactionConfig, operation_name: &str, ) -> std::result::Result<TransactionSubmissionResult, ToolError>
```

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
use solana_sdk::{transaction::Transaction, system_instruction};
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

## Functions (validation)

### validate_address

**Source**: `utils/validation.rs`

```rust
pub fn validate_address(address: &str) -> Result<Pubkey>
```

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

## Functions (config)

### get_rpc_url

**Source**: `utils/config.rs`

```rust
pub fn get_rpc_url() -> Result<String>
```

Get RPC URL from environment or use default

Retrieves the Solana RPC URL from the `SOLANA_RPC_URL` environment variable.
If not set or empty, defaults to mainnet-beta. Validates URL format to ensure
it's a proper HTTP/HTTPS/WebSocket URL.

# Returns

Returns the RPC URL string on success, or a `SolanaToolError::Generic` if
the URL format is invalid.

# Environment Variables

* `SOLANA_RPC_URL` - The RPC endpoint URL (optional)

# Examples

```rust,ignore
use riglr_solana_tools::utils::config::get_rpc_url;

// With environment variable set
std::env::set_var("SOLANA_RPC_URL", "https://api.devnet.solana.com");
let url = get_rpc_url()?;
assert_eq!(url, "https://api.devnet.solana.com");

// Without environment variable (uses default)
std::env::remove_var("SOLANA_RPC_URL");
let url = get_rpc_url()?;
assert_eq!(url, "https://api.mainnet-beta.solana.com");
```

# Security Notes

- Always validates URL format to prevent injection attacks
- Logs only the first 50 characters of custom URLs for privacy
- Defaults to mainnet for production safety

---


---

*This documentation was automatically generated from the source code.*