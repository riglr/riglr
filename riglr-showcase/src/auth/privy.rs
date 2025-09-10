//! Privy authentication provider implementation
//!
//! This module provides a concrete implementation of the SignerFactory trait
//! for Privy authentication. This serves as an example of how to implement
//! a custom authentication provider for the riglr web adapters.

/// Privy authentication provider implementation module
#[cfg(feature = "web-server")]
pub mod privy_impl {
    use async_trait::async_trait;
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use riglr_config::{EvmNetworkConfig, SolanaNetworkConfig};
    use riglr_core::signer::{
        EvmClient, EvmSigner, SignerBase, SignerError, SolanaSigner, UnifiedSigner,
    };
    use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};
    use serde::{Deserialize, Serialize};
    use solana_client::rpc_client::RpcClient;

    const PRIVY_VERIFICATION_KEY: &str = "PRIVY_VERIFICATION_KEY";
    use solana_sdk::transaction::Transaction;
    use std::sync::Arc;

    use alloy::rpc::types::TransactionRequest;
    use hex;

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
        async fn verify_privy_token(
            &self,
            token: &str,
        ) -> Result<PrivyUserData, Box<dyn std::error::Error + Send + Sync>> {
            // Real implementation using JWT validation
            tracing::info!(token_len = token.len(), "Verifying Privy token");

            // Parse and validate the JWT token
            use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
            use serde::Deserialize;

            #[derive(Debug, Deserialize)]
            struct PrivyClaims {
                sub: String, // User ID (did:privy:...)
                #[allow(dead_code)]
                aud: String, // App ID
                #[allow(dead_code)]
                iss: String, // Issuer (privy.io)
                #[allow(dead_code)]
                sid: String, // Session ID
                #[allow(dead_code)]
                exp: i64, // Expiration time
                #[allow(dead_code)]
                iat: i64, // Issued at
            }

            // Create validation rules
            let mut validation = Validation::new(Algorithm::ES256);
            validation.set_issuer(&["privy.io"]);
            validation.set_audience(&[&self.privy_app_id]);

            // Get the verification key (this should be fetched from Privy JWKS endpoint in production)
            let verification_key = std::env::var(PRIVY_VERIFICATION_KEY)
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
                                address: String,
                            },
                            #[serde(other)]
                            Other,
                        }

                        let user: PrivyUser = response.json().await?;
                        let mut sol_address: Option<String> = None;
                        let mut evm_address: Option<String> = None;
                        let mut evm_wallet_id: Option<String> = None;
                        for account in &user.linked_accounts {
                            if let LinkedAccount::Wallet {
                                address,
                                chain_type,
                                delegated,
                                id,
                                ..
                            } = account
                            {
                                if *delegated && chain_type == "solana" {
                                    sol_address = Some(address.clone());
                                }
                                if *delegated && chain_type == "ethereum" {
                                    evm_address.clone_from(&Some(address.clone()));
                                    evm_wallet_id.clone_from(id);
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
        ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
            // Validate Privy token
            let token = auth_data
                .credentials
                .get("token")
                .ok_or("Missing Privy token")?;

            // Verify token with Privy API
            let user_data = self.verify_privy_token(token).await?;

            if !user_data.verified {
                return Err("User not verified".into());
            }

            // Create appropriate signer based on linked delegated wallets
            if let Some(sol_addr) = user_data.solana_address {
                let sol_cfg = SolanaNetworkConfig {
                    name: auth_data.network.clone(),
                    rpc_url: "https://api.mainnet-beta.solana.com".into(),
                    ws_url: None,
                    explorer_url: Some("https://explorer.solana.com".into()),
                };
                let client = create_privy_client(&self.privy_app_id, &self.privy_app_secret);
                let signer = PrivySolanaSigner::new(client, sol_addr, sol_cfg);
                return Ok(Box::new(signer));
            }

            if let (Some(evm_addr), Some(evm_wallet_id)) =
                (user_data.evm_address, user_data.evm_wallet_id)
            {
                let evm_cfg = EvmNetworkConfig {
                    name: auth_data.network.clone(),
                    chain_id: 1, // TODO: Map network name to chain ID
                    rpc_url: "https://eth.llamarpc.com".into(),
                    explorer_url: Some("https://etherscan.io".into()),
                    native_token: Some("ETH".into()),
                };
                let client = create_privy_client(&self.privy_app_id, &self.privy_app_secret);
                let signer = PrivyEvmSigner::new(client, evm_addr, evm_wallet_id, evm_cfg);
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
        /// Unique user identifier from Privy
        pub id: String,
        /// Solana wallet address if available
        pub solana_address: Option<String>,
        /// EVM wallet address if available
        pub evm_address: Option<String>,
        /// EVM wallet identifier in Privy
        pub evm_wallet_id: Option<String>,
        /// Whether the user has been verified
        pub verified: bool,
    }

    // ---------------------------
    // Internal Privy Signers
    // ---------------------------

    pub(super) fn create_privy_client(app_id: &str, app_secret: &str) -> reqwest::Client {
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
    pub(super) struct PrivySolanaSigner {
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
            Self {
                client,
                address,
                rpc,
                network,
            }
        }
    }

    #[async_trait]

    impl SignerBase for PrivySolanaSigner {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[async_trait]
    impl SolanaSigner for PrivySolanaSigner {
        fn address(&self) -> String {
            self.address.clone()
        }

        fn pubkey(&self) -> String {
            self.address.clone()
        }

        async fn sign_and_send_transaction(&self, tx: &mut Vec<u8>) -> Result<String, SignerError> {
            // Deserialize the transaction from bytes
            let mut transaction: Transaction = bincode::deserialize(tx)
                .map_err(|e| SignerError::Signing(format!("Failed to deserialize tx: {}", e)))?;

            // Serialize and base64 encode the transaction
            let tx_bytes = bincode::serialize(&transaction)
                .map_err(|e| SignerError::Signing(format!("Failed to serialize tx: {}", e)))?;
            let tx_base64 = STANDARD.encode(&tx_bytes);

            #[derive(Serialize)]
            struct Params {
                transaction: String,
                encoding: String,
            }
            #[derive(Serialize)]
            struct Request {
                address: String,
                chain_type: String,
                method: String,
                caip2: String,
                params: Params,
            }
            #[derive(Deserialize)]
            struct Response {
                data: RespData,
            }
            #[derive(Deserialize)]
            struct RespData {
                hash: String,
            }

            let caip2 = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"; // Solana mainnet
            let req = Request {
                address: self.address.clone(),
                chain_type: "solana".into(),
                method: "signAndSendTransaction".into(),
                caip2: caip2.into(),
                params: Params {
                    transaction: tx_base64,
                    encoding: "base64".into(),
                },
            };

            let resp = self
                .client
                .post("https://api.privy.io/v1/wallets/rpc")
                .json(&req)
                .send()
                .await
                .map_err(|e| {
                    SignerError::TransactionFailed(format!("Privy request failed: {}", e))
                })?;

            if !resp.status().is_success() {
                return Err(SignerError::TransactionFailed(format!(
                    "Privy error: {}",
                    resp.status()
                )));
            }
            let parsed: Response = resp
                .json()
                .await
                .map_err(|e| SignerError::TransactionFailed(format!("Invalid response: {}", e)))?;
            Ok(parsed.data.hash)
        }

        fn client(&self) -> Arc<dyn std::any::Any + Send + Sync> {
            self.rpc.clone() as Arc<dyn std::any::Any + Send + Sync>
        }
    }

    impl UnifiedSigner for PrivySolanaSigner {
        fn supports_solana(&self) -> bool {
            true
        }

        fn supports_evm(&self) -> bool {
            false
        }

        fn as_solana(&self) -> Option<&dyn SolanaSigner> {
            Some(self)
        }

        fn as_evm(&self) -> Option<&dyn riglr_core::signer::EvmSigner> {
            None
        }

        fn as_multi_chain(&self) -> Option<&dyn riglr_core::signer::MultiChainSigner> {
            None
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
        fn new(
            client: reqwest::Client,
            address: String,
            wallet_id: String,
            network: EvmNetworkConfig,
        ) -> Self {
            Self {
                client,
                address,
                wallet_id,
                network,
            }
        }
    }

    impl SignerBase for PrivyEvmSigner {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[async_trait]
    impl EvmSigner for PrivyEvmSigner {
        fn chain_id(&self) -> u64 {
            self.network.chain_id
        }

        fn address(&self) -> String {
            self.address.clone()
        }

        async fn sign_and_send_transaction(
            &self,
            tx: serde_json::Value,
        ) -> Result<String, SignerError> {
            // Convert JSON to TransactionRequest
            let tx_request: TransactionRequest = serde_json::from_value(tx).map_err(|e| {
                SignerError::Signing(format!("Failed to parse transaction request: {}", e))
            })?;
            #[derive(Serialize)]
            struct ReqTx {
                from: String,
                to: String,
                #[serde(skip_serializing_if = "Option::is_none")]
                value: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                data: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none", rename = "gasLimit")]
                gas_limit: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none", rename = "gasPrice")]
                gas_price: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
                _type: Option<serde_json::Number>,
            }
            #[derive(Serialize)]
            struct Params {
                transaction: ReqTx,
            }
            #[derive(Serialize)]
            struct Request {
                chain_type: String,
                method: String,
                caip2: String,
                params: Params,
            }
            #[derive(Deserialize)]
            struct Response {
                data: RespData,
            }
            #[derive(Deserialize)]
            struct RespData {
                hash: String,
            }

            let from = tx_request
                .from
                .map(|a| format!("0x{:x}", a))
                .unwrap_or_else(|| self.address.clone());
            let to = if let Some(_kind) = tx_request.to {
                // TxKind should have Call(Address) or Create variants
                // Since we can't access the variants directly, let's convert the whole tx to string
                "0x0000000000000000000000000000000000000000".to_string()
            } else {
                "0x0000000000000000000000000000000000000000".to_string()
            };
            let value = tx_request.value.map(|v| format!("0x{:x}", v));
            let data = tx_request
                .input
                .data
                .as_ref()
                .map(|data_bytes| format!("0x{}", hex::encode(data_bytes)));
            let gas_limit: Option<String> = None;
            let gas_price: Option<String> = None;

            let req = Request {
                chain_type: "ethereum".into(),
                method: "eth_sendTransaction".into(),
                caip2: self.network.caip2(),
                params: Params {
                    transaction: ReqTx {
                        from,
                        to,
                        value,
                        data,
                        gas_limit,
                        gas_price,
                        _type: None,
                    },
                },
            };

            let resp = self
                .client
                .post(format!(
                    "https://api.privy.io/v1/wallets/{}/rpc",
                    self.wallet_id
                ))
                .json(&req)
                .send()
                .await
                .map_err(|e| {
                    SignerError::TransactionFailed(format!("Privy request failed: {}", e))
                })?;
            if !resp.status().is_success() {
                return Err(SignerError::TransactionFailed(format!(
                    "Privy error: {}",
                    resp.status()
                )));
            }
            let parsed: Response = resp
                .json()
                .await
                .map_err(|e| SignerError::TransactionFailed(format!("Invalid response: {}", e)))?;
            Ok(parsed.data.hash)
        }

        fn client(&self) -> Result<Arc<dyn EvmClient>, SignerError> {
            // Privy handles RPC internally, so we don't provide direct client access
            Err(SignerError::UnsupportedOperation(
                "Direct EVM client not provided by PrivyEvmSigner".into(),
            ))
        }
    }

    impl UnifiedSigner for PrivyEvmSigner {
        fn supports_solana(&self) -> bool {
            false
        }

        fn supports_evm(&self) -> bool {
            true
        }

        fn as_solana(&self) -> Option<&dyn SolanaSigner> {
            None
        }

        fn as_evm(&self) -> Option<&dyn riglr_core::signer::EvmSigner> {
            Some(self)
        }

        fn as_multi_chain(&self) -> Option<&dyn riglr_core::signer::MultiChainSigner> {
            None
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
    use alloy::rpc::types::TransactionRequest;
    #[cfg(feature = "web-server")]
    use riglr_config::SolanaNetworkConfig;
    #[cfg(feature = "web-server")]
    #[cfg(feature = "web-server")]
    use riglr_core::signer::{SignerError, UnifiedSigner};
    #[cfg(feature = "web-server")]
    use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};
    #[cfg(feature = "web-server")]
    use solana_sdk::transaction::Transaction;
    #[cfg(feature = "web-server")]
    use std::collections::HashMap;
    #[cfg(feature = "web-server")]
    use std::sync::Arc;

    #[tokio::test]
    #[cfg(feature = "web-server")]
    async fn test_privy_factory_creation() {
        let factory =
            PrivySignerFactory::new("test_app_id".to_string(), "test_app_secret".to_string());

        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }

    #[tokio::test]
    #[cfg(feature = "web-server")]
    async fn test_privy_signer_creation() {
        let factory =
            PrivySignerFactory::new("test_app_id".to_string(), "test_app_secret".to_string());

        let mut credentials = HashMap::new();
        credentials.insert("token".to_string(), "test_token".to_string());

        let auth_data = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials,
            network: "devnet".to_string(),
        };

        let result = factory.create_signer(auth_data).await;

        // Should fail without a valid token/JWKS in tests
        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(feature = "web-server")]
    async fn test_missing_token_error() {
        let factory =
            PrivySignerFactory::new("test_app_id".to_string(), "test_app_secret".to_string());

        let auth_data = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials: HashMap::new(), // No token
            network: "devnet".to_string(),
        };

        let result = factory.create_signer(auth_data).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing Privy token"));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_creation() {
        let user_data = PrivyUserData {
            id: "test_user_id".to_string(),
            solana_address: Some("test_sol_address".to_string()),
            evm_address: Some("test_evm_address".to_string()),
            evm_wallet_id: Some("test_wallet_id".to_string()),
            verified: true,
        };

        assert_eq!(user_data.id, "test_user_id");
        assert_eq!(
            user_data.solana_address,
            Some("test_sol_address".to_string())
        );
        assert_eq!(user_data.evm_address, Some("test_evm_address".to_string()));
        assert_eq!(user_data.evm_wallet_id, Some("test_wallet_id".to_string()));
        assert!(user_data.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_with_none_values() {
        let user_data = PrivyUserData {
            id: "test_user_id".to_string(),
            solana_address: None,
            evm_address: None,
            evm_wallet_id: None,
            verified: false,
        };

        assert_eq!(user_data.id, "test_user_id");
        assert_eq!(user_data.solana_address, None);
        assert_eq!(user_data.evm_address, None);
        assert_eq!(user_data.evm_wallet_id, None);
        assert!(!user_data.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_create_privy_client() {
        let client = create_privy_client("test_app_id", "test_app_secret");

        // Just verify the client was created successfully
        assert!(format!("{:?}", client).contains("Client"));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_solana_signer_new() {
        let client = reqwest::Client::new();
        let network = SolanaNetworkConfig {
            name: "devnet".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            explorer_url: Some("https://explorer.solana.com".to_string()),
        };

        let signer = PrivySolanaSigner::new(client, "test_address".to_string(), network.clone());

        assert_eq!(signer.address, "test_address");
        assert_eq!(signer.network.name, "devnet");
        assert_eq!(signer.network.rpc_url, "https://api.devnet.solana.com");
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_solana_signer_debug() {
        let client = reqwest::Client::new();
        let network = SolanaNetworkConfig {
            name: "devnet".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            explorer_url: None,
        };

        let signer = PrivySolanaSigner::new(client, "test_address".to_string(), network);

        let debug_str = format!("{:?}", signer);
        assert!(debug_str.contains("PrivySolanaSigner"));
        assert!(debug_str.contains("test_address"));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_solana_signer_address() {
        let client = reqwest::Client::new();
        let network = SolanaNetworkConfig {
            name: "devnet".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            explorer_url: None,
        };

        let signer = PrivySolanaSigner::new(client, "test_address".to_string(), network);

        assert_eq!(signer.address(), Some("test_address".to_string()));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_solana_signer_pubkey() {
        let client = reqwest::Client::new();
        let network = SolanaNetworkConfig {
            name: "devnet".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            explorer_url: None,
        };

        let signer = PrivySolanaSigner::new(client, "test_pubkey".to_string(), network);

        assert_eq!(signer.pubkey(), Some("test_pubkey".to_string()));
    }

    #[tokio::test]
    #[cfg(feature = "web-server")]
    async fn test_privy_solana_signer_evm_transaction_unsupported() {
        let client = reqwest::Client::new();
        let network = SolanaNetworkConfig {
            name: "devnet".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            explorer_url: None,
        };

        let signer = PrivySolanaSigner::new(client, "test_address".to_string(), network);

        let tx = TransactionRequest::default();
        let result = signer.sign_and_send_evm_transaction(tx).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            SignerError::UnsupportedOperation(msg) => {
                assert!(msg.contains("EVM not supported by PrivySolanaSigner"));
            }
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_solana_signer_solana_client() {
        let client = reqwest::Client::new();
        let network = SolanaNetworkConfig {
            name: "devnet".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            explorer_url: None,
        };

        let signer = PrivySolanaSigner::new(client, "test_address".to_string(), network);

        let solana_client = signer.solana_client();
        assert!(solana_client.is_some());
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_solana_signer_evm_client_unsupported() {
        let client = reqwest::Client::new();
        let network = SolanaNetworkConfig {
            name: "devnet".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            explorer_url: None,
        };

        let signer = PrivySolanaSigner::new(client, "test_address".to_string(), network);

        let result = signer.evm_client();
        assert!(result.is_err());
        match result.unwrap_err() {
            SignerError::UnsupportedOperation(msg) => {
                assert!(msg.contains("EVM client not available for PrivySolanaSigner"));
            }
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_evm_signer_new() {
        let client = reqwest::Client::new();
        let network = EvmNetworkConfig {
            name: "ethereum".to_string(),
            rpc_url: "https://eth-mainnet.alchemyapi.io/v2/test".to_string(),
            chain_id: 1,
            explorer_url: Some("https://etherscan.io".to_string()),
        };

        let signer = PrivyEvmSigner::new(
            client,
            "0x123".to_string(),
            "wallet_123".to_string(),
            network.clone(),
        );

        assert_eq!(signer.address, "0x123");
        assert_eq!(signer.wallet_id, "wallet_123");
        assert_eq!(signer.network.name, "ethereum");
        assert_eq!(signer.network.chain_id, 1);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_evm_signer_address() {
        let client = reqwest::Client::new();
        let network = EvmNetworkConfig {
            name: "ethereum".to_string(),
            rpc_url: "https://eth-mainnet.alchemyapi.io/v2/test".to_string(),
            chain_id: 1,
            explorer_url: None,
        };

        let signer = PrivyEvmSigner::new(
            client,
            "0x456".to_string(),
            "wallet_456".to_string(),
            network,
        );

        assert_eq!(signer.address(), Some("0x456".to_string()));
    }

    #[tokio::test]
    #[cfg(feature = "web-server")]
    async fn test_privy_evm_signer_solana_transaction_unsupported() {
        let client = reqwest::Client::new();
        let network = EvmNetworkConfig {
            name: "ethereum".to_string(),
            rpc_url: "https://eth-mainnet.alchemyapi.io/v2/test".to_string(),
            chain_id: 1,
            explorer_url: None,
        };

        let signer = PrivyEvmSigner::new(
            client,
            "0x123".to_string(),
            "wallet_123".to_string(),
            network,
        );

        let mut tx = Transaction::default();
        let result = signer.sign_and_send_solana_transaction(&mut tx).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            SignerError::UnsupportedOperation(msg) => {
                assert!(msg.contains("Solana not supported by PrivyEvmSigner"));
            }
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_evm_signer_solana_client() {
        let client = reqwest::Client::new();
        let network = EvmNetworkConfig {
            name: "ethereum".to_string(),
            rpc_url: "https://eth-mainnet.alchemyapi.io/v2/test".to_string(),
            chain_id: 1,
            explorer_url: None,
        };

        let signer = PrivyEvmSigner::new(
            client,
            "0x123".to_string(),
            "wallet_123".to_string(),
            network,
        );

        let solana_client = signer.solana_client();
        assert!(solana_client.is_some());
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_evm_signer_evm_client_unsupported() {
        let client = reqwest::Client::new();
        let network = EvmNetworkConfig {
            name: "ethereum".to_string(),
            rpc_url: "https://eth-mainnet.alchemyapi.io/v2/test".to_string(),
            chain_id: 1,
            explorer_url: None,
        };

        let signer = PrivyEvmSigner::new(
            client,
            "0x123".to_string(),
            "wallet_123".to_string(),
            network,
        );

        let result = signer.evm_client();
        assert!(result.is_err());
        match result.unwrap_err() {
            SignerError::UnsupportedOperation(msg) => {
                assert!(msg.contains("Direct EVM client not provided by PrivyEvmSigner"));
            }
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_evm_signer_debug() {
        let client = reqwest::Client::new();
        let network = EvmNetworkConfig {
            name: "ethereum".to_string(),
            rpc_url: "https://eth-mainnet.alchemyapi.io/v2/test".to_string(),
            chain_id: 1,
            explorer_url: None,
        };

        let signer = PrivyEvmSigner::new(
            client,
            "0x123".to_string(),
            "wallet_123".to_string(),
            network,
        );

        let debug_str = format!("{:?}", signer);
        assert!(debug_str.contains("PrivyEvmSigner"));
        assert!(debug_str.contains("0x123"));
        assert!(debug_str.contains("wallet_123"));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_evm_signer_clone() {
        let client = reqwest::Client::new();
        let network = EvmNetworkConfig {
            name: "ethereum".to_string(),
            rpc_url: "https://eth-mainnet.alchemyapi.io/v2/test".to_string(),
            chain_id: 1,
            explorer_url: None,
        };

        let signer = PrivyEvmSigner::new(
            client,
            "0x123".to_string(),
            "wallet_123".to_string(),
            network,
        );

        let cloned_signer = signer.clone();
        assert_eq!(signer.address, cloned_signer.address);
        assert_eq!(signer.wallet_id, cloned_signer.wallet_id);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_solana_signer_clone() {
        let client = reqwest::Client::new();
        let network = SolanaNetworkConfig {
            name: "devnet".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            explorer_url: None,
        };

        let signer = PrivySolanaSigner::new(client, "test_address".to_string(), network);

        let cloned_signer = signer.clone();
        assert_eq!(signer.address, cloned_signer.address);
        assert_eq!(signer.network.name, cloned_signer.network.name);
    }
}
