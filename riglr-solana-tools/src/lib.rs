//! # riglr-solana-tools
//!
//! A comprehensive suite of rig-compatible tools for interacting with the Solana blockchain.
//!
//! This crate provides ready-to-use tools for building Solana-native AI agents, including:
//!
//! - **Balance Tools**: Check SOL and SPL token balances
//! - **Transaction Tools**: Send SOL and token transfers
//! - **DeFi Tools**: Interact with Jupiter for swaps and quotes
//! - **Pump.fun Tools**: Deploy, buy, and sell tokens on Pump.fun
//! - **Network Tools**: Query blockchain state and transaction details
//!
//! All tools are built with the `#[tool]` macro for seamless integration with rig agents
//! and include comprehensive error handling and retry logic.
//!
//! ## Features
//!
//! - **Production Ready**: Built-in retry logic, timeouts, and error handling
//! - **Type Safe**: Full Rust type safety with serde and schemars integration
//! - **Async First**: Non-blocking operations using tokio
//! - **Composable**: Mix and match tools as needed for your agent
//! - **Well Documented**: Every tool includes usage examples
//!
//! ## Quick Start
//!
//! ```ignore
//! use riglr_solana_tools::balance::get_sol_balance;
//! use riglr_core::provider::{ApplicationContext, RpcProvider};
//! use riglr_core::{ToolWorker, ExecutionConfig, Job, idempotency::InMemoryIdempotencyStore};
//! use riglr_config::Config;
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Set up ApplicationContext with Solana RPC client
//! let config = Config::from_env();
//! let rpc_provider = Arc::new(RpcProvider::new());
//! let context = ApplicationContext::new(rpc_provider, config);
//!
//! // Create and register tools with worker
//! let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
//!     ExecutionConfig::default(),
//!     context
//! );
//!
//! worker.register_tool(Arc::new(get_sol_balance)).await;
//!
//! // Execute the tool
//! let job = Job::new(
//!     "get_sol_balance",
//!     &serde_json::json!({"address": "So11111111111111111111111111111111111111112"}),
//!     3
//! )?;
//!
//! let result = worker.process_job(job).await?;
//! println!("Balance result: {:?}", result);
//! # Ok(())
//! # }
//! ```
//!
//! ## Tool Categories
//!
//! - [`balance`] - Balance checking tools for SOL and SPL tokens
//! - [`transaction`] - Transaction creation and execution tools
//! - [`swap`] - Jupiter DEX integration for token swaps
//! - [`pump`] - Pump.fun integration for meme token deployment and trading
//! - [`network`] - Network state and blockchain query tools

pub mod balance;
pub mod clients;
pub mod common;
pub mod error;
pub mod network;
pub mod pump;
pub mod signer;
pub mod swap;
pub mod transaction;
pub mod utils;

// Re-export commonly used tools
pub use balance::*;
pub use network::*;
pub use pump::*;
pub use signer::*;
pub use swap::*;
pub use transaction::*;

// Re-export specific utilities to avoid ambiguous glob conflicts
pub use utils::{
    execute_solana_transaction, send_transaction, send_transaction_with_retry, validate_address,
    TransactionConfig, TransactionSubmissionResult,
};
// Note: generate_mint_keypair and create_token_with_mint_keypair are not re-exported
// at the top level to avoid conflicts. Use utils::generate_mint_keypair directly.

// Re-export error types
pub use error::SolanaToolError;

// Re-export signer types for convenience
pub use riglr_core::{signer::UnifiedSigner, SignerContext};

/// Current version of riglr-solana-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_valid() {
        // VERSION should be a valid semantic version string
        assert!(!VERSION.is_empty(), "VERSION should not be empty");
        assert!(
            VERSION.contains('.'),
            "VERSION should contain dots (semantic versioning)"
        );
        // Verify it's a proper version format (at least X.Y.Z)
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(
            parts.len() >= 3,
            "VERSION should have at least 3 parts (major.minor.patch)"
        );
        // Each part should be numeric (for basic semantic versioning)
        for (i, part) in parts.iter().take(3).enumerate() {
            // Remove any pre-release or build metadata for the first 3 parts
            let clean_part = part.split('-').next().unwrap().split('+').next().unwrap();
            assert!(
                clean_part.chars().all(|c| c.is_ascii_digit()),
                "VERSION part {} should be numeric, got: {}",
                i,
                clean_part
            );
        }
    }

    #[test]
    fn test_version_constant_accessible() {
        // Test that VERSION constant can be accessed and assigned
        let version_copy = VERSION;
        assert_eq!(version_copy, VERSION);
    }

    #[test]
    fn test_module_declarations_exist() {
        // Verify that all declared modules can be referenced
        // This test ensures the module declarations are valid and the modules exist

        // Test that we can access module paths (compilation test)
        let _balance_module = std::any::type_name::<balance::BalanceResult>();
        let _error_module = std::any::type_name::<error::SolanaToolError>();
        let _network_module = std::any::type_name::<network::get_block_height::Args>();
        let _signer_module = std::any::type_name::<signer::LocalSolanaSigner>();

        // These should compile without errors if modules exist
    }

    #[test]
    fn test_re_exported_types_accessible() {
        // Test that re-exported types from balance module are accessible
        let _get_sol_balance_args = std::any::type_name::<balance::get_sol_balance::Args>();
        let _get_spl_token_balance_args =
            std::any::type_name::<balance::get_spl_token_balance::Args>();

        // Test that re-exported types from network module are accessible
        let _get_block_height_args = std::any::type_name::<network::get_block_height::Args>();
        let _get_transaction_status_args =
            std::any::type_name::<network::get_transaction_status::Args>();

        // Test that re-exported types from transaction module are accessible
        let _transaction_result = std::any::type_name::<utils::TransactionSubmissionResult>();
        let _transaction_config = std::any::type_name::<utils::TransactionConfig>();

        // Test that re-exported types from signer module are accessible
        let _local_solana_signer = std::any::type_name::<signer::LocalSolanaSigner>();
    }

    #[test]
    fn test_utils_re_exports_accessible() {
        // Test that specific utils re-exports are accessible by type name
        let _transaction_config = std::any::type_name::<TransactionConfig>();
        let _transaction_result = std::any::type_name::<TransactionSubmissionResult>();
    }

    #[test]
    fn test_error_re_exports() {
        // Test that error re-exports work
        let _solana_tool_error = std::any::type_name::<SolanaToolError>();
    }

    #[test]
    fn test_riglr_core_re_exports() {
        // Test that riglr-core re-exports are accessible
        let _unified_signer = std::any::type_name::<dyn UnifiedSigner>();
        let _signer_context = std::any::type_name::<SignerContext>();
    }

    #[test]
    fn test_version_matches_cargo_pkg_version() {
        // VERSION should match the CARGO_PKG_VERSION environment variable
        // This is a compile-time guarantee, but we test the behavior
        let version = VERSION;

        // Basic validation that it looks like a version
        assert!(
            !version.is_empty() && version.contains('.'),
            "VERSION should be a non-empty version string with dots"
        );

        // Test that VERSION is a static string
        let version_ref: &'static str = VERSION;
        assert_eq!(version_ref, VERSION);
    }

    #[test]
    fn test_crate_documentation_constants() {
        // Test that the crate has the expected structure based on documentation
        // This validates that the public API matches what's documented

        // These should be accessible as documented in the crate docs
        let _version: &'static str = VERSION;

        // Verify the main re-exported modules exist by checking their types
        let _balance_exists = std::any::type_name::<balance::BalanceResult>();
        let _network_exists = std::any::type_name::<network::get_block_height::Args>();
        let _signer_exists = std::any::type_name::<signer::LocalSolanaSigner>();
        let _error_exists = std::any::type_name::<error::SolanaToolError>();
    }
}
