//! Granular signer traits for better separation of concerns
//!
//! This module provides more specific traits that can be composed
//! to create signers for different blockchain architectures.

use super::error::SignerError;
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;
use solana_sdk::transaction::Transaction;
use std::any::Any;
use std::sync::Arc;

/// Base trait for all signers providing common metadata and type erasure
pub trait SignerBase: Send + Sync + std::fmt::Debug + Any {
    /// User locale for localized responses
    fn locale(&self) -> String {
        "en".to_string()
    }

    /// Optional user identifier for multi-tenant scenarios
    fn user_id(&self) -> Option<String> {
        None
    }

    /// Get this signer as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Trait for Solana-specific signing capabilities
#[async_trait]
pub trait SolanaSigner: SignerBase {
    /// Solana wallet address (base58 encoded)
    fn address(&self) -> String;

    /// Solana public key string (base58 encoded)
    fn pubkey(&self) -> String;

    /// Sign and send a Solana transaction
    async fn sign_and_send_transaction(&self, tx: &mut Transaction) -> Result<String, SignerError>;

    /// Sign and send a Solana transaction with retry logic
    async fn sign_and_send_with_retry(&self, tx: &mut Transaction) -> Result<String, SignerError> {
        // Default implementation just calls the regular method
        self.sign_and_send_transaction(tx).await
    }

    /// Get Solana RPC client
    fn client(&self) -> Arc<solana_client::rpc_client::RpcClient>;
}

/// Trait for EVM-specific signing capabilities
#[async_trait]
pub trait EvmSigner: SignerBase {
    /// EVM chain ID for this signer
    fn chain_id(&self) -> u64;

    /// EVM wallet address (0x prefixed hex string)
    fn address(&self) -> String;

    /// Sign and send an EVM transaction
    async fn sign_and_send_transaction(
        &self,
        tx: TransactionRequest,
    ) -> Result<String, SignerError>;

    /// Sign and send an EVM transaction with retry logic
    async fn sign_and_send_with_retry(
        &self,
        tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        // Default implementation just calls the regular method
        self.sign_and_send_transaction(tx).await
    }

    /// Get EVM RPC client
    fn client(&self) -> Result<Arc<dyn super::traits::EvmClient>, SignerError>;
}

/// Trait for signers that support multiple chains
/// This combines the capabilities of both Solana and EVM signers
#[async_trait]
pub trait MultiChainSigner: SignerBase {
    /// Get the primary chain this signer operates on
    fn primary_chain(&self) -> Chain;

    /// Check if a specific chain is supported
    fn supports_chain(&self, chain: &Chain) -> bool;

    /// Get as Solana signer if supported
    fn as_solana(&self) -> Option<&dyn SolanaSigner>;

    /// Get as EVM signer if supported
    fn as_evm(&self) -> Option<&dyn EvmSigner>;
}

/// Enum representing supported blockchain types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chain {
    Solana,
    Evm { chain_id: u64 },
}

/// Unified signer trait that can represent any type of signer
/// This is used as the primary trait for SignerContext
pub trait UnifiedSigner: SignerBase {
    /// Check if this signer supports Solana
    fn supports_solana(&self) -> bool;

    /// Check if this signer supports EVM
    fn supports_evm(&self) -> bool;

    /// Try to get as a Solana signer
    fn as_solana(&self) -> Option<&dyn SolanaSigner>;

    /// Try to get as an EVM signer
    fn as_evm(&self) -> Option<&dyn EvmSigner>;

    /// Try to get as a MultiChain signer
    fn as_multi_chain(&self) -> Option<&dyn MultiChainSigner>;
}

/// Adapter to wrap TransactionSigner for use with granular traits
pub struct LegacySignerAdapter<T: super::traits::TransactionSigner + 'static> {
    inner: Arc<T>,
}

impl<T: super::traits::TransactionSigner + 'static> LegacySignerAdapter<T> {
    pub fn new(inner: Arc<T>) -> Self {
        Self { inner }
    }
}

impl<T: super::traits::TransactionSigner + 'static> SignerBase for LegacySignerAdapter<T> {
    fn locale(&self) -> String {
        self.inner.locale()
    }

    fn user_id(&self) -> Option<String> {
        self.inner.user_id()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<T: super::traits::TransactionSigner + 'static> std::fmt::Debug for LegacySignerAdapter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: super::traits::TransactionSigner + 'static> UnifiedSigner for LegacySignerAdapter<T> {
    fn supports_solana(&self) -> bool {
        self.inner.pubkey().is_some()
    }

    fn supports_evm(&self) -> bool {
        self.inner.chain_id().is_some()
    }

    fn as_solana(&self) -> Option<&dyn SolanaSigner> {
        // Legacy adapters don't implement the granular traits directly
        None
    }

    fn as_evm(&self) -> Option<&dyn EvmSigner> {
        // Legacy adapters don't implement the granular traits directly
        None
    }

    fn as_multi_chain(&self) -> Option<&dyn MultiChainSigner> {
        None
    }
}
