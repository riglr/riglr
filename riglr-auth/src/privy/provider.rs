//! Privy authentication provider implementation

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use riglr_core::config::RpcConfig;
use riglr_core::signer::TransactionSigner;
use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};
use tracing::info;
#[cfg(any(feature = "solana", feature = "evm"))]
use tracing::debug;

use crate::error::{AuthError, AuthResult};
use crate::provider::{AuthenticationProvider, UserInfo};
use crate::config::ProviderConfig;
use super::config::PrivyConfig;
use super::types::{PrivyClaims, PrivyUserData};

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
            std::num::NonZeroUsize::new(1000).unwrap()
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
        std::env::var("PRIVY_VERIFICATION_KEY")
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
        
        let response = self.client
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
        
        let user_data: PrivyUserData = response.json().await
            .map_err(|e| AuthError::ApiError(format!("Failed to parse user data: {}", e)))?;
        
        #[cfg(feature = "caching")]
        {
            // Update cache
            if self.config.enable_cache {
                if let Ok(mut cache) = self.user_cache.lock() {
                    cache.put(user_id.to_string(), (user_data.clone(), std::time::Instant::now()));
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
        _config: &RpcConfig,
    ) -> Result<Box<dyn TransactionSigner>, Box<dyn std::error::Error + Send + Sync>> {
        // Get token from credentials
        let token = auth_data.credentials.get("token")
            .ok_or_else(|| AuthError::MissingCredential("token".to_string()))?;
        
        // Verify token
        let claims = self.verify_token(token).await?;
        let user_id = claims.sub.replace("did:privy:", "");
        
        // Fetch user data
        let user_data = self.fetch_user_data(&user_id).await?;
        
        if !user_data.verified {
            return Err(Box::new(AuthError::NotVerified("User not verified".to_string())));
        }
        
        // Create appropriate signer based on available wallets
        #[cfg(feature = "solana")]
        {
            if let Some(sol_wallet) = user_data.solana_wallet() {
                debug!("Creating Solana signer for address {}", sol_wallet.address);
                let sol_config = _config.solana_networks
                    .get(&auth_data.network)
                    .cloned()
                    .unwrap_or_else(|| {
                        use riglr_core::config::SolanaNetworkConfig;
                        SolanaNetworkConfig {
                            name: "mainnet".to_string(),
                            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                            explorer_url: None,
                        }
                    });
                
                let signer = PrivySolanaSigner::new(
                    self.client.clone(),
                    sol_wallet.address.clone(),
                    sol_config,
                );
                return Ok(Box::new(signer));
            }
        }
        
        #[cfg(feature = "evm")]
        {
            if let Some(evm_wallet) = user_data.evm_wallet() {
                debug!("Creating EVM signer for address {}", evm_wallet.address);
                let evm_config = _config.evm_networks
                    .get(&auth_data.network)
                    .cloned()
                    .ok_or_else(|| AuthError::ConfigError(format!("Unsupported EVM network: {}", auth_data.network)))?;
                
                let wallet_id = evm_wallet.id.clone()
                    .ok_or_else(|| AuthError::ConfigError("EVM wallet missing ID".to_string()))?;
                
                let signer = PrivyEvmSigner::new(
                    self.client.clone(),
                    evm_wallet.address.clone(),
                    wallet_id,
                    evm_config,
                );
                return Ok(Box::new(signer));
            }
        }
        
        #[cfg(not(any(feature = "solana", feature = "evm")))]
        {
            let _ = user_data; // Suppress unused warning
        }
        
        Err(Box::new(AuthError::NoWallet("No delegated wallets found".to_string())))
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
            "Token refresh is handled client-side".to_string()
        ))
    }
}

/// Convenience function to create a Privy provider from environment variables
pub fn create_privy_provider() -> AuthResult<PrivyProvider> {
    let config = PrivyConfig::from_env()?;
    Ok(PrivyProvider::new(config))
}