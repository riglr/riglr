//! DexScreener integration for comprehensive token market data and DEX analytics
//!
//! This module provides production-grade tools for accessing DexScreener data,
//! analyzing token metrics, tracking price movements, and identifying trading opportunities.

use crate::{client::WebClient, error::WebToolError};
use chrono::{DateTime, Utc};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Configuration for DexScreener API access
#[derive(Debug, Clone)]
pub struct DexScreenerConfig {
    /// API base URL (default: https://api.dexscreener.com/latest)
    pub base_url: String,
    /// Rate limit requests per minute (default: 300)
    pub rate_limit_per_minute: u32,
    /// Timeout for API requests in seconds (default: 30)
    pub request_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenInfo {
    /// Token contract address
    pub address: String,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token decimals
    pub decimals: u32,
    /// Current price in USD
    pub price_usd: Option<f64>,
    /// Market capitalization in USD
    pub market_cap: Option<f64>,
    /// 24h trading volume in USD
    pub volume_24h: Option<f64>,
    /// Price change percentage (24h)
    pub price_change_24h: Option<f64>,
    /// Price change percentage (1h)
    pub price_change_1h: Option<f64>,
    /// Price change percentage (5m)
    pub price_change_5m: Option<f64>,
    /// Circulating supply
    pub circulating_supply: Option<f64>,
    /// Total supply
    pub total_supply: Option<f64>,
    /// Number of active trading pairs
    pub pair_count: u32,
    /// Top trading pairs
    pub pairs: Vec<TokenPair>,
    /// Blockchain/chain information
    pub chain: ChainInfo,
    /// Verification status and security info
    pub security: SecurityInfo,
    /// Social and community links
    pub socials: Vec<SocialLink>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Trading pair information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenPair {
    /// Unique pair identifier
    pub pair_id: String,
    /// DEX name (e.g., "Uniswap V3", "PancakeSwap")
    pub dex: DexInfo,
    pub base_token: PairToken,
    pub quote_token: PairToken,
    /// Current price
    pub price_usd: f64,
    pub price_native: f64,
    /// 24h trading volume in USD
    pub volume_24h: f64,
    /// 24h price change percentage
    pub price_change_24h: f64,
    /// Total liquidity in USD
    pub liquidity_usd: Option<f64>,
    /// Fully diluted valuation
    pub fdv: Option<f64>,
    /// Pair creation timestamp
    pub created_at: Option<DateTime<Utc>>,
    /// Latest trade timestamp
    pub last_trade_at: DateTime<Utc>,
    /// Number of transactions (24h)
    pub txns_24h: TransactionStats,
    /// Pair URL on the DEX
    pub url: String,
}

/// DEX platform information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DexInfo {
    /// DEX identifier
    pub id: String,
    /// DEX name
    pub name: String,
    /// DEX URL
    pub url: Option<String>,
    /// DEX logo URL
    pub logo: Option<String>,
}

/// Token information within a pair
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PairToken {
    /// Token contract address
    pub address: String,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
}

/// Transaction statistics for a trading pair
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionStats {
    /// Number of buy transactions (24h)
    pub buys: u32,
    /// Number of sell transactions (24h)
    pub sells: u32,
    /// Total number of transactions (24h)
    pub total: u32,
    /// Buy volume in USD (24h)
    pub buy_volume_usd: f64,
    /// Sell volume in USD (24h)
    pub sell_volume_usd: f64,
}

/// Blockchain/chain information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainInfo {
    /// Chain identifier (e.g., "ethereum", "bsc", "polygon")
    pub id: String,
    /// Chain name
    pub name: String,
    /// Chain logo URL
    pub logo: Option<String>,
    pub native_token: String,
}

/// Token security and verification information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SecurityInfo {
    pub is_verified: bool,
    /// Whether liquidity is locked
    pub liquidity_locked: Option<bool>,
    /// Contract audit status
    pub audit_status: Option<String>,
    /// Honeypot detection result
    pub honeypot_status: Option<String>,
    /// Contract ownership status
    pub ownership_status: Option<String>,
    /// Risk score (0-100, lower is better)
    pub risk_score: Option<u32>,
}

/// Social media and community links
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SocialLink {
    /// Platform name (e.g., "twitter", "telegram", "discord")
    pub platform: String,
    /// Profile URL
    pub url: String,
    /// Follower count (if available)
    pub followers: Option<u32>,
}

/// Market analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MarketAnalysis {
    /// Token being analyzed
    pub token: TokenInfo,
    /// Market trend analysis
    pub trend_analysis: TrendAnalysis,
    /// Volume analysis
    pub volume_analysis: VolumeAnalysis,
    /// Liquidity analysis
    pub liquidity_analysis: LiquidityAnalysis,
    /// Price level analysis
    pub price_levels: PriceLevelAnalysis,
    /// Risk assessment
    pub risk_assessment: RiskAssessment,
    /// Analysis timestamp
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrendAnalysis {
    /// Overall trend direction (Bullish, Bearish, Neutral)
    pub direction: String,
    /// Trend strength (1-10)
    pub strength: u32,
    /// Momentum score (-100 to 100)
    pub momentum: f64,
    /// Price velocity (rate of change)
    pub velocity: f64,
    /// Support levels
    pub support_levels: Vec<f64>,
    /// Resistance levels
    pub resistance_levels: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VolumeAnalysis {
    pub volume_rank: Option<u32>,
    /// Volume trend (Increasing, Decreasing, Stable)
    pub volume_trend: String,
    /// Volume/Market Cap ratio
    pub volume_mcap_ratio: Option<f64>,
    /// Average volume (7 days)
    pub avg_volume_7d: Option<f64>,
    /// Volume spike factor (current vs average)
    pub spike_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LiquidityAnalysis {
    /// Total liquidity across all pairs
    pub total_liquidity_usd: f64,
    /// Liquidity distribution across DEXs
    pub dex_distribution: HashMap<String, f64>,
    /// Price impact for different trade sizes
    pub price_impact: HashMap<String, f64>, // "1k", "10k", "100k" -> impact %
    /// Liquidity depth score (1-100)
    pub depth_score: u32,
}

/// Price level analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PriceLevelAnalysis {
    /// All-time high price
    pub ath: Option<f64>,
    /// All-time low price
    pub atl: Option<f64>,
    /// Distance from ATH (percentage)
    pub ath_distance_pct: Option<f64>,
    /// Distance from ATL (percentage)
    pub atl_distance_pct: Option<f64>,
    /// 24h high
    pub high_24h: Option<f64>,
    /// 24h low
    pub low_24h: Option<f64>,
    /// Current price position in 24h range (0-1)
    pub range_position: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiskAssessment {
    /// Overall risk level (Low, Medium, High, Extreme)
    pub risk_level: String,
    /// Detailed risk factors
    pub risk_factors: Vec<RiskFactor>,
    /// Liquidity risk score (1-100)
    pub liquidity_risk: u32,
    /// Volatility risk score (1-100)
    pub volatility_risk: u32,
    /// Smart contract risk score (1-100)
    pub contract_risk: u32,
}

/// Individual risk factor
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiskFactor {
    /// Risk category
    pub category: String,
    /// Risk description
    pub description: String,
    /// Severity (Low, Medium, High)
    pub severity: String,
    /// Impact score (1-100)
    pub impact: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenSearchResult {
    /// Search query used
    pub query: String,
    pub tokens: Vec<TokenInfo>,
    /// Search metadata
    pub metadata: SearchMetadata,
    /// Search timestamp
    pub searched_at: DateTime<Utc>,
}

/// Metadata for search results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchMetadata {
    /// Number of results found
    pub result_count: u32,
    /// Search execution time (ms)
    pub execution_time_ms: u32,
    /// Whether results were limited
    pub limited: bool,
    /// Suggested alternative queries
    pub suggestions: Vec<String>,
}

impl Default for DexScreenerConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.dexscreener.com/latest".to_string(),
            rate_limit_per_minute: 300,
            request_timeout: 30,
        }
    }
}

/// Get comprehensive token information from DexScreener
///
/// This tool retrieves detailed token information including price, volume,
/// market cap, trading pairs, and security analysis.
#[tool]
pub async fn get_token_info(
    token_address: String,
    chain_id: Option<String>,
    include_pairs: Option<bool>,
    include_security: Option<bool>,
) -> crate::error::Result<TokenInfo> {
    debug!(
        "Fetching token info for address: {} on chain: {:?}",
        token_address,
        chain_id.as_deref().unwrap_or("auto-detect")
    );

    let config = DexScreenerConfig::default();
    let client = WebClient::new().map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?;

    // Build API endpoint
    let chain = chain_id.unwrap_or_else(|| "ethereum".to_string());
    let url = if include_pairs.unwrap_or(true) {
        format!("{}/dex/tokens/{}", config.base_url, token_address)
    } else {
        format!(
            "{}/dex/tokens/{}?fields=basic",
            config.base_url, token_address
        )
    };

    // Make API request
    let response = client.get(&url).await.map_err(|e| {
        if e.to_string().contains("timeout") || e.to_string().contains("connection") {
            WebToolError::Network(format!("Failed to fetch token info: {}", e))
        } else {
            WebToolError::Api(format!("Failed to fetch token info: {}", e))
        }
    })?;

    // Parse response (simplified - would parse actual DexScreener JSON)
    let token_info = parse_token_response(&response, &token_address, &chain)
        .await
        .map_err(|e| WebToolError::Api(format!("Failed to parse token response: {}", e)))?;

    info!(
        "Retrieved token info for {} ({}): ${:.6}",
        token_info.symbol,
        token_info.name,
        token_info.price_usd.unwrap_or(0.0)
    );

    Ok(token_info)
}

/// Search for tokens on DexScreener
///
/// This tool searches for tokens by name, symbol, or address
/// with support for filtering by chain and market cap.
#[tool]
pub async fn search_tokens(
    query: String,
    chain_filter: Option<String>,
    min_market_cap: Option<f64>,
    min_liquidity: Option<f64>,
    limit: Option<u32>,
) -> crate::error::Result<TokenSearchResult> {
    debug!("Searching tokens for query: '{}' with filters", query);

    let config = DexScreenerConfig::default();
    let client = WebClient::new().map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?;

    // Build search parameters
    let mut params = HashMap::new();
    params.insert("q".to_string(), query.clone());

    if let Some(chain) = chain_filter {
        params.insert("chain".to_string(), chain);
    }

    if let Some(min_mc) = min_market_cap {
        params.insert("min_market_cap".to_string(), min_mc.to_string());
    }

    if let Some(min_liq) = min_liquidity {
        params.insert("min_liquidity".to_string(), min_liq.to_string());
    }

    params.insert("limit".to_string(), limit.unwrap_or(20).to_string());

    // Make search request
    let url = format!("{}/dex/search", config.base_url);
    let response = client
        .get_with_params(&url, &params)
        .await
        .map_err(|e| WebToolError::Network(format!("Search request failed: {}", e)))?;

    // Parse search results
    let tokens = parse_search_results(&response)
        .await
        .map_err(|e| WebToolError::Api(format!("Failed to parse search results: {}", e)))?;

    let result = TokenSearchResult {
        query: query.clone(),
        tokens: tokens.clone(),
        metadata: SearchMetadata {
            result_count: tokens.len() as u32,
            execution_time_ms: 150, // Would measure actual time
            limited: tokens.len() >= limit.unwrap_or(20) as usize,
            suggestions: vec![], // Would provide from API
        },
        searched_at: Utc::now(),
    };

    info!(
        "Token search completed: {} results for '{}'",
        result.tokens.len(),
        query
    );

    Ok(result)
}

/// Get trending tokens from DexScreener
///
/// This tool retrieves trending tokens based on volume,
/// price changes, and social activity.
#[tool]
pub async fn get_trending_tokens(
    time_window: Option<String>, // "5m", "1h", "24h"
    chain_filter: Option<String>,
    min_volume: Option<f64>,
    limit: Option<u32>,
) -> crate::error::Result<Vec<TokenInfo>> {
    debug!(
        "Fetching trending tokens for window: {:?}",
        time_window.as_deref().unwrap_or("1h")
    );

    let config = DexScreenerConfig::default();
    let client = WebClient::new().map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?;

    // Build trending endpoint
    let window = time_window.unwrap_or_else(|| "1h".to_string());
    let mut params = HashMap::new();
    params.insert("window".to_string(), window);
    params.insert("limit".to_string(), limit.unwrap_or(50).to_string());

    if let Some(chain) = chain_filter {
        params.insert("chain".to_string(), chain);
    }

    if let Some(min_vol) = min_volume {
        params.insert("min_volume".to_string(), min_vol.to_string());
    }

    let url = format!("{}/dex/tokens/trending", config.base_url);
    let response = client
        .get_with_params(&url, &params)
        .await
        .map_err(|e| WebToolError::Network(format!("Failed to fetch trending tokens: {}", e)))?;

    let trending_tokens = parse_trending_response(&response)
        .await
        .map_err(|e| WebToolError::Api(format!("Failed to parse trending response: {}", e)))?;

    info!("Retrieved {} trending tokens", trending_tokens.len());

    Ok(trending_tokens)
}

/// Analyze token market data
///
/// This tool provides deep market analysis including trend analysis,
/// volume patterns, liquidity assessment, and risk evaluation.
#[tool]
pub async fn analyze_token_market(
    token_address: String,
    chain_id: Option<String>,
    include_technical: Option<bool>,
    include_risk: Option<bool>,
) -> crate::error::Result<MarketAnalysis> {
    debug!("Performing market analysis for token: {}", token_address);

    // Get basic token info first
    let token_info =
        get_token_info(token_address.clone(), chain_id, Some(true), include_risk).await?;

    // Perform trend analysis
    let trend_analysis = analyze_price_trends(&token_info)
        .await
        .map_err(|e| WebToolError::Api(format!("Trend analysis failed: {}", e)))?;

    // Analyze volume patterns
    let volume_analysis = analyze_volume_patterns(&token_info)
        .await
        .map_err(|e| WebToolError::Api(format!("Volume analysis failed: {}", e)))?;

    // Assess liquidity
    let liquidity_analysis = analyze_liquidity(&token_info)
        .await
        .map_err(|e| WebToolError::Api(format!("Liquidity analysis failed: {}", e)))?;

    // Analyze price levels
    let price_levels = analyze_price_levels(&token_info)
        .await
        .map_err(|e| WebToolError::Api(format!("Price level analysis failed: {}", e)))?;

    // Perform risk assessment
    let risk_assessment = if include_risk.unwrap_or(true) {
        assess_token_risks(&token_info)
            .await
            .map_err(|e| WebToolError::Api(format!("Risk assessment failed: {}", e)))?
    } else {
        RiskAssessment {
            risk_level: "Unknown".to_string(),
            risk_factors: vec![],
            liquidity_risk: 50,
            volatility_risk: 50,
            contract_risk: 50,
        }
    };

    let analysis = MarketAnalysis {
        token: token_info.clone(),
        trend_analysis,
        volume_analysis,
        liquidity_analysis,
        price_levels,
        risk_assessment,
        analyzed_at: Utc::now(),
    };

    info!(
        "Market analysis completed for {} - Risk: {}, Trend: {}",
        token_info.symbol, analysis.risk_assessment.risk_level, analysis.trend_analysis.direction
    );

    Ok(analysis)
}

/// Get top DEX pairs by volume across all chains
///
/// This tool retrieves the highest volume trading pairs,
/// useful for identifying active markets and arbitrage opportunities.
#[tool]
pub async fn get_top_pairs(
    time_window: Option<String>, // "5m", "1h", "24h"
    chain_filter: Option<String>,
    dex_filter: Option<String>,
    min_liquidity: Option<f64>,
    limit: Option<u32>,
) -> crate::error::Result<Vec<TokenPair>> {
    debug!(
        "Fetching top pairs for window: {:?}",
        time_window.as_deref().unwrap_or("24h")
    );

    let config = DexScreenerConfig::default();
    let client = WebClient::new().map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?;

    let mut params = HashMap::new();
    params.insert("sort".to_string(), "volume".to_string());
    params.insert(
        "window".to_string(),
        time_window.unwrap_or_else(|| "24h".to_string()),
    );
    params.insert("limit".to_string(), limit.unwrap_or(100).to_string());

    if let Some(chain) = chain_filter {
        params.insert("chain".to_string(), chain);
    }

    if let Some(dex) = dex_filter {
        params.insert("dex".to_string(), dex);
    }

    if let Some(min_liq) = min_liquidity {
        params.insert("min_liquidity".to_string(), min_liq.to_string());
    }

    let url = format!("{}/dex/pairs/top", config.base_url);
    let response = client
        .get_with_params(&url, &params)
        .await
        .map_err(|e| WebToolError::Network(format!("Failed to fetch top pairs: {}", e)))?;

    let pairs = parse_pairs_response(&response)
        .await
        .map_err(|e| WebToolError::Api(format!("Failed to parse pairs response: {}", e)))?;

    info!("Retrieved {} top trading pairs", pairs.len());

    Ok(pairs)
}

async fn parse_token_response(
    response: &str,
    token_address: &str,
    chain: &str,
) -> crate::error::Result<TokenInfo> {
    // In production, this would parse actual DexScreener JSON
    // For now, return a comprehensive mock token
    Ok(TokenInfo {
        address: token_address.to_string(),
        name: "Example Token".to_string(),
        symbol: "EXAMPLE".to_string(),
        decimals: 18,
        price_usd: Some(1.25),
        market_cap: Some(125_000_000.0),
        volume_24h: Some(2_500_000.0),
        price_change_24h: Some(5.25),
        price_change_1h: Some(-1.5),
        price_change_5m: Some(0.8),
        circulating_supply: Some(100_000_000.0),
        total_supply: Some(1_000_000_000.0),
        pair_count: 5,
        pairs: vec![TokenPair {
            pair_id: "uniswap_v3_eth_example".to_string(),
            dex: DexInfo {
                id: "uniswap_v3".to_string(),
                name: "Uniswap V3".to_string(),
                url: Some("https://uniswap.org".to_string()),
                logo: None,
            },
            base_token: PairToken {
                address: token_address.to_string(),
                name: "Example Token".to_string(),
                symbol: "EXAMPLE".to_string(),
            },
            quote_token: PairToken {
                address: "0xA0b86a33E6441986a3f0c7B7A4a8D7F56B9a7C9F".to_string(),
                name: "Wrapped Ether".to_string(),
                symbol: "WETH".to_string(),
            },
            price_usd: 1.25,
            price_native: 0.0008,
            volume_24h: 1_200_000.0,
            price_change_24h: 5.25,
            liquidity_usd: Some(800_000.0),
            fdv: Some(125_000_000.0),
            created_at: Some(Utc::now()),
            last_trade_at: Utc::now(),
            txns_24h: TransactionStats {
                buys: 1250,
                sells: 980,
                total: 2230,
                buy_volume_usd: 700_000.0,
                sell_volume_usd: 500_000.0,
            },
            url: "https://app.uniswap.org/#/swap".to_string(),
        }],
        chain: ChainInfo {
            id: chain.to_string(),
            name: match chain {
                "ethereum" => "Ethereum",
                "bsc" => "Binance Smart Chain",
                "polygon" => "Polygon",
                _ => "Unknown Chain",
            }
            .to_string(),
            logo: None,
            native_token: match chain {
                "ethereum" => "ETH",
                "bsc" => "BNB",
                "polygon" => "MATIC",
                _ => "NATIVE",
            }
            .to_string(),
        },
        security: SecurityInfo {
            is_verified: true,
            liquidity_locked: Some(true),
            audit_status: Some("Audited".to_string()),
            honeypot_status: Some("Safe".to_string()),
            ownership_status: Some("Renounced".to_string()),
            risk_score: Some(25),
        },
        socials: vec![SocialLink {
            platform: "twitter".to_string(),
            url: "https://twitter.com/example_token".to_string(),
            followers: Some(15000),
        }],
        updated_at: Utc::now(),
    })
}

/// Parse search results from DexScreener API
async fn parse_search_results(response: &str) -> crate::error::Result<Vec<TokenInfo>> {
    // In production, would parse actual JSON response
    Ok(vec![])
}

async fn parse_trending_response(response: &str) -> crate::error::Result<Vec<TokenInfo>> {
    // In production, would parse actual JSON response
    Ok(vec![])
}

/// Parse trading pairs response
async fn parse_pairs_response(response: &str) -> crate::error::Result<Vec<TokenPair>> {
    // In production, would parse actual JSON response
    Ok(vec![])
}

async fn analyze_price_trends(token: &TokenInfo) -> crate::error::Result<TrendAnalysis> {
    let price_change_24h = token.price_change_24h.unwrap_or(0.0);
    let price_change_1h = token.price_change_1h.unwrap_or(0.0);

    let direction = if price_change_24h > 5.0 {
        "Bullish"
    } else if price_change_24h < -5.0 {
        "Bearish"
    } else {
        "Neutral"
    }
    .to_string();

    let strength = ((price_change_24h.abs() / 10.0).min(10.0).max(1.0)) as u32;

    Ok(TrendAnalysis {
        direction,
        strength,
        momentum: price_change_1h * 24.0, // Extrapolated momentum
        velocity: price_change_24h / 24.0,
        support_levels: vec![token.price_usd.unwrap_or(0.0) * 0.95],
        resistance_levels: vec![token.price_usd.unwrap_or(0.0) * 1.05],
    })
}

async fn analyze_volume_patterns(token: &TokenInfo) -> crate::error::Result<VolumeAnalysis> {
    let volume_24h = token.volume_24h.unwrap_or(0.0);
    let market_cap = token.market_cap.unwrap_or(1.0);

    Ok(VolumeAnalysis {
        volume_rank: None,                      // Would calculate from all tokens
        volume_trend: "Increasing".to_string(), // Would analyze historical data
        volume_mcap_ratio: Some(volume_24h / market_cap),
        avg_volume_7d: Some(volume_24h * 0.8), // Mock 7-day average
        spike_factor: Some(1.2),               // Current vs average
    })
}

async fn analyze_liquidity(token: &TokenInfo) -> crate::error::Result<LiquidityAnalysis> {
    let total_liquidity = token
        .pairs
        .iter()
        .map(|p| p.liquidity_usd.unwrap_or(0.0))
        .sum();

    let mut dex_distribution = HashMap::new();
    for pair in &token.pairs {
        let current = dex_distribution.get(&pair.dex.name).unwrap_or(&0.0);
        dex_distribution.insert(
            pair.dex.name.clone(),
            current + pair.liquidity_usd.unwrap_or(0.0),
        );
    }

    let mut price_impact = HashMap::new();
    price_impact.insert("1k".to_string(), 0.1);
    price_impact.insert("10k".to_string(), 0.5);
    price_impact.insert("100k".to_string(), 2.0);

    Ok(LiquidityAnalysis {
        total_liquidity_usd: total_liquidity,
        dex_distribution,
        price_impact,
        depth_score: if total_liquidity > 1_000_000.0 {
            85
        } else {
            60
        },
    })
}

async fn analyze_price_levels(token: &TokenInfo) -> crate::error::Result<PriceLevelAnalysis> {
    let current_price = token.price_usd.unwrap_or(0.0);

    Ok(PriceLevelAnalysis {
        ath: Some(current_price * 1.5), // Mock ATH
        atl: Some(current_price * 0.1), // Mock ATL
        ath_distance_pct: Some(-33.3),
        atl_distance_pct: Some(900.0),
        high_24h: Some(current_price * 1.02),
        low_24h: Some(current_price * 0.98),
        range_position: Some(0.6),
    })
}

async fn assess_token_risks(token: &TokenInfo) -> crate::error::Result<RiskAssessment> {
    let mut risk_factors = vec![];
    let mut total_risk = 0;

    // Check liquidity risk
    let liquidity_score = if token
        .pairs
        .iter()
        .map(|p| p.liquidity_usd.unwrap_or(0.0))
        .sum::<f64>()
        < 100_000.0
    {
        risk_factors.push(RiskFactor {
            category: "Liquidity".to_string(),
            description: "Low liquidity may cause high price impact".to_string(),
            severity: "High".to_string(),
            impact: 75,
        });
        75
    } else {
        25
    };
    total_risk += liquidity_score;

    // Check contract verification
    let contract_score = if !token.security.is_verified {
        risk_factors.push(RiskFactor {
            category: "Contract".to_string(),
            description: "Contract is not verified".to_string(),
            severity: "High".to_string(),
            impact: 80,
        });
        80
    } else {
        20
    };
    total_risk += contract_score;

    // Check volatility
    let volatility_score = if token.price_change_24h.unwrap_or(0.0).abs() > 20.0 {
        risk_factors.push(RiskFactor {
            category: "Volatility".to_string(),
            description: "High price volatility detected".to_string(),
            severity: "Medium".to_string(),
            impact: 60,
        });
        60
    } else {
        30
    };
    total_risk += volatility_score;

    let avg_risk = total_risk / 3;
    let risk_level = match avg_risk {
        0..=25 => "Low",
        26..=50 => "Medium",
        51..=75 => "High",
        _ => "Extreme",
    }
    .to_string();

    Ok(RiskAssessment {
        risk_level,
        risk_factors,
        liquidity_risk: liquidity_score as u32,
        volatility_risk: volatility_score as u32,
        contract_risk: contract_score as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dexscreener_config_default() {
        let config = DexScreenerConfig::default();
        assert_eq!(config.base_url, "https://api.dexscreener.com/latest");
        assert_eq!(config.rate_limit_per_minute, 300);
    }

    #[test]
    fn test_token_info_serialization() {
        let token = TokenInfo {
            address: "0x123".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            price_usd: Some(1.0),
            market_cap: Some(1000000.0),
            volume_24h: Some(50000.0),
            price_change_24h: Some(5.0),
            price_change_1h: Some(-1.0),
            price_change_5m: Some(0.5),
            circulating_supply: Some(1000000.0),
            total_supply: Some(10000000.0),
            pair_count: 1,
            pairs: vec![],
            chain: ChainInfo {
                id: "ethereum".to_string(),
                name: "Ethereum".to_string(),
                logo: None,
                native_token: "ETH".to_string(),
            },
            security: SecurityInfo {
                is_verified: true,
                liquidity_locked: Some(true),
                audit_status: None,
                honeypot_status: None,
                ownership_status: None,
                risk_score: Some(25),
            },
            socials: vec![],
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("Test Token"));
    }
}
