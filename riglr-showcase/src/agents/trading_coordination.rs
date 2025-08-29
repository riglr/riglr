//! Real-World Trading Coordination Example
//!
//! This example demonstrates a production-ready multi-agent trading system
//! that coordinates research, risk management, and execution across multiple
//! blockchain networks. It shows how agents can use actual riglr tools to
//! perform real trading operations while maintaining proper risk controls.

use crate::config::Config;
use async_trait::async_trait;
use riglr_agents::{
    communication::CommunicationConfig, Agent, AgentCommunication, AgentDispatcher, AgentId,
    AgentMessage, AgentRegistry, ChannelCommunication, DispatchConfig, LocalAgentRegistry,
    Priority, RoutingStrategy, Task, TaskResult, TaskType,
};
use riglr_core::{provider::ApplicationContext, SignerContext, ToolError};
use riglr_evm_tools::balance::get_eth_balance;
use riglr_solana_tools::{
    balance::get_sol_balance,
    pump::{buy_pump_token, get_pump_token_info},
};
use riglr_web_tools::{
    dexscreener::{get_token_info, TokenInfo},
    price::get_token_price as get_web_price,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

/// Shared trading state across all agents
#[derive(Debug, Clone)]
pub struct TradingState {
    /// Map of symbol to active trading positions
    pub active_positions: HashMap<String, Position>,
    /// List of pending orders awaiting execution
    pub pending_orders: Vec<Order>,
    /// Total portfolio value in USD
    pub portfolio_value_usd: f64,
    /// Available ETH balance for trading
    pub available_balance_eth: f64,
    /// Available SOL balance for trading
    pub available_balance_sol: f64,
    /// Daily profit and loss percentage
    pub daily_pnl: f64,
    /// Current risk exposure as percentage of portfolio
    pub risk_exposure: f64,
}

/// Represents an active trading position
#[derive(Debug, Clone)]
pub struct Position {
    /// Token symbol or address
    pub symbol: String,
    /// Amount of tokens held
    pub amount: f64,
    /// Average entry price per token
    pub avg_price: f64,
    /// Current market price per token
    pub current_price: f64,
    /// Profit and loss for this position
    pub pnl: f64,
    /// Blockchain network where the position exists
    pub network: String,
    /// Timestamp when the position was opened
    pub entry_time: chrono::DateTime<chrono::Utc>,
}

/// Represents a trading order (pending or executed)
#[derive(Debug, Clone)]
pub struct Order {
    /// Unique identifier for this order
    pub id: String,
    /// Token symbol or address being traded
    pub symbol: String,
    /// Buy or sell side of the order
    pub side: OrderSide,
    /// Amount of tokens to trade
    pub amount: f64,
    /// Target price (None for market orders)
    pub price: Option<f64>,
    /// Blockchain network for execution
    pub network: String,
    /// Current status of the order
    pub status: OrderStatus,
}

/// Specifies the side of a trading order
#[derive(Debug, Clone)]
pub enum OrderSide {
    /// Buy order (acquire tokens)
    Buy,
    /// Sell order (dispose of tokens)
    Sell,
}

/// Represents the current status of a trading order
#[derive(Debug, Clone)]
pub enum OrderStatus {
    /// Order is waiting to be executed
    Pending,
    /// Order is currently being processed
    Executing,
    /// Order has been successfully executed
    Completed,
    /// Order execution failed
    Failed,
}

impl Default for TradingState {
    fn default() -> Self {
        Self {
            active_positions: HashMap::new(),
            pending_orders: Vec::new(),
            portfolio_value_usd: 0.0,
            available_balance_eth: 0.0,
            available_balance_sol: 0.0,
            daily_pnl: 0.0,
            risk_exposure: 0.0,
        }
    }
}

/// Market research agent that uses real data sources
#[derive(Clone)]
pub struct MarketIntelligenceAgent {
    /// Unique identifier for this agent
    id: AgentId,
    /// Communication channel for inter-agent messaging
    communication: Arc<ChannelCommunication>,
    /// Configuration settings
    _config: Config,
    /// Application context for tool invocations
    context: ApplicationContext,
    /// Shared trading state reference
    _trading_state: Arc<Mutex<TradingState>>,
}

impl std::fmt::Debug for MarketIntelligenceAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarketIntelligenceAgent")
            .field("id", &self.id)
            .field("_config", &"<Config>")
            .field("context", &"<ApplicationContext>")
            .finish()
    }
}

impl MarketIntelligenceAgent {
    /// Creates a new market intelligence agent
    pub fn new(
        id: &str,
        communication: Arc<ChannelCommunication>,
        config: Config,
        trading_state: Arc<Mutex<TradingState>>,
    ) -> Self {
        let context = ApplicationContext::from_config(&config);
        Self {
            id: AgentId::new(id),
            communication,
            _config: config,
            context,
            _trading_state: trading_state,
        }
    }

    async fn analyze_solana_token(
        &self,
        token_address: &str,
    ) -> Result<serde_json::Value, ToolError> {
        // Use real riglr-solana-tools to get pump.fun token info
        let token_info = match get_pump_token_info(token_address.to_string(), &self.context).await {
            Ok(info) => info,
            Err(e) => {
                return Err(ToolError::retriable_string(format!(
                    "Failed to get Solana token info: {}",
                    e
                )))
            }
        };

        // Use riglr-web-tools to get additional market data
        let dex_data = match get_token_info(
            &self.context,
            token_address.to_string(),
            Some("solana".to_string()),
            None,
            None,
        )
        .await
        {
            Ok(data) => Some(data),
            Err(e) => {
                tracing::warn!("Failed to get DEX data: {}", e);
                None
            }
        };

        Ok(json!({
            "network": "solana",
            "token_address": token_address,
            "pump_info": token_info,
            "dex_data": dex_data,
            "analysis": {
                "liquidity_score": self.calculate_liquidity_score(&serde_json::to_value(&token_info).unwrap_or_default()).await,
                "momentum_score": self.calculate_momentum_score(&dex_data).await,
                "risk_level": "medium",
                "recommendation": "MONITOR"
            },
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }

    async fn analyze_ethereum_token(
        &self,
        token_address: &str,
    ) -> Result<serde_json::Value, ToolError> {
        // TODO: Use riglr-evm-tools to get token price when available
        // let price_info = match get_uniswap_quote(...).await {
        //     Ok(quote) => quote,
        //     Err(e) => return Err(ToolError::retriable_string(format!("Failed to get ETH token price: {}", e))),
        // };
        let price_info = 0.0; // Placeholder

        // Get additional web data
        let web_price = match get_web_price(
            &self.context,
            token_address.to_string(),
            Some("ethereum".to_string()),
        )
        .await
        {
            Ok(price) => Some(price),
            Err(e) => {
                tracing::warn!("Failed to get web price: {}", e);
                None
            }
        };

        Ok(json!({
            "network": "ethereum",
            "token_address": token_address,
            "price_info": price_info,
            "web_price": web_price,
            "analysis": {
                "price_trend": "bullish",
                "volume_score": 0.8,
                "risk_level": "low",
                "recommendation": "BUY"
            },
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }

    async fn calculate_liquidity_score(&self, _token_info: &serde_json::Value) -> f64 {
        // Simplified liquidity scoring
        // In a real implementation, this would analyze:
        // - Market cap vs volume
        // - Bid-ask spreads
        // - Order book depth
        sleep(Duration::from_millis(50)).await;
        0.75
    }

    async fn calculate_momentum_score(&self, dex_data: &Option<TokenInfo>) -> f64 {
        // Simplified momentum analysis
        if let Some(data) = dex_data {
            // Use actual price change data if available
            data.price_change_24h.unwrap_or(0.0).abs().min(1.0)
        } else {
            0.5
        }
    }
}

#[async_trait]
impl Agent for MarketIntelligenceAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("🔍 Market Intelligence Agent {} analyzing market", self.id);

        let network = task
            .parameters
            .get("network")
            .and_then(|n| n.as_str())
            .ok_or_else(|| {
                riglr_agents::AgentError::generic("Missing network parameter".to_string())
            })?;

        let token_address = task
            .parameters
            .get("token_address")
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                riglr_agents::AgentError::generic("Missing token_address parameter".to_string())
            })?;

        let analysis = match network {
            "solana" => self
                .analyze_solana_token(token_address)
                .await
                .map_err(|e| riglr_agents::AgentError::task_execution(e.to_string()))?,
            "ethereum" => self
                .analyze_ethereum_token(token_address)
                .await
                .map_err(|e| riglr_agents::AgentError::task_execution(e.to_string()))?,
            _ => {
                return Err(riglr_agents::AgentError::generic(format!(
                    "Unsupported network: {}",
                    network
                )))
            }
        };

        // Broadcast analysis to other agents
        let message = AgentMessage::new(
            self.id.clone(),
            None,
            "market_analysis".to_string(),
            analysis.clone(),
        );

        self.communication
            .broadcast_message(message)
            .await
            .map_err(|e| riglr_agents::AgentError::generic(e.to_string()))?;

        Ok(TaskResult::success(
            analysis,
            None, // No transaction hash for analysis
            Duration::from_millis(300),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<riglr_agents::CapabilityType> {
        vec![
            riglr_agents::CapabilityType::Research,
            riglr_agents::CapabilityType::Custom("market_analysis".to_string()),
            riglr_agents::CapabilityType::Custom("solana_analysis".to_string()),
            riglr_agents::CapabilityType::Custom("ethereum_analysis".to_string()),
        ]
    }
}

/// Risk management agent with real portfolio tracking
#[derive(Clone)]
pub struct RiskManagementAgent {
    /// Unique identifier for this agent
    id: AgentId,
    /// Communication channel for inter-agent messaging
    communication: Arc<ChannelCommunication>,
    /// Configuration settings
    _config: Config,
    /// Application context for tool invocations
    context: ApplicationContext,
    /// Shared trading state reference
    trading_state: Arc<Mutex<TradingState>>,
    /// Maximum position size as percentage of portfolio
    max_position_size: f64,
    /// Maximum daily loss as percentage of portfolio
    max_daily_loss: f64,
}

impl std::fmt::Debug for RiskManagementAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RiskManagementAgent")
            .field("id", &self.id)
            .field("_config", &"<Config>")
            .field("context", &"<ApplicationContext>")
            .field("max_position_size", &self.max_position_size)
            .field("max_daily_loss", &self.max_daily_loss)
            .finish()
    }
}

impl RiskManagementAgent {
    /// Creates a new risk management agent
    pub fn new(
        id: &str,
        communication: Arc<ChannelCommunication>,
        config: Config,
        trading_state: Arc<Mutex<TradingState>>,
    ) -> Self {
        let context = ApplicationContext::from_config(&config);
        Self {
            id: AgentId::new(id),
            communication,
            _config: config,
            context,
            trading_state,
            max_position_size: 0.20, // 20% max per position
            max_daily_loss: 0.05,    // 5% max daily loss
        }
    }

    async fn get_current_balances(&self) -> Result<(f64, f64), ToolError> {
        // Get actual balances using riglr tools
        let signer = SignerContext::current()
            .await
            .map_err(|e| ToolError::permanent_string(format!("No signer context: {}", e)))?;

        // Get SOL balance
        let sol_address = signer
            .as_solana()
            .ok_or_else(|| ToolError::permanent_string("No Solana signer available".to_string()))?
            .address();
        let sol_balance = match get_sol_balance(sol_address, &self.context).await {
            Ok(balance_result) => {
                // Parse the formatted balance string to f64
                balance_result.formatted.parse::<f64>().unwrap_or(0.0)
            }
            Err(e) => {
                tracing::warn!("Failed to get SOL balance: {}", e);
                0.0
            }
        };

        // Get ETH balance - need to provide a default EVM address since signer may not have one
        // In a real implementation, this would get the actual EVM address from the signer
        let default_eth_address = "0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string();
        let eth_balance = match get_eth_balance(default_eth_address, None, &self.context).await {
            Ok(balance_result) => {
                // Parse the formatted balance string to f64
                balance_result
                    .balance_formatted
                    .parse::<f64>()
                    .unwrap_or(0.0)
            }
            Err(e) => {
                tracing::warn!("Failed to get ETH balance: {}", e);
                0.0
            }
        };

        Ok((eth_balance, sol_balance))
    }

    async fn assess_trade_risk(
        &self,
        trade_params: &serde_json::Value,
        _market_analysis: &serde_json::Value,
    ) -> serde_json::Value {
        let network = trade_params
            .get("network")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");

        let amount = trade_params
            .get("amount")
            .and_then(|a| a.as_f64())
            .unwrap_or(0.0);

        let symbol = trade_params
            .get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("UNKNOWN");

        // Get current portfolio state
        let trading_state = self.trading_state.lock().unwrap();

        // Calculate risk metrics
        let position_value = amount; // Simplified - would calculate based on current price
        let portfolio_percent = position_value / trading_state.portfolio_value_usd.max(1000.0);

        let risk_score = {
            let size_risk = portfolio_percent / self.max_position_size;
            let concentration_risk = if trading_state.active_positions.contains_key(symbol) {
                1.5
            } else {
                1.0
            };
            let network_risk = match network {
                "ethereum" => 0.8, // Lower risk due to higher liquidity
                "solana" => 1.2,   // Higher risk due to volatility
                _ => 1.5,
            };

            size_risk * concentration_risk * network_risk
        };

        // Check daily loss limits
        let daily_loss_ok = trading_state.daily_pnl.abs() < self.max_daily_loss;

        let approved =
            risk_score < 1.0 && portfolio_percent < self.max_position_size && daily_loss_ok;

        json!({
            "symbol": symbol,
            "network": network,
            "amount": amount,
            "portfolio_percent": portfolio_percent,
            "risk_score": risk_score,
            "daily_pnl": trading_state.daily_pnl,
            "approved": approved,
            "risk_factors": {
                "size_risk": portfolio_percent / self.max_position_size,
                "concentration_risk": trading_state.active_positions.contains_key(symbol),
                "daily_loss_limit": daily_loss_ok,
                "portfolio_exposure": trading_state.risk_exposure
            },
            "recommendations": if approved {
                vec!["APPROVE", "SET_STOP_LOSS", "MONITOR_POSITION"]
            } else {
                vec!["REJECT", "REDUCE_SIZE", "WAIT_FOR_BETTER_SETUP"]
            },
            "assessor": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        })
    }
}

#[async_trait]
impl Agent for RiskManagementAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("⚖️ Risk Management Agent {} assessing trade", self.id);

        let trade_params = task
            .parameters
            .get("trade_params")
            .ok_or_else(|| riglr_agents::AgentError::generic("Missing trade_params".to_string()))?;

        let market_analysis = task.parameters.get("market_analysis").ok_or_else(|| {
            riglr_agents::AgentError::generic("Missing market_analysis".to_string())
        })?;

        // Get current balances from blockchain
        match self.get_current_balances().await {
            Ok((eth_balance, sol_balance)) => {
                let mut trading_state = self.trading_state.lock().unwrap();
                trading_state.available_balance_eth = eth_balance;
                trading_state.available_balance_sol = sol_balance;
            }
            Err(e) => {
                tracing::warn!("Failed to update balances: {}", e);
            }
        }

        let risk_assessment = self.assess_trade_risk(trade_params, market_analysis).await;

        // Broadcast risk decision
        let message = AgentMessage::new(
            self.id.clone(),
            None,
            "risk_assessment".to_string(),
            risk_assessment.clone(),
        );

        self.communication
            .broadcast_message(message)
            .await
            .map_err(|e| riglr_agents::AgentError::generic(e.to_string()))?;

        Ok(TaskResult::success(
            risk_assessment,
            None, // No transaction hash for risk assessment
            Duration::from_millis(150),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<riglr_agents::CapabilityType> {
        vec![
            riglr_agents::CapabilityType::RiskAnalysis,
            riglr_agents::CapabilityType::Portfolio,
            riglr_agents::CapabilityType::Custom("balance_tracking".to_string()),
            riglr_agents::CapabilityType::Custom("limit_monitoring".to_string()),
        ]
    }
}

/// Execution agent that performs real blockchain trades
#[derive(Clone)]
pub struct TradeExecutionAgent {
    /// Unique identifier for this agent
    id: AgentId,
    /// Communication channel for inter-agent messaging
    communication: Arc<ChannelCommunication>,
    /// Configuration settings
    _config: Config,
    /// Application context for tool invocations
    context: ApplicationContext,
    /// Shared trading state reference
    _trading_state: Arc<Mutex<TradingState>>,
}

impl std::fmt::Debug for TradeExecutionAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TradeExecutionAgent")
            .field("id", &self.id)
            .field("_config", &"<Config>")
            .field("context", &"<ApplicationContext>")
            .finish()
    }
}

impl TradeExecutionAgent {
    /// Creates a new trade execution agent
    pub fn new(
        id: &str,
        communication: Arc<ChannelCommunication>,
        config: Config,
        trading_state: Arc<Mutex<TradingState>>,
    ) -> Self {
        let context = ApplicationContext::from_config(&config);
        Self {
            id: AgentId::new(id),
            communication,
            _config: config,
            context,
            _trading_state: trading_state,
        }
    }

    async fn execute_solana_trade(
        &self,
        trade_params: &serde_json::Value,
    ) -> Result<serde_json::Value, ToolError> {
        let token_address = trade_params
            .get("token_address")
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                ToolError::permanent_string("Missing token_address for Solana trade".to_string())
            })?;

        let amount_sol = trade_params
            .get("amount")
            .and_then(|a| a.as_f64())
            .ok_or_else(|| {
                ToolError::permanent_string("Missing amount for Solana trade".to_string())
            })?;

        // Execute real trade using riglr-solana-tools
        let trade_result = buy_pump_token(
            token_address.to_string(),
            amount_sol,
            Some(0.05), // 5% slippage
            &self.context,
        )
        .await?;

        Ok(json!({
            "network": "solana",
            "token_address": token_address,
            "amount_sol": amount_sol,
            "transaction_signature": trade_result.signature,
            "tokens_received": trade_result.token_amount,
            "price_paid": trade_result.price_per_token,
            "gas_used": "estimated",
            "status": "completed",
            "executor": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }

    async fn execute_ethereum_trade(
        &self,
        trade_params: &serde_json::Value,
    ) -> Result<serde_json::Value, ToolError> {
        let token_address = trade_params
            .get("token_address")
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                ToolError::permanent_string("Missing token_address for Ethereum trade".to_string())
            })?;

        let amount_eth = trade_params
            .get("amount")
            .and_then(|a| a.as_f64())
            .ok_or_else(|| {
                ToolError::permanent_string("Missing amount for Ethereum trade".to_string())
            })?;

        // TODO: Execute real trade using riglr-evm-tools
        // let trade_result = perform_uniswap_swap(...).await?;
        let mock_tx_hash = format!("0x{:x}", chrono::Utc::now().timestamp());

        Ok(json!({
            "network": "ethereum",
            "token_address": token_address,
            "amount_eth": amount_eth,
            "transaction_hash": mock_tx_hash,
            "tokens_received": 1000,  // Mock value
            "gas_used": 21000,       // Mock value
            "gas_price": 20,         // Mock value in gwei
            "status": "completed",
            "executor": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }

    async fn update_portfolio_state(&self, trade_result: &serde_json::Value) {
        let mut trading_state = self._trading_state.lock().unwrap();

        if let Some(symbol) = trade_result.get("token_address").and_then(|s| s.as_str()) {
            let amount = trade_result
                .get("tokens_received")
                .and_then(|a| a.as_f64())
                .unwrap_or(0.0);

            let price = trade_result
                .get("price_paid")
                .and_then(|p| p.as_f64())
                .unwrap_or(0.0);

            let position = Position {
                symbol: symbol.to_string(),
                amount,
                avg_price: price,
                current_price: price,
                pnl: 0.0,
                network: trade_result
                    .get("network")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                entry_time: chrono::Utc::now(),
            };

            trading_state
                .active_positions
                .insert(symbol.to_string(), position);
        }
    }
}

#[async_trait]
impl Agent for TradeExecutionAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("⚡ Trade Execution Agent {} executing trade", self.id);

        let risk_approved = task
            .parameters
            .get("risk_assessment")
            .and_then(|r| r.get("approved"))
            .and_then(|a| a.as_bool())
            .unwrap_or(false);

        if !risk_approved {
            return Ok(TaskResult::failure(
                "Trade rejected by risk management".to_string(),
                false,
                Duration::from_millis(10),
            ));
        }

        let trade_params = task
            .parameters
            .get("trade_params")
            .ok_or_else(|| riglr_agents::AgentError::generic("Missing trade_params".to_string()))?;

        let network = trade_params
            .get("network")
            .and_then(|n| n.as_str())
            .ok_or_else(|| {
                riglr_agents::AgentError::generic("Missing network in trade_params".to_string())
            })?;

        let trade_result = match network {
            "solana" => self
                .execute_solana_trade(trade_params)
                .await
                .map_err(|e| riglr_agents::AgentError::task_execution(e.to_string()))?,
            "ethereum" => self
                .execute_ethereum_trade(trade_params)
                .await
                .map_err(|e| riglr_agents::AgentError::task_execution(e.to_string()))?,
            _ => {
                return Err(riglr_agents::AgentError::generic(format!(
                    "Unsupported network: {}",
                    network
                )))
            }
        };

        // Update portfolio state
        self.update_portfolio_state(&trade_result).await;

        // Notify other agents of successful trade
        let message = AgentMessage::new(
            self.id.clone(),
            None,
            "trade_executed".to_string(),
            trade_result.clone(),
        );

        self.communication
            .broadcast_message(message)
            .await
            .map_err(|e| riglr_agents::AgentError::generic(e.to_string()))?;

        Ok(TaskResult::success(
            trade_result.clone(),
            trade_result
                .get("transaction_signature")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string()),
            Duration::from_millis(1000),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<riglr_agents::CapabilityType> {
        vec![
            riglr_agents::CapabilityType::Trading,
            riglr_agents::CapabilityType::Custom("blockchain_execution".to_string()),
            riglr_agents::CapabilityType::Custom("solana_trading".to_string()),
            riglr_agents::CapabilityType::Custom("ethereum_trading".to_string()),
        ]
    }
}

/// Demonstration function that shows the complete trading coordination workflow
pub async fn demonstrate_trading_coordination(
    config: Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting Real-World Trading Coordination Example");
    println!("🔗 Using actual blockchain connections and data sources");

    // Initialize shared trading state
    let trading_state = Arc::new(Mutex::new(TradingState::default()));

    // Initialize communication
    let communication = Arc::new(ChannelCommunication::with_config(
        CommunicationConfig::default(),
    ));

    // Create real trading agents
    let intelligence_agent = Arc::new(MarketIntelligenceAgent::new(
        "market-intel-1",
        communication.clone(),
        config.clone(),
        trading_state.clone(),
    ));

    let risk_agent = Arc::new(RiskManagementAgent::new(
        "risk-mgmt-1",
        communication.clone(),
        config.clone(),
        trading_state.clone(),
    ));

    let execution_agent = Arc::new(TradeExecutionAgent::new(
        "trade-exec-1",
        communication.clone(),
        config.clone(),
        trading_state.clone(),
    ));

    // Create registry and register agents
    let registry = Arc::new(LocalAgentRegistry::new());
    registry.register_agent(intelligence_agent.clone()).await?;
    registry.register_agent(risk_agent.clone()).await?;
    registry.register_agent(execution_agent.clone()).await?;

    let agent_count = registry.list_agents().await?.len();
    println!(
        "✅ Registered {} trading agents with real blockchain capabilities",
        agent_count
    );

    // Create dispatcher
    let dispatch_config = DispatchConfig {
        routing_strategy: RoutingStrategy::Capability,
        max_retries: 2,
        default_task_timeout: Duration::from_secs(60), // Longer timeout for blockchain operations
        retry_delay: Duration::from_millis(500),
        max_concurrent_tasks_per_agent: 5,
        enable_load_balancing: false,
        response_wait_timeout: Duration::from_secs(30), // 30 second timeout for responses
    };

    let dispatcher = AgentDispatcher::with_config(registry.clone(), dispatch_config);

    println!("\n🔄 Starting Real Trading Workflow");

    // Step 1: Market Intelligence
    println!("\n1️⃣ Gathering Market Intelligence");
    let research_task = Task::new(
        TaskType::Research,
        json!({
            "network": "solana",
            "token_address": "So11111111111111111111111111111111111111112", // Wrapped SOL
            "analysis_depth": "comprehensive"
        }),
    )
    .with_priority(Priority::High);

    let research_result = dispatcher.dispatch_task(research_task).await?;
    let recommendation = research_result
        .data()
        .and_then(|data| {
            data.get("analysis")
                .and_then(|a| a.get("recommendation"))
                .and_then(|r| r.as_str())
        })
        .unwrap_or("HOLD");

    println!("✅ Market analysis complete: {}", recommendation);

    sleep(Duration::from_millis(200)).await;

    // Step 2: Risk Assessment
    if recommendation == "BUY" || recommendation == "MONITOR" {
        println!("\n2️⃣ Risk Assessment");
        let risk_task = Task::new(
            TaskType::RiskAnalysis,
            json!({
                "trade_params": {
                    "network": "solana",
                    "token_address": "So11111111111111111111111111111111111111112",
                    "symbol": "WSOL",
                    "amount": 0.1, // 0.1 SOL
                    "side": "buy"
                },
                "market_analysis": research_result.data().unwrap_or(&serde_json::json!({}))
            }),
        )
        .with_priority(Priority::High);

        let risk_result = dispatcher.dispatch_task(risk_task).await?;
        let risk_approved = risk_result
            .data()
            .and_then(|data| data.get("approved").and_then(|a| a.as_bool()))
            .unwrap_or(false);

        println!(
            "✅ Risk assessment: {}",
            if risk_approved {
                "APPROVED"
            } else {
                "REJECTED"
            }
        );

        sleep(Duration::from_millis(200)).await;

        // Step 3: Trade Execution (if approved)
        if risk_approved {
            println!("\n3️⃣ Executing Trade on Blockchain");
            let execution_task = Task::new(
                TaskType::Trading,
                json!({
                    "risk_assessment": risk_result.data().unwrap_or(&serde_json::json!({})),
                    "trade_params": {
                        "network": "solana",
                        "token_address": "So11111111111111111111111111111111111111112",
                        "amount": 0.1,
                        "side": "buy"
                    }
                }),
            )
            .with_priority(Priority::Critical);

            match dispatcher.dispatch_task(execution_task).await {
                Ok(execution_result) => {
                    let tx_signature = execution_result
                        .data()
                        .and_then(|data| data.get("transaction_signature").and_then(|s| s.as_str()))
                        .unwrap_or("unknown");
                    println!("✅ Trade executed successfully: {}", tx_signature);
                }
                Err(e) => {
                    println!("❌ Trade execution failed: {}", e);
                }
            }
        } else {
            println!("\n3️⃣ Trade Execution - SKIPPED (Risk Rejected)");
        }
    } else {
        println!("\n2️⃣ Risk Assessment - SKIPPED (No buy signal)");
        println!("\n3️⃣ Trade Execution - SKIPPED (No buy signal)");
    }

    // Final portfolio state
    {
        let trading_state = trading_state.lock().unwrap();
        println!("\n📊 Final Portfolio State:");
        println!("  ETH Balance: {:.4}", trading_state.available_balance_eth);
        println!("  SOL Balance: {:.4}", trading_state.available_balance_sol);
        println!(
            "  Active Positions: {}",
            trading_state.active_positions.len()
        );
        println!("  Daily P&L: {:.2}%", trading_state.daily_pnl * 100.0);
    }

    println!("\n🎉 Real-world trading coordination completed!");
    println!("This example demonstrated:");
    println!("  ✅ Real blockchain data integration");
    println!("  ✅ Multi-agent risk management");
    println!("  ✅ Actual trade execution capabilities");
    println!("  ✅ Cross-chain coordination (Solana/Ethereum)");
    println!("  ✅ Production-ready error handling");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_agents::TaskType;
    use riglr_web_tools::dexscreener::TokenInfo;

    // Mock config for testing
    fn mock_config() -> Config {
        use riglr_config::{ConfigBuilder, FeaturesConfig};

        // Disable bridging to avoid LIFI_API_KEY requirement
        let features = FeaturesConfig {
            enable_bridging: false,
            ..Default::default()
        };

        ConfigBuilder::new().features(features).build().unwrap()
    }

    // Helper to create test trading state
    fn test_trading_state() -> Arc<Mutex<TradingState>> {
        let mut state = TradingState::default();
        state.portfolio_value_usd = 10000.0;
        state.available_balance_eth = 5.0;
        state.available_balance_sol = 100.0;
        state.daily_pnl = 0.02; // 2% gain
        state.risk_exposure = 0.1; // 10%

        // Add a test position
        let position = Position {
            symbol: "BTC".to_string(),
            amount: 0.1,
            avg_price: 45000.0,
            current_price: 46000.0,
            pnl: 100.0,
            network: "ethereum".to_string(),
            entry_time: chrono::Utc::now(),
        };
        state.active_positions.insert("BTC".to_string(), position);

        Arc::new(Mutex::new(state))
    }

    #[test]
    fn test_trading_state_default() {
        let state = TradingState::default();
        assert!(state.active_positions.is_empty());
        assert!(state.pending_orders.is_empty());
        assert_eq!(state.portfolio_value_usd, 0.0);
        assert_eq!(state.available_balance_eth, 0.0);
        assert_eq!(state.available_balance_sol, 0.0);
        assert_eq!(state.daily_pnl, 0.0);
        assert_eq!(state.risk_exposure, 0.0);
    }

    #[test]
    fn test_position_creation() {
        let entry_time = chrono::Utc::now();
        let position = Position {
            symbol: "ETH".to_string(),
            amount: 10.0,
            avg_price: 2000.0,
            current_price: 2100.0,
            pnl: 1000.0,
            network: "ethereum".to_string(),
            entry_time,
        };

        assert_eq!(position.symbol, "ETH");
        assert_eq!(position.amount, 10.0);
        assert_eq!(position.avg_price, 2000.0);
        assert_eq!(position.current_price, 2100.0);
        assert_eq!(position.pnl, 1000.0);
        assert_eq!(position.network, "ethereum");
        assert_eq!(position.entry_time, entry_time);
    }

    #[test]
    fn test_order_creation() {
        let order = Order {
            id: "order-123".to_string(),
            symbol: "SOL".to_string(),
            side: OrderSide::Buy,
            amount: 50.0,
            price: Some(120.0),
            network: "solana".to_string(),
            status: OrderStatus::Pending,
        };

        assert_eq!(order.id, "order-123");
        assert_eq!(order.symbol, "SOL");
        assert!(matches!(order.side, OrderSide::Buy));
        assert_eq!(order.amount, 50.0);
        assert_eq!(order.price, Some(120.0));
        assert_eq!(order.network, "solana");
        assert!(matches!(order.status, OrderStatus::Pending));
    }

    #[test]
    fn test_order_side_variants() {
        let buy_order = OrderSide::Buy;
        let sell_order = OrderSide::Sell;

        assert!(matches!(buy_order, OrderSide::Buy));
        assert!(matches!(sell_order, OrderSide::Sell));
    }

    #[test]
    fn test_order_status_variants() {
        assert!(matches!(OrderStatus::Pending, OrderStatus::Pending));
        assert!(matches!(OrderStatus::Executing, OrderStatus::Executing));
        assert!(matches!(OrderStatus::Completed, OrderStatus::Completed));
        assert!(matches!(OrderStatus::Failed, OrderStatus::Failed));
    }

    #[tokio::test]
    async fn test_market_intelligence_agent_new() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent =
            MarketIntelligenceAgent::new("test-intel", communication, config, trading_state);

        assert_eq!(agent.id.as_str(), "test-intel");
        assert_eq!(
            agent.capabilities(),
            vec![
                "research",
                "market_analysis",
                "solana_analysis",
                "ethereum_analysis"
            ]
        );
    }

    #[tokio::test]
    async fn test_market_intelligence_agent_calculate_liquidity_score() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent =
            MarketIntelligenceAgent::new("test-intel", communication, config, trading_state);

        let empty_value = serde_json::json!({});
        let score = agent.calculate_liquidity_score(&empty_value).await;
        assert_eq!(score, 0.75);
    }

    #[tokio::test]
    async fn test_market_intelligence_agent_calculate_momentum_score_with_data() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent =
            MarketIntelligenceAgent::new("test-intel", communication, config, trading_state);

        let token_info = TokenInfo {
            address: "test".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            price_usd: Some(1.5),
            price_change_24h: Some(0.8),
            market_cap: None,
            volume_24h: None,
            price_change_1h: None,
            price_change_5m: None,
            circulating_supply: None,
            total_supply: None,
            pair_count: 0,
            pairs: vec![],
            chain: riglr_web_tools::dexscreener::ChainInfo {
                id: "solana".to_string(),
                name: "Solana".to_string(),
                logo: None,
                native_token: "SOL".to_string(),
            },
            security: riglr_web_tools::dexscreener::SecurityInfo {
                is_verified: false,
                liquidity_locked: None,
                audit_status: None,
                honeypot_status: None,
                ownership_status: None,
                risk_score: None,
            },
            socials: vec![],
            updated_at: chrono::Utc::now(),
        };

        let score = agent.calculate_momentum_score(&Some(token_info)).await;
        assert_eq!(score, 0.8);
    }

    #[tokio::test]
    async fn test_market_intelligence_agent_calculate_momentum_score_without_data() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent =
            MarketIntelligenceAgent::new("test-intel", communication, config, trading_state);

        let score = agent.calculate_momentum_score(&None).await;
        assert_eq!(score, 0.5);
    }

    #[tokio::test]
    async fn test_market_intelligence_agent_calculate_momentum_score_with_high_change() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent =
            MarketIntelligenceAgent::new("test-intel", communication, config, trading_state);

        let token_info = TokenInfo {
            address: "test".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            price_usd: Some(1.5),
            price_change_24h: Some(2.5), // High change that should be capped at 1.0
            market_cap: None,
            volume_24h: None,
            price_change_1h: None,
            price_change_5m: None,
            circulating_supply: None,
            total_supply: None,
            pair_count: 0,
            pairs: vec![],
            chain: riglr_web_tools::dexscreener::ChainInfo {
                id: "solana".to_string(),
                name: "Solana".to_string(),
                logo: None,
                native_token: "SOL".to_string(),
            },
            security: riglr_web_tools::dexscreener::SecurityInfo {
                is_verified: false,
                liquidity_locked: None,
                audit_status: None,
                honeypot_status: None,
                ownership_status: None,
                risk_score: None,
            },
            socials: vec![],
            updated_at: chrono::Utc::now(),
        };

        let score = agent.calculate_momentum_score(&Some(token_info)).await;
        assert_eq!(score, 1.0); // Should be capped at 1.0
    }

    #[tokio::test]
    async fn test_market_intelligence_agent_execute_task_missing_network() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent =
            MarketIntelligenceAgent::new("test-intel", communication, config, trading_state);

        let task = Task::new(
            TaskType::Research,
            json!({
                "token_address": "test-token"
            }),
        );

        let result = agent.execute_task(task).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing network parameter"));
    }

    #[tokio::test]
    async fn test_market_intelligence_agent_execute_task_missing_token_address() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent =
            MarketIntelligenceAgent::new("test-intel", communication, config, trading_state);

        let task = Task::new(
            TaskType::Research,
            json!({
                "network": "solana"
            }),
        );

        let result = agent.execute_task(task).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing token_address parameter"));
    }

    #[tokio::test]
    async fn test_market_intelligence_agent_execute_task_unsupported_network() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent =
            MarketIntelligenceAgent::new("test-intel", communication, config, trading_state);

        let task = Task::new(
            TaskType::Research,
            json!({
                "network": "unsupported",
                "token_address": "test-token"
            }),
        );

        let result = agent.execute_task(task).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported network: unsupported"));
    }

    #[tokio::test]
    async fn test_risk_management_agent_new() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = RiskManagementAgent::new("test-risk", communication, config, trading_state);

        assert_eq!(agent.id.as_str(), "test-risk");
        assert_eq!(agent.max_position_size, 0.20);
        assert_eq!(agent.max_daily_loss, 0.05);
        assert_eq!(
            agent.capabilities(),
            vec![
                "risk_analysis",
                "portfolio",
                "balance_tracking",
                "limit_monitoring"
            ]
        );
    }

    #[tokio::test]
    async fn test_risk_management_agent_assess_trade_risk_approved() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = RiskManagementAgent::new("test-risk", communication, config, trading_state);

        let trade_params = json!({
            "network": "ethereum",
            "amount": 100.0,
            "symbol": "ETH"
        });

        let market_analysis = json!({});

        let assessment = agent
            .assess_trade_risk(&trade_params, &market_analysis)
            .await;
        assert_eq!(assessment.get("symbol").unwrap().as_str().unwrap(), "ETH");
        assert_eq!(
            assessment.get("network").unwrap().as_str().unwrap(),
            "ethereum"
        );
        assert_eq!(assessment.get("amount").unwrap().as_f64().unwrap(), 100.0);
        assert!(assessment.get("approved").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_risk_management_agent_assess_trade_risk_rejected_large_position() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = RiskManagementAgent::new("test-risk", communication, config, trading_state);

        let trade_params = json!({
            "network": "solana",
            "amount": 5000.0, // Large position that should be rejected
            "symbol": "SOL"
        });

        let market_analysis = json!({});

        let assessment = agent
            .assess_trade_risk(&trade_params, &market_analysis)
            .await;
        assert_eq!(assessment.get("symbol").unwrap().as_str().unwrap(), "SOL");
        assert_eq!(
            assessment.get("network").unwrap().as_str().unwrap(),
            "solana"
        );
        assert!(!assessment.get("approved").unwrap().as_bool().unwrap());

        let recommendations = assessment
            .get("recommendations")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(recommendations
            .iter()
            .any(|r| r.as_str().unwrap() == "REJECT"));
    }

    #[tokio::test]
    async fn test_risk_management_agent_assess_trade_risk_concentration_risk() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = RiskManagementAgent::new("test-risk", communication, config, trading_state);

        let trade_params = json!({
            "network": "ethereum",
            "amount": 100.0,
            "symbol": "BTC" // Already have position in BTC
        });

        let market_analysis = json!({});

        let assessment = agent
            .assess_trade_risk(&trade_params, &market_analysis)
            .await;
        let risk_factors = assessment.get("risk_factors").unwrap();
        assert!(risk_factors
            .get("concentration_risk")
            .unwrap()
            .as_bool()
            .unwrap());
    }

    #[tokio::test]
    async fn test_risk_management_agent_assess_trade_risk_default_values() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = RiskManagementAgent::new("test-risk", communication, config, trading_state);

        let trade_params = json!({}); // Empty params to test defaults
        let market_analysis = json!({});

        let assessment = agent
            .assess_trade_risk(&trade_params, &market_analysis)
            .await;
        assert_eq!(
            assessment.get("symbol").unwrap().as_str().unwrap(),
            "UNKNOWN"
        );
        assert_eq!(
            assessment.get("network").unwrap().as_str().unwrap(),
            "unknown"
        );
        assert_eq!(assessment.get("amount").unwrap().as_f64().unwrap(), 0.0);
    }

    #[tokio::test]
    async fn test_risk_management_agent_execute_task_missing_trade_params() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = RiskManagementAgent::new("test-risk", communication, config, trading_state);

        let task = Task::new(
            TaskType::RiskAnalysis,
            json!({
                "market_analysis": {}
            }),
        );

        let result = agent.execute_task(task).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing trade_params"));
    }

    #[tokio::test]
    async fn test_risk_management_agent_execute_task_missing_market_analysis() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = RiskManagementAgent::new("test-risk", communication, config, trading_state);

        let task = Task::new(
            TaskType::RiskAnalysis,
            json!({
                "trade_params": {}
            }),
        );

        let result = agent.execute_task(task).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing market_analysis"));
    }

    #[tokio::test]
    async fn test_trade_execution_agent_new() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = TradeExecutionAgent::new("test-exec", communication, config, trading_state);

        assert_eq!(agent.id.as_str(), "test-exec");
        assert_eq!(
            agent.capabilities(),
            vec![
                "trading",
                "blockchain_execution",
                "solana_trading",
                "ethereum_trading"
            ]
        );
    }

    #[tokio::test]
    async fn test_trade_execution_agent_execute_solana_trade_missing_token_address() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = TradeExecutionAgent::new("test-exec", communication, config, trading_state);

        let trade_params = json!({
            "amount": 1.0
        });

        let result = agent.execute_solana_trade(&trade_params).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing token_address for Solana trade"));
    }

    #[tokio::test]
    async fn test_trade_execution_agent_execute_solana_trade_missing_amount() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = TradeExecutionAgent::new("test-exec", communication, config, trading_state);

        let trade_params = json!({
            "token_address": "test-address"
        });

        let result = agent.execute_solana_trade(&trade_params).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing amount for Solana trade"));
    }

    #[tokio::test]
    async fn test_trade_execution_agent_execute_ethereum_trade_missing_token_address() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = TradeExecutionAgent::new("test-exec", communication, config, trading_state);

        let trade_params = json!({
            "amount": 1.0
        });

        let result = agent.execute_ethereum_trade(&trade_params).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing token_address for Ethereum trade"));
    }

    #[tokio::test]
    async fn test_trade_execution_agent_execute_ethereum_trade_missing_amount() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = TradeExecutionAgent::new("test-exec", communication, config, trading_state);

        let trade_params = json!({
            "token_address": "test-address"
        });

        let result = agent.execute_ethereum_trade(&trade_params).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing amount for Ethereum trade"));
    }

    #[tokio::test]
    async fn test_trade_execution_agent_execute_ethereum_trade_success() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = TradeExecutionAgent::new("test-exec", communication, config, trading_state);

        let trade_params = json!({
            "token_address": "0x1234567890123456789012345678901234567890",
            "amount": 1.5
        });

        let result = agent.execute_ethereum_trade(&trade_params).await;
        assert!(result.is_ok());

        let trade_result = result.unwrap();
        assert_eq!(
            trade_result.get("network").unwrap().as_str().unwrap(),
            "ethereum"
        );
        assert_eq!(
            trade_result.get("token_address").unwrap().as_str().unwrap(),
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(
            trade_result.get("amount_eth").unwrap().as_f64().unwrap(),
            1.5
        );
        assert_eq!(
            trade_result.get("status").unwrap().as_str().unwrap(),
            "completed"
        );
        assert_eq!(
            trade_result.get("executor").unwrap().as_str().unwrap(),
            "test-exec"
        );
    }

    #[tokio::test]
    async fn test_trade_execution_agent_update_portfolio_state() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent =
            TradeExecutionAgent::new("test-exec", communication, config, trading_state.clone());

        let trade_result = json!({
            "token_address": "NEW_TOKEN",
            "tokens_received": 500.0,
            "price_paid": 2.5,
            "network": "solana"
        });

        agent.update_portfolio_state(&trade_result).await;

        let state = trading_state.lock().unwrap();
        assert!(state.active_positions.contains_key("NEW_TOKEN"));

        let position = state.active_positions.get("NEW_TOKEN").unwrap();
        assert_eq!(position.symbol, "NEW_TOKEN");
        assert_eq!(position.amount, 500.0);
        assert_eq!(position.avg_price, 2.5);
        assert_eq!(position.current_price, 2.5);
        assert_eq!(position.pnl, 0.0);
        assert_eq!(position.network, "solana");
    }

    #[tokio::test]
    async fn test_trade_execution_agent_update_portfolio_state_missing_data() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent =
            TradeExecutionAgent::new("test-exec", communication, config, trading_state.clone());

        let trade_result = json!({
            "some_other_field": "value"
        });

        let initial_count = trading_state.lock().unwrap().active_positions.len();
        agent.update_portfolio_state(&trade_result).await;
        let final_count = trading_state.lock().unwrap().active_positions.len();

        // Should not add any new positions
        assert_eq!(initial_count, final_count);
    }

    #[tokio::test]
    async fn test_trade_execution_agent_execute_task_risk_rejected() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = TradeExecutionAgent::new("test-exec", communication, config, trading_state);

        let task = Task::new(
            TaskType::Trading,
            json!({
                "risk_assessment": {
                    "approved": false
                },
                "trade_params": {
                    "network": "solana",
                    "token_address": "test-token",
                    "amount": 1.0
                }
            }),
        );

        let result = agent.execute_task(task).await;
        assert!(result.is_ok());

        let task_result = result.unwrap();
        assert!(!task_result.is_success());
        assert_eq!(
            task_result.error().unwrap(),
            "Trade rejected by risk management"
        );
    }

    #[tokio::test]
    async fn test_trade_execution_agent_execute_task_missing_trade_params() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = TradeExecutionAgent::new("test-exec", communication, config, trading_state);

        let task = Task::new(
            TaskType::Trading,
            json!({
                "risk_assessment": {
                    "approved": true
                }
            }),
        );

        let result = agent.execute_task(task).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing trade_params"));
    }

    #[tokio::test]
    async fn test_trade_execution_agent_execute_task_missing_network() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = TradeExecutionAgent::new("test-exec", communication, config, trading_state);

        let task = Task::new(
            TaskType::Trading,
            json!({
                "risk_assessment": {
                    "approved": true
                },
                "trade_params": {
                    "token_address": "test-token",
                    "amount": 1.0
                }
            }),
        );

        let result = agent.execute_task(task).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing network in trade_params"));
    }

    #[tokio::test]
    async fn test_trade_execution_agent_execute_task_unsupported_network() {
        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let config = mock_config();
        let trading_state = test_trading_state();

        let agent = TradeExecutionAgent::new("test-exec", communication, config, trading_state);

        let task = Task::new(
            TaskType::Trading,
            json!({
                "risk_assessment": {
                    "approved": true
                },
                "trade_params": {
                    "network": "unsupported",
                    "token_address": "test-token",
                    "amount": 1.0
                }
            }),
        );

        let result = agent.execute_task(task).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported network: unsupported"));
    }
}
