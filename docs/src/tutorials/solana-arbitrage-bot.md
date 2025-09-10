# Building a Solana Arbitrage Bot

In this tutorial, you'll build a production-ready arbitrage bot for Solana that monitors multiple DEXs, identifies profitable opportunities, and executes trades automatically.

## Overview

Arbitrage bots profit from price discrepancies between different exchanges. When the same token pair trades at different prices on different DEXs, the bot can buy low on one exchange and sell high on another, capturing the spread as profit.

### What You'll Build

- A bot that monitors Jupiter aggregator for optimal routes
- Price comparison across multiple DEXs (Raydium, Orca, etc.) via Jupiter
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
riglr-core = "0.3"
riglr-config = "0.3"
riglr-solana-tools = "0.3"
riglr-web-tools = "0.3"
riglr-macros = "0.2"
tokio = { version = "1", features = ["full"] }
solana-sdk = "1.17"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
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
    pub route_a_to_b: SwapRoute,
    pub route_b_to_a: SwapRoute,
    pub initial_amount: f64,
    pub expected_output: f64,
    pub expected_profit: f64,
    pub profit_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRoute {
    pub input_mint: String,
    pub output_mint: String,
    pub price: f64,
    pub price_impact: f64,
    pub dexes: Vec<String>, // DEXs used in the route
}

impl ArbitrageOpportunity {
    pub fn calculate_profit(&self, amount: f64) -> f64 {
        // Calculate round-trip profit after fees
        let output_after_first = amount / self.route_a_to_b.price;
        let final_output = output_after_first / self.route_b_to_a.price;
        final_output - amount
    }
    
    pub fn is_profitable(&self, min_profit_bps: u16, gas_cost_sol: f64) -> bool {
        let profit_bps = (self.profit_percentage * 10000.0) as u16;
        let net_profit = self.expected_profit - gas_cost_sol;
        profit_bps >= min_profit_bps && net_profit > 0.0
    }
}
```

## Step 2: Implement Price Monitoring

```rust
// src/monitor.rs
use crate::types::{ArbitrageOpportunity, SwapRoute};
use riglr_core::provider::ApplicationContext;
use riglr_solana_tools::swap::{get_jupiter_quote, get_token_price};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use anyhow::Result;
use tracing::{info, debug};

pub struct PriceMonitor {
    token_pairs: Vec<(Pubkey, Pubkey)>,
    min_profit_bps: u16,
    check_amount_sol: f64,
}

impl PriceMonitor {
    pub fn new(
        token_pairs: Vec<(Pubkey, Pubkey)>, 
        min_profit_bps: u16,
        check_amount_sol: f64
    ) -> Self {
        Self {
            token_pairs,
            min_profit_bps,
            check_amount_sol,
        }
    }
    
    pub async fn find_opportunities(
        &self,
        context: &ApplicationContext
    ) -> Result<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();
        
        for (token_a, token_b) in &self.token_pairs {
            debug!("Checking pair: {} <-> {}", token_a, token_b);
            
            // Check for triangular arbitrage opportunity
            if let Some(opp) = self.check_arbitrage_opportunity(
                token_a, 
                token_b, 
                context
            ).await? {
                if opp.is_profitable(self.min_profit_bps, 0.005) {
                    info!(
                        "Found opportunity: {:.2}% profit on {} <-> {}",
                        opp.profit_percentage * 100.0,
                        token_a,
                        token_b
                    );
                    opportunities.push(opp);
                }
            }
        }
        
        Ok(opportunities)
    }
    
    async fn check_arbitrage_opportunity(
        &self,
        token_a: &Pubkey,
        token_b: &Pubkey,
        context: &ApplicationContext,
    ) -> Result<Option<ArbitrageOpportunity>> {
        // Convert amount to lamports (1 SOL = 1e9 lamports)
        let amount_lamports = (self.check_amount_sol * 1e9) as u64;
        
        // Get quote for A -> B
        let quote_a_to_b = get_jupiter_quote(
            token_a.to_string(),
            token_b.to_string(),
            amount_lamports,
            50, // 0.5% slippage
            false, // allow multi-hop routes
            None,
            context,
        ).await.ok()?;
        
        // Get quote for B -> A using the output amount
        let quote_b_to_a = get_jupiter_quote(
            token_b.to_string(),
            token_a.to_string(),
            quote_a_to_b.out_amount,
            50,
            false,
            None,
            context,
        ).await.ok()?;
        
        // Calculate profit
        let final_amount = quote_b_to_a.out_amount as f64 / 1e9;
        let profit = final_amount - self.check_amount_sol;
        let profit_percentage = profit / self.check_amount_sol;
        
        // Only return if profitable
        if profit > 0.0 {
            // Extract DEX names from route plan
            let dexes_a_to_b: Vec<String> = quote_a_to_b.route_plan
                .iter()
                .filter_map(|step| step.swap_info.as_ref())
                .map(|info| info.label.clone().unwrap_or_default())
                .collect();
                
            let dexes_b_to_a: Vec<String> = quote_b_to_a.route_plan
                .iter()
                .filter_map(|step| step.swap_info.as_ref())
                .map(|info| info.label.clone().unwrap_or_default())
                .collect();
            
            Some(ArbitrageOpportunity {
                token_a: *token_a,
                token_b: *token_b,
                route_a_to_b: SwapRoute {
                    input_mint: token_a.to_string(),
                    output_mint: token_b.to_string(),
                    price: quote_a_to_b.out_amount as f64 / quote_a_to_b.in_amount as f64,
                    price_impact: quote_a_to_b.price_impact_pct,
                    dexes: dexes_a_to_b,
                },
                route_b_to_a: SwapRoute {
                    input_mint: token_b.to_string(),
                    output_mint: token_a.to_string(),
                    price: quote_b_to_a.out_amount as f64 / quote_b_to_a.in_amount as f64,
                    price_impact: quote_b_to_a.price_impact_pct,
                    dexes: dexes_b_to_a,
                },
                initial_amount: self.check_amount_sol,
                expected_output: final_amount,
                expected_profit: profit,
                profit_percentage,
            })
        } else {
            None
        }
    }
    
    pub async fn get_current_prices(
        &self,
        context: &ApplicationContext
    ) -> Result<HashMap<String, f64>> {
        let mut prices = HashMap::new();
        
        // Get prices for monitoring
        for (token_a, token_b) in &self.token_pairs {
            let price = get_token_price(
                token_a.to_string(),
                token_b.to_string(),
                context,
            ).await?;
            
            let pair_key = format!("{}/{}", token_a, token_b);
            prices.insert(pair_key, price.price);
        }
        
        Ok(prices)
    }
}
```

## Step 3: Create the Arbitrage Executor

```rust
// src/executor.rs
use crate::types::ArbitrageOpportunity;
use riglr_core::{signer::SignerContext, provider::ApplicationContext, ToolError};
use riglr_solana_tools::swap::perform_jupiter_swap;
use anyhow::Result;
use tracing::{info, warn, error};

pub struct ArbitrageExecutor {
    slippage_bps: u16,
    max_gas_sol: f64,
}

impl ArbitrageExecutor {
    pub fn new(slippage_bps: u16, max_gas_sol: f64) -> Self {
        Self {
            slippage_bps,
            max_gas_sol,
        }
    }
    
    pub async fn execute(
        &self,
        opportunity: &ArbitrageOpportunity,
        context: &ApplicationContext,
    ) -> Result<ExecutionResult> {
        // Final profitability check
        if !opportunity.is_profitable(25, self.max_gas_sol) {
            return Ok(ExecutionResult::Skipped {
                reason: "Not profitable after final check".to_string(),
            });
        }
        
        // Verify we have a signer
        if !SignerContext::is_available().await {
            return Ok(ExecutionResult::Skipped {
                reason: "No signer available".to_string(),
            });
        }
        
        info!(
            "Executing arbitrage: {} SOL -> {} -> {} (expected profit: {:.4} SOL)",
            opportunity.initial_amount,
            opportunity.token_b,
            opportunity.token_a,
            opportunity.expected_profit
        );
        
        // Execute first swap: Token A -> Token B
        let amount_lamports = (opportunity.initial_amount * 1e9) as u64;
        
        let swap1_result = perform_jupiter_swap(
            opportunity.route_a_to_b.input_mint.clone(),
            opportunity.route_a_to_b.output_mint.clone(),
            amount_lamports,
            self.slippage_bps,
            false, // allow multi-hop
            None,
            context,
        ).await;
        
        let swap1_tx = match swap1_result {
            Ok(result) => {
                info!("First swap completed: {}", result.signature);
                result
            }
            Err(e) => {
                error!("First swap failed: {}", e);
                return Ok(ExecutionResult::Failed {
                    error: format!("First swap failed: {}", e),
                });
            }
        };
        
        // Execute second swap: Token B -> Token A
        // Use the actual output from the first swap
        let swap2_result = perform_jupiter_swap(
            opportunity.route_b_to_a.input_mint.clone(),
            opportunity.route_b_to_a.output_mint.clone(),
            swap1_tx.out_amount,
            self.slippage_bps,
            false,
            None,
            context,
        ).await;
        
        let swap2_tx = match swap2_result {
            Ok(result) => {
                info!("Second swap completed: {}", result.signature);
                result
            }
            Err(e) => {
                error!("Second swap failed: {}", e);
                return Ok(ExecutionResult::Failed {
                    error: format!("Second swap failed: {}", e),
                });
            }
        };
        
        // Calculate actual profit
        let final_amount_sol = swap2_tx.out_amount as f64 / 1e9;
        let actual_profit = final_amount_sol - opportunity.initial_amount;
        
        info!(
            "Arbitrage completed! Initial: {} SOL, Final: {} SOL, Profit: {} SOL",
            opportunity.initial_amount,
            final_amount_sol,
            actual_profit
        );
        
        Ok(ExecutionResult::Success {
            swap1_signature: swap1_tx.signature,
            swap2_signature: swap2_tx.signature,
            initial_amount: opportunity.initial_amount,
            final_amount: final_amount_sol,
            actual_profit,
        })
    }
}

#[derive(Debug)]
pub enum ExecutionResult {
    Success {
        swap1_signature: String,
        swap2_signature: String,
        initial_amount: f64,
        final_amount: f64,
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

use riglr_config::Config;
use riglr_core::{provider::ApplicationContext, signer::SignerContext};
use riglr_solana_tools::signer::LocalSolanaSigner;
use monitor::PriceMonitor;
use executor::{ArbitrageExecutor, ExecutionResult};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error};
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("solana_arbitrage_bot=info".parse()?)
        )
        .init();
    
    // Load configuration
    let config = Config::from_env();
    
    // Create application context with extensions
    let mut app_context = ApplicationContext::from_config(&config);
    
    // Add API clients extension for Jupiter
    let api_clients = Arc::new(riglr_solana_tools::clients::ApiClients::new(&config));
    app_context.set_extension(api_clients);
    
    // Initialize Solana signer
    let signer = Arc::new(LocalSolanaSigner::from_env()?);
    
    // Define token pairs to monitor for arbitrage
    // These are popular pairs with good liquidity
    let token_pairs = vec![
        (
            Pubkey::from_str("So11111111111111111111111111111111111111112")?, // SOL
            Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?, // USDC
        ),
        (
            Pubkey::from_str("So11111111111111111111111111111111111111112")?, // SOL
            Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")?, // USDT
        ),
        (
            Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?, // USDC
            Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")?, // USDT
        ),
        // Add more pairs as needed
    ];
    
    // Bot configuration
    let min_profit_bps = 30; // 0.3% minimum profit
    let slippage_bps = 50; // 0.5% slippage
    let check_amount_sol = 10.0; // Check arbitrage for 10 SOL
    let max_gas_sol = 0.01; // Maximum 0.01 SOL for gas
    
    // Initialize components
    let monitor = PriceMonitor::new(token_pairs, min_profit_bps, check_amount_sol);
    let executor = ArbitrageExecutor::new(slippage_bps, max_gas_sol);
    
    info!("Starting Solana arbitrage bot...");
    info!("Monitoring {} token pairs", monitor.token_pairs.len());
    info!("Min profit: {} bps, Slippage: {} bps", min_profit_bps, slippage_bps);
    
    // Run bot within signer context
    SignerContext::with_signer(signer, async move {
        run_bot(monitor, executor, app_context).await
    }).await
}

async fn run_bot(
    monitor: PriceMonitor,
    executor: ArbitrageExecutor,
    app_context: ApplicationContext,
) -> anyhow::Result<()> {
    let poll_interval_secs = std::env::var("POLL_INTERVAL_SECS")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);
    
    let mut interval = interval(Duration::from_secs(poll_interval_secs));
    let mut total_profit = 0.0;
    let mut trades_executed = 0;
    let mut opportunities_found = 0;
    
    loop {
        interval.tick().await;
        
        // Get current prices for monitoring
        match monitor.get_current_prices(&app_context).await {
            Ok(prices) => {
                for (pair, price) in prices.iter() {
                    info!("{}: {:.6}", pair, price);
                }
            }
            Err(e) => {
                warn!("Failed to get prices: {}", e);
            }
        }
        
        // Find arbitrage opportunities
        match monitor.find_opportunities(&app_context).await {
            Ok(opportunities) => {
                if !opportunities.is_empty() {
                    opportunities_found += opportunities.len();
                    info!(
                        "Found {} arbitrage opportunities (total: {})",
                        opportunities.len(),
                        opportunities_found
                    );
                    
                    for opp in opportunities {
                        info!(
                            "Opportunity: {} <-> {} via {:?} and {:?}",
                            opp.token_a,
                            opp.token_b,
                            opp.route_a_to_b.dexes,
                            opp.route_b_to_a.dexes
                        );
                        info!(
                            "Expected: {} SOL -> {:.4} SOL (profit: {:.4} SOL, {:.2}%)",
                            opp.initial_amount,
                            opp.expected_output,
                            opp.expected_profit,
                            opp.profit_percentage * 100.0
                        );
                        
                        // Execute arbitrage if in production mode
                        if std::env::var("EXECUTE_TRADES").unwrap_or_default() == "true" {
                            match executor.execute(&opp, &app_context).await {
                                Ok(ExecutionResult::Success { 
                                    swap1_signature,
                                    swap2_signature,
                                    initial_amount,
                                    final_amount,
                                    actual_profit 
                                }) => {
                                    info!(
                                        "✅ Arbitrage executed successfully!"
                                    );
                                    info!(
                                        "Swap 1: https://solscan.io/tx/{}",
                                        swap1_signature
                                    );
                                    info!(
                                        "Swap 2: https://solscan.io/tx/{}",
                                        swap2_signature
                                    );
                                    info!(
                                        "Result: {} SOL -> {} SOL (profit: {} SOL)",
                                        initial_amount,
                                        final_amount,
                                        actual_profit
                                    );
                                    
                                    total_profit += actual_profit;
                                    trades_executed += 1;
                                }
                                Ok(ExecutionResult::Skipped { reason }) => {
                                    info!("⏭️  Skipped: {}", reason);
                                }
                                Ok(ExecutionResult::Failed { error }) => {
                                    error!("❌ Execution failed: {}", error);
                                }
                                Err(e) => {
                                    error!("❌ Unexpected error: {}", e);
                                }
                            }
                        } else {
                            info!("📊 Simulation mode - trade not executed");
                        }
                    }
                }
                
                // Print statistics
                if opportunities_found > 0 {
                    info!(
                        "📈 Stats: {} opportunities found, {} trades executed, Total profit: {:.4} SOL",
                        opportunities_found,
                        trades_executed,
                        total_profit
                    );
                }
            }
            Err(e) => {
                error!("Error finding opportunities: {}", e);
            }
        }
    }
}
```

## Step 5: Configuration and Environment

Create a `.env` file for your bot:

```env
# Required: Redis for caching
REDIS_URL=redis://localhost:6379

# Solana Network
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
# For better performance, use a dedicated RPC:
# SOLANA_RPC_URL=https://rpc.helius.xyz/?api-key=YOUR_KEY

# Wallet Configuration
SOLANA_PRIVATE_KEY=your_base58_private_key_here

# Bot Settings
POLL_INTERVAL_SECS=10          # Check every 10 seconds
EXECUTE_TRADES=false           # Set to true to execute real trades

# Logging
RUST_LOG=info,solana_arbitrage_bot=debug

# Optional: Performance monitoring
METRICS_ENABLED=true
METRICS_PORT=9090
```

## Step 6: Testing

Create comprehensive tests:

```rust
// src/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArbitrageOpportunity, SwapRoute};
    use solana_sdk::pubkey::Pubkey;
    
    #[test]
    fn test_arbitrage_calculation() {
        let opp = ArbitrageOpportunity {
            token_a: Pubkey::new_unique(),
            token_b: Pubkey::new_unique(),
            route_a_to_b: SwapRoute {
                input_mint: "SOL".to_string(),
                output_mint: "USDC".to_string(),
                price: 100.0,
                price_impact: 0.001,
                dexes: vec!["Raydium".to_string()],
            },
            route_b_to_a: SwapRoute {
                input_mint: "USDC".to_string(),
                output_mint: "SOL".to_string(),
                price: 0.0098, // Slightly better price creates arbitrage
                price_impact: 0.001,
                dexes: vec!["Orca".to_string()],
            },
            initial_amount: 1.0,
            expected_output: 1.02,
            expected_profit: 0.02,
            profit_percentage: 0.02,
        };
        
        assert!(opp.is_profitable(100, 0.005)); // 1% min profit, 0.005 SOL gas
        assert_eq!(opp.calculate_profit(1.0), 0.02);
    }
    
    #[tokio::test]
    async fn test_price_monitor_creation() {
        let monitor = PriceMonitor::new(vec![], 50, 10.0);
        assert_eq!(monitor.min_profit_bps, 50);
        assert_eq!(monitor.check_amount_sol, 10.0);
    }
}
```

## Deployment

### Docker Deployment

Create a `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*
    
COPY --from=builder /app/target/release/solana-arbitrage-bot /usr/local/bin/
ENV RUST_LOG=info
CMD ["solana-arbitrage-bot"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
      
  arbitrage-bot:
    build: .
    depends_on:
      - redis
    env_file:
      - .env
    environment:
      - REDIS_URL=redis://redis:6379
    restart: unless-stopped
    
volumes:
  redis_data:
```

## Advanced Features

### MEV Protection

Use priority fees and Jito bundles for MEV protection:

```rust
// Add to executor.rs
use riglr_solana_tools::jito::{send_bundle, create_tip_instruction};

pub async fn execute_with_mev_protection(
    &self,
    opportunity: &ArbitrageOpportunity,
    context: &ApplicationContext,
) -> Result<ExecutionResult> {
    // Create both swap instructions
    let swap1_ix = create_swap_instruction(&opportunity.route_a_to_b)?;
    let swap2_ix = create_swap_instruction(&opportunity.route_b_to_a)?;
    
    // Add tip for Jito bundle
    let tip_ix = create_tip_instruction(0.001)?; // 0.001 SOL tip
    
    // Send as atomic bundle
    let bundle_id = send_bundle(
        vec![swap1_ix, swap2_ix, tip_ix],
        context
    ).await?;
    
    info!("Bundle sent with MEV protection: {}", bundle_id);
    // ... rest of implementation
}
```

### Multi-Token Arbitrage

Extend to triangular and more complex arbitrage paths:

```rust
pub async fn find_triangular_arbitrage(
    token_a: &Pubkey,
    token_b: &Pubkey,
    token_c: &Pubkey,
    context: &ApplicationContext,
) -> Result<Option<TriangularOpportunity>> {
    // Check A -> B -> C -> A
    let quote_a_b = get_jupiter_quote(/*...*/);
    let quote_b_c = get_jupiter_quote(/*...*/);
    let quote_c_a = get_jupiter_quote(/*...*/);
    
    // Calculate round-trip profit
    // ... implementation
}
```

### Performance Monitoring

Add metrics collection:

```rust
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref OPPORTUNITIES_FOUND: Counter = 
        Counter::new("arbitrage_opportunities_total", "Total opportunities found").unwrap();
    static ref TRADES_EXECUTED: Counter = 
        Counter::new("arbitrage_trades_total", "Total trades executed").unwrap();
    static ref PROFIT_HISTOGRAM: Histogram = 
        Histogram::new("arbitrage_profit_sol", "Profit distribution in SOL").unwrap();
}

// In your execution code:
OPPORTUNITIES_FOUND.inc();
TRADES_EXECUTED.inc();
PROFIT_HISTOGRAM.observe(actual_profit);
```

## Safety and Best Practices

1. **Start Small**: Test with small amounts on mainnet
2. **Monitor Gas Costs**: Ensure profits exceed transaction fees
3. **Handle Failures**: Implement proper error recovery
4. **Rate Limiting**: Respect RPC rate limits
5. **Slippage Protection**: Use appropriate slippage settings
6. **Simulation First**: Always simulate trades before executing

## Conclusion

You've built a functional arbitrage bot for Solana that:
- Monitors Jupiter aggregator for arbitrage opportunities
- Executes profitable trades automatically
- Handles errors and manages risk
- Provides detailed logging and monitoring

### Next Steps

1. **Optimize Performance**: Use WebSocket subscriptions for real-time data
2. **Add More Strategies**: Implement statistical arbitrage, cross-DEX arbitrage
3. **Machine Learning**: Predict profitable opportunities using ML
4. **Scale Operations**: Deploy multiple bots for different token pairs
5. **Advanced MEV**: Integrate with Jito for advanced MEV strategies

Remember to thoroughly test on devnet and start with small amounts on mainnet!