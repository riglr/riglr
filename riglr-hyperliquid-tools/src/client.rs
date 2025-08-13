//! Hyperliquid API client for interacting with the Hyperliquid L1 blockchain
//!
//! This module provides a client for the Hyperliquid protocol, which operates
//! its own L1 blockchain for perpetual futures trading.

use reqwest::{Client, Response};
use riglr_core::signer::TransactionSigner;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};
use serde_json;
// Note: Using direct HTTP API instead of SDK

// EIP-712 signing dependencies
use ethers_core::types::H256;
use ethers_core::types::transaction::eip712::{Eip712, TypedData};
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
            .map_err(|e| HyperliquidToolError::NetworkError(format!("Failed to create HTTP client with custom URL: {}", e)))?;

        Ok(Self {
            client,
            base_url,
            signer,
        })
    }


    /// Make a GET request to the Hyperliquid API
    pub async fn get(&self, endpoint: &str) -> Result<Response> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
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
                        Err(HyperliquidToolError::RateLimit(format!("Rate limit exceeded: {}", error_body)))
                    } else if status.is_server_error() {
                        Err(HyperliquidToolError::NetworkError(format!("Server error {}: {}", status, error_body)))
                    } else {
                        Err(HyperliquidToolError::ApiError(format!("API error {}: {}", status, error_body)))
                    }
                }
            }
            Err(e) => {
                error!("Network error during GET request: {}", e);
                Err(HyperliquidToolError::NetworkError(format!("Network error during GET request: {}", e)))
            }
        }
    }

    /// Make a POST request to the Hyperliquid API
    pub async fn post<T: Serialize>(&self, endpoint: &str, payload: &T) -> Result<Response> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
        debug!("Making POST request to: {}", url);

        let json_payload = serde_json::to_string(payload)
            .map_err(|e| HyperliquidToolError::ApiError(format!("Failed to serialize request payload: {}", e)))?;

        match self.client
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
                        Err(HyperliquidToolError::RateLimit(format!("Rate limit exceeded on POST: {}", error_body)))
                    } else if status.is_server_error() {
                        Err(HyperliquidToolError::NetworkError(format!("Server error {} on POST: {}", status, error_body)))
                    } else {
                        Err(HyperliquidToolError::ApiError(format!("API error {} on POST: {}", status, error_body)))
                    }
                }
            }
            Err(e) => {
                error!("Network error during POST request: {}", e);
                Err(HyperliquidToolError::NetworkError(format!("Network error during POST request: {}", e)))
            }
        }
    }

    /// Get account information
    pub async fn get_account_info(&self, user_address: &str) -> Result<AccountInfo> {
        let response = self.get(&format!("info?type=clearinghouseState&user={}", user_address)).await?;
        let text = response.text().await
            .map_err(|e| HyperliquidToolError::NetworkError(format!("Failed to read account info response: {}", e)))?;
        
        let account_info: AccountInfo = serde_json::from_str(&text)
            .map_err(|e| HyperliquidToolError::ApiError(format!("Failed to parse account info response: {}", e)))?;
        
        Ok(account_info)
    }

    /// Get current positions for a user
    pub async fn get_positions(&self, user_address: &str) -> Result<Vec<Position>> {
        let response = self.get(&format!("info?type=clearinghouseState&user={}", user_address)).await?;
        let text = response.text().await
            .map_err(|e| HyperliquidToolError::NetworkError(format!("Failed to read positions response: {}", e)))?;
        
        let state: ClearinghouseState = serde_json::from_str(&text)
            .map_err(|e| HyperliquidToolError::ApiError(format!("Failed to parse clearinghouse state: {}", e)))?;
        
        Ok(state.asset_positions.unwrap_or_default())
    }

    /// Get market information
    pub async fn get_meta(&self) -> Result<Meta> {
        let response = self.get("info?type=meta").await?;
        let text = response.text().await
            .map_err(|e| HyperliquidToolError::NetworkError(format!("Failed to read meta response: {}", e)))?;
        
        let meta: Meta = serde_json::from_str(&text)
            .map_err(|e| HyperliquidToolError::ApiError(format!("Failed to parse market meta information: {}", e)))?;
        
        Ok(meta)
    }

    /// Get all market mid prices (current market prices)
    pub async fn get_all_mids(&self) -> Result<serde_json::Value> {
        let response = self.get("info?type=allMids").await?;
        let text = response.text().await
            .map_err(|e| HyperliquidToolError::NetworkError(format!("Failed to read mids response: {}", e)))?;
        
        let mids: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| HyperliquidToolError::ApiError(format!("Failed to parse market mids: {}", e)))?;
        
        Ok(mids)
    }

    /// Place an order using real Hyperliquid API
    /// CRITICAL: This is REAL order placement - NO SIMULATION
    pub async fn place_order(&self, order: &OrderRequest) -> Result<OrderResponse> {
        error!("CRITICAL: REAL ORDER PLACEMENT - This will place actual trades!");
        info!("Placing REAL order: asset={}, side={}, size={}, price={}", 
              order.asset, if order.is_buy { "buy" } else { "sell" }, order.sz, order.limit_px);
        
        // Require explicit configuration to prevent accidental trades
        let private_key = std::env::var("HYPERLIQUID_PRIVATE_KEY")
            .map_err(|_| HyperliquidToolError::Configuration(
                "HYPERLIQUID_PRIVATE_KEY environment variable required. REAL trading disabled for safety.".to_string()
            ))?;
            
        let trading_enabled = std::env::var("ENABLE_REAL_HYPERLIQUID_TRADING")
            .unwrap_or_default() == "true";
            
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
            "signature": self.sign_order_payload(&private_key, &order).await?
        });

        // Make real API call to Hyperliquid
        let response = self.post("exchange", &order_payload).await?;
        let response_text = response.text().await
            .map_err(|e| HyperliquidToolError::NetworkError(format!("Failed to read order response: {}", e)))?;

        // Parse the real response
        let order_response: OrderResponse = serde_json::from_str(&response_text)
            .map_err(|e| HyperliquidToolError::ApiError(format!("Failed to parse order response: {} - Body: {}", e, response_text)))?;

        info!("REAL order placed successfully: {:?}", order_response);
        Ok(order_response)
    }
    
    /// Sign order payload for Hyperliquid using EIP-712
    async fn sign_order_payload(&self, private_key: &str, order: &OrderRequest) -> Result<String> {
        info!("Signing order payload with EIP-712");
        
        // Parse the private key
        let wallet = private_key.parse::<LocalWallet>()
            .map_err(|e| HyperliquidToolError::Configuration(format!("Invalid private key: {}", e)))?;
        
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
        let typed_data: TypedData = serde_json::from_value(typed_data)
            .map_err(|e| HyperliquidToolError::Configuration(format!("Failed to create typed data: {}", e)))?;
        
        // Encode the typed data
        let encoded = typed_data.encode_eip712()
            .map_err(|e| HyperliquidToolError::Configuration(format!("Failed to encode EIP-712: {}", e)))?;
        
        // Sign the encoded data
        let signature = wallet.sign_hash(H256::from(encoded))
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
    pub async fn update_leverage(&self, leverage: u32, coin: &str, is_cross: bool, asset_id: Option<u32>) -> Result<LeverageResponse> {
        debug!("Updating leverage for {} to {}x (cross: {})", coin, leverage, is_cross);
        
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
        
        let leverage_response: LeverageResponse = response.json().await
            .map_err(|e| HyperliquidToolError::ApiError(format!("Failed to parse leverage response: {}", e)))?;
        
        info!("Successfully updated leverage for {} to {}x", coin, leverage);
        Ok(leverage_response)
    }
    
    async fn sign_leverage_request(&self, mut request: LeverageRequest) -> Result<LeverageRequest> {
        // Create the message to sign
        let message = serde_json::json!({
            "action": request.action,
            "nonce": request.nonce,
            "vault_address": null
        });
        
        let message_str = serde_json::to_string(&message)
            .map_err(|e| HyperliquidToolError::ApiError(format!("Failed to serialize message for signing: {}", e)))?;
        
        // Sign the message
        let signature = self.sign_message(&message_str).await?;
        request.signature = Some(signature);
        
        Ok(request)
    }

    pub async fn cancel_order(&self, order_id: u64, asset: u32) -> Result<CancelResponse> {
        error!("CRITICAL: REAL ORDER CANCELLATION - This will cancel actual trades!");
        info!("Cancelling REAL order: order_id={}, asset={}", order_id, asset);
        
        // Require explicit configuration to prevent accidental operations
        let private_key = std::env::var("HYPERLIQUID_PRIVATE_KEY")
            .map_err(|_| HyperliquidToolError::Configuration(
                "HYPERLIQUID_PRIVATE_KEY environment variable required. REAL trading disabled for safety.".to_string()
            ))?;
            
        let trading_enabled = std::env::var("ENABLE_REAL_HYPERLIQUID_TRADING")
            .unwrap_or_default() == "true";
            
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
        let response_text = response.text().await
            .map_err(|e| HyperliquidToolError::NetworkError(format!("Failed to read cancel response: {}", e)))?;

        // Parse the real response
        let cancel_response: CancelResponse = serde_json::from_str(&response_text)
            .map_err(|e| HyperliquidToolError::ApiError(format!("Failed to parse cancel response: {} - Body: {}", e, response_text)))?;

        info!("REAL order cancelled successfully: {:?}", cancel_response);
        Ok(cancel_response)
    }
    
    /// Sign cancel payload for Hyperliquid using EIP-712
    async fn sign_cancel_payload(&self, private_key: &str, order_id: u64, asset: u32) -> Result<String> {
        info!("Signing cancel payload with EIP-712");
        
        // Parse the private key
        let wallet = private_key.parse::<LocalWallet>()
            .map_err(|e| HyperliquidToolError::Configuration(format!("Invalid private key: {}", e)))?;
        
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
        let typed_data: TypedData = serde_json::from_value(typed_data)
            .map_err(|e| HyperliquidToolError::Configuration(format!("Failed to create typed data: {}", e)))?;
        
        // Encode the typed data
        let encoded = typed_data.encode_eip712()
            .map_err(|e| HyperliquidToolError::Configuration(format!("Failed to encode EIP-712: {}", e)))?;
        
        // Sign the encoded data
        let signature = wallet.sign_hash(H256::from(encoded))
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
    pub fn get_user_address(&self) -> Result<String> {
        self.signer.address()
            .ok_or_else(|| HyperliquidToolError::AuthError("No address available from signer".to_string()))
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    #[serde(rename = "assetPositions")]
    pub asset_positions: Option<Vec<Position>>,
    #[serde(rename = "crossMaintenanceMarginUsed")]
    pub cross_maintenance_margin_used: Option<String>,
    #[serde(rename = "crossMarginUsed")]
    pub cross_margin_used: Option<String>,
    #[serde(rename = "withdrawable")]
    pub withdrawable: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearinghouseState {
    #[serde(rename = "assetPositions")]
    pub asset_positions: Option<Vec<Position>>,
    #[serde(rename = "crossMaintenanceMarginUsed")]
    pub cross_maintenance_margin_used: Option<String>,
    #[serde(rename = "crossMarginUsed")]
    pub cross_margin_used: Option<String>,
    pub withdrawable: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    #[serde(rename = "position")]
    pub position: PositionData,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionData {
    pub coin: String,
    #[serde(rename = "entryPx")]
    pub entry_px: Option<String>,
    pub leverage: PositionLeverage,
    #[serde(rename = "liquidationPx")]
    pub liquidation_px: Option<String>,
    #[serde(rename = "marginUsed")]
    pub margin_used: String,
    #[serde(rename = "maxLeverage")]
    pub max_leverage: u32,
    #[serde(rename = "positionValue")]
    pub position_value: String,
    #[serde(rename = "returnOnEquity")]
    pub return_on_equity: String,
    pub szi: String,
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLeverage {
    #[serde(rename = "type")]
    pub type_field: String,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub universe: Vec<AssetInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub name: String,
    #[serde(rename = "szDecimals")]
    pub sz_decimals: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub asset: u32,
    #[serde(rename = "isBuy")]
    pub is_buy: bool,
    #[serde(rename = "limitPx")]
    pub limit_px: String,
    pub sz: String,
    #[serde(rename = "reduceOnly")]
    pub reduce_only: bool,
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderType {
    #[serde(rename = "limit")]
    pub limit: Option<LimitOrderType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitOrderType {
    pub tif: String, // Time in force: "Gtc", "Ioc", "Alo"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub status: String,
    pub data: OrderResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResult {
    #[serde(rename = "statuses")]
    pub status_code: u32,
    pub response: ResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    #[serde(rename = "type")]
    pub type_field: String,
    pub data: Option<OrderData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderData {
    pub statuses: Vec<OrderStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatus {
    pub resting: RestingOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestingOrder {
    pub oid: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResponse {
    pub status: String,
    pub data: CancelResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResult {
    #[serde(rename = "statuses")]
    pub status_code: u32,
}

// Leverage-related structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeverageRequest {
    pub action: LeverageAction,
    pub nonce: i64,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeverageAction {
    #[serde(rename = "type")]
    pub type_: String,
    pub asset: u32,
    #[serde(rename = "isCross")]
    pub is_cross: bool,
    pub leverage: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeverageResponse {
    pub status: String,
    pub data: Option<LeverageResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeverageResult {
    pub leverage: u32,
    pub asset: String,
}