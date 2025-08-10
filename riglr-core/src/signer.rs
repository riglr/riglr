use std::sync::Arc;
use tokio::task_local;

pub mod traits;
pub mod local;
pub mod error;

pub use traits::TransactionSigner;
pub use local::LocalSolanaSigner;
pub use error::SignerError;


// Thread-local storage for current signer context
// This provides secure isolation between different async tasks/requests
task_local! {
    static CURRENT_SIGNER: Arc<dyn TransactionSigner>;
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
/// ```rust
/// use riglr_core::signer::{SignerContext, LocalSolanaSigner};
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
/// ```rust
/// use riglr_core::signer::{SignerContext, TransactionSigner};
/// use std::sync::Arc;
/// 
/// async fn handle_user_request(
///     user_signer: Arc<dyn TransactionSigner>,
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

impl SignerContext {
    /// Execute a future with a specific signer context.
    /// 
    /// This creates an isolated context where the provided signer is available
    /// to all code running within the future via [`SignerContext::current()`].
    /// The context is automatically cleaned up when the future completes.
    /// 
    /// # Arguments
    /// * `signer` - The signer to make available in the context. This signer will be
    ///              accessible to all code executed within the future scope.
    /// * `future` - The async code to execute with the signer context. All async
    ///              operations within this future can access the signer.
    /// 
    /// # Returns
    /// The result of executing the future. If the future succeeds, returns its result.
    /// If the future fails with a [`SignerError`], that error is propagated.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use riglr_core::signer::{SignerContext, LocalSolanaSigner};
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
    /// let result = SignerContext::with_signer(signer, async {
    ///     // Multiple operations can access the same signer
    ///     let current = SignerContext::current().await?;
    ///     let user_id = current.user_id();
    ///     
    ///     // Call other async functions that need the signer
    ///     perform_blockchain_operation().await?;
    ///     
    ///     Ok(format!("Completed operations for user: {:?}", user_id))
    /// }).await?;
    /// 
    /// println!("{}", result);
    /// # Ok(())
    /// # }
    /// 
    /// async fn perform_blockchain_operation() -> Result<(), riglr_core::signer::SignerError> {
    ///     // This function can access the signer context
    ///     let signer = SignerContext::current().await?;
    ///     // ... perform blockchain operations
    ///     Ok(())
    /// }
    /// ```
    /// 
    /// # Thread Safety
    /// 
    /// Multiple tasks can run concurrently with different signer contexts:
    /// 
    /// ```rust
    /// use riglr_core::signer::{SignerContext, LocalSolanaSigner};
    /// use std::sync::Arc;
    /// # use solana_sdk::signer::keypair::Keypair;
    /// 
    /// # async fn concurrent_example() -> Result<(), riglr_core::signer::SignerError> {
    /// let signer1 = Arc::new(LocalSolanaSigner::new(
    ///     Keypair::new(), 
    ///     "https://api.devnet.solana.com".to_string()
    /// ));
    /// let signer2 = Arc::new(LocalSolanaSigner::new(
    ///     Keypair::new(),
    ///     "https://api.devnet.solana.com".to_string()
    /// ));
    /// 
    /// let (result1, result2) = tokio::join!(
    ///     SignerContext::with_signer(signer1, async {
    ///         // This task has access to signer1
    ///         Ok("Task 1 completed")
    ///     }),
    ///     SignerContext::with_signer(signer2, async {
    ///         // This task has access to signer2 (isolated from signer1)
    ///         Ok("Task 2 completed")
    ///     })
    /// );
    /// 
    /// println!("{}, {}", result1?, result2?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_signer<T, F>(
        signer: Arc<dyn TransactionSigner>,
        future: F,
    ) -> Result<T, SignerError>
    where
        F: std::future::Future<Output = Result<T, SignerError>> + Send,
    {
        CURRENT_SIGNER.scope(signer, future).await
    }
    
    /// Get the current signer from thread-local context.
    /// 
    /// This function retrieves the signer that was set by [`SignerContext::with_signer()`].
    /// It must be called within a signer context scope, otherwise it will return
    /// [`SignerError::NoSignerContext`].
    /// 
    /// # Returns
    /// * `Ok(Arc<dyn TransactionSigner>)` - The current signer if available
    /// * `Err(SignerError::NoSignerContext)` - If called outside a signer context
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use riglr_core::signer::{SignerContext, LocalSolanaSigner};
    /// use std::sync::Arc;
    /// # use solana_sdk::signer::keypair::Keypair;
    /// 
    /// async fn tool_that_needs_signer() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    ///     // Get the current signer from context
    ///     let signer = SignerContext::current().await
    ///         .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
    ///     
    ///     // Use the signer for operations
    ///     match signer.user_id() {
    ///         Some(user) => Ok(format!("Operating for user: {}", user)),
    ///         None => Ok("Operating for anonymous user".to_string()),
    ///     }
    /// }
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// let keypair = Keypair::new();
    /// let signer = Arc::new(LocalSolanaSigner::new(
    ///     keypair,
    ///     "https://api.devnet.solana.com".to_string()
    /// ));
    /// 
    /// SignerContext::with_signer(signer, async {
    ///     let result = tool_that_needs_signer().await.unwrap();
    ///     println!("{}", result);
    ///     Ok(())
    /// }).await.unwrap();
    /// # Ok(())
    /// # }
    /// ```
    /// 
    /// # Error Handling
    /// 
    /// This function will return an error if called outside a signer context:
    /// 
    /// ```rust
    /// use riglr_core::signer::{SignerContext, SignerError};
    /// 
    /// # async fn error_example() {
    /// // This will fail because we're not in a signer context
    /// let result = SignerContext::current().await;
    /// assert!(matches!(result, Err(SignerError::NoSignerContext)));
    /// # }
    /// ```
    pub async fn current() -> Result<Arc<dyn TransactionSigner>, SignerError> {
        CURRENT_SIGNER.try_with(|signer| signer.clone())
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
    /// ```rust
    /// use riglr_core::signer::{SignerContext, LocalSolanaSigner};
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
        CURRENT_SIGNER.try_with(|_| ()).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    
    // Mock signer for testing
    #[derive(Debug)]
    struct MockSigner {
        id: String,
    }
    
    #[async_trait::async_trait]
    impl TransactionSigner for MockSigner {
        fn user_id(&self) -> Option<String> {
            Some(self.id.clone())
        }
        
        async fn sign_and_send_solana_transaction(
            &self,
            _tx: &mut solana_sdk::transaction::Transaction,
        ) -> Result<String, SignerError> {
            Ok(format!("mock_signature_{}", self.id))
        }
        
        async fn sign_and_send_evm_transaction(
            &self,
            _tx: alloy::rpc::types::TransactionRequest,
        ) -> Result<String, SignerError> {
            Ok(format!("mock_evm_signature_{}", self.id))
        }
        
        fn solana_client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
            Arc::new(solana_client::rpc_client::RpcClient::new("http://localhost:8899"))
        }
        
        fn evm_client(&self) -> Result<Box<dyn std::any::Any + Send + Sync>, SignerError> {
            Err(SignerError::Configuration("Mock EVM client not implemented".to_string()))
        }
    }
    
    #[tokio::test]
    async fn test_signer_context_isolation() {
        let signer1 = Arc::new(MockSigner { id: "user1".to_string() });
        let signer2 = Arc::new(MockSigner { id: "user2".to_string() });
        
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
        
        let signer = Arc::new(MockSigner { id: "test".to_string() });
        
        SignerContext::with_signer(signer, async {
            // Inside context
            assert!(SignerContext::is_available().await);
            Ok(())
        }).await.unwrap();
        
        // Outside context again
        assert!(!SignerContext::is_available().await);
    }
}