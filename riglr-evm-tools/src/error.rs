//! Error types for EVM tools

use thiserror::Error;

/// Main error type for EVM tools
#[derive(Error, Debug)]
pub enum EvmToolError {
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(String),

    /// Transaction errors
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Contract interaction errors
    #[error("Contract error: {0}")]
    Contract(String),

    /// Invalid address or parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Insufficient balance
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    /// Provider errors
    #[error("Provider error: {0}")]
    Provider(String),

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Unsupported chain
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(u64),

    /// Provider-related errors (different from Provider above)
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Signer-related errors
    #[error("Signer error: {0}")]
    SignerError(#[from] riglr_core::SignerError),

    /// RPC-related errors
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Generic error
    #[error("{0}")]
    Generic(String),
}

/// Error classification for retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    /// Permanent errors that should not be retried
    Permanent,
    /// Retriable errors that may succeed on retry
    Retriable,
    /// Rate-limited errors that need backoff
    RateLimited,
}

/// Classify network error messages for retry behavior
fn classify_network_message(msg: &str) -> ErrorClass {
    if msg.contains("rate limit") || msg.contains("429") {
        ErrorClass::RateLimited
    } else {
        ErrorClass::Retriable
    }
}

/// Classify provider error messages for retry behavior
fn classify_provider_message(msg: &str) -> ErrorClass {
    if msg.contains("timeout") || msg.contains("connection") {
        ErrorClass::Retriable
    } else if msg.contains("rate") || msg.contains("quota") {
        ErrorClass::RateLimited
    } else {
        ErrorClass::Permanent
    }
}

/// Classify transaction error messages for retry behavior
fn classify_transaction_message(msg: &str) -> ErrorClass {
    if msg.contains("nonce") || msg.contains("pending") {
        ErrorClass::Retriable
    } else if msg.contains("insufficient funds") || msg.contains("reverted") {
        ErrorClass::Permanent
    } else {
        ErrorClass::Retriable
    }
}

/// Classify RPC error messages for retry behavior
fn classify_rpc_message(msg: &str) -> ErrorClass {
    if msg.contains("timeout") || msg.contains("connection") {
        ErrorClass::Retriable
    } else if msg.contains("rate limit") || msg.contains("429") {
        ErrorClass::RateLimited
    } else {
        ErrorClass::Retriable
    }
}

/// Classify generic error messages for retry behavior
fn classify_generic_message(msg: &str) -> ErrorClass {
    if msg.contains("timeout") || msg.contains("connection") {
        ErrorClass::Retriable
    } else {
        ErrorClass::Permanent
    }
}

/// Classify EVM errors to determine retry behavior
pub fn classify_evm_error(error: &EvmToolError) -> ErrorClass {
    match error {
        // Network errors are usually retriable
        EvmToolError::Network(msg) => classify_network_message(msg),

        // Provider errors might be retriable
        EvmToolError::Provider(msg) | EvmToolError::ProviderError(msg) => {
            classify_provider_message(msg)
        }

        // Transaction errors depend on the specific error
        EvmToolError::Transaction(msg) => classify_transaction_message(msg),

        // Contract errors are usually permanent
        EvmToolError::Contract(_) => ErrorClass::Permanent,

        // Invalid parameters are always permanent
        EvmToolError::InvalidParameter(_) => ErrorClass::Permanent,

        // Insufficient balance is permanent (until balance changes)
        EvmToolError::InsufficientBalance(_) => ErrorClass::Permanent,

        // Invalid address is permanent
        EvmToolError::InvalidAddress(_) => ErrorClass::Permanent,

        // Unsupported chain is permanent
        EvmToolError::UnsupportedChain(_) => ErrorClass::Permanent,

        // Signer errors are usually permanent
        EvmToolError::SignerError(_) => ErrorClass::Permanent,

        // RPC errors might be retriable
        EvmToolError::Rpc(msg) => classify_rpc_message(msg),

        // Generic errors default to permanent
        EvmToolError::Generic(msg) => classify_generic_message(msg),
    }
}

impl From<EvmToolError> for riglr_core::ToolError {
    fn from(err: EvmToolError) -> Self {
        match classify_evm_error(&err) {
            ErrorClass::Permanent => riglr_core::ToolError::permanent_string(err.to_string()),
            ErrorClass::Retriable => riglr_core::ToolError::retriable_string(err.to_string()),
            ErrorClass::RateLimited => riglr_core::ToolError::rate_limited_string(err.to_string()),
        }
    }
}
