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
    #[error("Operation can be retried")]
    Retriable {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        context: String,
    },
    
    /// Rate limited, retry after delay
    #[error("Rate limited, retry after delay")]
    RateLimited {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        context: String,
        retry_after: Option<std::time::Duration>,
    },
    
    /// Permanent error, do not retry
    #[error("Permanent error, do not retry")]
    Permanent {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        context: String,
    },
    
    /// Invalid input provided
    #[error("Invalid input provided: {context}")]
    InvalidInput {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        context: String,
    },
    
    /// Signer context error
    #[error("Signer context error")]
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
            crate::signer::SignerError::InvalidPrivateKey(msg) => SignerError::InvalidPrivateKey(msg),
            crate::signer::SignerError::ProviderError(msg) => SignerError::NetworkError(msg),
            crate::signer::SignerError::TransactionFailed(msg) => SignerError::TransactionFailed(msg),
            crate::signer::SignerError::UnsupportedOperation(msg) => SignerError::TransactionFailed(msg),
            crate::signer::SignerError::BlockhashError(msg) => SignerError::NetworkError(msg),
            crate::signer::SignerError::SigningError(msg) => SignerError::InvalidSignature(msg),
            crate::signer::SignerError::UnsupportedNetwork(msg) => SignerError::NetworkError(msg),
            crate::signer::SignerError::InvalidRpcUrl(msg) => SignerError::NetworkError(msg),
        }
    }
}

impl ToolError {
    /// Creates a retriable error with context and source preservation
    pub fn retriable_with_source<E: std::error::Error + Send + Sync + 'static>(source: E, context: impl Into<String>) -> Self {
        Self::Retriable {
            source: Box::new(source),
            context: context.into(),
        }
    }
    
    /// Creates a permanent error with context and source preservation
    pub fn permanent_with_source<E: std::error::Error + Send + Sync + 'static>(source: E, context: impl Into<String>) -> Self {
        Self::Permanent {
            source: Box::new(source),
            context: context.into(),
        }
    }
    
    /// Creates a rate limited error with optional retry duration
    pub fn rate_limited_with_source<E: std::error::Error + Send + Sync + 'static>(
        source: E, 
        context: impl Into<String>,
        retry_after: Option<std::time::Duration>
    ) -> Self {
        Self::RateLimited {
            source: Box::new(source),
            context: context.into(),
            retry_after,
        }
    }
    
    /// Creates an invalid input error
    pub fn invalid_input_with_source<E: std::error::Error + Send + Sync + 'static>(source: E, context: impl Into<String>) -> Self {
        Self::InvalidInput {
            source: Box::new(source),
            context: context.into(),
        }
    }

    // Backward compatibility methods - maintain old API
    /// Creates a new retriable error
    pub fn retriable<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::retriable_with_source(SimpleError(msg.clone()), msg)
    }

    /// Creates a new rate limited error
    pub fn rate_limited<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::rate_limited_with_source(SimpleError(msg.clone()), msg, None)
    }

    /// Creates a new permanent error
    pub fn permanent<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::permanent_with_source(SimpleError(msg.clone()), msg)
    }
    
    /// Creates a new invalid input error
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::invalid_input_with_source(SimpleError(msg.clone()), msg)
    }

    /// Returns whether this error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(self, ToolError::Retriable { .. } | ToolError::RateLimited { .. })
    }
    
    /// Returns the retry delay if this is a rate limited error
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            ToolError::RateLimited { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    /// Checks if the error is rate limited (for compatibility)
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, ToolError::RateLimited { .. })
    }

}

// For string errors, we need to create a simple error type
#[derive(Error, Debug)]
#[error("{0}")]
struct SimpleError(String);

// Common error conversions for tool usage
impl From<anyhow::Error> for ToolError {
    fn from(err: anyhow::Error) -> Self {
        // anyhow::Error doesn't implement std::error::Error directly, use string conversion
        ToolError::permanent(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ToolError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        ToolError::Permanent {
            source: err,
            context: "Generic error".to_string(),
        }
    }
}

impl From<String> for ToolError {
    fn from(err: String) -> Self {
        ToolError::permanent(err)
    }
}

impl From<&str> for ToolError {
    fn from(err: &str) -> Self {
        ToolError::permanent(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    
    #[derive(Error, Debug)]
    #[error("Test error")]
    struct TestError;

    #[test]
    fn test_error_source_preservation() {
        let original_error = TestError;
        let tool_error = ToolError::retriable_with_source(original_error, "Test context");
        
        // Verify source is preserved
        assert!(tool_error.source().is_some());
        
        // Verify we can downcast to original error type
        let source = tool_error.source().unwrap();
        assert!(source.downcast_ref::<TestError>().is_some());
    }
    
    #[test]
    fn test_error_classification() {
        let retriable = ToolError::retriable_with_source(TestError, "Retriable test");
        assert!(retriable.is_retriable());
        
        let permanent = ToolError::permanent_with_source(TestError, "Permanent test");
        assert!(!permanent.is_retriable());
        
        let rate_limited = ToolError::rate_limited_with_source(
            TestError, 
            "Rate limited test", 
            Some(std::time::Duration::from_secs(60))
        );
        assert!(rate_limited.is_retriable());
        assert_eq!(rate_limited.retry_after(), Some(std::time::Duration::from_secs(60)));
    }

    #[test]
    fn test_backward_compatibility() {
        let retriable = ToolError::retriable("Network timeout");
        assert!(retriable.is_retriable());
        assert!(!retriable.is_rate_limited());
        
        let rate_limited = ToolError::rate_limited("API rate limit exceeded");
        assert!(rate_limited.is_retriable());
        assert!(rate_limited.is_rate_limited());
        
        let permanent = ToolError::permanent("Invalid parameters");
        assert!(!permanent.is_retriable());
        assert!(!permanent.is_rate_limited());
    }
}

