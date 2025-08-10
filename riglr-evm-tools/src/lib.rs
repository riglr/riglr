//! EVM tools for interacting with Ethereum and EVM-compatible chains
//!
//! This crate provides a suite of tools for interacting with EVM-based blockchains,
//! including balance checking, transactions, and DeFi operations.

pub mod balance;
pub mod client;
pub mod error;
pub mod swap;
pub mod transaction;

pub use balance::{get_eth_balance, get_erc20_balance, BalanceResult, TokenBalanceResult};
pub use client::{EvmClient, EvmConfig, validate_address, wei_to_eth, eth_to_wei};
pub use error::{EvmToolError, Result};
pub use swap::{get_uniswap_quote, perform_uniswap_swap, UniswapQuote, UniswapSwapResult, UniswapConfig};
pub use transaction::{
    transfer_eth, transfer_erc20, get_transaction_receipt,
    TransactionResult
};

// Re-export signer types for convenience
pub use riglr_core::{SignerContext, signer::TransactionSigner};

/// Current version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(VERSION, "0.1.0");
    }
}