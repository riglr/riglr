// riglr-core/src/spawn.rs

use crate::SignerContext;
use std::future::Future;
use tokio::task::JoinHandle;

/// Spawns a new task while preserving SignerContext if available.
///
/// # Why This Is Necessary
///
/// The `SignerContext` is stored in `tokio::task_local!` storage, which means it is
/// **NOT** automatically propagated to new tasks spawned with `tokio::spawn`. This is
/// a fundamental limitation of task-local storage - it's only accessible within the
/// same task where it was set.
///
/// Without this function, the following would fail:
///
/// ```rust,ignore
/// // WRONG - This will fail with "No signer context"
/// SignerContext::with_signer(signer, async {
///     let handle = tokio::spawn(async {
///         // This will fail - SignerContext is not available here!
///         let current = SignerContext::current().await?; // ERROR
///         transfer_sol("recipient", 1.0).await
///     });
///     handle.await?
/// }).await
/// ```
///
/// The correct approach using `spawn_with_context`:
///
/// ```rust,ignore
/// // CORRECT - This properly propagates the SignerContext
/// SignerContext::with_signer(signer, async {
///     let handle = spawn_with_context(async {
///         // SignerContext is available here!
///         transfer_sol("recipient", 1.0).await
///     }).await;
///     handle.await?
/// }).await
/// ```
///
/// # How It Works
///
/// This function:
/// 1. Checks if a SignerContext exists in the current task
/// 2. If yes, captures it and wraps the spawned future with `SignerContext::with_signer`
/// 3. If no, spawns the task normally without context
///
/// This ensures that tools requiring signing operations work correctly when
/// executed through agent frameworks that use task spawning for parallelism.
///
/// Note: This function is async because it needs to check for the current signer.
/// The future passed in should return a Result<T, SignerError>.
pub async fn spawn_with_context<F, T>(future: F) -> JoinHandle<Result<T, crate::SignerError>>
where
    F: Future<Output = Result<T, crate::SignerError>> + Send + 'static,
    T: Send + 'static,
{
    // Try to get the current signer from the context
    let current_signer = SignerContext::current().await;
    if let Ok(signer) = current_signer {
        // We have a signer context - propagate it to the spawned task
        tokio::task::spawn(async move { SignerContext::with_signer(signer, future).await })
    } else {
        // No signer context - spawn normally
        tokio::task::spawn(future)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_with_context_without_signer() {
        // No signer context set - this should work fine
        let handle = spawn_with_context(async move {
            // Should not have a signer
            match SignerContext::current().await {
                Ok(_) => Err(crate::SignerError::NoSignerContext),
                Err(_) => Ok(true),
            }
        })
        .await;

        let result = handle.await.unwrap().unwrap();
        assert!(result, "No SignerContext should be available when not set");
    }

    #[tokio::test]
    async fn test_spawn_with_context_with_signer() {
        use crate::signer::granular_traits::{SignerBase, UnifiedSigner};
        use std::{any::Any, sync::Arc};

        // Create a mock signer implementation
        #[derive(Debug)]
        struct MockSigner;

        impl SignerBase for MockSigner {
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
            fn as_solana(&self) -> Option<&dyn crate::signer::granular_traits::SolanaSigner> {
                None
            }
            fn as_evm(&self) -> Option<&dyn crate::signer::granular_traits::EvmSigner> {
                None
            }
            fn as_multi_chain(
                &self,
            ) -> Option<&dyn crate::signer::granular_traits::MultiChainSigner> {
                None
            }
        }

        let mock_signer: Arc<dyn UnifiedSigner> = Arc::new(MockSigner);
        let signer_id = format!("{:?}", mock_signer);

        // Set the signer context and spawn a task within it
        let result = SignerContext::with_signer(mock_signer.clone(), async move {
            // Spawn a task that should have access to the signer
            let handle = spawn_with_context(async move {
                // Inside the spawned task, verify we can access the signer
                let current_signer = SignerContext::current().await?;
                let current_id = format!("{:?}", current_signer);

                // Return true if the signer matches
                if current_id == signer_id {
                    Ok(true)
                } else {
                    Ok(false)
                }
            })
            .await;

            // Await the spawned task and propagate its result
            handle.await.unwrap()
        })
        .await;

        assert!(
            result.is_ok(),
            "Should successfully get result from spawned task"
        );
        assert!(
            result.unwrap(),
            "SignerContext should be propagated to spawned task"
        );
    }
}
