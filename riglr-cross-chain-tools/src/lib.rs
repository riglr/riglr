//! # riglr-cross-chain-tools
//!
//! Cross-chain bridging tools for riglr, enabling seamless token transfers between different blockchain networks.
//!
//! This crate provides stateless tools for discovering cross-chain routes, executing bridge transactions,
//! and monitoring transfer status. It integrates with LiFi Protocol to provide access to multiple bridge
//! providers and DEX aggregators across EVM and Solana networks.
//!
//! ## Features
//!
//! - **Route Discovery**: Find optimal paths for cross-chain token transfers
//! - **Bridge Execution**: Execute cross-chain transfers with automatic transaction signing
//! - **Status Monitoring**: Track bridge transaction progress and completion
//! - **Fee Estimation**: Calculate costs and estimated completion times
//! - **Multi-Chain Support**: Works with both EVM chains and Solana
//! - **Stateless Design**: Tools work with riglr's SignerContext pattern for secure multi-tenant operation

pub mod error;
pub use error::CrossChainToolError;

pub mod bridge;
pub mod lifi;

// Re-export main bridge tools
pub use bridge::*;
pub use lifi::*;

// Re-export common error types
pub use riglr_core::ToolError;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");