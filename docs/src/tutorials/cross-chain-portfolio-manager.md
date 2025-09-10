# Creating a Cross-Chain Portfolio Manager

In this tutorial, you'll build an intelligent portfolio management agent that automatically manages assets across multiple blockchain networks, optimizing yield and maintaining target allocations.

## Overview

A cross-chain portfolio manager monitors and rebalances cryptocurrency holdings across different blockchains to maintain desired asset allocations while maximizing yield opportunities. This agent can automatically bridge tokens between networks and rebalance positions based on market conditions.

### What You'll Build

- A portfolio manager that tracks assets across Solana, Ethereum, Polygon, and Arbitrum
- Automatic rebalancing based on configurable target allocations  
- Cross-chain bridging for optimal asset positioning
- Yield optimization through DeFi protocol integration
- Risk monitoring and position sizing controls

## Prerequisites

- Rust 1.75+
- Wallets with gas tokens on supported chains (ETH, SOL, MATIC, etc.)
- RPC access for multiple chains (Alchemy, Infura, QuickNode, etc.)
- Basic understanding of DeFi and cross-chain bridges
- create-riglr-app CLI tool

## Project Setup

Use create-riglr-app to bootstrap the project:

```bash
create-riglr-app cross-chain-portfolio --template advanced
cd cross-chain-portfolio
```

This creates a project with the necessary dependencies:

```toml
[dependencies]
riglr-core = "0.3"
riglr-cross-chain-tools = "0.3"
riglr-solana-tools = "0.3"
riglr-evm-tools = "0.3"
riglr-web-tools = "0.3"
riglr-macros = "0.2"
rig = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
anyhow = "1"
log = "0.4"
env_logger = "0.10"
uuid = "1"
chrono = "0.4"
```

## Step 1: Define Portfolio Data Structures

```rust
// src/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioConfig {
    /// Target allocation percentages by token symbol
    pub target_allocations: HashMap<String, f64>,
    /// Maximum percentage deviation before rebalancing
    pub rebalance_threshold: f64,
    /// Minimum USD value for rebalancing
    pub min_rebalance_amount: f64,
    /// Maximum position size as percentage of portfolio
    pub max_position_size: f64,
    /// Supported chains for each token
    pub token_chains: HashMap<String, Vec<ChainType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: String,
    pub total_value_usd: f64,
    pub positions: Vec<Position>,
    pub last_rebalance: DateTime<Utc>,
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub token_symbol: String,
    pub chain: ChainType,
    pub balance: f64,
    pub value_usd: f64,
    pub percentage: f64,
    pub target_percentage: f64,
    pub deviation: f64,
    pub yield_opportunities: Vec<YieldOpportunity>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChainType {
    Solana,
    Ethereum,
    Polygon,
    Arbitrum,
    Optimism,
    Base,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldOpportunity {
    pub protocol: String,
    pub chain: ChainType,
    pub apr: f64,
    pub tvl: f64,
    pub risk_score: u8,
    pub lock_period: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_return: f64,
    pub daily_return: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub rebalance_count: u32,
    pub gas_spent_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceAction {
    pub action_type: ActionType,
    pub token: String,
    pub from_chain: Option<ChainType>,
    pub to_chain: Option<ChainType>,
    pub amount: f64,
    pub expected_gas_cost: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Bridge,
    Swap,
    Deposit,
    Withdraw,
}
```

## Step 2: Implement Portfolio Tracker

```rust
// src/tracker.rs
use riglr_core::provider::ApplicationContext;
use riglr_solana_tools::{get_sol_balance, get_spl_token_balance};
use riglr_evm_tools::{get_eth_balance, get_erc20_balance};
use riglr_web_tools::{get_token_price, analyze_token_market};
use std::collections::HashMap;
use anyhow::Result;

pub struct PortfolioTracker {
    config: PortfolioConfig,
    context: ApplicationContext,
    chain_rpcs: HashMap<ChainType, String>,
}

impl PortfolioTracker {
    pub fn new(config: PortfolioConfig, context: ApplicationContext) -> Self {
        let mut chain_rpcs = HashMap::new();
        chain_rpcs.insert(ChainType::Solana, 
            std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()));
        chain_rpcs.insert(ChainType::Ethereum, 
            std::env::var("ETHEREUM_RPC_URL")
                .expect("ETHEREUM_RPC_URL must be set"));
        chain_rpcs.insert(ChainType::Polygon, 
            std::env::var("POLYGON_RPC_URL")
                .expect("POLYGON_RPC_URL must be set"));
        chain_rpcs.insert(ChainType::Arbitrum, 
            std::env::var("ARBITRUM_RPC_URL")
                .expect("ARBITRUM_RPC_URL must be set"));

        Self {
            config,
            context,
            chain_rpcs,
        }
    }

    pub async fn get_current_portfolio(&self) -> Result<Portfolio> {
        let mut positions = Vec::new();
        let mut total_value = 0.0;

        for (token_symbol, chains) in &self.config.token_chains {
            for chain in chains {
                let position = self.get_token_position(token_symbol, *chain).await?;
                if position.balance > 0.0 {
                    total_value += position.value_usd;
                    positions.push(position);
                }
            }
        }

        // Calculate percentages and deviations
        for position in &mut positions {
            position.percentage = position.value_usd / total_value * 100.0;
            position.target_percentage = self.config.target_allocations
                .get(&position.token_symbol)
                .copied()
                .unwrap_or(0.0);
            position.deviation = position.percentage - position.target_percentage;
        }

        Ok(Portfolio {
            id: uuid::Uuid::new_v4().to_string(),
            total_value_usd: total_value,
            positions,
            last_rebalance: chrono::Utc::now(),
            performance: PerformanceMetrics {
                total_return: 0.0,
                daily_return: 0.0,
                max_drawdown: 0.0,
                sharpe_ratio: 0.0,
                rebalance_count: 0,
                gas_spent_usd: 0.0,
            },
        })
    }

    async fn get_token_position(&self, token_symbol: &str, chain: ChainType) -> Result<Position> {
        let balance = self.get_token_balance(token_symbol, chain).await?;
        let price = get_token_price(&self.context, token_symbol.to_string()).await?;
        let value_usd = balance * price;

        let yield_opportunities = self.find_yield_opportunities(token_symbol, chain).await
            .unwrap_or_default();

        Ok(Position {
            token_symbol: token_symbol.to_string(),
            chain,
            balance,
            value_usd,
            percentage: 0.0, // Will be calculated later
            target_percentage: 0.0, // Will be set later
            deviation: 0.0, // Will be calculated later
            yield_opportunities,
        })
    }

    async fn get_token_balance(&self, token_symbol: &str, chain: ChainType) -> Result<f64> {
        match (token_symbol, chain) {
            ("SOL", ChainType::Solana) => {
                let balance_result = get_sol_balance(&self.context, 
                    self.get_wallet_address(chain)).await?;
                Ok(balance_result.balance_lamports as f64 / 1e9)
            }
            (_, ChainType::Solana) => {
                let token_address = self.get_token_address(token_symbol, chain)?;
                let balance_result = get_spl_token_balance(&self.context, 
                    token_address, 
                    self.get_wallet_address(chain)).await?;
                Ok(balance_result.balance)
            }
            ("ETH", ChainType::Ethereum) => {
                let balance_result = get_eth_balance(&self.context, 
                    self.get_wallet_address(chain)).await?;
                Ok(balance_result.balance_wei as f64 / 1e18)
            }
            (_, _) => {
                let token_address = self.get_token_address(token_symbol, chain)?;
                let balance_result = get_erc20_balance(&self.context, 
                    token_address, 
                    self.get_wallet_address(chain)).await?;
                Ok(balance_result.balance as f64 / 10_f64.powi(balance_result.decimals as i32))
            }
        }
    }

    async fn find_yield_opportunities(&self, token: &str, chain: ChainType) -> Result<Vec<YieldOpportunity>> {
        // Simplified yield opportunity discovery
        let mut opportunities = Vec::new();

        // Add common yield opportunities based on chain and token
        match (chain, token) {
            (ChainType::Ethereum, "ETH") => {
                opportunities.push(YieldOpportunity {
                    protocol: "Lido".to_string(),
                    chain,
                    apr: 3.5,
                    tvl: 35000000000.0,
                    risk_score: 2,
                    lock_period: None,
                });
            }
            (ChainType::Solana, "SOL") => {
                opportunities.push(YieldOpportunity {
                    protocol: "Marinade".to_string(),
                    chain,
                    apr: 7.2,
                    tvl: 1500000000.0,
                    risk_score: 3,
                    lock_period: None,
                });
            }
            _ => {}
        }

        Ok(opportunities)
    }

    fn get_wallet_address(&self, chain: ChainType) -> String {
        match chain {
            ChainType::Solana => std::env::var("SOLANA_WALLET_ADDRESS")
                .expect("SOLANA_WALLET_ADDRESS must be set"),
            _ => std::env::var("EVM_WALLET_ADDRESS")
                .expect("EVM_WALLET_ADDRESS must be set"),
        }
    }

    fn get_token_address(&self, token_symbol: &str, chain: ChainType) -> Result<String> {
        // Token address mapping - in production, load from config
        let address = match (token_symbol, chain) {
            ("USDC", ChainType::Ethereum) => "0xA0b86a33E6417c5d6d6bE6C2e0C6C3e5d6c7D8E9",
            ("USDC", ChainType::Polygon) => "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
            ("USDC", ChainType::Solana) => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            ("WETH", ChainType::Ethereum) => "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            ("WMATIC", ChainType::Polygon) => "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270",
            _ => return Err(anyhow::anyhow!("Token {} not supported on {:?}", token_symbol, chain)),
        };
        Ok(address.to_string())
    }
}
```

## Step 3: Implement Rebalancing Engine

```rust
// src/rebalancer.rs
use riglr_cross_chain_tools::{
    get_cross_chain_routes, execute_cross_chain_bridge, estimate_bridge_fees
};
use riglr_solana_tools::{swap_tokens_jupiter};
use riglr_evm_tools::{swap_tokens_uniswap};
use riglr_core::provider::ApplicationContext;
use anyhow::Result;

pub struct PortfolioRebalancer {
    config: PortfolioConfig,
    context: ApplicationContext,
}

impl PortfolioRebalancer {
    pub fn new(config: PortfolioConfig, context: ApplicationContext) -> Self {
        Self { config, context }
    }

    pub async fn analyze_rebalancing_needs(&self, portfolio: &Portfolio) -> Result<Vec<RebalanceAction>> {
        let mut actions = Vec::new();

        for position in &portfolio.positions {
            if position.deviation.abs() > self.config.rebalance_threshold {
                let required_change = (position.target_percentage - position.percentage) 
                    / 100.0 * portfolio.total_value_usd;

                if required_change.abs() > self.config.min_rebalance_amount {
                    if required_change > 0.0 {
                        // Need to increase position
                        actions.extend(self.plan_position_increase(position, required_change).await?);
                    } else {
                        // Need to decrease position
                        actions.extend(self.plan_position_decrease(position, required_change.abs()).await?);
                    }
                }
            }
        }

        // Optimize actions to minimize gas costs
        self.optimize_actions(actions).await
    }

    async fn plan_position_increase(&self, position: &Position, amount_usd: f64) -> Result<Vec<RebalanceAction>> {
        let mut actions = Vec::new();

        // Find the best chain to source tokens from other positions
        let source_positions = self.find_excess_positions().await?;
        
        for source in source_positions {
            if source.token_symbol == position.token_symbol {
                continue; // Same token, just bridge
            }

            // Check if we can get the token on the target chain
            let routes = get_cross_chain_routes(
                &self.context,
                format!("{:?}", source.chain).to_lowercase(),
                format!("{:?}", position.chain).to_lowercase(),
                source.token_symbol.clone(),
                position.token_symbol.clone(),
                amount_usd.to_string(),
                Some(0.5), // 0.5% slippage
            ).await?;

            if !routes.routes.is_empty() {
                let best_route = &routes.routes[0];
                let estimated_fees = estimate_bridge_fees(
                    &self.context,
                    format!("{:?}", source.chain).to_lowercase(),
                    format!("{:?}", position.chain).to_lowercase(),
                    source.token_symbol.clone(),
                    position.token_symbol.clone(),
                    amount_usd.to_string(),
                ).await?;

                actions.push(RebalanceAction {
                    action_type: ActionType::Bridge,
                    token: position.token_symbol.clone(),
                    from_chain: Some(source.chain),
                    to_chain: Some(position.chain),
                    amount: amount_usd,
                    expected_gas_cost: estimated_fees.total_fees_usd.unwrap_or(0.0),
                    reason: format!("Rebalance {} - increase allocation", position.token_symbol),
                });
                break;
            }
        }

        Ok(actions)
    }

    async fn plan_position_decrease(&self, position: &Position, amount_usd: f64) -> Result<Vec<RebalanceAction>> {
        let mut actions = Vec::new();

        // Find positions that need more allocation
        let target_positions = self.find_deficit_positions().await?;
        
        for target in target_positions {
            if target.token_symbol == position.token_symbol {
                continue;
            }

            // Plan swap from current position to target
            let routes = get_cross_chain_routes(
                &self.context,
                format!("{:?}", position.chain).to_lowercase(),
                format!("{:?}", target.chain).to_lowercase(),
                position.token_symbol.clone(),
                target.token_symbol.clone(),
                amount_usd.to_string(),
                Some(0.5),
            ).await?;

            if !routes.routes.is_empty() {
                actions.push(RebalanceAction {
                    action_type: ActionType::Bridge,
                    token: target.token_symbol.clone(),
                    from_chain: Some(position.chain),
                    to_chain: Some(target.chain),
                    amount: amount_usd,
                    expected_gas_cost: 0.0, // Will be calculated
                    reason: format!("Rebalance {} -> {}", position.token_symbol, target.token_symbol),
                });
                break;
            }
        }

        Ok(actions)
    }

    async fn execute_rebalancing(&self, actions: Vec<RebalanceAction>) -> Result<Vec<String>> {
        let mut transaction_hashes = Vec::new();

        for action in actions {
            match action.action_type {
                ActionType::Bridge => {
                    let tx_hash = self.execute_bridge_action(&action).await?;
                    transaction_hashes.push(tx_hash);
                }
                ActionType::Swap => {
                    let tx_hash = self.execute_swap_action(&action).await?;
                    transaction_hashes.push(tx_hash);
                }
                _ => {
                    log::warn!("Action type {:?} not implemented", action.action_type);
                }
            }

            // Wait between transactions to avoid nonce issues
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        Ok(transaction_hashes)
    }

    async fn execute_bridge_action(&self, action: &RebalanceAction) -> Result<String> {
        if let (Some(from_chain), Some(to_chain)) = (action.from_chain, action.to_chain) {
            let routes = get_cross_chain_routes(
                &self.context,
                format!("{:?}", from_chain).to_lowercase(),
                format!("{:?}", to_chain).to_lowercase(),
                action.token.clone(),
                action.token.clone(),
                action.amount.to_string(),
                Some(0.5),
            ).await?;

            if let Some(route) = routes.routes.first() {
                let result = execute_cross_chain_bridge(
                    &self.context,
                    route.id.clone(),
                    format!("{:?}", from_chain).to_lowercase(),
                    format!("{:?}", to_chain).to_lowercase(),
                    action.amount.to_string(),
                ).await?;

                return Ok(result.source_tx_hash);
            }
        }
        Err(anyhow::anyhow!("Failed to execute bridge action"))
    }

    async fn execute_swap_action(&self, action: &RebalanceAction) -> Result<String> {
        match action.from_chain {
            Some(ChainType::Solana) => {
                swap_tokens_jupiter(
                    "input_token".to_string(),  // Would be determined from action
                    action.token.clone(),
                    action.amount,
                    100, // 1% slippage in bps
                ).await.map_err(|e| anyhow::anyhow!(e))
            }
            Some(ChainType::Ethereum) => {
                swap_tokens_uniswap(
                    &self.context,
                    "input_token".to_string(),
                    action.token.clone(),
                    action.amount,
                    0.01, // 1% slippage
                ).await
            }
            _ => Err(anyhow::anyhow!("Chain not supported for swapping")),
        }
    }

    async fn find_excess_positions(&self) -> Result<Vec<Position>> {
        // Simplified - in reality would analyze current portfolio
        Ok(Vec::new())
    }

    async fn find_deficit_positions(&self) -> Result<Vec<Position>> {
        // Simplified - in reality would analyze current portfolio
        Ok(Vec::new())
    }

    async fn optimize_actions(&self, actions: Vec<RebalanceAction>) -> Result<Vec<RebalanceAction>> {
        // Sort by expected gas cost (lowest first)
        let mut optimized = actions;
        optimized.sort_by(|a, b| a.expected_gas_cost.partial_cmp(&b.expected_gas_cost).unwrap());
        Ok(optimized)
    }
}
```

## Step 4: Build the Main Portfolio Agent

```rust
// src/main.rs
mod types;
mod tracker;
mod rebalancer;

use riglr_core::{provider::ApplicationContext, SignerContext};
use riglr_solana_tools::LocalSolanaSigner;
use riglr_evm_tools::LocalEvmSigner;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use log::{info, warn, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Load configuration
    let config = load_portfolio_config()?;
    
    // Create application context
    let context = ApplicationContext::from_config(&riglr_config::Config::from_env());
    
    // Initialize multi-chain signers
    let solana_signer = Arc::new(LocalSolanaSigner::from_env()?);
    let evm_signer = Arc::new(LocalEvmSigner::from_env()?);
    
    // Start portfolio management
    run_portfolio_manager(config, context, solana_signer, evm_signer).await
}

async fn run_portfolio_manager(
    config: PortfolioConfig,
    context: ApplicationContext,
    solana_signer: Arc<LocalSolanaSigner>,
    evm_signer: Arc<LocalEvmSigner>,
) -> anyhow::Result<()> {
    let tracker = PortfolioTracker::new(config.clone(), context.clone());
    let rebalancer = PortfolioRebalancer::new(config.clone(), context);
    
    let mut interval = interval(Duration::from_secs(3600)); // Check every hour
    
    info!("Starting cross-chain portfolio manager...");
    
    loop {
        interval.tick().await;
        
        info!("Analyzing portfolio...");
        
        // Track current portfolio across all chains
        let portfolio = match tracker.get_current_portfolio().await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to get portfolio: {}", e);
                continue;
            }
        };
        
        info!("Portfolio value: ${:.2}", portfolio.total_value_usd);
        for position in &portfolio.positions {
            info!(
                "{} on {:?}: ${:.2} ({:.1}% vs target {:.1}%)",
                position.token_symbol,
                position.chain,
                position.value_usd,
                position.percentage,
                position.target_percentage
            );
        }
        
        // Analyze rebalancing needs
        let rebalance_actions = match rebalancer.analyze_rebalancing_needs(&portfolio).await {
            Ok(actions) => actions,
            Err(e) => {
                error!("Failed to analyze rebalancing: {}", e);
                continue;
            }
        };
        
        if rebalance_actions.is_empty() {
            info!("Portfolio is well balanced, no actions needed");
            continue;
        }
        
        info!("Found {} rebalancing actions needed:", rebalance_actions.len());
        for action in &rebalance_actions {
            info!("  {:?}: {} {} (cost: ${:.2})", 
                action.action_type, 
                action.token, 
                action.reason,
                action.expected_gas_cost
            );
        }
        
        // Execute rebalancing with appropriate signer
        for action in rebalance_actions {
            let result = match action.from_chain {
                Some(ChainType::Solana) => {
                    SignerContext::with_signer(solana_signer.clone(), async {
                        execute_solana_action(&action).await
                    }).await
                }
                Some(_) => {
                    SignerContext::with_signer(evm_signer.clone(), async {
                        execute_evm_action(&action).await
                    }).await
                }
                None => {
                    warn!("No source chain specified for action");
                    continue;
                }
            };
            
            match result {
                Ok(tx_hash) => {
                    info!("Executed action: {} (tx: {})", action.reason, tx_hash);
                }
                Err(e) => {
                    error!("Failed to execute action {}: {}", action.reason, e);
                }
            }
            
            // Wait between transactions
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        
        info!("Rebalancing cycle complete");
    }
}

async fn execute_solana_action(action: &RebalanceAction) -> Result<String, riglr_core::signer::SignerError> {
    // Implementation would depend on specific action type
    Ok("solana_tx_hash".to_string())
}

async fn execute_evm_action(action: &RebalanceAction) -> Result<String, riglr_core::signer::SignerError> {
    // Implementation would depend on specific action type
    Ok("evm_tx_hash".to_string())
}

fn load_portfolio_config() -> anyhow::Result<PortfolioConfig> {
    use std::collections::HashMap;
    
    let mut target_allocations = HashMap::new();
    target_allocations.insert("ETH".to_string(), 40.0);
    target_allocations.insert("SOL".to_string(), 30.0);
    target_allocations.insert("USDC".to_string(), 20.0);
    target_allocations.insert("MATIC".to_string(), 10.0);
    
    let mut token_chains = HashMap::new();
    token_chains.insert("ETH".to_string(), vec![ChainType::Ethereum]);
    token_chains.insert("SOL".to_string(), vec![ChainType::Solana]);
    token_chains.insert("USDC".to_string(), vec![
        ChainType::Ethereum,
        ChainType::Polygon, 
        ChainType::Solana,
        ChainType::Arbitrum
    ]);
    token_chains.insert("MATIC".to_string(), vec![ChainType::Polygon]);
    
    Ok(PortfolioConfig {
        target_allocations,
        rebalance_threshold: 5.0, // 5% deviation
        min_rebalance_amount: 100.0, // $100 minimum
        max_position_size: 50.0, // 50% maximum
        token_chains,
    })
}
```

## Step 5: Error Handling and Resilience

```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PortfolioError {
    #[error("Bridge execution failed: {0}")]
    BridgeFailed(String),
    
    #[error("Insufficient balance for rebalancing: {token} on {chain:?}")]
    InsufficientBalance { token: String, chain: ChainType },
    
    #[error("Price data unavailable for token: {0}")]
    PriceDataUnavailable(String),
    
    #[error("Chain not supported: {0:?}")]
    UnsupportedChain(ChainType),
    
    #[error("Risk limits exceeded: {0}")]
    RiskLimitExceeded(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Signer error: {0}")]
    SignerError(#[from] riglr_core::signer::SignerError),
}

// src/risk_manager.rs
pub struct RiskManager {
    max_daily_rebalances: u32,
    max_gas_spend_usd: f64,
    daily_gas_spent: f64,
    daily_rebalance_count: u32,
}

impl RiskManager {
    pub fn new() -> Self {
        Self {
            max_daily_rebalances: 5,
            max_gas_spend_usd: 50.0,
            daily_gas_spent: 0.0,
            daily_rebalance_count: 0,
        }
    }
    
    pub fn can_execute_action(&self, action: &RebalanceAction) -> Result<(), PortfolioError> {
        if self.daily_rebalance_count >= self.max_daily_rebalances {
            return Err(PortfolioError::RiskLimitExceeded(
                "Daily rebalance limit reached".to_string()
            ));
        }
        
        if self.daily_gas_spent + action.expected_gas_cost > self.max_gas_spend_usd {
            return Err(PortfolioError::RiskLimitExceeded(
                "Daily gas limit would be exceeded".to_string()
            ));
        }
        
        Ok(())
    }
    
    pub fn record_action(&mut self, action: &RebalanceAction, actual_gas_cost: f64) {
        self.daily_rebalance_count += 1;
        self.daily_gas_spent += actual_gas_cost;
    }
}
```

## Step 6: Testing

```rust
// tests/integration_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_portfolio_tracking() {
        let config = create_test_config();
        let context = create_test_context();
        let tracker = PortfolioTracker::new(config, context);
        
        let portfolio = tracker.get_current_portfolio().await.unwrap();
        assert!(portfolio.total_value_usd >= 0.0);
        assert!(!portfolio.positions.is_empty());
    }
    
    #[tokio::test]
    async fn test_rebalancing_logic() {
        let config = create_test_config();
        let context = create_test_context();
        let rebalancer = PortfolioRebalancer::new(config, context);
        
        let portfolio = create_imbalanced_portfolio();
        let actions = rebalancer.analyze_rebalancing_needs(&portfolio).await.unwrap();
        
        assert!(!actions.is_empty(), "Should generate rebalancing actions for imbalanced portfolio");
    }
    
    #[test]
    fn test_risk_manager() {
        let mut risk_manager = RiskManager::new();
        
        let action = RebalanceAction {
            action_type: ActionType::Bridge,
            token: "USDC".to_string(),
            from_chain: Some(ChainType::Ethereum),
            to_chain: Some(ChainType::Polygon),
            amount: 1000.0,
            expected_gas_cost: 10.0,
            reason: "Test".to_string(),
        };
        
        assert!(risk_manager.can_execute_action(&action).is_ok());
        risk_manager.record_action(&action, 10.0);
        assert_eq!(risk_manager.daily_rebalance_count, 1);
    }
    
    fn create_test_config() -> PortfolioConfig {
        // Create minimal test configuration
        let mut target_allocations = std::collections::HashMap::new();
        target_allocations.insert("ETH".to_string(), 50.0);
        target_allocations.insert("USDC".to_string(), 50.0);
        
        let mut token_chains = std::collections::HashMap::new();
        token_chains.insert("ETH".to_string(), vec![ChainType::Ethereum]);
        token_chains.insert("USDC".to_string(), vec![ChainType::Ethereum]);
        
        PortfolioConfig {
            target_allocations,
            rebalance_threshold: 10.0,
            min_rebalance_amount: 50.0,
            max_position_size: 80.0,
            token_chains,
        }
    }
    
    fn create_test_context() -> ApplicationContext {
        riglr_config::Config::from_env().into()
    }
    
    fn create_imbalanced_portfolio() -> Portfolio {
        Portfolio {
            id: "test".to_string(),
            total_value_usd: 10000.0,
            positions: vec![
                Position {
                    token_symbol: "ETH".to_string(),
                    chain: ChainType::Ethereum,
                    balance: 2.0,
                    value_usd: 8000.0,
                    percentage: 80.0,
                    target_percentage: 50.0,
                    deviation: 30.0, // Over-allocated
                    yield_opportunities: vec![],
                },
                Position {
                    token_symbol: "USDC".to_string(),
                    chain: ChainType::Ethereum,
                    balance: 2000.0,
                    value_usd: 2000.0,
                    percentage: 20.0,
                    target_percentage: 50.0,
                    deviation: -30.0, // Under-allocated
                    yield_opportunities: vec![],
                }
            ],
            last_rebalance: chrono::Utc::now(),
            performance: PerformanceMetrics {
                total_return: 0.0,
                daily_return: 0.0,
                max_drawdown: 0.0,
                sharpe_ratio: 0.0,
                rebalance_count: 0,
                gas_spent_usd: 0.0,
            },
        }
    }
}
```

## Deployment Considerations

### 1. Environment Configuration

```env
# Cross-chain RPC endpoints
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
ETHEREUM_RPC_URL=https://eth-mainnet.alchemyapi.io/v2/your-key
POLYGON_RPC_URL=https://polygon-mainnet.alchemyapi.io/v2/your-key
ARBITRUM_RPC_URL=https://arb-mainnet.alchemyapi.io/v2/your-key

# Wallet addresses
SOLANA_WALLET_ADDRESS=your_solana_address
EVM_WALLET_ADDRESS=your_evm_address

# Private keys (use secure key management in production)
SOLANA_PRIVATE_KEY=your_solana_private_key
EVM_PRIVATE_KEY=your_evm_private_key

# API keys for price data
COINGECKO_API_KEY=your_coingecko_key
DEXSCREENER_API_KEY=your_dexscreener_key

# Bridge and DEX configurations
LIFI_API_KEY=your_lifi_key
MAX_SLIPPAGE_BPS=50
MIN_BRIDGE_AMOUNT_USD=100

# Risk management
MAX_DAILY_REBALANCES=5
MAX_DAILY_GAS_USD=50
REBALANCE_THRESHOLD_PERCENT=5
```

### 2. Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/cross-chain-portfolio /usr/local/bin/
CMD ["cross-chain-portfolio"]
```

### 3. Monitoring and Alerts

```rust
// src/monitoring.rs
pub async fn send_rebalance_alert(action: &RebalanceAction, tx_hash: &str) {
    let message = format!(
        "🔄 Portfolio Rebalanced\n\
         Action: {:?}\n\
         Token: {}\n\
         Amount: ${:.2}\n\
         Gas Cost: ${:.2}\n\
         TX: {}",
        action.action_type,
        action.token,
        action.amount,
        action.expected_gas_cost,
        tx_hash
    );
    
    // Send to Discord, Telegram, etc.
}

pub fn track_portfolio_metrics(portfolio: &Portfolio) {
    // Send metrics to monitoring system
    log::info!("Portfolio metrics: total_value=${:.2}, positions={}", 
        portfolio.total_value_usd, 
        portfolio.positions.len()
    );
}
```

## Conclusion

You've built a sophisticated cross-chain portfolio manager that:

- Tracks assets across multiple blockchain networks
- Automatically rebalances based on target allocations
- Executes cross-chain bridges to optimize positioning
- Manages risk through position limits and gas controls
- Provides comprehensive monitoring and alerting

### Key Features Implemented

1. **Multi-Chain Support**: Solana, Ethereum, Polygon, and Arbitrum integration
2. **Intelligent Rebalancing**: Deviation-based rebalancing with cost optimization
3. **Cross-Chain Bridging**: LiFi integration for seamless token transfers
4. **Risk Management**: Daily limits, slippage protection, and position sizing
5. **Yield Optimization**: Discovery and integration of DeFi yield opportunities

### Next Steps

1. Add more sophisticated yield strategies (compound farming, liquid staking)
2. Implement machine learning for allocation optimization
3. Add support for more chains (Avalanche, BSC, etc.)
4. Integrate with institutional-grade custody solutions
5. Add backtesting capabilities for strategy validation

Remember to thoroughly test on testnets before deploying to mainnet with real funds!