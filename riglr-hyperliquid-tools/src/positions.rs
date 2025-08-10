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
/// This tool retrieves all current positions for the user's account,
/// including position size, entry price, PnL, and leverage information.
#[tool]
pub async fn get_hyperliquid_positions() -> Result<Vec<HyperliquidPosition>, ToolError> {
    debug!("Getting Hyperliquid positions");

    // Get signer context
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    // Create client
    let client = HyperliquidClient::new(signer);

    // Get user address
    let user_address = client.get_user_address()?;

    // Get positions
    let positions = client.get_positions(&user_address).await?;

    // Convert to our result format
    let mut result_positions = Vec::new();
    for position in positions {
        let Position { position: pos_data, type_field } = position;
        // Only include positions with non-zero size
        let size = pos_data.szi.parse::<f64>().unwrap_or(0.0);
        if size != 0.0 {
            result_positions.push(HyperliquidPosition {
                symbol: pos_data.coin,
                size: pos_data.szi,
                entry_price: pos_data.entry_px.unwrap_or_default(),
                mark_price: "0.0".to_string(), // Would need separate API call to get current price
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
/// This tool places a market order to close an existing position.
/// It automatically determines the order direction based on the current position.
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
/// This tool retrieves detailed information about a position for a specific trading pair.
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
/// This tool calculates risk metrics for current positions including
/// total exposure, margin utilization, and risk-adjusted returns.
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