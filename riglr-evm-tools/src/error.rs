//! Error types for riglr-evm-tools.

use thiserror::Error;

/// Main error type for EVM tool operations.
#[derive(Error, Debug)]
pub enum EvmToolError {
    /// RPC client error
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Contract interaction failed
    #[error("Contract error: {0}")]
    Contract(String),

    /// Transaction failed
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Core riglr error
    #[error("Core error: {0}")]
    Core(#[from] riglr_core::CoreError),

    /// Generic error
    #[error("EVM tool error: {0}")]
    Generic(String),
}

/// Result type alias for EVM tool operations.
pub type Result<T> = std::result::Result<T, EvmToolError>;
