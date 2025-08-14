//! Integration tests for signer context system
//!
//! Tests the complete signer context lifecycle, thread safety,
//! and integration with tool execution workflows.

use riglr_core::signer::{SignerContext, TransactionSigner, SignerError};
use riglr_core::jobs::{Job, JobResult};
use riglr_core::tool::{Tool, ToolWorker, ExecutionConfig};
use riglr_core::idempotency::InMemoryIdempotencyStore;
use async_trait::async_trait;
use serde_json::json;
use solana_sdk::transaction::Transaction;
use solana_client::rpc_client::RpcClient;
use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
use std::time::Duration;
use tokio::time::timeout;

/// Mock signer implementation for testing
#[derive(Debug)]
struct MockSigner {
    id: String,
    should_fail: bool,
    delay: Duration,
}

impl MockSigner {
    fn new(id: &str, should_fail: bool) -> Self {
        Self {
            id: id.to_string(),
            should_fail,
            delay: Duration::from_millis(0),
        }
    }

    fn new_with_delay(id: &str, should_fail: bool, delay: Duration) -> Self {
        Self {
            id: id.to_string(),
            should_fail,
            delay,
        }
    }
}

#[async_trait]
impl TransactionSigner for MockSigner {
    fn address(&self) -> Option<String> {
        Some(format!("mock_address_{}", self.id))
    }

    fn pubkey(&self) -> Option<String> {
        Some(format!("mock_pubkey_{}", self.id))
    }

    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        if self.delay > Duration::from_millis(0) {
            tokio::time::sleep(self.delay).await;
        }

        if self.should_fail {
            Err(SignerError::Signing(format!("MockSigner {} failed to sign", self.id)))
        } else {
            Ok(format!("mock_tx_hash_from_{}", self.id))
        }
    }

    async fn sign_and_send_evm_transaction(
        &self,
        _tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String, SignerError> {
        if self.should_fail {
            Err(SignerError::Signing(format!("MockSigner {} failed to sign EVM tx", self.id)))
        } else {
            Ok(format!("mock_evm_tx_hash_from_{}", self.id))
        }
    }

    fn solana_client(&self) -> Arc<RpcClient> {
        Arc::new(RpcClient::new("https://api.devnet.solana.com"))
    }

    fn evm_client(&self) -> Result<Arc<dyn std::any::Any + Send + Sync>, SignerError> {
        Ok(Arc::new(format!("mock_evm_client_{}", self.id)))
    }
}

/// Tool that uses signer context
struct SignerAwareTool {
    name: String,
    require_signer: bool,
}

impl SignerAwareTool {
    fn new(name: &str, require_signer: bool) -> Self {
        Self {
            name: name.to_string(),
            require_signer,
        }
    }
}

#[async_trait]
impl Tool for SignerAwareTool {
    async fn execute(
        &self,
        params: serde_json::Value,
    ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
        if self.require_signer {
            if !SignerContext::is_available().await {
                return Ok(JobResult::permanent_failure("Signer context not available"));
            }

            let signer = SignerContext::current().await
                .map_err(|e| format!("Failed to get signer: {}", e))?;

            let address = signer.address()
                .ok_or("Signer has no address")?;

            Ok(JobResult::success(&json!({
                "tool": self.name,
                "signer_address": address,
                "params": params
            }))?)
        } else {
            Ok(JobResult::success(&json!({
                "tool": self.name,
                "params": params,
                "no_signer_required": true
            }))?)
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        ""
    }
}

#[tokio::test]
async fn test_basic_signer_context_lifecycle() {
    // Initially no signer should be available
    assert!(!SignerContext::is_available().await);

    let result = SignerContext::current().await;
    assert!(result.is_err());

    // Test with signer using safe RAII pattern
    let signer = Arc::new(MockSigner::new("test1", false));
    let result = SignerContext::with_signer(signer.clone(), async {
        // Now signer should be available
        assert!(SignerContext::is_available().await);

        let current = SignerContext::current().await.unwrap();
        assert_eq!(current.address(), Some("mock_address_test1".to_string()));
        
        Ok(())
    }).await;
    
    assert!(result.is_ok());

    // Should be unavailable again after scope ends
    assert!(!SignerContext::is_available().await);
}

#[tokio::test]
async fn test_signer_context_replacement() {
    // Test nested signer contexts
    let signer1 = Arc::new(MockSigner::new("signer1", false));
    let signer2 = Arc::new(MockSigner::new("signer2", false));
    
    let result = SignerContext::with_signer(signer1, async {
        let current = SignerContext::current().await.unwrap();
        assert_eq!(current.address(), Some("mock_address_signer1".to_string()));

        // Nested context with different signer
        SignerContext::with_signer(signer2, async {
            let current = SignerContext::current().await.unwrap();
            assert_eq!(current.address(), Some("mock_address_signer2".to_string()));
            Ok(())
        }).await?;

        // Should be back to original signer
        let current = SignerContext::current().await.unwrap();
        assert_eq!(current.address(), Some("mock_address_signer1".to_string()));
        
        Ok(())
    }).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_signer_context_access() {
    let signer1 = Arc::new(MockSigner::new("concurrent1", false));
    let signer2 = Arc::new(MockSigner::new("concurrent2", false));

    // Run concurrent tasks with isolated signer contexts
    let (result1, result2) = tokio::join!(
        SignerContext::with_signer(signer1, async {
            for _ in 0..10 {
                assert!(SignerContext::is_available().await);
                let current = SignerContext::current().await.unwrap();
                let address = current.address().unwrap();
                assert_eq!(address, "mock_address_concurrent1");
                
                // Small delay to encourage race conditions if they exist
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            Ok("task1_completed")
        }),
        SignerContext::with_signer(signer2, async {
            for _ in 0..10 {
                assert!(SignerContext::is_available().await);
                let current = SignerContext::current().await.unwrap();
                let address = current.address().unwrap();
                assert_eq!(address, "mock_address_concurrent2");
                
                // Small delay to encourage race conditions if they exist
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            Ok("task2_completed")
        })
    );

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_signer_context_with_tool_execution() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    
    // Register tools that require and don't require signers
    worker.register_tool(Arc::new(SignerAwareTool::new("requires_signer", true))).await;
    worker.register_tool(Arc::new(SignerAwareTool::new("no_signer_needed", false))).await;

    // Test tool execution without signer context
    let job1 = Job::new("requires_signer", &json!({"test": "data"}), 0).unwrap();
    let result = worker.process_job(job1).await.unwrap();
    assert!(!result.is_success());
    match result {
        JobResult::Failure { error, .. } => {
            assert!(error.contains("Signer context not available"));
        }
        _ => panic!("Expected failure without signer context"),
    }

    // Tool that doesn't require signer should work
    let job2 = Job::new("no_signer_needed", &json!({"test": "data"}), 0).unwrap();
    let result = worker.process_job(job2).await.unwrap();
    assert!(result.is_success());

    // Test with signer context using safe pattern
    let signer = Arc::new(MockSigner::new("worker_test", false));
    let result = SignerContext::with_signer(signer, async {
        // Now tool requiring signer should work
        let job3 = Job::new("requires_signer", &json!({"test": "data"}), 0).unwrap();
        let result = worker.process_job(job3).await.unwrap();
        assert!(result.is_success());

        match result {
            JobResult::Success { value, .. } => {
                assert_eq!(value["signer_address"], "mock_address_worker_test");
                assert_eq!(value["tool"], "requires_signer");
            }
            _ => panic!("Expected successful result"),
        }
        
        Ok(())
    }).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_signer_context_thread_safety() {
    let mut handles = vec![];

    // Spawn tasks that use different signers concurrently - each isolated
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let signer = Arc::new(MockSigner::new(&format!("thread_{}", i), false));
            
            SignerContext::with_signer(signer, async {
                // Verify we can access our signer
                if SignerContext::is_available().await {
                    let current = SignerContext::current().await.unwrap();
                    let expected_addr = format!("mock_address_thread_{}", i);
                    assert_eq!(current.address(), Some(expected_addr));
                    
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    
                    // Should still be available and correct
                    let current = SignerContext::current().await.unwrap();
                    let expected_addr = format!("mock_address_thread_{}", i);
                    assert_eq!(current.address(), Some(expected_addr));
                    
                    return Ok(i);
                }
                Err(SignerError::NoSignerContext)
            }).await
        });
        handles.push(handle);
    }

    // Wait for all tasks
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        results.push(result.unwrap());
    }

    // All tasks should have completed successfully
    assert_eq!(results.len(), 10);
    assert_eq!(results, (0..10).collect::<Vec<_>>());
}

#[tokio::test]
async fn test_signer_context_error_handling() {
    // Test error scenarios
    
    // Getting current signer when none is set should fail
    let result = SignerContext::current().await;
    assert!(result.is_err());
    
    // Test with failing signer
    let failing_signer = Arc::new(MockSigner::new("failing", true));
    let result = SignerContext::with_signer(failing_signer, async {
        // Context should be available even if signer will fail operations
        assert!(SignerContext::is_available().await);
        
        let current = SignerContext::current().await.unwrap();
        assert_eq!(current.address(), Some("mock_address_failing".to_string()));
        
        // But signing operations should fail
        let mut dummy_tx = Transaction::default();
        let sign_result = current.sign_and_send_solana_transaction(&mut dummy_tx).await;
        assert!(sign_result.is_err());
        
        let evm_tx = alloy::rpc::types::TransactionRequest::default();
        let evm_result = current.sign_and_send_evm_transaction(evm_tx).await;
        assert!(evm_result.is_err());
        
        Ok(())
    }).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_signer_context_with_concurrent_workers() {
    let worker1 = Arc::new(ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default()));
    let worker2 = Arc::new(ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default()));
    
    worker1.register_tool(Arc::new(SignerAwareTool::new("worker1_tool", true))).await;
    worker2.register_tool(Arc::new(SignerAwareTool::new("worker2_tool", true))).await;

    // Use different signers for different workers
    let signer1 = Arc::new(MockSigner::new("worker1", false));
    let signer2 = Arc::new(MockSigner::new("worker2", false));

    // Run jobs on both workers concurrently with isolated contexts
    let (result1, result2) = tokio::join!(
        SignerContext::with_signer(signer1, async {
            let job1 = Job::new("worker1_tool", &json!({"worker": 1}), 0).unwrap();
            worker1.process_job(job1).await.map_err(|e| SignerError::Configuration(e.to_string()))
        }),
        SignerContext::with_signer(signer2, async {
            let job2 = Job::new("worker2_tool", &json!({"worker": 2}), 0).unwrap();
            worker2.process_job(job2).await.map_err(|e| SignerError::Configuration(e.to_string()))
        })
    );

    let result1 = result1.unwrap().unwrap();
    let result2 = result2.unwrap().unwrap();

    assert!(result1.is_success());
    assert!(result2.is_success());

    // Each should have used their respective signer
    match (&result1, &result2) {
        (JobResult::Success { value: v1, .. }, JobResult::Success { value: v2, .. }) => {
            assert_eq!(v1["signer_address"], "mock_address_worker1");
            assert_eq!(v2["signer_address"], "mock_address_worker2");
        }
        _ => panic!("Expected successful results from both workers"),
    }
}

#[tokio::test]
async fn test_signer_context_persistence_across_operations() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(SignerAwareTool::new("persistent_test", true))).await;

    let signer = Arc::new(MockSigner::new("persistent", false));
    
    let result = SignerContext::with_signer(signer, async {
        // Execute multiple jobs - signer should persist throughout the scope
        for i in 0..5 {
            let job = Job::new(
                "persistent_test",
                &json!({"iteration": i}),
                0
            ).unwrap();

            let result = worker.process_job(job).await.unwrap();
            assert!(result.is_success());

            match result {
                JobResult::Success { value, .. } => {
                    assert_eq!(value["signer_address"], "mock_address_persistent");
                    assert_eq!(value["params"]["iteration"], i);
                }
                _ => panic!("Expected successful result for iteration {}", i),
            }
        }
        Ok(())
    }).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_signer_context_timeout_scenarios() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig {
        default_timeout: Duration::from_millis(100), // Short timeout
        ..Default::default()
    });
    
    worker.register_tool(Arc::new(SignerAwareTool::new("timeout_test", true))).await;

    // Test with slow signer - but our test only accesses context, not actual signing
    let slow_signer = Arc::new(MockSigner::new_with_delay("slow", false, Duration::from_millis(200)));
    
    let result = SignerContext::with_signer(slow_signer, async {
        let job = Job::new("timeout_test", &json!({}), 0).unwrap();
        
        // This should complete quickly since we're only testing context access, not actual signing
        let start = std::time::Instant::now();
        let result = worker.process_job(job).await.unwrap();
        let elapsed = start.elapsed();
        
        assert!(result.is_success());
        assert!(elapsed < Duration::from_secs(1)); // Should complete quickly
        
        Ok(())
    }).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_signer_context_error_propagation() {
    struct FailingTool;

    #[async_trait]
    impl Tool for FailingTool {
        async fn execute(
            &self,
            _params: serde_json::Value,
        ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
            if SignerContext::is_available().await {
                let signer = SignerContext::current().await
                    .map_err(|e| format!("Signer context error: {}", e))?;
                
                // Try to use the signer in a way that will fail
                let mut tx = Transaction::default();
                match signer.sign_and_send_solana_transaction(&mut tx).await {
                    Ok(hash) => Ok(JobResult::success(&json!({"tx_hash": hash}))?),
                    Err(e) => Ok(JobResult::retriable_failure(format!("Signing failed: {}", e))),
                }
            } else {
                Ok(JobResult::permanent_failure("No signer available"))
            }
        }

        fn name(&self) -> &str {
            "failing_tool"
        }

        fn description(&self) -> &str {
            ""
        }
    }

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(FailingTool)).await;

    // Test with no signer
    let job1 = Job::new("failing_tool", &json!({}), 0).unwrap();
    let result1 = worker.process_job(job1).await.unwrap();
    assert!(!result1.is_success());
    assert!(!result1.is_retriable()); // Should be permanent

    // Test with failing signer
    let failing_signer = Arc::new(MockSigner::new("will_fail", true));
    let result2 = SignerContext::with_signer(failing_signer, async {
        let job2 = Job::new("failing_tool", &json!({}), 0).unwrap();
        let result = worker.process_job(job2).await.unwrap();
        assert!(!result.is_success());
        assert!(result.is_retriable()); // Signing failures should be retriable
        Ok(())
    }).await;
    assert!(result2.is_ok());

    // Test with working signer
    let working_signer = Arc::new(MockSigner::new("will_work", false));
    let result3 = SignerContext::with_signer(working_signer, async {
        let job3 = Job::new("failing_tool", &json!({}), 0).unwrap();
        let result = worker.process_job(job3).await.unwrap();
        assert!(result.is_success());
        Ok(())
    }).await;
    assert!(result3.is_ok());
}

#[tokio::test]
async fn test_signer_context_access_patterns() {
    // Test various patterns of accessing signer context
    
    // Pattern 1: Check availability before access (outside context)
    assert!(!SignerContext::is_available().await);
    
    // Pattern 2: Direct access with error handling (outside context)
    match SignerContext::current().await {
        Ok(_) => panic!("Should not have signer available"),
        Err(_) => {} // Expected
    }
    
    let signer1 = Arc::new(MockSigner::new("pattern1", false));
    let result = SignerContext::with_signer(signer1, async {
        assert!(SignerContext::is_available().await);
        
        // Pattern 3: Direct access with error handling (inside context)
        match SignerContext::current().await {
            Ok(signer) => assert!(signer.address().is_some()),
            Err(_) => panic!("Should have signer available"),
        }
        
        // Pattern 4: Multiple consecutive accesses
        for _ in 0..10 {
            assert!(SignerContext::is_available().await);
            let signer = SignerContext::current().await.unwrap();
            assert_eq!(signer.address(), Some("mock_address_pattern1".to_string()));
        }
        
        Ok(())
    }).await;
    
    assert!(result.is_ok());
    
    // Pattern 5: Should be unavailable outside context again
    assert!(!SignerContext::is_available().await);
    assert!(SignerContext::current().await.is_err());
}

#[tokio::test]
async fn test_signer_capabilities_access() {
    let signer = Arc::new(MockSigner::new("capabilities", false));
    
    let result = SignerContext::with_signer(signer, async {
        let current = SignerContext::current().await.unwrap();

        // Test all capability methods
        assert!(current.address().is_some());
        assert!(current.pubkey().is_some());
        
        let solana_client = current.solana_client();
        assert!(Arc::strong_count(&solana_client) >= 1);
        
        let evm_client_result = current.evm_client();
        assert!(evm_client_result.is_ok());

        // Test transaction methods
        let mut tx = Transaction::default();
        let solana_result = current.sign_and_send_solana_transaction(&mut tx).await;
        assert!(solana_result.is_ok());

        let evm_tx = alloy::rpc::types::TransactionRequest::default();
        let evm_result = current.sign_and_send_evm_transaction(evm_tx).await;
        assert!(evm_result.is_ok());
        
        Ok(())
    }).await;
    
    assert!(result.is_ok());
}