# riglr-evm-tools Tool Reference

This page contains documentation for tools provided by the `riglr-evm-tools` crate.

## Available Tools

- [`get_eth_balance`](#get_eth_balance) - src/balance.rs
- [`get_erc20_balance`](#get_erc20_balance) - src/balance.rs
- [`call_contract_read`](#call_contract_read) - src/contract.rs
- [`call_contract_write`](#call_contract_write) - src/contract.rs
- [`read_erc20_info`](#read_erc20_info) - src/contract.rs
- [`get_uniswap_quote`](#get_uniswap_quote) - src/swap.rs
- [`perform_uniswap_swap`](#perform_uniswap_swap) - src/swap.rs
- [`transfer_eth`](#transfer_eth) - src/transaction.rs
- [`transfer_erc20`](#transfer_erc20) - src/transaction.rs
- [`get_transaction_receipt`](#get_transaction_receipt) - src/transaction.rs

## Tool Functions

### get_eth_balance

**Source**: `src/balance.rs`

```rust
pub async fn get_eth_balance( address: String, block_number: Option<u64>, ) -> std::result::Result<BalanceResult, Box<dyn std::error::Error + Send + Sync>>
```

**Documentation:**

Get ETH balance for an address

This tool retrieves the native ETH balance for any Ethereum wallet address on the current
EVM chain. The balance is returned in both wei (smallest unit) and ETH (human-readable format).

# Arguments

* `address` - The Ethereum wallet address to check (0x-prefixed hex string)
* `chain_id` - EVM chain identifier (1 for Ethereum mainnet, 42161 for Arbitrum, etc.)
* `block_number` - Optional specific block number to query (uses latest if None)

# Returns

Returns `BalanceResult` containing:
- `address`: The queried wallet address
- `balance_raw`: Balance in wei (1 ETH = 10^18 wei)
- `balance_formatted`: Balance in ETH with 6 decimal places
- `unit`: "ETH" currency identifier
- `chain_id`: EVM chain identifier (1 for Ethereum mainnet)
- `chain_name`: Human-readable chain name
- `block_number`: Block number at which balance was fetched

# Errors

* `EvmToolError::InvalidAddress` - When the address format is invalid
* `EvmToolError::Rpc` - When network connection issues occur
* `EvmToolError::Generic` - When no signer context is available

# Examples

```rust,ignore
use riglr_evm_tools::balance::get_eth_balance;
use riglr_core::SignerContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Check ETH balance for Vitalik's address
let balance = get_eth_balance(
"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
1, // Ethereum mainnet
None, // Use latest block
).await?;

println!("Address: {}", balance.address);
println!("Balance: {} ETH ({} wei)", balance.balance_formatted, balance.balance_raw);
println!("Chain: {} (ID: {})", balance.chain_name, balance.chain_id);
# Ok(())
# }
```

---

### get_erc20_balance

**Source**: `src/balance.rs`

```rust
pub async fn get_erc20_balance( address: String, token_address: String, fetch_metadata: Option<bool>, ) -> std::result::Result<TokenBalanceResult, Box<dyn std::error::Error + Send + Sync>>
```

**Documentation:**

Get ERC20 token balance for an address

This tool retrieves the balance of any ERC20 token for a given Ethereum wallet address.
It automatically fetches token metadata (symbol, name, decimals) and formats the balance
appropriately. Works with any standard ERC20 token contract.

# Arguments

* `address` - The Ethereum wallet address to check token balance for
* `token_address` - The ERC20 token contract address
* `chain_id` - EVM chain identifier (1 for Ethereum mainnet, 42161 for Arbitrum, etc.)
* `fetch_metadata` - Whether to fetch token metadata (symbol, name) - defaults to true

# Returns

Returns `TokenBalanceResult` containing:
- `address`: The wallet address queried
- `token_address`: The token contract address
- `token_symbol`, `token_name`: Token metadata (if fetched)
- `decimals`: Number of decimal places for the token
- `balance_raw`: Balance in token's smallest unit
- `balance_formatted`: Human-readable balance with decimal adjustment
- `chain_id`, `chain_name`: Network information

# Errors

* `EvmToolError::InvalidAddress` - When wallet or token address is invalid
* `EvmToolError::Rpc` - When network issues occur or token contract doesn't respond
* `EvmToolError::Generic` - When no signer context is available

# Examples

```rust,ignore
use riglr_evm_tools::balance::get_erc20_balance;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Check USDC balance
let balance = get_erc20_balance(
"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC contract
1, // Ethereum mainnet
Some(true), // Fetch metadata
).await?;

println!("Token: {} ({})", balance.token_symbol.unwrap_or_default(), balance.token_name.unwrap_or_default());
println!("Balance: {} (decimals: {})", balance.balance_formatted, balance.decimals);
println!("Raw balance: {}", balance.balance_raw);
# Ok(())
# }
```

---

### call_contract_read

**Source**: `src/contract.rs`

```rust
pub async fn call_contract_read( contract_address: String, function_selector: String, params: Vec<String>, ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
```

**Documentation:**

Call a contract read function (view/pure function)

---

### call_contract_write

**Source**: `src/contract.rs`

```rust
pub async fn call_contract_write( contract_address: String, function_selector: String, params: Vec<String>, gas_limit: Option<u64>, ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
```

**Documentation:**

Call a contract write function (state-mutating function)

---

### read_erc20_info

**Source**: `src/contract.rs`

```rust
pub async fn read_erc20_info( token_address: String, ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>
```

**Documentation:**

Read ERC20 token information

---

### get_uniswap_quote

**Source**: `src/swap.rs`

```rust
pub async fn get_uniswap_quote( token_in: String, token_out: String, amount_in: String, decimals_in: u8, decimals_out: u8, fee_tier: Option<u32>, slippage_bps: Option<u16>, ) -> std::result::Result<UniswapQuote, Box<dyn std::error::Error + Send + Sync>>
```

**Documentation:**

Get a quote from Uniswap V3

This tool queries Uniswap V3 to get a quote for swapping ERC20 tokens without executing
any transaction. It uses the Quoter contract to estimate output amounts and price impact.
Supports all EVM chains where Uniswap V3 is deployed.

# Arguments

* `token_in` - Input token contract address (ERC20)
* `token_out` - Output token contract address (ERC20)
* `amount_in` - Input amount as string (e.g., "1.5" for human-readable amount)
* `decimals_in` - Number of decimals for input token
* `decimals_out` - Number of decimals for output token
* `fee_tier` - Uniswap pool fee tier (100=0.01%, 500=0.05%, 3000=0.3%, 10000=1%)
* `slippage_bps` - Slippage tolerance in basis points (50 = 0.5%)

# Returns

Returns `UniswapQuote` containing:
- `token_in`, `token_out`: Token addresses
- `amount_in`, `amount_out`: Input and expected output amounts
- `price`: Exchange rate (output tokens per 1 input token)
- `fee_tier`: Pool fee used for the quote
- `slippage_bps`: Slippage tolerance applied
- `amount_out_minimum`: Minimum output after slippage

# Errors

* `EvmToolError::InvalidAddress` - When token addresses are invalid
* `EvmToolError::Rpc` - When Quoter contract call fails (often due to no liquidity)
* `EvmToolError::Generic` - When amount parsing fails

# Examples

```rust,ignore
use riglr_evm_tools::swap::get_uniswap_quote;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Get quote for swapping 1 WETH to USDC
let quote = get_uniswap_quote(
"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(), // WETH
"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
"1.0".to_string(), // 1 WETH
18, // WETH decimals
6,  // USDC decimals
Some(3000), // 0.3% fee tier
Some(50),   // 0.5% slippage
).await?;

println!("Quote: {} WETH -> {} USDC", quote.amount_in, quote.amount_out);
println!("Price: {} USDC per WETH", quote.price);
println!("Minimum output: {}", quote.amount_out_minimum);
# Ok(())
# }
```

---

### perform_uniswap_swap

**Source**: `src/swap.rs`

```rust
pub async fn perform_uniswap_swap( token_in: String, token_out: String, amount_in: String, decimals_in: u8, amount_out_minimum: String, fee_tier: Option<u32>, deadline_seconds: Option<u64>, ) -> std::result::Result<UniswapSwapResult, Box<dyn std::error::Error + Send + Sync>>
```

**Documentation:**

Perform a token swap on Uniswap V3

This tool executes an actual token swap on Uniswap V3 by calling the SwapRouter contract.
It constructs the appropriate transaction with slippage protection and deadline management.
Requires token approvals to be set beforehand for the SwapRouter contract.

# Arguments

* `token_in` - Input token contract address
* `token_out` - Output token contract address
* `amount_in` - Input amount as string (human-readable)
* `decimals_in` - Number of decimals for input token
* `amount_out_minimum` - Minimum acceptable output amount (for slippage protection)
* `fee_tier` - Uniswap pool fee tier to use
* `deadline_seconds` - Transaction deadline in seconds from now

# Returns

Returns `UniswapSwapResult` containing transaction details and actual amounts.

# Errors

* `EvmToolError::InvalidAddress` - When token addresses are invalid
* `EvmToolError::Transaction` - When insufficient token balance or allowance
* `EvmToolError::Rpc` - When transaction fails or network issues occur

# Examples

```rust,ignore
use riglr_evm_tools::swap::perform_uniswap_swap;
use riglr_core::SignerContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Note: Token approval for SwapRouter must be done first
let result = perform_uniswap_swap(
"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(), // WETH
"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
"0.5".to_string(), // 0.5 WETH
18, // WETH decimals
"900000000".to_string(), // Minimum 900 USDC out (6 decimals)
Some(3000), // 0.3% fee tier
Some(300),  // 5 minute deadline
).await?;

println!("Swap completed! Hash: {}", result.tx_hash);
println!("Swapped {} for {} tokens", result.amount_in, result.amount_out);
# Ok(())
# }
```

---

### transfer_eth

**Source**: `src/transaction.rs`

```rust
pub async fn transfer_eth( to_address: String, amount_eth: f64, gas_price_gwei: Option<u64>, nonce: Option<u64>, ) -> std::result::Result<TransactionResult, Box<dyn std::error::Error + Send + Sync>>
```

**Documentation:**

Transfer ETH to another address

This tool creates, signs, and executes an ETH transfer transaction using the current signer context.
It supports custom gas pricing and nonce management for transaction optimization and replacement.

# Arguments

* `to_address` - Recipient Ethereum address (0x-prefixed hex string)
* `amount_eth` - Amount to transfer in ETH (e.g., 0.01 for 0.01 ETH)
* `gas_price_gwei` - Optional gas price in Gwei (e.g., 20 for 20 Gwei)
* `nonce` - Optional transaction nonce (uses next available if None)

# Returns

Returns `TransactionResult` containing:
- `tx_hash`: Transaction hash for tracking on blockchain
- `from`, `to`: Sender and recipient addresses
- `value_wei`, `value_eth`: Transfer amount in both wei and ETH
- `gas_used`, `gas_price`: Gas consumption and pricing
- `block_number`: Block where transaction was confirmed
- `chain_id`: EVM chain identifier
- `status`: Whether transaction succeeded

# Errors

* `EvmToolError::InvalidAddress` - When recipient address is invalid
* `EvmToolError::Transaction` - When sender has insufficient ETH balance
* `EvmToolError::Rpc` - When network issues or transaction failures occur
* `EvmToolError::Generic` - When no signer context available

# Examples

```rust,ignore
use riglr_evm_tools::transaction::transfer_eth;
use riglr_core::SignerContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Transfer 0.01 ETH with custom gas price
let result = transfer_eth(
"0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(),
0.01, // 0.01 ETH
Some(25), // 25 Gwei gas price
None, // Auto-select nonce
).await?;

println!("Transfer completed! Hash: {}", result.tx_hash);
println!("Sent {} ETH from {} to {}", result.value_eth, result.from, result.to);
println!("Gas used: {} at {} wei", result.gas_used.unwrap_or(0), result.gas_price.unwrap_or(0));
# Ok(())
# }
```

---

### transfer_erc20

**Source**: `src/transaction.rs`

```rust
pub async fn transfer_erc20( token_address: String, to_address: String, amount: String, decimals: u8, gas_price_gwei: Option<u64>, ) -> std::result::Result<TransactionResult, Box<dyn std::error::Error + Send + Sync>>
```

**Documentation:**

Transfer ERC20 tokens to another address

This tool creates, signs, and executes an ERC20 token transfer transaction. It constructs
the appropriate contract call to the token's transfer function and handles decimal conversion.

# Arguments

* `token_address` - ERC20 token contract address
* `to_address` - Recipient Ethereum address
* `amount` - Amount to transfer as string (e.g., "100.5" for human-readable amount)
* `decimals` - Number of decimal places for the token (e.g., 6 for USDC, 18 for most tokens)
* `gas_price_gwei` - Optional gas price in Gwei

# Returns

Returns `TransactionResult` with transaction details. Note that `value_eth` will be 0
since this is a token transfer, not ETH transfer.

# Errors

* `EvmToolError::InvalidAddress` - When token or recipient address is invalid
* `EvmToolError::Transaction` - When sender has insufficient token balance
* `EvmToolError::Rpc` - When network issues or transaction failures occur

# Examples

```rust,ignore
use riglr_evm_tools::transaction::transfer_erc20;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Transfer 100.5 USDC (6 decimals)
let result = transfer_erc20(
"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC contract
"0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(), // Recipient
"100.5".to_string(), // 100.5 tokens
6, // USDC has 6 decimals
Some(30), // 30 Gwei gas price
).await?;

println!("Token transfer completed! Hash: {}", result.tx_hash);
println!("Transferred {} tokens", "100.5");
# Ok(())
# }
```

---

### get_transaction_receipt

**Source**: `src/transaction.rs`

```rust
pub async fn get_transaction_receipt( tx_hash: String, ) -> std::result::Result<TransactionResult, Box<dyn std::error::Error + Send + Sync>>
```

**Documentation:**

Get transaction receipt

This tool retrieves the receipt and details for a completed transaction using its hash.
Useful for checking transaction status, gas usage, and extracting event logs.

# Arguments

* `tx_hash` - Transaction hash to look up (0x-prefixed hex string)

# Returns

Returns `TransactionResult` with full transaction details including gas usage,
block number, and execution status.

# Errors

* `EvmToolError::Rpc` - When network issues occur or transaction lookup fails
* `EvmToolError::Generic` - When transaction is not found or invalid hash format

# Examples

```rust,ignore
use riglr_evm_tools::transaction::get_transaction_receipt;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let receipt = get_transaction_receipt(
"0x1234...abcd".to_string(), // Transaction hash
).await?;

println!("Transaction: {}", receipt.tx_hash);
println!("Block: {:?}", receipt.block_number);
println!("Gas used: {:?}", receipt.gas_used);
println!("Success: {}", receipt.status);
# Ok(())
# }
```

---


---

*This documentation was automatically generated from the source code.*