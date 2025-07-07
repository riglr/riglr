//! Error types for riglr-solana-tools.

use thiserror::Error;

/// Main error type for Solana tool operations.
#[derive(Error, Debug)]
pub enum SolanaToolError {
    /// RPC client error
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

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
    #[error("Solana tool error: {0}")]
    Generic(String),
}

/// Result type alias for Solana tool operations.
pub type Result<T> = std::result::Result<T, SolanaToolError>;

// Implement From conversion to riglr_core::ToolError for proper error handling
impl From<SolanaToolError> for riglr_core::error::ToolError {
    fn from(err: SolanaToolError) -> Self {
        match err {
            // RPC and HTTP errors are often retriable
            SolanaToolError::Rpc(msg) => {
                if msg.contains("429") || msg.contains("rate limit") || msg.contains("too many requests") {
                    riglr_core::error::ToolError::rate_limited(msg)
                } else if msg.contains("timeout") || msg.contains("connection") || msg.contains("network") {
                    riglr_core::error::ToolError::retriable(msg)
                } else {
                    riglr_core::error::ToolError::retriable(msg)
                }
            }
            
            SolanaToolError::Http(ref http_err) => {
                if http_err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                    riglr_core::error::ToolError::rate_limited(err.to_string())
                } else if http_err.is_timeout() || http_err.is_connect() {
                    riglr_core::error::ToolError::retriable(err.to_string())
                } else {
                    riglr_core::error::ToolError::permanent(err.to_string())
                }
            }

            // Address validation errors are permanent
            SolanaToolError::InvalidAddress(msg) => riglr_core::error::ToolError::permanent(msg),
            
            // Transaction errors could be retriable if they're network related
            SolanaToolError::Transaction(msg) => {
                if msg.contains("insufficient funds") || msg.contains("invalid") {
                    riglr_core::error::ToolError::permanent(msg)
                } else {
                    riglr_core::error::ToolError::retriable(msg)
                }
            }
            
            // Serialization errors are permanent
            SolanaToolError::Serialization(_) => riglr_core::error::ToolError::permanent(err.to_string()),
            
            // Core errors inherit their retriable nature
            SolanaToolError::Core(_) => riglr_core::error::ToolError::retriable(err.to_string()),
            
            // Generic errors default to retriable
            SolanaToolError::Generic(msg) => riglr_core::error::ToolError::retriable(msg),
        }
    }
}
