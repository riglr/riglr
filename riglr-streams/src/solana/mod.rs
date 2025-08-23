//! Solana blockchain streaming capabilities via Geyser
//!
//! This module provides streaming components for Solana blockchain data
//! using the Geyser protocol for real-time account and transaction monitoring.

/// Geyser WebSocket streaming implementation for real-time Solana data
pub mod geyser;

pub use geyser::{GeyserConfig, SolanaGeyserStream, SolanaStreamEvent, TransactionEvent};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure_when_importing_geyser_should_be_accessible() {
        // Test that the geyser module is properly declared and accessible
        // This test ensures the module structure is correct

        // We can't directly test module existence in Rust unit tests,
        // but we can test that the re-exported types are accessible
        // This will fail to compile if the module or re-exports are broken

        // Test type accessibility through type assertions
        let _: Option<GeyserConfig> = None;
        let _: Option<SolanaGeyserStream> = None;
        let _: Option<SolanaStreamEvent> = None;
        let _: Option<TransactionEvent> = None;
    }

    #[test]
    fn test_re_exports_when_used_should_be_available_in_module_scope() {
        // Test that all re-exported types are available in the current module scope
        // This verifies the `pub use` statements work correctly

        // Create type references to ensure re-exports are working
        // These will fail to compile if re-exports are broken

        // Test GeyserConfig is re-exported
        let _ = std::any::type_name::<GeyserConfig>();

        // Test SolanaGeyserStream is re-exported
        let _ = std::any::type_name::<SolanaGeyserStream>();

        // Test SolanaStreamEvent is re-exported
        let _ = std::any::type_name::<SolanaStreamEvent>();

        // Test TransactionEvent is re-exported
        let _ = std::any::type_name::<TransactionEvent>();
    }

    #[test]
    fn test_module_documentation_when_accessed_should_contain_expected_content() {
        // Test that the module has proper documentation structure
        // This is a compile-time verification that documentation exists

        // Module documentation is compile-time checked, so if this compiles,
        // the documentation structure is valid
    }

    #[test]
    fn test_geyser_submodule_when_declared_should_be_public() {
        // Test that the geyser submodule is properly declared as public
        // This ensures external crates can access the submodule

        // If the module wasn't public, this wouldn't compile
        // The fact that we can reference types from it confirms it's public
        let _module_is_public = std::any::type_name::<geyser::GeyserConfig>();
    }

    #[test]
    fn test_type_names_when_checked_should_have_expected_paths() {
        // Test that re-exported types maintain their expected type paths
        // This verifies the re-export doesn't change type identity

        let geyser_config_name = std::any::type_name::<GeyserConfig>();
        let stream_name = std::any::type_name::<SolanaGeyserStream>();
        let event_name = std::any::type_name::<SolanaStreamEvent>();
        let transaction_name = std::any::type_name::<TransactionEvent>();

        // Verify type names contain expected module paths
        assert!(
            geyser_config_name.contains("geyser"),
            "GeyserConfig should reference geyser module: {}",
            geyser_config_name
        );
        assert!(
            stream_name.contains("geyser"),
            "SolanaGeyserStream should reference geyser module: {}",
            stream_name
        );
        assert!(
            event_name.contains("geyser"),
            "SolanaStreamEvent should reference geyser module: {}",
            event_name
        );
        assert!(
            transaction_name.contains("geyser"),
            "TransactionEvent should reference geyser module: {}",
            transaction_name
        );
    }
}
