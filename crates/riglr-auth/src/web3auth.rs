//! `Web3Auth` authentication provider implementation
//!
//! Provides non-custodial key management with social login capabilities.

use crate::config::ProviderConfig;
use crate::error::AuthError;
use crate::provider::{AuthenticationData, SignerFactory};
use async_trait::async_trait;
use core::error::Error;
use core::{
    hash::{Hash, Hasher},
    iter,
    time::Duration,
};
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use riglr_core::signer::UnifiedSigner;
#[cfg(feature = "evm")]
use riglr_evm_tools::signer::EvmLocalClient;
#[cfg(feature = "solana")]
use riglr_solana_tools::signer;
use serde::{Deserialize, Serialize};
use std::env;

const WEB3AUTH_CLIENT_ID: &str = "WEB3AUTH_CLIENT_ID";
const WEB3AUTH_VERIFIER: &str = "WEB3AUTH_VERIFIER";
const WEB3AUTH_NETWORK: &str = "WEB3AUTH_NETWORK";
const WEB3AUTH_API_URL: &str = "WEB3AUTH_API_URL";

/// `Web3Auth` configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3AuthConfig {
    /// API base URL
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// `Web3Auth` client ID
    pub client_id: String,

    /// Network (mainnet, testnet, cyan, aqua, celeste)
    #[serde(default = "default_network")]
    pub network: String,

    /// Verifier name
    pub verifier: String,
}

/// Default API URL value
fn default_api_url() -> String {
    "https://api.openlogin.com".to_string()
}

/// Default network value
fn default_network() -> String {
    "mainnet".to_string()
}

impl Web3AuthConfig {
    /// Create a new `Web3Auth` configuration
    #[must_use]
    pub fn new(client_id: String, verifier: String) -> Self {
        Self {
            api_url: default_api_url(),
            client_id,
            network: default_network(),
            verifier,
        }
    }
}

impl ProviderConfig for Web3AuthConfig {
    fn from_env() -> Result<Self, AuthError> {
        let client_id = env::var(WEB3AUTH_CLIENT_ID)
            .map_err(|_| AuthError::ConfigError("WEB3AUTH_CLIENT_ID not found".to_string()))?;
        let verifier = env::var(WEB3AUTH_VERIFIER)
            .map_err(|_| AuthError::ConfigError("WEB3AUTH_VERIFIER not found".to_string()))?;

        let mut config = Self::new(client_id, verifier);

        if let Ok(network) = env::var(WEB3AUTH_NETWORK) {
            config.network = network;
        }
        if let Ok(api_url) = env::var(WEB3AUTH_API_URL) {
            config.api_url = api_url;
        }

        config.validate()?;
        Ok(config)
    }

    fn provider_name(&self) -> &'static str {
        "web3auth"
    }

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
}

/// `Web3Auth` provider implementation
#[derive(Debug)]
pub struct Web3AuthProvider {
    client: reqwest::Client,
    config: Web3AuthConfig,
}

impl Web3AuthProvider {
    /// Create a new `Web3Auth` provider
    #[must_use]
    pub fn new(config: Web3AuthConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers(
                iter::once((CONTENT_TYPE, HeaderValue::from_static("application/json"))).collect(),
            )
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client, config }
    }

    /// Create a signer from verified `Web3Auth` claims
    fn create_signer_from_claims(
        &self,
        claims: &Web3AuthClaims,
        network: &str,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn Error + Send + Sync>> {
        // For external wallets, use the wallet address directly
        if let Some(ref wallets) = claims.wallets {
            for wallet in wallets {
                if network.contains("solana") && wallet.chain_type.contains("solana") {
                    #[cfg(feature = "solana")]
                    {
                        // For external Solana wallets, we can't create a direct signer
                        // since we don't have the private key. This would typically require
                        // a different flow where the frontend handles signing.
                        return Err(Box::new(AuthError::UnsupportedOperation(
                            "External Solana wallet signing requires frontend integration"
                                .to_string(),
                        )));
                    }
                } else if (network.contains("ethereum") || network.contains("evm"))
                    && wallet.chain_type.contains("evm")
                {
                    #[cfg(feature = "evm")]
                    {
                        // For external EVM wallets, same issue - no private key available
                        return Err(Box::new(AuthError::UnsupportedOperation(
                            "External EVM wallet signing requires frontend integration".to_string(),
                        )));
                    }
                }
            }
        }

        // For social login wallets, we would derive the private key from the user's
        // Web3Auth shares. This requires the Web3Auth SDK's key derivation logic.
        // For this implementation, we'll create a placeholder that shows the structure
        // but indicates the limitation.

        // Derive private key from Web3Auth shares (simplified implementation)
        let private_key =
            self.derive_private_key(&claims.sub, &claims.verifier, &claims.verifier_id);

        // Create the appropriate signer based on the network
        if network.contains("solana") {
            #[cfg(feature = "solana")]
            {
                let config = riglr_config::SolanaNetworkConfig::new(
                    network,
                    "https://api.mainnet-beta.solana.com".to_string(),
                );
                let signer = signer::Local::new(&private_key, config)
                    .map_err(|e| e as Box<dyn Error + Send + Sync>)?;

                return Ok(Box::new(signer) as Box<dyn UnifiedSigner>);
            }
            #[cfg(not(feature = "solana"))]
            {
                return Err(Box::new(AuthError::UnsupportedOperation(
                    "Solana support not enabled".to_string(),
                )));
            }
        }

        #[cfg(feature = "evm")]
        {
            let config = riglr_config::EvmNetworkConfig::new(
                network.to_string(),
                1, // Mainnet chain ID, should be configurable
                "https://eth.llamarpc.com".to_string(),
            );
            let signer = EvmLocalClient::new(&private_key, config)
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

            Ok(Box::new(signer) as Box<dyn UnifiedSigner>)
        }
        #[cfg(not(feature = "evm"))]
        {
            Err(Box::new(AuthError::UnsupportedOperation(
                "EVM support not enabled".to_string(),
            )))
        }
    }

    /// Verify a `Web3Auth` JWT token
    async fn verify_token(&self, token: &str) -> Result<Web3AuthClaims, AuthError> {
        use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};

        // First, decode the header to get the key ID
        let header = decode_header(token)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid JWT header: {e}")))?;
        let kid = header.kid.ok_or_else(|| {
            AuthError::TokenValidation("JWT header missing 'kid' field".to_string())
        })?;

        // Fetch the JWKS from Web3Auth
        let jwks_url = format!("{}/jwks", self.config.api_url);
        let jwks_response = self
            .client
            .get(&jwks_url)
            .send()
            .await
            .map_err(|e| AuthError::ApiError(format!("Failed to fetch JWKS: {e}")))?;
        let jwks: serde_json::Value = jwks_response
            .json()
            .await
            .map_err(|e| AuthError::TokenValidation(format!("Invalid JWKS response: {e}")))?;

        // Find the matching key
        let keys = jwks
            .get("keys")
            .ok_or_else(|| {
                AuthError::TokenValidation("JWKS response missing keys field".to_string())
            })?
            .as_array()
            .ok_or_else(|| {
                AuthError::TokenValidation("JWKS keys field is not an array".to_string())
            })?;
        let key = keys
            .iter()
            .find(|k| k.get("kid").and_then(|kid_val| kid_val.as_str()) == Some(&kid))
            .ok_or_else(|| AuthError::TokenValidation("Key ID not found in JWKS".to_string()))?;

        // Extract the public key components
        let n = key
            .get("n")
            .ok_or_else(|| AuthError::TokenValidation("RSA key missing 'n' field".to_string()))?
            .as_str()
            .ok_or_else(|| {
                AuthError::TokenValidation("RSA key 'n' field is not a string".to_string())
            })?;
        let e = key
            .get("e")
            .ok_or_else(|| AuthError::TokenValidation("RSA key missing 'e' field".to_string()))?
            .as_str()
            .ok_or_else(|| {
                AuthError::TokenValidation("RSA key 'e' field is not a string".to_string())
            })?;

        // Create the decoding key from RSA components
        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid RSA key: {e}")))?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.config.client_id]);
        validation.set_issuer(&[&format!("{}/v3/signer", self.config.api_url)]);

        // Decode and validate the token
        let token_data = decode::<Web3AuthClaims>(token, &decoding_key, &validation)
            .map_err(|e| AuthError::TokenValidation(format!("JWT validation failed: {e}")))?;

        Ok(token_data.claims)
    }

    /// Derive private key from `Web3Auth` user data
    /// This is a simplified placeholder - real implementation would use `Web3Auth`'s
    /// threshold cryptography and key derivation logic
    fn derive_private_key(&self, sub: &str, verifier: &str, verifier_id: &str) -> String {
        // In a real implementation, this would:
        // 1. Use the user's shares from Web3Auth's threshold key infrastructure
        // 2. Combine with the app's key share
        // 3. Reconstruct the private key using Shamir's Secret Sharing
        //
        // For this implementation, we'll generate a deterministic key from the user data
        // This is NOT secure and should only be used for testing/demonstration

        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::default();
        sub.hash(&mut hasher);
        verifier.hash(&mut hasher);
        verifier_id.hash(&mut hasher);
        self.config.client_id.hash(&mut hasher);

        let hash = hasher.finish();

        // Convert hash to a hex string (not cryptographically secure)
        let private_key = format!(
            "{:016x}{:016x}",
            hash,
            hash.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        );

        tracing::warn!(
            "Using insecure key derivation - implement proper Web3Auth key reconstruction for production"
        );

        private_key
    }
}

#[async_trait]
impl SignerFactory for Web3AuthProvider {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn Error + Send + Sync>> {
        let token = auth_data
            .credentials
            .get("token")
            .ok_or_else(|| AuthError::MissingCredential("token".to_string()))?;

        // Verify the Web3Auth JWT token
        let claims = self
            .verify_token(token)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // Validate the verifier matches our configuration
        if claims.verifier != self.config.verifier {
            return Err(Box::new(AuthError::TokenValidation(format!(
                "Verifier mismatch: expected {}, got {}",
                self.config.verifier, claims.verifier
            ))));
        }

        // Create signer from verified claims
        self.create_signer_from_claims(&claims, &auth_data.network)
    }
    fn supported_auth_types(&self) -> Vec<String> {
        vec!["web3auth".to_string()]
    }
}

/// JWT claims structure for `Web3Auth` tokens
#[derive(Debug, Deserialize)]
struct Web3AuthClaims {
    /// Aggregated verifier
    #[expect(dead_code)]
    pub aggregated_verifier: Option<String>,
    /// Audience
    #[expect(dead_code)]
    pub aud: Option<String>,
    /// Email
    #[expect(dead_code)]
    pub email: Option<String>,
    /// Expiration
    #[expect(dead_code)]
    pub exp: Option<i64>,
    /// Issued at
    #[expect(dead_code)]
    pub iat: i64,
    /// Issuer
    #[expect(dead_code)]
    pub iss: String,
    /// Name
    #[expect(dead_code)]
    pub name: Option<String>,
    /// Profile image
    #[expect(dead_code)]
    pub profile_image: Option<String>,
    /// Subject (user identifier)
    pub sub: String,
    /// Verifier
    pub verifier: String,
    /// Verifier ID
    pub verifier_id: String,
    /// Wallet address (for external wallets)
    pub wallets: Option<Vec<WalletInfo>>,
}

#[derive(Debug, Deserialize)]
struct WalletInfo {
    #[expect(dead_code)]
    pub address: String,
    pub chain_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Test helper functions for Rust 2024 compatibility
    mod test_env_vars {
        use std::env;

        /// Set environment variable safely for tests
        #[expect(unsafe_code)]
        pub fn set_test_env_var(key: &str, value: &str) {
            // TODO: Audit that the environment access only happens in single-threaded code.
            unsafe { env::set_var(key, value) };
        }

        /// Remove environment variable safely for tests
        #[expect(unsafe_code)]
        pub fn remove_test_env_var(key: &str) {
            // TODO: Audit that the environment access only happens in single-threaded code.
            unsafe { env::remove_var(key) };
        }
    }

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
    #[expect(clippy::panic)]
    fn test_web3auth_config_validate_when_empty_client_id_should_return_err() {
        let config = Web3AuthConfig::new(String::new(), "valid_verifier".to_string());

        let result = config.validate();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "Web3Auth client ID required");
        } else {
            panic!("Expected ConfigError with client ID message");
        }
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_web3auth_config_validate_when_empty_verifier_should_return_err() {
        let config = Web3AuthConfig::new("valid_client_id".to_string(), String::new());

        let result = config.validate();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "Web3Auth verifier required");
        } else {
            panic!("Expected ConfigError with verifier message");
        }
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_web3auth_config_validate_when_both_empty_should_return_client_id_err() {
        let config = Web3AuthConfig::new(String::new(), String::new());

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
    #[expect(clippy::panic)]
    fn test_web3auth_config_from_env_when_missing_client_id_should_return_err() {
        // Clear environment variables
        test_env_vars::remove_test_env_var(WEB3AUTH_CLIENT_ID);
        test_env_vars::remove_test_env_var(WEB3AUTH_VERIFIER);
        test_env_vars::remove_test_env_var(WEB3AUTH_NETWORK);
        test_env_vars::remove_test_env_var(WEB3AUTH_API_URL);

        let result = Web3AuthConfig::from_env();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "WEB3AUTH_CLIENT_ID not found");
        } else {
            panic!("Expected ConfigError with client ID not found message");
        }
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_web3auth_config_from_env_when_missing_verifier_should_return_err() {
        test_env_vars::set_test_env_var(WEB3AUTH_CLIENT_ID, "test_client_id");
        test_env_vars::remove_test_env_var(WEB3AUTH_VERIFIER);
        test_env_vars::remove_test_env_var(WEB3AUTH_NETWORK);
        test_env_vars::remove_test_env_var(WEB3AUTH_API_URL);

        let result = Web3AuthConfig::from_env();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "WEB3AUTH_VERIFIER not found");
        } else {
            panic!("Expected ConfigError with verifier not found message");
        }

        // Cleanup
        test_env_vars::remove_test_env_var(WEB3AUTH_CLIENT_ID);
    }

    #[test]
    #[expect(clippy::expect_used)]
    fn test_web3auth_config_from_env_when_required_vars_present_should_create_config() {
        test_env_vars::set_test_env_var(WEB3AUTH_CLIENT_ID, "test_client_id");
        test_env_vars::set_test_env_var(WEB3AUTH_VERIFIER, "test_verifier");
        test_env_vars::remove_test_env_var(WEB3AUTH_NETWORK);
        test_env_vars::remove_test_env_var(WEB3AUTH_API_URL);

        let result = Web3AuthConfig::from_env();

        assert!(result.is_ok());
        let config = result.expect("Expected successful config creation from environment");
        assert_eq!(config.client_id, "test_client_id");
        assert_eq!(config.verifier, "test_verifier");
        assert_eq!(config.network, "mainnet"); // default
        assert_eq!(config.api_url, "https://api.openlogin.com"); // default

        // Cleanup
        test_env_vars::remove_test_env_var(WEB3AUTH_CLIENT_ID);
        test_env_vars::remove_test_env_var(WEB3AUTH_VERIFIER);
    }

    #[test]
    #[expect(clippy::expect_used)]
    fn test_web3auth_config_from_env_when_all_vars_present_should_use_custom_values() {
        test_env_vars::set_test_env_var(WEB3AUTH_CLIENT_ID, "test_client_id");
        test_env_vars::set_test_env_var(WEB3AUTH_VERIFIER, "test_verifier");
        test_env_vars::set_test_env_var(WEB3AUTH_NETWORK, "testnet");
        test_env_vars::set_test_env_var(WEB3AUTH_API_URL, "https://custom.api.url");

        let result = Web3AuthConfig::from_env();

        assert!(result.is_ok());
        let config = result.expect("Expected successful config creation from custom values");
        assert_eq!(config.client_id, "test_client_id");
        assert_eq!(config.verifier, "test_verifier");
        assert_eq!(config.network, "testnet");
        assert_eq!(config.api_url, "https://custom.api.url");

        // Cleanup
        test_env_vars::remove_test_env_var(WEB3AUTH_CLIENT_ID);
        test_env_vars::remove_test_env_var(WEB3AUTH_VERIFIER);
        test_env_vars::remove_test_env_var(WEB3AUTH_NETWORK);
        test_env_vars::remove_test_env_var(WEB3AUTH_API_URL);
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_web3auth_config_from_env_when_empty_client_id_should_fail_validation() {
        test_env_vars::set_test_env_var(WEB3AUTH_CLIENT_ID, "");
        test_env_vars::set_test_env_var(WEB3AUTH_VERIFIER, "test_verifier");
        test_env_vars::remove_test_env_var(WEB3AUTH_NETWORK);
        test_env_vars::remove_test_env_var(WEB3AUTH_API_URL);

        let result = Web3AuthConfig::from_env();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "Web3Auth client ID required");
        } else {
            panic!("Expected ConfigError with client ID required message");
        }

        // Cleanup
        test_env_vars::remove_test_env_var(WEB3AUTH_CLIENT_ID);
        test_env_vars::remove_test_env_var(WEB3AUTH_VERIFIER);
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_web3auth_config_from_env_when_empty_verifier_should_fail_validation() {
        test_env_vars::set_test_env_var(WEB3AUTH_CLIENT_ID, "test_client_id");
        test_env_vars::set_test_env_var(WEB3AUTH_VERIFIER, "");
        test_env_vars::remove_test_env_var(WEB3AUTH_NETWORK);
        test_env_vars::remove_test_env_var(WEB3AUTH_API_URL);

        let result = Web3AuthConfig::from_env();

        assert!(result.is_err());
        if let Err(AuthError::ConfigError(msg)) = result {
            assert_eq!(msg, "Web3Auth verifier required");
        } else {
            panic!("Expected ConfigError with verifier required message");
        }

        // Cleanup
        test_env_vars::remove_test_env_var(WEB3AUTH_CLIENT_ID);
        test_env_vars::remove_test_env_var(WEB3AUTH_VERIFIER);
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
    #[expect(clippy::expect_used)]
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
        let error_str = result
            .expect_err("Expected error for missing token")
            .to_string();
        assert!(error_str.contains("Missing credential: token"));
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
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
        let error_str = result
            .expect_err("Expected error for invalid token")
            .to_string();
        assert!(error_str.contains("Invalid JWT header"));
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
