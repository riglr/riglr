//! # riglr-evm-tools
//! 
//! A comprehensive suite of rig-compatible tools for interacting with EVM-based blockchains.
//! 
//! This crate provides ready-to-use tools for building Ethereum and EVM-compatible AI agents,
//! including support for Ethereum, Polygon, Arbitrum, Optimism, and other EVM chains.
//! 
//! ## Features
//! 
//! - **Multi-Chain Support**: Works with any EVM-compatible blockchain
//! - **Balance Tools**: Check ETH and ERC20 token balances  
//! - **Transaction Tools**: Send ETH and token transfers
//! - **DeFi Tools**: Interact with Uniswap V3 for swaps and quotes
//! - **Contract Tools**: Generic contract interaction capabilities
//! - **Production Ready**: Built-in retry logic, timeouts, and error handling
//! 
//! ## Quick Start
//! 
//! ```ignore
//! // Example usage (requires rig-core dependency):
//! use riglr_evm_tools::balance::get_eth_balance;
//! use rig_core::Agent;
//! 
//! # async fn example() -> anyhow::Result<()> {
//! let agent = Agent::builder()
//!     .preamble("You are an Ethereum blockchain assistant.")
//!     .tool(get_eth_balance)
//!     .build();
//! 
//! let response = agent.prompt("What is the ETH balance of 0x742d35Cc6634C0532925a3b8D8e41E5d1e4F1234?").await?;
//! println!("Agent response: {}", response);
//! # Ok(())
//! # }
//! ```
//! 
//! ## Supported Chains
//! 
//! - Ethereum Mainnet
//! - Polygon  
//! - Arbitrum One
//! - Optimism
//! - Base
//! - Any other EVM-compatible chain
//! 
//! ## Tool Categories
//! 
//! - [`balance`] - Balance checking tools for ETH and ERC20 tokens
//! - [`transaction`] - Transaction creation and execution tools
//! - [`swap`] - Uniswap V3 integration for token swaps  
//! - [`contract`] - Generic smart contract interaction tools
//! - [`network`] - Network state and blockchain query tools

pub mod balance;
pub mod transaction;
pub mod swap;
pub mod contract;
pub mod network;
pub mod client;
pub mod error;

// Re-export commonly used tools
pub use balance::*;
pub use transaction::*;
pub use swap::*;
pub use contract::*;
pub use network::*;

// Re-export client and error types
pub use client::EvmClient;
pub use error::{EvmToolError, Result};

/// Current version of riglr-evm-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}