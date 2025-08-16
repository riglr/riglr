//! Faster100x holder analysis integration for advanced on-chain analytics
//!
//! This module provides sophisticated holder analysis capabilities by integrating with Faster100x,
//! enabling AI agents to analyze token holder distribution, whale activity, and holder trends
//! for informed trading decisions.

use crate::{client::WebClient, error::WebToolError};
use chrono::{DateTime, Utc};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Faster100x API configuration
#[derive(Debug, Clone)]
pub struct Faster100xConfig {
    pub api_key: String,
    /// API base URL (default: https://api.faster100x.com/v1)
    pub base_url: String,
    /// Rate limit per minute (default: 100)
    pub rate_limit_per_minute: u32,
}

impl Default for Faster100xConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("FASTER100X_API_KEY").unwrap_or_default(),
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

/// Creates a Faster100x API client
async fn create_faster100x_client() -> Result<WebClient, WebToolError> {
    let config = Faster100xConfig::default();

    if config.api_key.is_empty() {
        return Err(WebToolError::Config(
            "FASTER100X_API_KEY environment variable not set".to_string(),
        ));
    }

    let client = WebClient::new()?.with_api_key("faster100x", config.api_key);

    Ok(client)
}

/// Validates and normalizes token address format
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

    let client = create_faster100x_client().await?;

    let mut params = HashMap::new();
    params.insert("chain".to_string(), chain.clone());
    params.insert("include_distribution".to_string(), "true".to_string());
    params.insert("include_top_holders".to_string(), "true".to_string());

    let config = Faster100xConfig::default();
    let url = format!("{}/tokens/{}/holders", config.base_url, normalized_address);

    let response_text = client.get_with_params(&url, &params).await?;

    let response_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {}", e)))?;

    let data = response_data
        .get("data")
        .ok_or_else(|| WebToolError::Parsing("Missing 'data' field".to_string()))?;

    // Parse basic token info
    let token_symbol = data
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN")
        .to_string();
    let token_name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Token")
        .to_string();
    let total_holders = data
        .get("total_holders")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let unique_holders = data
        .get("unique_holders")
        .and_then(|v| v.as_u64())
        .unwrap_or(total_holders);

    // Parse distribution data
    let distribution_data = data.get("distribution").unwrap_or(&serde_json::Value::Null);
    let distribution = HolderDistribution {
        top_1_percent: distribution_data
            .get("top_1_percent")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        top_5_percent: distribution_data
            .get("top_5_percent")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        top_10_percent: distribution_data
            .get("top_10_percent")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        whale_percentage: distribution_data
            .get("whale_percentage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        retail_percentage: distribution_data
            .get("retail_percentage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        gini_coefficient: distribution_data
            .get("gini_coefficient")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        holder_categories: HashMap::new(), // Would be populated from API response
    };

    // Parse top holders
    let mut top_holders = Vec::new();
    if let Some(holders_array) = data.get("top_holders").and_then(|v| v.as_array()) {
        for holder in holders_array.iter().take(10) {
            let wallet_address = holder
                .get("address")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let token_amount = holder
                .get("balance")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let percentage_of_supply = holder
                .get("percentage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let usd_value = holder
                .get("usd_value")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let wallet_type = holder
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let transaction_count = holder.get("tx_count").and_then(|v| v.as_u64()).unwrap_or(0);

            // Parse timestamps
            let first_acquired_str = holder
                .get("first_acquired")
                .and_then(|v| v.as_str())
                .unwrap_or("1970-01-01T00:00:00Z");
            let last_activity_str = holder
                .get("last_activity")
                .and_then(|v| v.as_str())
                .unwrap_or("1970-01-01T00:00:00Z");

            let first_acquired = DateTime::parse_from_rfc3339(first_acquired_str)
                .unwrap_or_else(|_| DateTime::from_timestamp(0, 0).unwrap().into())
                .with_timezone(&Utc);

            let last_activity = DateTime::parse_from_rfc3339(last_activity_str)
                .unwrap_or_else(|_| DateTime::from_timestamp(0, 0).unwrap().into())
                .with_timezone(&Utc);

            top_holders.push(WalletHolding {
                wallet_address,
                token_amount,
                percentage_of_supply,
                usd_value,
                first_acquired,
                last_activity,
                wallet_type,
                transaction_count,
                avg_holding_time_days: 0.0, // Would be calculated from API data
            });
        }
    }

    // Parse concentration risk
    let risk_data = data
        .get("concentration_risk")
        .unwrap_or(&serde_json::Value::Null);
    let concentration_risk = ConcentrationRisk {
        risk_level: risk_data
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("Medium")
            .to_string(),
        risk_score: risk_data
            .get("score")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as u8,
        wallets_controlling_50_percent: risk_data
            .get("wallets_50_percent")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        largest_holder_percentage: risk_data
            .get("largest_holder")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        exchange_holdings_percentage: risk_data
            .get("exchange_percentage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        locked_holdings_percentage: risk_data
            .get("locked_percentage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        risk_factors: Vec::new(), // Would be populated from API response
    };

    // Parse recent activity
    let activity_data = data
        .get("recent_activity")
        .unwrap_or(&serde_json::Value::Null);
    let recent_activity = HolderActivity {
        new_holders_24h: activity_data
            .get("new_holders_24h")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        exited_holders_24h: activity_data
            .get("exited_holders_24h")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        net_holder_change_24h: activity_data
            .get("net_change_24h")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        avg_buy_size_24h: activity_data
            .get("avg_buy_size_24h")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        avg_sell_size_24h: activity_data
            .get("avg_sell_size_24h")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        holder_growth_rate_7d: activity_data
            .get("growth_rate_7d")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
    };

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

    let client = create_faster100x_client().await?;

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

    let response_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {}", e)))?;

    let data = response_data
        .get("data")
        .ok_or_else(|| WebToolError::Parsing("Missing 'data' field".to_string()))?;

    // Parse whale transactions
    let mut whale_transactions = Vec::new();
    if let Some(transactions_array) = data.get("transactions").and_then(|v| v.as_array()) {
        for tx in transactions_array {
            let tx_hash = tx
                .get("hash")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let wallet_address = tx
                .get("from")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let transaction_type = tx
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let token_amount = tx
                .get("token_amount")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let usd_value = tx.get("usd_value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let gas_fee = tx.get("gas_fee").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let price_impact = tx
                .get("price_impact")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let timestamp_str = tx
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("1970-01-01T00:00:00Z");
            let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
                .unwrap_or_else(|_| DateTime::from_timestamp(0, 0).unwrap().into())
                .with_timezone(&Utc);

            whale_transactions.push(WhaleTransaction {
                tx_hash,
                wallet_address,
                transaction_type,
                token_amount,
                usd_value,
                timestamp,
                gas_fee,
                price_impact,
            });
        }
    }

    // Parse summary metrics
    let total_whale_buys = data
        .get("total_buys_usd")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let total_whale_sells = data
        .get("total_sells_usd")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let net_whale_flow = total_whale_buys - total_whale_sells;
    let active_whales = data
        .get("active_whales")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

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

    let client = create_faster100x_client().await?;

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

    let response_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {}", e)))?;

    let data = response_data
        .get("data")
        .ok_or_else(|| WebToolError::Parsing("Missing 'data' field".to_string()))?;

    // Parse trend data points
    let mut trend_data_points = Vec::new();
    if let Some(points_array) = data.get("points").and_then(|v| v.as_array()) {
        for point in points_array {
            let timestamp_str = point
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("1970-01-01T00:00:00Z");
            let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
                .unwrap_or_else(|_| DateTime::from_timestamp(0, 0).unwrap().into())
                .with_timezone(&Utc);

            let total_holders = point.get("holders").and_then(|v| v.as_u64()).unwrap_or(0);
            let holder_change = point.get("change").and_then(|v| v.as_i64()).unwrap_or(0);
            let token_price = point.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let whale_percentage = point
                .get("whale_percentage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let volume_24h = point
                .get("volume_24h")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            trend_data_points.push(HolderTrendPoint {
                timestamp,
                total_holders,
                holder_change,
                token_price,
                whale_percentage,
                volume_24h,
            });
        }
    }

    // Parse trend analysis
    let trend_direction = data
        .get("trend_direction")
        .and_then(|v| v.as_str())
        .unwrap_or("stable")
        .to_string();
    let trend_strength = data
        .get("trend_strength")
        .and_then(|v| v.as_u64())
        .unwrap_or(50) as u8;

    let insights = data
        .get("insights")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|insight| insight.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

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
}
