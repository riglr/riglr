# riglr-evm-tools

{{#include ../../../riglr-evm-tools/README.md}}

## API Reference

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Functions](#functions)
- [Type Aliases](#type-aliases)
- [Constants](#constants)

### Structs

> Core data structures and types.

#### `Args`

Arguments structure for the tool

---

#### `EthBalance`

Balance response for ETH

---

#### `GasConfig`

EVM gas configuration

---

#### `LocalEvmSigner`

Local EVM signer with private key management

---

#### `SwapQuote`

Swap quote response

---

#### `TokenBalance`

Balance response for ERC20 tokens

---

#### `Tool`

Tool implementation structure

---

#### `quoteExactInputSingleCall`

Returns the amount out received for a given exact input swap without executing the swap
 @param tokenIn The token being swapped in
 @param tokenOut The token being swapped out
 @param fee The fee of the token pool to consider for the pair
 @param amountIn The desired input amount
 @param sqrtPriceLimitX96 The price limit of the pool that cannot be exceeded by the swap
 @return amountOut The amount of `tokenOut` that would be received
 @return sqrtPriceX96After The sqrt price of the pool after the swap
 @return initializedTicksCrossed The number of initialized ticks crossed
 @return gasEstimate The estimate of the gas that the swap consumes
Function with signature `quoteExactInputSingle(address,address,uint24,uint256,uint160)` and selector `0xf7729d43`.
```solidity
function quoteExactInputSingle(address tokenIn, address tokenOut, uint24 fee, uint256 amountIn, uint160 sqrtPriceLimitX96) external returns (uint256 amountOut, uint160 sqrtPriceX96After, uint32 initializedTicksCrossed, uint256 gasEstimate);
```

---

#### `quoteExactInputSingleReturn`

Returns the amount out received for a given exact input swap without executing the swap
 @param tokenIn The token being swapped in
 @param tokenOut The token being swapped out
 @param fee The fee of the token pool to consider for the pair
 @param amountIn The desired input amount
 @param sqrtPriceLimitX96 The price limit of the pool that cannot be exceeded by the swap
 @return amountOut The amount of `tokenOut` that would be received
 @return sqrtPriceX96After The sqrt price of the pool after the swap
 @return initializedTicksCrossed The number of initialized ticks crossed
 @return gasEstimate The estimate of the gas that the swap consumes
Container type for the return parameters of the [`quoteExactInputSingle(address,address,uint24,uint256,uint160)`](quoteExactInputSingleCall) function.

---

### Enums

> Enumeration types for representing variants.

#### `ErrorClass`

Error classification for retry logic

**Variants:**

- `Permanent`
  - Permanent errors that should not be retried
- `Retriable`
  - Retriable errors that may succeed on retry
- `RateLimited`
  - Rate-limited errors that need backoff

---

#### `EvmToolError`

Main error type for EVM tools

**Variants:**

- `ProviderError`
  - Generic provider issues
- `TransactionReverted`
  - Transaction reverted on-chain
- `NonceTooLow`
  - Nonce is too low
- `NonceMismatch`
  - Nonce mismatch
- `InsufficientFunds`
  - Insufficient funds for transaction
- `GasEstimationFailed`
  - Gas estimation failed
- `InvalidAddress`
  - Invalid address format
- `ContractError`
  - Contract-related errors
- `SignerError`
  - Signer-related errors
- `NetworkError`
  - Network timeout or connection issues
- `RateLimited`
  - Rate limit exceeded
- `UnsupportedChain`
  - Unsupported chain
- `InvalidParameter`
  - Invalid parameters
- `Generic`
  - Generic error fallback

---

#### `IQuoterV2Calls`

Container for all the [`IQuoterV2`](self) function calls.

**Variants:**

- `quoteExactInputSingle`

---

### Functions

> Standalone functions and utilities.

#### `call_contract_read`

Call a contract read function (view/pure function)

This tool executes read-only contract functions that don't modify blockchain state.
It supports both function signatures and 4-byte selectors for maximum flexibility.

# Arguments

* `contract_address` - The smart contract address (checksummed hex format)
* `function_selector` - Either a function signature like "balanceOf(address)" or 4-byte selector like "0x70a08231"
* `params` - Function parameters as strings (addresses, numbers, hex data)

# Returns

Returns the decoded function result as a string. For complex return types,
returns JSON-formatted data.

# Errors

* `ToolError::Permanent` - When the contract address is invalid or function doesn't exist
* `ToolError::Retriable` - When network issues occur (timeouts, RPC errors)
* `ToolError::Permanent` - When no signer context is available

# Examples

```rust,ignore
// Check ERC20 token balance
let balance = call_contract_read(
    "0xA0b86a33E6441b8e606Fd25d43b2b6eaa8071CdB".to_string(),
    "balanceOf(address)".to_string(),
    vec!["0x742EEC0C53C37682b8c7d3210fd5D3e8D8054A8".to_string()]
).await?;
```

---

#### `call_contract_read_tool`

Factory function to create a new instance of the tool

---

#### `call_contract_write`

Call a contract write function (state-mutating function)

This tool executes state-changing contract functions that modify blockchain state.
It automatically handles transaction signing, gas estimation, and retry logic.

# Arguments

* `contract_address` - The smart contract address (checksummed hex format)
* `function_selector` - Either a function signature like "transfer(address,uint256)" or 4-byte selector
* `params` - Function parameters as strings (addresses, amounts, hex data)
* `gas_limit` - Optional gas limit override (default: 300,000)

# Returns

Returns the transaction hash as a string upon successful submission.
Note: This doesn't wait for confirmation, only successful broadcast.

# Errors

* `ToolError::Permanent` - When the contract address is invalid or function fails
* `ToolError::Retriable` - When network congestion or temporary RPC issues occur
* `ToolError::Permanent` - When insufficient funds or gas estimation fails
* `ToolError::Permanent` - When no signer context is available

# Examples

```rust,ignore
// Transfer ERC20 tokens
let tx_hash = call_contract_write(
    "0xA0b86a33E6441b8e606Fd25d43b2b6eaa8071CdB".to_string(),
    "transfer(address,uint256)".to_string(),
    vec![
        "0x742EEC0C53C37682b8c7d3210fd5D3e8D8054A8".to_string(),
        "1000000000000000000".to_string() // 1 token with 18 decimals
    ],
    Some(100000)
).await?;
```

---

#### `call_contract_write_tool`

Factory function to create a new instance of the tool

---

#### `classify_evm_error`

Classify EVM errors to determine retry behavior

This function uses structured error variants to determine classification,
with a fallback to string matching for provider errors that contain generic messages.

# Examples

```ignore
let error = EvmToolError::NonceTooLow;
assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);

let error = EvmToolError::InsufficientFunds;
assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);
```

---

#### `execute_evm_transaction`

Higher-order function to execute EVM transactions
Abstracts signer context retrieval and transaction signing

---

#### `get_block_number`

Get current block number

This tool implements smart chain ID resolution:
- If `chain_id` is provided, uses that specific chain
- If `chain_id` is None but there's an active EVM SignerContext, uses the signer's chain ID
- Otherwise defaults to Ethereum mainnet (chain_id = 1)

# Arguments
* `chain_id` - Optional chain ID. If None, attempts to resolve from SignerContext
* `context` - Application context containing provider and other extensions

---

#### `get_block_number_tool`

Factory function to create a new instance of the tool

---

#### `get_erc20_balance`

Get ERC20 token balance for an address

---

#### `get_erc20_balance_tool`

Factory function to create a new instance of the tool

---

#### `get_eth_balance`

Get ETH balance for an address on an EVM-compatible chain

This tool implements smart chain ID resolution:
- If `chain_id` is provided, uses that specific chain
- If `chain_id` is None but there's an active EVM SignerContext, uses the signer's chain ID
- Otherwise defaults to Ethereum mainnet (chain_id = 1)

# Arguments
* `address` - The wallet address to check (hex format, with or without 0x prefix)
* `chain_id` - Optional chain ID. If None, attempts to resolve from SignerContext
* `context` - Application context containing provider and other extensions

# Examples
```ignore
// Explicit chain ID
let balance = get_eth_balance("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb5".to_string(), Some(1), &context).await?;

// Auto-resolve from SignerContext
let balance = get_eth_balance("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb5".to_string(), None, &context).await?;
```

---

#### `get_eth_balance_tool`

Factory function to create a new instance of the tool

---

#### `get_gas_price`

Get gas price

This tool implements smart chain ID resolution:
- If `chain_id` is provided, uses that specific chain
- If `chain_id` is None but there's an active EVM SignerContext, uses the signer's chain ID
- Otherwise defaults to Ethereum mainnet (chain_id = 1)

# Arguments
* `chain_id` - Optional chain ID. If None, attempts to resolve from SignerContext
* `context` - Application context containing provider and other extensions

---

#### `get_gas_price_tool`

Factory function to create a new instance of the tool

---

#### `get_token_decimals`

Get token decimals from ERC20 contract

---

#### `get_token_name`

Get token name from ERC20 contract

---

#### `get_token_symbol`

Get token symbol from ERC20 contract

---

#### `get_uniswap_quote`

Get swap quote from Uniswap

This tool implements smart chain ID resolution:
- If `chain_id` is provided, uses that specific chain
- If `chain_id` is None but there's an active EVM SignerContext, uses the signer's chain ID
- Otherwise defaults to Ethereum mainnet (chain_id = 1)

# Arguments
* `token_in` - Address of input token
* `token_out` - Address of output token  
* `amount_in` - Amount to swap in (in token's smallest unit)
* `decimals_in` - Decimal places of input token
* `decimals_out` - Decimal places of output token
* `fee_tier` - Pool fee tier (default: 3000 = 0.3%)
* `slippage` - Slippage tolerance in basis points (default: 50 = 0.5%)
* `chain_id` - Optional chain ID. If None, attempts to resolve from SignerContext
* `context` - Application context containing provider and configuration

---

#### `get_uniswap_quote_tool`

Factory function to create a new instance of the tool

---

#### `make_provider`

Factory function for creating EVM providers
Centralizes provider creation and ensures consistent configuration

---

#### `read_erc20_info`

Read ERC20 token information

This tool retrieves comprehensive information about an ERC20 token including
name, symbol, decimals, and total supply. It handles standard ERC20 contracts
and gracefully degrades for non-standard implementations.

# Arguments

* `token_address` - The ERC20 token contract address (checksummed hex format)

# Returns

Returns a JSON object containing:
- `address`: The token contract address
- `name`: Token name (e.g., "Chainlink Token") or null if not available
- `symbol`: Token symbol (e.g., "LINK") or null if not available
- `decimals`: Number of decimal places (defaults to 18 if not available)
- `totalSupply`: Total token supply as a string in base units

# Errors

* `ToolError::Permanent` - When the token address is invalid
* `ToolError::Retriable` - When network issues prevent data retrieval
* `ToolError::Permanent` - When no signer context is available

# Examples

```rust,ignore
// Get USDC token information
let token_info = read_erc20_info(
    "0xA0b86a33E6441b8e606Fd25d43b2b6eaa8071CdB".to_string()
).await?;

println!("Token: {} ({})", token_info["name"], token_info["symbol"]);
println!("Decimals: {}", token_info["decimals"]);
```

---

#### `read_erc20_info_tool`

Factory function to create a new instance of the tool

---

#### `send_eth`

Send ETH to an address (requires SignerContext for transaction signing)

---

#### `send_eth_tool`

Factory function to create a new instance of the tool

---

### Type Aliases

#### `EvmProvider`

Type alias for an Arc-wrapped Ethereum provider

**Type:** `<_>`

---

### Constants

#### `VERSION`

Current version of riglr-evm-tools

**Type:** `&str`

---
