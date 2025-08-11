//! Magic.link authentication provider implementation
//!
//! Provides email-based authentication with embedded wallets.

use crate::config::ProviderConfig;
use crate::error::AuthError;
use async_trait::async_trait;
use riglr_core::config::RpcConfig;
use riglr_core::signer::TransactionSigner;
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
        _config: &RpcConfig,
    ) -> Result<Box<dyn TransactionSigner>, Box<dyn std::error::Error + Send + Sync>> {
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
