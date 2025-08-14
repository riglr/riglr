//! Error types for event processing operations.

use riglr_core::error::ToolError;
use thiserror::Error;

/// Main error type for event processing operations.
#[derive(Error, Debug)]
pub enum EventError {
    /// Event parsing failed
    #[error("Event parsing error: {context}")]
    ParseError {
        /// Error context
        context: String,
        /// Source error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Event stream error
    #[error("Event stream error: {context}")]
    StreamError {
        /// Error context
        context: String,
        /// Source error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Event filtering error
    #[error("Event filtering error: {context}")]
    FilterError {
        /// Error context
        context: String,
        /// Source error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Event handler error
    #[error("Event handler error: {context}")]
    HandlerError {
        /// Error context
        context: String,
        /// Source error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid configuration
    #[error("Invalid configuration: {message}")]
    InvalidConfig {
        /// Error message
        message: String,
    },

    /// Resource not found
    #[error("Resource not found: {resource}")]
    NotFound {
        /// Resource identifier
        resource: String,
    },

    /// Operation timeout
    #[error("Operation timed out after {duration:?}")]
    Timeout {
        /// Timeout duration
        duration: std::time::Duration,
    },

    /// Generic event error
    #[error("Event error: {message}")]
    Generic {
        /// Error message
        message: String,
    },
}

impl EventError {
    /// Create a parse error with source preservation
    pub fn parse_error<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        Self::ParseError {
            source: Box::new(source),
            context: context.into(),
        }
    }

    /// Create a stream error with source preservation
    pub fn stream_error<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        Self::StreamError {
            source: Box::new(source),
            context: context.into(),
        }
    }

    /// Create a filter error with source preservation
    pub fn filter_error<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        Self::FilterError {
            source: Box::new(source),
            context: context.into(),
        }
    }

    /// Create a handler error with source preservation
    pub fn handler_error<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        Self::HandlerError {
            source: Box::new(source),
            context: context.into(),
        }
    }

    /// Create an invalid configuration error
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(duration: std::time::Duration) -> Self {
        Self::Timeout { duration }
    }

    /// Create a generic error
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// Check if the error is retriable (follows riglr-core patterns)
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::StreamError { .. } => true,
            Self::Io(_) => true,
            Self::Timeout { .. } => true,
            Self::ParseError { .. } => false,
            Self::FilterError { .. } => false,
            Self::HandlerError { .. } => false,
            Self::Serialization(_) => false,
            Self::InvalidConfig { .. } => false,
            Self::NotFound { .. } => false,
            Self::Generic { .. } => false,
        }
    }

    /// Convert to ToolError for integration with riglr-core
    pub fn to_tool_error(self) -> ToolError {
        match self {
            err if err.is_retriable() => {
                ToolError::retriable_with_source(err, "Event processing failed - retriable")
            }
            err => ToolError::permanent_with_source(err, "Event processing failed - permanent"),
        }
    }
}

/// Specialized result type for event operations
pub type EventResult<T> = Result<T, EventError>;

// Implement conversions from common error types
impl From<String> for EventError {
    fn from(msg: String) -> Self {
        Self::generic(msg)
    }
}

impl From<&str> for EventError {
    fn from(msg: &str) -> Self {
        Self::generic(msg.to_string())
    }
}

impl From<anyhow::Error> for EventError {
    fn from(err: anyhow::Error) -> Self {
        Self::generic(err.to_string())
    }
}

// Convert EventError to ToolError for seamless integration
impl From<EventError> for ToolError {
    fn from(err: EventError) -> Self {
        err.to_tool_error()
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
        let event_error = EventError::parse_error(original_error, "Test parsing context");

        // Verify source is preserved
        assert!(event_error.source().is_some());

        // Verify we can downcast to original error type
        let source = event_error.source().unwrap();
        assert!(source.downcast_ref::<TestError>().is_some());
    }

    #[test]
    fn test_error_retriability() {
        let stream_error = EventError::stream_error(TestError, "Stream failed");
        assert!(stream_error.is_retriable());

        let parse_error = EventError::parse_error(TestError, "Parse failed");
        assert!(!parse_error.is_retriable());

        let timeout_error = EventError::timeout(std::time::Duration::from_secs(30));
        assert!(timeout_error.is_retriable());

        let config_error = EventError::invalid_config("Bad config");
        assert!(!config_error.is_retriable());
    }

    #[test]
    fn test_tool_error_conversion() {
        let retriable_error = EventError::stream_error(TestError, "Network issue");
        let tool_error = retriable_error.to_tool_error();
        assert!(tool_error.is_retriable());

        let permanent_error = EventError::parse_error(TestError, "Invalid data");
        let tool_error = permanent_error.to_tool_error();
        assert!(!tool_error.is_retriable());
    }

    #[test]
    fn test_error_display() {
        let error = EventError::parse_error(TestError, "Failed to parse transaction");
        assert!(error.to_string().contains("Event parsing error"));
        assert!(error.to_string().contains("Failed to parse transaction"));
    }

    #[test]
    fn test_convenience_constructors() {
        let timeout_err = EventError::timeout(std::time::Duration::from_secs(5));
        matches!(timeout_err, EventError::Timeout { .. });

        let not_found_err = EventError::not_found("block-123");
        matches!(not_found_err, EventError::NotFound { .. });

        let config_err = EventError::invalid_config("missing required field");
        matches!(config_err, EventError::InvalidConfig { .. });

        let generic_err = EventError::generic("something went wrong");
        matches!(generic_err, EventError::Generic { .. });
    }
}