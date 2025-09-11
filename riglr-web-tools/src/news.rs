//! Comprehensive cryptocurrency and financial news aggregation
//!
//! This module provides news aggregation with heuristic-based sentiment analysis
//! and market impact assessment for AI agents to stay informed about market developments.

use crate::{client::WebClient, error::WebToolError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use regex::Regex;
use riglr_core::provider::ApplicationContext;
use riglr_core::sentiment::SentimentAnalyzerMarker;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Trait for pluggable sentiment analysis
#[async_trait]
pub trait SentimentAnalyzer: Send + Sync {
    /// Analyze sentiment from text components
    async fn analyze(
        &self,
        title: &str,
        description: Option<&str>,
        content: Option<&str>,
    ) -> Result<NewsSentiment, WebToolError>;
}

/// Default lexicon-based sentiment analyzer
#[derive(Debug)]
pub struct LexiconSentimentAnalyzer {
    /// Positive words with their weights
    positive_words: Vec<(&'static str, f64)>,
    /// Negative words with their weights
    negative_words: Vec<(&'static str, f64)>,
}

impl Default for LexiconSentimentAnalyzer {
    fn default() -> Self {
        Self {
            positive_words: vec![
                ("bullish", 0.8),
                ("surge", 0.7),
                ("rally", 0.7),
                ("breakthrough", 0.8),
                ("adoption", 0.6),
                ("partnership", 0.6),
                ("growth", 0.5),
                ("success", 0.6),
                ("innovative", 0.5),
                ("leading", 0.4),
                ("strong", 0.5),
                ("positive", 0.5),
                ("gains", 0.6),
                ("rise", 0.5),
                ("increase", 0.4),
                ("improve", 0.5),
                ("upgrade", 0.6),
                ("expand", 0.5),
                ("launch", 0.4),
                ("milestone", 0.6),
            ],
            negative_words: vec![
                ("bearish", -0.8),
                ("crash", -0.9),
                ("plunge", -0.8),
                ("collapse", -0.9),
                ("hack", -0.9),
                ("exploit", -0.9),
                ("scam", -0.9),
                ("fraud", -0.9),
                ("decline", -0.6),
                ("fall", -0.5),
                ("drop", -0.5),
                ("loss", -0.6),
                ("failure", -0.7),
                ("risk", -0.4),
                ("concern", -0.4),
                ("warning", -0.5),
                ("threat", -0.6),
                ("vulnerable", -0.6),
                ("weak", -0.5),
                ("crisis", -0.8),
                ("panic", -0.7),
                ("fear", -0.6),
                ("uncertainty", -0.5),
                ("volatile", -0.4),
                ("dump", -0.7),
                ("rug", -0.9),
                ("regulatory", -0.3),
                ("lawsuit", -0.6),
                ("investigation", -0.5),
                ("ban", -0.7),
            ],
        }
    }
}

#[async_trait]
impl SentimentAnalyzer for LexiconSentimentAnalyzer {
    async fn analyze(
        &self,
        title: &str,
        description: Option<&str>,
        content: Option<&str>,
    ) -> Result<NewsSentiment, WebToolError> {
        // Call the existing analyze_sentiment_impl function with the lexicon
        Ok(self.analyze_sentiment_impl(title, description, content))
    }
}

// Implement the marker trait to allow storage in ApplicationContext
impl SentimentAnalyzerMarker for LexiconSentimentAnalyzer {}

impl LexiconSentimentAnalyzer {
    /// Implementation of lexicon-based sentiment analysis
    fn analyze_sentiment_impl(
        &self,
        title: &str,
        description: Option<&str>,
        content: Option<&str>,
    ) -> NewsSentiment {
        let full_text = format!(
            "{} {} {}",
            title,
            description.unwrap_or(""),
            content.unwrap_or("")
        );

        let text_lower = full_text.to_lowercase();

        // Calculate overall sentiment score
        let mut sentiment_score = 0.0;
        let mut word_count = 0;

        for (word, weight) in &self.positive_words {
            let count = text_lower.matches(word).count();
            sentiment_score += count as f64 * weight;
            word_count += count;
        }

        for (word, weight) in &self.negative_words {
            let count = text_lower.matches(word).count();
            sentiment_score += count as f64 * weight;
            word_count += count;
        }

        // Normalize sentiment score
        let overall_score = if word_count > 0 {
            (sentiment_score / word_count as f64).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        // Calculate confidence based on word count and text length
        let confidence = ((word_count as f64 / 10.0).min(1.0) * 0.5
            + (full_text.len() as f64 / 500.0).min(1.0) * 0.5)
            .clamp(0.3, 0.95);

        // Determine classification
        let classification = if overall_score > 0.2 {
            "Bullish"
        } else if overall_score < -0.2 {
            "Bearish"
        } else {
            "Neutral"
        }
        .to_string();

        // Calculate emotional indicators
        let emotions = self.calculate_emotions(&text_lower);

        // Extract key phrases that contribute to sentiment
        let key_phrases = self.extract_key_phrases(&full_text);

        // Topic-specific sentiments
        let topic_sentiments = self.calculate_topic_sentiments(&text_lower, overall_score);

        NewsSentiment {
            overall_score,
            confidence,
            classification,
            topic_sentiments,
            emotions,
            key_phrases,
        }
    }

    fn calculate_emotions(&self, text_lower: &str) -> EmotionalIndicators {
        let fear_words = ["fear", "panic", "crash", "crisis", "collapse"];
        let greed_words = ["moon", "rally", "surge", "bullish", "fomo"];

        let fear_count = fear_words
            .iter()
            .filter(|w| text_lower.contains(*w))
            .count();
        let greed_count = greed_words
            .iter()
            .filter(|w| text_lower.contains(*w))
            .count();

        EmotionalIndicators {
            fear: (fear_count as f64 / 5.0).min(1.0),
            greed: (greed_count as f64 / 5.0).min(1.0),
            excitement: if text_lower.contains("exciting") || text_lower.contains("breakthrough") {
                0.5
            } else {
                0.0
            },
            uncertainty: if text_lower.contains("uncertain") || text_lower.contains("volatile") {
                0.5
            } else {
                0.0
            },
            urgency: if text_lower.contains("urgent") || text_lower.contains("immediate") {
                0.5
            } else {
                0.0
            },
        }
    }

    fn extract_key_phrases(&self, full_text: &str) -> Vec<SentimentPhrase> {
        let mut key_phrases = Vec::new();

        // Look for specific phrase patterns
        let phrase_patterns = [
            (
                r"(?i)(bullish|positive|optimistic) (?:on|about|for) (\w+)",
                0.5,
            ),
            (
                r"(?i)(bearish|negative|pessimistic) (?:on|about|for) (\w+)",
                -0.5,
            ),
            (r"(?i)all.time.high", 0.6),
            (r"(?i)all.time.low", -0.6),
            (r"(?i)break(?:ing|s)?\s+(?:through|above)", 0.4),
            (r"(?i)break(?:ing|s)?\s+(?:below|down)", -0.4),
        ];

        for (pattern, contribution) in &phrase_patterns {
            let regex_result = Regex::new(pattern);
            if let Ok(re) = regex_result {
                for matched in re.find_iter(full_text) {
                    key_phrases.push(SentimentPhrase {
                        phrase: matched.as_str().to_string(),
                        sentiment_contribution: *contribution,
                        confidence: 0.7,
                    });
                }
            }
        }

        key_phrases
    }

    fn calculate_topic_sentiments(
        &self,
        text_lower: &str,
        overall_score: f64,
    ) -> HashMap<String, f64> {
        let mut topic_sentiments = HashMap::new();
        let topics = [
            "bitcoin",
            "ethereum",
            "defi",
            "nft",
            "regulation",
            "adoption",
        ];

        for topic in &topics {
            if text_lower.contains(topic) {
                // Calculate sentiment specific to this topic's context
                let topic_score = if text_lower.contains(&format!("{} surge", topic))
                    || text_lower.contains(&format!("{} rally", topic))
                {
                    0.5
                } else if text_lower.contains(&format!("{} crash", topic))
                    || text_lower.contains(&format!("{} plunge", topic))
                {
                    -0.5
                } else {
                    overall_score * 0.7 // Slightly dampened overall sentiment
                };
                topic_sentiments.insert(topic.to_string(), topic_score);
            }
        }

        topic_sentiments
    }
}

/// Configuration for news aggregation services
#[derive(Debug, Clone)]
pub struct NewsConfig {
    /// NewsAPI.org API key
    pub newsapi_key: String,
    /// CryptoPanic API key
    pub cryptopanic_key: String,
    /// Base URL for news aggregation service
    pub base_url: String,
    /// Maximum articles per request (default: 50)
    pub max_articles: u32,
    /// News freshness window in hours (default: 24)
    pub freshness_hours: u32,
    /// Minimum credibility score (0-100)
    pub min_credibility_score: u32,
}

/// Comprehensive news article with metadata and analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsArticle {
    /// Unique article identifier
    pub id: String,
    /// Article title
    pub title: String,
    /// Article URL
    pub url: String,
    /// Article description/summary
    pub description: Option<String>,
    /// Full article content (if extracted)
    pub content: Option<String>,
    /// Publication timestamp
    pub published_at: DateTime<Utc>,
    /// News source information
    pub source: NewsSource,
    /// Article category and tags
    pub category: NewsCategory,
    /// Sentiment analysis results
    pub sentiment: NewsSentiment,
    /// Market impact assessment
    pub market_impact: MarketImpact,
    /// Entities mentioned in the article
    pub entities: Vec<NewsEntity>,
    /// Related cryptocurrencies/assets
    pub related_assets: Vec<String>,
    /// Article quality metrics
    pub quality_metrics: QualityMetrics,
    /// Social engagement metrics
    pub social_metrics: Option<SocialMetrics>,
}

/// News source information and credibility
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsSource {
    /// Source identifier
    pub id: String,
    /// Source name (e.g., "CoinDesk", "Reuters")
    pub name: String,
    /// Source website URL
    pub url: String,
    /// Source category (Mainstream, Crypto-Native, Blog, etc.)
    pub category: String,
    /// Credibility score (0-100)
    pub credibility_score: u32,
    /// Historical accuracy rating
    pub accuracy_rating: Option<f64>,
    /// Source bias score (-1.0 to 1.0, -1 = bearish, 1 = bullish)
    pub bias_score: Option<f64>,
    /// Whether source is verified/trusted
    pub is_verified: bool,
    /// Source logo URL
    pub logo_url: Option<String>,
}

/// News category and classification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsCategory {
    /// Primary category (Breaking, Analysis, Opinion, etc.)
    pub primary: String,
    /// Sub-category (DeFi, NFT, Regulation, etc.)
    pub sub_category: Option<String>,
    /// Article tags
    pub tags: Vec<String>,
    /// Geographic relevance
    pub geographic_scope: Vec<String>,
    /// Target audience (Retail, Institutional, Developer)
    pub target_audience: String,
}

/// Sentiment analysis for news article
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsSentiment {
    /// Overall sentiment score (-1.0 to 1.0)
    pub overall_score: f64,
    /// Sentiment confidence (0.0 to 1.0)
    pub confidence: f64,
    /// Sentiment classification (Bullish, Bearish, Neutral)
    pub classification: String,
    /// Sentiment breakdown by topic
    pub topic_sentiments: HashMap<String, f64>,
    /// Emotional indicators
    pub emotions: EmotionalIndicators,
    /// Key sentiment phrases extracted
    pub key_phrases: Vec<SentimentPhrase>,
}

/// Emotional indicators in news content
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EmotionalIndicators {
    /// Fear level (0.0 to 1.0)
    pub fear: f64,
    /// Greed level (0.0 to 1.0)
    pub greed: f64,
    /// Excitement level (0.0 to 1.0)
    pub excitement: f64,
    /// Uncertainty level (0.0 to 1.0)
    pub uncertainty: f64,
    /// Urgency level (0.0 to 1.0)
    pub urgency: f64,
}

/// Key phrases contributing to sentiment
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SentimentPhrase {
    /// The phrase text
    pub phrase: String,
    /// Sentiment contribution (-1.0 to 1.0)
    pub sentiment_contribution: f64,
    /// Confidence in this analysis (0.0 to 1.0)
    pub confidence: f64,
}

/// Market impact assessment for news
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MarketImpact {
    /// Predicted impact level (High, Medium, Low, Negligible)
    pub impact_level: String,
    /// Impact score (0-100)
    pub impact_score: u32,
    /// Time horizon for impact (Immediate, Short-term, Long-term)
    pub time_horizon: String,
    /// Affected market sectors
    pub affected_sectors: Vec<String>,
    /// Potential price impact percentage
    pub potential_price_impact: Option<f64>,
    /// Historical correlation with similar news
    pub historical_correlation: Option<f64>,
    /// Risk factors identified
    pub risk_factors: Vec<String>,
}

/// Entities mentioned in news (people, companies, assets)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsEntity {
    /// Entity name
    pub name: String,
    /// Entity type (Person, Company, Cryptocurrency, etc.)
    pub entity_type: String,
    /// Relevance to the article (0.0 to 1.0)
    pub relevance_score: f64,
    /// Sentiment specifically towards this entity
    pub sentiment: Option<f64>,
    /// Number of mentions in the article
    pub mention_count: u32,
    /// Context of mentions
    pub contexts: Vec<String>,
}

/// Article quality assessment metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityMetrics {
    /// Overall quality score (0-100)
    pub overall_score: u32,
    /// Content depth assessment
    pub depth_score: u32,
    /// Fact-checking score
    pub factual_accuracy: u32,
    /// Writing quality score
    pub writing_quality: u32,
    /// Source citation quality
    pub citation_quality: u32,
    /// Uniqueness vs other articles (0-100)
    pub uniqueness_score: u32,
    /// Estimated reading difficulty (1-10)
    pub reading_difficulty: u32,
}

/// Social media engagement metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SocialMetrics {
    /// Total social shares
    pub total_shares: u32,
    /// Twitter mentions/shares
    pub twitter_shares: u32,
    /// Reddit discussions
    pub reddit_mentions: u32,
    /// LinkedIn shares
    pub linkedin_shares: u32,
    /// Social sentiment (different from article sentiment)
    pub social_sentiment: f64,
    /// Viral potential score (0-100)
    pub viral_score: u32,
    /// Influencer engagement
    pub influencer_mentions: u32,
}

/// Comprehensive news aggregation result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsAggregationResult {
    /// Search query or topic
    pub topic: String,
    /// Found news articles
    pub articles: Vec<NewsArticle>,
    /// Aggregation metadata
    pub metadata: AggregationMetadata,
    /// Market insights from the news
    pub insights: NewsInsights,
    /// Trending topics extracted
    pub trending_topics: Vec<TrendingTopic>,
    /// Aggregation timestamp
    pub aggregated_at: DateTime<Utc>,
}

/// Metadata about the news aggregation process
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AggregationMetadata {
    /// Total articles found across all sources
    pub total_articles: u32,
    /// Articles returned after filtering
    pub returned_articles: u32,
    /// Sources queried
    pub sources_queried: Vec<String>,
    /// Average credibility of returned articles
    pub avg_credibility: f64,
    /// Time range covered
    pub time_range_hours: u32,
    /// Duplicate articles removed
    pub duplicates_removed: u32,
}

/// Insights extracted from news aggregation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsInsights {
    /// Overall market sentiment from news
    pub overall_sentiment: f64,
    /// Sentiment trend over time
    pub sentiment_trend: String, // "Improving", "Declining", "Stable"
    /// Most mentioned entities
    pub top_entities: Vec<EntityMention>,
    /// Dominant themes/topics
    pub dominant_themes: Vec<String>,
    /// Geographical distribution of news
    pub geographic_distribution: HashMap<String, u32>,
    /// Source diversity metrics
    pub source_diversity: SourceDiversity,
    /// Market impact distribution
    pub impact_distribution: HashMap<String, u32>,
}

/// Entity mention statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityMention {
    /// Entity name
    pub name: String,
    /// Number of mentions across articles
    pub mention_count: u32,
    /// Average sentiment towards entity
    pub avg_sentiment: f64,
    /// Entity type
    pub entity_type: String,
    /// Trending status
    pub is_trending: bool,
}

/// Source diversity analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SourceDiversity {
    /// Number of unique sources
    pub unique_sources: u32,
    /// Source type distribution
    pub source_types: HashMap<String, u32>,
    /// Geographic source distribution
    pub geographic_sources: HashMap<String, u32>,
    /// Credibility distribution
    pub credibility_distribution: HashMap<String, u32>, // "High", "Medium", "Low"
}

/// Trending topic analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrendingTopic {
    /// Topic name
    pub topic: String,
    /// Number of articles mentioning this topic
    pub article_count: u32,
    /// Trend velocity (mentions per hour)
    pub velocity: f64,
    /// Sentiment towards this topic
    pub sentiment: f64,
    /// Related keywords
    pub related_keywords: Vec<String>,
    /// Geographic concentration
    pub geographic_focus: Vec<String>,
}

/// Breaking news alert
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BreakingNewsAlert {
    /// Alert ID
    pub id: String,
    /// Alert severity (Critical, High, Medium, Low)
    pub severity: String,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Related articles
    pub articles: Vec<NewsArticle>,
    /// Estimated market impact
    pub estimated_impact: MarketImpact,
    /// Alert timestamp
    pub created_at: DateTime<Utc>,
    /// Alert expiration
    pub expires_at: Option<DateTime<Utc>>,
}

impl Default for NewsConfig {
    fn default() -> Self {
        Self {
            newsapi_key: String::default(),
            cryptopanic_key: String::default(),
            base_url: "https://newsapi.org/v2".to_string(),
            max_articles: 50,
            freshness_hours: 24,
            min_credibility_score: 60,
        }
    }
}

impl NewsConfig {
    /// Create NewsConfig from ApplicationContext
    fn from_context(context: &ApplicationContext) -> Self {
        Self {
            newsapi_key: context
                .config
                .providers
                .newsapi_key
                .clone()
                .unwrap_or_default(),
            cryptopanic_key: context
                .config
                .providers
                .cryptopanic_key
                .clone()
                .unwrap_or_default(),
            base_url: "https://newsapi.org/v2".to_string(),
            max_articles: 50,
            freshness_hours: 24,
            min_credibility_score: 60,
        }
    }
}

/// Get comprehensive cryptocurrency news for a specific topic
///
/// This tool aggregates news from multiple sources, performs sentiment analysis,
/// and assesses market impact for cryptocurrency-related topics.
#[tool]
pub async fn get_crypto_news(
    context: &riglr_core::provider::ApplicationContext,
    topic: String,
    time_window: Option<String>,       // "1h", "6h", "24h", "week"
    source_types: Option<Vec<String>>, // "mainstream", "crypto", "analysis"
    min_credibility: Option<u32>,
    include_analysis: Option<bool>,
) -> crate::error::Result<NewsAggregationResult> {
    debug!(
        "Aggregating crypto news for topic: '{}' within {}",
        topic,
        time_window.as_deref().unwrap_or("24h")
    );

    let config = NewsConfig::from_context(context);
    if config.newsapi_key.is_empty() && config.cryptopanic_key.is_empty() {
        return Err(WebToolError::Auth(
            "No news API keys configured".to_string(),
        ));
    }

    let client = WebClient::default();

    // Query multiple news sources
    let mut all_articles = Vec::new();
    let mut sources_queried = Vec::new();

    // NewsAPI.org for mainstream coverage
    if !config.newsapi_key.is_empty() {
        match query_newsapi(&client, &config, &topic, &time_window).await {
            Ok(mut articles) => {
                all_articles.append(&mut articles);
                sources_queried.push("NewsAPI".to_string());
            }
            Err(e) => warn!("Failed to query NewsAPI: {}", e),
        }
    }

    // CryptoPanic for crypto-specific news
    if !config.cryptopanic_key.is_empty() {
        match query_cryptopanic(&client, &config, &topic, &time_window).await {
            Ok(mut articles) => {
                all_articles.append(&mut articles);
                sources_queried.push("CryptoPanic".to_string());
            }
            Err(e) => warn!("Failed to query CryptoPanic: {}", e),
        }
    }

    // Filter by source types if specified
    if let Some(types) = source_types {
        all_articles.retain(|article| types.contains(&article.source.category.to_lowercase()));
    }

    // Filter by minimum credibility
    let min_cred = min_credibility.unwrap_or(config.min_credibility_score);
    all_articles.retain(|article| article.source.credibility_score >= min_cred);

    // Remove duplicates and sort by recency
    let articles = deduplicate_articles(all_articles);

    // Generate insights if requested
    let insights = if include_analysis.unwrap_or(true) {
        analyze_news_collection(&articles).await?
    } else {
        NewsInsights {
            overall_sentiment: 0.0,
            sentiment_trend: "Unknown".to_string(),
            top_entities: vec![],
            dominant_themes: vec![],
            geographic_distribution: HashMap::new(),
            source_diversity: SourceDiversity {
                unique_sources: 0,
                source_types: HashMap::new(),
                geographic_sources: HashMap::new(),
                credibility_distribution: HashMap::new(),
            },
            impact_distribution: HashMap::new(),
        }
    };

    // Extract trending topics
    let trending_topics = extract_trending_topics(&articles).await?;

    let result = NewsAggregationResult {
        topic: topic.clone(),
        articles: articles.clone(),
        metadata: AggregationMetadata {
            total_articles: articles.len() as u32,
            returned_articles: articles.len() as u32,
            sources_queried,
            avg_credibility: calculate_avg_credibility(&articles),
            time_range_hours: parse_time_window(&time_window.unwrap_or_else(|| "24h".to_string())),
            duplicates_removed: 0, // Would track actual duplicates
        },
        insights,
        trending_topics,
        aggregated_at: Utc::now(),
    };

    info!(
        "Crypto news aggregation completed: {} articles for '{}'",
        result.articles.len(),
        topic
    );

    Ok(result)
}

/// Get trending cryptocurrency news across all topics
///
/// This tool identifies currently trending news and topics in the cryptocurrency space,
/// useful for staying updated on breaking developments and market movements.
#[tool]
pub async fn get_trending_news(
    context: &riglr_core::provider::ApplicationContext,
    time_window: Option<String>,     // "1h", "6h", "24h"
    categories: Option<Vec<String>>, // "defi", "nft", "regulation", "tech"
    min_impact_score: Option<u32>,
    limit: Option<u32>,
) -> crate::error::Result<NewsAggregationResult> {
    debug!(
        "Fetching trending crypto news within {}",
        time_window.as_deref().unwrap_or("6h")
    );

    let config = NewsConfig::from_context(context);
    let client = WebClient::default();

    // Get trending articles from multiple sources
    let trending_articles = fetch_trending_articles(
        &client,
        &config,
        &time_window,
        &categories,
        min_impact_score.unwrap_or(60),
    )
    .await?;

    let articles: Vec<NewsArticle> = trending_articles
        .into_iter()
        .take(limit.unwrap_or(30) as usize)
        .collect();

    // Analyze trending patterns
    let insights = analyze_trending_patterns(&articles).await?;
    let trending_topics = extract_trending_topics(&articles).await?;

    let result = NewsAggregationResult {
        topic: "Trending".to_string(),
        articles: articles.clone(),
        metadata: AggregationMetadata {
            total_articles: articles.len() as u32,
            returned_articles: articles.len() as u32,
            sources_queried: vec!["Multiple".to_string()],
            avg_credibility: calculate_avg_credibility(&articles),
            time_range_hours: parse_time_window(&time_window.unwrap_or_else(|| "6h".to_string())),
            duplicates_removed: 0,
        },
        insights,
        trending_topics,
        aggregated_at: Utc::now(),
    };

    info!(
        "Trending news aggregation completed: {} trending articles",
        result.articles.len()
    );

    Ok(result)
}

/// Monitor for breaking news and generate real-time alerts
///
/// This tool continuously monitors news sources for breaking news
/// and generates alerts based on severity and market impact criteria.
#[tool]
pub async fn monitor_breaking_news(
    context: &riglr_core::provider::ApplicationContext,
    keywords: Vec<String>,
    severity_threshold: Option<String>, // "Critical", "High", "Medium"
    impact_threshold: Option<u32>,      // 0-100
    _alert_channels: Option<Vec<String>>, // "webhook", "email", "slack"
) -> crate::error::Result<Vec<BreakingNewsAlert>> {
    debug!("Monitoring breaking news for keywords: {:?}", keywords);

    let config = NewsConfig::from_context(context);
    let client = WebClient::default();

    let mut alerts = Vec::new();

    // Check each keyword for breaking news
    for keyword in keywords {
        let breaking_news_result = detect_breaking_news(&client, &config, &keyword).await;
        match breaking_news_result {
            Ok(mut keyword_alerts) => {
                alerts.append(&mut keyword_alerts);
            }
            Err(e) => {
                warn!("Failed to check breaking news for '{}': {}", keyword, e);
            }
        }
    }

    // Filter by severity and impact thresholds
    let severity_level = severity_threshold.unwrap_or_else(|| "Medium".to_string());
    let impact_level = impact_threshold.unwrap_or(60);

    alerts.retain(|alert| {
        is_above_severity_threshold(&alert.severity, &severity_level)
            && alert.estimated_impact.impact_score >= impact_level
    });

    info!(
        "Breaking news monitoring completed: {} alerts generated",
        alerts.len()
    );

    Ok(alerts)
}

/// Analyze market sentiment from recent news
///
/// This tool provides comprehensive sentiment analysis across recent news articles,
/// helping to gauge overall market mood and potential price impact.
#[tool]
pub async fn analyze_market_sentiment(
    context: &riglr_core::provider::ApplicationContext,
    time_window: Option<String>,       // "1h", "6h", "24h", "week"
    asset_filter: Option<Vec<String>>, // Specific cryptocurrencies to focus on
    _source_weights: Option<HashMap<String, f64>>, // Weight different sources
    _include_social: Option<bool>,
) -> crate::error::Result<NewsInsights> {
    debug!(
        "Analyzing market sentiment from news over {}",
        time_window.as_deref().unwrap_or("24h")
    );

    let _config = NewsConfig::from_context(context);
    let _client = WebClient::default();

    // Gather recent news for sentiment analysis
    let recent_news = if let Some(assets) = &asset_filter {
        let mut all_news = Vec::new();
        for asset in assets {
            let news_result = get_crypto_news(
                context,
                asset.clone(),
                time_window.clone(),
                None,
                Some(70),    // Higher credibility for sentiment analysis
                Some(false), // Don't need full analysis
            )
            .await;
            match news_result {
                Ok(result) => all_news.extend(result.articles),
                Err(e) => warn!("Failed to get news for {}: {}", asset, e),
            }
        }
        all_news
    } else {
        // Get general market news
        match get_trending_news(context, time_window, None, Some(50), Some(100)).await {
            Ok(result) => result.articles,
            Err(_) => vec![], // Fallback to empty if trending fails
        }
    };

    // Perform comprehensive sentiment analysis
    let insights = analyze_news_collection(&recent_news).await?;

    info!(
        "Market sentiment analysis completed from {} articles",
        recent_news.len()
    );

    Ok(insights)
}

/// Query NewsAPI for articles
async fn query_newsapi(
    client: &WebClient,
    config: &NewsConfig,
    topic: &str,
    time_window: &Option<String>,
) -> crate::error::Result<Vec<NewsArticle>> {
    // Build URL and params for NewsAPI /v2/everything
    let url = format!("{}/everything", config.base_url);
    let window = time_window
        .clone()
        .unwrap_or_else(|| format!("{}h", config.freshness_hours));
    let hours = parse_time_window(&window) as i64;
    let from = (Utc::now() - chrono::Duration::hours(hours)).to_rfc3339();

    let mut params = std::collections::HashMap::new();
    params.insert("q".to_string(), topic.to_string());
    params.insert("language".to_string(), "en".to_string());
    params.insert("sortBy".to_string(), "publishedAt".to_string());
    params.insert("from".to_string(), from);
    params.insert("pageSize".to_string(), config.max_articles.to_string());

    // NewsAPI supports header X-Api-Key or apiKey parameter; prefer header
    let mut headers = std::collections::HashMap::new();
    headers.insert("X-Api-Key".to_string(), config.newsapi_key.clone());

    let response_result = client
        .get_with_params_and_headers(&url, &params, headers)
        .await;
    let resp_text =
        response_result.map_err(|e| WebToolError::Api(format!("NewsAPI request failed: {}", e)))?;

    // Parse JSON and map to NewsArticle
    let json: serde_json::Value = serde_json::from_str(&resp_text)
        .map_err(|e| WebToolError::Parsing(format!("NewsAPI parse error: {}", e)))?;

    if let Some(status) = json.get("status").and_then(|s| s.as_str()) {
        if status != "ok" {
            let msg = json
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            return Err(WebToolError::Api(format!("NewsAPI error: {}", msg)));
        }
    }

    let mut articles_out: Vec<NewsArticle> = Vec::new();
    if let Some(arr) = json.get("articles").and_then(|a| a.as_array()) {
        for a in arr {
            let title = a
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let url = a
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if url.is_empty() || title.is_empty() {
                continue;
            }
            let published_at = a
                .get("publishedAt")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));
            let description = a
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let content = a
                .get("content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let source_obj = a.get("source").cloned().unwrap_or_default();
            let source = NewsSource {
                id: source_obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                name: source_obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("NewsAPI")
                    .to_string(),
                url: url.clone(),
                category: "Mainstream".to_string(),
                credibility_score: 75,
                accuracy_rating: None,
                bias_score: None,
                is_verified: true,
                logo_url: None,
            };
            // Basic classification heuristics
            let category = NewsCategory {
                primary: "News".to_string(),
                sub_category: None,
                tags: vec![topic.to_lowercase()],
                geographic_scope: vec!["Global".to_string()],
                target_audience: "Retail".to_string(),
            };
            // Perform real sentiment analysis on title and content
            let sentiment = analyze_sentiment(&title, &description, &content);

            // Calculate market impact based on sentiment and source credibility
            let market_impact = calculate_market_impact(&sentiment, &source, &category);

            // Extract entities from title and content
            let entities = extract_entities_from_text(&title, &description, &content, topic);
            let article = NewsArticle {
                id: format!("newsapi_{}_{}", published_at.timestamp(), hash64(&url)),
                title,
                url,
                description,
                content,
                published_at,
                source,
                category,
                sentiment,
                market_impact,
                entities,
                related_assets: vec![topic.to_lowercase()],
                quality_metrics: QualityMetrics {
                    overall_score: 70,
                    depth_score: 60,
                    factual_accuracy: 75,
                    writing_quality: 70,
                    citation_quality: 60,
                    uniqueness_score: 50,
                    reading_difficulty: 5,
                },
                social_metrics: None,
            };
            articles_out.push(article);
        }
    }
    Ok(articles_out)
}

/// Query CryptoPanic for crypto-specific news
async fn query_cryptopanic(
    client: &WebClient,
    config: &NewsConfig,
    topic: &str,
    time_window: &Option<String>,
) -> crate::error::Result<Vec<NewsArticle>> {
    let base = "https://cryptopanic.com/api/v1/posts";
    let window = time_window.clone().unwrap_or_else(|| "24h".to_string());
    let _hours = parse_time_window(&window);

    let mut params = std::collections::HashMap::new();
    params.insert("auth_token".to_string(), config.cryptopanic_key.clone());
    params.insert("kind".to_string(), "news".to_string());
    params.insert("currencies".to_string(), topic.to_string());
    params.insert("public".to_string(), "true".to_string());
    params.insert("filter".to_string(), "rising".to_string());

    let response_result = client.get_with_params(base, &params).await;
    let resp_text = response_result
        .map_err(|e| WebToolError::Api(format!("CryptoPanic request failed: {}", e)))?;

    let json: serde_json::Value = serde_json::from_str(&resp_text)
        .map_err(|e| WebToolError::Parsing(format!("CryptoPanic parse error: {}", e)))?;

    let mut articles_out = Vec::new();
    if let Some(results) = json.get("results").and_then(|v| v.as_array()) {
        for item in results {
            let title = item
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let url = item
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if url.is_empty() || title.is_empty() {
                continue;
            }
            let published_at = item
                .get("published_at")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));
            let domain = item.get("domain").and_then(|v| v.as_str()).unwrap_or("");
            let source_obj = item.get("source").cloned().unwrap_or_default();
            let source = NewsSource {
                id: source_obj
                    .get("domain")
                    .and_then(|v| v.as_str())
                    .unwrap_or(domain)
                    .to_string(),
                name: source_obj
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("CryptoPanic")
                    .to_string(),
                url: url.clone(),
                category: "Crypto".to_string(),
                credibility_score: 70,
                accuracy_rating: None,
                bias_score: None,
                is_verified: true,
                logo_url: None,
            };
            let category = NewsCategory {
                primary: "News".to_string(),
                sub_category: None,
                tags: vec![topic.to_lowercase()],
                geographic_scope: vec!["Global".to_string()],
                target_audience: "Crypto".to_string(),
            };
            let article = NewsArticle {
                id: format!("cryptopanic_{}_{}", published_at.timestamp(), hash64(&url)),
                title: title.clone(),
                url,
                description: None,
                content: None,
                published_at,
                source,
                category,
                sentiment: analyze_sentiment(&title, &None, &None),
                market_impact: calculate_market_impact_simple(&title),
                entities: extract_entities_from_text(&title, &None, &None, topic),
                related_assets: vec![topic.to_lowercase()],
                quality_metrics: QualityMetrics {
                    overall_score: 68,
                    depth_score: 55,
                    factual_accuracy: 70,
                    writing_quality: 65,
                    citation_quality: 55,
                    uniqueness_score: 50,
                    reading_difficulty: 5,
                },
                social_metrics: None,
            };
            articles_out.push(article);
        }
    }
    Ok(articles_out)
}

/// Remove duplicate articles based on content similarity
fn deduplicate_articles(articles: Vec<NewsArticle>) -> Vec<NewsArticle> {
    // In production, would use content similarity algorithms
    // For now, simple URL-based deduplication
    let mut seen_urls = std::collections::HashSet::new();
    articles
        .into_iter()
        .filter(|article| seen_urls.insert(article.url.clone()))
        .collect()
}

/// Analyze a collection of news articles for insights
async fn analyze_news_collection(articles: &[NewsArticle]) -> crate::error::Result<NewsInsights> {
    let overall_sentiment = articles
        .iter()
        .map(|a| a.sentiment.overall_score)
        .sum::<f64>()
        / articles.len() as f64;

    let mut entity_mentions: HashMap<String, (u32, f64)> = HashMap::new();
    let mut themes = Vec::new();
    let mut geo_distribution = HashMap::new();

    for article in articles {
        // Collect entity mentions
        for entity in &article.entities {
            let entry = entity_mentions
                .entry(entity.name.clone())
                .or_insert((0, 0.0));
            entry.0 += entity.mention_count;
            entry.1 += entity.sentiment.unwrap_or(0.0);
        }

        // Collect themes
        themes.extend(article.category.tags.clone());

        // Geographic distribution
        for geo in &article.category.geographic_scope {
            *geo_distribution.entry(geo.clone()).or_insert(0) += 1;
        }
    }

    let top_entities: Vec<EntityMention> = entity_mentions
        .into_iter()
        .map(|(name, (count, sentiment))| EntityMention {
            name: name.clone(),
            mention_count: count,
            avg_sentiment: sentiment / count as f64,
            entity_type: "Unknown".to_string(), // Would determine from context
            is_trending: count > 5,             // Simple trending threshold
        })
        .collect();

    // Analyze source diversity
    let unique_sources = articles
        .iter()
        .map(|a| &a.source.name)
        .collect::<std::collections::HashSet<_>>()
        .len() as u32;

    let source_diversity = SourceDiversity {
        unique_sources,
        source_types: HashMap::new(), // Would calculate from actual data
        geographic_sources: HashMap::new(),
        credibility_distribution: HashMap::new(),
    };

    Ok(NewsInsights {
        overall_sentiment,
        sentiment_trend: determine_sentiment_trend(articles),
        top_entities,
        dominant_themes: themes,
        geographic_distribution: geo_distribution,
        source_diversity,
        impact_distribution: HashMap::new(), // Would calculate impact distribution
    })
}

/// Extract trending topics from articles
async fn extract_trending_topics(
    articles: &[NewsArticle],
) -> crate::error::Result<Vec<TrendingTopic>> {
    let mut topic_counts: HashMap<String, u32> = HashMap::new();
    let mut topic_sentiments: HashMap<String, f64> = HashMap::new();

    for article in articles {
        for tag in &article.category.tags {
            *topic_counts.entry(tag.clone()).or_insert(0) += 1;
            *topic_sentiments.entry(tag.clone()).or_insert(0.0) += article.sentiment.overall_score;
        }
    }

    let trending_topics: Vec<TrendingTopic> = topic_counts
        .into_iter()
        .filter(|(_, count)| *count >= 3) // Minimum threshold for trending
        .map(|(topic, count)| TrendingTopic {
            topic: topic.clone(),
            article_count: count,
            velocity: count as f64 / 24.0, // Articles per hour (assuming 24h window)
            sentiment: topic_sentiments.get(&topic).unwrap_or(&0.0) / count as f64,
            related_keywords: vec![], // Would extract related keywords
            geographic_focus: vec!["Global".to_string()],
        })
        .collect();

    Ok(trending_topics)
}

/// Helper functions
fn calculate_avg_credibility(articles: &[NewsArticle]) -> f64 {
    if articles.is_empty() {
        return 0.0;
    }
    articles
        .iter()
        .map(|a| a.source.credibility_score as f64)
        .sum::<f64>()
        / articles.len() as f64
}

fn parse_time_window(window: &str) -> u32 {
    match window {
        "1h" => 1,
        "6h" => 6,
        "24h" => 24,
        "week" => 168,
        _ => 24,
    }
}

fn determine_sentiment_trend(articles: &[NewsArticle]) -> String {
    // Simple trend analysis - would be more sophisticated in production
    let avg_sentiment = articles
        .iter()
        .map(|a| a.sentiment.overall_score)
        .sum::<f64>()
        / articles.len() as f64;

    if avg_sentiment > 0.1 {
        "Improving".to_string()
    } else if avg_sentiment < -0.1 {
        "Declining".to_string()
    } else {
        "Stable".to_string()
    }
}

async fn fetch_trending_articles(
    client: &WebClient,
    config: &NewsConfig,
    time_window: &Option<String>,
    _categories: &Option<Vec<String>>,
    _min_impact_score: u32,
) -> crate::error::Result<Vec<NewsArticle>> {
    // Prefer CryptoPanic trending (rising) plus NewsAPI top-headlines as fallback
    let mut out: Vec<NewsArticle> = Vec::new();

    if !config.cryptopanic_key.is_empty() {
        let mut params = std::collections::HashMap::new();
        params.insert("auth_token".to_string(), config.cryptopanic_key.clone());
        params.insert("filter".to_string(), "rising".to_string());
        params.insert("kind".to_string(), "news".to_string());
        params.insert("public".to_string(), "true".to_string());
        if let Some(window) = time_window.as_ref() {
            let _ = window; // not directly supported
        }
        if let Ok(resp) = client
            .get_with_params("https://cryptopanic.com/api/v1/posts", &params)
            .await
        {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                if let Some(results) = json.get("results").and_then(|v| v.as_array()) {
                    for item in results {
                        let title = item
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let url = item
                            .get("url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if title.is_empty() || url.is_empty() {
                            continue;
                        }
                        let published_at = item
                            .get("published_at")
                            .and_then(|v| v.as_str())
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));
                        out.push(NewsArticle {
                            id: format!(
                                "cp_trending_{}_{}",
                                published_at.timestamp(),
                                hash64(&url)
                            ),
                            title: title.clone(),
                            url: url.clone(),
                            description: None,
                            content: None,
                            published_at,
                            source: NewsSource {
                                id: "cryptopanic".to_string(),
                                name: "CryptoPanic".to_string(),
                                url,
                                category: "Crypto".to_string(),
                                credibility_score: 70,
                                accuracy_rating: None,
                                bias_score: None,
                                is_verified: true,
                                logo_url: None,
                            },
                            category: NewsCategory {
                                primary: "Trending".to_string(),
                                sub_category: None,
                                tags: vec![],
                                geographic_scope: vec!["Global".to_string()],
                                target_audience: "Crypto".to_string(),
                            },
                            sentiment: analyze_sentiment(&title, &None, &None),
                            market_impact: calculate_market_impact_simple(&title),
                            entities: vec![],
                            related_assets: vec![],
                            quality_metrics: QualityMetrics {
                                overall_score: 65,
                                depth_score: 55,
                                factual_accuracy: 70,
                                writing_quality: 65,
                                citation_quality: 55,
                                uniqueness_score: 50,
                                reading_difficulty: 5,
                            },
                            social_metrics: None,
                        });
                    }
                }
            }
        }
    }

    // Fallback to NewsAPI top-headlines about crypto
    if out.is_empty() && !config.newsapi_key.is_empty() {
        let url = format!("{}/top-headlines", config.base_url);
        let mut params = std::collections::HashMap::new();
        params.insert("q".to_string(), "crypto OR bitcoin OR ethereum".to_string());
        params.insert("language".to_string(), "en".to_string());
        params.insert("pageSize".to_string(), "20".to_string());
        let mut headers = std::collections::HashMap::new();
        headers.insert("X-Api-Key".to_string(), config.newsapi_key.clone());
        if let Ok(resp) = client
            .get_with_params_and_headers(&url, &params, headers)
            .await
        {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                if let Some(arts) = json.get("articles").and_then(|v| v.as_array()) {
                    for a in arts {
                        let title = a
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let url = a
                            .get("url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if title.is_empty() || url.is_empty() {
                            continue;
                        }
                        let published_at = a
                            .get("publishedAt")
                            .and_then(|v| v.as_str())
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));
                        // Parse and analyze the article properly
                        let description = a
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let content = a
                            .get("content")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let source_name = a
                            .get("source")
                            .and_then(|o| o.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("NewsAPI");

                        let source = NewsSource {
                            id: format!("newsapi_{}", hash64(&url)),
                            name: source_name.to_string(),
                            url: url.clone(),
                            category: "Mainstream".to_string(),
                            credibility_score: 75,
                            accuracy_rating: None,
                            bias_score: None,
                            is_verified: true,
                            logo_url: None,
                        };

                        let category = NewsCategory {
                            primary: "Trending".to_string(),
                            sub_category: None,
                            tags: extract_tags_from_text(&title, &description),
                            geographic_scope: vec!["Global".to_string()],
                            target_audience: "Retail".to_string(),
                        };

                        out.push(NewsArticle {
                            id: format!(
                                "newsapi_trending_{}_{}",
                                published_at.timestamp(),
                                hash64(&url)
                            ),
                            title: title.clone(),
                            url,
                            description: description.clone(),
                            content: content.clone(),
                            published_at,
                            source,
                            category,
                            sentiment: analyze_sentiment(&title, &description, &content),
                            market_impact: calculate_market_impact_from_content(
                                &title,
                                &description,
                                &content,
                            ),
                            entities: extract_entities_from_text(
                                &title,
                                &description,
                                &content,
                                "crypto",
                            ),
                            related_assets: extract_crypto_mentions(&title, &description, &content),
                            quality_metrics: calculate_quality_metrics(
                                &title,
                                &description,
                                &content,
                                75,
                            ),
                            social_metrics: None,
                        });
                    }
                }
            }
        }
    }

    Ok(out)
}

fn hash64(s: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::default();
    s.hash(&mut hasher);
    hasher.finish()
}

async fn analyze_trending_patterns(articles: &[NewsArticle]) -> crate::error::Result<NewsInsights> {
    // Similar to analyze_news_collection but with trending-specific logic
    let insights_result = analyze_news_collection(articles).await;
    insights_result
}

async fn detect_breaking_news(
    client: &WebClient,
    config: &NewsConfig,
    keyword: &str,
) -> crate::error::Result<Vec<BreakingNewsAlert>> {
    // Heuristic: fetch very recent items and detect urgency keywords
    let mut alerts: Vec<BreakingNewsAlert> = Vec::new();

    let mut articles: Vec<NewsArticle> = Vec::new();
    if !config.newsapi_key.is_empty() {
        if let Ok(mut a) = query_newsapi(client, config, keyword, &Some("1h".to_string())).await {
            articles.append(&mut a);
        }
    }
    if !config.cryptopanic_key.is_empty() {
        if let Ok(mut a) = query_cryptopanic(client, config, keyword, &Some("1h".to_string())).await
        {
            articles.append(&mut a);
        }
    }

    // Filter very recent (<= 2h) and containing strong terms
    let urgent_terms = [
        "breaking",
        "urgent",
        "exploit",
        "hack",
        "outage",
        "halt",
        "SEC",
        "lawsuit",
        "bankrupt",
        "halted",
        "paused",
        "breach",
        "attack",
        "flash loan",
        "rug",
    ];
    let now = Utc::now();
    let mut grouped: Vec<NewsArticle> = Vec::new();
    for a in articles.into_iter() {
        if (now - a.published_at) <= chrono::Duration::hours(2) {
            let hay = format!(
                "{} {} {}",
                a.title,
                a.description.clone().unwrap_or_default(),
                a.url
            );
            if urgent_terms
                .iter()
                .any(|t| hay.to_lowercase().contains(&t.to_lowercase()))
            {
                grouped.push(a);
            }
        }
    }

    if !grouped.is_empty() {
        let est_impact = MarketImpact {
            impact_level: "High".to_string(),
            impact_score: 80,
            time_horizon: "Immediate".to_string(),
            affected_sectors: vec!["Crypto".to_string()],
            potential_price_impact: Some(5.0),
            historical_correlation: None,
            risk_factors: vec!["Volatility".to_string()],
        };
        let alert = BreakingNewsAlert {
            id: format!("breaking_{}_{}", keyword.to_lowercase(), now.timestamp()),
            severity: "High".to_string(),
            title: format!("Breaking: {} - {} items", keyword, grouped.len()),
            description: format!("Detected urgent developments related to '{}'.", keyword),
            articles: grouped,
            estimated_impact: est_impact,
            created_at: now,
            expires_at: Some(now + chrono::Duration::hours(4)),
        };
        alerts.push(alert);
    }

    Ok(alerts)
}

fn is_above_severity_threshold(current_severity: &str, threshold: &str) -> bool {
    let severity_order = ["Low", "Medium", "High", "Critical"];
    let current_index = severity_order
        .iter()
        .position(|&s| s == current_severity)
        .unwrap_or(0);
    let threshold_index = severity_order
        .iter()
        .position(|&s| s == threshold)
        .unwrap_or(1);
    current_index >= threshold_index
}

/// Get sentiment analyzer from ApplicationContext or create default
#[allow(dead_code)]
fn get_sentiment_analyzer(_context: &ApplicationContext) -> Arc<LexiconSentimentAnalyzer> {
    // For now, always return the default analyzer
    // In the future, this can be extended to get from context extensions
    Arc::new(LexiconSentimentAnalyzer::default())
}

/// Helper function for backward compatibility
fn analyze_sentiment(
    title: &str,
    description: &Option<String>,
    content: &Option<String>,
) -> NewsSentiment {
    // Use the default analyzer for now
    let analyzer = LexiconSentimentAnalyzer::default();
    analyzer.analyze_sentiment_impl(title, description.as_deref(), content.as_deref())
}

/// Calculate market impact using heuristic rules based on sentiment and source credibility
///
/// This is a simplified calculation based on keyword presence and sentiment scores.
/// For production use, consider training a model on historical market reactions.
fn calculate_market_impact(
    sentiment: &NewsSentiment,
    source: &NewsSource,
    category: &NewsCategory,
) -> MarketImpact {
    // Base impact score from sentiment magnitude and confidence
    let sentiment_impact = (sentiment.overall_score.abs() * 100.0 * sentiment.confidence) as u32;

    // Adjust for source credibility
    let credibility_factor = source.credibility_score as f64 / 100.0;
    let base_score = (sentiment_impact as f64 * credibility_factor) as u32;

    // Category-based adjustments
    let category_multiplier = match category.primary.as_str() {
        "Breaking" => 1.5,
        "Regulation" => 1.4,
        "Security" => 1.3,
        "Analysis" => 1.1,
        _ => 1.0,
    };

    let impact_score = ((base_score as f64 * category_multiplier).min(100.0)) as u32;

    // Determine impact level
    let impact_level = match impact_score {
        80..=100 => "Critical",
        60..=79 => "High",
        40..=59 => "Medium",
        20..=39 => "Low",
        _ => "Negligible",
    }
    .to_string();

    // Time horizon based on urgency and category
    let time_horizon = if sentiment.emotions.urgency > 0.7 || category.primary == "Breaking" {
        "Immediate"
    } else if impact_score > 60 {
        "Short-term"
    } else {
        "Medium-term"
    }
    .to_string();

    // Identify affected sectors based on tags and content
    let mut affected_sectors = Vec::new();
    if category.tags.iter().any(|t| t.contains("defi")) {
        affected_sectors.push("DeFi".to_string());
    }
    if category.tags.iter().any(|t| t.contains("nft")) {
        affected_sectors.push("NFT".to_string());
    }
    if category.tags.iter().any(|t| t.contains("exchange")) {
        affected_sectors.push("CEX".to_string());
    }
    if category.tags.iter().any(|t| t.contains("regulation")) {
        affected_sectors.push("Regulatory".to_string());
    }
    if affected_sectors.is_empty() {
        affected_sectors.push("General".to_string());
    }

    // Estimate potential price impact
    let potential_price_impact = if impact_score > 70 {
        Some((sentiment.overall_score * 10.0).abs())
    } else if impact_score > 50 {
        Some((sentiment.overall_score * 5.0).abs())
    } else {
        None
    };

    // Identify risk factors
    let mut risk_factors = Vec::new();
    if sentiment.emotions.uncertainty > 0.6 {
        risk_factors.push("High uncertainty".to_string());
    }
    if sentiment.emotions.fear > 0.6 {
        risk_factors.push("Market fear".to_string());
    }
    if category
        .tags
        .iter()
        .any(|t| t.contains("hack") || t.contains("exploit"))
    {
        risk_factors.push("Security breach".to_string());
    }
    if category.tags.iter().any(|t| t.contains("regulation")) {
        risk_factors.push("Regulatory risk".to_string());
    }

    MarketImpact {
        impact_level,
        impact_score,
        time_horizon,
        affected_sectors,
        potential_price_impact,
        historical_correlation: None, // Would need historical data
        risk_factors,
    }
}

/// Simpler market impact calculation for when we only have title
fn calculate_market_impact_simple(title: &str) -> MarketImpact {
    let title_lower = title.to_lowercase();

    // High impact keywords
    let high_impact = [
        "hack",
        "exploit",
        "sec",
        "ban",
        "crash",
        "surge",
        "partnership",
        "adoption",
    ];
    let medium_impact = ["update", "launch", "announce", "report", "analysis"];

    let (impact_level, impact_score) = if high_impact.iter().any(|k| title_lower.contains(k)) {
        ("High".to_string(), 70)
    } else if medium_impact.iter().any(|k| title_lower.contains(k)) {
        ("Medium".to_string(), 50)
    } else {
        ("Low".to_string(), 30)
    };

    MarketImpact {
        impact_level,
        impact_score,
        time_horizon: "Short-term".to_string(),
        affected_sectors: vec!["General".to_string()],
        potential_price_impact: None,
        historical_correlation: None,
        risk_factors: vec![],
    }
}

/// Extract entities from text using regex pattern matching
///
/// This is a heuristic approach that matches against predefined entity patterns.
/// Only recognizes entities in the hardcoded list. For production use, consider
/// integrating with a proper NER (Named Entity Recognition) model.
fn extract_entities_from_text(
    title: &str,
    description: &Option<String>,
    content: &Option<String>,
    default_topic: &str,
) -> Vec<NewsEntity> {
    let full_text = format!(
        "{} {} {}",
        title,
        description.as_deref().unwrap_or(""),
        content.as_deref().unwrap_or("")
    );

    let mut entities = Vec::new();
    let mut entity_map: HashMap<String, (String, u32)> = HashMap::new(); // name -> (type, count)

    // Cryptocurrency patterns
    let crypto_pattern = r"\b(Bitcoin|BTC|Ethereum|ETH|Solana|SOL|Cardano|ADA|Polkadot|DOT|Chainlink|LINK|Avalanche|AVAX|Polygon|MATIC|Arbitrum|ARB|Optimism|OP)\b";
    if let Ok(re) = Regex::new(crypto_pattern) {
        for cap in re.captures_iter(&full_text) {
            if let Some(matched) = cap.get(0) {
                let name = matched.as_str();
                let entry = entity_map
                    .entry(name.to_string())
                    .or_insert(("Cryptocurrency".to_string(), 0));
                entry.1 += 1;
            }
        }
    }

    // Company patterns
    let company_pattern = r"\b(Coinbase|Binance|Kraken|FTX|OpenSea|Uniswap|Aave|Compound|MakerDAO|Circle|Tether|Block\.one|ConsenSys|Ripple|Grayscale|MicroStrategy|Tesla|Square|PayPal)\b";
    if let Ok(re) = Regex::new(company_pattern) {
        for cap in re.captures_iter(&full_text) {
            if let Some(matched) = cap.get(0) {
                let name = matched.as_str();
                let entry = entity_map
                    .entry(name.to_string())
                    .or_insert(("Company".to_string(), 0));
                entry.1 += 1;
            }
        }
    }

    // Person patterns (common crypto figures)
    let person_pattern = r"\b(Vitalik Buterin|Satoshi Nakamoto|CZ|Changpeng Zhao|Sam Bankman-Fried|SBF|Michael Saylor|Elon Musk|Gary Gensler|Jerome Powell)\b";
    if let Ok(re) = Regex::new(person_pattern) {
        for cap in re.captures_iter(&full_text) {
            if let Some(matched) = cap.get(0) {
                let name = matched.as_str();
                let entry = entity_map
                    .entry(name.to_string())
                    .or_insert(("Person".to_string(), 0));
                entry.1 += 1;
            }
        }
    }

    // Protocol/Platform patterns
    let protocol_pattern =
        r"\b(DeFi|NFT|DAO|DEX|CEX|Layer 2|L2|zkSync|StarkNet|Lightning Network|Cosmos|IBC)\b";
    if let Ok(re) = Regex::new(protocol_pattern) {
        for cap in re.captures_iter(&full_text) {
            if let Some(matched) = cap.get(0) {
                let name = matched.as_str();
                let entry = entity_map
                    .entry(name.to_string())
                    .or_insert(("Protocol".to_string(), 0));
                entry.1 += 1;
            }
        }
    }

    // Convert map to entities vector
    for (name, (entity_type, count)) in entity_map {
        let relevance_score = (count as f64 / 10.0).min(1.0);
        entities.push(NewsEntity {
            name: name.clone(),
            entity_type,
            relevance_score,
            sentiment: None, // Would need entity-specific sentiment analysis
            mention_count: count,
            contexts: vec![], // Would need to extract surrounding context
        });
    }

    // Add default topic if no entities found
    if entities.is_empty() {
        entities.push(NewsEntity {
            name: default_topic.to_string(),
            entity_type: "Topic".to_string(),
            relevance_score: 0.5,
            sentiment: None,
            mention_count: 1,
            contexts: vec![],
        });
    }

    // Sort by relevance
    entities.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    entities
}

/// Extract tags from text
fn extract_tags_from_text(title: &str, description: &Option<String>) -> Vec<String> {
    let full_text = format!(
        "{} {}",
        title.to_lowercase(),
        description.as_deref().unwrap_or("").to_lowercase()
    );

    let mut tags = Vec::new();

    // Topic keywords to tags
    let tag_keywords = [
        ("defi", "defi"),
        ("nft", "nft"),
        ("metaverse", "metaverse"),
        ("web3", "web3"),
        ("layer 2", "layer2"),
        ("stablecoin", "stablecoin"),
        ("cbdc", "cbdc"),
        ("mining", "mining"),
        ("staking", "staking"),
        ("governance", "governance"),
        ("dao", "dao"),
        ("smart contract", "smart-contracts"),
        ("regulation", "regulation"),
        ("sec", "regulation"),
        ("hack", "security"),
        ("exploit", "security"),
        ("partnership", "partnership"),
        ("integration", "integration"),
        ("upgrade", "upgrade"),
        ("mainnet", "mainnet"),
        ("testnet", "testnet"),
    ];

    for (keyword, tag) in &tag_keywords {
        if full_text.contains(keyword) {
            tags.push(tag.to_string());
        }
    }

    // Remove duplicates
    tags.sort();
    tags.dedup();

    tags
}

/// Extract cryptocurrency mentions from text
fn extract_crypto_mentions(
    title: &str,
    description: &Option<String>,
    content: &Option<String>,
) -> Vec<String> {
    let full_text = format!(
        "{} {} {}",
        title.to_lowercase(),
        description.as_deref().unwrap_or("").to_lowercase(),
        content.as_deref().unwrap_or("").to_lowercase()
    );

    let mut cryptos = Vec::new();

    let crypto_list = [
        ("bitcoin", "bitcoin"),
        ("btc", "bitcoin"),
        ("ethereum", "ethereum"),
        ("eth", "ethereum"),
        ("solana", "solana"),
        ("sol", "solana"),
        ("cardano", "cardano"),
        ("ada", "cardano"),
        ("polkadot", "polkadot"),
        ("dot", "polkadot"),
        ("chainlink", "chainlink"),
        ("link", "chainlink"),
        ("avalanche", "avalanche"),
        ("avax", "avalanche"),
        ("polygon", "polygon"),
        ("matic", "polygon"),
        ("arbitrum", "arbitrum"),
        ("optimism", "optimism"),
        ("bnb", "bnb"),
        ("xrp", "xrp"),
        ("doge", "dogecoin"),
        ("shib", "shiba-inu"),
    ];

    for (keyword, crypto) in &crypto_list {
        if full_text.contains(keyword) && !cryptos.contains(&crypto.to_string()) {
            cryptos.push(crypto.to_string());
        }
    }

    cryptos
}

/// Calculate quality metrics for an article
fn calculate_quality_metrics(
    title: &str,
    description: &Option<String>,
    content: &Option<String>,
    source_credibility: u32,
) -> QualityMetrics {
    let has_description = description.is_some() && !description.as_ref().unwrap().is_empty();
    let _has_content = content.is_some() && !content.as_ref().unwrap().is_empty();

    // Content depth based on length and structure
    let content_length = content.as_ref().map_or(0, |c| c.len());
    let depth_score = if content_length > 2000 {
        85
    } else if content_length > 1000 {
        70
    } else if content_length > 500 {
        55
    } else if has_description {
        40
    } else {
        25
    };

    // Writing quality based on title and description
    let title_words = title.split_whitespace().count();
    let writing_quality = if title_words > 5 && title_words < 20 && has_description {
        75
    } else if title_words > 3 {
        65
    } else {
        50
    };

    // Citation quality (would need to detect citations in real implementation)
    let citation_quality = if content_length > 1000 { 60 } else { 40 };

    // Overall score
    let overall_score = ((source_credibility as f64 * 0.3)
        + (depth_score as f64 * 0.3)
        + (writing_quality as f64 * 0.2)
        + (citation_quality as f64 * 0.2)) as u32;

    QualityMetrics {
        overall_score,
        depth_score,
        factual_accuracy: source_credibility, // Use source credibility as proxy
        writing_quality,
        citation_quality,
        uniqueness_score: 50, // Would need deduplication analysis
        reading_difficulty: if content_length > 2000 { 7 } else { 5 },
    }
}

/// Calculate market impact from content analysis
fn calculate_market_impact_from_content(
    title: &str,
    description: &Option<String>,
    content: &Option<String>,
) -> MarketImpact {
    let sentiment = analyze_sentiment(title, description, content);
    let full_text = format!(
        "{} {} {}",
        title.to_lowercase(),
        description.as_deref().unwrap_or("").to_lowercase(),
        content.as_deref().unwrap_or("").to_lowercase()
    );

    // Check for high-impact keywords
    let critical_keywords = [
        "hack",
        "exploit",
        "bankrupt",
        "sec enforcement",
        "criminal",
        "fraud",
    ];
    let high_keywords = [
        "partnership",
        "adoption",
        "integration",
        "launch",
        "acquisition",
    ];
    let medium_keywords = ["update", "upgrade", "announce", "report", "analysis"];

    let has_critical = critical_keywords.iter().any(|k| full_text.contains(k));
    let has_high = high_keywords.iter().any(|k| full_text.contains(k));
    let has_medium = medium_keywords.iter().any(|k| full_text.contains(k));

    let (impact_level, base_score) = if has_critical {
        ("Critical", 85)
    } else if has_high {
        ("High", 70)
    } else if has_medium {
        ("Medium", 50)
    } else {
        ("Low", 30)
    };

    // Adjust score based on sentiment magnitude
    let impact_score =
        ((base_score as f64 * (1.0 + sentiment.overall_score.abs() * 0.3)) as u32).min(100);

    MarketImpact {
        impact_level: impact_level.to_string(),
        impact_score,
        time_horizon: if has_critical {
            "Immediate"
        } else {
            "Short-term"
        }
        .to_string(),
        affected_sectors: extract_affected_sectors(&full_text),
        potential_price_impact: if impact_score > 70 {
            Some((sentiment.overall_score * 7.5).abs())
        } else if impact_score > 50 {
            Some((sentiment.overall_score * 4.0).abs())
        } else {
            None
        },
        historical_correlation: None,
        risk_factors: extract_risk_factors(&full_text),
    }
}

/// Extract affected sectors from text
fn extract_affected_sectors(text: &str) -> Vec<String> {
    let mut sectors = Vec::new();

    let sector_keywords = [
        ("defi", "DeFi"),
        ("nft", "NFT"),
        ("exchange", "CEX"),
        ("dex", "DEX"),
        ("stablecoin", "Stablecoins"),
        ("mining", "Mining"),
        ("layer 2", "Layer2"),
        ("lending", "Lending"),
        ("derivatives", "Derivatives"),
        ("gamefi", "GameFi"),
        ("metaverse", "Metaverse"),
    ];

    for (keyword, sector) in &sector_keywords {
        if text.contains(keyword) && !sectors.contains(&sector.to_string()) {
            sectors.push(sector.to_string());
        }
    }

    if sectors.is_empty() {
        sectors.push("General".to_string());
    }

    sectors
}

/// Extract risk factors from text
fn extract_risk_factors(text: &str) -> Vec<String> {
    let mut risks = Vec::new();

    let risk_keywords = [
        ("regulation", "Regulatory uncertainty"),
        ("sec", "Regulatory action"),
        ("hack", "Security vulnerability"),
        ("exploit", "Protocol vulnerability"),
        ("volatile", "Market volatility"),
        ("uncertain", "Market uncertainty"),
        ("lawsuit", "Legal risk"),
        ("investigation", "Regulatory investigation"),
        ("liquidity", "Liquidity risk"),
        ("contagion", "Contagion risk"),
    ];

    for (keyword, risk) in &risk_keywords {
        if text.contains(keyword) && !risks.contains(&risk.to_string()) {
            risks.push(risk.to_string());
        }
    }

    risks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_news_config_default() {
        let config = NewsConfig::default();
        assert_eq!(config.base_url, "https://newsapi.org/v2");
        assert_eq!(config.max_articles, 50);
    }

    #[test]
    fn test_basic_news_functionality() {
        // Simple test that verifies basic functionality
        let simple_title = "Bitcoin News Test".to_string();
        assert!(simple_title.contains("Bitcoin"));

        // Test the NewsConfig creation
        let config = NewsConfig::default();
        assert_eq!(config.base_url, "https://newsapi.org/v2");
        assert_eq!(config.max_articles, 50);
    }

    #[test]
    fn test_parse_time_window() {
        assert_eq!(parse_time_window("1h"), 1);
        assert_eq!(parse_time_window("24h"), 24);
        assert_eq!(parse_time_window("week"), 168);
    }

    #[test]
    fn test_severity_threshold() {
        assert!(is_above_severity_threshold("High", "Medium"));
        assert!(!is_above_severity_threshold("Medium", "High"));
        assert!(is_above_severity_threshold("Critical", "High"));
    }

    // Additional comprehensive tests for 100% coverage

    #[test]
    fn test_parse_time_window_all_cases() {
        assert_eq!(parse_time_window("6h"), 6);
        assert_eq!(parse_time_window("invalid"), 24); // Default case
        assert_eq!(parse_time_window(""), 24); // Empty string
        assert_eq!(parse_time_window("random_text"), 24);
    }

    #[test]
    fn test_hash64_function() {
        let test_string = "test_string";
        let hash1 = hash64(test_string);
        let hash2 = hash64(test_string);
        assert_eq!(hash1, hash2); // Same input should produce same hash

        let different_hash = hash64("different_string");
        assert_ne!(hash1, different_hash); // Different inputs should produce different hashes

        let empty_hash = hash64("");
        assert_ne!(hash1, empty_hash); // Empty string should produce different hash
    }

    #[test]
    fn test_severity_threshold_edge_cases() {
        // Test all valid severity levels
        assert!(is_above_severity_threshold("Critical", "Critical"));
        assert!(is_above_severity_threshold("High", "High"));
        assert!(is_above_severity_threshold("Medium", "Medium"));
        assert!(is_above_severity_threshold("Low", "Low"));

        // Test invalid severities (should default to position 0 and 1)
        assert!(!is_above_severity_threshold("Invalid", "Medium"));
        assert!(is_above_severity_threshold("Medium", "Invalid"));
        assert!(is_above_severity_threshold("Invalid", "Invalid"));
    }

    #[test]
    fn test_calculate_avg_credibility() {
        // Test with empty articles
        let empty_articles: Vec<NewsArticle> = vec![];
        assert_eq!(calculate_avg_credibility(&empty_articles), 0.0);

        // Test with single article
        let single_article = vec![create_test_article_with_credibility(75)];
        assert_eq!(calculate_avg_credibility(&single_article), 75.0);

        // Test with multiple articles
        let multiple_articles = vec![
            create_test_article_with_credibility(80),
            create_test_article_with_credibility(60),
            create_test_article_with_credibility(70),
        ];
        assert_eq!(calculate_avg_credibility(&multiple_articles), 70.0);
    }

    #[test]
    fn test_deduplicate_articles() {
        // Test with no articles
        let empty_articles: Vec<NewsArticle> = vec![];
        let result = deduplicate_articles(empty_articles);
        assert!(result.is_empty());

        // Test with unique articles
        let unique_articles = vec![
            create_test_article_with_url("https://example1.com"),
            create_test_article_with_url("https://example2.com"),
        ];
        let result = deduplicate_articles(unique_articles);
        assert_eq!(result.len(), 2);

        // Test with duplicate URLs
        let duplicate_articles = vec![
            create_test_article_with_url("https://example.com"),
            create_test_article_with_url("https://example.com"),
            create_test_article_with_url("https://different.com"),
        ];
        let result = deduplicate_articles(duplicate_articles);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_determine_sentiment_trend() {
        // Test improving trend
        let improving_articles = vec![
            create_test_article_with_sentiment(0.5),
            create_test_article_with_sentiment(0.6),
        ];
        assert_eq!(determine_sentiment_trend(&improving_articles), "Improving");

        // Test declining trend
        let declining_articles = vec![
            create_test_article_with_sentiment(-0.5),
            create_test_article_with_sentiment(-0.6),
        ];
        assert_eq!(determine_sentiment_trend(&declining_articles), "Declining");

        // Test stable trend
        let stable_articles = vec![
            create_test_article_with_sentiment(0.05),
            create_test_article_with_sentiment(-0.05),
        ];
        assert_eq!(determine_sentiment_trend(&stable_articles), "Stable");

        // Test edge cases for thresholds
        let edge_positive = vec![create_test_article_with_sentiment(0.1)];
        assert_eq!(determine_sentiment_trend(&edge_positive), "Stable");

        let edge_negative = vec![create_test_article_with_sentiment(-0.1)];
        assert_eq!(determine_sentiment_trend(&edge_negative), "Stable");

        let just_above_positive = vec![create_test_article_with_sentiment(0.11)];
        assert_eq!(determine_sentiment_trend(&just_above_positive), "Improving");

        let just_below_negative = vec![create_test_article_with_sentiment(-0.11)];
        assert_eq!(determine_sentiment_trend(&just_below_negative), "Declining");
    }

    #[test]
    fn test_analyze_sentiment_comprehensive() {
        // Test positive sentiment
        let positive_sentiment = analyze_sentiment(
            "Bitcoin surge brings bullish sentiment to crypto markets",
            &Some("Strong gains and positive developments".to_string()),
            &Some("The rally continues with strong adoption and growth".to_string()),
        );
        assert!(positive_sentiment.overall_score > 0.0);
        assert_eq!(positive_sentiment.classification, "Bullish");
        assert!(positive_sentiment.confidence > 0.0);

        // Test negative sentiment
        let negative_sentiment = analyze_sentiment(
            "Bitcoin crash brings bearish sentiment and market fears",
            &Some("Major decline and concerns about future".to_string()),
            &Some("The drop causes risk and threats to vulnerable markets".to_string()),
        );
        assert!(negative_sentiment.overall_score < 0.0);
        assert_eq!(negative_sentiment.classification, "Bearish");

        // Test neutral sentiment
        let neutral_sentiment = analyze_sentiment(
            "Bitcoin price analysis report",
            &Some("Regular market update".to_string()),
            &None,
        );
        assert_eq!(neutral_sentiment.classification, "Neutral");

        // Test edge case classifications
        let slightly_bullish =
            analyze_sentiment("Bitcoin shows mild growth and positive signs", &None, &None);
        assert!(slightly_bullish.classification.contains("Bullish"));

        // Test emotional indicators
        let fear_content =
            analyze_sentiment("Market crash panic fear worried investors", &None, &None);
        assert!(fear_content.emotions.fear > 0.0);

        let greed_content = analyze_sentiment(
            "Moon lambo rich massive explosive gains profit",
            &None,
            &None,
        );
        assert!(greed_content.emotions.greed > 0.0);

        let uncertainty_content = analyze_sentiment(
            "Maybe perhaps unclear uncertain volatile unpredictable",
            &None,
            &None,
        );
        assert!(uncertainty_content.emotions.uncertainty > 0.0);

        // Test topic-specific sentiments
        let bitcoin_surge = analyze_sentiment("Bitcoin surge hits new highs", &None, &None);
        assert!(bitcoin_surge.topic_sentiments.contains_key("bitcoin"));
        assert!(bitcoin_surge.topic_sentiments["bitcoin"] > 0.0);

        let ethereum_crash = analyze_sentiment("Ethereum crash causes major losses", &None, &None);
        assert!(ethereum_crash.topic_sentiments.contains_key("ethereum"));
        assert!(ethereum_crash.topic_sentiments["ethereum"] < 0.0);
    }

    #[test]
    fn test_analyze_sentiment_edge_cases() {
        // Test empty content
        let empty_sentiment = analyze_sentiment("", &None, &None);
        assert_eq!(empty_sentiment.overall_score, 0.0);
        assert_eq!(empty_sentiment.classification, "Neutral");

        // Test content with no sentiment words
        let neutral_words = analyze_sentiment(
            "The weather is nice today",
            &Some("Random content".to_string()),
            &None,
        );
        assert_eq!(neutral_words.overall_score, 0.0);

        // Test very long content
        let long_content = "bullish ".repeat(100);
        let long_sentiment = analyze_sentiment(&long_content, &None, &None);
        assert!(long_sentiment.overall_score > 0.0);
        assert!(long_sentiment.confidence > 0.0);
    }

    #[test]
    fn test_calculate_market_impact() {
        let sentiment = NewsSentiment {
            overall_score: 0.5,
            confidence: 0.8,
            classification: "Bullish".to_string(),
            topic_sentiments: HashMap::new(),
            emotions: EmotionalIndicators {
                fear: 0.0,
                greed: 0.0,
                excitement: 0.0,
                uncertainty: 0.0,
                urgency: 0.9, // High urgency
            },
            key_phrases: vec![],
        };

        let source = NewsSource {
            id: "test".to_string(),
            name: "Test Source".to_string(),
            url: "https://test.com".to_string(),
            category: "Mainstream".to_string(),
            credibility_score: 80,
            accuracy_rating: None,
            bias_score: None,
            is_verified: true,
            logo_url: None,
        };

        // Test breaking news category with high urgency
        let breaking_category = NewsCategory {
            primary: "Breaking".to_string(),
            sub_category: None,
            tags: vec!["hack".to_string()],
            geographic_scope: vec!["Global".to_string()],
            target_audience: "Retail".to_string(),
        };

        let impact = calculate_market_impact(&sentiment, &source, &breaking_category);
        assert_eq!(impact.time_horizon, "Immediate");
        assert!(impact.impact_score > 0);
        assert!(impact.risk_factors.len() > 0);

        // Test different categories
        let regulation_category = NewsCategory {
            primary: "Regulation".to_string(),
            sub_category: None,
            tags: vec!["regulation".to_string()],
            geographic_scope: vec!["US".to_string()],
            target_audience: "Institutional".to_string(),
        };

        let reg_impact = calculate_market_impact(&sentiment, &source, &regulation_category);
        assert!(reg_impact
            .risk_factors
            .contains(&"Regulatory risk".to_string()));

        // Test with different sentiment emotions
        let fearful_sentiment = NewsSentiment {
            overall_score: -0.5,
            confidence: 0.8,
            classification: "Bearish".to_string(),
            topic_sentiments: HashMap::new(),
            emotions: EmotionalIndicators {
                fear: 0.8,
                greed: 0.0,
                excitement: 0.0,
                uncertainty: 0.7,
                urgency: 0.0,
            },
            key_phrases: vec![],
        };

        let fear_impact = calculate_market_impact(&fearful_sentiment, &source, &breaking_category);
        assert!(fear_impact
            .risk_factors
            .contains(&"Market fear".to_string()));
        assert!(fear_impact
            .risk_factors
            .contains(&"High uncertainty".to_string()));
    }

    #[test]
    fn test_calculate_market_impact_simple() {
        // Test high impact keywords
        let hack_impact = calculate_market_impact_simple("Major hack exploit discovered");
        assert_eq!(hack_impact.impact_level, "High");
        assert_eq!(hack_impact.impact_score, 70);

        let sec_impact = calculate_market_impact_simple("SEC announces new ban");
        assert_eq!(sec_impact.impact_level, "High");
        assert_eq!(sec_impact.impact_score, 70);

        // Test medium impact keywords
        let launch_impact = calculate_market_impact_simple("Company announces new launch");
        assert_eq!(launch_impact.impact_level, "Medium");
        assert_eq!(launch_impact.impact_score, 50);

        // Test low impact (no special keywords)
        let normal_impact = calculate_market_impact_simple("Regular news update");
        assert_eq!(normal_impact.impact_level, "Low");
        assert_eq!(normal_impact.impact_score, 30);

        // Test case insensitivity
        let case_impact = calculate_market_impact_simple("HACK discovered in PROTOCOL");
        assert_eq!(case_impact.impact_level, "High");
    }

    #[test]
    fn test_extract_entities_from_text() {
        // Test cryptocurrency entities
        let crypto_text = "Bitcoin and Ethereum are leading cryptocurrencies";
        let entities = extract_entities_from_text(
            crypto_text,
            &Some("BTC and ETH analysis".to_string()),
            &Some("Solana SOL also mentioned".to_string()),
            "crypto",
        );

        let crypto_names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert!(crypto_names.contains(&"Bitcoin"));
        assert!(crypto_names.contains(&"Ethereum"));
        assert!(crypto_names.contains(&"Solana"));

        // Test company entities
        let company_text = "Coinbase and Binance are major exchanges";
        let company_entities = extract_entities_from_text(company_text, &None, &None, "exchanges");

        let company_names: Vec<&str> = company_entities.iter().map(|e| e.name.as_str()).collect();
        assert!(company_names.contains(&"Coinbase"));
        assert!(company_names.contains(&"Binance"));

        // Test person entities
        let person_text = "Vitalik Buterin and CZ discussed the future";
        let person_entities = extract_entities_from_text(person_text, &None, &None, "crypto");

        let person_names: Vec<&str> = person_entities.iter().map(|e| e.name.as_str()).collect();
        assert!(person_names.contains(&"Vitalik Buterin"));
        assert!(person_names.contains(&"CZ"));

        // Test protocol entities
        let protocol_text = "DeFi and NFT protocols are growing";
        let protocol_entities =
            extract_entities_from_text(protocol_text, &None, &None, "protocols");

        let protocol_names: Vec<&str> = protocol_entities.iter().map(|e| e.name.as_str()).collect();
        assert!(protocol_names.contains(&"DeFi"));
        assert!(protocol_names.contains(&"NFT"));

        // Test default topic when no entities found
        let no_entities =
            extract_entities_from_text("Random news content", &None, &None, "default_topic");
        assert_eq!(no_entities.len(), 1);
        assert_eq!(no_entities[0].name, "default_topic");
        assert_eq!(no_entities[0].entity_type, "Topic");

        // Test sorting by relevance
        let multi_mention_text = "Bitcoin Bitcoin Bitcoin Ethereum";
        let sorted_entities =
            extract_entities_from_text(multi_mention_text, &None, &None, "crypto");
        // Bitcoin should be first due to higher mention count
        assert_eq!(sorted_entities[0].name, "Bitcoin");
        assert!(sorted_entities[0].mention_count > sorted_entities[1].mention_count);
    }

    #[test]
    fn test_extract_tags_from_text() {
        // Test various tag extractions
        let defi_text = "DeFi protocols are revolutionizing finance";
        let defi_tags = extract_tags_from_text(defi_text, &None);
        assert!(defi_tags.contains(&"defi".to_string()));

        let nft_text = "NFT marketplace sees growth";
        let nft_tags = extract_tags_from_text(nft_text, &Some("NFT content".to_string()));
        assert!(nft_tags.contains(&"nft".to_string()));

        let multi_tag_text = "Layer 2 solutions improve smart contract efficiency";
        let multi_tags = extract_tags_from_text(multi_tag_text, &None);
        assert!(multi_tags.contains(&"layer2".to_string()));
        assert!(multi_tags.contains(&"smart-contracts".to_string()));

        // Test case insensitivity
        let case_text = "DEFI and NFT protocols";
        let case_tags = extract_tags_from_text(case_text, &None);
        assert!(case_tags.contains(&"defi".to_string()));
        assert!(case_tags.contains(&"nft".to_string()));

        // Test no tags found
        let no_tag_text = "Random content without keywords";
        let no_tags = extract_tags_from_text(no_tag_text, &None);
        assert!(no_tags.is_empty());

        // Test duplicate removal
        let duplicate_text = "DeFi DeFi protocols and DeFi systems";
        let dup_tags = extract_tags_from_text(duplicate_text, &None);
        assert_eq!(dup_tags.iter().filter(|&t| t == "defi").count(), 1);
    }

    #[test]
    fn test_extract_crypto_mentions() {
        // Test various crypto mentions
        let crypto_text = "Bitcoin BTC Ethereum ETH prices";
        let cryptos = extract_crypto_mentions(
            crypto_text,
            &Some("Solana SOL analysis".to_string()),
            &Some("Cardano ADA update".to_string()),
        );

        assert!(cryptos.contains(&"bitcoin".to_string()));
        assert!(cryptos.contains(&"ethereum".to_string()));
        assert!(cryptos.contains(&"solana".to_string()));
        assert!(cryptos.contains(&"cardano".to_string()));

        // Test case insensitivity
        let case_text = "BITCOIN and ethereum prices";
        let case_cryptos = extract_crypto_mentions(case_text, &None, &None);
        assert!(case_cryptos.contains(&"bitcoin".to_string()));
        assert!(case_cryptos.contains(&"ethereum".to_string()));

        // Test symbol vs name deduplication
        let symbol_text = "BTC Bitcoin analysis";
        let symbol_cryptos = extract_crypto_mentions(symbol_text, &None, &None);
        assert_eq!(symbol_cryptos.iter().filter(|&c| c == "bitcoin").count(), 1);

        // Test no crypto mentions
        let no_crypto_text = "Weather news today";
        let no_cryptos = extract_crypto_mentions(no_crypto_text, &None, &None);
        assert!(no_cryptos.is_empty());
    }

    #[test]
    fn test_calculate_quality_metrics() {
        // Test high quality metrics
        let high_quality = calculate_quality_metrics(
            "Comprehensive Analysis of Market Trends",
            &Some("Detailed description of market conditions".to_string()),
            &Some("a".repeat(2500)), // Long content > 2000 chars
            90,                      // High source credibility
        );
        assert_eq!(high_quality.depth_score, 85);
        assert_eq!(high_quality.writing_quality, 75);
        assert_eq!(high_quality.factual_accuracy, 90);
        assert!(high_quality.overall_score > 70);

        // Test medium quality metrics
        let medium_quality = calculate_quality_metrics(
            "Market Update",
            &Some("Brief description".to_string()),
            &Some("a".repeat(1500)), // Medium content length
            70,
        );
        assert_eq!(medium_quality.depth_score, 70);

        // Test low quality metrics
        let low_quality = calculate_quality_metrics("News", &None, &None, 50);
        assert_eq!(low_quality.depth_score, 25);
        assert_eq!(low_quality.writing_quality, 50);

        // Test edge cases
        let edge_case = calculate_quality_metrics(
            "Short title with exactly five words here",
            &Some("Description present".to_string()),
            &Some("a".repeat(500)), // Exactly 500 chars
            75,
        );
        assert_eq!(edge_case.depth_score, 55);
        assert_eq!(edge_case.writing_quality, 75);
    }

    #[test]
    fn test_calculate_market_impact_from_content() {
        // Test critical impact keywords
        let critical_content = calculate_market_impact_from_content(
            "Major hack exploit discovered",
            &Some("Criminal fraud investigation".to_string()),
            &Some("SEC enforcement action bankruptcy".to_string()),
        );
        assert_eq!(critical_content.impact_level, "Critical");
        assert_eq!(critical_content.time_horizon, "Immediate");
        assert!(critical_content.impact_score >= 85);

        // Test high impact keywords
        let high_content = calculate_market_impact_from_content(
            "Partnership announcement",
            &Some("Major adoption integration".to_string()),
            &Some("New launch acquisition".to_string()),
        );
        assert_eq!(high_content.impact_level, "High");

        // Test medium impact keywords
        let medium_content = calculate_market_impact_from_content(
            "Update announcement",
            &Some("Upgrade report".to_string()),
            &Some("Analysis of trends".to_string()),
        );
        assert_eq!(medium_content.impact_level, "Medium");

        // Test low impact (no keywords)
        let low_content = calculate_market_impact_from_content(
            "Regular news",
            &Some("Standard content".to_string()),
            &None,
        );
        assert_eq!(low_content.impact_level, "Low");

        // Test sentiment impact on score
        let positive_sentiment_content = calculate_market_impact_from_content(
            "Partnership bullish surge rally",
            &Some("Strong positive growth".to_string()),
            &None,
        );
        assert!(positive_sentiment_content.impact_score > 70);

        // Test potential price impact calculation
        assert!(critical_content.potential_price_impact.is_some());
        assert!(high_content.potential_price_impact.is_some());
        assert!(low_content.potential_price_impact.is_none());
    }

    #[test]
    fn test_extract_affected_sectors() {
        // Test single sector
        let defi_text = "defi protocols are growing";
        let defi_sectors = extract_affected_sectors(defi_text);
        assert!(defi_sectors.contains(&"DeFi".to_string()));

        // Test multiple sectors
        let multi_text = "nft marketplace and exchange listing";
        let multi_sectors = extract_affected_sectors(multi_text);
        assert!(multi_sectors.contains(&"NFT".to_string()));
        assert!(multi_sectors.contains(&"CEX".to_string()));

        // Test no sectors (should default to General)
        let general_text = "random news content";
        let general_sectors = extract_affected_sectors(general_text);
        assert_eq!(general_sectors, vec!["General".to_string()]);

        // Test deduplication
        let dup_text = "defi and defi protocols";
        let dup_sectors = extract_affected_sectors(dup_text);
        assert_eq!(dup_sectors.iter().filter(|&s| s == "DeFi").count(), 1);
    }

    #[test]
    fn test_extract_risk_factors() {
        // Test various risk factors
        let risk_text = "regulation SEC hack exploit volatile uncertain lawsuit investigation";
        let risks = extract_risk_factors(risk_text);

        assert!(risks.contains(&"Regulatory uncertainty".to_string()));
        assert!(risks.contains(&"Regulatory action".to_string()));
        assert!(risks.contains(&"Security vulnerability".to_string()));
        assert!(risks.contains(&"Protocol vulnerability".to_string()));
        assert!(risks.contains(&"Market volatility".to_string()));
        assert!(risks.contains(&"Market uncertainty".to_string()));
        assert!(risks.contains(&"Legal risk".to_string()));
        assert!(risks.contains(&"Regulatory investigation".to_string()));

        // Test no risk factors
        let safe_text = "positive news about growth";
        let no_risks = extract_risk_factors(safe_text);
        assert!(no_risks.is_empty());

        // Test deduplication
        let dup_risk_text = "regulation and regulation concerns";
        let dup_risks = extract_risk_factors(dup_risk_text);
        assert_eq!(
            dup_risks
                .iter()
                .filter(|&r| r == "Regulatory uncertainty")
                .count(),
            1
        );
    }

    // Helper functions for creating test data
    fn create_test_article_with_credibility(credibility: u32) -> NewsArticle {
        NewsArticle {
            id: "test".to_string(),
            title: "Test Article".to_string(),
            url: "https://test.com".to_string(),
            description: None,
            content: None,
            published_at: Utc::now(),
            source: NewsSource {
                id: "test".to_string(),
                name: "Test Source".to_string(),
                url: "https://test.com".to_string(),
                category: "Test".to_string(),
                credibility_score: credibility,
                accuracy_rating: None,
                bias_score: None,
                is_verified: true,
                logo_url: None,
            },
            category: NewsCategory {
                primary: "Test".to_string(),
                sub_category: None,
                tags: vec![],
                geographic_scope: vec![],
                target_audience: "Test".to_string(),
            },
            sentiment: NewsSentiment {
                overall_score: 0.0,
                confidence: 0.5,
                classification: "Neutral".to_string(),
                topic_sentiments: HashMap::new(),
                emotions: EmotionalIndicators {
                    fear: 0.0,
                    greed: 0.0,
                    excitement: 0.0,
                    uncertainty: 0.0,
                    urgency: 0.0,
                },
                key_phrases: vec![],
            },
            market_impact: MarketImpact {
                impact_level: "Low".to_string(),
                impact_score: 30,
                time_horizon: "Short-term".to_string(),
                affected_sectors: vec![],
                potential_price_impact: None,
                historical_correlation: None,
                risk_factors: vec![],
            },
            entities: vec![],
            related_assets: vec![],
            quality_metrics: QualityMetrics {
                overall_score: 50,
                depth_score: 50,
                factual_accuracy: 50,
                writing_quality: 50,
                citation_quality: 50,
                uniqueness_score: 50,
                reading_difficulty: 5,
            },
            social_metrics: None,
        }
    }

    fn create_test_article_with_url(url: &str) -> NewsArticle {
        let mut article = create_test_article_with_credibility(70);
        article.url = url.to_string();
        article
    }

    fn create_test_article_with_sentiment(sentiment_score: f64) -> NewsArticle {
        let mut article = create_test_article_with_credibility(70);
        article.sentiment.overall_score = sentiment_score;
        article
    }
}
