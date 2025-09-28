/// Basic Agents Example
///
/// A minimal working example that demonstrates:
/// - Creating agents with different capabilities
/// - Registering agents in a registry
/// - Dispatching tasks to appropriate agents
/// - Basic task execution
///
/// Run with: cargo run --example `basic_agents`
use async_trait::async_trait;
use core::{error::Error, time::Duration};
use riglr_agents::{
    Agent, AgentId, AgentRegistry, CapabilityType, Dispatcher, LocalAgentRegistry, Priority, Task,
    TaskResult, TaskType,
};
use serde_json::json;
use std::sync::Arc;
use tokio::time::sleep;

/// A simple trading agent
#[derive(Debug)]
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

        // Simulate trading logic
        println!("  🔹 Analyzing {action} order for {symbol}");

        // Simulate trade execution
        sleep(Duration::from_millis(100)).await;

        println!("  ✅ Executing {action} order for {symbol}");

        Ok(TaskResult::success(
            json!({
                "trade_id": uuid::Uuid::new_v4().to_string(),
                "symbol": symbol,
                "action": action,
                "status": "completed",
                "trader": self.id.as_str(),
                "timestamp": chrono::Utc::now().timestamp()
            }),
            None,
            Duration::from_millis(100),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<CapabilityType> {
        vec![
            CapabilityType::Trading,
            CapabilityType::Custom("execution".to_string()),
        ]
    }
}

/// A simple research agent with mock LLM operations
#[derive(Debug)]
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

        // Simulate intelligent analysis with mock data
        let llm_analysis = Some(
            "Mock analysis: The market shows bullish trends with strong momentum indicators."
                .to_string(),
        );
        println!("  ℹ️ Using simulated analysis for {symbol}");

        // Simulate research work
        sleep(Duration::from_millis(50)).await;

        println!("  🔹 Analyzing market data for {symbol}");

        Ok(TaskResult::success(
            json!({
                "symbol": symbol,
                "analysis": {
                    "trend": "bullish",
                    "strength": 8.2,
                    "support": 45000,
                    "resistance": 52000
                },
                "confidence": 0.85,
                "recommendation": "BUY",
                "analyst": self.id.as_str(),
                "llm_insights": llm_analysis,
                "timestamp": chrono::Utc::now().timestamp()
            }),
            None,
            Duration::from_millis(50),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<CapabilityType> {
        vec![
            CapabilityType::Research,
            CapabilityType::Custom("analysis".to_string()),
        ]
    }
}

/// A simple risk management agent with mock LLM operations
#[derive(Debug)]
struct RiskAgent {
    id: AgentId,
}

impl RiskAgent {
    fn new(id: &str) -> Self {
        Self {
            id: AgentId::new(id),
        }
    }
}

#[async_trait]
impl Agent for RiskAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("⚖️ Risk Agent {} executing task: {}", self.id, task.id);

        let symbol = task
            .parameters
            .get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("BTC");

        let amount = task
            .parameters
            .get("amount")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(1.0);

        // Simulate intelligent risk assessment with mock data
        let llm_assessment = Some(
            "Mock assessment: Risk level is moderate, position size is within acceptable limits."
                .to_string(),
        );
        println!("  ℹ️ Using simulated risk assessment for {amount} {symbol}");

        // Simulate risk assessment
        sleep(Duration::from_millis(30)).await;

        let risk_score = if amount > 5.0 { 0.8 } else { 0.3 };
        let approved = risk_score < 0.7;

        println!("  🔹 Risk assessment for {amount} {symbol} - Score: {risk_score:.2}");

        Ok(TaskResult::success(
            json!({
                "symbol": symbol,
                "amount": amount,
                "risk_score": risk_score,
                "approved": approved,
                "max_position": 10.0,
                "recommendation": if approved { "APPROVE" } else { "REDUCE_SIZE" },
                "assessor": self.id.as_str(),
                "llm_assessment": llm_assessment,
                "timestamp": chrono::Utc::now().timestamp()
            }),
            None,
            Duration::from_millis(30),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<CapabilityType> {
        vec![
            CapabilityType::RiskAnalysis,
            CapabilityType::Custom("compliance".to_string()),
        ]
    }
}

async fn setup_agents() -> Result<Arc<LocalAgentRegistry>, Box<dyn Error + Send + Sync>> {
    let trading_agent = Arc::new(TradingAgent::new("trader-001"));
    let research_agent = Arc::new(ResearchAgent::new("researcher-001"));
    let risk_agent = Arc::new(RiskAgent::new("risk-001"));

    let registry = Arc::new(LocalAgentRegistry::new());
    registry.register_agent(trading_agent).await?;
    registry.register_agent(research_agent).await?;
    registry.register_agent(risk_agent).await?;

    println!(
        "✅ Registered {} agents in the system\n",
        registry.agent_count().await?
    );

    Ok(registry)
}

async fn execute_trading_workflow<R: AgentRegistry>(
    dispatcher: &Dispatcher<R>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Phase 1: Market Research
    println!("🔬 Phase 1: Market Research");
    let research_task = Task::new(
        TaskType::Research,
        json!({
            "symbol": "BTC",
            "analysis_type": "technical",
            "timeframe": "1d"
        }),
    )
    .with_priority(Priority::High);

    let research_result = dispatcher.dispatch_task(research_task).await?;
    if let Some(data) = research_result.data() {
        let recommendation = data
            .get("recommendation")
            .and_then(|r| r.as_str())
            .unwrap_or("HOLD");
        let confidence = data
            .get("confidence")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        println!(
            "✅ Research completed: {} (confidence: {:.1}%)\n",
            recommendation,
            confidence * 100.0
        );
    }

    // Phase 2: Risk Assessment
    println!("⚖️ Phase 2: Risk Assessment");
    let risk_task = Task::new(
        TaskType::RiskAnalysis,
        json!({
            "symbol": "BTC",
            "amount": 2.5,
            "action": "buy"
        }),
    )
    .with_priority(Priority::High);

    let risk_result = dispatcher.dispatch_task(risk_task).await?;
    let risk_approved = risk_result.data().is_some_and(|data| {
        let approved = data
            .get("approved")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let risk_score = data
            .get("risk_score")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        println!(
            "✅ Risk assessment: {} (score: {:.2})\n",
            if approved { "APPROVED" } else { "REJECTED" },
            risk_score
        );
        approved
    });

    // Phase 3: Trade Execution
    println!("💰 Phase 3: Trade Execution");
    if risk_approved {
        let trading_task = Task::new(
            TaskType::Trading,
            json!({
                "symbol": "BTC",
                "action": "buy",
                "amount": 2.5
            }),
        )
        .with_priority(Priority::High);

        let trading_result = dispatcher.dispatch_task(trading_task).await?;
        if let Some(data) = trading_result.data() {
            let trade_id = data
                .get("trade_id")
                .and_then(|id| id.as_str())
                .unwrap_or("unknown");
            let status = data
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            println!("✅ Trade executed: {trade_id} (status: {status})\n");
        }
    } else {
        println!("❌ Trade execution cancelled due to risk assessment\n");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("🚀 Starting Basic Agents Example");
    println!("🤖 Demonstrating multi-agent task coordination with mock LLM integration\n");
    println!("ℹ️ Using simulated LLM responses for all agents");

    let registry = setup_agents().await?;
    let dispatcher = Dispatcher::new(registry.clone());

    execute_trading_workflow(&dispatcher).await?;

    // Display final agent information
    println!("📊 Agent Summary:");
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

    println!("\n🎉 Basic agents example completed successfully!");
    println!("This demonstrated:");
    println!("  ✅ Multi-agent task routing based on capabilities");
    println!("  ✅ Sequential task coordination (research → risk → trading)");
    println!("  ✅ Conditional execution based on previous results");
    println!("  ✅ Agent registry management and status reporting");
    println!("  ✅ Mock integration for LLM-powered agent intelligence");

    Ok(())
}
