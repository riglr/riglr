//! # riglr-asterdex
//!
//! A comprehensive suite of rig-compatible tools for interacting with Asterdex Futures v3 API.
//!
//! This crate provides ready-to-use tools for building derivatives trading AI agents, including:
//!
//! - **Trading Tools**: Place, cancel, and query perpetual futures orders
//! - **Position Management**: Monitor and manage trading positions
//! - **Account Tools**: Query account information, balances, and positions
//! - **Leverage Management**: Adjust leverage for trading pairs
//! - **v3 Authentication**: Web3-based signature authentication for secure API access
//!
//! All tools are built with the `#[tool]` macro for seamless integration with rig agents
//! and include comprehensive error handling and retry logic.

/// HTTP client for Asterdex API interactions with v3 authentication
pub mod client;
/// Error types and handling for Asterdex tools
pub mod error;

pub use error::Error;
/// Position management and account tools
pub mod positions;
/// Trading tools for order placement and management
pub mod trading;

// Re-export commonly used tools
pub use positions::*;
pub use trading::*;

// Re-export client types
pub use client::Client;

// Re-export signer types for convenience
pub use riglr_core::{signer::UnifiedSigner, SignerContext, ToolError};

/// Current version of riglr-asterdex
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant_is_accessible() {
        // VERSION should follow semantic versioning pattern (major.minor.patch)
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(
            parts.len() >= 2,
            "VERSION should have at least major.minor format, got: {VERSION}"
        );

        // Each part should be numeric
        for part in parts {
            // Handle pre-release versions by taking only the numeric portion before any '-'
            let cleaned_part = part.split('-').next().unwrap_or(part);
            assert!(
                cleaned_part.parse::<u32>().is_ok(),
                "VERSION part '{cleaned_part}' should be numeric in version: {VERSION}"
            );
        }
    }

    #[test]
    fn test_error_is_accessible() {
        // Verify that Error is properly re-exported
        use crate::Error;

        // Test that we can create instances of different error variants
        let api_error = Error::ApiError("test".to_string());
        assert!(matches!(api_error, Error::ApiError(_)));

        let symbol_error = Error::InvalidSymbol("INVALID".to_string());
        assert!(matches!(symbol_error, Error::InvalidSymbol(_)));
    }

    #[test]
    fn test_client_is_accessible() {
        // Verify that Client is properly re-exported from client module
        use crate::Client;

        // Just ensure the type exists and is accessible
        let _: Option<Client> = None;
    }
}
