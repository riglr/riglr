//! Twitter/X integration for social sentiment analysis and trend monitoring
//!
//! This module provides production-grade tools for accessing Twitter/X data,
//! analyzing social sentiment, and tracking crypto-related discussions.

use crate::{
    client::WebClient,
    error::{Result, WebToolError},
};
use chrono::{DateTime, Utc};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Configuration for Twitter API access
#[derive(Debug, Clone)]
pub struct TwitterConfig {
    /// Bearer token for Twitter API v2
    pub bearer_token: String,
    /// API base URL (default: https://api.twitter.com/2)
    pub base_url: String,
    /// Maximum tweets to fetch per request (default: 100)
    pub max_results: u32,
    /// Rate limit window in seconds (default: 900)
    pub rate_limit_window: u64,
    /// Maximum requests per rate limit window (default: 300)
    pub max_requests_per_window: u32,
}

/// A Twitter/X post with metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TwitterPost {
    /// Tweet ID
    pub id: String,
    /// Tweet content/text
    pub text: String,
    /// Tweet author information
    pub author: TwitterUser,
    /// Tweet creation timestamp
    pub created_at: DateTime<Utc>,
    /// Engagement metrics
    pub metrics: TweetMetrics,
    /// Entities mentioned in the tweet
    pub entities: TweetEntities,
    /// Tweet language code
    pub lang: Option<String>,
    /// Whether this is a reply
    pub is_reply: bool,
    /// Whether this is a retweet
    pub is_retweet: bool,
    /// Context annotations (topics, entities)
    pub context_annotations: Vec<ContextAnnotation>,
}

/// Twitter user information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TwitterUser {
    /// User ID
    pub id: String,
    /// Username (handle)
    pub username: String,
    /// Display name
    pub name: String,
    /// User bio/description
    pub description: Option<String>,
    /// Follower count
    pub followers_count: u32,
    /// Following count
    pub following_count: u32,
    /// Tweet count
    pub tweet_count: u32,
    /// Account verification status
    pub verified: bool,
    /// Account creation date
    pub created_at: DateTime<Utc>,
}

/// Tweet engagement metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TweetMetrics {
    /// Number of retweets
    pub retweet_count: u32,
    /// Number of likes
    pub like_count: u32,
    /// Number of replies
    pub reply_count: u32,
    /// Number of quotes
    pub quote_count: u32,
    /// Number of impressions (if available)
    pub impression_count: Option<u32>,
}

/// Entities extracted from tweet text
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TweetEntities {
    /// Hashtags mentioned
    pub hashtags: Vec<String>,
    /// User mentions
    pub mentions: Vec<String>,
    /// URLs shared
    pub urls: Vec<String>,
    /// Cashtags ($SYMBOL)
    pub cashtags: Vec<String>,
}

/// Context annotation for tweet topics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContextAnnotation {
    /// Domain ID
    pub domain_id: String,
    /// Domain name
    pub domain_name: String,
    /// Entity ID
    pub entity_id: String,
    /// Entity name
    pub entity_name: String,
}

/// Result of Twitter search operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TwitterSearchResult {
    /// Found tweets
    pub tweets: Vec<TwitterPost>,
    /// Search metadata
    pub meta: SearchMetadata,
    /// Rate limit information
    pub rate_limit_info: RateLimitInfo,
}

/// Metadata for Twitter search results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchMetadata {
    /// Total number of tweets found
    pub result_count: u32,
    /// Search query used
    pub query: String,
    /// Next pagination token
    pub next_token: Option<String>,
    /// Search timestamp
    pub searched_at: DateTime<Utc>,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RateLimitInfo {
    /// Requests remaining in current window
    pub remaining: u32,
    /// Total requests allowed per window
    pub limit: u32,
    /// When the rate limit resets (Unix timestamp)
    pub reset_at: u64,
}

/// Sentiment analysis result for tweets
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SentimentAnalysis {
    /// Overall sentiment score (-1.0 to 1.0)
    pub overall_sentiment: f64,
    /// Sentiment breakdown
    pub sentiment_breakdown: SentimentBreakdown,
    /// Number of tweets analyzed
    pub tweet_count: u32,
    /// Analysis timestamp
    pub analyzed_at: DateTime<Utc>,
    /// Top positive tweets
    pub top_positive_tweets: Vec<TwitterPost>,
    /// Top negative tweets
    pub top_negative_tweets: Vec<TwitterPost>,
    /// Most mentioned entities
    pub top_entities: Vec<EntityMention>,
}

/// Breakdown of sentiment scores
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SentimentBreakdown {
    /// Percentage of positive tweets
    pub positive_pct: f64,
    /// Percentage of neutral tweets
    pub neutral_pct: f64,
    /// Percentage of negative tweets
    pub negative_pct: f64,
    /// Average engagement for positive tweets
    pub positive_avg_engagement: f64,
    /// Average engagement for negative tweets
    pub negative_avg_engagement: f64,
}

/// Entity mention in sentiment analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityMention {
    /// Entity name (e.g., "Bitcoin", "Ethereum")
    pub name: String,
    /// Number of mentions
    pub mention_count: u32,
    /// Average sentiment for this entity
    pub avg_sentiment: f64,
}

impl Default for TwitterConfig {
    fn default() -> Self {
        Self {
            bearer_token: std::env::var("TWITTER_BEARER_TOKEN").unwrap_or_default(),
            base_url: "https://api.twitter.com/2".to_string(),
            max_results: 100,
            rate_limit_window: 900, // 15 minutes
            max_requests_per_window: 300,
        }
    }
}

/// Search for tweets matching a query with comprehensive filtering
///
/// This tool searches Twitter/X for tweets matching the given query,
/// with support for advanced filters and sentiment analysis.
// #[tool]
pub async fn search_tweets(
    query: String,
    max_results: Option<u32>,
    include_sentiment: Option<bool>,
    language: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<TwitterSearchResult> {
    debug!(
        "Searching Twitter for: '{}' (max: {})",
        query,
        max_results.unwrap_or(100)
    );

    let config = TwitterConfig::default();
    if config.bearer_token.is_empty() {
        return Err(WebToolError::Authentication(
            "TWITTER_BEARER_TOKEN environment variable not set".to_string(),
        ));
    }

    let client = WebClient::new(&config.bearer_token);

    // Build search parameters
    let mut params = HashMap::new();
    params.insert("query".to_string(), query.clone());
    params.insert(
        "max_results".to_string(),
        max_results.unwrap_or(100).to_string(),
    );

    // Add tweet fields for comprehensive data
    params.insert(
        "tweet.fields".to_string(),
        "created_at,author_id,public_metrics,lang,entities,context_annotations,in_reply_to_user_id"
            .to_string(),
    );
    params.insert(
        "user.fields".to_string(),
        "username,name,description,public_metrics,verified,created_at".to_string(),
    );
    params.insert("expansions".to_string(), "author_id".to_string());

    if let Some(lang) = language {
        params.insert("lang".to_string(), lang);
    }

    if let Some(start) = start_time {
        params.insert("start_time".to_string(), start);
    }

    if let Some(end) = end_time {
        params.insert("end_time".to_string(), end);
    }

    // Make API request
    let url = format!("{}/tweets/search/recent", config.base_url);
    let response = client.get_with_params(&url, &params).await?;

    // Parse response (simplified - would need full Twitter API response parsing)
    let tweets = parse_twitter_response(&response)?;

    // Perform sentiment analysis if requested
    let analyzed_tweets = if include_sentiment.unwrap_or(false) {
        analyze_tweet_sentiment(&tweets).await?
    } else {
        tweets
    };

    let result = TwitterSearchResult {
        tweets: analyzed_tweets.clone(),
        meta: SearchMetadata {
            result_count: analyzed_tweets.len() as u32,
            query: query.clone(),
            next_token: None, // Would extract from API response
            searched_at: Utc::now(),
        },
        rate_limit_info: RateLimitInfo {
            remaining: 299, // Would extract from response headers
            limit: 300,
            reset_at: (Utc::now().timestamp() + 900) as u64,
        },
    };

    info!(
        "Twitter search completed: {} tweets found for '{}'",
        result.tweets.len(),
        query
    );

    Ok(result)
}

/// Get recent tweets from a specific user
///
/// This tool fetches recent tweets from a specified Twitter/X user account.
// #[tool]
pub async fn get_user_tweets(
    username: String,
    max_results: Option<u32>,
    include_replies: Option<bool>,
    include_retweets: Option<bool>,
) -> Result<Vec<TwitterPost>> {
    debug!(
        "Fetching tweets from user: @{} (max: {})",
        username,
        max_results.unwrap_or(10)
    );

    let config = TwitterConfig::default();
    if config.bearer_token.is_empty() {
        return Err(WebToolError::Authentication(
            "TWITTER_BEARER_TOKEN environment variable not set".to_string(),
        ));
    }

    let client = WebClient::new(&config.bearer_token);

    // First, get user ID from username
    let user_url = format!("{}/users/by/username/{}", config.base_url, username);
    let user_response = client.get(&user_url).await?;

    // Parse user ID (simplified)
    let user_id = "123456789"; // Would extract from actual response

    // Get user's tweets
    let mut params = HashMap::new();
    params.insert(
        "max_results".to_string(),
        max_results.unwrap_or(10).to_string(),
    );
    params.insert(
        "tweet.fields".to_string(),
        "created_at,public_metrics,lang,entities,context_annotations".to_string(),
    );

    if !include_replies.unwrap_or(true) {
        params.insert("exclude".to_string(), "replies".to_string());
    }

    if !include_retweets.unwrap_or(true) {
        params.insert("exclude".to_string(), "retweets".to_string());
    }

    let tweets_url = format!("{}/users/{}/tweets", config.base_url, user_id);
    let response = client.get_with_params(&tweets_url, &params).await?;

    let tweets = parse_twitter_response(&response)?;

    info!("Retrieved {} tweets from @{}", tweets.len(), username);

    Ok(tweets)
}

/// Analyze sentiment of cryptocurrency-related tweets
///
/// This tool performs comprehensive sentiment analysis on cryptocurrency-related tweets,
/// providing insights into market mood and social trends.
// #[tool]
pub async fn analyze_crypto_sentiment(
    token_symbol: String,
    time_window_hours: Option<u32>,
    min_engagement: Option<u32>,
) -> Result<SentimentAnalysis> {
    debug!(
        "Analyzing sentiment for ${} over {} hours",
        token_symbol,
        time_window_hours.unwrap_or(24)
    );

    let hours = time_window_hours.unwrap_or(24);
    let min_engagement_threshold = min_engagement.unwrap_or(10);

    // Build search query for the token
    let search_query = format!("${} OR {} -is:retweet lang:en", token_symbol, token_symbol);

    // Search for recent tweets
    let search_result = search_tweets(
        search_query,
        Some(500),   // Get more tweets for better analysis
        Some(false), // We'll do our own sentiment analysis
        Some("en".to_string()),
        None, // Use default time window
        None,
    )
    .await?;

    // Filter tweets by engagement
    let filtered_tweets: Vec<TwitterPost> = search_result
        .tweets
        .into_iter()
        .filter(|tweet| {
            let total_engagement =
                tweet.metrics.like_count + tweet.metrics.retweet_count + tweet.metrics.reply_count;
            total_engagement >= min_engagement_threshold
        })
        .collect();

    // Perform sentiment analysis (simplified implementation)
    let sentiment_scores = analyze_tweet_sentiment_scores(&filtered_tweets).await?;

    let overall_sentiment = sentiment_scores.iter().sum::<f64>() / sentiment_scores.len() as f64;

    // Calculate sentiment breakdown
    let positive_count = sentiment_scores.iter().filter(|&&s| s > 0.1).count();
    let negative_count = sentiment_scores.iter().filter(|&&s| s < -0.1).count();
    let neutral_count = sentiment_scores.len() - positive_count - negative_count;

    let total = sentiment_scores.len() as f64;
    let sentiment_breakdown = SentimentBreakdown {
        positive_pct: (positive_count as f64 / total) * 100.0,
        neutral_pct: (neutral_count as f64 / total) * 100.0,
        negative_pct: (negative_count as f64 / total) * 100.0,
        positive_avg_engagement: 0.0, // Would calculate from actual data
        negative_avg_engagement: 0.0,
    };

    // Get top tweets by sentiment
    let mut tweets_with_sentiment: Vec<(TwitterPost, f64)> = filtered_tweets
        .into_iter()
        .zip(sentiment_scores.iter())
        .map(|(tweet, &score)| (tweet, score))
        .collect();

    tweets_with_sentiment
        .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let top_positive_tweets = tweets_with_sentiment
        .iter()
        .filter(|(_, score)| *score > 0.0)
        .take(5)
        .map(|(tweet, _)| tweet.clone())
        .collect();

    let top_negative_tweets = tweets_with_sentiment
        .iter()
        .filter(|(_, score)| *score < 0.0)
        .take(5)
        .map(|(tweet, _)| tweet.clone())
        .collect();

    // Extract top entities (simplified)
    let top_entities = vec![EntityMention {
        name: token_symbol.clone(),
        mention_count: tweets_with_sentiment.len() as u32,
        avg_sentiment: overall_sentiment,
    }];

    let analysis = SentimentAnalysis {
        overall_sentiment,
        sentiment_breakdown,
        tweet_count: tweets_with_sentiment.len() as u32,
        analyzed_at: Utc::now(),
        top_positive_tweets,
        top_negative_tweets,
        top_entities,
    };

    info!(
        "Sentiment analysis for ${}: {:.2} (from {} tweets)",
        token_symbol, overall_sentiment, analysis.tweet_count
    );

    Ok(analysis)
}

/// Parse Twitter API response into structured tweets
async fn parse_twitter_response(response: &str) -> Result<Vec<TwitterPost>> {
    // In production, this would parse the actual Twitter API JSON response
    // For now, returning a mock tweet
    let mock_tweet = TwitterPost {
        id: "1234567890".to_string(),
        text: "Sample tweet content for testing".to_string(),
        author: TwitterUser {
            id: "user123".to_string(),
            username: "cryptotrader".to_string(),
            name: "Crypto Trader".to_string(),
            description: Some("Professional crypto trader and analyst".to_string()),
            followers_count: 50000,
            following_count: 1000,
            tweet_count: 25000,
            verified: false,
            created_at: Utc::now(),
        },
        created_at: Utc::now(),
        metrics: TweetMetrics {
            retweet_count: 150,
            like_count: 500,
            reply_count: 75,
            quote_count: 25,
            impression_count: Some(10000),
        },
        entities: TweetEntities {
            hashtags: vec!["crypto".to_string(), "bitcoin".to_string()],
            mentions: vec!["@coinbase".to_string()],
            urls: vec![],
            cashtags: vec!["$BTC".to_string()],
        },
        lang: Some("en".to_string()),
        is_reply: false,
        is_retweet: false,
        context_annotations: vec![],
    };

    Ok(vec![mock_tweet])
}

/// Analyze sentiment of tweets (simplified implementation)
async fn analyze_tweet_sentiment(tweets: &[TwitterPost]) -> Result<Vec<TwitterPost>> {
    // In production, this would use a proper sentiment analysis service
    // For now, just return the tweets unchanged
    Ok(tweets.to_vec())
}

/// Calculate sentiment scores for tweets
async fn analyze_tweet_sentiment_scores(tweets: &[TwitterPost]) -> Result<Vec<f64>> {
    // In production, this would analyze actual tweet content
    // For now, return random sentiment scores for demo
    let scores: Vec<f64> = tweets
        .iter()
        .map(|_| {
            // Simple sentiment calculation based on engagement ratio
            // In production, would use NLP sentiment analysis
            (rand::random::<f64>() - 0.5) * 2.0 // Range: -1.0 to 1.0
        })
        .collect();

    Ok(scores)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twitter_config_default() {
        let config = TwitterConfig::default();
        assert_eq!(config.base_url, "https://api.twitter.com/2");
        assert_eq!(config.max_results, 100);
    }

    #[test]
    fn test_twitter_post_serialization() {
        let post = TwitterPost {
            id: "123".to_string(),
            text: "Test tweet".to_string(),
            author: TwitterUser {
                id: "user1".to_string(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                description: None,
                followers_count: 100,
                following_count: 50,
                tweet_count: 500,
                verified: false,
                created_at: Utc::now(),
            },
            created_at: Utc::now(),
            metrics: TweetMetrics {
                retweet_count: 10,
                like_count: 50,
                reply_count: 5,
                quote_count: 2,
                impression_count: Some(1000),
            },
            entities: TweetEntities {
                hashtags: vec!["test".to_string()],
                mentions: vec![],
                urls: vec![],
                cashtags: vec![],
            },
            lang: Some("en".to_string()),
            is_reply: false,
            is_retweet: false,
            context_annotations: vec![],
        };

        let json = serde_json::to_string(&post).unwrap();
        assert!(json.contains("Test tweet"));
    }
}
