//! Error types for the RIGLR indexer service

use thiserror::Error;

/// Result type used throughout the indexer
pub type IndexerResult<T> = Result<T, IndexerError>;

/// Main error type for the indexer service
#[derive(Error, Debug)]
pub enum IndexerError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Storage layer errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Network/API errors
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    /// Event processing errors
    #[error("Processing error: {0}")]
    Processing(#[from] ProcessingError),

    /// Metrics collection errors
    #[error("Metrics error: {0}")]
    Metrics(#[from] MetricsError),

    /// Service lifecycle errors
    #[error("Service error: {0}")]
    Service(#[from] ServiceError),

    /// External dependency errors
    #[error("Dependency error: {source}")]
    Dependency {
        /// The underlying error from a dependency
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation {
        /// Human-readable validation error message
        message: String,
    },

    /// Internal errors that should not occur in normal operation
    #[error("Internal error: {message}")]
    Internal {
        /// Description of the internal error
        message: String,
        /// Optional source error
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl IndexerError {
    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

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
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Check if the error is retriable
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::Network(e) => e.is_retriable(),
            Self::Storage(e) => e.is_retriable(),
            Self::Processing(e) => e.is_retriable(),
            Self::Service(e) => e.is_retriable(),
            Self::Dependency { .. } => false, // Conservative default
            _ => false,
        }
    }

    /// Get error category for metrics and logging
    pub fn category(&self) -> &'static str {
        match self {
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
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Missing required configuration field
    #[error("Missing required configuration: {field}")]
    MissingField {
        /// The missing field name
        field: String,
    },

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
        source: Box<dyn std::error::Error + Send + Sync>,
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
pub enum StorageError {
    /// Database connection failed
    #[error("Database connection failed: {message}")]
    ConnectionFailed {
        /// Connection failure message
        message: String,
    },

    /// Database query failed
    #[error("Database query failed: {query}")]
    QueryFailed {
        /// The failed query
        query: String,
    },

    /// Migration failed
    #[error("Migration failed: {version}")]
    MigrationFailed {
        /// Migration version that failed
        version: String,
    },

    /// Data corruption detected
    #[error("Data corruption detected: {details}")]
    DataCorruption {
        /// Corruption details
        details: String,
    },

    /// Storage capacity exceeded
    #[error("Storage capacity exceeded: {current_size}")]
    CapacityExceeded {
        /// Current storage size in bytes
        current_size: u64,
    },

    /// Transaction failed
    #[error("Transaction failed: {operation}")]
    TransactionFailed {
        /// Failed operation name
        operation: String,
    },

    /// Schema validation failed
    #[error("Schema validation failed: {table}")]
    SchemaValidationFailed {
        /// Table with schema issues
        table: String,
    },
}

impl StorageError {
    /// Check if the storage error is retriable
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::ConnectionFailed { .. } => true,
            Self::QueryFailed { .. } => true, // May be transient
            Self::TransactionFailed { .. } => true,
            Self::DataCorruption { .. } => false,
            Self::CapacityExceeded { .. } => false,
            Self::MigrationFailed { .. } => false,
            Self::SchemaValidationFailed { .. } => false,
        }
    }
}

/// Network and API errors
#[derive(Error, Debug)]
pub enum NetworkError {
    /// HTTP request failed
    #[error("HTTP request failed: {status} - {url}")]
    HttpFailed {
        /// HTTP status code
        status: u16,
        /// Request URL
        url: String,
    },

    /// WebSocket connection failed
    #[error("WebSocket connection failed: {reason}")]
    WebSocketFailed {
        /// Failure reason
        reason: String,
    },

    /// Request timeout
    #[error("Timeout occurred after {seconds}s")]
    Timeout {
        /// Timeout duration in seconds
        seconds: u64,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {limit} requests per {window}s")]
    RateLimited {
        /// Rate limit threshold
        limit: u32,
        /// Time window in seconds
        window: u32,
    },

    /// DNS resolution failed
    #[error("DNS resolution failed for {host}")]
    DnsResolutionFailed {
        /// Host that failed to resolve
        host: String,
    },

    /// TLS/SSL error
    #[error("TLS/SSL error: {message}")]
    TlsError {
        /// Error message
        message: String,
    },
}

impl NetworkError {
    /// Check if the network error is retriable
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::HttpFailed { status, .. } => {
                // 5xx errors are usually retriable, 4xx usually not
                *status >= 500 || *status == 429 || *status == 408
            }
            Self::WebSocketFailed { .. } => true,
            Self::Timeout { .. } => true,
            Self::RateLimited { .. } => true,
            Self::DnsResolutionFailed { .. } => true,
            Self::TlsError { .. } => false, // Usually configuration issue
        }
    }
}

/// Event processing errors
#[derive(Error, Debug)]
pub enum ProcessingError {
    /// Failed to parse event
    #[error("Failed to parse event: {event_type}")]
    ParseFailed {
        /// Type of event that failed to parse
        event_type: String,
    },

    /// Event validation failed
    #[error("Event validation failed: {reason}")]
    ValidationFailed {
        /// Validation failure reason
        reason: String,
    },

    /// Worker pool exhausted
    #[error("Worker pool exhausted: {active}/{max} workers")]
    WorkerPoolExhausted {
        /// Number of active workers
        active: usize,
        /// Maximum number of workers
        max: usize,
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

    /// Deserialization failed
    #[error("Deserialization failed: {format}")]
    DeserializationFailed {
        /// Format that failed to deserialize
        format: String,
    },

    /// Pipeline stalled
    #[error("Pipeline stalled: no events processed in {seconds}s")]
    PipelineStalled {
        /// Seconds since last event processed
        seconds: u64,
    },
}

impl ProcessingError {
    /// Check if the processing error is retriable
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::ParseFailed { .. } => false, // Data issue
            Self::ValidationFailed { .. } => false, // Data issue
            Self::WorkerPoolExhausted { .. } => true, // May recover
            Self::QueueOverflow { .. } => true, // May recover
            Self::SerializationFailed { .. } => false, // Data issue
            Self::DeserializationFailed { .. } => false, // Data issue
            Self::PipelineStalled { .. } => true, // May recover
        }
    }
}

/// Metrics collection errors
#[derive(Error, Debug)]
pub enum MetricsError {
    /// Metrics registry error
    #[error("Metrics registry error: {message}")]
    RegistryError {
        /// Error message
        message: String,
    },

    /// Metric recording failed
    #[error("Metric recording failed: {metric_name}")]
    RecordingFailed {
        /// Name of metric that failed to record
        metric_name: String,
    },

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
}

/// Service lifecycle errors
#[derive(Error, Debug)]
pub enum ServiceError {
    /// Service failed to start
    #[error("Service failed to start: {reason}")]
    StartupFailed {
        /// Startup failure reason
        reason: String,
    },

    /// Service shutdown timeout
    #[error("Service shutdown timeout after {seconds}s")]
    ShutdownTimeout {
        /// Timeout duration in seconds
        seconds: u64,
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

    /// Dependency unavailable
    #[error("Dependency unavailable: {service}")]
    DependencyUnavailable {
        /// Name of unavailable service
        service: String,
    },
}

impl ServiceError {
    /// Check if the service error is retriable
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::StartupFailed { .. } => false, // Usually configuration
            Self::ShutdownTimeout { .. } => false, // Lifecycle issue
            Self::HealthCheckFailed { .. } => true, // May recover
            Self::ResourceExhausted { .. } => true, // May recover
            Self::DependencyUnavailable { .. } => true, // May recover
        }
    }
}

/// Convert from anyhow::Error
impl From<anyhow::Error> for IndexerError {
    fn from(error: anyhow::Error) -> Self {
        Self::Dependency {
            source: error.into(),
        }
    }
}

/// Convert from riglr_events_core::EventError
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
            riglr_events_core::EventError::Generic { message } => {
                Self::internal(message)
            }
            _ => {
                Self::internal(error.to_string())
            }
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

