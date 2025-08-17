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

/// Error types and handling for cross-chain tools
pub mod error;
pub use error::CrossChainError;

/// Core bridge functionality for cross-chain transfers
pub mod bridge;
/// LiFi Protocol integration for multi-provider bridge access
pub mod lifi;

// Re-export main bridge tools
pub use bridge::*;
pub use lifi::*;

// Re-export common error types
pub use riglr_core::ToolError;

/// Version information for the riglr-cross-chain-tools crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
