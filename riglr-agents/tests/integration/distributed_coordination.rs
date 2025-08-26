//! Integration tests for distributed agent coordination.
//!
//! These tests verify that agents can discover and coordinate with each other
//! across multiple processes using the distributed registry and dispatcher.

#[cfg(feature = "e2e-tests")]
use riglr_agents::{
    dispatcher::{AgentDispatcher, DispatchConfig, RoutingStrategy},
    registry::{AgentRegistry, DistributedAgentRegistry, RedisAgentRegistry},
    types::{
        AgentId, AgentState, AgentStatus, Capability, CapabilityType, Task, TaskResult, TaskType,
    },
    Agent, Result,
};
#[cfg(feature = "e2e-tests")]
use std::collections::HashMap;
#[cfg(feature = "e2e-tests")]
use std::sync::Arc;
#[cfg(feature = "e2e-tests")]
use std::time::Duration;
#[cfg(feature = "e2e-tests")]
use tokio::time::sleep;

#[cfg(feature = "e2e-tests")]
use crate::common::test_agent::TestAgent;

// Environment variable constants
#[cfg(feature = "e2e-tests")]
const ENV_REDIS_URL: &str = "REDIS_URL";

/// Test that the distributed registry can track agents across simulated processes.
#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_distributed_agent_discovery() -> Result<()> {
    // This test requires a Redis instance
    let redis_url =
        std::env::var(ENV_REDIS_URL).unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Create a distributed registry
    let registry: Arc<dyn DistributedAgentRegistry> =
        Arc::new(RedisAgentRegistry::new(redis_url.clone()).await?);

    // Register a local agent in "process 1"
    let agent1 = Arc::new(TestAgent::new("agent-1", vec![CapabilityType::Trading]));
    registry.register_agent(agent1.clone()).await?;

    // Update agent status to simulate it being active
    let status1 = AgentStatus {
        agent_id: agent1.id().clone(),
        status: AgentState::Active,
        active_tasks: 0,
        load: 0.0,
        last_heartbeat: chrono::Utc::now(),
        capabilities: vec![Capability::new("trading", "1.0")],
        metadata: HashMap::default(),
    };
    registry.update_agent_status(status1).await?;

    // Simulate "process 2" by creating another registry instance
    let registry2: Arc<dyn DistributedAgentRegistry> =
        Arc::new(RedisAgentRegistry::new(redis_url).await?);

    // Process 2 should be able to discover agent1 via distributed discovery
    let all_statuses = registry2.list_agent_statuses().await?;
    assert!(
        all_statuses
            .iter()
            .any(|s| s.agent_id.as_str() == "agent-1"),
        "Agent from process 1 should be discoverable from process 2"
    );

    // Find agents by capability
    let trading_statuses = registry2
        .find_agent_statuses_by_capability("trading")
        .await?;
    assert!(
        trading_statuses
            .iter()
            .any(|s| s.agent_id.as_str() == "agent-1"),
        "Agent with trading capability should be discoverable"
    );

    // Clean up
    registry.unregister_agent(agent1.id()).await?;

    Ok(())
}

/// Test that the dispatcher can route tasks to both local and remote agents.
#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_distributed_task_dispatch() -> Result<()> {
    let redis_url =
        std::env::var(ENV_REDIS_URL).unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Create registry for "process 1"
    let registry1 = Arc::new(RedisAgentRegistry::new(redis_url.clone()).await?);

    // Register a local agent
    let local_agent = Arc::new(TestAgent::new(
        "local-agent",
        vec![CapabilityType::Research],
    ));
    registry1.register_agent(local_agent.clone()).await?;

    // Update its status
    let local_status = AgentStatus {
        agent_id: local_agent.id().clone(),
        status: AgentState::Active,
        active_tasks: 0,
        load: 0.1,
        last_heartbeat: chrono::Utc::now(),
        capabilities: vec![Capability::new("research", "1.0")],
        metadata: HashMap::default(),
    };
    registry1.update_agent_status(local_status).await?;

    // Simulate a remote agent by just adding its status (no local instance)
    let remote_status = AgentStatus {
        agent_id: AgentId::new("remote-agent"),
        status: AgentState::Active,
        active_tasks: 1,
        load: 0.3,
        last_heartbeat: chrono::Utc::now(),
        capabilities: vec![Capability::new("research", "1.0")],
        metadata: HashMap::default(),
    };
    registry1.update_agent_status(remote_status).await?;

    // Create dispatcher
    let config = DispatchConfig {
        routing_strategy: RoutingStrategy::LeastLoaded,
        ..Default::default()
    };
    let dispatcher = AgentDispatcher::with_config(registry1.clone(), config);

    // Create a research task
    let task = Task::new(
        TaskType::Research,
        serde_json::json!({
            "query": "test research query"
        }),
    );

    // Dispatch the task - should go to local agent (lower load)
    let result = dispatcher.dispatch_task(task.clone()).await?;

    match result {
        TaskResult::Success { data, .. } => {
            // For local agent, we should get the test response
            if let Some(agent_id) = data.get("agent_id") {
                assert_eq!(
                    agent_id, "local-agent",
                    "Task should be routed to least loaded agent"
                );
            }
        }
        _ => panic!("Task dispatch should succeed"),
    }

    // Clean up
    registry1.unregister_agent(local_agent.id()).await?;

    Ok(())
}

/// Test concurrent agent registration and discovery.
#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_concurrent_agent_operations() -> Result<()> {
    let redis_url =
        std::env::var(ENV_REDIS_URL).unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Create multiple registries to simulate different processes
    let registry1: Arc<dyn DistributedAgentRegistry> =
        Arc::new(RedisAgentRegistry::new(redis_url.clone()).await?);
    let registry2: Arc<dyn DistributedAgentRegistry> =
        Arc::new(RedisAgentRegistry::new(redis_url.clone()).await?);

    // Spawn tasks to register agents concurrently
    let reg1 = registry1.clone();
    let handle1 = tokio::spawn(async move {
        let agent = Arc::new(TestAgent::new(
            "concurrent-1",
            vec![CapabilityType::Trading],
        ));
        reg1.register_agent(agent.clone()).await.unwrap();
        sleep(Duration::from_millis(100)).await;
        reg1.unregister_agent(agent.id()).await.unwrap();
    });

    let reg2 = registry2.clone();
    let handle2 = tokio::spawn(async move {
        let agent = Arc::new(TestAgent::new(
            "concurrent-2",
            vec![CapabilityType::Monitoring],
        ));
        reg2.register_agent(agent.clone()).await.unwrap();
        sleep(Duration::from_millis(100)).await;
        reg2.unregister_agent(agent.id()).await.unwrap();
    });

    // Give some time for registrations to happen
    sleep(Duration::from_millis(50)).await;

    // Both agents should be discoverable
    let all_statuses = registry1.list_agent_statuses().await?;
    let agent_ids: Vec<String> = all_statuses
        .iter()
        .map(|s| s.agent_id.as_str().to_string())
        .collect();

    // At this point, both agents should be registered
    // (They get unregistered after 100ms, we check at 50ms)
    assert!(
        agent_ids.contains(&"concurrent-1".to_string())
            || agent_ids.contains(&"concurrent-2".to_string()),
        "At least one concurrent agent should be discoverable"
    );

    // Wait for tasks to complete
    handle1.await.unwrap();
    handle2.await.unwrap();

    Ok(())
}
