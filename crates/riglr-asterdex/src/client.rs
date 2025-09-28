//! Asterdex API client for interacting with the Asterdex Futures v3 API
//!
//! This module provides a client for the Asterdex protocol with v3 Web3-based
//! authentication signature mechanism.

use crate::error::{Error, Result};
use chrono::Utc;
use core::str::FromStr;
use core::time::Duration;
use ethabi::Token;
use ethers_core::types::{H160, U256};
use hex;
use reqwest::Response;
use riglr_core::signer::UnifiedSigner;
#[cfg(test)]
use riglr_core::signer::{error::Error as SignerError, SolanaSigner};
use serde::{Deserialize, Serialize};
use serde_json;
use sha3::{Digest, Keccak256};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::debug;

/// Asterdex API client - Real implementation using HTTP API with v3 authentication
#[derive(Debug)]
pub struct Client {
    base_url: String,
    #[expect(clippy::struct_field_names)]
    client: reqwest::Client,
    signer: Arc<dyn UnifiedSigner>,
}

impl Client {
    /// Create a new Asterdex client
    ///
    /// # Errors
    ///
    /// Returns error if HTTP client creation fails
    pub fn new(signer: Arc<dyn UnifiedSigner>) -> Result<Self> {
        Self::with_base_url(signer, "https://fapi.asterdex.com".to_string())
    }

    /// Create a new Asterdex client with custom base URL (for testing)
    ///
    /// # Errors
    ///
    /// Returns error if HTTP client creation fails or base URL is invalid
    pub fn with_base_url(signer: Arc<dyn UnifiedSigner>, base_url: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                Error::NetworkError(format!("Failed to create HTTP client with custom URL: {e}"))
            })?;

        Ok(Self {
            base_url,
            client,
            signer,
        })
    }

    /// Build full URL for an endpoint
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}{}", self.base_url, endpoint)
    }

    /// Build Asterdex v3 authentication signature
    /// This implements the Web3-based signing process for v3 API
    #[allow(clippy::cognitive_complexity)]
    async fn build_signed_payload(
        &self,
        mut params: BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>> {
        // Get user and signer addresses from the UnifiedSigner
        // Cast to EVM signer first
        let evm_signer = self
            .signer
            .as_evm()
            .ok_or_else(|| Error::SignatureError("Signer does not support EVM".to_string()))?;

        let user = evm_signer.address();
        let signer_address = user.clone();

        // Generate microsecond nonce
        #[allow(clippy::cast_sign_loss)]
        let nonce = (Utc::now().timestamp_micros() as u64).to_string();

        // Step 1: Convert all values to strings and ensure they're sorted (BTreeMap does this)
        // Add required timestamp and recvWindow if not present
        if !params.contains_key("timestamp") {
            #[allow(clippy::cast_sign_loss)]
            let timestamp = (Utc::now().timestamp_millis() as u64).to_string();
            params.insert("timestamp".to_string(), timestamp);
        }
        if !params.contains_key("recvWindow") {
            params.insert("recvWindow".to_string(), "50000".to_string());
        }

        // Convert all values to strings (they should already be strings)
        let mut string_params = BTreeMap::new();
        for (key, value) in &params {
            string_params.insert(key.clone(), value.to_string());
        }

        // Step 2: Serialize to compact JSON string
        let json_str = serde_json::to_string(&string_params)
            .map_err(|e| Error::SignatureError(format!("Failed to serialize params: {e}")))?;
        let json_str = json_str.replace(' ', "");
        debug!("JSON string for signing: {}", json_str);

        // Step 3: ABI encode the parameters
        // Convert addresses to H160 type
        let user_h160 = H160::from_str(&user)
            .map_err(|e| Error::SignatureError(format!("Invalid user address: {e}")))?;
        let signer_h160 = H160::from_str(&signer_address)
            .map_err(|e| Error::SignatureError(format!("Invalid signer address: {e}")))?;
        let nonce_u256 = U256::from_dec_str(&nonce)
            .map_err(|e| Error::SignatureError(format!("Invalid nonce: {e}")))?;

        // Create tokens for ABI encoding
        let tokens = vec![
            Token::String(json_str),
            Token::Address(user_h160),
            Token::Address(signer_h160),
            Token::Uint(nonce_u256),
        ];

        // ABI encode the tokens
        let encoded = ethabi::encode(&tokens);
        debug!("Encoded hex: {}", hex::encode(&encoded));

        // Step 4: Calculate Keccak-256 hash
        let mut hasher = Keccak256::new();
        hasher.update(&encoded);
        let hash = hasher.finalize();
        let hash_hex = format!("0x{}", hex::encode(hash));
        debug!("Keccak hash: {}", hash_hex);

        // Step 5: Sign the hash using ECDSA
        // Get the EVM signer and sign the hash
        let evm_signer = self
            .signer
            .as_evm()
            .ok_or_else(|| Error::SignatureError("Signer does not support EVM".to_string()))?;

        let signature = evm_signer
            .sign_message(hash.as_slice())
            .await
            .map_err(|e| Error::SignatureError(format!("Failed to sign message: {e}")))?;

        let signature_hex = format!("0x{signature}");
        debug!("Signature: {}", signature_hex);

        // Add authentication parameters to the original params
        params.insert("user".to_string(), user);
        params.insert("signer".to_string(), signer_address);
        params.insert("nonce".to_string(), nonce);
        params.insert("signature".to_string(), signature_hex);

        Ok(params)
    }

    /// Make a GET request to the Asterdex API with authentication
    ///
    /// # Errors
    ///
    /// Returns error if request fails or network error occurs
    pub async fn get(&self, endpoint: &str, params: BTreeMap<String, String>) -> Result<Response> {
        let url = self.build_url(endpoint);
        debug!("Making GET request to: {}", url);

        // Sign the parameters
        let signed_params = self.build_signed_payload(params).await?;

        let response = self
            .client
            .get(&url)
            .query(&signed_params)
            .send()
            .await
            .map_err(|e| Error::NetworkError(format!("GET request failed: {e}")))?;

        self.handle_response(response).await
    }

    /// Make a POST request to the Asterdex API with authentication
    ///
    /// # Errors
    ///
    /// Returns error if request fails or network error occurs
    pub async fn post(&self, endpoint: &str, params: BTreeMap<String, String>) -> Result<Response> {
        let url = self.build_url(endpoint);
        debug!("Making POST request to: {}", url);

        // Sign the parameters
        let signed_params = self.build_signed_payload(params).await?;

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&signed_params)
            .send()
            .await
            .map_err(|e| Error::NetworkError(format!("POST request failed: {e}")))?;

        self.handle_response(response).await
    }

    /// Make a DELETE request to the Asterdex API with authentication
    ///
    /// # Errors
    ///
    /// Returns error if request fails or network error occurs
    pub async fn delete(
        &self,
        endpoint: &str,
        params: BTreeMap<String, String>,
    ) -> Result<Response> {
        let url = self.build_url(endpoint);
        debug!("Making DELETE request to: {}", url);

        // Sign the parameters
        let signed_params = self.build_signed_payload(params).await?;

        let response = self
            .client
            .delete(&url)
            .form(&signed_params)
            .send()
            .await
            .map_err(|e| Error::NetworkError(format!("DELETE request failed: {e}")))?;

        self.handle_response(response).await
    }

    /// Handle API response and check for errors
    async fn handle_response(&self, response: Response) -> Result<Response> {
        let status = response.status();

        if status.is_success() {
            Ok(response)
        } else {
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response".to_string());

            // Parse error response if possible
            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&text) {
                return Err(Self::map_api_error(status.as_u16(), &error_response));
            }

            // Generic error based on status code
            match status.as_u16() {
                429 => Err(Error::RateLimit(format!("Rate limit exceeded: {text}"))),
                403 => Err(Error::AuthError(format!("Forbidden: {text}"))),
                401 => Err(Error::AuthError(format!("Unauthorized: {text}"))),
                400 => Err(Error::InvalidInput(format!("Bad request: {text}"))),
                500..=599 => Err(Error::ApiError(format!("Server error {status}: {text}"))),
                _ => Err(Error::ApiError(format!("API error {status}: {text}"))),
            }
        }
    }

    /// Map API error response to our error type
    fn map_api_error(status_code: u16, error: &ApiErrorResponse) -> Error {
        match error.code {
            -1121 => Error::InvalidSymbol(error.msg.clone()),
            -2010 => Error::InsufficientBalance(error.msg.clone()),
            -2022 => Error::ReduceOnlyRejected(error.msg.clone()),
            -4164 => Error::MinNotionalNotMet(error.msg.clone()),
            -2021 => Error::OrderWouldImmediatelyTrigger(error.msg.clone()),
            -1102 | -1103 => Error::InvalidInput(error.msg.clone()),
            -1000..=-1 => Error::ApiError(error.msg.clone()),
            _ => match status_code {
                429 => Error::RateLimit(error.msg.clone()),
                401 | 403 => Error::AuthError(error.msg.clone()),
                _ => Error::ApiError(error.msg.clone()),
            },
        }
    }
}

// Data structures for API responses

/// API error response
#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub code: i32,
    pub msg: String,
}

/// Order placement response
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub order_id: i64,
    pub symbol: String,
    pub status: String,
    pub client_order_id: Option<String>,
    pub price: Option<String>,
    pub avg_price: Option<String>,
    pub orig_qty: String,
    pub executed_qty: String,
    pub cumulative_quote_qty: Option<String>,
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub side: String,
    pub stop_price: Option<String>,
    pub orig_type: Option<String>,
    pub activate_price: Option<String>,
    pub price_rate: Option<String>,
    pub update_time: i64,
    pub working_type: Option<String>,
}

/// Order information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub order_id: i64,
    pub symbol: String,
    pub status: String,
    pub client_order_id: Option<String>,
    pub price: String,
    pub avg_price: Option<String>,
    pub orig_qty: String,
    pub executed_qty: String,
    pub cumulative_quote_qty: Option<String>,
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub side: String,
    pub position_side: String,
    pub stop_price: Option<String>,
    pub working_type: Option<String>,
    pub orig_type: Option<String>,
    pub activate_price: Option<String>,
    pub price_rate: Option<String>,
    pub update_time: i64,
    pub reduce_only: bool,
    pub close_position: bool,
}

/// Account balance
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalance {
    pub account_alias: String,
    pub asset: String,
    pub balance: String,
    pub cross_wallet_balance: String,
    pub cross_un_pnl: String,
    pub available_balance: String,
    pub max_withdraw_amount: String,
    pub margin_available: bool,
    pub update_time: Option<i64>,
}

/// Account information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub fee_tier: i32,
    pub can_trade: bool,
    pub can_deposit: bool,
    pub can_withdraw: bool,
    pub update_time: i64,
    pub total_initial_margin: String,
    pub total_maint_margin: String,
    pub total_wallet_balance: String,
    pub total_unrealized_profit: String,
    pub total_margin_balance: String,
    pub total_position_initial_margin: String,
    pub total_open_order_initial_margin: String,
    pub total_cross_wallet_balance: String,
    pub total_cross_un_pnl: String,
    pub available_balance: String,
    pub max_withdraw_amount: String,
    pub positions: Vec<Position>,
    pub assets: Vec<Asset>,
}

/// Position information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub symbol: String,
    pub position_amt: String,
    pub entry_price: String,
    pub mark_price: String,
    pub un_realized_profit: String,
    pub liquidation_price: String,
    pub leverage: String,
    pub max_notional_value: String,
    pub margin_type: String,
    pub isolated_margin: String,
    pub is_auto_add_margin: String,
    pub position_side: String,
    pub notional: String,
    pub isolated_wallet: String,
    pub update_time: i64,
}

/// Asset information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub asset: String,
    pub wallet_balance: String,
    pub unrealized_profit: String,
    pub margin_balance: String,
    pub maint_margin: String,
    pub initial_margin: String,
    pub position_initial_margin: String,
    pub open_order_initial_margin: String,
    pub cross_wallet_balance: String,
    pub cross_un_pnl: String,
    pub available_balance: String,
    pub max_withdraw_amount: String,
    pub margin_available: bool,
    pub update_time: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use core::result::Result as StdResult;
    use core::sync::atomic::{AtomicBool, Ordering};
    use riglr_core::signer::{Chain, EvmClient, EvmSigner, SignerBase};
    use serde_json::json;

    // Mock EVM signer for testing
    #[derive(Debug)]
    struct MockEvmSigner {
        address: String,
        sign_message_called: Arc<AtomicBool>,
    }

    impl MockEvmSigner {
        fn new(address: &str) -> Self {
            Self {
                address: address.to_string(),
                sign_message_called: Arc::new(AtomicBool::new(false)),
            }
        }
    }

    impl SignerBase for MockEvmSigner {
        fn user_id(&self) -> String {
            "test_user".to_string()
        }

        fn supported_chains(&self) -> &[Chain] {
            &[Chain::Evm]
        }
    }

    #[async_trait]
    impl EvmSigner for MockEvmSigner {
        fn address(&self) -> String {
            self.address.clone()
        }

        fn chain_id(&self) -> u64 {
            1
        }

        fn client(&self) -> &dyn EvmClient {
            unimplemented!("Not implemented for test")
        }

        async fn sign_and_send_transaction(
            &self,
            _tx: serde_json::Value,
        ) -> StdResult<String, Box<dyn SignerError>> {
            Ok("0xmocktxhash".to_string())
        }

        async fn sign_message(&self, _message: &[u8]) -> StdResult<String, Box<dyn SignerError>> {
            self.sign_message_called.store(true, Ordering::SeqCst);
            // Return a mock signature
            Ok("deadbeefcafe1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12".to_string())
        }
    }

    // Mock unified signer that wraps the EVM signer
    #[derive(Debug)]
    struct MockUnifiedSigner {
        evm_signer: MockEvmSigner,
    }

    impl MockUnifiedSigner {
        fn new(address: &str) -> Self {
            Self {
                evm_signer: MockEvmSigner::new(address),
            }
        }
    }

    impl SignerBase for MockUnifiedSigner {
        fn user_id(&self) -> String {
            "test_user".to_string()
        }

        fn supported_chains(&self) -> &[Chain] {
            &[Chain::Evm]
        }
    }

    impl UnifiedSigner for MockUnifiedSigner {
        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            Some(&self.evm_signer)
        }

        fn as_solana(&self) -> Option<&dyn SolanaSigner> {
            None
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_client_creation() {
        let signer = Arc::new(MockUnifiedSigner::new(
            "0x1234567890123456789012345678901234567890",
        ));
        let client = Client::new(signer);
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.base_url, "https://fapi.asterdex.com");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_client_with_custom_url() {
        let signer = Arc::new(MockUnifiedSigner::new(
            "0x1234567890123456789012345678901234567890",
        ));
        let client = Client::with_base_url(signer, "http://localhost:8080".to_string());
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_build_url() {
        let signer = Arc::new(MockUnifiedSigner::new(
            "0x1234567890123456789012345678901234567890",
        ));
        let client = Client::new(signer).unwrap();
        assert_eq!(
            client.build_url("/fapi/v3/order"),
            "https://fapi.asterdex.com/fapi/v3/order"
        );
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used)]
    async fn test_build_signed_payload() {
        let signer = Arc::new(MockUnifiedSigner::new(
            "0x1234567890123456789012345678901234567890",
        ));
        let client = Client::new(signer).unwrap();

        let mut params = BTreeMap::new();
        params.insert("symbol".to_string(), "BTCUSDT".to_string());
        params.insert("side".to_string(), "BUY".to_string());
        params.insert("quantity".to_string(), "0.001".to_string());

        let signed_result = client.build_signed_payload(params.clone()).await;
        assert!(signed_result.is_ok());

        let signed_params = signed_result.unwrap();

        // Verify all required fields are present
        assert!(signed_params.contains_key("user"));
        assert!(signed_params.contains_key("signer"));
        assert!(signed_params.contains_key("nonce"));
        assert!(signed_params.contains_key("signature"));
        assert!(signed_params.contains_key("timestamp"));
        assert!(signed_params.contains_key("recvWindow"));

        // Verify original params are preserved
        assert_eq!(signed_params.get("symbol"), Some(&"BTCUSDT".to_string()));
        assert_eq!(signed_params.get("side"), Some(&"BUY".to_string()));
        assert_eq!(signed_params.get("quantity"), Some(&"0.001".to_string()));

        // Verify signature format
        let signature = signed_params.get("signature").unwrap();
        assert!(signature.starts_with("0x"));
        assert!(signature.len() > 2); // More than just "0x"
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used)]
    async fn test_build_signed_payload_with_existing_timestamp() {
        let signer = Arc::new(MockUnifiedSigner::new(
            "0x1234567890123456789012345678901234567890",
        ));
        let client = Client::new(signer).unwrap();

        let mut params = BTreeMap::new();
        params.insert("symbol".to_string(), "BTCUSDT".to_string());
        params.insert("timestamp".to_string(), "1234567890000".to_string());
        params.insert("recvWindow".to_string(), "10000".to_string());

        let signed_result = client.build_signed_payload(params.clone()).await;
        assert!(signed_result.is_ok());

        let signed_params = signed_result.unwrap();

        // Verify custom timestamp and recvWindow are preserved
        assert_eq!(
            signed_params.get("timestamp"),
            Some(&"1234567890000".to_string())
        );
        assert_eq!(signed_params.get("recvWindow"), Some(&"10000".to_string()));
    }

    // Mock signer that doesn't support EVM
    #[derive(Debug)]
    struct NonEvmSigner;

    impl SignerBase for NonEvmSigner {
        fn user_id(&self) -> String {
            "test_user".to_string()
        }

        fn supported_chains(&self) -> &[Chain] {
            &[Chain::Solana]
        }
    }

    impl UnifiedSigner for NonEvmSigner {
        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            None
        }

        fn as_solana(&self) -> Option<&dyn SolanaSigner> {
            None // We don't need to implement Solana for this test
        }
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used, clippy::panic)]
    async fn test_build_signed_payload_non_evm_signer() {
        let signer = Arc::new(NonEvmSigner);
        let client = Client::new(signer).unwrap();

        let params = BTreeMap::new();
        let result = client.build_signed_payload(params).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::SignatureError(msg) => {
                assert!(msg.contains("Signer does not support EVM"));
            }
            _ => panic!("Expected SignatureError"),
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_map_api_error() {
        let signer = Arc::new(MockUnifiedSigner::new(
            "0x1234567890123456789012345678901234567890",
        ));
        let _client = Client::new(signer).unwrap();

        let test_cases = vec![
            (
                ApiErrorResponse {
                    code: -1121,
                    msg: "Invalid symbol".to_string(),
                },
                400,
                "InvalidSymbol",
            ),
            (
                ApiErrorResponse {
                    code: -2010,
                    msg: "Insufficient balance".to_string(),
                },
                400,
                "InsufficientBalance",
            ),
            (
                ApiErrorResponse {
                    code: -1102,
                    msg: "Invalid parameter".to_string(),
                },
                400,
                "InvalidInput",
            ),
            (
                ApiErrorResponse {
                    code: -1000,
                    msg: "Unknown error".to_string(),
                },
                400,
                "ApiError",
            ),
            (
                ApiErrorResponse {
                    code: 0,
                    msg: "Rate limit".to_string(),
                },
                429,
                "RateLimit",
            ),
            (
                ApiErrorResponse {
                    code: 0,
                    msg: "Unauthorized".to_string(),
                },
                401,
                "AuthError",
            ),
        ];

        for (api_err, status_code, expected_variant) in test_cases {
            let error = Client::map_api_error(status_code, &api_err);
            let error_str = format!("{error:?}");
            assert!(
                error_str.contains(expected_variant),
                "Expected {} for code {} status {}, got {:?}",
                expected_variant,
                api_err.code,
                status_code,
                error
            );
        }
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::panic)]
    fn test_api_error_response_deserialization() {
        let json = json!({
            "code": -1121,
            "msg": "Invalid symbol."
        });

        let result: StdResult<ApiErrorResponse, _> = serde_json::from_value(json);
        assert!(result.is_ok());

        let error = result.unwrap();
        assert_eq!(error.code, -1121);
        assert_eq!(error.msg, "Invalid symbol.");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_order_response_deserialization() {
        let json = json!({
            "orderId": 12345,
            "symbol": "BTCUSDT",
            "status": "NEW",
            "clientOrderId": "myorder123",
            "price": "50000.00",
            "avgPrice": "0.00",
            "origQty": "0.001",
            "executedQty": "0",
            "cumulativeQuoteQty": "0",
            "timeInForce": "GTC",
            "type": "LIMIT",
            "side": "BUY",
            "stopPrice": null,
            "origType": "LIMIT",
            "activatePrice": null,
            "priceRate": null,
            "updateTime": 1_234_567_890_000_i64,
            "workingType": "CONTRACT_PRICE"
        });

        let result: StdResult<OrderResponse, _> = serde_json::from_value(json);
        assert!(result.is_ok());

        let order = result.unwrap();
        assert_eq!(order.order_id, 12345);
        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.status, "NEW");
        assert_eq!(order.orig_qty, "0.001");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_account_balance_deserialization() {
        let json = json!({
            "accountAlias": "main",
            "asset": "USDT",
            "balance": "1000.50",
            "crossWalletBalance": "900.50",
            "crossUnPnl": "50.00",
            "availableBalance": "850.50",
            "maxWithdrawAmount": "800.00",
            "marginAvailable": true,
            "updateTime": 1_234_567_890_000_i64
        });

        let result: StdResult<AccountBalance, _> = serde_json::from_value(json);
        assert!(result.is_ok());

        let balance = result.unwrap();
        assert_eq!(balance.asset, "USDT");
        assert_eq!(balance.balance, "1000.50");
        assert!(balance.margin_available);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_position_deserialization() {
        let json = json!({
            "symbol": "BTCUSDT",
            "positionAmt": "0.001",
            "entryPrice": "50000.00",
            "markPrice": "51000.00",
            "unRealizedProfit": "1.00",
            "liquidationPrice": "45000.00",
            "leverage": "10",
            "maxNotionalValue": "100000",
            "marginType": "cross",
            "isolatedMargin": "0",
            "isAutoAddMargin": "false",
            "positionSide": "BOTH",
            "notional": "51.00",
            "isolatedWallet": "0",
            "updateTime": 1_234_567_890_000_i64
        });

        let result: StdResult<Position, _> = serde_json::from_value(json);
        assert!(result.is_ok());

        let position = result.unwrap();
        assert_eq!(position.symbol, "BTCUSDT");
        assert_eq!(position.position_amt, "0.001");
        assert_eq!(position.entry_price, "50000.00");
        assert_eq!(position.leverage, "10");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_order_deserialization_complete() {
        let json = json!({
            "orderId": 12345,
            "symbol": "BTCUSDT",
            "status": "NEW",
            "clientOrderId": "myorder123",
            "price": "50000.00",
            "avgPrice": "0.00",
            "origQty": "0.001",
            "executedQty": "0",
            "cumulativeQuoteQty": "0",
            "timeInForce": "GTC",
            "type": "LIMIT",
            "side": "BUY",
            "positionSide": "BOTH",
            "stopPrice": null,
            "workingType": "CONTRACT_PRICE",
            "origType": "LIMIT",
            "activatePrice": null,
            "priceRate": null,
            "updateTime": 1_234_567_890_000_i64,
            "reduceOnly": false,
            "closePosition": false
        });

        let result: StdResult<Order, _> = serde_json::from_value(json);
        assert!(result.is_ok());

        let order = result.unwrap();
        assert_eq!(order.order_id, 12345);
        assert_eq!(order.position_side, "BOTH");
        assert!(!order.reduce_only);
        assert!(!order.close_position);
    }
}
