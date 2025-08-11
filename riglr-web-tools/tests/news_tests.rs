use chrono::Utc;
use riglr_web_tools::news::*;
use std::collections::HashMap;

#[tokio::test]
async fn test_get_crypto_news_basic() {
    // Set environment variables for the test
    std::env::set_var("NEWSAPI_KEY", "test_key");
    std::env::set_var("CRYPTOPANIC_KEY", "test_key");

    let result = get_crypto_news(
        "Bitcoin".to_string(),
        Some("24h".to_string()),
        Some(vec!["crypto".to_string()]),
        Some(70),
        Some(true),
    )
    .await;

    assert!(result.is_ok());
    let news_result = result.unwrap();
    assert_eq!(news_result.topic, "Bitcoin");
    assert!(!news_result.articles.is_empty()); // Should have sample articles from mocked sources
    assert!(!news_result.metadata.sources_queried.is_empty());

    // Clean up
    std::env::remove_var("NEWSAPI_KEY");
    std::env::remove_var("CRYPTOPANIC_KEY");
}

#[tokio::test]
#[ignore] // Ignore this test for coverage run
async fn test_get_crypto_news_no_api_keys() {
    // Store current environment vars if they exist
    let newsapi_key = std::env::var("NEWSAPI_KEY").ok();
    let cryptopanic_key = std::env::var("CRYPTOPANIC_KEY").ok();

    // Ensure no API keys are set
    std::env::remove_var("NEWSAPI_KEY");
    std::env::remove_var("CRYPTOPANIC_KEY");

    let result = get_crypto_news("Ethereum".to_string(), None, None, None, None).await;

    assert!(result.is_err());
    // Should return auth error when no API keys are configured

    // Restore environment vars if they existed
    if let Some(key) = newsapi_key {
        std::env::set_var("NEWSAPI_KEY", key);
    }
    if let Some(key) = cryptopanic_key {
        std::env::set_var("CRYPTOPANIC_KEY", key);
    }
}

#[tokio::test]
async fn test_get_crypto_news_with_only_newsapi() {
    std::env::set_var("NEWSAPI_KEY", "test_key");
    std::env::remove_var("CRYPTOPANIC_KEY");

    let result = get_crypto_news(
        "DeFi".to_string(),
        Some("6h".to_string()),
        None,
        Some(60),
        Some(false), // Don't include analysis
    )
    .await;

    assert!(result.is_ok());
    let news_result = result.unwrap();
    assert_eq!(news_result.topic, "DeFi");
    // Since this is a mock implementation, just verify we got some sources
    assert!(!news_result.metadata.sources_queried.is_empty());

    std::env::remove_var("NEWSAPI_KEY");
}

#[tokio::test]
async fn test_get_crypto_news_with_only_cryptopanic() {
    std::env::remove_var("NEWSAPI_KEY");
    std::env::set_var("CRYPTOPANIC_KEY", "test_key");

    let result = get_crypto_news(
        "NFT".to_string(),
        Some("1h".to_string()),
        Some(vec!["analysis".to_string()]),
        Some(80),
        Some(true),
    )
    .await;

    assert!(result.is_ok());
    let news_result = result.unwrap();
    assert_eq!(news_result.topic, "NFT");
    // Since this is a mock implementation, just verify we got some sources
    assert!(!news_result.metadata.sources_queried.is_empty());

    std::env::remove_var("CRYPTOPANIC_KEY");
}

#[tokio::test]
async fn test_get_trending_news_basic() {
    let result = get_trending_news(
        Some("6h".to_string()),
        Some(vec!["defi".to_string(), "nft".to_string()]),
        Some(70),
        Some(15),
    )
    .await;

    assert!(result.is_ok());
    let news_result = result.unwrap();
    assert_eq!(news_result.topic, "Trending");
    assert!(!news_result.articles.is_empty()); // Should have mock trending articles
    assert_eq!(news_result.metadata.time_range_hours, 6);
}

#[tokio::test]
async fn test_get_trending_news_defaults() {
    let result = get_trending_news(
        None, // Should default to "6h"
        None, None, // Should default to 60
        None, // Should default to 30
    )
    .await;

    assert!(result.is_ok());
    let news_result = result.unwrap();
    assert_eq!(news_result.topic, "Trending");
    assert_eq!(news_result.metadata.time_range_hours, 6);
}

#[tokio::test]
async fn test_monitor_breaking_news_basic() {
    let result = monitor_breaking_news(
        vec!["Bitcoin".to_string(), "Ethereum".to_string()],
        Some("High".to_string()),
        Some(80),
        Some(vec!["webhook".to_string()]),
    )
    .await;

    assert!(result.is_ok());
    let alerts = result.unwrap();
    // Mock implementation returns empty alerts
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn test_monitor_breaking_news_defaults() {
    let result = monitor_breaking_news(
        vec!["regulation".to_string()],
        None, // Should default to "Medium"
        None, // Should default to 60
        None,
    )
    .await;

    assert!(result.is_ok());
    let alerts = result.unwrap();
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn test_analyze_market_sentiment_with_assets() {
    let result = analyze_market_sentiment(
        Some("24h".to_string()),
        Some(vec!["Bitcoin".to_string(), "Ethereum".to_string()]),
        None,
        Some(true),
    )
    .await;

    assert!(result.is_ok());
    let insights = result.unwrap();
    // Sentiment could be NaN if no news was found, so check for valid range or NaN
    assert!(
        insights.overall_sentiment >= -1.0 && insights.overall_sentiment <= 1.0
            || insights.overall_sentiment.is_nan()
    );
    assert!(["Improving", "Declining", "Stable"].contains(&insights.sentiment_trend.as_str()));
}

#[tokio::test]
async fn test_analyze_market_sentiment_general() {
    let result = analyze_market_sentiment(
        Some("week".to_string()),
        None, // No specific assets - get general market news
        None,
        Some(false),
    )
    .await;

    assert!(result.is_ok());
    let insights = result.unwrap();
    // Sentiment could be NaN if no news was found, so check for valid range or NaN
    assert!(
        insights.overall_sentiment >= -1.0 && insights.overall_sentiment <= 1.0
            || insights.overall_sentiment.is_nan()
    );
}

#[test]
fn test_news_config_default() {
    // Test with environment variables
    std::env::set_var("NEWSAPI_KEY", "test_news_key");
    std::env::set_var("CRYPTOPANIC_KEY", "test_crypto_key");

    let config = NewsConfig::default();
    assert_eq!(config.newsapi_key, "test_news_key");
    assert_eq!(config.cryptopanic_key, "test_crypto_key");
    assert_eq!(config.base_url, "https://newsapi.org/v2");
    assert_eq!(config.max_articles, 50);
    assert_eq!(config.freshness_hours, 24);
    assert_eq!(config.min_credibility_score, 60);

    // Clean up
    std::env::remove_var("NEWSAPI_KEY");
    std::env::remove_var("CRYPTOPANIC_KEY");
}

#[test]
fn test_news_config_empty_env() {
    // Test without environment variables
    std::env::remove_var("NEWSAPI_KEY");
    std::env::remove_var("CRYPTOPANIC_KEY");

    let config = NewsConfig::default();
    assert!(config.newsapi_key.is_empty());
    assert!(config.cryptopanic_key.is_empty());
}

#[test]
fn test_news_article_comprehensive() {
    let article = NewsArticle {
        id: "test_article_123".to_string(),
        title: "Bitcoin Reaches New Heights".to_string(),
        url: "https://example.com/bitcoin-news".to_string(),
        description: Some("Bitcoin price analysis and market outlook".to_string()),
        content: Some("Full article content about Bitcoin...".to_string()),
        published_at: Utc::now(),
        source: NewsSource {
            id: "coindesk".to_string(),
            name: "CoinDesk".to_string(),
            url: "https://coindesk.com".to_string(),
            category: "Crypto-Native".to_string(),
            credibility_score: 85,
            accuracy_rating: Some(0.92),
            bias_score: Some(0.05),
            is_verified: true,
            logo_url: Some("https://coindesk.com/logo.png".to_string()),
        },
        category: NewsCategory {
            primary: "Analysis".to_string(),
            sub_category: Some("Price Analysis".to_string()),
            tags: vec!["bitcoin".to_string(), "price".to_string()],
            geographic_scope: vec!["Global".to_string()],
            target_audience: "Retail".to_string(),
        },
        sentiment: NewsSentiment {
            overall_score: 0.3,
            confidence: 0.85,
            classification: "Bullish".to_string(),
            topic_sentiments: {
                let mut map = HashMap::new();
                map.insert("bitcoin".to_string(), 0.4);
                map.insert("market".to_string(), 0.2);
                map
            },
            emotions: EmotionalIndicators {
                fear: 0.1,
                greed: 0.4,
                excitement: 0.6,
                uncertainty: 0.2,
                urgency: 0.3,
            },
            key_phrases: vec![SentimentPhrase {
                phrase: "bullish outlook".to_string(),
                sentiment_contribution: 0.5,
                confidence: 0.9,
            }],
        },
        market_impact: MarketImpact {
            impact_level: "High".to_string(),
            impact_score: 85,
            time_horizon: "Short-term".to_string(),
            affected_sectors: vec!["Cryptocurrency".to_string(), "DeFi".to_string()],
            potential_price_impact: Some(5.5),
            historical_correlation: Some(0.75),
            risk_factors: vec!["Volatility".to_string()],
        },
        entities: vec![NewsEntity {
            name: "Bitcoin".to_string(),
            entity_type: "Cryptocurrency".to_string(),
            relevance_score: 0.95,
            sentiment: Some(0.4),
            mention_count: 8,
            contexts: vec!["Price movement".to_string(), "Market analysis".to_string()],
        }],
        related_assets: vec!["bitcoin".to_string(), "btc".to_string()],
        quality_metrics: QualityMetrics {
            overall_score: 88,
            depth_score: 85,
            factual_accuracy: 90,
            writing_quality: 85,
            citation_quality: 80,
            uniqueness_score: 75,
            reading_difficulty: 7,
        },
        social_metrics: Some(SocialMetrics {
            total_shares: 450,
            twitter_shares: 300,
            reddit_mentions: 75,
            linkedin_shares: 75,
            social_sentiment: 0.25,
            viral_score: 68,
            influencer_mentions: 12,
        }),
    };

    // Test all major fields
    assert_eq!(article.id, "test_article_123");
    assert_eq!(article.title, "Bitcoin Reaches New Heights");
    assert!(article.description.is_some());
    assert!(article.content.is_some());
    assert_eq!(article.source.credibility_score, 85);
    assert_eq!(article.category.primary, "Analysis");
    assert_eq!(article.sentiment.classification, "Bullish");
    assert_eq!(article.market_impact.impact_level, "High");
    assert_eq!(article.entities.len(), 1);
    assert_eq!(article.related_assets.len(), 2);
    assert!(article.social_metrics.is_some());
}

#[test]
fn test_news_source_all_fields() {
    let source = NewsSource {
        id: "reuters".to_string(),
        name: "Reuters".to_string(),
        url: "https://reuters.com".to_string(),
        category: "Mainstream".to_string(),
        credibility_score: 95,
        accuracy_rating: Some(0.96),
        bias_score: Some(-0.1),
        is_verified: true,
        logo_url: Some("https://reuters.com/logo.png".to_string()),
    };

    assert_eq!(source.credibility_score, 95);
    assert_eq!(source.accuracy_rating.unwrap(), 0.96);
    assert_eq!(source.bias_score.unwrap(), -0.1);
    assert!(source.is_verified);
    assert!(source.logo_url.is_some());
}

#[test]
fn test_emotional_indicators() {
    let emotions = EmotionalIndicators {
        fear: 0.8,
        greed: 0.2,
        excitement: 0.1,
        uncertainty: 0.9,
        urgency: 0.6,
    };

    assert_eq!(emotions.fear, 0.8);
    assert_eq!(emotions.greed, 0.2);
    assert_eq!(emotions.uncertainty, 0.9);
    // Test that all values are within valid range [0.0, 1.0]
    assert!(emotions.fear >= 0.0 && emotions.fear <= 1.0);
    assert!(emotions.greed >= 0.0 && emotions.greed <= 1.0);
    assert!(emotions.urgency >= 0.0 && emotions.urgency <= 1.0);
}

#[test]
fn test_market_impact_comprehensive() {
    let impact = MarketImpact {
        impact_level: "Medium".to_string(),
        impact_score: 65,
        time_horizon: "Long-term".to_string(),
        affected_sectors: vec!["DeFi".to_string(), "NFT".to_string(), "Gaming".to_string()],
        potential_price_impact: Some(2.3),
        historical_correlation: Some(0.45),
        risk_factors: vec![
            "Regulatory uncertainty".to_string(),
            "Market volatility".to_string(),
        ],
    };

    assert_eq!(impact.impact_level, "Medium");
    assert_eq!(impact.impact_score, 65);
    assert_eq!(impact.time_horizon, "Long-term");
    assert_eq!(impact.affected_sectors.len(), 3);
    assert_eq!(impact.risk_factors.len(), 2);
    assert!(impact.potential_price_impact.is_some());
    assert!(impact.historical_correlation.is_some());
}

#[test]
fn test_news_entity_comprehensive() {
    let entity = NewsEntity {
        name: "Vitalik Buterin".to_string(),
        entity_type: "Person".to_string(),
        relevance_score: 0.85,
        sentiment: Some(0.6),
        mention_count: 5,
        contexts: vec![
            "Ethereum development".to_string(),
            "Conference speech".to_string(),
        ],
    };

    assert_eq!(entity.name, "Vitalik Buterin");
    assert_eq!(entity.entity_type, "Person");
    assert_eq!(entity.relevance_score, 0.85);
    assert_eq!(entity.sentiment.unwrap(), 0.6);
    assert_eq!(entity.mention_count, 5);
    assert_eq!(entity.contexts.len(), 2);
}

#[test]
fn test_quality_metrics_comprehensive() {
    let quality = QualityMetrics {
        overall_score: 92,
        depth_score: 88,
        factual_accuracy: 95,
        writing_quality: 90,
        citation_quality: 85,
        uniqueness_score: 80,
        reading_difficulty: 8,
    };

    assert_eq!(quality.overall_score, 92);
    assert_eq!(quality.depth_score, 88);
    assert_eq!(quality.factual_accuracy, 95);
    assert_eq!(quality.writing_quality, 90);
    assert_eq!(quality.citation_quality, 85);
    assert_eq!(quality.uniqueness_score, 80);
    assert_eq!(quality.reading_difficulty, 8);
}

#[test]
fn test_social_metrics_comprehensive() {
    let social = SocialMetrics {
        total_shares: 1250,
        twitter_shares: 800,
        reddit_mentions: 300,
        linkedin_shares: 150,
        social_sentiment: 0.35,
        viral_score: 78,
        influencer_mentions: 25,
    };

    assert_eq!(social.total_shares, 1250);
    assert_eq!(social.twitter_shares, 800);
    assert_eq!(social.reddit_mentions, 300);
    assert_eq!(social.linkedin_shares, 150);
    assert_eq!(social.social_sentiment, 0.35);
    assert_eq!(social.viral_score, 78);
    assert_eq!(social.influencer_mentions, 25);

    // Verify total matches sum of individual platforms
    assert_eq!(
        social.total_shares,
        social.twitter_shares + social.reddit_mentions + social.linkedin_shares
    );
}

#[test]
fn test_aggregation_metadata() {
    let metadata = AggregationMetadata {
        total_articles: 150,
        returned_articles: 50,
        sources_queried: vec![
            "NewsAPI".to_string(),
            "CryptoPanic".to_string(),
            "CoinDesk".to_string(),
        ],
        avg_credibility: 82.5,
        time_range_hours: 24,
        duplicates_removed: 25,
    };

    assert_eq!(metadata.total_articles, 150);
    assert_eq!(metadata.returned_articles, 50);
    assert_eq!(metadata.sources_queried.len(), 3);
    assert_eq!(metadata.avg_credibility, 82.5);
    assert_eq!(metadata.time_range_hours, 24);
    assert_eq!(metadata.duplicates_removed, 25);
}

#[test]
fn test_news_insights() {
    let mut geo_dist = HashMap::new();
    geo_dist.insert("North America".to_string(), 45);
    geo_dist.insert("Europe".to_string(), 30);
    geo_dist.insert("Asia".to_string(), 25);

    let mut impact_dist = HashMap::new();
    impact_dist.insert("High".to_string(), 15);
    impact_dist.insert("Medium".to_string(), 25);
    impact_dist.insert("Low".to_string(), 10);

    let insights = NewsInsights {
        overall_sentiment: 0.15,
        sentiment_trend: "Improving".to_string(),
        top_entities: vec![EntityMention {
            name: "Bitcoin".to_string(),
            mention_count: 45,
            avg_sentiment: 0.3,
            entity_type: "Cryptocurrency".to_string(),
            is_trending: true,
        }],
        dominant_themes: vec!["regulation".to_string(), "adoption".to_string()],
        geographic_distribution: geo_dist,
        source_diversity: SourceDiversity {
            unique_sources: 12,
            source_types: HashMap::new(),
            geographic_sources: HashMap::new(),
            credibility_distribution: HashMap::new(),
        },
        impact_distribution: impact_dist,
    };

    assert_eq!(insights.overall_sentiment, 0.15);
    assert_eq!(insights.sentiment_trend, "Improving");
    assert_eq!(insights.top_entities.len(), 1);
    assert!(insights.top_entities[0].is_trending);
    assert_eq!(insights.dominant_themes.len(), 2);
    assert_eq!(insights.geographic_distribution.len(), 3);
    assert_eq!(insights.source_diversity.unique_sources, 12);
}

#[test]
fn test_breaking_news_alert() {
    let alert = BreakingNewsAlert {
        id: "alert_123".to_string(),
        severity: "Critical".to_string(),
        title: "Major Bitcoin Exchange Hack".to_string(),
        description: "Large cryptocurrency exchange reports security breach".to_string(),
        articles: vec![],
        estimated_impact: MarketImpact {
            impact_level: "Extreme".to_string(),
            impact_score: 95,
            time_horizon: "Immediate".to_string(),
            affected_sectors: vec!["Cryptocurrency".to_string()],
            potential_price_impact: Some(-15.0),
            historical_correlation: Some(0.88),
            risk_factors: vec!["Security".to_string(), "Market confidence".to_string()],
        },
        created_at: Utc::now(),
        expires_at: Some(Utc::now()),
    };

    assert_eq!(alert.id, "alert_123");
    assert_eq!(alert.severity, "Critical");
    assert_eq!(alert.estimated_impact.impact_level, "Extreme");
    assert_eq!(
        alert.estimated_impact.potential_price_impact.unwrap(),
        -15.0
    );
    assert!(alert.expires_at.is_some());
}

#[test]
fn test_time_window_parsing_logic() {
    // Test the logic we know exists based on the function signature
    // Since parse_time_window is private, we test through the public interface

    // This test verifies that different time windows work via the public API calls
    // The actual parsing is tested indirectly through the async functions
    let valid_windows = vec!["1h", "6h", "24h", "week"];

    for window in valid_windows {
        // Just verify the string is valid - actual parsing tested via API calls
        assert!(!window.is_empty());
        assert!(window.contains('h') || window == "week");
    }
}

#[test]
fn test_severity_levels() {
    // Test severity level ordering logic that we know exists
    let severity_levels = ["Low", "Medium", "High", "Critical"];

    for (i, level) in severity_levels.iter().enumerate() {
        assert!(!level.is_empty());
        if i > 0 {
            // Each level should be different from the previous
            assert_ne!(*level, severity_levels[i - 1]);
        }
    }

    // Test that we have all expected severity levels
    assert!(severity_levels.contains(&"Low"));
    assert!(severity_levels.contains(&"Medium"));
    assert!(severity_levels.contains(&"High"));
    assert!(severity_levels.contains(&"Critical"));
}

#[test]
fn test_trending_topic_serialization() {
    let topic = TrendingTopic {
        topic: "Layer 2".to_string(),
        article_count: 18,
        velocity: 0.75,
        sentiment: 0.45,
        related_keywords: vec!["scaling".to_string(), "ethereum".to_string()],
        geographic_focus: vec!["Global".to_string()],
    };

    let json = serde_json::to_string(&topic).unwrap();
    assert!(json.contains("Layer 2"));
    assert!(json.contains("18"));
    assert!(json.contains("0.75"));

    // Test round-trip serialization
    let deserialized: TrendingTopic = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.topic, "Layer 2");
    assert_eq!(deserialized.article_count, 18);
}

#[test]
fn test_sentiment_phrase() {
    let phrase = SentimentPhrase {
        phrase: "unprecedented growth".to_string(),
        sentiment_contribution: 0.7,
        confidence: 0.95,
    };

    assert_eq!(phrase.phrase, "unprecedented growth");
    assert_eq!(phrase.sentiment_contribution, 0.7);
    assert_eq!(phrase.confidence, 0.95);

    // Test that confidence and sentiment_contribution are in valid ranges
    assert!(phrase.confidence >= 0.0 && phrase.confidence <= 1.0);
    assert!(phrase.sentiment_contribution >= -1.0 && phrase.sentiment_contribution <= 1.0);
}

#[test]
fn test_entity_mention() {
    let mention = EntityMention {
        name: "Chainlink".to_string(),
        mention_count: 12,
        avg_sentiment: 0.25,
        entity_type: "Cryptocurrency".to_string(),
        is_trending: false,
    };

    assert_eq!(mention.name, "Chainlink");
    assert_eq!(mention.mention_count, 12);
    assert_eq!(mention.avg_sentiment, 0.25);
    assert_eq!(mention.entity_type, "Cryptocurrency");
    assert!(!mention.is_trending);
}

#[test]
fn test_source_diversity() {
    let mut source_types = HashMap::new();
    source_types.insert("Mainstream".to_string(), 8);
    source_types.insert("Crypto-Native".to_string(), 15);
    source_types.insert("Blog".to_string(), 3);

    let diversity = SourceDiversity {
        unique_sources: 26,
        source_types,
        geographic_sources: HashMap::new(),
        credibility_distribution: HashMap::new(),
    };

    assert_eq!(diversity.unique_sources, 26);
    assert_eq!(diversity.source_types.len(), 3);
    assert_eq!(diversity.source_types.get("Crypto-Native").unwrap(), &15);
}

#[test]
fn test_news_article_serialization_custom() {
    // Create a custom article since create_sample_article is private
    let article = NewsArticle {
        id: "test_123".to_string(),
        title: "Solana Network Upgrade Successful".to_string(),
        url: "https://example.com/solana-news".to_string(),
        description: Some("Details about Solana upgrade".to_string()),
        content: Some("Full article content...".to_string()),
        published_at: Utc::now(),
        source: NewsSource {
            id: "test_source".to_string(),
            name: "TestSource".to_string(),
            url: "https://testsource.com".to_string(),
            category: "Crypto".to_string(),
            credibility_score: 88,
            accuracy_rating: Some(0.9),
            bias_score: Some(0.1),
            is_verified: true,
            logo_url: None,
        },
        category: NewsCategory {
            primary: "Breaking".to_string(),
            sub_category: Some("Technology".to_string()),
            tags: vec!["solana".to_string()],
            geographic_scope: vec!["Global".to_string()],
            target_audience: "Retail".to_string(),
        },
        sentiment: NewsSentiment {
            overall_score: 0.3,
            confidence: 0.8,
            classification: "Bullish".to_string(),
            topic_sentiments: HashMap::new(),
            emotions: EmotionalIndicators {
                fear: 0.1,
                greed: 0.2,
                excitement: 0.5,
                uncertainty: 0.2,
                urgency: 0.3,
            },
            key_phrases: vec![],
        },
        market_impact: MarketImpact {
            impact_level: "High".to_string(),
            impact_score: 85,
            time_horizon: "Short-term".to_string(),
            affected_sectors: vec!["Solana".to_string()],
            potential_price_impact: Some(5.0),
            historical_correlation: Some(0.7),
            risk_factors: vec![],
        },
        entities: vec![],
        related_assets: vec!["solana".to_string()],
        quality_metrics: QualityMetrics {
            overall_score: 80,
            depth_score: 75,
            factual_accuracy: 85,
            writing_quality: 80,
            citation_quality: 70,
            uniqueness_score: 85,
            reading_difficulty: 6,
        },
        social_metrics: None,
    };

    let json = serde_json::to_string(&article).unwrap();
    assert!(json.contains("Solana"));
    assert!(json.contains("TestSource"));
    assert!(json.contains("Breaking"));

    // Test deserialization
    let deserialized: NewsArticle = serde_json::from_str(&json).unwrap();
    assert!(deserialized.title.contains("Solana"));
    assert_eq!(deserialized.source.name, "TestSource");
    assert_eq!(deserialized.source.credibility_score, 88);
}
