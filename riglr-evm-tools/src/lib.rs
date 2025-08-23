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

// New module structure following solana-tools pattern
pub mod common;
pub mod utils;

// Existing modules remain unchanged
pub mod balance;
pub mod contract;
pub mod error;
pub mod network;
pub mod signer;
pub mod swap;
pub mod transaction;
pub mod util; // Keep temporarily during migration

// Re-export common functionality at crate root (following solana-tools pattern)
pub use common::{
    // Address utilities
    address::{
        ensure_0x_prefix, format_address, format_address_string, is_checksummed, known_addresses,
        parse_evm_address, strip_0x_prefix, validate_evm_address,
    },
    // Chain mapping
    chain::{
        chain_id_to_name, chain_id_to_rpc_url, chain_name_to_id, get_address_url,
        get_block_explorer_url, get_chain_info, get_supported_chains, get_transaction_url,
        is_supported_chain, ChainInfo,
    },
    // Error types
    error::{EvmCommonError, EvmResult},
    // Core types
    types::{EvmAccount, EvmConfig, EvmToken, EvmTransactionData},
};

// Re-export main functionality
pub use balance::{
    get_erc20_balance, get_eth_balance, get_token_decimals, get_token_name, get_token_symbol,
    EthBalance, TokenBalance,
};
pub use contract::*;
pub use error::EvmToolError;
pub use network::*;
pub use signer::LocalEvmSigner;
pub use swap::{get_uniswap_quote, SwapQuote};
pub use transaction::*;
pub use utils::*;

// Re-export from riglr-core for convenience
pub use riglr_core::{signer::UnifiedSigner, SignerContext};

/// Current version of riglr-evm-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
