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
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("Content-Type", "application/json".parse().unwrap());
                headers
            })
            .build()
            .expect("Failed to build Web3Auth HTTP client");

        Self { config, client }
    }

    /// Verify a Web3Auth JWT token
    async fn verify_token(&self, token: &str) -> Result<Web3AuthClaims, AuthError> {
        use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};

        // First, decode the header to get the key ID
        let header = decode_header(token)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid JWT header: {}", e)))?;

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
            .map_err(|e| AuthError::ApiError(format!("Failed to fetch JWKS: {}", e)))?;

        let jwks: serde_json::Value = jwks_response
            .json()
            .await
            .map_err(|e| AuthError::TokenValidation(format!("Invalid JWKS response: {}", e)))?;

        // Find the matching key
        let keys = jwks["keys"]
            .as_array()
            .ok_or_else(|| AuthError::TokenValidation("JWKS missing keys array".to_string()))?;

        let key = keys
            .iter()
            .find(|k| k["kid"].as_str() == Some(&kid))
            .ok_or_else(|| AuthError::TokenValidation("Key ID not found in JWKS".to_string()))?;

        // Extract the public key components
        let n = key["n"]
            .as_str()
            .ok_or_else(|| AuthError::TokenValidation("Missing 'n' in key".to_string()))?;
        let e = key["e"]
            .as_str()
            .ok_or_else(|| AuthError::TokenValidation("Missing 'e' in key".to_string()))?;

        // Create the decoding key from RSA components
        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid RSA key: {}", e)))?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.config.client_id]);
        validation.set_issuer(&[&format!("{}/v3/signer", self.config.api_url)]);

        // Decode and validate the token
        let token_data = decode::<Web3AuthClaims>(token, &decoding_key, &validation)
            .map_err(|e| AuthError::TokenValidation(format!("JWT validation failed: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Create a signer from verified Web3Auth claims
    async fn create_signer_from_claims(
        &self,
        claims: &Web3AuthClaims,
        network: &str,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
        // For external wallets, use the wallet address directly
        if let Some(wallets) = &claims.wallets {
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
        let private_key = self
            .derive_private_key(&claims.sub, &claims.verifier, &claims.verifier_id)
            .await?;

        // Create the appropriate signer based on the network
        if network.contains("solana") {
            #[cfg(feature = "solana")]
            {
                let config = riglr_config::SolanaNetworkConfig::new(
                    network,
                    "https://api.mainnet-beta.solana.com".to_string(),
                );
                let signer =
                    riglr_solana_tools::signer::local::LocalSolanaSigner::new(private_key, config)
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

                Ok(Box::new(signer) as Box<dyn UnifiedSigner>)
            }
            #[cfg(not(feature = "solana"))]
            {
                Err(Box::new(AuthError::UnsupportedOperation(
                    "Solana support not enabled".to_string(),
                )))
            }
        } else {
            #[cfg(feature = "evm")]
            {
                let config = riglr_config::EvmNetworkConfig::new(
                    network,
                    1, // Mainnet chain ID, should be configurable
                    "https://eth.llamarpc.com".to_string(),
                );
                let signer = riglr_evm_tools::signer::LocalEvmSigner::new(private_key, config)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

                Ok(Box::new(signer) as Box<dyn UnifiedSigner>)
            }
            #[cfg(not(feature = "evm"))]
            {
                Err(Box::new(AuthError::UnsupportedOperation(
                    "EVM support not enabled".to_string(),
                )))
            }
        }
    }

    /// Derive private key from Web3Auth user data
    /// This is a simplified placeholder - real implementation would use Web3Auth's
    /// threshold cryptography and key derivation logic
    async fn derive_private_key(
        &self,
        sub: &str,
        verifier: &str,
        verifier_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would:
        // 1. Use the user's shares from Web3Auth's threshold key infrastructure
        // 2. Combine with the app's key share
        // 3. Reconstruct the private key using Shamir's Secret Sharing
        //
        // For this implementation, we'll generate a deterministic key from the user data
        // This is NOT secure and should only be used for testing/demonstration

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        sub.hash(&mut hasher);
        verifier.hash(&mut hasher);
        verifier_id.hash(&mut hasher);
        self.config.client_id.hash(&mut hasher);

        let hash = hasher.finish();

        // Convert hash to a hex string (not cryptographically secure)
        let private_key = format!(
            "{:016x}{:016x}",
            hash,
            hash.wrapping_mul(0x9e3779b97f4a7c15)
        );

        tracing::warn!(
            "Using insecure key derivation - implement proper Web3Auth key reconstruction for production"
        );

        Ok(private_key)
    }
}

#[async_trait]
impl SignerFactory for Web3AuthProvider {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
        let token = auth_data
            .credentials
            .get("token")
            .ok_or_else(|| AuthError::MissingCredential("token".to_string()))?;

        // Verify the Web3Auth JWT token
        let claims = self
            .verify_token(token)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Validate the verifier matches our configuration
        if claims.verifier != self.config.verifier {
            return Err(Box::new(AuthError::TokenValidation(format!(
                "Verifier mismatch: expected {}, got {}",
                self.config.verifier, claims.verifier
            ))));
        }

        // Create signer from verified claims
        self.create_signer_from_claims(&claims, &auth_data.network)
            .await
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

/// JWT claims structure for Web3Auth tokens
#[derive(Debug, Deserialize)]
struct Web3AuthClaims {
    /// Subject (user identifier)
    pub sub: String,
    /// Audience
    #[allow(dead_code)]
    pub aud: Option<String>,
    /// Issuer
    #[allow(dead_code)]
    pub iss: String,
    /// Issued at
    #[allow(dead_code)]
    pub iat: i64,
    /// Expiration
    #[allow(dead_code)]
    pub exp: Option<i64>,
    /// Email
    #[allow(dead_code)]
    pub email: Option<String>,
    /// Name
    #[allow(dead_code)]
    pub name: Option<String>,
    /// Profile image
    #[allow(dead_code)]
    pub profile_image: Option<String>,
    /// Aggregated verifier
    #[allow(dead_code)]
    pub aggregated_verifier: Option<String>,
    /// Verifier
    pub verifier: String,
    /// Verifier ID
    pub verifier_id: String,
    /// Wallet address (for external wallets)
    pub wallets: Option<Vec<WalletInfo>>,
}

#[derive(Debug, Deserialize)]
struct WalletInfo {
    #[allow(dead_code)]
    pub address: String,
    pub chain_type: String,
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
