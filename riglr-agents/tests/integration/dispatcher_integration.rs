use riglr_agents::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// Import test utilities
use crate::common::*;

#[tokio::test]
async fn test_dispatcher_registry_router_integration() {
    // Setup: Create registry with multiple mock agents
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register agents with different capabilities
    let trading_agent = Arc::new(MockTradingAgent::new(
        "trader-1",
        vec![CapabilityType::Trading],
    ));
    let research_agent = Arc::new(MockResearchAgent::new("researcher-1"));
    let risk_agent = Arc::new(MockRiskAgent::new("risk-1"));

    let trading_id = trading_agent.id().clone();
    let _research_id = research_agent.id().clone();
    let _risk_id = risk_agent.id().clone();

    registry.register_agent(trading_agent).await.unwrap();
    registry.register_agent(research_agent).await.unwrap();
    registry.register_agent(risk_agent).await.unwrap();

    // Verify agents are registered
    assert_eq!(registry.agent_count().await.unwrap(), 3);

    // Test all routing strategies
    test_capability_routing(&registry).await;
    test_round_robin_routing(&registry).await;
    test_least_loaded_routing(&registry).await;
    test_direct_routing(&registry, &trading_id).await;
}

async fn test_capability_routing(registry: &Arc<LocalAgentRegistry>) {
    let dispatcher = AgentDispatcher::new(registry.clone());

    let task = create_trading_task();

    let result = dispatcher.dispatch_task(task).await.unwrap();
    assert!(result.is_success());

    // Verify the result contains expected data
    let data = extract_task_result_data(&result).unwrap();
    assert_eq!(data["agent_id"], "trader-1");
    assert_eq!(data["task_type"], "trading");
}

async fn test_round_robin_routing(registry: &Arc<LocalAgentRegistry>) {
    let config = DispatchConfig {
        routing_strategy: RoutingStrategy::RoundRobin,
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    // Create multiple trading tasks to test round-robin distribution
    let tasks = create_load_test_tasks(6, TaskType::Trading);
    let mut agent_usage = HashMap::new();

    for task in tasks {
        let result = dispatcher.dispatch_task(task).await.unwrap();
        assert!(result.is_success());

        // Track which agent executed each task
        let data = extract_task_result_data(&result).unwrap();
        let agent_id = data["agent_id"].as_str().unwrap();
        *agent_usage.entry(agent_id.to_string()).or_insert(0) += 1;
    }

    // Verify distribution (should only use trading agent since only it has trading capability)
    assert!(agent_usage.contains_key("trader-1"));
    assert_eq!(agent_usage["trader-1"], 6);
}

async fn test_least_loaded_routing(registry: &Arc<LocalAgentRegistry>) {
    let config = DispatchConfig {
        routing_strategy: RoutingStrategy::LeastLoaded,
        enable_load_balancing: true,
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    let task = create_trading_task();
    let result = dispatcher.dispatch_task(task).await.unwrap();
    assert!(result.is_success());

    // The least loaded agent with trading capability should be selected
    let data = extract_task_result_data(&result).unwrap();
    assert_eq!(data["agent_id"], "trader-1");
}

async fn test_direct_routing(registry: &Arc<LocalAgentRegistry>, _agent_id: &AgentId) {
    let config = DispatchConfig {
        routing_strategy: RoutingStrategy::Direct,
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    // For direct routing, we need to specify the target agent in task metadata
    let task = create_trading_task();

    let result = dispatcher.dispatch_task(task).await.unwrap();
    assert!(result.is_success());
}

#[tokio::test]
async fn test_dispatcher_with_different_task_types() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register specialized agents
    let agents = create_test_agent_set();
    for agent in agents {
        registry.register_agent(agent).await.unwrap();
    }

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Test different task types
    let trading_task = create_trading_task();
    let research_task = create_research_task();
    let risk_task = create_risk_analysis_task();

    // Execute each task type
    let trading_result = dispatcher.dispatch_task(trading_task).await.unwrap();
    let research_result = dispatcher.dispatch_task(research_task).await.unwrap();
    let risk_result = dispatcher.dispatch_task(risk_task).await.unwrap();

    // Verify all tasks completed successfully
    assert!(trading_result.is_success());
    assert!(research_result.is_success());
    assert!(risk_result.is_success());

    // Verify correct agents were selected
    let trading_data = extract_task_result_data(&trading_result).unwrap();
    let research_data = extract_task_result_data(&research_result).unwrap();
    let risk_data = extract_task_result_data(&risk_result).unwrap();

    assert_eq!(trading_data["agent_id"], "trader-1");
    assert_eq!(research_data["agent_id"], "researcher-1");
    assert_eq!(risk_data["agent_id"], "risk-1");
}

#[tokio::test]
async fn test_dispatcher_concurrent_execution() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register multiple agents for concurrent testing
    for i in 0..3 {
        let agent = Arc::new(MockTradingAgent::new(
            &format!("trader-{}", i),
            vec![CapabilityType::Trading],
        ));
        registry.register_agent(agent).await.unwrap();
    }

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Create multiple tasks
    let tasks = create_load_test_tasks(10, TaskType::Trading);

    // Execute tasks concurrently
    let results = dispatcher.dispatch_tasks(tasks).await;

    // Verify all tasks completed successfully
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
    }
}

#[tokio::test]
async fn test_dispatcher_load_balancing() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register agents with different load levels
    let high_load_agent =
        Arc::new(MockTradingAgent::new("high-load", vec![CapabilityType::Trading]).with_load(0.9));

    let low_load_agent =
        Arc::new(MockTradingAgent::new("low-load", vec![CapabilityType::Trading]).with_load(0.1));

    registry.register_agent(high_load_agent).await.unwrap();
    registry.register_agent(low_load_agent).await.unwrap();

    let config = DispatchConfig {
        routing_strategy: RoutingStrategy::LeastLoaded,
        enable_load_balancing: true,
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    // Execute a task - should prefer low load agent
    let task = create_trading_task();
    let result = dispatcher.dispatch_task(task).await.unwrap();

    assert!(result.is_success());
    let data = extract_task_result_data(&result).unwrap();
    assert_eq!(data["agent_id"], "low-load");
}

#[tokio::test]
async fn test_dispatcher_stats_and_health() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register test agents
    let agents = create_test_agent_set();
    for agent in agents {
        registry.register_agent(agent).await.unwrap();
    }

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Test health check
    let health = dispatcher.health_check().await.unwrap();
    assert!(health);

    // Test stats
    let stats = dispatcher.stats().await.unwrap();
    assert_eq!(stats.registered_agents, 4);
    assert_eq!(stats.routing_strategy, RoutingStrategy::Capability);
    assert_eq!(stats.total_active_tasks, 0);
    assert_eq!(stats.average_agent_load, 0.0);
}

#[tokio::test]
async fn test_dispatcher_no_suitable_agent_error() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register only a trading agent
    let trading_agent = Arc::new(MockTradingAgent::new(
        "trader",
        vec![CapabilityType::Trading],
    ));
    registry.register_agent(trading_agent).await.unwrap();

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Try to execute a portfolio task (no suitable agent)
    let portfolio_task = create_portfolio_task();
    let result = dispatcher.dispatch_task(portfolio_task).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, AgentError::NoSuitableAgent { .. }));
}

#[tokio::test]
async fn test_dispatcher_agent_unavailability() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register an offline agent
    let offline_agent = Arc::new(
        MockTradingAgent::new("offline", vec![CapabilityType::Trading])
            .with_state(AgentState::Offline),
    );

    registry.register_agent(offline_agent).await.unwrap();

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Try to execute a task with only offline agent
    let task = create_trading_task();
    let result = dispatcher.dispatch_task(task).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, AgentError::NoSuitableAgent { .. }));
}

#[tokio::test]
async fn test_dispatcher_task_timeout() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register an agent with long execution delay
    let slow_agent = Arc::new(
        MockTradingAgent::new("slow", vec![CapabilityType::Trading])
            .with_delay(Duration::from_millis(200)),
    );

    registry.register_agent(slow_agent).await.unwrap();

    let config = DispatchConfig {
        default_task_timeout: Duration::from_millis(50), // Shorter than agent delay
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    let task = create_trading_task();
    let result = dispatcher.dispatch_task(task).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, AgentError::TaskTimeout { .. }));
}

#[tokio::test]
async fn test_dispatcher_routing_strategy_changes() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register multiple trading agents
    for i in 0..3 {
        let agent = Arc::new(MockTradingAgent::new(
            &format!("trader-{}", i),
            vec![CapabilityType::Trading],
        ));
        registry.register_agent(agent).await.unwrap();
    }

    // Test with capability routing
    let capability_config = DispatchConfig {
        routing_strategy: RoutingStrategy::Capability,
        ..create_test_dispatch_config()
    };
    let capability_dispatcher = AgentDispatcher::with_config(registry.clone(), capability_config);

    let task1 = create_trading_task();
    let result1 = capability_dispatcher.dispatch_task(task1).await.unwrap();
    assert!(result1.is_success());

    // Test with round-robin routing
    let round_robin_config = DispatchConfig {
        routing_strategy: RoutingStrategy::RoundRobin,
        ..create_test_dispatch_config()
    };
    let round_robin_dispatcher = AgentDispatcher::with_config(registry.clone(), round_robin_config);

    let task2 = create_trading_task();
    let result2 = round_robin_dispatcher.dispatch_task(task2).await.unwrap();
    assert!(result2.is_success());

    // Test with random routing
    let random_config = DispatchConfig {
        routing_strategy: RoutingStrategy::Random,
        ..create_test_dispatch_config()
    };
    let random_dispatcher = AgentDispatcher::with_config(registry.clone(), random_config);

    let task3 = create_trading_task();
    let result3 = random_dispatcher.dispatch_task(task3).await.unwrap();
    assert!(result3.is_success());
}

#[tokio::test]
async fn test_dispatcher_custom_task_type() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register an agent with custom capability
    let custom_agent = Arc::new(MockTradingAgent::new(
        "custom-agent",
        vec![CapabilityType::Custom("custom_capability".to_string())],
    ));

    registry.register_agent(custom_agent).await.unwrap();

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Execute custom task
    let custom_task = create_custom_task("custom_capability");
    let result = dispatcher.dispatch_task(custom_task).await.unwrap();

    assert!(result.is_success());
    let data = extract_task_result_data(&result).unwrap();
    assert_eq!(data["agent_id"], "custom-agent");
}

#[tokio::test]
async fn test_dispatcher_with_disabled_load_balancing() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register agents with different loads
    let agents = vec![
        Arc::new(MockTradingAgent::new("trader-1", vec![CapabilityType::Trading]).with_load(0.8)),
        Arc::new(MockTradingAgent::new("trader-2", vec![CapabilityType::Trading]).with_load(0.2)),
    ];

    for agent in agents {
        registry.register_agent(agent).await.unwrap();
    }

    let config = DispatchConfig {
        enable_load_balancing: false,
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

    let task = create_trading_task();
    let result = dispatcher.dispatch_task(task).await.unwrap();

    // Should succeed regardless of load balancing being disabled
    assert!(result.is_success());
}
