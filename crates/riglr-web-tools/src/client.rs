//! Web client for interacting with various web APIs

use crate::error::WebToolError;
use core::future::Future;
use core::result::Result;
use core::time::Duration;
use reqwest::{Client as ReqwestClient, ClientBuilder};
use serde::Serialize;
use std::collections::HashMap;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Result type alias for web client operations
type ClientResult<T> = Result<T, WebToolError>;

/// Configuration for HTTP client
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Enable exponential backoff
    pub exponential_backoff: bool,
    /// Jitter for retry delays (0.0 to 1.0)
    pub jitter_factor: f32,
    /// Maximum retries
    pub max_retries: u32,
    /// Retry delay
    pub retry_delay: Duration,
    /// Request timeout
    pub timeout: Duration,
    /// User agent
    pub user_agent: String,
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
    /// Alternative data API key
    pub alternative: Option<String>,
    /// `CryptoPanic` API key
    pub cryptopanic: Option<String>,
    /// `DexScreener` API key (optional)
    pub dexscreener: Option<String>,
    /// Exa API key
    pub exa: Option<String>,
    /// `LunarCrush` API key
    pub lunarcrush: Option<String>,
    /// `NewsAPI` key
    pub newsapi: Option<String>,
    /// Generic fallback for other services
    pub other: HashMap<String, String>,
    /// Twitter/X Bearer Token
    pub twitter: Option<String>,
}

impl ApiKeys {
    /// Check if an API key exists
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Get an API key by name
    #[must_use]
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

    /// Insert a new API key
    pub fn insert(&mut self, key: &str, value: String) {
        match key {
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

    /// Check if all API keys are empty
    #[must_use]
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

    /// Get the number of configured API keys
    #[must_use]
    pub fn len(&self) -> usize {
        let mut count: usize = 0;
        if self.twitter.is_some() {
            count = count.saturating_add(1);
        }
        if self.exa.is_some() {
            count = count.saturating_add(1);
        }
        if self.dexscreener.is_some() {
            count = count.saturating_add(1);
        }
        if self.newsapi.is_some() {
            count = count.saturating_add(1);
        }
        if self.cryptopanic.is_some() {
            count = count.saturating_add(1);
        }
        if self.lunarcrush.is_some() {
            count = count.saturating_add(1);
        }
        if self.alternative.is_some() {
            count = count.saturating_add(1);
        }
        count.saturating_add(self.other.len())
    }
}

/// Type-safe client configuration
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Base URL overrides for testing
    pub base_urls: BaseUrls,
    /// Rate limiting settings
    pub rate_limits: RateLimits,
}

impl Config {
    /// Get a configuration value by key
    #[must_use]
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

    /// Insert a configuration value
    pub fn insert(&mut self, key: &str, value: String) {
        match key {
            "dexscreener_url" => self.base_urls.dexscreener = value,
            "exa_url" => self.base_urls.exa = value,
            "newsapi_url" => self.base_urls.newsapi = value,
            "cryptopanic_url" => self.base_urls.cryptopanic = value,
            "lunarcrush_url" => self.base_urls.lunarcrush = value,
            "twitter_url" => self.base_urls.twitter = value,
            _ => {}
        }
    }

    /// Check if the config is empty
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false // Config always has default values
    }

    /// Get the number of configuration entries
    #[must_use]
    pub const fn len(&self) -> usize {
        6 // Fixed number of base URLs
    }
}

/// Base URL configuration for various services
#[derive(Debug, Clone)]
pub struct BaseUrls {
    /// `CryptoPanic` API base URL
    pub cryptopanic: String,
    /// `DexScreener` API base URL
    pub dexscreener: String,
    /// Exa API base URL
    pub exa: String,
    /// `LunarCrush` API base URL
    pub lunarcrush: String,
    /// News API base URL
    pub newsapi: String,
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
    /// `DexScreener` requests per minute limit
    pub dexscreener_per_minute: u32,
    /// Exa API requests per minute limit
    pub exa_per_minute: u32,
    /// News API requests per minute limit
    pub newsapi_per_minute: u32,
    /// Twitter requests per minute limit
    pub twitter_per_minute: u32,
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
pub struct Client {
    /// Type-safe API keys
    pub api_keys: ApiKeys,
    /// Type-safe configuration
    pub config: Config,
    /// HTTP client for making requests
    pub http_client: ReqwestClient,
    /// HTTP configuration
    pub http_config: HttpConfig,
}

/// Backward compatibility aliases
#[expect(clippy::module_name_repetitions)]
pub type WebClient = Client;
#[expect(clippy::module_name_repetitions)]
pub type ClientConfig = Config;

impl Default for Client {
    fn default() -> Self {
        let http_config = HttpConfig::default();
        // Create HTTP client - in Default implementation we use unwrap_or_else
        // to provide a basic fallback client if the configured one fails
        let http_client = ClientBuilder::new()
            .timeout(http_config.timeout)
            .user_agent(&http_config.user_agent)
            .build()
            .unwrap_or_else(|_| {
                // Fallback to a basic client if the configured one fails
                ClientBuilder::new().build().unwrap_or_else(|_| {
                    // If this fails, there's a fundamental issue with the HTTP client library
                    // Use a simpler client as last resort
                    #[expect(clippy::unwrap_used)]
                    {
                        ClientBuilder::new().build().unwrap()
                    }
                })
            });

        Self {
            http_client,
            api_keys: ApiKeys::default(),
            config: Config::default(),
            http_config,
        }
    }
}

impl Client {
    /// Calculate retry delay with exponential backoff and jitter
    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.http_config.retry_delay;

        let delay = if self.http_config.exponential_backoff {
            // Exponential backoff: delay * 2^(attempt - 1)
            base_delay.saturating_mul(2_u32.pow(attempt.saturating_sub(1)))
        } else {
            // Linear backoff: delay * attempt
            base_delay.saturating_mul(attempt)
        };

        // Add jitter if configured
        if self.http_config.jitter_factor > 0.0 {
            use rand::Rng;
            let mut rng = rand::rng();
            // Use f64 for better precision and safer conversion
            #[expect(clippy::cast_possible_truncation)]
            let delay_ms = delay.as_millis().min(u128::from(u64::MAX)) as u64;
            #[expect(clippy::cast_precision_loss)]
            let jitter_range = (delay_ms as f64) * f64::from(self.http_config.jitter_factor);
            let jitter = rng.random_range(-jitter_range..=jitter_range);
            #[expect(
                clippy::cast_precision_loss,
                clippy::cast_sign_loss,
                clippy::cast_possible_truncation
            )]
            let final_delay = ((delay_ms as f64) + jitter)
                .max(0.0)
                .clamp(0.0, u64::MAX as f64) as u64;
            return Duration::from_millis(final_delay);
        }
        delay
    }

    /// Make a DELETE request
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or returns a non-success status
    pub async fn delete(&self, url: &str) -> ClientResult<()> {
        debug!("DELETE request to: {}", url);

        let response = self
            .http_client
            .delete(url)
            .send()
            .await
            .map_err(|e| WebToolError::Network(format!("DELETE request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(WebToolError::Api(format!(
                "HTTP {status} from {url}: {error_text}"
            )));
        }

        info!("Successfully deleted: {}", url);
        Ok(())
    }

    /// Execute HTTP request with retry logic (internal helper)
    async fn execute_with_retry<F, Fut>(&self, url: &str, request_fn: F) -> ClientResult<String>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = reqwest::Result<reqwest::Response>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.http_config.max_retries {
            attempts = attempts.saturating_add(1);

            let request_result = request_fn().await;
            match request_result {
                Ok(response) => {
                    let handle_result = self.handle_response(response, url).await;
                    match handle_result {
                        Ok(text) => return Ok(text),
                        Err(WebToolError::Network(_))
                            if attempts < self.http_config.max_retries =>
                        {
                            // Network errors are retryable
                            last_error = Some("Network error".to_string());
                            self.retry_with_delay(attempts).await;
                        }
                        Err(e) => return Err(e), // Other errors (rate limit, API errors) are not retryable
                    }
                }
                Err(e) => {
                    if attempts < self.http_config.max_retries {
                        self.log_retry_attempt(url, attempts, &e);
                        last_error = Some(e.to_string());
                        self.retry_with_delay(attempts).await;
                    } else {
                        return Err(WebToolError::Api(format!(
                            "Request failed after {attempts} attempts: {e}"
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
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or returns a non-success status
    pub async fn get(&self, url: &str) -> ClientResult<String> {
        self.get_with_headers(url, HashMap::new()).await
    }

    /// Get API key for a service
    #[must_use]
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
    #[must_use]
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

    /// Make a GET request with headers and retry logic
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or returns a non-success status
    pub async fn get_with_headers(
        &self,
        url: &str,
        headers: HashMap<String, String>,
    ) -> ClientResult<String> {
        debug!("GET request to: {}", url);

        return self
            .execute_with_retry(url, || {
                let mut request = self.http_client.get(url);

                // Add headers
                for (key, value) in &headers {
                    request = request.header(key, value);
                }

                request.send()
            })
            .await;
    }

    /// Make GET request with query parameters
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or returns a non-success status
    pub async fn get_with_params(
        &self,
        url: &str,
        params: &HashMap<String, String>,
    ) -> ClientResult<String> {
        let result = self
            .get_with_params_and_headers(url, params, HashMap::new())
            .await;
        result
    }

    /// Make GET request with query parameters and headers
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or returns a non-success status
    pub async fn get_with_params_and_headers(
        &self,
        url: &str,
        params: &HashMap<String, String>,
        headers: HashMap<String, String>,
    ) -> ClientResult<String> {
        debug!("GET request to: {} with params: {:?}", url, params);

        return self
            .execute_with_retry(url, || {
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
            .await;
    }

    // Note: handle_json_response method removed as it was unused

    /// Handle HTTP response and classify errors
    async fn handle_response(
        &self,
        response: reqwest::Response,
        url: &str,
    ) -> ClientResult<String> {
        let status = response.status();

        if status.is_success() {
            let text = response
                .text()
                .await
                .map_err(|e| WebToolError::Network(format!("Failed to read response: {e}")))?;
            info!("Successfully fetched {} bytes from {}", text.len(), url);
            return Ok(text);
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let error_text = response.text().await.unwrap_or_default();
            return Err(WebToolError::RateLimit(format!(
                "HTTP 429 from {url}: {error_text}"
            )));
        }

        if Self::is_retryable_server_error(status) {
            return Err(WebToolError::Network(format!("HTTP {status}")));
        }

        // All other errors (e.g., 4xx) are permanent API errors
        let error_text = response.text().await.unwrap_or_default();
        Err(WebToolError::Api(format!(
            "HTTP {status} from {url}: {error_text}"
        )))
    }

    /// Check if status code represents a retryable server error
    const fn is_retryable_server_error(status: reqwest::StatusCode) -> bool {
        matches!(
            status,
            reqwest::StatusCode::BAD_GATEWAY
                | reqwest::StatusCode::SERVICE_UNAVAILABLE
                | reqwest::StatusCode::GATEWAY_TIMEOUT
        )
    }

    /// Log retry attempt
    fn log_retry_attempt(&self, url: &str, attempts: u32, error: &reqwest::Error) {
        warn!(
            "Request failed for {}, attempt {}/{}: {}",
            url, attempts, self.http_config.max_retries, error
        );
    }

    /// Make a POST request with JSON body
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or returns a non-success status
    pub async fn post<T: Serialize + Sync>(
        &self,
        url: &str,
        body: &T,
    ) -> ClientResult<serde_json::Value> {
        let result = self.post_with_headers(url, body, HashMap::new()).await;
        result
    }

    /// Make a POST request with JSON body and headers
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or returns a non-success status
    pub async fn post_with_headers<T: Serialize + Sync>(
        &self,
        url: &str,
        body: &T,
        headers: HashMap<String, String>,
    ) -> ClientResult<serde_json::Value> {
        debug!("POST request to: {}", url);

        let response_text = self
            .execute_with_retry(url, || {
                let mut request = self.http_client.post(url).json(body);

                // Add headers
                for (key, value) in &headers {
                    request = request.header(key, value);
                }

                request.send()
            })
            .await?;

        // Parse the response text as JSON
        serde_json::from_str(&response_text)
            .map_err(|e| WebToolError::Parsing(format!("Failed to parse JSON response: {e}")))
    }

    /// Perform retry delay and logging
    async fn retry_with_delay(&self, attempts: u32) {
        let delay = self.calculate_retry_delay(attempts);
        debug!("Retrying after {:?}", delay);
        sleep(delay).await;
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

    /// Set API key for a service (for backwards compatibility)
    #[must_use]
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

    /// Create with custom HTTP configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created
    pub fn with_config(http_config: HttpConfig) -> ClientResult<Self> {
        let http_client = ClientBuilder::new()
            .timeout(http_config.timeout)
            .user_agent(&http_config.user_agent)
            .build()
            .map_err(|e| WebToolError::Client(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            http_client,
            api_keys: ApiKeys::default(),
            config: Config::default(),
            http_config,
        })
    }

    /// Set `DexScreener` API key (if required)
    #[must_use]
    pub fn with_dexscreener_key<S: Into<String>>(mut self, key: S) -> Self {
        self.api_keys.dexscreener = Some(key.into());
        self
    }

    /// Set Exa API key
    #[must_use]
    pub fn with_exa_key<S: Into<String>>(mut self, key: S) -> Self {
        self.api_keys.exa = Some(key.into());
        self
    }

    /// Set News API key
    #[must_use]
    pub fn with_news_api_key<S: Into<String>>(mut self, key: S) -> Self {
        self.api_keys.newsapi = Some(key.into());
        self
    }

    /// Set Twitter/X Bearer Token
    #[must_use]
    pub fn with_twitter_token<S: Into<String>>(mut self, token: S) -> Self {
        self.api_keys.twitter = Some(token.into());
        self
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_web_client_creation() {
        let client = Client::default();
        assert!(client.api_keys.is_empty());
        assert!(!client.config.is_empty()); // Config always has default values
    }

    #[test]
    fn test_with_api_key() {
        let client = Client::default()
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
        let mut client = Client::default();
        client.set_config("test_key", "test_value");

        // Config no longer stores arbitrary test keys, only predefined URLs
        assert_eq!(client.get_config("unknown"), None);
    }

    #[test]
    fn test_http_config_default() {
        let config = HttpConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay, Duration::from_millis(500));
        assert_eq!(config.user_agent, "riglr-web-tools/0.1.0");
        assert!(config.exponential_backoff);
        assert!((config.jitter_factor - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_api_keys_default() {
        let keys = ApiKeys::default();
        assert!(keys.is_empty());
        assert!(keys.twitter.is_none());
        assert!(keys.exa.is_none());
        assert!(keys.dexscreener.is_none());
        assert!(keys.newsapi.is_none());
        assert!(keys.cryptopanic.is_none());
        assert!(keys.lunarcrush.is_none());
        assert!(keys.alternative.is_none());
        assert!(keys.other.is_empty());
    }

    #[test]
    fn test_api_keys_is_empty() {
        let mut keys = ApiKeys::default();
        assert!(keys.is_empty());

        keys.twitter = Some("token".to_string());
        assert!(!keys.is_empty());

        keys = ApiKeys::default();
        keys.other.insert("custom".to_string(), "value".to_string());
        assert!(!keys.is_empty());
    }

    #[test]
    fn test_api_keys_get() {
        let keys = ApiKeys {
            twitter: Some("twitter_token".to_string()),
            exa: Some("exa_key".to_string()),
            dexscreener: Some("dex_key".to_string()),
            newsapi: Some("news_key".to_string()),
            cryptopanic: Some("crypto_key".to_string()),
            lunarcrush: Some("lunar_key".to_string()),
            alternative: Some("alt_key".to_string()),
            other: HashMap::from([("custom".to_string(), "custom_key".to_string())]),
        };

        assert_eq!(keys.get("twitter"), Some(&"twitter_token".to_string()));
        assert_eq!(keys.get("exa"), Some(&"exa_key".to_string()));
        assert_eq!(keys.get("dexscreener"), Some(&"dex_key".to_string()));
        assert_eq!(keys.get("newsapi"), Some(&"news_key".to_string()));
        assert_eq!(keys.get("cryptopanic"), Some(&"crypto_key".to_string()));
        assert_eq!(keys.get("lunarcrush"), Some(&"lunar_key".to_string()));
        assert_eq!(keys.get("alternative"), Some(&"alt_key".to_string()));
        assert_eq!(keys.get("custom"), Some(&"custom_key".to_string()));
        assert_eq!(keys.get("unknown"), None);
    }

    #[test]
    fn test_api_keys_len() {
        let mut keys = ApiKeys::default();
        assert_eq!(keys.len(), 0);

        keys.twitter = Some("token".to_string());
        assert_eq!(keys.len(), 1);

        keys.exa = Some("key".to_string());
        keys.dexscreener = Some("key".to_string());
        keys.newsapi = Some("key".to_string());
        keys.cryptopanic = Some("key".to_string());
        keys.lunarcrush = Some("key".to_string());
        keys.alternative = Some("key".to_string());
        assert_eq!(keys.len(), 7);

        keys.other
            .insert("custom1".to_string(), "value1".to_string());
        keys.other
            .insert("custom2".to_string(), "value2".to_string());
        assert_eq!(keys.len(), 9);
    }

    #[test]
    fn test_api_keys_contains_key() {
        let mut keys = ApiKeys::default();
        assert!(!keys.contains_key("twitter"));

        keys.twitter = Some("token".to_string());
        assert!(keys.contains_key("twitter"));
        assert!(!keys.contains_key("exa"));

        keys.other.insert("custom".to_string(), "value".to_string());
        assert!(keys.contains_key("custom"));
        assert!(!keys.contains_key("unknown"));
    }

    #[test]
    fn test_api_keys_insert() {
        let mut keys = ApiKeys::default();

        keys.insert("twitter", "token".to_string());
        assert_eq!(keys.twitter, Some("token".to_string()));

        keys.insert("exa", "key".to_string());
        assert_eq!(keys.exa, Some("key".to_string()));

        keys.insert("dexscreener", "key".to_string());
        assert_eq!(keys.dexscreener, Some("key".to_string()));

        keys.insert("newsapi", "key".to_string());
        assert_eq!(keys.newsapi, Some("key".to_string()));

        keys.insert("cryptopanic", "key".to_string());
        assert_eq!(keys.cryptopanic, Some("key".to_string()));

        keys.insert("lunarcrush", "key".to_string());
        assert_eq!(keys.lunarcrush, Some("key".to_string()));

        keys.insert("alternative", "key".to_string());
        assert_eq!(keys.alternative, Some("key".to_string()));

        keys.insert("custom", "value".to_string());
        assert_eq!(keys.other.get("custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_client_config_default() {
        let config = Config::default();
        assert!(!config.is_empty()); // Always returns false
        assert_eq!(config.len(), 6); // Fixed number of URLs
    }

    #[test]
    fn test_client_config_get() {
        let config = Config::default();
        assert!(config.get("dexscreener_url").is_some());
        assert!(config.get("exa_url").is_some());
        assert!(config.get("newsapi_url").is_some());
        assert!(config.get("cryptopanic_url").is_some());
        assert!(config.get("lunarcrush_url").is_some());
        assert!(config.get("twitter_url").is_some());
        assert_eq!(config.get("unknown"), None);
    }

    #[test]
    fn test_client_config_insert() {
        let mut config = Config::default();
        let old_dex_url = config.base_urls.dexscreener.clone();

        config.insert("dexscreener_url", "https://custom.com".to_string());
        assert_eq!(config.base_urls.dexscreener, "https://custom.com");
        assert_ne!(config.base_urls.dexscreener, old_dex_url);

        config.insert("exa_url", "https://exa.custom.com".to_string());
        assert_eq!(config.base_urls.exa, "https://exa.custom.com");

        config.insert("newsapi_url", "https://news.custom.com".to_string());
        assert_eq!(config.base_urls.newsapi, "https://news.custom.com");

        config.insert("cryptopanic_url", "https://crypto.custom.com".to_string());
        assert_eq!(config.base_urls.cryptopanic, "https://crypto.custom.com");

        config.insert("lunarcrush_url", "https://lunar.custom.com".to_string());
        assert_eq!(config.base_urls.lunarcrush, "https://lunar.custom.com");

        config.insert("twitter_url", "https://twitter.custom.com".to_string());
        assert_eq!(config.base_urls.twitter, "https://twitter.custom.com");

        // Test unknown key does nothing
        let old_dex_url = config.base_urls.dexscreener.clone();
        config.insert("unknown", "value".to_string());
        assert_eq!(config.base_urls.dexscreener, old_dex_url);
    }

    #[test]
    fn test_base_urls_default() {
        let urls = BaseUrls::default();
        assert_eq!(urls.dexscreener, "https://api.dexscreener.com/latest");
        assert_eq!(urls.exa, "https://api.exa.ai");
        assert_eq!(urls.newsapi, "https://newsapi.org/v2");
        assert_eq!(urls.cryptopanic, "https://cryptopanic.com/api/v1");
        assert_eq!(urls.lunarcrush, "https://lunarcrush.com/api/3");
        assert_eq!(urls.twitter, "https://api.twitter.com/2");
    }

    #[test]
    fn test_rate_limits_default() {
        let limits = RateLimits::default();
        assert_eq!(limits.dexscreener_per_minute, 300);
        assert_eq!(limits.twitter_per_minute, 300);
        assert_eq!(limits.newsapi_per_minute, 500);
        assert_eq!(limits.exa_per_minute, 100);
    }

    #[test]
    fn test_web_client_default() {
        let client = Client::default();
        assert!(client.api_keys.is_empty());
        assert!(!client.config.is_empty());
        assert_eq!(client.http_config.max_retries, 3);
    }

    #[test]
    fn test_web_client_with_config() {
        let config = HttpConfig {
            timeout: Duration::from_secs(60),
            max_retries: 5,
            retry_delay: Duration::from_millis(1000),
            user_agent: "custom-agent".to_string(),
            exponential_backoff: false,
            jitter_factor: 0.2,
        };

        let client =
            Client::with_config(config).expect("Failed to create Client with custom config");
        assert_eq!(client.http_config.timeout, Duration::from_secs(60));
        assert_eq!(client.http_config.max_retries, 5);
        assert_eq!(client.http_config.retry_delay, Duration::from_millis(1000));
        assert_eq!(client.http_config.user_agent, "custom-agent");
        assert!(!client.http_config.exponential_backoff);
        {
            assert!((client.http_config.jitter_factor - 0.2).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_web_client_with_api_key() {
        let client = Client::default()
            .with_api_key("twitter", "twitter_key")
            .with_api_key("exa", "exa_key")
            .with_api_key("dexscreener", "dex_key")
            .with_api_key("newsapi", "news_key")
            .with_api_key("cryptopanic", "crypto_key")
            .with_api_key("lunarcrush", "lunar_key")
            .with_api_key("alternative", "alt_key")
            .with_api_key("custom", "custom_key");

        assert_eq!(client.api_keys.twitter, Some("twitter_key".to_string()));
        assert_eq!(client.api_keys.exa, Some("exa_key".to_string()));
        assert_eq!(client.api_keys.dexscreener, Some("dex_key".to_string()));
        assert_eq!(client.api_keys.newsapi, Some("news_key".to_string()));
        assert_eq!(client.api_keys.cryptopanic, Some("crypto_key".to_string()));
        assert_eq!(client.api_keys.lunarcrush, Some("lunar_key".to_string()));
        assert_eq!(client.api_keys.alternative, Some("alt_key".to_string()));
        assert_eq!(
            client.api_keys.other.get("custom"),
            Some(&"custom_key".to_string())
        );
    }

    #[test]
    fn test_web_client_builder_methods() {
        let client = Client::default()
            .with_twitter_token("twitter_token")
            .with_exa_key("exa_key")
            .with_dexscreener_key("dex_key")
            .with_news_api_key("news_key");

        assert_eq!(client.api_keys.twitter, Some("twitter_token".to_string()));
        assert_eq!(client.api_keys.exa, Some("exa_key".to_string()));
        assert_eq!(client.api_keys.dexscreener, Some("dex_key".to_string()));
        assert_eq!(client.api_keys.newsapi, Some("news_key".to_string()));
    }

    #[test]
    fn test_web_client_set_config() {
        let mut client = Client::default();
        let original_dex_url = client.config.base_urls.dexscreener.clone();

        client.set_config("base_url", "https://custom-dex.com");
        assert_eq!(
            client.config.base_urls.dexscreener,
            "https://custom-dex.com"
        );
        assert_ne!(client.config.base_urls.dexscreener, original_dex_url);

        client.set_config("dexscreener_base_url", "https://dex.custom.com");
        assert_eq!(
            client.config.base_urls.dexscreener,
            "https://dex.custom.com"
        );

        client.set_config("exa_base_url", "https://exa.custom.com");
        assert_eq!(client.config.base_urls.exa, "https://exa.custom.com");

        client.set_config("newsapi_base_url", "https://news.custom.com");
        assert_eq!(client.config.base_urls.newsapi, "https://news.custom.com");

        client.set_config("cryptopanic_base_url", "https://crypto.custom.com");
        assert_eq!(
            client.config.base_urls.cryptopanic,
            "https://crypto.custom.com"
        );

        client.set_config("lunarcrush_base_url", "https://lunar.custom.com");
        assert_eq!(
            client.config.base_urls.lunarcrush,
            "https://lunar.custom.com"
        );

        client.set_config("twitter_base_url", "https://twitter.custom.com");
        assert_eq!(
            client.config.base_urls.twitter,
            "https://twitter.custom.com"
        );

        // Test unknown key (should log warning)
        let old_dex_url = client.config.base_urls.dexscreener.clone();
        client.set_config("unknown_key", "value");
        assert_eq!(client.config.base_urls.dexscreener, old_dex_url);
    }

    #[test]
    fn test_web_client_get_api_key() {
        let client = Client::default()
            .with_api_key("twitter", "twitter_key")
            .with_api_key("exa", "exa_key")
            .with_api_key("dexscreener", "dex_key")
            .with_api_key("newsapi", "news_key")
            .with_api_key("cryptopanic", "crypto_key")
            .with_api_key("lunarcrush", "lunar_key")
            .with_api_key("alternative", "alt_key")
            .with_api_key("custom", "custom_key");

        assert_eq!(
            client.get_api_key("twitter"),
            Some(&"twitter_key".to_string())
        );
        assert_eq!(client.get_api_key("exa"), Some(&"exa_key".to_string()));
        assert_eq!(
            client.get_api_key("dexscreener"),
            Some(&"dex_key".to_string())
        );
        assert_eq!(client.get_api_key("newsapi"), Some(&"news_key".to_string()));
        assert_eq!(
            client.get_api_key("cryptopanic"),
            Some(&"crypto_key".to_string())
        );
        assert_eq!(
            client.get_api_key("lunarcrush"),
            Some(&"lunar_key".to_string())
        );
        assert_eq!(
            client.get_api_key("alternative"),
            Some(&"alt_key".to_string())
        );
        assert_eq!(
            client.get_api_key("custom"),
            Some(&"custom_key".to_string())
        );
        assert_eq!(client.get_api_key("unknown"), None);
    }

    #[test]
    fn test_web_client_get_config() {
        let client = Client::default();

        assert!(client.get_config("dexscreener_base_url").is_some());
        assert!(client.get_config("base_url").is_some());
        assert!(client.get_config("exa_base_url").is_some());
        assert!(client.get_config("newsapi_base_url").is_some());
        assert!(client.get_config("cryptopanic_base_url").is_some());
        assert!(client.get_config("lunarcrush_base_url").is_some());
        assert!(client.get_config("twitter_base_url").is_some());
        assert_eq!(client.get_config("unknown"), None);

        // Test that base_url and dexscreener_base_url return the same value
        assert_eq!(
            client.get_config("base_url"),
            client.get_config("dexscreener_base_url")
        );
    }

    #[test]
    fn test_calculate_retry_delay_exponential_backoff() {
        let config = HttpConfig {
            retry_delay: Duration::from_millis(100),
            exponential_backoff: true,
            jitter_factor: 0.0, // No jitter for predictable testing
            ..Default::default()
        };
        let client = Client::with_config(config)
            .expect("Failed to create Client for exponential backoff test");

        // Test exponential backoff: delay * 2^(attempt - 1)
        let delay1 = client.calculate_retry_delay(1);
        assert_eq!(delay1, Duration::from_millis(100)); // 100 * 2^0 = 100

        let delay2 = client.calculate_retry_delay(2);
        assert_eq!(delay2, Duration::from_millis(200)); // 100 * 2^1 = 200

        let delay3 = client.calculate_retry_delay(3);
        assert_eq!(delay3, Duration::from_millis(400)); // 100 * 2^2 = 400
    }

    #[test]
    fn test_calculate_retry_delay_linear_backoff() {
        let config = HttpConfig {
            retry_delay: Duration::from_millis(100),
            exponential_backoff: false,
            jitter_factor: 0.0, // No jitter for predictable testing
            ..Default::default()
        };
        let client =
            Client::with_config(config).expect("Failed to create Client for linear backoff test");

        // Test linear backoff: delay * attempt
        let delay1 = client.calculate_retry_delay(1);
        assert_eq!(delay1, Duration::from_millis(100)); // 100 * 1 = 100

        let delay2 = client.calculate_retry_delay(2);
        assert_eq!(delay2, Duration::from_millis(200)); // 100 * 2 = 200

        let delay3 = client.calculate_retry_delay(3);
        assert_eq!(delay3, Duration::from_millis(300)); // 100 * 3 = 300
    }

    #[test]
    fn test_calculate_retry_delay_with_jitter() {
        let config = HttpConfig {
            retry_delay: Duration::from_millis(100),
            exponential_backoff: false,
            jitter_factor: 0.5, // 50% jitter
            ..Default::default()
        };
        let client = Client::with_config(config).expect("Failed to create Client for jitter test");

        // With jitter, the delay should vary but be within expected range
        let delay = client.calculate_retry_delay(1);
        // The delay should be around 100ms but with jitter (50ms range)
        assert!(delay.as_millis() >= 50 && delay.as_millis() <= 150);
    }

    #[test]
    fn test_calculate_retry_delay_saturating_sub() {
        let config = HttpConfig {
            retry_delay: Duration::from_millis(100),
            exponential_backoff: true,
            jitter_factor: 0.0,
            ..Default::default()
        };
        let client =
            Client::with_config(config).expect("Failed to create Client for saturating_sub test");

        // Test with attempt = 0 (should not panic due to saturating_sub)
        let delay = client.calculate_retry_delay(0);
        assert_eq!(delay, Duration::from_millis(100)); // 100 * 2^0 = 100
    }
}
