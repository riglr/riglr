# riglr-evm-common API Reference

Comprehensive API documentation for the `riglr-evm-common` crate.

## Table of Contents

### Enums

- [`EvmCommonError`](#evmcommonerror)

### Functions (error)

- [`is_permanent`](#is_permanent)
- [`is_retriable`](#is_retriable)

### Constants

- [`BURN_ADDRESS`](#burn_address)
- [`USDC_ETHEREUM`](#usdc_ethereum)
- [`USDT_ETHEREUM`](#usdt_ethereum)
- [`VERSION`](#version)
- [`WETH_ETHEREUM`](#weth_ethereum)
- [`ZERO_ADDRESS`](#zero_address)

### Structs

- [`ChainInfo`](#chaininfo)
- [`EvmAccount`](#evmaccount)
- [`EvmConfig`](#evmconfig)
- [`EvmToken`](#evmtoken)
- [`EvmTransactionData`](#evmtransactiondata)

### Functions (types)

- [`contract_address`](#contract_address)
- [`data_bytes`](#data_bytes)
- [`display_address`](#display_address)
- [`for_chain`](#for_chain)
- [`format_amount`](#format_amount)
- [`gas_limit_u64`](#gas_limit_u64)
- [`gas_price_u256`](#gas_price_u256)
- [`is_native`](#is_native)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`timeout`](#timeout)
- [`to_address`](#to_address)
- [`to_address`](#to_address)
- [`validate`](#validate)
- [`value_u256`](#value_u256)
- [`with_name`](#with_name)

### Functions (address)

- [`ensure_0x_prefix`](#ensure_0x_prefix)
- [`format_address`](#format_address)
- [`format_address_string`](#format_address_string)
- [`is_checksummed`](#is_checksummed)
- [`parse_evm_address`](#parse_evm_address)
- [`strip_0x_prefix`](#strip_0x_prefix)
- [`validate_evm_address`](#validate_evm_address)

### Functions (chain)

- [`chain_id_to_name`](#chain_id_to_name)
- [`chain_id_to_rpc_url`](#chain_id_to_rpc_url)
- [`chain_name_to_id`](#chain_name_to_id)
- [`get_address_url`](#get_address_url)
- [`get_block_explorer_url`](#get_block_explorer_url)
- [`get_chain_info`](#get_chain_info)
- [`get_supported_chains`](#get_supported_chains)
- [`get_transaction_url`](#get_transaction_url)
- [`is_supported_chain`](#is_supported_chain)

## Enums

### EvmCommonError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Debug, Error)]
```

```rust
pub enum EvmCommonError { /// Invalid EVM address format #[error("Invalid EVM address: {0}")] InvalidAddress(String), /// Unsupported or unconfigured chain #[error("Unsupported chain ID: {0}. Configure RPC_URL_{0} environment variable")] UnsupportedChain(u64), /// Invalid chain name #[error("Invalid chain name: {0}")] InvalidChainName(String), /// RPC provider error #[error("RPC provider error: {0}")] ProviderError(String), /// Configuration validation error #[error("Configuration error: {0}")] InvalidConfig(String), /// Invalid transaction data #[error("Invalid transaction data: {0}")] InvalidData(String), /// Network connection error #[error("Network error: {0}")] NetworkError(String), /// Parsing error #[error("Parse error: {0}")] ParseError(String), }
```

Error types for EVM operations shared across crates

**Variants**:

- `InvalidAddress(String)`
- `UnsupportedChain(u64)`
- `InvalidChainName(String)`
- `ProviderError(String)`
- `InvalidConfig(String)`
- `InvalidData(String)`
- `NetworkError(String)`
- `ParseError(String)`

---

## Functions (error)

### is_permanent

**Source**: `src/error.rs`

```rust
pub fn is_permanent(&self) -> bool
```

Check if this error is permanent (configuration/validation issues)

---

### is_retriable

**Source**: `src/error.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if this error is retriable (network/temporary issues)

---

## Constants

### BURN_ADDRESS

**Source**: `src/address.rs`

```rust
const BURN_ADDRESS: &str
```

Burn address (0xdead)

---

### USDC_ETHEREUM

**Source**: `src/address.rs`

```rust
const USDC_ETHEREUM: &str
```

USDC address on Ethereum mainnet

---

### USDT_ETHEREUM

**Source**: `src/address.rs`

```rust
const USDT_ETHEREUM: &str
```

USDT address on Ethereum mainnet

---

### VERSION

**Source**: `src/lib.rs`

```rust
const VERSION: &str
```

Current version of riglr-evm-common

---

### WETH_ETHEREUM

**Source**: `src/address.rs`

```rust
const WETH_ETHEREUM: &str
```

WETH address on Ethereum mainnet

---

### ZERO_ADDRESS

**Source**: `src/address.rs`

```rust
const ZERO_ADDRESS: &str
```

Zero address (0x0)

---

## Structs

### ChainInfo

**Source**: `src/chain.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ChainInfo { /// Numeric chain ID (e.g., 1 for Ethereum, 137 for Polygon)
```

Chain information structure

---

### EvmAccount

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct EvmAccount { /// Address of the account (hex format with 0x prefix)
```

Common EVM account metadata

---

### EvmConfig

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct EvmConfig { /// RPC endpoint URL for the EVM chain pub rpc_url: String, /// Chain ID (e.g., 1 for Ethereum, 137 for Polygon)
```

Configuration for EVM operations shared across crates

---

### EvmToken

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct EvmToken { /// Contract address (0x0 for native token)
```

Token information for ERC20 and native tokens

---

### EvmTransactionData

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct EvmTransactionData { /// Target contract address pub to: String, /// Transaction data (hex encoded)
```

EVM transaction data for cross-chain operations

---

## Functions (types)

### contract_address

**Source**: `src/types.rs`

```rust
pub fn contract_address(&self) -> Result<Option<Address>, crate::EvmCommonError>
```

Get contract address as Alloy Address (for non-native tokens)

---

### data_bytes

**Source**: `src/types.rs`

```rust
pub fn data_bytes(&self) -> Result<Bytes, crate::EvmCommonError>
```

Parse data as Alloy Bytes

---

### display_address

**Source**: `src/types.rs`

```rust
pub fn display_address(&self) -> Result<String, crate::EvmCommonError>
```

Format address for display (checksummed)

---

### for_chain

**Source**: `src/types.rs`

```rust
pub fn for_chain(chain_id: u64) -> Result<Self, crate::EvmCommonError>
```

Create config for a specific chain ID

---

### format_amount

**Source**: `src/types.rs`

```rust
pub fn format_amount(&self, raw_amount: U256) -> String
```

Convert amount from smallest unit to human-readable format

---

### gas_limit_u64

**Source**: `src/types.rs`

```rust
pub fn gas_limit_u64(&self) -> Result<u64, crate::EvmCommonError>
```

Parse gas limit as u64

---

### gas_price_u256

**Source**: `src/types.rs`

```rust
pub fn gas_price_u256(&self) -> Result<U256, crate::EvmCommonError>
```

Parse gas price as U256

---

### is_native

**Source**: `src/types.rs`

```rust
pub fn is_native(&self) -> bool
```

Check if this is a native token (ETH, MATIC, etc.)

---

### new

**Source**: `src/types.rs`

```rust
pub fn new(address: &str, chain_id: u64) -> Result<Self, crate::EvmCommonError>
```

Create new EVM account with validation

---

### new

**Source**: `src/types.rs`

```rust
pub fn new( to: &str, data: &str, value: U256, gas_limit: u64, gas_price: U256, chain_id: u64, ) -> Result<Self, crate::EvmCommonError>
```

Create new EVM transaction data with validation

---

### new

**Source**: `src/types.rs`

```rust
pub fn new( address: &str, symbol: String, name: String, decimals: u8, chain_id: u64, ) -> Result<Self, crate::EvmCommonError>
```

Create new EVM token with validation

---

### timeout

**Source**: `src/types.rs`

```rust
pub fn timeout(&self) -> Duration
```

Get timeout as Duration

---

### to_address

**Source**: `src/types.rs`

```rust
pub fn to_address(&self) -> Result<Address, crate::EvmCommonError>
```

Get address as Alloy Address type

---

### to_address

**Source**: `src/types.rs`

```rust
pub fn to_address(&self) -> Result<Address, crate::EvmCommonError>
```

Parse to address as Alloy Address

---

### validate

**Source**: `src/types.rs`

```rust
pub fn validate(&self) -> Result<(), crate::EvmCommonError>
```

Validate configuration

---

### value_u256

**Source**: `src/types.rs`

```rust
pub fn value_u256(&self) -> Result<U256, crate::EvmCommonError>
```

Parse value as U256

---

### with_name

**Source**: `src/types.rs`

```rust
pub fn with_name( address: &str, chain_id: u64, name: String, ) -> Result<Self, crate::EvmCommonError>
```

Create new EVM account with name

---

## Functions (address)

### ensure_0x_prefix

**Source**: `src/address.rs`

```rust
pub fn ensure_0x_prefix(address: &str) -> String
```

Ensure address has 0x prefix

# Arguments
* `address` - Address string (with or without 0x prefix)

# Returns
* Address string guaranteed to have 0x prefix

# Examples
```rust,ignore
use riglr_evm_common::address::ensure_0x_prefix;

assert_eq!(ensure_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
assert_eq!(ensure_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
```

---

### format_address

**Source**: `src/address.rs`

```rust
pub fn format_address(address: &Address) -> String
```

Format an address for display with EIP-55 checksumming

Converts any valid address to its checksummed representation
for safe display and copy-paste operations.

# Arguments
* `address` - Alloy Address to format

# Returns
* Checksummed address string with 0x prefix

# Examples
```rust,ignore
use riglr_evm_common::address::{parse_evm_address, format_address};

let addr = parse_evm_address("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")?;
let formatted = format_address(&addr);
assert_eq!(formatted, "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
```

---

### format_address_string

**Source**: `src/address.rs`

```rust
pub fn format_address_string(address: &str) -> EvmResult<String>
```

Format an address string with EIP-55 checksumming

Convenience function that parses and formats in one step.

# Arguments
* `address` - Address string to format

# Returns
* Checksummed address string

# Examples
```rust,ignore
use riglr_evm_common::address::format_address_string;

let formatted = format_address_string("742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")?;
assert_eq!(formatted, "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
```

---

### is_checksummed

**Source**: `src/address.rs`

```rust
pub fn is_checksummed(address: &str) -> bool
```

Check if an address string is already checksummed according to EIP-55

# Arguments
* `address` - Address string to check

# Returns
* `true` if address is properly checksummed, `false` if not checksummed or invalid

# Examples
```rust,ignore
use riglr_evm_common::address::is_checksummed;

assert!(is_checksummed("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"));
assert!(!is_checksummed("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e"));
assert!(!is_checksummed("invalid"));
```

---

### parse_evm_address

**Source**: `src/address.rs`

```rust
pub fn parse_evm_address(address: &str) -> EvmResult<Address>
```

Parse an EVM address string into an Alloy Address type

Handles both checksummed and non-checksummed addresses.
Automatically adds 0x prefix if missing.

# Arguments
* `address` - Address string to parse

# Returns
* `Address` - Parsed Alloy address type

# Examples
```rust,ignore
use riglr_evm_common::address::parse_evm_address;

let addr = parse_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")?;
let addr2 = parse_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")?;
assert_eq!(addr, addr2);
```

---

### strip_0x_prefix

**Source**: `src/address.rs`

```rust
pub fn strip_0x_prefix(address: &str) -> &str
```

Extract the address portion without 0x prefix

# Arguments
* `address` - Address string (with or without 0x prefix)

# Returns
* Address hex string without 0x prefix

# Examples
```rust,ignore
use riglr_evm_common::address::strip_0x_prefix;

assert_eq!(strip_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
assert_eq!(strip_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
```

---

### validate_evm_address

**Source**: `src/address.rs`

```rust
pub fn validate_evm_address(address: &str) -> EvmResult<()>
```

Validate an EVM address string format

Accepts both checksummed and non-checksummed addresses.
Returns error for invalid formats or lengths.

# Arguments
* `address` - Address string to validate (with or without 0x prefix)

# Examples
```rust,ignore
use riglr_evm_common::address::validate_evm_address;

// Valid addresses
assert!(validate_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());
assert!(validate_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());

// Invalid addresses
assert!(validate_evm_address("invalid").is_err());
assert!(validate_evm_address("0x123").is_err());
```

---

## Functions (chain)

### chain_id_to_name

**Source**: `src/chain.rs`

```rust
pub fn chain_id_to_name(id: u64) -> EvmResult<String>
```

Convert chain ID to human-readable name

# Arguments
* `id` - Numeric chain ID

# Returns
* Human-readable chain name

# Examples
```rust,ignore
use riglr_evm_common::chain::chain_id_to_name;

assert_eq!(chain_id_to_name(1)?, "ethereum");
assert_eq!(chain_id_to_name(137)?, "polygon");
```

---

### chain_id_to_rpc_url

**Source**: `src/chain.rs`

```rust
pub fn chain_id_to_rpc_url(chain_id: u64) -> EvmResult<String>
```

Maps chain IDs to RPC URLs using convention-based environment variable lookup.
Uses format: RPC_URL_{CHAIN_ID}

This is the UNIFIED approach that eliminates conflicts between different
chain management systems across riglr crates.

# Arguments
* `chain_id` - Numeric chain ID (e.g., 1 for Ethereum, 137 for Polygon)

# Returns
* RPC URL string from environment or error if not configured

# Environment Variables
* `RPC_URL_1` - Ethereum mainnet
* `RPC_URL_137` - Polygon
* `RPC_URL_42161` - Arbitrum
* `RPC_URL_10` - Optimism
* `RPC_URL_8453` - Base
* etc.

# Examples
```rust,ignore
use riglr_evm_common::chain::chain_id_to_rpc_url;

// Configure environment
std::env::set_var("RPC_URL_1", "https://eth.llamarpc.com");

let url = chain_id_to_rpc_url(1)?;
assert_eq!(url, "https://eth.llamarpc.com");
```

---

### chain_name_to_id

**Source**: `src/chain.rs`

```rust
pub fn chain_name_to_id(name: &str) -> EvmResult<u64>
```

Convert chain name to chain ID

This provides a bridge between human-readable names and numeric IDs,
useful for cross-chain operations and user interfaces.

# Arguments
* `name` - Chain name (case-insensitive)

# Returns
* Numeric chain ID

# Supported Names
* "ethereum", "eth" → 1
* "polygon", "matic" → 137
* "arbitrum", "arb" → 42161
* "optimism", "op" → 10
* "base" → 8453
* "bsc", "binance" → 56
* "avalanche", "avax" → 43114
* "fantom", "ftm" → 250

# Examples
```rust,ignore
use riglr_evm_common::chain::chain_name_to_id;

assert_eq!(chain_name_to_id("ethereum")?, 1);
assert_eq!(chain_name_to_id("ETH")?, 1);
assert_eq!(chain_name_to_id("polygon")?, 137);
```

---

### get_address_url

**Source**: `src/chain.rs`

```rust
pub fn get_address_url(chain_id: u64, address: &str) -> EvmResult<String>
```

Get address URL for viewing an address in block explorer

# Arguments
* `chain_id` - Numeric chain ID
* `address` - Address (with or without 0x prefix)

# Returns
* Full URL to view address in block explorer

# Examples
```rust,ignore
use riglr_evm_common::chain::get_address_url;

let url = get_address_url(1, "0x742d35Cc...")?;
// Returns: https://etherscan.io/address/0x742d35Cc...
```

---

### get_block_explorer_url

**Source**: `src/chain.rs`

```rust
pub fn get_block_explorer_url(chain_id: u64) -> EvmResult<String>
```

Get block explorer URL for a chain

# Arguments
* `chain_id` - Numeric chain ID

# Returns
* Block explorer base URL if known

# Examples
```rust,ignore
use riglr_evm_common::chain::get_block_explorer_url;

let url = get_block_explorer_url(1)?;
assert_eq!(url, "https://etherscan.io");
```

---

### get_chain_info

**Source**: `src/chain.rs`

```rust
pub fn get_chain_info(chain_id: u64) -> Option<ChainInfo>
```

Get chain information by chain ID

---

### get_supported_chains

**Source**: `src/chain.rs`

```rust
pub fn get_supported_chains() -> Vec<u64>
```

Get list of all supported chain IDs

Scans environment variables for RPC_URL_* patterns and includes
chains with default RPC endpoints.

# Returns
* Vector of supported chain IDs

# Examples
```rust,ignore
use riglr_evm_common::chain::get_supported_chains;

let chains = get_supported_chains();
if chains.contains(&1) {
println!("Ethereum is supported!");
}
```

---

### get_transaction_url

**Source**: `src/chain.rs`

```rust
pub fn get_transaction_url(chain_id: u64, tx_hash: &str) -> EvmResult<String>
```

Get transaction URL for a specific transaction

# Arguments
* `chain_id` - Numeric chain ID
* `tx_hash` - Transaction hash (with or without 0x prefix)

# Returns
* Full URL to view transaction in block explorer

# Examples
```rust,ignore
use riglr_evm_common::chain::get_transaction_url;

let url = get_transaction_url(1, "0x123abc...")?;
// Returns: https://etherscan.io/tx/0x123abc...
```

---

### is_supported_chain

**Source**: `src/chain.rs`

```rust
pub fn is_supported_chain(chain_id: u64) -> bool
```

Check if a chain is supported (has RPC URL configured or has default)

# Arguments
* `chain_id` - Numeric chain ID to check

# Returns
* `true` if chain is supported, `false` otherwise

# Examples
```rust,ignore
use riglr_evm_common::chain::is_supported_chain;

// If RPC_URL_1 is configured or Ethereum has defaults
assert!(is_supported_chain(1));

// Unsupported chain
assert!(!is_supported_chain(999999));
```

---


---

*This documentation was automatically generated from the source code.*