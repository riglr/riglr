//! Integration tests for multi-agent coordination workflows.
//!
//! These tests verify end-to-end multi-agent coordination scenarios,
//! including task delegation, result aggregation, and workflow orchestration.

use riglr_agents::*;
use riglr_core::signer::SignerContext;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::Mutex;
use serde_json::json;

/// Mock trading agent that simulates blockchain trading operations.
#[derive(Clone)]
struct TradingAgent {
    id: AgentId,
    execution_delay: Duration,
    should_fail: bool,
}

#[async_trait::async_trait]
impl Agent for TradingAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        // Simulate execution delay
        tokio::time::sleep(self.execution_delay).await;

        if self.should_fail {
            return Ok(TaskResult::failure(
                "Trading operation failed".to_string(),
                true,
                self.execution_delay,
            ));
        }

        // Extract trade parameters
        let symbol = task.parameters.get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("BTC/USD");
        let action = task.parameters.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("buy");
        let amount = task.parameters.get("amount")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        // Simulate successful trade
        let result = json!({
            "agent_id": self.id.as_str(),
            "symbol": symbol,
            "action": action,
            "amount": amount,
            "price": 50000.0,
            "tx_hash": format!("0x{:x}", rand::random::<u64>()),
            "status": "completed"
        });

        Ok(TaskResult::success(result, Some("0x123".to_string()), self.execution_delay))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["trading".to_string(), "portfolio".to_string()]
    }
}

/// Mock risk analysis agent.
#[derive(Clone)]
struct RiskAnalysisAgent {
    id: AgentId,
    risk_threshold: f64,
}

#[async_trait::async_trait]
impl Agent for RiskAnalysisAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let symbol = task.parameters.get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("BTC/USD");
        let amount = task.parameters.get("amount")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        // Simulate risk calculation
        let risk_score = (amount * 0.1).min(1.0);
        let approved = risk_score <= self.risk_threshold;

        let result = json!({
            "agent_id": self.id.as_str(),
            "symbol": symbol,
            "amount": amount,
            "risk_score": risk_score,
            "approved": approved,
            "recommendation": if approved { "proceed" } else { "reject" }
        });

        Ok(TaskResult::success(result, None, Duration::from_millis(100)))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["risk_analysis".to_string()]
    }
}

/// Mock research agent.
#[derive(Clone)]
struct ResearchAgent {
    id: AgentId,
    knowledge_base: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

#[async_trait::async_trait]
impl Agent for ResearchAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        tokio::time::sleep(Duration::from_millis(200)).await;

        let query = task.parameters.get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let knowledge = self.knowledge_base.lock().await;
        let research_data = knowledge.get(query).cloned()
            .unwrap_or_else(|| json!({"error": "No data available"}));

        let result = json!({
            "agent_id": self.id.as_str(),
            "query": query,
            "data": research_data,
            "confidence": 0.85
        });

        Ok(TaskResult::success(result, None, Duration::from_millis(200)))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["research".to_string(), "monitoring".to_string()]
    }
}

/// Test basic multi-agent workflow coordination.
#[tokio::test]
async fn test_basic_multi_agent_workflow() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Register agents
    let trading_agent = Arc::new(TradingAgent {
        id: AgentId::new("trading-1"),
        execution_delay: Duration::from_millis(50),
        should_fail: false,
    });

    let risk_agent = Arc::new(RiskAnalysisAgent {
        id: AgentId::new("risk-1"),
        risk_threshold: 0.5,
    });

    registry.register_agent(trading_agent).await.unwrap();
    registry.register_agent(risk_agent).await.unwrap();

    // Create workflow tasks
    let risk_task = Task::new(
        TaskType::RiskAnalysis,
        json!({"symbol": "BTC/USD", "amount": 2.0}),
    ).with_priority(Priority::High);

    let trading_task = Task::new(
        TaskType::Trading,
        json!({"symbol": "BTC/USD", "action": "buy", "amount": 2.0}),
    ).with_priority(Priority::Normal);

    // Execute workflow: risk analysis first, then trading
    let risk_result = dispatcher.dispatch_task(risk_task).await.unwrap();
    assert!(risk_result.is_success());
    
    let risk_data = risk_result.data().unwrap();
    let approved = risk_data.get("approved").and_then(|v| v.as_bool()).unwrap_or(false);
    
    if approved {
        let trade_result = dispatcher.dispatch_task(trading_task).await.unwrap();
        assert!(trade_result.is_success());
        
        let trade_data = trade_result.data().unwrap();
        assert_eq!(trade_data.get("status").and_then(|v| v.as_str()).unwrap(), "completed");
    }
}

/// Test concurrent multi-agent task execution.
#[tokio::test]
async fn test_concurrent_multi_agent_execution() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Register multiple trading agents
    for i in 0..3 {
        let agent = Arc::new(TradingAgent {
            id: AgentId::new(&format!("trading-{}", i)),
            execution_delay: Duration::from_millis(100),
            should_fail: false,
        });
        registry.register_agent(agent).await.unwrap();
    }

    // Create multiple tasks
    let tasks: Vec<Task> = (0..5)
        .map(|i| {
            Task::new(
                TaskType::Trading,
                json!({
                    "symbol": format!("COIN{}/USD", i),
                    "action": "buy",
                    "amount": 1.0
                }),
            )
        })
        .collect();

    // Execute tasks concurrently
    let start_time = std::time::Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let execution_time = start_time.elapsed();

    // Verify all tasks completed successfully
    assert_eq!(results.len(), 5);
    assert!(results.iter().all(|r| r.is_ok()));

    // Should execute faster than sequential (each task takes 100ms)
    assert!(execution_time < Duration::from_millis(400)); // Allow some overhead
}

/// Test workflow with task dependencies and result passing.
#[tokio::test]
async fn test_dependent_task_workflow() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Create research agent with mock data
    let mut knowledge_base = HashMap::new();
    knowledge_base.insert(
        "BTC market analysis".to_string(),
        json!({
            "trend": "bullish",
            "volatility": "moderate",
            "recommendation": "buy",
            "confidence": 0.9
        })
    );

    let research_agent = Arc::new(ResearchAgent {
        id: AgentId::new("research-1"),
        knowledge_base: Arc::new(Mutex::new(knowledge_base)),
    });

    let risk_agent = Arc::new(RiskAnalysisAgent {
        id: AgentId::new("risk-1"),
        risk_threshold: 0.7,
    });

    let trading_agent = Arc::new(TradingAgent {
        id: AgentId::new("trading-1"),
        execution_delay: Duration::from_millis(50),
        should_fail: false,
    });

    registry.register_agent(research_agent).await.unwrap();
    registry.register_agent(risk_agent).await.unwrap();
    registry.register_agent(trading_agent).await.unwrap();

    // Step 1: Research
    let research_task = Task::new(
        TaskType::Research,
        json!({"query": "BTC market analysis"}),
    );

    let research_result = dispatcher.dispatch_task(research_task).await.unwrap();
    assert!(research_result.is_success());
    
    let research_data = research_result.data().unwrap();
    let recommendation = research_data.get("data")
        .and_then(|d| d.get("recommendation"))
        .and_then(|r| r.as_str())
        .unwrap();

    // Step 2: Risk analysis based on research
    if recommendation == "buy" {
        let risk_task = Task::new(
            TaskType::RiskAnalysis,
            json!({"symbol": "BTC/USD", "amount": 1.5}),
        );

        let risk_result = dispatcher.dispatch_task(risk_task).await.unwrap();
        assert!(risk_result.is_success());
        
        let risk_data = risk_result.data().unwrap();
        let approved = risk_data.get("approved").and_then(|v| v.as_bool()).unwrap();

        // Step 3: Execute trade if approved
        if approved {
            let trade_task = Task::new(
                TaskType::Trading,
                json!({
                    "symbol": "BTC/USD",
                    "action": "buy",
                    "amount": 1.5
                }),
            );

            let trade_result = dispatcher.dispatch_task(trade_task).await.unwrap();
            assert!(trade_result.is_success());
            
            let trade_data = trade_result.data().unwrap();
            assert!(trade_data.get("tx_hash").is_some());
        }
    }
}

/// Test workflow error handling and recovery.
#[tokio::test]
async fn test_workflow_error_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        max_retries: 2,
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

    // Register a failing agent and a backup agent
    let failing_agent = Arc::new(TradingAgent {
        id: AgentId::new("failing-trader"),
        execution_delay: Duration::from_millis(10),
        should_fail: true,
    });

    let backup_agent = Arc::new(TradingAgent {
        id: AgentId::new("backup-trader"),
        execution_delay: Duration::from_millis(10),
        should_fail: false,
    });

    registry.register_agent(failing_agent).await.unwrap();
    registry.register_agent(backup_agent).await.unwrap();

    // Create a task that will initially fail but should be retried
    let task = Task::new(
        TaskType::Trading,
        json!({"symbol": "BTC/USD", "action": "buy", "amount": 1.0}),
    );

    // The dispatcher will try the failing agent first (based on routing)
    // Since failures are retriable, it will return the failure result
    let result = dispatcher.dispatch_task(task).await.unwrap();
    assert!(!result.is_success());
    assert!(result.is_retriable());
}

/// Test complex multi-stage workflow with multiple agent types.
#[tokio::test]
async fn test_complex_multi_stage_workflow() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Setup agents
    let knowledge_base = Arc::new(Mutex::new(HashMap::from([
        ("ETH analysis".to_string(), json!({
            "trend": "bullish",
            "volatility": "high",
            "support_level": 3000.0,
            "resistance_level": 4000.0
        })),
        ("portfolio status".to_string(), json!({
            "cash_balance": 10000.0,
            "positions": {"BTC": 0.5, "ETH": 2.0},
            "total_value": 15000.0
        }))
    ])));

    let research_agent = Arc::new(ResearchAgent {
        id: AgentId::new("research-main"),
        knowledge_base: knowledge_base.clone(),
    });

    let risk_agent = Arc::new(RiskAnalysisAgent {
        id: AgentId::new("risk-main"),
        risk_threshold: 0.6,
    });

    let trading_agent = Arc::new(TradingAgent {
        id: AgentId::new("trading-main"),
        execution_delay: Duration::from_millis(30),
        should_fail: false,
    });

    registry.register_agent(research_agent).await.unwrap();
    registry.register_agent(risk_agent).await.unwrap();
    registry.register_agent(trading_agent).await.unwrap();

    // Stage 1: Portfolio analysis
    let portfolio_task = Task::new(
        TaskType::Research,
        json!({"query": "portfolio status"}),
    );

    let portfolio_result = dispatcher.dispatch_task(portfolio_task).await.unwrap();
    assert!(portfolio_result.is_success());

    // Stage 2: Market research
    let research_task = Task::new(
        TaskType::Research,
        json!({"query": "ETH analysis"}),
    );

    let research_result = dispatcher.dispatch_task(research_task).await.unwrap();
    assert!(research_result.is_success());

    // Stage 3: Risk assessment
    let risk_task = Task::new(
        TaskType::RiskAnalysis,
        json!({"symbol": "ETH/USD", "amount": 3.0}),
    );

    let risk_result = dispatcher.dispatch_task(risk_task).await.unwrap();
    assert!(risk_result.is_success());

    let approved = risk_result.data().unwrap()
        .get("approved").and_then(|v| v.as_bool()).unwrap_or(false);

    // Stage 4: Execute trade if approved
    if approved {
        let trade_task = Task::new(
            TaskType::Trading,
            json!({
                "symbol": "ETH/USD",
                "action": "buy",
                "amount": 3.0
            }),
        );

        let trade_result = dispatcher.dispatch_task(trade_task).await.unwrap();
        assert!(trade_result.is_success());
        
        // Verify trade execution
        let trade_data = trade_result.data().unwrap();
        assert_eq!(trade_data.get("symbol").unwrap().as_str().unwrap(), "ETH/USD");
        assert_eq!(trade_data.get("action").unwrap().as_str().unwrap(), "buy");
        assert_eq!(trade_data.get("amount").unwrap().as_f64().unwrap(), 3.0);
    }
}

/// Test agent load balancing in workflows.
#[tokio::test]
async fn test_workflow_load_balancing() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        enable_load_balancing: true,
        routing_strategy: RoutingStrategy::LeastLoaded,
        ..Default::default()
    };
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

    // Register multiple agents with different execution times
    let fast_agent = Arc::new(TradingAgent {
        id: AgentId::new("fast-trader"),
        execution_delay: Duration::from_millis(10),
        should_fail: false,
    });

    let slow_agent = Arc::new(TradingAgent {
        id: AgentId::new("slow-trader"),
        execution_delay: Duration::from_millis(100),
        should_fail: false,
    });

    registry.register_agent(fast_agent).await.unwrap();
    registry.register_agent(slow_agent).await.unwrap();

    // Execute multiple tasks to test load balancing
    let tasks: Vec<Task> = (0..4)
        .map(|i| {
            Task::new(
                TaskType::Trading,
                json!({"symbol": format!("COIN{}/USD", i), "action": "buy"}),
            )
        })
        .collect();

    let results = dispatcher.dispatch_tasks(tasks).await;
    
    // All tasks should complete successfully
    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|r| r.is_ok()));

    // Check dispatcher statistics
    let stats = dispatcher.stats().await.unwrap();
    assert_eq!(stats.registered_agents, 2);
}

/// Test workflow with task prioritization.
#[tokio::test]
async fn test_workflow_task_prioritization() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let trading_agent = Arc::new(TradingAgent {
        id: AgentId::new("priority-trader"),
        execution_delay: Duration::from_millis(20),
        should_fail: false,
    });

    registry.register_agent(trading_agent).await.unwrap();

    // Create tasks with different priorities
    let low_priority_task = Task::new(
        TaskType::Trading,
        json!({"symbol": "LOW/USD", "action": "buy"}),
    ).with_priority(Priority::Low);

    let high_priority_task = Task::new(
        TaskType::Trading,
        json!({"symbol": "HIGH/USD", "action": "buy"}),
    ).with_priority(Priority::High);

    let critical_task = Task::new(
        TaskType::Trading,
        json!({"symbol": "CRITICAL/USD", "action": "sell"}),
    ).with_priority(Priority::Critical);

    // Execute tasks (priority should be handled by routing in a real scenario)
    let critical_result = dispatcher.dispatch_task(critical_task).await.unwrap();
    let high_result = dispatcher.dispatch_task(high_priority_task).await.unwrap();
    let low_result = dispatcher.dispatch_task(low_priority_task).await.unwrap();

    // All should complete successfully
    assert!(critical_result.is_success());
    assert!(high_result.is_success());
    assert!(low_result.is_success());
}

/// Test workflow timeout handling.
#[tokio::test]
async fn test_workflow_timeout_handling() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        default_task_timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

    // Register a slow agent that will timeout
    let slow_agent = Arc::new(TradingAgent {
        id: AgentId::new("timeout-trader"),
        execution_delay: Duration::from_millis(100), // Longer than timeout
        should_fail: false,
    });

    registry.register_agent(slow_agent).await.unwrap();

    let task = Task::new(
        TaskType::Trading,
        json!({"symbol": "SLOW/USD", "action": "buy"}),
    );

    // Task should timeout
    let result = dispatcher.dispatch_task(task).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AgentError::TaskTimeout { .. }));
}

/// Test workflow with custom task timeouts.
#[tokio::test]
async fn test_workflow_custom_timeouts() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let agent = Arc::new(TradingAgent {
        id: AgentId::new("custom-timeout-trader"),
        execution_delay: Duration::from_millis(80),
        should_fail: false,
    });

    registry.register_agent(agent).await.unwrap();

    // Task with custom timeout that should succeed
    let task_success = Task::new(
        TaskType::Trading,
        json!({"symbol": "SUCCESS/USD", "action": "buy"}),
    ).with_timeout(Duration::from_millis(150));

    let result = dispatcher.dispatch_task(task_success).await.unwrap();
    assert!(result.is_success());

    // Task with custom timeout that should fail
    let task_timeout = Task::new(
        TaskType::Trading,
        json!({"symbol": "TIMEOUT/USD", "action": "buy"}),
    ).with_timeout(Duration::from_millis(50));

    let result = dispatcher.dispatch_task(task_timeout).await;
    assert!(result.is_err());
}

/// Performance test for high-throughput workflows.
#[tokio::test]
#[ignore] // Ignore for regular test runs
async fn test_high_throughput_workflow() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Register multiple fast agents
    for i in 0..10 {
        let agent = Arc::new(TradingAgent {
            id: AgentId::new(&format!("fast-agent-{}", i)),
            execution_delay: Duration::from_millis(1),
            should_fail: false,
        });
        registry.register_agent(agent).await.unwrap();
    }

    // Create a large number of tasks
    let task_count = 1000;
    let tasks: Vec<Task> = (0..task_count)
        .map(|i| {
            Task::new(
                TaskType::Trading,
                json!({"symbol": format!("COIN{}/USD", i % 100), "action": "buy"}),
            )
        })
        .collect();

    // Measure execution time
    let start_time = std::time::Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let execution_time = start_time.elapsed();

    // Verify results
    assert_eq!(results.len(), task_count);
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    
    println!("Executed {} tasks in {:?}, {} successful", 
             task_count, execution_time, success_count);
    
    // Should handle high throughput efficiently
    assert!(execution_time < Duration::from_secs(5));
    assert!(success_count > task_count * 95 / 100); // At least 95% success rate
}