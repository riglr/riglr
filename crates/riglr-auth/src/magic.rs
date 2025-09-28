//! Magic.link authentication provider implementation
//!
//! Provides email-based authentication with embedded wallets.

use crate::config::ProviderConfig;
use crate::error::AuthError;
use async_trait::async_trait;
use core::error::Error;
use reqwest::header::HeaderMap;
use riglr_core::signer::UnifiedSigner;
// use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};
use crate::provider::{AuthenticationData, SignerFactory};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

const MAGIC_PUBLISHABLE_KEY: &str = "MAGIC_PUBLISHABLE_KEY";
const MAGIC_SECRET_KEY: &str = "MAGIC_SECRET_KEY";
const MAGIC_API_URL: &str = "MAGIC_API_URL";
const MAGIC_NETWORK: &str = "MAGIC_NETWORK";

/// Magic.link configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[expect(clippy::module_name_repetitions)]
pub struct MagicConfig {
    /// API base URL
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// Network (mainnet or testnet)
    #[serde(default = "default_network")]
    pub network: String,

    /// Magic publishable API key
    pub publishable_key: String,

    /// Magic secret key
    pub secret_key: String,
}

impl MagicConfig {
    /// Create a new Magic configuration
    #[must_use]
    pub fn new(publishable_key: String, secret_key: String) -> Self {
        Self {
            api_url: default_api_url(),
            network: default_network(),
            publishable_key,
            secret_key,
        }
    }
}

impl ProviderConfig for MagicConfig {
    fn from_env() -> Result<Self, AuthError> {
        let publishable_key = env::var(MAGIC_PUBLISHABLE_KEY)
            .map_err(|_| AuthError::ConfigError("MAGIC_PUBLISHABLE_KEY not found".to_string()))?;
        let secret_key = env::var(MAGIC_SECRET_KEY)
            .map_err(|_| AuthError::ConfigError("MAGIC_SECRET_KEY not found".to_string()))?;

        let mut config = Self::new(publishable_key, secret_key);

        if let Ok(api_url) = env::var(MAGIC_API_URL) {
            config.api_url = api_url;
        }
        if let Ok(network) = env::var(MAGIC_NETWORK) {
            config.network = network;
        }

        config.validate()?;
        Ok(config)
    }

    fn provider_name(&self) -> &'static str {
        "magic"
    }

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
}

/// Magic.link provider implementation
#[derive(Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct MagicProvider {
    client: reqwest::Client,
    config: MagicConfig,
}

impl MagicProvider {
    /// Create a new Magic provider
    #[must_use]
    pub fn new(config: MagicConfig) -> Self {
        let mut headers = HeaderMap::new();

        // Insert headers with proper error handling
        if let Ok(secret_key_header) = config.secret_key.parse() {
            headers.insert("X-Magic-Secret-Key", secret_key_header);
        }
        if let Ok(content_type_header) = "application/json".parse() {
            headers.insert("Content-Type", content_type_header);
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client, config }
    }

    /// Decode and validate a Magic DID token
    async fn validate_did_token(&self, did_token: &str) -> Result<DIDClaim, AuthError> {
        // Decode the base64-encoded DID token
        use base64::{engine::general_purpose::STANDARD, Engine};
        let decoded_bytes = STANDARD
            .decode(did_token)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid base64 DID token: {e}")))?;

        let token_str = String::from_utf8(decoded_bytes)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid UTF-8 in DID token: {e}")))?;

        // Parse the [proof, claim] tuple
        let token_tuple: serde_json::Value = serde_json::from_str(&token_str)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid JSON in DID token: {e}")))?;

        let token_array = token_tuple
            .as_array()
            .ok_or_else(|| AuthError::TokenValidation("DID token must be an array".to_string()))?;

        if token_array.len() != 2 {
            return Err(AuthError::TokenValidation(
                "DID token must have exactly 2 elements".to_string(),
            ));
        }

        let _proof = token_array
            .first()
            .ok_or_else(|| {
                AuthError::TokenValidation("DID token must have proof element".to_string())
            })?
            .as_str()
            .ok_or_else(|| {
                AuthError::TokenValidation("DID token proof must be a string".to_string())
            })?;

        let claim_str = token_array
            .get(1)
            .ok_or_else(|| {
                AuthError::TokenValidation("DID token must have claim element".to_string())
            })?
            .as_str()
            .ok_or_else(|| {
                AuthError::TokenValidation("DID token claim must be a string".to_string())
            })?;

        // Parse the claim
        let claim: DIDClaim = serde_json::from_str(claim_str)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid claim format: {e}")))?;

        // Validate token timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AuthError::TokenValidation("System time error".to_string()))?
            .as_secs()
            .try_into()
            .map_err(|_| AuthError::TokenValidation("Timestamp conversion error".to_string()))?;

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

    /// Create a signer from Magic user data
    fn create_signer_from_user(
        user_data: &MagicUserData,
        network: &str,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn Error + Send + Sync>> {
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
        let user_info = match (user_data.email.as_ref(), user_data.wallet_type.as_ref()) {
            (Some(email), Some(wallet_type)) => {
                format!("email: {email}, wallet_type: {wallet_type}")
            }
            (Some(email), None) => format!("email: {email}"),
            (None, Some(wallet_type)) => format!("wallet_type: {wallet_type}"),
            (None, None) => "no additional user info".to_string(),
        };

        Err(Box::new(AuthError::UnsupportedOperation(
            format!(
                "Magic.link signers require custom implementation for delegated signing. User address: {wallet_address}, {user_info}"
            )
        )))
    }

    /// Get user metadata from Magic
    async fn get_user_metadata(&self, did_token: &str) -> Result<MagicUserData, AuthError> {
        let request = MagicRPCRequest {
            id: 2,
            jsonrpc: "2.0".to_string(),
            method: "magic_token_get_public_address".to_string(),
            params: serde_json::json!([did_token]),
        };

        let response = self
            .client
            .post(format!("{}/v1/admin/auth/user/get", self.config.api_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| AuthError::ApiError(format!("Failed to fetch user metadata: {e}")))?;

        let rpc_response: MagicRPCResponse<MagicUserResponse> = response
            .json()
            .await
            .map_err(|e| AuthError::ApiError(format!("Invalid response from Magic API: {e}")))?;

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

    /// Validate token using Magic's API
    async fn validate_token_with_api(&self, did_token: &str) -> Result<(), AuthError> {
        let request = MagicRPCRequest {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "magic_token_validate".to_string(),
            params: serde_json::json!([did_token]),
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
                AuthError::ApiError(format!("Failed to validate token with Magic API: {e}"))
            })?;

        let rpc_response: MagicRPCResponse<bool> = response
            .json()
            .await
            .map_err(|e| AuthError::ApiError(format!("Invalid response from Magic API: {e}")))?;

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
}

#[async_trait]
impl SignerFactory for MagicProvider {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn Error + Send + Sync>> {
        let did_token = auth_data
            .credentials
            .get("token")
            .ok_or_else(|| AuthError::MissingCredential("token".to_string()))?;

        // Validate the DID token
        let _claim = self
            .validate_did_token(did_token)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // Get user metadata
        let user_data = self
            .get_user_metadata(did_token)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // Create signer from user data
        Self::create_signer_from_user(&user_data, &auth_data.network)
    }

    fn supported_auth_types(&self) -> Vec<String> {
        vec!["magic".to_string()]
    }
}

/// DID token claim structure
#[derive(Debug, Deserialize)]
struct DIDClaim {
    /// Application's Magic entity ID
    pub aud: String,
    /// Expiration timestamp
    pub ext: i64,
    /// Issued at timestamp
    pub iat: i64,
    /// Issuer (user's Ethereum public key)
    pub iss: String,
    /// User's Magic entity ID
    pub sub: String,
    /// Unique token identifier
    pub tid: String,
}

fn default_api_url() -> String {
    "https://api.magic.link".to_string()
}

/// Magic user metadata response
#[derive(Debug, Deserialize)]
struct MagicUserResponse {
    data: MagicUserData,
}

fn default_network() -> String {
    "mainnet".to_string()
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
    id: u64,
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
}

/// Magic RPC response structure
#[derive(Debug, Deserialize)]
struct MagicRPCResponse<T> {
    error: Option<MagicRPCError>,
    result: Option<T>,
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
    fn test_magic_config_validate_when_valid_should_return_ok() {
        let config = MagicConfig::new("pk_test_123".to_string(), "sk_test_456".to_string());

        let result = config.validate();

        assert!(result.is_ok());
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_magic_config_validate_when_empty_publishable_key_should_return_err() {
        let config = MagicConfig::new(String::new(), "sk_test_456".to_string());

        let result = config.validate();

        assert!(result.is_err());
        match result {
            Err(AuthError::ConfigError(msg)) => {
                assert_eq!(msg, "Magic publishable key required");
            }
            _ => panic!("Expected ConfigError, got: {result:?}"),
        }
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_magic_config_validate_when_empty_secret_key_should_return_err() {
        let config = MagicConfig::new("pk_test_123".to_string(), String::new());

        let result = config.validate();

        assert!(result.is_err());
        match result {
            Err(AuthError::ConfigError(msg)) => {
                assert_eq!(msg, "Magic secret key required");
            }
            _ => panic!("Expected ConfigError, got: {result:?}"),
        }
    }

    #[test]
    fn test_magic_config_provider_name_should_return_magic() {
        let config = MagicConfig::new("pk_test_123".to_string(), "sk_test_456".to_string());

        let name = config.provider_name();

        assert_eq!(name, "magic");
    }

    #[test]
    fn test_magic_provider_new_when_valid_config_should_create_provider() {
        let config = MagicConfig::new("pk_test_123".to_string(), "sk_test_456".to_string());

        let provider = MagicProvider::new(config.clone());

        assert_eq!(provider.config.publishable_key, config.publishable_key);
        assert_eq!(provider.config.secret_key, config.secret_key);
    }

    #[tokio::test]
    #[expect(clippy::panic, clippy::expect_used)]
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
        let error = result.expect_err("Expected error for missing token credential");
        let auth_error = error
            .downcast::<AuthError>()
            .expect("Expected AuthError type");
        match *auth_error {
            AuthError::MissingCredential(ref field) => {
                assert_eq!(field, "token");
            }
            _ => panic!("Expected MissingCredential error, got: {auth_error:?}"),
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
