//! TweetScout integration for Twitter/X account analysis and credibility scoring
//!
//! This module provides tools for accessing TweetScout API to analyze Twitter/X accounts,
//! calculate credibility scores, and analyze social networks for crypto influencer detection.

use crate::{client::WebClient, error::WebToolError};
use riglr_core::provider::ApplicationContext;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tracing::{debug, info};

/// Environment variable name for TweetScout API key
const TWEETSCOUT_API_KEY_ENV: &str = "TWEETSCOUT_API_KEY";

/// Configuration for TweetScout API access
#[derive(Debug, Clone)]
pub struct TweetScoutConfig {
    /// API base URL (default: https://api.tweetscout.io/api)
    pub base_url: String,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Rate limit requests per minute (default: 60)
    pub rate_limit_per_minute: u32,
    /// Timeout for API requests in seconds (default: 30)
    pub request_timeout: u64,
}

impl Default for TweetScoutConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.tweetscout.io/api".to_string(),
            api_key: env::var(TWEETSCOUT_API_KEY_ENV).ok(),
            rate_limit_per_minute: 60,
            request_timeout: 30,
        }
    }
}

/// Helper function to get TweetScout API key from ApplicationContext
fn get_api_key_from_context(context: &ApplicationContext) -> Result<String, WebToolError> {
    context.config.providers.tweetscout_api_key
        .clone()
        .ok_or_else(|| WebToolError::Config(
            "TweetScout API key not configured. Set TWEETSCOUT_API_KEY in your environment.".to_string()
        ))
}

/// Account information response from TweetScout
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountInfo {
    /// User ID
    pub id: Option<String>,
    /// Display name
    pub name: Option<String>,
    /// Username/handle
    pub screen_name: Option<String>,
    /// Profile description/bio
    pub description: Option<String>,
    /// Avatar image URL
    pub avatar: Option<String>,
    /// Banner image URL
    pub banner: Option<String>,
    /// Number of followers
    pub followers_count: Option<i64>,
    /// Number of accounts following (friends)
    pub friends_count: Option<i64>,
    /// Number of tweets/posts
    pub statuses_count: Option<i64>,
    /// Account registration date
    pub register_date: Option<String>,
    /// Verification status
    pub verified: Option<bool>,
}

/// Score response from TweetScout
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScoreResponse {
    /// Credibility score (0-100)
    pub score: f64,
}

/// Account information for followers/friends lists
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Account {
    /// User ID
    pub id: Option<String>,
    /// Display name
    pub name: Option<String>,
    /// Username/handle (Note: API returns screeName, handling typo)
    #[serde(rename = "screeName")]
    pub screen_name: Option<String>,
    /// Profile description
    pub description: Option<String>,
    /// Avatar URL
    pub avatar: Option<String>,
    /// Banner URL
    pub banner: Option<String>,
    /// Followers count (Note: API uses camelCase)
    #[serde(rename = "followersCount")]
    pub followers_count: Option<i64>,
    /// Friends/following count
    #[serde(rename = "friendsCount")]
    pub friends_count: Option<i64>,
    /// Number of posts
    pub statuses_count: Option<i64>,
    /// Registration date
    #[serde(rename = "registerDate")]
    pub register_date: Option<String>,
    /// Verification status
    pub verified: Option<bool>,
    /// Account score
    pub score: Option<f64>,
}

/// Error response from TweetScout API
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorResponse {
    /// Error message
    pub message: String,
}

/// Comprehensive account analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountAnalysis {
    /// Username analyzed
    pub username: String,
    /// Basic account information
    pub info: AccountInfo,
    /// Credibility score (0-100)
    pub credibility_score: f64,
    /// Score classification
    pub score_level: ScoreLevel,
    /// Account age in days
    pub account_age_days: Option<i64>,
    /// Average tweets per day
    pub avg_tweets_per_day: Option<f64>,
    /// Follower to following ratio
    pub follower_ratio: Option<f64>,
    /// Engagement metrics
    pub engagement: EngagementMetrics,
    /// Risk indicators
    pub risk_indicators: Vec<String>,
    /// Summary assessment
    pub assessment: String,
}

/// Credibility score level classification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ScoreLevel {
    /// Excellent credibility (80-100)
    #[serde(rename = "excellent")]
    Excellent,
    /// Good credibility (60-80)
    #[serde(rename = "good")]
    Good,
    /// Fair credibility (40-60)
    #[serde(rename = "fair")]
    Fair,
    /// Poor credibility (20-40)
    #[serde(rename = "poor")]
    Poor,
    /// Very poor credibility (0-20)
    #[serde(rename = "very_poor")]
    VeryPoor,
}

/// Engagement metrics for an account
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EngagementMetrics {
    /// Total followers
    pub followers: i64,
    /// Total following
    pub following: i64,
    /// Total posts
    pub posts: i64,
    /// Engagement rate estimate
    pub engagement_rate: f64,
    /// Whether account is likely a bot
    pub likely_bot: bool,
    /// Whether account is likely spam
    pub likely_spam: bool,
}

/// Social network analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SocialNetworkAnalysis {
    /// Username analyzed
    pub username: String,
    /// Top followers with scores
    pub top_followers: Vec<ScoredAccount>,
    /// Top friends (following) with scores
    pub top_friends: Vec<ScoredAccount>,
    /// Average follower score
    pub avg_follower_score: f64,
    /// Average friend score
    pub avg_friend_score: f64,
    /// Quality of network
    pub network_quality: NetworkQuality,
    /// Key influencers in network
    pub key_influencers: Vec<String>,
    /// Network assessment
    pub assessment: String,
}

/// Account with score for network analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScoredAccount {
    /// Username
    pub username: String,
    /// Display name
    pub name: String,
    /// Follower count
    pub followers: i64,
    /// Score
    pub score: f64,
    /// Whether verified
    pub verified: bool,
    /// Influence level
    pub influence_level: String,
}

/// Network quality assessment
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum NetworkQuality {
    /// High quality network
    #[serde(rename = "high")]
    High,
    /// Medium quality network
    #[serde(rename = "medium")]
    Medium,
    /// Low quality network
    #[serde(rename = "low")]
    Low,
    /// Suspicious network
    #[serde(rename = "suspicious")]
    Suspicious,
}

/// Get basic information about a Twitter/X account.
#[tool]
pub async fn get_account_info(
    context: &ApplicationContext,
    username: String,
) -> crate::error::Result<AccountInfo> {
    debug!("Fetching account info for: {}", username);

    let config = TweetScoutConfig::default();
    let client = WebClient::default();

    let api_key = get_api_key_from_context(context)?;

    let url = format!("{}/info/{}", config.base_url, username);

    let mut headers = HashMap::new();
    headers.insert("ApiKey".to_string(), api_key);

    info!("Requesting account info from TweetScout for: {}", username);

    let response_text = client
        .get_with_headers(&url, headers)
        .await
        .map_err(|e| WebToolError::Network(format!("Failed to fetch account info: {}", e)))?;

    let info: AccountInfo = serde_json::from_str(&response_text).map_err(|e| {
        WebToolError::Parsing(format!("Failed to parse TweetScout response: {}", e))
    })?;

    info!(
        "Successfully fetched info for @{} - Followers: {:?}, Verified: {:?}",
        username, info.followers_count, info.verified
    );

    Ok(info)
}

/// Get the credibility score for a Twitter/X account.
/// Returns a score from 0-100 indicating account trustworthiness.
#[tool]
pub async fn get_account_score(
    context: &ApplicationContext,
    username: String,
) -> crate::error::Result<ScoreResponse> {
    debug!("Fetching credibility score for: {}", username);

    let config = TweetScoutConfig::default();
    let client = WebClient::default();

    let api_key = get_api_key_from_context(context)?;

    let url = format!("{}/score/{}", config.base_url, username);

    let mut headers = HashMap::new();
    headers.insert("ApiKey".to_string(), api_key);

    info!(
        "Requesting credibility score from TweetScout for: {}",
        username
    );

    let response_text = client
        .get_with_headers(&url, headers)
        .await
        .map_err(|e| WebToolError::Network(format!("Failed to fetch score: {}", e)))?;

    let score: ScoreResponse = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Failed to parse score response: {}", e)))?;

    info!(
        "Successfully fetched score for @{}: {:.1}/100",
        username, score.score
    );

    Ok(score)
}

/// Get the top 20 followers of a Twitter/X account with their scores.
#[tool]
pub async fn get_top_followers(
    context: &ApplicationContext,
    username: String,
) -> crate::error::Result<Vec<Account>> {
    debug!("Fetching top followers for: {}", username);

    let config = TweetScoutConfig::default();
    let client = WebClient::default();

    let api_key = get_api_key_from_context(context)?;

    let url = format!("{}/top-followers/{}", config.base_url, username);

    let mut headers = HashMap::new();
    headers.insert("ApiKey".to_string(), api_key);

    info!("Requesting top followers from TweetScout for: {}", username);

    let response_text = client
        .get_with_headers(&url, headers)
        .await
        .map_err(|e| WebToolError::Network(format!("Failed to fetch followers: {}", e)))?;

    let followers: Vec<Account> = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Failed to parse followers response: {}", e)))?;

    info!(
        "Successfully fetched {} top followers for @{}",
        followers.len(),
        username
    );

    Ok(followers)
}

/// Get the top 20 friends (accounts being followed) of a Twitter/X account with their scores.
#[tool]
pub async fn get_top_friends(
    context: &ApplicationContext,
    username: String,
) -> crate::error::Result<Vec<Account>> {
    debug!("Fetching top friends for: {}", username);

    let config = TweetScoutConfig::default();
    let client = WebClient::default();

    let api_key = get_api_key_from_context(context)?;

    let url = format!("{}/top-friends/{}", config.base_url, username);

    let mut headers = HashMap::new();
    headers.insert("ApiKey".to_string(), api_key);

    info!("Requesting top friends from TweetScout for: {}", username);

    let response_text = client
        .get_with_headers(&url, headers)
        .await
        .map_err(|e| WebToolError::Network(format!("Failed to fetch friends: {}", e)))?;

    let friends: Vec<Account> = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Failed to parse friends response: {}", e)))?;

    info!(
        "Successfully fetched {} top friends for @{}",
        friends.len(),
        username
    );

    Ok(friends)
}

/// Perform comprehensive analysis of a Twitter/X account including credibility scoring.
/// Combines account info and score into a detailed assessment.
#[tool]
pub async fn analyze_account(
    context: &ApplicationContext,
    username: String,
) -> crate::error::Result<AccountAnalysis> {
    debug!("Performing comprehensive analysis for: {}", username);

    // Fetch account info and score in parallel would be better, but let's do sequentially for simplicity
    let info = get_account_info(context, username.clone()).await?;
    let score_resp = get_account_score(context, username.clone()).await?;

    let account_age_days = calculate_account_age(&info);
    let avg_tweets_per_day = calculate_avg_tweets_per_day(&info, account_age_days);
    let follower_ratio = calculate_follower_ratio(&info);
    let score_level = determine_score_level(score_resp.score);
    let engagement = build_engagement_metrics(&info, score_resp.score);
    let risk_indicators =
        build_risk_indicators(&info, &engagement, follower_ratio, score_resp.score);
    let assessment = build_assessment(&username, score_resp.score, &score_level);

    Ok(AccountAnalysis {
        username,
        info,
        credibility_score: score_resp.score,
        score_level,
        account_age_days,
        avg_tweets_per_day,
        follower_ratio,
        engagement,
        risk_indicators,
        assessment,
    })
}

/// Calculate account age in days from registration date
fn calculate_account_age(info: &AccountInfo) -> Option<i64> {
    info.register_date.as_ref().and({
        // Parse date and calculate days (simplified, would need proper date parsing)
        // For now, return None as proper date parsing would require chrono
        None
    })
}

/// Calculate average tweets per day
fn calculate_avg_tweets_per_day(info: &AccountInfo, account_age_days: Option<i64>) -> Option<f64> {
    if let (Some(_tweets), Some(_age)) = (info.statuses_count, account_age_days) {
        None // Would calculate if we had proper age
    } else {
        None
    }
}

/// Calculate follower to following ratio
fn calculate_follower_ratio(info: &AccountInfo) -> Option<f64> {
    if let (Some(followers), Some(following)) = (info.followers_count, info.friends_count) {
        if following > 0 {
            Some(followers as f64 / following as f64)
        } else {
            None
        }
    } else {
        None
    }
}

/// Determine score level classification from numeric score
fn determine_score_level(score: f64) -> ScoreLevel {
    match score as i32 {
        80..=100 => ScoreLevel::Excellent,
        60..=79 => ScoreLevel::Good,
        40..=59 => ScoreLevel::Fair,
        20..=39 => ScoreLevel::Poor,
        _ => ScoreLevel::VeryPoor,
    }
}

/// Build engagement metrics with bot/spam detection
fn build_engagement_metrics(info: &AccountInfo, score: f64) -> EngagementMetrics {
    let followers = info.followers_count.unwrap_or(0);
    let following = info.friends_count.unwrap_or(0);
    let posts = info.statuses_count.unwrap_or(0);

    // Simple bot detection heuristics
    let likely_bot = score < 30.0
        || (following > followers * 10 && followers < 100)
        || (posts > 100000 && followers < 1000);

    let likely_spam = score < 20.0 || (following > 5000 && followers < 100);

    let engagement_rate = if posts > 0 {
        ((followers + following) as f64 / posts as f64).min(100.0)
    } else {
        0.0
    };

    EngagementMetrics {
        followers,
        following,
        posts,
        engagement_rate,
        likely_bot,
        likely_spam,
    }
}

/// Build list of risk indicators for the account
fn build_risk_indicators(
    info: &AccountInfo,
    engagement: &EngagementMetrics,
    follower_ratio: Option<f64>,
    score: f64,
) -> Vec<String> {
    let mut risk_indicators = Vec::new();

    if score < 40.0 {
        risk_indicators.push("Low credibility score".to_string());
    }

    if engagement.likely_bot {
        risk_indicators.push("Likely bot account".to_string());
    }

    if engagement.likely_spam {
        risk_indicators.push("Likely spam account".to_string());
    }

    if info.verified != Some(true) && engagement.followers > 10000 {
        risk_indicators.push("Large unverified account".to_string());
    }

    if let Some(ratio) = follower_ratio {
        if ratio < 0.1 && engagement.followers < 1000 {
            risk_indicators.push("Very low follower ratio".to_string());
        }
    }

    if engagement.posts == 0 {
        risk_indicators.push("No posts/tweets".to_string());
    }

    risk_indicators
}

/// Build textual assessment based on score level
fn build_assessment(username: &str, score: f64, score_level: &ScoreLevel) -> String {
    match score_level {
        ScoreLevel::Excellent => format!(
            "@{} has excellent credibility ({:.1}/100). This appears to be a highly trustworthy account.",
            username, score
        ),
        ScoreLevel::Good => format!(
            "@{} has good credibility ({:.1}/100). This account appears legitimate and trustworthy.",
            username, score
        ),
        ScoreLevel::Fair => format!(
            "@{} has fair credibility ({:.1}/100). Exercise some caution when engaging with this account.",
            username, score
        ),
        ScoreLevel::Poor => format!(
            "@{} has poor credibility ({:.1}/100). Be cautious - this account shows concerning patterns.",
            username, score
        ),
        ScoreLevel::VeryPoor => format!(
            "@{} has very poor credibility ({:.1}/100). High risk - avoid engagement with this account.",
            username, score
        ),
    }
}

/// Analyze the social network of a Twitter/X account including followers and friends.
/// Provides insights into the quality and influence of an account's network.
#[tool]
pub async fn analyze_social_network(
    context: &ApplicationContext,
    username: String,
) -> crate::error::Result<SocialNetworkAnalysis> {
    debug!("Analyzing social network for: {}", username);

    // Fetch followers and friends
    let followers = get_top_followers(context, username.clone()).await?;
    let friends = get_top_friends(context, username.clone()).await?;

    // Convert to scored accounts
    let top_followers: Vec<ScoredAccount> = followers
        .iter()
        .map(|acc| ScoredAccount {
            username: acc.screen_name.clone().unwrap_or_default(),
            name: acc.name.clone().unwrap_or_default(),
            followers: acc.followers_count.unwrap_or(0),
            score: acc.score.unwrap_or(0.0),
            verified: acc.verified.unwrap_or(false),
            influence_level: classify_influence(acc.followers_count.unwrap_or(0)),
        })
        .collect();

    let top_friends: Vec<ScoredAccount> = friends
        .iter()
        .map(|acc| ScoredAccount {
            username: acc.screen_name.clone().unwrap_or_default(),
            name: acc.name.clone().unwrap_or_default(),
            followers: acc.followers_count.unwrap_or(0),
            score: acc.score.unwrap_or(0.0),
            verified: acc.verified.unwrap_or(false),
            influence_level: classify_influence(acc.followers_count.unwrap_or(0)),
        })
        .collect();

    // Calculate average scores
    let avg_follower_score = if !top_followers.is_empty() {
        top_followers.iter().map(|a| a.score).sum::<f64>() / top_followers.len() as f64
    } else {
        0.0
    };

    let avg_friend_score = if !top_friends.is_empty() {
        top_friends.iter().map(|a| a.score).sum::<f64>() / top_friends.len() as f64
    } else {
        0.0
    };

    // Identify key influencers (high follower count + good score)
    let mut key_influencers: Vec<String> = top_followers
        .iter()
        .chain(top_friends.iter())
        .filter(|acc| acc.followers > 10000 && acc.score > 50.0)
        .map(|acc| format!("@{}", acc.username))
        .collect();
    key_influencers.dedup();
    key_influencers.truncate(5); // Keep top 5

    // Determine network quality
    let avg_network_score = (avg_follower_score + avg_friend_score) / 2.0;
    let network_quality = if avg_network_score > 70.0 {
        NetworkQuality::High
    } else if avg_network_score > 50.0 {
        NetworkQuality::Medium
    } else if avg_network_score > 30.0 {
        NetworkQuality::Low
    } else {
        NetworkQuality::Suspicious
    };

    // Generate assessment
    let assessment = match network_quality {
        NetworkQuality::High => format!(
            "@{} has a high-quality network with an average score of {:.1}. Strong connections with credible accounts.",
            username, avg_network_score
        ),
        NetworkQuality::Medium => format!(
            "@{} has a medium-quality network with an average score of {:.1}. Mixed credibility in connections.",
            username, avg_network_score
        ),
        NetworkQuality::Low => format!(
            "@{} has a low-quality network with an average score of {:.1}. Many connections show poor credibility.",
            username, avg_network_score
        ),
        NetworkQuality::Suspicious => format!(
            "@{} has a suspicious network with an average score of {:.1}. High risk of bot/spam connections.",
            username, avg_network_score
        ),
    };

    Ok(SocialNetworkAnalysis {
        username,
        top_followers,
        top_friends,
        avg_follower_score,
        avg_friend_score,
        network_quality,
        key_influencers,
        assessment,
    })
}

/// Classify influence level based on follower count
fn classify_influence(followers: i64) -> String {
    match followers {
        f if f >= 1000000 => "Mega Influencer".to_string(),
        f if f >= 100000 => "Macro Influencer".to_string(),
        f if f >= 10000 => "Mid-tier Influencer".to_string(),
        f if f >= 1000 => "Micro Influencer".to_string(),
        _ => "Regular User".to_string(),
    }
}

/// Quick credibility check for a Twitter/X account.
/// Returns a simple assessment of whether an account is trustworthy.
#[tool]
pub async fn is_account_credible(
    context: &ApplicationContext,
    username: String,
    threshold: Option<f64>,
) -> crate::error::Result<CredibilityCheck> {
    debug!("Performing quick credibility check for: {}", username);

    let threshold = threshold.unwrap_or(50.0); // Default threshold of 50/100
    let score_resp = get_account_score(context, username.clone()).await?;

    let is_credible = score_resp.score >= threshold;

    let verdict = if score_resp.score >= 80.0 {
        "HIGHLY CREDIBLE"
    } else if score_resp.score >= 60.0 {
        "CREDIBLE"
    } else if score_resp.score >= 40.0 {
        "QUESTIONABLE"
    } else if score_resp.score >= 20.0 {
        "LOW CREDIBILITY"
    } else {
        "NOT CREDIBLE"
    };

    let recommendation = if is_credible {
        format!(
            "@{} meets credibility threshold ({:.1}/{:.1}). Safe to engage.",
            username, score_resp.score, threshold
        )
    } else {
        format!(
            "@{} below credibility threshold ({:.1}/{:.1}). Exercise caution.",
            username, score_resp.score, threshold
        )
    };

    Ok(CredibilityCheck {
        username,
        score: score_resp.score,
        threshold,
        is_credible,
        verdict: verdict.to_string(),
        recommendation,
    })
}

/// Simple credibility check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CredibilityCheck {
    /// Username checked
    pub username: String,
    /// Credibility score (0-100)
    pub score: f64,
    /// Threshold used
    pub threshold: f64,
    /// Whether account meets credibility threshold
    pub is_credible: bool,
    /// Simple verdict
    pub verdict: String,
    /// Recommendation
    pub recommendation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tweetscout_config_default() {
        let config = TweetScoutConfig::default();
        assert_eq!(config.base_url, "https://api.tweetscout.io/api");
        assert_eq!(config.rate_limit_per_minute, 60);
        assert_eq!(config.request_timeout, 30);
    }

    #[test]
    fn test_score_level_serialization() {
        let level = ScoreLevel::Good;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"good\"");

        let level: ScoreLevel = serde_json::from_str("\"excellent\"").unwrap();
        assert!(matches!(level, ScoreLevel::Excellent));
    }

    #[test]
    fn test_network_quality_serialization() {
        let quality = NetworkQuality::High;
        let json = serde_json::to_string(&quality).unwrap();
        assert_eq!(json, "\"high\"");

        let quality: NetworkQuality = serde_json::from_str("\"suspicious\"").unwrap();
        assert!(matches!(quality, NetworkQuality::Suspicious));
    }

    #[test]
    fn test_account_info_deserialization() {
        let json = r#"{
            "id": "123456",
            "name": "Test User",
            "screen_name": "testuser",
            "followers_count": 1000,
            "verified": true
        }"#;

        let info: AccountInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, Some("123456".to_string()));
        assert_eq!(info.screen_name, Some("testuser".to_string()));
        assert_eq!(info.followers_count, Some(1000));
        assert_eq!(info.verified, Some(true));
    }

    #[test]
    fn test_score_response_deserialization() {
        let json = r#"{
            "score": 75.5
        }"#;

        let response: ScoreResponse = serde_json::from_str(json).unwrap();
        assert!((response.score - 75.5).abs() < 0.001);
    }

    #[test]
    fn test_account_deserialization_with_typo() {
        // Test that we handle the API's typo "screeName" correctly
        let json = r#"{
            "id": "123",
            "screeName": "testuser",
            "followersCount": 500,
            "friendsCount": 200,
            "score": 65.0
        }"#;

        let account: Account = serde_json::from_str(json).unwrap();
        assert_eq!(account.screen_name, Some("testuser".to_string()));
        assert_eq!(account.followers_count, Some(500));
        assert_eq!(account.friends_count, Some(200));
        assert_eq!(account.score, Some(65.0));
    }

    #[test]
    fn test_classify_influence() {
        assert_eq!(classify_influence(2000000), "Mega Influencer");
        assert_eq!(classify_influence(500000), "Macro Influencer");
        assert_eq!(classify_influence(50000), "Mid-tier Influencer");
        assert_eq!(classify_influence(5000), "Micro Influencer");
        assert_eq!(classify_influence(500), "Regular User");
    }
}
