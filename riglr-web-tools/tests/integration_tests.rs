//! Integration tests for riglr-web-tools
//!
//! These tests verify the functionality of web tools with basic functionality testing

//! These tests focus on structure validation and configuration testing

#[cfg(test)]
mod dexscreener_tests {
    use riglr_web_tools::dexscreener::{DexScreenerConfig, TokenInfo, ChainInfo, SecurityInfo};
    use chrono::Utc;

    #[test]
    fn test_dexscreener_config_creation() {
        let config = DexScreenerConfig::default();
        assert_eq!(config.base_url, "https://api.dexscreener.com/latest");
        assert_eq!(config.rate_limit_per_minute, 300);
        assert_eq!(config.request_timeout, 30);
    }

    #[test]
    fn test_token_info_structure() {
        let token_info = TokenInfo {
            address: "0x123".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            price_usd: Some(1.0),
            market_cap: Some(1000000.0),
            volume_24h: Some(50000.0),
            price_change_24h: Some(5.0),
            price_change_1h: Some(-1.0),
            price_change_5m: Some(0.5),
            circulating_supply: Some(1000000.0),
            total_supply: Some(10000000.0),
            pair_count: 1,
            pairs: vec![],
            chain: ChainInfo {
                id: "ethereum".to_string(),
                name: "Ethereum".to_string(),
                logo: None,
                native_token: "ETH".to_string(),
            },
            security: SecurityInfo {
                is_verified: true,
                liquidity_locked: Some(true),
                audit_status: None,
                honeypot_status: None,
                ownership_status: None,
                risk_score: Some(25),
            },
            socials: vec![],
            updated_at: Utc::now(),
        };

        assert_eq!(token_info.symbol, "TEST");
        assert_eq!(token_info.decimals, 18);
        assert!(token_info.security.is_verified);
    }
}

#[cfg(test)]
mod twitter_tests {
    use riglr_web_tools::twitter::{TwitterConfig, TwitterPost, TwitterUser, TweetMetrics, TweetEntities};
    use chrono::Utc;

    #[test]
    fn test_twitter_config_creation() {
        // Save original env var if it exists
        let original_token = std::env::var("TWITTER_BEARER_TOKEN").ok();
        
        // Test with environment variable
        std::env::set_var("TWITTER_BEARER_TOKEN", "test_token_123");
        let config = TwitterConfig::default();
        assert_eq!(config.bearer_token, "test_token_123");
        assert_eq!(config.base_url, "https://api.twitter.com/2");
        assert_eq!(config.max_results, 100);
        
        // Restore original env var
        if let Some(token) = original_token {
            std::env::set_var("TWITTER_BEARER_TOKEN", token);
        } else {
            std::env::remove_var("TWITTER_BEARER_TOKEN");
        }
    }

    #[test]
    fn test_twitter_post_structure() {
        let post = TwitterPost {
            id: "123456789".to_string(),
            text: "Test tweet content".to_string(),
            author: TwitterUser {
                id: "user123".to_string(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                description: Some("Test description".to_string()),
                followers_count: 1000,
                following_count: 500,
                tweet_count: 5000,
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
                mentions: vec!["@example".to_string()],
                urls: vec![],
                cashtags: vec!["$BTC".to_string()],
            },
            lang: Some("en".to_string()),
            is_reply: false,
            is_retweet: false,
            context_annotations: vec![],
        };

        assert_eq!(post.text, "Test tweet content");
        assert_eq!(post.author.username, "testuser");
        assert_eq!(post.metrics.like_count, 50);
        assert_eq!(post.entities.cashtags[0], "$BTC");
    }
}

#[cfg(test)]
mod news_tests {
    use riglr_web_tools::news::{NewsConfig, NewsSource};

    #[test]
    fn test_news_config_creation() {
        // Save original env vars
        let original_newsapi = std::env::var("NEWSAPI_KEY").ok();
        let original_cryptopanic = std::env::var("CRYPTOPANIC_KEY").ok();
        
        // Test with environment variables
        std::env::set_var("NEWSAPI_KEY", "test_news_key");
        std::env::set_var("CRYPTOPANIC_KEY", "test_crypto_key");
        
        let config = NewsConfig::default();
        assert_eq!(config.newsapi_key, "test_news_key");
        assert_eq!(config.cryptopanic_key, "test_crypto_key");
        assert_eq!(config.base_url, "https://newsapi.org/v2");
        assert_eq!(config.max_articles, 50);
        assert_eq!(config.freshness_hours, 24);
        
        // Restore original env vars
        if let Some(key) = original_newsapi {
            std::env::set_var("NEWSAPI_KEY", key);
        } else {
            std::env::remove_var("NEWSAPI_KEY");
        }
        if let Some(key) = original_cryptopanic {
            std::env::set_var("CRYPTOPANIC_KEY", key);
        } else {
            std::env::remove_var("CRYPTOPANIC_KEY");
        }
    }

    #[test]
    fn test_news_source_structure() {
        let source = NewsSource {
            id: "coindesk".to_string(),
            name: "CoinDesk".to_string(),
            url: "https://coindesk.com".to_string(),
            category: "Crypto-Native".to_string(),
            credibility_score: 85,
            accuracy_rating: Some(0.92),
            bias_score: Some(0.05),
            is_verified: true,
            logo_url: Some("https://coindesk.com/logo.png".to_string()),
        };

        assert_eq!(source.name, "CoinDesk");
        assert_eq!(source.credibility_score, 85);
        assert!(source.is_verified);
        assert!(source.accuracy_rating.is_some());
    }
}

#[cfg(test)]
mod web_search_tests {
    use riglr_web_tools::web_search::{WebSearchConfig, SearchResult, DomainInfo, PageMetadata, SocialMetadata, SeoMetadata, ContentType};
    use chrono::Utc;

    #[test]
    fn test_web_search_config_creation() {
        // Save original env var
        let original_key = std::env::var("EXA_API_KEY").ok();
        
        // Test with environment variable
        std::env::set_var("EXA_API_KEY", "test_exa_key");
        
        let config = WebSearchConfig::default();
        assert_eq!(config.exa_api_key, "test_exa_key");
        assert_eq!(config.exa_base_url, "https://api.exa.ai");
        assert_eq!(config.max_results, 20);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.include_content);
        
        // Restore original env var
        if let Some(key) = original_key {
            std::env::set_var("EXA_API_KEY", key);
        } else {
            std::env::remove_var("EXA_API_KEY");
        }
    }

    #[test]
    fn test_search_result_structure() {
        let result = SearchResult {
            id: "result_1".to_string(),
            title: "Test Article".to_string(),
            url: "https://example.com/article".to_string(),
            description: Some("Test description".to_string()),
            content: Some("Full article content...".to_string()),
            summary: Some("Article summary".to_string()),
            published_date: Some(Utc::now()),
            domain: DomainInfo {
                name: "example.com".to_string(),
                reputation_score: Some(85),
                category: Some("News".to_string()),
                is_trusted: true,
                authority_score: Some(75),
            },
            metadata: PageMetadata {
                author: Some("Jane Doe".to_string()),
                tags: vec!["tech".to_string(), "news".to_string()],
                social_meta: SocialMetadata {
                    og_title: Some("Test Article".to_string()),
                    og_description: Some("Test description".to_string()),
                    og_image: None,
                    twitter_card: Some("summary".to_string()),
                    twitter_site: None,
                },
                seo_meta: SeoMetadata {
                    meta_description: Some("Meta description".to_string()),
                    meta_keywords: vec!["keyword1".to_string()],
                    robots: Some("index,follow".to_string()),
                    schema_types: vec!["Article".to_string()],
                },
                canonical_url: None,
                last_modified: Some(Utc::now()),
            },
            relevance_score: 0.95,
            content_type: ContentType {
                primary: "Article".to_string(),
                format: "HTML".to_string(),
                is_paywalled: Some(false),
                quality_score: Some(90),
                length_category: "Medium".to_string(),
            },
            language: Some("en".to_string()),
            reading_time_minutes: Some(5),
        };

        assert_eq!(result.title, "Test Article");
        assert_eq!(result.domain.name, "example.com");
        assert!(result.domain.is_trusted);
        assert_eq!(result.relevance_score, 0.95);
        assert_eq!(result.content_type.primary, "Article");
    }
}

#[cfg(test)]
mod error_handling_tests {
    use riglr_web_tools::error::{WebToolError, Result};

    #[test]
    fn test_web_tool_error_types() {
        let auth_error = WebToolError::Auth("Missing API key".to_string());
        let network_error = WebToolError::Network("Connection failed".to_string());
        let api_error = WebToolError::Api("Invalid response".to_string());
        let config_error = WebToolError::Config("Invalid configuration".to_string());
        let client_error = WebToolError::Client("Client creation failed".to_string());

        // Test error display
        assert!(auth_error.to_string().contains("Missing API key"));
        assert!(network_error.to_string().contains("Connection failed"));
        assert!(api_error.to_string().contains("Invalid response"));
        assert!(config_error.to_string().contains("Invalid configuration"));
        assert!(client_error.to_string().contains("Client creation failed"));
    }

    #[test]
    fn test_error_result_handling() {
        let success_result: Result<String> = Ok("success".to_string());
        let error_result: Result<String> = Err(WebToolError::Auth("Failed".to_string()));

        assert!(success_result.is_ok());
        assert!(error_result.is_err());
        if let Ok(value) = success_result {
            assert_eq!(value, "success");
        }
        if let Err(err) = error_result {
            assert!(err.to_string().contains("Failed"));
        }
    }
}
