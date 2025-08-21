//! Basic Agent Dispatcher Example
//!
//! This example demonstrates how to create a simple multi-agent system with
//! task dispatching using rig-core integration. It shows:
//! - Creating agents with different capabilities using rig-core
//! - Registering agents in a registry
//! - Dispatching tasks to appropriate agents
//! - Basic inter-agent communication
//! - LLM-powered agent intelligence
//!
//! Run with: cargo run --example basic_dispatcher

use async_trait::async_trait;
// Removed rig_core and riglr_core imports - using mock implementations
use riglr_agents::{
    Agent, AgentCommunication, AgentDispatcher, AgentId, AgentMessage, AgentRegistry,
    ChannelCommunication, DispatchConfig, LocalAgentRegistry, Priority, RoutingStrategy, Task,
    TaskResult, TaskType,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// A simple research agent with mock LLM operations
#[derive(Debug)]
struct ResearchAgent {
    id: AgentId,
    communication: Arc<ChannelCommunication>,
}

impl ResearchAgent {
    fn new(id: &str, communication: Arc<ChannelCommunication>) -> Self {
        Self {
            id: AgentId::new(id),
            communication,
        }
    }

    async fn analyze_market(&self, symbol: &str) -> serde_json::Value {
        // Use mock analysis
        let llm_analysis = Some(format!(
            "Mock analysis for {}: Market shows positive momentum with strong technicals.",
            symbol
        ));

        // Simulate research work
        sleep(Duration::from_millis(100)).await;

        json!({
            "symbol": symbol,
            "price": 50000,
            "trend": "bullish",
            "sentiment": "positive",
            "volume": "high",
            "llm_analysis": llm_analysis,
            "research_timestamp": chrono::Utc::now().timestamp(),
            "analyst": self.id.as_str()
        })
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

        // Perform market analysis
        let analysis = self.analyze_market(symbol).await;

        // Broadcast research results to other agents
        let message = AgentMessage::new(
            self.id.clone(),
            None, // Broadcast to all
            "market_analysis".to_string(),
            analysis.clone(),
        );

        if let Err(e) = self.communication.broadcast_message(message).await {
            eprintln!("Failed to broadcast research results: {}", e);
        }

        Ok(TaskResult::success(
            json!({
                "analysis": analysis,
                "recommendations": ["HOLD", "MONITOR"],
                "confidence": 0.85
            }),
            None, // No transaction hash for research
            Duration::from_millis(100),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "research".to_string(),
            "market_analysis".to_string(),
            "sentiment_analysis".to_string(),
        ]
    }

    async fn handle_message(&self, message: AgentMessage) -> riglr_agents::Result<()> {
        println!(
            "🔬 Research Agent {} received message: {}",
            self.id, message.message_type
        );
        Ok(())
    }
}

/// A trading agent with mock LLM operations
#[derive(Debug)]
struct TradingAgent {
    id: AgentId,
    communication: Arc<ChannelCommunication>,
}

impl TradingAgent {
    fn new(id: &str, communication: Arc<ChannelCommunication>) -> Self {
        Self {
            id: AgentId::new(id),
            communication,
        }
    }

    async fn execute_trade(&self, action: &str, symbol: &str, amount: f64) -> serde_json::Value {
        // Use mock trade execution planning
        let llm_strategy = Some(format!(
            "Mock execution strategy for {} {} units of {}: Use TWAP execution over 5 minutes.",
            action, amount, symbol
        ));

        // Simulate trade execution
        sleep(Duration::from_millis(200)).await;

        // In a real implementation, this would use SignerContext::current()
        // to access blockchain signers and execute actual trades
        println!("💰 Executing {} trade: {} {} units", action, symbol, amount);

        json!({
            "trade_id": uuid::Uuid::new_v4().to_string(),
            "symbol": symbol,
            "action": action,
            "amount": amount,
            "price": 50000,
            "status": "executed",
            "llm_strategy": llm_strategy,
            "timestamp": chrono::Utc::now().timestamp(),
            "trader": self.id.as_str()
        })
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
            .and_then(|s| s.as_f64())
            .unwrap_or(1.0);

        // Execute the trade
        let trade_result = self.execute_trade(action, symbol, amount).await;

        // Notify other agents of the trade execution
        let message = AgentMessage::new(
            self.id.clone(),
            None, // Broadcast to all
            "trade_executed".to_string(),
            trade_result.clone(),
        );

        if let Err(e) = self.communication.broadcast_message(message).await {
            eprintln!("Failed to broadcast trade execution: {}", e);
        }

        Ok(TaskResult::success(
            trade_result,
            None, // Transaction hash would be included here in real implementation
            Duration::from_millis(200),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "trading".to_string(),
            "execution".to_string(),
            "order_management".to_string(),
        ]
    }

    async fn handle_message(&self, message: AgentMessage) -> riglr_agents::Result<()> {
        match message.message_type.as_str() {
            "market_analysis" => {
                println!(
                    "💰 Trading Agent {} received market analysis from {}",
                    self.id, message.from
                );
                // Could use this analysis to adjust trading strategy
            }
            _ => {
                println!(
                    "💰 Trading Agent {} received message: {}",
                    self.id, message.message_type
                );
            }
        }
        Ok(())
    }
}

/// A risk management agent with mock LLM operations
#[derive(Debug)]
struct RiskAgent {
    id: AgentId,
    communication: Arc<ChannelCommunication>,
    max_position_size: f64,
}

impl RiskAgent {
    fn new(id: &str, communication: Arc<ChannelCommunication>) -> Self {
        Self {
            id: AgentId::new(id),
            communication,
            max_position_size: 10000.0, // Example position limit
        }
    }

    async fn assess_risk(&self, symbol: &str, amount: f64) -> serde_json::Value {
        // Use mock risk assessment
        let llm_risk_analysis = Some(format!("Mock risk assessment for {} units of {}: Position size is acceptable within risk parameters.", amount, symbol));

        // Simulate risk assessment
        sleep(Duration::from_millis(50)).await;

        let risk_score = if amount > self.max_position_size {
            0.9
        } else {
            0.3
        };
        let approved = risk_score < 0.8;

        json!({
            "symbol": symbol,
            "amount": amount,
            "risk_score": risk_score,
            "approved": approved,
            "max_position": self.max_position_size,
            "recommendation": if approved { "APPROVE" } else { "REJECT" },
            "llm_risk_analysis": llm_risk_analysis,
            "timestamp": chrono::Utc::now().timestamp(),
            "assessor": self.id.as_str()
        })
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
            .and_then(|s| s.as_f64())
            .unwrap_or(1.0);

        // Assess risk
        let risk_assessment = self.assess_risk(symbol, amount).await;

        // Broadcast risk assessment
        let message = AgentMessage::new(
            self.id.clone(),
            None, // Broadcast to all
            "risk_assessment".to_string(),
            risk_assessment.clone(),
        );

        if let Err(e) = self.communication.broadcast_message(message).await {
            eprintln!("Failed to broadcast risk assessment: {}", e);
        }

        Ok(TaskResult::success(
            risk_assessment,
            None,
            Duration::from_millis(50),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "risk_analysis".to_string(),
            "compliance".to_string(),
            "position_monitoring".to_string(),
        ]
    }

    async fn handle_message(&self, message: AgentMessage) -> riglr_agents::Result<()> {
        match message.message_type.as_str() {
            "trade_executed" => {
                println!(
                    "⚖️ Risk Agent {} monitoring trade execution from {}",
                    self.id, message.from
                );
                // Could trigger post-trade risk checks
            }
            _ => {
                println!(
                    "⚖️ Risk Agent {} received message: {}",
                    self.id, message.message_type
                );
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting Basic Agent Dispatcher Example with rig-core integration");

    // Initialize communication system
    let communication = Arc::new(ChannelCommunication::default());

    // Create agents with different capabilities using mock implementations
    println!("ℹ️ Using mock LLM implementations for all agents");

    let research_agent = Arc::new(ResearchAgent::new("research-001", communication.clone()));
    let trading_agent = Arc::new(TradingAgent::new("trader-001", communication.clone()));
    let risk_agent = Arc::new(RiskAgent::new("risk-001", communication.clone()));

    // Create agent registry and register agents
    let registry = Arc::new(LocalAgentRegistry::new());
    registry.register_agent(research_agent.clone()).await?;
    registry.register_agent(trading_agent.clone()).await?;
    registry.register_agent(risk_agent.clone()).await?;

    println!("✅ Registered {} agents", registry.agent_count().await?);

    // Create dispatcher with capability-based routing
    let dispatch_config = DispatchConfig {
        routing_strategy: RoutingStrategy::Capability,
        max_retries: 3,
        default_task_timeout: Duration::from_secs(30),
        retry_delay: Duration::from_secs(1),
        max_concurrent_tasks_per_agent: 5,
        enable_load_balancing: true,
    };

    let dispatcher = AgentDispatcher::with_config(registry.clone(), dispatch_config);

    // Execute example tasks directly (no signer context needed for mock implementation)
    {
        println!("\n🔬 Dispatching Research Task");
        let research_task = Task::new(
            TaskType::Research,
            json!({
                "symbol": "BTC",
                "depth": "comprehensive",
                "timeframe": "1h"
            }),
        )
        .with_priority(Priority::High);

        match dispatcher.dispatch_task(research_task).await {
            Ok(result) => {
                let default_json = serde_json::json!({});
                let data = result.data().unwrap_or(&default_json);
                let trend = data
                    .get("analysis")
                    .and_then(|a| a.get("trend"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");
                println!("✅ Research completed: {}", trend);
            }
            Err(e) => eprintln!("❌ Research failed: {}", e),
        }

        // Wait for messages to propagate
        sleep(Duration::from_millis(100)).await;

        println!("\n⚖️ Dispatching Risk Analysis Task");
        let risk_task = Task::new(
            TaskType::RiskAnalysis,
            json!({
                "symbol": "BTC",
                "amount": 5000.0,
                "action": "buy"
            }),
        )
        .with_priority(Priority::High);

        match dispatcher.dispatch_task(risk_task).await {
            Ok(result) => {
                let default_json = serde_json::json!({});
                let data = result.data().unwrap_or(&default_json);
                let approved = data
                    .get("approved")
                    .and_then(|a| a.as_bool())
                    .unwrap_or(false);
                println!(
                    "✅ Risk analysis completed: {}",
                    if approved { "APPROVED" } else { "REJECTED" }
                );
            }
            Err(e) => eprintln!("❌ Risk analysis failed: {}", e),
        }

        // Wait for messages to propagate
        sleep(Duration::from_millis(100)).await;

        println!("\n💰 Dispatching Trading Task");
        let trading_task = Task::new(
            TaskType::Trading,
            json!({
                "symbol": "BTC",
                "action": "buy",
                "amount": 2.5
            }),
        )
        .with_priority(Priority::Critical);

        match dispatcher.dispatch_task(trading_task).await {
            Ok(result) => {
                let default_json = serde_json::json!({});
                let data = result.data().unwrap_or(&default_json);
                let trade_id = data
                    .get("trade_id")
                    .and_then(|id| id.as_str())
                    .unwrap_or("unknown");
                println!("✅ Trade executed: {}", trade_id);
            }
            Err(e) => eprintln!("❌ Trade failed: {}", e),
        }

        // Wait for final message propagation
        sleep(Duration::from_millis(200)).await;
    }

    // Display final agent statuses
    println!("\n📊 Final Agent Status:");
    let agents = registry.list_agents().await?;
    for agent in agents {
        let status = agent.status();
        println!(
            "  {} - {} (Load: {:.2})",
            status.agent_id,
            format!("{:?}", status.status).to_uppercase(),
            status.load
        );
    }

    println!("\n🎉 Basic dispatcher example with mock LLM integration completed successfully!");
    println!("This demonstrated:");
    println!("  ✅ Multi-agent task dispatching with capability-based routing");
    println!("  ✅ Inter-agent communication via messages");
    println!("  ✅ Simplified implementation without external dependencies");
    println!("  ✅ Mock LLM-powered agent intelligence");
    Ok(())
}
