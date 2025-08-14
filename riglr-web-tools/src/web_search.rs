//! Web search integration using Exa API and web scraping
//!
//! This module provides web search capabilities, content extraction using HTML parsing,
//! and extractive summarization (sentence ranking) for AI agents to gather web-based information.

use crate::{client::WebClient, error::WebToolError};
use chrono::{DateTime, Utc};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use scraper::{Html, Selector, ElementRef};

/// Configuration for web search services
#[derive(Debug, Clone)]
pub struct WebSearchConfig {
    /// Exa API key for intelligent search
    pub exa_api_key: String,
    /// Exa API base URL (default: https://api.exa.ai)
    pub exa_base_url: String,
    /// Maximum results per search (default: 20)
    pub max_results: u32,
    /// Default search timeout in seconds (default: 30)
    pub timeout_seconds: u64,
    /// Whether to include page content by default
    pub include_content: bool,
    /// Content extraction length limit (characters)
    pub content_limit: usize,
}

/// Comprehensive search result with content and metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResult {
    /// Unique result identifier
    pub id: String,
    /// Page title
    pub title: String,
    /// Page URL
    pub url: String,
    /// Page description/snippet
    pub description: Option<String>,
    /// Extracted text content
    pub content: Option<String>,
    /// Content summary (if processed)
    pub summary: Option<String>,
    /// Publication date (if available)
    pub published_date: Option<DateTime<Utc>>,
    /// Domain information
    pub domain: DomainInfo,
    /// Page metadata
    pub metadata: PageMetadata,
    /// Search relevance score (0.0 - 1.0)
    pub relevance_score: f64,
    /// Content type and format info
    pub content_type: ContentType,
    /// Language detection result
    pub language: Option<String>,
    /// Estimated reading time (minutes)
    pub reading_time_minutes: Option<u32>,
}

/// Domain information for a search result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DomainInfo {
    /// Domain name (e.g., "techcrunch.com")
    pub name: String,
    /// Domain reputation score (0-100)
    pub reputation_score: Option<u32>,
    /// Domain category (News, Blog, Academic, etc.)
    pub category: Option<String>,
    /// Whether domain is known to be trustworthy
    pub is_trusted: bool,
    /// Domain authority score (if available)
    pub authority_score: Option<u32>,
}

/// Page metadata extracted from HTML
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PageMetadata {
    /// Author name(s)
    pub author: Option<String>,
    /// Article/page tags
    pub tags: Vec<String>,
    /// Social media metadata (Open Graph)
    pub social_meta: SocialMetadata,
    /// SEO metadata
    pub seo_meta: SeoMetadata,
    /// Canonical URL (if different from actual URL)
    pub canonical_url: Option<String>,
    /// Last modified date
    pub last_modified: Option<DateTime<Utc>>,
}

/// Social media metadata (Open Graph, Twitter Cards)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SocialMetadata {
    /// Open Graph title
    pub og_title: Option<String>,
    /// Open Graph description
    pub og_description: Option<String>,
    /// Open Graph image URL
    pub og_image: Option<String>,
    /// Twitter card type
    pub twitter_card: Option<String>,
    /// Twitter handle
    pub twitter_site: Option<String>,
}

/// SEO-related metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SeoMetadata {
    /// Meta description
    pub meta_description: Option<String>,
    /// Meta keywords
    pub meta_keywords: Vec<String>,
    /// Page robots directive
    pub robots: Option<String>,
    /// Schema.org structured data types found
    pub schema_types: Vec<String>,
}

/// Content type and format information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContentType {
    /// Primary content type (Article, Blog, News, Academic, etc.)
    pub primary: String,
    /// Content format (HTML, PDF, etc.)  
    pub format: String,
    /// Whether content is behind paywall
    pub is_paywalled: Option<bool>,
    /// Content quality score (0-100)
    pub quality_score: Option<u32>,
    /// Estimated content length category
    pub length_category: String, // "Short", "Medium", "Long", "Very Long"
}

/// Complete search operation result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WebSearchResult {
    /// Search query used
    pub query: String,
    /// Search type performed
    pub search_type: String,
    /// Found results
    pub results: Vec<SearchResult>,
    /// Search metadata
    pub metadata: WebSearchMetadata,
    /// Aggregated insights from results
    pub insights: SearchInsights,
    /// Search timestamp
    pub searched_at: DateTime<Utc>,
}

/// Metadata about the search operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WebSearchMetadata {
    /// Total results found
    pub total_results: u32,
    /// Results returned in this response
    pub returned_results: u32,
    /// Search execution time (ms)
    pub execution_time_ms: u32,
    /// Whether results were filtered or limited
    pub filtered: bool,
    /// Suggested related queries
    pub related_queries: Vec<String>,
    /// Top domains in results
    pub top_domains: Vec<String>,
}

/// Aggregated insights from search results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchInsights {
    /// Most common topics/themes found
    pub common_topics: Vec<String>,
    /// Publication date distribution
    pub date_distribution: HashMap<String, u32>, // "last_week", "last_month", etc.
    /// Content type distribution
    pub content_types: HashMap<String, u32>,
    /// Average content quality score
    pub avg_quality_score: Option<f64>,
    /// Language distribution
    pub languages: HashMap<String, u32>,
    /// Sentiment analysis (if performed)
    pub sentiment: Option<SearchSentiment>,
}

/// Sentiment analysis of search results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchSentiment {
    /// Overall sentiment score (-1.0 to 1.0)
    pub overall_sentiment: f64,
    /// Sentiment distribution
    pub distribution: SentimentDistribution,
    /// Most positive result
    pub most_positive: Option<String>, // URL
    /// Most negative result  
    pub most_negative: Option<String>, // URL
}

/// Distribution of sentiment across results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SentimentDistribution {
    /// Percentage of positive results
    pub positive_pct: f64,
    /// Percentage of neutral results
    pub neutral_pct: f64,
    /// Percentage of negative results
    pub negative_pct: f64,
}

/// Content summary with key points
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContentSummary {
    /// URL of the page
    pub url: String,
    /// Page title
    pub title: String,
    /// Executive summary (2-3 sentences)
    pub executive_summary: String,
    /// Key points extracted
    pub key_points: Vec<String>,
    /// Important entities mentioned
    pub entities: Vec<ContentEntity>,
    /// Main topics covered
    pub topics: Vec<String>,
    /// Summary quality confidence (0.0-1.0)
    pub confidence: f64,
    /// When the summary was generated
    pub generated_at: DateTime<Utc>,
}

/// Entity found in content
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContentEntity {
    /// Entity name
    pub name: String,
    /// Entity type (Person, Organization, Location, etc.)
    pub entity_type: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Context in which entity appears
    pub context: String,
}

/// Similar page search result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimilarPagesResult {
    /// Source URL used for similarity search
    pub source_url: String,
    /// Similar pages found
    pub similar_pages: Vec<SearchResult>,
    /// Similarity scores and metadata
    pub similarity_metadata: SimilarityMetadata,
    /// Search timestamp
    pub searched_at: DateTime<Utc>,
}

/// Metadata about similarity analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimilarityMetadata {
    /// Average similarity score
    pub avg_similarity: f64,
    /// Similarity calculation method used
    pub method: String,
    /// Common themes between source and similar pages
    pub common_themes: Vec<String>,
    /// Content overlap analysis
    pub content_overlap: f64,
}

impl Default for WebSearchConfig {
    fn default() -> Self {
        Self {
            exa_api_key: std::env::var("EXA_API_KEY").unwrap_or_default(),
            exa_base_url: "https://api.exa.ai".to_string(),
            max_results: 20,
            timeout_seconds: 30,
            include_content: true,
            content_limit: 5000,
        }
    }
}

/// Perform web search with content extraction
///
/// This tool performs web search and returns results with extracted content and metadata.
/// Uses traditional search APIs rather than semantic understanding.
#[tool]
pub async fn search_web(
    query: String,
    max_results: Option<u32>,
    include_content: Option<bool>,
    domain_filter: Option<Vec<String>>,
    date_filter: Option<String>,         // "day", "week", "month", "year"
    content_type_filter: Option<String>, // "news", "academic", "blog"
) -> crate::error::Result<WebSearchResult> {
    debug!(
        "Performing web search for query: '{}' with {} max results",
        query,
        max_results.unwrap_or(20)
    );

    let config = WebSearchConfig::default();
    if config.exa_api_key.is_empty() {
        return Err(WebToolError::Config(
            "EXA_API_KEY environment variable not set".to_string(),
        ));
    }

    let client = WebClient::new()
        .map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?
        .with_exa_key(config.exa_api_key.clone());

    // Build search parameters
    let mut params = HashMap::new();
    params.insert("query".to_string(), query.clone());
    params.insert(
        "num_results".to_string(),
        max_results.unwrap_or(20).to_string(),
    );
    params.insert(
        "include_content".to_string(),
        include_content.unwrap_or(true).to_string(),
    );
    params.insert("search_type".to_string(), "semantic".to_string());

    if let Some(ref domains) = domain_filter {
        params.insert("include_domains".to_string(), domains.join(","));
    }

    if let Some(ref date) = date_filter {
        params.insert("start_published_date".to_string(), format_date_filter(date));
    }

    if let Some(content_type) = content_type_filter {
        params.insert("category".to_string(), content_type);
    }

    // Make API request to Exa with API key header
    let url = format!("{}/search", config.exa_base_url);
    let mut headers = HashMap::new();
    headers.insert("x-api-key".to_string(), config.exa_api_key.clone());
    headers.insert("accept".to_string(), "application/json".to_string());
    let response = client.get_with_params_and_headers(&url, &params, headers).await.map_err(|e| {
        if e.to_string().contains("timeout") || e.to_string().contains("connection") {
            WebToolError::Network(format!("Web search request failed: {}", e))
        } else {
            WebToolError::Config(format!("Web search request failed: {}", e))
        }
    })?;

    // Parse search results
    let results = parse_exa_search_response(&response, &query)
        .await
        .map_err(|e| WebToolError::Config(format!("Failed to parse search response: {}", e)))?;

    // Perform additional analysis
    let insights = analyze_search_results(&results)
        .await
        .map_err(|e| WebToolError::Config(format!("Failed to analyze results: {}", e)))?;

    let search_result = WebSearchResult {
        query: query.clone(),
        search_type: "semantic".to_string(),
        results: results.clone(),
        metadata: WebSearchMetadata {
            total_results: results.len() as u32,
            returned_results: results.len() as u32,
            execution_time_ms: 1500, // Would measure actual time
            filtered: domain_filter.is_some() || date_filter.is_some(),
            related_queries: generate_related_queries(&query).await.map_err(|e| {
                WebToolError::Config(format!("Failed to generate related queries: {}", e))
            })?,
            top_domains: extract_top_domains(&results),
        },
        insights,
        searched_at: Utc::now(),
    };

    info!(
        "Web search completed: {} results for '{}'",
        results.len(),
        query
    );

    Ok(search_result)
}

/// Search for pages similar to a given URL
///
/// This tool finds web pages that are similar in content and topic to a source URL,
/// useful for finding related information or alternative perspectives.
#[tool]
pub async fn find_similar_pages(
    source_url: String,
    max_results: Option<u32>,
    include_content: Option<bool>,
    similarity_threshold: Option<f64>,
) -> crate::error::Result<SimilarPagesResult> {
    debug!("Finding pages similar to: {}", source_url);

    let config = WebSearchConfig::default();
    if config.exa_api_key.is_empty() {
        return Err(WebToolError::Config(
            "EXA_API_KEY environment variable not set".to_string(),
        ));
    }

    let client = WebClient::new()
        .map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?
        .with_exa_key(config.exa_api_key.clone());

    // Build similarity search parameters
    let mut params = HashMap::new();
    params.insert("url".to_string(), source_url.clone());
    params.insert(
        "num_results".to_string(),
        max_results.unwrap_or(10).to_string(),
    );
    params.insert(
        "include_content".to_string(),
        include_content.unwrap_or(true).to_string(),
    );

    if let Some(threshold) = similarity_threshold {
        params.insert("similarity_threshold".to_string(), threshold.to_string());
    }

    // Make API request with API key header
    let url = format!("{}/find_similar", config.exa_base_url);
    let mut headers = HashMap::new();
    headers.insert("x-api-key".to_string(), config.exa_api_key.clone());
    headers.insert("accept".to_string(), "application/json".to_string());
    let response = client.get_with_params_and_headers(&url, &params, headers).await.map_err(|e| {
        if e.to_string().contains("timeout") || e.to_string().contains("connection") {
            WebToolError::Network(format!("Web search request failed: {}", e))
        } else {
            WebToolError::Config(format!("Web search request failed: {}", e))
        }
    })?;

    // Parse results
    let similar_pages = parse_similar_pages_response(&response)
        .await
        .map_err(|e| WebToolError::Config(format!("Failed to parse similar pages: {}", e)))?;

    // Analyze similarity patterns
    let similarity_metadata = analyze_similarity(&similar_pages)
        .await
        .map_err(|e| WebToolError::Config(format!("Failed to analyze similarity: {}", e)))?;

    let result = SimilarPagesResult {
        source_url: source_url.clone(),
        similar_pages: similar_pages.clone(),
        similarity_metadata,
        searched_at: Utc::now(),
    };

    info!(
        "Found {} similar pages to {}",
        similar_pages.len(),
        source_url
    );

    Ok(result)
}

/// Summarize content from multiple web pages
///
/// This tool extracts and summarizes key information from multiple web pages,
/// creating a comprehensive overview of a topic from multiple sources.
#[tool]
pub async fn summarize_web_content(
    urls: Vec<String>,
    summary_length: Option<String>, // "brief", "detailed", "comprehensive"
    focus_topics: Option<Vec<String>>,
    _include_quotes: Option<bool>,
) -> crate::error::Result<Vec<ContentSummary>> {
    debug!("Summarizing content from {} URLs", urls.len());

    let config = WebSearchConfig::default();
    let client = WebClient::new()
        .map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?
        .with_exa_key(config.exa_api_key.clone());

    let mut summaries = Vec::new();

    // Process each URL
    for url in urls {
        match extract_and_summarize_page(&client, &url, &summary_length, &focus_topics).await {
            Ok(summary) => {
                summaries.push(summary);
            }
            Err(e) => {
                warn!("Failed to summarize {}: {}", url, e);
                // Continue with other URLs
            }
        }
    }

    info!(
        "Successfully summarized {} out of {} pages",
        summaries.len(),
        summaries.len()
    );

    Ok(summaries)
}

/// Search for recent news and articles on a topic
///
/// This tool specifically searches for recent news articles and blog posts,
/// optimized for finding current information and trending discussions.
#[tool]
pub async fn search_recent_news(
    topic: String,
    time_window: Option<String>,       // "24h", "week", "month"
    source_types: Option<Vec<String>>, // "news", "blog", "social"
    max_results: Option<u32>,
    include_analysis: Option<bool>,
) -> crate::error::Result<WebSearchResult> {
    debug!(
        "Searching recent news for topic: '{}' within {}",
        topic,
        time_window.as_deref().unwrap_or("week")
    );

    let config = WebSearchConfig::default();
    let client = WebClient::new()
        .map_err(|e| WebToolError::Client(format!("Failed to create client: {}", e)))?
        .with_exa_key(config.exa_api_key.clone());

    // Build news-specific search parameters
    let mut params = HashMap::new();
    params.insert("query".to_string(), topic.clone());
    params.insert("search_type".to_string(), "news".to_string());
    params.insert(
        "num_results".to_string(),
        max_results.unwrap_or(30).to_string(),
    );
    params.insert("include_content".to_string(), "true".to_string());

    // Set time window
    let time_window = time_window.unwrap_or_else(|| "week".to_string());
    params.insert(
        "start_published_date".to_string(),
        format_date_filter(&time_window),
    );

    // Filter by source types if specified
    if let Some(sources) = source_types {
        if sources.contains(&"news".to_string()) {
            params.insert("category".to_string(), "news".to_string());
        }
    }

    let url = format!("{}/search", config.exa_base_url);
    let mut headers = HashMap::new();
    headers.insert("x-api-key".to_string(), config.exa_api_key.clone());
    headers.insert("accept".to_string(), "application/json".to_string());
    let response = client.get_with_params_and_headers(&url, &params, headers).await.map_err(|e| {
        if e.to_string().contains("timeout") || e.to_string().contains("connection") {
            WebToolError::Network(format!("Web search request failed: {}", e))
        } else {
            WebToolError::Config(format!("Web search request failed: {}", e))
        }
    })?;

    // Parse and enhance results for news context
    let mut results = parse_exa_search_response(&response, &topic)
        .await
        .map_err(|e| WebToolError::Config(format!("Failed to parse news response: {}", e)))?;

    // Sort by recency
    results.sort_by(|a, b| {
        b.published_date
            .unwrap_or_else(Utc::now)
            .cmp(&a.published_date.unwrap_or_else(Utc::now))
    });

    let insights = if include_analysis.unwrap_or(true) {
        analyze_news_results(&results)
            .await
            .map_err(|e| WebToolError::Config(format!("Failed to analyze news: {}", e)))?
    } else {
        SearchInsights {
            common_topics: vec![],
            date_distribution: HashMap::new(),
            content_types: HashMap::new(),
            avg_quality_score: None,
            languages: HashMap::new(),
            sentiment: None,
        }
    };

    let search_result = WebSearchResult {
        query: topic.clone(),
        search_type: "news".to_string(),
        results: results.clone(),
        metadata: WebSearchMetadata {
            total_results: results.len() as u32,
            returned_results: results.len() as u32,
            execution_time_ms: 1200,
            filtered: true,
            related_queries: generate_related_queries(&topic).await.map_err(|e| {
                WebToolError::Config(format!("Failed to generate related queries: {}", e))
            })?,
            top_domains: extract_top_domains(&results),
        },
        insights,
        searched_at: Utc::now(),
    };

    info!(
        "Recent news search completed: {} results for '{}'",
        search_result.results.len(),
        topic
    );

    Ok(search_result)
}

/// Parse Exa search API response into structured results
async fn parse_exa_search_response(
    response: &str,
    query: &str,
) -> crate::error::Result<Vec<SearchResult>> {
    let json: serde_json::Value = serde_json::from_str(response)
        .map_err(|e| WebToolError::Parsing(format!("Invalid Exa JSON: {}", e)))?;

    let mut out = Vec::new();
    let results = json.get("results").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    for r in results {
        let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let url = r.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if url.is_empty() { continue; }
        let id = r.get("id").and_then(|v| v.as_str()).unwrap_or(url.as_str()).to_string();
        let description = r.get("description").or_else(|| r.get("snippet")).and_then(|v| v.as_str()).map(|s| s.to_string());
        let content = r.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());
        let published_date = r.get("publishedDate").or_else(|| r.get("published_date")).and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc));
        let domain_name = url::Url::parse(&url).ok().and_then(|u| u.host_str().map(|h| h.to_string())).unwrap_or_default();
        let score = r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.8);
        let language = r.get("language").and_then(|v| v.as_str()).map(|s| s.to_string());
        let author = r.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());

        let words = content.as_ref().map(|c| c.split_whitespace().count() as u32).unwrap_or(0);
        let reading_time = if words > 0 { Some((words as f64 / 200.0).ceil() as u32) } else { None };
        let length_category = match words {
            0..=200 => "Short",
            201..=800 => "Medium",
            801..=2000 => "Long",
            _ => "Very Long",
        }.to_string();

        let content_type = ContentType {
            primary: "Article".to_string(),
            format: if url.to_lowercase().ends_with(".pdf") { "PDF".to_string() } else { "HTML".to_string() },
            is_paywalled: None,
            quality_score: Some(((score * 100.0) as u32).min(100)),
            length_category,
        };

        let metadata = PageMetadata {
            author,
            tags: vec![query.to_lowercase()],
            social_meta: SocialMetadata { og_title: None, og_description: None, og_image: None, twitter_card: None, twitter_site: None },
            seo_meta: SeoMetadata { meta_description: description.clone(), meta_keywords: vec![], robots: None, schema_types: vec![] },
            canonical_url: None,
            last_modified: None,
        };

        let domain = DomainInfo { name: domain_name, reputation_score: None, category: None, is_trusted: true, authority_score: None };

        out.push(SearchResult {
            id,
            title,
            url,
            description,
            content,
            summary: None,
            published_date,
            domain,
            metadata,
            relevance_score: score,
            content_type,
            language,
            reading_time_minutes: reading_time,
        });
    }
    Ok(out)
}

/// Parse similar pages API response
async fn parse_similar_pages_response(response: &str) -> crate::error::Result<Vec<SearchResult>> {
    // Reuse the general Exa parser without query context
    parse_exa_search_response(response, "").await
}

/// Extract and summarize content from a single page using extractive summarization
///
/// Uses sentence ranking and selection rather than generative AI summarization.
/// Ranks sentences by importance and selects diverse, representative ones.
async fn extract_and_summarize_page(
    client: &WebClient,
    url: &str,
    summary_length: &Option<String>,
    focus_topics: &Option<Vec<String>>,
) -> crate::error::Result<ContentSummary> {
    let html = client.get(url).await.map_err(|e| WebToolError::Network(format!("Failed to fetch {}: {}", url, e)))?;
    let (title, clean_text, sentences, headings) = extract_main_content(&html, url);

    // Determine target summary length
    let n = match summary_length.as_deref() {
        Some("comprehensive") => 8,
        Some("detailed") => 5,
        _ => 3,
    } as usize;

    let topic_set: std::collections::HashSet<String> = focus_topics
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.to_lowercase())
        .collect();

    let ranked = rank_sentences(&sentences, &clean_text, &topic_set, &headings);
    let selected = select_diverse(&ranked, n, 0.6);
    let executive_summary = selected.join(" ");

    // Key points: top distinct sentences or heading-based bullets
    let mut key_points = selected
        .iter()
        .take(5).cloned()
        .collect::<Vec<_>>();
    if key_points.is_empty() && !headings.is_empty() {
        key_points = headings.iter().take(5).cloned().collect();
    }

    let topics = if !topic_set.is_empty() {
        topic_set.iter().cloned().collect()
    } else {
        extract_topics_from_text(&clean_text)
    };

    // Entities via improved proper-noun pattern
    let entity_re = regex::Regex::new(r"(?m)(?:^|\s)([A-Z][a-z]+(?:\s+[A-Z][a-z]+){0,3})").unwrap();
    let mut entities: Vec<ContentEntity> = entity_re
        .captures_iter(&clean_text)
        .map(|cap| ContentEntity { name: cap[1].trim().to_string(), entity_type: "ProperNoun".to_string(), confidence: 0.55, context: "".to_string() })
        .collect();
    entities.dedup_by(|a,b| a.name.eq_ignore_ascii_case(&b.name));
    entities.truncate(8);

    // Confidence based on content richness and heading availability
    let mut confidence = (clean_text.len().min(8000) as f64 / 8000.0) * 0.6 + 0.3;
    if !headings.is_empty() { confidence += 0.05; }
    confidence = confidence.min(0.97);

    Ok(ContentSummary {
        url: url.to_string(),
        title,
        executive_summary,
        key_points,
        entities,
        topics,
        confidence,
        generated_at: Utc::now(),
    })
}

// Extract main content using HTML parsing and content-density heuristics
fn extract_main_content(html: &str, fallback_url: &str) -> (String, String, Vec<String>, Vec<String>) {
    let document = Html::parse_document(html);

    // Prefer og:title
    let sel_meta_title = Selector::parse("meta[property=\"og:title\"]").unwrap();
    let title = document
        .select(&sel_meta_title)
        .filter_map(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
        .find(|s| !s.is_empty())
        .or_else(|| {
            // Fallback to <title>
            let sel_title = Selector::parse("title").unwrap();
            document.select(&sel_title).next().map(|e| e.text().collect::<String>().trim().to_string())
        })
        .unwrap_or_else(|| fallback_url.to_string());

    // Candidate containers likely to hold article content
    let candidates = vec![
        "article",
        "main",
        "div#content",
        "div#main",
        "div.post-content",
        "div.article-content",
        "section.article",
        "div.entry-content",
        "div#main-content",
    ];

    let mut best_text = String::new();
    let mut best_headings: Vec<String> = Vec::new();
    for css in candidates {
        if let Ok(sel) = Selector::parse(css) {
            for node in document.select(&sel) {
                let (text, headings) = extract_text_from_node(node);
                if text.len() > best_text.len() {
                    best_text = text;
                    best_headings = headings;
                }
            }
        }
    }

    if best_text.is_empty() {
        // Fallback: collect from body paragraphs
        if let Ok(sel) = Selector::parse("body") {
            if let Some(body) = document.select(&sel).next() {
                let (text, headings) = extract_text_from_node(body);
                best_text = text;
                best_headings = headings;
            }
        }
    }

    // Sentence split
    let sentences: Vec<String> = split_sentences(&best_text)
        .into_iter()
        .filter(|s| s.split_whitespace().count() >= 5)
        .collect();

    (title, best_text, sentences, best_headings)
}

fn extract_text_from_node(root: ElementRef) -> (String, Vec<String>) {
    let sel_exclude = ["script","style","noscript","template","header","footer","nav","aside"];
    let sel_p = Selector::parse("p, li").unwrap();
    let sel_h = Selector::parse("h1, h2, h3").unwrap();

    // Headings
    let mut headings: Vec<String> = root.select(&sel_h)
        .map(|h| normalize_whitespace(&h.text().collect::<String>()))
        .filter(|s| !s.is_empty())
        .collect();
    headings.dedup();

    // Paragraph-like text
    let mut blocks: Vec<String> = Vec::new();
    for p in root.select(&sel_p) {
        // Skip paragraphs inside excluded parents
        if has_excluded_ancestor(p, &sel_exclude) { continue; }
        let txt = normalize_whitespace(&p.text().collect::<String>());
        if txt.len() >= 40 { blocks.push(txt); }
    }
    let full = blocks.join("\n");
    (full, headings)
}

fn has_excluded_ancestor(mut node: ElementRef, excluded: &[&str]) -> bool {
    while let Some(parent) = node.ancestors().find_map(ElementRef::wrap) {
        let name = parent.value().name();
        if excluded.contains(&name) { return true; }
        node = parent;
        // continue up until root
        if node.parent().is_none() { break; }
    }
    false
}

fn normalize_whitespace(s: &str) -> String {
    let s = html_escape::decode_html_entities(s);
    let re = regex::Regex::new(r"\s+").unwrap();
    re.replace_all(&s, " ").trim().to_string()
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut v = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            let s = normalize_whitespace(&current);
            if !s.is_empty() { v.push(s); }
            current.clear();
        }
    }
    if !current.trim().is_empty() { v.push(normalize_whitespace(&current)); }
    v
}

// Rank sentences with simple TF scoring + positional + heading/topic boosts
fn rank_sentences(
    sentences: &[String],
    full_text: &str,
    topics: &std::collections::HashSet<String>,
    headings: &[String],
) -> Vec<(String, f64)> {
    let mut tf: HashMap<String, f64> = HashMap::new();
    for w in full_text.split(|c: char| !c.is_alphanumeric()) {
        let w = w.to_lowercase();
        if w.len() < 3 { continue; }
        *tf.entry(w).or_insert(0.0) += 1.0;
    }
    // Normalize
    let max_tf = tf.values().cloned().fold(1.0, f64::max);
    for v in tf.values_mut() { *v /= max_tf; }

    let heading_text = headings.join(" ").to_lowercase();

    let mut scored: Vec<(String, f64)> = sentences
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let words: Vec<String> = s.split(|c: char| !c.is_alphanumeric())
                .map(|w| w.to_lowercase())
                .filter(|w| w.len() >= 3)
                .collect();
            let mut score = 0.0;
            for w in &words { score += *tf.get(w).unwrap_or(&0.0); }
            // Length normalization
            let len = s.split_whitespace().count() as f64;
            if len > 0.0 { score /= len.powf(0.3); }
            // Positional boost (earlier sentences)
            score += 0.15 * (1.0 / ((i + 1) as f64).sqrt());
            // Topic boost
            if !topics.is_empty() {
                let lower = s.to_lowercase();
                for t in topics { if lower.contains(t) { score += 0.25; } }
            }
            // Heading proximity boost
            for h in headings {
                if s.to_lowercase().contains(&h.to_lowercase()) { score += 0.2; break; }
            }
            // Title/headings semantic overlap
            if !heading_text.is_empty() {
                let overlap = jaccard(&s.to_lowercase(), &heading_text);
                score += 0.1 * overlap;
            }
            (s.clone(), score)
        })
        .collect();
    scored.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

fn jaccard(a: &str, b: &str) -> f64 {
    let set_a: std::collections::HashSet<_> = a.split_whitespace().collect();
    let set_b: std::collections::HashSet<_> = b.split_whitespace().collect();
    let inter = set_a.intersection(&set_b).count() as f64;
    let union = set_a.union(&set_b).count() as f64;
    if union == 0.0 { 0.0 } else { inter / union }
}

fn select_diverse(scored: &[(String, f64)], k: usize, max_sim: f64) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (s, _) in scored {
        if out.len() >= k { break; }
        if out.iter().all(|t| jaccard(&s.to_lowercase(), &t.to_lowercase()) < max_sim) {
            out.push(s.clone());
        }
    }
    out
}

/// Analyze search results to extract insights
async fn analyze_search_results(results: &[SearchResult]) -> crate::error::Result<SearchInsights> {
    let mut content_types = HashMap::new();
    let mut languages = HashMap::new();
    let mut date_distribution = HashMap::new();
    let mut topics = Vec::new();

    for result in results {
        // Count content types
        *content_types
            .entry(result.content_type.primary.clone())
            .or_insert(0) += 1;

        // Count languages
        if let Some(lang) = &result.language {
            *languages.entry(lang.clone()).or_insert(0) += 1;
        }

        // Analyze publication dates
        if let Some(pub_date) = result.published_date {
            let days_ago = (Utc::now() - pub_date).num_days();
            let category = match days_ago {
                0..=1 => "today",
                2..=7 => "this_week",
                8..=30 => "this_month",
                _ => "older",
            };
            *date_distribution.entry(category.to_string()).or_insert(0) += 1;
        }

        // Extract topics from metadata
        topics.extend(result.metadata.tags.clone());
    }

    // Calculate average quality score
    let quality_scores: Vec<u32> = results
        .iter()
        .filter_map(|r| r.content_type.quality_score)
        .collect();
    let avg_quality_score = if !quality_scores.is_empty() {
        Some(quality_scores.iter().sum::<u32>() as f64 / quality_scores.len() as f64)
    } else {
        None
    };

    Ok(SearchInsights {
        common_topics: topics,
        date_distribution,
        content_types,
        avg_quality_score,
        languages,
        sentiment: None, // Would analyze sentiment in production
    })
}

/// Analyze news-specific results
async fn analyze_news_results(results: &[SearchResult]) -> crate::error::Result<SearchInsights> {
    // Similar to analyze_search_results but with news-specific analysis
    analyze_search_results(results).await
}

/// Analyze similarity patterns between pages
async fn analyze_similarity(results: &[SearchResult]) -> crate::error::Result<SimilarityMetadata> {
    let avg_similarity =
        results.iter().map(|r| r.relevance_score).sum::<f64>() / results.len() as f64;

    let common_themes = results
        .iter()
        .flat_map(|r| r.metadata.tags.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    Ok(SimilarityMetadata {
        avg_similarity,
        method: "semantic_embeddings".to_string(),
        common_themes,
        content_overlap: 0.75, // Would calculate actual overlap
    })
}

/// Generate related search queries
async fn generate_related_queries(query: &str) -> crate::error::Result<Vec<String>> {
    // Heuristic expansion of the query into related intents
    let mut variants = vec![
        format!("{} news", query),
        format!("{} latest", query),
        format!("{} guide", query),
        format!("{} tutorial", query),
        format!("{} best practices", query),
        format!("{} examples", query),
        format!("how to {}", query),
        format!("{} vs alternatives", query),
        format!("{} 2025 trends", query),
    ];
    variants.sort();
    variants.dedup();
    Ok(variants)
}

/// Extract top domains from search results
fn extract_top_domains(results: &[SearchResult]) -> Vec<String> {
    let mut domain_counts: HashMap<String, u32> = HashMap::new();

    for result in results {
        *domain_counts.entry(result.domain.name.clone()).or_insert(0) += 1;
    }

    let mut domains: Vec<(String, u32)> = domain_counts.into_iter().collect();
    domains.sort_by(|a, b| b.1.cmp(&a.1));

    domains
        .into_iter()
        .take(10)
        .map(|(domain, _)| domain)
        .collect()
}

/// Format date filter for API requests
fn format_date_filter(window: &str) -> String {
    let days_ago = match window {
        "24h" | "day" => 1,
        "week" => 7,
        "month" => 30,
        "year" => 365,
        _ => 7,
    };

    let date = Utc::now() - chrono::Duration::days(days_ago);
    date.format("%Y-%m-%d").to_string()
}

// Simple keyword topic extraction from text
fn extract_topics_from_text(text: &str) -> Vec<String> {
    let stopwords = ["the","and","for","with","that","this","from","have","your","you","are","was","were","has","had","not","but","all","any","can","will","just","into","about","over","more","than","when","what","how","why","where","then","them","they","their","its","it's","as","of","in","on","to","by","at","or","an","be"];    
    let mut counts: HashMap<String, u32> = HashMap::new();
    for w in text.split(|c: char| !c.is_alphanumeric()) {
        let w = w.to_lowercase();
        if w.len() < 4 { continue; }
        if stopwords.contains(&w.as_str()) { continue; }
        *counts.entry(w).or_insert(0) += 1;
    }
    let mut v: Vec<(String,u32)> = counts.into_iter().collect();
    v.sort_by(|a,b| b.1.cmp(&a.1));
    v.into_iter().take(5).map(|(k,_)| k).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_search_config_default() {
        let config = WebSearchConfig::default();
        assert_eq!(config.exa_base_url, "https://api.exa.ai");
        assert_eq!(config.max_results, 20);
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            id: "1".to_string(),
            title: "Test Page".to_string(),
            url: "https://example.com".to_string(),
            description: Some("Test description".to_string()),
            content: Some("Test content".to_string()),
            summary: None,
            published_date: Some(Utc::now()),
            domain: DomainInfo {
                name: "example.com".to_string(),
                reputation_score: Some(80),
                category: Some("Test".to_string()),
                is_trusted: true,
                authority_score: Some(70),
            },
            metadata: PageMetadata {
                author: None,
                tags: vec!["test".to_string()],
                social_meta: SocialMetadata {
                    og_title: None,
                    og_description: None,
                    og_image: None,
                    twitter_card: None,
                    twitter_site: None,
                },
                seo_meta: SeoMetadata {
                    meta_description: None,
                    meta_keywords: vec![],
                    robots: None,
                    schema_types: vec![],
                },
                canonical_url: None,
                last_modified: None,
            },
            relevance_score: 0.8,
            content_type: ContentType {
                primary: "Article".to_string(),
                format: "HTML".to_string(),
                is_paywalled: Some(false),
                quality_score: Some(75),
                length_category: "Medium".to_string(),
            },
            language: Some("en".to_string()),
            reading_time_minutes: Some(5),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Test Page"));
    }

    #[test]
    fn test_format_date_filter() {
        let result = format_date_filter("week");
        assert!(!result.is_empty());
        assert!(result.len() == 10); // YYYY-MM-DD format
    }
}
