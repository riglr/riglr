use riglr_events_core::EventError;
use thiserror::Error;

/// Comprehensive error type for streaming operations
///
/// This extends EventError with streaming-specific error variants while
/// maintaining compatibility with the events-core error hierarchy.
#[derive(Error, Debug)]
pub enum StreamError {
    /// Connection-related errors
    #[error("Connection failed: {message}")]
    Connection {
        /// Error message describing the connection failure
        message: String,
        /// Whether this connection error can be retried
        retriable: bool,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration {
        /// Error message describing the configuration issue
        message: String,
    },

    /// Invalid configuration
    #[error("Invalid configuration: {reason}")]
    ConfigurationInvalid {
        /// Reason why the configuration is invalid
        reason: String,
    },

    /// Authentication failures
    #[error("Authentication failed: {message}")]
    Authentication {
        /// Error message describing the authentication failure
        message: String,
    },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        /// Error message describing the rate limit
        message: String,
        /// Optional duration in seconds to wait before retrying
        retry_after: Option<u64>,
    },

    /// Data parsing errors
    #[error("Parse error: {message}")]
    Parse {
        /// Error message describing the parsing failure
        message: String,
        /// The raw data that failed to parse
        data: String,
    },

    /// Resource exhaustion
    #[error("Resource exhausted: {message}")]
    ResourceExhausted {
        /// Error message describing the resource exhaustion
        message: String,
    },

    /// Stream already running
    #[error("Stream already running: {name}")]
    AlreadyRunning {
        /// Name of the stream that is already running
        name: String,
    },

    /// Stream not running
    #[error("Stream not running: {name}")]
    NotRunning {
        /// Name of the stream that is not running
        name: String,
    },

    /// Channel errors
    #[error("Channel error: {message}")]
    Channel {
        /// Error message describing the channel failure
        message: String,
    },

    /// Timeout errors
    #[error("Operation timed out: {message}")]
    Timeout {
        /// Error message describing the timeout
        message: String,
    },

    /// Processing errors
    #[error("Processing error: {message}")]
    Processing {
        /// Error message describing the processing failure
        message: String,
    },

    /// Backpressure errors
    #[error("Backpressure error: {message}")]
    Backpressure {
        /// Error message describing the backpressure issue
        message: String,
    },

    /// Internal errors
    #[error("Internal error: {source}")]
    Internal {
        /// The underlying error that caused this internal error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl StreamError {
    /// Check if this error is retriable
    pub fn is_retriable(&self) -> bool {
        match self {
            StreamError::Connection { retriable, .. } => *retriable,
            StreamError::RateLimit { .. } => true,
            StreamError::ResourceExhausted { .. } => true,
            StreamError::Timeout { .. } => true,
            StreamError::Channel { .. } => true,
            StreamError::Configuration { .. } => false,
            StreamError::ConfigurationInvalid { .. } => false,
            StreamError::Authentication { .. } => false,
            StreamError::Parse { .. } => false,
            StreamError::AlreadyRunning { .. } => false,
            StreamError::NotRunning { .. } => false,
            StreamError::Processing { .. } => true,
            StreamError::Backpressure { .. } => true,
            StreamError::Internal { .. } => false,
        }
    }

    /// Create a retriable connection error
    pub fn retriable_connection(message: impl Into<String>) -> Self {
        StreamError::Connection {
            message: message.into(),
            retriable: true,
        }
    }

    /// Create a permanent connection error
    pub fn permanent_connection(message: impl Into<String>) -> Self {
        StreamError::Connection {
            message: message.into(),
            retriable: false,
        }
    }

    /// Create a rate limit error with retry after duration
    pub fn rate_limited(message: impl Into<String>, retry_after_seconds: u64) -> Self {
        StreamError::RateLimit {
            message: message.into(),
            retry_after: Some(retry_after_seconds),
        }
    }

    /// Get retry after duration if available
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            StreamError::RateLimit {
                retry_after: Some(seconds),
                ..
            } => Some(std::time::Duration::from_secs(*seconds)),
            _ => None,
        }
    }

    /// Convert StreamError to EventError for integration with events-core
    pub fn to_event_error(self) -> EventError {
        match self {
            StreamError::Connection {
                message,
                retriable: true,
            } => EventError::stream_error(
                std::io::Error::new(std::io::ErrorKind::ConnectionRefused, message.clone()),
                message,
            ),
            StreamError::Connection {
                message,
                retriable: false,
            } => EventError::generic(format!("Permanent connection error: {}", message)),
            StreamError::Configuration { message }
            | StreamError::ConfigurationInvalid { reason: message } => {
                EventError::invalid_config(message)
            }
            StreamError::Authentication { message } => {
                EventError::generic(format!("Authentication error: {}", message))
            }
            StreamError::RateLimit { message, .. } => EventError::stream_error(
                std::io::Error::other(message.clone()),
                format!("Rate limit: {}", message),
            ),
            StreamError::Parse { message, .. } => EventError::parse_error(
                serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "parse error",
                )),
                message,
            ),
            StreamError::ResourceExhausted { message } => EventError::stream_error(
                std::io::Error::other(message.clone()),
                format!("Resource exhausted: {}", message),
            ),
            StreamError::AlreadyRunning { name } => {
                EventError::generic(format!("Stream already running: {}", name))
            }
            StreamError::NotRunning { name } => {
                EventError::generic(format!("Stream not running: {}", name))
            }
            StreamError::Channel { message } => EventError::stream_error(
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, message.clone()),
                format!("Channel error: {}", message),
            ),
            StreamError::Timeout { .. } => EventError::timeout(std::time::Duration::from_secs(30)),
            StreamError::Processing { message } => {
                EventError::generic(format!("Processing error: {}", message))
            }
            StreamError::Backpressure { message } => {
                EventError::generic(format!("Backpressure error: {}", message))
            }
            StreamError::Internal { source } => {
                EventError::generic(format!("Internal streaming error: {}", source))
            }
        }
    }

    /// Create a StreamError from an EventError
    pub fn from_event_error(event_error: EventError) -> Self {
        match event_error {
            EventError::StreamError { context, source } => {
                StreamError::retriable_connection(format!("{}: {}", context, source))
            }
            EventError::ParseError { context, .. } => StreamError::Parse {
                message: context,
                data: String::default(),
            },
            EventError::Timeout { duration } => StreamError::Timeout {
                message: format!("Operation timed out after {:?}", duration),
            },
            EventError::InvalidConfig { message } => StreamError::Configuration { message },
            EventError::NotFound { resource } => StreamError::Configuration {
                message: format!("Resource not found: {}", resource),
            },
            _ => StreamError::Internal {
                source: Box::new(event_error),
            },
        }
    }
}

/// Result type for streaming operations
pub type StreamResult<T> = Result<T, StreamError>;

/// Convert from standard IO errors
impl From<std::io::Error> for StreamError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted => {
                StreamError::retriable_connection(err.to_string())
            }
            std::io::ErrorKind::TimedOut => StreamError::Timeout {
                message: err.to_string(),
            },
            _ => StreamError::Internal {
                source: Box::new(err),
            },
        }
    }
}

/// Convert from serde_json errors
impl From<serde_json::Error> for StreamError {
    fn from(err: serde_json::Error) -> Self {
        StreamError::Parse {
            message: err.to_string(),
            data: String::default(),
        }
    }
}

/// Convert from tokio broadcast errors
impl<T> From<tokio::sync::broadcast::error::SendError<T>> for StreamError {
    fn from(err: tokio::sync::broadcast::error::SendError<T>) -> Self {
        StreamError::Channel {
            message: format!("Failed to send to broadcast channel: {}", err),
        }
    }
}

/// Convert StreamError to EventError for seamless integration with events-core
impl From<StreamError> for EventError {
    fn from(err: StreamError) -> Self {
        err.to_event_error()
    }
}

/// Convert EventError to StreamError for backward compatibility
impl From<EventError> for StreamError {
    fn from(err: EventError) -> Self {
        StreamError::from_event_error(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retriability() {
        assert!(StreamError::retriable_connection("test").is_retriable());
        assert!(!StreamError::permanent_connection("test").is_retriable());
        assert!(StreamError::rate_limited("test", 60).is_retriable());
        assert!(!StreamError::Configuration {
            message: "test".to_string()
        }
        .is_retriable());
        assert!(!StreamError::Authentication {
            message: "test".to_string()
        }
        .is_retriable());
        assert!(StreamError::Timeout {
            message: "test".to_string()
        }
        .is_retriable());
    }

    #[test]
    fn test_retry_after() {
        let err = StreamError::rate_limited("test", 60);
        assert_eq!(err.retry_after(), Some(std::time::Duration::from_secs(60)));

        let err = StreamError::retriable_connection("test");
        assert_eq!(err.retry_after(), None);
    }

    #[test]
    fn test_is_retriable_all_variants() {
        // Test all retriable cases
        assert!(StreamError::Connection {
            message: "test".to_string(),
            retriable: true
        }
        .is_retriable());
        assert!(StreamError::RateLimit {
            message: "test".to_string(),
            retry_after: None
        }
        .is_retriable());
        assert!(StreamError::ResourceExhausted {
            message: "test".to_string()
        }
        .is_retriable());
        assert!(StreamError::Timeout {
            message: "test".to_string()
        }
        .is_retriable());
        assert!(StreamError::Channel {
            message: "test".to_string()
        }
        .is_retriable());
        assert!(StreamError::Processing {
            message: "test".to_string()
        }
        .is_retriable());
        assert!(StreamError::Backpressure {
            message: "test".to_string()
        }
        .is_retriable());

        // Test all non-retriable cases
        assert!(!StreamError::Connection {
            message: "test".to_string(),
            retriable: false
        }
        .is_retriable());
        assert!(!StreamError::Configuration {
            message: "test".to_string()
        }
        .is_retriable());
        assert!(!StreamError::ConfigurationInvalid {
            reason: "test".to_string()
        }
        .is_retriable());
        assert!(!StreamError::Authentication {
            message: "test".to_string()
        }
        .is_retriable());
        assert!(!StreamError::Parse {
            message: "test".to_string(),
            data: "data".to_string()
        }
        .is_retriable());
        assert!(!StreamError::AlreadyRunning {
            name: "test".to_string()
        }
        .is_retriable());
        assert!(!StreamError::NotRunning {
            name: "test".to_string()
        }
        .is_retriable());
        assert!(!StreamError::Internal {
            source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test"))
        }
        .is_retriable());
    }

    #[test]
    fn test_retriable_connection_creation() {
        let error = StreamError::retriable_connection("connection failed");
        match error {
            StreamError::Connection { message, retriable } => {
                assert_eq!(message, "connection failed");
                assert!(retriable);
            }
            _ => panic!("Expected Connection variant"),
        }

        // Test with String
        let error = StreamError::retriable_connection("test".to_string());
        match error {
            StreamError::Connection { message, retriable } => {
                assert_eq!(message, "test");
                assert!(retriable);
            }
            _ => panic!("Expected Connection variant"),
        }
    }

    #[test]
    fn test_permanent_connection_creation() {
        let error = StreamError::permanent_connection("permanent failure");
        match error {
            StreamError::Connection { message, retriable } => {
                assert_eq!(message, "permanent failure");
                assert!(!retriable);
            }
            _ => panic!("Expected Connection variant"),
        }

        // Test with String
        let error = StreamError::permanent_connection("test".to_string());
        match error {
            StreamError::Connection { message, retriable } => {
                assert_eq!(message, "test");
                assert!(!retriable);
            }
            _ => panic!("Expected Connection variant"),
        }
    }

    #[test]
    fn test_rate_limited_creation() {
        let error = StreamError::rate_limited("too many requests", 120);
        match error {
            StreamError::RateLimit {
                message,
                retry_after,
            } => {
                assert_eq!(message, "too many requests");
                assert_eq!(retry_after, Some(120));
            }
            _ => panic!("Expected RateLimit variant"),
        }

        // Test with String
        let error = StreamError::rate_limited("test".to_string(), 0);
        match error {
            StreamError::RateLimit {
                message,
                retry_after,
            } => {
                assert_eq!(message, "test");
                assert_eq!(retry_after, Some(0));
            }
            _ => panic!("Expected RateLimit variant"),
        }
    }

    #[test]
    fn test_retry_after_all_variants() {
        // Test RateLimit with retry_after
        let error = StreamError::RateLimit {
            message: "test".to_string(),
            retry_after: Some(30),
        };
        assert_eq!(
            error.retry_after(),
            Some(std::time::Duration::from_secs(30))
        );

        // Test RateLimit without retry_after
        let error = StreamError::RateLimit {
            message: "test".to_string(),
            retry_after: None,
        };
        assert_eq!(error.retry_after(), None);

        // Test all other variants return None
        assert_eq!(
            StreamError::Connection {
                message: "test".to_string(),
                retriable: true
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::Configuration {
                message: "test".to_string()
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::ConfigurationInvalid {
                reason: "test".to_string()
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::Authentication {
                message: "test".to_string()
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::Parse {
                message: "test".to_string(),
                data: "data".to_string()
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::ResourceExhausted {
                message: "test".to_string()
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::AlreadyRunning {
                name: "test".to_string()
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::NotRunning {
                name: "test".to_string()
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::Channel {
                message: "test".to_string()
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::Timeout {
                message: "test".to_string()
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::Processing {
                message: "test".to_string()
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::Backpressure {
                message: "test".to_string()
            }
            .retry_after(),
            None
        );
        assert_eq!(
            StreamError::Internal {
                source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test"))
            }
            .retry_after(),
            None
        );
    }

    #[test]
    fn test_to_event_error_all_variants() {
        // Test retriable connection
        let error = StreamError::Connection {
            message: "connection failed".to_string(),
            retriable: true,
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("connection failed"));

        // Test permanent connection
        let error = StreamError::Connection {
            message: "permanent failure".to_string(),
            retriable: false,
        };
        let event_error = error.to_event_error();
        assert!(event_error
            .to_string()
            .contains("Permanent connection error"));

        // Test Configuration
        let error = StreamError::Configuration {
            message: "config error".to_string(),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("config error"));

        // Test ConfigurationInvalid
        let error = StreamError::ConfigurationInvalid {
            reason: "invalid config".to_string(),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("invalid config"));

        // Test Authentication
        let error = StreamError::Authentication {
            message: "auth failed".to_string(),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("Authentication error"));

        // Test RateLimit
        let error = StreamError::RateLimit {
            message: "rate limited".to_string(),
            retry_after: Some(60),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("Rate limit"));

        // Test Parse
        let error = StreamError::Parse {
            message: "parse failed".to_string(),
            data: "invalid data".to_string(),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("parse failed"));

        // Test ResourceExhausted
        let error = StreamError::ResourceExhausted {
            message: "out of memory".to_string(),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("Resource exhausted"));

        // Test AlreadyRunning
        let error = StreamError::AlreadyRunning {
            name: "stream1".to_string(),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("Stream already running"));

        // Test NotRunning
        let error = StreamError::NotRunning {
            name: "stream2".to_string(),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("Stream not running"));

        // Test Channel
        let error = StreamError::Channel {
            message: "channel closed".to_string(),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("Channel error"));

        // Test Timeout
        let error = StreamError::Timeout {
            message: "operation timeout".to_string(),
        };
        let event_error = error.to_event_error();
        // Timeout creates a timeout error with 30 seconds duration - verify it's created properly
        assert!(event_error.to_string().len() > 0);

        // Test Processing
        let error = StreamError::Processing {
            message: "processing failed".to_string(),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("Processing error"));

        // Test Backpressure
        let error = StreamError::Backpressure {
            message: "backpressure detected".to_string(),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("Backpressure error"));

        // Test Internal
        let error = StreamError::Internal {
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "internal error",
            )),
        };
        let event_error = error.to_event_error();
        assert!(event_error.to_string().contains("Internal streaming error"));
    }

    #[test]
    fn test_from_event_error_all_variants() {
        // Test StreamError conversion
        let event_error = EventError::stream_error(
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection failed"),
            "stream context".to_string(),
        );
        let stream_error = StreamError::from_event_error(event_error);
        match stream_error {
            StreamError::Connection { message, retriable } => {
                assert!(message.contains("stream context"));
                assert!(retriable);
            }
            _ => panic!("Expected Connection variant"),
        }

        // Test ParseError conversion
        let event_error = EventError::parse_error(
            serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "parse error",
            )),
            "parse context".to_string(),
        );
        let stream_error = StreamError::from_event_error(event_error);
        match stream_error {
            StreamError::Parse { message, data } => {
                assert_eq!(message, "parse context");
                assert_eq!(data, String::default());
            }
            _ => panic!("Expected Parse variant"),
        }

        // Test Timeout conversion
        let event_error = EventError::timeout(std::time::Duration::from_secs(30));
        let stream_error = StreamError::from_event_error(event_error);
        match stream_error {
            StreamError::Timeout { message } => {
                assert!(message.contains("timed out after"));
            }
            _ => panic!("Expected Timeout variant"),
        }

        // Test InvalidConfig conversion
        let event_error = EventError::invalid_config("invalid configuration".to_string());
        let stream_error = StreamError::from_event_error(event_error);
        match stream_error {
            StreamError::Configuration { message } => {
                assert_eq!(message, "invalid configuration");
            }
            _ => panic!("Expected Configuration variant"),
        }

        // Test NotFound conversion
        let event_error = EventError::not_found("missing resource".to_string());
        let stream_error = StreamError::from_event_error(event_error);
        match stream_error {
            StreamError::Configuration { message } => {
                assert!(message.contains("Resource not found"));
            }
            _ => panic!("Expected Configuration variant"),
        }

        // Test other event error conversion to Internal
        let event_error = EventError::generic("generic error".to_string());
        let stream_error = StreamError::from_event_error(event_error);
        match stream_error {
            StreamError::Internal { source } => {
                assert!(source.to_string().contains("generic error"));
            }
            _ => panic!("Expected Internal variant"),
        }
    }

    #[test]
    fn test_from_io_error() {
        // Test connection-related IO errors
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
        let stream_error = StreamError::from(io_error);
        match stream_error {
            StreamError::Connection { message, retriable } => {
                assert!(message.contains("connection refused"));
                assert!(retriable);
            }
            _ => panic!("Expected Connection variant"),
        }

        let io_error = std::io::Error::new(std::io::ErrorKind::ConnectionReset, "connection reset");
        let stream_error = StreamError::from(io_error);
        match stream_error {
            StreamError::Connection { message, retriable } => {
                assert!(message.contains("connection reset"));
                assert!(retriable);
            }
            _ => panic!("Expected Connection variant"),
        }

        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "connection aborted");
        let stream_error = StreamError::from(io_error);
        match stream_error {
            StreamError::Connection { message, retriable } => {
                assert!(message.contains("connection aborted"));
                assert!(retriable);
            }
            _ => panic!("Expected Connection variant"),
        }

        // Test timeout IO error
        let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "operation timed out");
        let stream_error = StreamError::from(io_error);
        match stream_error {
            StreamError::Timeout { message } => {
                assert!(message.contains("operation timed out"));
            }
            _ => panic!("Expected Timeout variant"),
        }

        // Test other IO error kinds
        let io_error =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let stream_error = StreamError::from(io_error);
        match stream_error {
            StreamError::Internal { source } => {
                assert!(source.to_string().contains("permission denied"));
            }
            _ => panic!("Expected Internal variant"),
        }
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_error = serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid json",
        ));
        let stream_error = StreamError::from(json_error);
        match stream_error {
            StreamError::Parse { message, data } => {
                assert!(message.contains("invalid"));
                assert_eq!(data, String::default());
            }
            _ => panic!("Expected Parse variant"),
        }
    }

    #[test]
    fn test_from_tokio_broadcast_send_error() {
        let (tx, _rx) = tokio::sync::broadcast::channel(1);
        drop(_rx); // Close receiver to cause send error
        let send_result = tx.send("test message");
        let send_error = send_result.unwrap_err();
        let stream_error = StreamError::from(send_error);
        match stream_error {
            StreamError::Channel { message } => {
                assert!(message.contains("Failed to send to broadcast channel"));
            }
            _ => panic!("Expected Channel variant"),
        }
    }

    #[test]
    fn test_stream_error_to_event_error_conversion() {
        let stream_error = StreamError::retriable_connection("test connection error");
        let event_error: EventError = stream_error.into();
        assert!(event_error.to_string().contains("test connection error"));
    }

    #[test]
    fn test_event_error_to_stream_error_conversion() {
        let event_error = EventError::timeout(std::time::Duration::from_secs(10));
        let stream_error: StreamError = event_error.into();
        match stream_error {
            StreamError::Timeout { message } => {
                assert!(message.contains("timed out after"));
            }
            _ => panic!("Expected Timeout variant"),
        }
    }

    #[test]
    fn test_error_display_formatting() {
        // Test error display messages for each variant
        let error = StreamError::Connection {
            message: "test connection".to_string(),
            retriable: true,
        };
        assert_eq!(format!("{}", error), "Connection failed: test connection");

        let error = StreamError::Configuration {
            message: "test config".to_string(),
        };
        assert_eq!(format!("{}", error), "Configuration error: test config");

        let error = StreamError::ConfigurationInvalid {
            reason: "test reason".to_string(),
        };
        assert_eq!(format!("{}", error), "Invalid configuration: test reason");

        let error = StreamError::Authentication {
            message: "test auth".to_string(),
        };
        assert_eq!(format!("{}", error), "Authentication failed: test auth");

        let error = StreamError::RateLimit {
            message: "test rate".to_string(),
            retry_after: Some(60),
        };
        assert_eq!(format!("{}", error), "Rate limit exceeded: test rate");

        let error = StreamError::Parse {
            message: "test parse".to_string(),
            data: "test data".to_string(),
        };
        assert_eq!(format!("{}", error), "Parse error: test parse");

        let error = StreamError::ResourceExhausted {
            message: "test resource".to_string(),
        };
        assert_eq!(format!("{}", error), "Resource exhausted: test resource");

        let error = StreamError::AlreadyRunning {
            name: "test stream".to_string(),
        };
        assert_eq!(format!("{}", error), "Stream already running: test stream");

        let error = StreamError::NotRunning {
            name: "test stream".to_string(),
        };
        assert_eq!(format!("{}", error), "Stream not running: test stream");

        let error = StreamError::Channel {
            message: "test channel".to_string(),
        };
        assert_eq!(format!("{}", error), "Channel error: test channel");

        let error = StreamError::Timeout {
            message: "test timeout".to_string(),
        };
        assert_eq!(format!("{}", error), "Operation timed out: test timeout");

        let error = StreamError::Processing {
            message: "test processing".to_string(),
        };
        assert_eq!(format!("{}", error), "Processing error: test processing");

        let error = StreamError::Backpressure {
            message: "test backpressure".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Backpressure error: test backpressure"
        );

        let error = StreamError::Internal {
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "test internal",
            )),
        };
        assert!(format!("{}", error).starts_with("Internal error:"));
    }

    #[test]
    fn test_stream_result_type_alias() {
        // Test that StreamResult type alias works correctly
        let success: StreamResult<String> = Ok("success".to_string());
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), "success");

        let failure: StreamResult<String> = Err(StreamError::retriable_connection("test error"));
        assert!(failure.is_err());
        match failure.unwrap_err() {
            StreamError::Connection { message, retriable } => {
                assert_eq!(message, "test error");
                assert!(retriable);
            }
            _ => panic!("Expected Connection variant"),
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test empty strings
        let error = StreamError::retriable_connection("");
        match error {
            StreamError::Connection { message, .. } => {
                assert_eq!(message, "");
            }
            _ => panic!("Expected Connection variant"),
        }

        // Test very long strings
        let long_message = "a".repeat(1000);
        let error = StreamError::Configuration {
            message: long_message.clone(),
        };
        match error {
            StreamError::Configuration { message } => {
                assert_eq!(message, long_message);
            }
            _ => panic!("Expected Configuration variant"),
        }

        // Test zero retry_after
        let error = StreamError::rate_limited("test", 0);
        assert_eq!(error.retry_after(), Some(std::time::Duration::from_secs(0)));

        // Test maximum u64 retry_after
        let error = StreamError::rate_limited("test", u64::MAX);
        assert_eq!(
            error.retry_after(),
            Some(std::time::Duration::from_secs(u64::MAX))
        );
    }
}
