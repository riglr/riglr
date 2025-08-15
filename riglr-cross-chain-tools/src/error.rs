use thiserror::Error;
use riglr_core::error::ToolError;

#[derive(Error, Debug)]
pub enum CrossChainError {
    /// Core tool error
    #[error("Core tool error: {0}")]
    ToolError(#[from] ToolError),
    
    /// Li.fi API error
    #[error("Li.fi API error: {0}")]
    LifiApiError(String),
    
    /// Quote fetch failed
    #[error("Quote fetch failed: {0}")]
    QuoteFetchError(String),
    
    /// Invalid route configuration
    #[error("Invalid route configuration: {0}")]
    InvalidRoute(String),
    
    /// Bridge operation failed
    #[error("Bridge operation failed: {0}")]
    BridgeExecutionError(String),
    
    /// Unsupported chain pair
    #[error("Unsupported chain pair: {from_chain} -> {to_chain}")]
    UnsupportedChainPair { from_chain: String, to_chain: String },
    
    /// Insufficient liquidity for amount
    #[error("Insufficient liquidity for amount: {amount}")]
    InsufficientLiquidity { amount: String },
}

impl From<CrossChainError> for ToolError {
    fn from(err: CrossChainError) -> Self {
        match err {
            CrossChainError::ToolError(tool_err) => tool_err,
            CrossChainError::LifiApiError(_) => ToolError::retriable_string(err.to_string()),
            CrossChainError::QuoteFetchError(_) => ToolError::retriable_string(err.to_string()),
            CrossChainError::InvalidRoute(_) => ToolError::invalid_input_string(err.to_string()),
            CrossChainError::UnsupportedChainPair { .. } => ToolError::invalid_input_string(err.to_string()),
            CrossChainError::InsufficientLiquidity { .. } => ToolError::permanent_string(err.to_string()),
            CrossChainError::BridgeExecutionError(_) => ToolError::retriable_string(err.to_string()),
        }
    }
}