//! Error types for riglr-core.

use thiserror::Error;

/// Main error type for riglr-core operations.
#[derive(Error, Debug)]
pub enum CoreError {
    /// Queue operation failed
    #[error("Queue error: {0}")]
    Queue(String),

    /// Job execution failed
    #[error("Job execution error: {0}")]
    JobExecution(String),

    /// Serialization/deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Redis connection error (only available with redis feature)
    #[cfg(feature = "redis")]
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Generic error
    #[error("Core error: {0}")]
    Generic(String),
}

/// Result type alias for riglr-core operations.
pub type Result<T> = std::result::Result<T, CoreError>;

impl From<&str> for CoreError {
    fn from(err: &str) -> Self {
        CoreError::Generic(err.to_string())
    }
}

/// Tool-specific error type for distinguishing retriable vs permanent failures.
#[derive(Error, Debug)]
pub enum ToolError {
    /// Operation can be retried
    #[error("Operation can be retried: {0}")]
    Retriable(String),
    
    /// Rate limited, retry after delay
    #[error("Rate limited, retry after delay: {0}")]
    RateLimited(String),
    
    /// Permanent error, do not retry
    #[error("Permanent error, do not retry: {0}")]
    Permanent(String),
    
    /// Invalid input provided
    #[error("Invalid input provided: {0}")]
    InvalidInput(String),
    
    /// Signer context error
    #[error("Signer context error: {0}")]
    SignerContext(#[from] SignerError),
}

/// Signer-specific error type
#[derive(Error, Debug)]
pub enum SignerError {
    /// No signer context available
    #[error("No signer context available")]
    NoContext,
    
    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    /// Network error during signing
    #[error("Network error during signing: {0}")]
    NetworkError(String),
    
    /// Insufficient funds
    #[error("Insufficient funds for operation")]
    InsufficientFunds,
    
    /// Invalid private key
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    
    /// Transaction failed
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

impl From<crate::signer::SignerError> for SignerError {
    fn from(err: crate::signer::SignerError) -> Self {
        match err {
            crate::signer::SignerError::NoSignerContext => SignerError::NoContext,
            crate::signer::SignerError::Configuration(msg) => SignerError::InvalidPrivateKey(msg),
            crate::signer::SignerError::Signing(msg) => SignerError::InvalidSignature(msg),
            crate::signer::SignerError::SolanaTransaction(err) => SignerError::TransactionFailed(err.to_string()),
            crate::signer::SignerError::EvmTransaction(msg) => SignerError::TransactionFailed(msg),
            crate::signer::SignerError::ClientCreation(msg) => SignerError::NetworkError(msg),
        }
    }
}

impl ToolError {
    /// Creates a new retriable error
    pub fn retriable<S: Into<String>>(msg: S) -> Self {
        ToolError::Retriable(msg.into())
    }

    /// Creates a new rate limited error
    pub fn rate_limited<S: Into<String>>(msg: S) -> Self {
        ToolError::RateLimited(msg.into())
    }

    /// Creates a new permanent error
    pub fn permanent<S: Into<String>>(msg: S) -> Self {
        ToolError::Permanent(msg.into())
    }
    
    /// Creates a new invalid input error
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        ToolError::InvalidInput(msg.into())
    }

    /// Checks if the error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(self, ToolError::Retriable(_) | ToolError::RateLimited(_))
    }

    /// Checks if the error is rate limited
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, ToolError::RateLimited(_))
    }
}

// Common error conversions for tool usage
impl From<anyhow::Error> for ToolError {
    fn from(err: anyhow::Error) -> Self {
        ToolError::Permanent(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ToolError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        ToolError::Permanent(err.to_string())
    }
}

impl From<String> for ToolError {
    fn from(err: String) -> Self {
        ToolError::Permanent(err)
    }
}

impl From<&str> for ToolError {
    fn from(err: &str) -> Self {
        ToolError::Permanent(err.to_string())
    }
}

