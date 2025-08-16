# riglr-web-tools API Reference

Comprehensive API documentation for the `riglr-web-tools` crate.

## Table of Contents

### Structs

- [`AggregationMetadata`](#aggregationmetadata)
- [`ApiKeys`](#apikeys)
- [`BaseUrls`](#baseurls)
- [`BreakingNewsAlert`](#breakingnewsalert)
- [`ChainInfo`](#chaininfo)
- [`ClientConfig`](#clientconfig)
- [`ConcentrationRisk`](#concentrationrisk)
- [`ContentEntity`](#contententity)
- [`ContentSummary`](#contentsummary)
- [`ContentType`](#contenttype)
- [`ContextAnnotation`](#contextannotation)
- [`DexInfo`](#dexinfo)
- [`DexScreenerConfig`](#dexscreenerconfig)
- [`DexScreenerResponse`](#dexscreenerresponse)
- [`DomainInfo`](#domaininfo)
- [`EmotionalIndicators`](#emotionalindicators)
- [`EntityMention`](#entitymention)
- [`EntityMention`](#entitymention)
- [`Faster100xConfig`](#faster100xconfig)
- [`HolderActivity`](#holderactivity)
- [`HolderDistribution`](#holderdistribution)
- [`HolderTrendPoint`](#holdertrendpoint)
- [`HolderTrends`](#holdertrends)
- [`HttpConfig`](#httpconfig)
- [`InfluencerMention`](#influencermention)
- [`InfluencerMentionsResult`](#influencermentionsresult)
- [`Liquidity`](#liquidity)
- [`LiquidityAnalysis`](#liquidityanalysis)
- [`LunarCrushConfig`](#lunarcrushconfig)
- [`MarketAnalysis`](#marketanalysis)
- [`MarketImpact`](#marketimpact)
- [`NewsAggregationResult`](#newsaggregationresult)
- [`NewsArticle`](#newsarticle)
- [`NewsCategory`](#newscategory)
- [`NewsConfig`](#newsconfig)
- [`NewsEntity`](#newsentity)
- [`NewsInsights`](#newsinsights)
- [`NewsSentiment`](#newssentiment)
- [`NewsSource`](#newssource)
- [`PageMetadata`](#pagemetadata)
- [`PairInfo`](#pairinfo)
- [`PairToken`](#pairtoken)
- [`PriceChange`](#pricechange)
- [`PriceLevelAnalysis`](#pricelevelanalysis)
- [`QualityMetrics`](#qualitymetrics)
- [`RateLimitInfo`](#ratelimitinfo)
- [`RateLimits`](#ratelimits)
- [`RiskAssessment`](#riskassessment)
- [`RiskFactor`](#riskfactor)
- [`SearchInsights`](#searchinsights)
- [`SearchMetadata`](#searchmetadata)
- [`SearchMetadata`](#searchmetadata)
- [`SearchResult`](#searchresult)
- [`SearchSentiment`](#searchsentiment)
- [`SecurityInfo`](#securityinfo)
- [`SentimentAnalysis`](#sentimentanalysis)
- [`SentimentBreakdown`](#sentimentbreakdown)
- [`SentimentData`](#sentimentdata)
- [`SentimentDistribution`](#sentimentdistribution)
- [`SentimentPhrase`](#sentimentphrase)
- [`SeoMetadata`](#seometadata)
- [`SimilarPagesResult`](#similarpagesresult)
- [`SimilarityMetadata`](#similaritymetadata)
- [`SocialLink`](#sociallink)
- [`SocialMetadata`](#socialmetadata)
- [`SocialMetrics`](#socialmetrics)
- [`SourceDiversity`](#sourcediversity)
- [`Token`](#token)
- [`TokenHolderAnalysis`](#tokenholderanalysis)
- [`TokenInfo`](#tokeninfo)
- [`TokenPair`](#tokenpair)
- [`TokenPriceResult`](#tokenpriceresult)
- [`TokenSearchResult`](#tokensearchresult)
- [`TransactionStats`](#transactionstats)
- [`TransactionStats`](#transactionstats)
- [`Transactions`](#transactions)
- [`TrendAnalysis`](#trendanalysis)
- [`TrendingCrypto`](#trendingcrypto)
- [`TrendingTopic`](#trendingtopic)
- [`TweetEntities`](#tweetentities)
- [`TweetMetrics`](#tweetmetrics)
- [`TwitterConfig`](#twitterconfig)
- [`TwitterPost`](#twitterpost)
- [`TwitterSearchResult`](#twittersearchresult)
- [`TwitterUser`](#twitteruser)
- [`Volume`](#volume)
- [`VolumeAnalysis`](#volumeanalysis)
- [`WalletHolding`](#walletholding)
- [`WebClient`](#webclient)
- [`WebSearchConfig`](#websearchconfig)
- [`WebSearchMetadata`](#websearchmetadata)
- [`WebSearchResult`](#websearchresult)
- [`WhaleActivity`](#whaleactivity)
- [`WhaleTransaction`](#whaletransaction)

### Functions

- [`contains_key`](#contains_key)
- [`delete`](#delete)
- [`find_best_liquidity_pair`](#find_best_liquidity_pair)
- [`get`](#get)
- [`get`](#get)
- [`get`](#get)
- [`get_api_key`](#get_api_key)
- [`get_config`](#get_config)
- [`get_pair_by_address`](#get_pair_by_address)
- [`get_pairs_by_token`](#get_pairs_by_token)
- [`get_token_price`](#get_token_price)
- [`get_with_headers`](#get_with_headers)
- [`get_with_params`](#get_with_params)
- [`get_with_params_and_headers`](#get_with_params_and_headers)
- [`insert`](#insert)
- [`insert`](#insert)
- [`is_empty`](#is_empty)
- [`is_empty`](#is_empty)
- [`len`](#len)
- [`len`](#len)
- [`new`](#new)
- [`post`](#post)
- [`post_with_headers`](#post_with_headers)
- [`search_ticker`](#search_ticker)
- [`set_config`](#set_config)
- [`with_api_key`](#with_api_key)
- [`with_config`](#with_config)
- [`with_dexscreener_key`](#with_dexscreener_key)
- [`with_exa_key`](#with_exa_key)
- [`with_news_api_key`](#with_news_api_key)
- [`with_twitter_token`](#with_twitter_token)

### Tools

- [`analyze_crypto_sentiment`](#analyze_crypto_sentiment)
- [`analyze_market_sentiment`](#analyze_market_sentiment)
- [`analyze_token_holders`](#analyze_token_holders)
- [`analyze_token_market`](#analyze_token_market)
- [`find_similar_pages`](#find_similar_pages)
- [`get_crypto_news`](#get_crypto_news)
- [`get_holder_trends`](#get_holder_trends)
- [`get_influencer_mentions`](#get_influencer_mentions)
- [`get_social_sentiment`](#get_social_sentiment)
- [`get_token_info`](#get_token_info)
- [`get_token_price`](#get_token_price)
- [`get_token_prices_batch`](#get_token_prices_batch)
- [`get_top_pairs`](#get_top_pairs)
- [`get_trending_cryptos`](#get_trending_cryptos)
- [`get_trending_news`](#get_trending_news)
- [`get_trending_tokens`](#get_trending_tokens)
- [`get_user_tweets`](#get_user_tweets)
- [`get_whale_activity`](#get_whale_activity)
- [`monitor_breaking_news`](#monitor_breaking_news)
- [`search_recent_news`](#search_recent_news)
- [`search_tokens`](#search_tokens)
- [`search_tweets`](#search_tweets)
- [`search_web`](#search_web)
- [`summarize_web_content`](#summarize_web_content)

### Enums

- [`WebToolError`](#webtoolerror)

### Constants

- [`VERSION`](#version)

## Structs

### AggregationMetadata

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct AggregationMetadata { /// Total articles found across all sources pub total_articles: u32, /// Articles returned after filtering pub returned_articles: u32, /// Sources queried pub sources_queried: Vec<String>, /// Average credibility of returned articles pub avg_credibility: f64, /// Time range covered pub time_range_hours: u32, /// Duplicate articles removed pub duplicates_removed: u32, }
```

Metadata about the news aggregation process

---

### ApiKeys

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct ApiKeys { /// Twitter/X Bearer Token pub twitter: Option<String>, /// Exa API key pub exa: Option<String>, /// DexScreener API key (optional)
```

Type-safe API keys configuration

---

### BaseUrls

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct BaseUrls { pub dexscreener: String, pub exa: String, pub newsapi: String, pub cryptopanic: String, pub lunarcrush: String, pub twitter: String, }
```

Base URL configuration for various services

---

### BreakingNewsAlert

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct BreakingNewsAlert { /// Alert ID pub id: String, /// Alert severity (Critical, High, Medium, Low)
```

Breaking news alert

---

### ChainInfo

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct ChainInfo { /// Chain identifier (e.g., "ethereum", "bsc", "polygon")
```

Blockchain/chain information

---

### ClientConfig

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct ClientConfig { /// Base URL overrides for testing pub base_urls: BaseUrls, /// Rate limiting settings pub rate_limits: RateLimits, }
```

Type-safe client configuration

---

### ConcentrationRisk

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct ConcentrationRisk { /// Risk level (Low, Medium, High, Critical)
```

Concentration risk analysis

---

### ContentEntity

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct ContentEntity { /// Entity name pub name: String, /// Entity type (Person, Organization, Location, etc.)
```

Entity found in content

---

### ContentSummary

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct ContentSummary { /// URL of the page pub url: String, /// Page title pub title: String, /// Executive summary (2-3 sentences)
```

Content summary with key points

---

### ContentType

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct ContentType { /// Primary content type (Article, Blog, News, Academic, etc.)
```

Content type and format information

---

### ContextAnnotation

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct ContextAnnotation { /// Domain ID pub domain_id: String, /// Domain name pub domain_name: String, /// Entity ID pub entity_id: String, /// Entity name pub entity_name: String, }
```

Context annotation for tweet topics

---

### DexInfo

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct DexInfo { /// DEX identifier pub id: String, /// DEX name pub name: String, /// DEX URL pub url: Option<String>, /// DEX logo URL pub logo: Option<String>, }
```

DEX platform information

---

### DexScreenerConfig

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct DexScreenerConfig { /// API base URL (default: https://api.dexscreener.com/latest)
```

Configuration for DexScreener API access

---

### DexScreenerResponse

**Source**: `src/dexscreener_api.rs`

**Attributes**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
```

```rust
pub struct DexScreenerResponse { #[serde(rename = "schemaVersion")]
```

---

### DomainInfo

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct DomainInfo { /// Domain name (e.g., "techcrunch.com")
```

Domain information for a search result

---

### EmotionalIndicators

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct EmotionalIndicators { /// Fear level (0.0 to 1.0)
```

Emotional indicators in news content

---

### EntityMention

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct EntityMention { /// Entity name pub name: String, /// Number of mentions across articles pub mention_count: u32, /// Average sentiment towards entity pub avg_sentiment: f64, /// Entity type pub entity_type: String, /// Trending status pub is_trending: bool, }
```

Entity mention statistics

---

### EntityMention

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct EntityMention { /// Entity name (e.g., "Bitcoin", "Ethereum")
```

Entity mention in sentiment analysis

---

### Faster100xConfig

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct Faster100xConfig { pub api_key: String, /// API base URL (default: https://api.faster100x.com/v1)
```

Faster100x API configuration

---

### HolderActivity

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HolderActivity { /// New holders in last 24h pub new_holders_24h: u64, /// Holders who sold in last 24h pub exited_holders_24h: u64, /// Net holder change in 24h pub net_holder_change_24h: i64, /// Average buy size in last 24h (USD)
```

Recent holder activity metrics

---

### HolderDistribution

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HolderDistribution { /// Percentage held by top 1% of holders pub top_1_percent: f64, /// Percentage held by top 5% of holders pub top_5_percent: f64, /// Percentage held by top 10% of holders pub top_10_percent: f64, /// Percentage held by whale wallets (>1% of supply)
```

Holder distribution breakdown

---

### HolderTrendPoint

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HolderTrendPoint { /// Data point timestamp pub timestamp: DateTime<Utc>, /// Total holders at this time pub total_holders: u64, /// Holder change from previous point pub holder_change: i64, /// Token price at this time pub token_price: f64, /// Whale percentage at this time pub whale_percentage: f64, /// Trading volume at this time pub volume_24h: f64, }
```

Individual holder trend data point

---

### HolderTrends

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HolderTrends { /// Token address pub token_address: String, /// Time series data points pub data_points: Vec<HolderTrendPoint>, /// Overall trend direction pub trend_direction: String, // "increasing", "decreasing", "stable" /// Trend strength (0-100)
```

Holder trends over time

---

### HttpConfig

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct HttpConfig { /// Request timeout pub timeout: Duration, /// Maximum retries pub max_retries: u32, /// Retry delay pub retry_delay: Duration, /// User agent pub user_agent: String, /// Enable exponential backoff pub exponential_backoff: bool, /// Jitter for retry delays (0.0 to 1.0)
```

Configuration for HTTP client

---

### InfluencerMention

**Source**: `src/lunarcrush.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct InfluencerMention { /// Unique mention ID pub id: String, /// Influencer username pub influencer_username: String, /// Influencer display name pub influencer_name: String, /// Number of followers pub followers: u64, /// Post content/text pub text: String, /// Post timestamp pub timestamp: DateTime<Utc>, /// Platform (Twitter, Reddit, etc.)
```

Influencer mention data

---

### InfluencerMentionsResult

**Source**: `src/lunarcrush.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct InfluencerMentionsResult { /// Token symbol that was searched pub symbol: String, /// List of influencer mentions pub mentions: Vec<InfluencerMention>, /// Total number of mentions found pub total_mentions: u64, /// Timeframe of the search pub timeframe: String, /// Average sentiment across all mentions pub avg_sentiment: f64, }
```

Result containing multiple influencer mentions

---

### Liquidity

**Source**: `src/dexscreener_api.rs`

**Attributes**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
```

```rust
pub struct Liquidity { pub usd: Option<f64>, pub base: Option<f64>, pub quote: Option<f64>, }
```

---

### LiquidityAnalysis

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct LiquidityAnalysis { /// Total liquidity across all pairs pub total_liquidity_usd: f64, /// Liquidity distribution across DEXs pub dex_distribution: HashMap<String, f64>, /// Price impact for different trade sizes pub price_impact: HashMap<String, f64>, // "1k", "10k", "100k" -> impact % /// Liquidity depth score (1-100)
```

---

### LunarCrushConfig

**Source**: `src/lunarcrush.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct LunarCrushConfig { pub api_key: String, /// API base URL (default: https://api.lunarcrush.com/v2)
```

LunarCrush API configuration

---

### MarketAnalysis

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct MarketAnalysis { /// Token being analyzed pub token: TokenInfo, /// Market trend analysis pub trend_analysis: TrendAnalysis, /// Volume analysis pub volume_analysis: VolumeAnalysis, /// Liquidity analysis pub liquidity_analysis: LiquidityAnalysis, /// Price level analysis pub price_levels: PriceLevelAnalysis, /// Risk assessment pub risk_assessment: RiskAssessment, /// Analysis timestamp pub analyzed_at: DateTime<Utc>, }
```

Market analysis result

---

### MarketImpact

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct MarketImpact { /// Predicted impact level (High, Medium, Low, Negligible)
```

Market impact assessment for news

---

### NewsAggregationResult

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct NewsAggregationResult { /// Search query or topic pub topic: String, /// Found news articles pub articles: Vec<NewsArticle>, /// Aggregation metadata pub metadata: AggregationMetadata, /// Market insights from the news pub insights: NewsInsights, /// Trending topics extracted pub trending_topics: Vec<TrendingTopic>, /// Aggregation timestamp pub aggregated_at: DateTime<Utc>, }
```

Comprehensive news aggregation result

---

### NewsArticle

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct NewsArticle { /// Unique article identifier pub id: String, /// Article title pub title: String, /// Article URL pub url: String, /// Article description/summary pub description: Option<String>, /// Full article content (if extracted)
```

Comprehensive news article with metadata and analysis

---

### NewsCategory

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct NewsCategory { /// Primary category (Breaking, Analysis, Opinion, etc.)
```

News category and classification

---

### NewsConfig

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct NewsConfig { /// NewsAPI.org API key pub newsapi_key: String, /// CryptoPanic API key pub cryptopanic_key: String, /// Base URL for news aggregation service pub base_url: String, /// Maximum articles per request (default: 50)
```

Configuration for news aggregation services

---

### NewsEntity

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct NewsEntity { /// Entity name pub name: String, /// Entity type (Person, Company, Cryptocurrency, etc.)
```

Entities mentioned in news (people, companies, assets)

---

### NewsInsights

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct NewsInsights { /// Overall market sentiment from news pub overall_sentiment: f64, /// Sentiment trend over time pub sentiment_trend: String, // "Improving", "Declining", "Stable" /// Most mentioned entities pub top_entities: Vec<EntityMention>, /// Dominant themes/topics pub dominant_themes: Vec<String>, /// Geographical distribution of news pub geographic_distribution: HashMap<String, u32>, /// Source diversity metrics pub source_diversity: SourceDiversity, /// Market impact distribution pub impact_distribution: HashMap<String, u32>, }
```

Insights extracted from news aggregation

---

### NewsSentiment

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct NewsSentiment { /// Overall sentiment score (-1.0 to 1.0)
```

Sentiment analysis for news article

---

### NewsSource

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct NewsSource { /// Source identifier pub id: String, /// Source name (e.g., "CoinDesk", "Reuters")
```

News source information and credibility

---

### PageMetadata

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct PageMetadata { /// Author name(s)
```

Page metadata extracted from HTML

---

### PairInfo

**Source**: `src/dexscreener_api.rs`

**Attributes**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
```

```rust
pub struct PairInfo { #[serde(rename = "chainId")]
```

---

### PairToken

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct PairToken { /// Token contract address pub address: String, /// Token name pub name: String, /// Token symbol pub symbol: String, }
```

Token information within a pair

---

### PriceChange

**Source**: `src/dexscreener_api.rs`

**Attributes**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
```

```rust
pub struct PriceChange { #[serde(default)]
```

---

### PriceLevelAnalysis

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct PriceLevelAnalysis { /// All-time high price pub ath: Option<f64>, /// All-time low price pub atl: Option<f64>, /// Distance from ATH (percentage)
```

Price level analysis

---

### QualityMetrics

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct QualityMetrics { /// Overall quality score (0-100)
```

Article quality assessment metrics

---

### RateLimitInfo

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct RateLimitInfo { /// Requests remaining in current window pub remaining: u32, /// Total requests allowed per window pub limit: u32, /// When the rate limit resets (Unix timestamp)
```

Rate limit information

---

### RateLimits

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct RateLimits { pub dexscreener_per_minute: u32, pub twitter_per_minute: u32, pub newsapi_per_minute: u32, pub exa_per_minute: u32, }
```

Rate limiting configuration

---

### RiskAssessment

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct RiskAssessment { /// Overall risk level (Low, Medium, High, Extreme)
```

---

### RiskFactor

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct RiskFactor { /// Risk category pub category: String, /// Risk description pub description: String, /// Severity (Low, Medium, High)
```

Individual risk factor

---

### SearchInsights

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SearchInsights { /// Most common topics/themes found pub common_topics: Vec<String>, /// Publication date distribution pub date_distribution: HashMap<String, u32>, // "last_week", "last_month", etc. /// Content type distribution pub content_types: HashMap<String, u32>, /// Average content quality score pub avg_quality_score: Option<f64>, /// Language distribution pub languages: HashMap<String, u32>, /// Sentiment analysis (if performed)
```

Aggregated insights from search results

---

### SearchMetadata

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SearchMetadata { /// Number of results found pub result_count: u32, /// Search execution time (ms)
```

Metadata for search results

---

### SearchMetadata

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SearchMetadata { /// Total number of tweets found pub result_count: u32, /// Search query used pub query: String, pub next_token: Option<String>, /// Search timestamp pub searched_at: DateTime<Utc>, }
```

Metadata for Twitter search results

---

### SearchResult

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SearchResult { /// Unique result identifier pub id: String, /// Page title pub title: String, /// Page URL pub url: String, /// Page description/snippet pub description: Option<String>, /// Extracted text content pub content: Option<String>, /// Content summary (if processed)
```

Comprehensive search result with content and metadata

---

### SearchSentiment

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SearchSentiment { /// Overall sentiment score (-1.0 to 1.0)
```

Sentiment analysis of search results

---

### SecurityInfo

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SecurityInfo { pub is_verified: bool, /// Whether liquidity is locked pub liquidity_locked: Option<bool>, /// Contract audit status pub audit_status: Option<String>, /// Honeypot detection result pub honeypot_status: Option<String>, /// Contract ownership status pub ownership_status: Option<String>, /// Risk score (0-100, lower is better)
```

Token security and verification information

---

### SentimentAnalysis

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SentimentAnalysis { /// Overall sentiment score (-1.0 to 1.0)
```

Sentiment analysis result for tweets

---

### SentimentBreakdown

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SentimentBreakdown { /// Percentage of positive tweets pub positive_pct: f64, /// Percentage of neutral tweets pub neutral_pct: f64, /// Percentage of negative tweets pub negative_pct: f64, /// Average engagement for positive tweets pub positive_avg_engagement: f64, /// Average engagement for negative tweets pub negative_avg_engagement: f64, }
```

Breakdown of sentiment scores

---

### SentimentData

**Source**: `src/lunarcrush.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SentimentData { /// Cryptocurrency symbol (e.g., "BTC", "ETH")
```

Social sentiment data for a cryptocurrency

---

### SentimentDistribution

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SentimentDistribution { /// Percentage of positive results pub positive_pct: f64, /// Percentage of neutral results pub neutral_pct: f64, /// Percentage of negative results pub negative_pct: f64, }
```

Distribution of sentiment across results

---

### SentimentPhrase

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SentimentPhrase { /// The phrase text pub phrase: String, /// Sentiment contribution (-1.0 to 1.0)
```

Key phrases contributing to sentiment

---

### SeoMetadata

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SeoMetadata { /// Meta description pub meta_description: Option<String>, /// Meta keywords pub meta_keywords: Vec<String>, /// Page robots directive pub robots: Option<String>, /// Schema.org structured data types found pub schema_types: Vec<String>, }
```

SEO-related metadata

---

### SimilarPagesResult

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SimilarPagesResult { /// Source URL used for similarity search pub source_url: String, /// Similar pages found pub similar_pages: Vec<SearchResult>, /// Similarity scores and metadata pub similarity_metadata: SimilarityMetadata, /// Search timestamp pub searched_at: DateTime<Utc>, }
```

Similar page search result

---

### SimilarityMetadata

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SimilarityMetadata { /// Average similarity score pub avg_similarity: f64, /// Similarity calculation method used pub method: String, /// Common themes between source and similar pages pub common_themes: Vec<String>, /// Content overlap analysis pub content_overlap: f64, }
```

Metadata about similarity analysis

---

### SocialLink

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SocialLink { /// Platform name (e.g., "twitter", "telegram", "discord")
```

Social media and community links

---

### SocialMetadata

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SocialMetadata { /// Open Graph title pub og_title: Option<String>, /// Open Graph description pub og_description: Option<String>, /// Open Graph image URL pub og_image: Option<String>, /// Twitter card type pub twitter_card: Option<String>, /// Twitter handle pub twitter_site: Option<String>, }
```

Social media metadata (Open Graph, Twitter Cards)

---

### SocialMetrics

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SocialMetrics { /// Total social shares pub total_shares: u32, /// Twitter mentions/shares pub twitter_shares: u32, /// Reddit discussions pub reddit_mentions: u32, /// LinkedIn shares pub linkedin_shares: u32, /// Social sentiment (different from article sentiment)
```

Social media engagement metrics

---

### SourceDiversity

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct SourceDiversity { /// Number of unique sources pub unique_sources: u32, /// Source type distribution pub source_types: HashMap<String, u32>, /// Geographic source distribution pub geographic_sources: HashMap<String, u32>, /// Credibility distribution pub credibility_distribution: HashMap<String, u32>, // "High", "Medium", "Low" }
```

Source diversity analysis

---

### Token

**Source**: `src/dexscreener_api.rs`

**Attributes**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
```

```rust
pub struct Token { pub address: String, pub name: String, pub symbol: String, }
```

---

### TokenHolderAnalysis

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TokenHolderAnalysis { /// Token contract address pub token_address: String, /// Token symbol pub token_symbol: String, /// Token name pub token_name: String, /// Total number of holders pub total_holders: u64, /// Number of unique holders pub unique_holders: u64, /// Holder distribution metrics pub distribution: HolderDistribution, /// Top holder wallets (anonymized)
```

Token holder analysis data

---

### TokenInfo

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TokenInfo { /// Token contract address pub address: String, /// Token name pub name: String, /// Token symbol pub symbol: String, /// Token decimals pub decimals: u32, /// Current price in USD pub price_usd: Option<f64>, /// Market capitalization in USD pub market_cap: Option<f64>, /// 24h trading volume in USD pub volume_24h: Option<f64>, /// Price change percentage (24h)
```

---

### TokenPair

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TokenPair { /// Unique pair identifier pub pair_id: String, /// DEX name (e.g., "Uniswap V3", "PancakeSwap")
```

Trading pair information

---

### TokenPriceResult

**Source**: `src/price.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TokenPriceResult { /// Token address that was queried pub token_address: String, /// Token symbol pub token_symbol: Option<String>, /// Current price in USD pub price_usd: String, /// DEX where price was sourced pub source_dex: Option<String>, /// Pair address used for pricing pub source_pair: Option<String>, /// Liquidity in USD of the source pair pub source_liquidity_usd: Option<f64>, /// Chain name where token exists pub chain: Option<String>, /// Timestamp of price fetch pub fetched_at: chrono::DateTime<chrono::Utc>, }
```

Price result with additional metadata

---

### TokenSearchResult

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TokenSearchResult { /// Search query used pub query: String, pub tokens: Vec<TokenInfo>, /// Search metadata pub metadata: SearchMetadata, /// Search timestamp pub searched_at: DateTime<Utc>, }
```

---

### TransactionStats

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TransactionStats { /// Number of buy transactions (24h)
```

Transaction statistics for a trading pair

---

### TransactionStats

**Source**: `src/dexscreener_api.rs`

**Attributes**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
```

```rust
pub struct TransactionStats { pub buys: Option<u64>, pub sells: Option<u64>, }
```

---

### Transactions

**Source**: `src/dexscreener_api.rs`

**Attributes**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
```

```rust
pub struct Transactions { #[serde(default)]
```

---

### TrendAnalysis

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TrendAnalysis { /// Overall trend direction (Bullish, Bearish, Neutral)
```

---

### TrendingCrypto

**Source**: `src/lunarcrush.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TrendingCrypto { /// Cryptocurrency symbol pub symbol: String, /// Full name of the cryptocurrency pub name: String, /// Current market rank pub market_rank: u32, /// Galaxy score (LunarCrush proprietary metric)
```

Trending cryptocurrency with social metrics

---

### TrendingTopic

**Source**: `src/news.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TrendingTopic { /// Topic name pub topic: String, /// Number of articles mentioning this topic pub article_count: u32, /// Trend velocity (mentions per hour)
```

Trending topic analysis

---

### TweetEntities

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TweetEntities { /// Hashtags mentioned pub hashtags: Vec<String>, /// User mentions pub mentions: Vec<String>, /// URLs shared pub urls: Vec<String>, /// Cashtags ($SYMBOL)
```

Entities extracted from tweet text

---

### TweetMetrics

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
```

```rust
pub struct TweetMetrics { /// Number of retweets pub retweet_count: u32, /// Number of likes pub like_count: u32, /// Number of replies pub reply_count: u32, /// Number of quotes pub quote_count: u32, /// Number of impressions (if available)
```

Tweet engagement metrics

---

### TwitterConfig

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct TwitterConfig { pub bearer_token: String, /// API base URL (default: https://api.twitter.com/2)
```

Configuration for Twitter API access

---

### TwitterPost

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TwitterPost { /// Tweet ID pub id: String, /// Tweet content/text pub text: String, /// Tweet author information pub author: TwitterUser, /// Tweet creation timestamp pub created_at: DateTime<Utc>, /// Engagement metrics pub metrics: TweetMetrics, /// Entities mentioned in the tweet pub entities: TweetEntities, /// Tweet language code pub lang: Option<String>, /// Whether this is a reply pub is_reply: bool, /// Whether this is a retweet pub is_retweet: bool, /// Context annotations (topics, entities)
```

A Twitter/X post with metadata

---

### TwitterSearchResult

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TwitterSearchResult { /// Found tweets pub tweets: Vec<TwitterPost>, /// Search metadata pub meta: SearchMetadata, /// Rate limit information pub rate_limit_info: RateLimitInfo, }
```

Result of Twitter search operation

---

### TwitterUser

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct TwitterUser { /// User ID pub id: String, /// Username (handle)
```

Twitter user information

---

### Volume

**Source**: `src/dexscreener_api.rs`

**Attributes**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
```

```rust
pub struct Volume { #[serde(default)]
```

---

### VolumeAnalysis

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct VolumeAnalysis { pub volume_rank: Option<u32>, /// Volume trend (Increasing, Decreasing, Stable)
```

---

### WalletHolding

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct WalletHolding { /// Wallet address (potentially anonymized)
```

Individual wallet holding information

---

### WebClient

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct WebClient { /// HTTP client for making requests pub http_client: Client, /// Type-safe API keys pub api_keys: ApiKeys, /// Type-safe configuration pub config: ClientConfig, /// HTTP configuration pub http_config: HttpConfig, }
```

A client for interacting with various web APIs and services

---

### WebSearchConfig

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct WebSearchConfig { /// Exa API key for intelligent search pub exa_api_key: String, /// Exa API base URL (default: https://api.exa.ai)
```

Configuration for web search services

---

### WebSearchMetadata

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct WebSearchMetadata { /// Total results found pub total_results: u32, /// Results returned in this response pub returned_results: u32, /// Search execution time (ms)
```

Metadata about the search operation

---

### WebSearchResult

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct WebSearchResult { /// Search query used pub query: String, /// Search type performed pub search_type: String, /// Found results pub results: Vec<SearchResult>, /// Search metadata pub metadata: WebSearchMetadata, /// Aggregated insights from results pub insights: SearchInsights, /// Search timestamp pub searched_at: DateTime<Utc>, }
```

Complete search operation result

---

### WhaleActivity

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct WhaleActivity { /// Token being analyzed pub token_address: String, /// Whale transactions in the specified timeframe pub whale_transactions: Vec<WhaleTransaction>, /// Total whale buy volume pub total_whale_buys: f64, /// Total whale sell volume pub total_whale_sells: f64, /// Net whale flow (buys - sells)
```

Whale activity tracking data

---

### WhaleTransaction

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct WhaleTransaction { /// Transaction hash pub tx_hash: String, /// Whale wallet address pub wallet_address: String, /// Transaction type (buy/sell)
```

Individual whale transaction

---

## Functions

### contains_key

**Source**: `src/client.rs`

```rust
pub fn contains_key(&self, key: &str) -> bool
```

Check if an API key exists

---

### delete

**Source**: `src/client.rs`

```rust
pub async fn delete(&self, url: &str) -> Result<()>
```

Make a DELETE request

---

### find_best_liquidity_pair

**Source**: `src/dexscreener_api.rs`

```rust
pub fn find_best_liquidity_pair(pairs: Vec<PairInfo>) -> Option<PairInfo>
```

Find the best liquidity pair for a token

---

### get

**Source**: `src/client.rs`

```rust
pub fn get(&self, key: &str) -> Option<&String>
```

Get an API key by name

---

### get

**Source**: `src/client.rs`

```rust
pub fn get(&self, key: &str) -> Option<String>
```

Get a configuration value by key

---

### get

**Source**: `src/client.rs`

```rust
pub async fn get(&self, url: &str) -> Result<String>
```

Make a GET request with retry logic

---

### get_api_key

**Source**: `src/client.rs`

```rust
pub fn get_api_key(&self, service: &str) -> Option<&String>
```

Get API key for a service

---

### get_config

**Source**: `src/client.rs`

```rust
pub fn get_config(&self, key: &str) -> Option<String>
```

Get config value (for backwards compatibility)

---

### get_pair_by_address

**Source**: `src/dexscreener_api.rs`

```rust
pub async fn get_pair_by_address(pair_address: &str) -> Result<PairInfo>
```

Get pairs by pair address

---

### get_pairs_by_token

**Source**: `src/dexscreener_api.rs`

```rust
pub async fn get_pairs_by_token(token_address: &str) -> Result<DexScreenerResponse>
```

Get token pairs by token address

---

### get_token_price

**Source**: `src/dexscreener_api.rs`

```rust
pub fn get_token_price(pairs: &[PairInfo], token_address: &str) -> Option<String>
```

Extract token price from the best pair

---

### get_with_headers

**Source**: `src/client.rs`

```rust
pub async fn get_with_headers( &self, url: &str, headers: HashMap<String, String>, ) -> Result<String>
```

Make a GET request with headers and retry logic

---

### get_with_params

**Source**: `src/client.rs`

```rust
pub async fn get_with_params( &self, url: &str, params: &HashMap<String, String>, ) -> Result<String>
```

Make GET request with query parameters

---

### get_with_params_and_headers

**Source**: `src/client.rs`

```rust
pub async fn get_with_params_and_headers( &self, url: &str, params: &HashMap<String, String>, headers: HashMap<String, String>, ) -> Result<String>
```

Make GET request with query parameters and headers

---

### insert

**Source**: `src/client.rs`

```rust
pub fn insert(&mut self, key: String, value: String)
```

Insert a new API key

---

### insert

**Source**: `src/client.rs`

```rust
pub fn insert(&mut self, key: String, value: String)
```

Insert a configuration value

---

### is_empty

**Source**: `src/client.rs`

```rust
pub fn is_empty(&self) -> bool
```

Check if all API keys are empty

---

### is_empty

**Source**: `src/client.rs`

```rust
pub fn is_empty(&self) -> bool
```

Check if the config is empty

---

### len

**Source**: `src/client.rs`

```rust
pub fn len(&self) -> usize
```

Get the number of configured API keys

---

### len

**Source**: `src/client.rs`

```rust
pub fn len(&self) -> usize
```

Get the number of configuration entries

---

### new

**Source**: `src/client.rs`

```rust
pub fn new() -> Result<Self>
```

Create a new web client

---

### post

**Source**: `src/client.rs`

```rust
pub async fn post<T: Serialize>(&self, url: &str, body: &T) -> Result<serde_json::Value>
```

Make a POST request with JSON body

---

### post_with_headers

**Source**: `src/client.rs`

```rust
pub async fn post_with_headers<T: Serialize>( &self, url: &str, body: &T, headers: HashMap<String, String>, ) -> Result<serde_json::Value>
```

Make a POST request with JSON body and headers

---

### search_ticker

**Source**: `src/dexscreener_api.rs`

```rust
pub async fn search_ticker(ticker: String) -> Result<DexScreenerResponse>
```

Search for tokens or pairs on DexScreener

---

### set_config

**Source**: `src/client.rs`

```rust
pub fn set_config<S: Into<String>>(&mut self, key: S, value: S)
```

Set configuration option (for backwards compatibility)

---

### with_api_key

**Source**: `src/client.rs`

```rust
pub fn with_api_key<S1: Into<String>, S2: Into<String>>( mut self, service: S1, api_key: S2, ) -> Self
```

Set API key for a service (for backwards compatibility)

---

### with_config

**Source**: `src/client.rs`

```rust
pub fn with_config(http_config: HttpConfig) -> Result<Self>
```

Create with custom HTTP configuration

---

### with_dexscreener_key

**Source**: `src/client.rs`

```rust
pub fn with_dexscreener_key<S: Into<String>>(mut self, key: S) -> Self
```

Set DexScreener API key (if required)

---

### with_exa_key

**Source**: `src/client.rs`

```rust
pub fn with_exa_key<S: Into<String>>(mut self, key: S) -> Self
```

Set Exa API key

---

### with_news_api_key

**Source**: `src/client.rs`

```rust
pub fn with_news_api_key<S: Into<String>>(mut self, key: S) -> Self
```

Set News API key

---

### with_twitter_token

**Source**: `src/client.rs`

```rust
pub fn with_twitter_token<S: Into<String>>(mut self, token: S) -> Self
```

Set Twitter/X Bearer Token

---

## Tools

### analyze_crypto_sentiment

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn analyze_crypto_sentiment( token_symbol: String, time_window_hours: Option<u32>, min_engagement: Option<u32>, ) -> crate::error::Result<SentimentAnalysis>
```

Analyze sentiment of cryptocurrency-related tweets

This tool performs comprehensive sentiment analysis on cryptocurrency-related tweets,
providing insights into market mood and social trends.

---

### analyze_market_sentiment

**Source**: `src/news.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn analyze_market_sentiment( time_window: Option<String>, // "1h", "6h", "24h", "week" asset_filter: Option<Vec<String>>, // Specific cryptocurrencies to focus on _source_weights: Option<HashMap<String, f64>>, // Weight different sources _include_social: Option<bool>, ) -> crate::error::Result<NewsInsights>
```

Analyze market sentiment from recent news

This tool provides comprehensive sentiment analysis across recent news articles,
helping to gauge overall market mood and potential price impact.

---

### analyze_token_holders

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn analyze_token_holders( token_address: String, chain: Option<String>, ) -> Result<TokenHolderAnalysis, WebToolError>
```

Analyze token holder distribution and concentration risks

This tool provides comprehensive analysis of a token's holder distribution,
including concentration risks, whale analysis, and holder categorization.

# Arguments
* `token_address` - Token contract address (must be valid hex address)
* `chain` - Blockchain network: "eth", "bsc", "polygon", "arbitrum", "base". Defaults to "eth"

# Returns
Detailed holder analysis including distribution metrics and risk assessment

---

### analyze_token_market

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn analyze_token_market( token_address: String, chain_id: Option<String>, _include_technical: Option<bool>, include_risk: Option<bool>, ) -> crate::error::Result<MarketAnalysis>
```

Analyze token market data using heuristic-based calculations

This tool provides market analysis based on available on-chain data including:
- Simple trend analysis based on price changes
- Volume pattern calculations from 24h data
- Liquidity assessment from DEX pairs
- Basic risk evaluation using heuristic scoring

Note: This is not a substitute for professional financial analysis or
machine learning models. All calculations are rule-based heuristics.

---

### find_similar_pages

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn find_similar_pages( source_url: String, max_results: Option<u32>, include_content: Option<bool>, similarity_threshold: Option<f64>, ) -> crate::error::Result<SimilarPagesResult>
```

Search for pages similar to a given URL

This tool finds web pages that are similar in content and topic to a source URL,
useful for finding related information or alternative perspectives.

---

### get_crypto_news

**Source**: `src/news.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_crypto_news( topic: String, time_window: Option<String>, // "1h", "6h", "24h", "week" source_types: Option<Vec<String>>, // "mainstream", "crypto", "analysis" min_credibility: Option<u32>, include_analysis: Option<bool>, ) -> crate::error::Result<NewsAggregationResult>
```

Get comprehensive cryptocurrency news for a specific topic

This tool aggregates news from multiple sources, performs sentiment analysis,
and assesses market impact for cryptocurrency-related topics.

---

### get_holder_trends

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_holder_trends( token_address: String, period: Option<String>, data_points: Option<u32>, ) -> Result<HolderTrends, WebToolError>
```

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

### get_influencer_mentions

**Source**: `src/lunarcrush.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_influencer_mentions( token_symbol: String, limit: Option<u32>, timeframe: Option<String>, ) -> Result<InfluencerMentionsResult, WebToolError>
```

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

### get_social_sentiment

**Source**: `src/lunarcrush.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_social_sentiment( symbol: String, timeframe: Option<String>, ) -> Result<SentimentData, WebToolError>
```

Get social sentiment data for a cryptocurrency from LunarCrush

This tool provides comprehensive social analytics for any cryptocurrency,
including sentiment scores, social volume, and platform-specific data.

# Arguments
* `symbol` - Cryptocurrency symbol (e.g., "BTC", "ETH", "SOL")
* `timeframe` - Optional timeframe for analysis ("1h", "24h", "7d", "30d"). Defaults to "24h"

# Returns
Detailed sentiment analysis including scores, volume metrics, and trending keywords

---

### get_token_info

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_token_info( token_address: String, chain_id: Option<String>, include_pairs: Option<bool>, include_security: Option<bool>, ) -> crate::error::Result<TokenInfo>
```

Get comprehensive token information from DexScreener

This tool retrieves detailed token information including price, volume,
market cap, trading pairs, and security analysis.

---

### get_token_price

**Source**: `src/price.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_token_price( token_address: String, chain: Option<String>, ) -> Result<TokenPriceResult, ToolError>
```

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

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_token_prices_batch( token_addresses: Vec<String>, chain: Option<String>, ) -> Result<Vec<TokenPriceResult>, ToolError>
```

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

### get_top_pairs

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_top_pairs( time_window: Option<String>, // "5m", "1h", "24h" chain_filter: Option<String>, dex_filter: Option<String>, min_liquidity: Option<f64>, limit: Option<u32>, ) -> crate::error::Result<Vec<TokenPair>>
```

Get top DEX pairs by volume across all chains

This tool retrieves the highest volume trading pairs,
useful for identifying active markets and arbitrage opportunities.

---

### get_trending_cryptos

**Source**: `src/lunarcrush.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_trending_cryptos( limit: Option<u32>, sort_by: Option<String>, ) -> Result<Vec<TrendingCrypto>, WebToolError>
```

Get trending cryptocurrencies by social metrics from LunarCrush

This tool identifies the most trending cryptocurrencies based on social media activity,
sentiment, and LunarCrush's proprietary Galaxy Score.

# Arguments
* `limit` - Maximum number of trending cryptocurrencies to return (1-50). Defaults to 10
* `sort_by` - Sort metric: "galaxy_score", "social_volume", "alt_rank". Defaults to "galaxy_score"

# Returns
List of trending cryptocurrencies with social and market metrics

---

### get_trending_news

**Source**: `src/news.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_trending_news( time_window: Option<String>, // "1h", "6h", "24h" categories: Option<Vec<String>>, // "defi", "nft", "regulation", "tech" min_impact_score: Option<u32>, limit: Option<u32>, ) -> crate::error::Result<NewsAggregationResult>
```

Get trending cryptocurrency news across all topics

This tool identifies currently trending news and topics in the cryptocurrency space,
useful for staying updated on breaking developments and market movements.

---

### get_trending_tokens

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_trending_tokens( time_window: Option<String>, // "5m", "1h", "24h" chain_filter: Option<String>, min_volume: Option<f64>, limit: Option<u32>, ) -> crate::error::Result<Vec<TokenInfo>>
```

Get trending tokens from DexScreener

This tool retrieves trending tokens based on volume,
price changes, and social activity.

---

### get_user_tweets

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_user_tweets( username: String, max_results: Option<u32>, include_replies: Option<bool>, include_retweets: Option<bool>, ) -> crate::error::Result<Vec<TwitterPost>>
```

Get recent tweets from a specific user

This tool fetches recent tweets from a specified Twitter/X user account.

---

### get_whale_activity

**Source**: `src/faster100x.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_whale_activity( token_address: String, timeframe: Option<String>, min_usd_value: Option<f64>, ) -> Result<WhaleActivity, WebToolError>
```

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

### monitor_breaking_news

**Source**: `src/news.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn monitor_breaking_news( keywords: Vec<String>, severity_threshold: Option<String>, // "Critical", "High", "Medium" impact_threshold: Option<u32>, // 0-100 _alert_channels: Option<Vec<String>>, // "webhook", "email", "slack" ) -> crate::error::Result<Vec<BreakingNewsAlert>>
```

Monitor for breaking news and generate real-time alerts

This tool continuously monitors news sources for breaking news
and generates alerts based on severity and market impact criteria.

---

### search_recent_news

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn search_recent_news( topic: String, time_window: Option<String>, // "24h", "week", "month" source_types: Option<Vec<String>>, // "news", "blog", "social" max_results: Option<u32>, include_analysis: Option<bool>, ) -> crate::error::Result<WebSearchResult>
```

Search for recent news and articles on a topic

This tool specifically searches for recent news articles and blog posts,
optimized for finding current information and trending discussions.

---

### search_tokens

**Source**: `src/dexscreener.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn search_tokens( query: String, chain_filter: Option<String>, min_market_cap: Option<f64>, min_liquidity: Option<f64>, limit: Option<u32>, ) -> crate::error::Result<TokenSearchResult>
```

Search for tokens on DexScreener

This tool searches for tokens by name, symbol, or address
with support for filtering by chain and market cap.

---

### search_tweets

**Source**: `src/twitter.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn search_tweets( query: String, max_results: Option<u32>, include_sentiment: Option<bool>, language: Option<String>, start_time: Option<String>, end_time: Option<String>, ) -> crate::error::Result<TwitterSearchResult>
```

Search for tweets matching a query with comprehensive filtering

This tool searches Twitter/X for tweets matching the given query,
with support for advanced filters and sentiment analysis.

---

### search_web

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn search_web( query: String, max_results: Option<u32>, include_content: Option<bool>, domain_filter: Option<Vec<String>>, date_filter: Option<String>, // "day", "week", "month", "year" content_type_filter: Option<String>, // "news", "academic", "blog" ) -> crate::error::Result<WebSearchResult>
```

Perform web search with content extraction

This tool performs web search and returns results with extracted content and metadata.
Uses traditional search APIs rather than semantic understanding.

---

### summarize_web_content

**Source**: `src/web_search.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn summarize_web_content( urls: Vec<String>, summary_length: Option<String>, // "brief", "detailed", "comprehensive" focus_topics: Option<Vec<String>>, _include_quotes: Option<bool>, ) -> crate::error::Result<Vec<ContentSummary>>
```

Summarize content from multiple web pages

This tool extracts and summarizes key information from multiple web pages,
creating a comprehensive overview of a topic from multiple sources.

---

## Enums

### WebToolError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug, IntoToolError)]
```

```rust
pub enum WebToolError { /// Network error (includes HTTP) - automatically retriable #[error("Network error: {0}")] Network(String), /// HTTP request error - automatically retriable (converted to Network) #[error("HTTP error: {0}")] Http(#[from] reqwest::Error), /// API error (includes general API issues) - automatically retriable #[error("API error: {0}")] Api(String), /// API rate limit exceeded - automatically handled as rate_limited #[error("Rate limit exceeded: {0}")] #[tool_error(rate_limited)] RateLimit(String), /// API authentication failed - permanent #[error("Authentication error: {0}")] #[tool_error(permanent)] Auth(String), /// Parsing error (includes JSON and response parsing) - permanent #[error("Parsing error: {0}")] #[tool_error(permanent)] Parsing(String), /// Serialization error - automatically permanent #[error("Serialization error: {0}")] Serialization(#[from] serde_json::Error), /// URL parsing error - permanent #[error("URL error: {0}")] #[tool_error(permanent)] Url(#[from] url::ParseError), /// Configuration error - permanent #[error("Configuration error: {0}")] #[tool_error(permanent)] Config(String), /// Client creation error - permanent #[error("Client error: {0}")] #[tool_error(permanent)] Client(String), /// Invalid input provided - permanent #[error("Invalid input: {0}")] #[tool_error(permanent)] InvalidInput(String), /// Core riglr error #[error("Core error: {0}")] #[tool_error(permanent)] Core(#[from] riglr_core::CoreError), }
```

Main error type for web tool operations.

The IntoToolError derive macro automatically classifies errors:
- Retriable: Network (includes HTTP), Api (includes request errors), RateLimit
- Permanent: Auth, Parsing (includes JSON), Config, Client, InvalidInput

**Variants**:

- `Network(String)`
- `Http(#[from] reqwest::Error)`
- `Api(String)`
- `RateLimit(String)`
- `Auth(String)`
- `Parsing(String)`
- `Serialization(#[from] serde_json::Error)`
- `Url(#[from] url::ParseError)`
- `Config(String)`
- `Client(String)`
- `InvalidInput(String)`
- `Core(#[from] riglr_core::CoreError)`

---

## Constants

### VERSION

**Source**: `src/lib.rs`

```rust
const VERSION: &str
```

Current version of riglr-web-tools

---


---

*This documentation was automatically generated from the source code.*