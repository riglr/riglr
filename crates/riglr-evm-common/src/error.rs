//! Error types for EVM operations shared across riglr crates
//!
//! This module provides standardized error types that can be used by both
//! riglr-evm-tools and riglr-cross-chain-tools for consistent error handling.

use thiserror::Error;

/// Error types for EVM operations shared across crates
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Invalid EVM address format
    #[error("Invalid EVM address: {0}")]
    InvalidAddress(String),

    /// Invalid chain name
    #[error("Invalid chain name: {0}")]
    InvalidChainName(String),

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

    /// RPC provider error
    #[error("RPC provider error: {0}")]
    ProviderError(String),

    /// Unsupported or unconfigured chain
    #[error("Unsupported chain ID: {0}. Configure RPC_URL_{0} environment variable")]
    UnsupportedChain(u64),
}

impl Error {
    /// Check if this error is permanent (configuration/validation issues)
    #[must_use]
    #[inline]
    pub const fn is_permanent(&self) -> bool {
        !self.is_retriable()
    }

    /// Check if this error is retriable (network/temporary issues)
    #[must_use]
    #[inline]
    pub const fn is_retriable(&self) -> bool {
        matches!(*self, Self::ProviderError(_) | Self::NetworkError(_))
    }
}

/// Result type alias for EVM operations
pub type EvmResult<T> = Result<T, Error>;

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn error_classification() {
        let network_err = Error::NetworkError("timeout".to_owned());
        assert!(network_err.is_retriable());
        assert!(!network_err.is_permanent());

        let config_err = Error::InvalidConfig("bad config".to_owned());
        assert!(!config_err.is_retriable());
        assert!(config_err.is_permanent());

        let address_err = Error::InvalidAddress("bad address".to_owned());
        assert!(address_err.is_permanent());
    }

    #[test]
    fn error_display() {
        let err = Error::UnsupportedChain(999);
        let msg = err.to_string();
        assert!(msg.contains("999"));
        assert!(msg.contains("RPC_URL_999"));
    }

    #[test]
    fn invalid_address_error_display() {
        let err = Error::InvalidAddress("0xinvalid".to_owned());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid EVM address: 0xinvalid");
    }

    #[test]
    fn invalid_address_error_empty_string() {
        let err = Error::InvalidAddress(String::new());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid EVM address: ");
    }

    #[test]
    fn unsupported_chain_error_display() {
        let err = Error::UnsupportedChain(1);
        let msg = err.to_string();
        assert_eq!(
            msg,
            "Unsupported chain ID: 1. Configure RPC_URL_1 environment variable"
        );
    }

    #[test]
    fn unsupported_chain_error_large_chain_id() {
        let err = Error::UnsupportedChain(u64::MAX);
        let msg = err.to_string();
        assert!(msg.contains(&u64::MAX.to_string()));
        assert!(msg.contains(&format!("RPC_URL_{}", u64::MAX)));
    }

    #[test]
    fn invalid_chain_name_error_display() {
        let err = Error::InvalidChainName("unknown_chain".to_owned());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid chain name: unknown_chain");
    }

    #[test]
    fn invalid_chain_name_error_empty_string() {
        let err = Error::InvalidChainName(String::new());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid chain name: ");
    }

    #[test]
    fn provider_error_display() {
        let err = Error::ProviderError("connection failed".to_owned());
        let msg = err.to_string();
        assert_eq!(msg, "RPC provider error: connection failed");
    }

    #[test]
    fn provider_error_empty_string() {
        let err = Error::ProviderError(String::new());
        let msg = err.to_string();
        assert_eq!(msg, "RPC provider error: ");
    }

    #[test]
    fn invalid_config_error_display() {
        let err = Error::InvalidConfig("missing key".to_owned());
        let msg = err.to_string();
        assert_eq!(msg, "Configuration error: missing key");
    }

    #[test]
    fn invalid_config_error_empty_string() {
        let err = Error::InvalidConfig(String::new());
        let msg = err.to_string();
        assert_eq!(msg, "Configuration error: ");
    }

    #[test]
    fn invalid_data_error_display() {
        let err = Error::InvalidData("malformed hex".to_owned());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid transaction data: malformed hex");
    }

    #[test]
    fn invalid_data_error_empty_string() {
        let err = Error::InvalidData(String::new());
        let msg = err.to_string();
        assert_eq!(msg, "Invalid transaction data: ");
    }

    #[test]
    fn network_error_display() {
        let err = Error::NetworkError("timeout".to_owned());
        let msg = err.to_string();
        assert_eq!(msg, "Network error: timeout");
    }

    #[test]
    fn network_error_empty_string() {
        let err = Error::NetworkError(String::new());
        let msg = err.to_string();
        assert_eq!(msg, "Network error: ");
    }

    #[test]
    fn parse_error_display() {
        let err = Error::ParseError("invalid format".to_owned());
        let msg = err.to_string();
        assert_eq!(msg, "Parse error: invalid format");
    }

    #[test]
    fn parse_error_empty_string() {
        let err = Error::ParseError(String::new());
        let msg = err.to_string();
        assert_eq!(msg, "Parse error: ");
    }

    #[test]
    fn provider_error_is_retriable() {
        let err = Error::ProviderError("connection timeout".to_owned());
        assert!(err.is_retriable());
        assert!(!err.is_permanent());
    }

    #[test]
    fn network_error_is_retriable() {
        let err = Error::NetworkError("dns resolution failed".to_owned());
        assert!(err.is_retriable());
        assert!(!err.is_permanent());
    }

    #[test]
    fn all_permanent_errors_are_not_retriable() {
        let errors = vec![
            Error::InvalidAddress("0xinvalid".to_owned()),
            Error::UnsupportedChain(999),
            Error::InvalidChainName("unknown".to_owned()),
            Error::InvalidConfig("bad config".to_owned()),
            Error::InvalidData("bad data".to_owned()),
            Error::ParseError("parse failed".to_owned()),
        ];

        for err in errors {
            assert!(!err.is_retriable());
            assert!(err.is_permanent());
        }
    }

    #[test]
    fn evm_result_type_alias_ok() {
        let result: EvmResult<String> = Ok("success".to_owned());
        assert!(result.is_ok());
        assert_eq!(result.as_ref().expect("Expected Ok result"), "success");
    }

    #[test]
    fn evm_result_type_alias_err() {
        let result: EvmResult<String> = Err(Error::InvalidAddress("0xinvalid".to_owned()));
        assert!(result.is_err());
        match *result.as_ref().expect_err("Expected Err result") {
            Error::InvalidAddress(ref addr) => assert_eq!(addr.as_str(), "0xinvalid"),
            Error::UnsupportedChain(_)
            | Error::InvalidChainName(_)
            | Error::ProviderError(_)
            | Error::InvalidConfig(_)
            | Error::InvalidData(_)
            | Error::NetworkError(_)
            | Error::ParseError(_) => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn error_debug_trait() {
        let err = Error::InvalidAddress("0xinvalid".to_owned());
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("InvalidAddress"));
        assert!(debug_str.contains("0xinvalid"));
    }
}
