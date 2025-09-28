//! Trading tools for Asterdex Futures v3
//!
//! This module provides tools for placing, canceling, and managing orders on Asterdex.

use crate::client::{Client, Order, OrderResponse};
use core::str::FromStr;
use riglr_core::provider::ApplicationContext;
use riglr_core::{SignerContext, ToolError};
use riglr_macros::tool;
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::{debug, info};

/// Parameters for placing an Asterdex order
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OrderParams {
    /// Trading symbol (e.g., "BTCUSDT", "ETHUSDT", "SOLUSDT")
    pub symbol: String,
    /// Order side: "BUY" or "SELL"
    pub side: String,
    /// Order quantity as string to preserve precision
    pub quantity: String,
    /// Order type: "MARKET" or "LIMIT"
    pub order_type: String,
    /// Price for limit orders (required for limit orders)
    pub price: Option<String>,
    /// Position side: "LONG", "SHORT", or "BOTH" (default: "BOTH")
    pub position_side: Option<String>,
    /// Whether this is a reduce-only order
    pub reduce_only: Option<bool>,
    /// Time in force: "GTC" (good till cancel), "IOC" (immediate or cancel), "FOK" (fill or kill), "GTX" (good till crossing)
    pub time_in_force: Option<String>,
}

/// Result from placing an Asterdex order
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AsterdexOrderResult {
    /// Order ID assigned by Asterdex
    pub order_id: i64,
    /// Trading symbol
    pub symbol: String,
    /// Order side (BUY/SELL)
    pub side: String,
    /// Order type (MARKET/LIMIT)
    pub order_type: String,
    /// Order status
    pub status: String,
    /// Original quantity
    pub quantity: String,
    /// Order price (for limit orders)
    pub price: Option<String>,
    /// Human-readable result message
    pub message: String,
}

/// Parameters for canceling an Asterdex order
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CancelOrderParams {
    /// Trading symbol
    pub symbol: String,
    /// Order ID to cancel
    pub order_id: Option<i64>,
    /// Client order ID to cancel (alternative to `order_id`)
    pub orig_client_order_id: Option<String>,
}

/// Result from canceling an Asterdex order
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AsterdexCancelResult {
    /// Order ID that was canceled
    pub order_id: i64,
    /// Trading symbol
    pub symbol: String,
    /// Order status after cancellation
    pub status: String,
    /// Human-readable result message
    pub message: String,
}

/// Place a futures order on Asterdex
///
/// This tool places market or limit orders for perpetual futures on the Asterdex exchange.
/// It supports both long and short positions with configurable time-in-force options
/// and risk management features like reduce-only orders.
///
/// # Arguments
///
/// * `params.symbol` - Trading pair symbol (e.g., "BTCUSDT", "ETHUSDT")
/// * `params.side` - Order side: "BUY" or "SELL"
/// * `params.quantity` - Position size as decimal string (e.g., "0.1")
/// * `params.order_type` - "MARKET" for immediate execution or "LIMIT" for price-specific order
/// * `params.price` - Limit price (required for limit orders, ignored for market orders)
/// * `params.position_side` - "LONG", "SHORT", or "BOTH" (default: "BOTH")
/// * `params.reduce_only` - If true, order can only reduce existing position size
/// * `params.time_in_force` - "GTC" (good-till-cancel), "IOC" (immediate-or-cancel), "FOK" (fill-or-kill), "GTX" (good-till-crossing)
///
/// # Returns
///
/// Returns `AsterdexOrderResult` containing:
/// - Order details (symbol, side, quantity, type, price)
/// - `status`: Order status
/// - `order_id`: Unique order identifier for tracking
/// - `message`: Human-readable result description
///
/// # Errors
///
/// * `InvalidInput` - When parameters are invalid (negative size, unknown symbol)
/// * `ApiError` - When Asterdex API rejects the order
/// * `NetworkError` - When connection issues occur
/// * `AuthError` - When authentication fails
#[tool]
#[allow(clippy::too_many_lines)]
pub async fn place_asterdex_order(
    _context: &ApplicationContext,
    params: OrderParams,
) -> Result<AsterdexOrderResult, ToolError> {
    debug!(
        "Placing Asterdex order: {} {} {} {}",
        params.side, params.quantity, params.symbol, params.order_type
    );

    // Get signer context
    let signer = SignerContext::current()
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {e}")))?;

    // Create client
    let client = Client::new(signer).map_err(ToolError::from)?;

    // Validate side
    let side = match params.side.to_uppercase().as_str() {
        "BUY" => "BUY",
        "SELL" => "SELL",
        _ => {
            return Err(ToolError::permanent_string(format!(
                "Invalid side '{}'. Must be 'BUY' or 'SELL'",
                params.side
            )))
        }
    };

    // Parse quantity
    let quantity_decimal = Decimal::from_str(&params.quantity).map_err(|e| {
        ToolError::permanent_string(format!("Invalid quantity '{}': {e}", params.quantity))
    })?;
    if quantity_decimal <= Decimal::ZERO {
        return Err(ToolError::permanent_string(
            "Quantity must be greater than 0".to_string(),
        ));
    }

    // Build order parameters
    let mut order_params = BTreeMap::new();
    order_params.insert("symbol".to_string(), params.symbol.clone());
    order_params.insert("side".to_string(), side.to_string());
    order_params.insert("quantity".to_string(), params.quantity.clone());

    // Handle position side
    let position_side = params.position_side.unwrap_or_else(|| "BOTH".to_string());
    order_params.insert("positionSide".to_string(), position_side);

    // Validate and set order type
    let order_type = match params.order_type.to_uppercase().as_str() {
        "MARKET" => {
            order_params.insert("type".to_string(), "MARKET".to_string());
            "MARKET"
        }
        "LIMIT" => {
            let price_str = params
                .price
                .as_ref()
                .ok_or_else(|| {
                    ToolError::permanent_string("Price is required for limit orders".to_string())
                })?
                .clone();

            // Validate price
            let price_decimal = Decimal::from_str(&price_str).map_err(|e| {
                ToolError::permanent_string(format!("Invalid price '{price_str}': {e}"))
            })?;
            if price_decimal <= Decimal::ZERO {
                return Err(ToolError::permanent_string(
                    "Price must be greater than 0".to_string(),
                ));
            }

            order_params.insert("type".to_string(), "LIMIT".to_string());
            order_params.insert("price".to_string(), price_str);

            // Time in force (default to GTC for limit orders)
            let tif = params
                .time_in_force
                .unwrap_or_else(|| "GTC".to_string())
                .to_uppercase();
            if !["GTC", "IOC", "FOK", "GTX"].contains(&tif.as_str()) {
                return Err(ToolError::permanent_string(format!(
                    "Invalid time_in_force '{tif}'. Must be GTC, IOC, FOK, or GTX"
                )));
            }
            order_params.insert("timeInForce".to_string(), tif);

            "LIMIT"
        }
        _ => {
            return Err(ToolError::permanent_string(format!(
                "Invalid order type '{}'. Must be 'MARKET' or 'LIMIT'",
                params.order_type
            )))
        }
    };

    // Add reduce_only if specified
    if let Some(reduce_only) = params.reduce_only {
        order_params.insert("reduceOnly".to_string(), reduce_only.to_string());
    }

    info!(
        "Submitting {} {} order for {} {} at {}",
        order_type,
        side,
        params.quantity,
        params.symbol,
        params.price.as_deref().unwrap_or("market")
    );

    // Place the order
    let response = client
        .post("/fapi/v3/order", order_params)
        .await
        .map_err(ToolError::from)?;

    // Parse response
    let order_response: OrderResponse = response
        .json()
        .await
        .map_err(|e| ToolError::permanent_string(format!("Failed to parse order response: {e}")))?;

    Ok(AsterdexOrderResult {
        order_id: order_response.order_id,
        symbol: order_response.symbol.clone(),
        side: order_response.side.clone(),
        order_type: order_response.order_type,
        status: order_response.status,
        quantity: order_response.orig_qty.clone(),
        price: order_response.price.clone(),
        message: format!(
            "Order {} placed successfully: {} {} {} at {}",
            order_response.order_id,
            order_response.side,
            order_response.orig_qty,
            order_response.symbol,
            order_response.price.as_deref().unwrap_or("market")
        ),
    })
}

/// Cancel an open order on Asterdex
///
/// This tool cancels an existing order on the Asterdex exchange.
///
/// # Arguments
///
/// * `params.symbol` - Trading pair symbol (e.g., "BTCUSDT", "ETHUSDT")
/// * `params.order_id` - Order ID to cancel
/// * `params.orig_client_order_id` - Client order ID to cancel (alternative to `order_id`)
///
/// # Returns
///
/// Returns `AsterdexCancelResult` containing:
/// - `order_id`: ID of the canceled order
/// - `symbol`: Trading symbol
/// - `status`: Order status after cancellation
/// - `message`: Human-readable result description
///
/// # Errors
///
/// * `InvalidInput` - When neither `order_id` nor `orig_client_order_id` is provided
/// * `ApiError` - When Asterdex API rejects the cancellation
/// * `NetworkError` - When connection issues occur
///
/// Note: The Asterdex v3 documentation does not explicitly specify the endpoint for
/// canceling an order. This implementation uses `DELETE /fapi/v3/order`, assuming
/// the pattern from the v1/v2 API (`DELETE /fapi/v1/order`) is carried forward to v3.
/// This is a reasonable assumption for `RESTful` `API` design.
#[tool]
pub async fn cancel_asterdex_order(
    _context: &ApplicationContext,
    params: CancelOrderParams,
) -> Result<AsterdexCancelResult, ToolError> {
    debug!("Canceling Asterdex order for symbol: {}", params.symbol);

    // Get signer context
    let signer = SignerContext::current()
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {e}")))?;

    // Create client
    let client = Client::new(signer).map_err(ToolError::from)?;

    // Build cancel parameters
    let mut cancel_params = BTreeMap::new();
    cancel_params.insert("symbol".to_string(), params.symbol.clone());

    // Add order ID or client order ID
    if let Some(order_id) = params.order_id {
        cancel_params.insert("orderId".to_string(), order_id.to_string());
    } else if let Some(client_order_id) = params.orig_client_order_id {
        cancel_params.insert("origClientOrderId".to_string(), client_order_id);
    } else {
        return Err(ToolError::permanent_string(
            "Either order_id or orig_client_order_id must be provided".to_string(),
        ));
    }

    info!("Canceling order for {}", params.symbol);

    // Cancel the order
    let response = client
        .delete("/fapi/v3/order", cancel_params)
        .await
        .map_err(ToolError::from)?;

    // Parse response
    let order: Order = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse cancel response: {e}"))
    })?;

    Ok(AsterdexCancelResult {
        order_id: order.order_id,
        symbol: order.symbol,
        status: order.status,
        message: format!("Order {} canceled successfully", order.order_id),
    })
}

/// Get information about a specific order on Asterdex
///
/// This tool retrieves information about an existing order on the Asterdex exchange.
///
/// # Arguments
///
/// * `symbol` - Trading pair symbol (e.g., "BTCUSDT", "ETHUSDT")
/// * `order_id` - Order ID to query
///
/// # Returns
///
/// Returns complete order information including:
/// - Order details (symbol, side, quantity, type, price)
/// - Execution status and filled amounts
/// - Timestamps
///
/// # Errors
///
/// * `ApiError` - When Asterdex API cannot find the order
/// * `NetworkError` - When connection issues occur
#[tool]
pub async fn get_asterdex_order(
    _context: &ApplicationContext,
    symbol: String,
    order_id: i64,
) -> Result<Order, ToolError> {
    debug!("Getting Asterdex order {} for symbol {}", order_id, symbol);

    // Get signer context
    let signer = SignerContext::current()
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {e}")))?;

    // Create client
    let client = Client::new(signer).map_err(ToolError::from)?;

    // Build query parameters
    let mut query_params = BTreeMap::new();
    query_params.insert("symbol".to_string(), symbol.clone());
    query_params.insert("orderId".to_string(), order_id.to_string());

    info!("Querying order {} for {}", order_id, symbol);

    // Get the order
    let response = client
        .get("/fapi/v3/order", query_params)
        .await
        .map_err(ToolError::from)?;

    // Parse response
    let order: Order = response
        .json()
        .await
        .map_err(|e| ToolError::permanent_string(format!("Failed to parse order response: {e}")))?;

    Ok(order)
}

/// Set leverage for a trading pair on Asterdex
///
/// This tool adjusts the leverage for a specific trading pair on Asterdex.
///
/// # Arguments
///
/// * `symbol` - Trading pair symbol (e.g., "BTCUSDT", "ETHUSDT")
/// * `leverage` - Desired leverage (1-125)
///
/// # Returns
///
/// Returns the updated leverage information
///
/// # Errors
///
/// * `InvalidInput` - When leverage is outside valid range
/// * `ApiError` - When Asterdex API rejects the change
/// * `NetworkError` - When connection issues occur
///
/// Note: As of the current Asterdex v3 documentation, there is no specified v3
/// endpoint for changing leverage. This tool uses the `/fapi/v1/leverage` endpoint,
/// which remains functional. This implementation should be updated if a v3
/// equivalent becomes available.
#[tool]
pub async fn set_asterdex_leverage(
    _context: &ApplicationContext,
    symbol: String,
    leverage: u32,
) -> Result<serde_json::Value, ToolError> {
    debug!("Setting leverage for {} to {}x", symbol, leverage);

    // Validate leverage
    if !(1..=125).contains(&leverage) {
        return Err(ToolError::permanent_string(
            "Leverage must be between 1 and 125".to_string(),
        ));
    }

    // Get signer context
    let signer = SignerContext::current()
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {e}")))?;

    // Create client
    let client = Client::new(signer).map_err(ToolError::from)?;

    // Build parameters
    let mut params = BTreeMap::new();
    params.insert("symbol".to_string(), symbol.clone());
    params.insert("leverage".to_string(), leverage.to_string());

    info!("Setting leverage for {} to {}x", symbol, leverage);

    // Set the leverage (Note: This might still be v1 endpoint as per docs)
    let response = client
        .post("/fapi/v1/leverage", params)
        .await
        .map_err(ToolError::from)?;

    // Parse response as generic JSON
    let result: serde_json::Value = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse leverage response: {e}"))
    })?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_params_structure() {
        let params = OrderParams {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: "0.1".to_string(),
            order_type: "MARKET".to_string(),
            price: None,
            position_side: None,
            reduce_only: None,
            time_in_force: None,
        };

        assert_eq!(params.symbol, "BTCUSDT");
        assert_eq!(params.side, "BUY");
        assert_eq!(params.quantity, "0.1");
        assert_eq!(params.order_type, "MARKET");
        assert!(params.price.is_none());
    }

    #[test]
    fn test_order_params_with_limit_order() {
        let params = OrderParams {
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            quantity: "1.5".to_string(),
            order_type: "LIMIT".to_string(),
            price: Some("2000.50".to_string()),
            position_side: Some("LONG".to_string()),
            reduce_only: Some(false),
            time_in_force: Some("GTC".to_string()),
        };

        assert_eq!(params.symbol, "ETHUSDT");
        assert_eq!(params.side, "SELL");
        assert_eq!(params.quantity, "1.5");
        assert_eq!(params.order_type, "LIMIT");
        assert_eq!(params.price, Some("2000.50".to_string()));
        assert_eq!(params.position_side, Some("LONG".to_string()));
        assert_eq!(params.reduce_only, Some(false));
        assert_eq!(params.time_in_force, Some("GTC".to_string()));
    }

    #[test]
    fn test_asterdex_order_result_structure() {
        let result = AsterdexOrderResult {
            order_id: 123_456,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            status: "FILLED".to_string(),
            quantity: "0.1".to_string(),
            price: None,
            message: "Order placed successfully".to_string(),
        };

        assert_eq!(result.order_id, 123_456);
        assert_eq!(result.symbol, "BTCUSDT");
        assert_eq!(result.side, "BUY");
        assert_eq!(result.order_type, "MARKET");
        assert_eq!(result.status, "FILLED");
        assert_eq!(result.quantity, "0.1");
        assert!(result.price.is_none());
        assert_eq!(result.message, "Order placed successfully");
    }

    #[test]
    fn test_cancel_order_params_with_order_id() {
        let params = CancelOrderParams {
            symbol: "BTCUSDT".to_string(),
            order_id: Some(123_456),
            orig_client_order_id: None,
        };

        assert_eq!(params.symbol, "BTCUSDT");
        assert_eq!(params.order_id, Some(123_456));
        assert!(params.orig_client_order_id.is_none());
    }

    #[test]
    fn test_cancel_order_params_with_client_order_id() {
        let params = CancelOrderParams {
            symbol: "ETHUSDT".to_string(),
            order_id: None,
            orig_client_order_id: Some("CLIENT_ORDER_123".to_string()),
        };

        assert_eq!(params.symbol, "ETHUSDT");
        assert!(params.order_id.is_none());
        assert_eq!(
            params.orig_client_order_id,
            Some("CLIENT_ORDER_123".to_string())
        );
    }

    #[test]
    fn test_asterdex_cancel_result_structure() {
        let result = AsterdexCancelResult {
            order_id: 654_321,
            symbol: "BTCUSDT".to_string(),
            status: "CANCELED".to_string(),
            message: "Order canceled successfully".to_string(),
        };

        assert_eq!(result.order_id, 654_321);
        assert_eq!(result.symbol, "BTCUSDT");
        assert_eq!(result.status, "CANCELED");
        assert_eq!(result.message, "Order canceled successfully");
    }

    #[test]
    fn test_order_params_clone() {
        let params1 = OrderParams {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: "0.1".to_string(),
            order_type: "MARKET".to_string(),
            price: None,
            position_side: None,
            reduce_only: None,
            time_in_force: None,
        };

        let params2 = params1.clone();
        assert_eq!(params1.symbol, params2.symbol);
        assert_eq!(params1.side, params2.side);
        assert_eq!(params1.quantity, params2.quantity);
        assert_eq!(params1.order_type, params2.order_type);
    }

    #[test]
    fn test_cancel_order_params_clone() {
        let params1 = CancelOrderParams {
            symbol: "BTCUSDT".to_string(),
            order_id: Some(123_456),
            orig_client_order_id: None,
        };

        let params2 = params1.clone();
        assert_eq!(params1.symbol, params2.symbol);
        assert_eq!(params1.order_id, params2.order_id);
        assert_eq!(params1.orig_client_order_id, params2.orig_client_order_id);
    }

    #[test]
    fn test_order_params_debug_trait() {
        let params = OrderParams {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: "0.1".to_string(),
            order_type: "MARKET".to_string(),
            price: None,
            position_side: None,
            reduce_only: None,
            time_in_force: None,
        };

        let debug_str = format!("{params:?}");
        assert!(debug_str.contains("OrderParams"));
        assert!(debug_str.contains("BTCUSDT"));
        assert!(debug_str.contains("BUY"));
    }

    #[test]
    fn test_cancel_order_params_debug_trait() {
        let params = CancelOrderParams {
            symbol: "BTCUSDT".to_string(),
            order_id: Some(123_456),
            orig_client_order_id: None,
        };

        let debug_str = format!("{params:?}");
        assert!(debug_str.contains("CancelOrderParams"));
        assert!(debug_str.contains("BTCUSDT"));
        assert!(debug_str.contains("123456"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_order_params_serialization() {
        let params = OrderParams {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: "0.1".to_string(),
            order_type: "MARKET".to_string(),
            price: None,
            position_side: None,
            reduce_only: None,
            time_in_force: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"symbol\":\"BTCUSDT\""));
        assert!(json.contains("\"side\":\"BUY\""));
        assert!(json.contains("\"quantity\":\"0.1\""));
        assert!(json.contains("\"order_type\":\"MARKET\""));

        let deserialized: OrderParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, params.symbol);
        assert_eq!(deserialized.side, params.side);
        assert_eq!(deserialized.quantity, params.quantity);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_cancel_order_params_serialization() {
        let params = CancelOrderParams {
            symbol: "BTCUSDT".to_string(),
            order_id: Some(123_456),
            orig_client_order_id: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"symbol\":\"BTCUSDT\""));
        assert!(json.contains("\"order_id\":123456"));

        let deserialized: CancelOrderParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, params.symbol);
        assert_eq!(deserialized.order_id, params.order_id);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_asterdex_order_result_serialization() {
        let result = AsterdexOrderResult {
            order_id: 123_456,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            status: "FILLED".to_string(),
            quantity: "0.1".to_string(),
            price: None,
            message: "Success".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"order_id\":123456"));
        assert!(json.contains("\"symbol\":\"BTCUSDT\""));
        assert!(json.contains("\"status\":\"FILLED\""));

        let deserialized: AsterdexOrderResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.order_id, result.order_id);
        assert_eq!(deserialized.status, result.status);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_asterdex_cancel_result_serialization() {
        let result = AsterdexCancelResult {
            order_id: 654_321,
            symbol: "BTCUSDT".to_string(),
            status: "CANCELED".to_string(),
            message: "Success".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"order_id\":654321"));
        assert!(json.contains("\"status\":\"CANCELED\""));

        let deserialized: AsterdexCancelResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.order_id, result.order_id);
        assert_eq!(deserialized.status, result.status);
    }

    #[test]
    fn test_order_params_with_all_fields() {
        let params = OrderParams {
            symbol: "SOLUSDT".to_string(),
            side: "SELL".to_string(),
            quantity: "10.5".to_string(),
            order_type: "LIMIT".to_string(),
            price: Some("100.25".to_string()),
            position_side: Some("SHORT".to_string()),
            reduce_only: Some(true),
            time_in_force: Some("IOC".to_string()),
        };

        assert_eq!(params.symbol, "SOLUSDT");
        assert_eq!(params.side, "SELL");
        assert_eq!(params.quantity, "10.5");
        assert_eq!(params.order_type, "LIMIT");
        assert_eq!(params.price, Some("100.25".to_string()));
        assert_eq!(params.position_side, Some("SHORT".to_string()));
        assert_eq!(params.reduce_only, Some(true));
        assert_eq!(params.time_in_force, Some("IOC".to_string()));
    }

    #[test]
    fn test_various_time_in_force_values() {
        let tif_values = vec!["GTC", "IOC", "FOK", "GTX"];

        for tif in tif_values {
            let params = OrderParams {
                symbol: "BTCUSDT".to_string(),
                side: "BUY".to_string(),
                quantity: "0.1".to_string(),
                order_type: "LIMIT".to_string(),
                price: Some("50000".to_string()),
                position_side: None,
                reduce_only: None,
                time_in_force: Some(tif.to_string()),
            };

            assert_eq!(params.time_in_force, Some(tif.to_string()));
        }
    }

    #[test]
    fn test_various_position_sides() {
        let sides = vec!["LONG", "SHORT", "BOTH"];

        for side in sides {
            let params = OrderParams {
                symbol: "BTCUSDT".to_string(),
                side: "BUY".to_string(),
                quantity: "0.1".to_string(),
                order_type: "MARKET".to_string(),
                price: None,
                position_side: Some(side.to_string()),
                reduce_only: None,
                time_in_force: None,
            };

            assert_eq!(params.position_side, Some(side.to_string()));
        }
    }

    #[test]
    fn test_order_result_with_price() {
        let result = AsterdexOrderResult {
            order_id: 789_123,
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "LIMIT".to_string(),
            status: "NEW".to_string(),
            quantity: "2.0".to_string(),
            price: Some("2500.00".to_string()),
            message: "Limit order placed".to_string(),
        };

        assert_eq!(result.order_id, 789_123);
        assert_eq!(result.price, Some("2500.00".to_string()));
        assert_eq!(result.order_type, "LIMIT");
        assert_eq!(result.status, "NEW");
    }
}
