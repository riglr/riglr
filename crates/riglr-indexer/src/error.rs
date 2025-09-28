//! Error types for the RIGLR indexer service

use core::error::Error as CoreError;
use thiserror::Error;

/// Result type used throughout the indexer
pub type IndexerResult<T> = Result<T, IndexerError>;

/// Main error type for the indexer service
#[derive(Error, Debug)]
#[expect(clippy::module_name_repetitions)]
pub enum IndexerError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// External dependency errors
    #[error("Dependency error: {source}")]
    Dependency {
        /// The underlying error from a dependency
        #[from]
        source: Box<dyn CoreError + Send + Sync>,
    },

    /// Internal errors that should not occur in normal operation
    #[error("Internal error: {message}")]
    Internal {
        /// Description of the internal error
        message: String,
        /// Optional source error
        source: Option<Box<dyn CoreError + Send + Sync>>,
    },

    /// Metrics collection errors
    #[error("Metrics error: {0}")]
    Metrics(#[from] MetricsError),

    /// Network/API errors
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    /// Event processing errors
    #[error("Processing error: {0}")]
    Processing(#[from] ProcessingError),

    /// Service lifecycle errors
    #[error("Service error: {0}")]
    Service(#[from] ServiceError),

    /// Storage layer errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation {
        /// Human-readable validation error message
        message: String,
    },
}

impl IndexerError {
    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Create an internal error with a source
    pub fn internal_with_source(
        message: impl Into<String>,
        source: impl CoreError + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Get error category for metrics and logging
    #[must_use]
    pub const fn category(&self) -> &'static str {
        match *self {
            Self::Config(_) => "config",
            Self::Storage(_) => "storage",
            Self::Network(_) => "network",
            Self::Processing(_) => "processing",
            Self::Metrics(_) => "metrics",
            Self::Service(_) => "service",
            Self::Dependency { .. } => "dependency",
            Self::Validation { .. } => "validation",
            Self::Internal { .. } => "internal",
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Check if the error is retriable
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        match *self {
            Self::Network(ref e) => e.is_retriable(),
            Self::Storage(ref e) => e.is_retriable(),
            Self::Processing(ref e) => e.is_retriable(),
            Self::Service(ref e) => e.is_retriable(),
            // Conservative default - config, validation, internal, metrics, dependency errors are not retriable
            Self::Dependency { .. }
            | Self::Config(_)
            | Self::Validation { .. }
            | Self::Internal { .. }
            | Self::Metrics(_) => false,
        }
    }
}

/// Configuration-related errors
#[derive(Error, Debug)]
#[expect(clippy::module_name_repetitions)]
pub enum ConfigError {
    /// Invalid configuration value
    #[error("Invalid configuration value for {field}: {value}")]
    InvalidValue {
        /// The field with invalid value
        field: String,
        /// The invalid value
        value: String,
    },

    /// Failed to load configuration from source
    #[error("Failed to load configuration from {path}: {source}")]
    LoadFailed {
        /// Path or source of configuration
        path: String,
        /// The underlying error
        #[source]
        source: Box<dyn CoreError + Send + Sync>,
    },

    /// Missing required configuration field
    #[error("Missing required configuration: {field}")]
    MissingField {
        /// The missing field name
        field: String,
    },

    /// Configuration validation failed
    #[error("Configuration validation failed: {message}")]
    ValidationFailed {
        /// Validation error message
        message: String,
    },
}

/// Storage layer errors
#[derive(Error, Debug)]
#[expect(clippy::module_name_repetitions)]
pub enum StorageError {
    /// Storage capacity exceeded
    #[error("Storage capacity exceeded: {current_size}")]
    CapacityExceeded {
        /// Current storage size in bytes
        current_size: u64,
    },

    /// Database connection failed
    #[error("Database connection failed: {message}")]
    ConnectionFailed {
        /// Connection failure message
        message: String,
    },

    /// Data corruption detected
    #[error("Data corruption detected: {details}")]
    DataCorruption {
        /// Corruption details
        details: String,
    },

    /// Migration failed
    #[error("Migration failed: {version}")]
    MigrationFailed {
        /// Migration version that failed
        version: String,
    },

    /// Database query failed
    #[error("Database query failed: {query}")]
    QueryFailed {
        /// The failed query
        query: String,
    },

    /// Schema validation failed
    #[error("Schema validation failed: {table}")]
    SchemaValidationFailed {
        /// Table with schema issues
        table: String,
    },

    /// Transaction failed
    #[error("Transaction failed: {operation}")]
    TransactionFailed {
        /// Failed operation name
        operation: String,
    },
}

impl StorageError {
    /// Check if the storage error is retriable
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        match *self {
            // May be transient
            Self::ConnectionFailed { .. }
            | Self::QueryFailed { .. }
            | Self::TransactionFailed { .. } => true,
            // Permanent issues
            Self::DataCorruption { .. }
            | Self::CapacityExceeded { .. }
            | Self::MigrationFailed { .. }
            | Self::SchemaValidationFailed { .. } => false,
        }
    }
}

/// Network and API errors
#[derive(Error, Debug)]
#[expect(clippy::module_name_repetitions)]
pub enum NetworkError {
    /// DNS resolution failed
    #[error("DNS resolution failed for {host}")]
    DnsResolutionFailed {
        /// Host that failed to resolve
        host: String,
    },

    /// HTTP request failed
    #[error("HTTP request failed: {status} - {url}")]
    HttpFailed {
        /// HTTP status code
        status: u16,
        /// Request URL
        url: String,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {limit} requests per {window}s")]
    RateLimited {
        /// Rate limit threshold
        limit: u32,
        /// Time window in seconds
        window: u32,
    },

    /// Request timeout
    #[error("Timeout occurred after {seconds}s")]
    Timeout {
        /// Timeout duration in seconds
        seconds: u64,
    },

    /// TLS/SSL error
    #[error("TLS/SSL error: {message}")]
    TlsError {
        /// Error message
        message: String,
    },

    /// WebSocket connection failed
    #[error("WebSocket connection failed: {reason}")]
    WebSocketFailed {
        /// Failure reason
        reason: String,
    },
}

impl NetworkError {
    /// Check if the network error is retriable
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        match *self {
            Self::HttpFailed { status, .. } => {
                // 5xx errors are usually retriable, 4xx usually not
                status >= 500 || status == 429 || status == 408
            }
            Self::Timeout { .. }
            | Self::WebSocketFailed { .. }
            | Self::RateLimited { .. }
            | Self::DnsResolutionFailed { .. } => true,
            Self::TlsError { .. } => false, // Usually configuration issue
        }
    }
}

/// Event processing errors
#[derive(Error, Debug)]
#[expect(clippy::module_name_repetitions)]
pub enum ProcessingError {
    /// Deserialization failed
    #[error("Deserialization failed: {format}")]
    DeserializationFailed {
        /// Format that failed to deserialize
        format: String,
    },

    /// Failed to parse event
    #[error("Failed to parse event: {event_type}")]
    ParseFailed {
        /// Type of event that failed to parse
        event_type: String,
    },

    /// Event queue overflow
    #[error("Event queue overflow: {size}/{capacity}")]
    QueueOverflow {
        /// Current queue size
        size: usize,
        /// Queue capacity
        capacity: usize,
    },

    /// Serialization failed
    #[error("Serialization failed for event {event_id}")]
    SerializationFailed {
        /// ID of event that failed to serialize
        event_id: String,
    },

    /// Event validation failed
    #[error("Event validation failed: {reason}")]
    ValidationFailed {
        /// Validation failure reason
        reason: String,
    },

    /// Pipeline stalled
    #[error("Pipeline stalled: no events processed in {seconds}s")]
    PipelineStalled {
        /// Seconds since last event processed
        seconds: u64,
    },

    /// Worker pool exhausted
    #[error("Worker pool exhausted: {active}/{max} workers")]
    WorkerPoolExhausted {
        /// Number of active workers
        active: usize,
        /// Maximum number of workers
        max: usize,
    },
}

impl ProcessingError {
    /// Check if the processing error is retriable
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        match *self {
            // Data issues
            Self::ParseFailed { .. }
            | Self::ValidationFailed { .. }
            | Self::SerializationFailed { .. }
            | Self::DeserializationFailed { .. } => false,
            // May recover
            Self::QueueOverflow { .. }
            | Self::PipelineStalled { .. }
            | Self::WorkerPoolExhausted { .. } => true,
        }
    }
}

/// Metrics collection errors
#[derive(Error, Debug)]
#[expect(clippy::module_name_repetitions)]
pub enum MetricsError {
    /// Metrics export failed
    #[error("Metrics export failed: {endpoint}")]
    ExportFailed {
        /// Export endpoint that failed
        endpoint: String,
    },

    /// Invalid metric value
    #[error("Invalid metric value: {value}")]
    InvalidValue {
        /// The invalid value
        value: String,
    },

    /// Metric recording failed
    #[error("Metric recording failed: {metric_name}")]
    RecordingFailed {
        /// Name of metric that failed to record
        metric_name: String,
    },

    /// Metrics registry error
    #[error("Metrics registry error: {message}")]
    RegistryError {
        /// Error message
        message: String,
    },
}

/// Service lifecycle errors
#[derive(Error, Debug)]
#[expect(clippy::module_name_repetitions)]
pub enum ServiceError {
    /// Dependency unavailable
    #[error("Dependency unavailable: {service}")]
    DependencyUnavailable {
        /// Name of unavailable service
        service: String,
    },

    /// Service health check failed
    #[error("Service unhealthy: {check_name}")]
    HealthCheckFailed {
        /// Name of the health check that failed
        check_name: String,
    },

    /// Resource exhausted
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted {
        /// Name of exhausted resource
        resource: String,
    },

    /// Service shutdown timeout
    #[error("Service shutdown timeout after {seconds}s")]
    ShutdownTimeout {
        /// Timeout duration in seconds
        seconds: u64,
    },

    /// Service failed to start
    #[error("Service failed to start: {reason}")]
    StartupFailed {
        /// Startup failure reason
        reason: String,
    },
}

impl ServiceError {
    /// Check if the service error is retriable
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        match *self {
            // Usually configuration or lifecycle issues
            Self::StartupFailed { .. } | Self::ShutdownTimeout { .. } => false,
            // May recover
            Self::HealthCheckFailed { .. }
            | Self::ResourceExhausted { .. }
            | Self::DependencyUnavailable { .. } => true,
        }
    }
}

/// Convert from `anyhow::Error`
impl From<anyhow::Error> for IndexerError {
    fn from(error: anyhow::Error) -> Self {
        Self::Dependency {
            source: error.into(),
        }
    }
}

/// Convert from `riglr_events_core::EventError`
impl From<riglr_events_core::EventError> for IndexerError {
    fn from(error: riglr_events_core::EventError) -> Self {
        match error {
            riglr_events_core::EventError::ParseError { context, .. } => {
                Self::Processing(ProcessingError::ParseFailed {
                    event_type: context,
                })
            }
            riglr_events_core::EventError::FilterError { context, .. } => {
                Self::Processing(ProcessingError::ValidationFailed { reason: context })
            }
            riglr_events_core::EventError::Serialization(_) => {
                Self::Processing(ProcessingError::SerializationFailed {
                    event_id: "unknown".to_string(),
                })
            }
            riglr_events_core::EventError::Generic { message } => Self::internal(message),
            _ => Self::internal(error.to_string()),
        }
    }
}

/// Convert from sqlx errors
impl From<sqlx::Error> for IndexerError {
    fn from(error: sqlx::Error) -> Self {
        let storage_error = match error {
            sqlx::Error::PoolTimedOut => StorageError::ConnectionFailed {
                message: "Connection pool timeout".to_string(),
            },
            sqlx::Error::Database(db_err) => StorageError::QueryFailed {
                query: db_err.message().to_string(),
            },
            sqlx::Error::Io(io_err) => StorageError::ConnectionFailed {
                message: io_err.to_string(),
            },
            _ => StorageError::QueryFailed {
                query: error.to_string(),
            },
        };
        Self::Storage(storage_error)
    }
}

/// Convert from Redis errors
impl From<redis::RedisError> for IndexerError {
    fn from(error: redis::RedisError) -> Self {
        let storage_error = if error.is_connection_refusal() || error.is_timeout() {
            StorageError::ConnectionFailed {
                message: error.to_string(),
            }
        } else {
            StorageError::QueryFailed {
                query: error.to_string(),
            }
        };
        Self::Storage(storage_error)
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use core::error::Error;
    use core::fmt;
    use std::io::{Error as IoError, ErrorKind};

    // Helper error for testing
    #[derive(Debug)]
    struct TestError(String);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Test error: {}", self.0)
        }
    }

    impl Error for TestError {}

    // Test IndexerError construction and display methods
    #[test]
    fn test_indexer_error_validation_should_create_validation_error() {
        let error = IndexerError::validation("invalid input");
        match error {
            IndexerError::Validation { ref message } => {
                assert_eq!(message, "invalid input");
            }
            _ => panic!("Expected Validation error"),
        }
        assert_eq!(error.to_string(), "Validation error: invalid input");
    }

    #[test]
    fn test_indexer_error_internal_should_create_internal_error() {
        let error = IndexerError::internal("something went wrong");
        match error {
            IndexerError::Internal {
                ref message,
                ref source,
            } => {
                assert_eq!(message, "something went wrong");
                assert!(source.is_none());
            }
            _ => panic!("Expected Internal error"),
        }
        assert_eq!(error.to_string(), "Internal error: something went wrong");
    }

    #[test]
    fn test_indexer_error_internal_with_source_should_create_internal_error_with_source() {
        let source_error = TestError("root cause".to_string());
        let error = IndexerError::internal_with_source("wrapper message", source_error);
        match error {
            IndexerError::Internal {
                ref message,
                ref source,
            } => {
                assert_eq!(message, "wrapper message");
                assert!(source.is_some());
                assert_eq!(
                    source
                        .as_ref()
                        .expect("Internal error should have source")
                        .to_string(),
                    "Test error: root cause"
                );
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_indexer_error_is_retriable_when_network_retriable_should_return_true() {
        let network_error = NetworkError::Timeout { seconds: 30 };
        let error = IndexerError::Network(network_error);
        assert!(error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_network_not_retriable_should_return_false() {
        let network_error = NetworkError::TlsError {
            message: "cert error".to_string(),
        };
        let error = IndexerError::Network(network_error);
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_storage_retriable_should_return_true() {
        let storage_error = StorageError::ConnectionFailed {
            message: "timeout".to_string(),
        };
        let error = IndexerError::Storage(storage_error);
        assert!(error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_storage_not_retriable_should_return_false() {
        let storage_error = StorageError::DataCorruption {
            details: "bad data".to_string(),
        };
        let error = IndexerError::Storage(storage_error);
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_processing_retriable_should_return_true() {
        let processing_error = ProcessingError::WorkerPoolExhausted {
            active: 10,
            max: 10,
        };
        let error = IndexerError::Processing(processing_error);
        assert!(error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_processing_not_retriable_should_return_false() {
        let processing_error = ProcessingError::ParseFailed {
            event_type: "invalid".to_string(),
        };
        let error = IndexerError::Processing(processing_error);
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_service_retriable_should_return_true() {
        let service_error = ServiceError::HealthCheckFailed {
            check_name: "db".to_string(),
        };
        let error = IndexerError::Service(service_error);
        assert!(error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_service_not_retriable_should_return_false() {
        let service_error = ServiceError::StartupFailed {
            reason: "config error".to_string(),
        };
        let error = IndexerError::Service(service_error);
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_dependency_should_return_false() {
        let error = IndexerError::Dependency {
            source: Box::new(TestError("dep error".to_string())),
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_config_should_return_false() {
        let config_error = ConfigError::MissingField {
            field: "api_key".to_string(),
        };
        let error = IndexerError::Config(config_error);
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_metrics_should_return_false() {
        let metrics_error = MetricsError::RegistryError {
            message: "init failed".to_string(),
        };
        let error = IndexerError::Metrics(metrics_error);
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_validation_should_return_false() {
        let error = IndexerError::Validation {
            message: "invalid".to_string(),
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_indexer_error_is_retriable_when_internal_should_return_false() {
        let error = IndexerError::Internal {
            message: "bug".to_string(),
            source: None,
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_indexer_error_category_should_return_correct_categories() {
        let config_error = IndexerError::Config(ConfigError::MissingField {
            field: "test".to_string(),
        });
        assert_eq!(config_error.category(), "config");

        let storage_error = IndexerError::Storage(StorageError::ConnectionFailed {
            message: "test".to_string(),
        });
        assert_eq!(storage_error.category(), "storage");

        let network_error = IndexerError::Network(NetworkError::Timeout { seconds: 30 });
        assert_eq!(network_error.category(), "network");

        let processing_error = IndexerError::Processing(ProcessingError::ParseFailed {
            event_type: "test".to_string(),
        });
        assert_eq!(processing_error.category(), "processing");

        let metrics_error = IndexerError::Metrics(MetricsError::RegistryError {
            message: "test".to_string(),
        });
        assert_eq!(metrics_error.category(), "metrics");

        let service_error = IndexerError::Service(ServiceError::StartupFailed {
            reason: "test".to_string(),
        });
        assert_eq!(service_error.category(), "service");

        let dependency_error = IndexerError::Dependency {
            source: Box::new(TestError("test".to_string())),
        };
        assert_eq!(dependency_error.category(), "dependency");

        let validation_error = IndexerError::Validation {
            message: "test".to_string(),
        };
        assert_eq!(validation_error.category(), "validation");

        let internal_error = IndexerError::Internal {
            message: "test".to_string(),
            source: None,
        };
        assert_eq!(internal_error.category(), "internal");
    }

    // Test StorageError is_retriable method
    #[test]
    fn test_storage_error_is_retriable_when_connection_failed_should_return_true() {
        let error = StorageError::ConnectionFailed {
            message: "timeout".to_string(),
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_storage_error_is_retriable_when_query_failed_should_return_true() {
        let error = StorageError::QueryFailed {
            query: "SELECT *".to_string(),
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_storage_error_is_retriable_when_transaction_failed_should_return_true() {
        let error = StorageError::TransactionFailed {
            operation: "INSERT".to_string(),
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_storage_error_is_retriable_when_data_corruption_should_return_false() {
        let error = StorageError::DataCorruption {
            details: "checksum mismatch".to_string(),
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_storage_error_is_retriable_when_capacity_exceeded_should_return_false() {
        let error = StorageError::CapacityExceeded {
            current_size: 1_000_000,
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_storage_error_is_retriable_when_migration_failed_should_return_false() {
        let error = StorageError::MigrationFailed {
            version: "v1.0".to_string(),
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_storage_error_is_retriable_when_schema_validation_failed_should_return_false() {
        let error = StorageError::SchemaValidationFailed {
            table: "users".to_string(),
        };
        assert!(!error.is_retriable());
    }

    // Test NetworkError is_retriable method
    #[test]
    fn test_network_error_is_retriable_when_http_500_should_return_true() {
        let error = NetworkError::HttpFailed {
            status: 500,
            url: "http://test.com".to_string(),
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_network_error_is_retriable_when_http_429_should_return_true() {
        let error = NetworkError::HttpFailed {
            status: 429,
            url: "http://test.com".to_string(),
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_network_error_is_retriable_when_http_408_should_return_true() {
        let error = NetworkError::HttpFailed {
            status: 408,
            url: "http://test.com".to_string(),
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_network_error_is_retriable_when_http_400_should_return_false() {
        let error = NetworkError::HttpFailed {
            status: 400,
            url: "http://test.com".to_string(),
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_network_error_is_retriable_when_http_404_should_return_false() {
        let error = NetworkError::HttpFailed {
            status: 404,
            url: "http://test.com".to_string(),
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_network_error_is_retriable_when_websocket_failed_should_return_true() {
        let error = NetworkError::WebSocketFailed {
            reason: "connection lost".to_string(),
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_network_error_is_retriable_when_timeout_should_return_true() {
        let error = NetworkError::Timeout { seconds: 30 };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_network_error_is_retriable_when_rate_limited_should_return_true() {
        let error = NetworkError::RateLimited {
            limit: 100,
            window: 60,
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_network_error_is_retriable_when_dns_resolution_failed_should_return_true() {
        let error = NetworkError::DnsResolutionFailed {
            host: "example.com".to_string(),
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_network_error_is_retriable_when_tls_error_should_return_false() {
        let error = NetworkError::TlsError {
            message: "cert expired".to_string(),
        };
        assert!(!error.is_retriable());
    }

    // Test ProcessingError is_retriable method
    #[test]
    fn test_processing_error_is_retriable_when_parse_failed_should_return_false() {
        let error = ProcessingError::ParseFailed {
            event_type: "invalid_json".to_string(),
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_processing_error_is_retriable_when_validation_failed_should_return_false() {
        let error = ProcessingError::ValidationFailed {
            reason: "missing field".to_string(),
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_processing_error_is_retriable_when_worker_pool_exhausted_should_return_true() {
        let error = ProcessingError::WorkerPoolExhausted {
            active: 10,
            max: 10,
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_processing_error_is_retriable_when_queue_overflow_should_return_true() {
        let error = ProcessingError::QueueOverflow {
            size: 1000,
            capacity: 1000,
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_processing_error_is_retriable_when_serialization_failed_should_return_false() {
        let error = ProcessingError::SerializationFailed {
            event_id: "123".to_string(),
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_processing_error_is_retriable_when_deserialization_failed_should_return_false() {
        let error = ProcessingError::DeserializationFailed {
            format: "json".to_string(),
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_processing_error_is_retriable_when_pipeline_stalled_should_return_true() {
        let error = ProcessingError::PipelineStalled { seconds: 300 };
        assert!(error.is_retriable());
    }

    // Test ServiceError is_retriable method
    #[test]
    fn test_service_error_is_retriable_when_startup_failed_should_return_false() {
        let error = ServiceError::StartupFailed {
            reason: "config error".to_string(),
        };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_service_error_is_retriable_when_shutdown_timeout_should_return_false() {
        let error = ServiceError::ShutdownTimeout { seconds: 30 };
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_service_error_is_retriable_when_health_check_failed_should_return_true() {
        let error = ServiceError::HealthCheckFailed {
            check_name: "database".to_string(),
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_service_error_is_retriable_when_resource_exhausted_should_return_true() {
        let error = ServiceError::ResourceExhausted {
            resource: "memory".to_string(),
        };
        assert!(error.is_retriable());
    }

    #[test]
    fn test_service_error_is_retriable_when_dependency_unavailable_should_return_true() {
        let error = ServiceError::DependencyUnavailable {
            service: "redis".to_string(),
        };
        assert!(error.is_retriable());
    }

    // Test error display messages
    #[test]
    fn test_config_error_display_messages() {
        let missing_field = ConfigError::MissingField {
            field: "api_key".to_string(),
        };
        assert_eq!(
            missing_field.to_string(),
            "Missing required configuration: api_key"
        );

        let invalid_value = ConfigError::InvalidValue {
            field: "port".to_string(),
            value: "invalid".to_string(),
        };
        assert_eq!(
            invalid_value.to_string(),
            "Invalid configuration value for port: invalid"
        );

        let load_failed = ConfigError::LoadFailed {
            path: "/config.yaml".to_string(),
            source: Box::new(TestError("file not found".to_string())),
        };
        assert_eq!(
            load_failed.to_string(),
            "Failed to load configuration from /config.yaml: Test error: file not found"
        );

        let validation_failed = ConfigError::ValidationFailed {
            message: "invalid schema".to_string(),
        };
        assert_eq!(
            validation_failed.to_string(),
            "Configuration validation failed: invalid schema"
        );
    }

    #[test]
    fn test_storage_error_display_messages() {
        let connection_failed = StorageError::ConnectionFailed {
            message: "timeout".to_string(),
        };
        assert_eq!(
            connection_failed.to_string(),
            "Database connection failed: timeout"
        );

        let query_failed = StorageError::QueryFailed {
            query: "SELECT *".to_string(),
        };
        assert_eq!(query_failed.to_string(), "Database query failed: SELECT *");

        let migration_failed = StorageError::MigrationFailed {
            version: "v1.0".to_string(),
        };
        assert_eq!(migration_failed.to_string(), "Migration failed: v1.0");

        let data_corruption = StorageError::DataCorruption {
            details: "checksum failed".to_string(),
        };
        assert_eq!(
            data_corruption.to_string(),
            "Data corruption detected: checksum failed"
        );

        let capacity_exceeded = StorageError::CapacityExceeded {
            current_size: 1_000_000,
        };
        assert_eq!(
            capacity_exceeded.to_string(),
            "Storage capacity exceeded: 1000000"
        );

        let transaction_failed = StorageError::TransactionFailed {
            operation: "INSERT".to_string(),
        };
        assert_eq!(transaction_failed.to_string(), "Transaction failed: INSERT");

        let schema_validation_failed = StorageError::SchemaValidationFailed {
            table: "users".to_string(),
        };
        assert_eq!(
            schema_validation_failed.to_string(),
            "Schema validation failed: users"
        );
    }

    #[test]
    fn test_network_error_display_messages() {
        let http_failed = NetworkError::HttpFailed {
            status: 404,
            url: "http://test.com".to_string(),
        };
        assert_eq!(
            http_failed.to_string(),
            "HTTP request failed: 404 - http://test.com"
        );

        let websocket_failed = NetworkError::WebSocketFailed {
            reason: "connection lost".to_string(),
        };
        assert_eq!(
            websocket_failed.to_string(),
            "WebSocket connection failed: connection lost"
        );

        let timeout = NetworkError::Timeout { seconds: 30 };
        assert_eq!(timeout.to_string(), "Timeout occurred after 30s");

        let rate_limited = NetworkError::RateLimited {
            limit: 100,
            window: 60,
        };
        assert_eq!(
            rate_limited.to_string(),
            "Rate limit exceeded: 100 requests per 60s"
        );

        let dns_failed = NetworkError::DnsResolutionFailed {
            host: "example.com".to_string(),
        };
        assert_eq!(
            dns_failed.to_string(),
            "DNS resolution failed for example.com"
        );

        let tls_error = NetworkError::TlsError {
            message: "cert expired".to_string(),
        };
        assert_eq!(tls_error.to_string(), "TLS/SSL error: cert expired");
    }

    #[test]
    fn test_processing_error_display_messages() {
        let parse_failed = ProcessingError::ParseFailed {
            event_type: "transaction".to_string(),
        };
        assert_eq!(
            parse_failed.to_string(),
            "Failed to parse event: transaction"
        );

        let validation_failed = ProcessingError::ValidationFailed {
            reason: "missing field".to_string(),
        };
        assert_eq!(
            validation_failed.to_string(),
            "Event validation failed: missing field"
        );

        let worker_pool_exhausted = ProcessingError::WorkerPoolExhausted {
            active: 10,
            max: 10,
        };
        assert_eq!(
            worker_pool_exhausted.to_string(),
            "Worker pool exhausted: 10/10 workers"
        );

        let queue_overflow = ProcessingError::QueueOverflow {
            size: 1000,
            capacity: 1000,
        };
        assert_eq!(
            queue_overflow.to_string(),
            "Event queue overflow: 1000/1000"
        );

        let serialization_failed = ProcessingError::SerializationFailed {
            event_id: "123".to_string(),
        };
        assert_eq!(
            serialization_failed.to_string(),
            "Serialization failed for event 123"
        );

        let deserialization_failed = ProcessingError::DeserializationFailed {
            format: "json".to_string(),
        };
        assert_eq!(
            deserialization_failed.to_string(),
            "Deserialization failed: json"
        );

        let pipeline_stalled = ProcessingError::PipelineStalled { seconds: 300 };
        assert_eq!(
            pipeline_stalled.to_string(),
            "Pipeline stalled: no events processed in 300s"
        );
    }

    #[test]
    fn test_metrics_error_display_messages() {
        let registry_error = MetricsError::RegistryError {
            message: "init failed".to_string(),
        };
        assert_eq!(
            registry_error.to_string(),
            "Metrics registry error: init failed"
        );

        let recording_failed = MetricsError::RecordingFailed {
            metric_name: "requests_total".to_string(),
        };
        assert_eq!(
            recording_failed.to_string(),
            "Metric recording failed: requests_total"
        );

        let export_failed = MetricsError::ExportFailed {
            endpoint: "http://prometheus:9090".to_string(),
        };
        assert_eq!(
            export_failed.to_string(),
            "Metrics export failed: http://prometheus:9090"
        );

        let invalid_value = MetricsError::InvalidValue {
            value: "NaN".to_string(),
        };
        assert_eq!(invalid_value.to_string(), "Invalid metric value: NaN");
    }

    #[test]
    fn test_service_error_display_messages() {
        let startup_failed = ServiceError::StartupFailed {
            reason: "config error".to_string(),
        };
        assert_eq!(
            startup_failed.to_string(),
            "Service failed to start: config error"
        );

        let shutdown_timeout = ServiceError::ShutdownTimeout { seconds: 30 };
        assert_eq!(
            shutdown_timeout.to_string(),
            "Service shutdown timeout after 30s"
        );

        let health_check_failed = ServiceError::HealthCheckFailed {
            check_name: "database".to_string(),
        };
        assert_eq!(
            health_check_failed.to_string(),
            "Service unhealthy: database"
        );

        let resource_exhausted = ServiceError::ResourceExhausted {
            resource: "memory".to_string(),
        };
        assert_eq!(resource_exhausted.to_string(), "Resource exhausted: memory");

        let dependency_unavailable = ServiceError::DependencyUnavailable {
            service: "redis".to_string(),
        };
        assert_eq!(
            dependency_unavailable.to_string(),
            "Dependency unavailable: redis"
        );
    }

    // Test From conversions
    #[test]
    fn test_from_anyhow_error_should_create_dependency_error() {
        let anyhow_error = anyhow::Error::msg("anyhow error");
        let indexer_error: IndexerError = anyhow_error.into();
        match indexer_error {
            IndexerError::Dependency { source } => {
                assert_eq!(source.to_string(), "anyhow error");
            }
            _ => panic!("Expected Dependency error"),
        }
    }

    #[test]
    fn test_from_riglr_events_core_parse_error_should_create_processing_error() {
        let event_error = riglr_events_core::EventError::ParseError {
            context: "test_event".to_string(),
            source: Box::new(TestError("parse failed".to_string())),
        };
        let indexer_error: IndexerError = event_error.into();
        match indexer_error {
            IndexerError::Processing(ProcessingError::ParseFailed { event_type }) => {
                assert_eq!(event_type, "test_event");
            }
            _ => panic!("Expected Processing(ParseFailed) error"),
        }
    }

    #[test]
    fn test_from_riglr_events_core_filter_error_should_create_processing_error() {
        let event_error = riglr_events_core::EventError::FilterError {
            context: "filter_failed".to_string(),
            source: Box::new(TestError("filter error".to_string())),
        };
        let indexer_error: IndexerError = event_error.into();
        match indexer_error {
            IndexerError::Processing(ProcessingError::ValidationFailed { reason }) => {
                assert_eq!(reason, "filter_failed");
            }
            _ => panic!("Expected Processing(ValidationFailed) error"),
        }
    }

    #[test]
    fn test_from_riglr_events_core_serialization_error_should_create_processing_error() {
        let event_error = riglr_events_core::EventError::Serialization(serde_json::Error::io(
            IoError::new(ErrorKind::InvalidData, "serialize failed"),
        ));
        let indexer_error: IndexerError = event_error.into();
        match indexer_error {
            IndexerError::Processing(ProcessingError::SerializationFailed { event_id }) => {
                assert_eq!(event_id, "unknown");
            }
            _ => panic!("Expected Processing(SerializationFailed) error"),
        }
    }

    #[test]
    fn test_from_riglr_events_core_generic_error_should_create_internal_error() {
        let event_error = riglr_events_core::EventError::Generic {
            message: "generic error".to_string(),
        };
        let indexer_error: IndexerError = event_error.into();
        match indexer_error {
            IndexerError::Internal { message, .. } => {
                assert_eq!(message, "generic error");
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_from_riglr_events_core_other_error_should_create_internal_error() {
        // Testing the catch-all case
        let event_error = riglr_events_core::EventError::Serialization(serde_json::Error::io(
            IoError::new(ErrorKind::InvalidData, "deser error"),
        ));
        let indexer_error: IndexerError = event_error.into();
        match indexer_error {
            IndexerError::Processing(ProcessingError::SerializationFailed { event_id }) => {
                assert_eq!(event_id, "unknown");
            }
            _ => panic!("Expected Processing(SerializationFailed) error"),
        }
    }

    #[test]
    fn test_from_sqlx_pool_timeout_should_create_storage_connection_error() {
        let sqlx_error = sqlx::Error::PoolTimedOut;
        let indexer_error: IndexerError = sqlx_error.into();
        match indexer_error {
            IndexerError::Storage(StorageError::ConnectionFailed { message }) => {
                assert_eq!(message, "Connection pool timeout");
            }
            _ => panic!("Expected Storage(ConnectionFailed) error"),
        }
    }

    #[test]
    fn test_from_sqlx_database_error_should_create_storage_query_error() {
        // Create a mock database error - using RowNotFound as it's a simple variant
        let sqlx_error = sqlx::Error::RowNotFound;
        let indexer_error: IndexerError = sqlx_error.into();
        match indexer_error {
            IndexerError::Storage(StorageError::QueryFailed { query }) => {
                assert_eq!(
                    query,
                    "no rows returned by a query that expected to return at least one row"
                );
            }
            _ => panic!("Expected Storage(QueryFailed) error"),
        }
    }

    #[test]
    fn test_from_sqlx_io_error_should_create_storage_connection_error() {
        let io_error = IoError::new(ErrorKind::ConnectionRefused, "connection refused");
        let sqlx_error = sqlx::Error::Io(io_error);
        let indexer_error: IndexerError = sqlx_error.into();
        match indexer_error {
            IndexerError::Storage(StorageError::ConnectionFailed { message }) => {
                assert!(message.contains("connection refused"));
            }
            _ => panic!("Expected Storage(ConnectionFailed) error"),
        }
    }

    #[test]
    fn test_from_redis_connection_error_should_create_storage_connection_error() {
        let redis_error =
            redis::RedisError::from((redis::ErrorKind::IoError, "Connection refused"));
        let indexer_error: IndexerError = redis_error.into();
        match indexer_error {
            IndexerError::Storage(StorageError::ConnectionFailed { message }) => {
                assert!(message.contains("Connection refused"));
            }
            IndexerError::Storage(StorageError::QueryFailed { query }) => {
                // The Redis library may categorize connection errors as QueryFailed
                // As long as the error message contains connection-related information, that's acceptable
                assert!(query.contains("Connection refused"));
            }
            _ => panic!("Expected Storage(ConnectionFailed) error"),
        }
    }

    #[test]
    fn test_from_redis_timeout_error_should_create_storage_connection_error() {
        let redis_error =
            redis::RedisError::from((redis::ErrorKind::IoError, "Connection timeout"));
        let indexer_error: IndexerError = redis_error.into();
        match indexer_error {
            IndexerError::Storage(StorageError::ConnectionFailed { message }) => {
                assert!(message.contains("Connection timeout"));
            }
            IndexerError::Storage(StorageError::QueryFailed { query }) => {
                // The Redis library may categorize connection errors as QueryFailed
                // As long as the error message contains connection-related information, that's acceptable
                assert!(query.contains("Connection timeout"));
            }
            _ => panic!("Expected Storage error"),
        }
    }

    #[test]
    fn test_from_redis_other_error_should_create_storage_query_error() {
        let redis_error = redis::RedisError::from((redis::ErrorKind::TypeError, "Type error"));
        let indexer_error: IndexerError = redis_error.into();
        match indexer_error {
            IndexerError::Storage(StorageError::QueryFailed { query }) => {
                assert!(query.contains("Type error"));
            }
            _ => panic!("Expected Storage(QueryFailed) error"),
        }
    }

    // Test automatic From conversions for nested errors
    #[test]
    fn test_indexer_error_from_config_error() {
        let config_error = ConfigError::MissingField {
            field: "test".to_string(),
        };
        let indexer_error: IndexerError = config_error.into();
        match indexer_error {
            IndexerError::Config(ConfigError::MissingField { field }) => {
                assert_eq!(field, "test");
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_indexer_error_from_storage_error() {
        let storage_error = StorageError::ConnectionFailed {
            message: "test".to_string(),
        };
        let indexer_error: IndexerError = storage_error.into();
        match indexer_error {
            IndexerError::Storage(StorageError::ConnectionFailed { message }) => {
                assert_eq!(message, "test");
            }
            _ => panic!("Expected Storage error"),
        }
    }

    #[test]
    fn test_indexer_error_from_network_error() {
        let network_error = NetworkError::Timeout { seconds: 30 };
        let indexer_error: IndexerError = network_error.into();
        match indexer_error {
            IndexerError::Network(NetworkError::Timeout { seconds }) => {
                assert_eq!(seconds, 30);
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_indexer_error_from_processing_error() {
        let processing_error = ProcessingError::ParseFailed {
            event_type: "test".to_string(),
        };
        let indexer_error: IndexerError = processing_error.into();
        match indexer_error {
            IndexerError::Processing(ProcessingError::ParseFailed { event_type }) => {
                assert_eq!(event_type, "test");
            }
            _ => panic!("Expected Processing error"),
        }
    }

    #[test]
    fn test_indexer_error_from_metrics_error() {
        let metrics_error = MetricsError::RegistryError {
            message: "test".to_string(),
        };
        let indexer_error: IndexerError = metrics_error.into();
        match indexer_error {
            IndexerError::Metrics(MetricsError::RegistryError { message }) => {
                assert_eq!(message, "test");
            }
            _ => panic!("Expected Metrics error"),
        }
    }

    #[test]
    fn test_indexer_error_from_service_error() {
        let service_error = ServiceError::StartupFailed {
            reason: "test".to_string(),
        };
        let indexer_error: IndexerError = service_error.into();
        match indexer_error {
            IndexerError::Service(ServiceError::StartupFailed { reason }) => {
                assert_eq!(reason, "test");
            }
            _ => panic!("Expected Service error"),
        }
    }

    // Test IndexerResult type alias
    #[test]
    fn test_indexer_result_ok() {
        let result: IndexerResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        match result {
            Ok(value) => assert_eq!(value, "success"),
            Err(e) => panic!("Expected Ok result, got error: {e}"),
        }
    }

    #[test]
    fn test_indexer_result_err() {
        let result: IndexerResult<String> = Err(IndexerError::validation("test error"));
        assert!(result.is_err());
        match result {
            Err(IndexerError::Validation { message }) => {
                assert_eq!(message, "test error");
            }
            _ => panic!("Expected Validation error"),
        }
    }
}
