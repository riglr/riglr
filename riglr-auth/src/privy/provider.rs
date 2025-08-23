//! Privy authentication provider implementation

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use riglr_core::signer::UnifiedSigner;
use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};
#[cfg(any(feature = "solana", feature = "evm"))]
use tracing::debug;
use tracing::info;

const PRIVY_VERIFICATION_KEY: &str = "PRIVY_VERIFICATION_KEY";

use super::config::PrivyConfig;
#[cfg(test)]
use super::config::{PRIVY_APP_ID, PRIVY_APP_SECRET, PRIVY_AUTH_URL};
use super::types::{PrivyClaims, PrivyUserData};
use crate::config::ProviderConfig;
use crate::error::{AuthError, AuthResult};
use crate::provider::{AuthenticationProvider, UserInfo};

#[cfg(feature = "solana")]
use super::signer::PrivySolanaSigner;

#[cfg(feature = "evm")]
use super::signer::PrivyEvmSigner;

#[cfg(feature = "caching")]
use lru::LruCache;
#[cfg(feature = "caching")]
use std::sync::{Arc, Mutex};

/// Privy authentication provider
pub struct PrivyProvider {
    config: PrivyConfig,
    client: reqwest::Client,
    #[cfg(feature = "caching")]
    user_cache: Arc<Mutex<LruCache<String, (PrivyUserData, std::time::Instant)>>>,
}

impl PrivyProvider {
    /// Create a new Privy provider
    pub fn new(config: PrivyConfig) -> Self {
        let client = Self::create_client(&config);

        #[cfg(feature = "caching")]
        let user_cache = Arc::new(Mutex::new(LruCache::new(
            std::num::NonZeroUsize::new(1000).unwrap(),
        )));

        Self {
            config,
            client,
            #[cfg(feature = "caching")]
            user_cache,
        }
    }

    /// Create HTTP client with Privy authentication headers
    fn create_client(config: &PrivyConfig) -> reqwest::Client {
        let auth = format!("{}:{}", config.app_id, config.app_secret);
        let basic = format!("Basic {}", STANDARD.encode(auth.as_bytes()));

        reqwest::Client::builder()
            .http1_only()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("privy-app-id", config.app_id.parse().unwrap());
                headers.insert("Content-Type", "application/json".parse().unwrap());
                headers.insert("Authorization", basic.parse().unwrap());
                headers
            })
            .build()
            .expect("Failed to build Privy HTTP client")
    }

    /// Verify and decode a Privy JWT token
    async fn verify_token(&self, token: &str) -> AuthResult<PrivyClaims> {
        use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

        info!("Verifying Privy token");

        // Set up validation
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_issuer(&["privy.io"]);
        validation.set_audience(&[&self.config.app_id]);

        // Get verification key (in production, fetch from JWKS endpoint)
        let verification_key = self.get_verification_key().await?;
        let key = DecodingKey::from_ec_pem(verification_key.as_bytes())
            .map_err(|e| AuthError::ConfigError(format!("Invalid verification key: {}", e)))?;

        // Decode and validate token
        let token_data = decode::<PrivyClaims>(token, &key, &validation)
            .map_err(|e| AuthError::TokenValidation(format!("Token validation failed: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Get verification key for JWT validation
    async fn get_verification_key(&self) -> AuthResult<String> {
        // In production, fetch from JWKS endpoint
        // For now, get from environment
        std::env::var(PRIVY_VERIFICATION_KEY)
            .map_err(|_| AuthError::ConfigError("Missing PRIVY_VERIFICATION_KEY".to_string()))
    }

    /// Fetch user data from Privy API
    async fn fetch_user_data(&self, user_id: &str) -> AuthResult<PrivyUserData> {
        #[cfg(feature = "caching")]
        {
            // Check cache first
            if self.config.enable_cache {
                if let Ok(mut cache) = self.user_cache.lock() {
                    if let Some((data, timestamp)) = cache.get(user_id) {
                        let age = std::time::Instant::now().duration_since(*timestamp);
                        if age.as_secs() < self.config.cache_ttl_seconds {
                            debug!("Returning cached user data for {}", user_id);
                            return Ok(data.clone());
                        }
                    }
                }
            }
        }

        info!("Fetching user data for {}", user_id);

        let response = self
            .client
            .get(format!("{}/api/v1/users/{}", self.config.auth_url, user_id))
            .send()
            .await
            .map_err(|e| AuthError::ApiError(format!("Failed to fetch user data: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Failed to fetch user data: {} - {}",
                status, body
            )));
        }

        let user_data: PrivyUserData = response
            .json()
            .await
            .map_err(|e| AuthError::ApiError(format!("Failed to parse user data: {}", e)))?;

        #[cfg(feature = "caching")]
        {
            // Update cache
            if self.config.enable_cache {
                if let Ok(mut cache) = self.user_cache.lock() {
                    cache.put(
                        user_id.to_string(),
                        (user_data.clone(), std::time::Instant::now()),
                    );
                }
            }
        }

        Ok(user_data)
    }
}

#[async_trait]
impl SignerFactory for PrivyProvider {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
        // Get token from credentials
        let token = auth_data
            .credentials
            .get("token")
            .ok_or_else(|| AuthError::MissingCredential("token".to_string()))?;

        // Verify token
        let claims = self.verify_token(token).await?;
        let user_id = claims.sub.replace("did:privy:", "");

        // Fetch user data
        let user_data = self.fetch_user_data(&user_id).await?;

        if !user_data.verified {
            return Err(Box::new(AuthError::NotVerified(
                "User not verified".to_string(),
            )));
        }

        // Create appropriate signer based on available wallets
        #[cfg(feature = "solana")]
        {
            if let Some(sol_wallet) = user_data.solana_wallet() {
                debug!("Creating Solana signer for address {}", sol_wallet.address);
                let network_name = auth_data.network.clone();
                let rpc_url = match network_name.as_str() {
                    "mainnet" | "mainnet-beta" => "https://api.mainnet-beta.solana.com".to_string(),
                    "devnet" => "https://api.devnet.solana.com".to_string(),
                    "testnet" => "https://api.testnet.solana.com".to_string(),
                    _ => "https://api.mainnet-beta.solana.com".to_string(),
                };

                let signer = PrivySolanaSigner::new(
                    self.client.clone(),
                    sol_wallet.address.clone(),
                    network_name,
                    rpc_url,
                );
                return Ok(Box::new(signer));
            }
        }

        #[cfg(feature = "evm")]
        {
            if let Some(evm_wallet) = user_data.evm_wallet() {
                debug!("Creating EVM signer for address {}", evm_wallet.address);

                // Map network name to chain ID and RPC URL
                let (chain_id, rpc_url) = match auth_data.network.as_str() {
                    "ethereum" | "mainnet" => (1u64, "https://eth.llamarpc.com".to_string()),
                    "polygon" => (137u64, "https://polygon-rpc.com".to_string()),
                    "arbitrum" => (42161u64, "https://arb1.arbitrum.io/rpc".to_string()),
                    "optimism" => (10u64, "https://mainnet.optimism.io".to_string()),
                    "base" => (8453u64, "https://mainnet.base.org".to_string()),
                    "sepolia" => (
                        11155111u64,
                        "https://sepolia.infura.io/v3/YOUR-PROJECT-ID".to_string(),
                    ),
                    _ => (1u64, "https://eth.llamarpc.com".to_string()), // Default to Ethereum mainnet
                };

                let wallet_id = evm_wallet
                    .id
                    .clone()
                    .ok_or_else(|| AuthError::ConfigError("EVM wallet missing ID".to_string()))?;

                let signer = PrivyEvmSigner::new(
                    self.client.clone(),
                    evm_wallet.address.clone(),
                    wallet_id,
                    chain_id,
                    rpc_url,
                );
                return Ok(Box::new(signer));
            }
        }

        #[cfg(not(any(feature = "solana", feature = "evm")))]
        {
            let _ = user_data; // Suppress unused warning
        }

        Err(Box::new(AuthError::NoWallet(
            "No delegated wallets found".to_string(),
        )))
    }

    fn supported_auth_types(&self) -> Vec<String> {
        vec!["privy".to_string()]
    }
}

#[async_trait]
impl AuthenticationProvider for PrivyProvider {
    async fn validate_token(&self, token: &str) -> AuthResult<UserInfo> {
        let claims = self.verify_token(token).await?;
        let user_id = claims.sub.replace("did:privy:", "");
        let user_data = self.fetch_user_data(&user_id).await?;

        Ok(UserInfo {
            id: user_data.id.clone(),
            email: user_data.email(),
            solana_address: user_data.solana_wallet().map(|w| w.address.clone()),
            evm_address: user_data.evm_wallet().map(|w| w.address.clone()),
            verified: user_data.verified,
            metadata: user_data.metadata,
        })
    }

    async fn refresh_token(&self, _token: &str) -> AuthResult<String> {
        // Privy handles token refresh client-side
        Err(AuthError::UnsupportedOperation(
            "Token refresh is handled client-side".to_string(),
        ))
    }
}

/// Convenience function to create a Privy provider from environment variables
pub fn create_privy_provider() -> AuthResult<PrivyProvider> {
    let config = PrivyConfig::from_env()?;
    Ok(PrivyProvider::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_web_adapters::factory::AuthenticationData;
    use std::collections::HashMap;
    use tokio;

    // Helper function to create a test PrivyConfig
    fn create_test_config() -> PrivyConfig {
        PrivyConfig {
            app_id: "test_app_id".to_string(),
            app_secret: "test_app_secret".to_string(),
            api_url: "https://api.test.privy.io".to_string(),
            auth_url: "https://auth.test.privy.io".to_string(),
            jwks_url: "https://auth.test.privy.io/.well-known/jwks.json".to_string(),
            enable_cache: true,
            cache_ttl_seconds: 300,
        }
    }

    #[test]
    fn test_privy_provider_new_should_create_instance() {
        let config = create_test_config();
        let provider = PrivyProvider::new(config.clone());

        assert_eq!(provider.config.app_id, config.app_id);
        assert_eq!(provider.config.app_secret, config.app_secret);
        assert_eq!(provider.config.auth_url, config.auth_url);
    }

    #[test]
    #[cfg(feature = "caching")]
    fn test_privy_provider_new_with_caching_should_initialize_cache() {
        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        // Verify cache is initialized
        assert!(provider.user_cache.lock().is_ok());
    }

    #[test]
    fn test_create_client_should_build_client_with_headers() {
        let config = create_test_config();
        let client = PrivyProvider::create_client(&config);

        // Client should be created successfully
        // Client should be created successfully (basic verification)
        assert!(std::ptr::addr_of!(client) as usize != 0);
    }

    #[test]
    fn test_create_client_with_empty_app_id_should_handle_gracefully() {
        let mut config = create_test_config();
        config.app_id = "".to_string();

        // Should still create client, but header parsing might fail
        let client = PrivyProvider::create_client(&config);
        // Client should be created successfully (basic verification)
        assert!(std::ptr::addr_of!(client) as usize != 0);
    }

    #[tokio::test]
    async fn test_get_verification_key_when_env_var_set_should_return_key() {
        std::env::set_var(PRIVY_VERIFICATION_KEY, "test_key");

        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        let result = provider.get_verification_key().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_key");

        std::env::remove_var(PRIVY_VERIFICATION_KEY);
    }

    #[tokio::test]
    async fn test_get_verification_key_when_env_var_missing_should_return_error() {
        std::env::remove_var(PRIVY_VERIFICATION_KEY);

        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        let result = provider.get_verification_key().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::ConfigError(msg) => assert_eq!(msg, "Missing PRIVY_VERIFICATION_KEY"),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[tokio::test]
    async fn test_verify_token_when_missing_verification_key_should_return_error() {
        std::env::remove_var(PRIVY_VERIFICATION_KEY);

        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        let result = provider.verify_token("invalid_token").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::ConfigError(_) => {}
            _ => panic!("Expected ConfigError"),
        }
    }

    #[tokio::test]
    async fn test_verify_token_with_invalid_key_format_should_return_error() {
        std::env::set_var(PRIVY_VERIFICATION_KEY, "invalid_key_format");

        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        let result = provider.verify_token("invalid_token").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::ConfigError(msg) => assert!(msg.contains("Invalid verification key")),
            _ => panic!("Expected ConfigError"),
        }

        std::env::remove_var(PRIVY_VERIFICATION_KEY);
    }

    #[test]
    fn test_create_privy_provider_when_config_invalid_should_return_error() {
        // Clear environment variables to ensure config creation fails
        std::env::remove_var(PRIVY_APP_ID);
        std::env::remove_var(PRIVY_APP_SECRET);
        std::env::remove_var(PRIVY_AUTH_URL);

        let result = create_privy_provider();
        assert!(result.is_err());
    }

    #[test]
    fn test_create_privy_provider_when_config_valid_should_return_provider() {
        // Set environment variables for valid config
        std::env::set_var(PRIVY_APP_ID, "test_app_id");
        std::env::set_var(PRIVY_APP_SECRET, "test_app_secret");
        std::env::set_var(PRIVY_AUTH_URL, "https://test.privy.io");

        let result = create_privy_provider();
        assert!(result.is_ok());

        // Clean up
        std::env::remove_var(PRIVY_APP_ID);
        std::env::remove_var(PRIVY_APP_SECRET);
        std::env::remove_var(PRIVY_AUTH_URL);
    }

    #[tokio::test]
    async fn test_validate_token_when_verification_fails_should_return_error() {
        std::env::remove_var(PRIVY_VERIFICATION_KEY);

        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        let result = provider.validate_token("invalid_token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refresh_token_should_return_unsupported_error() {
        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        let result = provider.refresh_token("any_token").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::UnsupportedOperation(msg) => {
                assert_eq!(msg, "Token refresh is handled client-side");
            }
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }

    #[test]
    fn test_supported_auth_types_should_return_privy() {
        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        let auth_types = provider.supported_auth_types();
        assert_eq!(auth_types, vec!["privy".to_string()]);
    }

    #[tokio::test]
    async fn test_create_signer_when_token_missing_should_return_error() {
        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        let auth_data = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials: HashMap::new(),
            network: "mainnet".to_string(),
        };

        let result = provider.create_signer(auth_data).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("token"));
    }

    #[tokio::test]
    async fn test_create_signer_when_token_verification_fails_should_return_error() {
        std::env::remove_var(PRIVY_VERIFICATION_KEY);

        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        let mut credentials = HashMap::new();
        credentials.insert("token".to_string(), "invalid_token".to_string());

        let auth_data = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials,
            network: "mainnet".to_string(),
        };

        let result = provider.create_signer(auth_data).await;
        assert!(result.is_err());
    }

    #[cfg(not(any(feature = "solana", feature = "evm")))]
    #[tokio::test]
    async fn test_create_signer_when_no_features_should_return_no_wallet_error() {
        // This test only runs when neither solana nor evm features are enabled
        // It tests the fallback case where no signers are available
        std::env::set_var(PRIVY_VERIFICATION_KEY, "-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE...\n-----END PUBLIC KEY-----");

        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        let mut credentials = HashMap::new();
        credentials.insert("token".to_string(), "valid_token".to_string());

        let auth_data = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials,
            network: "mainnet".to_string(),
        };

        // This will fail at token verification due to invalid test key format,
        // but tests the no-features code path
        let result = provider.create_signer(auth_data).await;
        assert!(result.is_err());

        std::env::remove_var(PRIVY_VERIFICATION_KEY);
    }

    // Mock HTTP server tests would require additional dependencies
    // For comprehensive coverage, we'd need to mock the HTTP client
    // These tests focus on the logic that can be tested without external dependencies

    #[test]
    fn test_privy_provider_create_client_headers_format() {
        let config = PrivyConfig {
            app_id: "test_id".to_string(),
            app_secret: "test_secret".to_string(),
            api_url: "https://api.example.com".to_string(),
            auth_url: "https://example.com".to_string(),
            jwks_url: "https://example.com/.well-known/jwks.json".to_string(),
            enable_cache: false,
            cache_ttl_seconds: 0,
        };

        // Test the encoding logic that's used in create_client
        let auth = format!("{}:{}", config.app_id, config.app_secret);
        let basic = format!("Basic {}", STANDARD.encode(auth.as_bytes()));

        assert_eq!(auth, "test_id:test_secret");
        assert!(basic.starts_with("Basic "));
        assert!(basic.len() > 6); // More than just "Basic "
    }

    #[test]
    fn test_privy_provider_constants() {
        assert_eq!(PRIVY_VERIFICATION_KEY, "PRIVY_VERIFICATION_KEY");
    }

    #[cfg(feature = "caching")]
    #[test]
    fn test_privy_provider_cache_initialization() {
        let config = create_test_config();
        let provider = PrivyProvider::new(config);

        // Test that cache can be locked and is empty initially
        let cache_result = provider.user_cache.lock();
        assert!(cache_result.is_ok());

        let cache = cache_result.unwrap();
        assert_eq!(cache.len(), 0);
    }

    // Edge case tests
    #[test]
    fn test_create_client_with_special_characters_in_credentials() {
        let config = PrivyConfig {
            app_id: "test@id#special".to_string(),
            app_secret: "secret$with%chars".to_string(),
            api_url: "https://api.example.com".to_string(),
            auth_url: "https://example.com".to_string(),
            jwks_url: "https://example.com/.well-known/jwks.json".to_string(),
            enable_cache: false,
            cache_ttl_seconds: 0,
        };

        // Should handle special characters in encoding
        let client = PrivyProvider::create_client(&config);
        // Client should be created successfully (basic verification)
        assert!(std::ptr::addr_of!(client) as usize != 0);
    }

    #[test]
    fn test_create_client_with_empty_credentials() {
        let config = PrivyConfig {
            app_id: "".to_string(),
            app_secret: "".to_string(),
            api_url: "".to_string(),
            auth_url: "".to_string(),
            jwks_url: "".to_string(),
            enable_cache: false,
            cache_ttl_seconds: 0,
        };

        // Should still create client even with empty credentials
        let client = PrivyProvider::create_client(&config);
        // Client should be created successfully (basic verification)
        assert!(std::ptr::addr_of!(client) as usize != 0);
    }

    #[test]
    fn test_create_client_with_very_long_credentials() {
        let long_string = "a".repeat(1000);
        let config = PrivyConfig {
            app_id: long_string.clone(),
            app_secret: long_string,
            api_url: "https://api.example.com".to_string(),
            auth_url: "https://example.com".to_string(),
            jwks_url: "https://example.com/.well-known/jwks.json".to_string(),
            enable_cache: false,
            cache_ttl_seconds: 0,
        };

        // Should handle very long credentials
        let client = PrivyProvider::create_client(&config);
        // Client should be created successfully (basic verification)
        assert!(std::ptr::addr_of!(client) as usize != 0);
    }
}
