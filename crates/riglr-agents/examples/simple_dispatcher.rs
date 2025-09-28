/// Simple Agent Dispatcher Example with rig-core integration
///
/// A minimal working example that demonstrates:
/// - Creating agents with different capabilities enhanced by rig-core
/// - Registering agents in a registry
/// - Dispatching tasks to appropriate agents
/// - Basic error handling
/// - Integration with rig-core for LLM-powered agent intelligence
///
/// Run with: cargo run --example `simple_dispatcher`
use async_trait::async_trait;
use core::{error::Error, time::Duration};
use riglr_agents::{
    Agent, AgentId, AgentRegistry, CapabilityType, Dispatcher, LocalAgentRegistry, Priority, Task,
    TaskResult, TaskType,
};
// Removed riglr_core and rig_core imports - using mock implementations
use serde_json::json;
use std::sync::Arc;
use tokio::time::sleep;

/// A simple trading agent with mock implementations
#[derive(Clone, Debug)]
struct TradingAgent {
    id: AgentId,
}

impl TradingAgent {
    fn new(id: &str) -> Self {
        Self {
            id: AgentId::new(id),
        }
    }
}

#[async_trait]
impl Agent for TradingAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("💰 Trading Agent {} executing task: {}", self.id, task.id);

        let symbol = task
            .parameters
            .get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("BTC");

        let action = task
            .parameters
            .get("action")
            .and_then(|s| s.as_str())
            .unwrap_or("buy");

        let amount = task
            .parameters
            .get("amount")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(1.0);

        // Use mock analysis
        let llm_analysis = Some(format!("Mock analysis: {action} {amount} units of {symbol} appears to be a good trade based on current market conditions."));

        // Simulate trade execution
        sleep(Duration::from_millis(100)).await;

        let mut result_data = json!({
            "trade_id": uuid::Uuid::new_v4().to_string(),
            "symbol": symbol,
            "action": action,
            "amount": amount,
            "status": "completed",
            "trader": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        });

        if let Some(analysis) = llm_analysis {
            if let serde_json::Value::Object(ref mut map) = result_data {
                map.insert(
                    "llm_analysis".to_string(),
                    serde_json::Value::String(analysis),
                );
            }
        }

        Ok(TaskResult::success(
            result_data,
            None,
            Duration::from_millis(100),
        ))
    }

    fn capabilities(&self) -> Vec<CapabilityType> {
        vec![CapabilityType::Trading]
    }

    fn id(&self) -> &AgentId {
        &self.id
    }
}

/// A simple research agent with mock implementations
#[derive(Clone, Debug)]
struct ResearchAgent {
    id: AgentId,
}

impl ResearchAgent {
    fn new(id: &str) -> Self {
        Self {
            id: AgentId::new(id),
        }
    }
}

#[async_trait]
impl Agent for ResearchAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("🔬 Research Agent {} executing task: {}", self.id, task.id);

        let symbol = task
            .parameters
            .get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("BTC");

        let analysis_type = task
            .parameters
            .get("analysis_type")
            .and_then(|t| t.as_str())
            .unwrap_or("technical");

        // Use mock analysis with simulated responses
        let llm_analysis = Some(format!(
            "Mock {analysis_type} analysis for {symbol}: Market shows positive momentum with bullish indicators."
        ));
        let llm_recommendation = "BUY";
        let llm_confidence = 0.85;

        // Simulate research work
        sleep(Duration::from_millis(50)).await;

        let mut result_data = json!({
            "symbol": symbol,
            "analysis_type": analysis_type,
            "analysis": {
                "trend": "bullish",
                "confidence": llm_confidence
            },
            "recommendation": llm_recommendation,
            "analyst": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        });

        if let Some(analysis) = llm_analysis {
            if let serde_json::Value::Object(ref mut map) = result_data {
                map.insert(
                    "llm_analysis".to_string(),
                    serde_json::Value::String(analysis),
                );
            }
        }

        Ok(TaskResult::success(
            result_data,
            None,
            Duration::from_millis(50),
        ))
    }

    fn capabilities(&self) -> Vec<CapabilityType> {
        vec![CapabilityType::Research]
    }

    fn id(&self) -> &AgentId {
        &self.id
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("🚀 Starting Simple Agent Dispatcher Example with mock LLM integration");

    // Create agents with different capabilities using mock implementations
    println!("⚠️ Using mock LLM implementations for all agents");

    let trading_agent = Arc::new(TradingAgent::new("trader-001"));
    let research_agent = Arc::new(ResearchAgent::new("researcher-001"));

    // Create agent registry and register agents
    let registry = Arc::new(LocalAgentRegistry::new());
    registry.register_agent(trading_agent).await?;
    registry.register_agent(research_agent).await?;

    println!("✅ Registered {} agents", registry.agent_count().await?);

    // Create dispatcher
    let dispatcher = Dispatcher::new(registry.clone());

    // Execute example tasks directly (no signer context needed for mock implementation)
    {
        println!("\n🔬 Dispatching Research Task");
        let research_task = Task::new(
            TaskType::Research,
            json!({
                "symbol": "BTC",
                "analysis_type": "technical"
            }),
        )
        .with_priority(Priority::High);

        match dispatcher.dispatch_task(research_task).await {
            Ok(result) => {
                if let Some(data) = result.data() {
                    let recommendation = data
                        .get("recommendation")
                        .and_then(|r| r.as_str())
                        .unwrap_or("HOLD");
                    println!("✅ Research completed: {recommendation}");
                } else {
                    println!("✅ Research completed successfully");
                }
            }
            Err(e) => eprintln!("❌ Research failed: {e}"),
        }

        println!("\n💰 Dispatching Trading Task");
        let trading_task = Task::new(
            TaskType::Trading,
            json!({
                "symbol": "BTC",
                "action": "buy",
                "amount": 1.0
            }),
        )
        .with_priority(Priority::High);

        match dispatcher.dispatch_task(trading_task).await {
            Ok(result) => {
                if let Some(data) = result.data() {
                    let trade_id = data
                        .get("trade_id")
                        .and_then(|id| id.as_str())
                        .unwrap_or("unknown");
                    println!("✅ Trade executed: {trade_id}");
                } else {
                    println!("✅ Trade completed successfully");
                }
            }
            Err(e) => eprintln!("❌ Trade failed: {e}"),
        }
    }

    // Display final agent statuses
    println!("\n📊 Agent Status:");
    let agents = registry.list_agents().await?;
    for agent in agents {
        let status = agent.status();
        println!(
            "  {} - {} capabilities: {:?}",
            status.agent_id,
            status.capabilities.len(),
            status
                .capabilities
                .iter()
                .map(|c| &c.name)
                .collect::<Vec<_>>()
        );
    }

    println!("\n🎉 Simple dispatcher example with mock LLM integration completed successfully!");
    println!("The example demonstrated:");
    println!("  - Basic agent creation with mock LLM capabilities");
    println!("  - Task routing based on agent capabilities");
    println!("  - Mock trading and research analysis");
    println!("  - Simplified implementation without external dependencies");

    Ok(())
}
