# Building a Solana Arbitrage Bot

In this tutorial, you'll build a production-ready arbitrage bot for Solana that monitors multiple DEXs, identifies profitable opportunities, and executes trades automatically.

## Overview

Arbitrage bots profit from price discrepancies between different exchanges. When the same token pair trades at different prices on different DEXs, the bot can buy low on one exchange and sell high on another, capturing the spread as profit.

### What You'll Build

- A bot that monitors Raydium, Jupiter, and Orca for price differences
- Automatic execution of profitable arbitrage trades
- MEV protection and slippage management
- Real-time performance tracking

## Prerequisites

- Rust 1.75+
- A Solana wallet with some SOL for gas
- RPC access (free tier from Helius or QuickNode works)
- Basic understanding of DeFi and AMMs

## Project Setup

Create a new Rust project:

```bash
cargo new solana-arbitrage-bot
cd solana-arbitrage-bot
```

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
riglr-core = "0.1"
riglr-solana-tools = "0.1"
riglr-web-tools = "0.1"
riglr-macros = "0.1"
rig = "0.1"
tokio = { version = "1", features = ["full"] }
solana-sdk = "1.17"
serde = { version = "1", features = ["derive"] }
anyhow = "1"
log = "0.4"
env_logger = "0.10"
```

## Step 1: Define the Arbitrage Opportunity Structure

```rust
// src/types.rs
use solana_sdk::pubkey::Pubkey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub buy_dex: DexType,
    pub sell_dex: DexType,
    pub buy_price: f64,
    pub sell_price: f64,
    pub max_amount: f64,
    pub expected_profit: f64,
    pub profit_percentage: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DexType {
    Raydium,
    Jupiter,
    Orca,
}

impl ArbitrageOpportunity {
    pub fn calculate_profit(&self, amount: f64) -> f64 {
        let buy_cost = amount * self.buy_price;
        let sell_revenue = amount * self.sell_price;
        sell_revenue - buy_cost
    }
    
    pub fn is_profitable(&self, min_profit_bps: u16) -> bool {
        let profit_bps = (self.profit_percentage * 10000.0) as u16;
        profit_bps >= min_profit_bps
    }
}
```

## Step 2: Implement Price Monitoring

```rust
// src/monitor.rs
use riglr_solana_tools::dex::{get_raydium_price, get_jupiter_price};
use riglr_web_tools::dexscreener::get_token_pairs;
use std::collections::HashMap;
use anyhow::Result;

pub struct PriceMonitor {
    token_pairs: Vec<(Pubkey, Pubkey)>,
    min_profit_bps: u16,  // Minimum profit in basis points
}

impl PriceMonitor {
    pub fn new(token_pairs: Vec<(Pubkey, Pubkey)>, min_profit_bps: u16) -> Self {
        Self {
            token_pairs,
            min_profit_bps,
        }
    }
    
    pub async fn find_opportunities(&self) -> Result<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();
        
        for (token_a, token_b) in &self.token_pairs {
            // Get prices from different DEXs
            let prices = self.get_all_prices(token_a, token_b).await?;
            
            // Find arbitrage opportunities
            if let Some(opp) = self.analyze_prices(token_a, token_b, prices) {
                if opp.is_profitable(self.min_profit_bps) {
                    opportunities.push(opp);
                }
            }
        }
        
        Ok(opportunities)
    }
    
    async fn get_all_prices(
        &self,
        token_a: &Pubkey,
        token_b: &Pubkey,
    ) -> Result<HashMap<DexType, f64>> {
        let mut prices = HashMap::new();
        
        // Fetch prices concurrently
        let (raydium, jupiter) = tokio::join!(
            get_raydium_price(token_a.to_string(), token_b.to_string()),
            get_jupiter_price(token_a.to_string(), token_b.to_string()),
        );
        
        if let Ok(price) = raydium {
            prices.insert(DexType::Raydium, price);
        }
        if let Ok(price) = jupiter {
            prices.insert(DexType::Jupiter, price);
        }
        
        Ok(prices)
    }
    
    fn analyze_prices(
        &self,
        token_a: &Pubkey,
        token_b: &Pubkey,
        prices: HashMap<DexType, f64>,
    ) -> Option<ArbitrageOpportunity> {
        if prices.len() < 2 {
            return None;
        }
        
        // Find min and max prices
        let mut min_price = f64::MAX;
        let mut max_price = f64::MIN;
        let mut buy_dex = DexType::Raydium;
        let mut sell_dex = DexType::Raydium;
        
        for (dex, price) in &prices {
            if *price < min_price {
                min_price = *price;
                buy_dex = *dex;
            }
            if *price > max_price {
                max_price = *price;
                sell_dex = *dex;
            }
        }
        
        // Calculate profit
        let profit_percentage = (max_price - min_price) / min_price;
        
        if buy_dex != sell_dex && profit_percentage > 0.0 {
            Some(ArbitrageOpportunity {
                token_a: *token_a,
                token_b: *token_b,
                buy_dex,
                sell_dex,
                buy_price: min_price,
                sell_price: max_price,
                max_amount: 100.0, // Calculate based on liquidity
                expected_profit: 0.0, // Will be calculated based on amount
                profit_percentage,
            })
        } else {
            None
        }
    }
}
```

## Step 3: Create the Arbitrage Executor

```rust
// src/executor.rs
use riglr_core::signer::SignerContext;
use riglr_solana_tools::{swap_tokens_jupiter, swap_tokens_raydium};
use riglr_macros::tool;
use riglr_core::ToolError;
use anyhow::Result;

pub struct ArbitrageExecutor {
    slippage_bps: u16,
    gas_estimate: f64,
}

impl ArbitrageExecutor {
    pub fn new(slippage_bps: u16) -> Self {
        Self {
            slippage_bps,
            gas_estimate: 0.005, // 0.005 SOL estimated gas
        }
    }
    
    pub async fn execute(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> Result<ExecutionResult> {
        // Verify profitability after gas
        let net_profit = opportunity.expected_profit - self.gas_estimate;
        if net_profit <= 0.0 {
            return Ok(ExecutionResult::Skipped {
                reason: "Not profitable after gas".to_string(),
            });
        }
        
        // Get the signer from context
        let signer = SignerContext::current().await?;
        
        // Execute buy transaction
        let buy_tx = self.execute_buy(opportunity).await?;
        
        // Execute sell transaction
        let sell_tx = self.execute_sell(opportunity).await?;
        
        Ok(ExecutionResult::Success {
            buy_tx,
            sell_tx,
            actual_profit: net_profit,
        })
    }
    
    async fn execute_buy(&self, opp: &ArbitrageOpportunity) -> Result<String> {
        match opp.buy_dex {
            DexType::Raydium => {
                swap_tokens_raydium(
                    opp.token_b.to_string(),
                    opp.token_a.to_string(),
                    opp.max_amount,
                    self.slippage_bps,
                ).await.map_err(|e| anyhow::anyhow!(e))
            }
            DexType::Jupiter => {
                swap_tokens_jupiter(
                    opp.token_b.to_string(),
                    opp.token_a.to_string(),
                    opp.max_amount,
                    self.slippage_bps,
                ).await.map_err(|e| anyhow::anyhow!(e))
            }
            _ => Err(anyhow::anyhow!("DEX not supported")),
        }
    }
    
    async fn execute_sell(&self, opp: &ArbitrageOpportunity) -> Result<String> {
        match opp.sell_dex {
            DexType::Raydium => {
                swap_tokens_raydium(
                    opp.token_a.to_string(),
                    opp.token_b.to_string(),
                    opp.max_amount,
                    self.slippage_bps,
                ).await.map_err(|e| anyhow::anyhow!(e))
            }
            DexType::Jupiter => {
                swap_tokens_jupiter(
                    opp.token_a.to_string(),
                    opp.token_b.to_string(),
                    opp.max_amount,
                    self.slippage_bps,
                ).await.map_err(|e| anyhow::anyhow!(e))
            }
            _ => Err(anyhow::anyhow!("DEX not supported")),
        }
    }
}

#[derive(Debug)]
pub enum ExecutionResult {
    Success {
        buy_tx: String,
        sell_tx: String,
        actual_profit: f64,
    },
    Skipped {
        reason: String,
    },
    Failed {
        error: String,
    },
}
```

## Step 4: Build the Main Bot Loop

```rust
// src/main.rs
mod types;
mod monitor;
mod executor;

use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use log::{info, warn, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Configuration
    let config = Config::from_env()?;
    
    // Initialize signer
    let signer = Arc::new(LocalSolanaSigner::from_env()?);
    
    // Token pairs to monitor (SOL/USDC, RAY/USDC, etc.)
    let token_pairs = vec![
        (
            solana_sdk::pubkey!("So11111111111111111111111111111111111111112"),  // SOL
            solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // USDC
        ),
        // Add more pairs
    ];
    
    // Initialize components
    let monitor = PriceMonitor::new(token_pairs, config.min_profit_bps);
    let executor = ArbitrageExecutor::new(config.slippage_bps);
    
    // Run bot within signer context
    SignerContext::with_signer(signer, async move {
        run_bot(monitor, executor, config).await
    }).await
}

async fn run_bot(
    monitor: PriceMonitor,
    executor: ArbitrageExecutor,
    config: Config,
) -> anyhow::Result<()> {
    let mut interval = interval(Duration::from_secs(config.poll_interval_secs));
    let mut total_profit = 0.0;
    let mut trades_executed = 0;
    
    info!("Starting arbitrage bot...");
    
    loop {
        interval.tick().await;
        
        // Find opportunities
        match monitor.find_opportunities().await {
            Ok(opportunities) => {
                info!("Found {} opportunities", opportunities.len());
                
                for opp in opportunities {
                    info!(
                        "Opportunity: Buy {} on {:?} at {}, Sell on {:?} at {} ({}% profit)",
                        opp.token_a,
                        opp.buy_dex,
                        opp.buy_price,
                        opp.sell_dex,
                        opp.sell_price,
                        opp.profit_percentage * 100.0
                    );
                    
                    // Execute arbitrage
                    match executor.execute(&opp).await {
                        Ok(ExecutionResult::Success { buy_tx, sell_tx, actual_profit }) => {
                            info!(
                                "Arbitrage executed! Buy: {}, Sell: {}, Profit: {} SOL",
                                buy_tx, sell_tx, actual_profit
                            );
                            total_profit += actual_profit;
                            trades_executed += 1;
                        }
                        Ok(ExecutionResult::Skipped { reason }) => {
                            info!("Skipped: {}", reason);
                        }
                        Ok(ExecutionResult::Failed { error }) => {
                            error!("Execution failed: {}", error);
                        }
                        Err(e) => {
                            error!("Execution error: {}", e);
                        }
                    }
                }
                
                info!(
                    "Stats: {} trades executed, Total profit: {} SOL",
                    trades_executed, total_profit
                );
            }
            Err(e) => {
                warn!("Error finding opportunities: {}", e);
            }
        }
    }
}

#[derive(Clone)]
struct Config {
    min_profit_bps: u16,
    slippage_bps: u16,
    poll_interval_secs: u64,
}

impl Config {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            min_profit_bps: std::env::var("MIN_PROFIT_BPS")
                .unwrap_or_else(|_| "50".to_string())
                .parse()?,
            slippage_bps: std::env::var("SLIPPAGE_BPS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            poll_interval_secs: std::env::var("POLL_INTERVAL_SECS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()?,
        })
    }
}
```

## Step 5: Testing

Create comprehensive tests:

```rust
// src/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arbitrage_calculation() {
        let opp = ArbitrageOpportunity {
            token_a: Pubkey::new_unique(),
            token_b: Pubkey::new_unique(),
            buy_dex: DexType::Raydium,
            sell_dex: DexType::Jupiter,
            buy_price: 1.0,
            sell_price: 1.05,
            max_amount: 100.0,
            expected_profit: 5.0,
            profit_percentage: 0.05,
        };
        
        assert_eq!(opp.calculate_profit(100.0), 5.0);
        assert!(opp.is_profitable(400)); // 400 bps = 4%
    }
    
    #[tokio::test]
    async fn test_price_monitor() {
        let monitor = PriceMonitor::new(vec![], 50);
        let opportunities = monitor.find_opportunities().await.unwrap();
        assert_eq!(opportunities.len(), 0);
    }
}
```

## Deployment

### 1. Environment Variables

Create a `.env` file:

```env
# Solana Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_PRIVATE_KEY=your_private_key_here

# Bot Configuration
MIN_PROFIT_BPS=50        # 0.5% minimum profit
SLIPPAGE_BPS=100         # 1% slippage tolerance
POLL_INTERVAL_SECS=5     # Check every 5 seconds

# Monitoring
LOG_LEVEL=info
DISCORD_WEBHOOK=https://discord.com/api/webhooks/...
```

### 2. Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/solana-arbitrage-bot /usr/local/bin/
ENV RUST_LOG=info
CMD ["solana-arbitrage-bot"]
```

### 3. Monitoring

Add monitoring and alerts:

```rust
// src/monitoring.rs
pub async fn send_discord_alert(message: &str) {
    // Send alerts for successful trades or errors
}

pub fn track_metrics(profit: f64, gas_used: f64) {
    // Track performance metrics
}
```

## Advanced Features

### MEV Protection

Implement flashloan-style atomic arbitrage:

```rust
// Execute both trades in a single transaction
pub async fn atomic_arbitrage(
    buy_instruction: Instruction,
    sell_instruction: Instruction,
) -> Result<String> {
    let transaction = Transaction::new_signed_with_payer(
        &[buy_instruction, sell_instruction],
        Some(&payer),
        &[&payer],
        recent_blockhash,
    );
    
    // Send with high priority fee for MEV protection
    send_transaction_with_priority(transaction).await
}
```

### Dynamic Position Sizing

Calculate optimal trade size based on liquidity:

```rust
pub fn calculate_optimal_size(
    liquidity: f64,
    price_impact: f64,
    max_position: f64,
) -> f64 {
    // Optimize for maximum profit considering price impact
    let optimal = (liquidity * 0.01).min(max_position);
    optimal * (1.0 - price_impact)
}
```

## Conclusion

You've built a functional arbitrage bot for Solana! This bot:

- Monitors multiple DEXs for price discrepancies
- Executes profitable arbitrage trades automatically
- Manages risk through slippage protection
- Tracks performance metrics

### Next Steps

1. Add more DEX integrations (Orca, Saber, etc.)
2. Implement more sophisticated strategies (triangular arbitrage)
3. Add machine learning for opportunity prediction
4. Deploy to a cloud provider with low-latency RPC access

Remember to always test thoroughly on devnet before deploying to mainnet!