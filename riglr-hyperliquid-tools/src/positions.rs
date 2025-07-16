//! Position management tools for Hyperliquid
//!
//! This module provides tools for managing and monitoring trading positions.

use crate::client::{HyperliquidClient, Position};
use riglr_core::{ToolError, SignerContext};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Get current positions on Hyperliquid
///
/// This tool retrieves all active perpetual futures positions for the user's account.
/// Returns comprehensive position data including unrealized PnL, leverage, liquidation prices,
/// and margin requirements. Only returns positions with non-zero size.
/// 
/// # Returns
/// 
/// Returns `Vec<HyperliquidPosition>` where each position contains:
/// - `symbol`: Trading pair (e.g., "ETH-PERP")
/// - `size`: Position size in contracts (positive for long, negative for short)
/// - `entry_price`: Average entry price of the position
/// - `mark_price`: Current market price fetched from Hyperliquid API
/// - `unrealized_pnl`: Unrealized profit/loss in USD
/// - `position_value`: Total position value in USD
/// - `leverage`: Current leverage multiplier
/// - `liquidation_price`: Price at which position gets liquidated
/// - `margin_used`: Amount of margin allocated to this position
/// - `return_on_equity`: ROE percentage
/// - `position_type`: Position mode (e.g., "oneWay")
/// 
/// # Errors
/// 
/// * `HyperliquidToolError::NetworkError` - When API connection fails
/// * `HyperliquidToolError::Generic` - When signer context unavailable or address parsing fails
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_hyperliquid_tools::positions::get_hyperliquid_positions;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let positions = get_hyperliquid_positions().await?;
/// 
/// for position in positions {
///     println!("Position: {} {} contracts", position.symbol, position.size);
///     println!("Entry: ${}, PnL: ${}", position.entry_price, position.unrealized_pnl);
///     println!("Leverage: {}x, Margin: ${}", position.leverage, position.margin_used);
/// }
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_hyperliquid_positions() -> Result<Vec<HyperliquidPosition>, ToolError> {
    debug!("Getting Hyperliquid positions");

    // Get signer context
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    // Create client
    let client = HyperliquidClient::new(signer)?;

    // Get user address
    let user_address = client.get_user_address()?;

    // Get positions
    let positions = client.get_positions(&user_address).await?;

    // Get current market prices for all assets
    let all_mids = client.get_all_mids().await?;

    // Convert to our result format
    let mut result_positions = Vec::new();
    for position in positions {
        let Position { position: pos_data, type_field } = position;
        // Only include positions with non-zero size
        let size = pos_data.szi.parse::<f64>().unwrap_or(0.0);
        if size != 0.0 {
            // Extract mark price from the all_mids response
            // The response format is typically an object with coin symbols as keys and prices as values
            let mark_price = if let Some(mids_obj) = all_mids.as_object() {
                // Try to find the price for this coin
                let coin_key = pos_data.coin.trim_end_matches("-PERP");
                mids_obj.get(coin_key)
                    .or_else(|| mids_obj.get(&pos_data.coin))
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.0")
                    .to_string()
            } else {
                "0.0".to_string()
            };
            
            result_positions.push(HyperliquidPosition {
                symbol: pos_data.coin,
                size: pos_data.szi,
                entry_price: pos_data.entry_px.unwrap_or_default(),
                mark_price,
                unrealized_pnl: pos_data.unrealized_pnl,
                position_value: pos_data.position_value,
                leverage: pos_data.leverage.value,
                liquidation_price: pos_data.liquidation_px.unwrap_or_default(),
                margin_used: pos_data.margin_used,
                return_on_equity: pos_data.return_on_equity,
                position_type: type_field,
            });
        }
    }

    info!("Retrieved {} positions", result_positions.len());
    Ok(result_positions)
}

/// Close a position on Hyperliquid
///
/// This tool automatically closes an existing perpetual futures position by placing a market order
/// in the opposite direction. It can close the entire position or a partial amount, with automatic
/// side detection (sells long positions, buys short positions).
/// 
/// # Arguments
/// 
/// * `symbol` - Trading pair symbol of the position to close
/// * `size` - Optional specific amount to close (closes entire position if None)
/// 
/// # Returns
/// 
/// Returns `HyperliquidCloseResult` containing:
/// - `symbol`: The trading pair that was closed
/// - `closed_size`: Amount of the position that was closed
/// - `order_side`: Direction of the closing order ("buy" or "sell")
/// - `order_id`: Order ID for the closing transaction
/// - `status`: Order status from the API
/// - `message`: Human-readable confirmation message
/// 
/// # Errors
/// 
/// * `HyperliquidToolError::InvalidInput` - When no position exists or size is invalid
/// * `HyperliquidToolError::ApiError` - When the closing order fails
/// * `HyperliquidToolError::NetworkError` - When connection issues occur
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_hyperliquid_tools::positions::close_hyperliquid_position;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Close entire position
/// let result = close_hyperliquid_position(
///     "ETH-PERP".to_string(),
///     None, // Close entire position
/// ).await?;
/// 
/// println!("Closed {} {} position", result.closed_size, result.symbol);
/// println!("Order ID: {:?}", result.order_id);
/// 
/// // Close partial position
/// let partial = close_hyperliquid_position(
///     "BTC-PERP".to_string(),
///     Some("0.05".to_string()), // Close 0.05 BTC worth
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn close_hyperliquid_position(
    symbol: String,
    size: Option<String>,
) -> Result<HyperliquidCloseResult, ToolError> {
    debug!("Closing Hyperliquid position for {}", symbol);

    // Get current positions to determine position direction and size
    let positions = get_hyperliquid_positions().await?;
    
    let position = positions.iter()
        .find(|p| p.symbol.eq_ignore_ascii_case(&symbol) || 
                  p.symbol.eq_ignore_ascii_case(&format!("{}-PERP", symbol)))
        .ok_or_else(|| ToolError::permanent(format!("No open position found for {}", symbol)))?;

    let current_size = position.size.parse::<f64>()
        .map_err(|e| ToolError::permanent(format!("Invalid position size: {}", e)))?;

    if current_size == 0.0 {
        return Err(ToolError::permanent(format!("No open position for {}", symbol)));
    }

    // Determine close size
    let close_size = if let Some(size_str) = size {
        let requested_size = size_str.parse::<f64>()
            .map_err(|e| ToolError::permanent(format!("Invalid size '{}': {}", size_str, e)))?;
        
        if requested_size > current_size.abs() {
            return Err(ToolError::permanent(format!(
                "Requested close size {} exceeds position size {}", 
                requested_size, current_size.abs()
            )));
        }
        requested_size
    } else {
        current_size.abs()
    };

    // Determine order side (opposite of position)
    let order_side = if current_size > 0.0 { "sell" } else { "buy" };

    // Use the trading module to place a market order
    let order_result = crate::trading::place_hyperliquid_order(
        position.symbol.clone(),
        order_side.to_string(),
        close_size.to_string(),
        "market".to_string(),
        None, // No price for market order
        Some(true), // Reduce only
        None, // No time in force needed for market orders
    ).await?;

    info!("Successfully placed close order for {} position", symbol);

    Ok(HyperliquidCloseResult {
        symbol: position.symbol.clone(),
        closed_size: close_size.to_string(),
        order_side: order_side.to_string(),
        order_id: order_result.order_id,
        status: order_result.status,
        message: format!("Position close order placed for {} {}", close_size, symbol),
    })
}

/// Get position details for a specific symbol
///
/// This tool retrieves detailed information about a position for a specific trading pair
/// on Hyperliquid. It performs flexible symbol matching, supporting both base symbols
/// (e.g., "ETH") and full perpetual contract names (e.g., "ETH-PERP").
/// 
/// # Arguments
/// 
/// * `symbol` - Trading pair symbol to query (e.g., "ETH", "BTC", "ETH-PERP")
/// 
/// # Returns
/// 
/// Returns `Option<HyperliquidPosition>`:
/// - `Some(position)` - If an active position exists for the symbol
/// - `None` - If no position is found for the symbol
/// 
/// When a position is found, it contains:
/// - `symbol`: Full trading pair name (e.g., "ETH-PERP")
/// - `size`: Position size (positive for long, negative for short)
/// - `entry_price`: Average entry price
/// - `unrealized_pnl`: Current profit/loss in USD
/// - `leverage`: Current leverage multiplier
/// - `liquidation_price`: Price level where position gets liquidated
/// - Additional margin and return metrics
/// 
/// # Errors
/// 
/// * `HyperliquidToolError::NetworkError` - When API connection fails
/// * `HyperliquidToolError::Generic` - When signer context unavailable
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_hyperliquid_tools::positions::get_hyperliquid_position_details;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Check if we have an ETH position
/// let position = get_hyperliquid_position_details("ETH".to_string()).await?;
/// 
/// match position {
///     Some(pos) => {
///         println!("ETH Position Found:");
///         println!("  Size: {} contracts", pos.size);
///         println!("  Entry Price: ${}", pos.entry_price);
///         println!("  PnL: ${}", pos.unrealized_pnl);
///         println!("  Leverage: {}x", pos.leverage);
///         println!("  Liquidation: ${}", pos.liquidation_price);
///     }
///     None => {
///         println!("No ETH position found");
///     }
/// }
/// 
/// // Also works with full symbol names
/// let btc_position = get_hyperliquid_position_details("BTC-PERP".to_string()).await?;
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_hyperliquid_position_details(
    symbol: String,
) -> Result<Option<HyperliquidPosition>, ToolError> {
    debug!("Getting position details for {}", symbol);

    let positions = get_hyperliquid_positions().await?;
    
    let position = positions.into_iter()
        .find(|p| p.symbol.eq_ignore_ascii_case(&symbol) || 
                  p.symbol.eq_ignore_ascii_case(&format!("{}-PERP", symbol)));

    match position {
        Some(pos) => {
            info!("Found position for {}: size = {}", symbol, pos.size);
            Ok(Some(pos))
        },
        None => {
            info!("No position found for {}", symbol);
            Ok(None)
        }
    }
}

/// Calculate position risk metrics
///
/// This tool performs comprehensive risk analysis of the entire Hyperliquid portfolio,
/// calculating exposure metrics, margin utilization, and identifying positions at risk.
/// Essential for risk management and portfolio monitoring in leveraged trading.
/// 
/// # Returns
/// 
/// Returns `HyperliquidRiskMetrics` containing:
/// - `total_positions`: Number of active positions
/// - `total_position_value`: Combined value of all positions in USD
/// - `total_unrealized_pnl`: Sum of unrealized P&L across all positions
/// - `margin_utilization_percent`: Percentage of available margin being used
/// - `max_leverage`: Highest leverage across all positions
/// - `positions_at_risk`: Number of positions with high leverage or negative P&L
/// - `risk_level`: Overall risk assessment ("LOW", "MEDIUM", "HIGH")
/// 
/// # Risk Level Calculation
/// 
/// Risk level is determined by:
/// - **HIGH**: >80% margin utilization, >50x max leverage, or >75% positions at risk
/// - **MEDIUM**: >50% margin utilization, >20x max leverage, or >50% positions at risk
/// - **LOW**: All other scenarios
/// 
/// # Errors
/// 
/// * `HyperliquidToolError::NetworkError` - When API connection fails
/// * `HyperliquidToolError::Generic` - When signer context unavailable
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_hyperliquid_tools::positions::get_hyperliquid_portfolio_risk;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let risk = get_hyperliquid_portfolio_risk().await?;
/// 
/// println!("Portfolio Risk Analysis:");
/// println!("  Risk Level: {}", risk.risk_level);
/// println!("  Total Positions: {}", risk.total_positions);
/// println!("  Total Exposure: ${}", risk.total_position_value);
/// println!("  Unrealized P&L: ${}", risk.total_unrealized_pnl);
/// println!("  Margin Utilization: {:.1}%", risk.margin_utilization_percent);
/// println!("  Max Leverage: {}x", risk.max_leverage);
/// println!("  Positions at Risk: {}", risk.positions_at_risk);
/// 
/// // Risk management alerts
/// match risk.risk_level.as_str() {
///     "HIGH" => println!("⚠️  HIGH RISK: Consider reducing positions"),
///     "MEDIUM" => println!("🔶 MEDIUM RISK: Monitor closely"),
///     "LOW" => println!("✅ LOW RISK: Portfolio within safe parameters"),
///     _ => {}
/// }
/// 
/// // Margin utilization warning
/// if risk.margin_utilization_percent > 70.0 {
///     println!("💡 Consider adding more margin or reducing position sizes");
/// }
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_hyperliquid_portfolio_risk() -> Result<HyperliquidRiskMetrics, ToolError> {
    debug!("Calculating portfolio risk metrics");

    // Get all positions
    let positions = get_hyperliquid_positions().await?;
    
    // Get account info for margin data
    let account_info = crate::trading::get_hyperliquid_account_info().await?;

    let mut total_position_value = 0.0;
    let mut total_unrealized_pnl = 0.0;
    let mut max_leverage = 0u32;
    let mut positions_at_risk = 0;

    for position in &positions {
        // Parse numeric values
        if let Ok(position_value) = position.position_value.parse::<f64>() {
            total_position_value += position_value.abs();
        }
        
        if let Ok(pnl) = position.unrealized_pnl.parse::<f64>() {
            total_unrealized_pnl += pnl;
        }

        if position.leverage > max_leverage {
            max_leverage = position.leverage;
        }

        // Count positions with high leverage or negative PnL as "at risk"
        if position.leverage > 10 || position.unrealized_pnl.parse::<f64>().unwrap_or(0.0) < 0.0 {
            positions_at_risk += 1;
        }
    }

    let margin_utilization = if let Ok(withdrawable) = account_info.withdrawable_balance.parse::<f64>() {
        if let Ok(margin_used) = account_info.cross_margin_used.parse::<f64>() {
            if withdrawable + margin_used > 0.0 {
                (margin_used / (withdrawable + margin_used)) * 100.0
            } else {
                0.0
            }
        } else { 0.0 }
    } else { 0.0 };

    info!("Calculated risk metrics: {} positions, {}% margin utilization", positions.len(), margin_utilization);

    Ok(HyperliquidRiskMetrics {
        total_positions: positions.len(),
        total_position_value: total_position_value.to_string(),
        total_unrealized_pnl: total_unrealized_pnl.to_string(),
        margin_utilization_percent: margin_utilization,
        max_leverage,
        positions_at_risk,
        risk_level: calculate_risk_level(margin_utilization, max_leverage, positions_at_risk, positions.len()),
    })
}

/// Helper function to calculate overall risk level
fn calculate_risk_level(margin_util: f64, max_lev: u32, at_risk: usize, total: usize) -> String {
    let at_risk_ratio = if total > 0 { (at_risk as f64 / total as f64) * 100.0 } else { 0.0 };
    
    if margin_util > 80.0 || max_lev > 50 || at_risk_ratio > 75.0 {
        "HIGH".to_string()
    } else if margin_util > 50.0 || max_lev > 20 || at_risk_ratio > 50.0 {
        "MEDIUM".to_string()
    } else {
        "LOW".to_string()
    }
}

// Result structures

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperliquidPosition {
    pub symbol: String,
    pub size: String,
    pub entry_price: String,
    pub mark_price: String,
    pub unrealized_pnl: String,
    pub position_value: String,
    pub leverage: u32,
    pub liquidation_price: String,
    pub margin_used: String,
    pub return_on_equity: String,
    pub position_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperliquidCloseResult {
    pub symbol: String,
    pub closed_size: String,
    pub order_side: String,
    pub order_id: Option<String>,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperliquidRiskMetrics {
    pub total_positions: usize,
    pub total_position_value: String,
    pub total_unrealized_pnl: String,
    pub margin_utilization_percent: f64,
    pub max_leverage: u32,
    pub positions_at_risk: usize,
    pub risk_level: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_structure() {
        let position = HyperliquidPosition {
            symbol: "ETH-PERP".to_string(),
            size: "1.5".to_string(),
            entry_price: "2000.0".to_string(),
            mark_price: "2100.0".to_string(),
            unrealized_pnl: "150.0".to_string(),
            position_value: "3150.0".to_string(),
            leverage: 10,
            liquidation_price: "1800.0".to_string(),
            margin_used: "315.0".to_string(),
            return_on_equity: "47.6".to_string(),
            position_type: "oneWay".to_string(),
        };

        assert_eq!(position.symbol, "ETH-PERP");
        assert_eq!(position.leverage, 10);
    }

    #[test]
    fn test_risk_level_calculation() {
        assert_eq!(calculate_risk_level(90.0, 100, 8, 10), "HIGH");
        assert_eq!(calculate_risk_level(60.0, 25, 5, 10), "MEDIUM");
        assert_eq!(calculate_risk_level(30.0, 5, 1, 10), "LOW");
    }
}