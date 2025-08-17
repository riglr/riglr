//! LunarCrush social analytics integration for cryptocurrency sentiment analysis
//!
//! This module provides advanced social analytics capabilities by integrating with LunarCrush,
//! enabling AI agents to access comprehensive social sentiment data, influencer tracking,
//! and trending cryptocurrency analysis.

use crate::{client::WebClient, error::WebToolError};
use chrono::{DateTime, Utc};
use riglr_macros::tool;
use schemars::JsonSchema;

const LUNARCRUSH_API_KEY: &str = "LUNARCRUSH_API_KEY";
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// LunarCrush API configuration
#[derive(Debug, Clone)]
pub struct LunarCrushConfig {
    /// LunarCrush API key
    pub api_key: String,
    /// API base URL (default: https://api.lunarcrush.com/v2)
    pub base_url: String,
    /// Rate limit per minute (default: 60)
    pub rate_limit_per_minute: u32,
}

impl Default for LunarCrushConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var(LUNARCRUSH_API_KEY).unwrap_or_default(),
            base_url: "https://api.lunarcrush.com/v2".to_string(),
            rate_limit_per_minute: 60,
        }
    }
}

/// Social sentiment data for a cryptocurrency
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SentimentData {
    /// Cryptocurrency symbol (e.g., "BTC", "ETH")
    pub symbol: String,
    /// Overall social score (0-100)
    pub social_score: f64,
    /// Sentiment score (-1 to 1, where 1 is very positive)
    pub sentiment_score: f64,
    /// Social volume (number of mentions)
    pub social_volume: u64,
    /// Total mentions across all platforms
    pub mentions: u64,
    /// Number of posts from influencers
    pub influencer_posts: u64,
    /// Average social volume over the timeframe
    pub avg_social_volume: f64,
    /// Social volume change percentage
    pub social_volume_change: f64,
    /// Sentiment breakdown by platform
    pub platform_sentiment: HashMap<String, f64>,
    /// Trending keywords associated with the token
    pub trending_keywords: Vec<String>,
    /// Data timestamp
    pub timestamp: DateTime<Utc>,
}

/// Trending cryptocurrency with social metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrendingCrypto {
    /// Cryptocurrency symbol
    pub symbol: String,
    /// Full name of the cryptocurrency
    pub name: String,
    /// Current market rank
    pub market_rank: u32,
    /// Galaxy score (LunarCrush proprietary metric)
    pub galaxy_score: f64,
    /// AltRank (alternative ranking metric)
    pub alt_rank: u32,
    /// Social score
    pub social_score: f64,
    /// Price in USD
    pub price_usd: f64,
    /// 24h price change percentage
    pub price_change_24h: f64,
    /// Social volume
    pub social_volume: u64,
    /// Social volume change percentage
    pub social_volume_change: f64,
    /// Market cap in USD
    pub market_cap: u64,
    /// 24h trading volume
    pub volume_24h: u64,
}

/// Influencer mention data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InfluencerMention {
    /// Unique mention ID
    pub id: String,
    /// Influencer username
    pub influencer_username: String,
    /// Influencer display name
    pub influencer_name: String,
    /// Number of followers
    pub followers: u64,
    /// Post content/text
    pub text: String,
    /// Post timestamp
    pub timestamp: DateTime<Utc>,
    /// Platform (Twitter, Reddit, etc.)
    pub platform: String,
    /// Engagement metrics
    pub likes: u64,
    /// Number of shares/retweets
    pub shares: u64,
    /// Number of comments
    pub comments: u64,
    /// Post URL
    pub url: Option<String>,
    /// Sentiment score for this specific mention
    pub sentiment: f64,
}

/// Result containing multiple influencer mentions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InfluencerMentionsResult {
    /// Token symbol that was searched
    pub symbol: String,
    /// List of influencer mentions
    pub mentions: Vec<InfluencerMention>,
    /// Total number of mentions found
    pub total_mentions: u64,
    /// Timeframe of the search
    pub timeframe: String,
    /// Average sentiment across all mentions
    pub avg_sentiment: f64,
}

/// Creates a LunarCrush API client
async fn create_lunarcrush_client() -> Result<WebClient, WebToolError> {
    let config = LunarCrushConfig::default();

    if config.api_key.is_empty() {
        return Err(WebToolError::Config(
            "LUNARCRUSH_API_KEY environment variable not set".to_string(),
        ));
    }

    let client = WebClient::new()?.with_api_key("lunarcrush", config.api_key);

    Ok(client)
}

#[tool]
/// Get social sentiment data for a cryptocurrency from LunarCrush
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
pub async fn get_social_sentiment(
    symbol: String,
    timeframe: Option<String>,
) -> Result<SentimentData, WebToolError> {
    info!(
        "Fetching social sentiment for {} (timeframe: {})",
        symbol,
        timeframe.as_deref().unwrap_or("24h")
    );

    let client = create_lunarcrush_client().await?;
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
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {}", e)))?;

    // Extract asset data from the response
    let data = response_data
        .get("data")
        .ok_or_else(|| WebToolError::Parsing("Missing 'data' field".to_string()))?;

    let assets = data
        .as_array()
        .ok_or_else(|| WebToolError::Parsing("'data' is not an array".to_string()))?;

    let asset = assets
        .first()
        .ok_or_else(|| WebToolError::Parsing(format!("No data found for symbol {}", symbol)))?;

    // Parse the asset data
    let social_score = asset
        .get("galaxy_score")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let sentiment_score = asset
        .get("sentiment")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let social_volume = asset
        .get("social_volume")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let mentions = asset
        .get("total_mentions")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let influencer_posts = asset
        .get("influencer_posts")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let avg_social_volume = asset
        .get("avg_social_volume")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let social_volume_change = asset
        .get("social_volume_change_24h")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    // Extract platform sentiment breakdown
    let mut platform_sentiment = HashMap::new();
    if let Some(platforms) = asset.get("platform_sentiment") {
        if let Some(twitter_sentiment) = platforms.get("twitter").and_then(|v| v.as_f64()) {
            platform_sentiment.insert("twitter".to_string(), twitter_sentiment);
        }
        if let Some(reddit_sentiment) = platforms.get("reddit").and_then(|v| v.as_f64()) {
            platform_sentiment.insert("reddit".to_string(), reddit_sentiment);
        }
    }

    // Extract trending keywords
    let trending_keywords = asset
        .get("keywords")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|kw| kw.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    debug!(
        "Successfully fetched sentiment data for {} with {} mentions",
        symbol, mentions
    );

    Ok(SentimentData {
        symbol: symbol.to_uppercase(),
        social_score,
        sentiment_score,
        social_volume,
        mentions,
        influencer_posts,
        avg_social_volume,
        social_volume_change,
        platform_sentiment,
        trending_keywords,
        timestamp: Utc::now(),
    })
}

#[tool]
/// Get trending cryptocurrencies by social metrics from LunarCrush
///
/// This tool identifies the most trending cryptocurrencies based on social media activity,
/// sentiment, and LunarCrush's proprietary Galaxy Score.
///
/// # Arguments
/// * `limit` - Maximum number of trending cryptocurrencies to return (1-50). Defaults to 10
/// * `sort_by` - Sort metric: "galaxy_score", "social_volume", "alt_rank". Defaults to "galaxy_score"
///
/// # Returns
/// List of trending cryptocurrencies with social and market metrics
pub async fn get_trending_cryptos(
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

    let client = create_lunarcrush_client().await?;

    let mut params = HashMap::new();
    params.insert("limit".to_string(), limit.to_string());
    params.insert("sort".to_string(), sort_by);

    let config = LunarCrushConfig::default();
    let url = format!("{}/market", config.base_url);

    let response_text = client.get_with_params(&url, &params).await?;

    let response_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {}", e)))?;

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

        let market_rank = asset
            .get("market_cap_rank")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let galaxy_score = asset
            .get("galaxy_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let alt_rank = asset.get("alt_rank").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let social_score = asset
            .get("social_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let price_usd = asset.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let price_change_24h = asset
            .get("percent_change_24h")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let social_volume = asset
            .get("social_volume")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let social_volume_change = asset
            .get("social_volume_change_24h")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let market_cap = asset
            .get("market_cap")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let volume_24h = asset
            .get("volume_24h")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        trending_cryptos.push(TrendingCrypto {
            symbol,
            name,
            market_rank,
            galaxy_score,
            alt_rank,
            social_score,
            price_usd,
            price_change_24h,
            social_volume,
            social_volume_change,
            market_cap,
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
/// Get influencer mentions for a specific token from LunarCrush
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
pub async fn get_influencer_mentions(
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

    let client = create_lunarcrush_client().await?;

    let mut params = HashMap::new();
    params.insert("symbol".to_string(), token_symbol.to_uppercase());
    params.insert("limit".to_string(), limit.to_string());
    params.insert("interval".to_string(), timeframe.clone());
    params.insert("sources".to_string(), "influencers".to_string());

    let config = LunarCrushConfig::default();
    let url = format!("{}/feeds", config.base_url);

    let response_text = client.get_with_params(&url, &params).await?;

    let response_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Invalid JSON response: {}", e)))?;

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
            .and_then(|v| v.as_u64())
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
                DateTime::from_timestamp(0, 0)
                    .map(|dt| dt.into())
                    .unwrap_or_else(|| Utc::now().into())
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
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let shares = post
            .get("interactions")
            .and_then(|i| i.get("retweets"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let comments = post
            .get("interactions")
            .and_then(|i| i.get("replies"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let url = post
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let sentiment = post
            .get("sentiment")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        total_sentiment += sentiment;

        mentions.push(InfluencerMention {
            id,
            influencer_username,
            influencer_name,
            followers,
            text,
            timestamp,
            platform,
            likes,
            shares,
            comments,
            url,
            sentiment,
        });
    }

    let avg_sentiment = if mentions.is_empty() {
        0.0
    } else {
        total_sentiment / mentions.len() as f64
    };

    debug!(
        "Successfully fetched {} influencer mentions for {} with avg sentiment {:.2}",
        mentions.len(),
        token_symbol,
        avg_sentiment
    );

    let total_mentions = mentions.len() as u64;

    Ok(InfluencerMentionsResult {
        symbol: token_symbol.to_uppercase(),
        mentions,
        total_mentions,
        timeframe,
        avg_sentiment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lunarcrush_config_default() {
        let config = LunarCrushConfig::default();
        assert_eq!(config.base_url, "https://api.lunarcrush.com/v2");
        assert_eq!(config.rate_limit_per_minute, 60);
    }

    #[test]
    fn test_sentiment_data_serialization() {
        let sentiment = SentimentData {
            symbol: "BTC".to_string(),
            social_score: 85.5,
            sentiment_score: 0.7,
            social_volume: 10000,
            mentions: 15000,
            influencer_posts: 250,
            avg_social_volume: 9500.0,
            social_volume_change: 15.2,
            platform_sentiment: HashMap::new(),
            trending_keywords: vec!["bitcoin".to_string(), "crypto".to_string()],
            timestamp: Utc::now(),
        };

        let serialized = serde_json::to_string(&sentiment).unwrap();
        assert!(serialized.contains("BTC"));
        assert!(serialized.contains("85.5"));
    }

    #[test]
    fn test_trending_crypto_serialization() {
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
            market_cap: 300000000000,
            volume_24h: 15000000000,
        };

        let serialized = serde_json::to_string(&trending).unwrap();
        assert!(serialized.contains("ETH"));
        assert!(serialized.contains("Ethereum"));
        assert!(serialized.contains("90"));
    }
}
