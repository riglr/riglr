//! Integration tests for SignerContext isolation in multi-agent systems.
//!
//! These tests verify that the SignerContext security model is maintained
//! across agent boundaries, ensuring no cross-agent signer access and
//! proper isolation of blockchain operations.

use riglr_agents::*;
use riglr_core::signer::{SignerContext, MockSigner, Signer};
use std::sync::Arc;
use std::time::Duration;
use serde_json::json;

/// Mock blockchain signer for testing isolation.
#[derive(Debug, Clone)]
struct TestSigner {
    id: String,
    address: String,
    operations: Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl TestSigner {
    fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            address: format!("0x{:x}", id_str.as_bytes().iter().fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64))),
            id: id_str,
            operations: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    async fn get_operations(&self) -> Vec<String> {
        self.operations.lock().await.clone()
    }

    async fn record_operation(&self, operation: String) {
        self.operations.lock().await.push(operation);
    }
}

#[async_trait::async_trait]
impl Signer for TestSigner {
    type Address = String;
    type Signature = String;

    async fn sign(&self, message: &[u8]) -> Result<Self::Signature, Box<dyn std::error::Error + Send + Sync>> {
        let operation = format!("sign:{}", String::from_utf8_lossy(message));
        self.record_operation(operation).await;
        Ok(format!("sig_{}_{}", self.id, hex::encode(message)))
    }

    async fn address(&self) -> Result<Self::Address, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.address.clone())
    }

    async fn chain_id(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        Ok(1) // Mainnet
    }
}

/// Agent that uses SignerContext to perform operations.
#[derive(Clone)]
struct ContextAwareAgent {
    id: AgentId,
    expected_signer_id: String,
}

#[async_trait::async_trait]
impl Agent for ContextAwareAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        // Use SignerContext to access the current signer
        match SignerContext::current::<TestSigner>().await {
            Ok(signer) => {
                // Verify we got the expected signer
                if signer.id != self.expected_signer_id {
                    return Ok(TaskResult::failure(
                        format!("Signer mismatch: expected {}, got {}", 
                               self.expected_signer_id, signer.id),
                        false,
                        Duration::from_millis(1),
                    ));
                }

                // Perform operation with the signer
                let message = format!("task_{}_{}", task.id, task.task_type);
                let signature = signer.sign(message.as_bytes()).await
                    .map_err(|e| AgentError::task_execution(format!("Signing failed: {}", e)))?;

                let result = json!({
                    "agent_id": self.id.as_str(),
                    "task_id": task.id,
                    "signer_id": signer.id,
                    "signer_address": signer.address().await.unwrap_or_default(),
                    "signature": signature,
                    "operation": "signed_transaction"
                });

                Ok(TaskResult::success(result, None, Duration::from_millis(10)))
            }
            Err(e) => Ok(TaskResult::failure(
                format!("No signer context available: {}", e),
                false,
                Duration::from_millis(1),
            ))
        }
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["trading".to_string(), "signing".to_string()]
    }
}

/// Agent that attempts to access SignerContext without proper isolation.
#[derive(Clone)]
struct MaliciousAgent {
    id: AgentId,
    attempted_operations: Arc<tokio::sync::Mutex<Vec<String>>>,
}

#[async_trait::async_trait]
impl Agent for MaliciousAgent {
    async fn execute_task(&self, _task: Task) -> Result<TaskResult> {
        let mut operations = self.attempted_operations.lock().await;
        
        // Attempt to access signer context (should fail if no context set)
        match SignerContext::current::<TestSigner>().await {
            Ok(signer) => {
                operations.push(format!("accessed_signer:{}", signer.id));
                // Attempt unauthorized operation
                let _ = signer.sign(b"malicious_operation").await;
                operations.push("performed_malicious_operation".to_string());
                
                Ok(TaskResult::success(
                    json!({"status": "unauthorized_access_succeeded"}),
                    None,
                    Duration::from_millis(10),
                ))
            }
            Err(_) => {
                operations.push("signer_access_denied".to_string());
                Ok(TaskResult::success(
                    json!({"status": "access_properly_denied"}),
                    None,
                    Duration::from_millis(10),
                ))
            }
        }
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["malicious".to_string()]
    }
}

/// Test basic SignerContext isolation between agents.
#[tokio::test]
async fn test_basic_signer_context_isolation() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Create signers for different contexts
    let signer_a = Arc::new(TestSigner::new("signer_a"));
    let signer_b = Arc::new(TestSigner::new("signer_b"));

    // Create agents that expect specific signers
    let agent_a = Arc::new(ContextAwareAgent {
        id: AgentId::new("agent_a"),
        expected_signer_id: "signer_a".to_string(),
    });

    let agent_b = Arc::new(ContextAwareAgent {
        id: AgentId::new("agent_b"),
        expected_signer_id: "signer_b".to_string(),
    });

    registry.register_agent(agent_a).await.unwrap();
    registry.register_agent(agent_b).await.unwrap();

    // Execute task with signer A context
    let task_a = Task::new(
        TaskType::Trading,
        json!({"symbol": "BTC/USD", "context": "A"}),
    );

    let result_a = SignerContext::new(signer_a.clone())
        .execute(async {
            dispatcher.dispatch_task(task_a).await
        })
        .await
        .unwrap();

    assert!(result_a.is_success());
    let data_a = result_a.data().unwrap();
    assert_eq!(data_a.get("signer_id").unwrap().as_str().unwrap(), "signer_a");

    // Execute task with signer B context
    let task_b = Task::new(
        TaskType::Trading,
        json!({"symbol": "ETH/USD", "context": "B"}),
    );

    let result_b = SignerContext::new(signer_b.clone())
        .execute(async {
            dispatcher.dispatch_task(task_b).await
        })
        .await
        .unwrap();

    assert!(result_b.is_success());
    let data_b = result_b.data().unwrap();
    assert_eq!(data_b.get("signer_id").unwrap().as_str().unwrap(), "signer_b");

    // Verify signers recorded separate operations
    let ops_a = signer_a.get_operations().await;
    let ops_b = signer_b.get_operations().await;
    
    assert!(!ops_a.is_empty());
    assert!(!ops_b.is_empty());
    assert_ne!(ops_a, ops_b);
}

/// Test that agents cannot access signers from other contexts.
#[tokio::test]
async fn test_cross_context_signer_access_prevention() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let signer = Arc::new(TestSigner::new("protected_signer"));
    let malicious_operations = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let malicious_agent = Arc::new(MaliciousAgent {
        id: AgentId::new("malicious_agent"),
        attempted_operations: malicious_operations.clone(),
    });

    registry.register_agent(malicious_agent).await.unwrap();

    // Execute task outside of any signer context
    let task = Task::new(
        TaskType::Custom("malicious".to_string()),
        json!({"target": "steal_signer"}),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();
    assert!(result.is_success());

    let data = result.data().unwrap();
    assert_eq!(data.get("status").unwrap().as_str().unwrap(), "access_properly_denied");

    // Verify the malicious agent couldn't access any signer
    let operations = malicious_operations.lock().await;
    assert!(operations.contains(&"signer_access_denied".to_string()));
    assert!(!operations.iter().any(|op| op.starts_with("accessed_signer:")));
    assert!(!operations.contains(&"performed_malicious_operation".to_string()));
}

/// Test concurrent signer context isolation.
#[tokio::test]
async fn test_concurrent_signer_context_isolation() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Create multiple agents expecting different signers
    for i in 0..3 {
        let agent = Arc::new(ContextAwareAgent {
            id: AgentId::new(&format!("concurrent_agent_{}", i)),
            expected_signer_id: format!("signer_{}", i),
        });
        registry.register_agent(agent).await.unwrap();
    }

    // Create tasks and signers
    let tasks_and_signers: Vec<(Task, Arc<TestSigner>)> = (0..3)
        .map(|i| {
            let task = Task::new(
                TaskType::Trading,
                json!({"symbol": format!("COIN{}/USD", i)}),
            );
            let signer = Arc::new(TestSigner::new(format!("signer_{}", i)));
            (task, signer)
        })
        .collect();

    // Execute tasks concurrently with different signer contexts
    let futures: Vec<_> = tasks_and_signers
        .into_iter()
        .map(|(task, signer)| {
            let dispatcher = dispatcher.clone();
            tokio::spawn(async move {
                SignerContext::new(signer.clone())
                    .execute(async move {
                        dispatcher.dispatch_task(task).await
                    })
                    .await
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(futures).await;

    // Verify all tasks completed successfully with correct signer isolation
    for (i, result) in results.into_iter().enumerate() {
        let task_result = result.unwrap().unwrap();
        assert!(task_result.is_success());
        
        let data = task_result.data().unwrap();
        let signer_id = data.get("signer_id").unwrap().as_str().unwrap();
        assert_eq!(signer_id, format!("signer_{}", i));
    }
}

/// Test signer context inheritance in nested operations.
#[tokio::test]
async fn test_signer_context_inheritance() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let parent_signer = Arc::new(TestSigner::new("parent_signer"));
    
    let agent = Arc::new(ContextAwareAgent {
        id: AgentId::new("inheritance_agent"),
        expected_signer_id: "parent_signer".to_string(),
    });
    
    registry.register_agent(agent).await.unwrap();

    // Create nested async operation
    let result = SignerContext::new(parent_signer.clone())
        .execute(async {
            // This should inherit the parent signer context
            let task = Task::new(
                TaskType::Trading,
                json!({"symbol": "BTC/USD", "nested": true}),
            );
            
            dispatcher.dispatch_task(task).await
        })
        .await
        .unwrap();

    assert!(result.is_success());
    let data = result.data().unwrap();
    assert_eq!(data.get("signer_id").unwrap().as_str().unwrap(), "parent_signer");

    // Verify the parent signer was used
    let operations = parent_signer.get_operations().await;
    assert!(!operations.is_empty());
    assert!(operations.iter().any(|op| op.contains("task_")));
}

/// Test signer context cleanup after agent execution.
#[tokio::test]
async fn test_signer_context_cleanup() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let signer = Arc::new(TestSigner::new("cleanup_signer"));
    let malicious_operations = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let normal_agent = Arc::new(ContextAwareAgent {
        id: AgentId::new("normal_agent"),
        expected_signer_id: "cleanup_signer".to_string(),
    });

    let malicious_agent = Arc::new(MaliciousAgent {
        id: AgentId::new("cleanup_malicious"),
        attempted_operations: malicious_operations.clone(),
    });

    registry.register_agent(normal_agent).await.unwrap();
    registry.register_agent(malicious_agent).await.unwrap();

    // Execute task within signer context
    {
        let result = SignerContext::new(signer.clone())
            .execute(async {
                let task = Task::new(
                    TaskType::Trading,
                    json!({"symbol": "BTC/USD"}),
                );
                dispatcher.dispatch_task(task).await
            })
            .await
            .unwrap();

        assert!(result.is_success());
    }

    // After context is dropped, malicious agent should not have access
    let malicious_task = Task::new(
        TaskType::Custom("malicious".to_string()),
        json!({"attempt": "post_cleanup"}),
    );

    let result = dispatcher.dispatch_task(malicious_task).await.unwrap();
    assert!(result.is_success());

    let data = result.data().unwrap();
    assert_eq!(data.get("status").unwrap().as_str().unwrap(), "access_properly_denied");

    // Verify no unauthorized access occurred
    let operations = malicious_operations.lock().await;
    assert!(!operations.iter().any(|op| op.starts_with("accessed_signer:")));
}

/// Test multiple concurrent signer contexts don't interfere.
#[tokio::test]
async fn test_multiple_signer_contexts_non_interference() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Create agents for different user contexts
    let user_a_agent = Arc::new(ContextAwareAgent {
        id: AgentId::new("user_a_agent"),
        expected_signer_id: "user_a_signer".to_string(),
    });

    let user_b_agent = Arc::new(ContextAwareAgent {
        id: AgentId::new("user_b_agent"),
        expected_signer_id: "user_b_signer".to_string(),
    });

    registry.register_agent(user_a_agent).await.unwrap();
    registry.register_agent(user_b_agent).await.unwrap();

    let user_a_signer = Arc::new(TestSigner::new("user_a_signer"));
    let user_b_signer = Arc::new(TestSigner::new("user_b_signer"));

    // Execute concurrent operations for different users
    let (result_a, result_b) = tokio::join!(
        SignerContext::new(user_a_signer.clone()).execute(async {
            let task = Task::new(
                TaskType::Trading,
                json!({"symbol": "BTC/USD", "user": "A"}),
            );
            dispatcher.dispatch_task(task).await
        }),
        
        SignerContext::new(user_b_signer.clone()).execute(async {
            let task = Task::new(
                TaskType::Trading,
                json!({"symbol": "ETH/USD", "user": "B"}),
            );
            dispatcher.dispatch_task(task).await
        })
    );

    // Both operations should succeed with their respective signers
    let result_a = result_a.unwrap();
    let result_b = result_b.unwrap();

    assert!(result_a.is_success());
    assert!(result_b.is_success());

    let data_a = result_a.data().unwrap();
    let data_b = result_b.data().unwrap();

    assert_eq!(data_a.get("signer_id").unwrap().as_str().unwrap(), "user_a_signer");
    assert_eq!(data_b.get("signer_id").unwrap().as_str().unwrap(), "user_b_signer");

    // Verify each signer only handled their own operations
    let ops_a = user_a_signer.get_operations().await;
    let ops_b = user_b_signer.get_operations().await;

    assert_eq!(ops_a.len(), 1);
    assert_eq!(ops_b.len(), 1);
    assert!(ops_a[0].contains("task_"));
    assert!(ops_b[0].contains("task_"));
    assert_ne!(ops_a[0], ops_b[0]); // Different task IDs
}

/// Test signer context with task failures.
#[tokio::test]
async fn test_signer_context_with_task_failures() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let signer = Arc::new(TestSigner::new("failure_test_signer"));

    // Agent that expects a different signer (will fail)
    let failing_agent = Arc::new(ContextAwareAgent {
        id: AgentId::new("failing_agent"),
        expected_signer_id: "wrong_signer".to_string(),
    });

    registry.register_agent(failing_agent).await.unwrap();

    let result = SignerContext::new(signer.clone())
        .execute(async {
            let task = Task::new(
                TaskType::Trading,
                json!({"symbol": "BTC/USD", "should_fail": true}),
            );
            dispatcher.dispatch_task(task).await
        })
        .await
        .unwrap();

    // Task should complete but report failure due to signer mismatch
    assert!(!result.is_success());
    
    if let Some(error) = result.error() {
        assert!(error.contains("Signer mismatch"));
    }

    // Signer should not have recorded any operations due to early failure
    let operations = signer.get_operations().await;
    assert!(operations.is_empty());
}

/// Test signer context isolation with custom task types.
#[tokio::test]
async fn test_signer_context_with_custom_tasks() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let signer = Arc::new(TestSigner::new("custom_signer"));

    let agent = Arc::new(ContextAwareAgent {
        id: AgentId::new("custom_agent"),
        expected_signer_id: "custom_signer".to_string(),
    });

    registry.register_agent(agent).await.unwrap();

    // Test with custom task type
    let result = SignerContext::new(signer.clone())
        .execute(async {
            let task = Task::new(
                TaskType::Custom("custom_operation".to_string()),
                json!({"operation": "custom", "params": {"key": "value"}}),
            );
            dispatcher.dispatch_task(task).await
        })
        .await
        .unwrap();

    assert!(result.is_success());
    
    let data = result.data().unwrap();
    assert_eq!(data.get("signer_id").unwrap().as_str().unwrap(), "custom_signer");
    assert!(data.get("signature").unwrap().as_str().unwrap().contains("custom"));

    // Verify custom operation was signed
    let operations = signer.get_operations().await;
    assert!(!operations.is_empty());
    assert!(operations[0].contains("custom_operation"));
}

/// Performance test for signer context switching overhead.
#[tokio::test]
#[ignore] // Ignore for regular test runs
async fn test_signer_context_switching_performance() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Create multiple agents and signers
    for i in 0..10 {
        let agent = Arc::new(ContextAwareAgent {
            id: AgentId::new(&format!("perf_agent_{}", i)),
            expected_signer_id: format!("perf_signer_{}", i),
        });
        registry.register_agent(agent).await.unwrap();
    }

    let signers: Vec<Arc<TestSigner>> = (0..10)
        .map(|i| Arc::new(TestSigner::new(format!("perf_signer_{}", i))))
        .collect();

    // Measure context switching performance
    let iterations = 100;
    let start_time = std::time::Instant::now();

    for i in 0..iterations {
        let signer_idx = i % signers.len();
        let signer = signers[signer_idx].clone();
        
        let _result = SignerContext::new(signer)
            .execute(async {
                let task = Task::new(
                    TaskType::Trading,
                    json!({"symbol": format!("PERF{}/USD", i)}),
                );
                dispatcher.dispatch_task(task).await
            })
            .await
            .unwrap();
    }

    let elapsed = start_time.elapsed();
    
    println!("Executed {} context switches in {:?}", iterations, elapsed);
    
    // Should handle context switching efficiently
    assert!(elapsed < Duration::from_secs(5));
    
    // Verify operations were recorded correctly
    for signer in &signers {
        let ops = signer.get_operations().await;
        assert!(!ops.is_empty());
    }
}