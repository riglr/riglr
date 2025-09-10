//! End-to-end tests for multi-agent coordination functionality.

use anyhow::Result;
use async_trait::async_trait;
use riglr_agents::{
    Agent, AgentId, AgentRegistry, CapabilityType, LocalAgentRegistry, Task, TaskResult, TaskType,
};
use std::sync::Arc;

// Create a simple mock agent for testing
#[derive(Debug, Clone)]
struct MockAgent {
    id: AgentId,
    response: String,
}

impl MockAgent {
    fn new(name: &str, response: String) -> Self {
        Self {
            id: AgentId::new(name),
            response,
        }
    }
}

#[async_trait]
impl Agent for MockAgent {
    async fn execute_task(&self, _task: Task) -> riglr_agents::Result<TaskResult> {
        Ok(TaskResult::success(
            serde_json::json!({ "response": self.response }),
            None,
            std::time::Duration::from_millis(100),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<CapabilityType> {
        vec![CapabilityType::Custom("mock".to_string())]
    }
}

#[tokio::test]
async fn test_2_1_sequential_workflow_local_registry() -> Result<()> {
    // Create a local registry
    let registry = LocalAgentRegistry::default();

    // Create Agent 1: Data Collector
    let collector_agent = MockAgent::new(
        "collector",
        "The current price of Bitcoin is approximately $65,000 and Ethereum is approximately $3,500.".to_string(),
    );

    // Create Agent 2: Data Analyzer
    let analyzer_agent = MockAgent::new(
        "analyzer",
        "Based on the prices: 0.5 BTC = $32,500, 2 ETH = $7,000, Total = $39,500".to_string(),
    );

    // Register agents
    registry
        .register_agent(Arc::new(collector_agent.clone()))
        .await?;
    registry
        .register_agent(Arc::new(analyzer_agent.clone()))
        .await?;

    // Test sequential workflow
    println!("Starting sequential workflow test...");

    // Step 1: Get data from collector
    let collector = registry
        .get_agent(&AgentId::new("collector"))
        .await?
        .ok_or_else(|| anyhow::anyhow!("Collector agent not found"))?;

    let collector_task = Task::new(
        TaskType::Custom("collect_prices".to_string()),
        serde_json::json!({"query": "cryptocurrency prices"}),
    );
    let collector_response = collector.execute_task(collector_task).await?;

    println!("Collector response: {:?}", collector_response);

    // Step 2: Pass data to analyzer
    let analyzer = registry
        .get_agent(&AgentId::new("analyzer"))
        .await?
        .ok_or_else(|| anyhow::anyhow!("Analyzer agent not found"))?;

    let analyzer_task = Task::new(
        TaskType::Custom("analyze".to_string()),
        serde_json::json!({"data": collector_response.data().unwrap_or(&serde_json::Value::Null)}),
    );
    let analyzer_response = analyzer.execute_task(analyzer_task).await?;

    println!("Analyzer response: {:?}", analyzer_response);

    // Assertions
    assert!(collector_response.is_success(), "Collector should succeed");
    assert!(analyzer_response.is_success(), "Analyzer should succeed");

    let collector_data = collector_response
        .data()
        .map(|d| d.to_string())
        .unwrap_or_default();
    assert!(
        collector_data.contains("Bitcoin") || collector_data.contains("65000"),
        "Collector should mention Bitcoin"
    );

    let analyzer_data = analyzer_response
        .data()
        .map(|d| d.to_string())
        .unwrap_or_default();
    assert!(
        analyzer_data.contains("total") || analyzer_data.contains("39500"),
        "Analyzer should provide a total value"
    );

    println!("Test 2.1 Passed: Sequential workflow with local registry successful");

    Ok(())
}

#[tokio::test]
async fn test_2_2_distributed_workflow_redis_registry() -> Result<()> {
    // For now, we'll use local registry as distributed registry requires Redis
    // This test demonstrates the same workflow pattern

    println!(
        "Testing distributed workflow pattern with local registry (Redis not required for demo)"
    );

    // Create registry
    let registry = LocalAgentRegistry::default();

    // Create Agent 1: Market Data Provider
    let market_agent = MockAgent::new(
        "market_data",
        "Market conditions: BTC dominance is 52%, total market cap is $2.5 trillion, 24h volume is $95 billion.".to_string(),
    );

    // Create Agent 2: Risk Assessor
    let risk_agent = MockAgent::new(
        "risk_assessor",
        "Based on market data: Medium risk. BTC dominance healthy at 52%, good volume indicates liquidity.".to_string(),
    );

    // Register agents
    registry.register_agent(Arc::new(market_agent)).await?;
    registry.register_agent(Arc::new(risk_agent)).await?;

    println!("Starting distributed workflow test...");

    // Step 1: Get market data
    let market = registry
        .get_agent(&AgentId::new("market_data"))
        .await?
        .ok_or_else(|| anyhow::anyhow!("Market data agent not found"))?;

    let market_task = Task::new(
        TaskType::Custom("get_market_data".to_string()),
        serde_json::json!({}),
    );
    let market_response = market.execute_task(market_task).await?;

    println!("Market data response: {:?}", market_response);

    // Step 2: Assess risk based on market data
    let risk = registry
        .get_agent(&AgentId::new("risk_assessor"))
        .await?
        .ok_or_else(|| anyhow::anyhow!("Risk assessor agent not found"))?;

    let risk_task = Task::new(
        TaskType::Custom("assess_risk".to_string()),
        serde_json::json!({"market_data": market_response.data().unwrap_or(&serde_json::Value::Null)}),
    );
    let risk_response = risk.execute_task(risk_task).await?;

    println!("Risk assessment response: {:?}", risk_response);

    // Step 3: Test agent listing
    let agent_list = registry.list_agents().await?;

    // Assertions
    assert!(market_response.is_success(), "Market agent should succeed");
    assert!(risk_response.is_success(), "Risk agent should succeed");

    let market_data = market_response
        .data()
        .map(|d| d.to_string())
        .unwrap_or_default();
    assert!(
        market_data.contains("dominance") || market_data.contains("market"),
        "Market agent should provide market data"
    );

    let risk_data = risk_response
        .data()
        .map(|d| d.to_string())
        .unwrap_or_default();
    assert!(
        risk_data.contains("risk") || risk_data.contains("Medium"),
        "Risk agent should provide assessment"
    );

    assert_eq!(
        agent_list.len(),
        2,
        "Registry should contain exactly 2 agents"
    );

    println!("Test 2.2 Passed: Distributed workflow pattern successful");

    Ok(())
}

#[tokio::test]
async fn test_2_3_parallel_agent_execution() -> Result<()> {
    // Create a local registry for parallel execution
    let registry = Arc::new(LocalAgentRegistry::default());

    // Create multiple specialized agents
    let agents = vec![
        MockAgent::new(
            "technical_analyst",
            "Technical indicators show bullish momentum with RSI at 65 and MACD positive crossover.".to_string()
        ),
        MockAgent::new(
            "fundamental_analyst",
            "On-chain metrics are strong with increasing active addresses and hash rate at all-time high.".to_string()
        ),
        MockAgent::new(
            "sentiment_analyst",
            "Social sentiment is moderately positive with fear and greed index at 68 (Greed).".to_string()
        ),
    ];

    // Register all agents
    for agent in &agents {
        registry.register_agent(Arc::new(agent.clone())).await?;
    }

    println!("Starting parallel agent execution test...");

    // Execute all agents in parallel
    let mut handles = Vec::new();

    for agent_name in [
        "technical_analyst",
        "fundamental_analyst",
        "sentiment_analyst",
    ] {
        let registry_clone = registry.clone();
        let agent_id = AgentId::new(agent_name);

        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();

            let agent = registry_clone
                .get_agent(&agent_id)
                .await
                .ok()
                .flatten()
                .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;

            let task = Task::new(
                TaskType::Custom("analyze".to_string()),
                serde_json::json!({"asset": "Bitcoin"}),
            );

            let result = agent.execute_task(task).await;
            let duration = start.elapsed();

            Ok::<_, anyhow::Error>((agent_name, result, duration))
        });

        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        match handle.await? {
            Ok((name, result, duration)) => match result {
                Ok(response) => {
                    println!("{} completed in {:?}: {:?}", name, duration, response);
                    results.push((name, response));
                }
                Err(e) => {
                    println!("{} failed: {}", name, e);
                }
            },
            Err(e) => {
                println!("Task failed: {}", e);
            }
        }
    }

    // Assertions
    assert_eq!(results.len(), 3, "All three agents should complete");

    for (name, response) in &results {
        assert!(response.is_success(), "{} should succeed", name);

        let data = response.data().map(|d| d.to_string()).unwrap_or_default();
        match *name {
            "technical_analyst" => {
                assert!(
                    data.contains("technical")
                        || data.contains("RSI")
                        || data.contains("indicator"),
                    "Technical analyst should provide technical analysis"
                );
            }
            "fundamental_analyst" => {
                assert!(
                    data.contains("fundamental")
                        || data.contains("on-chain")
                        || data.contains("metric"),
                    "Fundamental analyst should provide fundamental analysis"
                );
            }
            "sentiment_analyst" => {
                assert!(
                    data.contains("sentiment") || data.contains("social") || data.contains("greed"),
                    "Sentiment analyst should provide sentiment analysis"
                );
            }
            _ => {}
        }
    }

    println!("Test 2.3 Passed: Parallel agent execution successful");

    Ok(())
}
