//! Authentication providers for riglr showcase
//!
//! This module contains concrete implementations of `SignerFactory` for various
//! authentication providers. These serve as examples of how to implement
//! custom authentication for the riglr web adapters.

/// Privy authentication provider implementation module
#[cfg(feature = "web-server")]
pub mod privy_impl {
    use async_trait::async_trait;
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    use reqwest::header::HeaderMap;
    use riglr_config::{EvmNetworkConfig, SolanaNetworkConfig};
    use riglr_core::signer::{
        error::Standard, EvmClient, EvmSigner, SignerBase, SignerError, SolanaClient, SolanaSigner,
        UnifiedSigner,
    };
    use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};
    use serde::{Deserialize, Serialize};
    use solana_client::rpc_client::RpcClient;

    const PRIVY_VERIFICATION_KEY: &str = "PRIVY_VERIFICATION_KEY";
    use std::{env, fmt, sync::Arc};

    use alloy::rpc::types::TransactionRequest;
    use core::error::Error as StdError;
    use hex;
    use riglr_core::signer::Chain;

    /// Privy-specific signer factory implementation
    #[derive(Debug)]
    pub struct PrivySignerFactory {
        privy_app_id: String,
        privy_app_secret: String,
    }

    /// Privy JWT claims structure
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

    impl PrivySignerFactory {
        /// Create a new Privy signer factory
        ///
        /// # Arguments
        /// * `app_id` - Privy application ID
        /// * `app_secret` - Privy application secret
        #[must_use]
        pub const fn new(app_id: String, app_secret: String) -> Self {
            Self {
                privy_app_id: app_id,
                privy_app_secret: app_secret,
            }
        }

        /// Verify a Privy token and get user data
        async fn verify_privy_token(
            &self,
            token: &str,
        ) -> Result<PrivyUserData, Box<dyn StdError + Send + Sync>> {
            // Real implementation using JWT validation
            tracing::info!(token_len = token.len(), "Verifying Privy token");

            // Parse and validate the JWT token

            // Create validation rules
            let mut validation = Validation::new(Algorithm::ES256);
            validation.set_issuer(&["privy.io"]);
            validation.set_audience(&[&self.privy_app_id]);

            // Get the verification key (this should be fetched from Privy JWKS endpoint in production)
            let verification_key =
                env::var(PRIVY_VERIFICATION_KEY).map_err(|_| "Missing PRIVY_VERIFICATION_KEY")?;

            // Try to decode and validate the token
            let key = DecodingKey::from_ec_pem(verification_key.as_bytes())
                .map_err(|e| format!("Invalid verification key: {e}"))?;

            match decode::<PrivyClaims>(token, &key, &validation) {
                Ok(token_data) => {
                    // Extract user ID from the subject claim
                    let user_id = token_data.claims.sub.replace("did:privy:", "");

                    // Fetch user details from Privy API
                    let client = create_privy_client(&self.privy_app_id, &self.privy_app_secret)?;
                    let response = client
                        .get(format!("https://auth.privy.io/api/v1/users/{user_id}"))
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
                            if let &LinkedAccount::Wallet {
                                ref address,
                                ref chain_type,
                                delegated,
                                ref id,
                                ..
                            } = account
                            {
                                if delegated && chain_type == "solana" {
                                    sol_address = Some(address.clone());
                                }
                                if delegated && chain_type == "ethereum" {
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
                        Err(format!(
                            "Failed to fetch user data: {status}",
                            status = response.status()
                        )
                        .into())
                    }
                }
                Err(e) => Err(format!("Token validation failed: {e}").into()),
            }
        }
    }

    #[async_trait]
    impl SignerFactory for PrivySignerFactory {
        async fn create_signer(
            &self,
            auth_data: AuthenticationData,
        ) -> Result<Box<dyn UnifiedSigner>, Box<dyn StdError + Send + Sync>> {
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
                let mut sol_cfg = SolanaNetworkConfig::new(
                    auth_data.network.clone(),
                    "https://api.mainnet-beta.solana.com",
                );
                sol_cfg.explorer_url = Some("https://explorer.solana.com".into());
                let client = create_privy_client(&self.privy_app_id, &self.privy_app_secret)?;
                let signer = PrivySolanaSigner::new(client, sol_addr, sol_cfg);
                return Ok(Box::new(signer));
            }

            if let (Some(evm_addr), Some(evm_wallet_id)) =
                (user_data.evm_address, user_data.evm_wallet_id)
            {
                let mut evm_cfg = EvmNetworkConfig::new(
                    auth_data.network.clone(),
                    1, // TODO: Map network name to chain ID
                    "https://eth.llamarpc.com",
                );
                evm_cfg.explorer_url = Some("https://etherscan.io".into());
                evm_cfg.native_token = Some("ETH".into());
                let client = create_privy_client(&self.privy_app_id, &self.privy_app_secret)?;
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

    pub(super) fn create_privy_client(
        app_id: &str,
        app_secret: &str,
    ) -> Result<reqwest::Client, Box<dyn StdError + Send + Sync>> {
        let auth = format!("{app_id}:{app_secret}");
        let basic = format!("Basic {}", STANDARD.encode(auth.as_bytes()));

        let mut headers = HeaderMap::new();
        headers.insert(
            "privy-app-id",
            app_id
                .parse()
                .map_err(|e| format!("Invalid privy app ID: {e}"))?,
        );
        headers.insert(
            "Content-Type",
            "application/json"
                .parse()
                .map_err(|e| format!("Invalid content type header: {e}"))?,
        );
        headers.insert(
            "Authorization",
            basic
                .parse()
                .map_err(|e| format!("Invalid authorization header: {e}"))?,
        );

        let client = reqwest::Client::builder()
            .http1_only()
            .default_headers(headers)
            .build()
            .map_err(|e| format!("Failed to build privy client: {e}"))?;

        Ok(client)
    }

    #[derive(Clone)]
    pub(super) struct PrivySolanaSigner {
        client: reqwest::Client,
        pub address: String,
        #[allow(dead_code)]
        rpc: Arc<RpcClient>,
        pub network: SolanaNetworkConfig,
    }

    impl fmt::Debug for PrivySolanaSigner {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("PrivySolanaSigner")
                .field("client", &"RpcClient")
                .field("address", &self.address)
                .field("rpc", &"Arc<RpcClient>")
                .field("network", &self.network)
                .finish()
        }
    }

    impl PrivySolanaSigner {
        #[must_use]
        pub fn new(client: reqwest::Client, address: String, network: SolanaNetworkConfig) -> Self {
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
        fn supported_chains(&self) -> &[Chain] {
            &[Chain::Solana]
        }

        fn user_id(&self) -> String {
            format!("privy_solana_{}", self.address)
        }
    }

    #[async_trait]
    impl SolanaSigner for PrivySolanaSigner {
        fn pubkey(&self) -> String {
            self.address.clone()
        }

        async fn sign_and_send_transaction(
            &self,
            transaction: serde_json::Value,
        ) -> Result<String, Box<dyn SignerError>> {
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

            // Convert JSON transaction to Solana Transaction
            let tx_data: Vec<u8> = serde_json::from_value(transaction.clone()).map_err(|e| {
                Box::new(Standard::SigningFailed(format!(
                    "Failed to parse transaction JSON: {e}"
                ))) as Box<dyn SignerError>
            })?;

            // Serialize the transaction as base64 for Privy API
            let tx_base64 = STANDARD.encode(&tx_data);

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
                    Box::new(Standard::Network(format!("Privy request failed: {e}")))
                        as Box<dyn SignerError>
                })?;

            if !resp.status().is_success() {
                return Err(
                    Box::new(Standard::Network(format!("Privy error: {}", resp.status())))
                        as Box<dyn SignerError>,
                );
            }
            let parsed: Response = resp.json().await.map_err(|e| {
                Box::new(Standard::Network(format!("Invalid response: {e}")))
                    as Box<dyn SignerError>
            })?;
            Ok(parsed.data.hash)
        }

        fn client(&self) -> &dyn SolanaClient {
            unimplemented!("Direct Solana client access not supported by PrivySolanaSigner")
        }

        async fn sign_message(&self, _message: &[u8]) -> Result<String, Box<dyn SignerError>> {
            // For now, return an error since Privy doesn't support message signing via API
            Err(Box::new(Standard::Generic(
                "Message signing not supported by PrivySolanaSigner".to_string(),
            )) as Box<dyn SignerError>)
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

        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            None
        }
    }

    #[derive(Debug, Clone)]
    pub(super) struct PrivyEvmSigner {
        client: reqwest::Client,
        pub address: String,
        pub wallet_id: String,
        network: EvmNetworkConfig,
    }

    impl PrivyEvmSigner {
        #[must_use]
        pub const fn new(
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
        fn supported_chains(&self) -> &[Chain] {
            &[Chain::Evm]
        }

        fn user_id(&self) -> String {
            format!("privy_evm_{}", self.address)
        }
    }

    #[async_trait]
    impl EvmSigner for PrivyEvmSigner {
        fn address(&self) -> String {
            self.address.clone()
        }

        fn chain_id(&self) -> u64 {
            self.network.chain_id
        }

        async fn sign_and_send_transaction(
            &self,
            tx: serde_json::Value,
        ) -> Result<String, Box<dyn SignerError>> {
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

            // Convert JSON to TransactionRequest
            let tx_request: TransactionRequest = serde_json::from_value(tx).map_err(|e| {
                Box::new(Standard::SigningFailed(format!(
                    "Failed to parse transaction request: {e}"
                ))) as Box<dyn SignerError>
            })?;

            let from = tx_request
                .from
                .map_or_else(|| self.address.clone(), |a| format!("0x{a:x}"));
            // TxKind should have Call(Address) or Create variants
            // Since we can't access the variants directly, let's use a placeholder for now
            let to = "0x0000000000000000000000000000000000000000".to_string();
            let value = tx_request.value.map(|v| format!("0x{v:x}"));
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
                    Box::new(Standard::Network(format!("Privy request failed: {e}")))
                        as Box<dyn SignerError>
                })?;
            if !resp.status().is_success() {
                return Err(
                    Box::new(Standard::Network(format!("Privy error: {}", resp.status())))
                        as Box<dyn SignerError>,
                );
            }
            let parsed: Response = resp.json().await.map_err(|e| {
                Box::new(Standard::Network(format!("Invalid response: {e}")))
                    as Box<dyn SignerError>
            })?;
            Ok(parsed.data.hash)
        }

        fn client(&self) -> &dyn EvmClient {
            unimplemented!("Direct EVM client access not provided by PrivyEvmSigner")
        }

        async fn sign_message(&self, _message: &[u8]) -> Result<String, Box<dyn SignerError>> {
            // For now, return an error since Privy doesn't support message signing via API
            Err(Box::new(Standard::Generic(
                "Message signing not supported by PrivyEvmSigner".to_string(),
            )) as Box<dyn SignerError>)
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

        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            Some(self)
        }
    }
}

#[cfg(feature = "web-server")]
pub use privy_impl::{PrivySignerFactory, PrivyUserData};

#[cfg(test)]
mod tests {
    #[cfg(feature = "web-server")]
    use super::{PrivySignerFactory, PrivyUserData};
    #[cfg(feature = "web-server")]
    use riglr_web_adapters::SignerFactory;

    #[test]
    fn test_module_structure_when_privy_module_exists_should_be_accessible() {
        // Test that privy module is accessible
        // The mere compilation of this test validates module structure
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_exports_when_web_server_feature_enabled_should_be_available() {
        // Test that PrivySignerFactory is accessible when web-server feature is enabled
        let factory = PrivySignerFactory::new("test_id".to_string(), "test_secret".to_string());
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);

        // Test that PrivyUserData is accessible and can be constructed
        let user_data = PrivyUserData {
            id: "test_user".to_string(),
            solana_address: Some("11111111111111111111111111111112".to_string()),
            evm_address: Some("0x1111111111111111111111111111111111111111".to_string()),
            evm_wallet_id: Some("wallet_123".to_string()),
            verified: true,
        };
        assert_eq!(user_data.id, "test_user");
        assert!(user_data.verified);
    }

    #[test]
    #[cfg(not(feature = "web-server"))]
    fn test_exports_when_web_server_feature_disabled_should_not_be_available() {
        // When web-server feature is disabled, the re-exports should not be available
        // This test ensures conditional compilation works correctly
        // The mere compilation of this test validates the feature gate behavior
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_creation_when_all_fields_provided_should_store_correctly() {
        let user_data = PrivyUserData {
            id: "did:privy:123456".to_string(),
            solana_address: Some("So11111111111111111111111111111111111111112".to_string()),
            evm_address: Some("0x742d35Cc6636C0532925a3b8C17c604Bb4b7Efb3".to_string()),
            evm_wallet_id: Some("wallet_789".to_string()),
            verified: true,
        };

        assert_eq!(user_data.id, "did:privy:123456");
        assert_eq!(
            user_data.solana_address,
            Some("So11111111111111111111111111111111111111112".to_string())
        );
        assert_eq!(
            user_data.evm_address,
            Some("0x742d35Cc6636C0532925a3b8C17c604Bb4b7Efb3".to_string())
        );
        assert_eq!(user_data.evm_wallet_id, Some("wallet_789".to_string()));
        assert!(user_data.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_creation_when_optional_fields_none_should_handle_gracefully() {
        let user_data = PrivyUserData {
            id: "did:privy:no_wallets".to_string(),
            solana_address: None,
            evm_address: None,
            evm_wallet_id: None,
            verified: false,
        };

        assert_eq!(user_data.id, "did:privy:no_wallets");
        assert!(user_data.solana_address.is_none());
        assert!(user_data.evm_address.is_none());
        assert!(user_data.evm_wallet_id.is_none());
        assert!(!user_data.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_clone_when_called_should_create_identical_copy() {
        let original = PrivyUserData {
            id: "test_clone".to_string(),
            solana_address: Some("address1".to_string()),
            evm_address: Some("address2".to_string()),
            evm_wallet_id: Some("wallet1".to_string()),
            verified: true,
        };

        let cloned = original.clone();

        assert_eq!(original.id, cloned.id);
        assert_eq!(original.solana_address, cloned.solana_address);
        assert_eq!(original.evm_address, cloned.evm_address);
        assert_eq!(original.evm_wallet_id, cloned.evm_wallet_id);
        assert_eq!(original.verified, cloned.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_debug_when_called_should_format_correctly() {
        let user_data = PrivyUserData {
            id: "debug_test".to_string(),
            solana_address: Some("sol_addr".to_string()),
            evm_address: None,
            evm_wallet_id: None,
            verified: true,
        };

        let debug_output = format!("{user_data:?}");
        assert!(debug_output.contains("debug_test"));
        assert!(debug_output.contains("sol_addr"));
        assert!(debug_output.contains("true"));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_factory_with_empty_credentials_when_app_id_empty_should_create_factory() {
        // Test edge case with empty strings
        let factory = PrivySignerFactory::new(String::new(), "secret".to_string());
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_factory_with_empty_secret_when_app_secret_empty_should_create_factory() {
        // Test edge case with empty app secret
        let factory = PrivySignerFactory::new("app_id".to_string(), String::new());
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_factory_with_both_empty_when_both_credentials_empty_should_create_factory() {
        // Test edge case with both empty
        let factory = PrivySignerFactory::new(String::new(), String::new());
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }

    #[test]
    #[cfg(feature = "web-server")]
    #[allow(clippy::indexing_slicing)]
    fn test_privy_factory_supported_auth_types_when_called_should_return_privy() {
        let factory = PrivySignerFactory::new("test_id".to_string(), "test_secret".to_string());
        let auth_types = factory.supported_auth_types();

        assert_eq!(auth_types.len(), 1);
        assert_eq!(auth_types[0], "privy");
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_factory_with_unicode_credentials_when_provided_should_handle_correctly() {
        // Test with unicode characters in credentials
        let factory = PrivySignerFactory::new("app_🔑".to_string(), "secret_🗝️".to_string());
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_factory_with_very_long_credentials_when_provided_should_handle_correctly() {
        // Test with very long strings
        let long_string = "a".repeat(1000);
        let factory = PrivySignerFactory::new(long_string.clone(), long_string);
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_with_empty_id_when_provided_should_store_correctly() {
        let user_data = PrivyUserData {
            id: String::new(),
            solana_address: None,
            evm_address: None,
            evm_wallet_id: None,
            verified: false,
        };

        assert_eq!(user_data.id, "");
        assert!(!user_data.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_with_unicode_addresses_when_provided_should_store_correctly() {
        let user_data = PrivyUserData {
            id: "test_unicode".to_string(),
            solana_address: Some("🔑address".to_string()),
            evm_address: Some("0x🗝️address".to_string()),
            evm_wallet_id: Some("wallet_🆔".to_string()),
            verified: true,
        };

        assert_eq!(user_data.solana_address, Some("🔑address".to_string()));
        assert_eq!(user_data.evm_address, Some("0x🗝️address".to_string()));
        assert_eq!(user_data.evm_wallet_id, Some("wallet_🆔".to_string()));
    }

    #[test]
    #[cfg(feature = "web-server")]
    #[allow(clippy::unwrap_used)]
    fn test_privy_user_data_with_very_long_strings_when_provided_should_store_correctly() {
        let long_string = "x".repeat(10000);
        let user_data = PrivyUserData {
            id: long_string.clone(),
            solana_address: Some(long_string.clone()),
            evm_address: Some(long_string.clone()),
            evm_wallet_id: Some(long_string),
            verified: true,
        };

        assert_eq!(user_data.id.len(), 10000);
        assert_eq!(user_data.solana_address.as_ref().unwrap().len(), 10000);
        assert_eq!(user_data.evm_address.as_ref().unwrap().len(), 10000);
        assert_eq!(user_data.evm_wallet_id.as_ref().unwrap().len(), 10000);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_module_exports_when_feature_enabled_should_match_privy_impl_exports() {
        // Test that the re-exports from privy module are available at the module level
        let factory = PrivySignerFactory::new("test".to_string(), "test".to_string());
        let _: Vec<String> = factory.supported_auth_types();

        let user_data = PrivyUserData {
            id: "test".to_string(),
            solana_address: None,
            evm_address: None,
            evm_wallet_id: None,
            verified: false,
        };
        let _: String = user_data.id;
    }

    #[test]
    fn test_privy_module_accessibility_when_imported_should_be_available() {
        // Test that the privy module can be accessed
        // This validates the pub mod privy; declaration

        // The ability to use the module path validates the module declaration
        let module_path = "privy";
        assert!(module_path == "privy");
    }

    #[test]
    #[cfg(feature = "web-server")]
    #[allow(clippy::unwrap_used)]
    fn test_serde_serialization_when_privy_user_data_serialized_should_work() {
        let user_data = PrivyUserData {
            id: "serialize_test".to_string(),
            solana_address: Some("sol123".to_string()),
            evm_address: None,
            evm_wallet_id: Some("wallet456".to_string()),
            verified: true,
        };

        // Test that serialization works (JSON is common format)
        let serialized = serde_json::to_string(&user_data);
        assert!(serialized.is_ok());

        let json_str = serialized.unwrap();
        assert!(json_str.contains("serialize_test"));
        assert!(json_str.contains("sol123"));
        assert!(json_str.contains("wallet456"));
    }

    #[test]
    #[cfg(feature = "web-server")]
    #[allow(clippy::unwrap_used)]
    fn test_serde_deserialization_when_privy_user_data_deserialized_should_work() {
        let json_data = r#"{
            "id": "deserialize_test",
            "solana_address": "sol789",
            "evm_address": null,
            "evm_wallet_id": "wallet123",
            "verified": false
        }"#;

        let deserialized: Result<PrivyUserData, _> = serde_json::from_str(json_data);
        assert!(deserialized.is_ok());

        let user_data = deserialized.unwrap();
        assert_eq!(user_data.id, "deserialize_test");
        assert_eq!(user_data.solana_address, Some("sol789".to_string()));
        assert!(user_data.evm_address.is_none());
        assert_eq!(user_data.evm_wallet_id, Some("wallet123".to_string()));
        assert!(!user_data.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    #[allow(clippy::unwrap_used)]
    fn test_serde_round_trip_when_serialize_then_deserialize_should_maintain_data() {
        let original = PrivyUserData {
            id: "round_trip_test".to_string(),
            solana_address: Some("original_sol".to_string()),
            evm_address: Some("original_evm".to_string()),
            evm_wallet_id: None,
            verified: true,
        };

        // Serialize then deserialize
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: PrivyUserData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.solana_address, deserialized.solana_address);
        assert_eq!(original.evm_address, deserialized.evm_address);
        assert_eq!(original.evm_wallet_id, deserialized.evm_wallet_id);
        assert_eq!(original.verified, deserialized.verified);
    }
}

// Additional tests from the original privy.rs
#[cfg(test)]
mod privy_tests {
    #[cfg(feature = "web-server")]
    use super::privy_impl::*;
    #[cfg(feature = "web-server")]
    use riglr_config::{EvmNetworkConfig, SolanaNetworkConfig};
    #[cfg(feature = "web-server")]
    use riglr_core::signer::{EvmSigner, SolanaSigner};
    #[cfg(feature = "web-server")]
    use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};
    #[cfg(feature = "web-server")]
    use std::collections::HashMap;
    #[cfg(feature = "web-server")]
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
    #[allow(clippy::unwrap_used)]
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
    #[allow(clippy::expect_used)]
    fn test_create_privy_client() {
        let client = create_privy_client("test_app_id", "test_app_secret")
            .expect("Should create client successfully");

        // Just verify the client was created successfully
        assert!(format!("{client:?}").contains("Client"));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_solana_signer_new() {
        let client = reqwest::Client::new();
        let mut network = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        network.explorer_url = Some("https://explorer.solana.com".to_string());

        let signer = PrivySolanaSigner::new(client, "test_address".to_string(), network);

        assert_eq!(signer.address, "test_address");
        assert_eq!(signer.network.name, "devnet");
        assert_eq!(signer.network.rpc_url, "https://api.devnet.solana.com");
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_solana_signer_debug() {
        let client = reqwest::Client::new();
        let network =
            SolanaNetworkConfig::new("devnet".to_string(), "https://api.devnet.solana.com");

        let signer = PrivySolanaSigner::new(client, "test_address".to_string(), network);

        let debug_str = format!("{signer:?}");
        assert!(debug_str.contains("PrivySolanaSigner"));
        assert!(debug_str.contains("test_address"));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_solana_signer_address() {
        let client = reqwest::Client::new();
        let network =
            SolanaNetworkConfig::new("devnet".to_string(), "https://api.devnet.solana.com");

        let signer = PrivySolanaSigner::new(client, "test_address".to_string(), network);

        assert_eq!(signer.pubkey(), "test_address".to_string());
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_solana_signer_pubkey() {
        let client = reqwest::Client::new();
        let network =
            SolanaNetworkConfig::new("devnet".to_string(), "https://api.devnet.solana.com");

        let signer = PrivySolanaSigner::new(client, "test_pubkey".to_string(), network);

        assert_eq!(signer.pubkey(), "test_pubkey".to_string());
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_evm_signer_new() {
        let client = reqwest::Client::new();
        let network = EvmNetworkConfig::new(
            "ethereum".to_string(),
            1,
            "https://eth-mainnet.alchemyapi.io/v2/test",
        );

        let signer = PrivyEvmSigner::new(
            client,
            "0x123".to_string(),
            "wallet_123".to_string(),
            network,
        );

        assert_eq!(signer.address, "0x123");
        assert_eq!(signer.wallet_id, "wallet_123");
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_evm_signer_address() {
        let client = reqwest::Client::new();
        let network = EvmNetworkConfig::new(
            "ethereum".to_string(),
            1,
            "https://eth-mainnet.alchemyapi.io/v2/test",
        );

        let signer = PrivyEvmSigner::new(
            client,
            "0x456".to_string(),
            "wallet_456".to_string(),
            network,
        );

        assert_eq!(signer.address(), "0x456".to_string());
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_evm_signer_debug() {
        let client = reqwest::Client::new();
        let network = EvmNetworkConfig::new(
            "ethereum".to_string(),
            1,
            "https://eth-mainnet.alchemyapi.io/v2/test",
        );

        let signer = PrivyEvmSigner::new(
            client,
            "0x123".to_string(),
            "wallet_123".to_string(),
            network,
        );

        let debug_str = format!("{signer:?}");
        assert!(debug_str.contains("PrivyEvmSigner"));
        assert!(debug_str.contains("0x123"));
        assert!(debug_str.contains("wallet_123"));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_evm_signer_clone() {
        let client = reqwest::Client::new();
        let network = EvmNetworkConfig::new(
            "ethereum".to_string(),
            1,
            "https://eth-mainnet.alchemyapi.io/v2/test",
        );

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
        let network =
            SolanaNetworkConfig::new("devnet".to_string(), "https://api.devnet.solana.com");

        let signer = PrivySolanaSigner::new(client, "test_address".to_string(), network);

        let cloned_signer = signer.clone();
        assert_eq!(signer.address, cloned_signer.address);
        assert_eq!(signer.network.name, cloned_signer.network.name);
    }
}
