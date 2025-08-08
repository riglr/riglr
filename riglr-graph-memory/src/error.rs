//! Error types for riglr-graph-memory.

use thiserror::Error;

/// Main error type for graph memory operations.
#[derive(Error, Debug)]
pub enum GraphMemoryError {
    /// Database connection error
    #[error("Database error: {0}")]
    Database(String),

    /// Query execution failed
    #[error("Query error: {0}")]
    Query(String),

    /// Entity extraction failed
    #[error("Entity extraction error: {0}")]
    EntityExtraction(String),

    /// Vector embedding failed
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Core riglr error
    #[error("Core error: {0}")]
    Core(#[from] riglr_core::CoreError),

    /// Generic error
    #[error("Graph memory error: {0}")]
    Generic(String),
}

/// Result type alias for graph memory operations.
pub type Result<T> = std::result::Result<T, GraphMemoryError>;
