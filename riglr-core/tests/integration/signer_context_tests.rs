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
}

#[tokio::test]
async fn test_basic_signer_context_lifecycle() {
    // Initially no signer should be available
    assert!(!SignerContext::is_available().await);

    let result = SignerContext::current().await;
    assert!(result.is_err());

    // Set a signer
    let signer = Arc::new(MockSigner::new("test1", false));
    SignerContext::set_current(signer.clone()).await;

    // Now signer should be available
    assert!(SignerContext::is_available().await);

    let current = SignerContext::current().await.unwrap();
    assert_eq!(current.address(), Some("mock_address_test1".to_string()));

    // Clear the signer
    SignerContext::clear().await;

    // Should be unavailable again
    assert!(!SignerContext::is_available().await);
}

#[tokio::test]
async fn test_signer_context_replacement() {
    // Set first signer
    let signer1 = Arc::new(MockSigner::new("signer1", false));
    SignerContext::set_current(signer1).await;

    let current = SignerContext::current().await.unwrap();
    assert_eq!(current.address(), Some("mock_address_signer1".to_string()));

    // Replace with second signer
    let signer2 = Arc::new(MockSigner::new("signer2", false));
    SignerContext::set_current(signer2).await;

    let current = SignerContext::current().await.unwrap();
    assert_eq!(current.address(), Some("mock_address_signer2".to_string()));

    // Clean up
    SignerContext::clear().await;
}

#[tokio::test]
async fn test_concurrent_signer_context_access() {
    // Set a signer
    let signer = Arc::new(MockSigner::new("concurrent", false));
    SignerContext::set_current(signer).await;

    // Spawn multiple tasks that access the signer context concurrently
    let mut handles = vec![];
    for i in 0..20 {
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                assert!(SignerContext::is_available().await);
                let current = SignerContext::current().await.unwrap();
                let address = current.address().unwrap();
                assert_eq!(address, "mock_address_concurrent");
                
                // Small delay to encourage race conditions if they exist
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut completed = 0;
    for handle in handles {
        completed += handle.await.unwrap();
    }

    assert_eq!(completed, (0..20).sum::<i32>());

    // Clean up
    SignerContext::clear().await;
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

    // Set signer context
    let signer = Arc::new(MockSigner::new("worker_test", false));
    SignerContext::set_current(signer).await;

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

    // Clean up
    SignerContext::clear().await;
}

#[tokio::test]
async fn test_signer_context_thread_safety() {
    let counter = Arc::new(AtomicU32::new(0));
    let mut handles = vec![];

    // Spawn tasks that set different signers concurrently
    for i in 0..10 {
        let counter_clone = counter.clone();
        let handle = tokio::spawn(async move {
            let signer = Arc::new(MockSigner::new(&format!("thread_{}", i), false));
            SignerContext::set_current(signer).await;

            // Verify we can access the signer we just set
            // Note: Due to global state, we might get a different signer due to races
            if SignerContext::is_available().await {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            }

            tokio::time::sleep(Duration::from_millis(10)).await;

            if SignerContext::is_available().await {
                let current = SignerContext::current().await;
                if current.is_ok() {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // At least some operations should have succeeded
    assert!(counter.load(Ordering::Relaxed) > 0);

    // Clean up
    SignerContext::clear().await;
}

#[tokio::test]
async fn test_signer_context_error_handling() {
    // Test error scenarios
    
    // Clear context first
    SignerContext::clear().await;
    
    // Getting current signer when none is set should fail
    let result = SignerContext::current().await;
    assert!(result.is_err());
    
    // Set a failing signer
    let failing_signer = Arc::new(MockSigner::new("failing", true));
    SignerContext::set_current(failing_signer).await;
    
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

    // Clean up
    SignerContext::clear().await;
}

#[tokio::test]
async fn test_signer_context_with_concurrent_workers() {
    let worker1 = Arc::new(ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default()));
    let worker2 = Arc::new(ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default()));
    
    worker1.register_tool(Arc::new(SignerAwareTool::new("worker1_tool", true))).await;
    worker2.register_tool(Arc::new(SignerAwareTool::new("worker2_tool", true))).await;

    // Set a signer
    let signer = Arc::new(MockSigner::new("shared", false));
    SignerContext::set_current(signer).await;

    // Run jobs on both workers concurrently
    let job1 = Job::new("worker1_tool", &json!({"worker": 1}), 0).unwrap();
    let job2 = Job::new("worker2_tool", &json!({"worker": 2}), 0).unwrap();

    let handle1 = tokio::spawn({
        let worker1 = worker1.clone();
        async move { worker1.process_job(job1).await }
    });

    let handle2 = tokio::spawn({
        let worker2 = worker2.clone();
        async move { worker2.process_job(job2).await }
    });

    let result1 = handle1.await.unwrap().unwrap();
    let result2 = handle2.await.unwrap().unwrap();

    assert!(result1.is_success());
    assert!(result2.is_success());

    // Both should have used the same signer
    match (&result1, &result2) {
        (JobResult::Success { value: v1, .. }, JobResult::Success { value: v2, .. }) => {
            assert_eq!(v1["signer_address"], "mock_address_shared");
            assert_eq!(v2["signer_address"], "mock_address_shared");
        }
        _ => panic!("Expected successful results from both workers"),
    }

    // Clean up
    SignerContext::clear().await;
}

#[tokio::test]
async fn test_signer_context_persistence_across_operations() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(SignerAwareTool::new("persistent_test", true))).await;

    // Set signer
    let signer = Arc::new(MockSigner::new("persistent", false));
    SignerContext::set_current(signer).await;

    // Execute multiple jobs - signer should persist
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

    // Clean up
    SignerContext::clear().await;
}

#[tokio::test]
async fn test_signer_context_timeout_scenarios() {

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig {
        default_timeout: Duration::from_millis(100), // Short timeout
        ..Default::default()
    });
    
    worker.register_tool(Arc::new(SignerAwareTool::new("timeout_test", true))).await;

    // Set a slow signer
    let slow_signer = Arc::new(MockSigner::new_with_delay("slow", false, Duration::from_millis(200)));
    SignerContext::set_current(slow_signer).await;

    let job = Job::new("timeout_test", &json!({}), 0).unwrap();
    
    // This should complete quickly since we're only testing context access, not actual signing
    let start = std::time::Instant::now();
    let result = worker.process_job(job).await.unwrap();
    let elapsed = start.elapsed();
    
    assert!(result.is_success());
    assert!(elapsed < Duration::from_secs(1)); // Should complete quickly

    // Clean up
    SignerContext::clear().await;
}

#[tokio::test]
async fn test_signer_context_multiple_rapid_changes() {
    // Rapidly change signers to test for race conditions
    let mut handles = vec![];
    
    for i in 0..50 {
        let handle = tokio::spawn(async move {
            let signer = Arc::new(MockSigner::new(&format!("rapid_{}", i), false));
            SignerContext::set_current(signer).await;
            
            // Quick verification
            if SignerContext::is_available().await {
                let current = SignerContext::current().await;
                if let Ok(signer) = current {
                    if let Some(addr) = signer.address() {
                        return addr.contains("rapid_");
                    }
                }
            }
            false
        });
        handles.push(handle);
    }

    // Wait for all changes
    let mut success_count = 0;
    for handle in handles {
        if handle.await.unwrap() {
            success_count += 1;
        }
    }

    // Most changes should succeed
    assert!(success_count > 40);

    // Clean up
    SignerContext::clear().await;
}

#[tokio::test]
async fn test_signer_context_clear_during_use() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(SignerAwareTool::new("clear_test", true))).await;

    // Set signer
    let signer = Arc::new(MockSigner::new("clear_me", false));
    SignerContext::set_current(signer).await;

    // Start a job
    let job = Job::new("clear_test", &json!({}), 0).unwrap();
    let worker_clone = worker.clone();
    
    let job_handle = tokio::spawn(async move {
        worker_clone.process_job(job).await.unwrap()
    });

    // Clear context while job might be running
    tokio::time::sleep(Duration::from_millis(10)).await;
    SignerContext::clear().await;

    // Job should still complete successfully since it already captured the signer
    let result = job_handle.await.unwrap();
    assert!(result.is_success());
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
    SignerContext::set_current(failing_signer).await;

    let job2 = Job::new("failing_tool", &json!({}), 0).unwrap();
    let result2 = worker.process_job(job2).await.unwrap();
    assert!(!result2.is_success());
    assert!(result2.is_retriable()); // Signing failures should be retriable

    // Test with working signer
    let working_signer = Arc::new(MockSigner::new("will_work", false));
    SignerContext::set_current(working_signer).await;

    let job3 = Job::new("failing_tool", &json!({}), 0).unwrap();
    let result3 = worker.process_job(job3).await.unwrap();
    assert!(result3.is_success());

    // Clean up
    SignerContext::clear().await;
}

#[tokio::test]
async fn test_signer_context_access_patterns() {
    // Test various patterns of accessing signer context
    
    // Pattern 1: Check availability before access
    SignerContext::clear().await;
    assert!(!SignerContext::is_available().await);
    
    let signer1 = Arc::new(MockSigner::new("pattern1", false));
    SignerContext::set_current(signer1).await;
    assert!(SignerContext::is_available().await);
    
    // Pattern 2: Direct access with error handling
    match SignerContext::current().await {
        Ok(signer) => assert!(signer.address().is_some()),
        Err(_) => panic!("Should have signer available"),
    }
    
    // Pattern 3: Multiple consecutive accesses
    for _ in 0..10 {
        assert!(SignerContext::is_available().await);
        let signer = SignerContext::current().await.unwrap();
        assert_eq!(signer.address(), Some("mock_address_pattern1".to_string()));
    }
    
    // Pattern 4: Access after clear
    SignerContext::clear().await;
    assert!(!SignerContext::is_available().await);
    assert!(SignerContext::current().await.is_err());
    
    // Pattern 5: Rapid set/clear cycles
    for i in 0..5 {
        let signer = Arc::new(MockSigner::new(&format!("cycle_{}", i), false));
        SignerContext::set_current(signer).await;
        assert!(SignerContext::is_available().await);
        
        let current = SignerContext::current().await.unwrap();
        assert_eq!(current.address(), Some(format!("mock_address_cycle_{}", i)));
        
        SignerContext::clear().await;
        assert!(!SignerContext::is_available().await);
    }
}

#[tokio::test]
async fn test_signer_capabilities_access() {
    let signer = Arc::new(MockSigner::new("capabilities", false));
    SignerContext::set_current(signer).await;

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

    // Clean up
    SignerContext::clear().await;
}