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
//! use riglr_core::signer::{SignerContext, LocalSolanaSigner, LocalEvmSigner};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configuration from environment
//! let config = Config::from_env()?;
//!
//! // Create signers with proper network configuration
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

use std::sync::Arc;
use tokio::task_local;

pub mod error;
pub mod evm;
pub mod granular_traits;
pub mod solana;
pub mod traits;

pub use error::SignerError;
pub use evm::LocalEvmSigner;
pub use granular_traits::{
    Chain, EvmSigner, MultiChainSigner, SignerBase, SolanaSigner, UnifiedSigner,
};
pub use solana::LocalSolanaSigner;
pub use traits::{EvmClient, SolanaClient};

// Thread-local storage for current signer context
// This provides secure isolation between different async tasks/requests
task_local! {
    static CURRENT_UNIFIED_SIGNER: Arc<dyn UnifiedSigner>;
}

/// The SignerContext provides thread-local signer management for secure multi-tenant operation.
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
/// use riglr_solana_tools::LocalSolanaSigner;
/// use std::sync::Arc;
/// # use solana_sdk::signer::keypair::Keypair;
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
///     let current = SignerContext::current().await?;
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
/// ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
///     let signer = SignerContext::current().await?;
///     Ok(format!("Balance for user: {:?}", signer.user_id()))
/// }
///
/// async fn perform_transfer() -> Result<String, riglr_core::signer::SignerError> {
///     let signer = SignerContext::current().await?;
///     // Perform actual transfer using signer...
///     Ok("Transfer completed".to_string())
/// }
/// ```
///
/// ### Error Handling
///
/// Tools should always check for signer availability:
///
/// ```rust
/// use riglr_core::signer::SignerContext;
///
/// async fn safe_operation() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
///     if !SignerContext::is_available().await {
///         return Err("This operation requires a signer context".into());
///     }
///
///     let signer = SignerContext::current().await?;
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
pub struct SignerContext;

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
/// async fn solana_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     // Get a type-safe handle to the Solana signer
///     let signer = SignerContext::current_as_solana().await?;
///     
///     // Access Solana-specific methods directly
///     let pubkey = signer.pubkey();
///     let signature = signer.sign_message(b"Hello").await?;
///     
///     Ok(())
/// }
/// ```
pub struct SolanaSignerHandle(Arc<dyn UnifiedSigner>);

impl std::ops::Deref for SolanaSignerHandle {
    type Target = dyn SolanaSigner;

    fn deref(&self) -> &Self::Target {
        self.0
            .as_solana()
            .expect("Signer must support Solana. This is a bug in SignerContext.")
    }
}

/// A handle to the current EVM signer, providing type-safe access.
///
/// This handle guarantees that the underlying signer supports EVM operations.
/// It implements `Deref` to `dyn EvmSigner`, allowing direct access to all
/// EVM-specific methods.
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
/// use alloy::rpc::types::TransactionRequest;
///
/// async fn evm_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     // Get a type-safe handle to the EVM signer
///     let signer = SignerContext::current_as_evm().await?;
///     
///     // Access EVM-specific methods directly
///     let address = signer.address();
///     let chain_id = signer.chain_id();
///     
///     // Sign and send a transaction
///     let tx = TransactionRequest::default();
///     let tx_hash = signer.sign_and_send_transaction(tx).await?;
///     
///     Ok(())
/// }
/// ```
pub struct EvmSignerHandle(Arc<dyn UnifiedSigner>);

impl std::ops::Deref for EvmSignerHandle {
    type Target = dyn EvmSigner;

    fn deref(&self) -> &Self::Target {
        self.0
            .as_evm()
            .expect("Signer must support EVM. This is a bug in SignerContext.")
    }
}

impl SignerContext {
    /// Execute a future with a unified signer context.
    ///
    /// This is the primary method for setting up a signer context. It uses the new
    /// UnifiedSigner trait which provides better type safety and chain-specific access.
    ///
    /// # Arguments
    /// * `signer` - The unified signer to make available in the context
    /// * `future` - The async code to execute with the signer context
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
    ///     let solana = SignerContext::current_as_solana().await?;
    ///     let pubkey = solana.pubkey();
    ///     Ok(pubkey)
    /// }).await?;
    /// ```
    pub async fn with_signer<T, F>(
        signer: Arc<dyn UnifiedSigner>,
        future: F,
    ) -> Result<T, SignerError>
    where
        F: std::future::Future<Output = Result<T, SignerError>> + Send,
    {
        CURRENT_UNIFIED_SIGNER.scope(signer, future).await
    }

    /// Get the current unified signer from thread-local context.
    ///
    /// This function retrieves the signer that was set by [`SignerContext::with_signer()`].
    /// Returns the UnifiedSigner which can be cast to specific signer types.
    ///
    /// # Returns
    /// * `Ok(Arc<dyn UnifiedSigner>)` - The current signer if available
    /// * `Err(SignerError::NoSignerContext)` - If called outside a signer context
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::signer::SignerContext;
    ///
    /// async fn tool_that_needs_signer() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    ///     // Get the current unified signer
    ///     let signer = SignerContext::current().await?;
    ///     
    ///     // Check what chains it supports
    ///     if signer.supports_solana() {
    ///         let solana = SignerContext::current_as_solana().await?;
    ///         return Ok(format!("Solana pubkey: {}", solana.pubkey()));
    ///     }
    ///     
    ///     if signer.supports_evm() {
    ///         let evm = SignerContext::current_as_evm().await?;
    ///         return Ok(format!("EVM address: {}", evm.address()));
    ///     }
    ///     
    ///     Ok("Unknown signer type".to_string())
    /// }
    /// ```
    pub async fn current() -> Result<Arc<dyn UnifiedSigner>, SignerError> {
        CURRENT_UNIFIED_SIGNER
            .try_with(|signer| signer.clone())
            .map_err(|_| SignerError::NoSignerContext)
    }

    /// Check if there is currently a signer context available.
    ///
    /// This function returns `true` if the current async task is running within
    /// a [`SignerContext::with_signer()`] scope, and `false` otherwise.
    ///
    /// This is useful for tools that want to provide different behavior when called
    /// with or without a signer context, such as read-only vs. transactional operations.
    ///
    /// # Returns
    /// * `true` - If a signer context is available
    /// * `false` - If no signer context is available
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::signer::SignerContext;
    /// use riglr_solana_tools::LocalSolanaSigner;
    /// use std::sync::Arc;
    /// # use solana_sdk::signer::keypair::Keypair;
    ///
    /// async fn flexible_tool() -> Result<String, riglr_core::signer::SignerError> {
    ///     if SignerContext::is_available().await {
    ///         // We have a signer, can perform transactions
    ///         let signer = SignerContext::current().await?;
    ///         Ok("Performing transaction with signer".to_string())
    ///     } else {
    ///         // No signer available, provide read-only functionality
    ///         Ok("Read-only mode: no signer available".to_string())
    ///     }
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// // Test without signer context
    /// let result1 = flexible_tool().await.unwrap();
    /// assert_eq!(result1, "Read-only mode: no signer available");
    ///
    /// // Test with signer context
    /// let keypair = Keypair::new();
    /// let signer = Arc::new(LocalSolanaSigner::new(
    ///     keypair,
    ///     "https://api.devnet.solana.com".to_string()
    /// ));
    ///
    /// let result2 = SignerContext::with_signer(signer, async {
    ///     flexible_tool().await
    /// }).await.unwrap();
    /// assert_eq!(result2, "Performing transaction with signer");
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance
    ///
    /// This function is very lightweight and can be called frequently without
    /// performance concerns. It simply checks if the thread-local storage
    /// contains a signer reference.
    pub async fn is_available() -> bool {
        CURRENT_UNIFIED_SIGNER.try_with(|_| ()).is_ok()
    }

    /// Get the current signer as a specific type.
    ///
    /// This method allows type-safe access to chain-specific signer capabilities.
    /// Tools can require specific signer types and get compile-time guarantees.
    ///
    /// # Type Parameters
    /// * `T` - The specific signer trait to cast to (e.g., `dyn SolanaSigner`, `dyn EvmSigner`)
    ///
    /// # Returns
    /// * `Ok(&T)` - Reference to the signer with the requested capabilities
    /// * `Err(SignerError::UnsupportedOperation)` - If the current signer doesn't support the requested type
    /// * `Err(SignerError::NoSignerContext)` - If no signer context is available
    ///
    /// # Examples
    /// ```ignore
    /// use riglr_core::signer::{SignerContext, SolanaSigner, EvmSigner};
    ///
    /// // Require Solana signer
    /// async fn solana_operation() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    ///     let signer = SignerContext::current_as::<dyn SolanaSigner>().await?;
    ///     Ok(format!("Solana pubkey: {}", signer.pubkey()))
    /// }
    ///
    /// // Require EVM signer
    /// async fn evm_operation() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    ///     let signer = SignerContext::current_as::<dyn EvmSigner>().await?;
    ///     Ok(format!("EVM chain: {}", signer.chain_id()))
    /// }
    /// ```
    ///
    /// Get the current signer as a Solana signer with type-safe access.
    ///
    /// This method provides a strongly-typed handle to the current signer's Solana
    /// capabilities. The returned handle implements `Deref` to `dyn SolanaSigner`,
    /// allowing direct access to all Solana-specific methods.
    ///
    /// # Returns
    /// * `Ok(SolanaSignerHandle)` - A handle providing type-safe access to Solana operations
    /// * `Err(SignerError::NoSignerContext)` - If called outside a signer context
    /// * `Err(SignerError::UnsupportedOperation)` - If the current signer doesn't support Solana
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
    /// use solana_sdk::signature::Keypair;
    ///
    /// async fn solana_only_operation() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     // This will fail if the current signer doesn't support Solana
    ///     let signer = SignerContext::current_as_solana().await?;
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
    pub async fn current_as_solana() -> Result<SolanaSignerHandle, SignerError> {
        let unified = CURRENT_UNIFIED_SIGNER
            .try_with(|s| s.clone())
            .map_err(|_| SignerError::NoSignerContext)?;

        // Check if the signer supports Solana
        if !unified.supports_solana() {
            return Err(SignerError::UnsupportedOperation(
                "Current signer does not support Solana operations".to_string(),
            ));
        }

        Ok(SolanaSignerHandle(unified))
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
    /// # Thread Safety
    ///
    /// The returned handle is thread-safe and can be passed between async tasks.
    /// The underlying signer is immutable and reference-counted.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::signer::SignerContext;
    /// use alloy::primitives::{Address, U256};
    /// use alloy::rpc::types::TransactionRequest;
    ///
    /// async fn evm_only_operation() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     // This will fail if the current signer doesn't support EVM
    ///     let signer = SignerContext::current_as_evm().await?;
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
    pub async fn current_as_evm() -> Result<EvmSignerHandle, SignerError> {
        let unified = CURRENT_UNIFIED_SIGNER
            .try_with(|s| s.clone())
            .map_err(|_| SignerError::NoSignerContext)?;

        // Check if the signer supports EVM
        if !unified.supports_evm() {
            return Err(SignerError::UnsupportedOperation(
                "Current signer does not support EVM operations".to_string(),
            ));
        }

        Ok(EvmSignerHandle(unified))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use tokio;

    // Mock signer for testing
    #[derive(Debug)]
    struct MockSigner {
        id: String,
    }

    impl SignerBase for MockSigner {
        fn locale(&self) -> String {
            "en".to_string()
        }

        fn user_id(&self) -> Option<String> {
            Some(self.id.clone())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[async_trait::async_trait]
    impl SolanaSigner for MockSigner {
        fn address(&self) -> String {
            format!("mock_solana_address_{}", self.id)
        }

        fn pubkey(&self) -> String {
            format!("mock_solana_pubkey_{}", self.id)
        }

        async fn sign_and_send_transaction(
            &self,
            _tx: &mut solana_sdk::transaction::Transaction,
        ) -> Result<String, SignerError> {
            Ok(format!("mock_solana_signature_{}", self.id))
        }

        fn client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
            Arc::new(solana_client::rpc_client::RpcClient::new(
                "http://localhost:8899",
            ))
        }
    }

    #[async_trait::async_trait]
    impl EvmSigner for MockSigner {
        fn chain_id(&self) -> u64 {
            1337
        }

        fn address(&self) -> String {
            format!("0xmock_evm_address_{}", self.id)
        }

        async fn sign_and_send_transaction(
            &self,
            _tx: alloy::rpc::types::TransactionRequest,
        ) -> Result<String, SignerError> {
            Ok(format!("mock_evm_signature_{}", self.id))
        }

        fn client(&self) -> Result<Arc<dyn EvmClient>, SignerError> {
            Err(SignerError::Configuration(
                "Mock EVM client not implemented".to_string(),
            ))
        }
    }

    impl UnifiedSigner for MockSigner {
        fn supports_solana(&self) -> bool {
            true
        }

        fn supports_evm(&self) -> bool {
            true
        }

        fn as_solana(&self) -> Option<&dyn SolanaSigner> {
            Some(self)
        }

        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            Some(self)
        }

        fn as_multi_chain(&self) -> Option<&dyn MultiChainSigner> {
            None
        }
    }

    #[tokio::test]
    async fn test_signer_context_isolation() {
        let signer1 = Arc::new(MockSigner {
            id: "user1".to_string(),
        });
        let signer2 = Arc::new(MockSigner {
            id: "user2".to_string(),
        });

        let task1 = SignerContext::with_signer(signer1.clone(), async {
            let current = SignerContext::current().await.unwrap();
            assert_eq!(current.user_id(), Some("user1".to_string()));
            Ok(())
        });

        let task2 = SignerContext::with_signer(signer2.clone(), async {
            let current = SignerContext::current().await.unwrap();
            assert_eq!(current.user_id(), Some("user2".to_string()));
            Ok(())
        });

        // Run both tasks concurrently to test isolation
        let (_result1, _result2) = tokio::join!(task1, task2);
    }

    #[tokio::test]
    async fn test_no_context_error() {
        let result = SignerContext::current().await;
        assert!(matches!(result, Err(SignerError::NoSignerContext)));
    }

    #[tokio::test]
    async fn test_context_availability() {
        // Outside context
        assert!(!SignerContext::is_available().await);

        let signer = Arc::new(MockSigner {
            id: "test".to_string(),
        });

        SignerContext::with_signer(signer, async {
            // Inside context
            assert!(SignerContext::is_available().await);
            Ok(())
        })
        .await
        .unwrap();

        // Outside context again
        assert!(!SignerContext::is_available().await);
    }

    // Mock unified signer for testing
    #[derive(Debug)]
    struct MockUnifiedSigner {
        id: String,
        supports_solana: bool,
        supports_evm: bool,
    }

    impl SignerBase for MockUnifiedSigner {
        fn user_id(&self) -> Option<String> {
            Some(self.id.clone())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl UnifiedSigner for MockUnifiedSigner {
        fn supports_solana(&self) -> bool {
            self.supports_solana
        }

        fn supports_evm(&self) -> bool {
            self.supports_evm
        }

        fn as_solana(&self) -> Option<&dyn SolanaSigner> {
            if self.supports_solana {
                Some(self)
            } else {
                None
            }
        }

        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            if self.supports_evm {
                Some(self)
            } else {
                None
            }
        }

        fn as_multi_chain(&self) -> Option<&dyn MultiChainSigner> {
            None
        }
    }

    #[async_trait::async_trait]
    impl SolanaSigner for MockUnifiedSigner {
        fn address(&self) -> String {
            solana_sdk::pubkey::Pubkey::new_unique().to_string()
        }

        fn pubkey(&self) -> String {
            solana_sdk::pubkey::Pubkey::new_unique().to_string()
        }

        async fn sign_and_send_transaction(
            &self,
            _tx: &mut solana_sdk::transaction::Transaction,
        ) -> Result<String, SignerError> {
            Ok("mock_solana_signature".to_string())
        }

        fn client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
            Arc::new(solana_client::rpc_client::RpcClient::new(
                "http://localhost:8899",
            ))
        }
    }

    #[async_trait::async_trait]
    impl EvmSigner for MockUnifiedSigner {
        fn chain_id(&self) -> u64 {
            1337
        }

        fn address(&self) -> String {
            "0x0000000000000000000000000000000000000000".to_string()
        }

        async fn sign_and_send_transaction(
            &self,
            _tx: alloy::rpc::types::TransactionRequest,
        ) -> Result<String, SignerError> {
            Ok("mock_evm_signature".to_string())
        }

        fn client(&self) -> Result<Arc<dyn super::traits::EvmClient>, SignerError> {
            Err(SignerError::Configuration(
                "Mock EVM client not implemented".to_string(),
            ))
        }
    }

    #[tokio::test]
    async fn test_with_unified_signer_success() {
        let unified_signer = Arc::new(MockUnifiedSigner {
            id: "unified_user".to_string(),
            supports_solana: true,
            supports_evm: false,
        });

        let result = SignerContext::with_signer(unified_signer, async {
            // Test that the context is available
            assert!(SignerContext::is_available().await);
            Ok("success")
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_with_unified_signer_error_propagation() {
        let unified_signer = Arc::new(MockUnifiedSigner {
            id: "error_user".to_string(),
            supports_solana: false,
            supports_evm: false,
        });

        let result: Result<(), SignerError> = SignerContext::with_signer(unified_signer, async {
            Err(SignerError::Configuration("Test error".to_string()))
        })
        .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(SignerError::Configuration(_))));
    }

    #[tokio::test]
    async fn test_current_as_solana_with_unified_signer() {
        let unified_signer = Arc::new(MockUnifiedSigner {
            id: "solana_user".to_string(),
            supports_solana: true,
            supports_evm: false,
        });

        let result = SignerContext::with_signer(unified_signer, async {
            let solana_result = SignerContext::current_as_solana().await;
            // With handles, this should now work
            assert!(solana_result.is_ok());
            let handle = solana_result.unwrap();
            // Access should work through Deref
            let pubkey = handle.pubkey();
            assert!(!pubkey.is_empty());
            Ok(())
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_current_as_solana_no_context() {
        let result = SignerContext::current_as_solana().await;
        assert!(matches!(result, Err(SignerError::NoSignerContext)));
    }

    #[tokio::test]
    async fn test_current_as_evm_with_unified_signer() {
        let unified_signer = Arc::new(MockUnifiedSigner {
            id: "evm_user".to_string(),
            supports_solana: false,
            supports_evm: true,
        });

        let result = SignerContext::with_signer(unified_signer, async {
            let evm_result = SignerContext::current_as_evm().await;
            // Should work now with handles
            assert!(evm_result.is_ok());
            let handle = evm_result.unwrap();
            let chain_id = handle.chain_id();
            assert_eq!(chain_id, 1337);
            Ok(())
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_current_as_evm_with_mock_signer() {
        let mock_signer = Arc::new(MockSigner {
            id: "mock_user".to_string(),
        });

        let result = SignerContext::with_signer(mock_signer, async {
            let evm_result = SignerContext::current_as_evm().await;
            // Should succeed because MockSigner supports EVM
            assert!(evm_result.is_ok());
            let handle = evm_result.unwrap();
            let chain_id = handle.chain_id();
            assert_eq!(chain_id, 1337);
            Ok(())
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_current_as_evm_no_context() {
        let result = SignerContext::current_as_evm().await;
        assert!(matches!(result, Err(SignerError::NoSignerContext)));
    }

    #[tokio::test]
    async fn test_is_available_with_unified_signer() {
        // Test that is_available returns true for unified signer context
        let unified_signer = Arc::new(MockUnifiedSigner {
            id: "unified_test".to_string(),
            supports_solana: true,
            supports_evm: true,
        });

        assert!(!SignerContext::is_available().await);

        SignerContext::with_signer(unified_signer, async {
            assert!(SignerContext::is_available().await);
            Ok(())
        })
        .await
        .unwrap();

        assert!(!SignerContext::is_available().await);
    }

    #[tokio::test]
    async fn test_with_signer_error_propagation() {
        let signer = Arc::new(MockSigner {
            id: "error_test".to_string(),
        });

        let result: Result<(), SignerError> = SignerContext::with_signer(signer, async {
            Err(SignerError::Configuration("Propagated error".to_string()))
        })
        .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(SignerError::Configuration(_))));
    }

    #[tokio::test]
    async fn test_current_as_solana_unified_no_solana_support() {
        let unified_signer = Arc::new(MockUnifiedSigner {
            id: "no_solana".to_string(),
            supports_solana: false,
            supports_evm: true,
        });

        let result = SignerContext::with_signer(unified_signer, async {
            let solana_result = SignerContext::current_as_solana().await;
            // Should return UnsupportedOperation because unified signer doesn't support Solana
            assert!(solana_result.is_err());
            assert!(matches!(
                solana_result,
                Err(SignerError::UnsupportedOperation(_))
            ));
            Ok(())
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_current_as_evm_unified_no_evm_support() {
        let unified_signer = Arc::new(MockUnifiedSigner {
            id: "no_evm".to_string(),
            supports_solana: true,
            supports_evm: false,
        });

        let result = SignerContext::with_signer(unified_signer, async {
            let evm_result = SignerContext::current_as_evm().await;
            // Should return UnsupportedOperation because unified signer doesn't support EVM
            assert!(evm_result.is_err());
            assert!(matches!(
                evm_result,
                Err(SignerError::UnsupportedOperation(_))
            ));
            Ok(())
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_signer_methods() {
        let signer = MockSigner {
            id: "test_methods".to_string(),
        };

        // Test user_id method
        assert_eq!(signer.user_id(), Some("test_methods".to_string()));

        // Test SolanaSigner trait methods
        let mut tx = solana_sdk::transaction::Transaction::default();
        let result = SolanaSigner::sign_and_send_transaction(&signer, &mut tx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mock_solana_signature_test_methods");

        // Test EvmSigner trait methods
        let tx_request = alloy::rpc::types::TransactionRequest::default();
        let result = EvmSigner::sign_and_send_transaction(&signer, tx_request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mock_evm_signature_test_methods");

        // Test SolanaSigner client method
        let client = SolanaSigner::client(&signer);
        assert!(std::sync::Arc::strong_count(&client) > 0);

        // Test EvmSigner client method
        let evm_result = EvmSigner::client(&signer);
        assert!(evm_result.is_err());
        assert!(matches!(evm_result, Err(SignerError::Configuration(_))));
    }

    #[tokio::test]
    async fn test_nested_signer_contexts() {
        let signer1 = Arc::new(MockSigner {
            id: "outer".to_string(),
        });
        let signer2 = Arc::new(MockSigner {
            id: "inner".to_string(),
        });

        let result = SignerContext::with_signer(signer1, async {
            let outer_signer = SignerContext::current().await.unwrap();
            assert_eq!(outer_signer.user_id(), Some("outer".to_string()));

            // Nested context should override the outer one
            SignerContext::with_signer(signer2, async {
                let inner_signer = SignerContext::current().await.unwrap();
                assert_eq!(inner_signer.user_id(), Some("inner".to_string()));
                Ok(())
            })
            .await?;

            // After inner context ends, outer should be restored
            let restored_signer = SignerContext::current().await.unwrap();
            assert_eq!(restored_signer.user_id(), Some("outer".to_string()));
            Ok(())
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_concurrent_unified_signers() {
        let unified1 = Arc::new(MockUnifiedSigner {
            id: "unified1".to_string(),
            supports_solana: true,
            supports_evm: false,
        });
        let unified2 = Arc::new(MockUnifiedSigner {
            id: "unified2".to_string(),
            supports_solana: false,
            supports_evm: true,
        });

        let task1 = SignerContext::with_signer(unified1, async {
            assert!(SignerContext::is_available().await);
            Ok("task1")
        });

        let task2 = SignerContext::with_signer(unified2, async {
            assert!(SignerContext::is_available().await);
            Ok("task2")
        });

        let (result1, result2) = tokio::join!(task1, task2);
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), "task1");
        assert_eq!(result2.unwrap(), "task2");
    }
}
