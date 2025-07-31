//! Multi-agent coordination demos
//!
//! This module demonstrates the riglr-agents framework through various scenarios:
//! - Basic multi-agent coordination
//! - Real-world trading workflows
//! - Risk management systems
//! - Cross-chain agent coordination

use riglr_config::Config;
use std::sync::Arc;
use anyhow::Result;
#[allow(unused_imports)]
use riglr_core::SignerContext;
#[allow(unused_imports)]
use riglr_agents::AgentCommunication;

pub async fn run_demo(config: Arc<Config>, scenario: String) -> Result<()> {
    println!("🤖 Starting Multi-Agent Coordination Demo");
    println!("📋 Scenario: {}", scenario);
    
    match scenario.as_str() {
        "trading" => run_trading_coordination_demo(config).await,
        "risk" => run_risk_management_demo(config).await,
        "basic" => run_basic_coordination_demo(config).await,
        _ => {
            println!("❌ Unknown scenario: {}", scenario);
            println!("Available scenarios: trading, risk, basic");
            Err(anyhow::anyhow!("Unknown scenario: {}", scenario))
        }
    }
}

async fn run_trading_coordination_demo(_config: Arc<Config>) -> Result<()> {
    println!("\n🔄 Running Real-World Trading Coordination Demo");
    println!("This demo shows agents working together for actual blockchain operations");
    
    // Setup signer context for real blockchain operations
    // TODO: Create proper signer factory
    // let signer_factory = MemorySignerFactory::new();
    
    // TODO: Re-enable when proper signer factory is available
    // SignerContext::new(&signer_factory).execute(async {
    //     // Run the comprehensive trading coordination example
    //     trading_coordination::demonstrate_trading_coordination(config).await?;
    //     
    //     Ok::<(), riglr_core::ToolError>(())
    // }).await?;
    
    println!("⚠️  Agent demo temporarily disabled - signer factory needs implementation");
    
    Ok(())
}

async fn run_risk_management_demo(_config: Arc<Config>) -> Result<()> {
    println!("\n⚖️ Running Risk Management System Demo");
    println!("This demo shows coordinated risk assessment across multiple agents");
    
    // Import and run the risk management example
    // Note: In a real implementation, you would import from the examples
    // For now, we'll show a simplified version
    
    use riglr_agents::{
        Agent, AgentDispatcher, AgentRegistry, LocalAgentRegistry,
        Task, TaskResult, TaskType, Priority, AgentId, ChannelCommunication,
        DispatchConfig, RoutingStrategy
    };
    #[allow(unused_imports)]
    use riglr_agents::AgentCommunication;
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::time::Duration;
    use serde_json::json;

    #[derive(Clone)]
    struct SimpleRiskAgent {
        id: AgentId,
    }

    #[async_trait]
    impl Agent for SimpleRiskAgent {
        async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
            println!("⚖️ Risk agent {} assessing trade risk", self.id);
            
            let amount = task.parameters.get("amount")
                .and_then(|a| a.as_f64())
                .unwrap_or(0.0);
            
            // Simple risk assessment
            let risk_score = amount / 10000.0; // Simple calculation
            let approved = risk_score < 0.5;
            
            let result = json!({
                "approved": approved,
                "risk_score": risk_score,
                "recommendation": if approved { "APPROVE" } else { "REJECT" }
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

    // TODO: Create proper signer factory
    // let signer_factory = MemorySignerFactory::new();
    
    // TODO: Re-enable when proper signer factory is available
    // SignerContext::new(&signer_factory).execute(async {
        let _communication = Arc::new(ChannelCommunication::new());
        let risk_agent = Arc::new(SimpleRiskAgent {
            id: AgentId::new("risk-demo-1")
        });
        
        let registry = Arc::new(LocalAgentRegistry::new());
        registry.register_agent(risk_agent).await?;
        
        let dispatcher = AgentDispatcher::with_config(registry, DispatchConfig {
            routing_strategy: RoutingStrategy::Capability,
            max_retries: 2,
            default_task_timeout: Duration::from_secs(30),
            retry_delay: Duration::from_secs(1),
            max_concurrent_tasks_per_agent: 3,
            enable_load_balancing: false,
        });
        
        println!("📊 Testing risk assessment for different trade sizes");
        
        for (trade_size, expected) in [(1000.0, "APPROVE"), (8000.0, "REJECT")] {
            let task = Task::new(
                TaskType::RiskAnalysis,
                json!({"amount": trade_size, "symbol": "BTC"})
            ).with_priority(Priority::High);
            
            let result = dispatcher.dispatch_task(task).await?;
            let default_json = serde_json::json!({});
            let decision = result.data().unwrap_or(&default_json).get("recommendation")
                .and_then(|r| r.as_str())
                .unwrap_or("UNKNOWN");
            
            println!("  💰 Trade size: ${:.0} -> {}", trade_size, decision);
            assert_eq!(decision, expected, "Risk assessment mismatch for trade size {}", trade_size);
        }
        
        println!("✅ Risk management demo completed successfully");
        
        // Ok::<(), riglr_core::ToolError>(())
    // }).await?;
    
    println!("⚠️  Function temporarily disabled - needs signer factory implementation");
    
    Ok(())
}

async fn run_basic_coordination_demo(_config: Arc<Config>) -> Result<()> {
    println!("\n🔄 Running Basic Agent Coordination Demo");
    println!("This demo shows fundamental multi-agent communication patterns");
    
    use riglr_agents::{
        Agent, AgentDispatcher, AgentRegistry, LocalAgentRegistry,
        Task, TaskResult, TaskType, Priority, AgentId, AgentMessage, ChannelCommunication,
        DispatchConfig, RoutingStrategy
    };
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::time::Duration;
    use serde_json::json;
    use tokio::time::sleep;

    #[derive(Clone)]
    struct CoordinatorAgent {
        id: AgentId,
        communication: Arc<ChannelCommunication>,
    }

    #[derive(Clone)]
    struct WorkerAgent {
        id: AgentId,
        _communication: Arc<ChannelCommunication>,
    }

    #[async_trait]
    impl Agent for CoordinatorAgent {
        async fn execute_task(&self, _task: Task) -> riglr_agents::Result<TaskResult> {
            println!("👑 Coordinator {} orchestrating workflow", self.id);
            
            let workflow_steps = [
                "data_collection",
                "analysis", 
                "decision_making",
                "execution"
            ];
            
            for (i, step) in workflow_steps.iter().enumerate() {
                println!("  📋 Step {}: {}", i + 1, step);
                
                // Send message to workers
                let message = AgentMessage::new(
                    self.id.clone(),
                    None, // Broadcast
                    "workflow_step".to_string(),
                    json!({"step": step, "sequence": i + 1})
                );
                
                self.communication.broadcast_message(message).await
                    .map_err(|e| riglr_agents::AgentError::generic(e.to_string()))?;
                
                sleep(Duration::from_millis(100)).await;
            }
            
            Ok(TaskResult::success(
                json!({"workflow": "completed", "steps": workflow_steps.len()}),
                None,
                Duration::from_millis(400)
            ))
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<String> {
            vec!["portfolio".to_string(), "coordination".to_string()]
        }
    }

    #[async_trait]
    impl Agent for WorkerAgent {
        async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
            println!("🔧 Worker {} processing task", self.id);
            
            let work_type = task.parameters.get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("general");
            
            // Simulate work
            sleep(Duration::from_millis(50)).await;
            
            Ok(TaskResult::success(
                json!({
                    "worker": self.id.as_str(),
                    "work_type": work_type,
                    "status": "completed"
                }),
                None,
                Duration::from_millis(50)
            ))
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<String> {
            vec!["research".to_string(), "monitoring".to_string()]
        }

        async fn handle_message(&self, message: AgentMessage) -> riglr_agents::Result<()> {
            if message.message_type == "workflow_step" {
                let step = message.payload.get("step")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                let sequence = message.payload.get("sequence")
                    .and_then(|s| s.as_u64())
                    .unwrap_or(0);
                    
                println!("    🔧 Worker {} handling step {} ({})", self.id, sequence, step);
            }
            Ok(())
        }
    }

    // TODO: Create proper signer factory
    // let signer_factory = MemorySignerFactory::new();
    
    // TODO: Re-enable when proper signer factory is available
    // SignerContext::new(&signer_factory).execute(async {
        let communication = Arc::new(ChannelCommunication::new());
        
        let coordinator = Arc::new(CoordinatorAgent {
            id: AgentId::new("coordinator-1"),
            communication: communication.clone(),
        });
        
        let worker1 = Arc::new(WorkerAgent {
            id: AgentId::new("worker-1"),
            _communication: communication.clone(),
        });
        
        let worker2 = Arc::new(WorkerAgent {
            id: AgentId::new("worker-2"),
            _communication: communication.clone(),
        });
        
        let registry = Arc::new(LocalAgentRegistry::new());
        registry.register_agent(coordinator).await?;
        registry.register_agent(worker1).await?;
        registry.register_agent(worker2).await?;
        
        let agent_count = registry.list_agents().await?.len();
        println!("✅ Registered {} agents for coordination demo", agent_count);
        
        let dispatcher = AgentDispatcher::with_config(registry, DispatchConfig {
            routing_strategy: RoutingStrategy::Capability,
            max_retries: 1,
            default_task_timeout: Duration::from_secs(10),
            retry_delay: Duration::from_secs(1),
            max_concurrent_tasks_per_agent: 2,
            enable_load_balancing: true,
        });
        
        // Test coordination workflow
        let coordination_task = Task::new(
            TaskType::Portfolio,
            json!({"workflow": "multi_agent_demo"})
        ).with_priority(Priority::High);
        
        let coord_result = dispatcher.dispatch_task(coordination_task).await?;
        println!("✅ Coordination completed: {} steps", 
            coord_result.data().unwrap_or(&serde_json::json!({})).get("steps").and_then(|s| s.as_u64()).unwrap_or(0));
        
        sleep(Duration::from_millis(200)).await;
        
        // Test worker tasks
        println!("\n🔧 Testing individual worker capabilities");
        
        let research_task = Task::new(
            TaskType::Research,
            json!({"type": "market_research"})
        );
        
        let monitor_task = Task::new(
            TaskType::Monitoring,
            json!({"type": "system_monitoring"})
        );
        
        let (research_result, monitor_result) = tokio::join!(
            dispatcher.dispatch_task(research_task),
            dispatcher.dispatch_task(monitor_task)
        );
        
        match (research_result, monitor_result) {
            (Ok(r), Ok(m)) => {
                println!("✅ Research completed by: {}", 
                    r.data().unwrap_or(&serde_json::json!({})).get("worker").and_then(|w| w.as_str()).unwrap_or("unknown"));
                println!("✅ Monitoring completed by: {}", 
                    m.data().unwrap_or(&serde_json::json!({})).get("worker").and_then(|w| w.as_str()).unwrap_or("unknown"));
            }
            (Err(e), _) | (_, Err(e)) => {
                return Err(anyhow::anyhow!("Task execution failed: {}", e));
            }
        }
        
        println!("\n🎉 Basic coordination demo completed successfully!");
        println!("Demonstrated:");
        println!("  ✅ Multi-agent task routing");
        println!("  ✅ Inter-agent communication");
        println!("  ✅ Workflow orchestration");
        println!("  ✅ Parallel task execution");
        
        // Ok::<(), riglr_core::ToolError>(())
    // }).await?;
    
    println!("⚠️  Function temporarily disabled - needs signer factory implementation");
    
    Ok(())
}