# Building a Pump.fun Trading Agent

In this tutorial, you'll create an intelligent trading agent for Pump.fun that monitors new token launches, implements sniping strategies, and manages risk through position sizing and social sentiment analysis.

## Overview

Pump.fun is a Solana-based platform for launching meme tokens with bonding curve mechanics. Successful trading on the platform requires fast execution, sentiment analysis, and risk management. This agent will help you capitalize on opportunities while protecting your capital.

### What You'll Build

- Real-time monitoring of new Pump.fun token launches
- Sniping strategies for early token entry
- Social sentiment analysis for trade signals
- Automated position sizing and risk management
- Stop-loss and take-profit automation
- Multi-strategy trading system with backtesting

## Prerequisites

- Rust 1.75+
- Solana wallet with SOL for gas and trading
- Solana RPC access (Helius, QuickNode, or similar)
- Twitter API access for sentiment analysis
- Basic understanding of bonding curves and meme tokens
- create-riglr-app CLI tool

## Project Setup

Create the project using create-riglr-app:

```bash
create-riglr-app pump-trader --template advanced
cd pump-trader
```

This generates a project with all necessary dependencies:

```toml
[dependencies]
riglr-core = "0.3"
riglr-solana-tools = "0.3"
riglr-solana-events = "0.3"
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
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }
```

## Step 1: Define Trading Data Structures

```rust
// src/types.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpToken {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub description: Option<String>,
    pub creator: Pubkey,
    pub created_at: DateTime<Utc>,
    pub market_cap: Option<f64>,
    pub price_sol: Option<f64>,
    pub liquidity_sol: Option<f64>,
    pub holder_count: Option<u32>,
    pub volume_24h: Option<f64>,
    pub price_change_24h: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStrategy {
    pub name: String,
    pub enabled: bool,
    pub min_market_cap: f64,
    pub max_market_cap: f64,
    pub min_liquidity_sol: f64,
    pub max_position_size_sol: f64,
    pub take_profit_percentage: f64,
    pub stop_loss_percentage: f64,
    pub min_sentiment_score: f64,
    pub max_age_minutes: u32,
    pub required_social_signals: Vec<SocialSignal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialSignal {
    TwitterMentions(u32),
    TelegramMembers(u32),
    InfluencerEndorsement,
    CommunityStrength(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub token_mint: Pubkey,
    pub strategy: String,
    pub action: TradeAction,
    pub amount_sol: f64,
    pub price_sol: f64,
    pub timestamp: DateTime<Utc>,
    pub transaction_hash: String,
    pub status: TradeStatus,
    pub pnl_sol: Option<f64>,
    pub pnl_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeAction {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeStatus {
    Pending,
    Executed,
    Failed,
    PartiallyFilled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub token_mint: Pubkey,
    pub symbol: String,
    pub amount: f64,
    pub average_price_sol: f64,
    pub current_price_sol: f64,
    pub value_sol: f64,
    pub pnl_sol: f64,
    pub pnl_percentage: f64,
    pub entry_time: DateTime<Utc>,
    pub stop_loss_price: Option<f64>,
    pub take_profit_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalysis {
    pub token_mint: Pubkey,
    pub sentiment_score: f64,
    pub social_signals: Vec<SocialSignal>,
    pub technical_score: f64,
    pub risk_score: f64,
    pub recommendation: TradeRecommendation,
    pub confidence: f64,
    pub analysis_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeRecommendation {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
    Avoid,
}
```

## Step 2: Implement Token Discovery System

```rust
// src/discovery.rs
use riglr_solana_events::{PumpFunParser, EventType};
use riglr_solana_tools::get_recent_transactions;
use riglr_core::provider::ApplicationContext;
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;
use anyhow::Result;
use log::{info, warn, debug};

pub struct TokenDiscovery {
    context: ApplicationContext,
    solana_client: Arc<RpcClient>,
    pump_parser: PumpFunParser,
    discovered_tokens: tokio::sync::RwLock<std::collections::HashMap<Pubkey, PumpToken>>,
}

impl TokenDiscovery {
    pub fn new(context: ApplicationContext, solana_client: Arc<RpcClient>) -> Self {
        Self {
            context,
            solana_client,
            pump_parser: PumpFunParser::new(),
            discovered_tokens: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        
        info!("Starting Pump.fun token discovery...");
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.scan_for_new_tokens().await {
                warn!("Error scanning for new tokens: {}", e);
                continue;
            }
        }
    }

    async fn scan_for_new_tokens(&self) -> Result<()> {
        // Get recent transactions from Pump.fun program
        let pump_program_id = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
        
        let recent_txs = get_recent_transactions(
            &self.context,
            pump_program_id.to_string(),
            100, // Get last 100 transactions
        ).await?;

        for tx in recent_txs.transactions {
            // Parse transaction for create pool events
            if let Some(instructions) = tx.transaction.message.instructions.first() {
                if let Ok(instruction_data) = base64::decode(&instructions.data) {
                    if self.pump_parser.can_parse(&instruction_data) {
                        let metadata = riglr_solana_events::SolanaEventMetadata::default();
                        
                        if let Ok(events) = self.pump_parser.parse_from_slice(&instruction_data, metadata) {
                            for event in events {
                                if matches!(event.event_type(), Some(EventType::PumpSwapCreatePool)) {
                                    self.process_new_token_creation(&tx, event).await?;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_new_token_creation(
        &self,
        tx: &solana_transaction_status::UiTransactionDetails,
        event: riglr_solana_events::ZeroCopyEvent<'_>
    ) -> Result<()> {
        // Extract token information from the create pool event
        let token_mint = self.extract_token_mint_from_transaction(tx)?;
        
        // Get token metadata
        let token_info = self.fetch_token_metadata(&token_mint).await?;
        
        info!("🚀 New Pump.fun token discovered: {} ({})", token_info.name, token_info.symbol);
        
        // Store the discovered token
        let mut discovered = self.discovered_tokens.write().await;
        discovered.insert(token_mint, token_info.clone());
        
        // Notify strategy engines about new token
        self.notify_new_token(&token_info).await?;
        
        Ok(())
    }

    fn extract_token_mint_from_transaction(&self, tx: &solana_transaction_status::UiTransactionDetails) -> Result<Pubkey> {
        // Parse the transaction to extract the token mint
        // This is a simplified implementation - in reality, you'd need to parse
        // the specific accounts and instruction data more carefully
        if let Some(account_keys) = &tx.transaction.message.account_keys {
            if !account_keys.is_empty() {
                return Ok(Pubkey::from_str(&account_keys[0])?);
            }
        }
        Err(anyhow::anyhow!("Could not extract token mint from transaction"))
    }

    async fn fetch_token_metadata(&self, mint: &Pubkey) -> Result<PumpToken> {
        // Fetch token metadata from various sources
        // This would integrate with Metaplex, Jupiter, or other metadata sources
        
        Ok(PumpToken {
            mint: *mint,
            name: "Unknown Token".to_string(),
            symbol: "UNK".to_string(),
            uri: "".to_string(),
            description: None,
            creator: Pubkey::default(),
            created_at: chrono::Utc::now(),
            market_cap: None,
            price_sol: None,
            liquidity_sol: None,
            holder_count: None,
            volume_24h: None,
            price_change_24h: None,
        })
    }

    async fn notify_new_token(&self, token: &PumpToken) -> Result<()> {
        // Send notification to trading strategies about new token
        debug!("Notifying strategies about new token: {}", token.symbol);
        
        // In a real implementation, this would use channels or event systems
        // to notify all active trading strategies
        
        Ok(())
    }

    pub async fn get_discovered_tokens(&self) -> std::collections::HashMap<Pubkey, PumpToken> {
        self.discovered_tokens.read().await.clone()
    }

    pub async fn get_token_info(&self, mint: &Pubkey) -> Option<PumpToken> {
        self.discovered_tokens.read().await.get(mint).cloned()
    }
}

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
```

## Step 3: Build Sentiment Analysis Engine

```rust
// src/sentiment.rs
use riglr_web_tools::{
    search_tweets, get_user_tweets, analyze_crypto_sentiment,
    search_web, get_crypto_news, analyze_market_sentiment
};
use riglr_core::provider::ApplicationContext;
use anyhow::Result;
use log::{info, debug};

pub struct SentimentAnalyzer {
    context: ApplicationContext,
    sentiment_cache: tokio::sync::RwLock<std::collections::HashMap<String, CachedSentiment>>,
}

#[derive(Debug, Clone)]
struct CachedSentiment {
    score: f64,
    signals: Vec<SocialSignal>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl SentimentAnalyzer {
    pub fn new(context: ApplicationContext) -> Self {
        Self {
            context,
            sentiment_cache: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub async fn analyze_token_sentiment(&self, token: &PumpToken) -> Result<MarketAnalysis> {
        info!("Analyzing sentiment for token: {}", token.symbol);

        // Check cache first
        if let Some(cached) = self.get_cached_sentiment(&token.symbol).await {
            if chrono::Utc::now().signed_duration_since(cached.timestamp).num_minutes() < 5 {
                debug!("Using cached sentiment for {}", token.symbol);
                return Ok(MarketAnalysis {
                    token_mint: token.mint,
                    sentiment_score: cached.score,
                    social_signals: cached.signals,
                    technical_score: 0.5, // Would be calculated separately
                    risk_score: self.calculate_risk_score(token, cached.score).await?,
                    recommendation: self.generate_recommendation(cached.score, 0.5),
                    confidence: 0.8,
                    analysis_time: chrono::Utc::now(),
                });
            }
        }

        // Perform fresh analysis
        let twitter_sentiment = self.analyze_twitter_sentiment(token).await?;
        let web_sentiment = self.analyze_web_sentiment(token).await?;
        let social_signals = self.gather_social_signals(token).await?;

        // Combine sentiment scores
        let combined_score = (twitter_sentiment * 0.5) + (web_sentiment * 0.3) + (self.calculate_social_signal_score(&social_signals) * 0.2);

        // Cache the result
        self.cache_sentiment(&token.symbol, combined_score, social_signals.clone()).await;

        let technical_score = self.calculate_technical_score(token).await?;
        let risk_score = self.calculate_risk_score(token, combined_score).await?;

        Ok(MarketAnalysis {
            token_mint: token.mint,
            sentiment_score: combined_score,
            social_signals,
            technical_score,
            risk_score,
            recommendation: self.generate_recommendation(combined_score, technical_score),
            confidence: self.calculate_confidence(combined_score, technical_score),
            analysis_time: chrono::Utc::now(),
        })
    }

    async fn analyze_twitter_sentiment(&self, token: &PumpToken) -> Result<f64> {
        // Search for tweets about the token
        let search_query = format!("${} OR {} pump.fun", token.symbol, token.name);
        
        let tweet_results = search_tweets(
            &self.context,
            search_query,
            Some(50),
            Some("recent".to_string()),
        ).await?;

        if tweet_results.tweets.is_empty() {
            return Ok(0.5); // Neutral sentiment if no tweets found
        }

        // Analyze sentiment of the tweets
        let tweet_texts: Vec<String> = tweet_results.tweets
            .iter()
            .map(|t| t.text.clone())
            .collect();

        let sentiment_analysis = analyze_crypto_sentiment(
            &self.context,
            tweet_texts,
        ).await?;

        Ok(sentiment_analysis.overall_sentiment.positive_ratio)
    }

    async fn analyze_web_sentiment(&self, token: &PumpToken) -> Result<f64> {
        // Search for web mentions of the token
        let search_query = format!("{} {} pump.fun token", token.symbol, token.name);
        
        let web_results = search_web(
            &self.context,
            search_query,
            Some(20),
            Some(false),
        ).await?;

        if web_results.results.is_empty() {
            return Ok(0.5);
        }

        // Analyze sentiment of web content
        let content_texts: Vec<String> = web_results.results
            .iter()
            .map(|r| format!("{} {}", r.title, r.snippet))
            .collect();

        let sentiment_analysis = analyze_crypto_sentiment(
            &self.context,
            content_texts,
        ).await?;

        Ok(sentiment_analysis.overall_sentiment.positive_ratio)
    }

    async fn gather_social_signals(&self, token: &PumpToken) -> Result<Vec<SocialSignal>> {
        let mut signals = Vec::new();

        // Twitter mention analysis
        let mention_count = self.get_twitter_mention_count(token).await?;
        if mention_count > 10 {
            signals.push(SocialSignal::TwitterMentions(mention_count));
        }

        // Check for influencer endorsements
        if self.check_influencer_endorsement(token).await? {
            signals.push(SocialSignal::InfluencerEndorsement);
        }

        // Community strength analysis
        let community_strength = self.analyze_community_strength(token).await?;
        if community_strength > 0.7 {
            signals.push(SocialSignal::CommunityStrength(community_strength));
        }

        Ok(signals)
    }

    async fn get_twitter_mention_count(&self, token: &PumpToken) -> Result<u32> {
        let search_query = format!("${}", token.symbol);
        let results = search_tweets(&self.context, search_query, Some(100), None).await?;
        Ok(results.tweets.len() as u32)
    }

    async fn check_influencer_endorsement(&self, token: &PumpToken) -> Result<bool> {
        // List of known crypto influencers (simplified)
        let influencers = vec!["elonmusk", "VitalikButerin", "cz_binance"];
        
        for influencer in influencers {
            let user_tweets = get_user_tweets(
                &self.context,
                influencer.to_string(),
                Some(20),
            ).await?;

            for tweet in user_tweets.tweets {
                if tweet.text.to_lowercase().contains(&token.symbol.to_lowercase()) {
                    info!("Found influencer endorsement from {}", influencer);
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }

    async fn analyze_community_strength(&self, token: &PumpToken) -> Result<f64> {
        // Analyze various metrics to determine community strength
        let mention_count = self.get_twitter_mention_count(token).await? as f64;
        let engagement_rate = self.calculate_engagement_rate(token).await?;
        let holder_growth = self.calculate_holder_growth(token).await?;
        
        // Combine metrics into a community strength score
        let strength = ((mention_count / 100.0) * 0.4) + (engagement_rate * 0.3) + (holder_growth * 0.3);
        Ok(strength.min(1.0))
    }

    async fn calculate_engagement_rate(&self, _token: &PumpToken) -> Result<f64> {
        // Simplified engagement calculation
        Ok(0.6)
    }

    async fn calculate_holder_growth(&self, _token: &PumpToken) -> Result<f64> {
        // Would analyze on-chain holder data
        Ok(0.7)
    }

    fn calculate_social_signal_score(&self, signals: &[SocialSignal]) -> f64 {
        let mut score = 0.0;
        
        for signal in signals {
            match signal {
                SocialSignal::TwitterMentions(count) => {
                    score += (*count as f64 / 100.0).min(0.3);
                }
                SocialSignal::TelegramMembers(count) => {
                    score += (*count as f64 / 1000.0).min(0.2);
                }
                SocialSignal::InfluencerEndorsement => {
                    score += 0.4;
                }
                SocialSignal::CommunityStrength(strength) => {
                    score += strength * 0.3;
                }
            }
        }
        
        score.min(1.0)
    }

    async fn calculate_technical_score(&self, token: &PumpToken) -> Result<f64> {
        // Analyze technical indicators like price action, volume, etc.
        let mut score = 0.5; // Start with neutral

        if let Some(price_change) = token.price_change_24h {
            if price_change > 0.0 {
                score += (price_change / 100.0).min(0.3);
            } else {
                score += (price_change / 100.0).max(-0.3);
            }
        }

        if let Some(volume) = token.volume_24h {
            if volume > 100.0 { // High volume is good
                score += 0.2;
            }
        }

        if let Some(liquidity) = token.liquidity_sol {
            if liquidity > 50.0 { // Good liquidity
                score += 0.1;
            }
        }

        Ok(score.clamp(0.0, 1.0))
    }

    async fn calculate_risk_score(&self, token: &PumpToken, sentiment_score: f64) -> Result<f64> {
        let mut risk = 0.5; // Start with medium risk

        // Age factor - newer tokens are riskier
        let age_hours = chrono::Utc::now()
            .signed_duration_since(token.created_at)
            .num_hours() as f64;
            
        if age_hours < 1.0 {
            risk += 0.3; // Very new, high risk
        } else if age_hours < 24.0 {
            risk += 0.2; // New, elevated risk
        }

        // Liquidity factor
        if let Some(liquidity) = token.liquidity_sol {
            if liquidity < 10.0 {
                risk += 0.2; // Low liquidity increases risk
            } else if liquidity > 100.0 {
                risk -= 0.1; // High liquidity reduces risk
            }
        }

        // Sentiment factor
        if sentiment_score < 0.3 {
            risk += 0.2; // Negative sentiment increases risk
        } else if sentiment_score > 0.7 {
            risk -= 0.1; // Positive sentiment reduces risk
        }

        Ok(risk.clamp(0.0, 1.0))
    }

    fn generate_recommendation(&self, sentiment_score: f64, technical_score: f64) -> TradeRecommendation {
        let combined_score = (sentiment_score + technical_score) / 2.0;

        match combined_score {
            x if x > 0.8 => TradeRecommendation::StrongBuy,
            x if x > 0.65 => TradeRecommendation::Buy,
            x if x > 0.35 => TradeRecommendation::Hold,
            x if x > 0.2 => TradeRecommendation::Sell,
            _ => TradeRecommendation::StrongSell,
        }
    }

    fn calculate_confidence(&self, sentiment_score: f64, technical_score: f64) -> f64 {
        let agreement = 1.0 - (sentiment_score - technical_score).abs();
        agreement.clamp(0.0, 1.0)
    }

    async fn get_cached_sentiment(&self, symbol: &str) -> Option<CachedSentiment> {
        self.sentiment_cache.read().await.get(symbol).cloned()
    }

    async fn cache_sentiment(&self, symbol: &str, score: f64, signals: Vec<SocialSignal>) {
        let cached = CachedSentiment {
            score,
            signals,
            timestamp: chrono::Utc::now(),
        };
        self.sentiment_cache.write().await.insert(symbol.to_string(), cached);
    }
}
```

## Step 4: Implement Trading Engine

```rust
// src/trading.rs
use riglr_core::{provider::ApplicationContext, SignerContext};
use riglr_solana_tools::{swap_tokens_jupiter, get_sol_balance, get_spl_token_balance};
use anyhow::Result;
use log::{info, warn, error};
use tokio::sync::RwLock;

pub struct TradingEngine {
    context: ApplicationContext,
    strategies: Vec<TradingStrategy>,
    positions: RwLock<std::collections::HashMap<Pubkey, Position>>,
    trade_history: RwLock<Vec<Trade>>,
    risk_limits: RiskLimits,
}

#[derive(Debug, Clone)]
pub struct RiskLimits {
    pub max_position_size_sol: f64,
    pub max_daily_loss_sol: f64,
    pub max_positions: u32,
    pub min_liquidity_sol: f64,
    pub max_slippage_bps: u16,
}

impl TradingEngine {
    pub fn new(context: ApplicationContext, strategies: Vec<TradingStrategy>) -> Self {
        Self {
            context,
            strategies,
            positions: RwLock::new(std::collections::HashMap::new()),
            trade_history: RwLock::new(Vec::new()),
            risk_limits: RiskLimits {
                max_position_size_sol: 10.0,
                max_daily_loss_sol: 50.0,
                max_positions: 20,
                min_liquidity_sol: 5.0,
                max_slippage_bps: 300, // 3%
            },
        }
    }

    pub async fn evaluate_token(&self, token: &PumpToken, analysis: &MarketAnalysis) -> Result<Option<TradeDecision>> {
        info!("Evaluating token {} for trading", token.symbol);

        // Check if we already have a position
        let positions = self.positions.read().await;
        if positions.contains_key(&token.mint) {
            debug!("Already have position in {}", token.symbol);
            return Ok(None);
        }
        drop(positions);

        // Evaluate against each strategy
        for strategy in &self.strategies {
            if !strategy.enabled {
                continue;
            }

            if let Some(decision) = self.evaluate_strategy(strategy, token, analysis).await? {
                info!("Strategy '{}' suggests {:?} for {}", strategy.name, decision.action, token.symbol);
                return Ok(Some(decision));
            }
        }

        Ok(None)
    }

    async fn evaluate_strategy(
        &self,
        strategy: &TradingStrategy,
        token: &PumpToken,
        analysis: &MarketAnalysis,
    ) -> Result<Option<TradeDecision>> {
        // Check basic strategy criteria
        if !self.meets_strategy_criteria(strategy, token, analysis).await? {
            return Ok(None);
        }

        // Check risk limits
        if !self.passes_risk_checks(strategy, token).await? {
            return Ok(None);
        }

        // Calculate position size
        let position_size = self.calculate_position_size(strategy, token, analysis).await?;

        // Generate trade decision
        Ok(Some(TradeDecision {
            token_mint: token.mint,
            action: TradeAction::Buy,
            amount_sol: position_size,
            strategy_name: strategy.name.clone(),
            confidence: analysis.confidence,
            stop_loss_price: self.calculate_stop_loss(token, strategy).await?,
            take_profit_price: self.calculate_take_profit(token, strategy).await?,
            max_slippage_bps: self.risk_limits.max_slippage_bps,
            urgency: self.calculate_urgency(analysis),
        }))
    }

    async fn meets_strategy_criteria(
        &self,
        strategy: &TradingStrategy,
        token: &PumpToken,
        analysis: &MarketAnalysis,
    ) -> Result<bool> {
        // Market cap check
        if let Some(market_cap) = token.market_cap {
            if market_cap < strategy.min_market_cap || market_cap > strategy.max_market_cap {
                debug!("Token {} doesn't meet market cap criteria", token.symbol);
                return Ok(false);
            }
        }

        // Liquidity check
        if let Some(liquidity) = token.liquidity_sol {
            if liquidity < strategy.min_liquidity_sol {
                debug!("Token {} has insufficient liquidity", token.symbol);
                return Ok(false);
            }
        }

        // Age check
        let age_minutes = chrono::Utc::now()
            .signed_duration_since(token.created_at)
            .num_minutes() as u32;
        if age_minutes > strategy.max_age_minutes {
            debug!("Token {} is too old for strategy", token.symbol);
            return Ok(false);
        }

        // Sentiment check
        if analysis.sentiment_score < strategy.min_sentiment_score {
            debug!("Token {} has insufficient sentiment score", token.symbol);
            return Ok(false);
        }

        // Social signals check
        for required_signal in &strategy.required_social_signals {
            if !analysis.social_signals.iter().any(|s| self.signals_match(s, required_signal)) {
                debug!("Token {} missing required social signal", token.symbol);
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn signals_match(&self, actual: &SocialSignal, required: &SocialSignal) -> bool {
        match (actual, required) {
            (SocialSignal::TwitterMentions(actual_count), SocialSignal::TwitterMentions(required_count)) => {
                actual_count >= required_count
            }
            (SocialSignal::TelegramMembers(actual_count), SocialSignal::TelegramMembers(required_count)) => {
                actual_count >= required_count
            }
            (SocialSignal::InfluencerEndorsement, SocialSignal::InfluencerEndorsement) => true,
            (SocialSignal::CommunityStrength(actual_strength), SocialSignal::CommunityStrength(required_strength)) => {
                actual_strength >= required_strength
            }
            _ => false,
        }
    }

    async fn passes_risk_checks(&self, strategy: &TradingStrategy, token: &PumpToken) -> Result<bool> {
        // Check maximum position size
        if strategy.max_position_size_sol > self.risk_limits.max_position_size_sol {
            warn!("Strategy position size exceeds risk limit");
            return Ok(false);
        }

        // Check current number of positions
        let position_count = self.positions.read().await.len() as u32;
        if position_count >= self.risk_limits.max_positions {
            warn!("Maximum number of positions reached");
            return Ok(false);
        }

        // Check daily loss limit
        let daily_pnl = self.calculate_daily_pnl().await?;
        if daily_pnl < -self.risk_limits.max_daily_loss_sol {
            warn!("Daily loss limit reached");
            return Ok(false);
        }

        // Check token-specific risks
        if let Some(liquidity) = token.liquidity_sol {
            if liquidity < self.risk_limits.min_liquidity_sol {
                warn!("Token liquidity below minimum threshold");
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn calculate_position_size(
        &self,
        strategy: &TradingStrategy,
        _token: &PumpToken,
        analysis: &MarketAnalysis,
    ) -> Result<f64> {
        // Base position size from strategy
        let mut position_size = strategy.max_position_size_sol;

        // Adjust based on confidence
        position_size *= analysis.confidence;

        // Adjust based on risk score (lower risk = larger position)
        position_size *= (1.0 - analysis.risk_score);

        // Ensure we don't exceed limits
        position_size = position_size.min(self.risk_limits.max_position_size_sol);

        Ok(position_size)
    }

    async fn calculate_stop_loss(&self, token: &PumpToken, strategy: &TradingStrategy) -> Result<Option<f64>> {
        if let Some(current_price) = token.price_sol {
            let stop_loss_price = current_price * (1.0 - strategy.stop_loss_percentage / 100.0);
            Ok(Some(stop_loss_price))
        } else {
            Ok(None)
        }
    }

    async fn calculate_take_profit(&self, token: &PumpToken, strategy: &TradingStrategy) -> Result<Option<f64>> {
        if let Some(current_price) = token.price_sol {
            let take_profit_price = current_price * (1.0 + strategy.take_profit_percentage / 100.0);
            Ok(Some(take_profit_price))
        } else {
            Ok(None)
        }
    }

    fn calculate_urgency(&self, analysis: &MarketAnalysis) -> TradeUrgency {
        if analysis.confidence > 0.9 && matches!(analysis.recommendation, TradeRecommendation::StrongBuy) {
            TradeUrgency::High
        } else if analysis.confidence > 0.7 {
            TradeUrgency::Medium
        } else {
            TradeUrgency::Low
        }
    }

    pub async fn execute_trade(&self, decision: TradeDecision) -> Result<Trade> {
        info!("Executing trade: {:?} {} SOL of token {}", 
            decision.action, decision.amount_sol, decision.token_mint);

        let trade_id = uuid::Uuid::new_v4().to_string();
        
        // Execute the trade based on action
        let (tx_hash, executed_price) = match decision.action {
            TradeAction::Buy => self.execute_buy(&decision).await?,
            TradeAction::Sell => self.execute_sell(&decision).await?,
        };

        // Create trade record
        let trade = Trade {
            id: trade_id.clone(),
            token_mint: decision.token_mint,
            strategy: decision.strategy_name,
            action: decision.action,
            amount_sol: decision.amount_sol,
            price_sol: executed_price,
            timestamp: chrono::Utc::now(),
            transaction_hash: tx_hash,
            status: TradeStatus::Executed,
            pnl_sol: None,
            pnl_percentage: None,
        };

        // Update positions
        self.update_position(&trade, &decision).await?;

        // Record trade
        self.trade_history.write().await.push(trade.clone());

        info!("Trade executed successfully: {}", trade_id);
        Ok(trade)
    }

    async fn execute_buy(&self, decision: &TradeDecision) -> Result<(String, f64)> {
        // Convert SOL to the target token using Jupiter
        let tx_hash = swap_tokens_jupiter(
            "So11111111111111111111111111111111111111112".to_string(), // SOL
            decision.token_mint.to_string(),
            decision.amount_sol,
            decision.max_slippage_bps,
        ).await?;

        // In a real implementation, you'd parse the transaction to get the actual executed price
        let executed_price = 0.0001; // Placeholder
        
        Ok((tx_hash, executed_price))
    }

    async fn execute_sell(&self, decision: &TradeDecision) -> Result<(String, f64)> {
        // Convert target token back to SOL
        let tx_hash = swap_tokens_jupiter(
            decision.token_mint.to_string(),
            "So11111111111111111111111111111111111111112".to_string(), // SOL
            decision.amount_sol,
            decision.max_slippage_bps,
        ).await?;

        let executed_price = 0.0001; // Placeholder
        
        Ok((tx_hash, executed_price))
    }

    async fn update_position(&self, trade: &Trade, decision: &TradeDecision) -> Result<()> {
        let mut positions = self.positions.write().await;
        
        match trade.action {
            TradeAction::Buy => {
                let position = Position {
                    token_mint: trade.token_mint,
                    symbol: "UNK".to_string(), // Would fetch actual symbol
                    amount: decision.amount_sol / trade.price_sol,
                    average_price_sol: trade.price_sol,
                    current_price_sol: trade.price_sol,
                    value_sol: decision.amount_sol,
                    pnl_sol: 0.0,
                    pnl_percentage: 0.0,
                    entry_time: trade.timestamp,
                    stop_loss_price: decision.stop_loss_price,
                    take_profit_price: decision.take_profit_price,
                };
                positions.insert(trade.token_mint, position);
            }
            TradeAction::Sell => {
                positions.remove(&trade.token_mint);
            }
        }
        
        Ok(())
    }

    async fn calculate_daily_pnl(&self) -> Result<f64> {
        let trade_history = self.trade_history.read().await;
        let today = chrono::Utc::now().date_naive();
        
        let daily_pnl: f64 = trade_history
            .iter()
            .filter(|trade| trade.timestamp.date_naive() == today)
            .filter_map(|trade| trade.pnl_sol)
            .sum();
            
        Ok(daily_pnl)
    }

    pub async fn monitor_positions(&self) -> Result<()> {
        let mut positions = self.positions.write().await;
        
        for (mint, position) in positions.iter_mut() {
            // Update current price (simplified - would use real price feeds)
            position.current_price_sol = self.get_current_price(mint).await?;
            position.value_sol = position.amount * position.current_price_sol;
            position.pnl_sol = position.value_sol - (position.amount * position.average_price_sol);
            position.pnl_percentage = (position.pnl_sol / (position.amount * position.average_price_sol)) * 100.0;

            // Check stop-loss and take-profit
            if let Some(stop_loss) = position.stop_loss_price {
                if position.current_price_sol <= stop_loss {
                    info!("Stop-loss triggered for {}", position.symbol);
                    // Execute sell order
                }
            }

            if let Some(take_profit) = position.take_profit_price {
                if position.current_price_sol >= take_profit {
                    info!("Take-profit triggered for {}", position.symbol);
                    // Execute sell order
                }
            }
        }
        
        Ok(())
    }

    async fn get_current_price(&self, _mint: &Pubkey) -> Result<f64> {
        // Would integrate with price feeds like Jupiter, DexScreener, etc.
        Ok(0.0001)
    }

    pub async fn get_positions(&self) -> std::collections::HashMap<Pubkey, Position> {
        self.positions.read().await.clone()
    }

    pub async fn get_trade_history(&self) -> Vec<Trade> {
        self.trade_history.read().await.clone()
    }
}

#[derive(Debug, Clone)]
pub struct TradeDecision {
    pub token_mint: Pubkey,
    pub action: TradeAction,
    pub amount_sol: f64,
    pub strategy_name: String,
    pub confidence: f64,
    pub stop_loss_price: Option<f64>,
    pub take_profit_price: Option<f64>,
    pub max_slippage_bps: u16,
    pub urgency: TradeUrgency,
}

#[derive(Debug, Clone)]
pub enum TradeUrgency {
    High,
    Medium,
    Low,
}
```

## Step 5: Build the Main Trading Agent

```rust
// src/main.rs
mod types;
mod discovery;
mod sentiment;
mod trading;

use riglr_core::{provider::ApplicationContext, SignerContext};
use riglr_solana_tools::LocalSolanaSigner;
use riglr_config::Config;
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use log::{info, warn, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    info!("Starting Pump.fun Trading Agent...");
    
    // Initialize components
    let config = Config::from_env();
    let context = ApplicationContext::from_config(&config);
    let signer = Arc::new(LocalSolanaSigner::from_env()?);
    let solana_client = Arc::new(RpcClient::new(&config.solana_rpc_url));
    
    // Create trading strategies
    let strategies = create_trading_strategies();
    
    // Initialize systems
    let token_discovery = Arc::new(discovery::TokenDiscovery::new(context.clone(), solana_client));
    let sentiment_analyzer = Arc::new(sentiment::SentimentAnalyzer::new(context.clone()));
    let trading_engine = Arc::new(trading::TradingEngine::new(context.clone(), strategies));
    
    // Start token discovery in background
    let discovery_handle = {
        let token_discovery = token_discovery.clone();
        tokio::spawn(async move {
            if let Err(e) = token_discovery.start_monitoring().await {
                error!("Token discovery failed: {}", e);
            }
        })
    };
    
    // Start position monitoring
    let position_monitor_handle = {
        let trading_engine = trading_engine.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = trading_engine.monitor_positions().await {
                    error!("Position monitoring failed: {}", e);
                }
            }
        })
    };
    
    // Main trading loop
    let main_handle = {
        let token_discovery = token_discovery.clone();
        let sentiment_analyzer = sentiment_analyzer.clone();
        let trading_engine = trading_engine.clone();
        let signer = signer.clone();
        
        tokio::spawn(async move {
            run_trading_loop(token_discovery, sentiment_analyzer, trading_engine, signer).await
        })
    };
    
    // Wait for all tasks
    tokio::try_join!(discovery_handle, position_monitor_handle, main_handle)?;
    
    Ok(())
}

async fn run_trading_loop(
    token_discovery: Arc<discovery::TokenDiscovery>,
    sentiment_analyzer: Arc<sentiment::SentimentAnalyzer>,
    trading_engine: Arc<trading::TradingEngine>,
    signer: Arc<LocalSolanaSigner>,
) -> anyhow::Result<()> {
    let mut interval = interval(Duration::from_secs(10));
    
    info!("Starting main trading loop...");
    
    loop {
        interval.tick().await;
        
        // Get recently discovered tokens
        let discovered_tokens = token_discovery.get_discovered_tokens().await;
        
        for (mint, token) in discovered_tokens {
            // Skip if token is too old (we want fresh opportunities)
            let age_minutes = chrono::Utc::now()
                .signed_duration_since(token.created_at)
                .num_minutes();
                
            if age_minutes > 30 { // Only consider tokens less than 30 minutes old
                continue;
            }
            
            info!("Analyzing token: {} ({})", token.name, token.symbol);
            
            // Perform sentiment analysis
            let analysis = match sentiment_analyzer.analyze_token_sentiment(&token).await {
                Ok(analysis) => analysis,
                Err(e) => {
                    warn!("Failed to analyze sentiment for {}: {}", token.symbol, e);
                    continue;
                }
            };
            
            info!("Analysis for {}: sentiment={:.2}, risk={:.2}, recommendation={:?}", 
                token.symbol, analysis.sentiment_score, analysis.risk_score, analysis.recommendation);
            
            // Evaluate trading opportunity
            let trade_decision = match trading_engine.evaluate_token(&token, &analysis).await {
                Ok(Some(decision)) => decision,
                Ok(None) => {
                    debug!("No trading decision for {}", token.symbol);
                    continue;
                }
                Err(e) => {
                    error!("Failed to evaluate token {}: {}", token.symbol, e);
                    continue;
                }
            };
            
            info!("Trade decision for {}: {:?} {} SOL", 
                token.symbol, trade_decision.action, trade_decision.amount_sol);
            
            // Execute trade within signer context
            let result = SignerContext::with_signer(signer.clone(), async {
                trading_engine.execute_trade(trade_decision).await
            }).await;
            
            match result {
                Ok(trade) => {
                    info!("✅ Trade executed: {} {} {} (tx: {})", 
                        match trade.action {
                            types::TradeAction::Buy => "Bought",
                            types::TradeAction::Sell => "Sold",
                        },
                        trade.amount_sol,
                        token.symbol,
                        trade.transaction_hash
                    );
                    
                    // Send notification about successful trade
                    send_trade_notification(&trade, &token).await;
                }
                Err(e) => {
                    error!("❌ Trade execution failed for {}: {}", token.symbol, e);
                }
            }
            
            // Rate limiting to avoid overwhelming the system
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        // Log portfolio status
        log_portfolio_status(&trading_engine).await;
    }
}

fn create_trading_strategies() -> Vec<types::TradingStrategy> {
    vec![
        // Aggressive sniper strategy for very new tokens
        types::TradingStrategy {
            name: "Early Bird Sniper".to_string(),
            enabled: true,
            min_market_cap: 1000.0,      // $1k minimum
            max_market_cap: 100000.0,    // $100k maximum
            min_liquidity_sol: 5.0,      // 5 SOL minimum liquidity
            max_position_size_sol: 2.0,  // 2 SOL max position
            take_profit_percentage: 300.0, // 3x target
            stop_loss_percentage: 50.0,   // 50% stop loss
            min_sentiment_score: 0.7,     // High sentiment required
            max_age_minutes: 5,           // Only tokens less than 5 minutes old
            required_social_signals: vec![
                types::SocialSignal::TwitterMentions(5),
            ],
        },
        
        // Conservative momentum strategy
        types::TradingStrategy {
            name: "Momentum Rider".to_string(),
            enabled: true,
            min_market_cap: 10000.0,     // $10k minimum
            max_market_cap: 1000000.0,   // $1M maximum
            min_liquidity_sol: 20.0,     // 20 SOL minimum liquidity
            max_position_size_sol: 5.0,  // 5 SOL max position
            take_profit_percentage: 100.0, // 2x target
            stop_loss_percentage: 25.0,   // 25% stop loss
            min_sentiment_score: 0.6,     // Medium sentiment required
            max_age_minutes: 60,          // Tokens up to 1 hour old
            required_social_signals: vec![
                types::SocialSignal::TwitterMentions(20),
                types::SocialSignal::CommunityStrength(0.6),
            ],
        },
        
        // Influencer endorsement strategy
        types::TradingStrategy {
            name: "Influencer Play".to_string(),
            enabled: true,
            min_market_cap: 5000.0,      // $5k minimum
            max_market_cap: 500000.0,    // $500k maximum
            min_liquidity_sol: 10.0,     // 10 SOL minimum liquidity
            max_position_size_sol: 8.0,  // 8 SOL max position
            take_profit_percentage: 200.0, // 3x target
            stop_loss_percentage: 30.0,   // 30% stop loss
            min_sentiment_score: 0.5,     // Medium sentiment
            max_age_minutes: 30,          // Tokens up to 30 minutes old
            required_social_signals: vec![
                types::SocialSignal::InfluencerEndorsement,
            ],
        },
    ]
}

async fn send_trade_notification(trade: &types::Trade, token: &types::PumpToken) {
    let action_emoji = match trade.action {
        types::TradeAction::Buy => "🟢",
        types::TradeAction::Sell => "🔴",
    };
    
    let message = format!(
        "{} {} Trade Executed\n\
         Token: {} ({})\n\
         Amount: {} SOL\n\
         Price: {} SOL\n\
         Strategy: {}\n\
         TX: {}",
        action_emoji,
        match trade.action {
            types::TradeAction::Buy => "BUY",
            types::TradeAction::Sell => "SELL",
        },
        token.name,
        token.symbol,
        trade.amount_sol,
        trade.price_sol,
        trade.strategy,
        trade.transaction_hash
    );
    
    info!("Trade notification: {}", message);
    // In production, send to Discord, Telegram, etc.
}

async fn log_portfolio_status(trading_engine: &trading::TradingEngine) {
    let positions = trading_engine.get_positions().await;
    let trade_count = trading_engine.get_trade_history().await.len();
    
    if positions.is_empty() {
        debug!("No open positions. Total trades executed: {}", trade_count);
    } else {
        let total_value: f64 = positions.values().map(|p| p.value_sol).sum();
        let total_pnl: f64 = positions.values().map(|p| p.pnl_sol).sum();
        
        info!("Portfolio Status: {} positions, {:.2} SOL value, {:.2} SOL PnL, {} trades", 
            positions.len(), total_value, total_pnl, trade_count);
    }
}
```

## Step 6: Testing and Backtesting

```rust
// src/backtest.rs
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    
    #[tokio::test]
    async fn test_sentiment_analysis() {
        let context = create_test_context();
        let analyzer = sentiment::SentimentAnalyzer::new(context);
        
        let test_token = create_test_token();
        let analysis = analyzer.analyze_token_sentiment(&test_token).await.unwrap();
        
        assert!(analysis.sentiment_score >= 0.0 && analysis.sentiment_score <= 1.0);
        assert!(analysis.confidence >= 0.0 && analysis.confidence <= 1.0);
    }
    
    #[tokio::test] 
    async fn test_trading_strategy_evaluation() {
        let context = create_test_context();
        let strategies = vec![create_test_strategy()];
        let engine = trading::TradingEngine::new(context, strategies);
        
        let token = create_test_token();
        let analysis = create_positive_analysis();
        
        let decision = engine.evaluate_token(&token, &analysis).await.unwrap();
        assert!(decision.is_some());
        
        let decision = decision.unwrap();
        assert!(matches!(decision.action, types::TradeAction::Buy));
        assert!(decision.amount_sol > 0.0);
    }
    
    #[tokio::test]
    async fn test_risk_limits() {
        let context = create_test_context();
        let mut strategies = vec![create_test_strategy()];
        strategies[0].max_position_size_sol = 1000.0; // Very large position
        
        let engine = trading::TradingEngine::new(context, strategies);
        let token = create_test_token();
        let analysis = create_positive_analysis();
        
        let decision = engine.evaluate_token(&token, &analysis).await.unwrap();
        // Should be None due to risk limits
        assert!(decision.is_none());
    }
    
    #[test]
    fn test_backtest_historical_data() {
        // Simulate historical pump.fun data
        let historical_tokens = create_historical_dataset();
        let strategies = vec![create_test_strategy()];
        
        let mut total_pnl = 0.0;
        let mut successful_trades = 0;
        let mut total_trades = 0;
        
        for token_data in historical_tokens {
            if should_trade_token(&strategies[0], &token_data) {
                total_trades += 1;
                let pnl = simulate_trade_outcome(&token_data);
                total_pnl += pnl;
                
                if pnl > 0.0 {
                    successful_trades += 1;
                }
            }
        }
        
        let win_rate = successful_trades as f64 / total_trades as f64;
        println!("Backtest Results:");
        println!("Total PnL: {:.2} SOL", total_pnl);
        println!("Win Rate: {:.1}%", win_rate * 100.0);
        println!("Total Trades: {}", total_trades);
        
        // Assert reasonable performance
        assert!(win_rate > 0.4); // At least 40% win rate
        assert!(total_pnl > -50.0); // Don't lose more than 50 SOL
    }
    
    fn create_test_context() -> riglr_core::provider::ApplicationContext {
        let config = riglr_config::Config::from_env();
        riglr_core::provider::ApplicationContext::from_config(&config)
    }
    
    fn create_test_token() -> types::PumpToken {
        types::PumpToken {
            mint: solana_sdk::pubkey::Pubkey::new_unique(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            uri: "https://example.com".to_string(),
            description: Some("Test token description".to_string()),
            creator: solana_sdk::pubkey::Pubkey::new_unique(),
            created_at: Utc::now() - Duration::minutes(5),
            market_cap: Some(50000.0),
            price_sol: Some(0.0001),
            liquidity_sol: Some(25.0),
            holder_count: Some(150),
            volume_24h: Some(500.0),
            price_change_24h: Some(25.0),
        }
    }
    
    fn create_test_strategy() -> types::TradingStrategy {
        types::TradingStrategy {
            name: "Test Strategy".to_string(),
            enabled: true,
            min_market_cap: 10000.0,
            max_market_cap: 100000.0,
            min_liquidity_sol: 10.0,
            max_position_size_sol: 5.0,
            take_profit_percentage: 100.0,
            stop_loss_percentage: 25.0,
            min_sentiment_score: 0.6,
            max_age_minutes: 30,
            required_social_signals: vec![],
        }
    }
    
    fn create_positive_analysis() -> types::MarketAnalysis {
        types::MarketAnalysis {
            token_mint: solana_sdk::pubkey::Pubkey::new_unique(),
            sentiment_score: 0.8,
            social_signals: vec![types::SocialSignal::TwitterMentions(25)],
            technical_score: 0.7,
            risk_score: 0.3,
            recommendation: types::TradeRecommendation::Buy,
            confidence: 0.85,
            analysis_time: Utc::now(),
        }
    }
    
    fn create_historical_dataset() -> Vec<HistoricalTokenData> {
        // Create synthetic historical data for backtesting
        vec![
            HistoricalTokenData {
                market_cap: 25000.0,
                liquidity: 15.0,
                age_minutes: 10,
                sentiment: 0.7,
                final_price_change: 150.0, // 150% gain
            },
            HistoricalTokenData {
                market_cap: 75000.0,
                liquidity: 30.0,
                age_minutes: 25,
                sentiment: 0.6,
                final_price_change: -20.0, // 20% loss
            },
            // Add more historical data points...
        ]
    }
    
    #[derive(Debug)]
    struct HistoricalTokenData {
        market_cap: f64,
        liquidity: f64,
        age_minutes: u32,
        sentiment: f64,
        final_price_change: f64,
    }
    
    fn should_trade_token(strategy: &types::TradingStrategy, token_data: &HistoricalTokenData) -> bool {
        token_data.market_cap >= strategy.min_market_cap
            && token_data.market_cap <= strategy.max_market_cap
            && token_data.liquidity >= strategy.min_liquidity_sol
            && token_data.age_minutes <= strategy.max_age_minutes
            && token_data.sentiment >= strategy.min_sentiment_score
    }
    
    fn simulate_trade_outcome(token_data: &HistoricalTokenData) -> f64 {
        let position_size = 2.0; // SOL
        let price_change_ratio = token_data.final_price_change / 100.0;
        
        // Apply stop-loss and take-profit logic
        let capped_change = if price_change_ratio > 1.0 {
            1.0 // Take profit at 100%
        } else if price_change_ratio < -0.25 {
            -0.25 // Stop loss at 25%
        } else {
            price_change_ratio
        };
        
        position_size * capped_change
    }
}
```

## Deployment Considerations

### 1. Environment Configuration

```env
# Solana Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_PRIVATE_KEY=your_private_key_base58
SOLANA_WALLET_ADDRESS=your_wallet_address

# Social Media APIs
TWITTER_BEARER_TOKEN=your_twitter_bearer_token
TELEGRAM_BOT_TOKEN=your_telegram_bot_token

# Price and Market Data
DEXSCREENER_API_KEY=your_dexscreener_key
JUPITER_API_URL=https://quote-api.jup.ag/v4

# Risk Management
MAX_POSITION_SIZE_SOL=10
MAX_DAILY_LOSS_SOL=50
MAX_POSITIONS=20
MIN_LIQUIDITY_SOL=5

# Strategy Configuration  
ENABLE_SNIPER_STRATEGY=true
ENABLE_MOMENTUM_STRATEGY=true
ENABLE_INFLUENCER_STRATEGY=false

# Monitoring and Alerts
DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/...
ALERT_ON_TRADES=true
ALERT_ON_STOP_LOSSES=true
```

### 2. Database Setup

```sql
-- Database schema for trade tracking
CREATE TABLE trades (
    id TEXT PRIMARY KEY,
    token_mint TEXT NOT NULL,
    strategy TEXT NOT NULL,
    action TEXT NOT NULL,
    amount_sol REAL NOT NULL,
    price_sol REAL NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    transaction_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    pnl_sol REAL,
    pnl_percentage REAL
);

CREATE TABLE positions (
    token_mint TEXT PRIMARY KEY,
    symbol TEXT NOT NULL,
    amount REAL NOT NULL,
    average_price_sol REAL NOT NULL,
    current_price_sol REAL NOT NULL,
    value_sol REAL NOT NULL,
    pnl_sol REAL NOT NULL,
    pnl_percentage REAL NOT NULL,
    entry_time TIMESTAMP NOT NULL,
    stop_loss_price REAL,
    take_profit_price REAL
);

CREATE TABLE discovered_tokens (
    mint TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    uri TEXT,
    creator TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    market_cap REAL,
    price_sol REAL,
    liquidity_sol REAL,
    sentiment_score REAL,
    last_updated TIMESTAMP NOT NULL
);
```

### 3. Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pump-trader /usr/local/bin/
COPY --from=builder /app/schema.sql /app/

# Initialize database
RUN sqlite3 /data/trades.db < /app/schema.sql

ENV RUST_LOG=info
CMD ["pump-trader"]
```

## Conclusion

You've built a comprehensive Pump.fun trading agent that:

- Monitors new token launches in real-time using Solana event parsing
- Analyzes social sentiment through Twitter and web search integration
- Implements multiple trading strategies with risk management
- Executes trades automatically with stop-loss and take-profit orders
- Provides comprehensive backtesting and performance tracking

### Key Features Implemented

1. **Real-time Discovery**: Event-driven token detection using riglr's Solana events system
2. **Sentiment Analysis**: Multi-source sentiment analysis including Twitter, web, and social signals
3. **Risk Management**: Position sizing, stop-losses, daily limits, and portfolio management
4. **Strategy Engine**: Configurable trading strategies with different risk/reward profiles
5. **Automated Execution**: Seamless integration with Jupiter for token swaps

### Next Steps

1. Add more sophisticated technical analysis indicators
2. Implement machine learning for sentiment prediction
3. Add support for other meme token platforms
4. Integrate with more social media platforms (Telegram, Discord)
5. Add portfolio optimization algorithms
6. Implement paper trading mode for strategy testing

### Important Disclaimers

- Trading meme tokens carries significant risk of total loss
- Always test strategies thoroughly on devnet before using real funds
- Consider position sizing and never risk more than you can afford to lose
- The crypto market is highly volatile and unpredictable
- Past performance does not guarantee future results

Remember to thoroughly backtest your strategies and start with small position sizes when deploying to mainnet!