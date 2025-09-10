# riglr-web-tools

{{#include ../../../riglr-web-tools/README.md}}

## API Reference

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Functions](#functions)
- [Type Aliases](#type-aliases)
- [Constants](#constants)

### Structs

> Core data structures and types.

#### `Account`

Account information for followers/friends lists

---

#### `AccountAnalysis`

Comprehensive account analysis result

---

#### `AccountInfo`

Account information response from TweetScout

---

#### `AggregationMetadata`

Metadata about the news aggregation process

---

#### `ApiKeys`

Type-safe API keys configuration

---

#### `Args`

Arguments structure for the tool

---

#### `BaseUrls`

Base URL configuration for various services

---

#### `BoostsResponse`

Boosted token response (new v1 endpoint)

---

#### `BreakingNewsAlert`

Breaking news alert

---

#### `BundleAnalysis`

Analysis results for a bundle

---

#### `BundleAnalysisResult`

Simplified bundle analysis result

---

#### `BundleDetails`

Details about a specific bundle

---

#### `BundleResponse`

Main bundle response from TrenchBot

---

#### `BundleRiskCheck`

Simple bundle risk check result

---

#### `ChainInfo`

Blockchain/chain information

---

#### `ClientConfig`

Type-safe client configuration

---

#### `ConcentrationAnalysis`

Holder concentration analysis

---

#### `ConcentrationRisk`

Concentration risk analysis

---

#### `ContentEntity`

Entity found in content

---

#### `ContentSummary`

Content summary with key points

---

#### `ContentType`

Content type and format information

---

#### `ContextAnnotation`

Context annotation for tweet topics

---

#### `CreatorAnalysis`

Analysis of the token creator

---

#### `CreatorAnalysisResult`

Creator analysis result

---

#### `CreatorHistory`

Historical data on the creator's activity

---

#### `CreatorRiskAssessment`

Creator risk assessment

---

#### `CredibilityCheck`

Simple credibility check result

---

#### `DetailedRugAnalysis`

Detailed analysis result with additional insights

---

#### `DexInfo`

DEX platform information

---

#### `DexScreenerConfig`

Configuration for DexScreener API access

---

#### `DexScreenerResponse`

Clean response from DexScreener API

---

#### `DexScreenerResponseRaw`

Raw response from DexScreener API containing token pair information

---

#### `DomainInfo`

Domain information for a search result

---

#### `EmotionalIndicators`

Emotional indicators in news content

---

#### `EngagementMetrics`

Engagement metrics for an account

---

#### `EntityMention`

Entity mention statistics

---

#### `ErrorDetail`

Error detail structure

---

#### `ErrorResponse`

Error response from TweetScout API

---

#### `Faster100xConfig`

Faster100x API configuration

---

#### `FileMetadata`

File metadata associated with the token

---

#### `GraphDetectedData`

Graph-based insider detection data

---

#### `HolderActivity`

Recent holder activity metrics

---

#### `HolderDistribution`

Holder distribution breakdown

---

#### `HolderTrendPoint`

Individual holder trend data point

---

#### `HolderTrends`

Holder trends over time

---

#### `HttpConfig`

Configuration for HTTP client

---

#### `InfluencerMention`

Influencer mention data

---

#### `InfluencerMentionsResult`

Result containing multiple influencer mentions

---

#### `InsiderAnalysis`

Insider trading analysis

---

#### `InsiderDetectedData`

Insider detection data

---

#### `InsiderNetwork`

Insider network information

---

#### `KnownAccount`

Known account information

---

#### `LexiconSentimentAnalyzer`

Default lexicon-based sentiment analyzer

---

#### `Liquidity`

Clean liquidity information

---

#### `LiquidityAnalysis`

Liquidity analysis

---

#### `LiquidityRaw`

Raw liquidity information for a trading pair

---

#### `Locker`

Token locker information

---

#### `LunarCrushConfig`

LunarCrush API configuration

---

#### `Market`

Market trading pair information

---

#### `MarketAnalysis`

Market analysis result

---

#### `MarketImpact`

Market impact assessment for news

---

#### `MarketLP`

Market LP token information

---

#### `NewsAggregationResult`

Comprehensive news aggregation result

---

#### `NewsArticle`

Comprehensive news article with metadata and analysis

---

#### `NewsCategory`

News category and classification

---

#### `NewsConfig`

Configuration for news aggregation services

---

#### `NewsEntity`

Entities mentioned in news (people, companies, assets)

---

#### `NewsInsights`

Insights extracted from news aggregation

---

#### `NewsSentiment`

Sentiment analysis for news article

---

#### `NewsSource`

News source information and credibility

---

#### `Order`

Individual order information

---

#### `OrdersResponse`

Orders response for a token (new v1 endpoint)

---

#### `PageMetadata`

Page metadata extracted from HTML

---

#### `PairInfo`

Clean pair information

---

#### `PairInfoRaw`

Raw information about a trading pair from DexScreener

---

#### `PairToken`

Token information within a pair

---

#### `PocketUniverseConfig`

Configuration for PocketUniverse API access

---

#### `PreviousCoin`

Information about a previously created coin

---

#### `PriceChange`

Clean price change statistics

---

#### `PriceChangeRaw`

Raw price change statistics over different time periods

---

#### `PriceLevelAnalysis`

Price level analysis

---

#### `QualityMetrics`

Article quality assessment metrics

---

#### `RateLimitInfo`

Rate limit information

---

#### `RateLimits`

Rate limiting configuration

---

#### `Receiver`

Transfer receiver

---

#### `Risk`

Risk factor information

---

#### `RiskAnalysis`

Risk analysis summary

---

#### `RiskAssessment`

Comprehensive risk assessment including liquidity, volatility, and contract risks

---

#### `RiskFactor`

Individual risk factor

---

#### `RugCheckConfig`

Configuration for RugCheck API access

---

#### `RugCheckResult`

Simple rug check result

---

#### `SafetyCheck`

Simple safety check result

---

#### `ScamDetection`

Scam detection results

---

#### `ScoreResponse`

Score response from TweetScout

---

#### `ScoredAccount`

Account with score for network analysis

---

#### `SearchInsights`

Aggregated insights from search results

---

#### `SearchMetadata`

Metadata for search results

---

#### `SearchResult`

Comprehensive search result with content and metadata

---

#### `SearchSentiment`

Sentiment analysis of search results

---

#### `SecurityInfo`

Token security and verification information

---

#### `SentimentAnalysis`

Sentiment analysis result for tweets

---

#### `SentimentBreakdown`

Breakdown of sentiment scores

---

#### `SentimentData`

Social sentiment data for a cryptocurrency

---

#### `SentimentDistribution`

Distribution of sentiment across results

---

#### `SentimentPhrase`

Key phrases contributing to sentiment

---

#### `SeoMetadata`

SEO-related metadata

---

#### `SimilarPagesResult`

Similar page search result

---

#### `SimilarityMetadata`

Metadata about similarity analysis

---

#### `SocialLink`

Social media and community links

---

#### `SocialMetadata`

Social media metadata (Open Graph, Twitter Cards)

---

#### `SocialMetrics`

Social media engagement metrics

---

#### `SocialNetworkAnalysis`

Social network analysis result

---

#### `SourceDiversity`

Source diversity analysis

---

#### `Token`

Clean token information

---

#### `TokenCheck`

Main token check report from RugCheck

---

#### `TokenEvent`

Token event in history

---

#### `TokenHolder`

Token holder information

---

#### `TokenHolderAnalysis`

Token holder analysis data

---

#### `TokenInfo`

Comprehensive token information including price, volume, and market data

---

#### `TokenLink`

Token link information

---

#### `TokenMetadata`

Token metadata

---

#### `TokenPair`

Trading pair information

---

#### `TokenPriceResult`

Price result with additional metadata

---

#### `TokenProfile`

Token profile information (new v1 endpoint)

---

#### `TokenRaw`

Raw token information

---

#### `TokenSearchResult`

Token search results with metadata and execution information

---

#### `Tool`

Tool implementation structure

---

#### `TransactionStats`

Clean transaction stats

---

#### `TransactionStatsRaw`

Raw buy and sell transaction statistics

---

#### `Transactions`

Clean transaction statistics

---

#### `TransactionsRaw`

Raw transaction statistics over different time periods

---

#### `Transfer`

Transfer information

---

#### `TransferFee`

Transfer fee configuration

---

#### `TrenchBotConfig`

Configuration for TrenchBot API access

---

#### `TrendAnalysis`

Market trend analysis including direction, momentum, and key price levels

---

#### `TrendingCrypto`

Trending cryptocurrency with social metrics

---

#### `TrendingTopic`

Trending topic analysis

---

#### `TweetEntities`

Entities extracted from tweet text

---

#### `TweetMetrics`

Tweet engagement metrics

---

#### `TweetScoutConfig`

Configuration for TweetScout API access

---

#### `TwitterConfig`

Configuration for Twitter API access

---

#### `TwitterPost`

A Twitter/X post with metadata

---

#### `TwitterSearchResult`

Result of Twitter search operation

---

#### `TwitterTool`

Twitter tool for social sentiment analysis

---

#### `TwitterUser`

Twitter user information

---

#### `VerifiedToken`

Verified token information

---

#### `VerifiedTokenLinks`

Verified token links

---

#### `Volume`

Clean volume statistics

---

#### `VolumeAnalysis`

Volume analysis including trends, ratios, and trading activity metrics

---

#### `VolumeRaw`

Raw trading volume statistics over different time periods

---

#### `WalletCategoryBreakdown`

Wallet category breakdown

---

#### `WalletHolding`

Individual wallet holding information

---

#### `WalletInfo`

Information about a specific wallet's interaction with a bundle

---

#### `WebClient`

A client for interacting with various web APIs and services

---

#### `WebSearchConfig`

Configuration for web search services

---

#### `WebSearchMetadata`

Metadata about the search operation

---

#### `WebSearchResult`

Complete search operation result

---

#### `WhaleActivity`

Whale activity tracking data

---

#### `WhaleTransaction`

Individual whale transaction

---

### Enums

> Enumeration types for representing variants.

#### `NetworkQuality`

Network quality assessment

**Variants:**

- `High`
  - High quality network
- `Medium`
  - Medium quality network
- `Low`
  - Low quality network
- `Suspicious`
  - Suspicious network

---

#### `OrderStatus`

Order status enum

**Variants:**

- `Processing`
  - Order is currently being processed
- `Cancelled`
  - Order has been cancelled
- `OnHold`
  - Order is on hold
- `Approved`
  - Order has been approved
- `Rejected`
  - Order has been rejected

---

#### `OrderType`

Order type enum

**Variants:**

- `TokenProfile`
  - Token profile order type
- `CommunityTakeover`
  - Community takeover order type
- `TokenAd`
  - Token advertisement order type
- `TrendingBarAd`
  - Trending bar advertisement order type

---

#### `ProcessingStatus`

Processing status of the token

**Variants:**

- `Processed`
  - Token has been fully processed
- `NotProcessed`
  - Token not yet processed (insufficient data)
- `Error`
  - Error occurred during processing

---

#### `RiskLevel`

Risk level enumeration

**Variants:**

- `Low`
  - Low risk - Creator has good track record
- `Medium`
  - Medium risk - Some concerning patterns
- `High`
  - High risk - Multiple red flags detected

---

#### `RiskTolerance`

Risk tolerance levels for safety checks

**Variants:**

- `Low`
  - Only accept low risk tokens
- `Medium`
  - Accept low and medium risk tokens
- `High`
  - Accept all except extreme risk tokens

---

#### `RugApiResponse`

Main rug check API response from PocketUniverse

**Variants:**

- `NotProcessed`
  - Token has not been processed yet
- `Processed`
  - Token has been processed and analyzed

---

#### `ScoreLevel`

Credibility score level classification

**Variants:**

- `Excellent`
  - Excellent credibility (80-100)
- `Good`
  - Good credibility (60-80)
- `Fair`
  - Fair credibility (40-60)
- `Poor`
  - Poor credibility (20-40)
- `VeryPoor`
  - Very poor credibility (0-20)

---

#### `VolumeConcentration`

Volume concentration level

**Variants:**

- `Distributed`
  - Volume well distributed
- `Moderate`
  - Moderate concentration
- `Concentrated`
  - High concentration in few wallets
- `Extreme`
  - Extreme concentration (potential manipulation)

---

#### `WebToolError`

Main error type for web tool operations.

The IntoToolError derive macro automatically classifies errors:
- Retriable: Network (includes HTTP), Api (includes request errors), RateLimit
- Permanent: Auth, Parsing (includes JSON), Config, Client, InvalidInput

**Variants:**

- `Network`
  - Network error (includes HTTP) - automatically retriable
- `Http`
  - HTTP request error - automatically retriable (converted to Network)
- `Api`
  - API error (includes general API issues) - automatically retriable
- `RateLimit`
  - API rate limit exceeded - automatically handled as rate_limited
- `Auth`
  - API authentication failed - permanent
- `Parsing`
  - Parsing error (includes JSON and response parsing) - permanent
- `Serialization`
  - Serialization error - automatically permanent
- `Url`
  - URL parsing error - permanent
- `Config`
  - Configuration error - permanent
- `Client`
  - Client creation error - permanent
- `InvalidInput`
  - Invalid input provided - permanent
- `Core`
  - Core riglr error

---

### Traits

> Trait definitions for implementing common behaviors.

#### `SentimentAnalyzer`

Trait for pluggable sentiment analysis

**Methods:**

- `analyze()`
  - Analyze sentiment from text components

---

### Functions

> Standalone functions and utilities.

#### `aggregate_token_info`

Helper function to aggregate data from multiple pairs into a single TokenInfo

---

#### `analyze_account`

Perform comprehensive analysis of a Twitter/X account including credibility scoring.
Combines account info and score into a detailed assessment.

---

#### `analyze_account_tool`

Factory function to create a new instance of the tool

---

#### `analyze_creator_risk`

Analyze the creator of a Solana token for historical rug pull activity.
Provides detailed creator risk assessment based on their track record.

---

#### `analyze_creator_risk_tool`

Factory function to create a new instance of the tool

---

#### `analyze_crypto_sentiment`

Analyze sentiment of cryptocurrency-related tweets

This tool performs comprehensive sentiment analysis on cryptocurrency-related tweets,
providing insights into market mood and social trends.

---

#### `analyze_crypto_sentiment_tool`

Factory function to create a new instance of the tool

---

#### `analyze_market_sentiment`

Analyze market sentiment from recent news

This tool provides comprehensive sentiment analysis across recent news articles,
helping to gauge overall market mood and potential price impact.

---

#### `analyze_market_sentiment_tool`

Factory function to create a new instance of the tool

---

#### `analyze_rug_risk`

Perform detailed analysis of a token's rug pull risk with comprehensive insights.
Provides volume breakdown, risk factors, and actionable recommendations.

---

#### `analyze_rug_risk_tool`

Factory function to create a new instance of the tool

---

#### `analyze_social_network`

Analyze the social network of a Twitter/X account including followers and friends.
Provides insights into the quality and influence of an account's network.

---

#### `analyze_social_network_tool`

Factory function to create a new instance of the tool

---

#### `analyze_token_bundles`

Analyze a Solana token for bundling patterns and provide risk assessment.
This tool provides a simplified analysis based on TrenchBot data.

---

#### `analyze_token_bundles_tool`

Factory function to create a new instance of the tool

---

#### `analyze_token_holders`

Analyze token holder distribution and concentration risks

This tool provides comprehensive analysis of a token's holder distribution,
including concentration risks, whale analysis, and holder categorization.

# Arguments
* `token_address` - Token contract address (must be valid hex address)
* `chain` - Blockchain network: "eth", "bsc", "polygon", "arbitrum", "base". Defaults to "eth"

# Returns
Detailed holder analysis including distribution metrics and risk assessment

---

#### `analyze_token_holders_tool`

Factory function to create a new instance of the tool

---

#### `analyze_token_market`

Analyze token market data using heuristic-based calculations

This tool provides market analysis based on available on-chain data including:
- Simple trend analysis based on price changes
- Volume pattern calculations from 24h data
- Liquidity assessment from DEX pairs
- Basic risk evaluation using heuristic scoring

Note: This is not a substitute for professional financial analysis or
machine learning models. All calculations are rule-based heuristics.

---

#### `analyze_token_market_tool`

Factory function to create a new instance of the tool

---

#### `analyze_token_risks`

Analyze a Solana token's security risks and provide a comprehensive risk assessment.
This tool provides a simplified risk analysis based on RugCheck data.

---

#### `analyze_token_risks_tool`

Factory function to create a new instance of the tool

---

#### `check_bundle_risk`

Check if a Solana token shows signs of bundling manipulation.
Returns a simple assessment of bundling risk.

---

#### `check_bundle_risk_tool`

Factory function to create a new instance of the tool

---

#### `check_if_rugged`

Check if a Solana token has been rugged or shows signs of a rug pull.
Returns a simple boolean result with basic risk information.

---

#### `check_if_rugged_tool`

Factory function to create a new instance of the tool

---

#### `check_rug_pull`

Check a Solana token or pool for rug pull risk with simplified results.
Provides an easy-to-use risk assessment based on PocketUniverse data.

---

#### `check_rug_pull_raw`

Check a Solana token or pool for rug pull risk using PocketUniverse.
This is the raw API call that returns the direct response.

---

#### `check_rug_pull_raw_tool`

Factory function to create a new instance of the tool

---

#### `check_rug_pull_tool`

Factory function to create a new instance of the tool

---

#### `check_token_orders`

Check orders for a specific token

This tool retrieves paid orders for a token including
token profiles, community takeovers, and ads.

---

#### `check_token_orders_tool`

Factory function to create a new instance of the tool

---

#### `find_best_liquidity_pair`

Find the best liquidity pair for a token

---

#### `find_similar_pages`

Search for pages similar to a given URL

This tool finds web pages that are similar in content and topic to a source URL,
useful for finding related information or alternative perspectives.

---

#### `find_similar_pages_tool`

Factory function to create a new instance of the tool

---

#### `format_chain_name`

Format a chain ID into a human-readable display name

---

#### `format_dex_name`

Format a DEX ID into a human-readable display name

---

#### `get_account_info`

Get basic information about a Twitter/X account.

---

#### `get_account_info_tool`

Factory function to create a new instance of the tool

---

#### `get_account_score`

Get the credibility score for a Twitter/X account.
Returns a score from 0-100 indicating account trustworthiness.

---

#### `get_account_score_tool`

Factory function to create a new instance of the tool

---

#### `get_bundle_info`

Get comprehensive bundle information for a Solana token from TrenchBot.
This tool analyzes token distribution patterns to detect bundling and assess risk.

---

#### `get_bundle_info_tool`

Factory function to create a new instance of the tool

---

#### `get_crypto_news`

Get comprehensive cryptocurrency news for a specific topic

This tool aggregates news from multiple sources, performs sentiment analysis,
and assesses market impact for cryptocurrency-related topics.

---

#### `get_crypto_news_tool`

Factory function to create a new instance of the tool

---

#### `get_holder_trends`

Get holder trends over time for a token

This tool provides historical analysis of holder growth/decline patterns,
correlating holder changes with price movements and market events.

# Arguments
* `token_address` - Token contract address
* `period` - Analysis period: "7d", "30d", "90d". Defaults to "30d"
* `data_points` - Number of data points to return (10-100). Defaults to 30

# Returns
Time series analysis of holder trends with insights and correlations

---

#### `get_holder_trends_tool`

Factory function to create a new instance of the tool

---

#### `get_influencer_mentions`

Get influencer mentions for a specific token from LunarCrush

This tool tracks mentions from cryptocurrency influencers and high-follower accounts,
providing insights into what key opinion leaders are saying about specific tokens.

# Arguments
* `token_symbol` - Cryptocurrency symbol to search for (e.g., "BTC", "ETH", "SOL")
* `limit` - Maximum number of mentions to return (1-50). Defaults to 20
* `timeframe` - Time period to search: "24h", "7d", "30d". Defaults to "24h"

# Returns
Collection of influencer mentions with engagement metrics and sentiment analysis

---

#### `get_influencer_mentions_tool`

Factory function to create a new instance of the tool

---

#### `get_latest_boosted_tokens`

Get latest boosted tokens from DexScreener

This tool retrieves tokens that have been recently boosted
on the DexScreener platform.

---

#### `get_latest_boosted_tokens_tool`

Factory function to create a new instance of the tool

---

#### `get_latest_token_boosts`

Get the latest boosted tokens (rate-limit 60 requests per minute)

---

#### `get_latest_token_profiles`

Get the latest token profiles (rate-limit 60 requests per minute)

---

#### `get_latest_token_profiles_tool`

Factory function to create a new instance of the tool

---

#### `get_native_token`

Get the native token symbol for a given blockchain

---

#### `get_pair_by_address`

Legacy: Get pairs by pair address (without chainId)
Deprecated: Use get_pair_by_address_v1 instead

---

#### `get_pair_by_address_v1`

Get pairs by pair address (requires chainId in new API)

---

#### `get_pairs_by_token`

Legacy: Get token pairs by token address (without chainId)
Deprecated: Use get_pairs_by_token_v1 instead

---

#### `get_pairs_by_token_v1`

Get token pairs by token address (requires chainId in new API)

---

#### `get_social_sentiment`

Get social sentiment data for a cryptocurrency from LunarCrush

This tool provides comprehensive social analytics for any cryptocurrency,
including sentiment scores, social volume, and platform-specific data.

# Arguments
* `symbol` - Cryptocurrency symbol (e.g., "BTC", "ETH", "SOL")
* `timeframe` - Optional timeframe for analysis ("1h", "24h", "7d", "30d"). Defaults to "24h"

# Returns
Detailed sentiment analysis including scores, volume metrics, and trending keywords

---

#### `get_social_sentiment_tool`

Factory function to create a new instance of the tool

---

#### `get_token_info`

Get comprehensive token information from DexScreener

This tool retrieves detailed token information including price, volume,
market cap, trading pairs, and security analysis.

---

#### `get_token_info_tool`

Factory function to create a new instance of the tool

---

#### `get_token_orders`

Check orders paid for a token (rate-limit 60 requests per minute)

---

#### `get_token_pairs_v1`

Get the pools of a given token address (new v1 endpoint)

---

#### `get_token_price`

Extract token price from the best pair

---

#### `get_token_price_tool`

Factory function to create a new instance of the tool

---

#### `get_token_prices_batch`

Get multiple token prices in a batch request

This tool fetches prices for multiple tokens efficiently by making multiple
requests concurrently. Useful for portfolio tracking or multi-token analysis.

# Arguments

* `token_addresses` - List of token addresses to get prices for
* `chain` - Optional chain name to apply to all tokens

# Returns

Returns `Vec<TokenPriceResult>` with prices for all found tokens.
Tokens without available price data are omitted from results.

# Errors

* `ToolError::InvalidInput` - When token addresses list is empty
* `ToolError::Retriable` - When API requests fail

# Examples

```rust,ignore
use riglr_web_tools::price::get_token_prices_batch;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let tokens = vec![
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
    "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(), // USDT
];

let prices = get_token_prices_batch(tokens, Some("ethereum".to_string())).await?;

for price in prices {
    println!("{}: ${}", price.token_symbol.unwrap_or_default(), price.price_usd);
}
# Ok(())
# }
```

---

#### `get_token_prices_batch_tool`

Factory function to create a new instance of the tool

---

#### `get_token_report`

Get a comprehensive security report for a Solana token from RugCheck.
This tool analyzes tokens for rug pull risks, insider trading, and other security concerns.

---

#### `get_token_report_tool`

Factory function to create a new instance of the tool

---

#### `get_top_boosted_tokens`

Get top boosted tokens from DexScreener

This tool retrieves tokens with the most active boosts
on the DexScreener platform.

---

#### `get_top_boosted_tokens_tool`

Factory function to create a new instance of the tool

---

#### `get_top_followers`

Get the top 20 followers of a Twitter/X account with their scores.

---

#### `get_top_followers_tool`

Factory function to create a new instance of the tool

---

#### `get_top_friends`

Get the top 20 friends (accounts being followed) of a Twitter/X account with their scores.

---

#### `get_top_friends_tool`

Factory function to create a new instance of the tool

---

#### `get_top_pairs`

Get top DEX pairs by volume across all chains

This tool retrieves the highest volume trading pairs,
useful for identifying active markets and arbitrage opportunities.

---

#### `get_top_pairs_tool`

Factory function to create a new instance of the tool

---

#### `get_top_token_boosts`

Get the tokens with most active boosts (rate-limit 60 requests per minute)

---

#### `get_trending_cryptos`

Get trending cryptocurrencies by social metrics from LunarCrush

This tool identifies the most trending cryptocurrencies based on social media activity,
sentiment, and LunarCrush's proprietary Galaxy Score.

# Arguments
* `limit` - Maximum number of trending cryptocurrencies to return (1-50). Defaults to 10
* `sort_by` - Sort metric: "galaxy_score", "social_volume", "alt_rank". Defaults to "galaxy_score"

# Returns
List of trending cryptocurrencies with social and market metrics

---

#### `get_trending_cryptos_tool`

Factory function to create a new instance of the tool

---

#### `get_trending_news`

Get trending cryptocurrency news across all topics

This tool identifies currently trending news and topics in the cryptocurrency space,
useful for staying updated on breaking developments and market movements.

---

#### `get_trending_news_tool`

Factory function to create a new instance of the tool

---

#### `get_trending_tokens`

Get trending tokens from DexScreener

This tool retrieves trending tokens based on volume,
price changes, and social activity.

---

#### `get_trending_tokens_tool`

Factory function to create a new instance of the tool

---

#### `get_user_tweets`

Get recent tweets from a specific user

This tool fetches recent tweets from a specified Twitter/X user account.

---

#### `get_user_tweets_tool`

Factory function to create a new instance of the tool

---

#### `get_whale_activity`

Get whale activity for a specific token

This tool tracks large-holder (whale) transactions and movements,
providing insights into institutional and high-net-worth trading activity.

# Arguments
* `token_address` - Token contract address
* `timeframe` - Analysis timeframe: "1h", "4h", "24h", "7d". Defaults to "24h"
* `min_usd_value` - Minimum transaction value in USD to be considered whale activity. Defaults to 10000

# Returns
Whale activity analysis with transaction details and flow metrics

---

#### `get_whale_activity_tool`

Factory function to create a new instance of the tool

---

#### `is_account_credible`

Quick credibility check for a Twitter/X account.
Returns a simple assessment of whether an account is trustworthy.

---

#### `is_account_credible_tool`

Factory function to create a new instance of the tool

---

#### `is_token_safe`

Quick safety check for a Solana token - returns a simple safe/unsafe verdict.
Best for quick filtering of tokens before deeper analysis.

---

#### `is_token_safe_tool`

Factory function to create a new instance of the tool

---

#### `monitor_breaking_news`

Monitor for breaking news and generate real-time alerts

This tool continuously monitors news sources for breaking news
and generates alerts based on severity and market impact criteria.

---

#### `monitor_breaking_news_tool`

Factory function to create a new instance of the tool

---

#### `search_recent_news`

Search for recent news and articles on a topic

This tool specifically searches for recent news articles and blog posts,
optimized for finding current information and trending discussions.

---

#### `search_recent_news_tool`

Factory function to create a new instance of the tool

---

#### `search_ticker`

Search for tokens or pairs on DexScreener

---

#### `search_tokens`

Search for tokens on DexScreener

This tool searches for tokens by name, symbol, or address
with support for filtering by chain and market cap.

---

#### `search_tokens_tool`

Factory function to create a new instance of the tool

---

#### `search_tweets`

Search for tweets matching a query with comprehensive filtering

This tool searches Twitter/X for tweets matching the given query,
with support for advanced filters and sentiment analysis.

---

#### `search_tweets_tool`

Factory function to create a new instance of the tool

---

#### `search_web`

Perform web search with content extraction

This tool performs web search and returns results with extracted content and metadata.
Uses traditional search APIs rather than semantic understanding.

---

#### `search_web_tool`

Factory function to create a new instance of the tool

---

#### `search_web_with_context`

Internal function to perform web search with ApplicationContext

---

#### `summarize_web_content`

Summarize content from multiple web pages

This tool extracts and summarizes key information from multiple web pages,
creating a comprehensive overview of a topic from multiple sources.

---

#### `summarize_web_content_tool`

Factory function to create a new instance of the tool

---

### Type Aliases

#### `Result`

Result type alias for web tool operations.

**Type:** `<T, >`

---

### Constants

#### `VERSION`

Current version of riglr-web-tools

**Type:** `&str`

---
