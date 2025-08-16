//! Edge case and boundary condition tests for riglr-agents.
//!
//! These tests verify that the system handles unusual scenarios,
//! boundary conditions, and edge cases gracefully.

use riglr_agents::*;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

/// Agent that exhibits unusual behavior for edge case testing.
#[derive(Clone)]
struct EdgeCaseAgent {
    id: AgentId,
    behavior: EdgeCaseBehavior,
}

#[derive(Clone, Debug)]
enum EdgeCaseBehavior {
    /// Returns empty result data
    EmptyResult,
    /// Returns extremely large result data
    LargeResult,
    /// Takes exactly the maximum timeout
    MaxTimeout,
    /// Returns result with unusual data types
    WeirdDataTypes,
    /// Claims capabilities it doesn't actually have
    FalseCapabilities,
    /// Changes behavior between calls
    Inconsistent(Arc<std::sync::atomic::AtomicBool>),
}

impl EdgeCaseAgent {
    fn new(id: impl Into<String>, behavior: EdgeCaseBehavior) -> Self {
        Self {
            id: AgentId::new(id),
            behavior,
        }
    }
}

#[async_trait::async_trait]
impl Agent for EdgeCaseAgent {
    async fn execute_task(&self, _task: Task) -> Result<TaskResult> {
        match &self.behavior {
            EdgeCaseBehavior::EmptyResult => {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(TaskResult::success(
                    json!({}),
                    None,
                    Duration::from_millis(10),
                ))
            }

            EdgeCaseBehavior::LargeResult => {
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Create large result (1MB of data)
                let large_data = "x".repeat(1024 * 1024);
                Ok(TaskResult::success(
                    json!({
                        "agent_id": self.id.as_str(),
                        "large_field": large_data,
                        "metadata": {
                            "size_mb": 1,
                            "type": "large_result"
                        }
                    }),
                    None,
                    Duration::from_millis(50),
                ))
            }

            EdgeCaseBehavior::MaxTimeout => {
                // Sleep for almost the default timeout (but not quite)
                tokio::time::sleep(Duration::from_millis(4900)).await;
                Ok(TaskResult::success(
                    json!({"timeout_test": "completed_just_in_time"}),
                    None,
                    Duration::from_millis(4900),
                ))
            }

            EdgeCaseBehavior::WeirdDataTypes => {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(TaskResult::success(
                    json!({
                        "null_value": null,
                        "empty_string": "",
                        "empty_array": [],
                        "empty_object": {},
                        "unicode": "🚀🦀💎",
                        "very_long_key_with_special_characters_and_numbers_123456789": "value",
                        "nested": {
                            "deeply": {
                                "nested": {
                                    "structure": {
                                        "value": "deep"
                                    }
                                }
                            }
                        }
                    }),
                    None,
                    Duration::from_millis(10),
                ))
            }

            EdgeCaseBehavior::FalseCapabilities => {
                // Claim to handle any task but actually can't handle anything properly
                Ok(TaskResult::failure(
                    "Agent claimed false capabilities".to_string(),
                    false,
                    Duration::from_millis(10),
                ))
            }

            EdgeCaseBehavior::Inconsistent(state) => {
                let current_state = state.load(std::sync::atomic::Ordering::Relaxed);
                state.store(!current_state, std::sync::atomic::Ordering::Relaxed);

                if current_state {
                    Ok(TaskResult::success(
                        json!({"behavior": "success"}),
                        None,
                        Duration::from_millis(10),
                    ))
                } else {
                    Ok(TaskResult::failure(
                        "Inconsistent behavior".to_string(),
                        true,
                        Duration::from_millis(10),
                    ))
                }
            }
        }
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        match &self.behavior {
            EdgeCaseBehavior::FalseCapabilities => {
                // Falsely claim all capabilities
                vec![
                    "trading".to_string(),
                    "research".to_string(),
                    "risk_analysis".to_string(),
                    "portfolio".to_string(),
                    "monitoring".to_string(),
                ]
            }
            _ => vec!["edge_case".to_string()],
        }
    }

    fn can_handle(&self, _task: &Task) -> bool {
        match &self.behavior {
            EdgeCaseBehavior::FalseCapabilities => true, // Falsely claim to handle everything
            _ => true,
        }
    }
}

/// Test handling of empty results.
#[tokio::test]
async fn test_empty_result_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = AgentDispatcher::new(registry.clone());

    let empty_agent = Arc::new(EdgeCaseAgent::new(
        "empty_agent",
        EdgeCaseBehavior::EmptyResult,
    ));

    registry.register_agent(empty_agent).await.unwrap();

    let task = Task::new(
        TaskType::Custom("edge_case".to_string()),
        json!({"test": "empty_result"}),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();
    assert!(result.is_success());

    // Verify empty result is handled correctly
    let data = result.data().unwrap();
    assert!(data.as_object().unwrap().is_empty());
}

/// Test handling of extremely large results.
#[tokio::test]
async fn test_large_result_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = AgentDispatcher::new(registry.clone());

    let large_agent = Arc::new(EdgeCaseAgent::new(
        "large_agent",
        EdgeCaseBehavior::LargeResult,
    ));

    registry.register_agent(large_agent).await.unwrap();

    let task = Task::new(
        TaskType::Custom("edge_case".to_string()),
        json!({"test": "large_result"}),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();
    assert!(result.is_success());

    let data = result.data().unwrap();
    assert!(data.get("large_field").is_some());
    assert_eq!(data.get("metadata").unwrap().get("size_mb").unwrap(), 1);

    // Verify the system can handle large data without crashing
    let large_field = data.get("large_field").unwrap().as_str().unwrap();
    assert!(large_field.len() > 1_000_000);
}

/// Test edge case of tasks completing just before timeout.
#[tokio::test]
async fn test_near_timeout_completion() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        default_task_timeout: Duration::from_secs(5),
        ..Default::default()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    let timeout_agent = Arc::new(EdgeCaseAgent::new(
        "timeout_agent",
        EdgeCaseBehavior::MaxTimeout,
    ));

    registry.register_agent(timeout_agent).await.unwrap();

    let task = Task::new(
        TaskType::Custom("edge_case".to_string()),
        json!({"test": "near_timeout"}),
    );

    let start_time = std::time::Instant::now();
    let result = dispatcher.dispatch_task(task).await.unwrap();
    let elapsed = start_time.elapsed();

    assert!(result.is_success());
    assert!(elapsed < Duration::from_secs(5)); // Should complete before timeout
    assert!(elapsed > Duration::from_millis(4800)); // But should take significant time
}

/// Test handling of unusual data types in results.
#[tokio::test]
async fn test_weird_data_types_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = AgentDispatcher::new(registry.clone());

    let weird_agent = Arc::new(EdgeCaseAgent::new(
        "weird_agent",
        EdgeCaseBehavior::WeirdDataTypes,
    ));

    registry.register_agent(weird_agent).await.unwrap();

    let task = Task::new(
        TaskType::Custom("edge_case".to_string()),
        json!({"test": "weird_data_types"}),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();
    assert!(result.is_success());

    let data = result.data().unwrap();

    // Verify various data types are preserved
    assert!(data.get("null_value").unwrap().is_null());
    assert_eq!(data.get("empty_string").unwrap().as_str().unwrap(), "");
    assert!(data
        .get("empty_array")
        .unwrap()
        .as_array()
        .unwrap()
        .is_empty());
    assert!(data
        .get("empty_object")
        .unwrap()
        .as_object()
        .unwrap()
        .is_empty());
    assert_eq!(data.get("unicode").unwrap().as_str().unwrap(), "🚀🦀💎");

    // Verify nested structure is preserved
    let nested = data
        .get("nested")
        .unwrap()
        .get("deeply")
        .unwrap()
        .get("nested")
        .unwrap()
        .get("structure")
        .unwrap()
        .get("value")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(nested, "deep");
}

/// Test handling of agents with false capabilities.
#[tokio::test]
async fn test_false_capabilities_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = AgentDispatcher::new(registry.clone());

    let false_agent = Arc::new(EdgeCaseAgent::new(
        "false_agent",
        EdgeCaseBehavior::FalseCapabilities,
    ));

    registry.register_agent(false_agent).await.unwrap();

    // Try different types of tasks that the agent claims to support
    let tasks = vec![
        Task::new(TaskType::Trading, json!({"symbol": "BTC/USD"})),
        Task::new(TaskType::Research, json!({"query": "market analysis"})),
        Task::new(TaskType::RiskAnalysis, json!({"amount": 100})),
    ];

    for task in tasks {
        let result = dispatcher.dispatch_task(task).await.unwrap();
        // Agent should fail to execute despite claiming capabilities
        assert!(!result.is_success());
        assert!(!result.is_retriable()); // Permanent failure
    }
}

/// Test handling of inconsistent agent behavior.
#[tokio::test]
async fn test_inconsistent_agent_behavior() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        max_retries: 3,
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    let inconsistent_agent = Arc::new(EdgeCaseAgent::new(
        "inconsistent_agent",
        EdgeCaseBehavior::Inconsistent(Arc::new(std::sync::atomic::AtomicBool::new(false))),
    ));

    registry.register_agent(inconsistent_agent).await.unwrap();

    let mut results = Vec::new();

    // Execute multiple tasks to see inconsistent behavior
    for i in 0..6 {
        let task = Task::new(
            TaskType::Custom("edge_case".to_string()),
            json!({"test": "inconsistent", "iteration": i}),
        );

        let result = dispatcher.dispatch_task(task).await.unwrap();
        results.push(result.is_success());
    }

    // Should have a mix of successes and failures
    let success_count = results.iter().filter(|&&success| success).count();
    let failure_count = results.len() - success_count;

    assert!(success_count > 0);
    assert!(failure_count > 0);
}

/// Test handling of zero-timeout tasks.
#[tokio::test]
async fn test_zero_timeout_tasks() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = AgentDispatcher::new(registry.clone());

    let fast_agent = Arc::new(EdgeCaseAgent::new(
        "fast_agent",
        EdgeCaseBehavior::EmptyResult,
    ));

    registry.register_agent(fast_agent).await.unwrap();

    let task = Task::new(
        TaskType::Custom("edge_case".to_string()),
        json!({"test": "zero_timeout"}),
    )
    .with_timeout(Duration::ZERO);

    // Zero timeout should likely cause immediate timeout
    let result = dispatcher.dispatch_task(task).await;

    // Should either timeout or complete very quickly
    match result {
        Ok(task_result) => {
            // If it completes, it should be very fast
            assert!(task_result.is_success());
        }
        Err(agent_error) => {
            // Or it should timeout
            assert!(matches!(agent_error, AgentError::TaskTimeout { .. }));
        }
    }
}

/// Test handling of tasks with maximum retry count.
#[tokio::test]
async fn test_maximum_retry_count() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        max_retries: u32::MAX, // Maximum possible retries (impractical but tests boundary)
        retry_delay: Duration::from_millis(1),
        default_task_timeout: Duration::from_millis(100),
        ..Default::default()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    let always_fail_agent = Arc::new(EdgeCaseAgent::new(
        "always_fail_agent",
        EdgeCaseBehavior::FalseCapabilities,
    ));

    registry.register_agent(always_fail_agent).await.unwrap();

    let task = Task::new(
        TaskType::Custom("edge_case".to_string()),
        json!({"test": "max_retries"}),
    );

    let start_time = std::time::Instant::now();
    let result = dispatcher.dispatch_task(task).await.unwrap();
    let elapsed = start_time.elapsed();

    // Should fail without actually doing max retries (would take forever)
    assert!(!result.is_success());

    // Should not take an unreasonable amount of time
    assert!(elapsed < Duration::from_secs(10));
}

/// Test handling of tasks with extremely high priority.
#[tokio::test]
async fn test_extreme_priority_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = AgentDispatcher::new(registry.clone());

    let priority_agent = Arc::new(EdgeCaseAgent::new(
        "priority_agent",
        EdgeCaseBehavior::EmptyResult,
    ));

    registry.register_agent(priority_agent).await.unwrap();

    // Test with all priority levels including edge cases
    let priorities = vec![
        Priority::Critical,
        Priority::High,
        Priority::Normal,
        Priority::Low,
    ];

    for priority in priorities {
        let task = Task::new(
            TaskType::Custom("edge_case".to_string()),
            json!({"test": "priority", "level": priority as u8}),
        )
        .with_priority(priority);

        let result = dispatcher.dispatch_task(task).await.unwrap();
        assert!(result.is_success());
    }
}

/// Test handling of empty agent registry.
#[tokio::test]
async fn test_empty_registry_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = AgentDispatcher::new(registry.clone());

    // No agents registered
    assert_eq!(registry.agent_count().await.unwrap(), 0);

    let task = Task::new(
        TaskType::Custom("edge_case".to_string()),
        json!({"test": "no_agents"}),
    );

    let result = dispatcher.dispatch_task(task).await;

    // Should fail with no suitable agent error
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AgentError::NoSuitableAgent { .. }
    ));
}

/// Test handling of registry at capacity limit.
#[tokio::test]
async fn test_registry_capacity_edge_case() {
    let config = crate::registry::RegistryConfig {
        max_agents: Some(1), // Very small capacity
        ..Default::default()
    };
    let registry = LocalAgentRegistry::with_config(config);

    // Fill registry to capacity
    let agent1 = Arc::new(EdgeCaseAgent::new("agent1", EdgeCaseBehavior::EmptyResult));

    registry.register_agent(agent1).await.unwrap();
    assert_eq!(registry.agent_count().await.unwrap(), 1);

    // Try to register beyond capacity
    let agent2 = Arc::new(EdgeCaseAgent::new("agent2", EdgeCaseBehavior::EmptyResult));

    let result = registry.register_agent(agent2).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("capacity"));

    // Count should remain unchanged
    assert_eq!(registry.agent_count().await.unwrap(), 1);
}

/// Test handling of very long agent IDs and task IDs.
#[tokio::test]
async fn test_long_identifier_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = AgentDispatcher::new(registry.clone());

    // Create agent with very long ID
    let long_id = "a".repeat(1000);
    let long_id_agent = Arc::new(EdgeCaseAgent::new(&long_id, EdgeCaseBehavior::EmptyResult));

    registry.register_agent(long_id_agent).await.unwrap();

    // Verify agent can be retrieved with long ID
    let retrieved = registry.get_agent(&AgentId::new(&long_id)).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id().as_str(), long_id);

    // Create task with very long parameters
    let long_param = "x".repeat(10000);
    let task = Task::new(
        TaskType::Custom("edge_case".to_string()),
        json!({"very_long_parameter": long_param}),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();
    assert!(result.is_success());
}

/// Test handling of concurrent modifications to registry.
#[tokio::test]
async fn test_concurrent_registry_modifications() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Spawn multiple tasks that modify the registry concurrently
    let mut handles = Vec::new();

    for i in 0..10 {
        let registry_clone = registry.clone();
        let handle = tokio::spawn(async move {
            let agent = Arc::new(EdgeCaseAgent::new(
                format!("concurrent_agent_{}", i),
                EdgeCaseBehavior::EmptyResult,
            ));

            // Register agent
            let register_result = registry_clone.register_agent(agent).await;

            // Immediately try to unregister
            let unregister_result = registry_clone
                .unregister_agent(&AgentId::new(format!("concurrent_agent_{}", i)))
                .await;

            (register_result.is_ok(), unregister_result.is_ok())
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    let results: Vec<(bool, bool)> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // Most operations should succeed (some may fail due to races)
    let successful_registers = results.iter().filter(|(reg, _)| *reg).count();
    let successful_unregisters = results.iter().filter(|(_, unreg)| *unreg).count();

    // Should have some successful operations despite concurrency
    assert!(successful_registers > 0);
    assert!(successful_unregisters > 0);

    // Final state should be consistent
    let final_count = registry.agent_count().await.unwrap();
    assert!(final_count <= 10); // Should not exceed the number of attempted registrations
}

/// Test handling of message expiration edge cases.
#[tokio::test]
async fn test_message_expiration_edge_cases() {
    let comm = riglr_agents::ChannelCommunication::new();
    let agent_id = AgentId::new("expiration_test_agent");

    let _receiver = comm.subscribe(&agent_id).await.unwrap();

    // Test message that expires exactly now
    let mut message = AgentMessage::new(
        AgentId::new("sender"),
        Some(agent_id.clone()),
        "expiring_message".to_string(),
        json!({"test": "expiration"}),
    );

    // Set expiration to current time (edge case)
    message.expires_at = Some(chrono::Utc::now());

    // Small delay to ensure expiration
    tokio::time::sleep(Duration::from_millis(1)).await;

    let result = comm.send_message(message).await;

    // Should fail due to expiration
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("expired"));
}

/// Test handling of Unicode and special characters in all components.
#[tokio::test]
async fn test_unicode_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = AgentDispatcher::new(registry.clone());

    // Agent with Unicode ID
    let unicode_agent = Arc::new(EdgeCaseAgent::new(
        "🤖_エージェント_агент_代理",
        EdgeCaseBehavior::WeirdDataTypes,
    ));

    registry.register_agent(unicode_agent).await.unwrap();

    // Task with Unicode parameters
    let task = Task::new(
        TaskType::Custom("edge_case".to_string()),
        json!({
            "emoji": "🚀💎🦀",
            "japanese": "こんにちは",
            "chinese": "你好",
            "arabic": "مرحبا",
            "russian": "привет",
            "mixed": "Hello 世界 🌍"
        }),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();
    assert!(result.is_success());

    // Verify Unicode is preserved in results
    let data = result.data().unwrap();
    assert_eq!(data.get("unicode").unwrap().as_str().unwrap(), "🚀🦀💎");
}

/// Test boundary conditions for task retry counts.
#[tokio::test]
async fn test_retry_count_boundaries() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Test with zero retries
    let config_zero = DispatchConfig {
        max_retries: 0,
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };
    let dispatcher_zero = AgentDispatcher::with_config(registry.clone(), config_zero);

    let failing_agent = Arc::new(EdgeCaseAgent::new(
        "zero_retry_agent",
        EdgeCaseBehavior::FalseCapabilities,
    ));

    registry.register_agent(failing_agent).await.unwrap();

    let task = Task::new(
        TaskType::Custom("edge_case".to_string()),
        json!({"test": "zero_retries"}),
    );

    let result = dispatcher_zero.dispatch_task(task).await.unwrap();

    // Should fail immediately with no retries
    assert!(!result.is_success());
}

/// Test handling of system resource limits.
#[tokio::test]
#[ignore] // Ignore for regular runs as it may be resource-intensive
async fn test_resource_limit_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = AgentDispatcher::new(registry.clone());

    // Register many agents to test scaling limits
    for i in 0..1000 {
        let agent = Arc::new(EdgeCaseAgent::new(
            format!("scale_agent_{}", i),
            EdgeCaseBehavior::EmptyResult,
        ));
        registry.register_agent(agent).await.unwrap();
    }

    assert_eq!(registry.agent_count().await.unwrap(), 1000);

    // Create many concurrent tasks
    let tasks: Vec<Task> = (0..1000)
        .map(|i| {
            Task::new(
                TaskType::Custom("edge_case".to_string()),
                json!({"scale_test": i}),
            )
        })
        .collect();

    let start_time = std::time::Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let elapsed = start_time.elapsed();

    let success_count = results.iter().filter(|r| r.is_ok()).count();

    println!(
        "Scale test: {}/{} successful in {:?}",
        success_count, 1000, elapsed
    );

    // Should handle large scale reasonably well
    assert!(success_count >= 800); // Allow some failures under load
    assert!(elapsed < Duration::from_secs(30)); // Should complete in reasonable time
}
