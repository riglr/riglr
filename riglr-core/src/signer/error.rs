//! Error types for signer operations
//!
//! This module defines all error conditions that can occur during signer operations,
//! including context management, transaction signing, and blockchain interactions.

use thiserror::Error;

/// Error types for signer operations
#[derive(Error, Debug, Clone)]
pub enum SignerError {
    /// No signer context is currently available
    #[error("No signer context available - must be called within SignerContext::with_signer")]
    NoSignerContext,

    /// Blockchain transaction error (generic, preserves error as string)
    #[error("Blockchain transaction error: {0}")]
    BlockchainTransaction(String),

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

// Note: Chain-specific From implementations have been moved to their respective
// tools crates to maintain chain-agnostic nature of riglr-core

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_signer_context_error_display() {
        let error = SignerError::NoSignerContext;
        assert_eq!(
            error.to_string(),
            "No signer context available - must be called within SignerContext::with_signer"
        );
    }

    #[test]
    fn test_blockchain_transaction_error_display() {
        let error = SignerError::BlockchainTransaction("Connection refused".to_string());
        assert!(error.to_string().contains("Blockchain transaction error:"));
        assert!(error.to_string().contains("Connection refused"));
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

    // Chain-specific From trait tests have been moved to their respective tools crates

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
    fn test_blockchain_transaction_error_creation() {
        let error = SignerError::BlockchainTransaction("Request timed out".to_string());
        match error {
            SignerError::BlockchainTransaction(msg) => {
                assert_eq!(msg, "Request timed out");
            }
            _ => panic!("Expected SignerError::BlockchainTransaction variant"),
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
