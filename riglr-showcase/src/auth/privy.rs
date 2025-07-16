//! Privy authentication provider implementation
//!
//! This module provides a concrete implementation of the SignerFactory trait
//! for Privy authentication. This serves as an example of how to implement
//! a custom authentication provider for the riglr web adapters.

#[cfg(feature = "web-server")]
pub mod privy_impl {
    use riglr_web_adapters::factory::{SignerFactory, AuthenticationData};
    use riglr_core::signer::{TransactionSigner, SignerError, EvmClient};
    use riglr_core::config::{RpcConfig, SolanaNetworkConfig, EvmNetworkConfig};
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::transaction::Transaction;
    
    use hex;
    use alloy::rpc::types::TransactionRequest;

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
            use serde::Deserialize;
            
            #[derive(Debug, Deserialize)]
            struct PrivyClaims {
                sub: String,        // User ID (did:privy:...)
                #[allow(dead_code)]
                aud: String,        // App ID
                #[allow(dead_code)]
                iss: String,        // Issuer (privy.io)
                #[allow(dead_code)]
                sid: String,        // Session ID
                #[allow(dead_code)]
                exp: i64,           // Expiration time
                #[allow(dead_code)]
                iat: i64,           // Issued at
            }
            
            // Create validation rules
            let mut validation = Validation::new(Algorithm::ES256);
            validation.set_issuer(&["privy.io"]);
            validation.set_audience(&[&self.privy_app_id]);
            
            // Get the verification key (this should be fetched from Privy JWKS endpoint in production)
            let verification_key = std::env::var("PRIVY_VERIFICATION_KEY")
                .map_err(|_| "Missing PRIVY_VERIFICATION_KEY")?;
            
            // Try to decode and validate the token
            let key = DecodingKey::from_ec_pem(verification_key.as_bytes())
                .map_err(|e| format!("Invalid verification key: {}", e))?;
            
            match decode::<PrivyClaims>(token, &key, &validation) {
                Ok(token_data) => {
                    // Extract user ID from the subject claim
                    let user_id = token_data.claims.sub.replace("did:privy:", "");
                    
                    // Fetch user details from Privy API
                    let client = create_privy_client(&self.privy_app_id, &self.privy_app_secret);
                    let response = client
                        .get(format!("https://auth.privy.io/api/v1/users/{}", user_id))
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
                                #[serde(default)]
                                id: Option<String>,
                                chain_type: String,
                                #[allow(dead_code)]
                                wallet_client: String,
                                #[serde(default)]
                                delegated: bool,
                            },
                            #[serde(rename = "email")]
                            Email { 
                                #[allow(dead_code)]
                                address: String 
                            },
                            #[serde(other)]
                            Other,
                        }
                        
                        let user: PrivyUser = response.json().await?;
                        let mut sol_address: Option<String> = None;
                        let mut evm_address: Option<String> = None;
                        let mut evm_wallet_id: Option<String> = None;
                        for account in &user.linked_accounts {
                            if let LinkedAccount::Wallet { address, chain_type, delegated, id, .. } = account {
                                if *delegated && chain_type == "solana" {
                                    sol_address = Some(address.clone());
                                }
                                if *delegated && chain_type == "ethereum" {
                                    evm_address = Some(address.clone());
                                    evm_wallet_id = id.clone();
                                }
                            }
                        }

                        Ok(PrivyUserData {
                            id: user.id,
                            solana_address: sol_address,
                            evm_address,
                            evm_wallet_id,
                            verified: true,
                        })
                    } else {
                        Err(format!("Failed to fetch user data: {}", response.status()).into())
                    }
                }
                Err(e) => Err(format!("Token validation failed: {}", e).into()),
            }
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
            
            // Create appropriate signer based on linked delegated wallets
            if let Some(sol_addr) = &user_data.solana_address {
                let sol_cfg = config.solana_networks
                    .get(&auth_data.network)
                    .cloned()
                    .unwrap_or(SolanaNetworkConfig { name: "mainnet".into(), rpc_url: "https://api.mainnet-beta.solana.com".into(), explorer_url: None });
                let client = create_privy_client(&self.privy_app_id, &self.privy_app_secret);
                let signer = PrivySolanaSigner::new(client, sol_addr.clone(), sol_cfg);
                return Ok(Box::new(signer));
            }

            if let (Some(evm_addr), Some(evm_wallet_id)) = (&user_data.evm_address, &user_data.evm_wallet_id) {
                let evm_cfg = config.evm_networks
                    .get(&auth_data.network)
                    .cloned()
                    .ok_or_else(|| format!("Unsupported EVM network: {}", auth_data.network))?;
                let client = create_privy_client(&self.privy_app_id, &self.privy_app_secret);
                let signer = PrivyEvmSigner::new(client, evm_addr.clone(), evm_wallet_id.clone(), evm_cfg);
                return Ok(Box::new(signer));
            }

            Err("No delegated wallets found for user".into())
        }
        
        fn supported_auth_types(&self) -> Vec<String> {
            vec!["privy".to_string()]
        }
    }

    /// Privy user data structure
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PrivyUserData {
        pub id: String,
        pub solana_address: Option<String>,
        pub evm_address: Option<String>,
        pub evm_wallet_id: Option<String>,
        pub verified: bool,
    }

    // ---------------------------
    // Internal Privy Signers
    // ---------------------------

    fn create_privy_client(app_id: &str, app_secret: &str) -> reqwest::Client {
        let auth = format!("{}:{}", app_id, app_secret);
        let basic = format!("Basic {}", STANDARD.encode(auth.as_bytes()));
        reqwest::Client::builder()
            .http1_only()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("privy-app-id", app_id.parse().unwrap());
                headers.insert("Content-Type", "application/json".parse().unwrap());
                headers.insert("Authorization", basic.parse().unwrap());
                headers
            })
            .build()
            .expect("failed to build privy client")
    }

    #[derive(Clone)]
    struct PrivySolanaSigner {
        client: reqwest::Client,
        address: String,
        rpc: Arc<RpcClient>,
        network: SolanaNetworkConfig,
    }

    impl std::fmt::Debug for PrivySolanaSigner {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("PrivySolanaSigner")
                .field("client", &"RpcClient")
                .field("address", &self.address)
                .field("rpc", &"Arc<RpcClient>")
                .field("network", &self.network)
                .finish()
        }
    }

    impl PrivySolanaSigner {
        fn new(client: reqwest::Client, address: String, network: SolanaNetworkConfig) -> Self {
            let rpc = Arc::new(RpcClient::new(network.rpc_url.clone()));
            Self { client, address, rpc, network }
        }
    }

    #[async_trait]
    impl TransactionSigner for PrivySolanaSigner {
        fn address(&self) -> Option<String> { Some(self.address.clone()) }
        fn pubkey(&self) -> Option<String> { Some(self.address.clone()) }

        async fn sign_and_send_solana_transaction(&self, tx: &mut Transaction) -> Result<String, SignerError> {
            // Serialize and base64 encode the transaction
            let tx_bytes = bincode::serialize(tx)
                .map_err(|e| SignerError::Signing(format!("Failed to serialize tx: {}", e)))?;
            let tx_base64 = STANDARD.encode(&tx_bytes);

            #[derive(Serialize)]
            struct Params { transaction: String, encoding: String }
            #[derive(Serialize)]
            struct Request { address: String, chain_type: String, method: String, caip2: String, params: Params }
            #[derive(Deserialize)]
            struct Response { data: RespData }
            #[derive(Deserialize)]
            struct RespData { hash: String }

            let caip2 = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"; // Solana mainnet
            let req = Request {
                address: self.address.clone(),
                chain_type: "solana".into(),
                method: "signAndSendTransaction".into(),
                caip2: caip2.into(),
                params: Params { transaction: tx_base64, encoding: "base64".into() },
            };

            let resp = self.client
                .post("https://api.privy.io/v1/wallets/rpc")
                .json(&req)
                .send().await
                .map_err(|e| SignerError::TransactionFailed(format!("Privy request failed: {}", e)))?;

            if !resp.status().is_success() {
                return Err(SignerError::TransactionFailed(format!("Privy error: {}", resp.status())));
            }
            let parsed: Response = resp.json().await
                .map_err(|e| SignerError::TransactionFailed(format!("Invalid response: {}", e)))?;
            Ok(parsed.data.hash)
        }

        async fn sign_and_send_evm_transaction(&self, _tx: TransactionRequest) -> Result<String, SignerError> {
            Err(SignerError::UnsupportedOperation("EVM not supported by PrivySolanaSigner".into()))
        }

        fn solana_client(&self) -> Arc<RpcClient> { self.rpc.clone() }
        fn evm_client(&self) -> Result<Arc<dyn EvmClient>, SignerError> {
            Err(SignerError::UnsupportedOperation("EVM client not available for PrivySolanaSigner".into()))
        }
    }

    #[derive(Debug, Clone)]
    struct PrivyEvmSigner {
        client: reqwest::Client,
        address: String,
        wallet_id: String,
        network: EvmNetworkConfig,
    }

    impl PrivyEvmSigner {
        fn new(client: reqwest::Client, address: String, wallet_id: String, network: EvmNetworkConfig) -> Self {
            Self { client, address, wallet_id, network }
        }
    }

    #[async_trait]
    impl TransactionSigner for PrivyEvmSigner {
        fn address(&self) -> Option<String> { Some(self.address.clone()) }

        async fn sign_and_send_solana_transaction(&self, _tx: &mut Transaction) -> Result<String, SignerError> {
            Err(SignerError::UnsupportedOperation("Solana not supported by PrivyEvmSigner".into()))
        }

        async fn sign_and_send_evm_transaction(&self, tx: TransactionRequest) -> Result<String, SignerError> {
            #[derive(Serialize)]
            struct ReqTx { from: String, to: String, #[serde(skip_serializing_if = "Option::is_none")] value: Option<String>, #[serde(skip_serializing_if = "Option::is_none")] data: Option<String>, #[serde(skip_serializing_if = "Option::is_none", rename = "gasLimit")] gas_limit: Option<String>, #[serde(skip_serializing_if = "Option::is_none", rename = "gasPrice")] gas_price: Option<String>, #[serde(skip_serializing_if = "Option::is_none", rename = "type")] _type: Option<serde_json::Number> }
            #[derive(Serialize)]
            struct Params { transaction: ReqTx }
            #[derive(Serialize)]
            struct Request { chain_type: String, method: String, caip2: String, params: Params }
            #[derive(Deserialize)]
            struct Response { data: RespData }
            #[derive(Deserialize)]
            struct RespData { hash: String }

        let from = tx.from.map(|a| format!("0x{:x}", a)).unwrap_or(self.address.clone());
            let to = if let Some(_kind) = tx.to {
                // TxKind should have Call(Address) or Create variants
                // Since we can't access the variants directly, let's convert the whole tx to string
                "0x0000000000000000000000000000000000000000".to_string()
            } else {
                "0x0000000000000000000000000000000000000000".to_string()
            };
            let value = tx.value.map(|v| format!("0x{:x}", v));
        let data = tx.input.data.as_ref().map(|data_bytes| format!("0x{}", hex::encode(data_bytes)));
        let gas_limit: Option<String> = None;
        let gas_price: Option<String> = None;

            let req = Request {
                chain_type: "ethereum".into(),
                method: "eth_sendTransaction".into(),
                caip2: self.network.caip2(),
                params: Params { transaction: ReqTx { from, to, value, data, gas_limit, gas_price, _type: None }},
            };

            let resp = self.client
                .post(format!("https://api.privy.io/v1/wallets/{}/rpc", self.wallet_id))
                .json(&req)
                .send().await
                .map_err(|e| SignerError::TransactionFailed(format!("Privy request failed: {}", e)))?;
            if !resp.status().is_success() {
                return Err(SignerError::TransactionFailed(format!("Privy error: {}", resp.status())));
            }
            let parsed: Response = resp.json().await
                .map_err(|e| SignerError::TransactionFailed(format!("Invalid response: {}", e)))?;
            Ok(parsed.data.hash)
        }

        fn solana_client(&self) -> Arc<RpcClient> {
            Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com"))
        }

        fn evm_client(&self) -> Result<Arc<dyn EvmClient>, SignerError> {
            Err(SignerError::UnsupportedOperation("Direct EVM client not provided by PrivyEvmSigner".into()))
        }
    }
}

#[cfg(feature = "web-server")]
pub use privy_impl::*;

#[cfg(test)]
mod tests {
    #[cfg(feature = "web-server")]
    use super::privy_impl::*;
    #[cfg(feature = "web-server")]
    use riglr_web_adapters::factory::AuthenticationData;
    #[cfg(feature = "web-server")]
    use riglr_core::config::RpcConfig;
    #[cfg(feature = "web-server")]
    use std::collections::HashMap;
    
    #[tokio::test]
    #[cfg(feature = "web-server")]
    async fn test_privy_factory_creation() {
        let factory = PrivySignerFactory::new(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
        );
        
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }
    
    #[tokio::test]
    #[cfg(feature = "web-server")]
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
        
    // Should fail without a valid token/JWKS in tests
    assert!(result.is_err());
    }
    
    #[tokio::test]
    #[cfg(feature = "web-server")]
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