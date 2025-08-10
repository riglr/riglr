use std::fmt;

#[derive(Debug, Clone)]
pub enum HyperliquidToolError {
    Network(String),
    Auth(String), 
    RateLimit(String),
    InvalidSymbol(String),
    InsufficientBalance(String),
    OrderFailed(String),
    ApiError(String),
}

impl fmt::Display for HyperliquidToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HyperliquidToolError::Network(msg) => write!(f, "Network error: {}", msg),
            HyperliquidToolError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            HyperliquidToolError::RateLimit(msg) => write!(f, "Rate limit exceeded: {}", msg),
            HyperliquidToolError::InvalidSymbol(msg) => write!(f, "Invalid symbol: {}", msg),
            HyperliquidToolError::InsufficientBalance(msg) => write!(f, "Insufficient balance: {}", msg),
            HyperliquidToolError::OrderFailed(msg) => write!(f, "Order failed: {}", msg),
            HyperliquidToolError::ApiError(msg) => write!(f, "API error: {}", msg),
        }
    }
}

impl std::error::Error for HyperliquidToolError {}

impl From<HyperliquidToolError> for riglr_core::ToolError {
    fn from(error: HyperliquidToolError) -> Self {
        match error {
            HyperliquidToolError::Network(_) | HyperliquidToolError::RateLimit(_) 
            | HyperliquidToolError::InsufficientBalance(_) => {
                riglr_core::ToolError::Retriable(error.to_string())
            }
            HyperliquidToolError::Auth(_) | HyperliquidToolError::InvalidSymbol(_)
            | HyperliquidToolError::OrderFailed(_) | HyperliquidToolError::ApiError(_) => {
                riglr_core::ToolError::Permanent(error.to_string())
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, HyperliquidToolError>;