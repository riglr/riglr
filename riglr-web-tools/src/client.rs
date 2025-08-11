//! Web client for interacting with various web APIs

use crate::error::{Result, WebToolError};
use reqwest::{Client, ClientBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

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
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            user_agent: "riglr-web-tools/0.1.0".to_string(),
        }
    }
}

/// A client for interacting with various web APIs and services
#[derive(Debug, Clone)]
pub struct WebClient {
    /// HTTP client for making requests
    pub http_client: Client,
    /// API keys for various services
    pub api_keys: HashMap<String, String>,
    /// Optional configuration
    pub config: HashMap<String, String>,
    /// HTTP configuration
    pub http_config: HttpConfig,
}

impl WebClient {
    /// Create a new web client
    pub fn new() -> Result<Self> {
        let http_config = HttpConfig::default();
        
        let http_client = ClientBuilder::new()
            .timeout(http_config.timeout)
            .user_agent(&http_config.user_agent)
            .gzip(true)
            .build()
            .map_err(|e| WebToolError::Client(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            http_client,
            api_keys: HashMap::new(),
            config: HashMap::new(),
            http_config,
        })
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
            api_keys: HashMap::new(),
            config: HashMap::new(),
            http_config,
        })
    }

    /// Set API key for a service
    pub fn with_api_key<S1: Into<String>, S2: Into<String>>(mut self, service: S1, api_key: S2) -> Self {
        self.api_keys.insert(service.into(), api_key.into());
        self
    }

    /// Set Twitter/X Bearer Token
    pub fn with_twitter_token<S: Into<String>>(self, token: S) -> Self {
        self.with_api_key("twitter", token)
    }

    /// Set Exa API key
    pub fn with_exa_key<S: Into<String>>(self, key: S) -> Self {
        self.with_api_key("exa", key)
    }

    /// Set DexScreener API key (if required)
    pub fn with_dexscreener_key<S: Into<String>>(self, key: S) -> Self {
        self.with_api_key("dexscreener", key)
    }

    /// Set News API key
    pub fn with_news_api_key<S: Into<String>>(self, key: S) -> Self {
        self.with_api_key("newsapi", key)
    }

    /// Set configuration option
    pub fn set_config<S: Into<String>>(&mut self, key: S, value: S) {
        self.config.insert(key.into(), value.into());
    }

    /// Get API key for a service
    pub fn get_api_key(&self, service: &str) -> Option<&String> {
        self.api_keys.get(service)
    }

    /// Get config value
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
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
        
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.http_config.max_retries {
            attempts += 1;
            
            let mut request = self.http_client.get(url);
            
            // Add headers
            for (key, value) in &headers {
                request = request.header(key, value);
            }
            
            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    
                    if status.is_success() {
                        let text = response.text().await
                            .map_err(|e| WebToolError::Request(format!("Failed to read response: {}", e)))?;
                        
                        info!("Successfully fetched {} bytes from {}", text.len(), url);
                        return Ok(text);
                    } else if status.is_server_error() && attempts < self.http_config.max_retries {
                        // Retry on server errors
                        warn!("Server error {} from {}, attempt {}/{}", status, url, attempts, self.http_config.max_retries);
                        last_error = Some(format!("HTTP {}", status));
                        tokio::time::sleep(self.http_config.retry_delay * attempts).await;
                    } else {
                        // Don't retry on client errors
                        let error_text = response.text().await.unwrap_or_default();
                        return Err(WebToolError::Request(format!(
                            "HTTP {} from {}: {}",
                            status, url, error_text
                        )));
                    }
                }
                Err(e) => {
                    if attempts < self.http_config.max_retries {
                        warn!("Request failed for {}, attempt {}/{}: {}", url, attempts, self.http_config.max_retries, e);
                        last_error = Some(e.to_string());
                        tokio::time::sleep(self.http_config.retry_delay * attempts).await;
                    } else {
                        return Err(WebToolError::Request(format!("Request failed after {} attempts: {}", attempts, e)));
                    }
                }
            }
        }
        
        Err(WebToolError::Request(format!(
            "Request failed after {} attempts: {}",
            attempts,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )))
    }

    /// Make GET request with query parameters
    pub async fn get_with_params(
        &self,
        url: &str,
        params: &HashMap<String, String>,
    ) -> Result<String> {
        self.get_with_params_and_headers(url, params, HashMap::new()).await
    }

    /// Make GET request with query parameters and headers
    pub async fn get_with_params_and_headers(
        &self,
        url: &str,
        params: &HashMap<String, String>,
        headers: HashMap<String, String>,
    ) -> Result<String> {
        debug!("GET request to: {} with params: {:?}", url, params);
        
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.http_config.max_retries {
            attempts += 1;
            
            let mut request = self.http_client.get(url);
            
            // Add query parameters
            for (key, value) in params {
                request = request.query(&[(key, value)]);
            }
            
            // Add headers
            for (key, value) in &headers {
                request = request.header(key, value);
            }
            
            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    
                    if status.is_success() {
                        let text = response.text().await
                            .map_err(|e| WebToolError::Request(format!("Failed to read response: {}", e)))?;
                        
                        info!("Successfully fetched {} bytes from {}", text.len(), url);
                        return Ok(text);
                    } else if status.is_server_error() && attempts < self.http_config.max_retries {
                        warn!("Server error {} from {}, attempt {}/{}", status, url, attempts, self.http_config.max_retries);
                        last_error = Some(format!("HTTP {}", status));
                        tokio::time::sleep(self.http_config.retry_delay * attempts).await;
                    } else {
                        let error_text = response.text().await.unwrap_or_default();
                        return Err(WebToolError::Request(format!(
                            "HTTP {} from {}: {}",
                            status, url, error_text
                        )));
                    }
                }
                Err(e) => {
                    if attempts < self.http_config.max_retries {
                        warn!("Request failed for {}, attempt {}/{}: {}", url, attempts, self.http_config.max_retries, e);
                        last_error = Some(e.to_string());
                        tokio::time::sleep(self.http_config.retry_delay * attempts).await;
                    } else {
                        return Err(WebToolError::Request(format!("Request failed after {} attempts: {}", attempts, e)));
                    }
                }
            }
        }
        
        Err(WebToolError::Request(format!(
            "Request failed after {} attempts: {}",
            attempts,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )))
    }

    /// Make a POST request with JSON body
    pub async fn post<T: Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<serde_json::Value> {
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
        
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.http_config.max_retries {
            attempts += 1;
            
            let mut request = self.http_client
                .post(url)
                .json(body);
            
            // Add headers
            for (key, value) in &headers {
                request = request.header(key, value);
            }
            
            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    
                    if status.is_success() {
                        let json = response.json::<serde_json::Value>().await
                            .map_err(|e| WebToolError::Parse(format!("Failed to parse JSON response: {}", e)))?;
                        
                        info!("Successfully posted to {}", url);
                        return Ok(json);
                    } else if status.is_server_error() && attempts < self.http_config.max_retries {
                        warn!("Server error {} from {}, attempt {}/{}", status, url, attempts, self.http_config.max_retries);
                        last_error = Some(format!("HTTP {}", status));
                        tokio::time::sleep(self.http_config.retry_delay * attempts).await;
                    } else {
                        let error_text = response.text().await.unwrap_or_default();
                        return Err(WebToolError::Request(format!(
                            "HTTP {} from {}: {}",
                            status, url, error_text
                        )));
                    }
                }
                Err(e) => {
                    if attempts < self.http_config.max_retries {
                        warn!("Request failed for {}, attempt {}/{}: {}", url, attempts, self.http_config.max_retries, e);
                        last_error = Some(e.to_string());
                        tokio::time::sleep(self.http_config.retry_delay * attempts).await;
                    } else {
                        return Err(WebToolError::Request(format!("Request failed after {} attempts: {}", attempts, e)));
                    }
                }
            }
        }
        
        Err(WebToolError::Request(format!(
            "Request failed after {} attempts: {}",
            attempts,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )))
    }

    /// Make a DELETE request
    pub async fn delete(&self, url: &str) -> Result<()> {
        debug!("DELETE request to: {}", url);
        
        let response = self.http_client
            .delete(url)
            .send()
            .await
            .map_err(|e| WebToolError::Request(format!("DELETE request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(WebToolError::Request(format!(
                "HTTP {} from {}: {}",
                status, url, error_text
            )));
        }
        
        info!("Successfully deleted: {}", url);
        Ok(())
    }
}

impl Default for WebClient {
    fn default() -> Self {
        // Create a basic client without external dependencies
        Self {
            http_client: Client::new(), // Client::new() can't fail
            api_keys: HashMap::new(),
            config: HashMap::new(),
            http_config: HttpConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_client_creation() {
        let client = WebClient::new().unwrap();
        assert!(client.api_keys.is_empty());
        assert!(client.config.is_empty());
    }

    #[test]
    fn test_with_api_key() {
        let client = WebClient::new()
            .unwrap()
            .with_twitter_token("test_token")
            .with_exa_key("exa_key");
        
        assert_eq!(client.get_api_key("twitter"), Some(&"test_token".to_string()));
        assert_eq!(client.get_api_key("exa"), Some(&"exa_key".to_string()));
        assert_eq!(client.get_api_key("unknown"), None);
    }

    #[test]
    fn test_config() {
        let mut client = WebClient::new().unwrap();
        client.set_config("test_key", "test_value");
        
        assert_eq!(client.get_config("test_key"), Some(&"test_value".to_string()));
        assert_eq!(client.get_config("unknown"), None);
    }
}