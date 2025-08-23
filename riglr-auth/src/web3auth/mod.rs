//! Web3Auth authentication provider implementation
//!
//! Provides non-custodial key management with social login capabilities.

use crate::config::ProviderConfig;
use crate::error::AuthError;
use async_trait::async_trait;
use riglr_core::signer::UnifiedSigner;
use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};
use serde::{Deserialize, Serialize};

const WEB3AUTH_CLIENT_ID: &str = "WEB3AUTH_CLIENT_ID";
const WEB3AUTH_VERIFIER: &str = "WEB3AUTH_VERIFIER";
const WEB3AUTH_NETWORK: &str = "WEB3AUTH_NETWORK";
const WEB3AUTH_API_URL: &str = "WEB3AUTH_API_URL";

/// Web3Auth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3AuthConfig {
    /// Web3Auth client ID
    pub client_id: String,

    /// Network (mainnet, testnet, cyan, aqua, celeste)
    #[serde(default = "default_network")]
    pub network: String,

    /// Verifier name
    pub verifier: String,

    /// API base URL
    #[serde(default = "default_api_url")]
    pub api_url: String,
}

impl Web3AuthConfig {
    /// Create a new Web3Auth configuration
    pub fn new(client_id: String, verifier: String) -> Self {
        Self {
            client_id,
            verifier,
            network: default_network(),
            api_url: default_api_url(),
        }
    }
}

impl ProviderConfig for Web3AuthConfig {
    fn validate(&self) -> Result<(), AuthError> {
        if self.client_id.is_empty() {
            return Err(AuthError::ConfigError(
                "Web3Auth client ID required".to_string(),
            ));
        }
        if self.verifier.is_empty() {
            return Err(AuthError::ConfigError(
                "Web3Auth verifier required".to_string(),
            ));
        }
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "web3auth"
    }

    fn from_env() -> Result<Self, AuthError> {
        let client_id = std::env::var(WEB3AUTH_CLIENT_ID)
            .map_err(|_| AuthError::ConfigError("WEB3AUTH_CLIENT_ID not found".to_string()))?;
        let verifier = std::env::var(WEB3AUTH_VERIFIER)
            .map_err(|_| AuthError::ConfigError("WEB3AUTH_VERIFIER not found".to_string()))?;

        let mut config = Self::new(client_id, verifier);

        if let Ok(network) = std::env::var(WEB3AUTH_NETWORK) {
            config.network = network;
        }
        if let Ok(api_url) = std::env::var(WEB3AUTH_API_URL) {
            config.api_url = api_url;
        }

        config.validate()?;
        Ok(config)
    }
}

/// Web3Auth provider implementation
#[allow(dead_code)]
pub struct Web3AuthProvider {
    config: Web3AuthConfig,
    client: reqwest::Client,
}

impl Web3AuthProvider {
    /// Create a new Web3Auth provider
    pub fn new(config: Web3AuthConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl SignerFactory for Web3AuthProvider {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
        let _token = auth_data
            .credentials
            .get("token")
            .ok_or_else(|| AuthError::MissingCredential("token".to_string()))?;

        // TODO: Implement Web3Auth token verification and signer creation
        // This requires:
        // 1. Verifying the Web3Auth ID token
        // 2. Deriving the private key using Web3Auth's key management
        // 3. Creating appropriate signer (Solana or EVM)

        Err(Box::new(AuthError::UnsupportedOperation(
            "Web3Auth provider implementation pending".to_string(),
        )))
    }

    fn supported_auth_types(&self) -> Vec<String> {
        vec!["web3auth".to_string()]
    }
}

fn default_network() -> String {
    "mainnet".to_string()
}

fn default_api_url() -> String {
    "https://api.openlogin.com".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio;

    #[test]
    fn test_web3auth_config_new_when_valid_params_should_create_config() {
        let client_id = "test_client_id".to_string();
        let verifier = "test_verifier".to_string();

        let config = Web3AuthConfig::new(client_id.clone(), verifier.clone());

        assert_eq!(config.client_id, client_id);
        assert_eq!(config.verifier, verifier);
        assert_eq!(config.network, "mainnet");
        assert_eq!(config.api_url, "https://api.openlogin.com");
    }

    #[test]
    fn test_web3auth_config_validate_when_valid_should_return_ok() {
        let config =
            Web3AuthConfig::new("valid_client_id".to_string(), "valid_verifier".to_string());

        let result = config.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_web3auth_config_validate_when_empty_client_id_should_return_err() {
        let config = Web3AuthConfig::new("".to_string(), "valid_verifier".to_string());

        let result = config.validate();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "Web3Auth client ID required");
        } else {
            panic!("Expected ConfigError with client ID message");
        }
    }

    #[test]
    fn test_web3auth_config_validate_when_empty_verifier_should_return_err() {
        let config = Web3AuthConfig::new("valid_client_id".to_string(), "".to_string());

        let result = config.validate();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "Web3Auth verifier required");
        } else {
            panic!("Expected ConfigError with verifier message");
        }
    }

    #[test]
    fn test_web3auth_config_validate_when_both_empty_should_return_client_id_err() {
        let config = Web3AuthConfig::new("".to_string(), "".to_string());

        let result = config.validate();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "Web3Auth client ID required");
        } else {
            panic!("Expected ConfigError with client ID message");
        }
    }

    #[test]
    fn test_web3auth_config_provider_name_should_return_web3auth() {
        let config = Web3AuthConfig::new("test_client_id".to_string(), "test_verifier".to_string());

        assert_eq!(config.provider_name(), "web3auth");
    }

    #[test]
    fn test_web3auth_config_from_env_when_missing_client_id_should_return_err() {
        // Clear environment variables
        std::env::remove_var(WEB3AUTH_CLIENT_ID);
        std::env::remove_var(WEB3AUTH_VERIFIER);
        std::env::remove_var(WEB3AUTH_NETWORK);
        std::env::remove_var(WEB3AUTH_API_URL);

        let result = Web3AuthConfig::from_env();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "WEB3AUTH_CLIENT_ID not found");
        } else {
            panic!("Expected ConfigError with client ID not found message");
        }
    }

    #[test]
    fn test_web3auth_config_from_env_when_missing_verifier_should_return_err() {
        std::env::set_var(WEB3AUTH_CLIENT_ID, "test_client_id");
        std::env::remove_var(WEB3AUTH_VERIFIER);
        std::env::remove_var(WEB3AUTH_NETWORK);
        std::env::remove_var(WEB3AUTH_API_URL);

        let result = Web3AuthConfig::from_env();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "WEB3AUTH_VERIFIER not found");
        } else {
            panic!("Expected ConfigError with verifier not found message");
        }

        // Cleanup
        std::env::remove_var(WEB3AUTH_CLIENT_ID);
    }

    #[test]
    fn test_web3auth_config_from_env_when_required_vars_present_should_create_config() {
        std::env::set_var(WEB3AUTH_CLIENT_ID, "test_client_id");
        std::env::set_var(WEB3AUTH_VERIFIER, "test_verifier");
        std::env::remove_var(WEB3AUTH_NETWORK);
        std::env::remove_var(WEB3AUTH_API_URL);

        let result = Web3AuthConfig::from_env();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.client_id, "test_client_id");
        assert_eq!(config.verifier, "test_verifier");
        assert_eq!(config.network, "mainnet"); // default
        assert_eq!(config.api_url, "https://api.openlogin.com"); // default

        // Cleanup
        std::env::remove_var(WEB3AUTH_CLIENT_ID);
        std::env::remove_var(WEB3AUTH_VERIFIER);
    }

    #[test]
    fn test_web3auth_config_from_env_when_all_vars_present_should_use_custom_values() {
        std::env::set_var(WEB3AUTH_CLIENT_ID, "test_client_id");
        std::env::set_var(WEB3AUTH_VERIFIER, "test_verifier");
        std::env::set_var(WEB3AUTH_NETWORK, "testnet");
        std::env::set_var(WEB3AUTH_API_URL, "https://custom.api.url");

        let result = Web3AuthConfig::from_env();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.client_id, "test_client_id");
        assert_eq!(config.verifier, "test_verifier");
        assert_eq!(config.network, "testnet");
        assert_eq!(config.api_url, "https://custom.api.url");

        // Cleanup
        std::env::remove_var(WEB3AUTH_CLIENT_ID);
        std::env::remove_var(WEB3AUTH_VERIFIER);
        std::env::remove_var(WEB3AUTH_NETWORK);
        std::env::remove_var(WEB3AUTH_API_URL);
    }

    #[test]
    fn test_web3auth_config_from_env_when_empty_client_id_should_fail_validation() {
        std::env::set_var(WEB3AUTH_CLIENT_ID, "");
        std::env::set_var(WEB3AUTH_VERIFIER, "test_verifier");
        std::env::remove_var(WEB3AUTH_NETWORK);
        std::env::remove_var(WEB3AUTH_API_URL);

        let result = Web3AuthConfig::from_env();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "Web3Auth client ID required");
        } else {
            panic!("Expected ConfigError with client ID required message");
        }

        // Cleanup
        std::env::remove_var(WEB3AUTH_CLIENT_ID);
        std::env::remove_var(WEB3AUTH_VERIFIER);
    }

    #[test]
    fn test_web3auth_config_from_env_when_empty_verifier_should_fail_validation() {
        std::env::set_var(WEB3AUTH_CLIENT_ID, "test_client_id");
        std::env::set_var(WEB3AUTH_VERIFIER, "");
        std::env::remove_var(WEB3AUTH_NETWORK);
        std::env::remove_var(WEB3AUTH_API_URL);

        let result = Web3AuthConfig::from_env();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "Web3Auth verifier required");
        } else {
            panic!("Expected ConfigError with verifier required message");
        }

        // Cleanup
        std::env::remove_var(WEB3AUTH_CLIENT_ID);
        std::env::remove_var(WEB3AUTH_VERIFIER);
    }

    #[test]
    fn test_web3auth_provider_new_when_valid_config_should_create_provider() {
        let config = Web3AuthConfig::new("test_client_id".to_string(), "test_verifier".to_string());

        let provider = Web3AuthProvider::new(config.clone());

        assert_eq!(provider.config.client_id, config.client_id);
        assert_eq!(provider.config.verifier, config.verifier);
        assert_eq!(provider.config.network, config.network);
        assert_eq!(provider.config.api_url, config.api_url);
    }

    #[tokio::test]
    async fn test_web3auth_provider_create_signer_when_missing_token_should_return_err() {
        let config = Web3AuthConfig::new("test_client_id".to_string(), "test_verifier".to_string());
        let provider = Web3AuthProvider::new(config);
        let auth_data = AuthenticationData {
            auth_type: "web3auth".to_string(),
            credentials: HashMap::new(),
            network: "mainnet".to_string(),
        };

        let result = provider.create_signer(auth_data).await;

        assert!(result.is_err());
        let error_str = result.unwrap_err().to_string();
        assert!(error_str.contains("Missing credential: token"));
    }

    #[tokio::test]
    async fn test_web3auth_provider_create_signer_when_token_present_should_return_unsupported_err()
    {
        let config = Web3AuthConfig::new("test_client_id".to_string(), "test_verifier".to_string());
        let provider = Web3AuthProvider::new(config);
        let mut credentials = HashMap::new();
        credentials.insert("token".to_string(), "test_token".to_string());
        let auth_data = AuthenticationData {
            auth_type: "web3auth".to_string(),
            credentials,
            network: "mainnet".to_string(),
        };

        let result = provider.create_signer(auth_data).await;

        assert!(result.is_err());
        let error_str = result.unwrap_err().to_string();
        assert!(error_str.contains("Web3Auth provider implementation pending"));
    }

    #[test]
    fn test_web3auth_provider_supported_auth_types_should_return_web3auth() {
        let config = Web3AuthConfig::new("test_client_id".to_string(), "test_verifier".to_string());
        let provider = Web3AuthProvider::new(config);

        let auth_types = provider.supported_auth_types();

        assert_eq!(auth_types, vec!["web3auth".to_string()]);
    }

    #[test]
    fn test_default_network_should_return_mainnet() {
        assert_eq!(default_network(), "mainnet");
    }

    #[test]
    fn test_default_api_url_should_return_openlogin_url() {
        assert_eq!(default_api_url(), "https://api.openlogin.com");
    }
}
