use riglr_agents::*;
use std::sync::Arc;
use std::time::Duration;

// Import test utilities
use crate::common::*;

#[tokio::test]
async fn test_full_trading_workflow() {
    // Setup shared state between agents
    let (agents, shared_state) = create_workflow_agents();

    // Create registry and register agents
    let registry = Arc::new(LocalAgentRegistry::new());
    for agent in agents {
        registry.register_agent(agent).await.unwrap();
    }

    // Create communication system
    let comm_system = ChannelCommunication::new();

    // Execute workflow: Research → Risk → Execution
    let workflow_result =
        execute_trading_workflow(&registry, &comm_system, shared_state.clone()).await;

    // Verify workflow completion
    assert!(workflow_result.is_ok());

    // Verify shared state consistency
    let final_state = shared_state.read().await;
    assert!(final_state.market_analysis.is_some());
    assert!(final_state.risk_assessment.is_some());
    assert!(final_state.trade_executed);
    assert!(final_state.execution_result.is_some());
    assert!(final_state.is_workflow_complete());
}

async fn execute_trading_workflow(
    registry: &Arc<LocalAgentRegistry>,
    _comm_system: &ChannelCommunication,
    shared_state: Arc<tokio::sync::RwLock<SharedTradingState>>,
) -> Result<()> {
    let dispatcher = AgentDispatcher::new(registry.clone());

    // Step 1: Market Analysis
    let analysis_task = TestTaskBuilder::new(TaskType::Research)
        .with_parameters(serde_json::json!({"symbol": "BONK", "type": "market_analysis"}))
        .with_priority(Priority::High)
        .build();

    let analysis_result = dispatcher.dispatch_task(analysis_task).await?;
    assert!(analysis_result.is_success());

    // Verify analysis was stored in shared state
    {
        let state = shared_state.read().await;
        assert!(state.market_analysis.is_some());
    }

    // Step 2: Risk Assessment (depends on analysis)
    let risk_task = TestTaskBuilder::new(TaskType::RiskAnalysis)
        .with_parameters(serde_json::json!({"symbol": "BONK", "position_size": 1000}))
        .with_priority(Priority::High)
        .build();

    let risk_result = dispatcher.dispatch_task(risk_task).await?;
    assert!(risk_result.is_success());

    // Verify risk assessment was stored
    {
        let state = shared_state.read().await;
        assert!(state.risk_assessment.is_some());
    }

    // Step 3: Trade Execution (depends on risk approval)
    let execution_task = TestTaskBuilder::new(TaskType::Trading)
        .with_parameters(serde_json::json!({"symbol": "BONK", "action": "buy", "amount": 1000}))
        .with_priority(Priority::Critical)
        .build();

    let execution_result = dispatcher.dispatch_task(execution_task).await?;
    assert!(execution_result.is_success());

    // Verify execution was completed
    {
        let state = shared_state.read().await;
        assert!(state.trade_executed);
        assert!(state.execution_result.is_some());
    }

    Ok(())
}

#[tokio::test]
async fn test_workflow_failure_dependency() {
    // Test that execution fails without proper dependencies
    let (agents, shared_state) = create_workflow_agents();

    let registry = Arc::new(LocalAgentRegistry::new());
    for agent in agents {
        registry.register_agent(agent).await.unwrap();
    }

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Try to execute trade without market analysis and risk assessment
    let execution_task = TestTaskBuilder::new(TaskType::Trading)
        .with_parameters(serde_json::json!({"symbol": "BONK", "action": "buy"}))
        .build();

    let result = dispatcher.dispatch_task(execution_task).await.unwrap();

    // Should fail due to missing dependencies
    assert!(!result.is_success());
    assert!(result.is_retriable());

    let error = extract_task_result_error(&result).unwrap();
    assert!(error.contains("Cannot execute trade without analysis and risk assessment"));
}

#[tokio::test]
async fn test_parallel_agent_execution() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register multiple agents of the same type
    for i in 0..5 {
        let agent = Arc::new(MockTradingAgent::new(
            &format!("trader-{}", i),
            vec!["trading".to_string()],
        ));
        registry.register_agent(agent).await.unwrap();
    }

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Create tasks for parallel execution
    let tasks = create_load_test_tasks(10, TaskType::Trading);

    // Execute all tasks concurrently
    let start_time = std::time::Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let execution_time = start_time.elapsed();

    // Verify all tasks completed successfully
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
    }

    // With 5 agents and 10 tasks, execution should be faster than sequential
    // (This is a rough check - actual timing depends on system load)
    assert!(execution_time < Duration::from_secs(2));
}

#[tokio::test]
async fn test_agent_specialization_workflow() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register specialized agents
    let market_agent: Arc<dyn Agent> = Arc::new(MockResearchAgent::new("market-specialist"));
    let risk_agent: Arc<dyn Agent> = Arc::new(MockRiskAgent::new("risk-specialist"));
    let execution_agent: Arc<dyn Agent> = Arc::new(MockExecutionAgent::new("execution-specialist"));

    registry.register_agent(market_agent).await.unwrap();
    registry.register_agent(risk_agent).await.unwrap();
    registry.register_agent(execution_agent).await.unwrap();

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Execute specialized tasks
    let research_task = create_research_task();
    let risk_task = create_risk_analysis_task();
    let trading_task = create_trading_task();

    let research_result = dispatcher.dispatch_task(research_task).await.unwrap();
    let risk_result = dispatcher.dispatch_task(risk_task).await.unwrap();
    let trading_result = dispatcher.dispatch_task(trading_task).await.unwrap();

    // Verify each specialist handled their task
    assert!(research_result.is_success());
    assert!(risk_result.is_success());
    assert!(trading_result.is_success());

    let research_data = extract_task_result_data(&research_result).unwrap();
    let risk_data = extract_task_result_data(&risk_result).unwrap();
    let trading_data = extract_task_result_data(&trading_result).unwrap();

    assert_eq!(research_data["agent_id"], "market-specialist");
    assert_eq!(risk_data["agent_id"], "risk-specialist");
    assert_eq!(trading_data["agent_id"], "execution-specialist");
}

#[tokio::test]
async fn test_multi_step_coordination_with_state() {
    // Test a complex workflow where agents coordinate through shared state
    let shared_state = Arc::new(tokio::sync::RwLock::new(SharedTradingState::new()));

    let registry = Arc::new(LocalAgentRegistry::new());

    // Create agents that share state
    let agents: Vec<Arc<dyn Agent>> = vec![
        Arc::new(MockResearchAgent::new("researcher").with_shared_state(shared_state.clone())),
        Arc::new(MockRiskAgent::new("risk-manager").with_shared_state(shared_state.clone())),
        Arc::new(MockExecutionAgent::new("executor").with_shared_state(shared_state.clone())),
    ];

    for agent in agents {
        registry.register_agent(agent).await.unwrap();
    }

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Execute workflow in correct order
    let step1 = create_research_task();
    let step2 = create_risk_analysis_task();
    let step3 = create_trading_task();

    // Step 1: Research
    let result1 = dispatcher.dispatch_task(step1).await.unwrap();
    assert!(result1.is_success());

    // Verify state after step 1
    {
        let state = shared_state.read().await;
        assert!(state.market_analysis.is_some());
        assert!(state.risk_assessment.is_none());
        assert!(!state.trade_executed);
    }

    // Step 2: Risk analysis
    let result2 = dispatcher.dispatch_task(step2).await.unwrap();
    assert!(result2.is_success());

    // Verify state after step 2
    {
        let state = shared_state.read().await;
        assert!(state.market_analysis.is_some());
        assert!(state.risk_assessment.is_some());
        assert!(!state.trade_executed);
    }

    // Step 3: Execution
    let result3 = dispatcher.dispatch_task(step3).await.unwrap();
    assert!(result3.is_success());

    // Verify final state
    {
        let state = shared_state.read().await;
        assert!(state.is_workflow_complete());
    }
}

#[tokio::test]
async fn test_agent_coordination_with_retries() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register an agent that initially fails but succeeds on retry
    let failing_agent: Arc<dyn Agent> =
        Arc::new(MockTradingAgent::new("flaky-trader", vec!["trading".to_string()]).with_failure());

    let reliable_agent: Arc<dyn Agent> = Arc::new(MockTradingAgent::new(
        "reliable-trader",
        vec!["trading".to_string()],
    ));

    registry.register_agent(failing_agent).await.unwrap();
    registry.register_agent(reliable_agent).await.unwrap();

    let config = DispatchConfig {
        max_retries: 2,
        retry_delay: Duration::from_millis(10),
        routing_strategy: RoutingStrategy::RoundRobin,
        ..create_test_dispatch_config()
    };
    let dispatcher = AgentDispatcher::new(registry.clone());

    // Execute multiple tasks - should eventually use the reliable agent
    let tasks = create_load_test_tasks(3, TaskType::Trading);
    let results = dispatcher.dispatch_tasks(tasks).await;

    // Some tasks may fail (from flaky agent), but at least one should succeed
    let successful_count = results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_success())
        .count();

    assert!(successful_count > 0);
}

#[tokio::test]
async fn test_workflow_with_mixed_priorities() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register agents for different capabilities
    let agents = create_test_agent_set();
    for agent in agents {
        registry.register_agent(agent).await.unwrap();
    }

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Create tasks with different priorities
    let critical_task = TestTaskBuilder::new(TaskType::Trading)
        .with_priority(Priority::Critical)
        .with_parameters(serde_json::json!({"urgency": "critical"}))
        .build();

    let normal_task = TestTaskBuilder::new(TaskType::Research)
        .with_priority(Priority::Normal)
        .with_parameters(serde_json::json!({"urgency": "normal"}))
        .build();

    let low_task = TestTaskBuilder::new(TaskType::RiskAnalysis)
        .with_priority(Priority::Low)
        .with_parameters(serde_json::json!({"urgency": "low"}))
        .build();

    // Execute tasks concurrently
    let tasks = vec![low_task, normal_task, critical_task]; // Note: intentionally out of priority order
    let results = dispatcher.dispatch_tasks(tasks).await;

    // All tasks should complete successfully
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
    }
}

#[tokio::test]
async fn test_agent_coordination_performance() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register multiple agents for performance testing
    for i in 0..10 {
        let agent: Arc<dyn Agent> = Arc::new(
            MockTradingAgent::new(&format!("trader-{}", i), vec!["trading".to_string()])
                .with_delay(Duration::from_millis(50)),
        ); // Small delay to simulate work

        registry.register_agent(agent).await.unwrap();
    }

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Create a large batch of tasks
    let tasks = create_load_test_tasks(50, TaskType::Trading);

    // Measure execution time
    let start_time = std::time::Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let execution_time = start_time.elapsed();

    // Verify all tasks completed
    assert_eq!(results.len(), 50);
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
    }

    // With 10 agents and 50ms delay each, sequential execution would take ~2.5s
    // Parallel execution should be much faster
    assert!(execution_time < Duration::from_secs(2));

    println!("Executed 50 tasks with 10 agents in {:?}", execution_time);
}

#[tokio::test]
async fn test_agent_workflow_state_isolation() {
    // Test that different workflow instances don't interfere with each other
    let (agents1, state1) = create_workflow_agents();
    let (agents2, state2) = create_workflow_agents();

    let registry = Arc::new(LocalAgentRegistry::new());

    // Register agents from both workflows (with different IDs)
    for (i, agent) in agents1.into_iter().enumerate() {
        registry.register_agent(agent).await.unwrap();
    }

    // Register second set with different names
    let mut workflow2_agents: Vec<Arc<dyn Agent>> = Vec::new();
    workflow2_agents.push(Arc::new(
        MockResearchAgent::new("researcher-2").with_shared_state(state2.clone()),
    ));
    workflow2_agents.push(Arc::new(
        MockRiskAgent::new("risk-manager-2").with_shared_state(state2.clone()),
    ));
    workflow2_agents.push(Arc::new(
        MockExecutionAgent::new("executor-2").with_shared_state(state2.clone()),
    ));

    for agent in workflow2_agents {
        registry.register_agent(agent).await.unwrap();
    }

    let dispatcher = AgentDispatcher::new(registry.clone());

    // Execute workflow 1
    let task1 = TestTaskBuilder::new(TaskType::Research)
        .with_parameters(serde_json::json!({"workflow": "1"}))
        .build();

    let result1 = dispatcher.dispatch_task(task1).await.unwrap();
    assert!(result1.is_success());

    // Execute workflow 2
    let task2 = TestTaskBuilder::new(TaskType::Research)
        .with_parameters(serde_json::json!({"workflow": "2"}))
        .build();

    let result2 = dispatcher.dispatch_task(task2).await.unwrap();
    assert!(result2.is_success());

    // Verify both states have been updated independently
    let final_state1 = state1.read().await;
    let final_state2 = state2.read().await;

    assert!(final_state1.market_analysis.is_some());
    assert!(final_state2.market_analysis.is_some());

    // Verify they contain different agent IDs
    let data1 = extract_task_result_data(&result1).unwrap();
    let data2 = extract_task_result_data(&result2).unwrap();

    assert_ne!(data1["agent_id"], data2["agent_id"]);
}
