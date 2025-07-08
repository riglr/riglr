//! Hyperliquid API client for interacting with the Hyperliquid L1 blockchain
//!
//! This module provides a client for the Hyperliquid protocol, which operates
//! its own L1 blockchain for perpetual futures trading.

use anyhow::Result;
use reqwest::{Client, Response};
use riglr_core::{ToolError, signer::TransactionSigner};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, warn};

/// Hyperliquid API client
#[derive(Clone, Debug)]
pub struct HyperliquidClient {
    client: Client,
    base_url: String,
    signer: Arc<dyn TransactionSigner>,
}

impl HyperliquidClient {
    /// Create a new Hyperliquid client
    pub fn new(signer: Arc<dyn TransactionSigner>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: "https://api.hyperliquid.xyz".to_string(),
            signer,
        }
    }

    /// Create a new Hyperliquid client with custom base URL (for testing)
    pub fn with_base_url(signer: Arc<dyn TransactionSigner>, base_url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            signer,
        }
    }

    /// Make a GET request to the Hyperliquid API
    pub async fn get(&self, endpoint: &str) -> Result<Response, ToolError> {
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
                    
                    if status.is_server_error() || status == 429 {
                        Err(ToolError::retriable(format!("HTTP {} error: {}", status, error_body)))
                    } else {
                        Err(ToolError::permanent(format!("HTTP {} error: {}", status, error_body)))
                    }
                }
            }
            Err(e) => {
                error!("Network error during GET request: {}", e);
                Err(ToolError::retriable(format!("Network error: {}", e)))
            }
        }
    }

    /// Make a POST request to the Hyperliquid API
    pub async fn post<T: Serialize>(&self, endpoint: &str, payload: &T) -> Result<Response, ToolError> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
        debug!("Making POST request to: {}", url);

        let json_payload = serde_json::to_string(payload)
            .map_err(|e| ToolError::permanent(format!("Failed to serialize payload: {}", e)))?;

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
                    
                    if status.is_server_error() || status == 429 {
                        Err(ToolError::retriable(format!("HTTP {} error: {}", status, error_body)))
                    } else {
                        Err(ToolError::permanent(format!("HTTP {} error: {}", status, error_body)))
                    }
                }
            }
            Err(e) => {
                error!("Network error during POST request: {}", e);
                Err(ToolError::retriable(format!("Network error: {}", e)))
            }
        }
    }

    /// Get account information
    pub async fn get_account_info(&self, user_address: &str) -> Result<AccountInfo, ToolError> {
        let response = self.get(&format!("info?type=clearinghouseState&user={}", user_address)).await?;
        let text = response.text().await
            .map_err(|e| ToolError::retriable(format!("Failed to read response: {}", e)))?;
        
        let account_info: AccountInfo = serde_json::from_str(&text)
            .map_err(|e| ToolError::permanent(format!("Failed to parse account info: {}", e)))?;
        
        Ok(account_info)
    }

    /// Get current positions for a user
    pub async fn get_positions(&self, user_address: &str) -> Result<Vec<Position>, ToolError> {
        let response = self.get(&format!("info?type=clearinghouseState&user={}", user_address)).await?;
        let text = response.text().await
            .map_err(|e| ToolError::retriable(format!("Failed to read response: {}", e)))?;
        
        let state: ClearinghouseState = serde_json::from_str(&text)
            .map_err(|e| ToolError::permanent(format!("Failed to parse clearinghouse state: {}", e)))?;
        
        Ok(state.asset_positions.unwrap_or_default())
    }

    /// Get market information
    pub async fn get_meta(&self) -> Result<Meta, ToolError> {
        let response = self.get("info?type=meta").await?;
        let text = response.text().await
            .map_err(|e| ToolError::retriable(format!("Failed to read response: {}", e)))?;
        
        let meta: Meta = serde_json::from_str(&text)
            .map_err(|e| ToolError::permanent(format!("Failed to parse meta info: {}", e)))?;
        
        Ok(meta)
    }

    /// Place an order
    pub async fn place_order(&self, order: &OrderRequest) -> Result<OrderResponse, ToolError> {
        // For now, we'll simulate order placement since actual order signing requires
        // Hyperliquid-specific cryptographic operations
        warn!("Order placement simulation - would place order: {:?}", order);
        
        // In a real implementation, this would:
        // 1. Create the proper Hyperliquid transaction
        // 2. Sign it with the user's private key
        // 3. Submit to the Hyperliquid L1 chain
        
        Ok(OrderResponse {
            status: "simulated".to_string(),
            data: OrderResult {
                status_code: 0,
                response: ResponseData {
                    type_field: "order".to_string(),
                    data: Some(OrderData {
                        statuses: vec![OrderStatus {
                            resting: RestingOrder {
                                oid: 12345,
                            },
                        }],
                    }),
                },
            },
        })
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: u64, asset: u32) -> Result<CancelResponse, ToolError> {
        warn!("Order cancellation simulation - would cancel order {} for asset {}", order_id, asset);
        
        Ok(CancelResponse {
            status: "simulated".to_string(),
            data: CancelResult {
                status_code: 0,
            },
        })
    }

    /// Get the user's address from the signer
    pub fn get_user_address(&self) -> Result<String, ToolError> {
        self.signer.address()
            .ok_or_else(|| ToolError::permanent("No address available from signer".to_string()))
    }
}

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