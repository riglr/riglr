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
    /// Solana blockchain
    Solana,
    /// EVM-compatible blockchain with the specified chain ID
    Evm {
        /// The numeric chain identifier for this EVM network
        chain_id: u64,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockSigner {
        locale: String,
        user_id: Option<String>,
    }

    impl MockSigner {
        fn new() -> Self {
            Self {
                locale: "en".to_string(),
                user_id: None,
            }
        }

        fn with_locale(mut self, locale: &str) -> Self {
            self.locale = locale.to_string();
            self
        }

        fn with_user_id(mut self, user_id: &str) -> Self {
            self.user_id = Some(user_id.to_string());
            self
        }
    }

    impl SignerBase for MockSigner {
        fn locale(&self) -> String {
            self.locale.clone()
        }

        fn user_id(&self) -> Option<String> {
            self.user_id.clone()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl UnifiedSigner for MockSigner {
        fn supports_solana(&self) -> bool {
            false
        }

        fn supports_evm(&self) -> bool {
            false
        }

        fn as_solana(&self) -> Option<&dyn SolanaSigner> {
            None
        }

        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            None
        }

        fn as_multi_chain(&self) -> Option<&dyn MultiChainSigner> {
            None
        }
    }

    #[test]
    fn test_chain_solana() {
        let chain = Chain::Solana;
        assert_eq!(chain, Chain::Solana);
    }

    #[test]
    fn test_chain_evm() {
        let chain = Chain::Evm { chain_id: 1 };
        assert_eq!(chain, Chain::Evm { chain_id: 1 });
    }

    #[test]
    fn test_chain_evm_different_ids() {
        let chain1 = Chain::Evm { chain_id: 1 };
        let chain2 = Chain::Evm { chain_id: 42 };
        assert_ne!(chain1, chain2);
    }

    #[test]
    fn test_signer_base_default_locale() {
        let signer = MockSigner::new();
        assert_eq!(signer.locale(), "en");
    }

    #[test]
    fn test_signer_base_custom_locale() {
        let signer = MockSigner::new().with_locale("fr");
        assert_eq!(signer.locale(), "fr");
    }

    #[test]
    fn test_signer_base_default_user_id() {
        let signer = MockSigner::new();
        assert_eq!(signer.user_id(), None);
    }

    #[test]
    fn test_signer_base_custom_user_id() {
        let signer = MockSigner::new().with_user_id("user123");
        assert_eq!(signer.user_id(), Some("user123".to_string()));
    }

    #[test]
    fn test_signer_base_as_any() {
        let signer = MockSigner::new();
        let any_ref = signer.as_any();
        assert!(any_ref.is::<MockSigner>());
    }

    #[test]
    fn test_unified_signer_supports_nothing() {
        let signer = MockSigner::new();
        assert!(!signer.supports_solana());
        assert!(!signer.supports_evm());
        assert!(signer.as_solana().is_none());
        assert!(signer.as_evm().is_none());
        assert!(signer.as_multi_chain().is_none());
    }

    #[test]
    fn test_chain_debug() {
        let solana_chain = Chain::Solana;
        let evm_chain = Chain::Evm { chain_id: 1 };

        let debug_str = format!("{:?}", solana_chain);
        assert!(debug_str.contains("Solana"));

        let debug_str = format!("{:?}", evm_chain);
        assert!(debug_str.contains("Evm"));
        assert!(debug_str.contains("1"));
    }

    #[test]
    fn test_chain_clone() {
        let chain = Chain::Evm { chain_id: 42 };
        let cloned_chain = chain.clone();
        assert_eq!(chain, cloned_chain);
    }
}
