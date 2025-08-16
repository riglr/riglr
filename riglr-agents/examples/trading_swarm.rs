//! Trading Swarm Example
//!
//! This example demonstrates a sophisticated multi-agent trading system with:
//! - Research agents that analyze markets
//! - Risk analysis agents that assess and approve trades
//! - Execution agents that perform blockchain operations
//! - Portfolio management agents that monitor positions
//! - Coordinated workflows between agents
//!
//! The example shows how agents can work together in a swarm-like fashion,
//! with each agent having specialized capabilities but working towards
//! common trading objectives.
//!
//! Run with: cargo run --example trading_swarm

use riglr_agents::{
    Agent, AgentDispatcher, AgentRegistry, LocalAgentRegistry,
    Task, TaskResult, TaskType, Priority, AgentId, AgentMessage,
    DispatchConfig, RoutingStrategy, ChannelCommunication, AgentCommunication
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use serde_json::json;
use tokio::time::sleep;

/// Shared portfolio state that agents can read and update
#[derive(Debug, Clone)]
struct PortfolioState {
    positions: HashMap<String, f64>,
    available_capital: f64,
    total_value: f64,
    risk_exposure: f64,
}

impl PortfolioState {
    fn new(initial_capital: f64) -> Self {
        Self {
            positions: HashMap::new(),
            available_capital: initial_capital,
            total_value: initial_capital,
            risk_exposure: 0.0,
        }
    }
}

/// Market research agent with advanced analysis capabilities
#[derive(Clone)]
struct MarketResearchAgent {
    id: AgentId,
    communication: Arc<ChannelCommunication>,
    portfolio: Arc<Mutex<PortfolioState>>,
}

impl MarketResearchAgent {
    fn new(id: &str, communication: Arc<ChannelCommunication>, portfolio: Arc<Mutex<PortfolioState>>) -> Self {
        Self {
            id: AgentId::new(id),
            communication,
            portfolio,
        }
    }

    async fn perform_technical_analysis(&self, symbol: &str) -> serde_json::Value {
        // Simulate complex technical analysis
        sleep(Duration::from_millis(300)).await;
        
        // Check current portfolio exposure to this symbol
        let portfolio = self.portfolio.lock().unwrap();
        let current_position = portfolio.positions.get(symbol).copied().unwrap_or(0.0);
        let position_value = current_position * 50000.0; // Simulate price
        let exposure_ratio = if portfolio.total_value > 0.0 {
            position_value / portfolio.total_value
        } else {
            0.0
        };
        drop(portfolio); // Release lock early
        
        let indicators = json!({
            "rsi": 65.4,
            "macd": "bullish_crossover",
            "bollinger_bands": "upper_band_touch",
            "volume_profile": "high_volume_node",
            "support_levels": [48500, 47200, 46000],
            "resistance_levels": [52000, 53500, 55000],
            "current_exposure": exposure_ratio,
            "position_status": if exposure_ratio > 0.15 { "overweight" } else if exposure_ratio > 0.05 { "balanced" } else { "underweight" }
        });
        
        json!({
            "symbol": symbol,
            "analysis_type": "technical",
            "indicators": indicators,
            "trend": "bullish",
            "strength": 7.5,
            "timeframe": "4h",
            "confidence": 0.82,
            "analyst": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        })
    }

    async fn perform_sentiment_analysis(&self, symbol: &str) -> serde_json::Value {
        // Simulate sentiment analysis from various sources
        sleep(Duration::from_millis(200)).await;
        
        json!({
            "symbol": symbol,
            "analysis_type": "sentiment",
            "social_sentiment": 0.72,
            "news_sentiment": 0.68,
            "whale_activity": "accumulation",
            "fear_greed_index": 58,
            "market_sentiment": "cautiously_optimistic",
            "confidence": 0.75,
            "analyst": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        })
    }
}

#[async_trait]
impl Agent for MarketResearchAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("🔬 Research Agent {} analyzing: {}", self.id, 
            task.parameters.get("symbol").and_then(|s| s.as_str()).unwrap_or("N/A"));
        
        let symbol = task.parameters.get("symbol")
            .and_then(|s| s.as_str())
            .ok_or_else(|| riglr_agents::AgentError::task_execution("Missing symbol".to_string()))?;
        
        let _analysis_type = task.parameters.get("analysis_type")
            .and_then(|t| t.as_str())
            .unwrap_or("comprehensive");
        
        let (technical, sentiment) = tokio::join!(
            self.perform_technical_analysis(symbol),
            self.perform_sentiment_analysis(symbol)
        );
        
        // Calculate portfolio-aware recommendations
        let (current_exposure, recommended_position_size, action) = {
            let portfolio = self.portfolio.lock().unwrap();
            let current_position = portfolio.positions.get(symbol).copied().unwrap_or(0.0);
            let available_capital = portfolio.available_capital;
            let current_exposure = if portfolio.total_value > 0.0 {
                (current_position * 50000.0) / portfolio.total_value
            } else {
                0.0
            };
            
            // Adjust position size based on current exposure and available capital
            let recommended_position_size = if current_exposure > 0.10 {
                0.05 // Reduce position size if already heavily exposed
            } else if available_capital < 10000.0 {
                0.02 // Conservative sizing with low capital
            } else {
                0.15 // Standard position size
            };
            
            let action = if current_exposure > 0.20 {
                "hold" // Don't increase position if over-exposed
            } else {
                "buy"
            };
            
            (current_exposure, recommended_position_size, action)
        }; // MutexGuard is dropped here
        
        let comprehensive_analysis = json!({
            "symbol": symbol,
            "technical": technical,
            "sentiment": sentiment,
            "recommendation": {
                "action": action,
                "confidence": 0.78,
                "target_price": 55000,
                "stop_loss": 47000,
                "position_size_percent": recommended_position_size,
                "current_exposure": current_exposure,
                "reasoning": if current_exposure > 0.20 {
                    "Position already overweight, recommend hold"
                } else if recommended_position_size < 0.10 {
                    "Limited capital, conservative position sizing"
                } else {
                    "Standard position sizing recommended"
                }
            },
            "research_quality": "high",
            "analysis_timestamp": chrono::Utc::now().timestamp()
        });
        
        // Share analysis with other agents
        let message = AgentMessage::new(
            self.id.clone(),
            None,
            "comprehensive_analysis".to_string(),
            comprehensive_analysis.clone()
        );
        
        self.communication.broadcast_message(message).await
            .map_err(|e| riglr_agents::AgentError::communication(e.to_string()))?;
        
        Ok(TaskResult::success(
            comprehensive_analysis,
            None, // No transaction hash for research task
            Duration::from_millis(500)
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "research".to_string(),
            "technical_analysis".to_string(),
            "sentiment_analysis".to_string(),
            "market_intelligence".to_string(),
        ]
    }
}

/// Risk management agent with sophisticated risk models
#[derive(Clone)]
struct RiskManagementAgent {
    id: AgentId,
    communication: Arc<ChannelCommunication>,
    portfolio: Arc<Mutex<PortfolioState>>,
    max_position_percent: f64,
    max_portfolio_risk: f64,
}

impl RiskManagementAgent {
    fn new(
        id: &str, 
        communication: Arc<ChannelCommunication>, 
        portfolio: Arc<Mutex<PortfolioState>>
    ) -> Self {
        Self {
            id: AgentId::new(id),
            communication,
            portfolio,
            max_position_percent: 0.20, // Max 20% per position
            max_portfolio_risk: 0.60,   // Max 60% portfolio risk
        }
    }

    async fn calculate_position_risk(&self, _symbol: &str, amount: f64, price: f64) -> f64 {
        // Simulate sophisticated risk calculation
        sleep(Duration::from_millis(100)).await;
        
        let position_value = amount * price;
        let portfolio = self.portfolio.lock().unwrap();
        
        // Calculate risk based on position size, volatility, correlation, etc.
        let position_risk = position_value / portfolio.total_value;
        let volatility_multiplier = 1.2; // Simulate volatility factor
        
        position_risk * volatility_multiplier
    }

    async fn validate_trade(&self, analysis: &serde_json::Value, trade_params: &serde_json::Value) -> serde_json::Value {
        let symbol = trade_params.get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("UNKNOWN");
        
        let amount = trade_params.get("amount")
            .and_then(|a| a.as_f64())
            .unwrap_or(0.0);
        
        let price = analysis.get("technical")
            .and_then(|t| t.get("current_price"))
            .and_then(|p| p.as_f64())
            .unwrap_or(50000.0);
        
        let position_risk = self.calculate_position_risk(symbol, amount, price).await;
        let portfolio = self.portfolio.lock().unwrap();
        let new_total_risk = portfolio.risk_exposure + position_risk;
        
        let approved = position_risk <= self.max_position_percent && 
                      new_total_risk <= self.max_portfolio_risk;
        
        let risk_assessment = json!({
            "symbol": symbol,
            "position_risk": position_risk,
            "portfolio_risk": new_total_risk,
            "max_position_risk": self.max_position_percent,
            "max_portfolio_risk": self.max_portfolio_risk,
            "approved": approved,
            "risk_level": if position_risk > 0.15 { "high" } else if position_risk > 0.08 { "medium" } else { "low" },
            "recommendations": if approved {
                vec!["APPROVE", "MONITOR_CLOSELY"]
            } else {
                vec!["REJECT", "REDUCE_POSITION_SIZE"]
            },
            "assessor": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        risk_assessment
    }
}

#[async_trait]
impl Agent for RiskManagementAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("⚖️ Risk Agent {} assessing trade risk", self.id);
        
        let analysis = task.parameters.get("analysis")
            .ok_or_else(|| riglr_agents::AgentError::task_execution("Missing analysis data".to_string()))?;
        
        let trade_params = task.parameters.get("trade_params")
            .ok_or_else(|| riglr_agents::AgentError::task_execution("Missing trade parameters".to_string()))?;
        
        let risk_assessment = self.validate_trade(analysis, trade_params).await;
        
        // Notify other agents of risk decision
        let message = AgentMessage::new(
            self.id.clone(),
            None,
            "risk_decision".to_string(),
            risk_assessment.clone()
        );
        
        self.communication.broadcast_message(message).await
            .map_err(|e| riglr_agents::AgentError::communication(e.to_string()))?;
        
        Ok(TaskResult::success(
            risk_assessment,
            None, // No transaction hash for risk assessment
            Duration::from_millis(200)
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "risk_analysis".to_string(),
            "position_sizing".to_string(),
            "portfolio_risk".to_string(),
            "compliance_check".to_string(),
        ]
    }

    async fn handle_message(&self, message: AgentMessage) -> riglr_agents::Result<()> {
        if message.message_type.as_str() == "trade_executed" {
            println!("⚖️ Risk Agent {} updating portfolio after trade", self.id);
            // Update portfolio state after successful trade
        }
        Ok(())
    }
}

/// Execution agent that performs actual blockchain trades
#[derive(Clone)]
struct TradeExecutionAgent {
    id: AgentId,
    communication: Arc<ChannelCommunication>,
    portfolio: Arc<Mutex<PortfolioState>>,
}

impl TradeExecutionAgent {
    fn new(
        id: &str, 
        communication: Arc<ChannelCommunication>, 
        portfolio: Arc<Mutex<PortfolioState>>
    ) -> Self {
        Self {
            id: AgentId::new(id),
            communication,
            portfolio,
        }
    }

    async fn execute_blockchain_trade(&self, trade_params: &serde_json::Value) -> riglr_agents::Result<serde_json::Value> {
        // In a real implementation, this would use SignerContext::current()
        // to access blockchain signers and execute actual trades using
        // riglr-solana-tools, riglr-evm-tools, etc.
        
        let symbol = trade_params.get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("BTC");
        
        let action = trade_params.get("action")
            .and_then(|a| a.as_str())
            .unwrap_or("buy");
        
        let amount = trade_params.get("amount")
            .and_then(|a| a.as_f64())
            .unwrap_or(0.0);
        
        // Simulate blockchain interaction
        println!("🔗 Executing {} {} {} on blockchain...", action, amount, symbol);
        sleep(Duration::from_millis(500)).await;
        
        // Update portfolio state
        {
            let mut portfolio = self.portfolio.lock().unwrap();
            if action == "buy" {
                *portfolio.positions.entry(symbol.to_string()).or_insert(0.0) += amount;
                portfolio.available_capital -= amount * 50000.0; // Simulate price
            }
        }
        
        let trade_result = json!({
            "trade_id": uuid::Uuid::new_v4().to_string(),
            "symbol": symbol,
            "action": action,
            "amount": amount,
            "price": 50000.0,
            "total_value": amount * 50000.0,
            "status": "executed",
            "blockchain": "solana", // or "ethereum" depending on the trade
            "transaction_hash": format!("0x{}", uuid::Uuid::new_v4().to_string().replace("-", "")),
            "gas_used": 150000,
            "executor": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        Ok(trade_result)
    }
}

#[async_trait]
impl Agent for TradeExecutionAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("⚡ Execution Agent {} processing trade order", self.id);
        
        let risk_approval = task.parameters.get("risk_approval")
            .and_then(|r| r.get("approved"))
            .and_then(|a| a.as_bool())
            .unwrap_or(false);
        
        if !risk_approval {
            return Ok(TaskResult::failure(
                "Trade rejected by risk management".to_string(),
                false, // Not retriable
                Duration::from_millis(10)
            ));
        }
        
        let trade_params = task.parameters.get("trade_params")
            .ok_or_else(|| riglr_agents::AgentError::task_execution("Missing trade parameters".to_string()))?;
        
        let trade_result = self.execute_blockchain_trade(trade_params).await?;
        
        // Notify all agents of successful execution
        let message = AgentMessage::new(
            self.id.clone(),
            None,
            "trade_executed".to_string(),
            trade_result.clone()
        );
        
        self.communication.broadcast_message(message).await
            .map_err(|e| riglr_agents::AgentError::communication(e.to_string()))?;
        
        let tx_hash = trade_result.get("transaction_hash").and_then(|h| h.as_str()).map(|s| s.to_string());
        
        Ok(TaskResult::success(
            trade_result,
            tx_hash,
            Duration::from_millis(500)
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "trading".to_string(),
            "blockchain_execution".to_string(),
            "order_management".to_string(),
            "transaction_processing".to_string(),
        ]
    }
}

/// Portfolio monitoring agent that tracks overall performance
#[derive(Clone)]
struct PortfolioMonitorAgent {
    id: AgentId,
    communication: Arc<ChannelCommunication>,
    portfolio: Arc<Mutex<PortfolioState>>,
}

impl PortfolioMonitorAgent {
    fn new(
        id: &str, 
        communication: Arc<ChannelCommunication>, 
        portfolio: Arc<Mutex<PortfolioState>>
    ) -> Self {
        Self {
            id: AgentId::new(id),
            communication,
            portfolio,
        }
    }

    async fn generate_portfolio_report(&self) -> serde_json::Value {
        let portfolio = self.portfolio.lock().unwrap();
        
        let total_positions: f64 = portfolio.positions.values().sum();
        let position_diversity = portfolio.positions.len();
        let utilization_rate = (portfolio.total_value - portfolio.available_capital) / portfolio.total_value;
        
        json!({
            "total_value": portfolio.total_value,
            "available_capital": portfolio.available_capital,
            "total_positions": total_positions,
            "position_count": position_diversity,
            "utilization_rate": utilization_rate,
            "risk_exposure": portfolio.risk_exposure,
            "positions": portfolio.positions,
            "performance": {
                "daily_pnl": 1250.50, // Simulated
                "total_return": 0.125,  // 12.5% return
                "sharpe_ratio": 1.85,
                "max_drawdown": -0.08
            },
            "alerts": if portfolio.risk_exposure > 0.8 {
                vec!["HIGH_RISK_EXPOSURE"]
            } else {
                vec![]
            },
            "monitor": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        })
    }
}

#[async_trait]
impl Agent for PortfolioMonitorAgent {
    async fn execute_task(&self, _task: Task) -> riglr_agents::Result<TaskResult> {
        println!("📊 Portfolio Monitor {} generating report", self.id);
        
        let report = self.generate_portfolio_report().await;
        
        // Share portfolio status with other agents
        let message = AgentMessage::new(
            self.id.clone(),
            None,
            "portfolio_update".to_string(),
            report.clone()
        );
        
        self.communication.broadcast_message(message).await
            .map_err(|e| riglr_agents::AgentError::communication(e.to_string()))?;
        
        Ok(TaskResult::success(
            report,
            None, // No transaction hash for portfolio report
            Duration::from_millis(100)
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "portfolio".to_string(),
            "monitoring".to_string(),
            "performance_tracking".to_string(),
            "reporting".to_string(),
        ]
    }

    async fn handle_message(&self, message: AgentMessage) -> riglr_agents::Result<()> {
        if message.message_type.as_str() == "trade_executed" {
            println!("📊 Portfolio Monitor {} updating positions", self.id);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting Trading Swarm Example");
    println!("💼 Initializing portfolio with $100,000 capital");
    
    // Initialize shared portfolio state
    let portfolio = Arc::new(Mutex::new(PortfolioState::new(100000.0)));
    
    // Initialize communication system
    let communication = Arc::new(ChannelCommunication::new());
    
    // Create specialized trading agents
    let research_agent = Arc::new(MarketResearchAgent::new("research-alpha", communication.clone(), portfolio.clone()));
    let risk_agent = Arc::new(RiskManagementAgent::new("risk-guardian", communication.clone(), portfolio.clone()));
    let execution_agent = Arc::new(TradeExecutionAgent::new("executor-prime", communication.clone(), portfolio.clone()));
    let portfolio_monitor = Arc::new(PortfolioMonitorAgent::new("monitor-omega", communication.clone(), portfolio.clone()));
    
    // Create agent registry and register all agents
    let registry = Arc::new(LocalAgentRegistry::new());
    registry.register_agent(research_agent.clone()).await?;
    registry.register_agent(risk_agent.clone()).await?;
    registry.register_agent(execution_agent.clone()).await?;
    registry.register_agent(portfolio_monitor.clone()).await?;
    
    println!("✅ Registered {} specialized agents in trading swarm", registry.agent_count().await?);
    
    // Create dispatcher optimized for trading workflows
    let dispatch_config = DispatchConfig {
        routing_strategy: RoutingStrategy::Capability,
        max_retries: 2,
        default_task_timeout: Duration::from_secs(45),
        enable_load_balancing: false, // Sequential execution for trading workflow
        retry_delay: Duration::from_secs(1),
        max_concurrent_tasks_per_agent: 10,
    };
    
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), dispatch_config));
    
    // Execute coordinated trading workflow
    {
        println!("\n🔄 Starting Coordinated Trading Workflow");
        
        // Step 1: Market Research
        println!("\n1️⃣ Phase 1: Market Research & Analysis");
        let research_task = Task::new(
            TaskType::Research,
            json!({
                "symbol": "BTC",
                "analysis_type": "comprehensive",
                "urgency": "high"
            })
        ).with_priority(Priority::High);
        
        let research_result = dispatcher.dispatch_task(research_task).await?;
        println!("✅ Research completed with confidence: {}", 
            research_result.data().unwrap().get("recommendation")
                .and_then(|r| r.get("confidence"))
                .and_then(|c| c.as_f64())
                .unwrap_or(0.0));
        
        // Wait for message propagation
        sleep(Duration::from_millis(200)).await;
        
        // Step 2: Risk Assessment based on research
        println!("\n2️⃣ Phase 2: Risk Assessment");
        let risk_task = Task::new(
            TaskType::RiskAnalysis,
            json!({
                "analysis": research_result.data().unwrap(),
                "trade_params": {
                    "symbol": "BTC",
                    "action": "buy",
                    "amount": 2.5
                }
            })
        ).with_priority(Priority::High);
        
        let risk_result = dispatcher.dispatch_task(risk_task).await?;
        let risk_approved = risk_result.data().unwrap().get("approved")
            .and_then(|a| a.as_bool())
            .unwrap_or(false);
        
        println!("✅ Risk assessment: {} (Risk Level: {})", 
            if risk_approved { "APPROVED" } else { "REJECTED" },
            risk_result.data().unwrap().get("risk_level")
                .and_then(|l| l.as_str())
                .unwrap_or("unknown")
        );
        
        sleep(Duration::from_millis(200)).await;
        
        // Step 3: Trade Execution (if approved)
        if risk_approved {
            println!("\n3️⃣ Phase 3: Trade Execution");
            let execution_task = Task::new(
                TaskType::Trading,
                json!({
                    "risk_approval": risk_result.data().unwrap(),
                    "trade_params": {
                        "symbol": "BTC",
                        "action": "buy",
                        "amount": 2.5
                    },
                    "research_data": research_result.data().unwrap()
                })
            ).with_priority(Priority::Critical);
            
            let execution_result = dispatcher.dispatch_task(execution_task).await?;
            println!("✅ Trade executed: {}", 
                execution_result.data().unwrap().get("transaction_hash")
                    .and_then(|h| h.as_str())
                    .unwrap_or("unknown"));
        } else {
            println!("\n3️⃣ Phase 3: Trade Execution - SKIPPED (Risk Rejected)");
        }
        
        sleep(Duration::from_millis(300)).await;
        
        // Step 4: Portfolio Update
        println!("\n4️⃣ Phase 4: Portfolio Status Update");
        let monitor_task = Task::new(
            TaskType::Portfolio,
            json!({
                "report_type": "comprehensive",
                "include_performance": true
            })
        ).with_priority(Priority::Normal);
        
        let monitor_result = dispatcher.dispatch_task(monitor_task).await?;
        println!("✅ Portfolio updated - Total Value: ${:.2}", 
            monitor_result.data().unwrap().get("total_value")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0));
        
        sleep(Duration::from_millis(200)).await;
        
    }
    
    // Final status report
    println!("\n📊 Trading Swarm Final Status:");
    let agents = registry.list_agents().await?;
    for agent in agents {
        let status = agent.status();
        println!("  {} - {} capabilities: {}", 
            status.agent_id,
            status.capabilities.len(),
            status.capabilities.iter()
                .map(|c| c.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    
    // Show final portfolio state
    {
        let portfolio = portfolio.lock().unwrap();
        println!("\n💼 Final Portfolio State:");
        println!("  Total Value: ${:.2}", portfolio.total_value);
        println!("  Available Capital: ${:.2}", portfolio.available_capital);
        println!("  Active Positions: {}", portfolio.positions.len());
        for (symbol, amount) in &portfolio.positions {
            println!("    {}: {:.4}", symbol, amount);
        }
    }
    
    println!("\n🎉 Trading Swarm example completed successfully!");
    println!("The agents demonstrated coordinated workflow with:");
    println!("  - Comprehensive market research");
    println!("  - Risk assessment and approval");
    println!("  - Blockchain trade execution");
    println!("  - Portfolio monitoring and updates");
    
    Ok(())
}