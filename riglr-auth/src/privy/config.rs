//! Privy-specific configuration

use crate::config::ProviderConfig;
use crate::error::AuthError;
use serde::{Deserialize, Serialize};

pub const PRIVY_APP_ID: &str = "PRIVY_APP_ID";
pub const PRIVY_APP_SECRET: &str = "PRIVY_APP_SECRET";
pub const PRIVY_API_URL: &str = "PRIVY_API_URL";
pub const PRIVY_AUTH_URL: &str = "PRIVY_AUTH_URL";
pub const PRIVY_JWKS_URL: &str = "PRIVY_JWKS_URL";
pub const PRIVY_ENABLE_CACHE: &str = "PRIVY_ENABLE_CACHE";

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
        let app_id = std::env::var(PRIVY_APP_ID)
            .map_err(|_| AuthError::ConfigError("PRIVY_APP_ID not found".to_string()))?;
        let app_secret = std::env::var(PRIVY_APP_SECRET)
            .map_err(|_| AuthError::ConfigError("PRIVY_APP_SECRET not found".to_string()))?;

        let mut config = Self::new(app_id, app_secret);

        // Optional overrides from environment
        if let Ok(api_url) = std::env::var(PRIVY_API_URL) {
            config.api_url = api_url;
        }
        if let Ok(auth_url) = std::env::var(PRIVY_AUTH_URL) {
            config.auth_url = auth_url;
        }
        if let Ok(jwks_url) = std::env::var(PRIVY_JWKS_URL) {
            config.jwks_url = jwks_url;
        }
        if let Ok(cache) = std::env::var(PRIVY_ENABLE_CACHE) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_api_url_should_return_correct_url() {
        assert_eq!(default_api_url(), "https://api.privy.io");
    }

    #[test]
    fn test_default_auth_url_should_return_correct_url() {
        assert_eq!(default_auth_url(), "https://auth.privy.io");
    }

    #[test]
    fn test_default_jwks_url_should_return_correct_url() {
        assert_eq!(
            default_jwks_url(),
            "https://auth.privy.io/.well-known/jwks.json"
        );
    }

    #[test]
    fn test_default_cache_enabled_should_return_true() {
        assert!(default_cache_enabled());
    }

    #[test]
    fn test_default_cache_ttl_should_return_300() {
        assert_eq!(default_cache_ttl(), 300);
    }

    #[test]
    fn test_privy_config_new_should_create_config_with_defaults() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string());

        assert_eq!(config.app_id, "test_app_id");
        assert_eq!(config.app_secret, "test_secret");
        assert_eq!(config.api_url, "https://api.privy.io");
        assert_eq!(config.auth_url, "https://auth.privy.io");
        assert_eq!(
            config.jwks_url,
            "https://auth.privy.io/.well-known/jwks.json"
        );
        assert!(config.enable_cache);
        assert_eq!(config.cache_ttl_seconds, 300);
    }

    #[test]
    fn test_privy_config_with_api_url_should_override_default() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string())
            .with_api_url("https://custom.api.url".to_string());

        assert_eq!(config.api_url, "https://custom.api.url");
        // Other fields should remain default
        assert_eq!(config.auth_url, "https://auth.privy.io");
    }

    #[test]
    fn test_privy_config_with_auth_url_should_override_default() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string())
            .with_auth_url("https://custom.auth.url".to_string());

        assert_eq!(config.auth_url, "https://custom.auth.url");
        // Other fields should remain default
        assert_eq!(config.api_url, "https://api.privy.io");
    }

    #[test]
    fn test_privy_config_with_cache_true_should_enable_cache() {
        let config =
            PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string()).with_cache(true);

        assert!(config.enable_cache);
    }

    #[test]
    fn test_privy_config_with_cache_false_should_disable_cache() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string())
            .with_cache(false);

        assert!(!config.enable_cache);
    }

    #[test]
    fn test_privy_config_chaining_methods_should_work() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string())
            .with_api_url("https://custom.api.url".to_string())
            .with_auth_url("https://custom.auth.url".to_string())
            .with_cache(false);

        assert_eq!(config.api_url, "https://custom.api.url");
        assert_eq!(config.auth_url, "https://custom.auth.url");
        assert!(!config.enable_cache);
    }

    #[test]
    fn test_validate_when_valid_config_should_return_ok() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_when_empty_app_id_should_return_err() {
        let config = PrivyConfig::new("".to_string(), "test_secret".to_string());
        let result = config.validate();

        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::ConfigError(msg) => assert_eq!(msg, "Privy app ID is required"),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_validate_when_empty_app_secret_should_return_err() {
        let config = PrivyConfig::new("test_app_id".to_string(), "".to_string());
        let result = config.validate();

        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::ConfigError(msg) => assert_eq!(msg, "Privy app secret is required"),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_provider_name_should_return_privy() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string());
        assert_eq!(config.provider_name(), "privy");
    }

    #[test]
    fn test_from_env_when_required_vars_present_should_return_ok() {
        // Set required environment variables
        env::set_var(PRIVY_APP_ID, "test_app_id");
        env::set_var(PRIVY_APP_SECRET, "test_secret");

        let result = PrivyConfig::from_env();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.app_id, "test_app_id");
        assert_eq!(config.app_secret, "test_secret");
        assert_eq!(config.api_url, "https://api.privy.io");
        assert_eq!(config.auth_url, "https://auth.privy.io");
        assert_eq!(
            config.jwks_url,
            "https://auth.privy.io/.well-known/jwks.json"
        );
        assert!(config.enable_cache);

        // Clean up
        env::remove_var(PRIVY_APP_ID);
        env::remove_var(PRIVY_APP_SECRET);
    }

    #[test]
    fn test_from_env_when_app_id_missing_should_return_err() {
        env::remove_var(PRIVY_APP_ID);
        env::remove_var(PRIVY_APP_SECRET);

        let result = PrivyConfig::from_env();
        assert!(result.is_err());

        match result.unwrap_err() {
            AuthError::ConfigError(msg) => assert_eq!(msg, "PRIVY_APP_ID not found"),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_from_env_when_app_secret_missing_should_return_err() {
        env::set_var(PRIVY_APP_ID, "test_app_id");
        env::remove_var(PRIVY_APP_SECRET);

        let result = PrivyConfig::from_env();
        assert!(result.is_err());

        match result.unwrap_err() {
            AuthError::ConfigError(msg) => assert_eq!(msg, "PRIVY_APP_SECRET not found"),
            _ => panic!("Expected ConfigError"),
        }

        // Clean up
        env::remove_var(PRIVY_APP_ID);
    }

    #[test]
    fn test_from_env_with_custom_api_url_should_override_default() {
        env::set_var(PRIVY_APP_ID, "test_app_id");
        env::set_var(PRIVY_APP_SECRET, "test_secret");
        env::set_var(PRIVY_API_URL, "https://custom.api.url");

        let result = PrivyConfig::from_env();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.api_url, "https://custom.api.url");

        // Clean up
        env::remove_var(PRIVY_APP_ID);
        env::remove_var(PRIVY_APP_SECRET);
        env::remove_var(PRIVY_API_URL);
    }

    #[test]
    fn test_from_env_with_custom_auth_url_should_override_default() {
        env::set_var(PRIVY_APP_ID, "test_app_id");
        env::set_var(PRIVY_APP_SECRET, "test_secret");
        env::set_var(PRIVY_AUTH_URL, "https://custom.auth.url");

        let result = PrivyConfig::from_env();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.auth_url, "https://custom.auth.url");

        // Clean up
        env::remove_var(PRIVY_APP_ID);
        env::remove_var(PRIVY_APP_SECRET);
        env::remove_var(PRIVY_AUTH_URL);
    }

    #[test]
    fn test_from_env_with_custom_jwks_url_should_override_default() {
        env::set_var(PRIVY_APP_ID, "test_app_id");
        env::set_var(PRIVY_APP_SECRET, "test_secret");
        env::set_var(PRIVY_JWKS_URL, "https://custom.jwks.url");

        let result = PrivyConfig::from_env();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.jwks_url, "https://custom.jwks.url");

        // Clean up
        env::remove_var(PRIVY_APP_ID);
        env::remove_var(PRIVY_APP_SECRET);
        env::remove_var(PRIVY_JWKS_URL);
    }

    #[test]
    fn test_from_env_with_cache_true_should_enable_cache() {
        env::set_var(PRIVY_APP_ID, "test_app_id");
        env::set_var(PRIVY_APP_SECRET, "test_secret");
        env::set_var(PRIVY_ENABLE_CACHE, "true");

        let result = PrivyConfig::from_env();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.enable_cache);

        // Clean up
        env::remove_var(PRIVY_APP_ID);
        env::remove_var(PRIVY_APP_SECRET);
        env::remove_var(PRIVY_ENABLE_CACHE);
    }

    #[test]
    fn test_from_env_with_cache_false_should_disable_cache() {
        env::set_var(PRIVY_APP_ID, "test_app_id");
        env::set_var(PRIVY_APP_SECRET, "test_secret");
        env::set_var(PRIVY_ENABLE_CACHE, "false");

        let result = PrivyConfig::from_env();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(!config.enable_cache);

        // Clean up
        env::remove_var(PRIVY_APP_ID);
        env::remove_var(PRIVY_APP_SECRET);
        env::remove_var(PRIVY_ENABLE_CACHE);
    }

    #[test]
    fn test_from_env_with_invalid_cache_value_should_default_to_true() {
        env::set_var(PRIVY_APP_ID, "test_app_id");
        env::set_var(PRIVY_APP_SECRET, "test_secret");
        env::set_var(PRIVY_ENABLE_CACHE, "invalid");

        let result = PrivyConfig::from_env();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.enable_cache); // Should default to true on parse error

        // Clean up
        env::remove_var(PRIVY_APP_ID);
        env::remove_var(PRIVY_APP_SECRET);
        env::remove_var(PRIVY_ENABLE_CACHE);
    }

    #[test]
    fn test_from_env_with_all_optional_overrides_should_use_all_custom_values() {
        env::set_var(PRIVY_APP_ID, "test_app_id");
        env::set_var(PRIVY_APP_SECRET, "test_secret");
        env::set_var(PRIVY_API_URL, "https://custom.api.url");
        env::set_var(PRIVY_AUTH_URL, "https://custom.auth.url");
        env::set_var(PRIVY_JWKS_URL, "https://custom.jwks.url");
        env::set_var(PRIVY_ENABLE_CACHE, "false");

        let result = PrivyConfig::from_env();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.app_id, "test_app_id");
        assert_eq!(config.app_secret, "test_secret");
        assert_eq!(config.api_url, "https://custom.api.url");
        assert_eq!(config.auth_url, "https://custom.auth.url");
        assert_eq!(config.jwks_url, "https://custom.jwks.url");
        assert!(!config.enable_cache);

        // Clean up
        env::remove_var(PRIVY_APP_ID);
        env::remove_var(PRIVY_APP_SECRET);
        env::remove_var(PRIVY_API_URL);
        env::remove_var(PRIVY_AUTH_URL);
        env::remove_var(PRIVY_JWKS_URL);
        env::remove_var(PRIVY_ENABLE_CACHE);
    }

    #[test]
    fn test_from_env_when_validation_fails_should_return_err() {
        env::set_var(PRIVY_APP_ID, ""); // Empty app_id should fail validation
        env::set_var(PRIVY_APP_SECRET, "test_secret");

        let result = PrivyConfig::from_env();
        assert!(result.is_err());

        match result.unwrap_err() {
            AuthError::ConfigError(msg) => assert_eq!(msg, "Privy app ID is required"),
            _ => panic!("Expected ConfigError"),
        }

        // Clean up
        env::remove_var(PRIVY_APP_ID);
        env::remove_var(PRIVY_APP_SECRET);
    }

    #[test]
    fn test_serde_serialization_should_work() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string());
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_serde_deserialization_should_work() {
        let json = r#"{
            "app_id": "test_app_id",
            "app_secret": "test_secret"
        }"#;

        let config: Result<PrivyConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.app_id, "test_app_id");
        assert_eq!(config.app_secret, "test_secret");
        // Should use defaults for missing fields
        assert_eq!(config.api_url, "https://api.privy.io");
        assert_eq!(config.auth_url, "https://auth.privy.io");
        assert_eq!(
            config.jwks_url,
            "https://auth.privy.io/.well-known/jwks.json"
        );
        assert!(config.enable_cache);
        assert_eq!(config.cache_ttl_seconds, 300);
    }

    #[test]
    fn test_debug_trait_should_work() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string());
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("test_app_id"));
        assert!(debug_str.contains("test_secret"));
    }

    #[test]
    fn test_clone_trait_should_work() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string());
        let cloned = config.clone();

        assert_eq!(config.app_id, cloned.app_id);
        assert_eq!(config.app_secret, cloned.app_secret);
        assert_eq!(config.api_url, cloned.api_url);
        assert_eq!(config.auth_url, cloned.auth_url);
        assert_eq!(config.jwks_url, cloned.jwks_url);
        assert_eq!(config.enable_cache, cloned.enable_cache);
        assert_eq!(config.cache_ttl_seconds, cloned.cache_ttl_seconds);
    }
}
