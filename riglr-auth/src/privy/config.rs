//! Privy-specific configuration

use crate::config::ProviderConfig;
use crate::error::AuthError;
use serde::{Deserialize, Serialize};

/// Privy authentication provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyConfig {
    /// Privy application ID
    pub app_id: String,

    /// Privy application secret
    pub app_secret: String,

    /// Privy API base URL (defaults to <https://api.privy.io>)
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// Privy auth URL (defaults to <https://auth.privy.io>)
    #[serde(default = "default_auth_url")]
    pub auth_url: String,

    /// JWKS endpoint for token verification
    #[serde(default = "default_jwks_url")]
    pub jwks_url: String,

    /// Enable token caching
    #[serde(default = "default_cache_enabled")]
    pub enable_cache: bool,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

impl PrivyConfig {
    /// Create a new Privy configuration
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            app_id,
            app_secret,
            api_url: default_api_url(),
            auth_url: default_auth_url(),
            jwks_url: default_jwks_url(),
            enable_cache: default_cache_enabled(),
            cache_ttl_seconds: default_cache_ttl(),
        }
    }

    /// Create configuration with custom API URL
    pub fn with_api_url(mut self, url: String) -> Self {
        self.api_url = url;
        self
    }

    /// Create configuration with custom auth URL
    pub fn with_auth_url(mut self, url: String) -> Self {
        self.auth_url = url;
        self
    }

    /// Enable or disable caching
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.enable_cache = enabled;
        self
    }
}

impl ProviderConfig for PrivyConfig {
    fn validate(&self) -> Result<(), AuthError> {
        if self.app_id.is_empty() {
            return Err(AuthError::ConfigError(
                "Privy app ID is required".to_string(),
            ));
        }
        if self.app_secret.is_empty() {
            return Err(AuthError::ConfigError(
                "Privy app secret is required".to_string(),
            ));
        }
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "privy"
    }

    fn from_env() -> Result<Self, AuthError> {
        let app_id = std::env::var("PRIVY_APP_ID")
            .map_err(|_| AuthError::ConfigError("PRIVY_APP_ID not found".to_string()))?;
        let app_secret = std::env::var("PRIVY_APP_SECRET")
            .map_err(|_| AuthError::ConfigError("PRIVY_APP_SECRET not found".to_string()))?;

        let mut config = Self::new(app_id, app_secret);

        // Optional overrides from environment
        if let Ok(api_url) = std::env::var("PRIVY_API_URL") {
            config.api_url = api_url;
        }
        if let Ok(auth_url) = std::env::var("PRIVY_AUTH_URL") {
            config.auth_url = auth_url;
        }
        if let Ok(jwks_url) = std::env::var("PRIVY_JWKS_URL") {
            config.jwks_url = jwks_url;
        }
        if let Ok(cache) = std::env::var("PRIVY_ENABLE_CACHE") {
            config.enable_cache = cache.parse().unwrap_or(true);
        }

        config.validate()?;
        Ok(config)
    }
}

fn default_api_url() -> String {
    "https://api.privy.io".to_string()
}

fn default_auth_url() -> String {
    "https://auth.privy.io".to_string()
}

fn default_jwks_url() -> String {
    "https://auth.privy.io/.well-known/jwks.json".to_string()
}

fn default_cache_enabled() -> bool {
    true
}

fn default_cache_ttl() -> u64 {
    300 // 5 minutes
}
