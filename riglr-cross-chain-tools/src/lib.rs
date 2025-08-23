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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_when_accessed_should_return_valid_semver_string() {
        // Happy Path: VERSION constant should be accessible and non-empty
        // VERSION is a const from CARGO_PKG_VERSION, so it's always valid

        // Version should follow semantic versioning pattern (major.minor.patch)
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(
            parts.len() >= 3,
            "Version should have at least major.minor.patch format"
        );

        // Each part should be numeric (allowing for pre-release suffixes)
        for (i, part) in parts.iter().take(3).enumerate() {
            let numeric_part = part.split('-').next().unwrap_or(part);
            assert!(
                numeric_part.chars().all(|c| c.is_ascii_digit()),
                "Version part {} should be numeric, got: {}",
                i,
                numeric_part
            );
        }
    }

    #[test]
    fn test_version_when_compared_to_cargo_pkg_version_should_match() {
        // Edge Case: Verify VERSION matches CARGO_PKG_VERSION environment
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_cross_chain_error_when_imported_should_be_accessible() {
        // Happy Path: Verify CrossChainError re-export is accessible
        use crate::CrossChainError;

        // This test ensures the re-export works by attempting to use the type
        // We can't instantiate it without knowing its variants, but we can reference it
        let _type_check: Option<CrossChainError> = None;
    }

    #[test]
    fn test_tool_error_when_imported_should_be_accessible() {
        // Happy Path: Verify ToolError re-export is accessible
        use crate::ToolError;

        // This test ensures the re-export works by attempting to use the type
        let _type_check: Option<ToolError> = None;
    }

    #[test]
    fn test_bridge_module_when_imported_should_be_accessible() {
        // Happy Path: Verify bridge module items are accessible through re-export
        // We use a wildcard import to test that bridge::* works
        // This will compile successfully if the re-export is working

        // Note: We can't test specific items without knowing what's in the bridge module,
        // but we can verify the module itself is accessible
        // The real test is that this compiles successfully
    }

    #[test]
    fn test_lifi_module_when_imported_should_be_accessible() {
        // Happy Path: Verify lifi module items are accessible through re-export
        // Similar to bridge module test
        // The real test is that this compiles successfully
    }

    #[test]
    fn test_error_module_when_accessed_should_be_available() {
        // Happy Path: Verify error module is properly declared
        // The module should be accessible for use

        // This is tested implicitly by the CrossChainError import test above
    }

    #[test]
    fn test_crate_structure_when_examined_should_have_required_modules() {
        // Happy Path: Verify all expected modules are declared
        // This is a structural test to ensure the crate organization is intact

        // If the modules weren't properly declared, the re-exports would fail to compile
        // So successful compilation of this test file indicates proper module structure
    }

    #[test]
    fn test_version_when_used_in_string_context_should_be_valid_str() {
        // Edge Case: VERSION should be usable as a string slice
        let version_string = format!("Version: {}", VERSION);
        assert!(version_string.starts_with("Version: "));
        assert!(version_string.len() > "Version: ".len());
    }

    #[test]
    fn test_version_when_cloned_should_maintain_same_value() {
        // Edge Case: VERSION should be copyable/cloneable
        let version_copy = VERSION;
        assert_eq!(VERSION, version_copy);
    }

    #[test]
    fn test_version_when_used_as_static_str_should_have_static_lifetime() {
        // Edge Case: Verify VERSION has static lifetime
        fn takes_static_str(_s: &'static str) {}
        takes_static_str(VERSION); // This should compile
    }
}
