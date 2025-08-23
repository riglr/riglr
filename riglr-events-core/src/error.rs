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

    // Additional comprehensive tests for 100% coverage

    #[test]
    fn test_parse_error_constructor() {
        let source_error = TestError;
        let context = "Failed to parse JSON event";
        let error = EventError::parse_error(source_error, context);

        match error {
            EventError::ParseError { context: ctx, .. } => {
                assert_eq!(ctx, "Failed to parse JSON event");
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_parse_error_constructor_with_string_context() {
        let error = EventError::parse_error(TestError, String::from("String context"));
        match error {
            EventError::ParseError { context, .. } => {
                assert_eq!(context, "String context");
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_stream_error_constructor() {
        let source_error = TestError;
        let context = "Connection lost to event stream";
        let error = EventError::stream_error(source_error, context);

        match error {
            EventError::StreamError { context: ctx, .. } => {
                assert_eq!(ctx, "Connection lost to event stream");
            }
            _ => panic!("Expected StreamError variant"),
        }
    }

    #[test]
    fn test_filter_error_constructor() {
        let source_error = TestError;
        let context = "Invalid filter expression";
        let error = EventError::filter_error(source_error, context);

        match error {
            EventError::FilterError { context: ctx, .. } => {
                assert_eq!(ctx, "Invalid filter expression");
            }
            _ => panic!("Expected FilterError variant"),
        }
    }

    #[test]
    fn test_handler_error_constructor() {
        let source_error = TestError;
        let context = "Handler execution failed";
        let error = EventError::handler_error(source_error, context);

        match error {
            EventError::HandlerError { context: ctx, .. } => {
                assert_eq!(ctx, "Handler execution failed");
            }
            _ => panic!("Expected HandlerError variant"),
        }
    }

    #[test]
    fn test_invalid_config_constructor() {
        let message = "Missing required configuration field";
        let error = EventError::invalid_config(message);

        match error {
            EventError::InvalidConfig { message: msg } => {
                assert_eq!(msg, "Missing required configuration field");
            }
            _ => panic!("Expected InvalidConfig variant"),
        }
    }

    #[test]
    fn test_invalid_config_constructor_with_string() {
        let error = EventError::invalid_config(String::from("String message"));
        match error {
            EventError::InvalidConfig { message } => {
                assert_eq!(message, "String message");
            }
            _ => panic!("Expected InvalidConfig variant"),
        }
    }

    #[test]
    fn test_not_found_constructor() {
        let resource = "event-handler-123";
        let error = EventError::not_found(resource);

        match error {
            EventError::NotFound { resource: res } => {
                assert_eq!(res, "event-handler-123");
            }
            _ => panic!("Expected NotFound variant"),
        }
    }

    #[test]
    fn test_not_found_constructor_with_string() {
        let error = EventError::not_found(String::from("resource-456"));
        match error {
            EventError::NotFound { resource } => {
                assert_eq!(resource, "resource-456");
            }
            _ => panic!("Expected NotFound variant"),
        }
    }

    #[test]
    fn test_timeout_constructor() {
        let duration = std::time::Duration::from_millis(500);
        let error = EventError::timeout(duration);

        match error {
            EventError::Timeout { duration: dur } => {
                assert_eq!(dur, std::time::Duration::from_millis(500));
            }
            _ => panic!("Expected Timeout variant"),
        }
    }

    #[test]
    fn test_generic_constructor() {
        let message = "General event processing error";
        let error = EventError::generic(message);

        match error {
            EventError::Generic { message: msg } => {
                assert_eq!(msg, "General event processing error");
            }
            _ => panic!("Expected Generic variant"),
        }
    }

    #[test]
    fn test_generic_constructor_with_string() {
        let error = EventError::generic(String::from("String error message"));
        match error {
            EventError::Generic { message } => {
                assert_eq!(message, "String error message");
            }
            _ => panic!("Expected Generic variant"),
        }
    }

    #[test]
    fn test_is_retriable_all_variants() {
        // Retriable errors
        assert!(EventError::stream_error(TestError, "test").is_retriable());
        assert!(EventError::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "test"
        ))
        .is_retriable());
        assert!(EventError::timeout(std::time::Duration::from_secs(1)).is_retriable());

        // Non-retriable errors
        assert!(!EventError::parse_error(TestError, "test").is_retriable());
        assert!(!EventError::filter_error(TestError, "test").is_retriable());
        assert!(!EventError::handler_error(TestError, "test").is_retriable());
        assert!(
            !EventError::Serialization(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "test"
            )))
            .is_retriable()
        );
        assert!(!EventError::invalid_config("test").is_retriable());
        assert!(!EventError::not_found("test").is_retriable());
        assert!(!EventError::generic("test").is_retriable());
    }

    #[test]
    fn test_from_string_implementation() {
        let error_msg = String::from("Error from String");
        let error: EventError = error_msg.into();

        match error {
            EventError::Generic { message } => {
                assert_eq!(message, "Error from String");
            }
            _ => panic!("Expected Generic variant"),
        }
    }

    #[test]
    fn test_from_str_implementation() {
        let error_msg = "Error from &str";
        let error: EventError = error_msg.into();

        match error {
            EventError::Generic { message } => {
                assert_eq!(message, "Error from &str");
            }
            _ => panic!("Expected Generic variant"),
        }
    }

    #[test]
    fn test_from_anyhow_error_implementation() {
        let anyhow_error = anyhow::anyhow!("Anyhow error message");
        let error: EventError = anyhow_error.into();

        match error {
            EventError::Generic { message } => {
                assert!(message.contains("Anyhow error message"));
            }
            _ => panic!("Expected Generic variant"),
        }
    }

    #[test]
    fn test_from_serde_json_error_implementation() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error: EventError = json_error.into();

        match error {
            EventError::Serialization(_) => {
                // Successfully converted
            }
            _ => panic!("Expected Serialization variant"),
        }
    }

    #[test]
    fn test_from_io_error_implementation() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error: EventError = io_error.into();

        match error {
            EventError::Io(_) => {
                // Successfully converted
            }
            _ => panic!("Expected Io variant"),
        }
    }

    #[test]
    fn test_event_error_to_tool_error_conversion() {
        let event_error = EventError::stream_error(TestError, "Stream failed");
        let tool_error: ToolError = event_error.into();
        assert!(tool_error.is_retriable());
    }

    #[test]
    fn test_all_error_display_formats() {
        let parse_error = EventError::parse_error(TestError, "parsing failed");
        assert!(parse_error
            .to_string()
            .contains("Event parsing error: parsing failed"));

        let stream_error = EventError::stream_error(TestError, "stream failed");
        assert!(stream_error
            .to_string()
            .contains("Event stream error: stream failed"));

        let filter_error = EventError::filter_error(TestError, "filter failed");
        assert!(filter_error
            .to_string()
            .contains("Event filtering error: filter failed"));

        let handler_error = EventError::handler_error(TestError, "handler failed");
        assert!(handler_error
            .to_string()
            .contains("Event handler error: handler failed"));

        let serialization_error = EventError::Serialization(serde_json::Error::io(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "test"),
        ));
        assert!(serialization_error
            .to_string()
            .contains("Serialization error:"));

        let io_error = EventError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(io_error.to_string().contains("I/O error:"));

        let config_error = EventError::invalid_config("config invalid");
        assert!(config_error
            .to_string()
            .contains("Invalid configuration: config invalid"));

        let not_found_error = EventError::not_found("resource-123");
        assert!(not_found_error
            .to_string()
            .contains("Resource not found: resource-123"));

        let timeout_error = EventError::timeout(std::time::Duration::from_secs(30));
        assert!(timeout_error
            .to_string()
            .contains("Operation timed out after"));
        assert!(timeout_error.to_string().contains("30s"));

        let generic_error = EventError::generic("generic message");
        assert!(generic_error
            .to_string()
            .contains("Event error: generic message"));
    }

    #[test]
    fn test_to_tool_error_retriable_variants() {
        let stream_error = EventError::stream_error(TestError, "network issue");
        let tool_error = stream_error.to_tool_error();
        assert!(tool_error.is_retriable());

        let io_error = EventError::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"));
        let tool_error = io_error.to_tool_error();
        assert!(tool_error.is_retriable());

        let timeout_error = EventError::timeout(std::time::Duration::from_secs(10));
        let tool_error = timeout_error.to_tool_error();
        assert!(tool_error.is_retriable());
    }

    #[test]
    fn test_to_tool_error_permanent_variants() {
        let parse_error = EventError::parse_error(TestError, "bad data");
        let tool_error = parse_error.to_tool_error();
        assert!(!tool_error.is_retriable());

        let filter_error = EventError::filter_error(TestError, "invalid regex");
        let tool_error = filter_error.to_tool_error();
        assert!(!tool_error.is_retriable());

        let handler_error = EventError::handler_error(TestError, "handler crash");
        let tool_error = handler_error.to_tool_error();
        assert!(!tool_error.is_retriable());

        let serialization_error = EventError::Serialization(serde_json::Error::io(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "bad json"),
        ));
        let tool_error = serialization_error.to_tool_error();
        assert!(!tool_error.is_retriable());

        let config_error = EventError::invalid_config("missing field");
        let tool_error = config_error.to_tool_error();
        assert!(!tool_error.is_retriable());

        let not_found_error = EventError::not_found("missing-resource");
        let tool_error = not_found_error.to_tool_error();
        assert!(!tool_error.is_retriable());

        let generic_error = EventError::generic("unknown error");
        let tool_error = generic_error.to_tool_error();
        assert!(!tool_error.is_retriable());
    }

    #[test]
    fn test_edge_cases_empty_strings() {
        let error = EventError::invalid_config("");
        assert_eq!(error.to_string(), "Invalid configuration: ");

        let error = EventError::not_found("");
        assert_eq!(error.to_string(), "Resource not found: ");

        let error = EventError::generic("");
        assert_eq!(error.to_string(), "Event error: ");

        let error: EventError = "".into();
        assert_eq!(error.to_string(), "Event error: ");

        let error: EventError = "".into();
        assert_eq!(error.to_string(), "Event error: ");
    }

    #[test]
    fn test_edge_cases_special_characters() {
        let special_msg = "Error with special chars: üñîçødé 123!@#$%^&*()";
        let error = EventError::generic(special_msg);
        assert!(error.to_string().contains(special_msg));

        let error: EventError = special_msg.into();
        assert!(error.to_string().contains(special_msg));
    }

    #[test]
    fn test_edge_cases_very_long_strings() {
        let long_msg = "a".repeat(10000);
        let error = EventError::generic(&long_msg);
        assert!(error.to_string().contains(&long_msg));
    }

    #[test]
    fn test_zero_duration_timeout() {
        let error = EventError::timeout(std::time::Duration::from_secs(0));
        assert!(error.to_string().contains("0s"));
        assert!(error.is_retriable());
    }

    #[test]
    fn test_very_large_duration_timeout() {
        let error = EventError::timeout(std::time::Duration::from_secs(u64::MAX));
        assert!(error.is_retriable());
    }

    #[test]
    fn test_event_result_type_alias() {
        // Test that EventResult type alias works correctly
        let success: EventResult<i32> = Ok(42);
        assert_eq!(success.unwrap(), 42);

        let failure: EventResult<i32> = Err(EventError::generic("test error"));
        assert!(failure.is_err());
    }
}
