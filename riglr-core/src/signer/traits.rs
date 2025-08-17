//! Core signer traits for multi-chain transaction operations
//!
//! This module defines the primary traits used by the signer system to provide
//! unified interfaces for transaction signing across different blockchain networks.
//! It includes both legacy traits for backward compatibility and client traits
//! for blockchain network interactions.

use super::error::SignerError;
use alloy::primitives::{Bytes, TxHash, U256};
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction};
use std::sync::Arc;

/// A trait for transaction signing across multiple blockchain networks.
/// This trait provides a unified interface for signing transactions on different chains
/// while maintaining secure context isolation.
#[async_trait]
pub trait TransactionSigner: Send + Sync + std::fmt::Debug {
    /// User locale for localized responses
    fn locale(&self) -> String {
        "en".to_string()
    }

    /// Optional user identifier for multi-tenant scenarios
    fn user_id(&self) -> Option<String> {
        None
    }

    /// EVM chain ID for this signer
    fn chain_id(&self) -> Option<u64> {
        None
    }

    /// Solana wallet address (base58 encoded)
    fn address(&self) -> Option<String> {
        None
    }

    /// Solana public key string (base58 encoded)
    fn pubkey(&self) -> Option<String> {
        None
    }

    /// Sign and send a Solana transaction
    /// The transaction should be properly constructed with recent blockhash
    async fn sign_and_send_solana_transaction(
        &self,
        tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String, SignerError>;

    /// Sign and send an EVM transaction
    /// The transaction request should include all necessary fields (to, value, gas, etc.)
    async fn sign_and_send_evm_transaction(
        &self,
        tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String, SignerError>;

    /// Sign and send a Solana transaction with retry logic
    /// Uses default retry configuration
    async fn sign_and_send_solana_with_retry(
        &self,
        tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String, SignerError> {
        // Default implementation just calls the regular method
        // Concrete implementations can override with actual retry logic
        self.sign_and_send_solana_transaction(tx).await
    }

    /// Sign and send an EVM transaction with retry logic
    /// Uses default retry configuration
    async fn sign_and_send_evm_with_retry(
        &self,
        tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String, SignerError> {
        // Default implementation just calls the regular method
        // Concrete implementations can override with actual retry logic
        self.sign_and_send_evm_transaction(tx).await
    }

    /// Get Solana RPC client (derived from signer configuration)
    /// This client should be configured with the appropriate RPC endpoint
    /// Returns None if this signer doesn't support Solana
    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>>;

    /// Get EVM RPC client (derived from signer configuration)
    /// This client should be configured with the appropriate RPC endpoint and chain ID
    fn evm_client(&self) -> Result<Arc<dyn EvmClient>, SignerError>;
}

/// Type-safe EVM client interface
#[async_trait]
pub trait EvmClient: Send + Sync {
    /// Get the balance of an address in wei
    async fn get_balance(&self, address: &str) -> Result<U256, SignerError>;

    /// Send a transaction to the network and return the transaction hash
    async fn send_transaction(&self, tx: &TransactionRequest) -> Result<TxHash, SignerError>;

    /// Execute a call against the network without creating a transaction
    async fn call(&self, tx: &TransactionRequest) -> Result<Bytes, SignerError>;
}

/// Type-safe Solana client interface
#[async_trait]
pub trait SolanaClient: Send + Sync {
    /// Get the balance of a Solana account in lamports
    async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, SignerError>;

    /// Send a transaction to the Solana network and return the signature
    async fn send_transaction(&self, tx: &Transaction) -> Result<Signature, SignerError>;
}
