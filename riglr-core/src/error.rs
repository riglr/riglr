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

/// Tool-specific error type for distinguishing retriable vs permanent failures.
#[derive(Error, Debug)]
pub enum ToolError {
    /// A retriable error that may succeed on retry
    #[error("Retriable error: {0}")]
    Retriable(String),
    
    /// A permanent error that should not be retried
    #[error("Permanent error: {0}")]
    Permanent(String),
}

impl ToolError {
    /// Creates a new retriable error
    pub fn retriable<S: Into<String>>(msg: S) -> Self {
        ToolError::Retriable(msg.into())
    }
    
    /// Creates a new permanent error
    pub fn permanent<S: Into<String>>(msg: S) -> Self {
        ToolError::Permanent(msg.into())
    }
    
    /// Checks if the error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(self, ToolError::Retriable(_))
    }
}
