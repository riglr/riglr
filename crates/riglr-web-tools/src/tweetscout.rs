//! `TweetScout` integration for Twitter/X account analysis and credibility scoring
//!
//! This module provides tools for accessing `TweetScout` API to analyze Twitter/X accounts,
//! calculate credibility scores, and analyze social networks for crypto influencer detection.

use crate::{client::WebClient, error::WebToolError};
use riglr_core::provider::ApplicationContext;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tracing::{debug, info};

/// Environment variable name for `TweetScout` API key
const TWEETSCOUT_API_KEY_ENV: &str = "TWEETSCOUT_API_KEY";

/// Configuration for `TweetScout` API access
#[derive(Debug, Clone)]
pub struct TweetScoutConfig {
    /// API key for authentication
    pub api_key: Option<String>,
    /// API base URL (default: <https://api.tweetscout.io/api>)
    pub base_url: String,
    /// Rate limit requests per minute (default: 60)
    pub rate_limit_per_minute: u32,
    /// Timeout for API requests in seconds (default: 30)
    pub request_timeout: u64,
}

impl Default for TweetScoutConfig {
    fn default() -> Self {
        Self {
            api_key: env::var(TWEETSCOUT_API_KEY_ENV).ok(),
            base_url: "https://api.tweetscout.io/api".to_string(),
            rate_limit_per_minute: 60,
            request_timeout: 30,
        }
    }
}

/// Account information response from `TweetScout`
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountInfo {
    /// Avatar image URL
    pub avatar: Option<String>,
    /// Banner image URL
    pub banner: Option<String>,
    /// Profile description/bio
    pub description: Option<String>,
    /// Number of followers
    pub followers_count: Option<i64>,
    /// Number of accounts following (friends)
    pub friends_count: Option<i64>,
    /// User ID
    pub id: Option<String>,
    /// Display name
    pub name: Option<String>,
    /// Account registration date
    pub register_date: Option<String>,
    /// Username/handle
    pub screen_name: Option<String>,
    /// Number of tweets/posts
    pub statuses_count: Option<i64>,
    /// Verification status
    pub verified: Option<bool>,
}

/// Score response from `TweetScout`
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScoreResponse {
    /// Credibility score (0-100)
    pub score: f64,
}

/// Account information for followers/friends lists
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Account {
    /// Avatar URL
    pub avatar: Option<String>,
    /// Banner URL
    pub banner: Option<String>,
    /// Profile description
    pub description: Option<String>,
    /// Followers count (Note: API uses camelCase)
    #[serde(rename = "followersCount")]
    pub followers_count: Option<i64>,
    /// Friends/following count
    #[serde(rename = "friendsCount")]
    pub friends_count: Option<i64>,
    /// User ID
    pub id: Option<String>,
    /// Display name
    pub name: Option<String>,
    /// Registration date
    #[serde(rename = "registerDate")]
    pub register_date: Option<String>,
    /// Account score
    pub score: Option<f64>,
    /// Username/handle (Note: API returns screeName, handling typo)
    #[serde(rename = "screeName")]
    pub screen_name: Option<String>,
    /// Number of posts
    pub statuses_count: Option<i64>,
    /// Verification status
    pub verified: Option<bool>,
}

/// Helper function to get `TweetScout` API key from `ApplicationContext`
fn get_api_key_from_context(context: &ApplicationContext) -> Result<String, WebToolError> {
    context
        .config
        .providers
        .tweetscout_api_key
        .clone()
        .ok_or_else(|| {
            WebToolError::Config(
                "TweetScout API key not configured. Set TWEETSCOUT_API_KEY in your environment."
                    .to_string(),
            )
        })
}

/// Error response from `TweetScout` API
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorResponse {
    /// Error message
    pub message: String,
}

/// Comprehensive account analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountAnalysis {
    /// Account age in days
    pub account_age_days: Option<i64>,
    /// Summary assessment
    pub assessment: String,
    /// Average tweets per day
    pub avg_tweets_per_day: Option<f64>,
    /// Credibility score (0-100)
    pub credibility_score: f64,
    /// Engagement metrics
    pub engagement: EngagementMetrics,
    /// Follower to following ratio
    pub follower_ratio: Option<f64>,
    /// Basic account information
    pub info: AccountInfo,
    /// Risk indicators
    pub risk_indicators: Vec<String>,
    /// Score classification
    pub score_level: ScoreLevel,
    /// Username analyzed
    pub username: String,
}

/// Credibility score level classification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ScoreLevel {
    /// Excellent credibility (80-100)
    #[serde(rename = "excellent")]
    Excellent,
    /// Fair credibility (40-60)
    #[serde(rename = "fair")]
    Fair,
    /// Good credibility (60-80)
    #[serde(rename = "good")]
    Good,
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
    /// Engagement rate estimate
    pub engagement_rate: f64,
    /// Total followers
    pub followers: i64,
    /// Total following
    pub following: i64,
    /// Whether account is likely a bot
    pub likely_bot: bool,
    /// Whether account is likely spam
    pub likely_spam: bool,
    /// Total posts
    pub posts: i64,
}

/// Social network analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SocialNetworkAnalysis {
    /// Network assessment
    pub assessment: String,
    /// Average follower score
    pub avg_follower_score: f64,
    /// Average friend score
    pub avg_friend_score: f64,
    /// Key influencers in network
    pub key_influencers: Vec<String>,
    /// Quality of network
    pub network_quality: NetworkQuality,
    /// Top followers with scores
    pub top_followers: Vec<ScoredAccount>,
    /// Top friends (following) with scores
    pub top_friends: Vec<ScoredAccount>,
    /// Username analyzed
    pub username: String,
}

/// Account with score for network analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScoredAccount {
    /// Follower count
    pub followers: i64,
    /// Influence level
    pub influence_level: String,
    /// Display name
    pub name: String,
    /// Score
    pub score: f64,
    /// Username
    pub username: String,
    /// Whether verified
    pub verified: bool,
}

/// Network quality assessment
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum NetworkQuality {
    /// High quality network
    #[serde(rename = "high")]
    High,
    /// Low quality network
    #[serde(rename = "low")]
    Low,
    /// Medium quality network
    #[serde(rename = "medium")]
    Medium,
    /// Suspicious network
    #[serde(rename = "suspicious")]
    Suspicious,
}

/// Get basic information about a Twitter/X account.
///
/// # Errors
///
/// Returns an error if the API request fails or the response cannot be parsed.
#[tool]
pub async fn get_account_info(
    context: &ApplicationContext,
    username: String,
) -> Result<AccountInfo, WebToolError> {
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
        .map_err(|e| WebToolError::Network(format!("Failed to fetch account info: {e}")))?;

    let info: AccountInfo = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Failed to parse TweetScout response: {e}")))?;

    info!(
        "Successfully fetched info for @{} - Followers: {:?}, Verified: {:?}",
        username, info.followers_count, info.verified
    );

    Ok(info)
}

/// Get the credibility score for a Twitter/X account.
/// Returns a score from 0-100 indicating account trustworthiness.
///
/// # Errors
///
/// Returns an error if the API request fails or the response cannot be parsed.
#[tool]
pub async fn get_account_score(
    context: &ApplicationContext,
    username: String,
) -> Result<ScoreResponse, WebToolError> {
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
        .map_err(|e| WebToolError::Network(format!("Failed to fetch score: {e}")))?;

    let score: ScoreResponse = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Failed to parse score response: {e}")))?;

    info!(
        "Successfully fetched score for @{}: {:.1}/100",
        username, score.score
    );

    Ok(score)
}

/// Get the top 20 followers of a Twitter/X account with their scores.
///
/// # Errors
///
/// Returns an error if the API request fails or the response cannot be parsed.
#[tool]
pub async fn get_top_followers(
    context: &ApplicationContext,
    username: String,
) -> Result<Vec<Account>, WebToolError> {
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
        .map_err(|e| WebToolError::Network(format!("Failed to fetch followers: {e}")))?;

    let followers: Vec<Account> = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Failed to parse followers response: {e}")))?;

    info!(
        "Successfully fetched {} top followers for @{}",
        followers.len(),
        username
    );

    Ok(followers)
}

/// Get the top 20 friends (accounts being followed) of a Twitter/X account with their scores.
///
/// # Errors
///
/// Returns an error if the API request fails or the response cannot be parsed.
#[tool]
pub async fn get_top_friends(
    context: &ApplicationContext,
    username: String,
) -> Result<Vec<Account>, WebToolError> {
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
        .map_err(|e| WebToolError::Network(format!("Failed to fetch friends: {e}")))?;

    let friends: Vec<Account> = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Failed to parse friends response: {e}")))?;

    info!(
        "Successfully fetched {} top friends for @{}",
        friends.len(),
        username
    );

    Ok(friends)
}

/// Perform comprehensive analysis of a Twitter/X account including credibility scoring.
/// Combines account info and score into a detailed assessment.
///
/// # Errors
///
/// Returns an error if the API requests fail or the responses cannot be parsed.
#[tool]
pub async fn analyze_account(
    context: &ApplicationContext,
    username: String,
) -> Result<AccountAnalysis, WebToolError> {
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
        account_age_days,
        assessment,
        avg_tweets_per_day,
        credibility_score: score_resp.score,
        engagement,
        follower_ratio,
        info,
        risk_indicators,
        score_level,
        username,
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
const fn calculate_avg_tweets_per_day(
    _info: &AccountInfo,
    _account_age_days: Option<i64>,
) -> Option<f64> {
    // Would calculate if we had proper age implementation
    None
}

/// Calculate follower to following ratio
fn calculate_follower_ratio(info: &AccountInfo) -> Option<f64> {
    if let (Some(followers), Some(following)) = (info.followers_count, info.friends_count) {
        if following > 0 {
            #[expect(clippy::cast_precision_loss)]
            {
                Some(followers as f64 / following as f64)
            }
        } else {
            None
        }
    } else {
        None
    }
}

/// Determine score level classification from numeric score
const fn determine_score_level(score: f64) -> ScoreLevel {
    #[expect(clippy::cast_possible_truncation)]
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
    #[expect(clippy::arithmetic_side_effects)]
    let likely_bot = score < 30.0
        || (following > followers * 10 && followers < 100)
        || (posts > 100_000 && followers < 1000);

    let likely_spam = score < 20.0 || (following > 5000 && followers < 100);

    let engagement_rate = if posts > 0 {
        #[expect(clippy::cast_precision_loss, clippy::arithmetic_side_effects)]
        {
            ((followers + following) as f64 / posts as f64).min(100.0)
        }
    } else {
        0.0
    };

    EngagementMetrics {
        engagement_rate,
        followers,
        following,
        likely_bot,
        likely_spam,
        posts,
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

/// Simple credibility check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CredibilityCheck {
    /// Whether account meets credibility threshold
    pub is_credible: bool,
    /// Recommendation
    pub recommendation: String,
    /// Credibility score (0-100)
    pub score: f64,
    /// Threshold used
    pub threshold: f64,
    /// Username checked
    pub username: String,
    /// Simple verdict
    pub verdict: String,
}

/// Build textual assessment based on score level
fn build_assessment(username: &str, score: f64, score_level: &ScoreLevel) -> String {
    match *score_level {
        ScoreLevel::Excellent => format!(
            "User @{username} demonstrates excellent credibility with a score of {score:.2}. This account shows strong engagement patterns and reliable behavior."
        ),
        ScoreLevel::Good => format!(
            "User @{username} shows good credibility with a score of {score:.2}. Profile appears reliable with positive indicators."
        ),
        ScoreLevel::Fair => format!(
            "User @{username} shows moderate credibility with a score of {score:.2}. Some indicators suggest caution but overall profile appears legitimate."
        ),
        ScoreLevel::Poor => format!(
            "User @{username} has limited credibility with a score of {score:.2}. Multiple risk factors identified - proceed with caution."
        ),
        ScoreLevel::VeryPoor => format!(
            "User @{username} has very poor credibility with a score of {score:.2}. Significant risk factors identified - high caution advised."
        ),
    }
}

/// Analyze the social network of a Twitter/X account including followers and friends analysis.
/// Returns comprehensive metrics about the account's social connections and their credibility.
///
/// # Errors
///
/// Returns an error if the API requests fail or the responses cannot be parsed.
pub async fn analyze_social_network(
    context: &ApplicationContext,
    username: String,
) -> Result<SocialNetworkAnalysis, WebToolError> {
    debug!("Analyzing social network for: {}", username);

    // Fetch followers and friends
    let followers = get_top_followers(context, username.clone()).await?;
    let friends = get_top_friends(context, username.clone()).await?;

    // Convert to scored accounts
    let top_followers: Vec<ScoredAccount> = followers
        .iter()
        .map(|acc| ScoredAccount {
            followers: acc.followers_count.unwrap_or(0),
            influence_level: classify_influence(acc.followers_count.unwrap_or(0)),
            name: acc.name.clone().unwrap_or_default(),
            score: acc.score.unwrap_or(0.0),
            username: acc.screen_name.clone().unwrap_or_default(),
            verified: acc.verified.unwrap_or(false),
        })
        .collect();

    let top_friends: Vec<ScoredAccount> = friends
        .iter()
        .map(|acc| ScoredAccount {
            followers: acc.followers_count.unwrap_or(0),
            influence_level: classify_influence(acc.followers_count.unwrap_or(0)),
            name: acc.name.clone().unwrap_or_default(),
            score: acc.score.unwrap_or(0.0),
            username: acc.screen_name.clone().unwrap_or_default(),
            verified: acc.verified.unwrap_or(false),
        })
        .collect();

    // Calculate average scores
    let avg_follower_score = if top_followers.is_empty() {
        0.0
    } else {
        {
            #[expect(clippy::cast_precision_loss)]
            {
                top_followers.iter().map(|a| a.score).sum::<f64>() / { top_followers.len() as f64 }
            }
        }
    };

    let avg_friend_score = if top_friends.is_empty() {
        0.0
    } else {
        {
            #[expect(clippy::cast_precision_loss)]
            {
                top_friends.iter().map(|a| a.score).sum::<f64>() / { top_friends.len() as f64 }
            }
        }
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
    let avg_network_score = f64::midpoint(avg_follower_score, avg_friend_score);
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
            "@{username} has a high-quality network with an average score of {avg_network_score:.1}. Strong connections with credible accounts."
        ),
        NetworkQuality::Medium => format!(
            "@{username} has a medium-quality network with an average score of {avg_network_score:.1}. Mixed credibility in connections."
        ),
        NetworkQuality::Low => format!(
            "@{username} has a low-quality network with an average score of {avg_network_score:.1}. Many connections show poor credibility."
        ),
        NetworkQuality::Suspicious => format!(
            "@{username} has a suspicious network with an average score of {avg_network_score:.1}. High risk of bot/spam connections."
        ),
    };

    Ok(SocialNetworkAnalysis {
        assessment,
        avg_follower_score,
        avg_friend_score,
        key_influencers,
        network_quality,
        top_followers,
        top_friends,
        username,
    })
}

/// Classify influence level based on follower count
fn classify_influence(followers: i64) -> String {
    match followers {
        f if f >= 1_000_000 => "Mega Influencer".to_string(),
        f if f >= 100_000 => "Macro Influencer".to_string(),
        f if f >= 10000 => "Mid-tier Influencer".to_string(),
        f if f >= 1000 => "Micro Influencer".to_string(),
        _ => "Regular User".to_string(),
    }
}

/// Quick credibility check for a Twitter/X account.
/// Returns a simple assessment of whether an account is trustworthy.
///
/// # Errors
///
/// Returns an error if the API request fails or the response cannot be parsed.
#[tool]
pub async fn is_account_credible(
    context: &ApplicationContext,
    username: String,
    threshold: Option<f64>,
) -> Result<CredibilityCheck, WebToolError> {
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
        is_credible,
        recommendation,
        score: score_resp.score,
        threshold,
        username,
        verdict: verdict.to_string(),
    })
}

#[cfg(test)]
#[expect(clippy::expect_used)]
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
        let json = serde_json::to_string(&level).expect("Failed to serialize ScoreLevel in test");
        assert_eq!(json, "\"good\"");

        let level: ScoreLevel = serde_json::from_str("\"excellent\"")
            .expect("Failed to deserialize ScoreLevel from JSON in test");
        assert!(matches!(level, ScoreLevel::Excellent));
    }

    #[test]
    fn test_network_quality_serialization() {
        let quality = NetworkQuality::High;
        let json =
            serde_json::to_string(&quality).expect("Failed to serialize NetworkQuality in test");
        assert_eq!(json, "\"high\"");

        let quality: NetworkQuality = serde_json::from_str("\"suspicious\"")
            .expect("Failed to deserialize NetworkQuality from JSON in test");
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

        let info: AccountInfo = serde_json::from_str(json)
            .expect("Failed to deserialize AccountInfo from JSON in test");
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

        let response: ScoreResponse = serde_json::from_str(json)
            .expect("Failed to deserialize ScoreResponse from JSON in test");
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

        let account: Account = serde_json::from_str(json)
            .expect("Failed to deserialize Account from JSON in test with typo handling");
        assert_eq!(account.screen_name, Some("testuser".to_string()));
        assert_eq!(account.followers_count, Some(500));
        assert_eq!(account.friends_count, Some(200));
        assert_eq!(account.score, Some(65.0));
    }

    #[test]
    fn test_classify_influence() {
        assert_eq!(classify_influence(2_000_000), "Mega Influencer");
        assert_eq!(classify_influence(500_000), "Macro Influencer");
        assert_eq!(classify_influence(50000), "Mid-tier Influencer");
        assert_eq!(classify_influence(5000), "Micro Influencer");
        assert_eq!(classify_influence(500), "Regular User");
    }
}
