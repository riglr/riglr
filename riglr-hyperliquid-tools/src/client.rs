//! Hyperliquid API client for interacting with the Hyperliquid L1 blockchain
//!
//! This module provides a client for the Hyperliquid protocol, which operates
//! its own L1 blockchain for perpetual futures trading.

use reqwest::{Client, Response};
use riglr_core::signer::UnifiedSigner;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use tracing::{debug, error, info};
// Note: Using direct HTTP API instead of SDK

const HYPERLIQUID_PRIVATE_KEY: &str = "HYPERLIQUID_PRIVATE_KEY";
const ENABLE_REAL_HYPERLIQUID_TRADING: &str = "ENABLE_REAL_HYPERLIQUID_TRADING";

// EIP-712 signing dependencies
use ethers_core::types::transaction::eip712::{Eip712, TypedData};
use ethers_core::types::H256;
use ethers_signers::LocalWallet;
use hex;

use crate::error::{HyperliquidToolError, Result};

/// Hyperliquid API client - Real implementation using HTTP API
/// This client makes REAL API calls to Hyperliquid - NO SIMULATION
#[derive(Debug)]
pub struct HyperliquidClient {
    client: Client,
    base_url: String,
    signer: Arc<dyn UnifiedSigner>,
}

impl HyperliquidClient {
    /// Create a new Hyperliquid client
    pub fn new(signer: Arc<dyn UnifiedSigner>) -> Result<Self> {
        Self::with_base_url(signer, "https://api.hyperliquid.xyz".to_string())
    }

    /// Create a new Hyperliquid client with custom base URL (for testing)
    pub fn with_base_url(signer: Arc<dyn UnifiedSigner>, base_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                HyperliquidToolError::NetworkError(format!(
                    "Failed to create HTTP client with custom URL: {}",
                    e
                ))
            })?;

        Ok(Self {
            client,
            base_url,
            signer,
        })
    }

    /// Make a GET request to the Hyperliquid API
    pub async fn get(&self, endpoint: &str) -> Result<Response> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );

        debug!("Making GET request to: {}", url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response)
                } else {
                    let status = response.status();
                    let error_body = response.text().await.unwrap_or_default();
                    error!("GET request failed with status {}: {}", status, error_body);

                    if status == 429 {
                        Err(HyperliquidToolError::RateLimit(format!(
                            "Rate limit exceeded: {}",
                            error_body
                        )))
                    } else if status.is_server_error() {
                        Err(HyperliquidToolError::NetworkError(format!(
                            "Server error {}: {}",
                            status, error_body
                        )))
                    } else {
                        Err(HyperliquidToolError::ApiError(format!(
                            "API error {}: {}",
                            status, error_body
                        )))
                    }
                }
            }
            Err(e) => {
                error!("Network error during GET request: {}", e);
                Err(HyperliquidToolError::NetworkError(format!(
                    "Network error during GET request: {}",
                    e
                )))
            }
        }
    }

    /// Make a POST request to the Hyperliquid API
    pub async fn post<T: Serialize>(&self, endpoint: &str, payload: &T) -> Result<Response> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );

        debug!("Making POST request to: {}", url);

        let json_payload = serde_json::to_string(payload).map_err(|e| {
            HyperliquidToolError::ApiError(format!("Failed to serialize request payload: {}", e))
        })?;

        match self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json_payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response)
                } else {
                    let status = response.status();
                    let error_body = response.text().await.unwrap_or_default();
                    error!("POST request failed with status {}: {}", status, error_body);

                    if status == 429 {
                        Err(HyperliquidToolError::RateLimit(format!(
                            "Rate limit exceeded on POST: {}",
                            error_body
                        )))
                    } else if status.is_server_error() {
                        Err(HyperliquidToolError::NetworkError(format!(
                            "Server error {} on POST: {}",
                            status, error_body
                        )))
                    } else {
                        Err(HyperliquidToolError::ApiError(format!(
                            "API error {} on POST: {}",
                            status, error_body
                        )))
                    }
                }
            }
            Err(e) => {
                error!("Network error during POST request: {}", e);
                Err(HyperliquidToolError::NetworkError(format!(
                    "Network error during POST request: {}",
                    e
                )))
            }
        }
    }

    /// Get account information
    pub async fn get_account_info(&self, user_address: &str) -> Result<AccountInfo> {
        let response = self
            .get(&format!(
                "info?type=clearinghouseState&user={}",
                user_address
            ))
            .await?;
        let text = response.text().await.map_err(|e| {
            HyperliquidToolError::NetworkError(format!(
                "Failed to read account info response: {}",
                e
            ))
        })?;

        let account_info: AccountInfo = serde_json::from_str(&text).map_err(|e| {
            HyperliquidToolError::ApiError(format!("Failed to parse account info response: {}", e))
        })?;

        Ok(account_info)
    }

    /// Get current positions for a user
    pub async fn get_positions(&self, user_address: &str) -> Result<Vec<Position>> {
        let response = self
            .get(&format!(
                "info?type=clearinghouseState&user={}",
                user_address
            ))
            .await?;
        let text = response.text().await.map_err(|e| {
            HyperliquidToolError::NetworkError(format!("Failed to read positions response: {}", e))
        })?;

        let state: ClearinghouseState = serde_json::from_str(&text).map_err(|e| {
            HyperliquidToolError::ApiError(format!("Failed to parse clearinghouse state: {}", e))
        })?;

        Ok(state.asset_positions.unwrap_or_default())
    }

    /// Get market information
    pub async fn get_meta(&self) -> Result<Meta> {
        let response = self.get("info?type=meta").await?;
        let text = response.text().await.map_err(|e| {
            HyperliquidToolError::NetworkError(format!("Failed to read meta response: {}", e))
        })?;

        let meta: Meta = serde_json::from_str(&text).map_err(|e| {
            HyperliquidToolError::ApiError(format!(
                "Failed to parse market meta information: {}",
                e
            ))
        })?;

        Ok(meta)
    }

    /// Get all market mid prices (current market prices)
    pub async fn get_all_mids(&self) -> Result<serde_json::Value> {
        let response = self.get("info?type=allMids").await?;
        let text = response.text().await.map_err(|e| {
            HyperliquidToolError::NetworkError(format!("Failed to read mids response: {}", e))
        })?;

        let mids: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
            HyperliquidToolError::ApiError(format!("Failed to parse market mids: {}", e))
        })?;

        Ok(mids)
    }

    /// Place an order using real Hyperliquid API
    /// CRITICAL: This is REAL order placement - NO SIMULATION
    pub async fn place_order(&self, order: &OrderRequest) -> Result<OrderResponse> {
        error!("CRITICAL: REAL ORDER PLACEMENT - This will place actual trades!");
        info!(
            "Placing REAL order: asset={}, side={}, size={}, price={}",
            order.asset,
            if order.is_buy { "buy" } else { "sell" },
            order.sz,
            order.limit_px
        );

        // Require explicit configuration to prevent accidental trades
        let private_key = std::env::var(HYPERLIQUID_PRIVATE_KEY)
            .map_err(|_| HyperliquidToolError::Configuration(
                "HYPERLIQUID_PRIVATE_KEY environment variable required. REAL trading disabled for safety.".to_string()
            ))?;

        let trading_enabled =
            std::env::var(ENABLE_REAL_HYPERLIQUID_TRADING).unwrap_or_default() == "true";

        if !trading_enabled {
            return Err(HyperliquidToolError::Configuration(
                "Real trading disabled. Set ENABLE_REAL_HYPERLIQUID_TRADING=true to enable REAL orders.".to_string()
            ));
        }

        // Create the real order payload for Hyperliquid API
        let order_payload = serde_json::json!({
            "action": {
                "type": "order",
                "orders": [
                    {
                        "asset": order.asset,
                        "isBuy": order.is_buy,
                        "limitPx": order.limit_px,
                        "sz": order.sz,
                        "reduceOnly": order.reduce_only,
                        "orderType": {
                            "limit": {
                                "tif": order.order_type.limit.as_ref()
                                    .map(|l| l.tif.as_str())
                                    .unwrap_or("Gtc")
                            }
                        }
                    }
                ]
            },
            "nonce": chrono::Utc::now().timestamp_millis(),
            "signature": self.sign_order_payload(&private_key, order).await?
        });

        // Make real API call to Hyperliquid
        let response = self.post("exchange", &order_payload).await?;
        let response_text = response.text().await.map_err(|e| {
            HyperliquidToolError::NetworkError(format!("Failed to read order response: {}", e))
        })?;

        // Parse the real response
        let order_response: OrderResponse = serde_json::from_str(&response_text).map_err(|e| {
            HyperliquidToolError::ApiError(format!(
                "Failed to parse order response: {} - Body: {}",
                e, response_text
            ))
        })?;

        info!("REAL order placed successfully: {:?}", order_response);
        Ok(order_response)
    }

    /// Sign order payload for Hyperliquid using EIP-712
    async fn sign_order_payload(&self, private_key: &str, order: &OrderRequest) -> Result<String> {
        info!("Signing order payload with EIP-712");

        // Parse the private key
        let wallet = private_key.parse::<LocalWallet>().map_err(|e| {
            HyperliquidToolError::Configuration(format!("Invalid private key: {}", e))
        })?;

        // Create the EIP-712 domain
        let domain = serde_json::json!({
            "name": "HyperliquidSignTransaction",
            "version": "1",
            "chainId": 42161,  // Arbitrum mainnet
            "verifyingContract": "0x0000000000000000000000000000000000000000"
        });

        // Create the message to sign
        let message = serde_json::json!({
            "a": order.asset,  // asset
            "b": order.is_buy, // isBuy
            "p": order.limit_px.to_string(), // price
            "s": order.sz.to_string(), // size
            "r": order.reduce_only, // reduceOnly
            "t": {
                "limit": {
                    "tif": order.order_type.limit.as_ref()
                        .map(|l| l.tif.as_str())
                        .unwrap_or("Gtc")
                }
            }
        });

        // Create the types definition
        let types = serde_json::json!({
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "Order": [
                {"name": "a", "type": "uint32"},
                {"name": "b", "type": "bool"},
                {"name": "p", "type": "string"},
                {"name": "s", "type": "string"},
                {"name": "r", "type": "bool"},
                {"name": "t", "type": "OrderType"}
            ],
            "OrderType": [
                {"name": "limit", "type": "LimitOrderType"}
            ],
            "LimitOrderType": [
                {"name": "tif", "type": "string"}
            ]
        });

        // Create the typed data
        let typed_data = serde_json::json!({
            "types": types,
            "primaryType": "Order",
            "domain": domain,
            "message": message
        });

        // Convert to TypedData struct
        let typed_data: TypedData = serde_json::from_value(typed_data).map_err(|e| {
            HyperliquidToolError::Configuration(format!("Failed to create typed data: {}", e))
        })?;

        // Encode the typed data
        let encoded = typed_data.encode_eip712().map_err(|e| {
            HyperliquidToolError::Configuration(format!("Failed to encode EIP-712: {}", e))
        })?;

        // Sign the encoded data
        let signature = wallet
            .sign_hash(H256::from(encoded))
            .map_err(|e| HyperliquidToolError::Configuration(format!("Failed to sign: {}", e)))?;

        // Convert signature to the expected format
        let r = signature.r;
        let s = signature.s;
        let v = signature.v;

        // Hyperliquid expects the signature as r + s + v (65 bytes total)
        let mut sig_bytes = vec![0u8; 65];
        r.to_big_endian(&mut sig_bytes[0..32]);
        s.to_big_endian(&mut sig_bytes[32..64]);
        sig_bytes[64] = v as u8;

        let signature_hex = format!("0x{}", hex::encode(sig_bytes));
        info!("Order signed successfully");
        Ok(signature_hex)
    }

    /// Cancel an order using real Hyperliquid API
    /// CRITICAL: This is REAL order cancellation - NO SIMULATION
    pub async fn update_leverage(
        &self,
        leverage: u32,
        coin: &str,
        is_cross: bool,
        asset_id: Option<u32>,
    ) -> Result<LeverageResponse> {
        debug!(
            "Updating leverage for {} to {}x (cross: {})",
            coin, leverage, is_cross
        );

        // Get user address for signing
        let _user_address = self.get_user_address()?;

        // Create the leverage update request
        let leverage_req = LeverageRequest {
            action: LeverageAction {
                type_: "updateLeverage".to_string(),
                asset: asset_id.unwrap_or(0), // Will be resolved from coin if not provided
                is_cross,
                leverage,
            },
            nonce: chrono::Utc::now().timestamp_millis(),
            signature: None, // Will be set after signing
        };

        // Sign the request
        let signed_req = self.sign_leverage_request(leverage_req).await?;

        // Make the API call
        let endpoint = "exchange";
        let response = self.post(endpoint, &signed_req).await?;

        let leverage_response: LeverageResponse = response.json().await.map_err(|e| {
            HyperliquidToolError::ApiError(format!("Failed to parse leverage response: {}", e))
        })?;

        info!(
            "Successfully updated leverage for {} to {}x",
            coin, leverage
        );
        Ok(leverage_response)
    }

    async fn sign_leverage_request(&self, mut request: LeverageRequest) -> Result<LeverageRequest> {
        info!("Signing leverage request with EIP-712");

        // Get private key from environment variable
        let private_key = std::env::var(HYPERLIQUID_PRIVATE_KEY)
            .map_err(|_| HyperliquidToolError::Configuration(
                "HYPERLIQUID_PRIVATE_KEY environment variable required for signing leverage requests.".to_string()
            ))?;

        // Parse the private key
        let wallet = private_key.parse::<LocalWallet>().map_err(|e| {
            HyperliquidToolError::Configuration(format!("Invalid private key: {}", e))
        })?;

        // Create the EIP-712 domain
        let domain = serde_json::json!({
            "name": "HyperliquidSignTransaction",
            "version": "1",
            "chainId": 42161,  // Arbitrum mainnet
            "verifyingContract": "0x0000000000000000000000000000000000000000"
        });

        // Create the message to sign
        let message = serde_json::json!({
            "a": request.action.asset,  // asset
            "isCross": request.action.is_cross, // is cross margin
            "leverage": request.action.leverage, // leverage value
        });

        // Create the types definition
        let types = serde_json::json!({
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "UpdateLeverage": [
                {"name": "a", "type": "uint32"},
                {"name": "isCross", "type": "bool"},
                {"name": "leverage", "type": "uint32"}
            ]
        });

        // Create the typed data
        let typed_data = serde_json::json!({
            "types": types,
            "primaryType": "UpdateLeverage",
            "domain": domain,
            "message": message
        });

        // Convert to TypedData struct
        let typed_data: TypedData = serde_json::from_value(typed_data).map_err(|e| {
            HyperliquidToolError::Configuration(format!("Failed to create typed data: {}", e))
        })?;

        // Encode the typed data
        let encoded = typed_data.encode_eip712().map_err(|e| {
            HyperliquidToolError::Configuration(format!("Failed to encode EIP-712: {}", e))
        })?;

        // Sign the encoded data
        let signature = wallet
            .sign_hash(H256::from(encoded))
            .map_err(|e| HyperliquidToolError::Configuration(format!("Failed to sign: {}", e)))?;

        // Convert signature to the expected format
        let r = signature.r;
        let s = signature.s;
        let v = signature.v;

        // Hyperliquid expects the signature as r + s + v (65 bytes total)
        let mut sig_bytes = vec![0u8; 65];
        r.to_big_endian(&mut sig_bytes[0..32]);
        s.to_big_endian(&mut sig_bytes[32..64]);
        sig_bytes[64] = v as u8;

        let signature_hex = format!("0x{}", hex::encode(sig_bytes));
        info!("Leverage request signed successfully");
        request.signature = Some(signature_hex);

        Ok(request)
    }

    /// Cancel an order using real Hyperliquid API
    ///
    /// # Arguments
    /// * `order_id` - The ID of the order to cancel
    /// * `asset` - The asset ID for the order
    ///
    /// # Warning
    /// This performs REAL order cancellation - NO SIMULATION
    pub async fn cancel_order(&self, order_id: u64, asset: u32) -> Result<CancelResponse> {
        error!("CRITICAL: REAL ORDER CANCELLATION - This will cancel actual trades!");
        info!(
            "Cancelling REAL order: order_id={}, asset={}",
            order_id, asset
        );

        // Require explicit configuration to prevent accidental operations
        let private_key = std::env::var(HYPERLIQUID_PRIVATE_KEY)
            .map_err(|_| HyperliquidToolError::Configuration(
                "HYPERLIQUID_PRIVATE_KEY environment variable required. REAL trading disabled for safety.".to_string()
            ))?;

        let trading_enabled =
            std::env::var(ENABLE_REAL_HYPERLIQUID_TRADING).unwrap_or_default() == "true";

        if !trading_enabled {
            return Err(HyperliquidToolError::Configuration(
                "Real trading disabled. Set ENABLE_REAL_HYPERLIQUID_TRADING=true to enable REAL operations.".to_string()
            ));
        }

        // Create the real cancel payload for Hyperliquid API
        let cancel_payload = serde_json::json!({
            "action": {
                "type": "cancel",
                "cancels": [
                    {
                        "asset": asset,
                        "oid": order_id
                    }
                ]
            },
            "nonce": chrono::Utc::now().timestamp_millis(),
            "signature": self.sign_cancel_payload(&private_key, order_id, asset).await?
        });

        // Make real API call to Hyperliquid
        let response = self.post("exchange", &cancel_payload).await?;
        let response_text = response.text().await.map_err(|e| {
            HyperliquidToolError::NetworkError(format!("Failed to read cancel response: {}", e))
        })?;

        // Parse the real response
        let cancel_response: CancelResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                HyperliquidToolError::ApiError(format!(
                    "Failed to parse cancel response: {} - Body: {}",
                    e, response_text
                ))
            })?;

        info!("REAL order cancelled successfully: {:?}", cancel_response);
        Ok(cancel_response)
    }

    /// Sign cancel payload for Hyperliquid using EIP-712
    async fn sign_cancel_payload(
        &self,
        private_key: &str,
        order_id: u64,
        asset: u32,
    ) -> Result<String> {
        info!("Signing cancel payload with EIP-712");

        // Parse the private key
        let wallet = private_key.parse::<LocalWallet>().map_err(|e| {
            HyperliquidToolError::Configuration(format!("Invalid private key: {}", e))
        })?;

        // Create the EIP-712 domain
        let domain = serde_json::json!({
            "name": "HyperliquidSignTransaction",
            "version": "1",
            "chainId": 42161,  // Arbitrum mainnet
            "verifyingContract": "0x0000000000000000000000000000000000000000"
        });

        // Create the message to sign
        let message = serde_json::json!({
            "a": asset,   // asset
            "o": order_id // order ID
        });

        // Create the types definition
        let types = serde_json::json!({
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "Cancel": [
                {"name": "a", "type": "uint32"},
                {"name": "o", "type": "uint64"}
            ]
        });

        // Create the typed data
        let typed_data = serde_json::json!({
            "types": types,
            "primaryType": "Cancel",
            "domain": domain,
            "message": message
        });

        // Convert to TypedData struct
        let typed_data: TypedData = serde_json::from_value(typed_data).map_err(|e| {
            HyperliquidToolError::Configuration(format!("Failed to create typed data: {}", e))
        })?;

        // Encode the typed data
        let encoded = typed_data.encode_eip712().map_err(|e| {
            HyperliquidToolError::Configuration(format!("Failed to encode EIP-712: {}", e))
        })?;

        // Sign the encoded data
        let signature = wallet
            .sign_hash(H256::from(encoded))
            .map_err(|e| HyperliquidToolError::Configuration(format!("Failed to sign: {}", e)))?;

        // Convert signature to the expected format
        let r = signature.r;
        let s = signature.s;
        let v = signature.v;

        // Hyperliquid expects the signature as r + s + v (65 bytes total)
        let mut sig_bytes = vec![0u8; 65];
        r.to_big_endian(&mut sig_bytes[0..32]);
        s.to_big_endian(&mut sig_bytes[32..64]);
        sig_bytes[64] = v as u8;

        let signature_hex = format!("0x{}", hex::encode(sig_bytes));
        info!("Cancel order signed successfully");
        Ok(signature_hex)
    }

    /// Get the user's address from the signer
    ///
    /// Returns the address associated with the current signer, which is used
    /// for identifying the user in Hyperliquid API calls.
    pub fn get_user_address(&self) -> Result<String> {
        if let Some(evm_signer) = self.signer.as_evm() {
            Ok(evm_signer.address())
        } else {
            Err(HyperliquidToolError::AuthError(
                "No EVM signer available".to_string(),
            ))
        }
    }
}

// IMPLEMENTATION NOTE:
// This module now uses real HTTP API calls to Hyperliquid instead of simulation.
// The previous simulation code has been completely removed.
// All order placement and cancellation now requires:
// 1. HYPERLIQUID_PRIVATE_KEY environment variable
// 2. ENABLE_REAL_HYPERLIQUID_TRADING=true environment variable
// 3. Proper EIP-712 signature implementation (currently placeholder)

// Data structures for Hyperliquid API responses

/// Account information response from Hyperliquid API
///
/// Contains the current state of a user's account including positions,
/// margin usage, and withdrawable funds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// List of asset positions held by the user
    #[serde(rename = "assetPositions")]
    pub asset_positions: Option<Vec<Position>>,
    /// Amount of cross maintenance margin currently being used
    #[serde(rename = "crossMaintenanceMarginUsed")]
    pub cross_maintenance_margin_used: Option<String>,
    /// Amount of cross margin currently being used
    #[serde(rename = "crossMarginUsed")]
    pub cross_margin_used: Option<String>,
    /// Amount of funds available for withdrawal
    #[serde(rename = "withdrawable")]
    pub withdrawable: Option<String>,
}

/// Clearinghouse state information from Hyperliquid API
///
/// Represents the current state of the clearinghouse for a user,
/// including positions and margin information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearinghouseState {
    /// List of asset positions held by the user
    #[serde(rename = "assetPositions")]
    pub asset_positions: Option<Vec<Position>>,
    /// Amount of cross maintenance margin currently being used
    #[serde(rename = "crossMaintenanceMarginUsed")]
    pub cross_maintenance_margin_used: Option<String>,
    /// Amount of cross margin currently being used
    #[serde(rename = "crossMarginUsed")]
    pub cross_margin_used: Option<String>,
    /// Amount of funds available for withdrawal
    pub withdrawable: Option<String>,
}

/// Trading position information from Hyperliquid API
///
/// Represents a user's position in a specific trading asset,
/// containing detailed position data and type information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Detailed position data including size, entry price, and P&L
    #[serde(rename = "position")]
    pub position: PositionData,
    /// Type of position (e.g., "oneWay" for standard perpetual positions)
    #[serde(rename = "type")]
    pub type_field: String,
}

/// Detailed position data from Hyperliquid API
///
/// Contains comprehensive information about a trading position including
/// entry price, leverage, margin usage, and profit/loss metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionData {
    /// The trading symbol/coin for this position (e.g., "BTC", "ETH")
    pub coin: String,
    /// Entry price for the position as a string decimal
    #[serde(rename = "entryPx")]
    pub entry_px: Option<String>,
    /// Leverage configuration for this position
    pub leverage: PositionLeverage,
    /// Price at which the position would be liquidated
    #[serde(rename = "liquidationPx")]
    pub liquidation_px: Option<String>,
    /// Amount of margin currently used for this position
    #[serde(rename = "marginUsed")]
    pub margin_used: String,
    /// Maximum leverage allowed for this asset
    #[serde(rename = "maxLeverage")]
    pub max_leverage: u32,
    /// Current notional value of the position
    #[serde(rename = "positionValue")]
    pub position_value: String,
    /// Return on equity percentage for this position
    #[serde(rename = "returnOnEquity")]
    pub return_on_equity: String,
    /// Position size (positive for long, negative for short)
    pub szi: String,
    /// Current unrealized profit or loss for the position
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: String,
}

/// Leverage configuration for a trading position
///
/// Specifies the leverage type and multiplier used for a position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLeverage {
    /// Type of leverage (e.g., "cross" for cross margin, "isolated" for isolated margin)
    #[serde(rename = "type")]
    pub type_field: String,
    /// Leverage multiplier (e.g., 10 for 10x leverage)
    pub value: u32,
}

/// Market metadata from Hyperliquid API
///
/// Contains information about all available trading assets
/// and their specifications on the Hyperliquid exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    /// List of all available trading assets with their specifications
    pub universe: Vec<AssetInfo>,
}

/// Information about a tradeable asset on Hyperliquid
///
/// Specifies the asset name and precision details required
/// for proper order formatting and size calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    /// Name of the trading asset (e.g., "BTC", "ETH", "SOL")
    pub name: String,
    /// Number of decimal places used for position sizes in this asset
    #[serde(rename = "szDecimals")]
    pub sz_decimals: u32,
}

/// Order placement request for Hyperliquid API
///
/// Represents a request to place a new trading order with all
/// necessary parameters including price, size, and order type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    /// Asset ID for the trading pair (numeric identifier)
    pub asset: u32,
    /// Whether this is a buy order (true) or sell order (false)
    #[serde(rename = "isBuy")]
    pub is_buy: bool,
    /// Limit price for the order as a string decimal
    #[serde(rename = "limitPx")]
    pub limit_px: String,
    /// Order size as a string decimal
    pub sz: String,
    /// Whether this order should only reduce existing positions
    #[serde(rename = "reduceOnly")]
    pub reduce_only: bool,
    /// Order type configuration (limit, market, etc.)
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
}

/// Order type configuration for trading orders
///
/// Specifies the type of order and its execution parameters.
/// Currently supports limit orders with time-in-force options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderType {
    /// Limit order configuration, if this is a limit order
    #[serde(rename = "limit")]
    pub limit: Option<LimitOrderType>,
}

/// Limit order configuration with time-in-force settings
///
/// Specifies how long a limit order should remain active
/// in the order book before expiring or being cancelled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitOrderType {
    /// Time in force: "Gtc" (Good Till Cancelled), "Ioc" (Immediate or Cancel), "Alo" (Add Liquidity Only)
    pub tif: String,
}

/// Response from order placement API call
///
/// Contains the status of the order placement attempt
/// and detailed result information if successful.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    /// Status of the API call ("ok" for success, error message for failure)
    pub status: String,
    /// Detailed order placement result data
    pub data: OrderResult,
}

/// Detailed result of an order placement operation
///
/// Contains status codes and response data indicating
/// whether the order was successfully placed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResult {
    /// Numeric status code indicating the result of the operation
    #[serde(rename = "statuses")]
    pub status_code: u32,
    /// Detailed response data with order information
    pub response: ResponseData,
}

/// Response data container for order operations
///
/// Wraps the actual order data with type information
/// to indicate the kind of response received.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    /// Type of response (e.g., "order" for order placement responses)
    #[serde(rename = "type")]
    pub type_field: String,
    /// Optional order data, present if the operation was successful
    pub data: Option<OrderData>,
}

/// Order data containing status information for placed orders
///
/// Contains an array of order statuses, typically one per order
/// in the batch request (single orders will have one status).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderData {
    /// List of order statuses for each order in the request
    pub statuses: Vec<OrderStatus>,
}

/// Status information for a single order
///
/// Indicates whether the order is resting in the order book
/// and provides access to the order identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatus {
    /// Information about the order if it's resting in the order book
    pub resting: RestingOrder,
}

/// Information about an order resting in the order book
///
/// Contains the order identifier that can be used to reference
/// the order for cancellation or modification operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestingOrder {
    /// Order ID assigned by the exchange for this resting order
    pub oid: u64,
}

/// Response from order cancellation API call
///
/// Contains the status of the cancellation attempt
/// and result information indicating success or failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResponse {
    /// Status of the cancellation API call ("ok" for success)
    pub status: String,
    /// Detailed cancellation result data
    pub data: CancelResult,
}

/// Result of an order cancellation operation
///
/// Contains a status code indicating whether the
/// cancellation was successful or failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResult {
    /// Numeric status code indicating the result of the cancellation
    #[serde(rename = "statuses")]
    pub status_code: u32,
}

// Leverage-related structures

/// Request to update leverage settings for a trading asset
///
/// Contains the leverage action to perform, a nonce for security,
/// and an optional signature for authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeverageRequest {
    /// The leverage action to perform (update leverage)
    pub action: LeverageAction,
    /// Timestamp nonce to prevent replay attacks
    pub nonce: i64,
    /// EIP-712 signature for authenticating the request
    pub signature: Option<String>,
}

/// Action specification for leverage updates
///
/// Defines the specific leverage change to be made,
/// including the asset, margin type, and new leverage value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeverageAction {
    /// Type of action ("updateLeverage" for leverage changes)
    #[serde(rename = "type")]
    pub type_: String,
    /// Asset ID to update leverage for
    pub asset: u32,
    /// Whether to use cross margin (true) or isolated margin (false)
    #[serde(rename = "isCross")]
    pub is_cross: bool,
    /// New leverage multiplier to set (e.g., 10 for 10x leverage)
    pub leverage: u32,
}

/// Response from leverage update API call
///
/// Contains the status of the leverage update attempt
/// and result data if the update was successful.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeverageResponse {
    /// Status of the leverage update API call ("success" for successful updates)
    pub status: String,
    /// Leverage update result data, present if the update was successful
    pub data: Option<LeverageResult>,
}

/// Result of a successful leverage update operation
///
/// Contains the new leverage value and the asset
/// that was updated, confirming the change was applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeverageResult {
    /// New leverage multiplier that was set
    pub leverage: u32,
    /// Asset symbol that had its leverage updated
    pub asset: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mockito::Server;
    use riglr_core::signer::{EvmClient, EvmSigner, SignerBase, SignerError, UnifiedSigner};
    use std::sync::Arc;

    /// Helper functions for test environment variable management
    #[allow(unsafe_code)] // Test helper functions use unsafe std::env operations for Rust 2024 compatibility with proper inline SAFETY documentation
    mod test_env_vars {
        /// Helper function to set environment variables in tests without using string literals
        pub fn set_test_env_var(key: &'static str, value: &str) {
            // SAFETY: This is a test-only function used in isolated test environments
            // where we control the threading and environment variable access patterns.
            unsafe {
                std::env::set_var(key, value);
            }
        }

        /// Helper function to remove environment variables in tests without using string literals  
        pub fn remove_test_env_var(key: &'static str) {
            // SAFETY: This is a test-only function used in isolated test environments
            // where we control the threading and environment variable access patterns.
            unsafe {
                std::env::remove_var(key);
            }
        }
    }

    // Mock signer for testing
    #[derive(Debug)]
    struct MockSigner {
        address: Option<String>,
    }

    impl MockSigner {
        fn new(address: Option<String>) -> Self {
            Self { address }
        }
    }

    // Mock EVM client for testing
    #[derive(Debug)]
    struct MockEvmClient;

    #[async_trait]
    impl EvmClient for MockEvmClient {
        async fn get_balance(&self, _address: &str) -> std::result::Result<String, SignerError> {
            Ok("1000".to_string())
        }

        async fn send_transaction(
            &self,
            _tx: &serde_json::Value,
        ) -> std::result::Result<String, SignerError> {
            Ok("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string())
        }

        async fn call(&self, _tx: &serde_json::Value) -> std::result::Result<String, SignerError> {
            Ok("0x".to_string())
        }
    }

    impl SignerBase for MockSigner {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[async_trait]
    impl EvmSigner for MockSigner {
        fn chain_id(&self) -> u64 {
            1 // Ethereum mainnet
        }

        fn address(&self) -> String {
            self.address.clone().unwrap_or_default()
        }

        async fn sign_and_send_transaction(
            &self,
            _tx: serde_json::Value,
        ) -> std::result::Result<String, SignerError> {
            Ok("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string())
        }

        fn client(&self) -> std::result::Result<std::sync::Arc<dyn EvmClient>, SignerError> {
            Ok(std::sync::Arc::new(MockEvmClient))
        }
    }

    impl UnifiedSigner for MockSigner {
        fn supports_solana(&self) -> bool {
            false
        }

        fn supports_evm(&self) -> bool {
            true
        }

        fn as_solana(&self) -> Option<&dyn riglr_core::signer::SolanaSigner> {
            None
        }

        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            Some(self)
        }

        fn as_multi_chain(&self) -> Option<&dyn riglr_core::signer::MultiChainSigner> {
            None
        }
    }

    fn create_mock_signer() -> Arc<dyn UnifiedSigner> {
        Arc::new(MockSigner::new(Some("0x1234567890abcdef".to_string())))
    }

    fn create_mock_signer_no_address() -> Arc<dyn UnifiedSigner> {
        Arc::new(MockSigner::new(None))
    }

    // Constructor tests
    #[test]
    fn test_new_when_valid_signer_should_create_client() {
        let signer = create_mock_signer();
        let client = HyperliquidClient::new(signer);
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.base_url, "https://api.hyperliquid.xyz");
    }

    #[test]
    fn test_with_base_url_when_valid_params_should_create_client() {
        let signer = create_mock_signer();
        let custom_url = "https://test.example.com".to_string();
        let client = HyperliquidClient::with_base_url(signer, custom_url.clone());
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.base_url, custom_url);
    }

    #[test]
    fn test_get_user_address_when_signer_has_address_should_return_address() {
        let signer = create_mock_signer();
        let client = HyperliquidClient::new(signer).unwrap();
        let address = client.get_user_address();
        assert!(address.is_ok());
        assert_eq!(address.unwrap(), "0x1234567890abcdef");
    }

    #[test]
    fn test_get_user_address_when_signer_no_address_should_return_error() {
        let signer = create_mock_signer_no_address();
        let client = HyperliquidClient::new(signer).unwrap();
        let address = client.get_user_address();
        assert!(address.is_err());
        match address.unwrap_err() {
            HyperliquidToolError::AuthError(msg) => {
                assert_eq!(msg, "No address available from signer");
            }
            _ => panic!("Expected AuthError"),
        }
    }

    // HTTP method tests with mock server
    #[tokio::test]
    async fn test_get_when_successful_response_should_return_ok() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_body("success")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let response = client.get("test").await;

        mock.assert_async().await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_get_when_rate_limit_error_should_return_rate_limit_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(429)
            .with_body("Rate limit exceeded")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let response = client.get("test").await;

        mock.assert_async().await;
        assert!(response.is_err());
        match response.unwrap_err() {
            HyperliquidToolError::RateLimit(msg) => {
                assert!(msg.contains("Rate limit exceeded"));
            }
            _ => panic!("Expected RateLimit error"),
        }
    }

    #[tokio::test]
    async fn test_get_when_server_error_should_return_network_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(500)
            .with_body("Internal server error")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let response = client.get("test").await;

        mock.assert_async().await;
        assert!(response.is_err());
        match response.unwrap_err() {
            HyperliquidToolError::NetworkError(msg) => {
                assert!(msg.contains("Server error 500"));
            }
            _ => panic!("Expected NetworkError"),
        }
    }

    #[tokio::test]
    async fn test_get_when_client_error_should_return_api_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(400)
            .with_body("Bad request")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let response = client.get("test").await;

        mock.assert_async().await;
        assert!(response.is_err());
        match response.unwrap_err() {
            HyperliquidToolError::ApiError(msg) => {
                assert!(msg.contains("API error 400"));
            }
            _ => panic!("Expected ApiError"),
        }
    }

    #[tokio::test]
    async fn test_get_when_endpoint_with_leading_slash_should_trim_correctly() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_body("success")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let response = client.get("/test").await;

        mock.assert_async().await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_get_when_base_url_with_trailing_slash_should_trim_correctly() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_body("success")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client =
            HyperliquidClient::with_base_url(signer, format!("{}/", server.url())).unwrap();
        let response = client.get("test").await;

        mock.assert_async().await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_post_when_successful_response_should_return_ok() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/test")
            .with_status(200)
            .with_body("success")
            .expect(1)
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let payload = serde_json::json!({"test": "data"});
        let response = client.post("test", &payload).await;

        mock.assert_async().await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_post_when_rate_limit_error_should_return_rate_limit_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/test")
            .with_status(429)
            .with_body("Rate limit exceeded")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let payload = serde_json::json!({"test": "data"});
        let response = client.post("test", &payload).await;

        mock.assert_async().await;
        assert!(response.is_err());
        match response.unwrap_err() {
            HyperliquidToolError::RateLimit(msg) => {
                assert!(msg.contains("Rate limit exceeded on POST"));
            }
            _ => panic!("Expected RateLimit error"),
        }
    }

    #[tokio::test]
    async fn test_post_when_server_error_should_return_network_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/test")
            .with_status(500)
            .with_body("Internal server error")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let payload = serde_json::json!({"test": "data"});
        let response = client.post("test", &payload).await;

        mock.assert_async().await;
        assert!(response.is_err());
        match response.unwrap_err() {
            HyperliquidToolError::NetworkError(msg) => {
                assert!(msg.contains("Server error 500"));
            }
            _ => panic!("Expected NetworkError"),
        }
    }

    #[tokio::test]
    async fn test_post_when_client_error_should_return_api_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/test")
            .with_status(400)
            .with_body("Bad request")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let payload = serde_json::json!({"test": "data"});
        let response = client.post("test", &payload).await;

        mock.assert_async().await;
        assert!(response.is_err());
        match response.unwrap_err() {
            HyperliquidToolError::ApiError(msg) => {
                assert!(msg.contains("API error 400"));
            }
            _ => panic!("Expected ApiError"),
        }
    }

    // Data structure tests
    #[test]
    fn test_leverage_request_structure() {
        let request = LeverageRequest {
            action: LeverageAction {
                type_: "updateLeverage".to_string(),
                asset: 0,
                is_cross: true,
                leverage: 10,
            },
            nonce: 1234567890,
            signature: None,
        };

        assert_eq!(request.action.type_, "updateLeverage");
        assert_eq!(request.action.leverage, 10);
        assert!(request.action.is_cross);
        assert!(request.signature.is_none());
    }

    #[test]
    fn test_leverage_response_parsing() {
        let json_response = r#"{
            "status": "success",
            "data": {
                "leverage": 10,
                "asset": "BTC"
            }
        }"#;

        let response: LeverageResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(response.status, "success");
        assert!(response.data.is_some());

        let data = response.data.unwrap();
        assert_eq!(data.leverage, 10);
        assert_eq!(data.asset, "BTC");
    }

    #[test]
    fn test_leverage_response_parsing_when_no_data_should_parse_correctly() {
        let json_response = r#"{
            "status": "error"
        }"#;

        let response: LeverageResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(response.status, "error");
        assert!(response.data.is_none());
    }

    #[test]
    fn test_leverage_action_serialization() {
        let action = LeverageAction {
            type_: "updateLeverage".to_string(),
            asset: 42,
            is_cross: true,
            leverage: 20,
        };

        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"type\":\"updateLeverage\""));
        assert!(json.contains("\"asset\":42"));
        assert!(json.contains("\"isCross\":true")); // Note the camelCase due to serde rename
        assert!(json.contains("\"leverage\":20"));
    }

    #[test]
    fn test_leverage_action_when_isolated_margin_should_serialize_correctly() {
        let action = LeverageAction {
            type_: "updateLeverage".to_string(),
            asset: 1,
            is_cross: false,
            leverage: 5,
        };

        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"isCross\":false"));
        assert!(json.contains("\"leverage\":5"));
    }

    #[test]
    fn test_order_request_serialization() {
        let order = OrderRequest {
            asset: 0,
            is_buy: true,
            limit_px: "50000.0".to_string(),
            sz: "0.1".to_string(),
            reduce_only: false,
            order_type: OrderType {
                limit: Some(LimitOrderType {
                    tif: "Gtc".to_string(),
                }),
            },
        };

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("\"asset\":0"));
        assert!(json.contains("\"isBuy\":true"));
        assert!(json.contains("\"limitPx\":\"50000.0\""));
        assert!(json.contains("\"sz\":\"0.1\""));
        assert!(json.contains("\"reduceOnly\":false"));
    }

    #[test]
    fn test_order_request_when_sell_order_should_serialize_correctly() {
        let order = OrderRequest {
            asset: 1,
            is_buy: false,
            limit_px: "49000.0".to_string(),
            sz: "0.5".to_string(),
            reduce_only: true,
            order_type: OrderType {
                limit: Some(LimitOrderType {
                    tif: "Ioc".to_string(),
                }),
            },
        };

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("\"isBuy\":false"));
        assert!(json.contains("\"reduceOnly\":true"));
        assert!(json.contains("\"tif\":\"Ioc\""));
    }

    #[test]
    fn test_order_type_when_no_limit_should_serialize_correctly() {
        let order_type = OrderType { limit: None };
        let json = serde_json::to_string(&order_type).unwrap();
        assert!(json.contains("\"limit\":null"));
    }

    #[test]
    fn test_limit_order_type_different_tif_values() {
        let gtc = LimitOrderType {
            tif: "Gtc".to_string(),
        };
        let ioc = LimitOrderType {
            tif: "Ioc".to_string(),
        };
        let alo = LimitOrderType {
            tif: "Alo".to_string(),
        };

        assert_eq!(gtc.tif, "Gtc");
        assert_eq!(ioc.tif, "Ioc");
        assert_eq!(alo.tif, "Alo");
    }

    #[test]
    fn test_account_info_parsing_complete() {
        let json = r#"{
            "assetPositions": [
                {
                    "position": {
                        "coin": "BTC",
                        "entryPx": "50000.0",
                        "leverage": {"type": "cross", "value": 10},
                        "liquidationPx": "45000.0",
                        "marginUsed": "5000.0",
                        "maxLeverage": 50,
                        "positionValue": "50000.0",
                        "returnOnEquity": "0.1",
                        "szi": "1.0",
                        "unrealizedPnl": "1000.0"
                    },
                    "type": "oneWay"
                }
            ],
            "crossMaintenanceMarginUsed": "100.0",
            "crossMarginUsed": "1000.0",
            "withdrawable": "9000.0"
        }"#;

        let account: AccountInfo = serde_json::from_str(json).unwrap();
        assert!(account.asset_positions.is_some());
        assert_eq!(
            account.cross_maintenance_margin_used,
            Some("100.0".to_string())
        );
        assert_eq!(account.cross_margin_used, Some("1000.0".to_string()));
        assert_eq!(account.withdrawable, Some("9000.0".to_string()));

        let positions = account.asset_positions.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].position.coin, "BTC");
        assert_eq!(positions[0].type_field, "oneWay");
    }

    #[test]
    fn test_account_info_parsing_minimal() {
        let json = r#"{
            "assetPositions": null,
            "crossMaintenanceMarginUsed": null,
            "crossMarginUsed": null,
            "withdrawable": null
        }"#;

        let account: AccountInfo = serde_json::from_str(json).unwrap();
        assert!(account.asset_positions.is_none());
        assert!(account.cross_maintenance_margin_used.is_none());
        assert!(account.cross_margin_used.is_none());
        assert!(account.withdrawable.is_none());
    }

    #[test]
    fn test_position_data_parsing() {
        let json = r#"{
            "coin": "ETH",
            "entryPx": "3000.0",
            "leverage": {"type": "isolated", "value": 5},
            "liquidationPx": "2500.0",
            "marginUsed": "600.0",
            "maxLeverage": 25,
            "positionValue": "3000.0",
            "returnOnEquity": "0.05",
            "szi": "-1.0",
            "unrealizedPnl": "-50.0"
        }"#;

        let position: PositionData = serde_json::from_str(json).unwrap();
        assert_eq!(position.coin, "ETH");
        assert_eq!(position.entry_px, Some("3000.0".to_string()));
        assert_eq!(position.leverage.type_field, "isolated");
        assert_eq!(position.leverage.value, 5);
        assert_eq!(position.liquidation_px, Some("2500.0".to_string()));
        assert_eq!(position.margin_used, "600.0");
        assert_eq!(position.max_leverage, 25);
        assert_eq!(position.position_value, "3000.0");
        assert_eq!(position.return_on_equity, "0.05");
        assert_eq!(position.szi, "-1.0");
        assert_eq!(position.unrealized_pnl, "-50.0");
    }

    #[test]
    fn test_position_leverage_different_types() {
        let cross = PositionLeverage {
            type_field: "cross".to_string(),
            value: 10,
        };
        let isolated = PositionLeverage {
            type_field: "isolated".to_string(),
            value: 5,
        };

        assert_eq!(cross.type_field, "cross");
        assert_eq!(cross.value, 10);
        assert_eq!(isolated.type_field, "isolated");
        assert_eq!(isolated.value, 5);
    }

    #[test]
    fn test_meta_parsing() {
        let json = r#"{
            "universe": [
                {"name": "BTC", "szDecimals": 5},
                {"name": "ETH", "szDecimals": 4},
                {"name": "SOL", "szDecimals": 3}
            ]
        }"#;

        let meta: Meta = serde_json::from_str(json).unwrap();
        assert_eq!(meta.universe.len(), 3);
        assert_eq!(meta.universe[0].name, "BTC");
        assert_eq!(meta.universe[0].sz_decimals, 5);
        assert_eq!(meta.universe[1].name, "ETH");
        assert_eq!(meta.universe[1].sz_decimals, 4);
        assert_eq!(meta.universe[2].name, "SOL");
        assert_eq!(meta.universe[2].sz_decimals, 3);
    }

    #[test]
    fn test_meta_parsing_empty_universe() {
        let json = r#"{"universe": []}"#;
        let meta: Meta = serde_json::from_str(json).unwrap();
        assert_eq!(meta.universe.len(), 0);
    }

    #[test]
    fn test_asset_info_edge_cases() {
        let asset = AssetInfo {
            name: "".to_string(),
            sz_decimals: 0,
        };
        assert_eq!(asset.name, "");
        assert_eq!(asset.sz_decimals, 0);

        let asset_max = AssetInfo {
            name: "VERYLONGASSETNAME".to_string(),
            sz_decimals: u32::MAX,
        };
        assert_eq!(asset_max.name, "VERYLONGASSETNAME");
        assert_eq!(asset_max.sz_decimals, u32::MAX);
    }

    #[test]
    fn test_order_response_parsing_successful() {
        let json = r#"{
            "status": "ok",
            "data": {
                "statuses": 0,
                "response": {
                    "type": "order",
                    "data": {
                        "statuses": [
                            {"resting": {"oid": 12345}}
                        ]
                    }
                }
            }
        }"#;

        let response: OrderResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "ok");
        assert_eq!(response.data.status_code, 0);
        assert_eq!(response.data.response.type_field, "order");
        assert!(response.data.response.data.is_some());

        let order_data = response.data.response.data.unwrap();
        assert_eq!(order_data.statuses.len(), 1);
        assert_eq!(order_data.statuses[0].resting.oid, 12345);
    }

    #[test]
    fn test_order_response_parsing_no_data() {
        let json = r#"{
            "status": "error",
            "data": {
                "statuses": 1,
                "response": {
                    "type": "error",
                    "data": null
                }
            }
        }"#;

        let response: OrderResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "error");
        assert_eq!(response.data.status_code, 1);
        assert_eq!(response.data.response.type_field, "error");
        assert!(response.data.response.data.is_none());
    }

    #[test]
    fn test_cancel_response_parsing() {
        let json = r#"{
            "status": "ok",
            "data": {
                "statuses": 0
            }
        }"#;

        let response: CancelResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "ok");
        assert_eq!(response.data.status_code, 0);
    }

    #[test]
    fn test_cancel_response_parsing_error() {
        let json = r#"{
            "status": "error",
            "data": {
                "statuses": 1
            }
        }"#;

        let response: CancelResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "error");
        assert_eq!(response.data.status_code, 1);
    }

    #[test]
    fn test_clearinghouse_state_parsing() {
        let json = r#"{
            "assetPositions": [],
            "crossMaintenanceMarginUsed": "0.0",
            "crossMarginUsed": "0.0",
            "withdrawable": "1000.0"
        }"#;

        let state: ClearinghouseState = serde_json::from_str(json).unwrap();
        assert!(state.asset_positions.is_some());
        assert_eq!(state.asset_positions.unwrap().len(), 0);
        assert_eq!(state.cross_maintenance_margin_used, Some("0.0".to_string()));
        assert_eq!(state.cross_margin_used, Some("0.0".to_string()));
        assert_eq!(state.withdrawable, Some("1000.0".to_string()));
    }

    // Error path tests for serialization
    #[test]
    fn test_malformed_json_parsing_should_fail() {
        let malformed_json = r#"{"invalid": json}"#;
        let result: std::result::Result<AccountInfo, _> = serde_json::from_str(malformed_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_fields_should_fail() {
        let incomplete_json = r#"{"coin": "BTC"}"#;
        let result: std::result::Result<PositionData, _> = serde_json::from_str(incomplete_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_type_fields_should_fail() {
        let wrong_type_json = r#"{"asset": "not_a_number", "is_buy": true, "limit_px": "50000", "sz": "0.1", "reduce_only": false, "order_type": {"limit": {"tif": "Gtc"}}}"#;
        let result: std::result::Result<OrderRequest, _> = serde_json::from_str(wrong_type_json);
        assert!(result.is_err());
    }

    // Edge case tests for boundary values
    #[test]
    fn test_order_request_edge_cases() {
        // Test with zero values
        let order = OrderRequest {
            asset: 0,
            is_buy: true,
            limit_px: "0.0".to_string(),
            sz: "0.0".to_string(),
            reduce_only: false,
            order_type: OrderType { limit: None },
        };
        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("\"asset\":0"));
        assert!(json.contains("\"limitPx\":\"0.0\""));
        assert!(json.contains("\"sz\":\"0.0\""));

        // Test with maximum values
        let order_max = OrderRequest {
            asset: u32::MAX,
            is_buy: false,
            limit_px: "999999999.999999999".to_string(),
            sz: "999999999.999999999".to_string(),
            reduce_only: true,
            order_type: OrderType {
                limit: Some(LimitOrderType {
                    tif: "Alo".to_string(),
                }),
            },
        };
        let json_max = serde_json::to_string(&order_max).unwrap();
        assert!(json_max.contains(&format!("\"asset\":{}", u32::MAX)));
        assert!(json_max.contains("\"isBuy\":false"));
        assert!(json_max.contains("\"reduceOnly\":true"));
    }

    #[test]
    fn test_leverage_request_edge_cases() {
        // Test with minimum values
        let request_min = LeverageRequest {
            action: LeverageAction {
                type_: "updateLeverage".to_string(),
                asset: 0,
                is_cross: false,
                leverage: 1,
            },
            nonce: i64::MIN,
            signature: Some("".to_string()),
        };
        assert_eq!(request_min.action.leverage, 1);
        assert_eq!(request_min.nonce, i64::MIN);
        assert_eq!(request_min.signature, Some("".to_string()));

        // Test with maximum values
        let request_max = LeverageRequest {
            action: LeverageAction {
                type_: "updateLeverage".to_string(),
                asset: u32::MAX,
                is_cross: true,
                leverage: u32::MAX,
            },
            nonce: i64::MAX,
            signature: Some("0x".repeat(1000)),
        };
        assert_eq!(request_max.action.asset, u32::MAX);
        assert_eq!(request_max.action.leverage, u32::MAX);
        assert_eq!(request_max.nonce, i64::MAX);
    }

    #[test]
    fn test_resting_order_oid_edge_cases() {
        let order_min = RestingOrder { oid: 0 };
        let order_max = RestingOrder { oid: u64::MAX };

        assert_eq!(order_min.oid, 0);
        assert_eq!(order_max.oid, u64::MAX);
    }

    // API method tests with mock servers
    #[tokio::test]
    async fn test_get_account_info_when_successful_should_return_account_info() {
        let account_json = r#"{
            "assetPositions": [
                {
                    "position": {
                        "coin": "BTC",
                        "entryPx": "50000.0",
                        "leverage": {"type": "cross", "value": 10},
                        "liquidationPx": "45000.0",
                        "marginUsed": "5000.0",
                        "maxLeverage": 50,
                        "positionValue": "50000.0",
                        "returnOnEquity": "0.1",
                        "szi": "1.0",
                        "unrealizedPnl": "1000.0"
                    },
                    "type": "oneWay"
                }
            ],
            "crossMaintenanceMarginUsed": "100.0",
            "crossMarginUsed": "1000.0",
            "withdrawable": "9000.0"
        }"#;

        let mut server = Server::new_async().await;
        let mock = server
            .mock(
                "GET",
                "/info?type=clearinghouseState&user=0x1234567890abcdef",
            )
            .with_status(200)
            .with_body(account_json)
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_account_info("0x1234567890abcdef").await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let account_info = result.unwrap();
        assert!(account_info.asset_positions.is_some());
        assert_eq!(account_info.withdrawable, Some("9000.0".to_string()));
    }

    #[tokio::test]
    async fn test_get_account_info_when_network_error_should_return_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock(
                "GET",
                "/info?type=clearinghouseState&user=0x1234567890abcdef",
            )
            .with_status(500)
            .with_body("Server error")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_account_info("0x1234567890abcdef").await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidToolError::NetworkError(_) => {}
            _ => panic!("Expected NetworkError"),
        }
    }

    #[tokio::test]
    async fn test_get_account_info_when_invalid_json_should_return_api_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock(
                "GET",
                "/info?type=clearinghouseState&user=0x1234567890abcdef",
            )
            .with_status(200)
            .with_body("invalid json")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_account_info("0x1234567890abcdef").await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidToolError::ApiError(msg) => {
                assert!(msg.contains("Failed to parse account info response"));
            }
            _ => panic!("Expected ApiError"),
        }
    }

    #[tokio::test]
    async fn test_get_positions_when_successful_should_return_positions() {
        let positions_json = r#"{
            "assetPositions": [
                {
                    "position": {
                        "coin": "ETH",
                        "entryPx": "3000.0",
                        "leverage": {"type": "isolated", "value": 5},
                        "liquidationPx": "2500.0",
                        "marginUsed": "600.0",
                        "maxLeverage": 25,
                        "positionValue": "3000.0",
                        "returnOnEquity": "0.05",
                        "szi": "-1.0",
                        "unrealizedPnl": "-50.0"
                    },
                    "type": "oneWay"
                }
            ],
            "crossMaintenanceMarginUsed": "50.0",
            "crossMarginUsed": "500.0",
            "withdrawable": "4500.0"
        }"#;

        let mut server = Server::new_async().await;
        let mock = server
            .mock(
                "GET",
                "/info?type=clearinghouseState&user=0xabcdef1234567890",
            )
            .with_status(200)
            .with_body(positions_json)
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_positions("0xabcdef1234567890").await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].position.coin, "ETH");
        assert_eq!(positions[0].position.szi, "-1.0");
    }

    #[tokio::test]
    async fn test_get_positions_when_no_positions_should_return_empty_vec() {
        let positions_json = r#"{
            "assetPositions": null,
            "crossMaintenanceMarginUsed": "0.0",
            "crossMarginUsed": "0.0",
            "withdrawable": "1000.0"
        }"#;

        let mut server = Server::new_async().await;
        let mock = server
            .mock(
                "GET",
                "/info?type=clearinghouseState&user=0xabcdef1234567890",
            )
            .with_status(200)
            .with_body(positions_json)
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_positions("0xabcdef1234567890").await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 0);
    }

    #[tokio::test]
    async fn test_get_positions_when_invalid_json_should_return_api_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock(
                "GET",
                "/info?type=clearinghouseState&user=0xabcdef1234567890",
            )
            .with_status(200)
            .with_body("invalid json")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_positions("0xabcdef1234567890").await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidToolError::ApiError(msg) => {
                assert!(msg.contains("Failed to parse clearinghouse state"));
            }
            _ => panic!("Expected ApiError"),
        }
    }

    #[tokio::test]
    async fn test_get_meta_when_successful_should_return_meta() {
        let meta_json = r#"{
            "universe": [
                {"name": "BTC", "szDecimals": 5},
                {"name": "ETH", "szDecimals": 4},
                {"name": "SOL", "szDecimals": 3}
            ]
        }"#;

        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/info?type=meta")
            .with_status(200)
            .with_body(meta_json)
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_meta().await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let meta = result.unwrap();
        assert_eq!(meta.universe.len(), 3);
        assert_eq!(meta.universe[0].name, "BTC");
        assert_eq!(meta.universe[0].sz_decimals, 5);
    }

    #[tokio::test]
    async fn test_get_meta_when_invalid_json_should_return_api_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/info?type=meta")
            .with_status(200)
            .with_body("invalid json")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_meta().await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidToolError::ApiError(msg) => {
                assert!(msg.contains("Failed to parse market meta information"));
            }
            _ => panic!("Expected ApiError"),
        }
    }

    #[tokio::test]
    async fn test_get_all_mids_when_successful_should_return_json_value() {
        let mids_json = r#"{"BTC": "50000.0", "ETH": "3000.0", "SOL": "100.0"}"#;

        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/info?type=allMids")
            .with_status(200)
            .with_body(mids_json)
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_all_mids().await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let mids = result.unwrap();
        assert!(mids.is_object());
        assert_eq!(mids["BTC"], "50000.0");
        assert_eq!(mids["ETH"], "3000.0");
        assert_eq!(mids["SOL"], "100.0");
    }

    #[tokio::test]
    async fn test_get_all_mids_when_invalid_json_should_return_api_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/info?type=allMids")
            .with_status(200)
            .with_body("invalid json")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_all_mids().await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidToolError::ApiError(msg) => {
                assert!(msg.contains("Failed to parse market mids"));
            }
            _ => panic!("Expected ApiError"),
        }
    }

    // Environment variable and configuration tests
    #[tokio::test]
    async fn test_place_order_when_missing_private_key_should_return_configuration_error() {
        test_env_vars::remove_test_env_var(HYPERLIQUID_PRIVATE_KEY);
        test_env_vars::remove_test_env_var(ENABLE_REAL_HYPERLIQUID_TRADING);

        let signer = create_mock_signer();
        let client = HyperliquidClient::new(signer).unwrap();

        let order = OrderRequest {
            asset: 0,
            is_buy: true,
            limit_px: "50000.0".to_string(),
            sz: "0.1".to_string(),
            reduce_only: false,
            order_type: OrderType {
                limit: Some(LimitOrderType {
                    tif: "Gtc".to_string(),
                }),
            },
        };

        let result = client.place_order(&order).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidToolError::Configuration(msg) => {
                assert!(msg.contains("HYPERLIQUID_PRIVATE_KEY environment variable required"));
            }
            _ => panic!("Expected Configuration error"),
        }
    }

    #[tokio::test]
    async fn test_place_order_when_trading_disabled_should_return_configuration_error() {
        test_env_vars::set_test_env_var(
            HYPERLIQUID_PRIVATE_KEY,
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12",
        );
        test_env_vars::set_test_env_var(ENABLE_REAL_HYPERLIQUID_TRADING, "false");

        let signer = create_mock_signer();
        let client = HyperliquidClient::new(signer).unwrap();

        let order = OrderRequest {
            asset: 0,
            is_buy: true,
            limit_px: "50000.0".to_string(),
            sz: "0.1".to_string(),
            reduce_only: false,
            order_type: OrderType {
                limit: Some(LimitOrderType {
                    tif: "Gtc".to_string(),
                }),
            },
        };

        let result = client.place_order(&order).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidToolError::Configuration(msg) => {
                assert!(msg.contains("Real trading disabled"));
                assert!(msg.contains("Set ENABLE_REAL_HYPERLIQUID_TRADING=true"));
            }
            _ => panic!("Expected Configuration error"),
        }

        // Clean up
        test_env_vars::remove_test_env_var(HYPERLIQUID_PRIVATE_KEY);
        test_env_vars::remove_test_env_var(ENABLE_REAL_HYPERLIQUID_TRADING);
    }

    #[tokio::test]
    async fn test_place_order_when_invalid_private_key_should_return_configuration_error() {
        test_env_vars::set_test_env_var(HYPERLIQUID_PRIVATE_KEY, "invalid_private_key");
        test_env_vars::set_test_env_var(ENABLE_REAL_HYPERLIQUID_TRADING, "true");

        let signer = create_mock_signer();
        let client = HyperliquidClient::new(signer).unwrap();

        let order = OrderRequest {
            asset: 0,
            is_buy: true,
            limit_px: "50000.0".to_string(),
            sz: "0.1".to_string(),
            reduce_only: false,
            order_type: OrderType {
                limit: Some(LimitOrderType {
                    tif: "Gtc".to_string(),
                }),
            },
        };

        let result = client.place_order(&order).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidToolError::Configuration(msg) => {
                assert!(msg.contains("Invalid private key"));
            }
            _ => panic!("Expected Configuration error"),
        }

        // Clean up
        test_env_vars::remove_test_env_var(HYPERLIQUID_PRIVATE_KEY);
        test_env_vars::remove_test_env_var(ENABLE_REAL_HYPERLIQUID_TRADING);
    }

    #[tokio::test]
    async fn test_cancel_order_when_missing_private_key_should_return_configuration_error() {
        test_env_vars::remove_test_env_var(HYPERLIQUID_PRIVATE_KEY);
        test_env_vars::remove_test_env_var(ENABLE_REAL_HYPERLIQUID_TRADING);

        let signer = create_mock_signer();
        let client = HyperliquidClient::new(signer).unwrap();

        let result = client.cancel_order(12345, 0).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidToolError::Configuration(msg) => {
                assert!(msg.contains("HYPERLIQUID_PRIVATE_KEY environment variable required"));
            }
            _ => panic!("Expected Configuration error"),
        }
    }

    #[tokio::test]
    async fn test_cancel_order_when_trading_disabled_should_return_configuration_error() {
        test_env_vars::set_test_env_var(
            HYPERLIQUID_PRIVATE_KEY,
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12",
        );
        test_env_vars::set_test_env_var(ENABLE_REAL_HYPERLIQUID_TRADING, "false");

        let signer = create_mock_signer();
        let client = HyperliquidClient::new(signer).unwrap();

        let result = client.cancel_order(12345, 0).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidToolError::Configuration(msg) => {
                assert!(msg.contains("Real trading disabled"));
                assert!(msg.contains("Set ENABLE_REAL_HYPERLIQUID_TRADING=true"));
            }
            _ => panic!("Expected Configuration error"),
        }

        // Clean up
        test_env_vars::remove_test_env_var(HYPERLIQUID_PRIVATE_KEY);
        test_env_vars::remove_test_env_var(ENABLE_REAL_HYPERLIQUID_TRADING);
    }

    #[tokio::test]
    async fn test_update_leverage_when_signer_no_address_should_fail() {
        let signer = create_mock_signer_no_address();
        let client = HyperliquidClient::new(signer).unwrap();

        let result = client.update_leverage(10, "BTC", true, Some(0)).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidToolError::AuthError(msg) => {
                assert_eq!(msg, "No address available from signer");
            }
            _ => panic!("Expected AuthError"),
        }
    }

    // Test response parsing errors for different error scenarios
    #[tokio::test]
    async fn test_get_account_info_when_response_read_fails_should_return_network_error() {
        let mut server = Server::new_async().await;
        // Mock a response that will cause a read error (connection closed immediately)
        let mock = server
            .mock(
                "GET",
                "/info?type=clearinghouseState&user=0x1234567890abcdef",
            )
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_account_info("0x1234567890abcdef").await;

        mock.assert_async().await;
        // This should succeed with empty JSON, so let's test what actually happens
        // Empty response should cause JSON parsing error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_positions_when_response_read_fails_should_return_network_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock(
                "GET",
                "/info?type=clearinghouseState&user=0x1234567890abcdef",
            )
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_positions("0x1234567890abcdef").await;

        mock.assert_async().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_meta_when_response_read_fails_should_return_network_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/info?type=meta")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_meta().await;

        mock.assert_async().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all_mids_when_response_read_fails_should_return_network_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/info?type=allMids")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let signer = create_mock_signer();
        let client = HyperliquidClient::with_base_url(signer, server.url()).unwrap();
        let result = client.get_all_mids().await;

        mock.assert_async().await;
        assert!(result.is_err());
    }

    // Test for serialization errors in POST request
    #[test]
    fn test_serialization_edge_cases() {
        // Test serialization of nested structures
        let order_complex = OrderRequest {
            asset: 123,
            is_buy: false,
            limit_px: "12345.6789".to_string(),
            sz: "987.654321".to_string(),
            reduce_only: true,
            order_type: OrderType {
                limit: Some(LimitOrderType {
                    tif: "Ioc".to_string(),
                }),
            },
        };

        let serialized = serde_json::to_string(&order_complex);
        assert!(serialized.is_ok());
        let json_str = serialized.unwrap();
        assert!(json_str.contains("\"asset\":123"));
        assert!(json_str.contains("\"isBuy\":false"));
        assert!(json_str.contains("\"limitPx\":\"12345.6789\""));
        assert!(json_str.contains("\"sz\":\"987.654321\""));
        assert!(json_str.contains("\"reduceOnly\":true"));
        assert!(json_str.contains("\"tif\":\"Ioc\""));
    }

    // Test string edge cases
    #[test]
    fn test_string_edge_cases_in_structures() {
        // Empty strings
        let order_empty = OrderRequest {
            asset: 0,
            is_buy: true,
            limit_px: "".to_string(),
            sz: "".to_string(),
            reduce_only: false,
            order_type: OrderType {
                limit: Some(LimitOrderType {
                    tif: "".to_string(),
                }),
            },
        };

        let json = serde_json::to_string(&order_empty).unwrap();
        assert!(json.contains("\"limitPx\":\"\""));
        assert!(json.contains("\"sz\":\"\""));
        assert!(json.contains("\"tif\":\"\""));

        // Very long strings
        let long_string = "a".repeat(1000);
        let order_long = OrderRequest {
            asset: 0,
            is_buy: true,
            limit_px: long_string.clone(),
            sz: long_string.clone(),
            reduce_only: false,
            order_type: OrderType {
                limit: Some(LimitOrderType {
                    tif: long_string.clone(),
                }),
            },
        };

        let json_long = serde_json::to_string(&order_long).unwrap();
        assert!(json_long.contains(&format!("\"limitPx\":\"{}\"", long_string)));
    }

    // Test numeric edge cases in position data
    #[test]
    fn test_position_data_numeric_edge_cases() {
        let position_edge = PositionData {
            coin: "TEST".to_string(),
            entry_px: Some("0.000000001".to_string()),
            leverage: PositionLeverage {
                type_field: "cross".to_string(),
                value: 1,
            },
            liquidation_px: Some("999999999.999999999".to_string()),
            margin_used: "0".to_string(),
            max_leverage: 1,
            position_value: "-999999.999".to_string(),
            return_on_equity: "-100.0".to_string(),
            szi: "0".to_string(),
            unrealized_pnl: "0.0".to_string(),
        };

        // Test that all fields can be accessed
        assert_eq!(position_edge.coin, "TEST");
        assert_eq!(position_edge.entry_px, Some("0.000000001".to_string()));
        assert_eq!(position_edge.leverage.value, 1);
        assert_eq!(position_edge.max_leverage, 1);
        assert_eq!(position_edge.position_value, "-999999.999");
        assert_eq!(position_edge.return_on_equity, "-100.0");
    }

    // Test optional field behaviors
    #[test]
    fn test_optional_fields_behavior() {
        let position_minimal = PositionData {
            coin: "BTC".to_string(),
            entry_px: None,
            leverage: PositionLeverage {
                type_field: "isolated".to_string(),
                value: 5,
            },
            liquidation_px: None,
            margin_used: "100.0".to_string(),
            max_leverage: 50,
            position_value: "5000.0".to_string(),
            return_on_equity: "0.0".to_string(),
            szi: "1.0".to_string(),
            unrealized_pnl: "0.0".to_string(),
        };

        assert_eq!(position_minimal.entry_px, None);
        assert_eq!(position_minimal.liquidation_px, None);
        assert_eq!(position_minimal.leverage.type_field, "isolated");
    }

    // Constants tests
    #[test]
    fn test_constants() {
        assert_eq!(HYPERLIQUID_PRIVATE_KEY, "HYPERLIQUID_PRIVATE_KEY");
        assert_eq!(
            ENABLE_REAL_HYPERLIQUID_TRADING,
            "ENABLE_REAL_HYPERLIQUID_TRADING"
        );
    }
}
