use thiserror::Error;
use riglr_events_core::EventError;

/// Comprehensive error type for streaming operations
/// 
/// This extends EventError with streaming-specific error variants while
/// maintaining compatibility with the events-core error hierarchy.
#[derive(Error, Debug)]
pub enum StreamError {
    /// Connection-related errors
    #[error("Connection failed: {message}")]
    Connection {
        message: String,
        retriable: bool,
    },
    
    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
    },
    
    /// Invalid configuration
    #[error("Invalid configuration: {reason}")]
    ConfigurationInvalid {
        reason: String,
    },
    
    /// Authentication failures
    #[error("Authentication failed: {message}")]
    Authentication {
        message: String,
    },
    
    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        retry_after: Option<u64>,
    },
    
    /// Data parsing errors
    #[error("Parse error: {message}")]
    Parse {
        message: String,
        data: String,
    },
    
    /// Resource exhaustion
    #[error("Resource exhausted: {message}")]
    ResourceExhausted {
        message: String,
    },
    
    /// Stream already running
    #[error("Stream already running: {name}")]
    AlreadyRunning {
        name: String,
    },
    
    /// Stream not running
    #[error("Stream not running: {name}")]
    NotRunning {
        name: String,
    },
    
    /// Channel errors
    #[error("Channel error: {message}")]
    Channel {
        message: String,
    },
    
    /// Timeout errors
    #[error("Operation timed out: {message}")]
    Timeout {
        message: String,
    },
    
    /// Processing errors
    #[error("Processing error: {message}")]
    Processing {
        message: String,
    },
    
    /// Backpressure errors
    #[error("Backpressure error: {message}")]
    Backpressure {
        message: String,
    },
    
    /// Internal errors
    #[error("Internal error: {source}")]
    Internal {
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
            StreamError::RateLimit { retry_after: Some(seconds), .. } => {
                Some(std::time::Duration::from_secs(*seconds))
            }
            _ => None,
        }
    }

    /// Convert StreamError to EventError for integration with events-core
    pub fn to_event_error(self) -> EventError {
        match self {
            StreamError::Connection { message, retriable: true } => {
                EventError::stream_error(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, message.clone()), message)
            }
            StreamError::Connection { message, retriable: false } => {
                EventError::generic(format!("Permanent connection error: {}", message))
            }
            StreamError::Configuration { message } | StreamError::ConfigurationInvalid { reason: message } => {
                EventError::invalid_config(message)
            }
            StreamError::Authentication { message } => {
                EventError::generic(format!("Authentication error: {}", message))
            }
            StreamError::RateLimit { message, .. } => {
                EventError::stream_error(std::io::Error::new(std::io::ErrorKind::Other, message.clone()), format!("Rate limit: {}", message))
            }
            StreamError::Parse { message, .. } => {
                EventError::parse_error(serde_json::Error::from(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, "parse error"))), message)
            }
            StreamError::ResourceExhausted { message } => {
                EventError::stream_error(std::io::Error::new(std::io::ErrorKind::Other, message.clone()), format!("Resource exhausted: {}", message))
            }
            StreamError::AlreadyRunning { name } => {
                EventError::generic(format!("Stream already running: {}", name))
            }
            StreamError::NotRunning { name } => {
                EventError::generic(format!("Stream not running: {}", name))
            }
            StreamError::Channel { message } => {
                EventError::stream_error(std::io::Error::new(std::io::ErrorKind::BrokenPipe, message.clone()), format!("Channel error: {}", message))
            }
            StreamError::Timeout { message: _ } => {
                EventError::timeout(std::time::Duration::from_secs(30))
            }
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
            EventError::ParseError { context, .. } => {
                StreamError::Parse {
                    message: context,
                    data: String::new(),
                }
            }
            EventError::Timeout { duration } => {
                StreamError::Timeout {
                    message: format!("Operation timed out after {:?}", duration),
                }
            }
            EventError::InvalidConfig { message } => {
                StreamError::Configuration { message }
            }
            EventError::NotFound { resource } => {
                StreamError::Configuration {
                    message: format!("Resource not found: {}", resource),
                }
            }
            _ => StreamError::Internal {
                source: Box::new(event_error),
            }
        }
    }
}

/// Result type for streaming operations
pub type StreamResult<T> = Result<T, StreamError>;

/// Convert from standard IO errors
impl From<std::io::Error> for StreamError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::ConnectionRefused |
            std::io::ErrorKind::ConnectionReset |
            std::io::ErrorKind::ConnectionAborted => {
                StreamError::retriable_connection(err.to_string())
            }
            std::io::ErrorKind::TimedOut => {
                StreamError::Timeout {
                    message: err.to_string(),
                }
            }
            _ => StreamError::Internal {
                source: Box::new(err),
            }
        }
    }
}

/// Convert from serde_json errors
impl From<serde_json::Error> for StreamError {
    fn from(err: serde_json::Error) -> Self {
        StreamError::Parse {
            message: err.to_string(),
            data: String::new(),
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
        assert!(!StreamError::Configuration { message: "test".to_string() }.is_retriable());
        assert!(!StreamError::Authentication { message: "test".to_string() }.is_retriable());
        assert!(StreamError::Timeout { message: "test".to_string() }.is_retriable());
    }
    
    #[test]
    fn test_retry_after() {
        let err = StreamError::rate_limited("test", 60);
        assert_eq!(err.retry_after(), Some(std::time::Duration::from_secs(60)));
        
        let err = StreamError::retriable_connection("test");
        assert_eq!(err.retry_after(), None);
    }
    
}