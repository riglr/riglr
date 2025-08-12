//! Web client for interacting with various web APIs

use crate::error::{Result, WebToolError};
use reqwest::{Client, ClientBuilder};
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Configuration for HTTP client
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Maximum retries
    pub max_retries: u32,
    /// Retry delay
    pub retry_delay: Duration,
    /// User agent
    pub user_agent: String,
    /// Enable exponential backoff
    pub exponential_backoff: bool,
    /// Jitter for retry delays (0.0 to 1.0)
    pub jitter_factor: f32,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            user_agent: "riglr-web-tools/0.1.0".to_string(),
            exponential_backoff: true,
            jitter_factor: 0.1,
        }
    }
}

/// Type-safe API keys configuration
#[derive(Debug, Clone, Default)]
pub struct ApiKeys {
    /// Twitter/X Bearer Token
    pub twitter: Option<String>,
    /// Exa API key
    pub exa: Option<String>,
    /// DexScreener API key (optional)
    pub dexscreener: Option<String>,
    /// NewsAPI key
    pub newsapi: Option<String>,
    /// CryptoPanic API key
    pub cryptopanic: Option<String>,
    /// LunarCrush API key
    pub lunarcrush: Option<String>,
    /// Alternative data API key
    pub alternative: Option<String>,
    /// Generic fallback for other services
    pub other: HashMap<String, String>,
}

impl ApiKeys {
    /// Check if all API keys are empty
    pub fn is_empty(&self) -> bool {
        self.twitter.is_none()
            && self.exa.is_none()
            && self.dexscreener.is_none()
            && self.newsapi.is_none()
            && self.cryptopanic.is_none()
            && self.lunarcrush.is_none()
            && self.alternative.is_none()
            && self.other.is_empty()
    }

    /// Get an API key by name
    pub fn get(&self, key: &str) -> Option<&String> {
        match key {
            "twitter" => self.twitter.as_ref(),
            "exa" => self.exa.as_ref(),
            "dexscreener" => self.dexscreener.as_ref(),
            "newsapi" => self.newsapi.as_ref(),
            "cryptopanic" => self.cryptopanic.as_ref(),
            "lunarcrush" => self.lunarcrush.as_ref(),
            "alternative" => self.alternative.as_ref(),
            other => self.other.get(other),
        }
    }

    /// Get the number of configured API keys
    pub fn len(&self) -> usize {
        let mut count = 0;
        if self.twitter.is_some() {
            count += 1;
        }
        if self.exa.is_some() {
            count += 1;
        }
        if self.dexscreener.is_some() {
            count += 1;
        }
        if self.newsapi.is_some() {
            count += 1;
        }
        if self.cryptopanic.is_some() {
            count += 1;
        }
        if self.lunarcrush.is_some() {
            count += 1;
        }
        if self.alternative.is_some() {
            count += 1;
        }
        count + self.other.len()
    }

    /// Check if an API key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Insert a new API key
    pub fn insert(&mut self, key: String, value: String) {
        match key.as_str() {
            "twitter" => self.twitter = Some(value),
            "exa" => self.exa = Some(value),
            "dexscreener" => self.dexscreener = Some(value),
            "newsapi" => self.newsapi = Some(value),
            "cryptopanic" => self.cryptopanic = Some(value),
            "lunarcrush" => self.lunarcrush = Some(value),
            "alternative" => self.alternative = Some(value),
            other => {
                self.other.insert(other.to_string(), value);
            }
        }
    }
}

/// Type-safe client configuration
#[derive(Debug, Clone, Default)]
pub struct ClientConfig {
    /// Base URL overrides for testing
    pub base_urls: BaseUrls,
    /// Rate limiting settings
    pub rate_limits: RateLimits,
}

impl ClientConfig {
    /// Check if the config is empty
    pub fn is_empty(&self) -> bool {
        false // Config always has default values
    }

    /// Get a configuration value by key
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "dexscreener_url" => Some(self.base_urls.dexscreener.clone()),
            "exa_url" => Some(self.base_urls.exa.clone()),
            "newsapi_url" => Some(self.base_urls.newsapi.clone()),
            "cryptopanic_url" => Some(self.base_urls.cryptopanic.clone()),
            "lunarcrush_url" => Some(self.base_urls.lunarcrush.clone()),
            "twitter_url" => Some(self.base_urls.twitter.clone()),
            _ => None,
        }
    }

    /// Get the number of configuration entries
    pub fn len(&self) -> usize {
        6 // Fixed number of base URLs
    }

    /// Insert a configuration value
    pub fn insert(&mut self, key: String, value: String) {
        match key.as_str() {
            "dexscreener_url" => self.base_urls.dexscreener = value,
            "exa_url" => self.base_urls.exa = value,
            "newsapi_url" => self.base_urls.newsapi = value,
            "cryptopanic_url" => self.base_urls.cryptopanic = value,
            "lunarcrush_url" => self.base_urls.lunarcrush = value,
            "twitter_url" => self.base_urls.twitter = value,
            _ => {}
        }
    }
}

/// Base URL configuration for various services
#[derive(Debug, Clone)]
pub struct BaseUrls {
    /// DexScreener API base URL
    pub dexscreener: String,
    /// Exa API base URL
    pub exa: String,
    /// News API base URL
    pub newsapi: String,
    /// CryptoPanic API base URL
    pub cryptopanic: String,
    /// LunarCrush API base URL
    pub lunarcrush: String,
    /// Twitter API base URL
    pub twitter: String,
}

impl Default for BaseUrls {
    fn default() -> Self {
        Self {
            dexscreener: "https://api.dexscreener.com/latest".to_string(),
            exa: "https://api.exa.ai".to_string(),
            newsapi: "https://newsapi.org/v2".to_string(),
            cryptopanic: "https://cryptopanic.com/api/v1".to_string(),
            lunarcrush: "https://lunarcrush.com/api/3".to_string(),
            twitter: "https://api.twitter.com/2".to_string(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimits {
    /// DexScreener requests per minute limit
    pub dexscreener_per_minute: u32,
    /// Twitter requests per minute limit
    pub twitter_per_minute: u32,
    /// News API requests per minute limit
    pub newsapi_per_minute: u32,
    /// Exa API requests per minute limit
    pub exa_per_minute: u32,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            dexscreener_per_minute: 300,
            twitter_per_minute: 300,
            newsapi_per_minute: 500,
            exa_per_minute: 100,
        }
    }
}

/// A client for interacting with various web APIs and services
#[derive(Debug, Clone)]
pub struct WebClient {
    /// HTTP client for making requests
    pub http_client: Client,
    /// Type-safe API keys
    pub api_keys: ApiKeys,
    /// Type-safe configuration
    pub config: ClientConfig,
    /// HTTP configuration
    pub http_config: HttpConfig,
}

impl Default for WebClient {
    fn default() -> Self {
        let http_config = HttpConfig::default();
        let http_client = ClientBuilder::new()
            .timeout(http_config.timeout)
            .user_agent(&http_config.user_agent)
            .gzip(true)
            .build()
            .expect("Failed to create default HTTP client");

        Self {
            http_client,
            api_keys: ApiKeys::default(),
            config: ClientConfig::default(),
            http_config,
        }
    }
}

impl WebClient {
    /// Create a new web client
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// Create with custom HTTP configuration
    pub fn with_config(http_config: HttpConfig) -> Result<Self> {
        let http_client = ClientBuilder::new()
            .timeout(http_config.timeout)
            .user_agent(&http_config.user_agent)
            .gzip(true)
            .build()
            .map_err(|e| WebToolError::Client(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http_client,
            api_keys: ApiKeys::default(),
            config: ClientConfig::default(),
            http_config,
        })
    }

    /// Set API key for a service (for backwards compatibility)
    pub fn with_api_key<S1: Into<String>, S2: Into<String>>(
        mut self,
        service: S1,
        api_key: S2,
    ) -> Self {
        let service = service.into();
        let api_key = api_key.into();

        match service.as_str() {
            "twitter" => self.api_keys.twitter = Some(api_key),
            "exa" => self.api_keys.exa = Some(api_key),
            "dexscreener" => self.api_keys.dexscreener = Some(api_key),
            "newsapi" => self.api_keys.newsapi = Some(api_key),
            "cryptopanic" => self.api_keys.cryptopanic = Some(api_key),
            "lunarcrush" => self.api_keys.lunarcrush = Some(api_key),
            "alternative" => self.api_keys.alternative = Some(api_key),
            _ => {
                self.api_keys.other.insert(service, api_key);
            }
        }
        self
    }

    /// Set Twitter/X Bearer Token
    pub fn with_twitter_token<S: Into<String>>(mut self, token: S) -> Self {
        self.api_keys.twitter = Some(token.into());
        self
    }

    /// Set Exa API key
    pub fn with_exa_key<S: Into<String>>(mut self, key: S) -> Self {
        self.api_keys.exa = Some(key.into());
        self
    }

    /// Set DexScreener API key (if required)
    pub fn with_dexscreener_key<S: Into<String>>(mut self, key: S) -> Self {
        self.api_keys.dexscreener = Some(key.into());
        self
    }

    /// Set News API key
    pub fn with_news_api_key<S: Into<String>>(mut self, key: S) -> Self {
        self.api_keys.newsapi = Some(key.into());
        self
    }

    /// Set configuration option (for backwards compatibility)
    pub fn set_config<S: Into<String>>(&mut self, key: S, value: S) {
        let key = key.into();
        let value = value.into();

        // Map old config keys to new structure
        match key.as_str() {
            "base_url" | "dexscreener_base_url" => self.config.base_urls.dexscreener = value,
            "exa_base_url" => self.config.base_urls.exa = value,
            "newsapi_base_url" => self.config.base_urls.newsapi = value,
            "cryptopanic_base_url" => self.config.base_urls.cryptopanic = value,
            "lunarcrush_base_url" => self.config.base_urls.lunarcrush = value,
            "twitter_base_url" => self.config.base_urls.twitter = value,
            _ => {
                // Log unrecognized config keys
                warn!("Unrecognized config key: {}", key);
            }
        }
    }

    /// Get API key for a service
    pub fn get_api_key(&self, service: &str) -> Option<&String> {
        match service {
            "twitter" => self.api_keys.twitter.as_ref(),
            "exa" => self.api_keys.exa.as_ref(),
            "dexscreener" => self.api_keys.dexscreener.as_ref(),
            "newsapi" => self.api_keys.newsapi.as_ref(),
            "cryptopanic" => self.api_keys.cryptopanic.as_ref(),
            "lunarcrush" => self.api_keys.lunarcrush.as_ref(),
            "alternative" => self.api_keys.alternative.as_ref(),
            _ => self.api_keys.other.get(service),
        }
    }

    /// Get config value (for backwards compatibility)
    pub fn get_config(&self, key: &str) -> Option<String> {
        match key {
            "dexscreener_base_url" | "base_url" => Some(self.config.base_urls.dexscreener.clone()),
            "exa_base_url" => Some(self.config.base_urls.exa.clone()),
            "newsapi_base_url" => Some(self.config.base_urls.newsapi.clone()),
            "cryptopanic_base_url" => Some(self.config.base_urls.cryptopanic.clone()),
            "lunarcrush_base_url" => Some(self.config.base_urls.lunarcrush.clone()),
            "twitter_base_url" => Some(self.config.base_urls.twitter.clone()),
            _ => None,
        }
    }

    /// Calculate retry delay with exponential backoff and jitter
    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.http_config.retry_delay;

        let delay = if self.http_config.exponential_backoff {
            // Exponential backoff: delay * 2^(attempt - 1)
            base_delay * (2_u32.pow(attempt.saturating_sub(1)))
        } else {
            // Linear backoff: delay * attempt
            base_delay * attempt
        };

        // Add jitter if configured
        if self.http_config.jitter_factor > 0.0 {
            use rand::Rng;
            let mut rng = rand::rng();
            let jitter_range = delay.as_millis() as f32 * self.http_config.jitter_factor;
            let jitter = rng.random_range(-jitter_range..=jitter_range) as u64;
            let final_delay = (delay.as_millis() as i64 + jitter as i64).max(0) as u64;
            Duration::from_millis(final_delay)
        } else {
            delay
        }
    }

    /// Execute HTTP request with retry logic (internal helper)
    async fn execute_with_retry<F, Fut>(&self, url: &str, request_fn: F) -> Result<String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = reqwest::Result<reqwest::Response>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.http_config.max_retries {
            attempts += 1;

            match request_fn().await {
                Ok(response) => {
                    let status = response.status();

                    if status.is_success() {
                        let text = response.text().await.map_err(|e| {
                            WebToolError::Network(format!("Failed to read response: {}", e))
                        })?;

                        info!("Successfully fetched {} bytes from {}", text.len(), url);
                        return Ok(text);
                    } else if status.is_server_error() && attempts < self.http_config.max_retries {
                        // Retry on server errors
                        warn!(
                            "Server error {} from {}, attempt {}/{}",
                            status, url, attempts, self.http_config.max_retries
                        );
                        last_error = Some(format!("HTTP {}", status));

                        let delay = self.calculate_retry_delay(attempts);
                        debug!("Retrying after {:?}", delay);
                        tokio::time::sleep(delay).await;
                    } else {
                        // Don't retry on client errors
                        let error_text = response.text().await.unwrap_or_default();
                        return Err(WebToolError::Api(format!(
                            "HTTP {} from {}: {}",
                            status, url, error_text
                        )));
                    }
                }
                Err(e) => {
                    if attempts < self.http_config.max_retries {
                        warn!(
                            "Request failed for {}, attempt {}/{}: {}",
                            url, attempts, self.http_config.max_retries, e
                        );
                        last_error = Some(e.to_string());

                        let delay = self.calculate_retry_delay(attempts);
                        debug!("Retrying after {:?}", delay);
                        tokio::time::sleep(delay).await;
                    } else {
                        return Err(WebToolError::Api(format!(
                            "Request failed after {} attempts: {}",
                            attempts, e
                        )));
                    }
                }
            }
        }

        Err(WebToolError::Api(format!(
            "Request failed after {} attempts: {}",
            attempts,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )))
    }

    /// Execute HTTP POST request with retry logic returning JSON (internal helper)
    async fn execute_post_with_retry<F, Fut>(
        &self,
        url: &str,
        request_fn: F,
    ) -> Result<serde_json::Value>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = reqwest::Result<reqwest::Response>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.http_config.max_retries {
            attempts += 1;

            match request_fn().await {
                Ok(response) => {
                    let status = response.status();

                    if status.is_success() {
                        let json = response.json::<serde_json::Value>().await.map_err(|e| {
                            WebToolError::Parsing(format!("Failed to parse JSON response: {}", e))
                        })?;

                        info!("Successfully posted to {}", url);
                        return Ok(json);
                    } else if status.is_server_error() && attempts < self.http_config.max_retries {
                        warn!(
                            "Server error {} from {}, attempt {}/{}",
                            status, url, attempts, self.http_config.max_retries
                        );
                        last_error = Some(format!("HTTP {}", status));

                        let delay = self.calculate_retry_delay(attempts);
                        debug!("Retrying after {:?}", delay);
                        tokio::time::sleep(delay).await;
                    } else {
                        let error_text = response.text().await.unwrap_or_default();
                        return Err(WebToolError::Api(format!(
                            "HTTP {} from {}: {}",
                            status, url, error_text
                        )));
                    }
                }
                Err(e) => {
                    if attempts < self.http_config.max_retries {
                        warn!(
                            "Request failed for {}, attempt {}/{}: {}",
                            url, attempts, self.http_config.max_retries, e
                        );
                        last_error = Some(e.to_string());

                        let delay = self.calculate_retry_delay(attempts);
                        debug!("Retrying after {:?}", delay);
                        tokio::time::sleep(delay).await;
                    } else {
                        return Err(WebToolError::Api(format!(
                            "Request failed after {} attempts: {}",
                            attempts, e
                        )));
                    }
                }
            }
        }

        Err(WebToolError::Api(format!(
            "Request failed after {} attempts: {}",
            attempts,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )))
    }

    /// Make a GET request with retry logic
    pub async fn get(&self, url: &str) -> Result<String> {
        self.get_with_headers(url, HashMap::new()).await
    }

    /// Make a GET request with headers and retry logic
    pub async fn get_with_headers(
        &self,
        url: &str,
        headers: HashMap<String, String>,
    ) -> Result<String> {
        debug!("GET request to: {}", url);

        self.execute_with_retry(url, || {
            let mut request = self.http_client.get(url);

            // Add headers
            for (key, value) in &headers {
                request = request.header(key, value);
            }

            request.send()
        })
        .await
    }

    /// Make GET request with query parameters
    pub async fn get_with_params(
        &self,
        url: &str,
        params: &HashMap<String, String>,
    ) -> Result<String> {
        self.get_with_params_and_headers(url, params, HashMap::new())
            .await
    }

    /// Make GET request with query parameters and headers
    pub async fn get_with_params_and_headers(
        &self,
        url: &str,
        params: &HashMap<String, String>,
        headers: HashMap<String, String>,
    ) -> Result<String> {
        debug!("GET request to: {} with params: {:?}", url, params);

        self.execute_with_retry(url, || {
            let mut request = self.http_client.get(url);

            // Add query parameters
            for (key, value) in params {
                request = request.query(&[(key, value)]);
            }

            // Add headers
            for (key, value) in &headers {
                request = request.header(key, value);
            }

            request.send()
        })
        .await
    }

    /// Make a POST request with JSON body
    pub async fn post<T: Serialize>(&self, url: &str, body: &T) -> Result<serde_json::Value> {
        self.post_with_headers(url, body, HashMap::new()).await
    }

    /// Make a POST request with JSON body and headers
    pub async fn post_with_headers<T: Serialize>(
        &self,
        url: &str,
        body: &T,
        headers: HashMap<String, String>,
    ) -> Result<serde_json::Value> {
        debug!("POST request to: {}", url);

        self.execute_post_with_retry(url, || {
            let mut request = self.http_client.post(url).json(body);

            // Add headers
            for (key, value) in &headers {
                request = request.header(key, value);
            }

            request.send()
        })
        .await
    }

    /// Make a DELETE request
    pub async fn delete(&self, url: &str) -> Result<()> {
        debug!("DELETE request to: {}", url);

        let response = self
            .http_client
            .delete(url)
            .send()
            .await
            .map_err(|e| WebToolError::Network(format!("DELETE request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(WebToolError::Api(format!(
                "HTTP {} from {}: {}",
                status, url, error_text
            )));
        }

        info!("Successfully deleted: {}", url);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_client_creation() {
        let client = WebClient::default().unwrap();
        assert!(client.api_keys.is_empty());
        assert!(!client.config.is_empty()); // Config always has default values
    }

    #[test]
    fn test_with_api_key() {
        let client = WebClient::default()
            .unwrap()
            .with_twitter_token("test_token")
            .with_exa_key("exa_key");

        assert_eq!(
            client.get_api_key("twitter"),
            Some(&"test_token".to_string())
        );
        assert_eq!(client.get_api_key("exa"), Some(&"exa_key".to_string()));
        assert_eq!(client.get_api_key("unknown"), None);
    }

    #[test]
    fn test_config() {
        let mut client = WebClient::default().unwrap();
        client.set_config("test_key", "test_value");

        // Config no longer stores arbitrary test keys, only predefined URLs
        assert_eq!(client.get_config("unknown"), None);
    }
}
