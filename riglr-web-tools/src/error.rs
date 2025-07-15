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
                if e.is_timeout() || e.is_connect() {
                    ToolError::Retriable(format!("HTTP error: {}", e))
                } else {
                    ToolError::Permanent(format!("HTTP error: {}", e))
                }
            }
            WebToolError::Network(msg) => ToolError::Retriable(msg),
            WebToolError::RateLimit(msg) => ToolError::Retriable(msg),
            WebToolError::Auth(msg) => ToolError::Permanent(msg),
            WebToolError::InvalidResponse(msg) => ToolError::Permanent(msg),
            WebToolError::Config(msg) => ToolError::Permanent(msg),
            WebToolError::Api(msg) => ToolError::Permanent(msg),
            WebToolError::Parsing(msg) => ToolError::Permanent(msg),
            WebToolError::Request(msg) => ToolError::Retriable(msg),
            WebToolError::Parse(msg) => ToolError::Permanent(msg),
            WebToolError::JsonParseError(msg) => ToolError::Permanent(msg),
            WebToolError::Url(e) => ToolError::Permanent(format!("URL error: {}", e)),
            WebToolError::Serialization(e) => {
                ToolError::Permanent(format!("Serialization error: {}", e))
            }
            WebToolError::Core(e) => ToolError::Permanent(format!("Core error: {}", e)),
            WebToolError::Client(msg) => ToolError::Permanent(msg),
            WebToolError::Generic(msg) => ToolError::Permanent(msg),
        }
    }
}

/// Result type alias for web tool operations.
pub type Result<T> = std::result::Result<T, WebToolError>;
