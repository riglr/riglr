//! # riglr-evm-tools
//!
//! A comprehensive suite of tools for interacting with EVM-compatible blockchains.
//!
//! This crate provides ready-to-use tools for building EVM-native AI agents, including:
//!
//! - **Balance Tools**: Check ETH and ERC20 token balances
//! - **Transaction Tools**: Send ETH and token transfers
//! - **DeFi Tools**: Interact with Uniswap, Sushiswap, and other DEXs
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
    // Chain mapping
    chain_id_to_name,
    chain_id_to_rpc_url,
    chain_name_to_id,
    // Address utilities
    ensure_0x_prefix,
    // Conversion utilities
    eth_to_wei,
    format_address,
    format_address_string,
    // Formatting utilities
    format_gas_price_gwei,
    format_token_amount,
    format_wei_to_eth,
    get_address_url,
    get_block_explorer_url,
    get_chain_info,
    get_supported_chains,
    get_transaction_url,
    gwei_to_wei,
    is_checksummed,
    is_supported_chain,
    known_addresses,
    parse_evm_address,
    smallest_unit_to_token,
    strip_0x_prefix,
    token_to_smallest_unit,
    truncate_address,
    // Validation utilities
    validate_chain_id,
    validate_evm_address,
    validate_gas_params,
    wei_to_eth,
    wei_to_gwei,
    ChainInfo,
    // Core types
    EvmAccount,
    // Error types
    EvmCommonError,
    EvmConfig,
    EvmResult,
    EvmToken,
    EvmTransactionData,
};

// Re-export main functionality
pub use balance::{
    get_erc20_balance, get_eth_balance, get_token_decimals, get_token_name, get_token_symbol,
    EthBalance, TokenBalance,
};
pub use contract::*;
pub use error::EvmToolError;
pub use network::*;
pub use provider::{execute_evm_transaction, make_provider, EvmProvider};
pub use signer::LocalEvmSigner;
pub use swap::{get_uniswap_quote, SwapQuote};
pub use transaction::*;

// Re-export from riglr-core for convenience
pub use riglr_core::{signer::UnifiedSigner, SignerContext};

/// Current version of riglr-evm-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
