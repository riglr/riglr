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
//! - **Market Data Tools**: `DexScreener` integration for token metrics
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
//! - `DEXSCREENER_API_KEY` - For `DexScreener` (if required)
//! - `LUNARCRUSH_API_KEY` - For `LunarCrush` social analytics
//! - `FASTER100X_API_KEY` - For Faster100x holder analysis
//!
//! ## Tool Categories
//!
//! - [`twitter`] - Twitter/X integration for social sentiment
//! - [`dexscreener`] - Token market data and trading metrics
//! - [`web_search`] - Intelligent web search capabilities
//! - [`news`] - Cryptocurrency news aggregation
//! - [`lunarcrush`] - `LunarCrush` social analytics and sentiment tracking
//! - [`faster100x`] - Token holder analysis and whale activity tracking
//! - [`rugcheck`] - Solana token security analysis and rug pull detection
//! - [`trenchbot`] - Solana token bundle analysis and sniper detection
//! - [`pocketuniverse`] - Solana token rug pull detection based on wallet history
//! - [`tweetscout`] - Twitter/X account credibility scoring and social network analysis

pub mod client;
pub mod dexscreener;
pub mod dexscreener_api;
pub mod error;
pub mod faster100x;
pub mod lunarcrush;
pub mod news;
pub mod pocketuniverse;
pub mod price;
pub mod rugcheck;
pub mod trenchbot;
pub mod tweetscout;
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
    analyze_market_sentiment, get_crypto_articles, get_trending_articles, monitor_breaking_alerts,
    AggregationResult, Article, LexiconSentimentAnalyzer, Sentiment, SentimentAnalyzer, Source,
};

// From twitter
pub use twitter::{
    analyze_crypto_sentiment, get_user_tweets, search_tweets, SentimentAnalysis,
    SentimentBreakdown, TwitterPost, TwitterSearchResult, TwitterUser,
};

// From web_search
pub use web_search::{
    find_similar_pages, search_recent_news, search_web, summarize_web_content, ContentSummary,
    SearchOperation, SearchResult,
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

// From rugcheck
pub use rugcheck::{
    analyze_token_risks, check_if_rugged, get_token_report, RiskAnalysis, RiskLevel,
    RugCheckResult, TokenCheck, TokenHolder as RugCheckTokenHolder,
};

// From trenchbot
pub use trenchbot::{
    analyze_creator_risk, analyze_token_bundles, check_bundle_risk, get_bundle_info,
    BundleAnalysisResult, BundleResponse, BundleRiskCheck, CreatorAnalysisResult,
};

// From pocketuniverse
pub use pocketuniverse::{
    analyze_rug_risk, check_rug_pull, check_rug_pull_raw, is_token_safe, DetailedRugAnalysis,
    RugApiResponse, RugCheckResult as PocketUniverseRugCheck, SafetyCheck,
};

// From tweetscout
pub use tweetscout::{
    analyze_account, analyze_social_network, get_account_info, get_account_score,
    get_top_followers, get_top_friends, is_account_credible, AccountAnalysis, AccountInfo,
    CredibilityCheck, SocialNetworkAnalysis,
};

// Re-export client and error types
pub use client::WebClient;
pub use error::{Result, WebToolError};

/// Current version of riglr-web-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;
    use core::{marker::PhantomData, result::Result};

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
        // Note: This test is primarily for documentation/consistency - VERSION from env!() is never empty
        #[expect(clippy::const_is_empty)]
        {
            assert!(!VERSION.is_empty(), "VERSION should not be empty");
        }
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
            parts.first().unwrap().parse::<u32>().is_ok(),
            "Major version should be numeric"
        );

        // Verify minor version is numeric (may contain pre-release info)
        let minor_part = parts
            .get(1)
            .unwrap()
            .split('-')
            .next()
            .unwrap_or_else(|| parts.get(1).unwrap());
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
        use crate::{WebClient, WebToolError, VERSION};

        // These should compile without issues, proving the re-exports work
        let _ = VERSION;
        let _ = PhantomData::<WebToolError>;
        let _ = PhantomData::<Result<(), WebToolError>>;
        let _ = PhantomData::<WebClient>;
    }

    #[test]
    fn test_dexscreener_re_exports_are_accessible() {
        // Test dexscreener re-exports
        use crate::{ChainInfo, MarketAnalysis, TokenInfo, TokenPair};

        let _ = PhantomData::<ChainInfo>;
        let _ = PhantomData::<MarketAnalysis>;
        let _ = PhantomData::<TokenInfo>;
        let _ = PhantomData::<TokenPair>;
    }

    #[test]
    fn test_news_re_exports_are_accessible() {
        // Test news re-exports
        use crate::{AggregationResult, Article, Source};

        let _ = PhantomData::<AggregationResult>;
        let _ = PhantomData::<Article>;
        let _ = PhantomData::<Source>;
    }

    #[test]
    fn test_twitter_re_exports_are_accessible() {
        // Test twitter re-exports
        use crate::{
            SentimentAnalysis, SentimentBreakdown, TwitterPost, TwitterSearchResult, TwitterUser,
        };

        let _ = PhantomData::<SentimentAnalysis>;
        let _ = PhantomData::<SentimentBreakdown>;
        let _ = PhantomData::<TwitterPost>;
        let _ = PhantomData::<TwitterSearchResult>;
        let _ = PhantomData::<TwitterUser>;
    }

    #[test]
    fn test_web_search_re_exports_are_accessible() {
        // Test web_search re-exports
        use crate::{ContentSummary, SearchResult};

        let _ = PhantomData::<ContentSummary>;
        let _ = PhantomData::<SearchResult>;
    }

    #[test]
    fn test_lunarcrush_re_exports_are_accessible() {
        // Test lunarcrush re-exports
        use crate::{InfluencerMention, InfluencerMentionsResult, SentimentData, TrendingCrypto};

        let _ = PhantomData::<InfluencerMention>;
        let _ = PhantomData::<InfluencerMentionsResult>;
        let _ = PhantomData::<SentimentData>;
        let _ = PhantomData::<TrendingCrypto>;
    }

    #[test]
    fn test_faster100x_re_exports_are_accessible() {
        // Test faster100x re-exports
        use crate::{
            ConcentrationRisk, HolderTrends, TokenHolderAnalysis, WalletHolding, WhaleActivity,
        };

        let _ = PhantomData::<ConcentrationRisk>;
        let _ = PhantomData::<HolderTrends>;
        let _ = PhantomData::<TokenHolderAnalysis>;
        let _ = PhantomData::<WalletHolding>;
        let _ = PhantomData::<WhaleActivity>;
    }

    #[test]
    fn test_price_re_exports_are_accessible() {
        // Test price re-exports
        use crate::TokenPriceResult;

        let _ = PhantomData::<TokenPriceResult>;
    }

    #[test]
    fn test_rugcheck_re_exports_are_accessible() {
        // Test rugcheck re-exports
        use crate::{RiskAnalysis, RiskLevel, RugCheckResult, RugCheckTokenHolder, TokenCheck};

        let _ = PhantomData::<RiskAnalysis>;
        let _ = PhantomData::<RiskLevel>;
        let _ = PhantomData::<RugCheckResult>;
        let _ = PhantomData::<TokenCheck>;
        let _ = PhantomData::<RugCheckTokenHolder>;
    }

    #[test]
    fn test_trenchbot_re_exports_are_accessible() {
        // Test trenchbot re-exports
        use crate::{BundleAnalysisResult, BundleResponse, BundleRiskCheck, CreatorAnalysisResult};

        let _ = PhantomData::<BundleAnalysisResult>;
        let _ = PhantomData::<BundleResponse>;
        let _ = PhantomData::<BundleRiskCheck>;
        let _ = PhantomData::<CreatorAnalysisResult>;
    }

    #[test]
    fn test_pocketuniverse_re_exports_are_accessible() {
        // Test pocketuniverse re-exports
        use crate::{DetailedRugAnalysis, PocketUniverseRugCheck, RugApiResponse, SafetyCheck};

        let _ = PhantomData::<DetailedRugAnalysis>;
        let _ = PhantomData::<PocketUniverseRugCheck>;
        let _ = PhantomData::<RugApiResponse>;
        let _ = PhantomData::<SafetyCheck>;
    }

    #[test]
    fn test_tweetscout_re_exports_are_accessible() {
        // Test tweetscout re-exports
        use crate::{AccountAnalysis, AccountInfo, CredibilityCheck, SocialNetworkAnalysis};

        let _ = PhantomData::<AccountAnalysis>;
        let _ = PhantomData::<AccountInfo>;
        let _ = PhantomData::<CredibilityCheck>;
        let _ = PhantomData::<SocialNetworkAnalysis>;
    }

    #[test]
    fn test_all_function_re_exports_are_accessible() {
        // Test that function re-exports are accessible (compile-time check)
        use crate::{
            analyze_account, analyze_creator_risk, analyze_crypto_sentiment,
            analyze_market_sentiment, analyze_rug_risk, analyze_social_network,
            analyze_token_bundles, analyze_token_holders, analyze_token_market,
            analyze_token_risks, check_bundle_risk, check_if_rugged, check_rug_pull,
            check_rug_pull_raw, find_similar_pages, get_account_info, get_account_score,
            get_bundle_info, get_crypto_articles, get_holder_trends, get_influencer_mentions,
            get_social_sentiment, get_token_info, get_token_price, get_token_prices_batch,
            get_token_report, get_top_followers, get_top_friends, get_top_pairs,
            get_trending_articles, get_trending_cryptos, get_trending_tokens, get_user_tweets,
            get_whale_activity, is_account_credible, is_token_safe, monitor_breaking_alerts,
            search_recent_news, search_tokens, search_tweets, search_web, summarize_web_content,
        };

        // These functions are async and return impl Future, so we can't cast them to simple function pointers.
        // Instead, we verify they exist by referencing them as function items
        let _ = analyze_token_market;
        let _ = get_token_info;
        let _ = get_top_pairs;
        let _ = get_trending_tokens;
        let _ = search_tokens;

        let _ = analyze_market_sentiment;
        let _ = get_crypto_articles;
        let _ = get_trending_articles;
        let _ = monitor_breaking_alerts;

        let _ = analyze_crypto_sentiment;
        let _ = get_user_tweets;
        let _ = search_tweets;

        let _ = find_similar_pages;
        let _ = search_recent_news;
        let _ = search_web;
        let _ = summarize_web_content;

        let _ = get_influencer_mentions;
        let _ = get_social_sentiment;
        let _ = get_trending_cryptos;

        let _ = analyze_token_holders;
        let _ = get_holder_trends;
        let _ = get_whale_activity;

        let _ = get_token_price;
        let _ = get_token_prices_batch;

        let _ = analyze_token_risks;
        let _ = check_if_rugged;
        let _ = get_token_report;

        let _ = analyze_creator_risk;
        let _ = analyze_token_bundles;
        let _ = check_bundle_risk;
        let _ = get_bundle_info;

        let _ = analyze_rug_risk;
        let _ = check_rug_pull;
        let _ = check_rug_pull_raw;
        let _ = is_token_safe;

        let _ = analyze_account;
        let _ = analyze_social_network;
        let _ = get_account_info;
        let _ = get_account_score;
        let _ = get_top_followers;
        let _ = get_top_friends;
        let _ = is_account_credible;
    }

    #[test]
    fn test_version_constant_is_static() {
        // Test that VERSION is a static string reference
        let version_ref: &'static str = VERSION;
        assert!(!version_ref.is_empty(), "VERSION should not be empty");
    }

    #[test]
    #[expect(clippy::absolute_paths)]
    fn test_module_declarations_are_public() {
        // This test verifies that all the modules are properly declared as public
        // by attempting to access module paths (compile-time verification)

        // Test that all modules can be referenced
        let _ = crate::client::WebClient::default;
        let _ = crate::dexscreener::get_token_info;
        // dexscreener_api module contains utility functions and types, not a main struct
        let _ = PhantomData::<crate::error::WebToolError>;
        let _ = crate::faster100x::analyze_token_holders;
        let _ = crate::lunarcrush::get_social_sentiment;
        let _ = crate::news::get_crypto_articles;
        let _ = crate::pocketuniverse::check_rug_pull;
        let _ = crate::price::get_token_price;
        let _ = crate::rugcheck::get_token_report;
        let _ = crate::trenchbot::get_bundle_info;
        let _ = crate::tweetscout::get_account_info;
        let _ = crate::twitter::search_tweets;
        let _ = crate::web_search::search_web;
    }
}
