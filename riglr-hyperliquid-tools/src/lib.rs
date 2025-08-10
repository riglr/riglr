//! # riglr-hyperliquid-tools
//!
//! A comprehensive suite of rig-compatible tools for interacting with Hyperliquid perpetual futures.
//!
//! This crate provides ready-to-use tools for building derivatives trading AI agents, including:
//!
//! - **Trading Tools**: Place, cancel, and modify perpetual futures orders
//! - **Position Management**: Open, close, and monitor trading positions
//! - **Account Tools**: Query account information, margins, and PnL
//! - **Market Data**: Access real-time market data and statistics
//! - **Risk Management**: Set leverage, manage risk parameters
//!
//! All tools are built with the `#[tool]` macro for seamless integration with rig agents
//! and include comprehensive error handling and retry logic.

pub mod error;

pub mod client;
pub mod positions;
pub mod trading;

// Re-export commonly used tools
pub use positions::*;
pub use trading::*;

// Re-export client types
pub use client::HyperliquidClient;

// Re-export signer types for convenience
pub use riglr_core::{SignerContext, ToolError, signer::TransactionSigner};

/// Current version of riglr-hyperliquid-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}