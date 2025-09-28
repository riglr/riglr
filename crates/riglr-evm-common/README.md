# riglr-evm-common

Common EVM utilities shared across the Riglr workspace.

## Overview

This crate provides a single source of truth for EVM-related functionality that is shared between multiple crates in the Riglr workspace. It eliminates code duplication and ensures consistent behavior across all EVM operations.

## Features

- **Address utilities**: Validation, parsing, formatting, and checksumming of EVM addresses
- **Chain management**: Chain ID to name mapping, RPC URL resolution, block explorer URLs
- **Type conversions**: Wei/ETH/Gwei conversions, token amount formatting
- **Common types**: Standardized EVM configuration, account, token, and transaction data structures
- **Error handling**: Unified error types for EVM operations
- **Validation utilities**: Chain ID and gas parameter validation

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
riglr-evm-common = { path = "../riglr-evm-common" }
```

Then use the utilities in your code:

```rust
use riglr_evm_common::{
    parse_evm_address,
    validate_evm_address,
    eth_to_wei,
    chain_id_to_name,
    EvmConfig,
};

// Parse and validate addresses
let address = parse_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb5")?;

// Convert between units
let wei_amount = eth_to_wei("1.5")?;

// Get chain information
let chain_name = chain_id_to_name(1)?; // "Ethereum"
```

## Architecture

This crate is designed to be:
- **Dependency-free from riglr-core**: Ensures no circular dependencies
- **Provider-agnostic**: Works with any Alloy provider implementation
- **Type-safe**: Leverages Rust's type system for safety
- **Well-tested**: Comprehensive test coverage for all utilities

## License

MIT OR Apache-2.0