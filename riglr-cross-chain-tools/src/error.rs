use std::fmt;

#[derive(Debug, Clone)]
pub enum CrossChainToolError {
    Network(String),
    UnsupportedChain(String),
    BridgeError(String),
    InsufficientBalance(String),
    InvalidAddress(String),
    TransactionFailed(String),
    ConfigurationError(String),
}

impl fmt::Display for CrossChainToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrossChainToolError::Network(msg) => write!(f, "Network error: {}", msg),
            CrossChainToolError::UnsupportedChain(msg) => write!(f, "Unsupported chain: {}", msg),
            CrossChainToolError::BridgeError(msg) => write!(f, "Bridge error: {}", msg),
            CrossChainToolError::InsufficientBalance(msg) => write!(f, "Insufficient balance: {}", msg),
            CrossChainToolError::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            CrossChainToolError::TransactionFailed(msg) => write!(f, "Transaction failed: {}", msg),
            CrossChainToolError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for CrossChainToolError {}

impl From<CrossChainToolError> for riglr_core::ToolError {
    fn from(error: CrossChainToolError) -> Self {
        match error {
            CrossChainToolError::Network(_) | CrossChainToolError::InsufficientBalance(_) => {
                riglr_core::ToolError::Retriable(error.to_string())
            }
            CrossChainToolError::UnsupportedChain(_) | CrossChainToolError::BridgeError(_)
            | CrossChainToolError::InvalidAddress(_) | CrossChainToolError::TransactionFailed(_)
            | CrossChainToolError::ConfigurationError(_) => {
                riglr_core::ToolError::Permanent(error.to_string())
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, CrossChainToolError>;