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
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
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

use std::sync::Arc;
use tokio::task_local;

pub mod error;
pub mod granular_traits;
pub mod traits;

pub use error::SignerError;
pub use granular_traits::{
    Chain, EvmSigner, MultiChainSigner, SignerBase, SolanaSigner, UnifiedSigner,
};
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
    /// // Concrete signers from tools crates:
    /// // use riglr_solana_tools::LocalSolanaSigner;
    /// use std::sync::Arc;
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
    // Tests removed as they depend on blockchain SDKs
    // The concrete implementations and their tests are now in the tools crates
    // (riglr-solana-tools and riglr-evm-tools)
    //
    // Tests for SignerContext and UnifiedSigner functionality have been moved
    // to integration tests in the respective tools crates where concrete
    // implementations are available.
}
