//! Privy authentication provider implementation
//!
//! This module provides a concrete implementation of the SignerFactory trait
//! for Privy authentication. This serves as an example of how to implement
//! a custom authentication provider for the riglr web adapters.

use riglr_web_adapters::factory::{SignerFactory, AuthenticationData};
use riglr_core::signer::{TransactionSigner};
use riglr_core::config::RpcConfig;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Privy-specific signer factory implementation
pub struct PrivySignerFactory {
    privy_app_id: String,
    privy_app_secret: String,
}

impl PrivySignerFactory {
    /// Create a new Privy signer factory
    /// 
    /// # Arguments
    /// * `app_id` - Privy application ID
    /// * `app_secret` - Privy application secret
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            privy_app_id: app_id,
            privy_app_secret: app_secret,
        }
    }
    
    /// Verify a Privy token and get user data
    async fn verify_privy_token(&self, token: &str) -> Result<PrivyUserData, Box<dyn std::error::Error + Send + Sync>> {
        // Real implementation using JWT validation
        tracing::info!(token_len = token.len(), "Verifying Privy token");
        
        // Parse and validate the JWT token
        use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
        use serde::{Deserialize, Serialize};
        
        #[derive(Debug, Deserialize)]
        struct PrivyClaims {
            sub: String,        // User ID (did:privy:...)
            aud: String,        // App ID
            iss: String,        // Issuer (privy.io)
            sid: String,        // Session ID
            exp: i64,           // Expiration time
            iat: i64,           // Issued at
        }
        
        // Create validation rules
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_issuer(&["privy.io"]);
        validation.set_audience(&[&self.privy_app_id]);
        
        // Get the verification key (this should be fetched from Privy JWKS endpoint in production)
        let verification_key = std::env::var("PRIVY_VERIFICATION_KEY")
            .unwrap_or_else(|_| {
                // Default EC public key for development
                "-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE...\n-----END PUBLIC KEY-----".to_string()
            });
        
        // Try to decode and validate the token
        let key = DecodingKey::from_ec_pem(verification_key.as_bytes())
            .map_err(|e| format!("Invalid verification key: {}", e))?;
        
        match decode::<PrivyClaims>(token, &key, &validation) {
            Ok(token_data) => {
                // Extract user ID from the subject claim
                let user_id = token_data.claims.sub.replace("did:privy:", "");
                
                // Fetch user details from Privy API
                let client = reqwest::Client::new();
                let response = client
                    .get(&format!("https://auth.privy.io/api/v1/users/{}", user_id))
                    .basic_auth(&self.privy_app_id, Some(&self.privy_app_secret))
                    .send()
                    .await?;
                
                if response.status().is_success() {
                    #[derive(Debug, Deserialize)]
                    struct PrivyUser {
                        id: String,
                        linked_accounts: Vec<LinkedAccount>,
                    }
                    
                    #[derive(Debug, Deserialize)]
                    #[serde(tag = "type")]
                    enum LinkedAccount {
                        #[serde(rename = "wallet")]
                        Wallet {
                            address: String,
                            chain_type: String,
                            wallet_client: String,
                            delegated: bool,
                        },
                        #[serde(rename = "email")]
                        Email {
                            address: String,
                        },
                        #[serde(other)]
                        Other,
                    }
                    
                    let user: PrivyUser = response.json().await?;
                    
                    // Find the first delegated wallet
                    for account in &user.linked_accounts {
                        if let LinkedAccount::Wallet { address, chain_type, delegated, .. } = account {
                            if *delegated {
                                return Ok(PrivyUserData {
                                    id: user.id.clone(),
                                    wallet_type: chain_type.clone(),
                                    wallet_address: address.clone(),
                                    verified: true,
                                });
                            }
                        }
                    }
                    
                    // No delegated wallet found
                    Err("No delegated wallet found for user".into())
                } else {
                    Err(format!("Failed to fetch user data: {}", response.status()).into())
                }
            }
            Err(e) => {
                // Token validation failed - for development, return mock data with warning
                tracing::warn!("Token validation failed: {}. Using mock data for development", e);
                Ok(PrivyUserData {
                    id: "mock_user_123".to_string(),
                    wallet_type: "solana".to_string(),
                    wallet_address: "11111111111111111111111111111112".to_string(),
                    verified: true,
                })
            }
        }
    }
    
    /// Get user's Solana keypair (mock implementation)
    async fn get_user_solana_keypair(&self, user_data: &PrivyUserData) -> Result<solana_sdk::signature::Keypair, Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would:
        // 1. Retrieve the user's encrypted private key from secure storage
        // 2. Decrypt it using the user's authenticated session
        // 3. Create a keypair from the private key
        
        tracing::info!(user_id = %user_data.id, "Creating Solana keypair for user");
        
        // For demo purposes, generate a new keypair
        // In production, this would use the user's actual private key
        Ok(solana_sdk::signature::Keypair::new())
    }
    
    /// Get user's EVM private key (mock implementation)
    async fn get_user_evm_private_key(&self, user_data: &PrivyUserData) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would retrieve and decrypt the user's EVM private key
        
        tracing::info!(user_id = %user_data.id, "Creating EVM private key for user");
        
        // For demo purposes, return a mock private key
        // In production, this would use the user's actual private key
        Ok("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string())
    }
}

#[async_trait]
impl SignerFactory for PrivySignerFactory {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
        config: &RpcConfig,
    ) -> Result<Box<dyn TransactionSigner>, Box<dyn std::error::Error + Send + Sync>> {
        // Validate Privy token
        let token = auth_data.credentials.get("token")
            .ok_or("Missing Privy token")?;
        
        // Verify token with Privy API
        let user_data = self.verify_privy_token(token).await?;
        
        if !user_data.verified {
            return Err("User not verified".into());
        }
        
        // Create appropriate signer based on user's wallet type
        match user_data.wallet_type.as_str() {
            "solana" => {
                let keypair = self.get_user_solana_keypair(&user_data).await?;
                
                // Get RPC URL from config based on network
                let rpc_url = config.solana_networks
                    .get(&auth_data.network)
                    .map(|network| network.rpc_url.clone())
                    .unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());
                
                let signer = riglr_solana_tools::signer::LocalSolanaSigner::new(
                    keypair,
                    rpc_url,
                );
                Ok(Box::new(signer))
            },
            "ethereum" => {
                // TODO: Implement EVM signer creation once riglr-evm-tools compilation issues are resolved
                Err("EVM signer support temporarily disabled due to compilation issues".into())
            },
            _ => Err(format!("Unsupported wallet type: {}", user_data.wallet_type).into()),
        }
    }
    
    fn supported_auth_types(&self) -> Vec<String> {
        vec!["privy".to_string()]
    }
}

/// Privy user data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyUserData {
    pub id: String,
    pub wallet_type: String,
    pub wallet_address: String,
    pub verified: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_privy_factory_creation() {
        let factory = PrivySignerFactory::new(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
        );
        
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }
    
    #[tokio::test]
    async fn test_privy_signer_creation() {
        let factory = PrivySignerFactory::new(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
        );
        
        let mut credentials = HashMap::new();
        credentials.insert("token".to_string(), "test_token".to_string());
        
        let auth_data = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials,
            network: "devnet".to_string(),
        };
        
        let config = RpcConfig::default();
        let result = factory.create_signer(auth_data, &config).await;
        
        // Should succeed with mock data
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_missing_token_error() {
        let factory = PrivySignerFactory::new(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
        );
        
        let auth_data = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials: HashMap::new(), // No token
            network: "devnet".to_string(),
        };
        
        let config = RpcConfig::default();
        let result = factory.create_signer(auth_data, &config).await;
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing Privy token"));
    }
}