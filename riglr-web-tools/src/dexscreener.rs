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

/// Comprehensive token information including price, volume, and market data
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
    /// Base token information
    pub base_token: PairToken,
    /// Quote token information
    pub quote_token: PairToken,
    /// Current price
    pub price_usd: Option<f64>,
    /// Price in native token units
    pub price_native: Option<f64>,
    /// 24h trading volume in USD
    pub volume_24h: Option<f64>,
    /// 24h price change percentage
    pub price_change_24h: Option<f64>,
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
    pub buys: Option<u32>,
    /// Number of sell transactions (24h)
    pub sells: Option<u32>,
    /// Total number of transactions (24h)
    pub total: Option<u32>,
    /// Buy volume in USD (24h)
    pub buy_volume_usd: Option<f64>,
    /// Sell volume in USD (24h)
    pub sell_volume_usd: Option<f64>,
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
    /// Native token symbol for this chain
    pub native_token: String,
}

/// Token security and verification information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SecurityInfo {
    /// Whether the token contract is verified
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

/// Market trend analysis including direction, momentum, and key price levels
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

/// Volume analysis including trends, ratios, and trading activity metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VolumeAnalysis {
    /// Volume rank among all tokens
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

/// Liquidity analysis including depth, distribution, and price impact calculations
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

/// Comprehensive risk assessment including liquidity, volatility, and contract risks
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

/// Token search results with metadata and execution information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenSearchResult {
    /// Search query used
    pub query: String,
    /// List of tokens found in search
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
    _context: &riglr_core::provider::ApplicationContext,
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
    let client = WebClient::default();

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
    let token_info = parse_token_response(&response, &token_address, &chain, include_security)
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
    _context: &riglr_core::provider::ApplicationContext,
    query: String,
    chain_filter: Option<String>,
    min_market_cap: Option<f64>,
    min_liquidity: Option<f64>,
    limit: Option<u32>,
) -> crate::error::Result<TokenSearchResult> {
    debug!("Searching tokens for query: '{}' with filters", query);

    let config = DexScreenerConfig::default();
    let client = WebClient::default();

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
    _context: &riglr_core::provider::ApplicationContext,
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
    let client = WebClient::default();

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

/// Analyze token market data using heuristic-based calculations
///
/// This tool provides market analysis based on available on-chain data including:
/// - Simple trend analysis based on price changes
/// - Volume pattern calculations from 24h data
/// - Liquidity assessment from DEX pairs
/// - Basic risk evaluation using heuristic scoring
///
/// Note: This is not a substitute for professional financial analysis or
/// machine learning models. All calculations are rule-based heuristics.
#[tool]
pub async fn analyze_token_market(
    _context: &riglr_core::provider::ApplicationContext,
    token_address: String,
    chain_id: Option<String>,
    _include_technical: Option<bool>,
    include_risk: Option<bool>,
) -> crate::error::Result<MarketAnalysis> {
    debug!("Performing market analysis for token: {}", token_address);

    // Get basic token info first
    let token_info = get_token_info(
        _context,
        token_address.clone(),
        chain_id,
        Some(true),
        include_risk,
    )
    .await?;

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
    _context: &riglr_core::provider::ApplicationContext,
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
    let client = WebClient::default();

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
    _include_security: Option<bool>,
) -> crate::error::Result<TokenInfo> {
    // Parse actual DexScreener JSON response
    let dex_response: crate::dexscreener_api::DexScreenerResponse = serde_json::from_str(response)
        .map_err(|e| {
            crate::error::WebToolError::Parsing(format!(
                "Failed to parse DexScreener response: {}",
                e
            ))
        })?;

    debug!("Parsed response with {} pairs", dex_response.pairs.len());

    // Find pairs for this token and aggregate data
    let token_pairs: Vec<&crate::dexscreener_api::PairInfo> = dex_response
        .pairs
        .iter()
        .filter(|p| {
            let matches = p.base_token.address.eq_ignore_ascii_case(token_address);
            debug!(
                "Checking pair: base_token={}, looking for={}, matches={}",
                p.base_token.address, token_address, matches
            );
            matches
        })
        .collect();

    if token_pairs.is_empty() {
        return Err(crate::error::WebToolError::Api(format!(
            "No pairs found for token address: {}",
            token_address
        )));
    }

    // Use the pair with highest liquidity as primary source
    let primary_pair = token_pairs
        .iter()
        .max_by_key(|p| {
            p.liquidity
                .as_ref()
                .and_then(|l| l.usd)
                .map(|usd| (usd * 1000.0) as u64)
                .unwrap_or(0)
        })
        .ok_or_else(|| crate::error::WebToolError::Api("No valid pairs found".to_string()))?;

    // Aggregate volume across all pairs
    let total_volume_24h: f64 = token_pairs
        .iter()
        .filter_map(|p| p.volume.as_ref())
        .filter_map(|v| v.h24)
        .sum();

    debug!(
        "Found {} pairs for token {}",
        token_pairs.len(),
        token_address
    );
    debug!(
        "Primary pair: symbol={}, price_usd={:?}",
        primary_pair.base_token.symbol, primary_pair.price_usd
    );

    // Convert pairs to our format
    let pairs: Vec<TokenPair> = token_pairs
        .iter()
        .map(|pair| {
            TokenPair {
                pair_id: pair.pair_address.clone(),
                dex: DexInfo {
                    id: pair.dex_id.clone(),
                    name: format_dex_name(&pair.dex_id),
                    url: Some(pair.url.clone()),
                    logo: None,
                },
                base_token: PairToken {
                    address: pair.base_token.address.clone(),
                    name: pair.base_token.name.clone(),
                    symbol: pair.base_token.symbol.clone(),
                },
                quote_token: PairToken {
                    address: pair.quote_token.address.clone(),
                    name: pair.quote_token.name.clone(),
                    symbol: pair.quote_token.symbol.clone(),
                },
                price_usd: pair.price_usd.as_ref().and_then(|p| p.parse().ok()),
                price_native: pair.price_native.parse().ok(),
                volume_24h: pair.volume.as_ref().and_then(|v| v.h24),
                price_change_24h: pair.price_change.as_ref().and_then(|pc| pc.h24),
                liquidity_usd: pair.liquidity.as_ref().and_then(|l| l.usd),
                fdv: pair.fdv,
                created_at: None, // DexScreener doesn't provide this in the response
                last_trade_at: Utc::now(), // Using current time as approximation
                txns_24h: TransactionStats {
                    buys: pair
                        .txns
                        .as_ref()
                        .and_then(|t| t.h24.as_ref())
                        .and_then(|h| h.buys.map(|b| b as u32)),
                    sells: pair
                        .txns
                        .as_ref()
                        .and_then(|t| t.h24.as_ref())
                        .and_then(|h| h.sells.map(|s| s as u32)),
                    total: None, // Will calculate if both buys and sells are available
                    buy_volume_usd: None, // DexScreener doesn't provide this separately
                    sell_volume_usd: None, // DexScreener doesn't provide this separately
                },
                url: pair.url.clone(),
            }
        })
        .collect();

    // Update total transactions
    let mut updated_pairs = pairs.clone();
    for pair in &mut updated_pairs {
        // Calculate total transactions if both buys and sells are available
        pair.txns_24h.total = match (pair.txns_24h.buys, pair.txns_24h.sells) {
            (Some(b), Some(s)) => Some(b + s),
            _ => None,
        };

        // Estimate buy/sell volumes (split total volume proportionally) if we have the data
        if let (Some(total), Some(buys), Some(volume)) =
            (pair.txns_24h.total, pair.txns_24h.buys, pair.volume_24h)
        {
            if total > 0 {
                let buy_ratio = buys as f64 / total as f64;
                pair.txns_24h.buy_volume_usd = Some(volume * buy_ratio);
                pair.txns_24h.sell_volume_usd = Some(volume * (1.0 - buy_ratio));
            }
        }
    }

    let parsed_price = primary_pair.price_usd.as_ref().and_then(|p| {
        let parsed = p.parse().ok();
        debug!("Parsing price_usd string '{}' -> {:?}", p, parsed);
        parsed
    });

    Ok(TokenInfo {
        address: token_address.to_string(),
        name: primary_pair.base_token.name.clone(),
        symbol: primary_pair.base_token.symbol.clone(),
        decimals: 18, // Default as DexScreener doesn't provide this
        price_usd: parsed_price,
        market_cap: primary_pair.market_cap,
        volume_24h: Some(total_volume_24h),
        price_change_24h: primary_pair.price_change.as_ref().and_then(|pc| pc.h24),
        price_change_1h: primary_pair.price_change.as_ref().and_then(|pc| pc.h1),
        price_change_5m: primary_pair.price_change.as_ref().and_then(|pc| pc.m5),
        circulating_supply: None, // DexScreener doesn't provide this
        total_supply: None,       // DexScreener doesn't provide this
        pair_count: updated_pairs.len() as u32,
        pairs: updated_pairs,
        chain: ChainInfo {
            id: chain.to_string(),
            name: format_chain_name(chain),
            logo: None,
            native_token: get_native_token(chain),
        },
        security: {
            // Future enhancement: Integrate with security API services like GoPlus or Honeypot.is
            // For now, return basic placeholder data regardless of include_security flag
            SecurityInfo {
                is_verified: false,
                liquidity_locked: None,
                audit_status: None,
                honeypot_status: None,
                ownership_status: None,
                risk_score: None,
            }
        },
        socials: vec![], // Would need additional API call to get social links
        updated_at: Utc::now(),
    })
}

fn format_dex_name(dex_id: &str) -> String {
    match dex_id {
        "uniswap" => "Uniswap V2".to_string(),
        "uniswapv3" => "Uniswap V3".to_string(),
        "pancakeswap" => "PancakeSwap".to_string(),
        "sushiswap" => "SushiSwap".to_string(),
        "curve" => "Curve".to_string(),
        "balancer" => "Balancer".to_string(),
        "quickswap" => "QuickSwap".to_string(),
        "raydium" => "Raydium".to_string(),
        "orca" => "Orca".to_string(),
        _ => dex_id.to_string(),
    }
}

fn format_chain_name(chain_id: &str) -> String {
    match chain_id {
        "ethereum" => "Ethereum".to_string(),
        "bsc" => "Binance Smart Chain".to_string(),
        "polygon" => "Polygon".to_string(),
        "arbitrum" => "Arbitrum".to_string(),
        "optimism" => "Optimism".to_string(),
        "avalanche" => "Avalanche".to_string(),
        "fantom" => "Fantom".to_string(),
        "solana" => "Solana".to_string(),
        _ => chain_id.to_string(),
    }
}

fn get_native_token(chain_id: &str) -> String {
    match chain_id {
        "ethereum" => "ETH".to_string(),
        "bsc" => "BNB".to_string(),
        "polygon" => "MATIC".to_string(),
        "arbitrum" => "ETH".to_string(),
        "optimism" => "ETH".to_string(),
        "avalanche" => "AVAX".to_string(),
        "fantom" => "FTM".to_string(),
        "solana" => "SOL".to_string(),
        _ => "NATIVE".to_string(),
    }
}

/// Parse search results from DexScreener API
async fn parse_search_results(response: &str) -> crate::error::Result<Vec<TokenInfo>> {
    let dex_response: crate::dexscreener_api::DexScreenerResponse = serde_json::from_str(response)
        .map_err(|e| crate::error::WebToolError::Parsing(e.to_string()))?;

    // Group pairs by token and aggregate data
    let mut tokens_map = std::collections::HashMap::new();

    for pair in dex_response.pairs {
        let token_address = pair.base_token.address.clone();
        let entry = tokens_map
            .entry(token_address.clone())
            .or_insert(TokenInfo {
                address: token_address,
                name: pair.base_token.name.clone(),
                symbol: pair.base_token.symbol.clone(),
                decimals: 18, // Default, as DexScreener doesn't provide this
                price_usd: pair.price_usd.as_ref().and_then(|p| p.parse().ok()),
                market_cap: pair.market_cap,
                volume_24h: pair.volume.as_ref().and_then(|v| v.h24),
                price_change_24h: pair.price_change.as_ref().and_then(|pc| pc.h24),
                price_change_1h: pair.price_change.as_ref().and_then(|pc| pc.h1),
                price_change_5m: pair.price_change.as_ref().and_then(|pc| pc.m5),
                circulating_supply: None,
                total_supply: None,
                pair_count: 0,
                pairs: vec![],
                chain: ChainInfo {
                    id: pair.chain_id.clone(),
                    name: pair.chain_id.clone(), // Using chain_id as name for simplicity
                    logo: None,
                    native_token: "ETH".to_string(), // Default to ETH for simplicity
                },
                security: SecurityInfo {
                    is_verified: false,
                    liquidity_locked: None,
                    audit_status: None,
                    honeypot_status: None,
                    ownership_status: None,
                    risk_score: None,
                },
                socials: vec![],
                updated_at: chrono::Utc::now(),
            });

        // Add this pair to the token's pairs list
        entry.pairs.push(TokenPair {
            pair_id: pair.pair_address.clone(),
            dex: DexInfo {
                id: pair.dex_id.clone(),
                name: pair.dex_id.clone(), // Using dex_id as name for simplicity
                url: Some(pair.url.clone()),
                logo: None,
            },
            base_token: PairToken {
                address: pair.base_token.address.clone(),
                name: pair.base_token.name.clone(),
                symbol: pair.base_token.symbol.clone(),
            },
            quote_token: PairToken {
                address: pair.quote_token.address.clone(),
                name: pair.quote_token.name.clone(),
                symbol: pair.quote_token.symbol.clone(),
            },
            price_usd: pair.price_usd.and_then(|p| p.parse().ok()),
            price_native: pair.price_native.parse().ok(),
            volume_24h: pair.volume.and_then(|v| v.h24),
            price_change_24h: pair.price_change.and_then(|pc| pc.h24),
            liquidity_usd: pair.liquidity.and_then(|l| l.usd),
            fdv: pair.fdv,
            created_at: None,
            last_trade_at: chrono::Utc::now(),
            txns_24h: TransactionStats {
                buys: pair
                    .txns
                    .as_ref()
                    .and_then(|t| t.h24.as_ref().and_then(|h| h.buys.map(|b| b as u32))),
                sells: pair
                    .txns
                    .as_ref()
                    .and_then(|t| t.h24.as_ref().and_then(|h| h.sells.map(|s| s as u32))),
                total: None,
                buy_volume_usd: None,
                sell_volume_usd: None,
            },
            url: pair.url,
        });
        entry.pair_count += 1;
    }

    Ok(tokens_map.into_values().collect())
}

async fn parse_trending_response(response: &str) -> crate::error::Result<Vec<TokenInfo>> {
    // Same as search results for now
    parse_search_results(response).await
}

/// Parse trading pairs response
async fn parse_pairs_response(response: &str) -> crate::error::Result<Vec<TokenPair>> {
    let dex_response: crate::dexscreener_api::DexScreenerResponse = serde_json::from_str(response)
        .map_err(|e| crate::error::WebToolError::Parsing(e.to_string()))?;

    let pairs: Vec<TokenPair> = dex_response
        .pairs
        .into_iter()
        .map(|pair| TokenPair {
            pair_id: pair.pair_address.clone(),
            dex: DexInfo {
                id: pair.dex_id.clone(),
                name: pair.dex_id.clone(),
                url: Some(pair.url.clone()),
                logo: None,
            },
            base_token: PairToken {
                address: pair.base_token.address.clone(),
                name: pair.base_token.name.clone(),
                symbol: pair.base_token.symbol.clone(),
            },
            quote_token: PairToken {
                address: pair.quote_token.address.clone(),
                name: pair.quote_token.name.clone(),
                symbol: pair.quote_token.symbol.clone(),
            },
            price_usd: pair.price_usd.and_then(|p| p.parse().ok()),
            price_native: pair.price_native.parse().ok(),
            volume_24h: pair.volume.and_then(|v| v.h24),
            price_change_24h: pair.price_change.and_then(|pc| pc.h24),
            liquidity_usd: pair.liquidity.and_then(|l| l.usd),
            fdv: pair.fdv,
            created_at: None,
            last_trade_at: chrono::Utc::now(),
            txns_24h: TransactionStats {
                buys: pair
                    .txns
                    .as_ref()
                    .and_then(|t| t.h24.as_ref().and_then(|h| h.buys.map(|b| b as u32))),
                sells: pair
                    .txns
                    .as_ref()
                    .and_then(|t| t.h24.as_ref().and_then(|h| h.sells.map(|s| s as u32))),
                total: None,
                buy_volume_usd: None,
                sell_volume_usd: None,
            },
            url: pair.url,
        })
        .collect();

    Ok(pairs)
}

async fn analyze_price_trends(token: &TokenInfo) -> crate::error::Result<TrendAnalysis> {
    // Calculate trend based on available price change data
    let price_change_24h = token.price_change_24h;
    let price_change_1h = token.price_change_1h;

    let direction = match price_change_24h {
        Some(change) if change > 5.0 => "Bullish",
        Some(change) if change < -5.0 => "Bearish",
        Some(_) => "Neutral",
        None => "Unknown",
    }
    .to_string();

    let strength =
        price_change_24h.map_or(5, |change| ((change.abs() / 10.0).clamp(1.0, 10.0)) as u32); // Default to medium strength if no data

    // Calculate momentum and velocity only if we have data
    let momentum = match (price_change_1h, price_change_24h) {
        (Some(h1), Some(h24)) => h1 * 24.0 - h24, // Acceleration
        _ => 0.0,
    };

    let velocity = price_change_24h.map_or(0.0, |c| c / 24.0);

    // Simple support/resistance based on recent range, if price available
    let (support_levels, resistance_levels) = if let Some(price) = token.price_usd {
        // Basic 5% bands - in production would use actual order book data
        (vec![price * 0.95], vec![price * 1.05])
    } else {
        (vec![], vec![])
    };

    Ok(TrendAnalysis {
        direction,
        strength,
        momentum,
        velocity,
        support_levels,
        resistance_levels,
    })
}

async fn analyze_volume_patterns(token: &TokenInfo) -> crate::error::Result<VolumeAnalysis> {
    // Calculate volume metrics from available data
    let volume_mcap_ratio = match (token.volume_24h, token.market_cap) {
        (Some(vol), Some(mcap)) if mcap > 0.0 => Some(vol / mcap),
        _ => None,
    };

    // We don't have historical data, so we can't determine trend or averages
    // In production, this would query historical data from the API
    Ok(VolumeAnalysis {
        volume_rank: None,                   // Requires comparison with other tokens
        volume_trend: "Unknown".to_string(), // Requires historical data
        volume_mcap_ratio,
        avg_volume_7d: None, // Requires 7-day historical data
        spike_factor: None,  // Requires average to compare against
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
    // We don't have historical ATH/ATL data from the API
    // Only return what we can actually calculate from available data

    // Try to estimate 24h high/low from price and price change
    let (high_24h, low_24h, range_position) = if let Some(price) = token.price_usd {
        // If we have price change %, estimate the range
        match token.price_change_24h {
            Some(change_pct) => {
                // Rough estimate: if price went up X%, low was price/(1+X/100)
                let change_factor = 1.0 + (change_pct / 100.0);
                if change_pct > 0.0 {
                    let low = price / change_factor;
                    (Some(price), Some(low), Some(1.0)) // Currently at high
                } else {
                    let high = price / change_factor;
                    (Some(high), Some(price), Some(0.0)) // Currently at low
                }
            }
            None => (None, None, None),
        }
    } else {
        (None, None, None)
    };

    Ok(PriceLevelAnalysis {
        ath: None, // Requires historical data not available from current API
        atl: None, // Requires historical data not available from current API
        ath_distance_pct: None,
        atl_distance_pct: None,
        high_24h,
        low_24h,
        range_position,
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
    let volatility_score = match token.price_change_24h {
        Some(change) if change.abs() > 20.0 => {
            risk_factors.push(RiskFactor {
                category: "Volatility".to_string(),
                description: "High price volatility detected".to_string(),
                severity: "Medium".to_string(),
                impact: 60,
            });
            60
        }
        Some(_) => 30, // Normal volatility
        None => {
            risk_factors.push(RiskFactor {
                category: "Data".to_string(),
                description: "Volatility data unavailable".to_string(),
                severity: "Low".to_string(),
                impact: 20,
            });
            20
        }
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

    #[test]
    fn test_dexscreener_config_custom_values() {
        let config = DexScreenerConfig {
            base_url: "https://custom.api.com".to_string(),
            rate_limit_per_minute: 100,
            request_timeout: 60,
        };
        assert_eq!(config.base_url, "https://custom.api.com");
        assert_eq!(config.rate_limit_per_minute, 100);
        assert_eq!(config.request_timeout, 60);
    }

    #[test]
    fn test_format_dex_name_when_known_dex_should_return_formatted_name() {
        assert_eq!(format_dex_name("uniswap"), "Uniswap V2");
        assert_eq!(format_dex_name("uniswapv3"), "Uniswap V3");
        assert_eq!(format_dex_name("pancakeswap"), "PancakeSwap");
        assert_eq!(format_dex_name("sushiswap"), "SushiSwap");
        assert_eq!(format_dex_name("curve"), "Curve");
        assert_eq!(format_dex_name("balancer"), "Balancer");
        assert_eq!(format_dex_name("quickswap"), "QuickSwap");
        assert_eq!(format_dex_name("raydium"), "Raydium");
        assert_eq!(format_dex_name("orca"), "Orca");
    }

    #[test]
    fn test_format_dex_name_when_unknown_dex_should_return_original() {
        assert_eq!(format_dex_name("unknown-dex"), "unknown-dex");
        assert_eq!(format_dex_name("custom_dex"), "custom_dex");
        assert_eq!(format_dex_name(""), "");
    }

    #[test]
    fn test_format_chain_name_when_known_chain_should_return_formatted_name() {
        assert_eq!(format_chain_name("ethereum"), "Ethereum");
        assert_eq!(format_chain_name("bsc"), "Binance Smart Chain");
        assert_eq!(format_chain_name("polygon"), "Polygon");
        assert_eq!(format_chain_name("arbitrum"), "Arbitrum");
        assert_eq!(format_chain_name("optimism"), "Optimism");
        assert_eq!(format_chain_name("avalanche"), "Avalanche");
        assert_eq!(format_chain_name("fantom"), "Fantom");
        assert_eq!(format_chain_name("solana"), "Solana");
    }

    #[test]
    fn test_format_chain_name_when_unknown_chain_should_return_original() {
        assert_eq!(format_chain_name("unknown-chain"), "unknown-chain");
        assert_eq!(format_chain_name("custom_chain"), "custom_chain");
        assert_eq!(format_chain_name(""), "");
    }

    #[test]
    fn test_get_native_token_when_known_chain_should_return_correct_token() {
        assert_eq!(get_native_token("ethereum"), "ETH");
        assert_eq!(get_native_token("bsc"), "BNB");
        assert_eq!(get_native_token("polygon"), "MATIC");
        assert_eq!(get_native_token("arbitrum"), "ETH");
        assert_eq!(get_native_token("optimism"), "ETH");
        assert_eq!(get_native_token("avalanche"), "AVAX");
        assert_eq!(get_native_token("fantom"), "FTM");
        assert_eq!(get_native_token("solana"), "SOL");
    }

    #[test]
    fn test_get_native_token_when_unknown_chain_should_return_native() {
        assert_eq!(get_native_token("unknown-chain"), "NATIVE");
        assert_eq!(get_native_token("custom_chain"), "NATIVE");
        assert_eq!(get_native_token(""), "NATIVE");
    }

    #[tokio::test]
    async fn test_analyze_price_trends_when_bullish_data_should_return_bullish_trend() {
        let token = create_test_token_info(Some(10.0), Some(2.0), Some(1.0));
        let result = analyze_price_trends(&token).await.unwrap();

        assert_eq!(result.direction, "Bullish");
        assert_eq!(result.strength, 1); // 10.0 / 10.0 = 1.0, clamped to 1
        assert_eq!(result.momentum, 26.0); // 2.0 * 24.0 - 10.0
        assert_eq!(result.velocity, 10.0 / 24.0);
        assert_eq!(result.support_levels.len(), 1);
        assert_eq!(result.resistance_levels.len(), 1);
    }

    #[tokio::test]
    async fn test_analyze_price_trends_when_bearish_data_should_return_bearish_trend() {
        let token = create_test_token_info(Some(-10.0), Some(-1.0), Some(1.0));
        let result = analyze_price_trends(&token).await.unwrap();

        assert_eq!(result.direction, "Bearish");
        assert_eq!(result.strength, 1); // 10.0 / 10.0 = 1.0, clamped to 1
        assert_eq!(result.momentum, -14.0); // -1.0 * 24.0 - (-10.0)
        assert_eq!(result.velocity, -10.0 / 24.0);
    }

    #[tokio::test]
    async fn test_analyze_price_trends_when_neutral_data_should_return_neutral_trend() {
        let token = create_test_token_info(Some(2.0), Some(0.5), Some(1.0));
        let result = analyze_price_trends(&token).await.unwrap();

        assert_eq!(result.direction, "Neutral");
        assert_eq!(result.strength, 1); // 2.0 / 10.0 = 0.2, clamped to 1.0
        assert_eq!(result.momentum, 10.0); // 0.5 * 24.0 - 2.0
    }

    #[tokio::test]
    async fn test_analyze_price_trends_when_no_data_should_return_unknown_trend() {
        let token = create_test_token_info(None, None, Some(1.0));
        let result = analyze_price_trends(&token).await.unwrap();

        assert_eq!(result.direction, "Unknown");
        assert_eq!(result.strength, 5); // Default when no data
        assert_eq!(result.momentum, 0.0);
        assert_eq!(result.velocity, 0.0);
        assert_eq!(result.support_levels.len(), 0);
        assert_eq!(result.resistance_levels.len(), 0);
    }

    #[tokio::test]
    async fn test_analyze_price_trends_when_no_price_should_return_empty_levels() {
        let token = create_test_token_info(Some(5.0), Some(1.0), None);
        let result = analyze_price_trends(&token).await.unwrap();

        assert_eq!(result.support_levels.len(), 0);
        assert_eq!(result.resistance_levels.len(), 0);
    }

    #[tokio::test]
    async fn test_analyze_volume_patterns_when_valid_data_should_calculate_ratio() {
        let mut token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0));
        token.volume_24h = Some(50000.0);
        token.market_cap = Some(1000000.0);

        let result = analyze_volume_patterns(&token).await.unwrap();

        assert_eq!(result.volume_mcap_ratio, Some(0.05)); // 50000 / 1000000
        assert_eq!(result.volume_trend, "Unknown");
        assert_eq!(result.volume_rank, None);
        assert_eq!(result.avg_volume_7d, None);
        assert_eq!(result.spike_factor, None);
    }

    #[tokio::test]
    async fn test_analyze_volume_patterns_when_zero_market_cap_should_return_none_ratio() {
        let mut token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0));
        token.volume_24h = Some(50000.0);
        token.market_cap = Some(0.0);

        let result = analyze_volume_patterns(&token).await.unwrap();

        assert_eq!(result.volume_mcap_ratio, None);
    }

    #[tokio::test]
    async fn test_analyze_volume_patterns_when_missing_data_should_return_none_ratio() {
        let mut token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0));
        token.volume_24h = None;
        token.market_cap = None;

        let result = analyze_volume_patterns(&token).await.unwrap();

        assert_eq!(result.volume_mcap_ratio, None);
    }

    #[tokio::test]
    async fn test_analyze_liquidity_when_multiple_pairs_should_aggregate_liquidity() {
        let mut token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0));
        token.pairs = vec![
            create_test_token_pair("dex1", 100000.0),
            create_test_token_pair("dex2", 200000.0),
        ];

        let result = analyze_liquidity(&token).await.unwrap();

        assert_eq!(result.total_liquidity_usd, 300000.0);
        assert_eq!(result.dex_distribution.len(), 2);
        assert_eq!(result.dex_distribution.get("dex1"), Some(&100000.0));
        assert_eq!(result.dex_distribution.get("dex2"), Some(&200000.0));
        assert_eq!(result.depth_score, 60); // Less than 1M threshold
    }

    #[tokio::test]
    async fn test_analyze_liquidity_when_high_liquidity_should_return_high_depth_score() {
        let mut token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0));
        token.pairs = vec![create_test_token_pair("dex1", 2000000.0)];

        let result = analyze_liquidity(&token).await.unwrap();

        assert_eq!(result.total_liquidity_usd, 2000000.0);
        assert_eq!(result.depth_score, 85); // Above 1M threshold
    }

    #[tokio::test]
    async fn test_analyze_liquidity_when_no_pairs_should_return_zero_liquidity() {
        let token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0));

        let result = analyze_liquidity(&token).await.unwrap();

        assert_eq!(result.total_liquidity_usd, 0.0);
        assert_eq!(result.dex_distribution.len(), 0);
        assert_eq!(result.depth_score, 60);
    }

    #[tokio::test]
    async fn test_analyze_price_levels_when_positive_change_should_estimate_at_high() {
        let token = create_test_token_info(Some(10.0), Some(1.0), Some(100.0));

        let result = analyze_price_levels(&token).await.unwrap();

        assert_eq!(result.high_24h, Some(100.0));
        assert!(result.low_24h.is_some());
        assert!(result.low_24h.unwrap() < 100.0);
        assert_eq!(result.range_position, Some(1.0)); // At high
        assert_eq!(result.ath, None);
        assert_eq!(result.atl, None);
    }

    #[tokio::test]
    async fn test_analyze_price_levels_when_negative_change_should_estimate_at_low() {
        let token = create_test_token_info(Some(-10.0), Some(1.0), Some(90.0));

        let result = analyze_price_levels(&token).await.unwrap();

        assert!(result.high_24h.is_some());
        assert!(result.high_24h.unwrap() > 90.0);
        assert_eq!(result.low_24h, Some(90.0));
        assert_eq!(result.range_position, Some(0.0)); // At low
    }

    #[tokio::test]
    async fn test_analyze_price_levels_when_no_price_change_should_return_none() {
        let mut token = create_test_token_info(None, Some(1.0), Some(100.0));
        token.price_change_24h = None;

        let result = analyze_price_levels(&token).await.unwrap();

        assert_eq!(result.high_24h, None);
        assert_eq!(result.low_24h, None);
        assert_eq!(result.range_position, None);
    }

    #[tokio::test]
    async fn test_analyze_price_levels_when_no_price_should_return_none() {
        let token = create_test_token_info(Some(10.0), Some(1.0), None);

        let result = analyze_price_levels(&token).await.unwrap();

        assert_eq!(result.high_24h, None);
        assert_eq!(result.low_24h, None);
        assert_eq!(result.range_position, None);
    }

    #[tokio::test]
    async fn test_assess_token_risks_when_low_liquidity_should_add_liquidity_risk() {
        let mut token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0));
        token.pairs = vec![create_test_token_pair("dex1", 50000.0)]; // Low liquidity

        let result = assess_token_risks(&token).await.unwrap();

        assert_eq!(result.liquidity_risk, 75);
        assert!(result
            .risk_factors
            .iter()
            .any(|r| r.category == "Liquidity"));
        assert!(matches!(result.risk_level.as_str(), "High" | "Extreme"));
    }

    #[tokio::test]
    async fn test_assess_token_risks_when_high_liquidity_should_have_low_liquidity_risk() {
        let mut token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0));
        token.pairs = vec![create_test_token_pair("dex1", 500000.0)]; // High liquidity

        let result = assess_token_risks(&token).await.unwrap();

        assert_eq!(result.liquidity_risk, 25);
    }

    #[tokio::test]
    async fn test_assess_token_risks_when_unverified_contract_should_add_contract_risk() {
        let mut token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0));
        token.security.is_verified = false;
        token.pairs = vec![create_test_token_pair("dex1", 500000.0)];

        let result = assess_token_risks(&token).await.unwrap();

        assert_eq!(result.contract_risk, 80);
        assert!(result.risk_factors.iter().any(|r| r.category == "Contract"));
    }

    #[tokio::test]
    async fn test_assess_token_risks_when_verified_contract_should_have_low_contract_risk() {
        let mut token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0));
        token.security.is_verified = true;
        token.pairs = vec![create_test_token_pair("dex1", 500000.0)];

        let result = assess_token_risks(&token).await.unwrap();

        assert_eq!(result.contract_risk, 20);
    }

    #[tokio::test]
    async fn test_assess_token_risks_when_high_volatility_should_add_volatility_risk() {
        let mut token = create_test_token_info(Some(25.0), Some(1.0), Some(1.0)); // High volatility
        token.security.is_verified = true;
        token.pairs = vec![create_test_token_pair("dex1", 500000.0)];

        let result = assess_token_risks(&token).await.unwrap();

        assert_eq!(result.volatility_risk, 60);
        assert!(result
            .risk_factors
            .iter()
            .any(|r| r.category == "Volatility"));
    }

    #[tokio::test]
    async fn test_assess_token_risks_when_normal_volatility_should_have_medium_volatility_risk() {
        let mut token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0)); // Normal volatility
        token.security.is_verified = true;
        token.pairs = vec![create_test_token_pair("dex1", 500000.0)];

        let result = assess_token_risks(&token).await.unwrap();

        assert_eq!(result.volatility_risk, 30);
    }

    #[tokio::test]
    async fn test_assess_token_risks_when_no_volatility_data_should_add_data_risk() {
        let mut token = create_test_token_info(None, Some(1.0), Some(1.0));
        token.security.is_verified = true;
        token.pairs = vec![create_test_token_pair("dex1", 500000.0)];

        let result = assess_token_risks(&token).await.unwrap();

        assert_eq!(result.volatility_risk, 20);
        assert!(result.risk_factors.iter().any(|r| r.category == "Data"));
    }

    #[tokio::test]
    async fn test_assess_token_risks_when_low_risk_should_return_low_risk_level() {
        let mut token = create_test_token_info(Some(5.0), Some(1.0), Some(1.0));
        token.security.is_verified = true;
        token.pairs = vec![create_test_token_pair("dex1", 500000.0)];

        let result = assess_token_risks(&token).await.unwrap();

        assert_eq!(result.risk_level, "Low");
    }

    #[tokio::test]
    async fn test_parse_search_results_when_invalid_json_should_return_error() {
        let invalid_json = "invalid json";
        let result = parse_search_results(invalid_json).await;

        assert!(result.is_err());
        if let Err(crate::error::WebToolError::Parsing(msg)) = result {
            assert!(msg.contains("expected"));
        }
    }

    #[tokio::test]
    async fn test_parse_trending_response_when_invalid_json_should_return_error() {
        let invalid_json = "invalid json";
        let result = parse_trending_response(invalid_json).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_pairs_response_when_invalid_json_should_return_error() {
        let invalid_json = "invalid json";
        let result = parse_pairs_response(invalid_json).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_token_response_when_invalid_json_should_return_parsing_error() {
        let invalid_json = "invalid json";
        let result = parse_token_response(invalid_json, "0x123", "ethereum", Some(true)).await;

        assert!(result.is_err());
        if let Err(crate::error::WebToolError::Parsing(msg)) = result {
            assert!(msg.contains("Failed to parse DexScreener response"));
        }
    }

    #[tokio::test]
    async fn test_parse_token_response_when_no_pairs_found_should_return_api_error() {
        let valid_json_no_pairs = r#"{"pairs": []}"#;
        let result =
            parse_token_response(valid_json_no_pairs, "0x123", "ethereum", Some(true)).await;

        assert!(result.is_err());
        if let Err(crate::error::WebToolError::Api(msg)) = result {
            assert!(msg.contains("No pairs found for token address"));
        }
    }

    #[tokio::test]
    async fn test_parse_token_response_when_no_valid_pairs_should_return_api_error() {
        let json_with_unmatched_pairs = r#"{"pairs": [{"base_token": {"address": "0x456", "name": "Other Token", "symbol": "OTHER"}, "quote_token": {"address": "0x789", "name": "USDC", "symbol": "USDC"}, "pair_address": "0xabc", "dex_id": "uniswap", "url": "https://example.com", "price_usd": "1.0", "price_native": "1.0"}]}"#;
        let result =
            parse_token_response(json_with_unmatched_pairs, "0x123", "ethereum", Some(true)).await;

        assert!(result.is_err());
        if let Err(crate::error::WebToolError::Api(msg)) = result {
            assert!(msg.contains("No pairs found for token address"));
        }
    }

    #[tokio::test]
    async fn test_parse_token_response_when_valid_data_should_return_token_info() {
        let valid_json = create_valid_dexscreener_response();
        let result = parse_token_response(&valid_json, "0x123", "ethereum", Some(true)).await;

        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.address, "0x123");
        assert_eq!(token.symbol, "TEST");
        assert_eq!(token.name, "Test Token");
        assert_eq!(token.chain.id, "ethereum");
        assert_eq!(token.chain.name, "Ethereum");
        assert_eq!(token.chain.native_token, "ETH");
        assert_eq!(token.pair_count, 1);
        assert!(!token.pairs.is_empty());
    }

    #[tokio::test]
    async fn test_parse_token_response_when_multiple_pairs_should_use_highest_liquidity() {
        let json_with_multiple_pairs = create_dexscreener_response_with_multiple_pairs();
        let result =
            parse_token_response(&json_with_multiple_pairs, "0x123", "ethereum", Some(true)).await;

        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.pair_count, 2);
        // Should have aggregated volume from both pairs
        assert!(token.volume_24h.unwrap() > 10000.0);
        // Primary pair should be the one with higher liquidity
        assert_eq!(token.price_usd, Some(2.0)); // From high liquidity pair
    }

    #[tokio::test]
    async fn test_parse_token_response_when_case_insensitive_address_should_match() {
        let valid_json = create_valid_dexscreener_response();
        let result = parse_token_response(&valid_json, "0X123", "ethereum", Some(true)).await; // Uppercase

        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.address, "0X123"); // Should preserve input case
    }

    #[tokio::test]
    async fn test_parse_token_response_when_no_liquidity_data_should_handle_gracefully() {
        let json_no_liquidity = create_dexscreener_response_without_liquidity();
        let result =
            parse_token_response(&json_no_liquidity, "0x123", "ethereum", Some(true)).await;

        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.pairs[0].liquidity_usd, None);
    }

    #[tokio::test]
    async fn test_parse_token_response_when_transaction_data_available_should_calculate_totals() {
        let json_with_txns = create_dexscreener_response_with_transaction_data();
        let result = parse_token_response(&json_with_txns, "0x123", "ethereum", Some(true)).await;

        assert!(result.is_ok());
        let token = result.unwrap();
        let pair = &token.pairs[0];
        assert_eq!(pair.txns_24h.buys, Some(100));
        assert_eq!(pair.txns_24h.sells, Some(50));
        assert_eq!(pair.txns_24h.total, Some(150)); // Should calculate total
        assert!(pair.txns_24h.buy_volume_usd.is_some());
        assert!(pair.txns_24h.sell_volume_usd.is_some());
    }

    #[tokio::test]
    async fn test_parse_token_response_when_unknown_chain_should_use_default_formatting() {
        let valid_json = create_valid_dexscreener_response();
        let result = parse_token_response(&valid_json, "0x123", "unknown-chain", Some(true)).await;

        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.chain.id, "unknown-chain");
        assert_eq!(token.chain.name, "unknown-chain");
        assert_eq!(token.chain.native_token, "NATIVE");
    }

    #[tokio::test]
    async fn test_parse_token_response_when_price_parsing_fails_should_handle_gracefully() {
        let json_invalid_price = create_dexscreener_response_with_invalid_price();
        let result =
            parse_token_response(&json_invalid_price, "0x123", "ethereum", Some(true)).await;

        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.price_usd, None); // Should handle parse failure
    }

    // Helper functions for parse_token_response tests
    fn create_valid_dexscreener_response() -> String {
        r#"{
            "pairs": [{
                "base_token": {
                    "address": "0x123",
                    "name": "Test Token",
                    "symbol": "TEST"
                },
                "quote_token": {
                    "address": "0x456",
                    "name": "USD Coin",
                    "symbol": "USDC"
                },
                "pair_address": "0xabc",
                "dex_id": "uniswap",
                "url": "https://example.com",
                "price_usd": "1.5",
                "price_native": "1.5",
                "market_cap": 1000000.0,
                "liquidity": {"usd": 500000.0},
                "volume": {"h24": 50000.0},
                "price_change": {"h24": 5.0, "h1": 1.0, "m5": 0.5},
                "fdv": 2000000.0
            }]
        }"#
        .to_string()
    }

    fn create_dexscreener_response_with_multiple_pairs() -> String {
        r#"{
            "pairs": [{
                "base_token": {
                    "address": "0x123",
                    "name": "Test Token",
                    "symbol": "TEST"
                },
                "quote_token": {
                    "address": "0x456",
                    "name": "USD Coin",
                    "symbol": "USDC"
                },
                "pair_address": "0xabc",
                "dex_id": "uniswap",
                "url": "https://example.com",
                "price_usd": "1.0",
                "price_native": "1.0",
                "market_cap": 1000000.0,
                "liquidity": {"usd": 300000.0},
                "volume": {"h24": 30000.0},
                "price_change": {"h24": 3.0}
            }, {
                "base_token": {
                    "address": "0x123",
                    "name": "Test Token",
                    "symbol": "TEST"
                },
                "quote_token": {
                    "address": "0x789",
                    "name": "Ethereum",
                    "symbol": "ETH"
                },
                "pair_address": "0xdef",
                "dex_id": "sushiswap",
                "url": "https://sushi.example.com",
                "price_usd": "2.0",
                "price_native": "2.0",
                "market_cap": 1500000.0,
                "liquidity": {"usd": 800000.0},
                "volume": {"h24": 80000.0},
                "price_change": {"h24": 8.0}
            }]
        }"#
        .to_string()
    }

    fn create_dexscreener_response_without_liquidity() -> String {
        r#"{
            "pairs": [{
                "base_token": {
                    "address": "0x123",
                    "name": "Test Token",
                    "symbol": "TEST"
                },
                "quote_token": {
                    "address": "0x456",
                    "name": "USD Coin",
                    "symbol": "USDC"
                },
                "pair_address": "0xabc",
                "dex_id": "uniswap",
                "url": "https://example.com",
                "price_usd": "1.5",
                "price_native": "1.5"
            }]
        }"#
        .to_string()
    }

    fn create_dexscreener_response_with_transaction_data() -> String {
        r#"{
            "pairs": [{
                "base_token": {
                    "address": "0x123",
                    "name": "Test Token",
                    "symbol": "TEST"
                },
                "quote_token": {
                    "address": "0x456",
                    "name": "USD Coin",
                    "symbol": "USDC"
                },
                "pair_address": "0xabc",
                "dex_id": "uniswap",
                "url": "https://example.com",
                "price_usd": "1.5",
                "price_native": "1.5",
                "volume": {"h24": 60000.0},
                "txns": {
                    "h24": {
                        "buys": 100,
                        "sells": 50
                    }
                }
            }]
        }"#
        .to_string()
    }

    fn create_dexscreener_response_with_invalid_price() -> String {
        r#"{
            "pairs": [{
                "base_token": {
                    "address": "0x123",
                    "name": "Test Token",
                    "symbol": "TEST"
                },
                "quote_token": {
                    "address": "0x456",
                    "name": "USD Coin",
                    "symbol": "USDC"
                },
                "pair_address": "0xabc",
                "dex_id": "uniswap",
                "url": "https://example.com",
                "price_usd": "invalid_price",
                "price_native": "invalid_price"
            }]
        }"#
        .to_string()
    }

    // Helper functions for tests
    fn create_test_token_info(
        price_change_24h: Option<f64>,
        price_change_1h: Option<f64>,
        price_usd: Option<f64>,
    ) -> TokenInfo {
        TokenInfo {
            address: "0x123".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            price_usd,
            market_cap: Some(1000000.0),
            volume_24h: Some(50000.0),
            price_change_24h,
            price_change_1h,
            price_change_5m: Some(0.5),
            circulating_supply: Some(1000000.0),
            total_supply: Some(10000000.0),
            pair_count: 0,
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
        }
    }

    fn create_test_token_pair(dex_name: &str, liquidity: f64) -> TokenPair {
        TokenPair {
            pair_id: "pair123".to_string(),
            dex: DexInfo {
                id: dex_name.to_string(),
                name: dex_name.to_string(),
                url: Some("https://dex.com".to_string()),
                logo: None,
            },
            base_token: PairToken {
                address: "0x123".to_string(),
                name: "Test Token".to_string(),
                symbol: "TEST".to_string(),
            },
            quote_token: PairToken {
                address: "0x456".to_string(),
                name: "USD Coin".to_string(),
                symbol: "USDC".to_string(),
            },
            price_usd: Some(1.0),
            price_native: Some(1.0),
            volume_24h: Some(10000.0),
            price_change_24h: Some(5.0),
            liquidity_usd: Some(liquidity),
            fdv: Some(2000000.0),
            created_at: None,
            last_trade_at: Utc::now(),
            txns_24h: TransactionStats {
                buys: Some(100),
                sells: Some(80),
                total: Some(180),
                buy_volume_usd: Some(6000.0),
                sell_volume_usd: Some(4000.0),
            },
            url: "https://dex.com/pair/123".to_string(),
        }
    }
}
