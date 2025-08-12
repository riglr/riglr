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
                riglr_core::error::ToolError::rate_limited(msg)
            }
            HyperliquidToolError::NetworkError(msg) => {
                if msg.contains("timeout") || msg.contains("connection") {
                    riglr_core::error::ToolError::retriable(msg)
                } else {
                    riglr_core::error::ToolError::permanent(msg)
                }
            }
            HyperliquidToolError::ApiError(msg) => {
                if msg.contains("429") || msg.contains("rate limit") {
                    riglr_core::error::ToolError::rate_limited(msg)
                } else if msg.contains("503") || msg.contains("502") {
                    riglr_core::error::ToolError::retriable(msg)
                } else {
                    riglr_core::error::ToolError::permanent(msg)
                }
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