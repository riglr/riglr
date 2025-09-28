// riglr-core/src/spawn.rs

use crate::signer::error::Error as SignerError;
#[cfg(test)]
use crate::signer::error::Standard;
use crate::SignerContext;
use core::future::Future;
use tokio::task::{spawn, JoinHandle};

/// Spawns a new task while preserving `SignerContext` if available.
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
///         let current = SignerContext::current()?; // ERROR
///         transfer_sol("recipient", 1.0).await
///     });
///     handle.await?
/// }).await
/// ```
///
/// The correct approach using `with_context`:
///
/// ```rust,ignore
/// // CORRECT - This properly propagates the SignerContext
/// SignerContext::with_signer(signer, async {
///     let handle = with_context(async {
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
/// 1. Checks if a `SignerContext` exists in the current task
/// 2. If yes, captures it and wraps the spawned future with `SignerContext::with_signer`
/// 3. If no, spawns the task normally without context
///
/// This ensures that tools requiring signing operations work correctly when
/// executed through agent frameworks that use task spawning for parallelism.
///
/// The future passed in should return a Result<T, `SignerError`>.
#[inline]
pub fn with_context<F, T>(future: F) -> JoinHandle<Result<T, Box<dyn SignerError>>>
where
    F: Future<Output = Result<T, Box<dyn SignerError>>> + Send + 'static,
    T: Send + 'static,
{
    // Try to get the current signer from the context
    let current_signer = SignerContext::current();
    if let Ok(signer) = current_signer {
        // We have a signer context - propagate it to the spawned task
        spawn(async move { SignerContext::with_signer(signer, future).await })
    } else {
        // No signer context - spawn normally
        spawn(future)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::error::Error;

    #[tokio::test]
    async fn spawn_with_context_without_signer() -> Result<(), Box<dyn Error>> {
        // No signer context set - this should work fine
        let handle = with_context(async move {
            // Should not have a signer
            match SignerContext::current() {
                Ok(_) => Err(Box::new(Standard::NoContext) as Box<dyn SignerError>),
                Err(_) => Ok(true),
            }
        });

        let result = handle
            .await
            .map_err(|_ignored_join_error| "Failed to complete spawn_with_context task")?
            .map_err(|_ignored_task_error| {
                "Task should return success when no signer context is set"
            })?;

        assert!(result, "No SignerContext should be available when not set");
        Ok(())
    }

    #[tokio::test]
    async fn spawn_with_context_with_signer() -> Result<(), Box<dyn Error>> {
        use crate::signer::{Chain, EvmSigner, SignerBase, SolanaSigner, UnifiedSigner};
        use alloc::sync::Arc;

        // Create a mock signer implementation
        #[derive(Debug)]
        struct MockSigner;

        impl SignerBase for MockSigner {
            fn supported_chains(&self) -> &[Chain] {
                &[]
            }

            fn supports_chain(&self, _chain: Chain) -> bool {
                false
            }

            fn user_id(&self) -> String {
                "mock-user".to_owned()
            }
        }

        impl UnifiedSigner for MockSigner {
            fn as_evm(&self) -> Option<&dyn EvmSigner> {
                None
            }

            fn as_solana(&self) -> Option<&dyn SolanaSigner> {
                None
            }

            fn supports_evm(&self) -> bool {
                false
            }

            fn supports_solana(&self) -> bool {
                false
            }
        }

        let mock_signer: Arc<dyn UnifiedSigner> = Arc::new(MockSigner);
        let signer_id = format!("{mock_signer:?}");

        // Set the signer context and spawn a task within it
        let result = SignerContext::with_signer(Arc::clone(&mock_signer), async move {
            // Spawn a task that should have access to the signer
            let handle = with_context(async move {
                // Inside the spawned task, verify we can access the signer
                let current_signer = SignerContext::current()?;
                let current_id = format!("{current_signer:?}");

                // Return true if the signer matches
                if current_id == signer_id {
                    return Ok(true);
                }
                Ok(false)
            });

            // Await the spawned task and propagate its result
            return handle.await.map_err(|_ignored_join_error| {
                Box::new(Standard::NoContext) as Box<dyn SignerError>
            })?;
        })
        .await;

        // Test assertions - appropriate for test code
        assert!(
            result.is_ok(),
            "Should successfully get result from spawned task"
        );
        let success = result.map_err(|e| e.to_string())?;
        assert!(
            success,
            "SignerContext should be propagated to spawned task"
        );
        Ok(())
    }
}
