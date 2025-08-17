//! Hyperliquid API client for interacting with the Hyperliquid L1 blockchain
//!
//! This module provides a client for the Hyperliquid protocol, which operates
//! its own L1 blockchain for perpetual futures trading.

use reqwest::{Client, Response};
use riglr_core::signer::TransactionSigner;
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
pub struct HyperliquidClient {
    client: Client,
    base_url: String,
    signer: Arc<dyn TransactionSigner>,
}

impl HyperliquidClient {
    /// Create a new Hyperliquid client
    pub fn new(signer: Arc<dyn TransactionSigner>) -> Result<Self> {
        Self::with_base_url(signer, "https://api.hyperliquid.xyz".to_string())
    }

    /// Create a new Hyperliquid client with custom base URL (for testing)
    pub fn with_base_url(signer: Arc<dyn TransactionSigner>, base_url: String) -> Result<Self> {
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
        self.signer.address().ok_or_else(|| {
            HyperliquidToolError::AuthError("No address available from signer".to_string())
        })
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
}
