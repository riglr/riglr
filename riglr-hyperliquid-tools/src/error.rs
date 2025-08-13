use thiserror::Error;

#[derive(Error, Debug)]
pub enum HyperliquidToolError {
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Rate limited: {0}")]
    RateLimit(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),
    
    #[error("Order error: {0}")]
    OrderError(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Result type alias for Hyperliquid tool operations.
pub type Result<T> = std::result::Result<T, HyperliquidToolError>;

impl From<HyperliquidToolError> for riglr_core::error::ToolError {
    fn from(err: HyperliquidToolError) -> Self {
        match err {
            HyperliquidToolError::RateLimit(msg) => {
                // RateLimit errors are already properly categorized
                riglr_core::error::ToolError::rate_limited(msg)
            }
            HyperliquidToolError::NetworkError(msg) => {
                // NetworkError typically indicates temporary issues (already categorized as 5xx errors)
                // All network errors should be retriable
                riglr_core::error::ToolError::retriable(msg)
            }
            HyperliquidToolError::ApiError(msg) => {
                // ApiError typically indicates client errors (4xx) which are not retriable
                // The categorization is already done when creating the error
                riglr_core::error::ToolError::permanent(msg)
            }
            HyperliquidToolError::AuthError(msg) => {
                riglr_core::error::ToolError::permanent(msg)
            }
            HyperliquidToolError::InvalidSymbol(msg) => {
                riglr_core::error::ToolError::permanent(msg)
            }
            HyperliquidToolError::InsufficientBalance(msg) => {
                riglr_core::error::ToolError::permanent(msg)
            }
            HyperliquidToolError::OrderError(msg) => {
                riglr_core::error::ToolError::retriable(msg)
            }
            HyperliquidToolError::Configuration(msg) => {
                riglr_core::error::ToolError::permanent(msg)
            }
        }
    }
}