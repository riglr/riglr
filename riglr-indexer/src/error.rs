//! Error types for the RIGLR indexer service

use std::fmt;
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
    #[error("Missing required configuration: {field}")]
    MissingField { field: String },

    #[error("Invalid configuration value for {field}: {value}")]
    InvalidValue { field: String, value: String },

    #[error("Failed to load configuration from {path}: {source}")]
    LoadFailed {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Configuration validation failed: {message}")]
    ValidationFailed { message: String },
}

/// Storage layer errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("Database query failed: {query}")]
    QueryFailed { query: String },

    #[error("Migration failed: {version}")]
    MigrationFailed { version: String },

    #[error("Data corruption detected: {details}")]
    DataCorruption { details: String },

    #[error("Storage capacity exceeded: {current_size}")]
    CapacityExceeded { current_size: u64 },

    #[error("Transaction failed: {operation}")]
    TransactionFailed { operation: String },

    #[error("Schema validation failed: {table}")]
    SchemaValidationFailed { table: String },
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
    #[error("HTTP request failed: {status} - {url}")]
    HttpFailed { status: u16, url: String },

    #[error("WebSocket connection failed: {reason}")]
    WebSocketFailed { reason: String },

    #[error("Timeout occurred after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("Rate limit exceeded: {limit} requests per {window}s")]
    RateLimited { limit: u32, window: u32 },

    #[error("DNS resolution failed for {host}")]
    DnsResolutionFailed { host: String },

    #[error("TLS/SSL error: {message}")]
    TlsError { message: String },
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
    #[error("Failed to parse event: {event_type}")]
    ParseFailed { event_type: String },

    #[error("Event validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Worker pool exhausted: {active}/{max} workers")]
    WorkerPoolExhausted { active: usize, max: usize },

    #[error("Event queue overflow: {size}/{capacity}")]
    QueueOverflow { size: usize, capacity: usize },

    #[error("Serialization failed for event {event_id}")]
    SerializationFailed { event_id: String },

    #[error("Deserialization failed: {format}")]
    DeserializationFailed { format: String },

    #[error("Pipeline stalled: no events processed in {seconds}s")]
    PipelineStalled { seconds: u64 },
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
    #[error("Metrics registry error: {message}")]
    RegistryError { message: String },

    #[error("Metric recording failed: {metric_name}")]
    RecordingFailed { metric_name: String },

    #[error("Metrics export failed: {endpoint}")]
    ExportFailed { endpoint: String },

    #[error("Invalid metric value: {value}")]
    InvalidValue { value: String },
}

/// Service lifecycle errors
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Service failed to start: {reason}")]
    StartupFailed { reason: String },

    #[error("Service shutdown timeout after {seconds}s")]
    ShutdownTimeout { seconds: u64 },

    #[error("Service unhealthy: {check_name}")]
    HealthCheckFailed { check_name: String },

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    #[error("Dependency unavailable: {service}")]
    DependencyUnavailable { service: String },
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