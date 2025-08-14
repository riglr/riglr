//! Error types for EVM operations shared across riglr crates
//!
//! This module provides standardized error types that can be used by both
//! riglr-evm-tools and riglr-cross-chain-tools for consistent error handling.

use thiserror::Error;

/// Error types for EVM operations shared across crates
#[derive(Debug, Error)]
pub enum EvmCommonError {
    /// Invalid EVM address format
    #[error("Invalid EVM address: {0}")]
    InvalidAddress(String),
    
    /// Unsupported or unconfigured chain
    #[error("Unsupported chain ID: {0}. Configure RPC_URL_{0} environment variable")]
    UnsupportedChain(u64),
    
    /// Invalid chain name  
    #[error("Invalid chain name: {0}")]
    InvalidChainName(String),
    
    /// RPC provider error
    #[error("RPC provider error: {0}")]
    ProviderError(String),
    
    /// Configuration validation error
    #[error("Configuration error: {0}")]
    InvalidConfig(String),
    
    /// Invalid transaction data
    #[error("Invalid transaction data: {0}")]
    InvalidData(String),
    
    /// Network connection error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Parsing error
    #[error("Parse error: {0}")]
    ParseError(String),
}

impl EvmCommonError {
    /// Check if this error is retriable (network/temporary issues)
    pub fn is_retriable(&self) -> bool {
        matches!(self, 
            EvmCommonError::ProviderError(_) |
            EvmCommonError::NetworkError(_)
        )
    }
    
    /// Check if this error is permanent (configuration/validation issues)
    pub fn is_permanent(&self) -> bool {
        !self.is_retriable()
    }
}

/// Result type alias for EVM operations
pub type EvmResult<T> = Result<T, EvmCommonError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_classification() {
        let network_err = EvmCommonError::NetworkError("timeout".to_string());
        assert!(network_err.is_retriable());
        assert!(!network_err.is_permanent());
        
        let config_err = EvmCommonError::InvalidConfig("bad config".to_string());
        assert!(!config_err.is_retriable());
        assert!(config_err.is_permanent());
        
        let address_err = EvmCommonError::InvalidAddress("bad address".to_string());
        assert!(address_err.is_permanent());
    }
    
    #[test]
    fn test_error_display() {
        let err = EvmCommonError::UnsupportedChain(999);
        let msg = err.to_string();
        assert!(msg.contains("999"));
        assert!(msg.contains("RPC_URL_999"));
    }
}