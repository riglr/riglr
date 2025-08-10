//! Trading tools for Hyperliquid perpetual futures
//!
//! This module provides tools for placing, canceling, and managing orders on Hyperliquid.

use crate::client::{HyperliquidClient, OrderRequest, OrderType, LimitOrderType, Meta};
use riglr_core::{ToolError, SignerContext};
use riglr_macros::tool;
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{debug, info, warn};

/// Place a perpetual futures order on Hyperliquid
///
/// This tool places a market or limit order for perpetual futures on Hyperliquid.
/// Supports both buy and sell orders with optional price limits and risk management.
#[tool]
pub async fn place_hyperliquid_order(
    symbol: String,
    side: String,
    size: String,
    order_type: String,
    price: Option<String>,
    reduce_only: Option<bool>,
    time_in_force: Option<String>,
) -> Result<HyperliquidOrderResult, ToolError> {
    debug!("Placing Hyperliquid order: {} {} {} {}", side, size, symbol, order_type);

    // Get signer context
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    // Create client
    let client = HyperliquidClient::new(signer);

    // Validate side
    let is_buy = match side.to_lowercase().as_str() {
        "buy" | "long" => true,
        "sell" | "short" => false,
        _ => return Err(ToolError::permanent(format!("Invalid side '{}'. Must be 'buy', 'sell', 'long', or 'short'", side))),
    };

    // Parse size
    let size_decimal = Decimal::from_str(&size)
        .map_err(|e| ToolError::permanent(format!("Invalid size '{}': {}", size, e)))?;
    if size_decimal <= Decimal::ZERO {
        return Err(ToolError::permanent("Size must be greater than 0".to_string()));
    }

    // Get market metadata to find asset ID
    let meta = client.get_meta().await?;
    let asset_id = find_asset_id(&meta, &symbol)?;

    // Validate and parse order type
    let (limit_px, order_type_obj) = match order_type.to_lowercase().as_str() {
        "market" => {
            // For market orders, we use a very high/low price to ensure execution
            let market_price = if is_buy { "999999999" } else { "0.000001" };
            (market_price.to_string(), OrderType {
                limit: Some(LimitOrderType {
                    tif: "Ioc".to_string(), // Immediate or Cancel for market orders
                }),
            })
        },
        "limit" => {
            let price_str = price.as_ref().ok_or_else(|| 
                ToolError::permanent("Price is required for limit orders".to_string()))?.clone();
            
            let price_decimal = Decimal::from_str(&price_str)
                .map_err(|e| ToolError::permanent(format!("Invalid price '{}': {}", price_str, e)))?;
            if price_decimal <= Decimal::ZERO {
                return Err(ToolError::permanent("Price must be greater than 0".to_string()));
            }

            let tif = match time_in_force.as_deref().unwrap_or("gtc").to_lowercase().as_str() {
                "gtc" | "good_till_cancel" => "Gtc",
                "ioc" | "immediate_or_cancel" => "Ioc", 
                "alo" | "add_liquidity_only" => "Alo",
                other => return Err(ToolError::permanent(format!("Invalid time in force '{}'. Must be 'gtc', 'ioc', or 'alo'", other))),
            };

            (price_str, OrderType {
                limit: Some(LimitOrderType {
                    tif: tif.to_string(),
                }),
            })
        },
        _ => return Err(ToolError::permanent(format!("Invalid order type '{}'. Must be 'market' or 'limit'", order_type))),
    };

    // Create order request
    let order = OrderRequest {
        asset: asset_id,
        is_buy,
        limit_px: limit_px,
        sz: size.clone(),
        reduce_only: reduce_only.unwrap_or(false),
        order_type: order_type_obj,
    };

    // Place the order
    let response = client.place_order(&order).await?;

    info!("Successfully placed Hyperliquid order: {:?}", response);

    Ok(HyperliquidOrderResult {
        symbol,
        side,
        size,
        order_type,
        price,
        status: response.status,
        order_id: response.data.response.data
            .and_then(|d| d.statuses.first().map(|s| s.resting.oid))
            .map(|id| id.to_string()),
        message: "Order placed successfully".to_string(),
    })
}

/// Cancel an existing order on Hyperliquid
///
/// This tool cancels a previously placed order using its order ID.
#[tool]
pub async fn cancel_hyperliquid_order(
    symbol: String,
    order_id: String,
) -> Result<HyperliquidCancelResult, ToolError> {
    debug!("Canceling Hyperliquid order: {} for {}", order_id, symbol);

    // Get signer context
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    // Create client
    let client = HyperliquidClient::new(signer);

    // Parse order ID
    let oid = order_id.parse::<u64>()
        .map_err(|e| ToolError::permanent(format!("Invalid order ID '{}': {}", order_id, e)))?;

    // Get market metadata to find asset ID
    let meta = client.get_meta().await?;
    let asset_id = find_asset_id(&meta, &symbol)?;

    // Cancel the order
    let response = client.cancel_order(oid, asset_id).await?;

    info!("Successfully canceled Hyperliquid order: {}", order_id);

    Ok(HyperliquidCancelResult {
        symbol,
        order_id,
        status: response.status,
        message: "Order canceled successfully".to_string(),
    })
}

/// Get account information from Hyperliquid
///
/// This tool retrieves account balance, margin usage, and other account details.
#[tool]
pub async fn get_hyperliquid_account_info() -> Result<HyperliquidAccountResult, ToolError> {
    debug!("Getting Hyperliquid account info");

    // Get signer context
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    // Create client
    let client = HyperliquidClient::new(signer);

    // Get user address
    let user_address = client.get_user_address()?;

    // Get account info
    let account_info = client.get_account_info(&user_address).await?;

    Ok(HyperliquidAccountResult {
        user_address,
        withdrawable_balance: account_info.withdrawable.unwrap_or_default(),
        cross_margin_used: account_info.cross_margin_used.unwrap_or_default(),
        cross_maintenance_margin_used: account_info.cross_maintenance_margin_used.unwrap_or_default(),
        positions_count: account_info.asset_positions.as_ref().map(|p| p.len()).unwrap_or(0),
    })
}

/// Set leverage for a trading pair on Hyperliquid
///
/// This tool sets the leverage multiplier for a specific asset.
/// Note: This is a placeholder implementation as leverage setting requires specific API endpoints.
#[tool]
pub async fn set_leverage(
    symbol: String,
    leverage: u32,
) -> Result<HyperliquidLeverageResult, ToolError> {
    debug!("Setting leverage for {}: {}x", symbol, leverage);

    // Validate leverage range (typical range for perpetual futures)
    if leverage < 1 || leverage > 100 {
        return Err(ToolError::permanent(format!("Invalid leverage {}. Must be between 1 and 100", leverage)));
    }

    // Get signer context
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    // Create client  
    let client = HyperliquidClient::new(signer);

    // Get market metadata to validate symbol
    let meta = client.get_meta().await?;
    let _asset_id = find_asset_id(&meta, &symbol)?; // Validate symbol exists

    // Note: In a real implementation, this would make an API call to set leverage
    warn!("Leverage setting simulation - would set {}x leverage for {}", leverage, symbol);

    Ok(HyperliquidLeverageResult {
        symbol: symbol.clone(),
        leverage,
        status: "simulated".to_string(),
        message: format!("Leverage set to {}x for {}", leverage, symbol),
    })
}

/// Helper function to find asset ID by symbol
fn find_asset_id(meta: &Meta, symbol: &str) -> Result<u32, ToolError> {
    // Normalize symbol (remove -PERP suffix if present)
    let normalized_symbol = symbol.trim_end_matches("-PERP").trim_end_matches("-perp");
    
    for (index, asset) in meta.universe.iter().enumerate() {
        if asset.name.eq_ignore_ascii_case(symbol) || 
           asset.name.eq_ignore_ascii_case(normalized_symbol) ||
           asset.name.eq_ignore_ascii_case(&format!("{}-PERP", normalized_symbol)) {
            return Ok(index as u32);
        }
    }
    
    Err(ToolError::permanent(format!("Asset '{}' not found. Available assets: {}", 
        symbol, 
        meta.universe.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")
    )))
}

// Result structures

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperliquidOrderResult {
    pub symbol: String,
    pub side: String,
    pub size: String,
    pub order_type: String,
    pub price: Option<String>,
    pub status: String,
    pub order_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperliquidCancelResult {
    pub symbol: String,
    pub order_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperliquidAccountResult {
    pub user_address: String,
    pub withdrawable_balance: String,
    pub cross_margin_used: String,
    pub cross_maintenance_margin_used: String,
    pub positions_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperliquidLeverageResult {
    pub symbol: String,
    pub leverage: u32,
    pub status: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_structures() {
        let order_result = HyperliquidOrderResult {
            symbol: "ETH-PERP".to_string(),
            side: "buy".to_string(),
            size: "0.1".to_string(),
            order_type: "limit".to_string(),
            price: Some("2000.0".to_string()),
            status: "success".to_string(),
            order_id: Some("12345".to_string()),
            message: "Order placed successfully".to_string(),
        };

        assert_eq!(order_result.symbol, "ETH-PERP");
        assert_eq!(order_result.side, "buy");
    }

    #[test]
    fn test_find_asset_id() {
        let meta = Meta {
            universe: vec![
                crate::client::AssetInfo {
                    name: "BTC-PERP".to_string(),
                    sz_decimals: 4,
                },
                crate::client::AssetInfo {
                    name: "ETH-PERP".to_string(),
                    sz_decimals: 3,
                },
            ],
        };

        assert_eq!(find_asset_id(&meta, "BTC-PERP").unwrap(), 0);
        assert_eq!(find_asset_id(&meta, "ETH-PERP").unwrap(), 1);
        assert_eq!(find_asset_id(&meta, "BTC").unwrap(), 0); // Should find BTC-PERP
        assert_eq!(find_asset_id(&meta, "eth").unwrap(), 1); // Case insensitive

        assert!(find_asset_id(&meta, "INVALID").is_err());
    }
}