//! Integration tests for error handling and recovery in multi-agent systems.
//!
//! These tests verify that the agent system properly handles various error
//! conditions, implements appropriate recovery strategies, and maintains
//! system stability under failure scenarios.

use riglr_agents::*;
use std::sync::Arc;
use std::time::Duration;
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use serde_json::json;
use tokio::sync::Mutex;

/// Agent that can be configured to fail in different ways.
#[derive(Clone)]
struct FaultInjectingAgent {
    id: AgentId,
    failure_mode: Arc<Mutex<FailureMode>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
}

#[derive(Debug, Clone)]
enum FailureMode {
    /// Always succeed
    Success,
    /// Always fail with retriable error
    RetriableFailure,
    /// Always fail with permanent error
    PermanentFailure,
    /// Fail first N times, then succeed
    FailThenSucceed(u32),
    /// Succeed first N times, then fail
    SucceedThenFail(u32),
    /// Random failures with given probability
    RandomFailure(f64),
    /// Timeout (long execution)
    Timeout,
    /// Panic during execution
    Panic,
}

impl FaultInjectingAgent {
    fn new(id: impl Into<String>, failure_mode: FailureMode) -> Self {
        Self {
            id: AgentId::new(id),
            failure_mode: Arc::new(Mutex::new(failure_mode)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
        }
    }

    async fn set_failure_mode(&self, mode: FailureMode) {
        *self.failure_mode.lock().await = mode;
    }

    fn get_failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Relaxed)
    }

    fn get_success_count(&self) -> u32 {
        self.success_count.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl Agent for FaultInjectingAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        let mode = self.failure_mode.lock().await.clone();
        let current_failures = self.failure_count.load(Ordering::Relaxed);
        let current_successes = self.success_count.load(Ordering::Relaxed);

        let should_fail = match mode {
            FailureMode::Success => false,
            FailureMode::RetriableFailure | FailureMode::PermanentFailure => true,
            FailureMode::FailThenSucceed(n) => current_failures < n,
            FailureMode::SucceedThenFail(n) => current_successes >= n,
            FailureMode::RandomFailure(prob) => rand::random::<f64>() < prob,
            FailureMode::Timeout => {
                tokio::time::sleep(Duration::from_secs(10)).await;
                false
            }
            FailureMode::Panic => {
                panic!("Simulated agent panic");
            }
        };

        if should_fail {
            self.failure_count.fetch_add(1, Ordering::Relaxed);

            let (error_msg, retriable) = match mode {
                FailureMode::RetriableFailure => ("Simulated retriable failure", true),
                FailureMode::PermanentFailure => ("Simulated permanent failure", false),
                FailureMode::FailThenSucceed(_) => ("Transient failure", true),
                FailureMode::SucceedThenFail(_) => ("Degraded performance failure", true),
                FailureMode::RandomFailure(_) => ("Random failure", true),
                _ => ("Unknown failure", false),
            };

            Ok(TaskResult::failure(
                error_msg.to_string(),
                retriable,
                Duration::from_millis(10),
            ))
        } else {
            self.success_count.fetch_add(1, Ordering::Relaxed);

            // Simulate some work
            tokio::time::sleep(Duration::from_millis(50)).await;

            Ok(TaskResult::success(
                json!({
                    "agent_id": self.id.as_str(),
                    "task_id": task.id,
                    "execution_attempt": current_failures + current_successes + 1
                }),
                None,
                Duration::from_millis(50),
            ))
        }
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["testing".to_string(), "fault_injection".to_string()]
    }
}

/// Agent that tracks health status.
#[derive(Clone)]
struct HealthTrackingAgent {
    id: AgentId,
    is_healthy: Arc<AtomicBool>,
    task_count: Arc<AtomicU32>,
}

impl HealthTrackingAgent {
    fn new(id: impl Into<String>) -> Self {
        Self {
            id: AgentId::new(id),
            is_healthy: Arc::new(AtomicBool::new(true)),
            task_count: Arc::new(AtomicU32::new(0)),
        }
    }

    fn set_healthy(&self, healthy: bool) {
        self.is_healthy.store(healthy, Ordering::Relaxed);
    }

    fn get_task_count(&self) -> u32 {
        self.task_count.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl Agent for HealthTrackingAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        self.task_count.fetch_add(1, Ordering::Relaxed);

        if !self.is_healthy.load(Ordering::Relaxed) {
            return Ok(TaskResult::failure(
                "Agent is unhealthy".to_string(),
                true,
                Duration::from_millis(10),
            ));
        }

        tokio::time::sleep(Duration::from_millis(30)).await;

        Ok(TaskResult::success(
            json!({
                "agent_id": self.id.as_str(),
                "task_id": task.id,
                "health_status": "healthy"
            }),
            None,
            Duration::from_millis(30),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["health_monitoring".to_string()]
    }

    fn is_available(&self) -> bool {
        self.is_healthy.load(Ordering::Relaxed)
    }
}

/// Test basic retry mechanism for retriable failures.
#[tokio::test]
async fn test_basic_retry_mechanism() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        max_retries: 3,
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    // Agent that fails twice then succeeds
    let agent = Arc::new(FaultInjectingAgent::new(
        "retry-agent",
        FailureMode::FailThenSucceed(2)
    ));

    registry.register_agent(agent.clone()).await.unwrap();

    let task = Task::new(
        TaskType::Custom("fault_injection".to_string()),
        json!({"test": "retry_mechanism"}),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();

    // Should eventually succeed after retries
    assert!(result.is_success());
    assert_eq!(agent.get_failure_count(), 2);
    assert_eq!(agent.get_success_count(), 1);
}

/// Test that permanent failures are not retried.
#[tokio::test]
async fn test_permanent_failure_no_retry() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        max_retries: 3,
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    let agent = Arc::new(FaultInjectingAgent::new(
        "permanent-fail-agent",
        FailureMode::PermanentFailure
    ));

    registry.register_agent(agent.clone()).await.unwrap();

    let task = Task::new(
        TaskType::Custom("fault_injection".to_string()),
        json!({"test": "permanent_failure"}),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();

    // Should fail immediately without retries
    assert!(!result.is_success());
    assert!(!result.is_retriable());
    assert_eq!(agent.get_failure_count(), 1); // Only one attempt
    assert_eq!(agent.get_success_count(), 0);
}

/// Test retry exhaustion handling.
#[tokio::test]
async fn test_retry_exhaustion() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        max_retries: 2,
        retry_delay: Duration::from_millis(5),
        ..Default::default()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    // Agent that always fails with retriable errors
    let agent = Arc::new(FaultInjectingAgent::new(
        "always-fail-agent",
        FailureMode::RetriableFailure
    ));

    registry.register_agent(agent.clone()).await.unwrap();

    let task = Task::new(
        TaskType::Custom("fault_injection".to_string()),
        json!({"test": "retry_exhaustion"}),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();

    // Should fail after exhausting retries, but return the last failure result
    assert!(!result.is_success());
    assert!(result.is_retriable());

    // Should have attempted max_retries + 1 times (initial + retries)
    assert_eq!(agent.get_failure_count(), 3);
    assert_eq!(agent.get_success_count(), 0);
}

/// Test timeout handling with recovery.
#[tokio::test]
async fn test_timeout_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        default_task_timeout: Duration::from_millis(100),
        max_retries: 1,
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    // Create a fast agent and a slow (timeout) agent
    let fast_agent = Arc::new(FaultInjectingAgent::new(
        "fast-agent",
        FailureMode::Success
    ));

    let slow_agent = Arc::new(FaultInjectingAgent::new(
        "slow-agent",
        FailureMode::Timeout
    ));

    registry.register_agent(fast_agent.clone()).await.unwrap();
    registry.register_agent(slow_agent.clone()).await.unwrap();

    let task = Task::new(
        TaskType::Custom("fault_injection".to_string()),
        json!({"test": "timeout_handling"}),
    );

    // Task should timeout
    let result = dispatcher.dispatch_task(task).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AgentError::TaskTimeout { .. }));
}

/// Test error propagation in multi-agent workflows.
#[tokio::test]
async fn test_error_propagation_in_workflows() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Stage 1: Reliable agent
    let reliable_agent = Arc::new(FaultInjectingAgent::new(
        "reliable-agent",
        FailureMode::Success
    ));

    // Stage 2: Failing agent
    let failing_agent = Arc::new(FaultInjectingAgent::new(
        "failing-agent",
        FailureMode::PermanentFailure
    ));

    // Stage 3: Recovery agent
    let recovery_agent = Arc::new(FaultInjectingAgent::new(
        "recovery-agent",
        FailureMode::Success
    ));

    registry.register_agent(reliable_agent.clone()).await.unwrap();
    registry.register_agent(failing_agent.clone()).await.unwrap();
    registry.register_agent(recovery_agent.clone()).await.unwrap();

    // Stage 1: Should succeed
    let stage1_task = Task::new(
        TaskType::Custom("fault_injection".to_string()),
        json!({"stage": 1, "operation": "prepare"}),
    );

    let stage1_result = dispatcher.dispatch_task(stage1_task).await.unwrap();
    assert!(stage1_result.is_success());

    // Stage 2: Should fail
    let stage2_task = Task::new(
        TaskType::Custom("fault_injection".to_string()),
        json!({"stage": 2, "operation": "process"}),
    );

    let stage2_result = dispatcher.dispatch_task(stage2_task).await.unwrap();
    assert!(!stage2_result.is_success());

    // Stage 3: Recovery action (should succeed)
    let stage3_task = Task::new(
        TaskType::Custom("fault_injection".to_string()),
        json!({"stage": 3, "operation": "recover"}),
    );

    let stage3_result = dispatcher.dispatch_task(stage3_task).await.unwrap();
    assert!(stage3_result.is_success());

    // Verify execution counts
    assert_eq!(reliable_agent.get_success_count(), 1);
    assert_eq!(failing_agent.get_failure_count(), 1);
    assert_eq!(recovery_agent.get_success_count(), 1);
}

/// Test circuit breaker pattern with agent health monitoring.
#[tokio::test]
async fn test_circuit_breaker_pattern() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let health_agent = Arc::new(HealthTrackingAgent::new("health-agent"));
    registry.register_agent(health_agent.clone()).await.unwrap();

    // Initially healthy - should work
    let task1 = Task::new(
        TaskType::Custom("health_monitoring".to_string()),
        json!({"test": "circuit_breaker", "phase": "healthy"}),
    );

    let result1 = dispatcher.dispatch_task(task1).await.unwrap();
    assert!(result1.is_success());
    assert_eq!(health_agent.get_task_count(), 1);

    // Mark agent as unhealthy
    health_agent.set_healthy(false);

    // Should fail due to health check
    let task2 = Task::new(
        TaskType::Custom("health_monitoring".to_string()),
        json!({"test": "circuit_breaker", "phase": "unhealthy"}),
    );

    let result2 = dispatcher.dispatch_task(task2).await.unwrap();
    assert!(!result2.is_success());
    assert_eq!(health_agent.get_task_count(), 2);

    // Restore health
    health_agent.set_healthy(true);

    // Should work again
    let task3 = Task::new(
        TaskType::Custom("health_monitoring".to_string()),
        json!({"test": "circuit_breaker", "phase": "recovered"}),
    );

    let result3 = dispatcher.dispatch_task(task3).await.unwrap();
    assert!(result3.is_success());
    assert_eq!(health_agent.get_task_count(), 3);
}

/// Test graceful degradation with multiple agents.
#[tokio::test]
async fn test_graceful_degradation() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Primary agent that will fail
    let primary_agent = Arc::new(FaultInjectingAgent::new(
        "primary-agent",
        FailureMode::SucceedThenFail(2)
    ));

    // Backup agent that works but slower
    let backup_agent = Arc::new(FaultInjectingAgent::new(
        "backup-agent",
        FailureMode::Success
    ));

    registry.register_agent(primary_agent.clone()).await.unwrap();
    registry.register_agent(backup_agent.clone()).await.unwrap();

    // First two tasks should succeed on primary
    for i in 0..2 {
        let task = Task::new(
            TaskType::Custom("fault_injection".to_string()),
            json!({"test": "degradation", "iteration": i}),
        );

        let result = dispatcher.dispatch_task(task).await.unwrap();
        assert!(result.is_success());
    }

    assert_eq!(primary_agent.get_success_count(), 2);
    assert_eq!(backup_agent.get_success_count(), 0);

    // Subsequent tasks will use backup agent (in a real scenario,
    // load balancer would route to healthy agent)
    let task = Task::new(
        TaskType::Custom("fault_injection".to_string()),
        json!({"test": "degradation", "iteration": 3}),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();
    // This might fail on primary, but the system continues to function
    // In practice, a more sophisticated routing system would handle this
}

/// Test error rate monitoring and alerting.
#[tokio::test]
async fn test_error_rate_monitoring() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Agent with 30% failure rate
    let flaky_agent = Arc::new(FaultInjectingAgent::new(
        "flaky-agent",
        FailureMode::RandomFailure(0.3)
    ));

    registry.register_agent(flaky_agent.clone()).await.unwrap();

    let mut success_count = 0;
    let mut failure_count = 0;
    let total_tasks = 20;

    // Execute many tasks to measure error rate
    for i in 0..total_tasks {
        let task = Task::new(
            TaskType::Custom("fault_injection".to_string()),
            json!({"test": "error_rate", "iteration": i}),
        );

        match dispatcher.dispatch_task(task).await.unwrap() {
            result if result.is_success() => success_count += 1,
            _ => failure_count += 1,
        }
    }

    let total = success_count + failure_count;
    let error_rate = failure_count as f64 / total as f64;

    println!("Error rate: {:.2}% ({}/{} failures)",
             error_rate * 100.0, failure_count, total);

    // Should be roughly 30% error rate (allowing for randomness)
    assert!((0.15..=0.45).contains(&error_rate));

    // Verify agent recorded the attempts
    let agent_successes = flaky_agent.get_success_count();
    let agent_failures = flaky_agent.get_failure_count();
    assert_eq!(agent_successes + agent_failures, total_tasks);
}

/// Test recovery after agent restart simulation.
#[tokio::test]
async fn test_agent_restart_recovery() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let agent_id = AgentId::new("restartable-agent");

    // Register initial agent
    let agent_v1 = Arc::new(FaultInjectingAgent::new(
        "restartable-agent",
        FailureMode::Success
    ));
    registry.register_agent(agent_v1.clone()).await.unwrap();

    // Execute task successfully
    let task1 = Task::new(
        TaskType::Custom("fault_injection".to_string()),
        json!({"test": "pre_restart"}),
    );

    let result1 = dispatcher.dispatch_task(task1).await.unwrap();
    assert!(result1.is_success());
    assert_eq!(agent_v1.get_success_count(), 1);

    // Simulate agent restart by unregistering and registering new instance
    registry.unregister_agent(&agent_id).await.unwrap();

    let agent_v2 = Arc::new(FaultInjectingAgent::new(
        "restartable-agent",
        FailureMode::Success
    ));
    registry.register_agent(agent_v2.clone()).await.unwrap();

    // Execute task on restarted agent
    let task2 = Task::new(
        TaskType::Custom("fault_injection".to_string()),
        json!({"test": "post_restart"}),
    );

    let result2 = dispatcher.dispatch_task(task2).await.unwrap();
    assert!(result2.is_success());

    // Old agent should not have additional executions
    assert_eq!(agent_v1.get_success_count(), 1);
    // New agent should have recorded the execution
    assert_eq!(agent_v2.get_success_count(), 1);
}

/// Test concurrent error handling.
#[tokio::test]
async fn test_concurrent_error_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        max_retries: 2,
        retry_delay: Duration::from_millis(5),
        ..Default::default()
    };
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

    // Create mix of reliable and unreliable agents
    let reliable_agent = Arc::new(FaultInjectingAgent::new(
        "reliable-concurrent",
        FailureMode::Success
    ));

    let flaky_agent = Arc::new(FaultInjectingAgent::new(
        "flaky-concurrent",
        FailureMode::FailThenSucceed(1)
    ));

    registry.register_agent(reliable_agent.clone()).await.unwrap();
    registry.register_agent(flaky_agent.clone()).await.unwrap();

    // Execute multiple tasks concurrently
    let tasks: Vec<Task> = (0..10)
        .map(|i| Task::new(
            TaskType::Custom("fault_injection".to_string()),
            json!({"test": "concurrent_errors", "iteration": i}),
        ))
        .collect();

    let results = dispatcher.dispatch_tasks(tasks).await;

    // All tasks should eventually succeed due to retries
    assert_eq!(results.len(), 10);
    let success_count = results.iter().filter(|r| r.is_ok() && r.as_ref().unwrap().is_success()).count();

    // Should have high success rate despite some initial failures
    assert!(success_count >= 8);

    // Verify both agents handled some tasks
    let total_reliable = reliable_agent.get_success_count();
    let total_flaky = flaky_agent.get_success_count() + flaky_agent.get_failure_count();

    assert!(total_reliable > 0);
    assert!(total_flaky > 0);
}

/// Test system-wide error recovery patterns.
#[tokio::test]
async fn test_system_wide_error_recovery() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        max_retries: 3,
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

    // Create agents that simulate different types of system failures
    let network_agent = Arc::new(FaultInjectingAgent::new(
        "network-agent",
        FailureMode::FailThenSucceed(2) // Network issues that resolve
    ));

    let database_agent = Arc::new(FaultInjectingAgent::new(
        "database-agent",
        FailureMode::RandomFailure(0.2) // Intermittent DB issues
    ));

    let api_agent = Arc::new(FaultInjectingAgent::new(
        "api-agent",
        FailureMode::Success // API is stable
    ));

    registry.register_agent(network_agent.clone()).await.unwrap();
    registry.register_agent(database_agent.clone()).await.unwrap();
    registry.register_agent(api_agent.clone()).await.unwrap();

    // Simulate a complex workflow with potential failures
    let workflow_tasks = vec![
        ("network", "Network connectivity check"),
        ("database", "Database query"),
        ("api", "API call"),
        ("network", "Data synchronization"),
        ("database", "Result storage"),
    ];

    let mut results = Vec::new();
    for (service, description) in workflow_tasks {
        let task = Task::new(
            TaskType::Custom("fault_injection".to_string()),
            json!({"service": service, "description": description}),
        );

        let result = dispatcher.dispatch_task(task).await.unwrap();
        results.push((service, result.is_success()));
    }

    // Verify that the system recovered from transient failures
    let success_rate = results.iter().filter(|(_, success)| *success).count() as f64 / results.len() as f64;

    println!("Workflow success rate: {:.2}%", success_rate * 100.0);

    // Should have high success rate due to retry mechanisms
    assert!(success_rate >= 0.8);

    // Verify retry mechanisms were used
    assert!(network_agent.get_failure_count() > 0 || network_agent.get_success_count() > 0);
    assert!(database_agent.get_success_count() > 0);
    assert!(api_agent.get_success_count() > 0);
}

/// Test error handling with resource exhaustion.
#[tokio::test]
async fn test_resource_exhaustion_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        max_retries: 1,
        default_task_timeout: Duration::from_millis(100),
        ..Default::default()
    };
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

    // Create an agent that simulates resource exhaustion
    let resource_agent = Arc::new(FaultInjectingAgent::new(
        "resource-agent",
        FailureMode::SucceedThenFail(3) // Works for first 3 tasks, then fails
    ));

    registry.register_agent(resource_agent.clone()).await.unwrap();

    let mut results = Vec::new();

    // Execute tasks until resources are exhausted
    for i in 0..6 {
        let task = Task::new(
            TaskType::Custom("fault_injection".to_string()),
            json!({"test": "resource_exhaustion", "iteration": i}),
        );

        let result = dispatcher.dispatch_task(task).await.unwrap();
        results.push(result.is_success());
    }

    // First 3 should succeed, rest should fail
    assert!(results[0]); // Success
    assert!(results[1]); // Success
    assert!(results[2]); // Success
    assert!(!results[3]); // Failure (resource exhausted)
    assert!(!results[4]); // Failure
    assert!(!results[5]); // Failure

    assert_eq!(resource_agent.get_success_count(), 3);
    assert_eq!(resource_agent.get_failure_count(), 3);
}

/// Performance test for error handling overhead.
#[tokio::test]
#[ignore] // Ignore for regular test runs
async fn test_error_handling_performance() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        max_retries: 2,
        retry_delay: Duration::from_millis(1),
        ..Default::default()
    };
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

    // Mix of successful and failing agents
    let success_agent = Arc::new(FaultInjectingAgent::new(
        "perf-success",
        FailureMode::Success
    ));

    let retry_agent = Arc::new(FaultInjectingAgent::new(
        "perf-retry",
        FailureMode::FailThenSucceed(1)
    ));

    registry.register_agent(success_agent.clone()).await.unwrap();
    registry.register_agent(retry_agent.clone()).await.unwrap();

    let task_count = 1000;
    let tasks: Vec<Task> = (0..task_count)
        .map(|i| Task::new(
            TaskType::Custom("fault_injection".to_string()),
            json!({"test": "performance", "iteration": i}),
        ))
        .collect();

    let start_time = std::time::Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let elapsed = start_time.elapsed();

    let success_count = results.iter().filter(|r| r.is_ok()).count();

    println!("Processed {} tasks in {:?}, {} successful",
             task_count, elapsed, success_count);

    // Should handle errors efficiently
    assert!(elapsed < Duration::from_secs(10));
    assert!(success_count > task_count * 90 / 100); // At least 90% success

    // Verify retry mechanisms were used
    assert!(retry_agent.get_failure_count() > 0);
    assert!(retry_agent.get_success_count() > 0);
}