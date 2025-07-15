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
            // Classify errors explicitly with context preservation
            EvmToolError::ProviderError(provider_err) => {
                ToolError::retriable_with_source(EvmToolError::ProviderError(provider_err.clone()), format!("EVM provider error: {}", provider_err))
            },
            EvmToolError::InsufficientBalance => {
                ToolError::permanent_with_source(EvmToolError::InsufficientBalance, "Insufficient balance for transaction")
            },
            EvmToolError::InvalidAddress(addr) => {
                ToolError::invalid_input_with_source(EvmToolError::InvalidAddress(addr.clone()), format!("Invalid Ethereum address: {}", addr))
            },
            EvmToolError::UnsupportedChain(chain_id) => {
                ToolError::invalid_input_with_source(EvmToolError::UnsupportedChain(chain_id), format!("Unsupported chain ID: {}", chain_id))
            },
            EvmToolError::Http(http_err) => {
                ToolError::retriable(format!("HTTP error: {}", http_err))
            },
            EvmToolError::Rpc(rpc_err) => {
                ToolError::retriable(format!("RPC error: {}", rpc_err))
            },
            EvmToolError::Contract(contract_err) => {
                ToolError::permanent(format!("Contract error: {}", contract_err))
            },
            EvmToolError::InvalidKey(key_err) => {
                ToolError::invalid_input(format!("Invalid key: {}", key_err))
            },
            EvmToolError::Transaction(tx_err) => {
                ToolError::retriable(format!("Transaction error: {}", tx_err))
            },
            EvmToolError::Generic(generic_err) => {
                ToolError::permanent(format!("Generic error: {}", generic_err))
            },
            EvmToolError::TransactionBuildError(details) => {
                ToolError::permanent(format!("Failed to build transaction: {}", details))
            },
            EvmToolError::ToolError(tool_err) => tool_err, // Pass through
            EvmToolError::SignerError(signer_err) => ToolError::SignerContext(signer_err),
            EvmToolError::Serialization(_) => ToolError::permanent("Serialization error"),
            EvmToolError::Core(_) => ToolError::permanent("Core error"),
            // No catch-all pattern - compiler enforces exhaustive matching
        }
    }
}

