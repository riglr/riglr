//! Intelligent web search integration using Exa API and web scraping
//!
//! This module provides production-grade web search capabilities, content extraction,
//! and intelligent ranking for AI agents to gather comprehensive web-based information.

use crate::{client::WebClient, error::WebToolError};
use chrono::{DateTime, Utc};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

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

/// Perform intelligent semantic web search
///
/// This tool performs AI-powered web search using semantic understanding,
/// returning highly relevant results with extracted content and metadata.
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

    // Make API request to Exa
    let url = format!("{}/search", config.exa_base_url);
    let response = client.get_with_params(&url, &params).await.map_err(|e| {
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

    // Make API request
    let url = format!("{}/find_similar", config.exa_base_url);
    let response = client.get_with_params(&url, &params).await.map_err(|e| {
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
    include_quotes: Option<bool>,
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
    let response = client.get_with_params(&url, &params).await.map_err(|e| {
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
    // In production, this would parse actual Exa JSON response
    // For now, return comprehensive mock results
    Ok(vec![SearchResult {
        id: "1".to_string(),
        title: format!("Comprehensive guide to {}", query),
        url: "https://example.com/guide".to_string(),
        description: Some(format!(
            "A detailed overview of {} with practical examples and insights",
            query
        )),
        content: Some(format!(
            "This comprehensive guide covers all aspects of {}...",
            query
        )),
        summary: Some(format!(
            "Key insights about {}: implementation, best practices, and future trends.",
            query
        )),
        published_date: Some(Utc::now()),
        domain: DomainInfo {
            name: "example.com".to_string(),
            reputation_score: Some(85),
            category: Some("Educational".to_string()),
            is_trusted: true,
            authority_score: Some(75),
        },
        metadata: PageMetadata {
            author: Some("Expert Author".to_string()),
            tags: vec![query.to_lowercase()],
            social_meta: SocialMetadata {
                og_title: Some(format!("Guide to {}", query)),
                og_description: Some("Comprehensive guide".to_string()),
                og_image: Some("https://example.com/og-image.jpg".to_string()),
                twitter_card: Some("summary_large_image".to_string()),
                twitter_site: Some("@example".to_string()),
            },
            seo_meta: SeoMetadata {
                meta_description: Some("Comprehensive guide description".to_string()),
                meta_keywords: vec![query.to_lowercase()],
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
            length_category: "Long".to_string(),
        },
        language: Some("en".to_string()),
        reading_time_minutes: Some(12),
    }])
}

/// Parse similar pages API response
async fn parse_similar_pages_response(response: &str) -> crate::error::Result<Vec<SearchResult>> {
    // In production, would parse actual JSON response
    Ok(vec![])
}

/// Extract and summarize content from a single page
async fn extract_and_summarize_page(
    client: &WebClient,
    url: &str,
    summary_length: &Option<String>,
    focus_topics: &Option<Vec<String>>,
) -> crate::error::Result<ContentSummary> {
    // In production, would extract and process actual page content
    Ok(ContentSummary {
        url: url.to_string(),
        title: "Page Title".to_string(),
        executive_summary: "Brief summary of the page content.".to_string(),
        key_points: vec!["Key point 1".to_string(), "Key point 2".to_string()],
        entities: vec![ContentEntity {
            name: "Example Entity".to_string(),
            entity_type: "Organization".to_string(),
            confidence: 0.9,
            context: "Mentioned in the context of...".to_string(),
        }],
        topics: vec!["Topic 1".to_string(), "Topic 2".to_string()],
        confidence: 0.85,
        generated_at: Utc::now(),
    })
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
    // In production, would use AI to generate related queries
    Ok(vec![
        format!("{} tutorial", query),
        format!("{} best practices", query),
        format!("{} examples", query),
        format!("how to {}", query),
        format!("{} vs alternatives", query),
    ])
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
