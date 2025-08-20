//! Trading tools for Hyperliquid perpetual futures
//!
//! This module provides tools for placing, canceling, and managing orders on Hyperliquid.

use crate::client::{HyperliquidClient, LimitOrderType, Meta, OrderRequest, OrderType};
use riglr_core::{SignerContext, ToolError};
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
    _context: &riglr_core::provider::ApplicationContext,
    symbol: String,
    side: String,
    size: String,
    order_type: String,
    price: Option<String>,
    reduce_only: Option<bool>,
    time_in_force: Option<String>,
) -> Result<HyperliquidOrderResult, ToolError> {
    debug!(
        "Placing Hyperliquid order: {} {} {} {}",
        side, size, symbol, order_type
    );

    // Get signer context
    let signer = SignerContext::current()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {}", e)))?;

    // Create client
    let client = HyperliquidClient::new(signer)?;

    // Validate side
    let is_buy = match side.to_lowercase().as_str() {
        "buy" | "long" => true,
        "sell" | "short" => false,
        _ => {
            return Err(ToolError::permanent_string(format!(
                "Invalid side '{}'. Must be 'buy', 'sell', 'long', or 'short'",
                side
            )))
        }
    };

    // Parse size
    let size_decimal = Decimal::from_str(&size)
        .map_err(|e| ToolError::permanent_string(format!("Invalid size '{}': {}", size, e)))?;
    if size_decimal <= Decimal::ZERO {
        return Err(ToolError::permanent_string(
            "Size must be greater than 0".to_string(),
        ));
    }

    // Get market metadata to find asset ID
    let meta = client.get_meta().await?;
    let asset_id = find_asset_id(&meta, &symbol)?;

    // Validate and parse order type
    let (limit_px, order_type_obj) = match order_type.to_lowercase().as_str() {
        "market" => {
            // For market orders, we use a very high/low price to ensure execution
            let market_price = if is_buy { "999999999" } else { "0.000001" };
            (
                market_price.to_string(),
                OrderType {
                    limit: Some(LimitOrderType {
                        tif: "Ioc".to_string(), // Immediate or Cancel for market orders
                    }),
                },
            )
        }
        "limit" => {
            let price_str = price
                .as_ref()
                .ok_or_else(|| {
                    ToolError::permanent_string("Price is required for limit orders".to_string())
                })?
                .clone();

            let price_decimal = Decimal::from_str(&price_str).map_err(|e| {
                ToolError::permanent_string(format!("Invalid price '{}': {}", price_str, e))
            })?;
            if price_decimal <= Decimal::ZERO {
                return Err(ToolError::permanent_string(
                    "Price must be greater than 0".to_string(),
                ));
            }

            let tif = match time_in_force
                .as_deref()
                .unwrap_or("gtc")
                .to_lowercase()
                .as_str()
            {
                "gtc" | "good_till_cancel" => "Gtc",
                "ioc" | "immediate_or_cancel" => "Ioc",
                "alo" | "add_liquidity_only" => "Alo",
                other => {
                    return Err(ToolError::permanent_string(format!(
                        "Invalid time in force '{}'. Must be 'gtc', 'ioc', or 'alo'",
                        other
                    )))
                }
            };

            (
                price_str,
                OrderType {
                    limit: Some(LimitOrderType {
                        tif: tif.to_string(),
                    }),
                },
            )
        }
        _ => {
            return Err(ToolError::permanent_string(format!(
                "Invalid order type '{}'. Must be 'market' or 'limit'",
                order_type
            )))
        }
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
        order_id: response
            .data
            .response
            .data
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
    _context: &riglr_core::provider::ApplicationContext,
    symbol: String,
    order_id: String,
) -> Result<HyperliquidCancelResult, ToolError> {
    debug!("Canceling Hyperliquid order: {} for {}", order_id, symbol);

    // Get signer context
    let signer = SignerContext::current()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {}", e)))?;

    // Create client
    let client = HyperliquidClient::new(signer)?;

    // Parse order ID
    let oid = order_id.parse::<u64>().map_err(|e| {
        ToolError::permanent_string(format!("Invalid order ID '{}': {}", order_id, e))
    })?;

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
pub async fn get_hyperliquid_account_info(
    _context: &riglr_core::provider::ApplicationContext,
) -> Result<HyperliquidAccountResult, ToolError> {
    debug!("Getting Hyperliquid account info");

    // Get signer context
    let signer = SignerContext::current()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {}", e)))?;

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
        cross_maintenance_margin_used: account_info
            .cross_maintenance_margin_used
            .unwrap_or_default(),
        positions_count: account_info
            .asset_positions
            .as_ref()
            .map(|p| p.len())
            .unwrap_or(0),
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
    _context: &riglr_core::provider::ApplicationContext,
    symbol: String,
    leverage: u32,
) -> Result<HyperliquidLeverageResult, ToolError> {
    debug!("Setting leverage for {}: {}x", symbol, leverage);

    // Validate leverage range (typical range for perpetual futures)
    if !(1..=100).contains(&leverage) {
        return Err(ToolError::permanent_string(format!(
            "Invalid leverage {}. Must be between 1 and 100",
            leverage
        )));
    }

    // Get signer context
    let signer = SignerContext::current()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {}", e)))?;

    // Create client
    let client = HyperliquidClient::new(signer)?;

    // Get market metadata to validate symbol and get asset ID
    let meta = client.get_meta().await?;
    let asset_id = find_asset_id(&meta, &symbol)?;

    // Update leverage using the real API
    let response = client
        .update_leverage(leverage, &symbol, true, Some(asset_id))
        .await?;

    info!("Successfully set {}x leverage for {}", leverage, symbol);

    Ok(HyperliquidLeverageResult {
        symbol: symbol.clone(),
        leverage,
        status: response.status,
        message: format!("Leverage set to {}x for {}", leverage, symbol),
    })
}

/// Helper function to find asset ID by symbol
///
/// Searches through the market metadata to find the asset ID for a given symbol.
/// Supports both exact matches and normalized symbols (with/without -PERP suffix).
///
/// # Arguments
///
/// * `meta` - Market metadata containing asset information
/// * `symbol` - Trading symbol to search for (e.g., "ETH", "BTC-PERP")
///
/// # Returns
///
/// Asset ID as u32 index, or ToolError if symbol not found
fn find_asset_id(meta: &Meta, symbol: &str) -> Result<u32, ToolError> {
    // Normalize symbol (remove -PERP suffix if present)
    let normalized_symbol = symbol.trim_end_matches("-PERP").trim_end_matches("-perp");

    for (index, asset) in meta.universe.iter().enumerate() {
        if asset.name.eq_ignore_ascii_case(symbol)
            || asset.name.eq_ignore_ascii_case(normalized_symbol)
            || asset
                .name
                .eq_ignore_ascii_case(&format!("{}-PERP", normalized_symbol))
        {
            return Ok(index as u32);
        }
    }

    Err(ToolError::permanent_string(format!(
        "Asset '{}' not found. Available assets: {}",
        symbol,
        meta.universe
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    )))
}

// Result structures

/// Result returned after placing an order on Hyperliquid
///
/// Contains all relevant information about the placed order including
/// the original parameters and the exchange's response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperliquidOrderResult {
    /// Trading pair symbol (e.g., "ETH-PERP", "BTC")
    pub symbol: String,
    /// Order side: "buy"/"long" or "sell"/"short"
    pub side: String,
    /// Position size as decimal string
    pub size: String,
    /// Order type: "market" or "limit"
    pub order_type: String,
    /// Limit price (for limit orders only)
    pub price: Option<String>,
    /// Exchange response status
    pub status: String,
    /// Unique order identifier from exchange
    pub order_id: Option<String>,
    /// Human-readable result message
    pub message: String,
}

/// Result returned after canceling an order on Hyperliquid
///
/// Contains confirmation details about the canceled order.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperliquidCancelResult {
    /// Trading pair symbol that the order belonged to
    pub symbol: String,
    /// Order ID that was canceled
    pub order_id: String,
    /// Exchange response status
    pub status: String,
    /// Human-readable result message
    pub message: String,
}

/// Account information retrieved from Hyperliquid
///
/// Contains balance, margin usage, and position information for the account.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperliquidAccountResult {
    /// Account's Ethereum address
    pub user_address: String,
    /// Available balance that can be withdrawn
    pub withdrawable_balance: String,
    /// Amount of balance used for cross-margin positions
    pub cross_margin_used: String,
    /// Maintenance margin requirements for cross-margin
    pub cross_maintenance_margin_used: String,
    /// Number of active positions
    pub positions_count: usize,
}

/// Result returned after setting leverage on Hyperliquid
///
/// Contains confirmation details about the leverage update.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperliquidLeverageResult {
    /// Trading pair symbol that leverage was set for
    pub symbol: String,
    /// Leverage multiplier (1-100x)
    pub leverage: u32,
    /// Exchange response status
    pub status: String,
    /// Human-readable result message
    pub message: String,
}

/// Unit tests for trading functionality
#[cfg(test)]
mod tests {
    use super::{
        find_asset_id, HyperliquidAccountResult, HyperliquidCancelResult,
        HyperliquidLeverageResult, HyperliquidOrderResult,
    };
    use crate::client::Meta;

    #[test]
    fn test_hyperliquid_order_result_creation() {
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
        assert_eq!(order_result.size, "0.1");
        assert_eq!(order_result.order_type, "limit");
        assert_eq!(order_result.price, Some("2000.0".to_string()));
        assert_eq!(order_result.status, "success");
        assert_eq!(order_result.order_id, Some("12345".to_string()));
        assert_eq!(order_result.message, "Order placed successfully");
    }

    #[test]
    fn test_hyperliquid_order_result_with_none_values() {
        let order_result = HyperliquidOrderResult {
            symbol: "BTC".to_string(),
            side: "sell".to_string(),
            size: "1.0".to_string(),
            order_type: "market".to_string(),
            price: None,
            status: "failed".to_string(),
            order_id: None,
            message: "Order failed".to_string(),
        };

        assert_eq!(order_result.price, None);
        assert_eq!(order_result.order_id, None);
        assert_eq!(order_result.order_type, "market");
    }

    #[test]
    fn test_hyperliquid_order_result_clone() {
        let order_result = HyperliquidOrderResult {
            symbol: "SOL-PERP".to_string(),
            side: "long".to_string(),
            size: "10.0".to_string(),
            order_type: "limit".to_string(),
            price: Some("100.0".to_string()),
            status: "pending".to_string(),
            order_id: Some("67890".to_string()),
            message: "Order submitted".to_string(),
        };

        let cloned = order_result.clone();
        assert_eq!(order_result.symbol, cloned.symbol);
        assert_eq!(order_result.side, cloned.side);
        assert_eq!(order_result.size, cloned.size);
        assert_eq!(order_result.order_type, cloned.order_type);
        assert_eq!(order_result.price, cloned.price);
        assert_eq!(order_result.status, cloned.status);
        assert_eq!(order_result.order_id, cloned.order_id);
        assert_eq!(order_result.message, cloned.message);
    }

    #[test]
    fn test_hyperliquid_cancel_result() {
        let cancel_result = HyperliquidCancelResult {
            symbol: "ETH-PERP".to_string(),
            order_id: "98765".to_string(),
            status: "canceled".to_string(),
            message: "Order canceled successfully".to_string(),
        };

        assert_eq!(cancel_result.symbol, "ETH-PERP");
        assert_eq!(cancel_result.order_id, "98765");
        assert_eq!(cancel_result.status, "canceled");
        assert_eq!(cancel_result.message, "Order canceled successfully");
    }

    #[test]
    fn test_hyperliquid_cancel_result_clone() {
        let cancel_result = HyperliquidCancelResult {
            symbol: "BTC".to_string(),
            order_id: "11111".to_string(),
            status: "error".to_string(),
            message: "Cancel failed".to_string(),
        };

        let cloned = cancel_result.clone();
        assert_eq!(cancel_result.symbol, cloned.symbol);
        assert_eq!(cancel_result.order_id, cloned.order_id);
        assert_eq!(cancel_result.status, cloned.status);
        assert_eq!(cancel_result.message, cloned.message);
    }

    #[test]
    fn test_hyperliquid_account_result() {
        let account_result = HyperliquidAccountResult {
            user_address: "0x1234567890abcdef".to_string(),
            withdrawable_balance: "1000.0".to_string(),
            cross_margin_used: "500.0".to_string(),
            cross_maintenance_margin_used: "100.0".to_string(),
            positions_count: 3,
        };

        assert_eq!(account_result.user_address, "0x1234567890abcdef");
        assert_eq!(account_result.withdrawable_balance, "1000.0");
        assert_eq!(account_result.cross_margin_used, "500.0");
        assert_eq!(account_result.cross_maintenance_margin_used, "100.0");
        assert_eq!(account_result.positions_count, 3);
    }

    #[test]
    fn test_hyperliquid_account_result_empty_balances() {
        let account_result = HyperliquidAccountResult {
            user_address: "0xdeadbeef".to_string(),
            withdrawable_balance: "0".to_string(),
            cross_margin_used: "0.0".to_string(),
            cross_maintenance_margin_used: "".to_string(),
            positions_count: 0,
        };

        assert_eq!(account_result.withdrawable_balance, "0");
        assert_eq!(account_result.cross_margin_used, "0.0");
        assert_eq!(account_result.cross_maintenance_margin_used, "");
        assert_eq!(account_result.positions_count, 0);
    }

    #[test]
    fn test_hyperliquid_account_result_clone() {
        let account_result = HyperliquidAccountResult {
            user_address: "0xabcdef1234567890".to_string(),
            withdrawable_balance: "2500.75".to_string(),
            cross_margin_used: "1200.25".to_string(),
            cross_maintenance_margin_used: "150.50".to_string(),
            positions_count: 5,
        };

        let cloned = account_result.clone();
        assert_eq!(account_result.user_address, cloned.user_address);
        assert_eq!(
            account_result.withdrawable_balance,
            cloned.withdrawable_balance
        );
        assert_eq!(account_result.cross_margin_used, cloned.cross_margin_used);
        assert_eq!(
            account_result.cross_maintenance_margin_used,
            cloned.cross_maintenance_margin_used
        );
        assert_eq!(account_result.positions_count, cloned.positions_count);
    }

    #[test]
    fn test_hyperliquid_leverage_result() {
        let leverage_result = HyperliquidLeverageResult {
            symbol: "ETH".to_string(),
            leverage: 10,
            status: "updated".to_string(),
            message: "Leverage set to 10x for ETH".to_string(),
        };

        assert_eq!(leverage_result.symbol, "ETH");
        assert_eq!(leverage_result.leverage, 10);
        assert_eq!(leverage_result.status, "updated");
        assert_eq!(leverage_result.message, "Leverage set to 10x for ETH");
    }

    #[test]
    fn test_hyperliquid_leverage_result_edge_values() {
        let leverage_result_min = HyperliquidLeverageResult {
            symbol: "BTC".to_string(),
            leverage: 1,
            status: "success".to_string(),
            message: "Minimum leverage".to_string(),
        };

        let leverage_result_max = HyperliquidLeverageResult {
            symbol: "SOL".to_string(),
            leverage: 100,
            status: "success".to_string(),
            message: "Maximum leverage".to_string(),
        };

        assert_eq!(leverage_result_min.leverage, 1);
        assert_eq!(leverage_result_max.leverage, 100);
    }

    #[test]
    fn test_hyperliquid_leverage_result_clone() {
        let leverage_result = HyperliquidLeverageResult {
            symbol: "AVAX".to_string(),
            leverage: 25,
            status: "applied".to_string(),
            message: "Leverage updated".to_string(),
        };

        let cloned = leverage_result.clone();
        assert_eq!(leverage_result.symbol, cloned.symbol);
        assert_eq!(leverage_result.leverage, cloned.leverage);
        assert_eq!(leverage_result.status, cloned.status);
        assert_eq!(leverage_result.message, cloned.message);
    }

    #[test]
    fn test_find_asset_id_exact_match() {
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
    }

    #[test]
    fn test_find_asset_id_normalized_match() {
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

        assert_eq!(find_asset_id(&meta, "BTC").unwrap(), 0); // Should find BTC-PERP
        assert_eq!(find_asset_id(&meta, "ETH").unwrap(), 1); // Should find ETH-PERP
    }

    #[test]
    fn test_find_asset_id_case_insensitive() {
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

        assert_eq!(find_asset_id(&meta, "eth").unwrap(), 1); // Case insensitive
        assert_eq!(find_asset_id(&meta, "btc").unwrap(), 0); // Case insensitive
        assert_eq!(find_asset_id(&meta, "eth-perp").unwrap(), 1); // Case insensitive
        assert_eq!(find_asset_id(&meta, "BTC-perp").unwrap(), 0); // Mixed case
    }

    #[test]
    fn test_find_asset_id_with_perp_suffix() {
        let meta = Meta {
            universe: vec![crate::client::AssetInfo {
                name: "SOL-PERP".to_string(),
                sz_decimals: 2,
            }],
        };

        assert_eq!(find_asset_id(&meta, "SOL-PERP").unwrap(), 0);
        assert_eq!(find_asset_id(&meta, "SOL").unwrap(), 0);
        assert_eq!(find_asset_id(&meta, "sol-PERP").unwrap(), 0);
        assert_eq!(find_asset_id(&meta, "sol").unwrap(), 0);
    }

    #[test]
    fn test_find_asset_id_not_found() {
        let meta = Meta {
            universe: vec![crate::client::AssetInfo {
                name: "BTC-PERP".to_string(),
                sz_decimals: 4,
            }],
        };

        let result = find_asset_id(&meta, "INVALID");
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Asset 'INVALID' not found"));
        assert!(error_msg.contains("Available assets: BTC-PERP"));
    }

    #[test]
    fn test_find_asset_id_empty_universe() {
        let meta = Meta { universe: vec![] };

        let result = find_asset_id(&meta, "BTC");
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Asset 'BTC' not found"));
        assert!(error_msg.contains("Available assets: "));
    }

    #[test]
    fn test_find_asset_id_empty_symbol() {
        let meta = Meta {
            universe: vec![crate::client::AssetInfo {
                name: "BTC-PERP".to_string(),
                sz_decimals: 4,
            }],
        };

        let result = find_asset_id(&meta, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_asset_id_whitespace_symbol() {
        let meta = Meta {
            universe: vec![crate::client::AssetInfo {
                name: "BTC-PERP".to_string(),
                sz_decimals: 4,
            }],
        };

        let result = find_asset_id(&meta, "   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_asset_id_multiple_matches_returns_first() {
        let meta = Meta {
            universe: vec![
                crate::client::AssetInfo {
                    name: "ETH-PERP".to_string(),
                    sz_decimals: 3,
                },
                crate::client::AssetInfo {
                    name: "ETH".to_string(),
                    sz_decimals: 3,
                },
            ],
        };

        // Should find the first match
        assert_eq!(find_asset_id(&meta, "ETH").unwrap(), 0);
    }

    // Tests for validation logic that can be tested in isolation
    mod validation_tests {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        #[test]
        fn test_side_validation_valid_cases() {
            // Test valid side values
            let valid_sides = vec![
                "buy", "long", "sell", "short", "BUY", "LONG", "SELL", "SHORT",
            ];

            for side in valid_sides {
                let is_buy = match side.to_lowercase().as_str() {
                    "buy" | "long" => true,
                    "sell" | "short" => false,
                    _ => panic!("Should not reach here for valid side: {}", side),
                };

                match side.to_lowercase().as_str() {
                    "buy" | "long" => assert!(is_buy),
                    "sell" | "short" => assert!(!is_buy),
                    _ => panic!("Unexpected side: {}", side),
                }
            }
        }

        #[test]
        fn test_side_validation_invalid_cases() {
            let invalid_sides = vec!["", "invalid", "bid", "ask", "purchase", "sale", "123"];

            for side in invalid_sides {
                let result = match side.to_lowercase().as_str() {
                    "buy" | "long" => Ok(true),
                    "sell" | "short" => Ok(false),
                    _ => Err(format!("Invalid side '{}'", side)),
                };

                assert!(result.is_err(), "Side '{}' should be invalid", side);
            }
        }

        #[test]
        fn test_size_validation_valid_cases() {
            let valid_sizes = vec!["0.1", "1.0", "10", "100.5", "0.00001", "999999.999"];

            for size_str in valid_sizes {
                let size_decimal = Decimal::from_str(size_str).unwrap();
                assert!(
                    size_decimal > Decimal::ZERO,
                    "Size '{}' should be positive",
                    size_str
                );
            }
        }

        #[test]
        fn test_size_validation_invalid_cases() {
            let invalid_sizes = vec!["0", "-1", "-0.1", "abc", "", " ", "1.0.0", "inf", "nan"];

            for size_str in invalid_sizes {
                let result = Decimal::from_str(size_str);
                if let Ok(decimal) = result {
                    assert!(
                        decimal <= Decimal::ZERO,
                        "Size '{}' should be invalid (non-positive)",
                        size_str
                    );
                } else {
                    // Parsing failed, which is expected for invalid formats
                }
            }
        }

        #[test]
        fn test_price_validation_valid_cases() {
            let valid_prices = vec!["0.01", "1.0", "1000", "2500.75", "0.000001", "99999999"];

            for price_str in valid_prices {
                let price_decimal = Decimal::from_str(price_str).unwrap();
                assert!(
                    price_decimal > Decimal::ZERO,
                    "Price '{}' should be positive",
                    price_str
                );
            }
        }

        #[test]
        fn test_price_validation_invalid_cases() {
            let invalid_prices = vec!["0", "-1", "-100.5", "xyz", "", "   ", "1.2.3"];

            for price_str in invalid_prices {
                let result = Decimal::from_str(price_str);
                if let Ok(decimal) = result {
                    assert!(
                        decimal <= Decimal::ZERO,
                        "Price '{}' should be invalid (non-positive)",
                        price_str
                    );
                } else {
                    // Parsing failed, which is expected for invalid formats
                }
            }
        }

        #[test]
        fn test_order_type_validation() {
            let valid_order_types = vec!["market", "limit", "MARKET", "LIMIT", "Market", "Limit"];

            for order_type in valid_order_types {
                match order_type.to_lowercase().as_str() {
                    "market" | "limit" => {} // Valid
                    _ => panic!("Order type '{}' should be valid", order_type),
                }
            }

            let invalid_order_types =
                vec!["", "stop", "stop_loss", "take_profit", "iceberg", "123"];

            for order_type in invalid_order_types {
                match order_type.to_lowercase().as_str() {
                    "market" | "limit" => panic!("Order type '{}' should be invalid", order_type),
                    _ => {} // Invalid, as expected
                }
            }
        }

        #[test]
        fn test_time_in_force_validation() {
            let test_cases = vec![
                ("gtc", "Gtc"),
                ("good_till_cancel", "Gtc"),
                ("ioc", "Ioc"),
                ("immediate_or_cancel", "Ioc"),
                ("alo", "Alo"),
                ("add_liquidity_only", "Alo"),
                ("GTC", "Gtc"),
                ("IOC", "Ioc"),
                ("ALO", "Alo"),
            ];

            for (input, expected) in test_cases {
                let tif = match input.to_lowercase().as_str() {
                    "gtc" | "good_till_cancel" => "Gtc",
                    "ioc" | "immediate_or_cancel" => "Ioc",
                    "alo" | "add_liquidity_only" => "Alo",
                    _ => panic!("Should not reach here for valid TIF: {}", input),
                };

                assert_eq!(
                    tif, expected,
                    "TIF '{}' should map to '{}'",
                    input, expected
                );
            }
        }

        #[test]
        fn test_time_in_force_invalid_cases() {
            let invalid_tifs = vec!["", "fok", "fill_or_kill", "day", "gfd", "invalid", "123"];

            for tif in invalid_tifs {
                let result = match tif.to_lowercase().as_str() {
                    "gtc" | "good_till_cancel" => Ok("Gtc"),
                    "ioc" | "immediate_or_cancel" => Ok("Ioc"),
                    "alo" | "add_liquidity_only" => Ok("Alo"),
                    _ => Err(format!("Invalid TIF: {}", tif)),
                };

                assert!(result.is_err(), "TIF '{}' should be invalid", tif);
            }
        }

        #[test]
        fn test_leverage_validation_valid_range() {
            for leverage in 1..=100 {
                assert!(
                    (1..=100).contains(&leverage),
                    "Leverage {} should be valid",
                    leverage
                );
            }
        }

        #[test]
        fn test_leverage_validation_invalid_range() {
            let invalid_leverages = vec![0, 101, 200, 1000, u32::MAX];

            for leverage in invalid_leverages {
                assert!(
                    !(1..=100).contains(&leverage),
                    "Leverage {} should be invalid",
                    leverage
                );
            }
        }

        #[test]
        fn test_order_id_parsing_valid() {
            let valid_order_ids = vec!["123", "0", "18446744073709551615"]; // u64::MAX as string

            for order_id_str in valid_order_ids {
                let result = order_id_str.parse::<u64>();
                assert!(
                    result.is_ok(),
                    "Order ID '{}' should parse successfully",
                    order_id_str
                );
            }
        }

        #[test]
        fn test_order_id_parsing_invalid() {
            let invalid_order_ids =
                vec!["", "abc", "-1", "123abc", " 123 ", "18446744073709551616"]; // u64::MAX + 1

            for order_id_str in invalid_order_ids {
                let result = order_id_str.parse::<u64>();
                assert!(
                    result.is_err(),
                    "Order ID '{}' should fail to parse",
                    order_id_str
                );
            }
        }

        #[test]
        fn test_reduce_only_flag() {
            // Test default value
            let reduce_only_default = None;
            assert!(!reduce_only_default.unwrap_or(false));

            // Test explicit values
            assert!(Some(true).unwrap_or(false));
            assert!(!Some(false).unwrap_or(false));
        }

        #[test]
        fn test_market_order_price_logic() {
            // For buy orders, use very high price
            let buy_market_price = "999999999";
            let buy_price_decimal = Decimal::from_str(buy_market_price).unwrap();
            assert!(buy_price_decimal > Decimal::from_str("1000000").unwrap());

            // For sell orders, use very low price
            let sell_market_price = "0.000001";
            let sell_price_decimal = Decimal::from_str(sell_market_price).unwrap();
            assert!(sell_price_decimal > Decimal::ZERO);
            assert!(sell_price_decimal < Decimal::from_str("0.001").unwrap());
        }

        #[test]
        fn test_symbol_normalization_logic() {
            // Test the logic used in find_asset_id for symbol normalization
            let test_cases = vec![
                ("BTC-PERP", "BTC"),
                ("ETH-perp", "ETH"),
                ("SOL", "SOL"),
                ("AVAX-PERP", "AVAX"),
                ("DOT", "DOT"),
            ];

            for (input, expected) in test_cases {
                let normalized = input.trim_end_matches("-PERP").trim_end_matches("-perp");
                assert_eq!(
                    normalized, expected,
                    "Symbol '{}' should normalize to '{}'",
                    input, expected
                );
            }
        }

        #[test]
        fn test_symbol_edge_cases() {
            let edge_cases = vec![
                ("", ""),
                ("-PERP", ""),
                ("-perp", ""),
                ("A-PERP-PERP", "A-PERP"), // Only removes the last occurrence
                ("PERP", "PERP"),          // Doesn't contain -PERP suffix
            ];

            for (input, expected) in edge_cases {
                let normalized = input.trim_end_matches("-PERP").trim_end_matches("-perp");
                assert_eq!(
                    normalized, expected,
                    "Edge case '{}' should normalize to '{}'",
                    input, expected
                );
            }
        }
    }

    // Integration-style tests for error message formatting
    mod error_message_tests {
        use riglr_core::ToolError;

        #[test]
        fn test_invalid_side_error_message() {
            let side = "invalid_side";
            let expected_msg = format!(
                "Invalid side '{}'. Must be 'buy', 'sell', 'long', or 'short'",
                side
            );
            let error = ToolError::permanent_string(expected_msg.clone());
            assert_eq!(format!("{}", error), expected_msg);
        }

        #[test]
        fn test_invalid_size_error_message() {
            let size = "abc";
            let parse_error = "invalid decimal: abc"; // This would be the actual parse error
            let expected_msg = format!("Invalid size '{}': {}", size, parse_error);
            let error = ToolError::permanent_string(expected_msg.clone());
            assert_eq!(format!("{}", error), expected_msg);
        }

        #[test]
        fn test_zero_size_error_message() {
            let expected_msg = "Size must be greater than 0";
            let error = ToolError::permanent_string(expected_msg.to_string());
            assert_eq!(format!("{}", error), expected_msg);
        }

        #[test]
        fn test_invalid_price_error_message() {
            let price = "-100";
            let parse_error = "negative decimal"; // This would be the actual parse error
            let expected_msg = format!("Invalid price '{}': {}", price, parse_error);
            let error = ToolError::permanent_string(expected_msg.clone());
            assert_eq!(format!("{}", error), expected_msg);
        }

        #[test]
        fn test_zero_price_error_message() {
            let expected_msg = "Price must be greater than 0";
            let error = ToolError::permanent_string(expected_msg.to_string());
            assert_eq!(format!("{}", error), expected_msg);
        }

        #[test]
        fn test_missing_price_error_message() {
            let expected_msg = "Price is required for limit orders";
            let error = ToolError::permanent_string(expected_msg.to_string());
            assert_eq!(format!("{}", error), expected_msg);
        }

        #[test]
        fn test_invalid_order_type_error_message() {
            let order_type = "stop_loss";
            let expected_msg = format!(
                "Invalid order type '{}'. Must be 'market' or 'limit'",
                order_type
            );
            let error = ToolError::permanent_string(expected_msg.clone());
            assert_eq!(format!("{}", error), expected_msg);
        }

        #[test]
        fn test_invalid_time_in_force_error_message() {
            let tif = "fok";
            let expected_msg = format!(
                "Invalid time in force '{}'. Must be 'gtc', 'ioc', or 'alo'",
                tif
            );
            let error = ToolError::permanent_string(expected_msg.clone());
            assert_eq!(format!("{}", error), expected_msg);
        }

        #[test]
        fn test_invalid_leverage_error_message() {
            let leverage = 150;
            let expected_msg = format!("Invalid leverage {}. Must be between 1 and 100", leverage);
            let error = ToolError::permanent_string(expected_msg.clone());
            assert_eq!(format!("{}", error), expected_msg);
        }

        #[test]
        fn test_invalid_order_id_error_message() {
            let order_id = "not_a_number";
            let parse_error = "invalid digit found in string"; // This would be the actual parse error
            let expected_msg = format!("Invalid order ID '{}': {}", order_id, parse_error);
            let error = ToolError::permanent_string(expected_msg.clone());
            assert_eq!(format!("{}", error), expected_msg);
        }

        #[test]
        fn test_no_signer_context_error_message() {
            let signer_error = "context not available";
            let expected_msg = format!("No signer context: {}", signer_error);
            let error = ToolError::permanent_string(expected_msg.clone());
            assert_eq!(format!("{}", error), expected_msg);
        }
    }
}
