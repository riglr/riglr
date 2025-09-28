//! Error types for event processing operations.

use core::{error::Error as CoreError, time::Duration};
use riglr_core::error::ToolError;
use std::io;
use thiserror::Error as ThisError;

/// Main error type for event processing operations.
#[derive(ThisError, Debug)]
#[non_exhaustive]
#[expect(clippy::module_name_repetitions)]
pub enum EventError {
    /// Event filtering error
    #[error("Event filtering error: {context}")]
    FilterError {
        /// Error context
        context: String,
        /// Source error
        #[source]
        source: Box<dyn CoreError + Send + Sync>,
    },

    /// Generic event error
    #[error("Event error: {message}")]
    Generic {
        /// Error message
        message: String,
    },

    /// Event handler error
    #[error("Event handler error: {context}")]
    HandlerError {
        /// Error context
        context: String,
        /// Source error
        #[source]
        source: Box<dyn CoreError + Send + Sync>,
    },

    /// Invalid configuration
    #[error("Invalid configuration: {message}")]
    InvalidConfig {
        /// Error message
        message: String,
    },

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Resource not found
    #[error("Resource not found: {resource}")]
    NotFound {
        /// Resource identifier
        resource: String,
    },

    /// Event parsing failed
    #[error("Event parsing error: {context}")]
    ParseError {
        /// Error context
        context: String,
        /// Source error
        #[source]
        source: Box<dyn CoreError + Send + Sync>,
    },

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Event stream error
    #[error("Event stream error: {context}")]
    StreamError {
        /// Error context
        context: String,
        /// Source error
        #[source]
        source: Box<dyn CoreError + Send + Sync>,
    },

    /// Operation timeout
    #[error("Operation timed out after {duration:?}")]
    Timeout {
        /// Timeout duration
        duration: Duration,
    },
}

impl EventError {
    /// Create a filter error with source preservation
    #[inline]
    pub fn filter_error<E, C>(source: E, context: C) -> Self
    where
        E: CoreError + Send + Sync + 'static,
        C: Into<String>,
    {
        Self::FilterError {
            source: Box::new(source),
            context: context.into(),
        }
    }

    /// Create a generic error
    #[inline]
    pub fn generic<M: Into<String>>(message: M) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// Create a handler error with source preservation
    #[inline]
    pub fn handler_error<E, C>(source: E, context: C) -> Self
    where
        E: CoreError + Send + Sync + 'static,
        C: Into<String>,
    {
        Self::HandlerError {
            source: Box::new(source),
            context: context.into(),
        }
    }

    /// Create an invalid configuration error
    #[inline]
    pub fn invalid_config<M: Into<String>>(message: M) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    /// Check if the error is retriable (follows riglr-core patterns)
    #[inline]
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        match *self {
            Self::StreamError { .. } | Self::Io(_) | Self::Timeout { .. } => true,
            Self::ParseError { .. }
            | Self::FilterError { .. }
            | Self::HandlerError { .. }
            | Self::Serialization(_)
            | Self::InvalidConfig { .. }
            | Self::NotFound { .. }
            | Self::Generic { .. } => false,
        }
    }

    /// Create a not found error
    #[inline]
    pub fn not_found<R: Into<String>>(resource: R) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a parse error with source preservation
    #[inline]
    pub fn parse_error<E, C>(source: E, context: C) -> Self
    where
        E: CoreError + Send + Sync + 'static,
        C: Into<String>,
    {
        Self::ParseError {
            source: Box::new(source),
            context: context.into(),
        }
    }

    /// Create a stream error with source preservation
    #[inline]
    pub fn stream_error<E, C>(source: E, context: C) -> Self
    where
        E: CoreError + Send + Sync + 'static,
        C: Into<String>,
    {
        Self::StreamError {
            source: Box::new(source),
            context: context.into(),
        }
    }

    /// Create a timeout error
    #[inline]
    #[must_use]
    pub const fn timeout(duration: Duration) -> Self {
        Self::Timeout { duration }
    }

    /// Convert to `ToolError` for integration with riglr-core
    #[inline]
    #[must_use]
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
    #[inline]
    fn from(msg: String) -> Self {
        Self::generic(msg)
    }
}

impl From<&str> for EventError {
    #[inline]
    fn from(msg: &str) -> Self {
        Self::generic(msg.to_owned())
    }
}

impl From<anyhow::Error> for EventError {
    #[inline]
    fn from(err: anyhow::Error) -> Self {
        Self::generic(err.to_string())
    }
}

// Convert EventError to ToolError for seamless integration
impl From<EventError> for ToolError {
    #[inline]
    fn from(err: EventError) -> Self {
        err.to_tool_error()
    }
}

// Type alias for compatibility
pub type Error = EventError;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(thiserror::Error, Debug)]
    #[error("Test error")]
    struct TestError;

    #[test]
    #[expect(clippy::panic)]
    fn error_source_preservation() {
        let original_error = TestError;
        let event_error = EventError::parse_error(original_error, "Test parsing context");

        // Verify source is preserved
        assert!(event_error.source().is_some());

        // Verify we can downcast to original error type
        if let Some(source) = event_error.source() {
            assert!(source.downcast_ref::<TestError>().is_some());
        } else {
            panic!("source should be preserved in ParseError");
        }
    }

    #[test]
    fn error_retriability() {
        let stream_error = EventError::stream_error(TestError, "Stream failed");
        assert!(stream_error.is_retriable());

        let parse_error = EventError::parse_error(TestError, "Parse failed");
        assert!(!parse_error.is_retriable());

        let timeout_error = EventError::timeout(Duration::from_secs(30_u64));
        assert!(timeout_error.is_retriable());

        let config_error = EventError::invalid_config("Bad config");
        assert!(!config_error.is_retriable());
    }

    #[test]
    fn tool_error_conversion() {
        let retriable_error = EventError::stream_error(TestError, "Network issue");
        let tool_error_retriable = retriable_error.to_tool_error();
        assert!(tool_error_retriable.is_retriable());

        let permanent_error = EventError::parse_error(TestError, "Invalid data");
        let tool_error_permanent = permanent_error.to_tool_error();
        assert!(!tool_error_permanent.is_retriable());
    }

    #[test]
    fn error_display() {
        let error = EventError::parse_error(TestError, "Failed to parse transaction");
        assert!(error.to_string().contains("Event parsing error"));
        assert!(error.to_string().contains("Failed to parse transaction"));
    }

    #[test]
    fn convenience_constructors() {
        let timeout_err = EventError::timeout(Duration::from_secs(5));
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
    fn parse_error_constructor() {
        let source_error = TestError;
        let context = "Failed to parse JSON event";
        let error = EventError::parse_error(source_error, context);

        match error {
            EventError::ParseError { context: ctx, .. } => {
                assert_eq!(ctx, "Failed to parse JSON event");
            }
            _ => unreachable!("EventError::parse_error should create ParseError variant"),
        }
    }

    #[test]
    fn parse_error_constructor_with_string_context() {
        let error = EventError::parse_error(TestError, String::from("String context"));
        match error {
            EventError::ParseError { context, .. } => {
                assert_eq!(context, "String context");
            }
            _ => unreachable!("EventError::parse_error should create ParseError variant"),
        }
    }

    #[test]
    fn stream_error_constructor() {
        let source_error = TestError;
        let context = "Connection lost to event stream";
        let error = EventError::stream_error(source_error, context);

        match error {
            EventError::StreamError { context: ctx, .. } => {
                assert_eq!(ctx, "Connection lost to event stream");
            }
            _ => unreachable!("EventError::stream_error should create StreamError variant"),
        }
    }

    #[test]
    fn filter_error_constructor() {
        let source_error = TestError;
        let context = "Invalid filter expression";
        let error = EventError::filter_error(source_error, context);

        match error {
            EventError::FilterError { context: ctx, .. } => {
                assert_eq!(ctx, "Invalid filter expression");
            }
            _ => unreachable!("EventError::filter_error should create FilterError variant"),
        }
    }

    #[test]
    fn handler_error_constructor() {
        let source_error = TestError;
        let context = "Handler execution failed";
        let error = EventError::handler_error(source_error, context);

        match error {
            EventError::HandlerError { context: ctx, .. } => {
                assert_eq!(ctx, "Handler execution failed");
            }
            _ => unreachable!("EventError::handler_error should create HandlerError variant"),
        }
    }

    #[test]
    fn invalid_config_constructor() {
        let message = "Missing required configuration field";
        let error = EventError::invalid_config(message);

        match error {
            EventError::InvalidConfig { message: msg } => {
                assert_eq!(msg, "Missing required configuration field");
            }
            _ => unreachable!("EventError::invalid_config should create InvalidConfig variant"),
        }
    }

    #[test]
    fn invalid_config_constructor_with_string() {
        let error = EventError::invalid_config(String::from("String message"));
        match error {
            EventError::InvalidConfig { message } => {
                assert_eq!(message, "String message");
            }
            _ => unreachable!("EventError::invalid_config should create InvalidConfig variant"),
        }
    }

    #[test]
    fn not_found_constructor() {
        let resource = "event-handler-123";
        let error = EventError::not_found(resource);

        match error {
            EventError::NotFound { resource: res } => {
                assert_eq!(res, "event-handler-123");
            }
            _ => unreachable!("EventError::not_found should create NotFound variant"),
        }
    }

    #[test]
    fn not_found_constructor_with_string() {
        let error = EventError::not_found(String::from("resource-456"));
        match error {
            EventError::NotFound { resource } => {
                assert_eq!(resource, "resource-456");
            }
            _ => unreachable!("EventError::not_found should create NotFound variant"),
        }
    }

    #[test]
    fn timeout_constructor() {
        let duration = Duration::from_millis(500_u64);
        let error = EventError::timeout(duration);

        match error {
            EventError::Timeout { duration: dur } => {
                assert_eq!(dur, Duration::from_millis(500_u64));
            }
            _ => unreachable!("EventError::timeout should create Timeout variant"),
        }
    }

    #[test]
    fn generic_constructor() {
        let message = "General event processing error";
        let error = EventError::generic(message);

        match error {
            EventError::Generic { message: msg } => {
                assert_eq!(msg, "General event processing error");
            }
            _ => unreachable!("EventError::generic should create Generic variant"),
        }
    }

    #[test]
    fn generic_constructor_with_string() {
        let error = EventError::generic(String::from("String error message"));
        match error {
            EventError::Generic { message } => {
                assert_eq!(message, "String error message");
            }
            _ => unreachable!("EventError::generic should create Generic variant"),
        }
    }

    #[test]
    fn is_retriable_all_variants() {
        // Retriable errors
        assert!(EventError::stream_error(TestError, "test").is_retriable());
        assert!(
            EventError::Io(io::Error::new(io::ErrorKind::ConnectionRefused, "test")).is_retriable()
        );
        assert!(EventError::timeout(Duration::from_secs(1_u64)).is_retriable());

        // Non-retriable errors
        assert!(!EventError::parse_error(TestError, "test").is_retriable());
        assert!(!EventError::filter_error(TestError, "test").is_retriable());
        assert!(!EventError::handler_error(TestError, "test").is_retriable());
        assert!(
            !EventError::Serialization(serde_json::Error::io(io::Error::new(
                io::ErrorKind::InvalidData,
                "test"
            )))
            .is_retriable()
        );
        assert!(!EventError::invalid_config("test").is_retriable());
        assert!(!EventError::not_found("test").is_retriable());
        assert!(!EventError::generic("test").is_retriable());
    }

    #[test]
    fn from_string_implementation() {
        let error_msg = String::from("Error from String");
        let error: EventError = error_msg.into();

        match error {
            EventError::Generic { message } => {
                assert_eq!(message, "Error from String");
            }
            _ => unreachable!("String::into should create Generic variant"),
        }
    }

    #[test]
    fn from_str_implementation() {
        let error_msg = "Error from &str";
        let error: EventError = error_msg.into();

        match error {
            EventError::Generic { message } => {
                assert_eq!(message, "Error from &str");
            }
            _ => unreachable!("&str::into should create Generic variant"),
        }
    }

    #[test]
    fn from_anyhow_error_implementation() {
        let anyhow_error = anyhow::anyhow!("Anyhow error message");
        let error: EventError = anyhow_error.into();

        match error {
            EventError::Generic { message } => {
                assert!(message.contains("Anyhow error message"));
            }
            _ => unreachable!("anyhow::Error::into should create Generic variant"),
        }
    }

    #[test]
    #[expect(clippy::panic)]
    fn from_serde_json_error_implementation() {
        let json_result = serde_json::from_str::<serde_json::Value>("invalid json");
        let Err(json_error) = json_result else {
            unreachable!("Expected parsing to fail for invalid json")
        };
        let error: EventError = json_error.into();

        match error {
            EventError::Serialization(_) => {
                // Successfully converted
            }
            _ => {
                panic!("Expected Serialization variant but got: {error:?}");
            }
        }
    }

    #[test]
    #[expect(clippy::panic)]
    fn from_io_error_implementation() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error: EventError = io_error.into();

        match error {
            EventError::Io(_) => {
                // Successfully converted
            }
            _ => {
                panic!("Expected Io variant but got: {error:?}");
            }
        }
    }

    #[test]
    fn event_error_to_tool_error_conversion() {
        let event_error = EventError::stream_error(TestError, "Stream failed");
        let tool_error: ToolError = event_error.into();
        assert!(tool_error.is_retriable());
    }

    #[test]
    fn all_error_display_formats() {
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

        let serialization_error = EventError::Serialization(serde_json::Error::io(io::Error::new(
            io::ErrorKind::InvalidData,
            "test",
        )));
        assert!(serialization_error
            .to_string()
            .contains("Serialization error:"));

        let io_error = EventError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        assert!(io_error.to_string().contains("I/O error:"));

        let config_error = EventError::invalid_config("config invalid");
        assert!(config_error
            .to_string()
            .contains("Invalid configuration: config invalid"));

        let not_found_error = EventError::not_found("resource-123");
        assert!(not_found_error
            .to_string()
            .contains("Resource not found: resource-123"));

        let timeout_error = EventError::timeout(Duration::from_secs(30_u64));
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
    fn to_tool_error_retriable_variants() {
        let stream_error = EventError::stream_error(TestError, "network issue");
        let tool_error_stream = stream_error.to_tool_error();
        assert!(tool_error_stream.is_retriable());

        let io_error = EventError::Io(io::Error::new(io::ErrorKind::TimedOut, "timeout"));
        let tool_error_io = io_error.to_tool_error();
        assert!(tool_error_io.is_retriable());

        let timeout_error = EventError::timeout(Duration::from_secs(10_u64));
        let tool_error_timeout = timeout_error.to_tool_error();
        assert!(tool_error_timeout.is_retriable());
    }

    #[test]
    fn to_tool_error_permanent_variants() {
        let parse_error = EventError::parse_error(TestError, "bad data");
        let tool_error_parse = parse_error.to_tool_error();
        assert!(!tool_error_parse.is_retriable());

        let filter_error = EventError::filter_error(TestError, "invalid regex");
        let tool_error_filter = filter_error.to_tool_error();
        assert!(!tool_error_filter.is_retriable());

        let handler_error = EventError::handler_error(TestError, "handler crash");
        let tool_error_handler = handler_error.to_tool_error();
        assert!(!tool_error_handler.is_retriable());

        let serialization_error = EventError::Serialization(serde_json::Error::io(io::Error::new(
            io::ErrorKind::InvalidData,
            "bad json",
        )));
        let tool_error_serialization = serialization_error.to_tool_error();
        assert!(!tool_error_serialization.is_retriable());

        let config_error = EventError::invalid_config("missing field");
        let tool_error_config = config_error.to_tool_error();
        assert!(!tool_error_config.is_retriable());

        let not_found_error = EventError::not_found("missing-resource");
        let tool_error_not_found = not_found_error.to_tool_error();
        assert!(!tool_error_not_found.is_retriable());

        let generic_error = EventError::generic("unknown error");
        let tool_error_generic = generic_error.to_tool_error();
        assert!(!tool_error_generic.is_retriable());
    }

    #[test]
    fn edge_cases_empty_strings() {
        let config_error = EventError::invalid_config("");
        assert_eq!(config_error.to_string(), "Invalid configuration: ");

        let not_found_error = EventError::not_found("");
        assert_eq!(not_found_error.to_string(), "Resource not found: ");

        let generic_error = EventError::generic("");
        assert_eq!(generic_error.to_string(), "Event error: ");

        let from_str_error: EventError = "".into();
        assert_eq!(from_str_error.to_string(), "Event error: ");

        let from_str_error2: EventError = "".into();
        assert_eq!(from_str_error2.to_string(), "Event error: ");
    }

    #[test]
    fn edge_cases_special_characters() {
        let special_msg = "Error with special chars: üñîçødé 123!@#$%^&*()";
        let generic_error = EventError::generic(special_msg);
        assert!(generic_error.to_string().contains(special_msg));

        let from_str_error: EventError = special_msg.into();
        assert!(from_str_error.to_string().contains(special_msg));
    }

    #[test]
    fn edge_cases_very_long_strings() {
        let long_msg = "a".repeat(10000);
        let error = EventError::generic(&long_msg);
        assert!(error.to_string().contains(&long_msg));
    }

    #[test]
    fn zero_duration_timeout() {
        let error = EventError::timeout(Duration::from_secs(0_u64));
        // Zero duration formats as "0ns", not "0s"
        assert!(error.to_string().contains("0ns"));
        assert!(error.is_retriable());
    }

    #[test]
    fn very_large_duration_timeout() {
        let error = EventError::timeout(Duration::from_secs(u64::MAX));
        assert!(error.is_retriable());
    }

    #[test]
    #[expect(clippy::panic)]
    fn event_result_type_alias() {
        // Test that EventResult type alias works correctly
        let success: EventResult<i32> = Ok(42_i32);
        match success {
            Ok(value) => assert_eq!(value, 42_i32),
            Err(err) => {
                panic!("Expected Ok(42) but got error: {err:?}");
            }
        }

        let failure = EventError::generic("test error");
        // Test that we can construct the failure case
        assert_eq!(failure.to_string(), "Event error: test error");
    }
}
