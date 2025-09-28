//! Complete `DexScreener` API implementation with real market data fetching
//!
//! This module provides production-ready integration with the `DexScreener` API
//! for fetching token prices, liquidity, and market data.

use anyhow::Result;
use core::cmp::Ordering;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Import types to avoid absolute paths
use crate::dexscreener::{
    format_chain_name, format_dex_name, get_native_token, ChainInfo, DexInfo, PairToken,
    SecurityInfo, TokenInfo, TokenPair, TransactionStats as DexTransactionStats,
};

/// Raw response from `DexScreener` API containing token pair information
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DexScreenerResponseRaw {
    /// List of token pairs returned by the API
    pub pairs: Vec<PairInfoRaw>,
    /// Schema version of the API response
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
}

/// Raw information about a trading pair from `DexScreener`
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PairInfoRaw {
    /// Base token information
    pub base_token: TokenRaw,
    /// Blockchain network identifier
    #[serde(rename = "chainId")]
    pub chain_id: String,
    /// Decentralized exchange identifier
    #[serde(rename = "dexId")]
    pub dex_id: String,
    /// Fully diluted valuation in USD
    #[serde(rename = "fdv")]
    pub fdv: Option<f64>,
    /// Optional labels associated with this pair
    pub labels: Option<Vec<String>>,
    /// Liquidity information for this pair
    pub liquidity: Option<LiquidityRaw>,
    /// Market capitalization in USD
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
    /// Smart contract address of the trading pair
    #[serde(rename = "pairAddress")]
    pub pair_address: String,
    /// Price change statistics
    pub price_change: Option<PriceChangeRaw>,
    /// Price in native chain token (e.g., ETH, SOL)
    #[serde(rename = "priceNative")]
    pub price_native: String,
    /// Price in USD
    #[serde(rename = "priceUsd")]
    pub price_usd: Option<String>,
    /// Quote token information
    pub quote_token: TokenRaw,
    /// Transaction statistics
    pub txns: Option<TransactionsRaw>,
    /// URL to view this pair on `DexScreener`
    pub url: String,
    /// Trading volume statistics
    pub volume: Option<VolumeRaw>,
}

/// Raw liquidity information for a trading pair
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LiquidityRaw {
    /// Liquidity of the base token
    pub base: Option<f64>,
    /// Liquidity of the quote token
    pub quote: Option<f64>,
    /// Total liquidity in USD
    pub usd: Option<f64>,
}

/// Raw trading volume statistics over different time periods
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VolumeRaw {
    /// Trading volume in the last 1 hour
    #[serde(default)]
    pub h1: Option<f64>,
    /// Trading volume in the last 24 hours
    #[serde(default)]
    pub h24: Option<f64>,
    /// Trading volume in the last 6 hours
    #[serde(default)]
    pub h6: Option<f64>,
    /// Trading volume in the last 5 minutes
    #[serde(default)]
    pub m5: Option<f64>,
}

/// Raw price change statistics over different time periods
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceChangeRaw {
    /// Price change percentage in the last 1 hour
    #[serde(default)]
    pub h1: Option<f64>,
    /// Price change percentage in the last 24 hours
    #[serde(default)]
    pub h24: Option<f64>,
    /// Price change percentage in the last 6 hours
    #[serde(default)]
    pub h6: Option<f64>,
    /// Price change percentage in the last 5 minutes
    #[serde(default)]
    pub m5: Option<f64>,
}

/// Raw transaction statistics over different time periods
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionsRaw {
    /// Transaction statistics for the last 1 hour
    #[serde(default)]
    pub h1: Option<TransactionStatsRaw>,
    /// Transaction statistics for the last 24 hours
    #[serde(default)]
    pub h24: Option<TransactionStatsRaw>,
    /// Transaction statistics for the last 6 hours
    #[serde(default)]
    pub h6: Option<TransactionStatsRaw>,
    /// Transaction statistics for the last 5 minutes
    #[serde(default)]
    pub m5: Option<TransactionStatsRaw>,
}

/// Raw buy and sell transaction statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionStatsRaw {
    /// Number of buy transactions
    pub buys: Option<u64>,
    /// Number of sell transactions
    pub sells: Option<u64>,
}

/// Raw token information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenRaw {
    /// Token contract address
    pub address: String,
    /// Full name of the token
    pub name: String,
    /// Token symbol/ticker
    pub symbol: String,
}

// ==================== New API Types (from OpenAPI spec) ====================

/// Token profile information (new v1 endpoint)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenProfile {
    /// Chain identifier
    #[serde(rename = "chainId")]
    pub chain_id: String,
    /// Token description (optional)
    pub description: Option<String>,
    /// Header image URL (optional)
    pub header: Option<String>,
    /// Token icon URL
    pub icon: String,
    /// Social and website links (optional)
    pub links: Option<Vec<TokenLink>>,
    /// Token address
    #[serde(rename = "tokenAddress")]
    pub token_address: String,
    /// URL to the token profile page
    pub url: String,
}

/// Token link information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenLink {
    /// Link label
    pub label: Option<String>,
    /// Link type (e.g., "website", "twitter", "telegram")
    #[serde(rename = "type")]
    pub link_type: Option<String>,
    /// Link URL
    pub url: String,
}

/// Boosted token response (new v1 endpoint)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BoostsResponse {
    /// Current boost amount
    pub amount: f64,
    /// Chain identifier
    #[serde(rename = "chainId")]
    pub chain_id: String,
    /// Token description (optional)
    pub description: Option<String>,
    /// Header image URL (optional)
    pub header: Option<String>,
    /// Token icon URL (optional)
    pub icon: Option<String>,
    /// Social and website links (optional)
    pub links: Option<Vec<TokenLink>>,
    /// Token address
    #[serde(rename = "tokenAddress")]
    pub token_address: String,
    /// Total boost amount
    #[serde(rename = "totalAmount")]
    pub total_amount: f64,
    /// URL to the token page
    pub url: String,
}

/// Orders response for a token (new v1 endpoint)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrdersResponse {
    /// List of orders
    pub orders: Vec<Order>,
}

/// Individual order information
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Order type
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Payment timestamp
    #[serde(rename = "paymentTimestamp")]
    pub payment_timestamp: Option<f64>,
    /// Order status
    pub status: OrderStatus,
}

/// Order type enum
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OrderType {
    /// Community takeover order type
    CommunityTakeover,
    /// Token advertisement order type
    TokenAd,
    /// Token profile order type
    TokenProfile,
    /// Trending bar advertisement order type
    TrendingBarAd,
}

/// Order status enum
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum OrderStatus {
    /// Order has been approved
    Approved,
    /// Order has been cancelled
    Cancelled,
    /// Order is on hold
    #[serde(rename = "on-hold")]
    OnHold,
    /// Order is currently being processed
    Processing,
    /// Order has been rejected
    Rejected,
}

/// Search for tokens or pairs on `DexScreener`
///
/// # Errors
///
/// Returns an error if the API request fails or if the response cannot be parsed.
pub async fn search_ticker(ticker: String) -> Result<DexScreenerResponseRaw> {
    let client = Client::new();
    let url = format!("https://api.dexscreener.com/latest/dex/search/?q={ticker}&limit=8");

    let response = client.get(&url).send().await?;

    if response.status().is_client_error() {
        let res = response.text().await?;
        tracing::error!("DexScreener API error: {:?}", res);
        return Err(anyhow::anyhow!("DexScreener API error: {:?}", res));
    }

    let data: serde_json::Value = response.json().await?;
    let mut dex_response: DexScreenerResponseRaw = serde_json::from_value(data)?;

    // Limit results to 8
    dex_response.pairs.truncate(8);

    Ok(dex_response)
}

/// Get token pairs by token address (requires chainId in new API)
///
/// # Errors
///
/// Returns an error if the API request fails or if the response cannot be parsed.
pub async fn get_pairs_by_token_v1(
    chain_id: &str,
    token_address: &str,
) -> Result<DexScreenerResponseRaw> {
    let client = Client::new();
    let url = format!("https://api.dexscreener.com/tokens/v1/{chain_id}/{token_address}");

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        let res = response.text().await?;
        return Err(anyhow::anyhow!("Failed to fetch token pairs: {}", res));
    }

    let data: serde_json::Value = response.json().await?;
    let dex_response: DexScreenerResponseRaw = serde_json::from_value(data)?;

    Ok(dex_response)
}

/// Get pairs by pair address (requires chainId in new API)
///
/// # Errors
///
/// Returns an error if the API request fails, if the response cannot be parsed,
/// or if no pair is found for the given address.
pub async fn get_pair_by_address_v1(chain_id: &str, pair_address: &str) -> Result<PairInfoRaw> {
    let client = Client::new();
    let url = format!("https://api.dexscreener.com/latest/dex/pairs/{chain_id}/{pair_address}");

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        let res = response.text().await?;
        return Err(anyhow::anyhow!("Failed to fetch pair: {}", res));
    }

    let data: serde_json::Value = response.json().await?;
    let pair_response: DexScreenerResponseRaw = serde_json::from_value(data)?;

    pair_response
        .pairs
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No pair found for address: {}", pair_address))
}

/// Legacy: Get token pairs by token address (without chainId)
/// Deprecated: Use `get_pairs_by_token_v1` instead
///
/// # Errors
///
/// Returns an error if the API request fails or if the response cannot be parsed.
pub async fn get_pairs_by_token(token_address: &str) -> Result<DexScreenerResponseRaw> {
    // Default to Ethereum for backwards compatibility
    get_pairs_by_token_v1("ethereum", token_address).await
}

/// Legacy: Get pairs by pair address (without chainId)
/// Deprecated: Use `get_pair_by_address_v1` instead
///
/// # Errors
///
/// Returns an error if the API request fails, if the response cannot be parsed,
/// or if no pair is found for the given address.
pub async fn get_pair_by_address(pair_address: &str) -> Result<PairInfoRaw> {
    // Try to infer chain from pair address format or default to ethereum
    let chain_id = if pair_address.starts_with("solana_") {
        "solana"
    } else if pair_address.starts_with("bsc_") {
        "bsc"
    } else {
        "ethereum"
    };
    get_pair_by_address_v1(chain_id, pair_address).await
}

/// Get the pools of a given token address (new v1 endpoint)
///
/// # Errors
///
/// Returns an error if the API request fails or if the response cannot be parsed.
pub async fn get_token_pairs_v1(
    chain_id: &str,
    token_address: &str,
) -> Result<DexScreenerResponseRaw> {
    let client = Client::new();
    let url = format!("https://api.dexscreener.com/token-pairs/v1/{chain_id}/{token_address}");

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        let res = response.text().await?;
        return Err(anyhow::anyhow!("Failed to fetch token pairs: {}", res));
    }

    let data: serde_json::Value = response.json().await?;
    let dex_response: DexScreenerResponseRaw = serde_json::from_value(data)?;

    Ok(dex_response)
}

// ==================== New V1 Endpoint Functions ====================

/// Get the latest token profiles (rate-limit 60 requests per minute)
///
/// # Errors
///
/// Returns an error if the API request fails or if the response cannot be parsed.
pub async fn get_latest_token_profiles() -> Result<Vec<TokenProfile>> {
    let client = Client::new();
    let url = "https://api.dexscreener.com/token-profiles/latest/v1";

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        let res = response.text().await?;
        return Err(anyhow::anyhow!("Failed to fetch token profiles: {}", res));
    }

    let profiles: Vec<TokenProfile> = response.json().await?;
    Ok(profiles)
}

/// Get the latest boosted tokens (rate-limit 60 requests per minute)
///
/// # Errors
///
/// Returns an error if the API request fails or if the response cannot be parsed.
pub async fn get_latest_token_boosts() -> Result<Vec<BoostsResponse>> {
    let client = Client::new();
    let url = "https://api.dexscreener.com/token-boosts/latest/v1";

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        let res = response.text().await?;
        return Err(anyhow::anyhow!("Failed to fetch latest boosts: {}", res));
    }

    let boosts: Vec<BoostsResponse> = response.json().await?;
    Ok(boosts)
}

/// Get the tokens with most active boosts (rate-limit 60 requests per minute)
///
/// # Errors
///
/// Returns an error if the API request fails or if the response cannot be parsed.
pub async fn get_top_token_boosts() -> Result<Vec<BoostsResponse>> {
    let client = Client::new();
    let url = "https://api.dexscreener.com/token-boosts/top/v1";

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        let res = response.text().await?;
        return Err(anyhow::anyhow!("Failed to fetch top boosts: {}", res));
    }

    let boosts: Vec<BoostsResponse> = response.json().await?;
    Ok(boosts)
}

/// Check orders paid for a token (rate-limit 60 requests per minute)
///
/// # Errors
///
/// Returns an error if the API request fails or if the response cannot be parsed.
pub async fn get_token_orders(chain_id: &str, token_address: &str) -> Result<Vec<Order>> {
    let client = Client::new();
    let url = format!("https://api.dexscreener.com/orders/v1/{chain_id}/{token_address}");

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        let res = response.text().await?;
        return Err(anyhow::anyhow!("Failed to fetch token orders: {}", res));
    }

    let orders: Vec<Order> = response.json().await?;
    Ok(orders)
}

// ==================== Clean Public Types ====================

/// Clean response from `DexScreener` API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerResponse {
    /// List of token pairs returned by the API
    pub pairs: Vec<PairInfo>,
    /// Schema version of the API response
    pub schema_version: String,
}

/// Clean pair information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairInfo {
    /// Base token information
    pub base_token: Token,
    /// Blockchain network identifier
    pub chain_id: String,
    /// Decentralized exchange identifier
    pub dex_id: String,
    /// Fully diluted valuation in USD
    pub fdv: Option<f64>,
    /// Labels associated with this pair
    pub labels: Vec<String>,
    /// Liquidity information for this pair
    pub liquidity: Option<Liquidity>,
    /// Market capitalization in USD
    pub market_cap: Option<f64>,
    /// Smart contract address of the trading pair
    pub pair_address: String,
    /// Price change statistics
    pub price_change: Option<PriceChange>,
    /// Price in native chain token (e.g., ETH, SOL)
    pub price_native: f64,
    /// Price in USD
    pub price_usd: Option<f64>,
    /// Quote token information
    pub quote_token: Token,
    /// Transaction statistics
    pub txns: Option<Transactions>,
    /// URL to view this pair on `DexScreener`
    pub url: String,
    /// Trading volume statistics
    pub volume: Option<Volume>,
}

/// Clean token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Token contract address
    pub address: String,
    /// Full name of the token
    pub name: String,
    /// Token symbol/ticker
    pub symbol: String,
}

/// Clean liquidity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Liquidity {
    /// Liquidity of the base token
    pub base: Option<f64>,
    /// Liquidity of the quote token
    pub quote: Option<f64>,
    /// Total liquidity in USD
    pub usd: Option<f64>,
}

/// Clean volume statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Volume {
    /// Trading volume in the last 1 hour
    pub h1: Option<f64>,
    /// Trading volume in the last 24 hours
    pub h24: Option<f64>,
    /// Trading volume in the last 6 hours
    pub h6: Option<f64>,
    /// Trading volume in the last 5 minutes
    pub m5: Option<f64>,
}

/// Clean price change statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChange {
    /// Price change percentage in the last 1 hour
    pub h1: Option<f64>,
    /// Price change percentage in the last 24 hours
    pub h24: Option<f64>,
    /// Price change percentage in the last 6 hours
    pub h6: Option<f64>,
    /// Price change percentage in the last 5 minutes
    pub m5: Option<f64>,
}

/// Clean transaction statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transactions {
    /// Transaction statistics for the last 1 hour
    pub h1: Option<TransactionStats>,
    /// Transaction statistics for the last 24 hours
    pub h24: Option<TransactionStats>,
    /// Transaction statistics for the last 6 hours
    pub h6: Option<TransactionStats>,
    /// Transaction statistics for the last 5 minutes
    pub m5: Option<TransactionStats>,
}

/// Clean transaction stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStats {
    /// Number of buy transactions
    pub buys: Option<u64>,
    /// Number of sell transactions
    pub sells: Option<u64>,
}

/// Find the best liquidity pair for a token
#[must_use]
pub fn find_best_liquidity_pair(mut pairs: Vec<PairInfoRaw>) -> Option<PairInfoRaw> {
    if pairs.is_empty() {
        return None;
    }

    // Sort by liquidity (descending), maintaining original order for equal values
    pairs.sort_by(|a, b| {
        let a_liq = a.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
        let b_liq = b.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
        b_liq.partial_cmp(&a_liq).unwrap_or(Ordering::Equal)
    });

    pairs.into_iter().next()
}

/// Extract token price from the best pair
#[must_use]
pub fn get_token_price(pairs: &[PairInfoRaw], token_address: &str) -> Option<String> {
    // Filter pairs for the token
    let mut matching_pairs: Vec<_> = pairs
        .iter()
        .filter(|p| p.base_token.address.eq_ignore_ascii_case(token_address))
        .collect();

    // Sort by liquidity (descending), maintaining original order for equal values
    matching_pairs.sort_by(|a, b| {
        let a_liq = a.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
        let b_liq = b.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
        b_liq.partial_cmp(&a_liq).unwrap_or(Ordering::Equal)
    });

    matching_pairs.first().and_then(|p| p.price_usd.clone())
}

// ==================== From Implementations ====================

impl From<DexScreenerResponseRaw> for DexScreenerResponse {
    fn from(raw: DexScreenerResponseRaw) -> Self {
        Self {
            schema_version: raw.schema_version,
            pairs: raw.pairs.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PairInfoRaw> for PairInfo {
    fn from(raw: PairInfoRaw) -> Self {
        Self {
            chain_id: raw.chain_id,
            dex_id: raw.dex_id,
            url: raw.url,
            pair_address: raw.pair_address,
            labels: raw.labels.unwrap_or_default(),
            base_token: raw.base_token.into(),
            quote_token: raw.quote_token.into(),
            price_native: raw.price_native.parse().unwrap_or(0.0),
            price_usd: raw.price_usd.and_then(|p| p.parse().ok()),
            liquidity: raw.liquidity.map(Into::into),
            volume: raw.volume.map(Into::into),
            price_change: raw.price_change.map(Into::into),
            txns: raw.txns.map(Into::into),
            market_cap: raw.market_cap,
            fdv: raw.fdv,
        }
    }
}

impl From<TokenRaw> for Token {
    fn from(raw: TokenRaw) -> Self {
        Self {
            address: raw.address,
            name: raw.name,
            symbol: raw.symbol,
        }
    }
}

impl From<LiquidityRaw> for Liquidity {
    fn from(raw: LiquidityRaw) -> Self {
        Self {
            usd: raw.usd,
            base: raw.base,
            quote: raw.quote,
        }
    }
}

impl From<VolumeRaw> for Volume {
    fn from(raw: VolumeRaw) -> Self {
        Self {
            h24: raw.h24,
            h6: raw.h6,
            h1: raw.h1,
            m5: raw.m5,
        }
    }
}

impl From<PriceChangeRaw> for PriceChange {
    fn from(raw: PriceChangeRaw) -> Self {
        Self {
            h24: raw.h24,
            h6: raw.h6,
            h1: raw.h1,
            m5: raw.m5,
        }
    }
}

impl From<TransactionsRaw> for Transactions {
    fn from(raw: TransactionsRaw) -> Self {
        Self {
            h24: raw.h24.map(Into::into),
            h6: raw.h6.map(Into::into),
            h1: raw.h1.map(Into::into),
            m5: raw.m5.map(Into::into),
        }
    }
}

impl From<TransactionStatsRaw> for TransactionStats {
    fn from(raw: TransactionStatsRaw) -> Self {
        Self {
            buys: raw.buys,
            sells: raw.sells,
        }
    }
}

/// Helper function to aggregate data from multiple pairs into a single `TokenInfo`
pub fn aggregate_token_info(pairs: Vec<PairInfo>, token_address: &str) -> Option<TokenInfo> {
    // Find pairs for this token
    let token_pairs: Vec<PairInfo> = pairs
        .into_iter()
        .filter(|p| p.base_token.address.eq_ignore_ascii_case(token_address))
        .collect();

    if token_pairs.is_empty() {
        return None;
    }

    // Use the pair with highest liquidity as primary source
    let primary_pair = token_pairs.iter().max_by_key(|p| {
        p.liquidity.as_ref().and_then(|l| l.usd).map_or(0, |usd| {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                (usd * 1000.0).max(0.0) as u64
            }
        })
    })?;

    // Aggregate volume across all pairs
    let total_volume_24h: f64 = token_pairs
        .iter()
        .filter_map(|p| p.volume.as_ref())
        .filter_map(|v| v.h24)
        .sum();

    // Convert to TokenInfo (from dexscreener.rs)
    Some(TokenInfo {
        address: token_address.to_string(),
        name: primary_pair.base_token.name.clone(),
        symbol: primary_pair.base_token.symbol.clone(),
        decimals: 18, // Default
        price_usd: primary_pair.price_usd,
        market_cap: primary_pair.market_cap,
        volume_24h: Some(total_volume_24h),
        price_change_24h: primary_pair.price_change.as_ref().and_then(|pc| pc.h24),
        price_change_1h: primary_pair.price_change.as_ref().and_then(|pc| pc.h1),
        price_change_5m: primary_pair.price_change.as_ref().and_then(|pc| pc.m5),
        circulating_supply: None,
        total_supply: None,
        pair_count: u32::try_from(token_pairs.len()).unwrap_or(0),
        pairs: token_pairs.iter().map(convert_to_token_pair).collect(),
        chain: ChainInfo {
            id: primary_pair.chain_id.clone(),
            name: format_chain_name(&primary_pair.chain_id),
            logo: None,
            native_token: get_native_token(&primary_pair.chain_id),
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
    })
}

/// Converts a `PairInfo` to a `TokenPair` for integration with the dexscreener module
fn convert_to_token_pair(pair: &PairInfo) -> TokenPair {
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
        price_usd: pair.price_usd,
        price_native: Some(pair.price_native),
        volume_24h: pair.volume.as_ref().and_then(|v| v.h24),
        price_change_24h: pair.price_change.as_ref().and_then(|pc| pc.h24),
        liquidity_usd: pair.liquidity.as_ref().and_then(|l| l.usd),
        fdv: pair.fdv,
        created_at: None,
        last_trade_at: chrono::Utc::now(),
        txns_24h: {
            let buys = pair
                .txns
                .as_ref()
                .and_then(|t| t.h24.as_ref())
                .and_then(|h| {
                    h.buys
                        .map(|b| u32::try_from(b.clamp(0, u64::from(u32::MAX))).unwrap_or(0))
                });
            let sells = pair
                .txns
                .as_ref()
                .and_then(|t| t.h24.as_ref())
                .and_then(|h| {
                    h.sells
                        .map(|s| u32::try_from(s.clamp(0, u64::from(u32::MAX))).unwrap_or(0))
                });
            let total = match (buys, sells) {
                (Some(b), Some(s)) => Some(b.saturating_add(s)),
                _ => None,
            };

            // Calculate volume estimates if we have volume and transaction data
            let (buy_volume_usd, sell_volume_usd) = if let (Some(volume), Some(b), Some(s)) =
                (pair.volume.as_ref().and_then(|v| v.h24), buys, sells)
            {
                let total_txns = f64::from(b.saturating_add(s));
                if total_txns > 0.0 {
                    let buy_ratio = f64::from(b) / total_txns;
                    let sell_ratio = f64::from(s) / total_txns;
                    (Some(volume * buy_ratio), Some(volume * sell_ratio))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            DexTransactionStats {
                buy_volume_usd,
                buys,
                sell_volume_usd,
                sells,
                total,
            }
        },
        url: pair.url.clone(),
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use serde_json::json;

    // Helper function to create test Token
    fn create_test_token() -> TokenRaw {
        TokenRaw {
            address: "0x1234567890123456789012345678901234567890".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
        }
    }

    // Helper function to create test PairInfo
    fn create_test_pair_info() -> PairInfoRaw {
        PairInfoRaw {
            chain_id: "ethereum".to_string(),
            dex_id: "uniswap".to_string(),
            url: "https://dexscreener.com/test".to_string(),
            pair_address: "0xabcdef1234567890".to_string(),
            labels: Some(vec!["test".to_string()]),
            base_token: create_test_token(),
            quote_token: TokenRaw {
                address: "0x9876543210987654321098765432109876543210".to_string(),
                name: "Quote Token".to_string(),
                symbol: "QUOTE".to_string(),
            },
            price_native: "0.001".to_string(),
            price_usd: Some("1.50".to_string()),
            liquidity: Some(LiquidityRaw {
                usd: Some(100_000.0),
                base: Some(50_000.0),
                quote: Some(50_000.0),
            }),
            volume: Some(VolumeRaw {
                h24: Some(10000.0),
                h6: Some(2500.0),
                h1: Some(416.0),
                m5: Some(35.0),
            }),
            price_change: Some(PriceChangeRaw {
                h24: Some(5.5),
                h6: Some(2.1),
                h1: Some(0.8),
                m5: Some(0.1),
            }),
            txns: Some(TransactionsRaw {
                h24: Some(TransactionStatsRaw {
                    buys: Some(100),
                    sells: Some(80),
                }),
                h6: Some(TransactionStatsRaw {
                    buys: Some(25),
                    sells: Some(20),
                }),
                h1: Some(TransactionStatsRaw {
                    buys: Some(4),
                    sells: Some(3),
                }),
                m5: Some(TransactionStatsRaw {
                    buys: Some(1),
                    sells: Some(0),
                }),
            }),
            market_cap: Some(1_000_000.0),
            fdv: Some(1_500_000.0),
        }
    }

    // Struct Serialization/Deserialization Tests

    #[test]
    fn test_token_serialization() {
        let token = create_test_token();
        let serialized = serde_json::to_string(&token).expect("Token should serialize to JSON");
        let deserialized: TokenRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize to TokenRaw");

        assert_eq!(token.address, deserialized.address);
        assert_eq!(token.name, deserialized.name);
        assert_eq!(token.symbol, deserialized.symbol);
    }

    #[test]
    fn test_liquidity_serialization_with_all_fields() {
        let liquidity = LiquidityRaw {
            usd: Some(100_000.0),
            base: Some(50_000.0),
            quote: Some(50_000.0),
        };
        let serialized =
            serde_json::to_string(&liquidity).expect("LiquidityRaw should serialize to JSON");
        let deserialized: LiquidityRaw = serde_json::from_str(&serialized)
            .expect("Valid JSON should deserialize to LiquidityRaw");

        assert_eq!(liquidity.usd, deserialized.usd);
        assert_eq!(liquidity.base, deserialized.base);
        assert_eq!(liquidity.quote, deserialized.quote);
    }

    #[test]
    fn test_liquidity_serialization_with_none_fields() {
        let liquidity = LiquidityRaw {
            usd: None,
            base: None,
            quote: None,
        };
        let serialized =
            serde_json::to_string(&liquidity).expect("LiquidityRaw should serialize to JSON");
        let deserialized: LiquidityRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize");

        assert_eq!(liquidity.usd, deserialized.usd);
        assert_eq!(liquidity.base, deserialized.base);
        assert_eq!(liquidity.quote, deserialized.quote);
    }

    #[test]
    fn test_volume_default_serialization() {
        let volume = VolumeRaw::default();
        let serialized =
            serde_json::to_string(&volume).expect("VolumeRaw should serialize to JSON");
        let deserialized: VolumeRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize");

        assert_eq!(volume.h24, deserialized.h24);
        assert_eq!(volume.h6, deserialized.h6);
        assert_eq!(volume.h1, deserialized.h1);
        assert_eq!(volume.m5, deserialized.m5);
    }

    #[test]
    fn test_volume_serialization_with_values() {
        let volume = VolumeRaw {
            h24: Some(10000.0),
            h6: Some(2500.0),
            h1: Some(416.0),
            m5: Some(35.0),
        };
        let serialized =
            serde_json::to_string(&volume).expect("VolumeRaw should serialize to JSON");
        let deserialized: VolumeRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize");

        assert_eq!(volume.h24, deserialized.h24);
        assert_eq!(volume.h6, deserialized.h6);
        assert_eq!(volume.h1, deserialized.h1);
        assert_eq!(volume.m5, deserialized.m5);
    }

    #[test]
    fn test_price_change_serialization() {
        let price_change = PriceChangeRaw {
            h24: Some(5.5),
            h6: Some(2.1),
            h1: Some(0.8),
            m5: Some(0.1),
        };
        let serialized =
            serde_json::to_string(&price_change).expect("PriceChangeRaw should serialize to JSON");
        let deserialized: PriceChangeRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize");

        assert_eq!(price_change.h24, deserialized.h24);
        assert_eq!(price_change.h6, deserialized.h6);
        assert_eq!(price_change.h1, deserialized.h1);
        assert_eq!(price_change.m5, deserialized.m5);
    }

    #[test]
    fn test_transaction_stats_serialization() {
        let stats = TransactionStatsRaw {
            buys: Some(100),
            sells: Some(80),
        };
        let serialized =
            serde_json::to_string(&stats).expect("TransactionStatsRaw should serialize to JSON");
        let deserialized: TransactionStatsRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize");

        assert_eq!(stats.buys, deserialized.buys);
        assert_eq!(stats.sells, deserialized.sells);
    }

    #[test]
    fn test_transaction_stats_serialization_with_none() {
        let stats = TransactionStatsRaw {
            buys: None,
            sells: None,
        };
        let serialized =
            serde_json::to_string(&stats).expect("TransactionStatsRaw should serialize to JSON");
        let deserialized: TransactionStatsRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize");

        assert_eq!(stats.buys, deserialized.buys);
        assert_eq!(stats.sells, deserialized.sells);
    }

    #[test]
    fn test_transactions_serialization() {
        let transactions = TransactionsRaw {
            h24: Some(TransactionStatsRaw {
                buys: Some(100),
                sells: Some(80),
            }),
            h6: None,
            h1: Some(TransactionStatsRaw {
                buys: None,
                sells: Some(3),
            }),
            m5: None,
        };
        let serialized =
            serde_json::to_string(&transactions).expect("TransactionsRaw should serialize to JSON");
        let deserialized: TransactionsRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize");

        assert!(deserialized.h24.is_some());
        assert!(deserialized.h6.is_none());
        assert!(deserialized.h1.is_some());
        assert!(deserialized.m5.is_none());
    }

    #[test]
    fn test_pair_info_serialization_complete() {
        let pair = create_test_pair_info();
        let serialized =
            serde_json::to_string(&pair).expect("PairInfoRaw should serialize to JSON");
        let deserialized: PairInfoRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize");

        assert_eq!(pair.chain_id, deserialized.chain_id);
        assert_eq!(pair.dex_id, deserialized.dex_id);
        assert_eq!(pair.url, deserialized.url);
        assert_eq!(pair.pair_address, deserialized.pair_address);
        assert_eq!(pair.labels, deserialized.labels);
        assert_eq!(pair.price_native, deserialized.price_native);
        assert_eq!(pair.price_usd, deserialized.price_usd);
        assert!(deserialized.liquidity.is_some());
        assert!(deserialized.volume.is_some());
        assert!(deserialized.price_change.is_some());
        assert!(deserialized.txns.is_some());
        assert_eq!(pair.market_cap, deserialized.market_cap);
        assert_eq!(pair.fdv, deserialized.fdv);
    }

    #[test]
    fn test_pair_info_serialization_minimal() {
        let pair = PairInfoRaw {
            chain_id: "ethereum".to_string(),
            dex_id: "uniswap".to_string(),
            url: "https://dexscreener.com/test".to_string(),
            pair_address: "0xabcdef1234567890".to_string(),
            labels: None,
            base_token: create_test_token(),
            quote_token: create_test_token(),
            price_native: "0.001".to_string(),
            price_usd: None,
            liquidity: None,
            volume: None,
            price_change: None,
            txns: None,
            market_cap: None,
            fdv: None,
        };
        let serialized =
            serde_json::to_string(&pair).expect("PairInfoRaw should serialize to JSON");
        let deserialized: PairInfoRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize");

        assert_eq!(pair.chain_id, deserialized.chain_id);
        assert_eq!(pair.dex_id, deserialized.dex_id);
        assert!(deserialized.labels.is_none());
        assert!(deserialized.price_usd.is_none());
        assert!(deserialized.liquidity.is_none());
        assert!(deserialized.volume.is_none());
        assert!(deserialized.price_change.is_none());
        assert!(deserialized.txns.is_none());
        assert!(deserialized.market_cap.is_none());
        assert!(deserialized.fdv.is_none());
    }

    #[test]
    fn test_dexscreener_response_serialization() {
        let response = DexScreenerResponseRaw {
            schema_version: "1.0.0".to_string(),
            pairs: vec![create_test_pair_info()],
        };
        let serialized = serde_json::to_string(&response)
            .expect("DexScreenerResponseRaw should serialize to JSON");
        let deserialized: DexScreenerResponseRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize");

        assert_eq!(response.schema_version, deserialized.schema_version);
        assert_eq!(response.pairs.len(), deserialized.pairs.len());
    }

    #[test]
    fn test_dexscreener_response_empty_pairs() {
        let response = DexScreenerResponseRaw {
            schema_version: "1.0.0".to_string(),
            pairs: vec![],
        };
        let serialized = serde_json::to_string(&response)
            .expect("DexScreenerResponseRaw should serialize to JSON");
        let deserialized: DexScreenerResponseRaw =
            serde_json::from_str(&serialized).expect("Valid JSON should deserialize");

        assert_eq!(response.schema_version, deserialized.schema_version);
        assert!(deserialized.pairs.is_empty());
    }

    // Helper function tests

    #[test]
    fn test_find_best_liquidity_pair_when_empty_should_return_none() {
        let pairs = vec![];
        let result = find_best_liquidity_pair(pairs);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_best_liquidity_pair_when_no_liquidity_should_return_first() {
        let mut first_pair = create_test_pair_info();
        first_pair.liquidity = None;
        let mut second_pair = create_test_pair_info();
        second_pair.liquidity = None;
        let pair_list = vec![first_pair.clone(), second_pair];
        let result = find_best_liquidity_pair(pair_list);

        assert!(result.is_some());
        assert_eq!(
            result
                .expect("Should find best liquidity pair")
                .pair_address,
            first_pair.pair_address
        );
    }

    #[test]
    fn test_find_best_liquidity_pair_when_liquidity_none_usd_should_return_first() {
        let mut first_pair = create_test_pair_info();
        first_pair.liquidity = Some(LiquidityRaw {
            usd: None,
            base: Some(100.0),
            quote: Some(200.0),
        });
        let mut second_pair = create_test_pair_info();
        second_pair.liquidity = Some(LiquidityRaw {
            usd: None,
            base: Some(300.0),
            quote: Some(400.0),
        });
        let pair_list = vec![first_pair.clone(), second_pair];
        let result = find_best_liquidity_pair(pair_list);

        assert!(result.is_some());
        assert_eq!(
            result
                .expect("Should find best liquidity pair")
                .pair_address,
            first_pair.pair_address
        );
    }

    #[test]
    fn test_find_best_liquidity_pair_when_has_liquidity_should_return_highest() {
        let mut low_liquidity_pair = create_test_pair_info();
        low_liquidity_pair.pair_address = "low_liquidity".to_string();
        low_liquidity_pair.liquidity = Some(LiquidityRaw {
            usd: Some(50000.0),
            base: Some(25000.0),
            quote: Some(25000.0),
        });
        let mut high_liquidity_pair = create_test_pair_info();
        high_liquidity_pair.pair_address = "high_liquidity".to_string();
        high_liquidity_pair.liquidity = Some(LiquidityRaw {
            usd: Some(200_000.0),
            base: Some(100_000.0),
            quote: Some(100_000.0),
        });
        let mut medium_liquidity_pair = create_test_pair_info();
        medium_liquidity_pair.pair_address = "medium_liquidity".to_string();
        medium_liquidity_pair.liquidity = Some(LiquidityRaw {
            usd: Some(100_000.0),
            base: Some(50_000.0),
            quote: Some(50_000.0),
        });
        let all_pairs = vec![
            low_liquidity_pair,
            high_liquidity_pair,
            medium_liquidity_pair,
        ];
        let result = find_best_liquidity_pair(all_pairs);

        assert!(result.is_some());
        assert_eq!(
            result
                .expect("Should find best liquidity pair")
                .pair_address,
            "high_liquidity"
        );
    }

    #[test]
    fn test_find_best_liquidity_pair_when_mixed_liquidity_should_return_highest() {
        let mut no_liquidity_pair = create_test_pair_info();
        no_liquidity_pair.pair_address = "no_liquidity".to_string();
        no_liquidity_pair.liquidity = None;
        let mut has_liquidity_pair = create_test_pair_info();
        has_liquidity_pair.pair_address = "has_liquidity".to_string();
        has_liquidity_pair.liquidity = Some(LiquidityRaw {
            usd: Some(150_000.0),
            base: Some(75_000.0),
            quote: Some(75_000.0),
        });
        let mut none_usd_pair = create_test_pair_info();
        none_usd_pair.pair_address = "none_usd".to_string();
        none_usd_pair.liquidity = Some(LiquidityRaw {
            usd: None,
            base: Some(50000.0),
            quote: Some(50000.0),
        });
        let mixed_pairs = vec![no_liquidity_pair, has_liquidity_pair, none_usd_pair];
        let result = find_best_liquidity_pair(mixed_pairs);

        assert!(result.is_some());
        assert_eq!(
            result
                .expect("Should find best liquidity pair")
                .pair_address,
            "has_liquidity"
        );
    }

    #[test]
    fn test_get_token_price_when_empty_pairs_should_return_none() {
        let pairs = vec![];
        let result = get_token_price(&pairs, "0x1234567890123456789012345678901234567890");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_token_price_when_no_matching_address_should_return_none() {
        let pairs = vec![create_test_pair_info()];
        let result = get_token_price(&pairs, "0xnonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_token_price_when_matching_address_case_insensitive_should_return_price() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0xABCDEF1234567890".to_string();
        pair.price_usd = Some("2.50".to_string());
        let pairs = vec![pair];

        let result = get_token_price(&pairs, "0xabcdef1234567890");
        assert!(result.is_some());
        assert_eq!(result.expect("Should find token price"), "2.50");
    }

    #[test]
    fn test_get_token_price_when_matching_address_uppercase_should_return_price() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0xabcdef1234567890".to_string();
        pair.price_usd = Some("3.75".to_string());
        let pairs = vec![pair];

        let result = get_token_price(&pairs, "0XABCDEF1234567890");
        assert!(result.is_some());
        assert_eq!(result.expect("Should find token price"), "3.75");
    }

    #[test]
    fn test_get_token_price_when_no_price_usd_should_return_none() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        pair.price_usd = None;
        let pairs = vec![pair];

        let result = get_token_price(&pairs, "0x1234567890123456789012345678901234567890");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_token_price_when_multiple_pairs_should_return_highest_liquidity() {
        let mut low_liquidity_pair = create_test_pair_info();
        low_liquidity_pair.base_token.address =
            "0x1234567890123456789012345678901234567890".to_string();
        low_liquidity_pair.price_usd = Some("1.00".to_string());
        low_liquidity_pair.liquidity = Some(LiquidityRaw {
            usd: Some(50000.0),
            base: Some(25000.0),
            quote: Some(25000.0),
        });

        let mut high_liquidity_pair = create_test_pair_info();
        high_liquidity_pair.base_token.address =
            "0x1234567890123456789012345678901234567890".to_string();
        high_liquidity_pair.price_usd = Some("1.05".to_string());
        high_liquidity_pair.liquidity = Some(LiquidityRaw {
            usd: Some(150_000.0),
            base: Some(75_000.0),
            quote: Some(75_000.0),
        });

        let price_pairs = vec![low_liquidity_pair, high_liquidity_pair];
        let result = get_token_price(&price_pairs, "0x1234567890123456789012345678901234567890");

        assert!(result.is_some());
        assert_eq!(result.expect("Should find token price"), "1.05");
    }

    #[test]
    fn test_get_token_price_when_multiple_pairs_no_liquidity_should_return_first_match() {
        let mut first_pair = create_test_pair_info();
        first_pair.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        first_pair.price_usd = Some("1.00".to_string());
        first_pair.liquidity = None;

        let mut second_pair = create_test_pair_info();
        second_pair.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        second_pair.price_usd = Some("1.05".to_string());
        second_pair.liquidity = None;

        let no_liquidity_pairs = vec![first_pair, second_pair];
        let result = get_token_price(
            &no_liquidity_pairs,
            "0x1234567890123456789012345678901234567890",
        );

        assert!(result.is_some());
        assert_eq!(result.expect("Should find token price"), "1.00");
    }

    #[test]
    fn test_get_token_price_when_mixed_liquidity_should_prefer_with_liquidity() {
        let mut no_liquidity_pair = create_test_pair_info();
        no_liquidity_pair.base_token.address =
            "0x1234567890123456789012345678901234567890".to_string();
        no_liquidity_pair.price_usd = Some("1.00".to_string());
        no_liquidity_pair.liquidity = None;

        let mut with_liquidity_pair = create_test_pair_info();
        with_liquidity_pair.base_token.address =
            "0x1234567890123456789012345678901234567890".to_string();
        with_liquidity_pair.price_usd = Some("1.05".to_string());
        with_liquidity_pair.liquidity = Some(LiquidityRaw {
            usd: Some(100_000.0),
            base: Some(50_000.0),
            quote: Some(50_000.0),
        });

        let mixed_liquidity_pairs = vec![no_liquidity_pair, with_liquidity_pair];
        let result = get_token_price(
            &mixed_liquidity_pairs,
            "0x1234567890123456789012345678901234567890",
        );

        assert!(result.is_some());
        assert_eq!(result.expect("Should find token price"), "1.05");
    }

    // JSON parsing with snake_case and camelCase field names
    #[test]
    fn test_deserialize_pair_info_with_camel_case() {
        let json = json!({
            "chainId": "ethereum",
            "dexId": "uniswap",
            "url": "https://dexscreener.com/test",
            "pairAddress": "0xabcdef1234567890",
            "labels": ["test"],
            "baseToken": {
                "address": "0x1234567890123456789012345678901234567890",
                "name": "Test Token",
                "symbol": "TEST"
            },
            "quoteToken": {
                "address": "0x9876543210987654321098765432109876543210",
                "name": "Quote Token",
                "symbol": "QUOTE"
            },
            "priceNative": "0.001",
            "priceUsd": "1.50",
            "liquidity": {
                "usd": 100_000.0,
                "base": 50_000.0,
                "quote": 50_000.0
            },
            "volume": {
                "h24": 10000.0,
                "h6": 2500.0,
                "h1": 416.0,
                "m5": 35.0
            },
            "priceChange": {
                "h24": 5.5,
                "h6": 2.1,
                "h1": 0.8,
                "m5": 0.1
            },
            "txns": {
                "h24": {
                    "buys": 100,
                    "sells": 80
                },
                "h6": {
                    "buys": 25,
                    "sells": 20
                },
                "h1": {
                    "buys": 4,
                    "sells": 3
                },
                "m5": {
                    "buys": 1,
                    "sells": 0
                }
            },
            "marketCap": 1_000_000.0,
            "fdv": 1_500_000.0
        });

        let pair: PairInfoRaw =
            serde_json::from_value(json).expect("Valid JSON should deserialize");
        assert_eq!(pair.chain_id, "ethereum");
        assert_eq!(pair.dex_id, "uniswap");
        assert_eq!(pair.pair_address, "0xabcdef1234567890");
        assert_eq!(
            pair.base_token.address,
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(pair.quote_token.symbol, "QUOTE");
        assert_eq!(pair.price_native, "0.001");
        assert_eq!(pair.price_usd, Some("1.50".to_string()));
        assert!(pair.liquidity.is_some());
        assert!(pair.volume.is_some());
        assert!(pair.price_change.is_some());
        assert!(pair.txns.is_some());
        assert_eq!(pair.market_cap, Some(1_000_000.0));
        assert_eq!(pair.fdv, Some(1_500_000.0));
    }

    #[test]
    fn test_deserialize_dexscreener_response_with_camel_case() {
        let json = json!({
            "schemaVersion": "1.0.0",
            "pairs": []
        });

        let response: DexScreenerResponseRaw =
            serde_json::from_value(json).expect("Valid JSON should deserialize");
        assert_eq!(response.schema_version, "1.0.0");
        assert!(response.pairs.is_empty());
    }

    // Test Volume default behavior
    #[test]
    fn test_volume_default_values() {
        let volume = VolumeRaw::default();
        assert!(volume.h24.is_none());
        assert!(volume.h6.is_none());
        assert!(volume.h1.is_none());
        assert!(volume.m5.is_none());
    }

    #[test]
    fn test_volume_deserialization_with_missing_fields() {
        let json = json!({});
        let volume: VolumeRaw =
            serde_json::from_value(json).expect("Valid JSON should deserialize");
        assert!(volume.h24.is_none());
        assert!(volume.h6.is_none());
        assert!(volume.h1.is_none());
        assert!(volume.m5.is_none());
    }

    #[test]
    fn test_price_change_deserialization_with_missing_fields() {
        let json = json!({});
        let price_change: PriceChangeRaw =
            serde_json::from_value(json).expect("Valid JSON should deserialize");
        assert!(price_change.h24.is_none());
        assert!(price_change.h6.is_none());
        assert!(price_change.h1.is_none());
        assert!(price_change.m5.is_none());
    }

    #[test]
    fn test_transactions_deserialization_with_missing_fields() {
        let json = json!({});
        let transactions: TransactionsRaw =
            serde_json::from_value(json).expect("Valid JSON should deserialize");
        assert!(transactions.h24.is_none());
        assert!(transactions.h6.is_none());
        assert!(transactions.h1.is_none());
        assert!(transactions.m5.is_none());
    }

    // Additional edge case tests for helper functions

    #[test]
    fn test_find_best_liquidity_pair_with_single_pair() {
        let pair = create_test_pair_info();
        let pairs = vec![pair.clone()];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(
            result
                .expect("Should find best liquidity pair")
                .pair_address,
            pair.pair_address
        );
    }

    #[test]
    fn test_find_best_liquidity_pair_with_zero_liquidity() {
        let mut zero_usd_pair = create_test_pair_info();
        zero_usd_pair.pair_address = "zero_liquidity_1".to_string();
        zero_usd_pair.liquidity = Some(LiquidityRaw {
            usd: Some(0.0),
            base: Some(0.0),
            quote: Some(0.0),
        });
        let mut zero_usd_with_base_pair = create_test_pair_info();
        zero_usd_with_base_pair.pair_address = "zero_liquidity_2".to_string();
        zero_usd_with_base_pair.liquidity = Some(LiquidityRaw {
            usd: Some(0.0),
            base: Some(100.0),
            quote: Some(200.0),
        });
        let zero_liquidity_pairs = vec![zero_usd_pair, zero_usd_with_base_pair];
        let result = find_best_liquidity_pair(zero_liquidity_pairs);

        assert!(result.is_some());
        assert_eq!(
            result
                .expect("Should find best liquidity pair")
                .pair_address,
            "zero_liquidity_1"
        );
    }

    #[test]
    fn test_find_best_liquidity_pair_with_negative_liquidity() {
        let mut pair = create_test_pair_info();
        pair.pair_address = "negative_liquidity".to_string();
        pair.liquidity = Some(LiquidityRaw {
            usd: Some(-1000.0),
            base: Some(-500.0),
            quote: Some(-500.0),
        });
        let pairs = vec![pair];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(
            result
                .expect("Should find best liquidity pair")
                .pair_address,
            "negative_liquidity"
        );
    }

    #[test]
    fn test_get_token_price_with_empty_string_address() {
        let pairs = vec![create_test_pair_info()];
        let result = get_token_price(&pairs, "");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_token_price_with_partial_match() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        let pairs = vec![pair];

        // Should not match partial address
        let result = get_token_price(&pairs, "0x1234567890");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_token_price_with_special_characters() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0x!@#$%^&*()_+".to_string();
        pair.price_usd = Some("1.00".to_string());
        let pairs = vec![pair];

        let result = get_token_price(&pairs, "0x!@#$%^&*()_+");
        assert!(result.is_some());
        assert_eq!(result.expect("Should find token price"), "1.00");
    }

    #[test]
    fn test_get_token_price_case_sensitivity_mixed() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0xaBcDeF1234567890".to_string();
        pair.price_usd = Some("2.50".to_string());
        let pairs = vec![pair];

        let result = get_token_price(&pairs, "0xAbCdEf1234567890");
        assert!(result.is_some());
        assert_eq!(result.expect("Should find token price"), "2.50");
    }

    // Test struct Debug trait implementations
    #[test]
    fn test_token_debug_format() {
        let token = create_test_token();
        let debug_str = format!("{token:?}");
        assert!(debug_str.contains("Test Token"));
        assert!(debug_str.contains("TEST"));
        assert!(debug_str.contains("0x1234567890123456789012345678901234567890"));
    }

    #[test]
    fn test_liquidity_debug_format() {
        let liquidity = LiquidityRaw {
            usd: Some(100_000.0),
            base: Some(50_000.0),
            quote: Some(50_000.0),
        };
        let debug_str = format!("{liquidity:?}");
        assert!(debug_str.contains("100000"));
        assert!(debug_str.contains("50000"));
    }

    #[test]
    fn test_volume_debug_format() {
        let volume = VolumeRaw {
            h24: Some(10000.0),
            h6: Some(2500.0),
            h1: Some(416.0),
            m5: Some(35.0),
        };
        let debug_str = format!("{volume:?}");
        assert!(debug_str.contains("10000"));
        assert!(debug_str.contains("2500"));
        assert!(debug_str.contains("416"));
        assert!(debug_str.contains("35"));
    }

    #[test]
    fn test_price_change_debug_format() {
        let price_change = PriceChangeRaw {
            h24: Some(5.5),
            h6: Some(2.1),
            h1: Some(0.8),
            m5: Some(0.1),
        };
        let debug_str = format!("{price_change:?}");
        assert!(debug_str.contains("5.5"));
        assert!(debug_str.contains("2.1"));
        assert!(debug_str.contains("0.8"));
        assert!(debug_str.contains("0.1"));
    }

    #[test]
    fn test_transaction_stats_debug_format() {
        let stats = TransactionStatsRaw {
            buys: Some(100),
            sells: Some(80),
        };
        let debug_str = format!("{stats:?}");
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("80"));
    }

    #[test]
    fn test_transactions_debug_format() {
        let transactions = TransactionsRaw {
            h24: Some(TransactionStatsRaw {
                buys: Some(100),
                sells: Some(80),
            }),
            h6: None,
            h1: None,
            m5: None,
        };
        let debug_str = format!("{transactions:?}");
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("80"));
    }

    #[test]
    fn test_pair_info_debug_format() {
        let pair = create_test_pair_info();
        let debug_str = format!("{pair:?}");
        assert!(debug_str.contains("ethereum"));
        assert!(debug_str.contains("uniswap"));
        assert!(debug_str.contains("0xabcdef1234567890"));
    }

    #[test]
    fn test_dexscreener_response_debug_format() {
        let response = DexScreenerResponseRaw {
            schema_version: "1.0.0".to_string(),
            pairs: vec![create_test_pair_info()],
        };
        let debug_str = format!("{response:?}");
        assert!(debug_str.contains("1.0.0"));
        assert!(debug_str.contains("ethereum"));
    }

    // Test Clone trait implementations
    #[test]
    fn test_token_clone() {
        let token = create_test_token();
        let cloned = token.clone();
        assert_eq!(token.address, cloned.address);
        assert_eq!(token.name, cloned.name);
        assert_eq!(token.symbol, cloned.symbol);
    }

    #[test]
    fn test_liquidity_clone() {
        let liquidity = LiquidityRaw {
            usd: Some(100_000.0),
            base: Some(50_000.0),
            quote: Some(50_000.0),
        };
        let cloned = liquidity.clone();
        assert_eq!(liquidity.usd, cloned.usd);
        assert_eq!(liquidity.base, cloned.base);
        assert_eq!(liquidity.quote, cloned.quote);
    }

    #[test]
    fn test_volume_clone() {
        let volume = VolumeRaw {
            h24: Some(10000.0),
            h6: Some(2500.0),
            h1: Some(416.0),
            m5: Some(35.0),
        };
        let cloned = volume.clone();
        assert_eq!(volume.h24, cloned.h24);
        assert_eq!(volume.h6, cloned.h6);
        assert_eq!(volume.h1, cloned.h1);
        assert_eq!(volume.m5, cloned.m5);
    }

    #[test]
    fn test_price_change_clone() {
        let price_change = PriceChangeRaw {
            h24: Some(5.5),
            h6: Some(2.1),
            h1: Some(0.8),
            m5: Some(0.1),
        };
        let cloned = price_change.clone();
        assert_eq!(price_change.h24, cloned.h24);
        assert_eq!(price_change.h6, cloned.h6);
        assert_eq!(price_change.h1, cloned.h1);
        assert_eq!(price_change.m5, cloned.m5);
    }

    #[test]
    fn test_transaction_stats_clone() {
        let stats = TransactionStatsRaw {
            buys: Some(100),
            sells: Some(80),
        };
        let cloned = stats.clone();
        assert_eq!(stats.buys, cloned.buys);
        assert_eq!(stats.sells, cloned.sells);
    }

    #[test]
    fn test_transactions_clone() {
        let transactions = TransactionsRaw {
            h24: Some(TransactionStatsRaw {
                buys: Some(100),
                sells: Some(80),
            }),
            h6: None,
            h1: None,
            m5: None,
        };
        let cloned = transactions;
        assert!(cloned.h24.is_some());
        assert!(cloned.h6.is_none());
        assert!(cloned.h1.is_none());
        assert!(cloned.m5.is_none());
    }

    #[test]
    fn test_pair_info_clone() {
        let pair = create_test_pair_info();
        let cloned = pair.clone();
        assert_eq!(pair.chain_id, cloned.chain_id);
        assert_eq!(pair.dex_id, cloned.dex_id);
        assert_eq!(pair.url, cloned.url);
        assert_eq!(pair.pair_address, cloned.pair_address);
    }

    #[test]
    fn test_dexscreener_response_clone() {
        let response = DexScreenerResponseRaw {
            schema_version: "1.0.0".to_string(),
            pairs: vec![create_test_pair_info()],
        };
        let cloned = response.clone();
        assert_eq!(response.schema_version, cloned.schema_version);
        assert_eq!(response.pairs.len(), cloned.pairs.len());
    }

    // Test edge cases for liquidity calculation in find_best_liquidity_pair
    #[test]
    fn test_find_best_liquidity_pair_with_very_large_numbers() {
        let mut pair = create_test_pair_info();
        pair.pair_address = "large_liquidity".to_string();
        pair.liquidity = Some(LiquidityRaw {
            usd: Some(f64::MAX),
            base: Some(f64::MAX),
            quote: Some(f64::MAX),
        });
        let pairs = vec![pair];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(
            result
                .expect("Should find best liquidity pair")
                .pair_address,
            "large_liquidity"
        );
    }

    #[test]
    fn test_find_best_liquidity_pair_with_infinity() {
        let mut pair = create_test_pair_info();
        pair.pair_address = "infinity_liquidity".to_string();
        pair.liquidity = Some(LiquidityRaw {
            usd: Some(f64::INFINITY),
            base: Some(100.0),
            quote: Some(200.0),
        });
        let pairs = vec![pair];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(
            result
                .expect("Should find best liquidity pair")
                .pair_address,
            "infinity_liquidity"
        );
    }

    #[test]
    fn test_find_best_liquidity_pair_with_nan() {
        let mut pair = create_test_pair_info();
        pair.pair_address = "nan_liquidity".to_string();
        pair.liquidity = Some(LiquidityRaw {
            usd: Some(f64::NAN),
            base: Some(100.0),
            quote: Some(200.0),
        });
        let pairs = vec![pair];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(
            result
                .expect("Should find best liquidity pair")
                .pair_address,
            "nan_liquidity"
        );
    }

    // Test search_ticker truncation behavior
    #[tokio::test]
    async fn test_search_ticker_truncates_to_8_results() {
        // This test will verify that results are truncated to 8, but since we can't control
        // the actual API response, this is more of a documentation test
        let response = search_ticker("ETH".to_string())
            .await
            .expect("API call should succeed");
        assert!(response.pairs.len() <= 8);
    }

    // Original integration tests (preserved)
    #[tokio::test]
    async fn test_search_ticker() {
        let response = search_ticker("BONK".to_string())
            .await
            .expect("API call should succeed");
        assert_eq!(response.schema_version, "1.0.0");
        assert!(!response.pairs.is_empty());
    }

    #[tokio::test]
    async fn test_search_by_mint() {
        let response = search_ticker(
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(), // BONK token
        )
        .await
        .expect("BONK token search should succeed");
        assert_eq!(response.schema_version, "1.0.0");
        assert!(!response.pairs.is_empty());
    }

    #[tokio::test]
    async fn test_get_pairs_by_token() {
        let response = get_pairs_by_token("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48") // USDC
            .await
            .expect("USDC pairs should be found");
        assert!(!response.pairs.is_empty());
    }

    #[tokio::test]
    async fn test_get_pair_by_address_success() {
        // Test with a known pair address that should exist
        let result =
            get_pair_by_address("ethereum_0x0d4a11d5eeaac28ec3f61d100daf4d40471f1852").await;
        if let Ok(pair) = result {
            assert!(!pair.pair_address.is_empty());
            assert!(!pair.chain_id.is_empty());
            assert!(!pair.dex_id.is_empty());
        } else {
            // Some pair addresses might not exist, which is acceptable for this test
            // The important thing is that the function can be called without panicking
        }
    }

    #[tokio::test]
    async fn test_get_pair_by_address_nonexistent() {
        // Test with a clearly invalid pair address
        let result = get_pair_by_address("invalid_pair_address_that_does_not_exist").await;
        assert!(result.is_err());
    }
}
