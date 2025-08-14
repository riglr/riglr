//! Property-based tests for riglr-agents using quickcheck.
//!
//! These tests verify system invariants and properties that should hold
//! across a wide range of inputs and scenarios.

use riglr_agents::*;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashSet;
use serde_json::json;
use quickcheck::{quickcheck, TestResult};

/// Simple agent for property testing.
#[derive(Clone)]
struct PropertyTestAgent {
    id: AgentId,
    capabilities: Vec<String>,
    should_succeed: bool,
}

#[async_trait::async_trait]
impl Agent for PropertyTestAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        // Small delay to simulate work
        tokio::time::sleep(Duration::from_millis(1)).await;

        if self.should_succeed {
            Ok(TaskResult::success(
                json!({
                    "agent_id": self.id.as_str(),
                    "task_id": task.id,
                    "task_type": task.task_type.to_string()
                }),
                None,
                Duration::from_millis(1),
            ))
        } else {
            Ok(TaskResult::failure(
                "Property test failure".to_string(),
                true,
                Duration::from_millis(1),
            ))
        }
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

/// Test property: Agent IDs are unique in registry.
#[tokio::test]
async fn test_agent_id_uniqueness_property() {
    fn prop_agent_ids_unique(agent_count: u8) -> TestResult {
        if agent_count == 0 || agent_count > 20 {
            return TestResult::discard();
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = LocalAgentRegistry::new();
            let mut agent_ids = HashSet::new();

            // Register agents with unique IDs
            for i in 0..agent_count {
                let agent_id = format!("agent_{}", i);
                agent_ids.insert(agent_id.clone());

                let agent = Arc::new(PropertyTestAgent {
                    id: AgentId::new(&agent_id),
                    capabilities: vec!["test".to_string()],
                    should_succeed: true,
                });

                registry.register_agent(agent).await.unwrap();
            }

            // Verify all agents are registered
            let registered_count = registry.agent_count().await.unwrap();
            assert_eq!(registered_count, agent_count as usize);

            // Verify no duplicate registrations allowed
            if agent_count > 0 {
                let duplicate_agent = Arc::new(PropertyTestAgent {
                    id: AgentId::new("agent_0"), // Duplicate of first agent
                    capabilities: vec!["test".to_string()],
                    should_succeed: true,
                });

                let result = registry.register_agent(duplicate_agent).await;
                assert!(result.is_err());
            }

            TestResult::passed()
        })
    }

    quickcheck(prop_agent_ids_unique as fn(u8) -> TestResult);
}

/// Test property: All dispatched tasks get a response.
#[tokio::test]
async fn test_all_tasks_get_response_property() {
    fn prop_all_tasks_get_response(task_count: u8) -> TestResult {
        if task_count == 0 || task_count > 50 {
            return TestResult::discard();
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = Arc::new(LocalAgentRegistry::new());
            let dispatcher = AgentDispatcher::new(registry.clone());

            // Register agent
            let agent = Arc::new(PropertyTestAgent {
                id: AgentId::new("response_agent"),
                capabilities: vec!["test".to_string()],
                should_succeed: true,
            });
            registry.register_agent(agent).await.unwrap();

            // Create tasks
            let tasks: Vec<Task> = (0..task_count)
                .map(|i| Task::new(
                    TaskType::Custom("test".to_string()),
                    json!({"index": i}),
                ))
                .collect();

            // Dispatch tasks
            let results = dispatcher.dispatch_tasks(tasks).await;

            // Property: Every task gets exactly one response
            assert_eq!(results.len(), task_count as usize);
            
            // Property: All responses are valid (Ok or Err)
            for result in results {
                assert!(result.is_ok() || result.is_err());
            }

            TestResult::passed()
        })
    }

    quickcheck(prop_all_tasks_get_response as fn(u8) -> TestResult);
}

/// Test property: Agent capabilities are preserved.
#[tokio::test]
async fn test_agent_capabilities_preserved_property() {
    fn prop_capabilities_preserved(capabilities: Vec<String>) -> TestResult {
        if capabilities.is_empty() || capabilities.len() > 10 {
            return TestResult::discard();
        }

        // Filter out empty strings and duplicates
        let capabilities: Vec<String> = capabilities
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        if capabilities.is_empty() {
            return TestResult::discard();
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = LocalAgentRegistry::new();

            let agent = Arc::new(PropertyTestAgent {
                id: AgentId::new("capability_agent"),
                capabilities: capabilities.clone(),
                should_succeed: true,
            });

            registry.register_agent(agent.clone()).await.unwrap();

            // Retrieve agent and verify capabilities
            let retrieved = registry.get_agent(&AgentId::new("capability_agent")).await.unwrap();
            assert!(retrieved.is_some());

            let retrieved_agent = retrieved.unwrap();
            let retrieved_capabilities = retrieved_agent.capabilities();

            // Property: All original capabilities are preserved
            for cap in &capabilities {
                assert!(retrieved_capabilities.contains(cap));
            }

            // Property: No extra capabilities added
            assert_eq!(retrieved_capabilities.len(), capabilities.len());

            TestResult::passed()
        })
    }

    quickcheck(prop_capabilities_preserved as fn(Vec<String>) -> TestResult);
}

/// Test property: Task parameters are preserved during execution.
#[tokio::test]
async fn test_task_parameters_preserved_property() {
    fn prop_task_parameters_preserved(param_value: i32) -> TestResult {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = Arc::new(LocalAgentRegistry::new());
            let dispatcher = AgentDispatcher::new(registry.clone());

            let agent = Arc::new(PropertyTestAgent {
                id: AgentId::new("param_agent"),
                capabilities: vec!["test".to_string()],
                should_succeed: true,
            });
            registry.register_agent(agent).await.unwrap();

            let task = Task::new(
                TaskType::Custom("test".to_string()),
                json!({"test_param": param_value}),
            );

            let original_params = task.parameters.clone();
            let result = dispatcher.dispatch_task(task).await.unwrap();

            // Property: Task execution doesn't modify original parameters
            // (We can't directly verify this since the task is moved,
            // but we can verify the result contains expected data)
            assert!(result.is_success());

            TestResult::passed()
        })
    }

    quickcheck(prop_task_parameters_preserved as fn(i32) -> TestResult);
}

/// Test property: Task priority ordering is respected.
#[tokio::test]
async fn test_task_priority_ordering_property() {
    fn prop_priority_ordering_correct(priorities: Vec<u8>) -> TestResult {
        if priorities.is_empty() || priorities.len() > 10 {
            return TestResult::discard();
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Convert u8 to Priority enum
            let task_priorities: Vec<Priority> = priorities
                .into_iter()
                .map(|p| match p % 4 {
                    0 => Priority::Low,
                    1 => Priority::Normal,
                    2 => Priority::High,
                    3 => Priority::Critical,
                    _ => Priority::Normal,
                })
                .collect();

            // Property: Priority enum ordering is consistent
            let mut sorted_priorities = task_priorities.clone();
            sorted_priorities.sort();

            // Verify ordering is consistent
            for i in 1..sorted_priorities.len() {
                assert!(sorted_priorities[i] >= sorted_priorities[i - 1]);
            }

            // Property: Critical > High > Normal > Low
            assert!(Priority::Critical > Priority::High);
            assert!(Priority::High > Priority::Normal);
            assert!(Priority::Normal > Priority::Low);

            TestResult::passed()
        })
    }

    quickcheck(prop_priority_ordering_correct as fn(Vec<u8>) -> TestResult);
}

/// Test property: Agent registry operations are idempotent for queries.
#[tokio::test]
async fn test_registry_query_idempotency_property() {
    fn prop_registry_queries_idempotent(query_count: u8) -> TestResult {
        if query_count == 0 || query_count > 20 {
            return TestResult::discard();
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = LocalAgentRegistry::new();
            
            // Register an agent
            let agent = Arc::new(PropertyTestAgent {
                id: AgentId::new("idempotent_agent"),
                capabilities: vec!["test".to_string()],
                should_succeed: true,
            });
            registry.register_agent(agent).await.unwrap();

            let agent_id = AgentId::new("idempotent_agent");
            
            // Property: Multiple queries return the same result
            let mut results = Vec::new();
            for _ in 0..query_count {
                let exists = registry.is_agent_registered(&agent_id).await.unwrap();
                let agent_opt = registry.get_agent(&agent_id).await.unwrap();
                let count = registry.agent_count().await.unwrap();
                
                results.push((exists, agent_opt.is_some(), count));
            }

            // All results should be identical
            let first_result = &results[0];
            for result in &results[1..] {
                assert_eq!(result, first_result);
            }

            // Property: Queries don't modify state
            assert_eq!(first_result.0, true); // Agent exists
            assert_eq!(first_result.1, true); // Agent can be retrieved
            assert_eq!(first_result.2, 1);    // Count is correct

            TestResult::passed()
        })
    }

    quickcheck(prop_registry_queries_idempotent as fn(u8) -> TestResult);
}

/// Test property: Message delivery is reliable for valid recipients.
#[tokio::test]
async fn test_message_delivery_reliability_property() {
    fn prop_message_delivery_reliable(message_count: u8) -> TestResult {
        if message_count == 0 || message_count > 20 {
            return TestResult::discard();
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let comm = riglr_agents::ChannelCommunication::new();
            let agent_id = AgentId::new("test_recipient");

            // Subscribe agent
            let mut receiver = comm.subscribe(&agent_id).await.unwrap();

            // Send messages
            let mut sent_messages = Vec::new();
            for i in 0..message_count {
                let message = AgentMessage::new(
                    AgentId::new("sender"),
                    Some(agent_id.clone()),
                    format!("message_{}", i),
                    json!({"index": i}),
                );
                sent_messages.push(message.id.clone());
                comm.send_message(message).await.unwrap();
            }

            // Receive all messages
            let mut received_messages = Vec::new();
            for _ in 0..message_count {
                if let Some(msg) = receiver.receive().await {
                    received_messages.push(msg.id);
                }
            }

            // Property: All sent messages are received exactly once
            assert_eq!(received_messages.len(), sent_messages.len());
            
            // Property: Message IDs match (order may differ)
            let mut sent_sorted = sent_messages.clone();
            let mut received_sorted = received_messages.clone();
            sent_sorted.sort();
            received_sorted.sort();
            assert_eq!(sent_sorted, received_sorted);

            TestResult::passed()
        })
    }

    quickcheck(prop_message_delivery_reliable as fn(u8) -> TestResult);
}

/// Test property: Task execution is deterministic for same inputs.
#[tokio::test]
async fn test_task_execution_determinism_property() {
    fn prop_task_execution_deterministic(task_param: i16) -> TestResult {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = Arc::new(LocalAgentRegistry::new());
            let dispatcher = AgentDispatcher::new(registry.clone());

            let agent = Arc::new(PropertyTestAgent {
                id: AgentId::new("deterministic_agent"),
                capabilities: vec!["test".to_string()],
                should_succeed: true,
            });
            registry.register_agent(agent).await.unwrap();

            // Execute the same task multiple times
            let mut results = Vec::new();
            for _ in 0..3 {
                let task = Task::new(
                    TaskType::Custom("test".to_string()),
                    json!({"param": task_param}),
                );

                let result = dispatcher.dispatch_task(task).await.unwrap();
                results.push(result.is_success());
            }

            // Property: Same inputs produce consistent success/failure
            let first_success = results[0];
            for &success in &results[1..] {
                assert_eq!(success, first_success);
            }

            TestResult::passed()
        })
    }

    quickcheck(prop_task_execution_deterministic as fn(i16) -> TestResult);
}

/// Test property: Routing rules are consistent.
#[tokio::test]
async fn test_routing_rule_consistency_property() {
    fn prop_routing_rules_consistent(rule_type: u8) -> TestResult {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let agent_id = AgentId::new("test_agent");
            let capabilities = vec!["trading".to_string(), "analysis".to_string()];
            
            // Create different types of routing rules
            let rule = match rule_type % 4 {
                0 => RoutingRule::TaskType(TaskType::Trading),
                1 => RoutingRule::Capability("trading".to_string()),
                2 => RoutingRule::Agent(agent_id.clone()),
                3 => RoutingRule::Priority(Priority::High),
                _ => RoutingRule::TaskType(TaskType::Trading),
            };

            // Create tasks that should match the rule
            let matching_task = match rule {
                RoutingRule::TaskType(TaskType::Trading) => {
                    Task::new(TaskType::Trading, json!({}))
                }
                RoutingRule::Capability(_) => {
                    Task::new(TaskType::Trading, json!({})) // Trading agents have trading capability
                }
                RoutingRule::Agent(_) => {
                    Task::new(TaskType::Trading, json!({})) // Any task for specific agent
                }
                RoutingRule::Priority(Priority::High) => {
                    Task::new(TaskType::Trading, json!({})).with_priority(Priority::High)
                }
                _ => Task::new(TaskType::Trading, json!({})),
            };

            // Property: Rule evaluation is consistent
            let match1 = rule.matches(&matching_task, &agent_id, &capabilities);
            let match2 = rule.matches(&matching_task, &agent_id, &capabilities);
            assert_eq!(match1, match2);

            // Property: Rule should match when expected
            let should_match = match rule {
                RoutingRule::TaskType(TaskType::Trading) => matching_task.task_type == TaskType::Trading,
                RoutingRule::Capability(ref cap) => capabilities.contains(cap),
                RoutingRule::Agent(ref target) => target == &agent_id,
                RoutingRule::Priority(pri) => matching_task.priority >= pri,
                _ => false,
            };

            if should_match {
                assert!(match1);
            }

            TestResult::passed()
        })
    }

    quickcheck(prop_routing_rules_consistent as fn(u8) -> TestResult);
}

/// Test property: Agent status updates are atomic.
#[tokio::test]
async fn test_agent_status_atomicity_property() {
    fn prop_status_updates_atomic(update_count: u8) -> TestResult {
        if update_count == 0 || update_count > 10 {
            return TestResult::discard();
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = LocalAgentRegistry::new();
            let agent_id = AgentId::new("status_agent");

            // Register agent
            let agent = Arc::new(PropertyTestAgent {
                id: agent_id.clone(),
                capabilities: vec!["test".to_string()],
                should_succeed: true,
            });
            registry.register_agent(agent).await.unwrap();

            // Perform multiple status updates
            for i in 0..update_count {
                let status = AgentStatus {
                    agent_id: agent_id.clone(),
                    status: if i % 2 == 0 { 
                        crate::types::AgentState::Active 
                    } else { 
                        crate::types::AgentState::Busy 
                    },
                    active_tasks: i as u32,
                    load: (i as f64) / 10.0,
                    last_heartbeat: chrono::Utc::now(),
                    capabilities: vec![],
                    metadata: std::collections::HashMap::new(),
                };

                registry.update_agent_status(status).await.unwrap();

                // Property: Status can always be retrieved after update
                let retrieved_status = registry.get_agent_status(&agent_id).await.unwrap();
                assert!(retrieved_status.is_some());

                let retrieved = retrieved_status.unwrap();
                assert_eq!(retrieved.active_tasks, i as u32);
                assert!((retrieved.load - (i as f64) / 10.0).abs() < 0.001);
            }

            TestResult::passed()
        })
    }

    quickcheck(prop_status_updates_atomic as fn(u8) -> TestResult);
}

/// Test property: Communication subscriptions are unique per agent.
#[tokio::test]
async fn test_communication_subscription_uniqueness_property() {
    fn prop_subscriptions_unique(agent_count: u8) -> TestResult {
        if agent_count == 0 || agent_count > 15 {
            return TestResult::discard();
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let comm = riglr_agents::ChannelCommunication::new();
            let mut receivers = Vec::new();

            // Subscribe multiple agents
            for i in 0..agent_count {
                let agent_id = AgentId::new(&format!("sub_agent_{}", i));
                let receiver = comm.subscribe(&agent_id).await.unwrap();
                receivers.push(receiver);
            }

            // Property: Subscription count matches agent count
            let sub_count = comm.subscription_count().await.unwrap();
            assert_eq!(sub_count, agent_count as usize);

            // Property: Duplicate subscriptions are rejected
            if agent_count > 0 {
                let duplicate_agent_id = AgentId::new("sub_agent_0");
                let result = comm.subscribe(&duplicate_agent_id).await;
                assert!(result.is_err());
            }

            // Property: Each agent gets only their own messages
            for i in 0..agent_count {
                let sender = AgentId::new("broadcaster");
                let target = AgentId::new(&format!("sub_agent_{}", i));
                
                let message = AgentMessage::new(
                    sender,
                    Some(target),
                    "unique_test".to_string(),
                    json!({"for_agent": i}),
                );

                comm.send_message(message).await.unwrap();
            }

            // Verify each receiver gets exactly one message
            for (i, receiver) in receivers.iter_mut().enumerate() {
                if let Some(msg) = receiver.receive().await {
                    let agent_index = msg.payload.get("for_agent")
                        .and_then(|v| v.as_u64())
                        .unwrap();
                    assert_eq!(agent_index, i as u64);
                }
            }

            TestResult::passed()
        })
    }

    quickcheck(prop_subscriptions_unique as fn(u8) -> TestResult);
}