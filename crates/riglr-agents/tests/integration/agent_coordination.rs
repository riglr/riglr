#![allow(clippy::expect_used)]
use core::time::Duration;
use riglr_agents::*;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

// Import test utilities
use crate::common::*;

#[tokio::test]
async fn test_full_trading_workflow() {
    // Setup shared state between agents
    let (agents, shared_state) = create_workflow_agents();

    // Create registry and register agents
    let registry = Arc::new(LocalAgentRegistry::new());
    for agent in agents {
        registry
            .register_agent(agent)
            .await
            .expect("Failed to register agent in test_full_trading_workflow");
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
    drop(final_state);
}

async fn execute_trading_workflow(
    registry: &Arc<LocalAgentRegistry>,
    _comm_system: &ChannelCommunication,
    shared_state: Arc<RwLock<SharedTradingState>>,
) -> Result<()> {
    let dispatcher = Dispatcher::new(registry.clone());

    // Step 1: Market Analysis
    let analysis_task = TestTaskBuilder::new(TaskType::Research)
        .with_parameters(serde_json::json!({"symbol": "BONK", "type": "market_analysis"}))
        .with_priority(Priority::High)
        .build();

    let analysis_result = dispatcher.dispatch_task(analysis_task).await?;
    assert!(analysis_result.is_success());

    // Verify analysis was stored in shared state
    {
        assert!(shared_state.read().await.market_analysis.is_some());
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
        assert!(shared_state.read().await.risk_assessment.is_some());
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
        drop(state);
    }

    Ok(())
}

#[tokio::test]
async fn test_workflow_failure_dependency() {
    // Test that execution fails without proper dependencies
    let (agents, _shared_state) = create_workflow_agents();

    let registry = Arc::new(LocalAgentRegistry::new());
    for agent in agents {
        registry
            .register_agent(agent)
            .await
            .expect("Failed to register agent in test_workflow_failure_dependency");
    }

    let dispatcher = Dispatcher::new(registry.clone());

    // Try to execute trade without market analysis and risk assessment
    let execution_task = TestTaskBuilder::new(TaskType::Trading)
        .with_parameters(serde_json::json!({"symbol": "BONK", "action": "buy"}))
        .build();

    let result = dispatcher
        .dispatch_task(execution_task)
        .await
        .expect("Failed to dispatch task in test_workflow_failure_dependency");

    // Should fail due to missing dependencies
    assert!(!result.is_success());
    assert!(result.is_retriable());

    let error = extract_task_result_error(&result)
        .expect("Failed to extract task result error in test_workflow_failure_dependency");
    assert!(error.contains("Cannot execute trade without analysis and risk assessment"));
}

#[tokio::test]
async fn test_parallel_agent_execution() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register multiple agents of the same type
    for i in 0..5 {
        let agent = Arc::new(MockTradingAgent::new(
            &format!("trader-{i}"),
            vec![CapabilityType::Trading],
        ));
        registry
            .register_agent(agent)
            .await
            .expect("Failed to register trading agent in test_parallel_agent_execution");
    }

    let dispatcher = Dispatcher::new(registry.clone());

    // Create tasks for parallel execution
    let tasks = create_load_test_tasks(10, &TaskType::Trading);

    // Execute all tasks concurrently
    let start_time = Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let execution_time = start_time.elapsed();

    // Verify all tasks completed successfully
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok());
        assert!(result
            .expect("Failed to get result in test_parallel_agent_execution")
            .is_success());
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

    registry
        .register_agent(market_agent)
        .await
        .expect("Failed to register market agent in test_agent_specialization_workflow");
    registry
        .register_agent(risk_agent)
        .await
        .expect("Failed to register risk agent in test_agent_specialization_workflow");
    registry
        .register_agent(execution_agent)
        .await
        .expect("Failed to register execution agent in test_agent_specialization_workflow");

    let dispatcher = Dispatcher::new(registry.clone());

    // Execute specialized tasks
    let research_task = create_research_task();
    let risk_task = create_risk_analysis_task();
    let trading_task = create_trading_task();

    let research_result = dispatcher
        .dispatch_task(research_task)
        .await
        .expect("Failed to dispatch research task in test_agent_specialization_workflow");
    let risk_result = dispatcher
        .dispatch_task(risk_task)
        .await
        .expect("Failed to dispatch risk task in test_agent_specialization_workflow");
    let trading_result = dispatcher
        .dispatch_task(trading_task)
        .await
        .expect("Failed to dispatch trading task in test_agent_specialization_workflow");

    // Verify each specialist handled their task
    assert!(research_result.is_success());
    assert!(risk_result.is_success());
    assert!(trading_result.is_success());

    let research_data = extract_task_result_data(&research_result)
        .expect("Failed to extract research data in test_agent_specialization_workflow");
    let risk_data = extract_task_result_data(&risk_result)
        .expect("Failed to extract risk data in test_agent_specialization_workflow");
    let trading_data = extract_task_result_data(&trading_result)
        .expect("Failed to extract trading data in test_agent_specialization_workflow");

    assert_eq!(research_data["agent_id"], "market-specialist");
    assert_eq!(risk_data["agent_id"], "risk-specialist");
    assert_eq!(trading_data["agent_id"], "execution-specialist");
}

#[tokio::test]
async fn test_multi_step_coordination_with_state() {
    // Test a complex workflow where agents coordinate through shared state
    let shared_state = Arc::new(RwLock::new(SharedTradingState::new()));

    let registry = Arc::new(LocalAgentRegistry::new());

    // Create agents that share state
    let agents: Vec<Arc<dyn Agent>> = vec![
        Arc::new(MockResearchAgent::new("researcher").with_shared_state(shared_state.clone())),
        Arc::new(MockRiskAgent::new("risk-manager").with_shared_state(shared_state.clone())),
        Arc::new(MockExecutionAgent::new("executor").with_shared_state(shared_state.clone())),
    ];

    for agent in agents {
        registry
            .register_agent(agent)
            .await
            .expect("Failed to register agent in test_multi_step_coordination_with_state");
    }

    let dispatcher = Dispatcher::new(registry.clone());

    // Execute workflow in correct order
    let step1 = create_research_task();
    let step2 = create_risk_analysis_task();
    let step3 = create_trading_task();

    // Step 1: Research
    let result1 = dispatcher
        .dispatch_task(step1)
        .await
        .expect("Failed to dispatch step1 in test_multi_step_coordination_with_state");
    assert!(result1.is_success());

    // Verify state after step 1
    {
        let state = shared_state.read().await;
        assert!(state.market_analysis.is_some());
        assert!(state.risk_assessment.is_none());
        assert!(!state.trade_executed);
        drop(state);
    }

    // Step 2: Risk analysis
    let result2 = dispatcher
        .dispatch_task(step2)
        .await
        .expect("Failed to dispatch step2 in test_multi_step_coordination_with_state");
    assert!(result2.is_success());

    // Verify state after step 2
    {
        let state = shared_state.read().await;
        assert!(state.market_analysis.is_some());
        assert!(state.risk_assessment.is_some());
        assert!(!state.trade_executed);
        drop(state);
    }

    // Step 3: Execution
    let result3 = dispatcher
        .dispatch_task(step3)
        .await
        .expect("Failed to dispatch step3 in test_multi_step_coordination_with_state");
    assert!(result3.is_success());

    // Verify final state
    {
        assert!(shared_state.read().await.is_workflow_complete());
    }
}

#[tokio::test]
async fn test_agent_coordination_with_retries() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register an agent that initially fails but succeeds on retry
    let failing_agent: Arc<dyn Agent> = Arc::new(
        MockTradingAgent::new("flaky-trader", vec![CapabilityType::Trading]).with_failure(),
    );

    let reliable_agent: Arc<dyn Agent> = Arc::new(MockTradingAgent::new(
        "reliable-trader",
        vec![CapabilityType::Trading],
    ));

    registry
        .register_agent(failing_agent)
        .await
        .expect("Failed to register failing agent in test_agent_coordination_with_retries");
    registry
        .register_agent(reliable_agent)
        .await
        .expect("Failed to register reliable agent in test_agent_coordination_with_retries");

    let _config = DispatchConfig {
        max_retries: 2,
        retry_delay: Duration::from_millis(10),
        routing_strategy: RoutingStrategy::RoundRobin,
        ..create_test_dispatch_config()
    };
    let dispatcher = Dispatcher::new(registry.clone());

    // Execute multiple tasks - should eventually use the reliable agent
    let tasks = create_load_test_tasks(3, &TaskType::Trading);
    let results = dispatcher.dispatch_tasks(tasks).await;

    // Some tasks may fail (from flaky agent), but at least one should succeed
    let successful_count = results
        .iter()
        .filter(|r| {
            r.is_ok()
                && r.as_ref()
                    .expect(
                        "Failed to get result reference in test_agent_coordination_with_retries",
                    )
                    .is_success()
        })
        .count();

    assert!(successful_count > 0);
}

#[tokio::test]
async fn test_workflow_with_mixed_priorities() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register agents for different capabilities
    let agents = create_test_agent_set();
    for agent in agents {
        registry
            .register_agent(agent)
            .await
            .expect("Failed to register agent in test_workflow_with_mixed_priorities");
    }

    let dispatcher = Dispatcher::new(registry.clone());

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
        assert!(result
            .expect("Failed to get result in test_workflow_with_mixed_priorities")
            .is_success());
    }
}

#[tokio::test]
async fn test_agent_coordination_performance() {
    let registry = Arc::new(LocalAgentRegistry::new());

    // Register multiple agents for performance testing
    for i in 0..10 {
        let agent: Arc<dyn Agent> = Arc::new(
            MockTradingAgent::new(&format!("trader-{i}"), vec![CapabilityType::Trading])
                .with_delay(Duration::from_millis(50)),
        ); // Small delay to simulate work

        registry
            .register_agent(agent)
            .await
            .expect("Failed to register agent in test_agent_coordination_performance");
    }

    let dispatcher = Dispatcher::new(registry.clone());

    // Create a large batch of tasks
    let tasks = create_load_test_tasks(50, &TaskType::Trading);

    // Measure execution time
    let start_time = Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let execution_time = start_time.elapsed();

    // Verify all tasks completed
    assert_eq!(results.len(), 50);
    for result in results {
        assert!(result.is_ok());
        assert!(result
            .expect("Failed to get result in test_agent_coordination_performance")
            .is_success());
    }

    // With 10 agents and 50ms delay each, sequential execution would take ~2.5s
    // Parallel execution should be much faster
    assert!(execution_time < Duration::from_secs(2));

    println!("Executed 50 tasks with 10 agents in {execution_time:?}");
}

#[tokio::test]
async fn test_agent_workflow_state_isolation() {
    // Test that different workflow instances don't interfere with each other
    let (agents1, state1) = create_workflow_agents();
    let (_agents2, state2) = create_workflow_agents();

    let registry = Arc::new(LocalAgentRegistry::new());

    // Register agents from both workflows (with different IDs)
    for agent in agents1 {
        registry
            .register_agent(agent)
            .await
            .expect("Failed to register agent1 in test_agent_workflow_state_isolation");
    }

    // Register second set with different names
    let workflow2_agents: Vec<Arc<dyn Agent>> = vec![
        Arc::new(MockResearchAgent::new("researcher-2").with_shared_state(state2.clone())),
        Arc::new(MockRiskAgent::new("risk-manager-2").with_shared_state(state2.clone())),
        Arc::new(MockExecutionAgent::new("executor-2").with_shared_state(state2.clone())),
    ];

    for agent in workflow2_agents {
        registry
            .register_agent(agent)
            .await
            .expect("Failed to register agent2 in test_agent_workflow_state_isolation");
    }

    let dispatcher = Dispatcher::new(registry.clone());

    // Execute workflow 1
    let task1 = TestTaskBuilder::new(TaskType::Research)
        .with_parameters(serde_json::json!({"workflow": "1"}))
        .build();

    let result1 = dispatcher
        .dispatch_task(task1)
        .await
        .expect("Failed to dispatch task1 in test_agent_workflow_state_isolation");
    assert!(result1.is_success());

    // Execute workflow 2
    let task2 = TestTaskBuilder::new(TaskType::Research)
        .with_parameters(serde_json::json!({"workflow": "2"}))
        .build();

    let result2 = dispatcher
        .dispatch_task(task2)
        .await
        .expect("Failed to dispatch task2 in test_agent_workflow_state_isolation");
    assert!(result2.is_success());

    // Verify both states have been updated independently
    assert!(state1.read().await.market_analysis.is_some());
    assert!(state2.read().await.market_analysis.is_some());

    // Verify they contain different agent IDs
    let data1 = extract_task_result_data(&result1)
        .expect("Failed to extract data1 in test_agent_workflow_state_isolation");
    let data2 = extract_task_result_data(&result2)
        .expect("Failed to extract data2 in test_agent_workflow_state_isolation");

    assert_ne!(data1["agent_id"], data2["agent_id"]);
}

// TODO: Re-enable this test once rig trait compatibility is resolved
// #[tokio::test]
// async fn test_tool_calling_agent_parallel_execution() {
//     // Test implementation commented out due to rig trait compatibility issues
// }

// TODO: Re-enable this test once rig trait compatibility is resolved
// #[tokio::test]
// async fn test_tool_calling_agent_handles_partial_failures() {
//     // Test implementation commented out due to rig trait compatibility issues
// }
