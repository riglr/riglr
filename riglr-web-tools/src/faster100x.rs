//! Faster100x holder analysis integration for advanced on-chain analytics
//!
//! This module provides sophisticated holder analysis capabilities by integrating with Faster100x,
//! enabling AI agents to analyze token holder distribution, whale activity, and holder trends
//! for informed trading decisions.

use crate::{client::WebClient, error::WebToolError};
use chrono::{DateTime, Utc};
use riglr_core::provider::ApplicationContext;
use riglr_macros::tool;
use schemars::JsonSchema;

const FASTER100X_API_KEY: &str = "FASTER100X_API_KEY";
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

// Private module for raw API types
mod api_types {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct ApiResponseRaw {
        pub data: DataRaw,
        pub success: Option<bool>,
        pub error: Option<String>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct DataRaw {
        pub symbol: Option<String>,
        pub name: Option<String>,
        pub total_holders: Option<u64>,
        pub unique_holders: Option<u64>,
        pub distribution: Option<DistributionRaw>,
        pub top_holders: Option<Vec<HolderRaw>>,
        pub concentration_risk: Option<ConcentrationRiskRaw>,
        pub recent_activity: Option<ActivityRaw>,
        pub transactions: Option<Vec<WhaleTransactionRaw>>,
        pub total_buys_usd: Option<f64>,
        pub total_sells_usd: Option<f64>,
        pub active_whales: Option<u64>,
        pub points: Option<Vec<TrendPointRaw>>,
        pub trend_direction: Option<String>,
        pub trend_strength: Option<u64>,
        pub insights: Option<Vec<String>>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct DistributionRaw {
        pub top_1_percent: Option<f64>,
        pub top_5_percent: Option<f64>,
        pub top_10_percent: Option<f64>,
        pub whale_percentage: Option<f64>,
        pub retail_percentage: Option<f64>,
        pub gini_coefficient: Option<f64>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct HolderRaw {
        pub address: Option<String>,
        pub balance: Option<f64>,
        pub percentage: Option<f64>,
        pub usd_value: Option<f64>,
        #[serde(rename = "type")]
        pub wallet_type: Option<String>,
        pub tx_count: Option<u64>,
        pub first_acquired: Option<String>,
        pub last_activity: Option<String>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct ConcentrationRiskRaw {
        pub level: Option<String>,
        pub score: Option<u64>,
        pub wallets_50_percent: Option<u64>,
        pub largest_holder: Option<f64>,
        pub exchange_percentage: Option<f64>,
        pub locked_percentage: Option<f64>,
        pub risk_factors: Option<Vec<String>>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct ActivityRaw {
        pub new_holders_24h: Option<u64>,
        pub exited_holders_24h: Option<u64>,
        pub net_change_24h: Option<i64>,
        pub avg_buy_size_24h: Option<f64>,
        pub avg_sell_size_24h: Option<f64>,
        pub growth_rate_7d: Option<f64>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct WhaleTransactionRaw {
        pub hash: Option<String>,
        pub from: Option<String>,
        #[serde(rename = "type")]
        pub transaction_type: Option<String>,
        pub token_amount: Option<f64>,
        pub usd_value: Option<f64>,
        pub timestamp: Option<String>,
        pub price_impact: Option<f64>,
        pub gas_fee: Option<f64>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct WhaleStatsRaw {
        pub total_whale_count: Option<u64>,
        pub whale_holding_percentage: Option<f64>,
        pub whale_activity_score: Option<f64>,
        pub whale_accumulation_trend: Option<String>,
        pub recent_whale_trend: Option<String>,
        pub average_whale_balance: Option<f64>,
        pub whale_dominance: Option<f64>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct TrendPointRaw {
        pub timestamp: Option<String>,
        pub holders: Option<u64>,
        pub change: Option<i64>,
        pub price: Option<f64>,
        pub whale_percentage: Option<f64>,
        pub volume_24h: Option<f64>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct TrendSummaryRaw {
        pub trend_direction: Option<String>,
        pub growth_rate_7d: Option<f64>,
        pub growth_rate_30d: Option<f64>,
        pub volatility_score: Option<f64>,
        pub health_score: Option<f64>,
        pub recommendation: Option<String>,
        pub risk_factors: Option<Vec<String>>,
        pub positive_factors: Option<Vec<String>>,
    }
}

/// Faster100x API configuration
#[derive(Debug, Clone)]
pub struct Faster100xConfig {
    /// API key for authentication with Faster100x service
    pub api_key: String,
    /// API base URL (default: https://api.faster100x.com/v1)
    pub base_url: String,
    /// Rate limit per minute (default: 100)
    pub rate_limit_per_minute: u32,
}

impl Default for Faster100xConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var(FASTER100X_API_KEY).unwrap_or_default(),
            base_url: "https://api.faster100x.com/v1".to_string(),
            rate_limit_per_minute: 100,
        }
    }
}

/// Token holder analysis data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenHolderAnalysis {
    /// Token contract address
    pub token_address: String,
    /// Token symbol
    pub token_symbol: String,
    /// Token name
    pub token_name: String,
    /// Total number of holders
    pub total_holders: u64,
    /// Number of unique holders
    pub unique_holders: u64,
    /// Holder distribution metrics
    pub distribution: HolderDistribution,
    /// Top holder wallets (anonymized)
    pub top_holders: Vec<WalletHolding>,
    /// Concentration risk metrics
    pub concentration_risk: ConcentrationRisk,
    /// Recent holder activity
    pub recent_activity: HolderActivity,
    /// Analysis timestamp
    pub timestamp: DateTime<Utc>,
    /// Chain identifier (eth, bsc, polygon, etc.)
    pub chain: String,
}

/// Holder distribution breakdown
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HolderDistribution {
    /// Percentage held by top 1% of holders
    pub top_1_percent: f64,
    /// Percentage held by top 5% of holders
    pub top_5_percent: f64,
    /// Percentage held by top 10% of holders
    pub top_10_percent: f64,
    /// Percentage held by whale wallets (>1% of supply)
    pub whale_percentage: f64,
    /// Percentage held by retail holders (<0.1% of supply)
    pub retail_percentage: f64,
    /// Gini coefficient (wealth inequality measure, 0-1)
    pub gini_coefficient: f64,
    /// Number of holders by category
    pub holder_categories: HashMap<String, u64>,
}

/// Individual wallet holding information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WalletHolding {
    /// Wallet address (potentially anonymized)
    pub wallet_address: String,
    /// Tokens held (in token units)
    pub token_amount: f64,
    /// Percentage of total supply
    pub percentage_of_supply: f64,
    /// USD value of holdings
    pub usd_value: f64,
    /// First acquisition date
    pub first_acquired: DateTime<Utc>,
    /// Last transaction date
    pub last_activity: DateTime<Utc>,
    /// Wallet type classification
    pub wallet_type: String, // "whale", "institution", "retail", "contract", "exchange"
    /// Number of transactions
    pub transaction_count: u64,
    /// Average holding time in days
    pub avg_holding_time_days: f64,
}

/// Concentration risk analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConcentrationRisk {
    /// Risk level (Low, Medium, High, Critical)
    pub risk_level: String,
    /// Risk score (0-100)
    pub risk_score: u8,
    /// Number of wallets that control 50% of supply
    pub wallets_controlling_50_percent: u64,
    /// Largest single holder percentage
    pub largest_holder_percentage: f64,
    /// Exchange holdings percentage
    pub exchange_holdings_percentage: f64,
    /// Contract/locked holdings percentage
    pub locked_holdings_percentage: f64,
    /// Risk factors
    pub risk_factors: Vec<String>,
}

/// Recent holder activity metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HolderActivity {
    /// New holders in last 24h
    pub new_holders_24h: u64,
    /// Holders who sold in last 24h
    pub exited_holders_24h: u64,
    /// Net holder change in 24h
    pub net_holder_change_24h: i64,
    /// Average buy size in last 24h (USD)
    pub avg_buy_size_24h: f64,
    /// Average sell size in last 24h (USD)
    pub avg_sell_size_24h: f64,
    /// Holder growth rate (7-day)
    pub holder_growth_rate_7d: f64,
}

/// Whale activity tracking data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WhaleActivity {
    /// Token being analyzed
    pub token_address: String,
    /// Whale transactions in the specified timeframe
    pub whale_transactions: Vec<WhaleTransaction>,
    /// Total whale buy volume
    pub total_whale_buys: f64,
    /// Total whale sell volume
    pub total_whale_sells: f64,
    /// Net whale flow (buys - sells)
    pub net_whale_flow: f64,
    /// Number of active whale wallets
    pub active_whales: u64,
    /// Analysis timeframe
    pub timeframe: String,
    /// Analysis timestamp
    pub timestamp: DateTime<Utc>,
}

/// Individual whale transaction
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WhaleTransaction {
    /// Transaction hash
    pub tx_hash: String,
    /// Whale wallet address
    pub wallet_address: String,
    /// Transaction type (buy/sell)
    pub transaction_type: String,
    /// Token amount
    pub token_amount: f64,
    /// USD value
    pub usd_value: f64,
    /// Transaction timestamp
    pub timestamp: DateTime<Utc>,
    /// Gas fee paid
    pub gas_fee: f64,
    /// Price impact percentage
    pub price_impact: f64,
}

/// Holder trends over time
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HolderTrends {
    /// Token address
    pub token_address: String,
    /// Time series data points
    pub data_points: Vec<HolderTrendPoint>,
    /// Overall trend direction
    pub trend_direction: String, // "increasing", "decreasing", "stable"
    /// Trend strength (0-100)
    pub trend_strength: u8,
    /// Analysis period
    pub analysis_period: String,
    /// Key insights
    pub insights: Vec<String>,
}

/// Individual holder trend data point
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HolderTrendPoint {
    /// Data point timestamp
    pub timestamp: DateTime<Utc>,
    /// Total holders at this time
    pub total_holders: u64,
    /// Holder change from previous point
    pub holder_change: i64,
    /// Token price at this time
    pub token_price: f64,
    /// Whale percentage at this time
    pub whale_percentage: f64,
    /// Trading volume at this time
    pub volume_24h: f64,
}

// Conversion functions from Raw to Clean types

/// Converts raw holder data to clean WalletHolding
fn convert_raw_holder(raw: &api_types::HolderRaw) -> WalletHolding {
    let first_acquired = raw
        .first_acquired
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));

    let last_activity = raw
        .last_activity
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));

    WalletHolding {
        wallet_address: raw.address.clone().unwrap_or_else(|| "unknown".to_string()),
        token_amount: raw.balance.unwrap_or(0.0),
        percentage_of_supply: raw.percentage.unwrap_or(0.0),
        usd_value: raw.usd_value.unwrap_or(0.0),
        first_acquired,
        last_activity,
        wallet_type: raw
            .wallet_type
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        transaction_count: raw.tx_count.unwrap_or(0),
        avg_holding_time_days: 0.0, // Would be calculated separately
    }
}

/// Converts raw distribution data to clean HolderDistribution
fn convert_raw_distribution(raw: &api_types::DistributionRaw) -> HolderDistribution {
    HolderDistribution {
        top_1_percent: raw.top_1_percent.unwrap_or(0.0),
        top_5_percent: raw.top_5_percent.unwrap_or(0.0),
        top_10_percent: raw.top_10_percent.unwrap_or(0.0),
        whale_percentage: raw.whale_percentage.unwrap_or(0.0),
        retail_percentage: raw.retail_percentage.unwrap_or(0.0),
        gini_coefficient: raw.gini_coefficient.unwrap_or(0.0),
        holder_categories: HashMap::new(),
    }
}

/// Converts raw concentration risk data to clean ConcentrationRisk
fn convert_raw_concentration_risk(raw: &api_types::ConcentrationRiskRaw) -> ConcentrationRisk {
    ConcentrationRisk {
        risk_level: raw.level.clone().unwrap_or_else(|| "Medium".to_string()),
        risk_score: raw.score.unwrap_or(50) as u8,
        wallets_controlling_50_percent: raw.wallets_50_percent.unwrap_or(0),
        largest_holder_percentage: raw.largest_holder.unwrap_or(0.0),
        exchange_holdings_percentage: raw.exchange_percentage.unwrap_or(0.0),
        locked_holdings_percentage: raw.locked_percentage.unwrap_or(0.0),
        risk_factors: raw.risk_factors.clone().unwrap_or_default(),
    }
}

/// Converts raw activity data to clean HolderActivity
fn convert_raw_activity(raw: &api_types::ActivityRaw) -> HolderActivity {
    HolderActivity {
        new_holders_24h: raw.new_holders_24h.unwrap_or(0),
        exited_holders_24h: raw.exited_holders_24h.unwrap_or(0),
        net_holder_change_24h: raw.net_change_24h.unwrap_or(0),
        avg_buy_size_24h: raw.avg_buy_size_24h.unwrap_or(0.0),
        avg_sell_size_24h: raw.avg_sell_size_24h.unwrap_or(0.0),
        holder_growth_rate_7d: raw.growth_rate_7d.unwrap_or(0.0),
    }
}

/// Converts raw whale transaction to clean WhaleTransaction
fn convert_raw_whale_transaction(raw: &api_types::WhaleTransactionRaw) -> WhaleTransaction {
    let timestamp = raw
        .timestamp
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));

    WhaleTransaction {
        tx_hash: raw.hash.clone().unwrap_or_default(),
        wallet_address: raw.from.clone().unwrap_or_default(),
        transaction_type: raw
            .transaction_type
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        token_amount: raw.token_amount.unwrap_or(0.0),
        usd_value: raw.usd_value.unwrap_or(0.0),
        timestamp,
        price_impact: raw.price_impact.unwrap_or(0.0),
        gas_fee: raw.gas_fee.unwrap_or(0.0),
    }
}

/// Converts raw trend point to clean HolderTrendPoint
fn convert_raw_trend_point(raw: &api_types::TrendPointRaw) -> HolderTrendPoint {
    let timestamp = raw
        .timestamp
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));

    HolderTrendPoint {
        timestamp,
        total_holders: raw.holders.unwrap_or(0),
        holder_change: raw.change.unwrap_or(0),
        token_price: raw.price.unwrap_or(0.0),
        whale_percentage: raw.whale_percentage.unwrap_or(0.0),
        volume_24h: raw.volume_24h.unwrap_or(0.0),
    }
}

/// Creates a Faster100x API client with proper authentication
///
/// Initializes a WebClient configured for the Faster100x API with the API key
/// from the ApplicationContext or FASTER100X_API_KEY environment variable.
///
/// # Returns
/// A configured WebClient instance for Faster100x API calls
///
/// Helper function to get Faster100x API key from ApplicationContext
fn get_api_key_from_context(context: &ApplicationContext) -> Result<String, WebToolError> {
    context
        .config
        .providers
        .faster100x_api_key
        .clone()
        .ok_or_else(|| {
            WebToolError::Config(
                "Faster100x API key not configured. Set FASTER100X_API_KEY in your environment."
                    .to_string(),
            )
        })
}

/// # Errors
/// Returns WebToolError::Config if the FASTER100X_API_KEY is not found in context
async fn create_faster100x_client_with_context(
    context: &ApplicationContext,
) -> Result<WebClient, WebToolError> {
    let api_key = get_api_key_from_context(context)?;
    let client = WebClient::default().with_api_key("faster100x", api_key);
    Ok(client)
}

/// Creates a Faster100x API client with proper authentication
///
/// Initializes a WebClient configured for the Faster100x API with the API key
/// from the FASTER100X_API_KEY environment variable.
///
/// # Returns
/// A configured WebClient instance for Faster100x API calls
///
/// # Errors
/// Returns WebToolError::Config if the FASTER100X_API_KEY environment variable is not set
#[allow(dead_code)]
async fn create_faster100x_client() -> Result<WebClient, WebToolError> {
    let config = Faster100xConfig::default();

    if config.api_key.is_empty() {
        return Err(WebToolError::Config(
            "FASTER100X_API_KEY environment variable not set".to_string(),
        ));
    }

    let client = WebClient::default().with_api_key("faster100x", config.api_key);

    Ok(client)
}

/// Validates and normalizes token address format
///
/// Ensures the provided token address follows the correct Ethereum-style format
/// (42 characters starting with "0x") and normalizes it to lowercase.
///
/// # Arguments
/// * `address` - The token contract address to validate and normalize
///
/// # Returns
/// The normalized lowercase token address
///
/// # Errors
/// Returns WebToolError::Config if the address format is invalid
fn normalize_token_address(address: &str) -> Result<String, WebToolError> {
    let address = address.trim();

    // Basic validation for Ethereum-style addresses
    if address.len() != 42 || !address.starts_with("0x") {
        return Err(WebToolError::Config(
            "Invalid token address format. Expected 42-character hex string starting with 0x"
                .to_string(),
        ));
    }

    // Convert to lowercase for consistency
    Ok(address.to_lowercase())
}

#[tool]
/// Analyze token holder distribution and concentration risks
///
/// This tool provides comprehensive analysis of a token's holder distribution,
/// including concentration risks, whale analysis, and holder categorization.
///
/// # Arguments
/// * `token_address` - Token contract address (must be valid hex address)
/// * `chain` - Blockchain network: "eth", "bsc", "polygon", "arbitrum", "base". Defaults to "eth"
///
/// # Returns
/// Detailed holder analysis including distribution metrics and risk assessment
pub async fn analyze_token_holders(
    context: &ApplicationContext,
    token_address: String,
    chain: Option<String>,
) -> Result<TokenHolderAnalysis, WebToolError> {
    let chain = chain.unwrap_or_else(|| "eth".to_string());
    let normalized_address = normalize_token_address(&token_address)?;

    info!(
        "Analyzing token holders for {} on {}",
        normalized_address, chain
    );

    // Validate chain parameter
    if !["eth", "bsc", "polygon", "arbitrum", "base", "avalanche"].contains(&chain.as_str()) {
        return Err(WebToolError::Config(
            "Invalid chain. Must be one of: eth, bsc, polygon, arbitrum, base, avalanche"
                .to_string(),
        ));
    }

    let client = create_faster100x_client_with_context(context).await?;

    let mut params = HashMap::new();
    params.insert("chain".to_string(), chain.clone());
    params.insert("include_distribution".to_string(), "true".to_string());
    params.insert("include_top_holders".to_string(), "true".to_string());

    let config = Faster100xConfig::default();
    let url = format!("{}/tokens/{}/holders", config.base_url, normalized_address);

    let response_text = client.get_with_params(&url, &params).await?;

    // Deserialize response into raw API types
    let response: api_types::ApiResponseRaw = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {}", e)))?;

    let data = response.data;

    // Extract basic token info
    let token_symbol = data.symbol.unwrap_or_else(|| "UNKNOWN".to_string());
    let token_name = data.name.unwrap_or_else(|| "Unknown Token".to_string());
    let total_holders = data.total_holders.unwrap_or(0);
    let unique_holders = data.unique_holders.unwrap_or(total_holders);

    // Convert distribution data
    let distribution = data.distribution.as_ref().map_or_else(
        || HolderDistribution {
            top_1_percent: 0.0,
            top_5_percent: 0.0,
            top_10_percent: 0.0,
            whale_percentage: 0.0,
            retail_percentage: 0.0,
            gini_coefficient: 0.0,
            holder_categories: HashMap::new(),
        },
        convert_raw_distribution,
    );

    // Convert top holders
    let top_holders = data
        .top_holders
        .as_ref()
        .map(|holders| holders.iter().take(10).map(convert_raw_holder).collect())
        .unwrap_or_default();

    // Convert concentration risk
    let concentration_risk = data.concentration_risk.as_ref().map_or_else(
        || ConcentrationRisk {
            risk_level: "Medium".to_string(),
            risk_score: 50,
            wallets_controlling_50_percent: 0,
            largest_holder_percentage: 0.0,
            exchange_holdings_percentage: 0.0,
            locked_holdings_percentage: 0.0,
            risk_factors: vec![],
        },
        convert_raw_concentration_risk,
    );

    // Convert recent activity
    let recent_activity = data.recent_activity.as_ref().map_or_else(
        || HolderActivity {
            new_holders_24h: 0,
            exited_holders_24h: 0,
            net_holder_change_24h: 0,
            avg_buy_size_24h: 0.0,
            avg_sell_size_24h: 0.0,
            holder_growth_rate_7d: 0.0,
        },
        convert_raw_activity,
    );

    debug!(
        "Successfully analyzed {} holders for token {} with {}% whale concentration",
        total_holders, token_symbol, distribution.whale_percentage
    );

    Ok(TokenHolderAnalysis {
        token_address: normalized_address,
        token_symbol,
        token_name,
        total_holders,
        unique_holders,
        distribution,
        top_holders,
        concentration_risk,
        recent_activity,
        timestamp: Utc::now(),
        chain,
    })
}

#[tool]
/// Get whale activity for a specific token
///
/// This tool tracks large-holder (whale) transactions and movements,
/// providing insights into institutional and high-net-worth trading activity.
///
/// # Arguments
/// * `token_address` - Token contract address
/// * `timeframe` - Analysis timeframe: "1h", "4h", "24h", "7d". Defaults to "24h"
/// * `min_usd_value` - Minimum transaction value in USD to be considered whale activity. Defaults to 10000
///
/// # Returns
/// Whale activity analysis with transaction details and flow metrics
pub async fn get_whale_activity(
    context: &ApplicationContext,
    token_address: String,
    timeframe: Option<String>,
    min_usd_value: Option<f64>,
) -> Result<WhaleActivity, WebToolError> {
    let timeframe = timeframe.unwrap_or_else(|| "24h".to_string());
    let min_usd_value = min_usd_value.unwrap_or(10000.0);
    let normalized_address = normalize_token_address(&token_address)?;

    info!(
        "Analyzing whale activity for {} (timeframe: {}, min value: ${})",
        normalized_address, timeframe, min_usd_value
    );

    // Validate timeframe
    if !["1h", "4h", "24h", "7d"].contains(&timeframe.as_str()) {
        return Err(WebToolError::Config(
            "Invalid timeframe. Must be one of: 1h, 4h, 24h, 7d".to_string(),
        ));
    }

    let client = create_faster100x_client_with_context(context).await?;

    let mut params = HashMap::new();
    params.insert("timeframe".to_string(), timeframe.clone());
    params.insert("min_usd".to_string(), min_usd_value.to_string());
    params.insert("include_details".to_string(), "true".to_string());

    let config = Faster100xConfig::default();
    let url = format!(
        "{}/tokens/{}/whale-activity",
        config.base_url, normalized_address
    );

    let response_text = client.get_with_params(&url, &params).await?;

    // Deserialize response into raw API types
    let response: api_types::ApiResponseRaw = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {}", e)))?;

    let data = response.data;

    // Convert whale transactions
    let whale_transactions: Vec<WhaleTransaction> = data
        .transactions
        .as_ref()
        .map(|txs| txs.iter().map(convert_raw_whale_transaction).collect())
        .unwrap_or_default();

    // Extract summary metrics
    let total_whale_buys = data.total_buys_usd.unwrap_or(0.0);
    let total_whale_sells = data.total_sells_usd.unwrap_or(0.0);
    let net_whale_flow = total_whale_buys - total_whale_sells;
    let active_whales = data.active_whales.unwrap_or(0);

    debug!(
        "Found {} whale transactions with net flow of ${:.2} for token {}",
        whale_transactions.len(),
        net_whale_flow,
        normalized_address
    );

    Ok(WhaleActivity {
        token_address: normalized_address,
        whale_transactions,
        total_whale_buys,
        total_whale_sells,
        net_whale_flow,
        active_whales,
        timeframe,
        timestamp: Utc::now(),
    })
}

#[tool]
/// Get holder trends over time for a token
///
/// This tool provides historical analysis of holder growth/decline patterns,
/// correlating holder changes with price movements and market events.
///
/// # Arguments
/// * `token_address` - Token contract address
/// * `period` - Analysis period: "7d", "30d", "90d". Defaults to "30d"
/// * `data_points` - Number of data points to return (10-100). Defaults to 30
///
/// # Returns
/// Time series analysis of holder trends with insights and correlations
pub async fn get_holder_trends(
    context: &ApplicationContext,
    token_address: String,
    period: Option<String>,
    data_points: Option<u32>,
) -> Result<HolderTrends, WebToolError> {
    let period = period.unwrap_or_else(|| "30d".to_string());
    let data_points = data_points.unwrap_or(30).clamp(10, 100);
    let normalized_address = normalize_token_address(&token_address)?;

    info!(
        "Analyzing holder trends for {} (period: {}, data points: {})",
        normalized_address, period, data_points
    );

    // Validate period
    if !["7d", "30d", "90d"].contains(&period.as_str()) {
        return Err(WebToolError::Config(
            "Invalid period. Must be one of: 7d, 30d, 90d".to_string(),
        ));
    }

    let client = create_faster100x_client_with_context(context).await?;

    let mut params = HashMap::new();
    params.insert("period".to_string(), period.clone());
    params.insert("points".to_string(), data_points.to_string());
    params.insert("include_price".to_string(), "true".to_string());

    let config = Faster100xConfig::default();
    let url = format!(
        "{}/tokens/{}/holder-trends",
        config.base_url, normalized_address
    );

    let response_text = client.get_with_params(&url, &params).await?;

    // Deserialize response into raw API types
    let response: api_types::ApiResponseRaw = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {}", e)))?;

    let data = response.data;

    // Convert trend data points
    let trend_data_points: Vec<HolderTrendPoint> = data
        .points
        .as_ref()
        .map(|points| points.iter().map(convert_raw_trend_point).collect())
        .unwrap_or_default();

    // Extract trend analysis
    let trend_direction = data.trend_direction.unwrap_or_else(|| "stable".to_string());
    let trend_strength = data.trend_strength.unwrap_or(50) as u8;
    let insights = data.insights.unwrap_or_default();

    debug!(
        "Analyzed holder trends with {} data points, trend: {} (strength: {})",
        trend_data_points.len(),
        trend_direction,
        trend_strength
    );

    Ok(HolderTrends {
        token_address: normalized_address,
        data_points: trend_data_points,
        trend_direction,
        trend_strength,
        analysis_period: period,
        insights,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faster100x_config_default() {
        let config = Faster100xConfig::default();
        assert_eq!(config.base_url, "https://api.faster100x.com/v1");
        assert_eq!(config.rate_limit_per_minute, 100);
    }

    #[test]
    fn test_normalize_token_address() {
        // Valid address
        let result = normalize_token_address("0x1234567890123456789012345678901234567890");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "0x1234567890123456789012345678901234567890"
        );

        // Invalid address - too short
        let result = normalize_token_address("0x123");
        assert!(result.is_err());

        // Invalid address - no 0x prefix
        let result = normalize_token_address("1234567890123456789012345678901234567890");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_holder_analysis_serialization() {
        let analysis = TokenHolderAnalysis {
            token_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_symbol: "TEST".to_string(),
            token_name: "Test Token".to_string(),
            total_holders: 1000,
            unique_holders: 950,
            distribution: HolderDistribution {
                top_1_percent: 50.0,
                top_5_percent: 75.0,
                top_10_percent: 85.0,
                whale_percentage: 25.0,
                retail_percentage: 40.0,
                gini_coefficient: 0.65,
                holder_categories: HashMap::new(),
            },
            top_holders: Vec::new(),
            concentration_risk: ConcentrationRisk {
                risk_level: "Medium".to_string(),
                risk_score: 60,
                wallets_controlling_50_percent: 5,
                largest_holder_percentage: 15.0,
                exchange_holdings_percentage: 20.0,
                locked_holdings_percentage: 10.0,
                risk_factors: Vec::new(),
            },
            recent_activity: HolderActivity {
                new_holders_24h: 50,
                exited_holders_24h: 30,
                net_holder_change_24h: 20,
                avg_buy_size_24h: 500.0,
                avg_sell_size_24h: 300.0,
                holder_growth_rate_7d: 5.2,
            },
            timestamp: Utc::now(),
            chain: "eth".to_string(),
        };

        let serialized = serde_json::to_string(&analysis).unwrap();
        assert!(serialized.contains("TEST"));
        assert!(serialized.contains("eth"));
        assert!(serialized.contains("60"));
    }

    #[test]
    fn test_faster100x_config_with_env_var() {
        // Test when environment variable is set
        std::env::set_var(FASTER100X_API_KEY, "test-api-key");
        let config = Faster100xConfig::default();
        assert_eq!(config.api_key, "test-api-key");
        std::env::remove_var(FASTER100X_API_KEY);
    }

    #[test]
    fn test_faster100x_config_without_env_var() {
        // Test when environment variable is not set
        std::env::remove_var(FASTER100X_API_KEY);
        let config = Faster100xConfig::default();
        assert_eq!(config.api_key, "");
    }

    #[test]
    fn test_normalize_token_address_with_whitespace() {
        // Test address with leading/trailing whitespace
        let result = normalize_token_address("  0x1234567890123456789012345678901234567890  ");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "0x1234567890123456789012345678901234567890"
        );
    }

    #[test]
    fn test_normalize_token_address_uppercase_to_lowercase() {
        // Test uppercase address conversion to lowercase
        let result = normalize_token_address("0X1234567890ABCDEF1234567890ABCDEF12345678");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "0x1234567890abcdef1234567890abcdef12345678"
        );
    }

    #[test]
    fn test_normalize_token_address_invalid_length_too_long() {
        // Test address that's too long
        let result = normalize_token_address("0x12345678901234567890123456789012345678901");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid token address format"));
    }

    #[test]
    fn test_normalize_token_address_invalid_length_too_short() {
        // Test address that's too short
        let result = normalize_token_address("0x123456789012345678901234567890123456789");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid token address format"));
    }

    #[test]
    fn test_normalize_token_address_missing_0x_prefix() {
        // Test address without 0x prefix
        let result = normalize_token_address("1234567890123456789012345678901234567890");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid token address format"));
    }

    #[test]
    fn test_normalize_token_address_wrong_prefix() {
        // Test address with wrong prefix
        let result = normalize_token_address("0y1234567890123456789012345678901234567890");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid token address format"));
    }

    #[test]
    fn test_normalize_token_address_empty_string() {
        // Test empty string
        let result = normalize_token_address("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid token address format"));
    }

    #[test]
    fn test_normalize_token_address_exact_42_chars() {
        // Test address with exactly 42 characters
        let result = normalize_token_address("0x1234567890123456789012345678901234567890");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 42);
    }

    #[test]
    fn test_whale_activity_serialization() {
        let whale_activity = WhaleActivity {
            token_address: "0x1234567890123456789012345678901234567890".to_string(),
            whale_transactions: vec![WhaleTransaction {
                tx_hash: "0xabc123".to_string(),
                wallet_address: "0x9876543210987654321098765432109876543210".to_string(),
                transaction_type: "buy".to_string(),
                token_amount: 1000.0,
                usd_value: 50000.0,
                timestamp: Utc::now(),
                gas_fee: 0.01,
                price_impact: 0.5,
            }],
            total_whale_buys: 100000.0,
            total_whale_sells: 50000.0,
            net_whale_flow: 50000.0,
            active_whales: 5,
            timeframe: "24h".to_string(),
            timestamp: Utc::now(),
        };

        let serialized = serde_json::to_string(&whale_activity).unwrap();
        assert!(serialized.contains("buy"));
        assert!(serialized.contains("24h"));
        assert!(serialized.contains("50000"));
    }

    #[test]
    fn test_holder_trends_serialization() {
        let holder_trends = HolderTrends {
            token_address: "0x1234567890123456789012345678901234567890".to_string(),
            data_points: vec![HolderTrendPoint {
                timestamp: Utc::now(),
                total_holders: 1000,
                holder_change: 50,
                token_price: 1.5,
                whale_percentage: 25.0,
                volume_24h: 100000.0,
            }],
            trend_direction: "increasing".to_string(),
            trend_strength: 80,
            analysis_period: "30d".to_string(),
            insights: vec!["Steady growth in holder count".to_string()],
        };

        let serialized = serde_json::to_string(&holder_trends).unwrap();
        assert!(serialized.contains("increasing"));
        assert!(serialized.contains("30d"));
        assert!(serialized.contains("Steady growth"));
    }

    #[test]
    fn test_holder_distribution_default_values() {
        let distribution = HolderDistribution {
            top_1_percent: 0.0,
            top_5_percent: 0.0,
            top_10_percent: 0.0,
            whale_percentage: 0.0,
            retail_percentage: 0.0,
            gini_coefficient: 0.0,
            holder_categories: HashMap::new(),
        };

        assert_eq!(distribution.top_1_percent, 0.0);
        assert_eq!(distribution.gini_coefficient, 0.0);
        assert!(distribution.holder_categories.is_empty());
    }

    #[test]
    fn test_wallet_holding_creation() {
        let now = Utc::now();
        let wallet = WalletHolding {
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_amount: 1000.0,
            percentage_of_supply: 5.0,
            usd_value: 50000.0,
            first_acquired: now,
            last_activity: now,
            wallet_type: "whale".to_string(),
            transaction_count: 25,
            avg_holding_time_days: 30.5,
        };

        assert_eq!(wallet.token_amount, 1000.0);
        assert_eq!(wallet.wallet_type, "whale");
        assert_eq!(wallet.transaction_count, 25);
    }

    #[test]
    fn test_concentration_risk_values() {
        let risk = ConcentrationRisk {
            risk_level: "High".to_string(),
            risk_score: 85,
            wallets_controlling_50_percent: 3,
            largest_holder_percentage: 30.0,
            exchange_holdings_percentage: 15.0,
            locked_holdings_percentage: 5.0,
            risk_factors: vec!["Large single holder".to_string()],
        };

        assert_eq!(risk.risk_score, 85);
        assert_eq!(risk.wallets_controlling_50_percent, 3);
        assert!(!risk.risk_factors.is_empty());
    }

    #[test]
    fn test_holder_activity_negative_change() {
        let activity = HolderActivity {
            new_holders_24h: 20,
            exited_holders_24h: 30,
            net_holder_change_24h: -10,
            avg_buy_size_24h: 250.0,
            avg_sell_size_24h: 500.0,
            holder_growth_rate_7d: -2.5,
        };

        assert_eq!(activity.net_holder_change_24h, -10);
        assert_eq!(activity.holder_growth_rate_7d, -2.5);
    }

    #[test]
    fn test_whale_transaction_values() {
        let now = Utc::now();
        let tx = WhaleTransaction {
            tx_hash: "0xabcdef123456".to_string(),
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
            transaction_type: "sell".to_string(),
            token_amount: 5000.0,
            usd_value: 100000.0,
            timestamp: now,
            gas_fee: 0.05,
            price_impact: 2.5,
        };

        assert_eq!(tx.transaction_type, "sell");
        assert_eq!(tx.usd_value, 100000.0);
        assert_eq!(tx.price_impact, 2.5);
    }

    #[test]
    fn test_holder_trend_point_creation() {
        let now = Utc::now();
        let point = HolderTrendPoint {
            timestamp: now,
            total_holders: 1500,
            holder_change: 100,
            token_price: 2.5,
            whale_percentage: 20.0,
            volume_24h: 500000.0,
        };

        assert_eq!(point.total_holders, 1500);
        assert_eq!(point.holder_change, 100);
        assert_eq!(point.token_price, 2.5);
    }

    #[test]
    fn test_faster100x_config_clone() {
        let config = Faster100xConfig {
            api_key: "test-key".to_string(),
            base_url: "https://test.api.com".to_string(),
            rate_limit_per_minute: 50,
        };

        let cloned = config.clone();
        assert_eq!(config.api_key, cloned.api_key);
        assert_eq!(config.base_url, cloned.base_url);
        assert_eq!(config.rate_limit_per_minute, cloned.rate_limit_per_minute);
    }

    #[test]
    fn test_token_holder_analysis_clone() {
        let analysis = TokenHolderAnalysis {
            token_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_symbol: "TEST".to_string(),
            token_name: "Test Token".to_string(),
            total_holders: 1000,
            unique_holders: 950,
            distribution: HolderDistribution {
                top_1_percent: 50.0,
                top_5_percent: 75.0,
                top_10_percent: 85.0,
                whale_percentage: 25.0,
                retail_percentage: 40.0,
                gini_coefficient: 0.65,
                holder_categories: HashMap::new(),
            },
            top_holders: Vec::new(),
            concentration_risk: ConcentrationRisk {
                risk_level: "Medium".to_string(),
                risk_score: 60,
                wallets_controlling_50_percent: 5,
                largest_holder_percentage: 15.0,
                exchange_holdings_percentage: 20.0,
                locked_holdings_percentage: 10.0,
                risk_factors: Vec::new(),
            },
            recent_activity: HolderActivity {
                new_holders_24h: 50,
                exited_holders_24h: 30,
                net_holder_change_24h: 20,
                avg_buy_size_24h: 500.0,
                avg_sell_size_24h: 300.0,
                holder_growth_rate_7d: 5.2,
            },
            timestamp: Utc::now(),
            chain: "eth".to_string(),
        };

        let cloned = analysis.clone();
        assert_eq!(analysis.token_symbol, cloned.token_symbol);
        assert_eq!(analysis.total_holders, cloned.total_holders);
    }

    #[test]
    fn test_debug_implementations() {
        let config = Faster100xConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("Faster100xConfig"));

        let distribution = HolderDistribution {
            top_1_percent: 50.0,
            top_5_percent: 75.0,
            top_10_percent: 85.0,
            whale_percentage: 25.0,
            retail_percentage: 40.0,
            gini_coefficient: 0.65,
            holder_categories: HashMap::new(),
        };
        let debug_str = format!("{:?}", distribution);
        assert!(debug_str.contains("HolderDistribution"));
    }

    #[tokio::test]
    async fn test_create_faster100x_client_missing_api_key() {
        // Remove API key environment variable
        std::env::remove_var(FASTER100X_API_KEY);

        let result = create_faster100x_client().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("FASTER100X_API_KEY environment variable not set"));
    }

    #[tokio::test]
    async fn test_create_faster100x_client_with_api_key() {
        // Set API key environment variable
        std::env::set_var(FASTER100X_API_KEY, "test-api-key");

        let result = create_faster100x_client().await;
        // This might fail due to WebClient::default() requiring actual network setup,
        // but we can at least test the API key validation path
        if result.is_err() {
            // Ensure the error is not about missing API key
            assert!(!result
                .unwrap_err()
                .to_string()
                .contains("FASTER100X_API_KEY environment variable not set"));
        }

        std::env::remove_var(FASTER100X_API_KEY);
    }
}
