use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrossChainToolError {
    #[error("Bridge error: {0}")]
    BridgeError(String),
    
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),
    
    #[error("Route not found: {0}")]
    RouteNotFound(String),
    
    #[error("Insufficient liquidity: {0}")]
    InsufficientLiquidity(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Rate limited: {0}")]
    RateLimit(String),
    
    #[error("Transaction error: {0}")]
    TransactionError(String),
}

impl From<CrossChainToolError> for riglr_core::error::ToolError {
    fn from(err: CrossChainToolError) -> Self {
        match err {
            CrossChainToolError::RateLimit(msg) => {
                riglr_core::error::ToolError::rate_limited(msg)
            }
            CrossChainToolError::NetworkError(msg) => {
                if msg.contains("timeout") || msg.contains("connection") {
                    riglr_core::error::ToolError::retriable(msg)
                } else {
                    riglr_core::error::ToolError::permanent(msg)
                }
            }
            CrossChainToolError::BridgeError(msg) => {
                if msg.contains("503") || msg.contains("maintenance") {
                    riglr_core::error::ToolError::retriable(msg)
                } else {
                    riglr_core::error::ToolError::permanent(msg)
                }
            }
            CrossChainToolError::UnsupportedChain(msg) => {
                riglr_core::error::ToolError::permanent(msg)
            }
            CrossChainToolError::RouteNotFound(msg) => {
                riglr_core::error::ToolError::retriable(msg)
            }
            CrossChainToolError::InsufficientLiquidity(msg) => {
                riglr_core::error::ToolError::retriable(msg)
            }
            CrossChainToolError::TransactionError(msg) => {
                riglr_core::error::ToolError::retriable(msg)
            }
        }
    }
}