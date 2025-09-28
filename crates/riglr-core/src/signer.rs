//! Blockchain signer implementations and context management for secure transactions.
//!
//! This module provides the SignerContext pattern for thread-safe transaction signing,
//! along with unified signer abstractions for both EVM and Solana chains.
//!
//! # Architecture
//!
//! The module implements two key patterns:
//! - **SignerContext**: Thread-local storage for secure transaction signing
//! - **Configuration-driven signers**: Type-safe network configuration from riglr-config
//!
//! # Usage
//!
//! ```ignore
//! use riglr_config::Config;
//! use riglr_core::signer::SignerContext;
//! // Import concrete signers from tools crates
//! use riglr_solana_tools::LocalSolanaSigner;
//! use riglr_evm_tools::LocalEvmSigner;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::SignerError>> {
//! // Load configuration from environment
//! let config = Config::from_env()?;
//!
//! // Create signers with proper network configuration
//! // NOTE: Concrete signers are in tools crates
//! let signer = Arc::new(LocalSolanaSigner::new(
//!     keypair,
//!     config.providers.solana.network_config()
//! )?);
//!
//! // Execute transactional operations within SignerContext
//! SignerContext::with_signer(signer, async {
//!     // Tools can now access the signer via SignerContext::current()
//!     // for operations that require transaction signing
//!     transfer_sol("recipient", 1.0).await?;
//!     Ok(())
//! }).await?;
//! # Ok(())
//! # }
//! ```

/// `SignerError` types for signer operations and context management.
///
/// This module provides comprehensive error handling for all signer-related operations,
/// including context access, transaction signing, and multi-chain coordination.
///
/// The error types are designed to provide detailed context for debugging while
/// maintaining security by not exposing sensitive information in error messages.
pub mod error {
    use alloc::string::{String, ToString};
    use core::{error::Error as StdError, result};
    use thiserror::Error;

    /// Core error trait for all signer-related operations.
    ///
    /// This trait provides a common interface for error handling across different
    /// signer implementations, allowing for extensibility and custom error types.
    pub trait Error: StdError + Send + Sync + 'static {
        /// Check if this error indicates a temporary condition that might succeed on retry.
        ///
        /// # Returns
        /// `true` for network errors and some signing failures that might be transient,
        /// `false` for configuration errors and permanent failures.
        fn is_retriable(&self) -> bool;

        /// Get a human-readable error message.
        fn message(&self) -> String;

        /// Get the error kind for categorization.
        fn kind(&self) -> SignerErrorType;
    }

    /// Enumeration of signer error categories for high-level error handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SignerErrorType {
        Configuration,
        Generic,
        InvalidInput,
        Network,
        NoContext,
        SigningFailed,
        UnsupportedOperation,
    }

    /// Standard implementation of `Error` for common use cases.
    #[derive(Error, Debug, Clone)]
    pub enum Standard {
        #[error("Signer configuration error: {0}")]
        Configuration(String),

        #[error("Generic signer error: {0}")]
        Generic(String),

        #[error("Invalid input to signer operation: {0}")]
        InvalidInput(String),

        #[error("Network error in signer operation: {0}")]
        Network(String),

        #[error("No signer context available")]
        NoContext,

        #[error("Transaction signing failed: {0}")]
        SigningFailed(String),

        #[error("Operation not supported by current signer: {0}")]
        UnsupportedOperation(String),
    }

    impl Error for Standard {
        fn is_retriable(&self) -> bool {
            matches!(*self, Self::Network(_) | Self::SigningFailed(_))
        }

        fn message(&self) -> String {
            self.to_string()
        }

        fn kind(&self) -> SignerErrorType {
            match *self {
                Self::Configuration(_) => SignerErrorType::Configuration,
                Self::Generic(_) => SignerErrorType::Generic,
                Self::InvalidInput(_) => SignerErrorType::InvalidInput,
                Self::Network(_) => SignerErrorType::Network,
                Self::NoContext => SignerErrorType::NoContext,
                Self::SigningFailed(_) => SignerErrorType::SigningFailed,
                Self::UnsupportedOperation(_) => SignerErrorType::UnsupportedOperation,
            }
        }
    }

    /// Convenience type alias for Results that can contain any `Error`.
    pub type Result<T> = result::Result<T, Box<dyn Error>>;

    /// Type alias for backwards compatibility
    pub type StandardErrorType = Standard;
}

/// Granular traits for type-safe multi-chain signer abstractions.
///
/// This module provides a comprehensive trait hierarchy for blockchain signers,
/// designed for maximum composability and type safety. The traits support both
/// single-chain and multi-chain use cases with clear capability declaration.
pub mod granular_traits {
    use alloc::string::String;
    use core::fmt::{self, Debug, Display};

    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};

    use super::error::Result;
    use super::traits::{EvmClient, SolanaClient};

    /// Represents a blockchain network type for capability checking.
    ///
    /// This enum is used throughout the signer system to declare and check
    /// which blockchain networks a signer supports. It enables type-safe
    /// casting and capability verification.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[non_exhaustive]
    pub enum Chain {
        /// Ethereum Virtual Machine compatible chains (Ethereum, Polygon, BSC, etc.)
        Evm,
        /// Solana blockchain
        Solana,
    }

    impl Display for Chain {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                Self::Evm => write!(f, "EVM"),
                Self::Solana => write!(f, "Solana"),
            }
        }
    }

    /// Base signer functionality that all signers must implement.
    ///
    /// This trait defines the minimal interface that every signer must provide,
    /// including capability declaration and user identification for multi-tenant scenarios.
    ///
    /// # Thread Safety
    ///
    /// All signer implementations must be `Send + Sync` to support use in
    /// async contexts and across thread boundaries.
    #[async_trait]
    pub trait SignerBase: Send + Sync + Debug {
        /// Returns the chains this signer supports.
        ///
        /// This method is used for capability checking before attempting
        /// chain-specific operations. Implementations should return all
        /// chains they can handle.
        ///
        /// # Examples
        /// ```ignore
        /// use riglr_core::signer::{SignerBase, Chain};
        ///
        /// let signer = SomeSigner::new();
        /// let chains = signer.supported_chains();
        /// assert!(chains.contains(&Chain::Solana));
        /// ```
        fn supported_chains(&self) -> &[Chain];

        /// Check if the signer supports a specific chain.
        ///
        /// This is a convenience method that checks if the given chain
        /// is in the list returned by [`supported_chains()`].
        ///
        /// # Arguments
        /// * `chain` - The blockchain to check support for
        ///
        /// # Returns
        /// `true` if the signer supports the chain, `false` otherwise
        #[inline]
        fn supports_chain(&self, chain: Chain) -> bool {
            self.supported_chains().contains(&chain)
        }

        /// Returns a user identifier for this signer context.
        ///
        /// This is crucial for multi-tenant applications where different users
        /// may have different signers. The `user_id` should be stable and unique
        /// for each user/signer combination.
        ///
        /// # Security Considerations
        ///
        /// - The `user_id` should not contain sensitive information
        /// - It should be consistent across requests for the same user
        /// - Consider using UUIDs or hashed identifiers rather than email/username
        ///
        /// # Returns
        /// A string that uniquely identifies the user/tenant for this signer
        fn user_id(&self) -> String;
    }

    /// EVM-specific signer functionality for Ethereum-compatible chains.
    ///
    /// This trait extends [`SignerBase`] with EVM-specific operations like
    /// transaction signing, message signing, and account management.
    ///
    /// # Implementation Requirements
    ///
    /// Implementors must provide EVM client access and handle all standard
    /// Ethereum transaction types including legacy, EIP-1559, and contract calls.
    #[async_trait]
    pub trait EvmSigner: SignerBase {
        /// Returns the Ethereum address for this signer.
        ///
        /// This should be the checksummed address string that can be used
        /// in transactions and balance queries.
        ///
        /// # Returns
        /// The Ethereum address as a hexadecimal string with "0x" prefix
        ///
        /// # Examples
        /// ```ignore
        /// let address = signer.address();
        /// assert!(address.starts_with("0x"));
        /// assert_eq!(address.len(), 42); // 0x + 40 hex chars
        /// ```
        fn address(&self) -> String;

        /// Returns the chain ID for this signer's network.
        ///
        /// The chain ID is used in transaction signing to prevent
        /// replay attacks across different networks.
        ///
        /// # Returns
        /// The EIP-155 chain ID as a 64-bit integer
        fn chain_id(&self) -> u64;

        /// Get a reference to the EVM client for read operations.
        ///
        /// This provides access to the underlying RPC client for querying
        /// blockchain state, estimating gas, and other read-only operations.
        ///
        /// # Returns
        /// A reference to an object implementing [`EvmClient`]
        fn client(&self) -> &dyn EvmClient;

        /// Sign and submit a transaction to the network.
        ///
        /// This method handles the complete transaction lifecycle:
        /// 1. Transaction preparation (nonce, gas estimation if needed)
        /// 2. Cryptographic signing with the signer's private key
        /// 3. Submission to the network via RPC
        /// 4. Return of transaction hash for tracking
        ///
        /// # Arguments
        /// * `transaction_request` - The transaction to sign and send
        ///
        /// # Returns
        /// The transaction hash as a hexadecimal string, or an error
        ///
        /// # `SignerErrors`
        /// - [`SignerError::SigningFailed`] if transaction signing fails
        /// - [`SignerError::NetworkSignerError`] if submission to network fails
        /// - [`SignerError::InvalidInput`] if transaction parameters are invalid
        async fn sign_and_send_transaction(
            &self,
            transaction_request: serde_json::Value,
        ) -> Result<String>;

        /// Sign a raw message with the signer's private key.
        ///
        /// This implements Ethereum's `personal_sign` method, which prefixes
        /// the message with "\x19Ethereum Signed Message:\n{len}" before signing.
        ///
        /// # Arguments
        /// * `message` - The raw bytes to sign
        ///
        /// # Returns
        /// The signature as a hexadecimal string, or an error if signing fails
        ///
        /// # Security
        /// Always use this for user-facing message signing rather than raw ECDSA
        /// to prevent signature reuse in transactions.
        async fn sign_message(&self, message: &[u8]) -> Result<String>;
    }

    /// Solana-specific signer functionality.
    ///
    /// This trait extends [`SignerBase`] with Solana-specific operations including
    /// transaction signing, message signing, and program interaction.
    ///
    /// # Implementation Requirements
    ///
    /// Implementors must handle Solana's unique transaction format, including
    /// multiple instruction support, recent blockhash management, and fee handling.
    #[async_trait]
    pub trait SolanaSigner: SignerBase {
        /// Get a reference to the Solana client for read operations.
        ///
        /// This provides access to the underlying RPC client for querying
        /// account data, program state, and other blockchain information.
        ///
        /// # Returns
        /// A reference to an object implementing [`SolanaClient`]
        fn client(&self) -> &dyn SolanaClient;

        /// Returns the Solana public key for this signer.
        ///
        /// This is the base58-encoded public key that serves as the account
        /// address on Solana.
        ///
        /// # Returns
        /// The Solana public key as a base58-encoded string
        ///
        /// # Examples
        /// ```ignore
        /// let pubkey = signer.pubkey();
        /// // Solana pubkeys are 32 bytes = 44 base58 chars (typically)
        /// assert!(pubkey.len() >= 32 && pubkey.len() <= 44);
        /// ```
        fn pubkey(&self) -> String;

        /// Sign and submit a transaction to the Solana network.
        ///
        /// This method handles the complete Solana transaction lifecycle:
        /// 1. Recent blockhash retrieval (if not provided)
        /// 2. Transaction serialization and signing
        /// 3. Submission to the network
        /// 4. Return of transaction signature for tracking
        ///
        /// # Arguments
        /// * `transaction` - The transaction data as JSON (Solana transaction format)
        ///
        /// # Returns
        /// The transaction signature as a base58-encoded string, or an error
        ///
        /// # `SignerErrors`
        /// - [`SignerError::SigningFailed`] if transaction signing fails
        /// - [`SignerError::NetworkSignerError`] if submission fails
        /// - [`SignerError::InvalidInput`] if transaction format is invalid
        async fn sign_and_send_transaction(&self, transaction: serde_json::Value)
            -> Result<String>;

        /// Sign a raw message with the signer's private key.
        ///
        /// Unlike Ethereum, Solana message signing doesn't use a standard prefix.
        /// The message is signed directly with Ed25519.
        ///
        /// # Arguments
        /// * `message` - The raw bytes to sign
        ///
        /// # Returns
        /// The Ed25519 signature as a base58-encoded string, or an error
        ///
        /// # Security
        /// Be careful with raw message signing - ensure the message content
        /// is not a valid transaction that could be replayed.
        async fn sign_message(&self, message: &[u8]) -> Result<String>;
    }

    /// Unified signer interface supporting multiple blockchain networks.
    ///
    /// This trait provides a single interface for signers that can handle multiple
    /// blockchain types. It includes convenience methods for checking capabilities
    /// and accessing chain-specific functionality.
    ///
    /// # Design Philosophy
    ///
    /// The `UnifiedSigner` trait allows tools to work with signers generically
    /// while still providing type-safe access to chain-specific functionality
    /// through the `as_evm()` and `as_solana()` methods.
    ///
    /// # Thread Safety
    ///
    /// All unified signers must be thread-safe (`Send + Sync`) to support
    /// concurrent use in multi-threaded applications.
    pub trait UnifiedSigner: SignerBase {
        /// Cast to an EVM signer if supported.
        ///
        /// This method provides type-safe access to EVM-specific functionality.
        /// It should only return `Some` if [`supports_evm()`] returns `true`.
        ///
        /// # Returns
        /// `Some(&dyn EvmSigner)` if EVM is supported, `None` otherwise
        ///
        /// # Examples
        /// ```ignore
        /// if let Some(evm_signer) = unified.as_evm() {
        ///     let address = evm_signer.address();
        ///     println!("EVM address: {}", address);
        /// }
        /// ```
        fn as_evm(&self) -> Option<&dyn EvmSigner>;

        /// Cast to a Solana signer if supported.
        ///
        /// This method provides type-safe access to Solana-specific functionality.
        /// It should only return `Some` if [`supports_solana()`] returns `true`.
        ///
        /// # Returns
        /// `Some(&dyn SolanaSigner)` if Solana is supported, `None` otherwise
        ///
        /// # Examples
        /// ```ignore
        /// if let Some(solana_signer) = unified.as_solana() {
        ///     let pubkey = solana_signer.pubkey();
        ///     println!("Solana address: {}", pubkey);
        /// }
        /// ```
        fn as_solana(&self) -> Option<&dyn SolanaSigner>;

        /// Check if this signer supports EVM operations.
        ///
        /// # Returns
        /// `true` if the signer can handle EVM transactions, `false` otherwise
        #[inline]
        fn supports_evm(&self) -> bool {
            self.supports_chain(Chain::Evm)
        }

        /// Check if this signer supports Solana operations.
        ///
        /// # Returns
        /// `true` if the signer can handle Solana transactions, `false` otherwise
        #[inline]
        fn supports_solana(&self) -> bool {
            self.supports_chain(Chain::Solana)
        }
    }

    /// Multi-chain signer that implements multiple blockchain protocols.
    ///
    /// This trait is for signers that can natively handle multiple chains
    /// simultaneously, such as hardware wallets or multi-chain software wallets.
    ///
    /// # Use Cases
    ///
    /// - Hardware wallets supporting multiple cryptocurrencies
    /// - Software wallets with multiple private keys
    /// - Custodial services managing multiple blockchain accounts
    /// - Development tools that need to work across chains
    #[async_trait]
    pub trait MultiChainSigner: UnifiedSigner + EvmSigner + SolanaSigner {
        /// Get the native chain for this signer (primary/default chain).
        ///
        /// For multi-chain signers, this indicates which blockchain should be
        /// used as the default when the context doesn't specify a preference.
        ///
        /// # Returns
        /// The [`Chain`] that represents this signer's primary blockchain
        fn primary_chain(&self) -> Chain;

        /// Switch the signer's active context to a specific chain.
        ///
        /// This allows multi-chain signers to change their operating mode
        /// dynamically based on the current operation's requirements.
        ///
        /// # Arguments
        /// * `chain` - The blockchain to switch to
        ///
        /// # `SignerErrors`
        /// Returns [`SignerError::UnsupportedOperation`] if the chain is not supported
        ///
        /// # Examples
        /// ```ignore
        /// let mut signer = MultiChainWallet::new();
        /// signer.switch_chain(Chain::Solana).await?;
        /// // Now signer operations will default to Solana
        /// ```
        async fn switch_chain(&mut self, chain: Chain) -> Result<()>;
    }
}

/// Client traits for blockchain RPC operations.
///
/// This module defines the interface that blockchain clients must implement
/// to be compatible with the riglr signer system. These traits abstract
/// over specific RPC libraries while providing the functionality needed
/// for transaction operations.
pub mod traits {
    use alloc::string::String;
    use async_trait::async_trait;
    use serde_json::Value as JsonValue;

    use super::error::Result;

    /// Abstract interface for EVM-compatible blockchain clients.
    ///
    /// This trait defines the minimum RPC functionality needed to support
    /// EVM signer operations. Implementations should handle connection management,
    /// error mapping, and network-specific configuration.
    ///
    /// # Implementation Notes
    ///
    /// - All methods should handle network errors gracefully
    /// - Amounts should be handled in wei (smallest unit) to avoid precision issues
    /// - Transaction hashes and addresses should be returned as lowercase hex with 0x prefix
    #[async_trait]
    pub trait EvmClient: Send + Sync {
        /// Call a contract method without sending a transaction (view/pure functions).
        ///
        /// # Arguments
        /// * `call_request` - Contract call parameters (to, data, from, etc.)
        /// * `block` - Block parameter ("latest", "pending", or block number)
        ///
        /// # Returns
        /// The return data as a hex string, or an error
        async fn call(&self, call_request: &JsonValue, block: Option<&str>) -> Result<String>;

        /// Estimate gas needed for a transaction.
        ///
        /// # Arguments
        /// * `transaction` - Transaction request object with to, from, data, value, etc.
        ///
        /// # Returns
        /// Estimated gas as a decimal string, or an error
        async fn estimate_gas(&self, transaction: &JsonValue) -> Result<String>;

        /// Get the balance of an address in wei.
        ///
        /// # Arguments
        /// * `address` - The Ethereum address to query (with or without 0x prefix)
        ///
        /// # Returns
        /// The balance in wei as a decimal string, or an error
        ///
        /// # Examples
        /// ```ignore
        /// let balance = client.get_balance("0x742d35Cc6634C0532925a3b8D8f0ee68Ad3E1af2").await?;
        /// println!("Balance: {} wei", balance);
        /// ```
        async fn get_balance(&self, address: &str) -> Result<String>;

        /// Get the latest block number.
        ///
        /// # Returns
        /// The latest block number as a decimal string, or an error
        async fn get_block_number(&self) -> Result<String>;

        /// Get the current chain ID from the network.
        ///
        /// # Returns
        /// The EIP-155 chain ID as a decimal string, or an error
        async fn get_chain_id(&self) -> Result<String>;

        /// Get the current gas price from the network.
        ///
        /// For EIP-1559 networks, this should return a reasonable base fee + priority fee.
        ///
        /// # Returns
        /// Gas price in wei as a decimal string, or an error
        async fn get_gas_price(&self) -> Result<String>;

        /// Get the current nonce for an address.
        ///
        /// This should return the transaction count (number of transactions sent)
        /// for the address, which is used as the nonce for the next transaction.
        ///
        /// # Arguments
        /// * `address` - The Ethereum address to query
        ///
        /// # Returns
        /// The nonce as a decimal string, or an error
        async fn get_nonce(&self, address: &str) -> Result<String>;

        /// Get transaction by hash.
        ///
        /// # Arguments
        /// * `tx_hash` - Transaction hash as hex string
        ///
        /// # Returns
        /// Transaction data as JSON, or None if not found, or an error
        async fn get_transaction(&self, tx_hash: &str) -> Result<Option<JsonValue>>;

        /// Get transaction receipt by hash.
        ///
        /// # Arguments
        /// * `tx_hash` - Transaction hash as hex string
        ///
        /// # Returns
        /// Transaction receipt as JSON, or None if not found, or an error
        async fn get_transaction_receipt(&self, tx_hash: &str) -> Result<Option<JsonValue>>;

        /// Broadcast a signed transaction to the network.
        ///
        /// # Arguments
        /// * `signed_tx` - The signed transaction as a hex string (with or without 0x prefix)
        ///
        /// # Returns
        /// The transaction hash as a hex string with 0x prefix, or an error
        async fn send_raw_transaction(&self, signed_tx: &str) -> Result<String>;
    }

    /// Abstract interface for Solana blockchain clients.
    ///
    /// This trait defines the minimum RPC functionality needed to support
    /// Solana signer operations. Implementations should handle connection management,
    /// commitment levels, and Solana-specific error conditions.
    ///
    /// # Implementation Notes
    ///
    /// - All SOL amounts should be handled in lamports (1 SOL = 1,000,000,000 lamports)
    /// - Public keys and signatures should be base58-encoded
    /// - Recent blockhashes should be fetched with appropriate commitment levels
    #[async_trait]
    pub trait SolanaClient: Send + Sync {
        /// Confirm a transaction by signature.
        ///
        /// # Arguments
        /// * `signature` - Transaction signature as a base58 string
        ///
        /// # Returns
        /// Transaction confirmation status as JSON, or an error
        async fn confirm_transaction(&self, signature: &str) -> Result<JsonValue>;

        /// Get account information.
        ///
        /// # Arguments
        /// * `pubkey` - The account's public key as a base58 string
        ///
        /// # Returns
        /// Account information as JSON (owner, lamports, data, etc.), or an error
        async fn get_account_info(&self, pubkey: &str) -> Result<Option<JsonValue>>;

        /// Get the balance of an account in lamports.
        ///
        /// # Arguments
        /// * `pubkey` - The Solana public key as a base58 string
        ///
        /// # Returns
        /// The balance in lamports as a decimal string, or an error
        ///
        /// # Examples
        /// ```ignore
        /// let balance = client.get_balance("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zZtAWWM").await?;
        /// println!("Balance: {} lamports", balance);
        /// ```
        async fn get_balance(&self, pubkey: &str) -> Result<String>;

        /// Get the fee for a transaction.
        ///
        /// # Arguments
        /// * `transaction` - Transaction data to estimate fees for
        ///
        /// # Returns
        /// The fee in lamports as a decimal string, or an error
        async fn get_fee_for_transaction(&self, transaction: &JsonValue) -> Result<String>;

        /// Get a recent blockhash for transaction creation.
        ///
        /// Recent blockhashes are required for all Solana transactions and expire
        /// after approximately 2 minutes. This should return a blockhash that's
        /// recent enough to be accepted by the network.
        ///
        /// # Returns
        /// A recent blockhash as a base58 string, or an error
        async fn get_latest_blockhash(&self) -> Result<String>;

        /// Get minimum balance required for rent exemption.
        ///
        /// # Arguments
        /// * `data_len` - Length of account data in bytes
        ///
        /// # Returns
        /// Minimum balance in lamports as a decimal string, or an error
        async fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> Result<String>;

        /// Get the slot (block) number.
        ///
        /// # Returns
        /// The current slot number as a decimal string, or an error
        async fn get_slot(&self) -> Result<String>;

        /// Get transaction details by signature.
        ///
        /// # Arguments
        /// * `signature` - Transaction signature as a base58 string
        ///
        /// # Returns
        /// Transaction details as JSON, or None if not found, or an error
        async fn get_transaction(&self, signature: &str) -> Result<Option<JsonValue>>;

        /// Send a signed transaction to the network.
        ///
        /// # Arguments
        /// * `signed_transaction` - The signed transaction as a base64 or base58 string
        ///
        /// # Returns
        /// The transaction signature as a base58 string, or an error
        async fn send_transaction(&self, signed_transaction: &str) -> Result<String>;
    }
}

use crate::signer::error::{Error, Standard};
use alloc::sync::Arc;
use core::future::Future;
use tokio::task_local;

// Re-export error types for convenience
#[expect(clippy::module_name_repetitions)]
pub use error::Error as SignerError;
#[expect(clippy::module_name_repetitions)]
pub use granular_traits::{
    Chain, EvmSigner, MultiChainSigner, SignerBase, SolanaSigner, UnifiedSigner,
};
pub use traits::{EvmClient, SolanaClient};

// Thread-local storage for current signer context
// This provides secure isolation between different async tasks/requests
task_local! {
    static CURRENT_UNIFIED_SIGNER: Arc<dyn UnifiedSigner>;
}

/// The `SignerContext` provides thread-local signer management for secure multi-tenant operation.
///
/// This enables stateless tools that can access the appropriate signer without explicit passing,
/// while maintaining strict isolation between different async tasks and users.
///
/// ## Security Features
///
/// - **Thread isolation**: Each async task has its own isolated signer context
/// - **No signer leakage**: Contexts cannot access signers from other tasks
/// - **Safe concurrent access**: Multiple tasks can run concurrently with different signers
/// - **Automatic cleanup**: Contexts are automatically cleaned up when tasks complete
///
/// ## Usage Patterns
///
/// ### Basic Usage
///
/// ```ignore
/// use riglr_core::signer::SignerContext;
/// // Concrete signers are from tools crates:
/// // use riglr_solana_tools::LocalSolanaSigner;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), riglr_core::signer::SignerError> {
/// let keypair = Keypair::new();
/// let signer = Arc::new(LocalSolanaSigner::new(
///     keypair,
///     "https://api.devnet.solana.com".to_string()
/// ));
///
/// // Execute code with signer context
/// let result = SignerContext::with_signer(signer, async {
///     // Inside this scope, tools can access the signer
///     let current = SignerContext::current()?;
///     let user_id = current.user_id();
///     Ok(format!("Processing for user: {:?}", user_id))
/// }).await?;
///
/// println!("Result: {}", result);
/// # Ok(())
/// # }
/// ```
///
/// ### Multi-Tenant Service Example
///
/// ```ignore
/// use riglr_core::signer::{SignerContext, UnifiedSigner};
/// use std::sync::Arc;
///
/// async fn handle_user_request(
///     user_signer: Arc<dyn UnifiedSigner>,
///     operation: &str
/// ) -> Result<String, Box<dyn std::error::SignerError + Send + Sync>> {
///     SignerContext::with_signer(user_signer, async {
///         // All operations in this scope use the user's signer
///         match operation {
///             "balance" => check_balance().await,
///             "transfer" => perform_transfer().await,
///             _ => Err(riglr_core::signer::SignerError::NoSignerContext)
///         }
///     }).await.map_err(Into::into)
/// }
///
/// async fn check_balance() -> Result<String, riglr_core::signer::SignerError> {
///     let signer = SignerContext::current()?;
///     Ok(format!("Balance for user: {:?}", signer.user_id()))
/// }
///
/// async fn perform_transfer() -> Result<String, riglr_core::signer::SignerError> {
///     let signer = SignerContext::current()?;
///     // Perform actual transfer using signer...
///     Ok("Transfer completed".to_string())
/// }
/// ```
///
/// ### `SignerError` Handling
///
/// Tools should always check for signer availability:
///
/// ```rust
/// use riglr_core::signer::SignerContext;
///
/// async fn safe_operation() -> Result<String, Box<dyn std::error::SignerError + Send + Sync>> {
///     if !SignerContext::is_available() {
///         Err("This operation requires a signer context".into())
///     }
///
///     let signer = SignerContext::current()?;
///     // Proceed with operation...
///     Ok("Operation completed".to_string())
/// }
/// ```
///
/// ## Security Considerations
///
/// - **Never store signers globally**: Always use the context pattern
/// - **Validate user permissions**: Check that users own the addresses they're operating on
/// - **Audit all operations**: Log all signer usage for security auditing
/// - **Use environment-specific endpoints**: Different signers for mainnet/testnet
#[derive(Debug)]
#[non_exhaustive]
pub struct Context;

/// A handle to the current Solana signer, providing type-safe access.
///
/// This handle guarantees that the underlying signer supports Solana operations.
/// It implements `Deref` to `dyn SolanaSigner`, allowing direct access to all
/// Solana-specific methods.
///
/// # Thread Safety
///
/// The handle is `Send + Sync` and can be passed between threads safely.
/// The underlying signer is reference-counted and immutable.
///
/// # Examples
///
/// ```ignore
/// use riglr_core::signer::SignerContext;
///
/// async fn solana_transfer() -> Result<(), Box<dyn std::error::SignerError + Send + Sync>> {
///     // Get a type-safe handle to the Solana signer
///     let signer = SignerContext::current_as_solana()?;
///
///     // Access Solana-specific methods directly
///     let pubkey = signer.pubkey();
///     let signature = signer.sign_message(b"Hello").await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct SolanaSignerHandle(Arc<dyn UnifiedSigner>);

impl SolanaSignerHandle {
    /// Get access to the Solana signer
    ///
    /// # Panics
    ///
    /// This function will panic if the underlying `UnifiedSigner` does not support Solana operations.
    #[must_use]
    pub fn signer(&self) -> &dyn SolanaSigner {
        #[expect(clippy::expect_used)]
        {
            self.0
                .as_solana()
                .expect("SolanaSignerHandle must contain a valid Solana signer")
        }
    }
}

/// EVM-specific signer handle that provides type-safe access to EVM signing capabilities
#[derive(Debug)]
pub struct EvmSignerHandle(Arc<dyn UnifiedSigner>);

impl EvmSignerHandle {
    /// Get access to the EVM signer
    ///
    /// # Panics
    ///
    /// This function will panic if the underlying `UnifiedSigner` does not support EVM operations.
    #[must_use]
    pub fn signer(&self) -> &dyn EvmSigner {
        #[expect(clippy::expect_used)]
        {
            self.0
                .as_evm()
                .expect("EvmSignerHandle must contain a valid EVM signer")
        }
    }
}

// SolanaSignerHandle and EvmSignerHandle automatically implement Send and Sync
// since Arc<dyn UnifiedSigner> implements Send + Sync

impl Context {
    /// Get the current unified signer from thread-local context.
    ///
    /// This function retrieves the signer that was set by [`Context::with_signer()`].
    /// Returns the `UnifiedSigner` which can be cast to specific signer types.
    ///
    /// # Returns
    /// * `Ok(Arc<dyn UnifiedSigner>)` - The current signer if available
    /// * `Err(SignerError::NoSignerContext)` - If called outside a signer context
    ///
    /// # `SignerErrors`
    /// Returns an error if called outside a signer context.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::signer::SignerContext;
    ///
    /// async fn tool_that_needs_signer() -> Result<String, Box<dyn std::error::SignerError + Send + Sync>> {
    ///     // Get the current unified signer
    ///     let signer = SignerContext::current()?;
    ///
    ///     // Check what chains it supports
    ///     if signer.supports_solana() {
    ///         let solana = SignerContext::current_as_solana()?;
    ///         Ok(format!("Solana pubkey: {}", solana.pubkey()))
    ///     }
    ///
    ///     if signer.supports_evm() {
    ///         let evm = SignerContext::current_as_evm()?;
    ///         Ok(format!("EVM address: {}", evm.address()))
    ///     }
    ///
    ///     Ok("Unknown signer type".to_owned())
    /// }
    /// ```
    #[inline]
    /// Get the current unified signer from thread-local context.
    ///
    /// # Returns
    /// The current signer if available.
    ///
    /// # Errors
    /// Returns an error if called outside a signer context.
    pub fn current() -> Result<Arc<dyn UnifiedSigner>, Box<dyn SignerError>> {
        CURRENT_UNIFIED_SIGNER
            .try_with(Clone::clone)
            .map_err(|_ignored_access_error| Box::new(Standard::NoContext) as Box<dyn SignerError>)
    }

    /// Get the current signer as an EVM signer with type-safe access.
    ///
    /// This method provides a strongly-typed handle to the current signer's EVM
    /// capabilities. The returned handle implements `Deref` to `dyn EvmSigner`,
    /// allowing direct access to all EVM-specific methods.
    ///
    /// # Returns
    /// * `Ok(EvmSignerHandle)` - A handle providing type-safe access to EVM operations
    /// * `Err(SignerError::NoSignerContext)` - If called outside a signer context
    /// * `Err(SignerError::UnsupportedOperation)` - If the current signer doesn't support EVM
    ///
    /// # Errors
    /// This function will return an error if:
    /// * It is called outside a signer context
    /// * The current signer doesn't support EVM operations
    ///
    /// # Thread Safety
    ///
    /// The returned handle is thread-safe and can be passed between async tasks.
    /// The underlying signer is immutable and reference-counted.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::signer::SignerContext;
    ///
    /// async fn evm_only_operation() -> Result<(), Box<dyn std::error::SignerError + Send + Sync>> {
    ///     // This will fail if the current signer doesn't support EVM
    ///     let signer = SignerContext::current_as_evm()?;
    ///
    ///     // Now we have type-safe access to EVM methods
    ///     let address = signer.address();
    ///     let chain_id = signer.chain_id();
    ///
    ///     println!("Operating on chain {} with address {}", chain_id, address);
    ///
    ///     // Build and send a transaction
    ///     let tx = TransactionRequest::default()
    ///         .to(Address::ZERO)
    ///         .value(U256::from(1000000000000000u64)); // 0.001 ETH
    ///
    ///     let tx_hash = signer.sign_and_send_transaction(tx).await?;
    ///     println!("Transaction sent: {}", tx_hash);
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn current_as_evm() -> Result<EvmSignerHandle, Box<dyn SignerError>> {
        let unified =
            CURRENT_UNIFIED_SIGNER
                .try_with(Clone::clone)
                .map_err(|_ignored_access_error| {
                    Box::new(Standard::NoContext) as Box<dyn SignerError>
                })?;

        // Check if the signer supports EVM
        if !unified.supports_evm() {
            return Err(Box::new(Standard::UnsupportedOperation(
                "Current signer does not support EVM operations".to_owned(),
            )) as Box<dyn SignerError>);
        }

        Ok(EvmSignerHandle(unified))
    }

    /// Get the current signer as a Solana signer with type-safe access.
    ///
    /// This method provides a strongly-typed handle to the current signer's Solana
    /// capabilities. The returned handle implements `Deref` to `dyn SolanaSigner`,
    /// allowing direct access to all Solana-specific methods.
    ///
    /// # Returns
    /// * `Ok(SolanaSignerHandle)` - A handle providing type-safe access to Solana operations
    /// * `Err(Standard::NoContext)` - If called outside a signer context
    /// * `Err(Standard::UnsupportedOperation)` - If the current signer doesn't support Solana
    ///
    /// # Errors
    /// This function will return an error if:
    /// * It is called outside a signer context
    /// * The current signer doesn't support Solana operations
    ///
    /// # Thread Safety
    ///
    /// The returned handle is thread-safe and can be passed between async tasks.
    /// The underlying signer is immutable and reference-counted.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::signer::SignerContext;
    ///
    /// async fn solana_only_operation() -> Result<(), Box<dyn std::error::SignerError + Send + Sync>> {
    ///     // This will fail if the current signer doesn't support Solana
    ///     let signer = SignerContext::current_as_solana()?;
    ///
    ///     // Now we have type-safe access to Solana methods
    ///     let pubkey = signer.pubkey();
    ///     let client = signer.client();
    ///
    ///     // Perform Solana-specific operations
    ///     let balance = client.get_balance(&pubkey.parse()?).await?;
    ///     println!("Balance: {} SOL", balance);
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn current_as_solana() -> Result<SolanaSignerHandle, Box<dyn SignerError>> {
        let unified =
            CURRENT_UNIFIED_SIGNER
                .try_with(Clone::clone)
                .map_err(|_ignored_access_error| {
                    Box::new(Standard::NoContext) as Box<dyn SignerError>
                })?;

        // Check if the signer supports Solana
        if !unified.supports_solana() {
            return Err(Box::new(Standard::UnsupportedOperation(
                "Current signer does not support Solana operations".to_owned(),
            )) as Box<dyn SignerError>);
        }

        Ok(SolanaSignerHandle(unified))
    }

    /// Check if there is currently a signer context available.
    ///
    /// This function returns `true` if the current async task is running within
    /// a [`Context::with_signer()`] scope, and `false` otherwise.
    ///
    /// This is useful for tools that want to provide different behavior when called
    /// with or without a signer context, such as read-only vs. transactional operations.
    ///
    /// # Returns
    /// * `true` - If a signer context is available
    /// * `false` - If no signer context is available
    ///
    /// # Performance
    ///
    /// This function is very lightweight and can be called frequently without
    /// performance concerns. It simply checks if the thread-local storage
    /// contains a signer reference.
    #[inline]
    #[must_use]
    pub fn is_available() -> bool {
        CURRENT_UNIFIED_SIGNER.try_with(|_| ()).is_ok()
    }

    /// Execute a future with a unified signer context.
    ///
    /// This is the primary method for setting up a signer context. It uses the new
    /// `UnifiedSigner` trait which provides better type safety and chain-specific access.
    ///
    /// # Arguments
    /// * `signer` - The unified signer to make available in the context
    /// * `future` - The async code to execute with the signer context
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The future fails to execute
    /// * There are issues with setting up or maintaining the signer context
    ///
    /// # Examples
    /// ```ignore
    /// use riglr_core::signer::{SignerContext, UnifiedSigner};
    /// use riglr_solana_tools::LocalSolanaSigner;
    /// use std::sync::Arc;
    ///
    /// let signer: Arc<dyn UnifiedSigner> = Arc::new(LocalSolanaSigner::new(
    ///     keypair,
    ///     "https://api.devnet.solana.com".to_string()
    /// ));
    ///
    /// let result = SignerContext::with_signer(signer, async {
    ///     // Access as Solana signer
    ///     let solana = SignerContext::current_as_solana()?;
    ///     let pubkey = solana.pubkey();
    ///     Ok(pubkey)
    /// }).await?;
    /// ```
    #[inline]
    pub async fn with_signer<T, F>(
        signer: Arc<dyn UnifiedSigner>,
        future: F,
    ) -> Result<T, Box<dyn Error>>
    where
        F: Future<Output = Result<T, Box<dyn Error>>> + Send,
    {
        CURRENT_UNIFIED_SIGNER
            .scope(signer, future)
            .await
            .map_err(|e| Box::new(Standard::Generic(e.to_string())) as Box<dyn Error>)
    }
}

// Type alias for backwards compatibility
#[expect(clippy::module_name_repetitions)]
pub type SignerContext = Context;

#[cfg(test)]
mod tests {
    // Tests removed as they depend on blockchain SDKs
    // The concrete implementations and their tests are now in the tools crates
    // (riglr-solana-tools and riglr-evm-tools)
    //
    // Tests for SignerContext and UnifiedSigner functionality have been moved
    // to integration tests in the respective tools crates where concrete
    // implementations are available.
}
