//! # riglr-web-tools
//!
//! Web-based data tools for riglr agents, providing access to social media, market data,
//! and web search capabilities.
//!
//! This crate bridges the gap between on-chain data and off-chain information sources,
//! enabling AI agents to gather comprehensive market intelligence and social sentiment.
//!
//! ## Features
//!
//! - **Social Media Tools**: Twitter/X integration for sentiment analysis
//! - **Market Data Tools**: DexScreener integration for token metrics
//! - **Web Search Tools**: Exa API integration for intelligent web search
//! - **Rate Limiting**: Built-in rate limiting and API quota management
//! - **Caching**: Optional response caching to improve performance
//!
//! ## Quick Start
//!
//! ```ignore
//! // Example usage (requires rig-core dependency):
//! use riglr_web_tools::twitter::search_tweets;
//! use rig_core::Agent;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let agent = Agent::builder()
//!     .preamble("You are a market sentiment analyst.")
//!     .tool(search_tweets)
//!     .build();
//!
//! let response = agent.prompt("What's the current sentiment on Twitter about $SOL?").await?;
//! println!("Agent response: {}", response);
//! # Ok(())
//! # }
//! ```
//!
//! ## API Configuration
//!
//! Most tools require API keys. Set the following environment variables:
//!
//! - `TWITTER_BEARER_TOKEN` - For Twitter API access
//! - `EXA_API_KEY` - For Exa web search
//! - `DEXSCREENER_API_KEY` - For DexScreener (if required)
//! - `LUNARCRUSH_API_KEY` - For LunarCrush social analytics
//! - `FASTER100X_API_KEY` - For Faster100x holder analysis
//!
//! ## Tool Categories
//!
//! - [`twitter`] - Twitter/X integration for social sentiment
//! - [`dexscreener`] - Token market data and trading metrics
//! - [`web_search`] - Intelligent web search capabilities
//! - [`news`] - Cryptocurrency news aggregation
//! - [`lunarcrush`] - LunarCrush social analytics and sentiment tracking
//! - [`faster100x`] - Token holder analysis and whale activity tracking

pub mod client;
pub mod dexscreener;
pub mod dexscreener_api;
pub mod error;
pub mod faster100x;
pub mod lunarcrush;
pub mod news;
pub mod price;
pub mod twitter;
pub mod web_search;

// Re-export commonly used tools - be selective to avoid name conflicts
// From dexscreener
pub use dexscreener::{
    analyze_token_market, get_token_info, get_top_pairs, get_trending_tokens, search_tokens,
    ChainInfo, MarketAnalysis, TokenInfo, TokenPair,
};

// From news
pub use news::{
    analyze_market_sentiment, get_crypto_news, get_trending_news, monitor_breaking_news,
    NewsAggregationResult, NewsArticle, NewsSource,
};

// From twitter
pub use twitter::{
    analyze_crypto_sentiment, get_user_tweets, search_tweets, SentimentAnalysis,
    SentimentBreakdown, TwitterPost, TwitterSearchResult, TwitterUser,
};

// From web_search
pub use web_search::{
    find_similar_pages, search_recent_news, search_web, summarize_web_content, ContentSummary,
    SearchResult, WebSearchResult,
};

// From lunarcrush
pub use lunarcrush::{
    get_influencer_mentions, get_social_sentiment, get_trending_cryptos, InfluencerMention,
    InfluencerMentionsResult, SentimentData, TrendingCrypto,
};

// From faster100x
pub use faster100x::{
    analyze_token_holders, get_holder_trends, get_whale_activity, ConcentrationRisk, HolderTrends,
    TokenHolderAnalysis, WalletHolding, WhaleActivity,
};

// From price
pub use price::{get_token_price, get_token_prices_batch, TokenPriceResult};

// Re-export client and error types
pub use client::WebClient;
pub use error::{Result, WebToolError};

/// Current version of riglr-web-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_when_valid_should_start_with_semver_digit() {
        // Happy Path: VERSION should be a valid semver
        assert!(
            VERSION.starts_with("0.") || VERSION.starts_with("1."),
            "VERSION should be a valid semver"
        );
    }

    #[test]
    fn test_version_when_called_should_not_be_empty() {
        // Edge Case: VERSION should not be empty
        assert!(!VERSION.is_empty(), "VERSION should not be empty");
    }

    #[test]
    fn test_version_when_called_should_contain_dots() {
        // Edge Case: VERSION should contain dots for semver format
        assert!(
            VERSION.contains('.'),
            "VERSION should contain dots for semver format"
        );
    }

    #[test]
    fn test_version_when_called_should_be_valid_utf8() {
        // Edge Case: VERSION should be valid UTF-8
        assert!(VERSION.is_ascii(), "VERSION should be valid ASCII");
    }

    #[test]
    fn test_version_when_called_should_match_cargo_version() {
        // Integration test: VERSION should match what's in Cargo.toml
        let version = env!("CARGO_PKG_VERSION");
        assert_eq!(VERSION, version, "VERSION should match CARGO_PKG_VERSION");
    }

    #[test]
    fn test_version_when_parsed_should_have_major_minor_patch() {
        // Edge Case: VERSION should have at least major.minor format
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(
            parts.len() >= 2,
            "VERSION should have at least major.minor format"
        );

        // Verify major version is numeric
        assert!(
            parts[0].parse::<u32>().is_ok(),
            "Major version should be numeric"
        );

        // Verify minor version is numeric (may contain pre-release info)
        let minor_part = parts[1].split('-').next().unwrap_or(parts[1]);
        assert!(
            minor_part.parse::<u32>().is_ok(),
            "Minor version should be numeric"
        );
    }

    #[test]
    fn test_module_re_exports_are_accessible() {
        // Test that re-exported types are accessible
        // This ensures the pub use statements work correctly

        // Test a few key re-exports from different modules
        use crate::{Result, WebClient, WebToolError, VERSION};

        // These should compile without issues, proving the re-exports work
        let _version: &str = VERSION;
        let _error_type = std::marker::PhantomData::<WebToolError>;
        let _result_type = std::marker::PhantomData::<Result<()>>;
        let _client_type = std::marker::PhantomData::<WebClient>;
    }

    #[test]
    fn test_dexscreener_re_exports_are_accessible() {
        // Test dexscreener re-exports
        use crate::{ChainInfo, MarketAnalysis, TokenInfo, TokenPair};

        let _chain_info_type = std::marker::PhantomData::<ChainInfo>;
        let _market_analysis_type = std::marker::PhantomData::<MarketAnalysis>;
        let _token_info_type = std::marker::PhantomData::<TokenInfo>;
        let _token_pair_type = std::marker::PhantomData::<TokenPair>;
    }

    #[test]
    fn test_news_re_exports_are_accessible() {
        // Test news re-exports
        use crate::{NewsAggregationResult, NewsArticle, NewsSource};

        let _news_aggregation_type = std::marker::PhantomData::<NewsAggregationResult>;
        let _news_article_type = std::marker::PhantomData::<NewsArticle>;
        let _news_source_type = std::marker::PhantomData::<NewsSource>;
    }

    #[test]
    fn test_twitter_re_exports_are_accessible() {
        // Test twitter re-exports
        use crate::{
            SentimentAnalysis, SentimentBreakdown, TwitterPost, TwitterSearchResult, TwitterUser,
        };

        let _sentiment_analysis_type = std::marker::PhantomData::<SentimentAnalysis>;
        let _sentiment_breakdown_type = std::marker::PhantomData::<SentimentBreakdown>;
        let _twitter_post_type = std::marker::PhantomData::<TwitterPost>;
        let _twitter_search_result_type = std::marker::PhantomData::<TwitterSearchResult>;
        let _twitter_user_type = std::marker::PhantomData::<TwitterUser>;
    }

    #[test]
    fn test_web_search_re_exports_are_accessible() {
        // Test web_search re-exports
        use crate::{ContentSummary, SearchResult, WebSearchResult};

        let _content_summary_type = std::marker::PhantomData::<ContentSummary>;
        let _search_result_type = std::marker::PhantomData::<SearchResult>;
        let _web_search_result_type = std::marker::PhantomData::<WebSearchResult>;
    }

    #[test]
    fn test_lunarcrush_re_exports_are_accessible() {
        // Test lunarcrush re-exports
        use crate::{InfluencerMention, InfluencerMentionsResult, SentimentData, TrendingCrypto};

        let _influencer_mention_type = std::marker::PhantomData::<InfluencerMention>;
        let _influencer_mentions_result_type = std::marker::PhantomData::<InfluencerMentionsResult>;
        let _sentiment_data_type = std::marker::PhantomData::<SentimentData>;
        let _trending_crypto_type = std::marker::PhantomData::<TrendingCrypto>;
    }

    #[test]
    fn test_faster100x_re_exports_are_accessible() {
        // Test faster100x re-exports
        use crate::{
            ConcentrationRisk, HolderTrends, TokenHolderAnalysis, WalletHolding, WhaleActivity,
        };

        let _concentration_risk_type = std::marker::PhantomData::<ConcentrationRisk>;
        let _holder_trends_type = std::marker::PhantomData::<HolderTrends>;
        let _token_holder_analysis_type = std::marker::PhantomData::<TokenHolderAnalysis>;
        let _wallet_holding_type = std::marker::PhantomData::<WalletHolding>;
        let _whale_activity_type = std::marker::PhantomData::<WhaleActivity>;
    }

    #[test]
    fn test_price_re_exports_are_accessible() {
        // Test price re-exports
        use crate::TokenPriceResult;

        let _token_price_result_type = std::marker::PhantomData::<TokenPriceResult>;
    }

    #[test]
    fn test_all_function_re_exports_are_accessible() {
        // Test that function re-exports are accessible (compile-time check)
        use crate::{
            analyze_crypto_sentiment, analyze_market_sentiment, analyze_token_holders,
            analyze_token_market, find_similar_pages, get_crypto_news, get_holder_trends,
            get_influencer_mentions, get_social_sentiment, get_token_info, get_token_price,
            get_token_prices_batch, get_top_pairs, get_trending_cryptos, get_trending_news,
            get_trending_tokens, get_user_tweets, get_whale_activity, monitor_breaking_news,
            search_recent_news, search_tokens, search_tweets, search_web, summarize_web_content,
        };

        // These functions are async and return impl Future, so we can't cast them to simple function pointers.
        // Instead, we verify they exist by referencing them as function items
        let _analyze_token_market = analyze_token_market;
        let _get_token_info = get_token_info;
        let _get_top_pairs = get_top_pairs;
        let _get_trending_tokens = get_trending_tokens;
        let _search_tokens = search_tokens;

        let _analyze_market_sentiment = analyze_market_sentiment;
        let _get_crypto_news = get_crypto_news;
        let _get_trending_news = get_trending_news;
        let _monitor_breaking_news = monitor_breaking_news;

        let _analyze_crypto_sentiment = analyze_crypto_sentiment;
        let _get_user_tweets = get_user_tweets;
        let _search_tweets = search_tweets;

        let _find_similar_pages = find_similar_pages;
        let _search_recent_news = search_recent_news;
        let _search_web = search_web;
        let _summarize_web_content = summarize_web_content;

        let _get_influencer_mentions = get_influencer_mentions;
        let _get_social_sentiment = get_social_sentiment;
        let _get_trending_cryptos = get_trending_cryptos;

        let _analyze_token_holders = analyze_token_holders;
        let _get_holder_trends = get_holder_trends;
        let _get_whale_activity = get_whale_activity;

        let _get_token_price = get_token_price;
        let _get_token_prices_batch = get_token_prices_batch;
    }

    #[test]
    fn test_version_constant_is_static() {
        // Test that VERSION is a static string reference
        let version_ref: &'static str = VERSION;
        assert!(!version_ref.is_empty(), "VERSION should not be empty");
    }

    #[test]
    fn test_module_declarations_are_public() {
        // This test verifies that all the modules are properly declared as public
        // by attempting to access module paths (compile-time verification)

        // Test that all modules can be referenced
        let _client_mod = crate::client::WebClient::default;
        let _dexscreener_mod = crate::dexscreener::get_token_info;
        // dexscreener_api module contains utility functions and types, not a main struct
        let _error_mod = std::marker::PhantomData::<crate::error::WebToolError>;
        let _faster100x_mod = crate::faster100x::analyze_token_holders;
        let _lunarcrush_mod = crate::lunarcrush::get_social_sentiment;
        let _news_mod = crate::news::get_crypto_news;
        let _price_mod = crate::price::get_token_price;
        let _twitter_mod = crate::twitter::search_tweets;
        let _web_search_mod = crate::web_search::search_web;
    }
}
