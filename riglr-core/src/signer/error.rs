//! Error types for signer operations
//!
//! This module defines all error conditions that can occur during signer operations,
//! including context management, transaction signing, and blockchain interactions.

use thiserror::Error;

/// Error types for signer operations
#[derive(Error, Debug)]
pub enum SignerError {
    /// No signer context is currently available
    #[error("No signer context available - must be called within SignerContext::with_signer")]
    NoSignerContext,

    /// Solana blockchain transaction error
    #[error("Solana transaction error: {0}")]
    SolanaTransaction(#[from] Box<solana_client::client_error::ClientError>),

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

impl From<solana_sdk::signer::SignerError> for SignerError {
    fn from(err: solana_sdk::signer::SignerError) -> Self {
        SignerError::Signing(err.to_string())
    }
}
