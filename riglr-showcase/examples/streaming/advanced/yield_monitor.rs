//! DeFi Yield Farming Monitor Example
#![allow(missing_docs, dead_code)]
//!
//! Demonstrates real-time monitoring of DeFi yield farming opportunities across multiple protocols.
//! This example shows how to:
//! - Track yield rates across different DeFi protocols
//! - Monitor liquidity pool changes and impermanent loss
//! - Implement automated yield optimization strategies
//! - Handle multi-protocol event correlation
//! - Generate alerts for profitable opportunities

use anyhow::Result;
use dashmap::DashMap;
use riglr_events_core::prelude::*;
use riglr_showcase::config::Config;
use riglr_streams::core::{WindowManager, WindowType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{debug, info, warn};

// Helper for serde to provide a default Instant when deserializing skipped fields
fn now_instant() -> Instant {
    Instant::now()
}

/// DeFi protocol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    pub name: String,
    pub chain: String,
    pub contract_address: String,
    pub protocol_type: ProtocolType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolType {
    LiquidityPool,
    LendingProtocol,
    YieldFarm,
    Staking,
    Vault,
}

/// Yield farming opportunity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldOpportunity {
    pub protocol: Protocol,
    pub pool_address: String,
    pub token_pair: (String, String),
    pub apy: f64,
    pub tvl: f64,
    pub liquidity: f64,
    pub volume_24h: f64,
    pub fees_24h: f64,
    pub impermanent_loss_risk: ImpermanentLossRisk,
    #[serde(skip_serializing, skip_deserializing, default = "now_instant")]
    pub last_updated: Instant,
    pub rewards: Vec<RewardToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardToken {
    pub token: String,
    pub amount_per_day: f64,
    pub current_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpermanentLossRisk {
    Low,    // < 1% expected IL
    Medium, // 1-5% expected IL
    High,   // > 5% expected IL
}

/// Yield monitoring alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldAlert {
    pub alert_type: AlertType,
    pub opportunity: YieldOpportunity,
    pub message: String,
    pub urgency: AlertUrgency,
    #[serde(skip_serializing, skip_deserializing, default = "now_instant")]
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighYield,
    LiquidityDrop,
    PoolDrained,
    ImpermanentLossWarning,
    NewOpportunity,
    APYIncrease,
    APYDecrease,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertUrgency {
    Low,
    Medium,
    High,
    Critical,
}

/// DeFi yield monitor with multi-protocol tracking
pub struct YieldMonitor {
    /// Tracked protocols and their configurations
    protocols: HashMap<String, Protocol>,

    /// Current yield opportunities
    opportunities: Arc<DashMap<String, YieldOpportunity>>,

    /// Price data for impermanent loss calculations
    price_history: WindowManager<PriceData>,

    /// Alert system
    alert_system: AlertSystem,

    /// Performance metrics
    metrics: Arc<RwLock<MonitoringMetrics>>,

    /// Configuration
    config: YieldMonitorConfig,
}

#[derive(Debug, Clone)]
pub struct YieldMonitorConfig {
    pub min_apy_threshold: f64,
    pub min_tvl_threshold: f64,
    pub max_impermanent_loss_risk: ImpermanentLossRisk,
    pub alert_cooldown: Duration,
    pub price_update_interval: Duration,
    pub opportunity_refresh_interval: Duration,
}

impl Default for YieldMonitorConfig {
    fn default() -> Self {
        Self {
            min_apy_threshold: 5.0,       // 5% APY minimum
            min_tvl_threshold: 100_000.0, // $100k TVL minimum
            max_impermanent_loss_risk: ImpermanentLossRisk::Medium,
            alert_cooldown: Duration::from_secs(5 * 60),
            price_update_interval: Duration::from_secs(30),
            opportunity_refresh_interval: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub token: String,
    pub price: f64,
    #[serde(skip_serializing, skip_deserializing, default = "now_instant")]
    pub timestamp: Instant,
    pub source: String,
}

/// Alert system for yield opportunities
pub struct AlertSystem {
    alert_handlers: Vec<Box<dyn AlertHandler>>,
    last_alerts: HashMap<String, Instant>,
    config: YieldMonitorConfig,
}

pub trait AlertHandler: Send + Sync {
    fn handle_alert(&self, alert: &YieldAlert) -> Result<()>;
}

/// Console alert handler
pub struct ConsoleAlertHandler;

impl AlertHandler for ConsoleAlertHandler {
    fn handle_alert(&self, alert: &YieldAlert) -> Result<()> {
        let urgency_icon = match alert.urgency {
            AlertUrgency::Low => "📢",
            AlertUrgency::Medium => "⚠️",
            AlertUrgency::High => "🚨",
            AlertUrgency::Critical => "🔴",
        };

        info!("{} YIELD ALERT: {}", urgency_icon, alert.message);
        info!(
            "   Protocol: {} ({:?})",
            alert.opportunity.protocol.name, alert.opportunity.protocol.protocol_type
        );
        info!(
            "   Pool: {} / {}",
            alert.opportunity.token_pair.0, alert.opportunity.token_pair.1
        );
        info!("   APY: {:.2}%", alert.opportunity.apy);
        info!("   TVL: ${:.0}", alert.opportunity.tvl);
        info!("   IL Risk: {:?}", alert.opportunity.impermanent_loss_risk);

        Ok(())
    }
}

/// Webhook alert handler (for external integrations)
pub struct WebhookAlertHandler {
    webhook_url: String,
}

impl WebhookAlertHandler {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }
}

impl AlertHandler for WebhookAlertHandler {
    fn handle_alert(&self, alert: &YieldAlert) -> Result<()> {
        // In a real implementation, this would send HTTP requests to webhooks
        debug!("📡 Sending alert to webhook: {}", self.webhook_url);
        debug!("   Alert: {:?}", alert);

        // Simulate webhook call
        tokio::spawn({
            let _url = self.webhook_url.clone();
            let _alert = alert.clone();
            async move {
                // simulate_webhook_call(url, alert).await;
                debug!("✅ Webhook notification sent");
            }
        });

        Ok(())
    }
}

impl AlertSystem {
    pub fn new(config: YieldMonitorConfig) -> Self {
        let mut system = Self {
            alert_handlers: Vec::new(),
            last_alerts: HashMap::new(),
            config,
        };

        // Add default handlers
        system.alert_handlers.push(Box::new(ConsoleAlertHandler));

        system
    }

    pub fn add_handler(&mut self, handler: Box<dyn AlertHandler>) {
        self.alert_handlers.push(handler);
    }

    pub async fn send_alert(&mut self, alert: YieldAlert) -> Result<()> {
        let alert_key = format!("{}_{:?}", alert.opportunity.pool_address, alert.alert_type);

        // Check cooldown period
        if let Some(last_alert_time) = self.last_alerts.get(&alert_key) {
            if last_alert_time.elapsed() < self.config.alert_cooldown {
                debug!("⏰ Alert cooldown active for: {}", alert_key);
                return Ok(());
            }
        }

        // Send alert to all handlers
        for handler in &self.alert_handlers {
            if let Err(e) = handler.handle_alert(&alert) {
                warn!("Failed to send alert through handler: {}", e);
            }
        }

        // Update last alert time
        self.last_alerts.insert(alert_key, alert.timestamp);

        Ok(())
    }
}

/// Monitoring performance metrics
#[derive(Debug, Default, Clone)]
pub struct MonitoringMetrics {
    pub opportunities_tracked: usize,
    pub alerts_sent: u64,
    pub total_events_processed: u64,
    pub protocol_updates: HashMap<String, u64>,
    pub avg_processing_time_ms: f64,
    pub impermanent_loss_calculations: u64,
}

impl YieldMonitor {
    pub fn new(config: YieldMonitorConfig) -> Self {
        Self {
            protocols: HashMap::new(),
            opportunities: Arc::new(DashMap::new()),
            price_history: WindowManager::new(WindowType::Sliding {
                size: Duration::from_secs(24 * 60 * 60), // 24-hour price history
                step: Duration::from_secs(60),
            }),
            alert_system: AlertSystem::new(config.clone()),
            metrics: Arc::new(RwLock::new(MonitoringMetrics::default())),
            config,
        }
    }

    pub fn add_protocol(&mut self, protocol: Protocol) {
        info!(
            "📋 Adding protocol: {} on {}",
            protocol.name, protocol.chain
        );
        self.protocols.insert(protocol.name.clone(), protocol);
    }

    pub async fn process_pool_event(&mut self, event: &dyn Event) -> Result<()> {
        let start_time = Instant::now();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_events_processed += 1;
        }

        // Extract pool information from event
        if let Some(pool_data) = self.extract_pool_data(event) {
            // Update yield opportunity
            let opportunity = self.calculate_yield_opportunity(pool_data).await?;

            // Store updated opportunity
            let pool_key = opportunity.pool_address.clone();
            let previous_opportunity = self.opportunities.get(&pool_key).map(|r| r.value().clone());
            self.opportunities
                .insert(pool_key.clone(), opportunity.clone());

            // Check for alerts
            self.check_for_alerts(opportunity, previous_opportunity)
                .await?;
        }

        // Update processing time metrics
        let processing_time = start_time.elapsed().as_millis() as f64;
        {
            let mut metrics = self.metrics.write().await;
            let total_events = metrics.total_events_processed as f64;
            metrics.avg_processing_time_ms =
                (metrics.avg_processing_time_ms * (total_events - 1.0) + processing_time)
                    / total_events;
        }

        Ok(())
    }

    pub async fn process_price_event(&mut self, price_data: PriceData) -> Result<()> {
        // Add price to history window
        let completed_windows = self.price_history.add_event(price_data.clone());

        // Process completed price windows for impermanent loss calculations
        for window in completed_windows {
            if window.events.len() < 2 {
                continue;
            }

            debug!(
                "📊 Processing price window for {} with {} data points",
                price_data.token,
                window.events.len()
            );

            // Calculate impermanent loss for all relevant pools
            self.update_impermanent_loss_estimates(&window.events)
                .await?;
        }

        Ok(())
    }

    fn extract_pool_data(&self, event: &dyn Event) -> Option<PoolEventData> {
        // Downcast to GenericEvent to access JSON payload
        if let Some(ge) = event.as_any().downcast_ref::<GenericEvent>() {
            let data = &ge.data;
            let source = event.source();
            match source {
                s if s.contains("raydium") => self.parse_raydium_event(data),
                s if s.contains("orca") => self.parse_orca_event(data),
                s if s.contains("jupiter") => self.parse_jupiter_event(data),
                s if s.contains("marinade") => self.parse_marinade_event(data),
                _ => {
                    debug!("Unknown protocol source: {}", source);
                    None
                }
            }
        } else {
            None
        }
    }

    fn parse_raydium_event(&self, data: &serde_json::Value) -> Option<PoolEventData> {
        // Parse Raydium AMM pool events
        Some(PoolEventData {
            protocol_name: "Raydium".to_string(),
            pool_address: data.get("pool")?.as_str()?.to_string(),
            token_a: data.get("token_a")?.as_str()?.to_string(),
            token_b: data.get("token_b")?.as_str()?.to_string(),
            liquidity_a: data.get("liquidity_a")?.as_f64()?,
            liquidity_b: data.get("liquidity_b")?.as_f64()?,
            fee_rate: data.get("fee_rate")?.as_f64().unwrap_or(0.25), // 0.25% default
            volume_24h: data.get("volume_24h")?.as_f64().unwrap_or(0.0),
            fees_24h: data.get("fees_24h")?.as_f64().unwrap_or(0.0),
        })
    }

    fn parse_orca_event(&self, data: &serde_json::Value) -> Option<PoolEventData> {
        // Parse Orca pool events
        Some(PoolEventData {
            protocol_name: "Orca".to_string(),
            pool_address: data.get("whirlpool")?.as_str()?.to_string(),
            token_a: data.get("token_mint_a")?.as_str()?.to_string(),
            token_b: data.get("token_mint_b")?.as_str()?.to_string(),
            liquidity_a: data.get("vault_a_amount")?.as_f64()?,
            liquidity_b: data.get("vault_b_amount")?.as_f64()?,
            fee_rate: data.get("fee_rate")?.as_f64().unwrap_or(0.30), // 0.30% default
            volume_24h: 0.0, // Would be calculated separately
            fees_24h: 0.0,   // Would be calculated separately
        })
    }

    fn parse_jupiter_event(&self, _data: &serde_json::Value) -> Option<PoolEventData> {
        // Jupiter is primarily an aggregator, so we might track route efficiency
        None
    }

    fn parse_marinade_event(&self, data: &serde_json::Value) -> Option<PoolEventData> {
        // Parse Marinade liquid staking events
        Some(PoolEventData {
            protocol_name: "Marinade".to_string(),
            pool_address: "marinade_pool".to_string(), // Simplified
            token_a: "SOL".to_string(),
            token_b: "mSOL".to_string(),
            liquidity_a: data.get("total_sol")?.as_f64()?,
            liquidity_b: data.get("total_msol")?.as_f64()?,
            fee_rate: 0.0, // Marinade has different reward mechanics
            volume_24h: data.get("stake_volume_24h")?.as_f64().unwrap_or(0.0),
            fees_24h: 0.0, // Rewards instead of fees
        })
    }

    async fn calculate_yield_opportunity(
        &self,
        pool_data: PoolEventData,
    ) -> Result<YieldOpportunity> {
        // Get protocol info
        let protocol = self
            .protocols
            .get(&pool_data.protocol_name)
            .cloned()
            .unwrap_or_else(|| Protocol {
                name: pool_data.protocol_name.clone(),
                chain: "solana".to_string(),
                contract_address: pool_data.pool_address.clone(),
                protocol_type: ProtocolType::LiquidityPool,
            });

        // Calculate TVL
        let tvl = pool_data.liquidity_a + pool_data.liquidity_b; // Simplified - would need price data

        // Calculate APY based on fees and rewards
        let daily_fees = pool_data.fees_24h;
        let annual_fees = daily_fees * 365.0;
        let fee_apy = if tvl > 0.0 {
            (annual_fees / tvl) * 100.0
        } else {
            0.0
        };

        // Add reward APY (would query reward token prices and emission rates)
        let reward_apy = self.calculate_reward_apy(&pool_data).await;
        let total_apy = fee_apy + reward_apy;

        // Estimate impermanent loss risk
        let il_risk = self
            .estimate_impermanent_loss_risk(&pool_data.token_a, &pool_data.token_b)
            .await;

        let opportunity = YieldOpportunity {
            protocol,
            pool_address: pool_data.pool_address,
            token_pair: (pool_data.token_a, pool_data.token_b),
            apy: total_apy,
            tvl,
            liquidity: tvl, // Simplified
            volume_24h: pool_data.volume_24h,
            fees_24h: pool_data.fees_24h,
            impermanent_loss_risk: il_risk,
            last_updated: Instant::now(),
            rewards: vec![], // Would populate with actual reward tokens
        };

        debug!(
            "📈 Calculated yield opportunity: {} APY for {}/{}",
            total_apy, opportunity.token_pair.0, opportunity.token_pair.1
        );

        Ok(opportunity)
    }

    async fn calculate_reward_apy(&self, _pool_data: &PoolEventData) -> f64 {
        // In a real implementation, this would:
        // 1. Query reward token emission rates
        // 2. Get current prices for reward tokens
        // 3. Calculate annual reward value
        // 4. Express as percentage of TVL

        // Simulate reward APY
        match _pool_data.protocol_name.as_str() {
            "Raydium" => 3.5,  // RAY rewards
            "Orca" => 2.8,     // ORCA rewards
            "Marinade" => 5.2, // Staking rewards
            _ => 1.0,
        }
    }

    async fn estimate_impermanent_loss_risk(
        &self,
        token_a: &str,
        token_b: &str,
    ) -> ImpermanentLossRisk {
        // Analyze historical price correlation and volatility
        // For this example, we'll use simplified heuristics

        match (token_a, token_b) {
            // Stablecoin pairs have low IL risk
            (a, b)
                if (a.contains("USD") || a.contains("USDT") || a.contains("USDC"))
                    && (b.contains("USD") || b.contains("USDT") || b.contains("USDC")) =>
            {
                ImpermanentLossRisk::Low
            }
            // SOL/mSOL or other liquid staking pairs have low IL risk
            ("SOL", "mSOL") | ("mSOL", "SOL") => ImpermanentLossRisk::Low,
            // Major asset pairs have medium risk
            (a, b)
                if (a == "SOL" || a == "BTC" || a == "ETH")
                    && (b == "SOL" || b == "BTC" || b == "ETH") =>
            {
                ImpermanentLossRisk::Medium
            }
            // Everything else is high risk
            _ => ImpermanentLossRisk::High,
        }
    }

    async fn check_for_alerts(
        &mut self,
        opportunity: YieldOpportunity,
        previous: Option<YieldOpportunity>,
    ) -> Result<()> {
        let mut alerts = Vec::new();
        let opp_token_pair = opportunity.token_pair.clone();
        let _opp_pool_address = opportunity.pool_address.clone();
        let opp_il_risk = opportunity.impermanent_loss_risk.clone();

        // Check for high yield opportunities
        if opportunity.apy >= self.config.min_apy_threshold
            && opportunity.tvl >= self.config.min_tvl_threshold
            && matches!(opp_il_risk, r if matches!(r,
               ImpermanentLossRisk::Low | ImpermanentLossRisk::Medium) ||
               matches!(self.config.max_impermanent_loss_risk, ImpermanentLossRisk::High))
        {
            alerts.push(YieldAlert {
                alert_type: AlertType::HighYield,
                opportunity: opportunity.clone(),
                message: format!(
                    "High yield opportunity: {:.2}% APY on {}/{}",
                    opportunity.apy, opp_token_pair.0, opp_token_pair.1
                ),
                urgency: if opportunity.apy > 20.0 {
                    AlertUrgency::High
                } else {
                    AlertUrgency::Medium
                },
                timestamp: Instant::now(),
            });
        }

        // Check for APY changes
        if let Some(prev) = previous {
            let apy_change = opportunity.apy - prev.apy;

            if apy_change > 5.0 {
                alerts.push(YieldAlert {
                    alert_type: AlertType::APYIncrease,
                    opportunity: opportunity.clone(),
                    message: format!(
                        "APY increased by {:.2}% for {}/{}",
                        apy_change, opportunity.token_pair.0, opportunity.token_pair.1
                    ),
                    urgency: AlertUrgency::Medium,
                    timestamp: Instant::now(),
                });
            } else if apy_change < -5.0 {
                alerts.push(YieldAlert {
                    alert_type: AlertType::APYDecrease,
                    opportunity: opportunity.clone(),
                    message: format!(
                        "APY decreased by {:.2}% for {}/{}",
                        apy_change.abs(),
                        opportunity.token_pair.0,
                        opportunity.token_pair.1
                    ),
                    urgency: AlertUrgency::Low,
                    timestamp: Instant::now(),
                });
            }

            // Check for liquidity drops
            let liquidity_change_pct =
                ((opportunity.liquidity - prev.liquidity) / prev.liquidity) * 100.0;
            if liquidity_change_pct < -20.0 {
                alerts.push(YieldAlert {
                    alert_type: AlertType::LiquidityDrop,
                    opportunity: opportunity.clone(),
                    message: format!(
                        "Liquidity dropped by {:.1}% for {}/{}",
                        liquidity_change_pct.abs(),
                        opp_token_pair.0,
                        opp_token_pair.1
                    ),
                    urgency: AlertUrgency::High,
                    timestamp: Instant::now(),
                });
            }
        }

        // Send all alerts
        for alert in alerts {
            self.alert_system.send_alert(alert).await?;

            let mut metrics = self.metrics.write().await;
            metrics.alerts_sent += 1;
        }

        Ok(())
    }

    async fn update_impermanent_loss_estimates(
        &mut self,
        price_history: &[PriceData],
    ) -> Result<()> {
        // Calculate impermanent loss for all tracked opportunities
        for entry in self.opportunities.iter() {
            let pool_key = entry.key();
            let opportunity = entry.value();
            // Find price data for both tokens in the pair
            let token_a_prices: Vec<_> = price_history
                .iter()
                .filter(|p| p.token == opportunity.token_pair.0)
                .collect();

            let token_b_prices: Vec<_> = price_history
                .iter()
                .filter(|p| p.token == opportunity.token_pair.1)
                .collect();

            if token_a_prices.len() >= 2 && token_b_prices.len() >= 2 {
                // Calculate impermanent loss based on price ratio changes
                let initial_ratio = token_a_prices[0].price / token_b_prices[0].price;
                let current_ratio =
                    token_a_prices.last().unwrap().price / token_b_prices.last().unwrap().price;

                let il_percentage = self.calculate_impermanent_loss(initial_ratio, current_ratio);

                debug!(
                    "📊 Impermanent Loss for {}: {:.2}%",
                    pool_key,
                    il_percentage * 100.0
                );

                // Update metrics
                let mut metrics = self.metrics.write().await;
                metrics.impermanent_loss_calculations += 1;
            }
        }

        Ok(())
    }

    fn calculate_impermanent_loss(&self, initial_ratio: f64, current_ratio: f64) -> f64 {
        // Standard impermanent loss formula for 50/50 pools
        let ratio_change = current_ratio / initial_ratio;
        let il = 2.0 * (ratio_change.sqrt() / (1.0 + ratio_change)) - 1.0;
        il.abs()
    }

    pub async fn get_metrics(&self) -> MonitoringMetrics {
        let guard = self.metrics.read().await;
        (*guard).clone()
    }

    pub async fn get_top_opportunities(&self, limit: usize) -> Vec<YieldOpportunity> {
        let mut sorted_opportunities: Vec<_> = self
            .opportunities
            .iter()
            .map(|r| r.value().clone())
            .collect();

        // Sort by APY descending
        sorted_opportunities.sort_by(|a, b| {
            b.apy
                .partial_cmp(&a.apy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        sorted_opportunities.into_iter().take(limit).collect()
    }
}

#[derive(Debug, Clone)]
struct PoolEventData {
    protocol_name: String,
    pool_address: String,
    token_a: String,
    token_b: String,
    liquidity_a: f64,
    liquidity_b: f64,
    fee_rate: f64,
    volume_24h: f64,
    fees_24h: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,yield_monitor=debug,riglr_streams=info")
        .init();

    info!("🌾 Starting DeFi Yield Farming Monitor");

    // Load configuration
    let config = Config::from_env();
    config.validate()?;

    // Create yield monitor with custom configuration
    let monitor_config = YieldMonitorConfig {
        min_apy_threshold: 8.0,      // 8% minimum APY
        min_tvl_threshold: 50_000.0, // $50k minimum TVL
        max_impermanent_loss_risk: ImpermanentLossRisk::Medium,
        ..Default::default()
    };

    let mut yield_monitor = YieldMonitor::new(monitor_config);

    // Add protocols to monitor
    yield_monitor.add_protocol(Protocol {
        name: "Raydium".to_string(),
        chain: "solana".to_string(),
        contract_address: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
        protocol_type: ProtocolType::LiquidityPool,
    });

    yield_monitor.add_protocol(Protocol {
        name: "Orca".to_string(),
        chain: "solana".to_string(),
        contract_address: "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(),
        protocol_type: ProtocolType::LiquidityPool,
    });

    yield_monitor.add_protocol(Protocol {
        name: "Marinade".to_string(),
        chain: "solana".to_string(),
        contract_address: "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD".to_string(),
        protocol_type: ProtocolType::Staking,
    });

    info!("🔄 Starting yield opportunity monitoring...");

    // Simulate monitoring with periodic updates
    let mut metrics_interval = tokio::time::interval(Duration::from_secs(30));
    let mut opportunity_interval = tokio::time::interval(Duration::from_secs(10));
    let mut price_interval = tokio::time::interval(Duration::from_secs(5));

    let start_time = Instant::now();

    loop {
        tokio::select! {
            _ = metrics_interval.tick() => {
                let metrics = yield_monitor.get_metrics().await;
                info!("📊 Monitoring Metrics:");
                info!("   Opportunities tracked: {}", metrics.opportunities_tracked);
                info!("   Events processed: {}", metrics.total_events_processed);
                info!("   Alerts sent: {}", metrics.alerts_sent);
                info!("   Avg processing time: {:.2}ms", metrics.avg_processing_time_ms);
                info!("   IL calculations: {}", metrics.impermanent_loss_calculations);

                // Show top opportunities
                let top_opportunities = yield_monitor.get_top_opportunities(3).await;
                if !top_opportunities.is_empty() {
                    info!("🏆 Top Yield Opportunities:");
                    for (i, opp) in top_opportunities.iter().enumerate() {
                        info!("   {}. {}/{} - {:.2}% APY (TVL: ${:.0})",
                             i + 1, opp.token_pair.0, opp.token_pair.1, opp.apy, opp.tvl);
                    }
                }
            }

            _ = opportunity_interval.tick() => {
                // Simulate pool events from different protocols
                let protocols = ["Raydium", "Orca", "Marinade"];
                let protocol = protocols[fastrand::usize(0..protocols.len())];

                let pool_event = create_simulated_pool_event(protocol);
                if let Err(e) = yield_monitor.process_pool_event(&*pool_event).await {
                    warn!("Failed to process pool event: {}", e);
                }
            }

            _ = price_interval.tick() => {
                // Simulate price updates
                let tokens = ["SOL", "USDC", "mSOL", "RAY", "ORCA"];
                for token in &tokens {
                    let price_data = create_simulated_price_data(token);
                    if let Err(e) = yield_monitor.process_price_event(price_data).await {
                        warn!("Failed to process price event: {}", e);
                    }
                }
            }
        }

        // Stop after reasonable test duration
        if start_time.elapsed() > Duration::from_secs(180) {
            info!("⏰ Test duration complete");
            break;
        }
    }

    // Final report
    let final_metrics = yield_monitor.get_metrics().await;
    info!("🏁 Final Monitoring Report:");
    info!(
        "   Total events processed: {}",
        final_metrics.total_events_processed
    );
    info!("   Total alerts sent: {}", final_metrics.alerts_sent);
    info!(
        "   Impermanent loss calculations: {}",
        final_metrics.impermanent_loss_calculations
    );

    Ok(())
}

fn create_simulated_pool_event(protocol: &str) -> Box<dyn Event> {
    let data = match protocol {
        "Raydium" => serde_json::json!({
            "pool": format!("pool_{}", fastrand::u64(1000..9999)),
            "token_a": "SOL",
            "token_b": "USDC",
            "liquidity_a": fastrand::f64() * 100_000.0 + 50_000.0,
            "liquidity_b": fastrand::f64() * 100_000.0 + 50_000.0,
            "fee_rate": 0.25,
            "volume_24h": fastrand::f64() * 1_000_000.0,
            "fees_24h": fastrand::f64() * 1_000.0,
        }),
        "Orca" => serde_json::json!({
            "whirlpool": format!("whirlpool_{}", fastrand::u64(1000..9999)),
            "token_mint_a": "SOL",
            "token_mint_b": "USDC",
            "vault_a_amount": fastrand::f64() * 80_000.0 + 40_000.0,
            "vault_b_amount": fastrand::f64() * 80_000.0 + 40_000.0,
            "fee_rate": 0.30,
        }),
        "Marinade" => serde_json::json!({
            "total_sol": fastrand::f64() * 500_000.0 + 100_000.0,
            "total_msol": fastrand::f64() * 480_000.0 + 95_000.0,
            "stake_volume_24h": fastrand::f64() * 50_000.0,
        }),
        _ => serde_json::json!({}),
    };

    Box::new(GenericEvent::new(
        format!(
            "{}_{}",
            protocol.to_lowercase(),
            fastrand::u64(10000..99999)
        )
        .into(),
        EventKind::Custom(protocol.to_string()),
        data,
    ))
}

fn create_simulated_price_data(token: &str) -> PriceData {
    let base_price = match token {
        "SOL" => 150.0,
        "USDC" => 1.0,
        "mSOL" => 165.0,
        "RAY" => 0.85,
        "ORCA" => 3.2,
        _ => 100.0,
    };

    // Add some random price movement
    let price_change = (fastrand::f64() - 0.5) * 0.02; // ±1% change
    let current_price = base_price * (1.0 + price_change);

    PriceData {
        token: token.to_string(),
        price: current_price,
        timestamp: Instant::now(),
        source: "simulated".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_yield_monitor_creation() {
        let config = YieldMonitorConfig::default();
        let monitor = YieldMonitor::new(config);

        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.opportunities_tracked, 0);
        assert_eq!(metrics.total_events_processed, 0);
    }

    #[test]
    fn test_impermanent_loss_calculation() {
        let config = YieldMonitorConfig::default();
        let monitor = YieldMonitor::new(config);

        // Test case: 50% price increase in one token
        let initial_ratio = 1.0; // 1:1 price ratio
        let current_ratio = 1.5; // 1.5:1 price ratio (50% increase)

        let il = monitor.calculate_impermanent_loss(initial_ratio, current_ratio);
        assert!(il > 0.0 && il < 0.1); // Should be positive but less than 10%
    }

    #[test]
    fn test_impermanent_loss_risk_estimation() {
        let config = YieldMonitorConfig::default();
        let monitor = YieldMonitor::new(config);

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Stablecoin pairs should be low risk
            let risk = monitor.estimate_impermanent_loss_risk("USDC", "USDT").await;
            assert!(matches!(risk, ImpermanentLossRisk::Low));

            // SOL/mSOL should be low risk
            let risk = monitor.estimate_impermanent_loss_risk("SOL", "mSOL").await;
            assert!(matches!(risk, ImpermanentLossRisk::Low));

            // Random tokens should be high risk
            let risk = monitor
                .estimate_impermanent_loss_risk("TokenA", "TokenB")
                .await;
            assert!(matches!(risk, ImpermanentLossRisk::High));
        });
    }

    #[tokio::test]
    async fn test_alert_system() {
        let config = YieldMonitorConfig::default();
        let mut alert_system = AlertSystem::new(config);

        let opportunity = YieldOpportunity {
            protocol: Protocol {
                name: "TestProtocol".to_string(),
                chain: "test".to_string(),
                contract_address: "test_address".to_string(),
                protocol_type: ProtocolType::LiquidityPool,
            },
            pool_address: "test_pool".to_string(),
            token_pair: ("TokenA".to_string(), "TokenB".to_string()),
            apy: 25.0,
            tvl: 100_000.0,
            liquidity: 100_000.0,
            volume_24h: 50_000.0,
            fees_24h: 125.0,
            impermanent_loss_risk: ImpermanentLossRisk::Low,
            last_updated: Instant::now(),
            rewards: vec![],
        };

        let alert = YieldAlert {
            alert_type: AlertType::HighYield,
            opportunity,
            message: "Test alert".to_string(),
            urgency: AlertUrgency::High,
            timestamp: Instant::now(),
        };

        let result = alert_system.send_alert(alert).await;
        assert!(result.is_ok());
    }
}
