//! Position and account management tools for Asterdex Futures v3
//!
//! This module provides tools for querying account information, positions, and balances on Asterdex.

use crate::client::{AccountBalance, AccountInfo, Client, Order, Position};
use core::str::FromStr;
use riglr_core::provider::ApplicationContext;
use riglr_core::{SignerContext, ToolError};
use riglr_macros::tool;
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::{debug, info};

/// Asterdex position information with simplified fields
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AsterdexPosition {
    /// Trading symbol
    pub symbol: String,
    /// Position amount (positive for long, negative for short)
    pub position_amt: String,
    /// Average entry price
    pub entry_price: String,
    /// Current mark price
    pub mark_price: String,
    /// Unrealized profit/loss
    pub unrealized_profit: String,
    /// Liquidation price
    pub liquidation_price: String,
    /// Current leverage
    pub leverage: String,
    /// Position side (LONG/SHORT/BOTH)
    pub position_side: String,
    /// Notional value of position
    pub notional: String,
    /// Margin type (cross/isolated)
    pub margin_type: String,
}

/// Asterdex account information with simplified fields
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AsterdexAccountInfo {
    /// Whether the account can trade
    pub can_trade: bool,
    /// Total wallet balance
    pub total_wallet_balance: String,
    /// Total unrealized profit/loss
    pub total_unrealized_profit: String,
    /// Total margin balance
    pub total_margin_balance: String,
    /// Available balance for trading
    pub available_balance: String,
    /// Number of open positions
    pub open_positions_count: usize,
    /// Last update timestamp
    pub update_time: i64,
}

/// Asterdex balance information
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AsterdexBalance {
    /// Asset symbol (e.g., "USDT")
    pub asset: String,
    /// Wallet balance
    pub balance: String,
    /// Available balance
    pub available_balance: String,
    /// Unrealized profit/loss
    pub cross_un_pnl: String,
}

/// Get current positions on Asterdex
///
/// This tool retrieves all open positions or a specific position for the account.
///
/// # Arguments
///
/// * `symbol` - Optional trading pair symbol to filter positions (e.g., "BTCUSDT")
///
/// # Returns
///
/// Returns a list of `AsterdexPosition` containing:
/// - Position details (symbol, amount, side)
/// - Pricing information (entry price, mark price)
/// - P&L information (unrealized profit)
/// - Risk metrics (leverage, liquidation price)
///
/// # Errors
///
/// * `ApiError` - When Asterdex API request fails
/// * `NetworkError` - When connection issues occur
/// * `AuthError` - When authentication fails
#[tool]
#[allow(clippy::cognitive_complexity)]
pub async fn get_open_holdings(
    _context: &ApplicationContext,
    symbol: Option<String>,
) -> Result<Vec<AsterdexPosition>, ToolError> {
    debug!("Getting Asterdex positions");

    // Get signer context
    let signer = SignerContext::current()
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {e}")))?;

    // Create client
    let client = Client::new(signer).map_err(ToolError::from)?;

    // Build query parameters
    let mut query_params = BTreeMap::new();
    if let Some(sym) = symbol {
        query_params.insert("symbol".to_string(), sym);
        info!("Querying positions for specific symbol");
    } else {
        info!("Querying all open positions");
    }

    // Get positions
    let response = client
        .get("/fapi/v3/positionRisk", query_params)
        .await
        .map_err(ToolError::from)?;

    // Parse response
    let positions: Vec<Position> = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse positions response: {e}"))
    })?;

    // Convert to simplified format
    let asterdex_positions: Vec<AsterdexPosition> = positions
        .into_iter()
        .filter(|p| {
            // Filter out positions with zero amount
            Decimal::from_str(&p.position_amt).unwrap_or(Decimal::ZERO) != Decimal::ZERO
        })
        .map(|p| AsterdexPosition {
            symbol: p.symbol,
            position_amt: p.position_amt,
            entry_price: p.entry_price,
            mark_price: p.mark_price,
            unrealized_profit: p.un_realized_profit,
            liquidation_price: p.liquidation_price,
            leverage: p.leverage,
            position_side: p.position_side,
            notional: p.notional,
            margin_type: p.margin_type,
        })
        .collect();

    info!("Found {} open positions", asterdex_positions.len());

    Ok(asterdex_positions)
}

/// Get account information on Asterdex
///
/// This tool retrieves comprehensive account information including balances,
/// margin requirements, and trading permissions.
///
/// # Returns
///
/// Returns `AsterdexAccountInfo` containing:
/// - Trading permissions (`can_trade`)
/// - Balance information (wallet balance, margin balance, available balance)
/// - P&L information (unrealized profit)
/// - Position count
///
/// # Errors
///
/// * `ApiError` - When Asterdex API request fails
/// * `NetworkError` - When connection issues occur
/// * `AuthError` - When authentication fails
#[tool]
pub async fn get_asterdex_account_info(
    _context: &ApplicationContext,
) -> Result<AsterdexAccountInfo, ToolError> {
    debug!("Getting Asterdex account information");

    // Get signer context
    let signer = SignerContext::current()
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {e}")))?;

    // Create client
    let client = Client::new(signer).map_err(ToolError::from)?;

    info!("Querying account information");

    // Get account info
    let response = client
        .get("/fapi/v3/account", BTreeMap::new())
        .await
        .map_err(ToolError::from)?;

    // Parse response
    let account_info: AccountInfo = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse account info response: {e}"))
    })?;

    // Convert to simplified format
    let asterdex_account = AsterdexAccountInfo {
        can_trade: account_info.can_trade,
        total_wallet_balance: account_info.total_wallet_balance,
        total_unrealized_profit: account_info.total_unrealized_profit,
        total_margin_balance: account_info.total_margin_balance,
        available_balance: account_info.available_balance,
        open_positions_count: account_info
            .positions
            .iter()
            .filter(|p| {
                Decimal::from_str(&p.position_amt).unwrap_or(Decimal::ZERO) != Decimal::ZERO
            })
            .count(),
        update_time: account_info.update_time,
    };

    info!(
        "Account info: can_trade={}, balance={}, available={}, positions={}",
        asterdex_account.can_trade,
        asterdex_account.total_wallet_balance,
        asterdex_account.available_balance,
        asterdex_account.open_positions_count
    );

    Ok(asterdex_account)
}

/// Get account balances on Asterdex
///
/// This tool retrieves all asset balances for the account.
///
/// # Returns
///
/// Returns a list of `AsterdexBalance` containing:
/// - Asset information (symbol, balance)
/// - Available balance for trading
/// - Unrealized P&L
///
/// # Errors
///
/// * `ApiError` - When Asterdex API request fails
/// * `NetworkError` - When connection issues occur
/// * `AuthError` - When authentication fails
#[tool]
pub async fn get_asterdex_balance(
    _context: &ApplicationContext,
) -> Result<Vec<AsterdexBalance>, ToolError> {
    debug!("Getting Asterdex account balances");

    // Get signer context
    let signer = SignerContext::current()
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {e}")))?;

    // Create client
    let client = Client::new(signer).map_err(ToolError::from)?;

    info!("Querying account balances");

    // Get balances
    let response = client
        .get("/fapi/v3/balance", BTreeMap::new())
        .await
        .map_err(ToolError::from)?;

    // Parse response
    let balances: Vec<AccountBalance> = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse balances response: {e}"))
    })?;

    // Convert to simplified format, filtering out zero balances
    let asterdex_balances: Vec<AsterdexBalance> = balances
        .into_iter()
        .filter(|b| {
            // Filter out assets with zero balance
            Decimal::from_str(&b.balance).unwrap_or(Decimal::ZERO) != Decimal::ZERO
        })
        .map(|b| AsterdexBalance {
            asset: b.asset,
            balance: b.balance,
            available_balance: b.available_balance,
            cross_un_pnl: b.cross_un_pnl,
        })
        .collect();

    info!("Found {} assets with balance", asterdex_balances.len());

    Ok(asterdex_balances)
}

/// Get all open orders on Asterdex
///
/// This tool retrieves all currently open orders for the account.
///
/// # Arguments
///
/// * `symbol` - Optional trading pair symbol to filter orders (e.g., "BTCUSDT")
///
/// # Returns
///
/// Returns a list of open orders with complete order information
///
/// # Errors
///
/// * `ApiError` - When Asterdex API request fails
/// * `NetworkError` - When connection issues occur
/// * `AuthError` - When authentication fails
#[tool]
#[allow(clippy::cognitive_complexity)]
pub async fn get_asterdex_open_orders(
    _context: &ApplicationContext,
    symbol: Option<String>,
) -> Result<Vec<Order>, ToolError> {
    debug!("Getting Asterdex open orders");

    // Get signer context
    let signer = SignerContext::current()
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {e}")))?;

    // Create client
    let client = Client::new(signer).map_err(ToolError::from)?;

    // Build query parameters
    let mut query_params = BTreeMap::new();
    if let Some(sym) = symbol {
        query_params.insert("symbol".to_string(), sym);
        info!("Querying open orders for specific symbol");
    } else {
        info!("Querying all open orders");
    }

    // Get open orders
    let response = client
        .get("/fapi/v3/openOrders", query_params)
        .await
        .map_err(ToolError::from)?;

    // Parse response
    let orders: Vec<Order> = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse open orders response: {e}"))
    })?;

    info!("Found {} open orders", orders.len());

    Ok(orders)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asterdex_position_structure() {
        let position = AsterdexPosition {
            symbol: "BTCUSDT".to_string(),
            position_amt: "0.5".to_string(),
            entry_price: "50000.00".to_string(),
            mark_price: "51000.00".to_string(),
            unrealized_profit: "500.00".to_string(),
            liquidation_price: "45000.00".to_string(),
            leverage: "10".to_string(),
            position_side: "LONG".to_string(),
            notional: "25500.00".to_string(),
            margin_type: "cross".to_string(),
        };

        assert_eq!(position.symbol, "BTCUSDT");
        assert_eq!(position.position_amt, "0.5");
        assert_eq!(position.entry_price, "50000.00");
        assert_eq!(position.mark_price, "51000.00");
        assert_eq!(position.unrealized_profit, "500.00");
        assert_eq!(position.liquidation_price, "45000.00");
        assert_eq!(position.leverage, "10");
        assert_eq!(position.position_side, "LONG");
        assert_eq!(position.notional, "25500.00");
        assert_eq!(position.margin_type, "cross");
    }

    #[test]
    fn test_asterdex_position_short() {
        let position = AsterdexPosition {
            symbol: "ETHUSDT".to_string(),
            position_amt: "-1.0".to_string(),
            entry_price: "2500.00".to_string(),
            mark_price: "2450.00".to_string(),
            unrealized_profit: "50.00".to_string(),
            liquidation_price: "2750.00".to_string(),
            leverage: "5".to_string(),
            position_side: "SHORT".to_string(),
            notional: "-2450.00".to_string(),
            margin_type: "isolated".to_string(),
        };

        assert_eq!(position.symbol, "ETHUSDT");
        assert_eq!(position.position_amt, "-1.0");
        assert_eq!(position.position_side, "SHORT");
        assert_eq!(position.margin_type, "isolated");
    }

    #[test]
    fn test_asterdex_account_info_structure() {
        let account = AsterdexAccountInfo {
            can_trade: true,
            total_wallet_balance: "10000.00".to_string(),
            total_unrealized_profit: "250.00".to_string(),
            total_margin_balance: "10250.00".to_string(),
            available_balance: "8000.00".to_string(),
            open_positions_count: 3,
            update_time: 1_234_567_890_000,
        };

        assert!(account.can_trade);
        assert_eq!(account.total_wallet_balance, "10000.00");
        assert_eq!(account.total_unrealized_profit, "250.00");
        assert_eq!(account.total_margin_balance, "10250.00");
        assert_eq!(account.available_balance, "8000.00");
        assert_eq!(account.open_positions_count, 3);
        assert_eq!(account.update_time, 1_234_567_890_000);
    }

    #[test]
    fn test_asterdex_account_info_cannot_trade() {
        let account = AsterdexAccountInfo {
            can_trade: false,
            total_wallet_balance: "100.00".to_string(),
            total_unrealized_profit: "-50.00".to_string(),
            total_margin_balance: "50.00".to_string(),
            available_balance: "10.00".to_string(),
            open_positions_count: 1,
            update_time: 1_234_567_890_000,
        };

        assert!(!account.can_trade);
        assert_eq!(account.total_unrealized_profit, "-50.00");
        assert_eq!(account.available_balance, "10.00");
    }

    #[test]
    fn test_asterdex_balance_structure() {
        let balance = AsterdexBalance {
            asset: "USDT".to_string(),
            balance: "10000.00".to_string(),
            available_balance: "8000.00".to_string(),
            cross_un_pnl: "250.00".to_string(),
        };

        assert_eq!(balance.asset, "USDT");
        assert_eq!(balance.balance, "10000.00");
        assert_eq!(balance.available_balance, "8000.00");
        assert_eq!(balance.cross_un_pnl, "250.00");
    }

    #[test]
    fn test_asterdex_balance_with_negative_pnl() {
        let balance = AsterdexBalance {
            asset: "BTC".to_string(),
            balance: "1.5".to_string(),
            available_balance: "1.2".to_string(),
            cross_un_pnl: "-0.1".to_string(),
        };

        assert_eq!(balance.asset, "BTC");
        assert_eq!(balance.balance, "1.5");
        assert_eq!(balance.available_balance, "1.2");
        assert_eq!(balance.cross_un_pnl, "-0.1");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_position_serialization() {
        let position = AsterdexPosition {
            symbol: "BTCUSDT".to_string(),
            position_amt: "0.5".to_string(),
            entry_price: "50000.00".to_string(),
            mark_price: "51000.00".to_string(),
            unrealized_profit: "500.00".to_string(),
            liquidation_price: "45000.00".to_string(),
            leverage: "10".to_string(),
            position_side: "LONG".to_string(),
            notional: "25500.00".to_string(),
            margin_type: "cross".to_string(),
        };

        let json = serde_json::to_string(&position).unwrap();
        assert!(json.contains("\"symbol\":\"BTCUSDT\""));
        assert!(json.contains("\"position_amt\":\"0.5\""));
        assert!(json.contains("\"entry_price\":\"50000.00\""));

        let deserialized: AsterdexPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, position.symbol);
        assert_eq!(deserialized.position_amt, position.position_amt);
        assert_eq!(deserialized.entry_price, position.entry_price);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_account_info_serialization() {
        let account = AsterdexAccountInfo {
            can_trade: true,
            total_wallet_balance: "10000.00".to_string(),
            total_unrealized_profit: "250.00".to_string(),
            total_margin_balance: "10250.00".to_string(),
            available_balance: "8000.00".to_string(),
            open_positions_count: 3,
            update_time: 1_234_567_890_000,
        };

        let json = serde_json::to_string(&account).unwrap();
        assert!(json.contains("\"can_trade\":true"));
        assert!(json.contains("\"total_wallet_balance\":\"10000.00\""));
        assert!(json.contains("\"open_positions_count\":3"));

        let deserialized: AsterdexAccountInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.can_trade, account.can_trade);
        assert_eq!(
            deserialized.total_wallet_balance,
            account.total_wallet_balance
        );
        assert_eq!(
            deserialized.open_positions_count,
            account.open_positions_count
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_balance_serialization() {
        let balance = AsterdexBalance {
            asset: "USDT".to_string(),
            balance: "10000.00".to_string(),
            available_balance: "8000.00".to_string(),
            cross_un_pnl: "250.00".to_string(),
        };

        let json = serde_json::to_string(&balance).unwrap();
        assert!(json.contains("\"asset\":\"USDT\""));
        assert!(json.contains("\"balance\":\"10000.00\""));

        let deserialized: AsterdexBalance = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.asset, balance.asset);
        assert_eq!(deserialized.balance, balance.balance);
    }

    #[test]
    fn test_position_debug_trait() {
        let position = AsterdexPosition {
            symbol: "BTCUSDT".to_string(),
            position_amt: "0.5".to_string(),
            entry_price: "50000.00".to_string(),
            mark_price: "51000.00".to_string(),
            unrealized_profit: "500.00".to_string(),
            liquidation_price: "45000.00".to_string(),
            leverage: "10".to_string(),
            position_side: "LONG".to_string(),
            notional: "25500.00".to_string(),
            margin_type: "cross".to_string(),
        };

        let debug_str = format!("{position:?}");
        assert!(debug_str.contains("AsterdexPosition"));
        assert!(debug_str.contains("BTCUSDT"));
    }

    #[test]
    fn test_account_info_debug_trait() {
        let account = AsterdexAccountInfo {
            can_trade: true,
            total_wallet_balance: "10000.00".to_string(),
            total_unrealized_profit: "250.00".to_string(),
            total_margin_balance: "10250.00".to_string(),
            available_balance: "8000.00".to_string(),
            open_positions_count: 3,
            update_time: 1_234_567_890_000,
        };

        let debug_str = format!("{account:?}");
        assert!(debug_str.contains("AsterdexAccountInfo"));
        assert!(debug_str.contains("can_trade"));
    }

    #[test]
    fn test_balance_debug_trait() {
        let balance = AsterdexBalance {
            asset: "USDT".to_string(),
            balance: "10000.00".to_string(),
            available_balance: "8000.00".to_string(),
            cross_un_pnl: "250.00".to_string(),
        };

        let debug_str = format!("{balance:?}");
        assert!(debug_str.contains("AsterdexBalance"));
        assert!(debug_str.contains("USDT"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_multiple_positions() {
        let positions = vec![
            AsterdexPosition {
                symbol: "BTCUSDT".to_string(),
                position_amt: "0.5".to_string(),
                entry_price: "50000.00".to_string(),
                mark_price: "51000.00".to_string(),
                unrealized_profit: "500.00".to_string(),
                liquidation_price: "45000.00".to_string(),
                leverage: "10".to_string(),
                position_side: "LONG".to_string(),
                notional: "25500.00".to_string(),
                margin_type: "cross".to_string(),
            },
            AsterdexPosition {
                symbol: "ETHUSDT".to_string(),
                position_amt: "-1.0".to_string(),
                entry_price: "2500.00".to_string(),
                mark_price: "2450.00".to_string(),
                unrealized_profit: "50.00".to_string(),
                liquidation_price: "2750.00".to_string(),
                leverage: "5".to_string(),
                position_side: "SHORT".to_string(),
                notional: "-2450.00".to_string(),
                margin_type: "isolated".to_string(),
            },
        ];

        assert_eq!(positions.len(), 2);
        assert_eq!(positions.first().unwrap().symbol, "BTCUSDT");
        assert_eq!(positions.get(1).unwrap().symbol, "ETHUSDT");
        assert_eq!(positions.first().unwrap().position_side, "LONG");
        assert_eq!(positions.get(1).unwrap().position_side, "SHORT");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_multiple_balances() {
        let balances = vec![
            AsterdexBalance {
                asset: "USDT".to_string(),
                balance: "10000.00".to_string(),
                available_balance: "8000.00".to_string(),
                cross_un_pnl: "250.00".to_string(),
            },
            AsterdexBalance {
                asset: "BTC".to_string(),
                balance: "0.5".to_string(),
                available_balance: "0.4".to_string(),
                cross_un_pnl: "0.01".to_string(),
            },
            AsterdexBalance {
                asset: "ETH".to_string(),
                balance: "5.0".to_string(),
                available_balance: "4.5".to_string(),
                cross_un_pnl: "-0.1".to_string(),
            },
        ];

        assert_eq!(balances.len(), 3);
        assert_eq!(balances.first().unwrap().asset, "USDT");
        assert_eq!(balances.get(1).unwrap().asset, "BTC");
        assert_eq!(balances.get(2).unwrap().asset, "ETH");
    }

    #[test]
    fn test_position_with_zero_values() {
        let position = AsterdexPosition {
            symbol: "SOLUSDT".to_string(),
            position_amt: "0.0".to_string(),
            entry_price: "0.00".to_string(),
            mark_price: "100.00".to_string(),
            unrealized_profit: "0.00".to_string(),
            liquidation_price: "0.00".to_string(),
            leverage: "1".to_string(),
            position_side: "BOTH".to_string(),
            notional: "0.00".to_string(),
            margin_type: "cross".to_string(),
        };

        assert_eq!(position.position_amt, "0.0");
        assert_eq!(position.unrealized_profit, "0.00");
        assert_eq!(position.notional, "0.00");
    }

    #[test]
    fn test_account_info_with_no_positions() {
        let account = AsterdexAccountInfo {
            can_trade: true,
            total_wallet_balance: "1000.00".to_string(),
            total_unrealized_profit: "0.00".to_string(),
            total_margin_balance: "1000.00".to_string(),
            available_balance: "1000.00".to_string(),
            open_positions_count: 0,
            update_time: 1_234_567_890_000,
        };

        assert_eq!(account.open_positions_count, 0);
        assert_eq!(account.total_unrealized_profit, "0.00");
        assert_eq!(account.total_wallet_balance, account.total_margin_balance);
    }

    #[test]
    fn test_balance_with_zero_available() {
        let balance = AsterdexBalance {
            asset: "USDT".to_string(),
            balance: "1000.00".to_string(),
            available_balance: "0.00".to_string(),
            cross_un_pnl: "-100.00".to_string(),
        };

        assert_eq!(balance.available_balance, "0.00");
        assert_eq!(balance.cross_un_pnl, "-100.00");
    }

    #[test]
    fn test_position_margin_types() {
        let cross_position = AsterdexPosition {
            symbol: "BTCUSDT".to_string(),
            position_amt: "1.0".to_string(),
            entry_price: "50000.00".to_string(),
            mark_price: "50000.00".to_string(),
            unrealized_profit: "0.00".to_string(),
            liquidation_price: "45000.00".to_string(),
            leverage: "10".to_string(),
            position_side: "LONG".to_string(),
            notional: "50000.00".to_string(),
            margin_type: "cross".to_string(),
        };

        let isolated_position = AsterdexPosition {
            symbol: "ETHUSDT".to_string(),
            position_amt: "1.0".to_string(),
            entry_price: "2500.00".to_string(),
            mark_price: "2500.00".to_string(),
            unrealized_profit: "0.00".to_string(),
            liquidation_price: "2250.00".to_string(),
            leverage: "10".to_string(),
            position_side: "LONG".to_string(),
            notional: "2500.00".to_string(),
            margin_type: "isolated".to_string(),
        };

        assert_eq!(cross_position.margin_type, "cross");
        assert_eq!(isolated_position.margin_type, "isolated");
    }

    #[test]
    fn test_position_sides() {
        let long_position = AsterdexPosition {
            symbol: "BTCUSDT".to_string(),
            position_amt: "1.0".to_string(),
            entry_price: "50000.00".to_string(),
            mark_price: "50000.00".to_string(),
            unrealized_profit: "0.00".to_string(),
            liquidation_price: "45000.00".to_string(),
            leverage: "10".to_string(),
            position_side: "LONG".to_string(),
            notional: "50000.00".to_string(),
            margin_type: "cross".to_string(),
        };

        let short_position = AsterdexPosition {
            symbol: "BTCUSDT".to_string(),
            position_amt: "-1.0".to_string(),
            entry_price: "50000.00".to_string(),
            mark_price: "50000.00".to_string(),
            unrealized_profit: "0.00".to_string(),
            liquidation_price: "55000.00".to_string(),
            leverage: "10".to_string(),
            position_side: "SHORT".to_string(),
            notional: "-50000.00".to_string(),
            margin_type: "cross".to_string(),
        };

        let both_position = AsterdexPosition {
            symbol: "BTCUSDT".to_string(),
            position_amt: "0.0".to_string(),
            entry_price: "0.00".to_string(),
            mark_price: "50000.00".to_string(),
            unrealized_profit: "0.00".to_string(),
            liquidation_price: "0.00".to_string(),
            leverage: "10".to_string(),
            position_side: "BOTH".to_string(),
            notional: "0.00".to_string(),
            margin_type: "cross".to_string(),
        };

        assert_eq!(long_position.position_side, "LONG");
        assert_eq!(short_position.position_side, "SHORT");
        assert_eq!(both_position.position_side, "BOTH");
    }
}
