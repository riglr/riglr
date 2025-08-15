//! Error types for signer operations
//!
//! This module defines all error conditions that can occur during signer operations,
//! including context management, transaction signing, and blockchain interactions.

use std::sync::Arc;
use thiserror::Error;

/// Error types for signer operations
#[derive(Error, Debug, Clone)]
pub enum SignerError {
    /// No signer context is currently available
    #[error("No signer context available - must be called within SignerContext::with_signer")]
    NoSignerContext,

    /// Solana blockchain transaction error
    #[error("Solana transaction error: {0}")]
    SolanaTransaction(Arc<solana_client::client_error::ClientError>),

    /// EVM blockchain transaction error
    #[error("EVM transaction error: {0}")]
    EvmTransaction(String),

    /// Invalid signer configuration
    #[error("Invalid configuration: {0}")]
    Configuration(String),

    /// Transaction signing operation failed
    #[error("Signing error: {0}")]
    Signing(String),

    /// Blockchain client creation failed
    #[error("Client creation error: {0}")]
    ClientCreation(String),

    /// Private key format or content is invalid
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    /// Blockchain provider error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Transaction execution failed on blockchain
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Operation not supported by current signer
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Blockchain hash retrieval or validation error
    #[error("Blockhash error: {0}")]
    BlockhashError(String),

    /// Network not supported by signer
    #[error("Unsupported network: {0}")]
    UnsupportedNetwork(String),

    /// RPC URL format is invalid or unreachable
    #[error("Invalid RPC URL: {0}")]
    InvalidRpcUrl(String),
}

// From implementations for converting Solana client errors to SignerError
impl From<solana_client::client_error::ClientError> for SignerError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        SignerError::SolanaTransaction(Arc::new(err))
    }
}

impl From<Box<solana_client::client_error::ClientError>> for SignerError {
    fn from(err: Box<solana_client::client_error::ClientError>) -> Self {
        SignerError::SolanaTransaction(Arc::from(err))
    }
}

impl From<solana_sdk::signer::SignerError> for SignerError {
    fn from(err: solana_sdk::signer::SignerError) -> Self {
        SignerError::Signing(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_client::client_error::{ClientError, ClientErrorKind};
    use solana_client::rpc_request::RpcRequest;

    #[test]
    fn test_no_signer_context_error_display() {
        let error = SignerError::NoSignerContext;
        assert_eq!(
            error.to_string(),
            "No signer context available - must be called within SignerContext::with_signer"
        );
    }

    #[test]
    fn test_solana_transaction_error_display() {
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Connection refused",
            )),
            RpcRequest::GetAccountInfo,
        );
        let error = SignerError::SolanaTransaction(Arc::new(client_error));
        assert!(error.to_string().contains("Solana transaction error:"));
    }

    #[test]
    fn test_evm_transaction_error_display() {
        let error = SignerError::EvmTransaction("Transaction reverted".to_string());
        assert_eq!(
            error.to_string(),
            "EVM transaction error: Transaction reverted"
        );
    }

    #[test]
    fn test_configuration_error_display() {
        let error = SignerError::Configuration("Invalid network config".to_string());
        assert_eq!(
            error.to_string(),
            "Invalid configuration: Invalid network config"
        );
    }

    #[test]
    fn test_signing_error_display() {
        let error = SignerError::Signing("Key derivation failed".to_string());
        assert_eq!(error.to_string(), "Signing error: Key derivation failed");
    }

    #[test]
    fn test_client_creation_error_display() {
        let error = SignerError::ClientCreation("Failed to initialize RPC client".to_string());
        assert_eq!(
            error.to_string(),
            "Client creation error: Failed to initialize RPC client"
        );
    }

    #[test]
    fn test_invalid_private_key_error_display() {
        let error = SignerError::InvalidPrivateKey("Invalid hex format".to_string());
        assert_eq!(error.to_string(), "Invalid private key: Invalid hex format");
    }

    #[test]
    fn test_provider_error_display() {
        let error = SignerError::ProviderError("Provider timeout".to_string());
        assert_eq!(error.to_string(), "Provider error: Provider timeout");
    }

    #[test]
    fn test_transaction_failed_error_display() {
        let error = SignerError::TransactionFailed("Insufficient funds".to_string());
        assert_eq!(error.to_string(), "Transaction failed: Insufficient funds");
    }

    #[test]
    fn test_unsupported_operation_error_display() {
        let error = SignerError::UnsupportedOperation("Multi-sig not supported".to_string());
        assert_eq!(
            error.to_string(),
            "Unsupported operation: Multi-sig not supported"
        );
    }

    #[test]
    fn test_blockhash_error_display() {
        let error = SignerError::BlockhashError("Failed to fetch recent blockhash".to_string());
        assert_eq!(
            error.to_string(),
            "Blockhash error: Failed to fetch recent blockhash"
        );
    }

    #[test]
    fn test_unsupported_network_error_display() {
        let error = SignerError::UnsupportedNetwork("Polygon".to_string());
        assert_eq!(error.to_string(), "Unsupported network: Polygon");
    }

    #[test]
    fn test_invalid_rpc_url_error_display() {
        let error = SignerError::InvalidRpcUrl("malformed://url".to_string());
        assert_eq!(error.to_string(), "Invalid RPC URL: malformed://url");
    }

    #[test]
    fn test_from_solana_signer_error() {
        let solana_error =
            solana_sdk::signer::SignerError::Custom("Custom signing error".to_string());
        let signer_error: SignerError = solana_error.into();

        match signer_error {
            SignerError::Signing(msg) => {
                assert!(msg.contains("Custom signing error"));
            }
            _ => panic!("Expected SignerError::Signing variant"),
        }
    }

    #[test]
    fn test_from_solana_signer_error_keypair_type() {
        let solana_error = solana_sdk::signer::SignerError::Custom("keypair not found".to_string());
        let signer_error: SignerError = solana_error.into();

        match signer_error {
            SignerError::Signing(msg) => {
                assert!(msg.contains("keypair not found"));
            }
            _ => panic!("Expected SignerError::Signing variant"),
        }
    }

    #[test]
    fn test_error_variants_are_debug() {
        let error = SignerError::NoSignerContext;
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("NoSignerContext"));
    }

    #[test]
    fn test_all_error_variants_with_empty_strings() {
        let errors = vec![
            SignerError::EvmTransaction(String::new()),
            SignerError::Configuration(String::new()),
            SignerError::Signing(String::new()),
            SignerError::ClientCreation(String::new()),
            SignerError::InvalidPrivateKey(String::new()),
            SignerError::ProviderError(String::new()),
            SignerError::TransactionFailed(String::new()),
            SignerError::UnsupportedOperation(String::new()),
            SignerError::BlockhashError(String::new()),
            SignerError::UnsupportedNetwork(String::new()),
            SignerError::InvalidRpcUrl(String::new()),
        ];

        for error in errors {
            // Test that all variants can be created with empty strings
            let _error_string = error.to_string();
            let _debug_string = format!("{:?}", error);
        }
    }

    #[test]
    fn test_solana_transaction_error_from_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "Request timed out");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(io_error),
            RpcRequest::GetAccountInfo,
        );
        let signer_error: SignerError = client_error.into();

        match signer_error {
            SignerError::SolanaTransaction(_) => {
                // Success - the From trait worked correctly
            }
            _ => panic!("Expected SignerError::SolanaTransaction variant"),
        }
    }

    #[test]
    fn test_error_variants_with_special_characters() {
        let special_chars = "特殊文字 🚀 \n\t\"'\\";
        let errors = vec![
            SignerError::EvmTransaction(special_chars.to_string()),
            SignerError::Configuration(special_chars.to_string()),
            SignerError::Signing(special_chars.to_string()),
            SignerError::ClientCreation(special_chars.to_string()),
            SignerError::InvalidPrivateKey(special_chars.to_string()),
            SignerError::ProviderError(special_chars.to_string()),
            SignerError::TransactionFailed(special_chars.to_string()),
            SignerError::UnsupportedOperation(special_chars.to_string()),
            SignerError::BlockhashError(special_chars.to_string()),
            SignerError::UnsupportedNetwork(special_chars.to_string()),
            SignerError::InvalidRpcUrl(special_chars.to_string()),
        ];

        for error in errors {
            let error_string = error.to_string();
            assert!(error_string.contains(special_chars));
        }
    }
}
