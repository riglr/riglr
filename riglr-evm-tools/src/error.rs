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

    /// Invalid private key
    #[error("Invalid key: {0}")]
    InvalidKey(String),

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

// Additional From implementations for error conversion
impl From<String> for EvmToolError {
    fn from(err: String) -> Self {
        EvmToolError::Generic(err)
    }
}

impl From<&str> for EvmToolError {
    fn from(err: &str) -> Self {
        EvmToolError::Generic(err.to_string())
    }
}

impl From<EvmToolError> for riglr_core::ToolError {
    fn from(error: EvmToolError) -> Self {
        match error {
            EvmToolError::Rpc(_) | EvmToolError::Http(_) => {
                riglr_core::ToolError::Retriable(error.to_string())
            }
            EvmToolError::InvalidAddress(_) | EvmToolError::InvalidKey(_) 
            | EvmToolError::Contract(_) | EvmToolError::Transaction(_) 
            | EvmToolError::Serialization(_) | EvmToolError::Core(_) 
            | EvmToolError::Generic(_) => {
                riglr_core::ToolError::Permanent(error.to_string())
            }
        }
    }
}

