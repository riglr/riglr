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
    /// A retriable error that may succeed on retry
    #[error("Retriable error: {0}")]
    Retriable(String),

    /// A rate-limited error that should be retried with backoff
    #[error("Rate limited: {0}")]
    RateLimited(String),

    /// A permanent error that should not be retried
    #[error("Permanent error: {0}")]
    Permanent(String),
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

