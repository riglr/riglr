//! `LunarCrush` social analytics integration for cryptocurrency sentiment analysis
//!
//! This module provides advanced social analytics capabilities by integrating with `LunarCrush`,
//! enabling AI agents to access comprehensive social sentiment data, influencer tracking,
//! and trending cryptocurrency analysis.

use crate::{client::WebClient, error::WebToolError};
use chrono::{DateTime, Utc};
use core::convert::Into;
use riglr_core::provider::ApplicationContext;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};
use tracing::{debug, info};

const LUNARCRUSH_API_KEY: &str = "LUNARCRUSH_API_KEY";

/// `LunarCrush` API configuration
#[derive(Debug, Clone)]
pub struct LunarCrushConfig {
    /// `LunarCrush` API key
    pub api_key: String,
    /// API base URL (default: <https://api.lunarcrush.com/v2>)
    pub base_url: String,
    /// Rate limit per minute (default: 60)
    pub rate_limit_per_minute: u32,
}

impl Default for LunarCrushConfig {
    fn default() -> Self {
        Self {
            api_key: env::var(LUNARCRUSH_API_KEY).unwrap_or_default(),
            base_url: "https://api.lunarcrush.com/v2".to_string(),
            rate_limit_per_minute: 60,
        }
    }
}

/// Social sentiment data for a cryptocurrency
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SentimentData {
    /// Average social volume over the timeframe
    pub avg_social_volume: f64,
    /// Number of posts from influencers
    pub influencer_posts: u64,
    /// Total mentions across all platforms
    pub mentions: u64,
    /// Sentiment breakdown by platform
    pub platform_sentiment: HashMap<String, f64>,
    /// Sentiment score (-1 to 1, where 1 is very positive)
    pub sentiment_score: f64,
    /// Overall social score (0-100)
    pub social_score: f64,
    /// Social volume (number of mentions)
    pub social_volume: u64,
    /// Social volume change percentage
    pub social_volume_change: f64,
    /// Cryptocurrency symbol (e.g., "BTC", "ETH")
    pub symbol: String,
    /// Data timestamp
    pub timestamp: DateTime<Utc>,
    /// Trending keywords associated with the token
    pub trending_keywords: Vec<String>,
}

/// Trending cryptocurrency with social metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrendingCrypto {
    /// `AltRank` (alternative ranking metric)
    pub alt_rank: u32,
    /// Galaxy score (`LunarCrush` proprietary metric)
    pub galaxy_score: f64,
    /// Market cap in USD
    pub market_cap: u64,
    /// Current market rank
    pub market_rank: u32,
    /// Full name of the cryptocurrency
    pub name: String,
    /// 24h price change percentage
    pub price_change_24h: f64,
    /// Price in USD
    pub price_usd: f64,
    /// Social score
    pub social_score: f64,
    /// Social volume
    pub social_volume: u64,
    /// Social volume change percentage
    pub social_volume_change: f64,
    /// Cryptocurrency symbol
    pub symbol: String,
    /// 24h trading volume
    pub volume_24h: u64,
}

/// Influencer mention data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InfluencerMention {
    /// Number of comments
    pub comments: u64,
    /// Number of followers
    pub followers: u64,
    /// Unique mention ID
    pub id: String,
    /// Influencer display name
    pub influencer_name: String,
    /// Influencer username
    pub influencer_username: String,
    /// Engagement metrics
    pub likes: u64,
    /// Platform (Twitter, Reddit, etc.)
    pub platform: String,
    /// Sentiment score for this specific mention
    pub sentiment: f64,
    /// Number of shares/retweets
    pub shares: u64,
    /// Post content/text
    pub text: String,
    /// Post timestamp
    pub timestamp: DateTime<Utc>,
    /// Post URL
    pub url: Option<String>,
}

/// Result containing multiple influencer mentions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InfluencerMentionsResult {
    /// Average sentiment across all mentions
    pub avg_sentiment: f64,
    /// List of influencer mentions
    pub mentions: Vec<InfluencerMention>,
    /// Token symbol that was searched
    pub symbol: String,
    /// Timeframe of the search
    pub timeframe: String,
    /// Total number of mentions found
    pub total_mentions: u64,
}

/// Helper function to get `LunarCrush` API key from `ApplicationContext`
fn get_api_key_from_context(context: &ApplicationContext) -> Result<String, WebToolError> {
    context
        .config
        .providers
        .lunarcrush_api_key
        .clone()
        .ok_or_else(|| {
            WebToolError::Config(
                "LunarCrush API key not configured. Set LUNARCRUSH_API_KEY in your environment."
                    .to_string(),
            )
        })
}

/// Creates a `LunarCrush` API client with context
fn create_lunarcrush_client_with_context(
    context: &ApplicationContext,
) -> Result<WebClient, WebToolError> {
    let api_key = get_api_key_from_context(context)?;
    let client = WebClient::default().with_api_key("lunarcrush", api_key);
    Ok(client)
}

/// Creates a `LunarCrush` API client from environment variables
#[cfg(test)]
fn create_lunarcrush_client() -> Result<WebClient, WebToolError> {
    let api_key = env::var(LUNARCRUSH_API_KEY).map_err(|_| {
        WebToolError::Config("LUNARCRUSH_API_KEY environment variable not set".to_string())
    })?;

    if api_key.is_empty() {
        return Err(WebToolError::Config(
            "LUNARCRUSH_API_KEY cannot be empty".to_string(),
        ));
    }

    let client = WebClient::default().with_api_key("lunarcrush", api_key);
    Ok(client)
}

#[tool]
/// Get social sentiment data for a cryptocurrency from `LunarCrush`
///
/// This tool provides comprehensive social analytics for any cryptocurrency,
/// including sentiment scores, social volume, and platform-specific data.
///
/// # Arguments
/// * `symbol` - Cryptocurrency symbol (e.g., "BTC", "ETH", "SOL")
/// * `timeframe` - Optional timeframe for analysis ("1h", "24h", "7d", "30d"). Defaults to "24h"
///
/// # Returns
/// Detailed sentiment analysis including scores, volume metrics, and trending keywords
///
/// # Errors
/// Returns `WebToolError::Config` if API key is not configured or timeframe is invalid.
/// Returns `WebToolError::Parsing` if the API response cannot be parsed or is missing required fields.
pub async fn get_social_sentiment(
    context: &ApplicationContext,
    symbol: String,
    timeframe: Option<String>,
) -> Result<SentimentData, WebToolError> {
    info!(
        "Fetching social sentiment for {} (timeframe: {})",
        symbol,
        timeframe.as_deref().unwrap_or("24h")
    );

    let client = create_lunarcrush_client_with_context(context)?;
    let timeframe = timeframe.unwrap_or_else(|| "24h".to_string());

    // Validate timeframe
    if !["1h", "24h", "7d", "30d"].contains(&timeframe.as_str()) {
        return Err(WebToolError::Config(
            "Invalid timeframe. Must be one of: 1h, 24h, 7d, 30d".to_string(),
        ));
    }

    let mut params = HashMap::new();
    params.insert("symbol".to_string(), symbol.to_uppercase());
    params.insert("data_points".to_string(), "1".to_string());
    params.insert("interval".to_string(), timeframe.clone());

    let config = LunarCrushConfig::default();
    let url = format!("{}/assets", config.base_url);

    let response_text = client.get_with_params(&url, &params).await?;

    let response_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {e}")))?;

    // Extract asset data from the response
    let data = response_data
        .get("data")
        .ok_or_else(|| WebToolError::Parsing("Missing 'data' field".to_string()))?;

    let assets = data
        .as_array()
        .ok_or_else(|| WebToolError::Parsing("'data' is not an array".to_string()))?;

    let asset = assets
        .first()
        .ok_or_else(|| WebToolError::Parsing(format!("No data found for symbol {symbol}")))?;

    // Parse the asset data
    let social_score = asset
        .get("galaxy_score")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let sentiment_score = asset
        .get("sentiment")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let social_volume = asset
        .get("social_volume")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let mentions = asset
        .get("total_mentions")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let influencer_posts = asset
        .get("influencer_posts")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let avg_social_volume = asset
        .get("avg_social_volume")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let social_volume_change = asset
        .get("social_volume_change_24h")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);

    // Extract platform sentiment breakdown
    let mut platform_sentiment = HashMap::new();
    if let Some(platforms) = asset.get("platform_sentiment") {
        if let Some(twitter_sentiment) =
            platforms.get("twitter").and_then(serde_json::Value::as_f64)
        {
            platform_sentiment.insert("twitter".to_string(), twitter_sentiment);
        }
        if let Some(reddit_sentiment) = platforms.get("reddit").and_then(serde_json::Value::as_f64)
        {
            platform_sentiment.insert("reddit".to_string(), reddit_sentiment);
        }
    }

    // Extract trending keywords
    let trending_keywords = asset
        .get("keywords")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|kw| kw.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();

    debug!(
        "Successfully fetched sentiment data for {} with {} mentions",
        symbol, mentions
    );

    Ok(SentimentData {
        avg_social_volume,
        influencer_posts,
        mentions,
        platform_sentiment,
        sentiment_score,
        social_score,
        social_volume,
        social_volume_change,
        symbol: symbol.to_uppercase(),
        timestamp: Utc::now(),
        trending_keywords,
    })
}

#[tool]
/// Get trending cryptocurrencies by social metrics from `LunarCrush`
///
/// This tool identifies the most trending cryptocurrencies based on social media activity,
/// sentiment, and `LunarCrush`'s proprietary Galaxy Score.
///
/// # Arguments
/// * `limit` - Maximum number of trending cryptocurrencies to return (1-50). Defaults to 10
/// * `sort_by` - Sort metric: "`galaxy_score`", "`social_volume`", "`alt_rank`". Defaults to "`galaxy_score`"
///
/// # Returns
/// List of trending cryptocurrencies with social and market metrics
///
/// # Errors
/// Returns `WebToolError::Config` if API key is not configured or `sort_by` parameter is invalid.
/// Returns `WebToolError::Parsing` if the API response cannot be parsed or is missing required fields.
#[expect(clippy::too_many_lines)]
pub async fn get_trending_cryptos(
    context: &ApplicationContext,
    limit: Option<u32>,
    sort_by: Option<String>,
) -> Result<Vec<TrendingCrypto>, WebToolError> {
    let limit = limit.unwrap_or(10).clamp(1, 50);
    let sort_by = sort_by.unwrap_or_else(|| "galaxy_score".to_string());

    info!(
        "Fetching top {} trending cryptos sorted by {}",
        limit, sort_by
    );

    // Validate sort_by parameter
    if !["galaxy_score", "social_volume", "alt_rank"].contains(&sort_by.as_str()) {
        return Err(WebToolError::Config(
            "Invalid sort_by parameter. Must be one of: galaxy_score, social_volume, alt_rank"
                .to_string(),
        ));
    }

    let client = create_lunarcrush_client_with_context(context)?;

    let mut params = HashMap::new();
    params.insert("limit".to_string(), limit.to_string());
    params.insert("sort".to_string(), sort_by);

    let config = LunarCrushConfig::default();
    let url = format!("{}/market", config.base_url);

    let response_text = client.get_with_params(&url, &params).await?;

    let response_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {e}")))?;

    let data = response_data
        .get("data")
        .ok_or_else(|| WebToolError::Parsing("Missing 'data' field".to_string()))?;

    let assets = data
        .as_array()
        .ok_or_else(|| WebToolError::Parsing("'data' is not an array".to_string()))?;

    let mut trending_cryptos = Vec::new();

    for asset in assets.iter().take(limit as usize) {
        let symbol = asset
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN")
            .to_string();

        let name = asset
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let market_rank = u32::try_from(
            asset
                .get("market_cap_rank")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
        )
        .unwrap_or(0);
        let galaxy_score = asset
            .get("galaxy_score")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let alt_rank = u32::try_from(
            asset
                .get("alt_rank")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
        )
        .unwrap_or(0);
        let social_score = asset
            .get("social_score")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let price_usd = asset
            .get("price")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let price_change_24h = asset
            .get("percent_change_24h")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let social_volume = asset
            .get("social_volume")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let social_volume_change = asset
            .get("social_volume_change_24h")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let market_cap = asset
            .get("market_cap")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let volume_24h = asset
            .get("volume_24h")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        trending_cryptos.push(TrendingCrypto {
            alt_rank,
            galaxy_score,
            market_cap,
            market_rank,
            name,
            price_change_24h,
            price_usd,
            social_score,
            social_volume,
            social_volume_change,
            symbol,
            volume_24h,
        });
    }

    debug!(
        "Successfully fetched {} trending cryptocurrencies",
        trending_cryptos.len()
    );

    Ok(trending_cryptos)
}

#[tool]
/// Get influencer mentions for a specific token from `LunarCrush`
///
/// This tool tracks mentions from cryptocurrency influencers and high-follower accounts,
/// providing insights into what key opinion leaders are saying about specific tokens.
///
/// # Arguments
/// * `token_symbol` - Cryptocurrency symbol to search for (e.g., "BTC", "ETH", "SOL")
/// * `limit` - Maximum number of mentions to return (1-50). Defaults to 20
/// * `timeframe` - Time period to search: "24h", "7d", "30d". Defaults to "24h"
///
/// # Returns
/// Collection of influencer mentions with engagement metrics and sentiment analysis
///
/// # Errors
/// Returns `WebToolError::Config` if API key is not configured or timeframe is invalid.
/// Returns `WebToolError::Parsing` if the API response cannot be parsed or is missing required fields.
#[expect(clippy::too_many_lines)]
pub async fn get_influencer_mentions(
    context: &ApplicationContext,
    token_symbol: String,
    limit: Option<u32>,
    timeframe: Option<String>,
) -> Result<InfluencerMentionsResult, WebToolError> {
    let limit = limit.unwrap_or(20).clamp(1, 50);
    let timeframe = timeframe.unwrap_or_else(|| "24h".to_string());

    info!(
        "Fetching influencer mentions for {} (limit: {}, timeframe: {})",
        token_symbol, limit, timeframe
    );

    // Validate timeframe
    if !["24h", "7d", "30d"].contains(&timeframe.as_str()) {
        return Err(WebToolError::Config(
            "Invalid timeframe. Must be one of: 24h, 7d, 30d".to_string(),
        ));
    }

    let client = create_lunarcrush_client_with_context(context)?;

    let mut params = HashMap::new();
    params.insert("symbol".to_string(), token_symbol.to_uppercase());
    params.insert("limit".to_string(), limit.to_string());
    params.insert("interval".to_string(), timeframe.clone());
    params.insert("sources".to_string(), "influencers".to_string());

    let config = LunarCrushConfig::default();
    let url = format!("{}/feeds", config.base_url);

    let response_text = client.get_with_params(&url, &params).await?;

    let response_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {e}")))?;

    let data = response_data
        .get("data")
        .ok_or_else(|| WebToolError::Parsing("Missing 'data' field".to_string()))?;

    let posts = data
        .as_array()
        .ok_or_else(|| WebToolError::Parsing("'data' is not an array".to_string()))?;

    let mut mentions = Vec::new();
    let mut total_sentiment = 0.0;

    for post in posts.iter().take(limit as usize) {
        let id = post
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let influencer_username = post
            .get("user_name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let influencer_name = post
            .get("user_display_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&influencer_username)
            .to_string();

        let followers = post
            .get("user_followers")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        let text = post
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let timestamp_str = post
            .get("created_time")
            .and_then(|v| v.as_str())
            .unwrap_or("1970-01-01T00:00:00Z");

        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .unwrap_or_else(|_| {
                DateTime::from_timestamp(0, 0).map_or_else(|| Utc::now().into(), Into::into)
            })
            .with_timezone(&Utc);

        let platform = post
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("twitter")
            .to_string();

        let likes = post
            .get("interactions")
            .and_then(|i| i.get("likes"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let shares = post
            .get("interactions")
            .and_then(|i| i.get("retweets"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let comments = post
            .get("interactions")
            .and_then(|i| i.get("replies"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        let url = post
            .get("url")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let sentiment = post
            .get("sentiment")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        total_sentiment += sentiment;

        mentions.push(InfluencerMention {
            comments,
            followers,
            id,
            influencer_name,
            influencer_username,
            likes,
            platform,
            sentiment,
            shares,
            text,
            timestamp,
            url,
        });
    }

    let avg_sentiment = if mentions.is_empty() {
        0.0
    } else {
        // Precision loss acceptable for typical social media data volumes
        #[expect(clippy::cast_precision_loss)]
        {
            total_sentiment / (mentions.len() as f64)
        }
    };

    debug!(
        "Successfully fetched {} influencer mentions for {} with avg sentiment {:.2}",
        mentions.len(),
        token_symbol,
        avg_sentiment
    );

    let total_mentions = mentions.len() as u64;

    Ok(InfluencerMentionsResult {
        avg_sentiment,
        mentions,
        symbol: token_symbol.to_uppercase(),
        timeframe,
        total_mentions,
    })
}

#[cfg(test)]
// Test helper functions with proper inline SAFETY documentation - unsafe blocks are required for Rust 2024 compatibility with std::env functions in test contexts
// Test code appropriately uses expect() with descriptive error messages for better debugging
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::env;

    // Tests for LunarCrushConfig
    #[test]
    #[expect(unsafe_code)]
    fn test_lunarcrush_config_default_without_env_var() {
        // Remove the env var if it exists
        // SAFETY: This is a test-only function used in isolated test environments
        // where we control the threading and environment variable access patterns.
        unsafe {
            env::remove_var(LUNARCRUSH_API_KEY);
        }

        let config = LunarCrushConfig::default();
        assert_eq!(config.api_key, "");
        assert_eq!(config.base_url, "https://api.lunarcrush.com/v2");
        assert_eq!(config.rate_limit_per_minute, 60);
    }

    #[test]
    #[expect(unsafe_code)]
    fn test_lunarcrush_config_default_with_env_var() {
        // SAFETY: This is a test-only function used in isolated test environments
        // where we control the threading and environment variable access patterns.
        unsafe {
            env::set_var(LUNARCRUSH_API_KEY, "test_api_key");
        }

        let config = LunarCrushConfig::default();
        assert_eq!(config.api_key, "test_api_key");
        assert_eq!(config.base_url, "https://api.lunarcrush.com/v2");
        assert_eq!(config.rate_limit_per_minute, 60);

        // SAFETY: This is a test-only function used in isolated test environments
        // where we control the threading and environment variable access patterns.
        unsafe {
            env::remove_var(LUNARCRUSH_API_KEY);
        }
    }

    #[test]
    fn test_lunarcrush_config_debug_trait() {
        let config = LunarCrushConfig {
            api_key: "test_key".to_string(),
            base_url: "https://test.com".to_string(),
            rate_limit_per_minute: 30,
        };

        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("test_key"));
        assert!(debug_str.contains("https://test.com"));
        assert!(debug_str.contains("30"));
    }

    #[test]
    fn test_lunarcrush_config_clone() {
        let config = LunarCrushConfig {
            api_key: "test_key".to_string(),
            base_url: "https://test.com".to_string(),
            rate_limit_per_minute: 30,
        };

        let cloned = config.clone();
        assert_eq!(config.api_key, cloned.api_key);
        assert_eq!(config.base_url, cloned.base_url);
        assert_eq!(config.rate_limit_per_minute, cloned.rate_limit_per_minute);
    }

    // Tests for SentimentData
    #[test]
    #[expect(clippy::unwrap_used)]
    fn test_sentiment_data_serialization_and_deserialization() {
        let mut platform_sentiment = HashMap::new();
        platform_sentiment.insert("twitter".to_string(), 0.8);
        platform_sentiment.insert("reddit".to_string(), 0.6);

        let sentiment = SentimentData {
            symbol: "BTC".to_string(),
            social_score: 85.5,
            sentiment_score: 0.7,
            social_volume: 10000,
            mentions: 15000,
            influencer_posts: 250,
            avg_social_volume: 9500.0,
            social_volume_change: 15.2,
            platform_sentiment,
            trending_keywords: vec!["bitcoin".to_string(), "crypto".to_string()],
            timestamp: Utc::now(),
        };

        let serialized = serde_json::to_string(&sentiment).unwrap();
        assert!(serialized.contains("BTC"));
        assert!(serialized.contains("85.5"));
        assert!(serialized.contains("twitter"));
        assert!(serialized.contains("bitcoin"));

        let deserialized: SentimentData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.symbol, "BTC");
        assert!((deserialized.social_score - 85.5).abs() < f64::EPSILON);
        assert_eq!(deserialized.platform_sentiment.len(), 2);
        assert_eq!(deserialized.trending_keywords.len(), 2);
    }

    #[test]
    fn test_sentiment_data_debug_trait() {
        let sentiment = SentimentData {
            symbol: "ETH".to_string(),
            social_score: 90.0,
            sentiment_score: 0.5,
            social_volume: 5000,
            mentions: 8000,
            influencer_posts: 100,
            avg_social_volume: 4800.0,
            social_volume_change: -5.2,
            platform_sentiment: HashMap::new(),
            trending_keywords: vec![],
            timestamp: Utc::now(),
        };

        let debug_str = format!("{sentiment:?}");
        assert!(debug_str.contains("ETH"));
        assert!(debug_str.contains("90"));
    }

    #[test]
    #[expect(clippy::float_cmp)]
    fn test_sentiment_data_clone() {
        let sentiment = SentimentData {
            symbol: "SOL".to_string(),
            social_score: 75.0,
            sentiment_score: 0.3,
            social_volume: 3000,
            mentions: 4000,
            influencer_posts: 50,
            avg_social_volume: 2900.0,
            social_volume_change: 10.5,
            platform_sentiment: HashMap::new(),
            trending_keywords: vec!["solana".to_string()],
            timestamp: Utc::now(),
        };

        let cloned = sentiment.clone();
        assert_eq!(sentiment.symbol, cloned.symbol);
        assert_eq!(sentiment.social_score, cloned.social_score);
        assert_eq!(sentiment.trending_keywords, cloned.trending_keywords);
    }

    // Tests for TrendingCrypto
    #[test]
    #[expect(clippy::unwrap_used)]
    fn test_trending_crypto_serialization_and_deserialization() {
        let trending = TrendingCrypto {
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            market_rank: 2,
            galaxy_score: 90.0,
            alt_rank: 1,
            social_score: 88.0,
            price_usd: 2500.0,
            price_change_24h: 5.2,
            social_volume: 25000,
            social_volume_change: 12.5,
            market_cap: 300_000_000_000,
            volume_24h: 15_000_000_000,
        };

        let serialized = serde_json::to_string(&trending).unwrap();
        assert!(serialized.contains("ETH"));
        assert!(serialized.contains("Ethereum"));
        assert!(serialized.contains("90"));

        let deserialized: TrendingCrypto = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.symbol, "ETH");
        assert_eq!(deserialized.name, "Ethereum");
        assert_eq!(deserialized.market_rank, 2);
    }

    #[test]
    fn test_trending_crypto_debug_trait() {
        let trending = TrendingCrypto {
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            market_rank: 1,
            galaxy_score: 95.0,
            alt_rank: 1,
            social_score: 92.0,
            price_usd: 50000.0,
            price_change_24h: -2.1,
            social_volume: 50000,
            social_volume_change: -8.3,
            market_cap: 1_000_000_000_000,
            volume_24h: 30_000_000_000,
        };

        let debug_str = format!("{trending:?}");
        assert!(debug_str.contains("BTC"));
        assert!(debug_str.contains("Bitcoin"));
        assert!(debug_str.contains("95"));
    }

    #[test]
    fn test_trending_crypto_clone() {
        let trending = TrendingCrypto {
            symbol: "ADA".to_string(),
            name: "Cardano".to_string(),
            market_rank: 5,
            galaxy_score: 70.0,
            alt_rank: 4,
            social_score: 68.0,
            price_usd: 1.5,
            price_change_24h: 3.4,
            social_volume: 8000,
            social_volume_change: 15.7,
            market_cap: 50_000_000_000,
            volume_24h: 2_000_000_000,
        };

        let cloned = trending.clone();
        assert_eq!(trending.symbol, cloned.symbol);
        assert_eq!(trending.name, cloned.name);
        assert_eq!(trending.market_rank, cloned.market_rank);
    }

    // Tests for InfluencerMention
    #[test]
    #[expect(clippy::unwrap_used)]
    fn test_influencer_mention_serialization_and_deserialization() {
        let mention = InfluencerMention {
            id: "12345".to_string(),
            influencer_username: "crypto_guru".to_string(),
            influencer_name: "Crypto Guru".to_string(),
            followers: 100_000,
            text: "Bitcoin is looking bullish!".to_string(),
            timestamp: Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).single().unwrap(),
            platform: "twitter".to_string(),
            likes: 500,
            shares: 100,
            comments: 50,
            url: Some("https://twitter.com/post/12345".to_string()),
            sentiment: 0.8,
        };

        let serialized = serde_json::to_string(&mention).unwrap();
        assert!(serialized.contains("12345"));
        assert!(serialized.contains("crypto_guru"));
        assert!(serialized.contains("Bitcoin is looking bullish"));

        let deserialized: InfluencerMention = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, "12345");
        assert_eq!(deserialized.influencer_username, "crypto_guru");
        assert_eq!(deserialized.followers, 100_000);
    }

    #[test]
    #[expect(clippy::unwrap_used)]
    fn test_influencer_mention_without_url() {
        let mention = InfluencerMention {
            id: "67890".to_string(),
            influencer_username: "trader_joe".to_string(),
            influencer_name: "Trader Joe".to_string(),
            followers: 50000,
            text: "Market is volatile".to_string(),
            timestamp: Utc::now(),
            platform: "reddit".to_string(),
            likes: 200,
            shares: 25,
            comments: 75,
            url: None,
            sentiment: -0.2,
        };

        let serialized = serde_json::to_string(&mention).unwrap();
        let deserialized: InfluencerMention = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.url, None);
        assert!((deserialized.sentiment - (-0.2)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_influencer_mention_debug_trait() {
        let mention = InfluencerMention {
            id: "test_id".to_string(),
            influencer_username: "test_user".to_string(),
            influencer_name: "Test User".to_string(),
            followers: 1000,
            text: "Test content".to_string(),
            timestamp: Utc::now(),
            platform: "twitter".to_string(),
            likes: 10,
            shares: 5,
            comments: 2,
            url: None,
            sentiment: 0.0,
        };

        let debug_str = format!("{mention:?}");
        assert!(debug_str.contains("test_id"));
        assert!(debug_str.contains("test_user"));
    }

    #[test]
    fn test_influencer_mention_clone() {
        let mention = InfluencerMention {
            id: "clone_test".to_string(),
            influencer_username: "clone_user".to_string(),
            influencer_name: "Clone User".to_string(),
            followers: 2000,
            text: "Clone test".to_string(),
            timestamp: Utc::now(),
            platform: "reddit".to_string(),
            likes: 20,
            shares: 10,
            comments: 5,
            url: Some("test_url".to_string()),
            sentiment: 0.5,
        };

        let cloned = mention.clone();
        assert_eq!(mention.id, cloned.id);
        assert_eq!(mention.influencer_username, cloned.influencer_username);
        assert_eq!(mention.url, cloned.url);
    }

    // Tests for InfluencerMentionsResult
    #[test]
    #[expect(clippy::unwrap_used)]
    fn test_influencer_mentions_result_serialization_and_deserialization() {
        let mentions = vec![InfluencerMention {
            id: "1".to_string(),
            influencer_username: "user1".to_string(),
            influencer_name: "User One".to_string(),
            followers: 1000,
            text: "Content 1".to_string(),
            timestamp: Utc::now(),
            platform: "twitter".to_string(),
            likes: 10,
            shares: 5,
            comments: 2,
            url: None,
            sentiment: 0.5,
        }];

        let result = InfluencerMentionsResult {
            symbol: "BTC".to_string(),
            mentions,
            total_mentions: 1,
            timeframe: "24h".to_string(),
            avg_sentiment: 0.5,
        };

        let serialized = serde_json::to_string(&result).unwrap();
        assert!(serialized.contains("BTC"));
        assert!(serialized.contains("24h"));

        let deserialized: InfluencerMentionsResult = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.symbol, "BTC");
        assert_eq!(deserialized.total_mentions, 1);
        assert_eq!(deserialized.timeframe, "24h");
    }

    #[test]
    fn test_influencer_mentions_result_empty_mentions() {
        let result = InfluencerMentionsResult {
            symbol: "ETH".to_string(),
            mentions: vec![],
            total_mentions: 0,
            timeframe: "7d".to_string(),
            avg_sentiment: 0.0,
        };

        assert_eq!(result.mentions.len(), 0);
        assert_eq!(result.total_mentions, 0);
        assert!((result.avg_sentiment - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_influencer_mentions_result_debug_trait() {
        let result = InfluencerMentionsResult {
            symbol: "SOL".to_string(),
            mentions: vec![],
            total_mentions: 5,
            timeframe: "30d".to_string(),
            avg_sentiment: 0.3,
        };

        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("SOL"));
        assert!(debug_str.contains("30d"));
        assert!(debug_str.contains("0.3"));
    }

    #[test]
    #[expect(clippy::float_cmp)]
    fn test_influencer_mentions_result_clone() {
        let result = InfluencerMentionsResult {
            symbol: "DOGE".to_string(),
            mentions: vec![],
            total_mentions: 10,
            timeframe: "24h".to_string(),
            avg_sentiment: -0.1,
        };

        let cloned = result.clone();
        assert_eq!(result.symbol, cloned.symbol);
        assert_eq!(result.total_mentions, cloned.total_mentions);
        assert_eq!(result.timeframe, cloned.timeframe);
        assert_eq!(result.avg_sentiment, cloned.avg_sentiment);
    }

    // Tests for create_lunarcrush_client function
    #[tokio::test]
    #[expect(unsafe_code)]
    async fn test_create_lunarcrush_client_missing_api_key() {
        // SAFETY: This is a test-only function used in isolated test environments
        // where we control the threading and environment variable access patterns.
        unsafe {
            env::remove_var(LUNARCRUSH_API_KEY);
        }

        let result = create_lunarcrush_client();
        assert!(result.is_err());

        if let Err(WebToolError::Config(msg)) = result {
            assert_eq!(msg, "LUNARCRUSH_API_KEY environment variable not set");
        } else {
            unreachable!("Expected Config error");
        }
    }

    #[tokio::test]
    #[expect(unsafe_code)]
    async fn test_create_lunarcrush_client_with_empty_api_key() {
        // SAFETY: This is a test-only function used in isolated test environments
        // where we control the threading and environment variable access patterns.
        unsafe {
            env::set_var(LUNARCRUSH_API_KEY, "");
        }

        let result = create_lunarcrush_client();
        assert!(result.is_err());

        // SAFETY: This is a test-only function used in isolated test environments
        // where we control the threading and environment variable access patterns.
        unsafe {
            env::remove_var(LUNARCRUSH_API_KEY);
        }
    }

    // Test timeframe validation in various functions
    #[test]
    fn test_timeframe_validation_get_social_sentiment() {
        // This tests the validation logic that would be used in get_social_sentiment
        let valid_timeframes = ["1h", "24h", "7d", "30d"];
        let invalid_timeframes = ["2h", "1d", "1w", "1m", "invalid"];

        for tf in valid_timeframes {
            assert!(["1h", "24h", "7d", "30d"].contains(&tf));
        }

        for tf in invalid_timeframes {
            assert!(!["1h", "24h", "7d", "30d"].contains(&tf));
        }
    }

    #[test]
    fn test_timeframe_validation_get_influencer_mentions() {
        // This tests the validation logic that would be used in get_influencer_mentions
        let valid_timeframes = ["24h", "7d", "30d"];
        let invalid_timeframes = ["1h", "1d", "1w", "1m", "invalid"];

        for tf in valid_timeframes {
            assert!(["24h", "7d", "30d"].contains(&tf));
        }

        for tf in invalid_timeframes {
            assert!(!["24h", "7d", "30d"].contains(&tf));
        }
    }

    #[test]
    fn test_sort_by_validation_get_trending_cryptos() {
        // This tests the validation logic that would be used in get_trending_cryptos
        let valid_sort_options = ["galaxy_score", "social_volume", "alt_rank"];
        let invalid_sort_options = ["price", "market_cap", "invalid"];

        for sort_by in valid_sort_options {
            assert!(["galaxy_score", "social_volume", "alt_rank"].contains(&sort_by));
        }

        for sort_by in invalid_sort_options {
            assert!(!["galaxy_score", "social_volume", "alt_rank"].contains(&sort_by));
        }
    }

    #[test]
    fn test_limit_clamping() {
        // Test limit clamping logic used in get_trending_cryptos
        let test_cases = [
            (None, 10),      // Default case
            (Some(0), 1),    // Below minimum
            (Some(5), 5),    // Valid value
            (Some(25), 25),  // Valid value
            (Some(100), 50), // Above maximum
        ];

        for (input, expected) in test_cases {
            let result = input.unwrap_or(10).clamp(1, 50);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_limit_clamping_influencer_mentions() {
        // Test limit clamping logic used in get_influencer_mentions
        let test_cases = [
            (None, 20),      // Default case
            (Some(0), 1),    // Below minimum
            (Some(10), 10),  // Valid value
            (Some(30), 30),  // Valid value
            (Some(100), 50), // Above maximum
        ];

        for (input, expected) in test_cases {
            let result = input.unwrap_or(20).clamp(1, 50);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_string_case_conversion() {
        // Test the string case conversion logic used throughout the module
        let symbols = ["btc", "ETH", "SoL", "DOGE"];
        let expected = ["BTC", "ETH", "SOL", "DOGE"];

        for (input, expected_output) in symbols.iter().zip(expected.iter()) {
            assert_eq!(input.to_uppercase(), *expected_output);
        }
    }

    #[test]
    fn test_datetime_parsing_edge_cases() {
        use chrono::DateTime;

        // Test valid RFC3339 datetime
        let valid_datetime = "2023-01-01T12:00:00Z";
        let parsed = DateTime::parse_from_rfc3339(valid_datetime);
        assert!(parsed.is_ok());

        // Test invalid datetime format
        let invalid_datetime = "invalid-datetime";
        let parsed = DateTime::parse_from_rfc3339(invalid_datetime);
        assert!(parsed.is_err());

        // Test fallback behavior (mimics the code in get_influencer_mentions)
        let fallback_timestamp = DateTime::parse_from_rfc3339("invalid")
            .unwrap_or_else(|_| {
                DateTime::from_timestamp(0, 0).map_or_else(|| Utc::now().into(), Into::into)
            })
            .with_timezone(&Utc);

        // Should be either epoch time or current time, both valid
        assert!(fallback_timestamp.timestamp() >= 0);
    }

    #[test]
    fn test_average_sentiment_calculation() {
        // Test sentiment average calculation logic used in get_influencer_mentions

        // Empty case
        let sentiments: Vec<f64> = vec![];
        let avg = if sentiments.is_empty() {
            0.0
        } else {
            {
                #[expect(clippy::cast_precision_loss)]
                {
                    sentiments.iter().sum::<f64>() / (sentiments.len() as f64)
                }
            }
        };
        assert!((avg - 0.0).abs() < f64::EPSILON);

        // Single value
        let sentiments = [0.5];
        #[expect(clippy::cast_precision_loss)]
        let avg = sentiments.iter().sum::<f64>() / (sentiments.len() as f64);
        assert!((avg - 0.5).abs() < f64::EPSILON);

        // Multiple values
        let sentiments = [0.8, -0.2, 0.1, 0.3];
        #[expect(clippy::cast_precision_loss)]
        let avg = sentiments.iter().sum::<f64>() / (sentiments.len() as f64);
        assert!((avg - 0.25).abs() < f64::EPSILON);

        // All positive
        let sentiments = [0.9, 0.8, 0.7];
        #[expect(clippy::cast_precision_loss)]
        let avg = sentiments.iter().sum::<f64>() / (sentiments.len() as f64);
        assert!((avg - 0.8).abs() < f64::EPSILON);

        // All negative
        let sentiments = [-0.5, -0.3, -0.2];
        #[expect(clippy::cast_precision_loss)]
        let avg = sentiments.iter().sum::<f64>() / (sentiments.len() as f64);
        assert!((avg - (-1.0 / 3.0)).abs() < 0.01);
    }

    #[test]
    fn test_json_field_extraction_patterns() {
        use serde_json::json;

        // Test the pattern used for extracting values with defaults
        let test_json = json!({
            "galaxy_score": 85.5,
            "sentiment": null,
            "social_volume": "not_a_number",
            "missing_field": null
        });

        // Test f64 extraction with default
        let galaxy_score = test_json
            .get("galaxy_score")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        assert!((galaxy_score - 85.5).abs() < f64::EPSILON);

        // Test f64 extraction with null value
        let sentiment = test_json
            .get("sentiment")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        assert!((sentiment - 0.0).abs() < f64::EPSILON);

        // Test u64 extraction with invalid string
        let social_volume = test_json
            .get("social_volume")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        assert_eq!(social_volume, 0);

        // Test missing field
        let missing = test_json
            .get("missing_field")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(99.9);
        assert!((missing - 99.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_platform_sentiment_extraction() {
        use serde_json::json;

        let test_data = json!({
            "platform_sentiment": {
                "twitter": 0.8,
                "reddit": 0.6,
                "youtube": "invalid"
            }
        });

        let mut platform_sentiment = HashMap::new();
        if let Some(platforms) = test_data.get("platform_sentiment") {
            if let Some(twitter_sentiment) =
                platforms.get("twitter").and_then(serde_json::Value::as_f64)
            {
                platform_sentiment.insert("twitter".to_string(), twitter_sentiment);
            }
            if let Some(reddit_sentiment) =
                platforms.get("reddit").and_then(serde_json::Value::as_f64)
            {
                platform_sentiment.insert("reddit".to_string(), reddit_sentiment);
            }
            if let Some(youtube_sentiment) =
                platforms.get("youtube").and_then(serde_json::Value::as_f64)
            {
                platform_sentiment.insert("youtube".to_string(), youtube_sentiment);
            }
        }

        assert_eq!(platform_sentiment.len(), 2);
        assert_eq!(platform_sentiment.get("twitter"), Some(&0.8));
        assert_eq!(platform_sentiment.get("reddit"), Some(&0.6));
        assert_eq!(platform_sentiment.get("youtube"), None);
    }

    #[test]
    fn test_keywords_extraction() {
        use serde_json::json;

        // Test valid keywords array
        let test_data = json!({
            "keywords": ["bitcoin", "crypto", "blockchain"]
        });

        let trending_keywords: Vec<String> = test_data
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|kw| kw.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        assert_eq!(trending_keywords.len(), 3);
        assert!(trending_keywords.contains(&"bitcoin".to_string()));

        // Test missing keywords field
        let test_data_empty = json!({});
        let trending_keywords_empty: Vec<String> = test_data_empty
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|kw| kw.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        assert_eq!(trending_keywords_empty.len(), 0);

        // Test keywords with mixed types
        let test_data_mixed = json!({
            "keywords": ["bitcoin", 123, null, "ethereum"]
        });

        let trending_keywords_mixed: Vec<String> = test_data_mixed
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|kw| kw.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        assert_eq!(trending_keywords_mixed.len(), 2);
        assert!(trending_keywords_mixed.contains(&"bitcoin".to_string()));
        assert!(trending_keywords_mixed.contains(&"ethereum".to_string()));
    }

    #[test]
    fn test_take_limit_behavior() {
        // Test the behavior of iterating with take() as used in the API functions
        let test_vec = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        // Take fewer items than available
        assert_eq!(test_vec.iter().take(3).count(), 3);

        // Take more items than available
        #[expect(clippy::iter_out_of_bounds)]
        let count = test_vec.iter().take(20).count();
        assert_eq!(count, 10); // Should only get what's available

        // Take zero items
        assert_eq!(test_vec.iter().take(0).count(), 0);
    }

    #[test]
    fn test_const_api_key_name() {
        // Test that the constant is correctly defined
        assert_eq!(LUNARCRUSH_API_KEY, "LUNARCRUSH_API_KEY");
    }
}
