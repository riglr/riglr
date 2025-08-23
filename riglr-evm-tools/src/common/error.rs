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
        matches!(
            self,
            EvmCommonError::ProviderError(_) | EvmCommonError::NetworkError(_)
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

    #[test]
    fn test_invalid_address_error_display() {
        let err = EvmCommonError::InvalidAddress("0xinvalid".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid EVM address: 0xinvalid");
    }

    #[test]
    fn test_invalid_address_error_empty_string() {
        let err = EvmCommonError::InvalidAddress("".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid EVM address: ");
    }

    #[test]
    fn test_unsupported_chain_error_display() {
        let err = EvmCommonError::UnsupportedChain(1);
        let msg = err.to_string();
        assert_eq!(
            msg,
            "Unsupported chain ID: 1. Configure RPC_URL_1 environment variable"
        );
    }

    #[test]
    fn test_unsupported_chain_error_large_chain_id() {
        let err = EvmCommonError::UnsupportedChain(u64::MAX);
        let msg = err.to_string();
        assert!(msg.contains(&u64::MAX.to_string()));
        assert!(msg.contains(&format!("RPC_URL_{}", u64::MAX)));
    }

    #[test]
    fn test_invalid_chain_name_error_display() {
        let err = EvmCommonError::InvalidChainName("unknown_chain".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid chain name: unknown_chain");
    }

    #[test]
    fn test_invalid_chain_name_error_empty_string() {
        let err = EvmCommonError::InvalidChainName("".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid chain name: ");
    }

    #[test]
    fn test_provider_error_display() {
        let err = EvmCommonError::ProviderError("connection failed".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "RPC provider error: connection failed");
    }

    #[test]
    fn test_provider_error_empty_string() {
        let err = EvmCommonError::ProviderError("".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "RPC provider error: ");
    }

    #[test]
    fn test_invalid_config_error_display() {
        let err = EvmCommonError::InvalidConfig("missing key".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Configuration error: missing key");
    }

    #[test]
    fn test_invalid_config_error_empty_string() {
        let err = EvmCommonError::InvalidConfig("".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Configuration error: ");
    }

    #[test]
    fn test_invalid_data_error_display() {
        let err = EvmCommonError::InvalidData("malformed hex".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid transaction data: malformed hex");
    }

    #[test]
    fn test_invalid_data_error_empty_string() {
        let err = EvmCommonError::InvalidData("".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid transaction data: ");
    }

    #[test]
    fn test_network_error_display() {
        let err = EvmCommonError::NetworkError("timeout".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Network error: timeout");
    }

    #[test]
    fn test_network_error_empty_string() {
        let err = EvmCommonError::NetworkError("".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Network error: ");
    }

    #[test]
    fn test_parse_error_display() {
        let err = EvmCommonError::ParseError("invalid format".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Parse error: invalid format");
    }

    #[test]
    fn test_parse_error_empty_string() {
        let err = EvmCommonError::ParseError("".to_string());
        let msg = err.to_string();
        assert_eq!(msg, "Parse error: ");
    }

    #[test]
    fn test_provider_error_is_retriable() {
        let err = EvmCommonError::ProviderError("connection timeout".to_string());
        assert!(err.is_retriable());
        assert!(!err.is_permanent());
    }

    #[test]
    fn test_network_error_is_retriable() {
        let err = EvmCommonError::NetworkError("dns resolution failed".to_string());
        assert!(err.is_retriable());
        assert!(!err.is_permanent());
    }

    #[test]
    fn test_all_permanent_errors_are_not_retriable() {
        let errors = vec![
            EvmCommonError::InvalidAddress("0xinvalid".to_string()),
            EvmCommonError::UnsupportedChain(999),
            EvmCommonError::InvalidChainName("unknown".to_string()),
            EvmCommonError::InvalidConfig("bad config".to_string()),
            EvmCommonError::InvalidData("bad data".to_string()),
            EvmCommonError::ParseError("parse failed".to_string()),
        ];

        for err in errors {
            assert!(!err.is_retriable());
            assert!(err.is_permanent());
        }
    }

    #[test]
    fn test_evm_result_type_alias_ok() {
        let result: EvmResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_evm_result_type_alias_err() {
        let result: EvmResult<String> =
            Err(EvmCommonError::InvalidAddress("0xinvalid".to_string()));
        assert!(result.is_err());
        match result.unwrap_err() {
            EvmCommonError::InvalidAddress(addr) => assert_eq!(addr, "0xinvalid"),
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_error_debug_trait() {
        let err = EvmCommonError::InvalidAddress("0xinvalid".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidAddress"));
        assert!(debug_str.contains("0xinvalid"));
    }
}
