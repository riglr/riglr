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

/// Worker-specific error type for distinguishing system-level worker failures
/// from tool execution failures.
#[derive(Error, Debug)]
pub enum WorkerError {
    /// Tool not found in the worker's registry
    #[error("Tool '{tool_name}' not found in worker registry")]
    ToolNotFound { tool_name: String },

    /// Failed to acquire semaphore for concurrency control
    #[error("Failed to acquire semaphore for tool '{tool_name}': {source}")]
    SemaphoreAcquisition {
        tool_name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Idempotency store operation failed
    #[error("Idempotency store operation failed: {source}")]
    IdempotencyStore {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Job serialization/deserialization error
    #[error("Job serialization error: {source}")]
    JobSerialization {
        #[source]
        source: serde_json::Error,
    },

    /// Tool execution exceeded configured timeout
    #[error("Tool execution timed out after {timeout:?}")]
    ExecutionTimeout { timeout: std::time::Duration },

    /// Internal worker system error
    #[error("Internal worker error: {message}")]
    Internal { message: String },
}

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
    SignerContext(#[from] crate::signer::SignerError),
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

    // Simple convenience methods (discouraged - prefer _with_source versions)
    /// Creates a retriable error from a simple message.
    /// 
    /// **Discouraged**: Use [`retriable_with_source`](Self::retriable_with_source) instead 
    /// to preserve error context and enable better debugging.
    /// 
    /// This method creates a simple error wrapper that loses type information.
    /// For better error handling, prefer:
    /// ```rust
    /// # use riglr_core::ToolError;
    /// # #[derive(thiserror::Error, Debug)]
    /// # #[error("Network error")]
    /// # struct NetworkError;
    /// let error = NetworkError;
    /// let tool_error = ToolError::retriable_with_source(error, "Failed to connect to API");
    /// ```
    pub fn retriable_from_msg<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::retriable_with_source(SimpleError(msg.clone()), msg)
    }

    /// Creates a rate limited error from a simple message.
    /// 
    /// **Discouraged**: Use [`rate_limited_with_source`](Self::rate_limited_with_source) instead 
    /// to preserve error context and enable better debugging.
    pub fn rate_limited_from_msg<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::rate_limited_with_source(SimpleError(msg.clone()), msg, None)
    }

    /// Creates a permanent error from a simple message.
    /// 
    /// **Discouraged**: Use [`permanent_with_source`](Self::permanent_with_source) instead 
    /// to preserve error context and enable better debugging.
    pub fn permanent_from_msg<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::permanent_with_source(SimpleError(msg.clone()), msg)
    }
    
    /// Creates an invalid input error from a simple message.
    /// 
    /// **Discouraged**: Use [`invalid_input_with_source`](Self::invalid_input_with_source) instead 
    /// to preserve error context and enable better debugging.
    pub fn invalid_input_from_msg<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::invalid_input_with_source(SimpleError(msg.clone()), msg)
    }

    // Legacy compatibility aliases (deprecated)
    #[deprecated(since = "0.1.0", note = "Use `retriable_with_source` for better error handling or `retriable_from_msg` for simple messages")]
    pub fn retriable<S: Into<String>>(msg: S) -> Self {
        Self::retriable_from_msg(msg)
    }

    #[deprecated(since = "0.1.0", note = "Use `rate_limited_with_source` for better error handling or `rate_limited_from_msg` for simple messages")]
    pub fn rate_limited<S: Into<String>>(msg: S) -> Self {
        Self::rate_limited_from_msg(msg)
    }

    #[deprecated(since = "0.1.0", note = "Use `permanent_with_source` for better error handling or `permanent_from_msg` for simple messages")]
    pub fn permanent<S: Into<String>>(msg: S) -> Self {
        Self::permanent_from_msg(msg)
    }
    
    #[deprecated(since = "0.1.0", note = "Use `invalid_input_with_source` for better error handling or `invalid_input_from_msg` for simple messages")]
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        Self::invalid_input_from_msg(msg)
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
        ToolError::permanent_from_msg(err.to_string())
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
        ToolError::permanent_from_msg(err)
    }
}

impl From<&str> for ToolError {
    fn from(err: &str) -> Self {
        ToolError::permanent_from_msg(err.to_string())
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
        let retriable = ToolError::retriable_from_msg("Network timeout");
        assert!(retriable.is_retriable());
        assert!(!retriable.is_rate_limited());
        
        let rate_limited = ToolError::rate_limited_from_msg("API rate limit exceeded");
        assert!(rate_limited.is_retriable());
        assert!(rate_limited.is_rate_limited());
        
        let permanent = ToolError::permanent_from_msg("Invalid parameters");
        assert!(!permanent.is_retriable());
        assert!(!permanent.is_rate_limited());
    }
}

