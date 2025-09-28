//! # riglr-evm-tools
//!
//! A comprehensive suite of tools for interacting with EVM-compatible blockchains.
//!
//! This crate provides ready-to-use tools for building EVM-native AI agents, including:
//!
//! - **Balance Tools**: Check ETH and ERC20 token balances
//! - **Transaction Tools**: Send ETH and token transfers
//! - **`DeFi` Tools**: Interact with Uniswap, Sushiswap, and other DEXs
//! - **Contract Tools**: Deploy and interact with smart contracts
//! - **Network Tools**: Query blockchain state and transaction details
//!
//! All tools are built with the `#[tool]` macro for seamless integration with rig agents
//! and include comprehensive error handling and retry logic.

// Existing modules remain unchanged
pub mod balance;
pub mod contract;
pub mod error;
pub mod network;
pub mod provider;
pub mod signer;
pub mod swap;
pub mod transaction;

// Import and re-export common functionality from riglr-evm-common
pub use riglr_evm_common::{
    // Address utilities
    ensure_0x_prefix,
    // Conversion utilities
    eth_to_wei,
    format,
    // Formatting utilities
    format_gas_price_gwei,
    format_string,
    format_token_amount,
    format_wei_to_eth,
    get_address_url,
    get_block_explorer_url,
    get_info as get_chain_info,
    get_supported as get_supported_chains,
    get_transaction_url,
    gwei_to_wei,
    // Chain mapping
    id_to_name as chain_id_to_name,
    id_to_rpc_url as chain_id_to_rpc_url,
    is_checksummed,
    is_supported as is_supported_chain,
    known_addresses,
    name_to_id as chain_name_to_id,
    parse,
    smallest_unit_to_token,
    strip_0x_prefix,
    token_to_smallest_unit,
    truncate_address,
    validate,
    // Validation utilities
    validate_chain_id,
    validate_gas_params,
    wei_to_eth,
    wei_to_gwei,
    // Core types
    EvmAccount,
    EvmAddressValidator,
    // Error types
    EvmCommonError,
    EvmConfig,
    EvmResult,
    EvmToken,
    EvmTransactionData,
    Info as ChainInfo,
};

// Re-export main functionality
pub use balance::{
    check_erc20_token, check_eth, get_token_decimals, get_token_name, get_token_symbol, Balance,
    Token,
};
pub use contract::*;
pub use error::Error as EvmToolError;
pub use network::*;
pub use provider::{create_http_client, execute_evm_transaction, HttpClient};
pub use signer::EvmLocalClient;
pub use swap::{get_uniswap_quote, Parameters as SwapParameters, Quote as SwapQuote};
pub use transaction::*;

// Re-export from riglr-core for convenience
pub use riglr_core::{signer::UnifiedSigner, SignerContext};

// Backward compatibility aliases
pub use format as format_address;
pub use format_string as format_address_string;
pub use parse as parse_evm_address;
pub use validate as validate_evm_address;

/// Current version of riglr-evm-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
