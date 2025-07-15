//! Error types for riglr-web-tools.

use riglr_core::ToolError;
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

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network error (retriable)
    #[error("Network error: {0}")]
    Network(String),

    /// API error (non-retriable)
    #[error("API error: {0}")]
    Api(String),

    /// Client creation error
    #[error("Client error: {0}")]
    Client(String),

    /// Parsing error
    #[error("Parsing error: {0}")]
    Parsing(String),

    /// Request error
    #[error("Request error: {0}")]
    Request(String),

    /// Generic error
    #[error("Error: {0}")]
    Generic(String),

    /// Parse error (alias for Parsing)
    #[error("Parse error: {0}")]
    Parse(String),
    
    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    JsonParseError(String),
}

impl From<WebToolError> for ToolError {
    fn from(err: WebToolError) -> Self {
        match err {
            WebToolError::Http(e) => {
                if e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                    ToolError::rate_limited(format!("HTTP 429: {}", e))
                } else if e.is_timeout() || e.is_connect() {
                    ToolError::retriable(format!("HTTP error: {}", e))
                } else {
                    ToolError::permanent(format!("HTTP error: {}", e))
                }
            }
            WebToolError::Network(msg) => ToolError::retriable(msg),
            WebToolError::RateLimit(msg) => ToolError::rate_limited(msg),
            WebToolError::Auth(msg) => ToolError::permanent(msg),
            WebToolError::InvalidResponse(msg) => ToolError::permanent(msg),
            WebToolError::Config(msg) => ToolError::permanent(msg),
            WebToolError::Api(msg) => ToolError::permanent(msg),
            WebToolError::Parsing(msg) => ToolError::permanent(msg),
            WebToolError::Request(msg) => ToolError::retriable(msg),
            WebToolError::Parse(msg) => ToolError::permanent(msg),
            WebToolError::JsonParseError(msg) => ToolError::permanent(msg),
            WebToolError::Url(e) => ToolError::permanent(format!("URL error: {}", e)),
            WebToolError::Serialization(e) => {
                ToolError::permanent(format!("Serialization error: {}", e))
            }
            WebToolError::Core(e) => ToolError::permanent(format!("Core error: {}", e)),
            WebToolError::Client(msg) => ToolError::permanent(msg),
            WebToolError::Generic(msg) => ToolError::permanent(msg),
        }
    }
}

/// Result type alias for web tool operations.
pub type Result<T> = std::result::Result<T, WebToolError>;
