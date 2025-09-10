# riglr-evm-common

{{#include ../../../riglr-evm-common/README.md}}

## API Reference

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Functions](#functions)
- [Type Aliases](#type-aliases)
- [Constants](#constants)

### Structs

> Core data structures and types.

#### `ChainInfo`

Chain information structure

---

#### `EvmAccount`

Common EVM account metadata

---

#### `EvmAddressValidator`

EVM address validator that implements the AddressValidator trait

This allows EVM address validation to be used with the riglr-config
validation system without creating tight coupling between the config
and blockchain-specific crates.

---

#### `EvmConfig`

Configuration for EVM operations shared across crates

---

#### `EvmToken`

Token information for ERC20 and native tokens

---

#### `EvmTransactionData`

EVM transaction data for cross-chain operations

---

### Enums

> Enumeration types for representing variants.

#### `EvmCommonError`

Error types for EVM operations shared across crates

**Variants:**

- `InvalidAddress`
  - Invalid EVM address format
- `UnsupportedChain`
  - Unsupported or unconfigured chain
- `InvalidChainName`
  - Invalid chain name
- `ProviderError`
  - RPC provider error
- `InvalidConfig`
  - Configuration validation error
- `InvalidData`
  - Invalid transaction data
- `NetworkError`
  - Network connection error
- `ParseError`
  - Parsing error

---

### Functions

> Standalone functions and utilities.

#### `chain_id_to_name`

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

#### `chain_id_to_rpc_url`

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

#### `chain_name_to_id`

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

#### `ensure_0x_prefix`

Ensure address has 0x prefix

# Arguments
* `address` - Address string (with or without 0x prefix)

# Returns
* Address string guaranteed to have 0x prefix

# Examples
```rust,ignore
use riglr_evm_tools::common::address::ensure_0x_prefix;

assert_eq!(ensure_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
assert_eq!(ensure_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
```

---

#### `eth_to_wei`

Convert ETH to wei (high-precision Decimal version)

---

#### `format_address`

Format an address for display with EIP-55 checksumming

Converts any valid address to its checksummed representation
for safe display and copy-paste operations.

# Arguments
* `address` - Alloy Address to format

# Returns
* Checksummed address string with 0x prefix

# Examples
```rust,ignore
use riglr_evm_tools::common::address::{parse_evm_address, format_address};

let addr = parse_evm_address("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")?;
let formatted = format_address(&addr);
assert_eq!(formatted, "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
```

---

#### `format_address_string`

Format an address string with EIP-55 checksumming

Convenience function that parses and formats in one step.

# Arguments
* `address` - Address string to format

# Returns
* Checksummed address string

# Examples
```rust,ignore
use riglr_evm_tools::common::address::format_address_string;

let formatted = format_address_string("742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")?;
assert_eq!(formatted, "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
```

---

#### `format_gas_price_gwei`

Format gas price in gwei

---

#### `format_token_amount`

Format token amount with decimals

---

#### `format_wei_to_eth`

Format wei amount to ETH with specified decimal places

---

#### `get_address_url`

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

#### `get_block_explorer_url`

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

#### `get_chain_info`

Get chain information by chain ID

---

#### `get_supported_chains`

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

#### `get_transaction_url`

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

#### `gwei_to_wei`

Convert gwei to wei (high-precision Decimal version)

---

#### `is_checksummed`

Check if an address string is already checksummed according to EIP-55

# Arguments
* `address` - Address string to check

# Returns
* `true` if address is properly checksummed, `false` if not checksummed or invalid

# Examples
```rust,ignore
use riglr_evm_tools::common::address::is_checksummed;

assert!(is_checksummed("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"));
assert!(!is_checksummed("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e"));
assert!(!is_checksummed("invalid"));
```

---

#### `is_supported_chain`

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

#### `parse_evm_address`

Parse an EVM address string into an Alloy Address type

Handles both checksummed and non-checksummed addresses.
Automatically adds 0x prefix if missing.

# Arguments
* `address` - Address string to parse

# Returns
* `Address` - Parsed Alloy address type

# Examples
```rust,ignore
use riglr_evm_tools::common::address::parse_evm_address;

let addr = parse_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")?;
let addr2 = parse_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")?;
assert_eq!(addr, addr2);
```

---

#### `smallest_unit_to_token`

Convert smallest unit to token amount based on decimals (high-precision Decimal version)

---

#### `strip_0x_prefix`

Extract the address portion without 0x prefix

# Arguments
* `address` - Address string (with or without 0x prefix)

# Returns
* Address hex string without 0x prefix

# Examples
```rust,ignore
use riglr_evm_tools::common::address::strip_0x_prefix;

assert_eq!(strip_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
assert_eq!(strip_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
```

---

#### `token_to_smallest_unit`

Convert token amount to smallest unit based on decimals (high-precision Decimal version)

---

#### `truncate_address`

Truncate address for display

---

#### `validate_chain_id`

Validate chain ID is supported

---

#### `validate_evm_address`

Validate an EVM address string format

Accepts both checksummed and non-checksummed addresses.
Returns error for invalid formats or lengths.

# Arguments
* `address` - Address string to validate (with or without 0x prefix)

# Examples
```rust,ignore
use riglr_evm_tools::common::address::validate_evm_address;

// Valid addresses
assert!(validate_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());
assert!(validate_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());

// Invalid addresses
assert!(validate_evm_address("invalid").is_err());
assert!(validate_evm_address("0x123").is_err());
```

---

#### `validate_gas_params`

Validate gas parameters

---

#### `wei_to_eth`

Convert wei to ETH (high-precision Decimal version)

---

#### `wei_to_gwei`

Convert wei to gwei (high-precision Decimal version)

---

### Type Aliases

#### `EvmResult`

Result type alias for EVM operations

**Type:** `<T, >`

---

### Constants

#### `BURN_ADDRESS`

Burn address (0xdead)

**Type:** `&str`

---

#### `USDC_ETHEREUM`

USDC address on Ethereum mainnet

**Type:** `&str`

---

#### `USDT_ETHEREUM`

USDT address on Ethereum mainnet

**Type:** `&str`

---

#### `WETH_ETHEREUM`

WETH address on Ethereum mainnet

**Type:** `&str`

---

#### `ZERO_ADDRESS`

Zero address (0x0)

**Type:** `&str`

---
