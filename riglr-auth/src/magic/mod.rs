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
                headers.insert("Content-Type", "application/json".parse().unwrap());
                headers
            })
            .build()
            .expect("Failed to build Magic HTTP client");

        Self { config, client }
    }

    /// Decode and validate a Magic DID token
    async fn validate_did_token(&self, did_token: &str) -> Result<DIDClaim, AuthError> {
        // Decode the base64-encoded DID token
        use base64::{engine::general_purpose::STANDARD, Engine};
        let decoded_bytes = STANDARD
            .decode(did_token)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid base64 DID token: {}", e)))?;

        let token_str = String::from_utf8(decoded_bytes).map_err(|e| {
            AuthError::TokenValidation(format!("Invalid UTF-8 in DID token: {}", e))
        })?;

        // Parse the [proof, claim] tuple
        let token_tuple: serde_json::Value = serde_json::from_str(&token_str)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid JSON in DID token: {}", e)))?;

        let token_array = token_tuple
            .as_array()
            .ok_or_else(|| AuthError::TokenValidation("DID token must be an array".to_string()))?;

        if token_array.len() != 2 {
            return Err(AuthError::TokenValidation(
                "DID token must have exactly 2 elements".to_string(),
            ));
        }

        let _proof = token_array[0].as_str().ok_or_else(|| {
            AuthError::TokenValidation("DID token proof must be a string".to_string())
        })?;

        let claim_str = token_array[1].as_str().ok_or_else(|| {
            AuthError::TokenValidation("DID token claim must be a string".to_string())
        })?;

        // Parse the claim
        let claim: DIDClaim = serde_json::from_str(claim_str)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid claim format: {}", e)))?;

        // Validate token timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        if claim.iat > now {
            return Err(AuthError::TokenValidation(
                "Token issued in the future".to_string(),
            ));
        }

        if claim.ext < now {
            return Err(AuthError::TokenValidation("Token has expired".to_string()));
        }

        // Validate JWT claims
        if claim.aud != self.config.publishable_key {
            return Err(AuthError::TokenValidation(format!(
                "Token audience mismatch. Expected: {}, Got: {}",
                self.config.publishable_key, claim.aud
            )));
        }

        if claim.iss.is_empty() {
            return Err(AuthError::TokenValidation(
                "Token issuer (user public key) cannot be empty".to_string(),
            ));
        }

        if claim.sub.is_empty() {
            return Err(AuthError::TokenValidation(
                "Token subject (user ID) cannot be empty".to_string(),
            ));
        }

        if claim.tid.is_empty() {
            return Err(AuthError::TokenValidation(
                "Token ID cannot be empty".to_string(),
            ));
        }

        // Validate via Magic's API
        self.validate_token_with_api(did_token).await?;

        Ok(claim)
    }

    /// Validate token using Magic's API
    async fn validate_token_with_api(&self, did_token: &str) -> Result<(), AuthError> {
        let request = MagicRPCRequest {
            jsonrpc: "2.0".to_string(),
            method: "magic_token_validate".to_string(),
            params: serde_json::json!([did_token]),
            id: 1,
        };

        let response = self
            .client
            .post(format!(
                "{}/v1/admin/auth/token/validate",
                self.config.api_url
            ))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AuthError::ApiError(format!("Failed to validate token with Magic API: {}", e))
            })?;

        let rpc_response: MagicRPCResponse<bool> = response
            .json()
            .await
            .map_err(|e| AuthError::ApiError(format!("Invalid response from Magic API: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(AuthError::TokenValidation(format!(
                "Magic API validation failed (code {}): {}",
                error.code, error.message
            )));
        }

        if rpc_response.result != Some(true) {
            return Err(AuthError::TokenValidation(
                "Token validation failed".to_string(),
            ));
        }

        Ok(())
    }

    /// Get user metadata from Magic
    async fn get_user_metadata(&self, did_token: &str) -> Result<MagicUserData, AuthError> {
        let request = MagicRPCRequest {
            jsonrpc: "2.0".to_string(),
            method: "magic_token_get_public_address".to_string(),
            params: serde_json::json!([did_token]),
            id: 2,
        };

        let response = self
            .client
            .post(format!("{}/v1/admin/auth/user/get", self.config.api_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| AuthError::ApiError(format!("Failed to fetch user metadata: {}", e)))?;

        let rpc_response: MagicRPCResponse<MagicUserResponse> = response
            .json()
            .await
            .map_err(|e| AuthError::ApiError(format!("Invalid response from Magic API: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(AuthError::ApiError(format!(
                "Magic API error (code {}): {}",
                error.code, error.message
            )));
        }

        let user_response = rpc_response.result.ok_or_else(|| {
            AuthError::ApiError("No user data returned from Magic API".to_string())
        })?;

        Ok(user_response.data)
    }

    /// Create a signer from Magic user data
    async fn create_signer_from_user(
        &self,
        user_data: &MagicUserData,
        network: &str,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
        // Magic.link manages the private keys, so we can't create a traditional LocalSigner
        // Instead, we would need to implement a delegated signer that makes API calls to Magic
        // for signing operations. For this implementation, we'll return an error explaining
        // the architectural limitation.

        let wallet_address = user_data
            .public_address
            .as_ref()
            .ok_or_else(|| AuthError::NoWallet("No wallet address found for user".to_string()))?;

        // For a real implementation, you would create a Magic-specific signer that:
        // 1. Stores the user's public address and Magic metadata
        // 2. Implements UnifiedSigner trait methods
        // 3. Makes API calls to Magic for signing operations
        // 4. Handles transaction submission through Magic's infrastructure

        tracing::info!(
            "Magic user authenticated - address: {}, email: {:?}, wallet_type: {:?} (network: {})",
            wallet_address,
            user_data.email,
            user_data.wallet_type,
            network
        );

        // Return an informative error for now
        let user_info = match (&user_data.email, &user_data.wallet_type) {
            (Some(email), Some(wallet_type)) => {
                format!("email: {}, wallet_type: {}", email, wallet_type)
            }
            (Some(email), None) => format!("email: {}", email),
            (None, Some(wallet_type)) => format!("wallet_type: {}", wallet_type),
            (None, None) => "no additional user info".to_string(),
        };

        Err(Box::new(AuthError::UnsupportedOperation(
            format!(
                "Magic.link signers require custom implementation for delegated signing. User address: {}, {}",
                wallet_address,
                user_info
            )
        )))
    }
}

#[async_trait]
impl SignerFactory for MagicProvider {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
        let did_token = auth_data
            .credentials
            .get("token")
            .ok_or_else(|| AuthError::MissingCredential("token".to_string()))?;

        // Validate the DID token
        let _claim = self
            .validate_did_token(did_token)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Get user metadata
        let user_data = self
            .get_user_metadata(did_token)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Create signer from user data
        self.create_signer_from_user(&user_data, &auth_data.network)
            .await
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

/// DID token claim structure
#[derive(Debug, Deserialize)]
struct DIDClaim {
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub ext: i64,
    /// Issuer (user's Ethereum public key)
    pub iss: String,
    /// User's Magic entity ID
    pub sub: String,
    /// Application's Magic entity ID
    pub aud: String,
    /// Unique token identifier
    pub tid: String,
}

/// Magic user metadata response
#[derive(Debug, Deserialize)]
struct MagicUserResponse {
    data: MagicUserData,
}

#[derive(Debug, Deserialize)]
struct MagicUserData {
    email: Option<String>,
    public_address: Option<String>,
    wallet_type: Option<String>,
}

/// Magic RPC request structure
#[derive(Debug, Serialize)]
struct MagicRPCRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

/// Magic RPC response structure
#[derive(Debug, Deserialize)]
struct MagicRPCResponse<T> {
    result: Option<T>,
    error: Option<MagicRPCError>,
}

#[derive(Debug, Deserialize)]
struct MagicRPCError {
    code: i32,
    message: String,
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
