use riglr_agents::*;
use std::sync::Arc;
use std::time::Duration;

// Import test utilities
use crate::common::*;

#[tokio::test]
async fn test_agent_failure_recovery() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register agents, including one configured to fail
    let failing_agent = Arc::new(
        MockTradingAgent::new("failing-trader", vec![CapabilityType::Trading]).with_failure(),
    );

    let backup_agent = Arc::new(MockTradingAgent::new(
        "backup-trader",
        vec![CapabilityType::Trading],
    ));

    registry.register_agent(failing_agent).await.unwrap();
    registry.register_agent(backup_agent).await.unwrap();

    let config = DispatchConfig {
        routing_strategy: RoutingStrategy::RoundRobin,
        max_retries: 2,
        retry_delay: Duration::from_millis(10),
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::with_config(registry, config);

    let task = create_trading_task();

    // Execute multiple tasks - some will fail, some will succeed
    let results = dispatcher
        .dispatch_tasks(vec![task.clone(), task.clone(), task])
        .await;

    // At least some tasks should succeed (from backup agent)
    let successful_count = results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_success())
        .count();

    assert!(successful_count > 0);
}

#[tokio::test]
async fn test_registry_capacity_errors() {
    // Test registry capacity limits
    let config = registry::RegistryConfig {
        max_agents: Some(1), // Very small capacity
        ..Default::default()
    };
    let registry = LocalAgentRegistry::with_config(config);

    let agent1 = Arc::new(MockTradingAgent::new(
        "agent1",
        vec![CapabilityType::Custom("test".to_string())],
    ));
    let agent2 = Arc::new(MockTradingAgent::new(
        "agent2",
        vec![CapabilityType::Custom("test".to_string())],
    ));

    // First registration should succeed
    assert!(registry.register_agent(agent1).await.is_ok());

    // Second registration should fail due to capacity
    let result = registry.register_agent(agent2).await;
    assert!(result.is_err());

    // Verify specific error type
    let error = result.unwrap_err();
    assert!(matches!(error, AgentError::Registry { .. }));
}

#[tokio::test]
async fn test_communication_error_handling() {
    let comm_system = ChannelCommunication::new();

    // Try to send message without subscription
    let message = create_test_message("sender", Some("nonexistent"), "test", serde_json::json!({}));

    let result = comm_system.send_message(message).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(error, AgentError::AgentNotFound { .. }));
}

#[tokio::test]
async fn test_task_timeout_error_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register an agent with very long execution delay
    let slow_agent = Arc::new(
        MockTradingAgent::new("very-slow", vec![CapabilityType::Trading])
            .with_delay(Duration::from_millis(500)),
    );

    registry.register_agent(slow_agent).await.unwrap();

    let config = DispatchConfig {
        default_task_timeout: Duration::from_millis(50), // Much shorter than agent delay
        max_retries: 0,                                  // No retries to test timeout directly
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::with_config(registry, config);

    let task = create_trading_task();
    let result = dispatcher.dispatch_task(task).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, AgentError::TaskTimeout { .. }));
}

#[tokio::test]
async fn test_no_suitable_agent_error() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register only trading agents
    let trading_agent = Arc::new(MockTradingAgent::new(
        "trader",
        vec![CapabilityType::Trading],
    ));
    registry.register_agent(trading_agent).await.unwrap();

    let dispatcher = AgentDispatcher::new(registry);

    // Try to execute a research task with no research agents
    let research_task = create_research_task();
    let result = dispatcher.dispatch_task(research_task).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, AgentError::NoSuitableAgent { .. }));
}

#[tokio::test]
async fn test_agent_unavailability_error() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register agents in various unavailable states
    let offline_agent = Arc::new(
        MockTradingAgent::new("offline", vec![CapabilityType::Trading])
            .with_state(AgentState::Offline),
    );

    let maintenance_agent = Arc::new(
        MockTradingAgent::new("maintenance", vec![CapabilityType::Trading])
            .with_state(AgentState::Maintenance),
    );

    let full_agent = Arc::new(
        MockTradingAgent::new("full", vec![CapabilityType::Trading]).with_state(AgentState::Full),
    );

    registry.register_agent(offline_agent).await.unwrap();
    registry.register_agent(maintenance_agent).await.unwrap();
    registry.register_agent(full_agent).await.unwrap();

    let dispatcher = AgentDispatcher::new(registry);

    // Try to execute task with only unavailable agents
    let task = create_trading_task();
    let result = dispatcher.dispatch_task(task).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, AgentError::NoSuitableAgent { .. }));
}

#[tokio::test]
async fn test_message_expiration_error() {
    let comm_system = ChannelCommunication::new();
    let agent_id = AgentId::new("test-agent");

    let _receiver = comm_system.subscribe(&agent_id).await.unwrap();

    // Create an expired message
    let mut message = create_test_message(
        "sender",
        Some("test-agent"),
        "expired",
        serde_json::json!({}),
    );
    message.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));

    let result = comm_system.send_message(message).await;
    assert!(result.is_err());

    // Verify error message content
    let error = result.unwrap_err();
    assert!(error.to_string().contains("expired"));
}

#[tokio::test]
async fn test_error_propagation_in_workflow() {
    let (agents, shared_state) = create_workflow_agents();

    let registry = Arc::new(LocalAgentRegistry::new());
    for agent in agents {
        registry.register_agent(agent).await.unwrap();
    }

    let dispatcher = AgentDispatcher::new(registry);

    // Try to execute risk analysis without market analysis first
    let risk_task = create_risk_analysis_task();
    let result = dispatcher.dispatch_task(risk_task).await.unwrap();

    // Should fail due to missing dependency
    assert!(!result.is_success());
    assert!(result.is_retriable());

    let error = extract_task_result_error(&result).unwrap();
    assert!(error.contains("Cannot perform risk analysis without market analysis"));

    // Verify shared state wasn't corrupted
    let state = shared_state.read().await;
    assert!(state.market_analysis.is_none());
    assert!(state.risk_assessment.is_none());
}

#[tokio::test]
async fn test_retry_exhaustion() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register an agent that always fails
    let always_failing_agent = Arc::new(
        MockTradingAgent::new("always-fails", vec![CapabilityType::Trading]).with_failure(),
    );

    registry.register_agent(always_failing_agent).await.unwrap();

    let config = DispatchConfig {
        max_retries: 3,
        retry_delay: Duration::from_millis(1),
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::with_config(registry, config);

    let task = create_trading_task();
    let result = dispatcher.dispatch_task(task).await.unwrap();

    // Should return the failure result after exhausting retries
    assert!(!result.is_success());
    assert!(result.is_retriable());

    let error = extract_task_result_error(&result).unwrap();
    assert!(error.contains("Mock agent configured to fail"));
}

#[tokio::test]
async fn test_concurrent_error_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register mix of working and failing agents
    let working_agent = Arc::new(MockTradingAgent::new(
        "working",
        vec![CapabilityType::Trading],
    ));

    let failing_agent =
        Arc::new(MockTradingAgent::new("failing", vec![CapabilityType::Trading]).with_failure());

    registry.register_agent(working_agent).await.unwrap();
    registry.register_agent(failing_agent).await.unwrap();

    let config = DispatchConfig {
        routing_strategy: RoutingStrategy::RoundRobin,
        max_retries: 1,
        retry_delay: Duration::from_millis(1),
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::with_config(registry, config);

    // Execute multiple tasks concurrently
    let tasks = create_load_test_tasks(10, TaskType::Trading);
    let results = dispatcher.dispatch_tasks(tasks).await;

    // Some should succeed, some should fail
    let success_count = results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_success())
        .count();

    let failure_count = results
        .iter()
        .filter(|r| r.is_ok() && !r.as_ref().unwrap().is_success())
        .count();

    assert!(success_count > 0);
    assert!(failure_count > 0);
    assert_eq!(success_count + failure_count, 10);
}

#[tokio::test]
async fn test_communication_subscription_errors() {
    let comm_system = ChannelCommunication::new();
    let agent_id = AgentId::new("test-agent");

    // First subscription should succeed
    let _receiver1 = comm_system.subscribe(&agent_id).await.unwrap();

    // Second subscription to same agent should fail
    let result = comm_system.subscribe(&agent_id).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error
        .to_string()
        .contains("already has an active subscription"));
}

#[tokio::test]
async fn test_unsubscribe_nonexistent_agent() {
    let comm_system = ChannelCommunication::new();
    let nonexistent_id = AgentId::new("nonexistent");

    let result = comm_system.unsubscribe(&nonexistent_id).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(error, AgentError::AgentNotFound { .. }));
}

#[tokio::test]
async fn test_error_recovery_with_graceful_degradation() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register agents with different failure rates
    let reliable_agent = Arc::new(MockTradingAgent::new(
        "reliable",
        vec![CapabilityType::Trading],
    ));

    let unreliable_agent =
        Arc::new(MockTradingAgent::new("unreliable", vec![CapabilityType::Trading]).with_failure());

    registry.register_agent(reliable_agent).await.unwrap();
    registry.register_agent(unreliable_agent).await.unwrap();

    let config = DispatchConfig {
        routing_strategy: RoutingStrategy::LeastLoaded,
        max_retries: 2,
        retry_delay: Duration::from_millis(10),
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::with_config(registry, config);

    // Execute tasks - system should gracefully handle failures
    let tasks = create_load_test_tasks(5, TaskType::Trading);
    let results = dispatcher.dispatch_tasks(tasks).await;

    // System should maintain some level of service despite failures
    let successful_results = results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_success())
        .count();

    // At least some tasks should succeed through the reliable agent
    assert!(successful_results >= 1);
}

#[tokio::test]
async fn test_dispatcher_health_check_with_failures() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Empty registry - should be unhealthy
    let dispatcher = AgentDispatcher::new(registry.clone());
    let health = dispatcher.health_check().await.unwrap();
    assert!(!health);

    // Add an agent - should become healthy
    let agent = Arc::new(MockTradingAgent::new(
        "trader",
        vec![CapabilityType::Trading],
    ));
    registry.register_agent(agent).await.unwrap();

    let health = dispatcher.health_check().await.unwrap();
    assert!(health);
}

#[tokio::test]
async fn test_error_context_preservation() {
    let registry = Arc::new(LocalAgentRegistry::new());

    let failing_agent = Arc::new(
        MockTradingAgent::new("context-failing", vec![CapabilityType::Trading]).with_failure(),
    );

    registry.register_agent(failing_agent).await.unwrap();

    let dispatcher = AgentDispatcher::new(registry);

    // Create task with specific parameters for error context
    let task = TestTaskBuilder::new(TaskType::Trading)
        .with_parameters(serde_json::json!({
            "symbol": "BTC/USD",
            "amount": 1000,
            "context": "error_context_test"
        }))
        .with_metadata("test_id", serde_json::json!("error_context_123"))
        .build();

    let result = dispatcher.dispatch_task(task).await.unwrap();

    // Error should preserve context about the failed task
    assert!(!result.is_success());

    let error = extract_task_result_error(&result).unwrap();
    assert!(error.contains("Mock agent configured to fail"));

    // The task ID should be preserved in the error context
    // (This would be verified through error tracing in a real system)
}
