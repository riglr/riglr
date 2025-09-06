# riglr-web-tools

Comprehensive web data tools for riglr agents, bridging on-chain and off-chain intelligence with production-grade integrations for social media, market data, news, and web search.

## Features

- 🐦 **Twitter/X Integration**: Real-time social sentiment analysis and trend monitoring
- 📊 **DexScreener Market Data**: Token metrics, trading pairs, and market analysis
- 🔍 **Intelligent Web Search**: Semantic search using Exa API for contextual results
- 📰 **News Aggregation**: Cryptocurrency news monitoring and sentiment analysis
- ⚡ **High Performance**: Async/await with built-in rate limiting and caching
- 🛡️ **Error Resilience**: Distinguishes between retriable and permanent failures
- 🔐 **Secure API Management**: Environment-based configuration for API keys

## Architecture

riglr-web-tools provides off-chain data integration for riglr agents, bridging web2 and web3 data sources with structured tools for market intelligence.

### Design Principles

- **API Abstraction**: Uniform interface across different data providers
- **Rate Limit Aware**: Built-in rate limiting and backoff strategies
- **Caching First**: In-memory caching to reduce API calls and costs
- **Error Classification**: Distinguishes retriable, permanent, and rate-limited errors
- **Type-Safe Responses**: Strongly typed data models for all API responses
- **Tool-Ready**: All functions use `#[tool]` macro for agent integration

### Core Components

1. **Twitter Module**: Social media monitoring and sentiment analysis
2. **DexScreener Module**: DEX market data and token analytics
3. **Exa Search Module**: Semantic web search with AI-powered relevance
4. **News Module**: Crypto news aggregation and analysis
5. **Common Utilities**: Shared HTTP client, rate limiting, error handling

### Data Flow

```
External API → Rate Limiter → HTTP Client → Response Parser → Cache → Tool Result
                    ↓              ↓             ↓
              Retry Logic    Error Mapping   Type Validation
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
riglr-web-tools = "0.1.0"
```

## Quick Start

### Setting up API Keys

Configure your environment variables:

```bash
# Twitter/X API access
export TWITTER_BEARER_TOKEN=your_twitter_bearer_token

# Exa web search API
export EXA_API_KEY=your_exa_api_key

# Optional: DexScreener (no auth required by default)
export DEXSCREENER_BASE_URL=https://api.dexscreener.com/latest

# Optional: News API endpoints
export NEWS_API_URL=https://api.yournewsprovider.com
export NEWS_API_KEY=your_news_api_key
```

### Basic Usage

```rust
use riglr_web_tools::{search_tweets, get_token_info, search_web};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Search Twitter for crypto sentiment
    let tweets = search_tweets(
        "$SOL OR Solana".to_string(),
        Some(100),        // Max results
        Some(true),       // Include sentiment analysis
        Some("en".to_string()),
        None,
        None,
    ).await?;
    
    println!("Found {} tweets about Solana", tweets.tweets.len());
    
    // Get token information from DexScreener
    let token = get_token_info(
        "So11111111111111111111111111111111111112".to_string(), // SOL address
        Some("solana".to_string()), // Chain ID
        None,  // Include pairs (defaults to true)
        None,  // Include security info
    ).await?;
    
    println!("SOL price: ${:.2}", token.price_usd.unwrap_or(0.0));
    
    // Search the web for market analysis
    let search_results = search_web(
        "Solana ecosystem growth 2024".to_string(),
        Some(10),         // Max results
        Some(true),       // Include content
        None,             // Domain filter
        None,             // Date filter
        None,             // Content type filter
    ).await?;
    
    println!("Found {} relevant articles", search_results.results.len());
    
    Ok(())
}
```

## Available Tools

### Twitter/X Tools

#### `search_tweets`

Search for tweets with advanced filtering and sentiment analysis.

```rust
let result = search_tweets(
    "Bitcoin halving".to_string(),
    Some(200),              // Max results
    Some(true),             // Include sentiment
    Some("en".to_string()), // Language filter
    Some("2024-01-01T00:00:00Z".to_string()), // Start time
    None,
).await?;

println!("Found {} tweets", result.tweets.len());
```

#### `get_user_tweets`

Fetch recent tweets from a specific user.

```rust
let tweets = get_user_tweets(
    "VitalikButerin".to_string(),
    Some(50),    // Max results
    Some(false), // Exclude replies
    Some(false), // Exclude retweets
).await?;
```

#### `analyze_crypto_sentiment`

Comprehensive sentiment analysis for cryptocurrency tokens.

```rust
let sentiment = analyze_crypto_sentiment(
    "ETH".to_string(),
    Some(24),   // Time window in hours
    Some(100),  // Minimum engagement threshold
).await?;

println!("ETH sentiment: {:.2} ({:.1}% positive)", 
    sentiment.overall_sentiment,
    sentiment.sentiment_breakdown.positive_pct
);
```

### DexScreener Tools

#### `get_token_info`

Get detailed token information and metrics.

```rust
let token = get_token_info(
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
    Some("ethereum".to_string()),  // Chain ID
    None,                           // Include pairs
    None,                           // Include security
).await?;

println!("Market cap: ${:.2}M", token.market_cap.unwrap_or(0.0) / 1_000_000.0);
```

#### `search_tokens`

Search for tokens by name or symbol.

```rust
let results = search_tokens(
    "pepe".to_string(),
    None,       // Chain filter
    None,       // Min market cap
    None,       // Min liquidity
    Some(10),   // Limit
).await?;
```

#### `get_trending_tokens`

Get currently trending tokens across chains.

```rust
let trending = get_trending_tokens(
    Some("1h".to_string()),        // Time window
    Some("solana".to_string()),    // Chain filter
    None,                           // Min volume
    Some(20),                       // Limit
).await?;
```

#### `analyze_token_market`

Deep market analysis for a specific token.

```rust
let analysis = analyze_token_market(
    "bonk".to_string(),             // Token address
    Some("solana".to_string()),     // Chain ID
    None,                           // Include technical
    Some(true),                     // Include risk
).await?;
```

#### `get_top_pairs`

Get top trading pairs by volume or liquidity.

```rust
let pairs = get_top_pairs(
    Some("24h".to_string()),        // Time window
    Some("ethereum".to_string()),   // Chain filter
    None,                           // DEX filter
    None,                           // Min liquidity
    Some(50),                       // Limit
).await?;
```

### News Tools

#### `get_crypto_news`

Fetch latest cryptocurrency news.

```rust
let news = get_crypto_news(
    "bitcoin".to_string(),             // Topic (required)
    Some("24h".to_string()),           // Time window
    None,                              // Source types
    Some(70),                          // Min credibility score
    Some(true),                        // Include analysis
).await?;

for article in news.articles {
    println!("📰 {} - {}", article.title, article.source);
}
```

#### `get_trending_news`

Get currently trending news articles.

```rust
let trending = get_trending_news(
    Some("defi".to_string()),
    Some(20),
).await?;
```

#### `monitor_breaking_news`

Monitor for breaking news with alerts.

```rust
let breaking = monitor_breaking_news(
    vec!["hack".to_string(), "exploit".to_string()],
    Some(60), // Check interval in seconds
).await?;

if !breaking.alerts.is_empty() {
    println!("⚠️ BREAKING: {}", breaking.alerts[0].title);
}
```

#### `analyze_market_sentiment`

Aggregate sentiment analysis across news sources.

```rust
let sentiment = analyze_market_sentiment(
    Some("ethereum".to_string()),
    Some(48), // Time window in hours
).await?;

println!("Market sentiment: {} ({:.2})", 
    sentiment.overall_sentiment,
    sentiment.score
);
```

### Web Search Tools

#### `search_web`

Intelligent semantic web search using Exa API.

```rust
let results = search_web(
    "DeFi yield farming strategies 2024".to_string(),
    Some(20),
    Some("neural".to_string()), // Search type
    Some(true), // Include text content
).await?;

for result in results.results {
    println!("📄 {}: {}", result.title, result.url);
}
```

#### `find_similar_pages`

Find pages similar to a given URL.

```rust
let similar = find_similar_pages(
    "https://defillama.com".to_string(),
    Some(10),
    Some(vec!["defi".to_string(), "analytics".to_string()]),
).await?;
```

#### `summarize_web_content`

Get AI-powered summaries of web pages.

```rust
let summary = summarize_web_content(
    "https://ethereum.org/en/roadmap/".to_string(),
    Some(500), // Max summary length
).await?;

println!("Summary: {}", summary.summary);
```

#### `search_recent_news`

Search for recent news articles.

```rust
let news = search_recent_news(
    "cryptocurrency regulation".to_string(),
    Some("week".to_string()),
    Some(50),
).await?;
```

## Advanced Usage

### Combining Tools for Market Intelligence

```rust
use riglr_web_tools::*;

async fn comprehensive_token_analysis(symbol: &str) -> anyhow::Result<MarketInsight> {
    // Get token metrics
    let token_info = get_token_info(symbol.to_string(), None).await?;
    
    // Analyze social sentiment
    let twitter_sentiment = analyze_crypto_sentiment(
        symbol.to_string(),
        Some(24),
        Some(50),
    ).await?;
    
    // Get market news
    let news = get_crypto_news(
        Some(symbol.to_string()),
        Some(20),
        Some(24),
    ).await?;
    
    // Search for analysis articles
    let web_analysis = search_web(
        format!("{} technical analysis price prediction", symbol),
        Some(10),
        Some("neural".to_string()),
        None,
    ).await?;
    
    // Combine insights
    Ok(MarketInsight {
        token: token_info,
        social_sentiment: twitter_sentiment.overall_sentiment,
        news_sentiment: news.sentiment_score,
        web_mentions: web_analysis.results.len(),
        recommendation: calculate_recommendation(/* ... */),
    })
}
```

### Integration with rig

All tools are compatible with the rig agent framework:

```rust
use rig::agents::Agent;
use riglr_web_tools::{search_tweets, get_token_info, search_web};

let market_analyst = Agent::builder()
    .preamble("You are a cryptocurrency market analyst with access to real-time data.")
    .tool(search_tweets)
    .tool(get_token_info)
    .tool(search_web)
    .tool(analyze_crypto_sentiment)
    .tool(get_crypto_news)
    .build();

let analysis = market_analyst
    .prompt("Analyze the current market sentiment for Ethereum and provide insights.")
    .await?;
```

## Error Handling

All tools use `ToolError` to distinguish between retriable and permanent errors:

```rust
use riglr_core::ToolError;

match get_token_info(address, None, None, None).await {
    Ok(info) => println!("Success: {}", info.symbol),
    Err(ToolError::Retriable(msg)) => {
        // Network error, timeout, rate limit - can retry
        println!("Temporary error (will retry): {}", msg);
    }
    Err(ToolError::Permanent(msg)) => {
        // Invalid input, auth error - don't retry
        println!("Permanent error: {}", msg);
    }
}
```

## Rate Limiting

The tools automatically handle rate limiting:

- **DexScreener**: 300 requests/minute (no auth required)
- **Twitter/X**: Based on your API tier
- **NewsAPI**: Based on your subscription
- **Exa**: Based on your plan

## Testing

```bash
# Run all tests
cargo test -p riglr-web-tools

# Run with integration tests (requires API keys)
cargo test -p riglr-web-tools --features integration-tests

# Run specific test suite
cargo test -p riglr-web-tools dexscreener_tests
```

## Security

- API keys are never exposed to the agent's reasoning context
- All URLs and parameters are validated
- Automatic retry logic for transient failures
- Rate limit compliance to prevent API bans

## License

MIT

## Contributing

Contributions are welcome! Please ensure all tests pass and add new tests for any new functionality.
