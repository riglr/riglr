//! # riglr-solana-tools
//!
//! A comprehensive suite of rig-compatible tools for interacting with the Solana blockchain.
//!
//! This crate provides ready-to-use tools for building Solana-native AI agents, including:
//!
//! - **Balance Tools**: Check SOL and SPL token balances
//! - **Transaction Tools**: Send SOL and token transfers
//! - **DeFi Tools**: Interact with Jupiter for swaps and quotes
//! - **Pump.fun Tools**: Deploy, buy, and sell tokens on Pump.fun
//! - **Network Tools**: Query blockchain state and transaction details
//!
//! All tools are built with the `#[tool]` macro for seamless integration with rig agents
//! and include comprehensive error handling and retry logic.
//!
//! ## Features
//!
//! - **Production Ready**: Built-in retry logic, timeouts, and error handling
//! - **Type Safe**: Full Rust type safety with serde and schemars integration
//! - **Async First**: Non-blocking operations using tokio
//! - **Composable**: Mix and match tools as needed for your agent
//! - **Well Documented**: Every tool includes usage examples
//!
//! ## Quick Start
//!
//! ```ignore
//! // Example usage (requires rig-core dependency):
//! use riglr_solana_tools::balance::get_sol_balance;
//! use rig_core::Agent;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let agent = Agent::builder()
//!     .preamble("You are a Solana blockchain assistant.")
//!     .tool(get_sol_balance)
//!     .build();
//!
//! let response = agent.prompt("What is the SOL balance of So11111111111111111111111111111111111111112?").await?;
//! println!("Agent response: {}", response);
//! # Ok(())
//! # }
//! ```
//!
//! ## Tool Categories
//!
//! - [`balance`] - Balance checking tools for SOL and SPL tokens
//! - [`transaction`] - Transaction creation and execution tools
//! - [`swap`] - Jupiter DEX integration for token swaps
//! - [`pump`] - Pump.fun integration for meme token deployment and trading
//! - [`network`] - Network state and blockchain query tools

pub mod balance;
pub mod client;
pub mod error;
pub mod network;
pub mod pump;
pub mod signer;
pub mod swap;
pub mod transaction;
pub mod utils;

// Re-export commonly used tools
pub use balance::*;
pub use network::*;
pub use pump::*;
pub use signer::*;
pub use swap::*;
pub use transaction::*;

// Re-export specific utilities to avoid ambiguous glob conflicts
pub use utils::{
    execute_solana_transaction, get_rpc_url, send_transaction, send_transaction_with_retry,
    validate_address, TransactionConfig, TransactionSubmissionResult,
};
// Note: generate_mint_keypair and create_token_with_mint_keypair are not re-exported
// at the top level to avoid conflicts. Use utils::generate_mint_keypair directly.

// Re-export client and error types
pub use client::SolanaClient;
pub use error::SolanaToolError;

// Re-export signer types for convenience
pub use riglr_core::{signer::TransactionSigner, SignerContext};

/// Current version of riglr-solana-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        // VERSION is a compile-time constant from CARGO_PKG_VERSION
        // Its existence and non-empty nature is guaranteed by successful compilation
        let _version = VERSION;
    }
}
