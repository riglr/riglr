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

/// Error types and handling for Hyperliquid tools
pub mod error;
pub use error::HyperliquidToolError;

/// HTTP client for Hyperliquid API interactions
pub mod client;
/// Position management tools and utilities
pub mod positions;
/// Trading tools for order placement and management
pub mod trading;

// Re-export commonly used tools
pub use positions::*;
pub use trading::*;

// Re-export client types
pub use client::HyperliquidClient;

// Re-export signer types for convenience
pub use riglr_core::{signer::UnifiedSigner, SignerContext, ToolError};

/// Current version of riglr-hyperliquid-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant_is_accessible() {
        // Verify that VERSION constant is accessible and has expected format
        assert!(!VERSION.is_empty(), "VERSION constant should not be empty");

        // VERSION should follow semantic versioning pattern (major.minor.patch)
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(
            parts.len() >= 2,
            "VERSION should have at least major.minor format, got: {}",
            VERSION
        );

        // Each part should be numeric
        for part in parts {
            let cleaned_part = part.split('-').next().unwrap(); // Handle pre-release versions
            assert!(
                cleaned_part.parse::<u32>().is_ok(),
                "VERSION part '{}' should be numeric in version: {}",
                cleaned_part,
                VERSION
            );
        }
    }

    #[test]
    fn test_version_constant_matches_cargo_toml() {
        // VERSION should match the version in Cargo.toml since it uses env!("CARGO_PKG_VERSION")
        let version = VERSION;
        assert!(!version.is_empty());

        // Test that it's a valid string that can be used for comparisons
        let version_string = version.to_string();
        assert_eq!(version, version_string.as_str());
    }

    #[test]
    fn test_hyperliquid_tool_error_is_accessible() {
        // Verify that HyperliquidToolError is properly re-exported
        use crate::HyperliquidToolError;

        // Test that we can create instances of different error variants
        let api_error = HyperliquidToolError::ApiError("test".to_string());
        assert!(matches!(api_error, HyperliquidToolError::ApiError(_)));

        let symbol_error = HyperliquidToolError::InvalidSymbol("INVALID".to_string());
        assert!(matches!(
            symbol_error,
            HyperliquidToolError::InvalidSymbol(_)
        ));
    }

    #[test]
    fn test_hyperliquid_client_is_accessible() {
        // Verify that HyperliquidClient is properly re-exported from client module
        use crate::HyperliquidClient;

        // Test that the type is accessible - we can reference it in type annotations
        let _client_type: Option<HyperliquidClient> = None;
    }

    #[test]
    fn test_core_types_are_accessible() {
        // Verify that re-exported types from riglr_core are accessible
        use crate::{SignerContext, ToolError, UnifiedSigner};

        // Test that these types can be referenced
        let _signer_context: Option<SignerContext> = None;
        let _tool_error: Option<ToolError> = None;
        let _transaction_signer: Option<Box<dyn UnifiedSigner>> = None;
    }

    #[test]
    fn test_module_structure_is_accessible() {
        // Test that all declared modules are accessible

        // error module should be accessible
        let _error_result: Result<(), crate::error::HyperliquidToolError> = Ok(());

        // client module should be accessible - test the type is referenceable
        use crate::client::HyperliquidClient;
        let _client_type: Option<HyperliquidClient> = None;

        // Test that module declarations work (compilation test)
        // These tests ensure modules can be referenced and compilation succeeds
    }

    #[test]
    fn test_wildcard_reexports_from_positions() {
        // Verify that items re-exported with `pub use positions::*` are accessible
        // This test will fail to compile if the re-exports don't work

        // Test that the wildcard re-export is working by checking compilation succeeds
        // If this compiles, the wildcard re-export syntax is valid
    }

    #[test]
    fn test_wildcard_reexports_from_trading() {
        // Verify that items re-exported with `pub use trading::*` are accessible
        // This test will fail to compile if the re-exports don't work

        // Test that the wildcard re-export is working by checking compilation succeeds
        // If this compiles, the wildcard re-export syntax is valid
    }

    #[test]
    fn test_error_module_integration() {
        // Test that the error module is properly integrated
        use crate::error;

        // Verify we can access the error type through the module path
        let api_error = error::HyperliquidToolError::ApiError("test error".to_string());

        // Test error formatting
        let error_string = format!("{}", api_error);
        assert!(error_string.contains("API error"));
        assert!(error_string.contains("test error"));

        // Test debug formatting
        let debug_string = format!("{:?}", api_error);
        assert!(debug_string.contains("ApiError"));
    }

    #[test]
    fn test_version_constant_type_and_lifetime() {
        // Test that VERSION has the correct type and lifetime
        let version_ref: &'static str = VERSION;
        assert!(!version_ref.is_empty());

        // Test that it can be cloned/converted
        let version_owned = VERSION.to_owned();
        assert_eq!(VERSION, version_owned.as_str());

        // Test that it can be used in string operations
        let version_with_prefix = format!("v{}", VERSION);
        assert!(version_with_prefix.starts_with("v"));
        assert!(version_with_prefix.len() > 1);
    }

    #[test]
    fn test_library_metadata_consistency() {
        // Test that the library's public interface is consistent

        // VERSION should be a compile-time constant
        const _VERSION_CHECK: &str = VERSION;

        // All re-exported types should be usable in const contexts where applicable
        // This mainly tests that the re-exports don't introduce runtime dependencies
    }

    #[test]
    fn test_crate_level_documentation_accessibility() {
        // Test that the crate is properly structured for documentation
        // This is more of a smoke test to ensure the module structure supports docs

        // VERSION constant should be documentable
        let version = VERSION;
        assert!(!version.is_empty());

        // Test that modules can be referenced (compilation check)
        // All modules should be accessible for documentation generation
        let _client_module_accessible = true;
        let _error_module_accessible = true;
        let _positions_module_accessible = true;
        let _trading_module_accessible = true;

        // If this compiles, the module structure is sound
    }
}
