//! Error types for riglr-evm-tools.

use thiserror::Error;
use riglr_core::error::{ToolError, SignerError};

/// Main error type for EVM tool operations.
#[derive(Error, Debug)]
pub enum EvmToolError {
    /// Core tool error
    #[error("Core tool error: {0}")]
    ToolError(#[from] ToolError),
    
    /// Signer context error
    #[error("Signer context error: {0}")]
    SignerError(#[from] SignerError),
    
    /// Provider error
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    /// Transaction build error
    #[error("Transaction build error: {0}")]
    TransactionBuildError(String),
    
    /// Invalid address format
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    
    /// Insufficient balance for operation
    #[error("Insufficient balance for operation")]
    InsufficientBalance,
    
    /// Unsupported chain ID
    #[error("Unsupported chain ID: {0}")]
    UnsupportedChain(u64),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Core riglr error
    #[error("Core error: {0}")]
    Core(#[from] riglr_core::CoreError),

    /// RPC client error
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Contract interaction failed
    #[error("Contract error: {0}")]
    Contract(String),

    /// Invalid private key
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Transaction failed
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Generic error
    #[error("EVM tool error: {0}")]
    Generic(String),
}

/// Result type alias for EVM tool operations.
pub type Result<T> = std::result::Result<T, EvmToolError>;



impl From<EvmToolError> for ToolError {
    fn from(err: EvmToolError) -> Self {
        match err {
            EvmToolError::ProviderError(_) => ToolError::retriable(err.to_string()),
            EvmToolError::InsufficientBalance => ToolError::permanent(err.to_string()),
            EvmToolError::InvalidAddress(_) => ToolError::invalid_input(err.to_string()),
            EvmToolError::UnsupportedChain(_) => ToolError::invalid_input(err.to_string()),
            EvmToolError::Http(_) => ToolError::retriable(err.to_string()),
            EvmToolError::Rpc(_) => ToolError::retriable(err.to_string()),
            EvmToolError::Contract(_) => ToolError::permanent(err.to_string()),
            EvmToolError::InvalidKey(_) => ToolError::invalid_input(err.to_string()),
            EvmToolError::Transaction(_) => ToolError::retriable(err.to_string()),
            EvmToolError::Generic(_) => ToolError::permanent(err.to_string()),
            _ => ToolError::permanent(err.to_string()),
        }
    }
}

