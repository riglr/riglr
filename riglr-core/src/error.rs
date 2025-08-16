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
        Self::Generic(err.to_string())
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
    pub fn retriable_with_source<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        Self::Retriable {
            source: Box::new(source),
            context: context.into(),
        }
    }

    /// Creates a permanent error with context and source preservation
    pub fn permanent_with_source<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        Self::Permanent {
            source: Box::new(source),
            context: context.into(),
        }
    }

    /// Creates a rate limited error with optional retry duration
    pub fn rate_limited_with_source<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
        retry_after: Option<std::time::Duration>,
    ) -> Self {
        Self::RateLimited {
            source: Box::new(source),
            context: context.into(),
            retry_after,
        }
    }

    /// Creates an invalid input error
    pub fn invalid_input_with_source<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        Self::InvalidInput {
            source: Box::new(source),
            context: context.into(),
        }
    }

    /// Returns whether this error is retriable
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        matches!(self, Self::Retriable { .. } | Self::RateLimited { .. })
    }

    /// Returns the retry delay if this is a rate limited error
    #[must_use]
    pub const fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            Self::RateLimited { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    /// Checks if the error is rate limited (for compatibility)
    #[must_use]
    pub const fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited { .. })
    }

    /// Creates a retriable error from a string message
    pub fn retriable_string<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::retriable_with_source(StringError(msg.clone()), msg)
    }

    /// Creates a permanent error from a string message
    pub fn permanent_string<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::permanent_with_source(StringError(msg.clone()), msg)
    }

    /// Creates a rate limited error from a string message
    pub fn rate_limited_string<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::rate_limited_with_source(StringError(msg.clone()), msg, None)
    }

    /// Creates an invalid input error from a string message
    pub fn invalid_input_string<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::invalid_input_with_source(StringError(msg.clone()), msg)
    }
}

/// Simple error wrapper for string messages
#[derive(Error, Debug)]
#[error("{0}")]
struct StringError(String);

// Common error conversions for tool usage
impl From<anyhow::Error> for ToolError {
    fn from(err: anyhow::Error) -> Self {
        // anyhow::Error doesn't implement std::error::Error directly, use string conversion
        Self::permanent_string(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ToolError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Permanent {
            source: err,
            context: "Generic error".to_string(),
        }
    }
}

impl From<String> for ToolError {
    fn from(err: String) -> Self {
        Self::permanent_string(err)
    }
}

impl From<&str> for ToolError {
    fn from(err: &str) -> Self {
        Self::permanent_string(err.to_string())
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
            Some(std::time::Duration::from_secs(60)),
        );
        assert!(rate_limited.is_retriable());
        assert_eq!(
            rate_limited.retry_after(),
            Some(std::time::Duration::from_secs(60))
        );
    }

    #[test]
    fn test_string_error_creation() {
        let retriable = ToolError::retriable_string("Network timeout");
        assert!(retriable.is_retriable());
        assert!(!retriable.is_rate_limited());

        let rate_limited = ToolError::rate_limited_string("API rate limit exceeded");
        assert!(rate_limited.is_retriable());
        assert!(rate_limited.is_rate_limited());

        let permanent = ToolError::permanent_string("Invalid parameters");
        assert!(!permanent.is_retriable());
        assert!(!permanent.is_rate_limited());
    }
}
