//! Trading Bot Example - Advanced automated trading agent
//!
//! This specialized binary demonstrates how to build a sophisticated trading bot
//! using the riglr ecosystem. It includes risk management, portfolio tracking,
//! and automated decision making.

use anyhow::Result;
use clap::Parser;
use rig_core::{Agent, Provider};
use std::time::Duration;
use tokio::time;
use tracing::{info, warn, error};

// Import trading-specific tools
{% if primary-chain == "solana" or primary-chain == "both" -%}
use riglr_solana_tools::{get_sol_balance, transfer_sol, get_jupiter_quote, perform_jupiter_swap};
{% endif %}
{% if primary-chain == "ethereum" or primary-chain == "both" -%}
use riglr_evm_tools::{get_eth_balance, transfer_eth, swap_on_uniswap};
{% endif %}
{% if include-web-tools -%}
use riglr_web_tools::{
    dexscreener::{get_token_info, search_tokens, analyze_token_market},
    twitter::analyze_crypto_sentiment,
    news::get_crypto_news,
    price::get_token_price,
};
{% endif %}

#[derive(Debug, Parser)]
#[command(name = "{{project-name}}-trading-bot")]
#[command(about = "Advanced cryptocurrency trading bot")]
pub struct TradingConfig {
    /// Trading mode (paper, live)
    #[arg(long, default_value = "paper")]
    mode: String,

    /// Assets to trade (comma-separated)
    #[arg(long, default_value = "SOL,ETH,BTC")]
    assets: String,

    /// Maximum trade size in USD
    #[arg(long, default_value = "100.0")]
    max_trade_size: f64,

    /// Stop loss percentage
    #[arg(long, default_value = "5.0")]
    stop_loss: f64,

    /// Take profit percentage  
    #[arg(long, default_value = "10.0")]
    take_profit: f64,

    /// Trading interval in minutes
    #[arg(long, default_value = "15")]
    interval: u64,

    /// Minimum confidence score for trades (0.0-1.0)
    #[arg(long, default_value = "0.7")]
    min_confidence: f64,
}

struct TradingBot {
    config: TradingConfig,
    agent: Agent<Provider>,
    assets: Vec<String>,
    active_positions: std::collections::HashMap<String, Position>,
}

#[derive(Debug, Clone)]
struct Position {
    asset: String,
    entry_price: f64,
    quantity: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
    stop_loss: f64,
    take_profit: f64,
    unrealized_pnl: f64,
}

impl TradingBot {
    pub async fn new(config: TradingConfig) -> Result<Self> {
        info!("Initializing trading bot with mode: {}", config.mode);

        if config.mode == "live" {
            warn!("⚠️  LIVE TRADING MODE ENABLED - Real money at risk!");
        } else {
            info!("📋 Paper trading mode - No real transactions");
        }

        // Parse assets list
        let assets: Vec<String> = config.assets
            .split(',')
            .map(|s| s.trim().to_uppercase().to_string())
            .collect();

        info!("Trading assets: {:?}", assets);

        // Initialize AI agent with trading-focused prompt
        let provider = Provider::new("your-provider-config")?;
        let mut agent = Agent::builder(&provider)
            .preamble(Self::trading_system_prompt(&config))
            .temperature(0.3); // Lower temperature for more consistent trading decisions

        // Add blockchain tools
        {% if primary-chain == "solana" or primary-chain == "both" -%}
        agent = agent
            .tool(get_sol_balance)
            .tool(transfer_sol)
            .tool(get_jupiter_quote)
            .tool(perform_jupiter_swap);
        {% endif %}

        {% if primary-chain == "ethereum" or primary-chain == "both" -%}
        agent = agent
            .tool(get_eth_balance)
            .tool(transfer_eth)
            .tool(swap_on_uniswap);
        {% endif %}

        {% if include-web-tools -%}
        // Add market data and sentiment analysis tools
        agent = agent
            .tool(get_token_info)
            .tool(search_tokens)
            .tool(analyze_token_market)
            .tool(analyze_crypto_sentiment)
            .tool(get_crypto_news)
            .tool(get_token_price); // Add the new price fetching tool
        {% endif %}

        let agent = agent.build();

        Ok(Self {
            config,
            agent,
            assets,
            active_positions: std::collections::HashMap::new(),
        })
    }

    fn trading_system_prompt(config: &TradingConfig) -> String {
        format!(
            r#"You are an advanced cryptocurrency trading bot with the following configuration:

TRADING PARAMETERS:
- Mode: {} ({}REAL MONEY)
- Maximum trade size: ${:.2} USD
- Stop loss: {:.1}%
- Take profit: {:.1}%
- Minimum confidence: {:.1}%

CAPABILITIES:
- Real-time market data analysis
- Social sentiment monitoring
- News impact assessment
- Risk management enforcement
- Portfolio optimization

TRADING RULES:
1. NEVER exceed the maximum trade size
2. ALWAYS set stop-loss and take-profit levels
3. Require minimum confidence score before trading
4. Consider market sentiment and news impact
5. Maintain proper position sizing
6. Monitor correlations between assets
7. Avoid overexposure to any single asset

RISK MANAGEMENT:
- Maximum portfolio risk: 2% per trade
- Daily loss limit: 5% of portfolio
- Maximum open positions: 3 simultaneous
- Required confirmation for trades > $500

DECISION PROCESS:
1. Analyze current market conditions
2. Review sentiment and news
3. Calculate risk/reward ratio
4. Determine position size
5. Set stop-loss and take-profit
6. Execute trade if confidence > {:.1}%
7. Monitor and manage position

Remember: Capital preservation is priority #1. Only take high-probability trades with favorable risk/reward ratios."#,
            config.mode,
            if config.mode == "live" { "" } else { "NO " },
            config.max_trade_size,
            config.stop_loss,
            config.take_profit,
            config.min_confidence * 100.0,
            config.min_confidence * 100.0
        )
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("🚀 Starting trading bot...");
        
        // Initial portfolio check
        self.check_portfolio().await?;
        
        // Main trading loop
        let mut interval = time::interval(Duration::from_secs(self.config.interval * 60));
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.trading_cycle().await {
                error!("Error in trading cycle: {}", e);
                // Continue running despite errors
            }
        }
    }

    async fn trading_cycle(&mut self) -> Result<()> {
        info!("🔄 Starting trading cycle");

        // 1. Update active positions and check for exits
        self.manage_positions().await?;

        // 2. Analyze market opportunities
        for asset in &self.assets.clone() {
            if let Err(e) = self.analyze_asset(asset).await {
                warn!("Failed to analyze {}: {}", asset, e);
            }
        }

        // 3. Portfolio rebalancing check
        self.check_rebalancing().await?;

        Ok(())
    }

    async fn manage_positions(&mut self) -> Result<()> {
        let positions: Vec<String> = self.active_positions.keys().cloned().collect();
        
        for asset in positions {
            if let Some(position) = self.active_positions.get(&asset) {
                let current_price = self.get_current_price(&asset).await?;
                let pnl_pct = (current_price - position.entry_price) / position.entry_price * 100.0;
                
                info!("Position {}: Entry ${:.4}, Current ${:.4}, PnL: {:.2}%", 
                      asset, position.entry_price, current_price, pnl_pct);

                // Check stop-loss
                if pnl_pct <= -self.config.stop_loss {
                    warn!("🛑 Stop-loss triggered for {}", asset);
                    self.close_position(&asset, "stop_loss").await?;
                }
                // Check take-profit
                else if pnl_pct >= self.config.take_profit {
                    info!("🎯 Take-profit triggered for {}", asset);
                    self.close_position(&asset, "take_profit").await?;
                }
            }
        }

        Ok(())
    }

    async fn analyze_asset(&mut self, asset: &str) -> Result<()> {
        info!("📊 Analyzing {}", asset);

        let query = format!(
            "Analyze {} for potential trading opportunities. Consider:\n\
             1. Current market conditions and price action\n\
             2. Social sentiment and news impact\n\
             3. Technical indicators and patterns\n\
             4. Risk/reward analysis\n\
             5. Position sizing recommendation\n\
             \n\
             Provide a trading recommendation with confidence score (0-100%).",
            asset
        );

        match self.agent.prompt(&query).await {
            Ok(analysis) => {
                info!("Analysis for {}: {}", asset, analysis);
                
                // Parse confidence score and recommendation from response
                if let Some(confidence) = self.extract_confidence(&analysis) {
                    if confidence >= self.config.min_confidence {
                        info!("✅ High confidence signal for {}: {:.1}%", asset, confidence * 100.0);
                        
                        // Extract and execute trading recommendation
                        if let Some(action) = self.extract_trading_action(&analysis) {
                            match self.execute_trade(asset, &action, confidence).await {
                                Ok(_) => info!("✅ Trade executed for {}: {:?}", asset, action),
                                Err(e) => warn!("❌ Failed to execute trade for {}: {}", asset, e),
                            }
                        }
                    }
                }
            }
            Err(e) => warn!("Failed to analyze {}: {}", asset, e),
        }

        Ok(())
    }

    async fn get_current_price(&self, asset: &str) -> Result<f64> {
        // Map asset to token address and chain for DexScreener
        let (token_address, chain) = match asset {
            // Solana
            "SOL" => ("So11111111111111111111111111111111111111112", "solana"),
            // EVM (use canonical wrapped/native representations)
            "ETH" => ("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", "ethereum"), // WETH
            "BTC" => ("0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599", "ethereum"), // WBTC
            "USDC" => ("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", "ethereum"),
            _ => anyhow::bail!("Unsupported asset for pricing: {}", asset),
        };

        // Prefer direct API tool over LLM prompts
        #[allow(unused_imports)]
        use riglr_web_tools::price::get_token_price as get_price_tool;

        let res = get_price_tool(token_address.to_string(), Some(chain.to_string()))
            .await
            .map_err(|e| anyhow::anyhow!("Price API error: {}", e))?;

        let price = res
            .price_usd
            .parse::<f64>()
            .map_err(|e| anyhow::anyhow!("Invalid price format from API: {}", e))?;
        Ok(price)
    }

    async fn close_position(&mut self, asset: &str, reason: &str) -> Result<()> {
        if let Some(position) = self.active_positions.remove(asset) {
            info!("💰 Closing position in {} (reason: {})", asset, reason);
            
            if self.config.mode == "live" {
                // Execute actual trade
                let trade_query = format!(
                    "Execute sell order for {} quantity {} at market price",
                    asset, position.quantity
                );
                
                match self.agent.prompt(&trade_query).await {
                    Ok(result) => info!("Trade executed: {}", result),
                    Err(e) => error!("Failed to execute trade: {}", e),
                }
            } else {
                info!("📋 Paper trade: Sold {} {} at current price", position.quantity, asset);
            }
        }

        Ok(())
    }

    async fn check_portfolio(&self) -> Result<()> {
        info!("💼 Checking portfolio status");
        
        let portfolio_query = "Check current portfolio balances and provide summary";
        match self.agent.prompt(portfolio_query).await {
            Ok(summary) => info!("Portfolio: {}", summary),
            Err(e) => warn!("Failed to get portfolio summary: {}", e),
        }

        Ok(())
    }

    async fn check_rebalancing(&self) -> Result<()> {
        if self.active_positions.len() < 2 {
            return Ok(());
        }

        info!("⚖️  Checking portfolio rebalancing opportunities");
        
        let rebalance_query = "Analyze current portfolio allocation and suggest rebalancing if needed";
        match self.agent.prompt(rebalance_query).await {
            Ok(analysis) => info!("Rebalancing analysis: {}", analysis),
            Err(e) => warn!("Failed rebalancing analysis: {}", e),
        }

        Ok(())
    }

    fn extract_confidence(&self, text: &str) -> Option<f64> {
        // Simple confidence extraction - would be more sophisticated in production
        if text.to_lowercase().contains("high confidence") {
            Some(0.8)
        } else if text.to_lowercase().contains("medium confidence") {
            Some(0.6)
        } else if text.to_lowercase().contains("low confidence") {
            Some(0.4)
        } else {
            Some(0.5) // Default confidence
        }
    }

    fn extract_trading_action(&self, text: &str) -> Option<TradingAction> {
        let lower_text = text.to_lowercase();
        if lower_text.contains("buy") || lower_text.contains("long") {
            Some(TradingAction::Buy)
        } else if lower_text.contains("sell") || lower_text.contains("short") {
            Some(TradingAction::Sell)
        } else {
            None
        }
    }

    async fn execute_trade(&mut self, asset: &str, action: &TradingAction, confidence: f64) -> Result<()> {
        let confidence = confidence.clamp(0.1, 1.0);
        let price = self.get_current_price(asset).await?; // USD per unit
        let usd_notional = (self.config.max_trade_size * confidence).clamp(10.0, self.config.max_trade_size);

        info!("🎯 Executing {:?} for {} | price ${:.4} | notional ${:.2}", action, asset, price, usd_notional);

        // Paper vs live
        let live = self.config.mode.eq_ignore_ascii_case("live");

        match asset {
            // -------------------- Solana via Jupiter --------------------
            "SOL" => {
                let sol_mint = "So11111111111111111111111111111111111111112".to_string();
                let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string();
                let slippage_bps: u16 = 50;

                match action {
                    TradingAction::Buy => {
                        // Swap USDC -> SOL
                        let amount_usdc = usd_notional; // human USDC
                        let amount_usdc_lamports = (amount_usdc * 1_000_000.0) as u64; // USDC 6 decimals

                        if live {
                            // Get quote then perform swap
                            let _quote = riglr_solana_tools::swap::get_jupiter_quote(
                                usdc_mint.clone(),
                                sol_mint.clone(),
                                amount_usdc_lamports,
                                slippage_bps,
                                false,
                                None,
                            ).await?;

                            let res = riglr_solana_tools::swap::perform_jupiter_swap(
                                usdc_mint.clone(),
                                sol_mint.clone(),
                                amount_usdc_lamports,
                                slippage_bps,
                                None,
                                true,
                            ).await?;
                            info!("✅ SOL buy tx: {} (in: {}, out est: {})", res.signature, res.in_amount, res.out_amount);
                        } else {
                            info!("📋 Paper trade: Buy SOL with ${:.2} USDC", amount_usdc);
                        }

                        // Record position (paper/live)
                        let qty = usd_notional / price;
                        self.active_positions.insert(
                            asset.to_string(),
                            Position {
                                asset: asset.to_string(),
                                entry_price: price,
                                quantity: qty,
                                timestamp: chrono::Utc::now(),
                                stop_loss: price * (1.0 - self.config.stop_loss / 100.0),
                                take_profit: price * (1.0 + self.config.take_profit / 100.0),
                                unrealized_pnl: 0.0,
                            },
                        );
                    }
                    TradingAction::Sell => {
                        // Swap SOL -> USDC
                        let qty_sol = usd_notional / price;
                        let amount_sol_lamports = (qty_sol * 1_000_000_000.0) as u64; // SOL 9 decimals

                        if live {
                            let _quote = riglr_solana_tools::swap::get_jupiter_quote(
                                sol_mint.clone(),
                                usdc_mint.clone(),
                                amount_sol_lamports,
                                slippage_bps,
                                false,
                                None,
                            ).await?;

                            let res = riglr_solana_tools::swap::perform_jupiter_swap(
                                sol_mint.clone(),
                                usdc_mint.clone(),
                                amount_sol_lamports,
                                slippage_bps,
                                None,
                                true,
                            ).await?;
                            info!("✅ SOL sell tx: {} (in: {}, out est: {})", res.signature, res.in_amount, res.out_amount);
                        } else {
                            info!("📋 Paper trade: Sell {:.6} SOL for USDC", qty_sol);
                        }

                        // Remove position if exists
                        self.active_positions.remove(asset);
                    }
                }
            }

            // -------------------- EVM via Uniswap V3 --------------------
            "ETH" | "BTC" => {
                let (base_addr, base_decimals) = match asset {
                    "ETH" => ("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", 18u8), // WETH
                    "BTC" => ("0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599", 8u8),   // WBTC
                    _ => unreachable!(),
                };
                let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"; // USDC
                let usdc_decimals = 6u8;
                let fee_tier = Some(3000u32);
                let slippage_bps = Some(50u16);

                match action {
                    TradingAction::Buy => {
                        // USDC -> BASE
                        let usdc_amount_human = usd_notional; // USDC ~ USD
                        let quote = riglr_evm_tools::swap::get_uniswap_quote(
                            usdc_addr.to_string(),
                            base_addr.to_string(),
                            format!("{:.6}", usdc_amount_human),
                            usdc_decimals,
                            base_decimals,
                            fee_tier,
                            slippage_bps,
                        ).await?;

                        if live {
                            let res = riglr_evm_tools::swap::perform_uniswap_swap(
                                usdc_addr.to_string(),
                                base_addr.to_string(),
                                format!("{:.6}", usdc_amount_human),
                                usdc_decimals,
                                quote.amount_out_minimum.clone(),
                                fee_tier,
                                Some(300),
                            ).await?;
                            info!("✅ {} buy tx: {} (min out: {})", asset, res.tx_hash, quote.amount_out_minimum);
                        } else {
                            info!("📋 Paper trade: Buy {} with ${:.2} USDC", asset, usdc_amount_human);
                        }

                        // Record position
                        let qty = usd_notional / price;
                        self.active_positions.insert(
                            asset.to_string(),
                            Position {
                                asset: asset.to_string(),
                                entry_price: price,
                                quantity: qty,
                                timestamp: chrono::Utc::now(),
                                stop_loss: price * (1.0 - self.config.stop_loss / 100.0),
                                take_profit: price * (1.0 + self.config.take_profit / 100.0),
                                unrealized_pnl: 0.0,
                            },
                        );
                    }
                    TradingAction::Sell => {
                        // BASE -> USDC
                        let base_qty_human = usd_notional / price;
                        let quote = riglr_evm_tools::swap::get_uniswap_quote(
                            base_addr.to_string(),
                            usdc_addr.to_string(),
                            format!("{:.8}", base_qty_human),
                            base_decimals,
                            usdc_decimals,
                            fee_tier,
                            slippage_bps,
                        ).await?;

                        if live {
                            let res = riglr_evm_tools::swap::perform_uniswap_swap(
                                base_addr.to_string(),
                                usdc_addr.to_string(),
                                format!("{:.8}", base_qty_human),
                                base_decimals,
                                quote.amount_out_minimum.clone(),
                                fee_tier,
                                Some(300),
                            ).await?;
                            info!("✅ {} sell tx: {} (min out: {})", asset, res.tx_hash, quote.amount_out_minimum);
                        } else {
                            info!("📋 Paper trade: Sell {:.6} {} for USDC", base_qty_human, asset);
                        }

                        self.active_positions.remove(asset);
                    }
                }
            }

            other => {
                anyhow::bail!("Unsupported asset for trading: {}", other);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
enum TradingAction {
    Buy,
    Sell,
}

/// Helper function to extract price from agent response
fn extract_price_from_response(response: &str) -> Option<f64> {
    // Simple regex to find price patterns like "$1234.56"
    use regex::Regex;
    
    let price_patterns = [
        r"\$(\d+\.?\d*)",           // $123.45
        r"price[:\s]*\$?(\d+\.?\d*)", // price: $123.45 or price: 123.45
        r"(\d+\.?\d*)\s*USD",       // 123.45 USD
    ];
    
    for pattern in &price_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(response) {
                if let Some(price_str) = captures.get(1) {
                    if let Ok(price) = price_str.as_str().parse::<f64>() {
                        return Some(price);
                    }
                }
            }
        }
    }
    
    None
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Load environment
    dotenvy::dotenv().ok();

    let config = TradingConfig::parse();
    let mut bot = TradingBot::new(config).await?;
    
    info!("🤖 Trading bot initialized successfully");
    bot.run().await?;

    Ok(())
}