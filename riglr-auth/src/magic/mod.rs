//! Magic.link authentication provider implementation
//!
//! Provides email-based authentication with embedded wallets.

use crate::config::ProviderConfig;
use crate::error::AuthError;
use async_trait::async_trait;
use riglr_core::signer::UnifiedSigner;
use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};
use serde::{Deserialize, Serialize};

const MAGIC_PUBLISHABLE_KEY: &str = "MAGIC_PUBLISHABLE_KEY";
const MAGIC_SECRET_KEY: &str = "MAGIC_SECRET_KEY";
const MAGIC_API_URL: &str = "MAGIC_API_URL";
const MAGIC_NETWORK: &str = "MAGIC_NETWORK";

/// Magic.link configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicConfig {
    /// Magic publishable API key
    pub publishable_key: String,

    /// Magic secret key
    pub secret_key: String,

    /// API base URL
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// Network (mainnet or testnet)
    #[serde(default = "default_network")]
    pub network: String,
}

impl MagicConfig {
    /// Create a new Magic configuration
    pub fn new(publishable_key: String, secret_key: String) -> Self {
        Self {
            publishable_key,
            secret_key,
            api_url: default_api_url(),
            network: default_network(),
        }
    }
}

impl ProviderConfig for MagicConfig {
    fn validate(&self) -> Result<(), AuthError> {
        if self.publishable_key.is_empty() {
            return Err(AuthError::ConfigError(
                "Magic publishable key required".to_string(),
            ));
        }
        if self.secret_key.is_empty() {
            return Err(AuthError::ConfigError(
                "Magic secret key required".to_string(),
            ));
        }
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "magic"
    }

    fn from_env() -> Result<Self, AuthError> {
        let publishable_key = std::env::var(MAGIC_PUBLISHABLE_KEY)
            .map_err(|_| AuthError::ConfigError("MAGIC_PUBLISHABLE_KEY not found".to_string()))?;
        let secret_key = std::env::var(MAGIC_SECRET_KEY)
            .map_err(|_| AuthError::ConfigError("MAGIC_SECRET_KEY not found".to_string()))?;

        let mut config = Self::new(publishable_key, secret_key);

        if let Ok(api_url) = std::env::var(MAGIC_API_URL) {
            config.api_url = api_url;
        }
        if let Ok(network) = std::env::var(MAGIC_NETWORK) {
            config.network = network;
        }

        config.validate()?;
        Ok(config)
    }
}

/// Magic.link provider implementation
#[allow(dead_code)]
pub struct MagicProvider {
    config: MagicConfig,
    client: reqwest::Client,
}

impl MagicProvider {
    /// Create a new Magic provider
    pub fn new(config: MagicConfig) -> Self {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("X-Magic-Secret-Key", config.secret_key.parse().unwrap());
                headers
            })
            .build()
            .expect("Failed to build Magic HTTP client");

        Self { config, client }
    }
}

#[async_trait]
impl SignerFactory for MagicProvider {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
        let _token = auth_data
            .credentials
            .get("token")
            .ok_or_else(|| AuthError::MissingCredential("token".to_string()))?;

        // TODO: Implement Magic.link token verification and signer creation
        // This requires:
        // 1. Verifying the DID token with Magic Admin SDK
        // 2. Getting user metadata and wallet addresses
        // 3. Creating delegated signers for transactions

        Err(Box::new(AuthError::UnsupportedOperation(
            "Magic.link provider implementation pending".to_string(),
        )))
    }

    fn supported_auth_types(&self) -> Vec<String> {
        vec!["magic".to_string()]
    }
}

fn default_api_url() -> String {
    "https://api.magic.link".to_string()
}

fn default_network() -> String {
    "mainnet".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_magic_config_new_when_valid_inputs_should_create_config() {
        let publishable_key = "pk_test_123".to_string();
        let secret_key = "sk_test_456".to_string();

        let config = MagicConfig::new(publishable_key.clone(), secret_key.clone());

        assert_eq!(config.publishable_key, publishable_key);
        assert_eq!(config.secret_key, secret_key);
        assert_eq!(config.api_url, "https://api.magic.link");
        assert_eq!(config.network, "mainnet");
    }

    #[test]
    fn test_magic_config_new_when_empty_strings_should_create_config() {
        let config = MagicConfig::new("".to_string(), "".to_string());

        assert_eq!(config.publishable_key, "");
        assert_eq!(config.secret_key, "");
        assert_eq!(config.api_url, "https://api.magic.link");
        assert_eq!(config.network, "mainnet");
    }

    #[test]
    fn test_magic_config_validate_when_valid_should_return_ok() {
        let config = MagicConfig::new("pk_test_123".to_string(), "sk_test_456".to_string());

        let result = config.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_magic_config_validate_when_empty_publishable_key_should_return_err() {
        let config = MagicConfig::new("".to_string(), "sk_test_456".to_string());

        let result = config.validate();

        assert!(result.is_err());
        match result {
            Err(AuthError::ConfigError(msg)) => {
                assert_eq!(msg, "Magic publishable key required");
            }
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_magic_config_validate_when_empty_secret_key_should_return_err() {
        let config = MagicConfig::new("pk_test_123".to_string(), "".to_string());

        let result = config.validate();

        assert!(result.is_err());
        match result {
            Err(AuthError::ConfigError(msg)) => {
                assert_eq!(msg, "Magic secret key required");
            }
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_magic_config_validate_when_both_keys_empty_should_return_publishable_key_error() {
        let config = MagicConfig::new("".to_string(), "".to_string());

        let result = config.validate();

        assert!(result.is_err());
        match result {
            Err(AuthError::ConfigError(msg)) => {
                assert_eq!(msg, "Magic publishable key required");
            }
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_magic_config_provider_name_should_return_magic() {
        let config = MagicConfig::new("pk_test_123".to_string(), "sk_test_456".to_string());

        let name = config.provider_name();

        assert_eq!(name, "magic");
    }

    #[test]
    fn test_magic_config_from_env_when_missing_publishable_key_should_return_err() {
        std::env::remove_var(MAGIC_PUBLISHABLE_KEY);
        std::env::remove_var(MAGIC_SECRET_KEY);

        let result = MagicConfig::from_env();

        assert!(result.is_err());
        match result {
            Err(AuthError::ConfigError(msg)) => {
                assert_eq!(msg, "MAGIC_PUBLISHABLE_KEY not found");
            }
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_magic_config_from_env_when_missing_secret_key_should_return_err() {
        std::env::set_var(MAGIC_PUBLISHABLE_KEY, "pk_test_123");
        std::env::remove_var(MAGIC_SECRET_KEY);

        let result = MagicConfig::from_env();

        assert!(result.is_err());
        match result {
            Err(AuthError::ConfigError(msg)) => {
                assert_eq!(msg, "MAGIC_SECRET_KEY not found");
            }
            _ => panic!("Expected ConfigError"),
        }

        std::env::remove_var(MAGIC_PUBLISHABLE_KEY);
    }

    #[test]
    fn test_magic_config_from_env_when_basic_vars_present_should_return_ok() {
        std::env::set_var(MAGIC_PUBLISHABLE_KEY, "pk_test_123");
        std::env::set_var(MAGIC_SECRET_KEY, "sk_test_456");
        std::env::remove_var(MAGIC_API_URL);
        std::env::remove_var(MAGIC_NETWORK);

        let result = MagicConfig::from_env();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.publishable_key, "pk_test_123");
        assert_eq!(config.secret_key, "sk_test_456");
        assert_eq!(config.api_url, "https://api.magic.link");
        assert_eq!(config.network, "mainnet");

        std::env::remove_var(MAGIC_PUBLISHABLE_KEY);
        std::env::remove_var(MAGIC_SECRET_KEY);
    }

    #[test]
    fn test_magic_config_from_env_when_all_vars_present_should_return_ok() {
        std::env::set_var(MAGIC_PUBLISHABLE_KEY, "pk_test_123");
        std::env::set_var(MAGIC_SECRET_KEY, "sk_test_456");
        std::env::set_var(MAGIC_API_URL, "https://custom.api.url");
        std::env::set_var(MAGIC_NETWORK, "testnet");

        let result = MagicConfig::from_env();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.publishable_key, "pk_test_123");
        assert_eq!(config.secret_key, "sk_test_456");
        assert_eq!(config.api_url, "https://custom.api.url");
        assert_eq!(config.network, "testnet");

        std::env::remove_var(MAGIC_PUBLISHABLE_KEY);
        std::env::remove_var(MAGIC_SECRET_KEY);
        std::env::remove_var(MAGIC_API_URL);
        std::env::remove_var(MAGIC_NETWORK);
    }

    #[test]
    fn test_magic_config_from_env_when_validation_fails_should_return_err() {
        std::env::set_var(MAGIC_PUBLISHABLE_KEY, "");
        std::env::set_var(MAGIC_SECRET_KEY, "sk_test_456");

        let result = MagicConfig::from_env();

        assert!(result.is_err());
        match result {
            Err(AuthError::ConfigError(msg)) => {
                assert_eq!(msg, "Magic publishable key required");
            }
            _ => panic!("Expected ConfigError"),
        }

        std::env::remove_var(MAGIC_PUBLISHABLE_KEY);
        std::env::remove_var(MAGIC_SECRET_KEY);
    }

    #[test]
    fn test_magic_provider_new_when_valid_config_should_create_provider() {
        let config = MagicConfig::new("pk_test_123".to_string(), "sk_test_456".to_string());

        let provider = MagicProvider::new(config.clone());

        assert_eq!(provider.config.publishable_key, config.publishable_key);
        assert_eq!(provider.config.secret_key, config.secret_key);
    }

    #[test]
    fn test_magic_provider_new_when_empty_config_should_create_provider() {
        let config = MagicConfig::new("".to_string(), "".to_string());

        let provider = MagicProvider::new(config.clone());

        assert_eq!(provider.config.publishable_key, "");
        assert_eq!(provider.config.secret_key, "");
    }

    #[tokio::test]
    async fn test_magic_provider_create_signer_when_missing_token_should_return_err() {
        let config = MagicConfig::new("pk_test_123".to_string(), "sk_test_456".to_string());
        let provider = MagicProvider::new(config);

        let auth_data = AuthenticationData {
            auth_type: "magic".to_string(),
            credentials: HashMap::new(),
            network: "mainnet".to_string(),
        };

        let result = provider.create_signer(auth_data).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        let auth_error = error.downcast::<AuthError>().unwrap();
        match *auth_error {
            AuthError::MissingCredential(ref field) => {
                assert_eq!(field, "token");
            }
            _ => panic!("Expected MissingCredential error"),
        }
    }

    #[tokio::test]
    async fn test_magic_provider_create_signer_when_token_present_should_return_unsupported_err() {
        let config = MagicConfig::new("pk_test_123".to_string(), "sk_test_456".to_string());
        let provider = MagicProvider::new(config);

        let mut credentials = HashMap::new();
        credentials.insert("token".to_string(), "test_token".to_string());
        let auth_data = AuthenticationData {
            auth_type: "magic".to_string(),
            credentials,
            network: "mainnet".to_string(),
        };

        let result = provider.create_signer(auth_data).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        let auth_error = error.downcast::<AuthError>().unwrap();
        match *auth_error {
            AuthError::UnsupportedOperation(ref msg) => {
                assert_eq!(msg, "Magic.link provider implementation pending");
            }
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }

    #[test]
    fn test_magic_provider_supported_auth_types_should_return_magic() {
        let config = MagicConfig::new("pk_test_123".to_string(), "sk_test_456".to_string());
        let provider = MagicProvider::new(config);

        let auth_types = provider.supported_auth_types();

        assert_eq!(auth_types, vec!["magic".to_string()]);
    }

    #[test]
    fn test_default_api_url_should_return_magic_api_url() {
        let url = default_api_url();

        assert_eq!(url, "https://api.magic.link");
    }

    #[test]
    fn test_default_network_should_return_mainnet() {
        let network = default_network();

        assert_eq!(network, "mainnet");
    }
}
