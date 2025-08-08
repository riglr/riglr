//! Error types for riglr-web-tools.

use thiserror::Error;

/// Main error type for web tool operations.
#[derive(Error, Debug)]
pub enum WebToolError {
    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// API authentication failed
    #[error("Authentication error: {0}")]
    Auth(String),

    /// API rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Invalid API response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// URL parsing error
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Core riglr error
    #[error("Core error: {0}")]
    Core(#[from] riglr_core::CoreError),

    /// Generic error
    #[error("Web tool error: {0}")]
    Generic(String),
}

/// Result type alias for web tool operations.
pub type Result<T> = std::result::Result<T, WebToolError>;
