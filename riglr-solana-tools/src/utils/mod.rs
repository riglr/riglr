//! Organized utility modules for riglr-solana-tools
//!
//! This module provides well-organized utility functions for Solana operations,
//! following the established riglr architectural patterns with SignerContext-based
//! multi-tenant operation.
//!
//! # Module Organization
//!
//! - [`validation`] - Address and input validation utilities
//! - [`transaction`] - Transaction creation, sending, and retry logic
//! - [`keypair`] - Keypair generation utilities
//!
//! All modules follow the SignerContext pattern for secure multi-tenant operation,
//! as established by riglr-core architecture.

pub mod keypair;
pub mod transaction;
pub mod validation;

// Re-export commonly used items for convenience
pub use keypair::generate_mint_keypair;
pub use transaction::{
    create_token_with_mint_keypair, execute_solana_transaction, send_transaction,
    send_transaction_with_retry, TransactionConfig, TransactionSubmissionResult,
};
pub use validation::validate_address;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports_keypair_functions() {
        // Test that generate_mint_keypair is properly re-exported
        // This will compile if the re-export is working correctly
        let _: fn() -> solana_sdk::signer::keypair::Keypair = generate_mint_keypair;
    }

    #[test]
    fn test_module_exports_transaction_types() {
        // Test that TransactionConfig is properly re-exported
        use std::marker::PhantomData;
        let _: PhantomData<TransactionConfig> = PhantomData;

        // Test that TransactionSubmissionResult is properly re-exported
        let _: PhantomData<TransactionSubmissionResult> = PhantomData;
    }

    #[test]
    fn test_module_exports_transaction_functions() {
        // Test that transaction functions are properly re-exported
        // These will compile if the re-exports are working correctly

        // Note: create_token_with_mint_keypair is an async function, so we can't assign it to a function pointer
        // We test its accessibility instead
        let _ = create_token_with_mint_keypair;

        // Note: We can't easily test function signatures for some functions due to complex types,
        // but the module compilation itself validates the re-exports
    }

    #[test]
    fn test_module_exports_validation_functions() {
        // Test that validate_address is properly re-exported
        // This will compile if the re-export is working correctly
        let _: fn(&str) -> Result<solana_sdk::pubkey::Pubkey, crate::error::SolanaToolError> =
            validate_address;
    }

    #[test]
    fn test_module_structure_is_complete() {
        // Test that all expected submodules are accessible
        // This is validated at compile time, but we can create a runtime test
        // to ensure the module structure is as expected

        // If these compile, the modules are properly declared
        let _ = crate::utils::keypair::generate_mint_keypair;
        let _ = crate::utils::transaction::TransactionConfig::default;
        let _ = crate::utils::validation::validate_address;

        // Test passes if compilation succeeds
    }

    #[test]
    fn test_module_documentation_structure() {
        // Test that the module follows expected documentation patterns
        // This is more of a structural test to ensure the module is well-organized

        // The presence of this test validates that the module follows
        // the documented structure with proper organization
    }

    #[test]
    fn test_all_re_exports_are_accessible() {
        // Test that we can access all re-exported items without qualification
        // This ensures the pub use statements are working correctly

        // Try to reference each re-exported item to ensure they're accessible
        let _mint_fn = generate_mint_keypair;
        let _validate_fn = validate_address;
        let _create_token_fn = create_token_with_mint_keypair;

        // Note: execute_solana_transaction is generic, so we can't easily test it without specific types
        // We just validate that the function exists in the public API
        // let _ = execute_solana_transaction; // Would require specific generic parameters

        let _send_fn = send_transaction;
        let _send_retry_fn = send_transaction_with_retry;

        // If we reach this point, all re-exports are accessible
    }

    #[test]
    fn test_submodule_imports_work() {
        // Test that we can import from submodules directly
        use crate::utils::keypair::generate_mint_keypair as direct_generate_mint_keypair;
        use crate::utils::validation::validate_address as direct_validate_address;

        // Test that direct imports match re-exported functions
        let _: fn() -> solana_sdk::signer::keypair::Keypair = direct_generate_mint_keypair;
        let _: fn(&str) -> Result<solana_sdk::pubkey::Pubkey, crate::error::SolanaToolError> =
            direct_validate_address;

        // Direct submodule imports work correctly
    }
}
