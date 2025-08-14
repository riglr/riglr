//! Simple Agent Dispatcher Example
//!
//! A minimal working example that demonstrates:
//! - Creating agents with different capabilities
//! - Registering agents in a registry
//! - Dispatching tasks to appropriate agents
//! - Basic error handling
//!
//! Run with: cargo run --example simple_dispatcher

use riglr_agents::{
    Agent, AgentDispatcher, AgentRegistry, LocalAgentRegistry,
    Task, TaskResult, TaskType, Priority, AgentId
};
use riglr_core::{
    SignerContext, 
    signer::{LocalSolanaSigner, TransactionSigner},
    config::SolanaNetworkConfig
};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use serde_json::json;

/// A simple trading agent
#[derive(Clone)]
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
        
        let symbol = task.parameters.get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("BTC");
            
        let action = task.parameters.get("action")
            .and_then(|s| s.as_str())
            .unwrap_or("buy");
        
        // Simulate trade execution
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(TaskResult::success(
            json!({
                "trade_id": uuid::Uuid::new_v4().to_string(),
                "symbol": symbol,
                "action": action,
                "status": "completed",
                "trader": self.id.as_str()
            }),
            None,
            Duration::from_millis(100)
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["trading".to_string()]
    }
}

/// A simple research agent
#[derive(Clone)]
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
        
        let symbol = task.parameters.get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("BTC");
        
        // Simulate research work
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        Ok(TaskResult::success(
            json!({
                "symbol": symbol,
                "analysis": {
                    "trend": "bullish",
                    "confidence": 0.85
                },
                "recommendation": "BUY",
                "analyst": self.id.as_str()
            }),
            None,
            Duration::from_millis(50)
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["research".to_string()]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting Simple Agent Dispatcher Example");
    
    // Create agents with different capabilities
    let trading_agent = Arc::new(TradingAgent::new("trader-001"));
    let research_agent = Arc::new(ResearchAgent::new("researcher-001"));
    
    // Create agent registry and register agents
    let registry = Arc::new(LocalAgentRegistry::new());
    registry.register_agent(trading_agent).await?;
    registry.register_agent(research_agent).await?;
    
    println!("✅ Registered {} agents", registry.agent_count().await?);
    
    // Create dispatcher
    let dispatcher = AgentDispatcher::new(registry.clone());
    
    // Setup a Solana signer for blockchain operations
    let solana_config = SolanaNetworkConfig {
        name: "devnet".to_string(),
        rpc_url: "https://api.devnet.solana.com".to_string(),
        explorer_url: Some("https://explorer.solana.com".to_string()),
    };
    // Create a test keypair for the example
    let test_private_key = "5J1F7GHadZqcsLLuK1EeZ2UfzqyFJyW2EHGJhfDmzwCm2c1xJGjzq3K9t5Z4y7M8xR5B4A9Sd3FfhYfJgPuWmN9X"; // Test key - don't use in production
    let signer = Arc::new(LocalSolanaSigner::new(
        test_private_key.to_string(),
        solana_config
    ).unwrap_or_else(|_| {
        // Fallback to using a generated keypair if the test key fails
        use solana_sdk::signer::{keypair::Keypair, Signer};
        let keypair = Keypair::new();
        let private_key_bytes = keypair.to_bytes();
        let private_key_base58 = bs58::encode(private_key_bytes).into_string();
        LocalSolanaSigner::new(private_key_base58, solana_config.clone()).expect("Failed to create signer")
    })) as Arc<dyn TransactionSigner>;
    
    // Execute within signer context
    SignerContext::with_signer(signer, async {
        println!("\n🔬 Dispatching Research Task");
        let research_task = Task::new(
            TaskType::Research,
            json!({
                "symbol": "BTC",
                "analysis_type": "technical"
            })
        ).with_priority(Priority::High);
        
        match dispatcher.dispatch_task(research_task).await {
            Ok(result) => {
                if let Some(data) = result.data() {
                    let recommendation = data.get("recommendation")
                        .and_then(|r| r.as_str())
                        .unwrap_or("HOLD");
                    println!("✅ Research completed: {}", recommendation);
                } else {
                    println!("✅ Research completed successfully");
                }
            }
            Err(e) => eprintln!("❌ Research failed: {}", e),
        }
        
        println!("\n💰 Dispatching Trading Task");
        let trading_task = Task::new(
            TaskType::Trading,
            json!({
                "symbol": "BTC",
                "action": "buy",
                "amount": 1.0
            })
        ).with_priority(Priority::High);
        
        match dispatcher.dispatch_task(trading_task).await {
            Ok(result) => {
                if let Some(data) = result.data() {
                    let trade_id = data.get("trade_id")
                        .and_then(|id| id.as_str())
                        .unwrap_or("unknown");
                    println!("✅ Trade executed: {}", trade_id);
                } else {
                    println!("✅ Trade completed successfully");
                }
            }
            Err(e) => eprintln!("❌ Trade failed: {}", e),
        }
        
        Ok::<(), riglr_core::signer::SignerError>(())
    }).await?;
    
    // Display final agent statuses
    println!("\n📊 Agent Status:");
    let agents = registry.list_agents().await?;
    for agent in agents {
        let status = agent.status();
        println!("  {} - {} capabilities: {:?}", 
            status.agent_id,
            status.capabilities.len(),
            status.capabilities.iter().map(|c| &c.name).collect::<Vec<_>>()
        );
    }
    
    println!("\n🎉 Simple dispatcher example completed successfully!");
    Ok(())
}