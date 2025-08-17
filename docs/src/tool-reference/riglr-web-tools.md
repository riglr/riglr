# riglr-web-tools Tool Reference

This page contains documentation for tools provided by the `riglr-web-tools` crate.

## Available Tools

- [`get_token_price`](#get_token_price) - src/price.rs
- [`get_token_prices_batch`](#get_token_prices_batch) - src/price.rs
- [`get_token_info`](#get_token_info) - src/dexscreener.rs
- [`search_tokens`](#search_tokens) - src/dexscreener.rs
- [`get_trending_tokens`](#get_trending_tokens) - src/dexscreener.rs
- [`analyze_token_market`](#analyze_token_market) - src/dexscreener.rs
- [`get_top_pairs`](#get_top_pairs) - src/dexscreener.rs
- [`search_tweets`](#search_tweets) - src/twitter.rs
- [`get_user_tweets`](#get_user_tweets) - src/twitter.rs
- [`analyze_crypto_sentiment`](#analyze_crypto_sentiment) - src/twitter.rs
- [`get_crypto_news`](#get_crypto_news) - src/news.rs
- [`get_trending_news`](#get_trending_news) - src/news.rs
- [`monitor_breaking_news`](#monitor_breaking_news) - src/news.rs
- [`analyze_market_sentiment`](#analyze_market_sentiment) - src/news.rs
- [`search_web`](#search_web) - src/web_search.rs
- [`find_similar_pages`](#find_similar_pages) - src/web_search.rs
- [`summarize_web_content`](#summarize_web_content) - src/web_search.rs
- [`search_recent_news`](#search_recent_news) - src/web_search.rs

## Tool Functions

### get_token_price

**Source**: `src/price.rs`

```rust
pub async fn get_token_price( token_address: String, chain: Option<String>, ) -> Result<TokenPriceResult, ToolError>
```

**Documentation:**

Get token price from DexScreener with highest liquidity pair

This tool fetches the most reliable token price by finding the trading pair
with the highest liquidity on DexScreener. Using the highest liquidity pair
ensures the most accurate and stable price data.

# Arguments

* `token_address` - Token contract address to get price for
* `chain` - Optional chain name (e.g., "ethereum", "bsc", "polygon", "solana")

# Returns

Returns `TokenPriceResult` containing:
- `token_address`: The queried token address
- `token_symbol`: Token symbol if available
- `price_usd`: Current price in USD as string
- `source_dex`: DEX where price was sourced from
- `source_pair`: Trading pair address used
- `source_liquidity_usd`: Liquidity of the source pair
- `chain`: Chain name
- `fetched_at`: Timestamp when price was fetched

# Errors

* `ToolError::InvalidInput` - When token address format is invalid
* `ToolError::Retriable` - When DexScreener API request fails
* `ToolError::Permanent` - When no trading pairs found for token

# Examples

```rust,ignore
use riglr_web_tools::price::get_token_price;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Get USDC price on Ethereum
let price = get_token_price(
"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
Some("ethereum".to_string()),
).await?;

println!("USDC price: ${}", price.price_usd);
println!("Source: {} (${:.2} liquidity)",
price.source_dex.unwrap_or_default(),
price.source_liquidity_usd.unwrap_or(0.0));
# Ok(())
# }
```

---

### get_token_prices_batch

**Source**: `src/price.rs`

```rust
pub async fn get_token_prices_batch( token_addresses: Vec<String>, chain: Option<String>, ) -> Result<Vec<TokenPriceResult>, ToolError>
```

**Documentation:**

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

### get_token_info

**Source**: `src/dexscreener.rs`

```rust
pub async fn get_token_info( token_address: String, chain_id: Option<String>, include_pairs: Option<bool>, include_security: Option<bool>, ) -> crate::error::Result<TokenInfo>
```

**Documentation:**

Get comprehensive token information from DexScreener

This tool retrieves detailed token information including price, volume,
market cap, trading pairs, and security analysis.

---

### search_tokens

**Source**: `src/dexscreener.rs`

```rust
pub async fn search_tokens( query: String, chain_filter: Option<String>, min_market_cap: Option<f64>, min_liquidity: Option<f64>, limit: Option<u32>, ) -> crate::error::Result<TokenSearchResult>
```

**Documentation:**

Search for tokens on DexScreener

This tool searches for tokens by name, symbol, or address
with support for filtering by chain and market cap.

---

### get_trending_tokens

**Source**: `src/dexscreener.rs`

```rust
pub async fn get_trending_tokens( time_window: Option<String>, // "5m", "1h", "24h" chain_filter: Option<String>, min_volume: Option<f64>, limit: Option<u32>, ) -> crate::error::Result<Vec<TokenInfo>>
```

**Documentation:**

Get trending tokens from DexScreener

This tool retrieves trending tokens based on volume,
price changes, and social activity.

---

### analyze_token_market

**Source**: `src/dexscreener.rs`

```rust
pub async fn analyze_token_market( token_address: String, chain_id: Option<String>, _include_technical: Option<bool>, include_risk: Option<bool>, ) -> crate::error::Result<MarketAnalysis>
```

**Documentation:**

Analyze token market data using heuristic-based calculations

This tool provides market analysis based on available on-chain data including:
- Simple trend analysis based on price changes
- Volume pattern calculations from 24h data
- Liquidity assessment from DEX pairs
- Basic risk evaluation using heuristic scoring

Note: This is not a substitute for professional financial analysis or
machine learning models. All calculations are rule-based heuristics.

---

### get_top_pairs

**Source**: `src/dexscreener.rs`

```rust
pub async fn get_top_pairs( time_window: Option<String>, // "5m", "1h", "24h" chain_filter: Option<String>, dex_filter: Option<String>, min_liquidity: Option<f64>, limit: Option<u32>, ) -> crate::error::Result<Vec<TokenPair>>
```

**Documentation:**

Get top DEX pairs by volume across all chains

This tool retrieves the highest volume trading pairs,
useful for identifying active markets and arbitrage opportunities.

---

### search_tweets

**Source**: `src/twitter.rs`

```rust
pub async fn search_tweets( query: String, max_results: Option<u32>, include_sentiment: Option<bool>, language: Option<String>, start_time: Option<String>, end_time: Option<String>, ) -> crate::error::Result<TwitterSearchResult>
```

**Documentation:**

Search for tweets matching a query with comprehensive filtering

This tool searches Twitter/X for tweets matching the given query,
with support for advanced filters and sentiment analysis.

---

### get_user_tweets

**Source**: `src/twitter.rs`

```rust
pub async fn get_user_tweets( username: String, max_results: Option<u32>, include_replies: Option<bool>, include_retweets: Option<bool>, ) -> crate::error::Result<Vec<TwitterPost>>
```

**Documentation:**

Get recent tweets from a specific user

This tool fetches recent tweets from a specified Twitter/X user account.

---

### analyze_crypto_sentiment

**Source**: `src/twitter.rs`

```rust
pub async fn analyze_crypto_sentiment( token_symbol: String, time_window_hours: Option<u32>, min_engagement: Option<u32>, ) -> crate::error::Result<SentimentAnalysis>
```

**Documentation:**

Analyze sentiment of cryptocurrency-related tweets

This tool performs comprehensive sentiment analysis on cryptocurrency-related tweets,
providing insights into market mood and social trends.

---

### get_crypto_news

**Source**: `src/news.rs`

```rust
pub async fn get_crypto_news( topic: String, time_window: Option<String>, // "1h", "6h", "24h", "week" source_types: Option<Vec<String>>, // "mainstream", "crypto", "analysis" min_credibility: Option<u32>, include_analysis: Option<bool>, ) -> crate::error::Result<NewsAggregationResult>
```

**Documentation:**

Get comprehensive cryptocurrency news for a specific topic

This tool aggregates news from multiple sources, performs sentiment analysis,
and assesses market impact for cryptocurrency-related topics.

---

### get_trending_news

**Source**: `src/news.rs`

```rust
pub async fn get_trending_news( time_window: Option<String>, // "1h", "6h", "24h" categories: Option<Vec<String>>, // "defi", "nft", "regulation", "tech" min_impact_score: Option<u32>, limit: Option<u32>, ) -> crate::error::Result<NewsAggregationResult>
```

**Documentation:**

Get trending cryptocurrency news across all topics

This tool identifies currently trending news and topics in the cryptocurrency space,
useful for staying updated on breaking developments and market movements.

---

### monitor_breaking_news

**Source**: `src/news.rs`

```rust
pub async fn monitor_breaking_news( keywords: Vec<String>, severity_threshold: Option<String>, // "Critical", "High", "Medium" impact_threshold: Option<u32>, // 0-100 _alert_channels: Option<Vec<String>>, // "webhook", "email", "slack" ) -> crate::error::Result<Vec<BreakingNewsAlert>>
```

**Documentation:**

Monitor for breaking news and generate real-time alerts

This tool continuously monitors news sources for breaking news
and generates alerts based on severity and market impact criteria.

---

### analyze_market_sentiment

**Source**: `src/news.rs`

```rust
pub async fn analyze_market_sentiment( time_window: Option<String>, // "1h", "6h", "24h", "week" asset_filter: Option<Vec<String>>, // Specific cryptocurrencies to focus on _source_weights: Option<HashMap<String, f64>>, // Weight different sources _include_social: Option<bool>, ) -> crate::error::Result<NewsInsights>
```

**Documentation:**

Analyze market sentiment from recent news

This tool provides comprehensive sentiment analysis across recent news articles,
helping to gauge overall market mood and potential price impact.

---

### search_web

**Source**: `src/web_search.rs`

```rust
pub async fn search_web( query: String, max_results: Option<u32>, include_content: Option<bool>, domain_filter: Option<Vec<String>>, date_filter: Option<String>, // "day", "week", "month", "year" content_type_filter: Option<String>, // "news", "academic", "blog" ) -> crate::error::Result<WebSearchResult>
```

**Documentation:**

Perform web search with content extraction

This tool performs web search and returns results with extracted content and metadata.
Uses traditional search APIs rather than semantic understanding.

---

### find_similar_pages

**Source**: `src/web_search.rs`

```rust
pub async fn find_similar_pages( source_url: String, max_results: Option<u32>, include_content: Option<bool>, similarity_threshold: Option<f64>, ) -> crate::error::Result<SimilarPagesResult>
```

**Documentation:**

Search for pages similar to a given URL

This tool finds web pages that are similar in content and topic to a source URL,
useful for finding related information or alternative perspectives.

---

### summarize_web_content

**Source**: `src/web_search.rs`

```rust
pub async fn summarize_web_content( urls: Vec<String>, summary_length: Option<String>, // "brief", "detailed", "comprehensive" focus_topics: Option<Vec<String>>, _include_quotes: Option<bool>, ) -> crate::error::Result<Vec<ContentSummary>>
```

**Documentation:**

Summarize content from multiple web pages

This tool extracts and summarizes key information from multiple web pages,
creating a comprehensive overview of a topic from multiple sources.

---

### search_recent_news

**Source**: `src/web_search.rs`

```rust
pub async fn search_recent_news( topic: String, time_window: Option<String>, // "24h", "week", "month" source_types: Option<Vec<String>>, // "news", "blog", "social" max_results: Option<u32>, include_analysis: Option<bool>, ) -> crate::error::Result<WebSearchResult>
```

**Documentation:**

Search for recent news and articles on a topic

This tool specifically searches for recent news articles and blog posts,
optimized for finding current information and trending discussions.

---


---

*This documentation was automatically generated from the source code.*