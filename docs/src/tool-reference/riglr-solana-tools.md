# riglr-solana-tools Tool Reference

This page contains documentation for tools provided by the `riglr-solana-tools` crate.

## Available Tools

- [`get_sol_balance`](#get_sol_balance) - src/balance.rs
- [`get_spl_token_balance`](#get_spl_token_balance) - src/balance.rs
- [`get_multiple_balances`](#get_multiple_balances) - src/balance.rs
- [`get_block_height`](#get_block_height) - src/network.rs
- [`get_transaction_status`](#get_transaction_status) - src/network.rs
- [`deploy_pump_token`](#deploy_pump_token) - src/pump.rs
- [`buy_pump_token`](#buy_pump_token) - src/pump.rs
- [`sell_pump_token`](#sell_pump_token) - src/pump.rs
- [`get_pump_token_info`](#get_pump_token_info) - src/pump.rs
- [`analyze_pump_transaction`](#analyze_pump_transaction) - src/pump.rs
- [`get_trending_pump_tokens`](#get_trending_pump_tokens) - src/pump.rs
- [`get_jupiter_quote`](#get_jupiter_quote) - src/swap.rs
- [`perform_jupiter_swap`](#perform_jupiter_swap) - src/swap.rs
- [`get_token_price`](#get_token_price) - src/swap.rs
- [`transfer_sol`](#transfer_sol) - src/transaction.rs
- [`transfer_spl_token`](#transfer_spl_token) - src/transaction.rs
- [`create_spl_token_mint`](#create_spl_token_mint) - src/transaction.rs

## Tool Functions

### get_sol_balance

**Source**: `src/balance.rs`

```rust
pub async fn get_sol_balance(address: String) -> Result<BalanceResult, ToolError>
```

**Documentation:**

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

```rust
pub async fn get_spl_token_balance( owner_address: String, mint_address: String, ) -> Result<TokenBalanceResult, ToolError>
```

**Documentation:**

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

### get_multiple_balances

**Source**: `src/balance.rs`

```rust
pub async fn get_multiple_balances( addresses: Vec<String>, ) -> Result<Vec<BalanceResult>, ToolError>
```

**Documentation:**

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

### get_block_height

**Source**: `src/network.rs`

```rust
pub async fn get_block_height() -> Result<u64, ToolError>
```

**Documentation:**

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

### get_transaction_status

**Source**: `src/network.rs`

```rust
pub async fn get_transaction_status(signature: String) -> Result<String, ToolError>
```

**Documentation:**

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

### deploy_pump_token

**Source**: `src/pump.rs`

```rust
pub async fn deploy_pump_token( name: String, symbol: String, description: String, image_url: Option<String>, initial_buy_sol: Option<f64>, ) -> Result<PumpTokenInfo, ToolError>
```

**Documentation:**

Deploy a new token on Pump.fun

This tool creates and deploys a new meme token on the Pump.fun platform.
Optionally performs an initial buy to bootstrap liquidity.

---

### buy_pump_token

**Source**: `src/pump.rs`

```rust
pub async fn buy_pump_token( token_mint: String, sol_amount: f64, slippage_percent: Option<f64>, ) -> Result<PumpTradeResult, ToolError>
```

**Documentation:**

Buy tokens on Pump.fun

This tool executes a buy order for a specific token on Pump.fun
with configurable slippage protection.

---

### sell_pump_token

**Source**: `src/pump.rs`

```rust
pub async fn sell_pump_token( token_mint: String, token_amount: u64, slippage_percent: Option<f64>, ) -> Result<PumpTradeResult, ToolError>
```

**Documentation:**

Sell tokens on Pump.fun

This tool executes a sell order for a specific token on Pump.fun
with configurable slippage protection.

---

### get_pump_token_info

**Source**: `src/pump.rs`

```rust
pub async fn get_pump_token_info(token_mint: String) -> Result<PumpTokenInfo, ToolError>
```

**Documentation:**

Get token information from Pump.fun

This tool fetches current token information, price, and market data
for a specific token on the Pump.fun platform.

---

### analyze_pump_transaction

**Source**: `src/pump.rs`

```rust
pub async fn analyze_pump_transaction( signature: String, user_address: String, token_mint: String, ) -> Result<PumpTradeAnalysis, ToolError>
```

**Documentation:**

Analyze a Pump.fun trade transaction to extract actual amounts and price

This tool parses a completed Pump.fun transaction to determine the actual
token amounts, SOL amounts, and price per token that were executed.
Separate from action tools following riglr separation of concerns pattern.

---

### get_trending_pump_tokens

**Source**: `src/pump.rs`

```rust
pub async fn get_trending_pump_tokens(limit: Option<u32>) -> Result<Vec<PumpTokenInfo>, ToolError>
```

**Documentation:**

Get trending tokens on Pump.fun

This tool fetches the currently trending tokens on the Pump.fun platform.

---

### get_jupiter_quote

**Source**: `src/swap.rs`

```rust
pub async fn get_jupiter_quote( input_mint: String, output_mint: String, amount: u64, slippage_bps: u16, only_direct_routes: bool, jupiter_api_url: Option<String>, ) -> Result<SwapQuote, ToolError>
```

**Documentation:**

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

### perform_jupiter_swap

**Source**: `src/swap.rs`

```rust
pub async fn perform_jupiter_swap( input_mint: String, output_mint: String, amount: u64, slippage_bps: u16, jupiter_api_url: Option<String>, use_versioned_transaction: bool, ) -> Result<SwapResult, ToolError>
```

**Documentation:**

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

### get_token_price

**Source**: `src/swap.rs`

```rust
pub async fn get_token_price( base_mint: String, quote_mint: String, jupiter_api_url: Option<String>, ) -> Result<PriceInfo, ToolError>
```

**Documentation:**

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

### transfer_sol

**Source**: `src/transaction.rs`

```rust
pub async fn transfer_sol( to_address: String, amount_sol: f64, memo: Option<String>, priority_fee: Option<u64>, ) -> Result<TransactionResult, ToolError>
```

**Documentation:**

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

```rust
pub async fn transfer_spl_token( to_address: String, mint_address: String, amount: u64, decimals: u8, create_ata_if_needed: bool, ) -> Result<TokenTransferResult, ToolError>
```

**Documentation:**

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

### create_spl_token_mint

**Source**: `src/transaction.rs`

```rust
pub async fn create_spl_token_mint( decimals: u8, initial_supply: u64, freezable: bool, ) -> Result<CreateMintResult, ToolError>
```

**Documentation:**

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


---

*This documentation was automatically generated from the source code.*