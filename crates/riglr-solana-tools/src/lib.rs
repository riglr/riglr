//! # riglr-solana-tools
//!
//! A comprehensive suite of rig-compatible tools for interacting with the Solana blockchain.

// Allow blanket clippy restriction lints since we enable them via command line
//!
//! This crate provides ready-to-use tools for building Solana-native AI agents, including:
//!
//! - **Balance Tools**: Check SOL and SPL token balances
//! - **Transaction Tools**: Send SOL and token transfers
//! - **`DeFi` Tools**: Interact with Jupiter for swaps and quotes
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
//! use riglr_core::provider::ApplicationContext;
//! use riglr_core::{ToolWorker, ExecutionConfig, Job, idempotency::InMemoryIdempotencyStore};
//! use riglr_config::Config;
//! use solana_client::rpc_client::RpcClient;
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Set up ApplicationContext with Solana RPC client
//! let config = Config::from_env();
//! let context = ApplicationContext::from_config(&config);
//! let solana_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com"));
//! context.set_extension(solana_client);
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
pub mod common_conversions;
pub mod common_newtypes;
pub mod common_types;
pub mod common_utils;
pub mod error;
pub mod network;
pub mod pump;
pub mod signer;
pub mod swap;
pub mod transaction;
pub mod utils;

// Re-export commonly used tools - balance module
pub use balance::{
    get_multiple_balances, get_sol, get_spl_token, SolBalanceResult, TokenBalanceResult,
};

// Re-export commonly used tools - network module
pub use network::{get_block_height, get_transaction_status};

// Re-export commonly used tools - pump module
pub use pump::{
    analyze_pump_transaction, buy_pump_token, deploy_pump_token, get_pump_token_info,
    get_trending_pump_tokens, sell_pump_token, TokenInfo as PumpTokenInfo,
    TradeAnalysis as PumpTradeAnalysis, TradeResult as PumpTradeResult, TradeType as PumpTradeType,
};

// Re-export commonly used tools - signer module
pub use signer::{Local as LocalSigner, PriorityFeeConfig};

// Re-export commonly used tools - swap module
pub use swap::{
    execute, get_jupiter_quote, get_token_price, JupiterInfo, JupiterQuote, JupiterResult,
    PriceInfo, RoutePlanStep,
};

// Re-export commonly used tools - transaction module
pub use transaction::{
    create_spl_token_mint, transfer_sol, transfer_spl_token, CreateMintResult, SolTransferResult,
    Status, TokenTransferResult,
};

// Re-export specific utilities to avoid ambiguous glob conflicts
pub use utils::{
    execute as execute_solana_transaction, send as send_transaction, send_transaction_with_retry,
    validate_address, Config, SubmissionResult,
};
// Note: generate_mint and create_token_with_mint_keypair are not re-exported
// at the top level to avoid conflicts. Use utils::generate_mint directly.

// Re-export error types
pub use error::Error as SolanaToolError;

// Re-export signer types for convenience
pub use riglr_core::{signer::UnifiedSigner, SignerContext};

/// Current version of riglr-solana-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;
    use core::any::type_name;

    #[test]
    fn version_is_valid() {
        // VERSION should be a valid semantic version string
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
                clean_part.chars().all(|char| char.is_ascii_digit()),
                "VERSION part {i} should be numeric, got: {clean_part}"
            );
        }
    }

    #[test]
    fn version_constant_accessible() {
        // Test that VERSION constant can be accessed and assigned
        let version_copy = VERSION;
        assert_eq!(version_copy, VERSION);
    }

    #[test]
    fn module_declarations_exist() {
        // Verify that all declared modules can be referenced
        // This test ensures the module declarations are valid and the modules exist

        // Test that we can access module paths (compilation test)
        let _balance_module = type_name::<balance::SolBalanceResult>();
        let _error_module = type_name::<SolanaToolError>();
        // Note: Tool-generated Args types are in different namespace after macro changes
        let _signer_module = type_name::<signer::Local>();

        // These should compile without errors if modules exist
    }

    #[test]
    fn re_exported_types_accessible() {
        // Test that re-exported types from balance module are accessible
        // Note: Tool-generated Args types are in different namespace after macro changes
        // We can still verify that the tool functions themselves are accessible
        let _balance_result = type_name::<balance::SolBalanceResult>();

        // Test that re-exported types from transaction module are accessible
        let _transaction_result = type_name::<utils::SubmissionResult>();
        let _transaction_config = type_name::<utils::Config>();

        // Test that re-exported types from signer module are accessible
        let _local_solana_signer = type_name::<signer::Local>();
    }

    #[test]
    fn utils_re_exports_accessible() {
        // Test that specific utils re-exports are accessible by type name
        let _transaction_config = type_name::<Config>();
        let _transaction_result = type_name::<SubmissionResult>();
    }

    #[test]
    fn error_re_exports() {
        // Test that error re-exports work
        let _solana_tool_error = type_name::<SolanaToolError>();
    }

    #[test]
    fn riglr_core_re_exports() {
        // Test that riglr-core re-exports are accessible
        let _unified_signer = type_name::<dyn UnifiedSigner>();
        let _signer_context = type_name::<SignerContext>();
    }

    #[test]
    fn version_matches_cargo_pkg_version() {
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
    fn crate_documentation_constants() {
        // Test that the crate has the expected structure based on documentation
        // This validates that the public API matches what's documented

        // These should be accessible as documented in the crate docs
        let _: &str = VERSION;

        // Verify the main re-exported modules exist by checking their types
        let _balance_exists = type_name::<balance::SolBalanceResult>();
        // Note: Tool-generated Args types are in different namespace after macro changes
        let _signer_exists = type_name::<signer::Local>();
        let _error_exists = type_name::<SolanaToolError>();
    }
}
