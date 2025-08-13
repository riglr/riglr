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
use tracing::{debug, info};

/// Place a perpetual futures order on Hyperliquid
///
/// This tool places market or limit orders for perpetual futures on the Hyperliquid DEX.
/// It supports both long and short positions with configurable leverage, time-in-force options,
/// and risk management features like reduce-only orders.
/// 
/// # Arguments
/// 
/// * `symbol` - Trading pair symbol (e.g., "ETH-PERP", "BTC", "SOL-PERP")
/// * `side` - Order side: "buy"/"long" or "sell"/"short"
/// * `size` - Position size as decimal string (e.g., "0.1" for 0.1 contracts)
/// * `order_type` - "market" for immediate execution or "limit" for price-specific order
/// * `price` - Limit price (required for limit orders, ignored for market orders)
/// * `reduce_only` - If true, order can only reduce existing position size
/// * `time_in_force` - "gtc" (good-till-cancel), "ioc" (immediate-or-cancel), or "alo" (add-liquidity-only)
/// 
/// # Returns
/// 
/// Returns `HyperliquidOrderResult` containing:
/// - Order details (symbol, side, size, type, price)
/// - `status`: API response status
/// - `order_id`: Unique order identifier for tracking
/// - `message`: Human-readable result description
/// 
/// # Errors
/// 
/// * `HyperliquidToolError::InvalidInput` - When parameters are invalid (negative size, unknown symbol)
/// * `HyperliquidToolError::ApiError` - When Hyperliquid API rejects the order
/// * `HyperliquidToolError::NetworkError` - When connection issues occur
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_hyperliquid_tools::trading::place_hyperliquid_order;
/// use riglr_core::SignerContext;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Place a limit buy order for 0.1 ETH-PERP at $2000
/// let result = place_hyperliquid_order(
///     "ETH-PERP".to_string(),
///     "buy".to_string(),
///     "0.1".to_string(),
///     "limit".to_string(),
///     Some("2000.0".to_string()),
///     Some(false), // Not reduce-only
///     Some("gtc".to_string()), // Good till cancel
/// ).await?;
/// 
/// println!("Order placed! ID: {:?}", result.order_id);
/// println!("Status: {}", result.status);
/// # Ok(())
/// # }
/// ```
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
    let client = HyperliquidClient::new(signer)?;

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
        limit_px,
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
/// This tool cancels a previously placed order that is still active (not filled or expired).
/// Both the symbol and order ID are required to ensure the correct order is canceled.
/// 
/// # Arguments
/// 
/// * `symbol` - Trading pair symbol that the order belongs to
/// * `order_id` - Unique order identifier returned from place_hyperliquid_order
/// 
/// # Returns
/// 
/// Returns `HyperliquidCancelResult` containing cancellation confirmation details.
/// 
/// # Errors
/// 
/// * `HyperliquidToolError::InvalidInput` - When order_id format is invalid or symbol unknown
/// * `HyperliquidToolError::ApiError` - When order doesn't exist or already filled
/// * `HyperliquidToolError::NetworkError` - When connection issues occur
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_hyperliquid_tools::trading::cancel_hyperliquid_order;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let result = cancel_hyperliquid_order(
///     "ETH-PERP".to_string(),
///     "12345678".to_string(), // Order ID from previous order
/// ).await?;
/// 
/// println!("Order canceled: {}", result.message);
/// # Ok(())
/// # }
/// ```
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
    let client = HyperliquidClient::new(signer)?;

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
/// This tool retrieves comprehensive account information including balance, margin usage,
/// and position counts. Essential for monitoring account health and available trading capital.
/// 
/// # Returns
/// 
/// Returns `HyperliquidAccountResult` containing:
/// - `user_address`: The account's Ethereum address
/// - `withdrawable_balance`: Available balance that can be withdrawn
/// - `cross_margin_used`: Amount of balance used for cross-margin positions
/// - `cross_maintenance_margin_used`: Maintenance margin requirements
/// - `positions_count`: Number of active positions
/// 
/// # Errors
/// 
/// * `HyperliquidToolError::NetworkError` - When API connection fails
/// * `HyperliquidToolError::Generic` - When signer context is unavailable
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_hyperliquid_tools::trading::get_hyperliquid_account_info;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let account = get_hyperliquid_account_info().await?;
/// 
/// println!("Account: {}", account.user_address);
/// println!("Withdrawable: ${}", account.withdrawable_balance);
/// println!("Margin used: ${}", account.cross_margin_used);
/// println!("Active positions: {}", account.positions_count);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_hyperliquid_account_info() -> Result<HyperliquidAccountResult, ToolError> {
    debug!("Getting Hyperliquid account info");

    // Get signer context
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    // Create client
    let client = HyperliquidClient::new(signer)?;

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
/// This tool sets the leverage multiplier for a specific asset on Hyperliquid.
/// The leverage determines the maximum position size relative to your margin.
/// 
/// # Arguments
/// 
/// * `symbol` - Trading pair symbol (e.g., "ETH", "BTC", "SOL")
/// * `leverage` - Leverage multiplier (1-100x)
/// 
/// # Returns
/// 
/// Returns `HyperliquidLeverageResult` containing the updated leverage settings.
/// 
/// # Errors
/// 
/// * `ToolError::permanent` - Invalid leverage value or symbol
/// * `ToolError::retriable` - Network or API errors
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_hyperliquid_tools::trading::set_leverage;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Set 10x leverage for ETH perpetual futures
/// let result = set_leverage(
///     "ETH".to_string(),
///     10,
/// ).await?;
/// 
/// println!("Leverage updated: {}", result.message);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn set_leverage(
    symbol: String,
    leverage: u32,
) -> Result<HyperliquidLeverageResult, ToolError> {
    debug!("Setting leverage for {}: {}x", symbol, leverage);

    // Validate leverage range (typical range for perpetual futures)
    if !(1..=100).contains(&leverage) {
        return Err(ToolError::permanent(format!("Invalid leverage {}. Must be between 1 and 100", leverage)));
    }

    // Get signer context
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    // Create client  
    let client = HyperliquidClient::new(signer)?;

    // Get market metadata to validate symbol and get asset ID
    let meta = client.get_meta().await?;
    let asset_id = find_asset_id(&meta, &symbol)?;

    // Update leverage using the real API
    let response = client.update_leverage(leverage, &symbol, true, Some(asset_id)).await
        .map_err(|e| {
            if e.to_string().contains("rate limit") {
                ToolError::retriable(format!("Rate limited while setting leverage: {}", e))
            } else if e.to_string().contains("network") || e.to_string().contains("timeout") {
                ToolError::retriable(format!("Network error while setting leverage: {}", e))
            } else {
                ToolError::permanent(format!("Failed to set leverage: {}", e))
            }
        })?;

    info!("Successfully set {}x leverage for {}", leverage, symbol);

    Ok(HyperliquidLeverageResult {
        symbol: symbol.clone(),
        leverage,
        status: response.status,
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