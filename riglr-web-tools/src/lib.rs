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
//!
//! ## Tool Categories
//!
//! - [`twitter`] - Twitter/X integration for social sentiment
//! - [`dexscreener`] - Token market data and trading metrics
//! - [`web_search`] - Intelligent web search capabilities
//! - [`news`] - Cryptocurrency news aggregation

pub mod client;
pub mod dexscreener;
pub mod error;
pub mod news;
pub mod twitter;
pub mod web_search;

// Re-export commonly used tools - be selective to avoid name conflicts
// From dexscreener
pub use dexscreener::{
    get_token_info, search_tokens, get_trending_tokens, analyze_token_market, get_top_pairs,
    TokenInfo, TokenPair, ChainInfo, MarketAnalysis,
};

// From news
pub use news::{
    get_crypto_news, get_trending_news, monitor_breaking_news, analyze_market_sentiment,
    NewsArticle, NewsSource, NewsAggregationResult,
};

// From twitter
pub use twitter::{
    search_tweets, get_user_tweets, analyze_crypto_sentiment,
    TwitterPost, TwitterUser, TwitterSearchResult, SentimentAnalysis, SentimentBreakdown,
};

// From web_search
pub use web_search::{
    search_web, find_similar_pages, summarize_web_content, search_recent_news,
    WebSearchResult, SearchResult, ContentSummary,
};

// Re-export client and error types
pub use client::WebClient;
pub use error::{Result, WebToolError};

/// Current version of riglr-web-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
