//! API clients for external services used by Solana tools
//!
//! This module provides pre-configured HTTP clients for external APIs like Jupiter and Pump.fun,
//! eliminating direct environment variable access from tools and enabling dependency injection.

extern crate alloc;

use alloc::sync::Arc;
use reqwest::Client;
use riglr_config::ProvidersConfig;
use std::env;

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
    /// API URL for Jupiter service
    api_url: String,
    /// HTTP client for making requests
    http_client: Arc<Client>,
}

impl JupiterClient {
    /// Get the Jupiter API base URL
    #[must_use]
    #[inline]
    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    /// Get the underlying HTTP client
    #[must_use]
    #[inline]
    pub fn http_client(&self) -> &Client {
        &self.http_client
    }

    /// Create a new Jupiter client from configuration
    #[must_use]
    #[inline]
    pub fn new(config: &ProvidersConfig) -> Self {
        let api_url = config
            .jupiter_api_url
            .clone()
            .or_else(|| env::var(JUPITER_API_URL).ok())
            .unwrap_or_else(|| DEFAULT_JUPITER_API_URL.to_owned());

        Self {
            api_url,
            http_client: Arc::new(Client::new()),
        }
    }

    /// Create a Jupiter client with custom URL (for testing)
    #[must_use]
    #[inline]
    pub fn with_url(api_url: String) -> Self {
        Self {
            api_url,
            http_client: Arc::new(Client::new()),
        }
    }
}

/// HTTP client for Pump.fun API
#[derive(Debug, Clone)]
pub struct PumpClient {
    /// API URL for Pump.fun service
    api_url: String,
    /// HTTP client for making requests
    http_client: Arc<Client>,
}

impl PumpClient {
    /// Get the Pump.fun API base URL
    #[must_use]
    #[inline]
    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    /// Get the underlying HTTP client
    #[must_use]
    #[inline]
    pub fn http_client(&self) -> &Client {
        &self.http_client
    }

    /// Create a new Pump client from configuration
    #[must_use]
    #[inline]
    pub fn new(config: &ProvidersConfig) -> Self {
        let api_url = config
            .pump_api_url
            .clone()
            .or_else(|| env::var(PUMP_API_URL).ok())
            .unwrap_or_else(|| DEFAULT_PUMP_API_URL.to_owned());

        Self {
            api_url,
            http_client: Arc::new(Client::new()),
        }
    }

    /// Create a Pump client with custom URL (for testing)
    #[must_use]
    #[inline]
    pub fn with_url(api_url: String) -> Self {
        Self {
            api_url,
            http_client: Arc::new(Client::new()),
        }
    }
}

/// Collection of all external API clients
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Clients {
    /// Jupiter aggregator client
    pub jupiter: JupiterClient,
    /// Pump.fun API client
    pub pump: PumpClient,
}

impl Clients {
    /// Create all API clients from configuration
    #[must_use]
    #[inline]
    pub fn new(config: &ProvidersConfig) -> Self {
        Self {
            jupiter: JupiterClient::new(config),
            pump: PumpClient::new(config),
        }
    }
}
