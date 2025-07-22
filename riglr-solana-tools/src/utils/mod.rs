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
//! - [`config`] - Configuration and environment variable handling
//!
//! All modules follow the SignerContext pattern for secure multi-tenant operation,
//! as established by riglr-core architecture.

pub mod validation;
pub mod transaction;
pub mod keypair;
pub mod config;

// Re-export commonly used items for convenience
pub use validation::validate_address;
pub use transaction::{
    send_transaction, 
    send_transaction_with_retry,
    execute_solana_transaction,
    create_token_with_mint_keypair,
    TransactionConfig,
    TransactionSubmissionResult,
};
pub use keypair::generate_mint_keypair;
pub use config::get_rpc_url;