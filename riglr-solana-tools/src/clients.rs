//! API clients for external services used by Solana tools
//!
//! This module provides pre-configured HTTP clients for external APIs like Jupiter and Pump.fun,
//! eliminating direct environment variable access from tools and enabling dependency injection.

use reqwest::Client;
use riglr_config::ProvidersConfig;
use std::sync::Arc;

/// Default Jupiter API URL
const DEFAULT_JUPITER_API_URL: &str = "https://quote-api.jup.ag/v6";

/// Default Pump.fun API URL  
const DEFAULT_PUMP_API_URL: &str = "https://pumpapi.fun/api";

/// Environment variable for Jupiter API URL
pub const JUPITER_API_URL: &str = "JUPITER_API_URL";

/// Environment variable for Pump.fun API URL
const PUMP_API_URL: &str = "PUMP_API_URL";

/// HTTP client for Jupiter aggregator API
#[derive(Debug, Clone)]
pub struct JupiterClient {
    pub(crate) http_client: Arc<Client>,
    pub(crate) api_url: String,
}

impl JupiterClient {
    /// Create a new Jupiter client from configuration
    pub fn new(config: &ProvidersConfig) -> Self {
        let api_url = config
            .jupiter_api_url
            .clone()
            .or_else(|| std::env::var(JUPITER_API_URL).ok())
            .unwrap_or_else(|| DEFAULT_JUPITER_API_URL.to_string());

        Self {
            http_client: Arc::new(Client::new()),
            api_url,
        }
    }

    /// Create a Jupiter client with custom URL (for testing)
    pub fn with_url(api_url: String) -> Self {
        Self {
            http_client: Arc::new(Client::new()),
            api_url,
        }
    }

    /// Get the Jupiter API base URL
    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    /// Get the underlying HTTP client
    pub fn http_client(&self) -> &Client {
        &self.http_client
    }
}

/// HTTP client for Pump.fun API
#[derive(Debug, Clone)]
pub struct PumpClient {
    pub(crate) http_client: Arc<Client>,
    pub(crate) api_url: String,
}

impl PumpClient {
    /// Create a new Pump client from configuration
    pub fn new(config: &ProvidersConfig) -> Self {
        let api_url = config
            .pump_api_url
            .clone()
            .or_else(|| std::env::var(PUMP_API_URL).ok())
            .unwrap_or_else(|| DEFAULT_PUMP_API_URL.to_string());

        Self {
            http_client: Arc::new(Client::new()),
            api_url,
        }
    }

    /// Create a Pump client with custom URL (for testing)
    pub fn with_url(api_url: String) -> Self {
        Self {
            http_client: Arc::new(Client::new()),
            api_url,
        }
    }

    /// Get the Pump.fun API base URL
    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    /// Get the underlying HTTP client
    pub fn http_client(&self) -> &Client {
        &self.http_client
    }
}

/// Collection of all external API clients
#[derive(Debug, Clone)]
pub struct ApiClients {
    /// Jupiter aggregator client
    pub jupiter: JupiterClient,
    /// Pump.fun API client
    pub pump: PumpClient,
}

impl ApiClients {
    /// Create all API clients from configuration
    pub fn new(config: &ProvidersConfig) -> Self {
        Self {
            jupiter: JupiterClient::new(config),
            pump: PumpClient::new(config),
        }
    }
}
