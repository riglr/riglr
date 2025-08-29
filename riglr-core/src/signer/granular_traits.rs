//! Chain-agnostic signer traits for multi-blockchain support
//!
//! This module provides a hierarchy of traits that enable riglr-core to remain
//! blockchain-agnostic while supporting multiple chain implementations. The traits
//! use serialization and type erasure to avoid direct dependencies on blockchain SDKs.
//!
//! # Architecture
//!
//! The trait hierarchy follows a composition pattern:
//! - `SignerBase`: Common functionality for all signers
//! - `SolanaSigner`: Solana-specific operations
//! - `EvmSigner`: EVM-compatible chain operations
//! - `MultiChainSigner`: Unified interface for multi-chain support
//!
//! # Chain-Agnostic Design
//!
//! To maintain chain-agnosticism:
//! - Transactions are passed as serialized bytes (Solana) or JSON (EVM)
//! - Clients are exposed as `Arc<dyn Any>` for type erasure
//! - Concrete implementations live in chain-specific crates

use super::error::SignerError;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

/// Base trait for all signers providing common metadata and type erasure.
///
/// This trait provides the foundation for all signer implementations,
/// enabling multi-tenant support and runtime type identification.
pub trait SignerBase: Send + Sync + std::fmt::Debug + Any {
    /// Returns the user's locale for localized error messages and responses.
    ///
    /// Defaults to "en" for English.
    fn locale(&self) -> String {
        "en".to_string()
    }

    /// Returns an optional user identifier for multi-tenant scenarios.
    ///
    /// This is used to isolate operations between different users
    /// in a shared application context.
    fn user_id(&self) -> Option<String> {
        None
    }

    /// Returns a reference to self as `Any` for downcasting to concrete types.
    ///
    /// This enables runtime type checking and casting when needed.
    fn as_any(&self) -> &dyn Any;
}

/// Trait for Solana blockchain signing capabilities.
///
/// Implementations of this trait handle Solana-specific operations
/// without exposing Solana SDK types to riglr-core.
///
/// # Chain-Agnostic Design
///
/// Transactions are passed as serialized byte arrays using bincode,
/// avoiding direct dependency on solana-sdk types.
#[async_trait]
pub trait SolanaSigner: SignerBase {
    /// Returns the Solana wallet address in base58 encoding.
    fn address(&self) -> String;

    /// Returns the Solana public key in base58 encoding.
    fn pubkey(&self) -> String;

    /// Signs and sends a Solana transaction to the network.
    ///
    /// # Parameters
    /// * `tx` - Mutable byte array containing a bincode-serialized Solana transaction
    ///
    /// # Returns
    /// * `Ok(String)` - Transaction signature on success
    /// * `Err(SignerError)` - Error details on failure
    ///
    /// # Implementation Notes
    /// Implementations should deserialize the bytes, sign the transaction,
    /// and submit it to the Solana network.
    async fn sign_and_send_transaction(&self, tx: &mut Vec<u8>) -> Result<String, SignerError>;

    /// Signs and sends a transaction with automatic retry on transient failures.
    ///
    /// Default implementation delegates to `sign_and_send_transaction`.
    /// Override for custom retry logic.
    async fn sign_and_send_with_retry(&self, tx: &mut Vec<u8>) -> Result<String, SignerError> {
        // Default implementation just calls the regular method
        self.sign_and_send_transaction(tx).await
    }

    /// Returns the underlying Solana RPC client as a type-erased trait object.
    ///
    /// The client should be downcast to the appropriate type
    /// (e.g., `solana_client::rpc_client::RpcClient`) by the caller.
    fn client(&self) -> Arc<dyn std::any::Any + Send + Sync>;
}

/// Trait for EVM-compatible blockchain signing capabilities.
///
/// Implementations of this trait handle operations for Ethereum
/// and EVM-compatible chains (Polygon, Arbitrum, Optimism, etc.)
/// without exposing alloy or ethers types to riglr-core.
///
/// # Chain-Agnostic Design
///
/// Transactions are passed as JSON values to avoid dependency on
/// specific Ethereum libraries (alloy, ethers-rs, etc.).
#[async_trait]
pub trait EvmSigner: SignerBase {
    /// Returns the EVM chain ID this signer is configured for.
    ///
    /// Common chain IDs:
    /// - 1: Ethereum Mainnet
    /// - 137: Polygon
    /// - 42161: Arbitrum One
    /// - 10: Optimism
    fn chain_id(&self) -> u64;

    /// Returns the EVM wallet address in checksummed hex format.
    ///
    /// The address should be 0x-prefixed and follow EIP-55 checksum encoding.
    fn address(&self) -> String;

    /// Signs and sends an EVM transaction to the network.
    ///
    /// # Parameters
    /// * `tx` - JSON object representing the transaction request
    ///
    /// # Returns
    /// * `Ok(String)` - Transaction hash (0x-prefixed) on success
    /// * `Err(SignerError)` - Error details on failure
    ///
    /// # Transaction Format
    /// The JSON should conform to the standard Ethereum transaction format:
    /// ```json
    /// {
    ///   "to": "0x...",
    ///   "value": "0x...",
    ///   "data": "0x...",
    ///   "gas": "0x...",
    ///   "gasPrice": "0x..." // or maxFeePerGas/maxPriorityFeePerGas for EIP-1559
    /// }
    /// ```
    async fn sign_and_send_transaction(&self, tx: serde_json::Value)
        -> Result<String, SignerError>;

    /// Signs and sends a transaction with automatic retry on transient failures.
    ///
    /// Default implementation delegates to `sign_and_send_transaction`.
    /// Override for custom retry logic with exponential backoff.
    async fn sign_and_send_with_retry(&self, tx: serde_json::Value) -> Result<String, SignerError> {
        // Default implementation just calls the regular method
        self.sign_and_send_transaction(tx).await
    }

    /// Returns the underlying EVM RPC client.
    ///
    /// The client trait provides chain-agnostic EVM operations.
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
