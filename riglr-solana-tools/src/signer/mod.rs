//! Solana-specific signer implementations.
//!
//! This module contains signer implementations that are specific to the Solana blockchain,
//! providing concrete implementations of the `UnifiedSigner` trait from `riglr-core`.

/// Local Solana signer implementation that manages keypairs in memory.
pub mod local;

pub use local::LocalSolanaSigner;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_module_exists() {
        // Test that the local module is accessible
        // This ensures the module declaration is correct
        let _module_exists = std::any::type_name::<local::LocalSolanaSigner>();
        assert!(_module_exists.contains("LocalSolanaSigner"));
    }

    #[test]
    fn test_local_solana_signer_re_export() {
        // Test that LocalSolanaSigner is properly re-exported
        // This verifies the `pub use` statement works correctly
        let type_name = std::any::type_name::<LocalSolanaSigner>();
        assert!(type_name.contains("LocalSolanaSigner"));

        // Verify it's the same type as the one from the local module
        let local_type_name = std::any::type_name::<local::LocalSolanaSigner>();
        assert_eq!(type_name, local_type_name);
    }

    #[test]
    fn test_module_structure_accessibility() {
        // Test that we can access the module structure correctly
        // This ensures all public items are accessible as expected

        // Create a dummy keypair for testing module accessibility
        let keypair = solana_sdk::signature::Keypair::new();
        let rpc_url = "https://api.devnet.solana.com".to_string();

        // Test that we can create a LocalSolanaSigner through the re-export
        let _signer = LocalSolanaSigner::new(keypair, rpc_url);

        // Test passes if we can construct the type without compilation errors
    }

    #[test]
    fn test_module_documentation_accessible() {
        // Test that module items maintain their documentation and structure
        // This is a compile-time verification that the module organization is correct

        // Verify the re-exported type has the same interface
        let keypair = solana_sdk::signature::Keypair::new();
        let rpc_url = "https://api.devnet.solana.com".to_string();
        let signer = LocalSolanaSigner::new(keypair, rpc_url);

        // Test that methods are accessible through the re-export
        let _rpc_url = signer.rpc_url();
        let _keypair = signer.keypair();
    }

    #[test]
    fn test_from_seed_phrase_through_reexport() {
        // Test that static methods work through the re-export
        let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let rpc_url = "https://api.devnet.solana.com".to_string();

        // This should work through the re-export
        let result = LocalSolanaSigner::from_seed_phrase(seed_phrase, rpc_url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_seed_phrase_invalid_seed_through_reexport() {
        // Test error handling through the re-export
        let invalid_seed = "invalid seed phrase";
        let rpc_url = "https://api.devnet.solana.com".to_string();

        let result = LocalSolanaSigner::from_seed_phrase(invalid_seed, rpc_url);
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(err.to_string().contains("Invalid seed phrase"));
        }
    }

    #[test]
    fn test_debug_implementation_through_reexport() {
        // Test that Debug trait is accessible through re-export
        let keypair = solana_sdk::signature::Keypair::new();
        let rpc_url = "https://api.devnet.solana.com".to_string();
        let signer = LocalSolanaSigner::new(keypair, rpc_url);

        let debug_output = format!("{:?}", signer);
        assert!(debug_output.contains("LocalSolanaSigner"));
        assert!(debug_output.contains("pubkey"));
        assert!(debug_output.contains("rpc_url"));

        // Ensure sensitive data is not exposed
        assert!(!debug_output.contains("keypair"));
        assert!(!debug_output.contains("client"));
    }

    #[test]
    fn test_trait_implementations_accessible() {
        // Test that trait implementations are accessible through the re-export
        use riglr_core::signer::{SolanaSigner, UnifiedSigner};

        let keypair = solana_sdk::signature::Keypair::new();
        let rpc_url = "https://api.devnet.solana.com".to_string();
        let signer = LocalSolanaSigner::new(keypair, rpc_url);

        // Test SolanaSigner trait methods are accessible
        let address = signer.address();
        assert!(!address.is_empty());

        let pubkey = signer.pubkey();
        assert!(!pubkey.is_empty());

        // Verify they return the same value
        assert_eq!(address, pubkey);

        // Test Solana client access
        let client = signer.client();
        assert!(std::sync::Arc::strong_count(&client) > 0);

        // Test that Solana signer doesn't support EVM
        assert!(signer.supports_solana());
        assert!(!signer.supports_evm());
        assert!(signer.as_solana().is_some());
        assert!(signer.as_evm().is_none());
    }
}
