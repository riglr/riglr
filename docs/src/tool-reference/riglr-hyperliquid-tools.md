# riglr-hyperliquid-tools Tool Reference

This page contains documentation for tools provided by the `riglr-hyperliquid-tools` crate.

## Available Tools

- [`get_hyperliquid_positions`](#get_hyperliquid_positions) - src/positions.rs
- [`close_hyperliquid_position`](#close_hyperliquid_position) - src/positions.rs
- [`get_hyperliquid_position_details`](#get_hyperliquid_position_details) - src/positions.rs
- [`get_hyperliquid_portfolio_risk`](#get_hyperliquid_portfolio_risk) - src/positions.rs
- [`place_hyperliquid_order`](#place_hyperliquid_order) - src/trading.rs
- [`cancel_hyperliquid_order`](#cancel_hyperliquid_order) - src/trading.rs
- [`get_hyperliquid_account_info`](#get_hyperliquid_account_info) - src/trading.rs
- [`set_leverage`](#set_leverage) - src/trading.rs

## Tool Functions

### get_hyperliquid_positions

**Source**: `src/positions.rs`

```rust
pub async fn get_hyperliquid_positions() -> Result<Vec<HyperliquidPosition>, ToolError>
```

**Documentation:**

Get current positions on Hyperliquid

This tool retrieves all active perpetual futures positions for the user's account.
Returns comprehensive position data including unrealized PnL, leverage, liquidation prices,
and margin requirements. Only returns positions with non-zero size.

# Returns

Returns `Vec<HyperliquidPosition>` where each position contains:
- `symbol`: Trading pair (e.g., "ETH-PERP")
- `size`: Position size in contracts (positive for long, negative for short)
- `entry_price`: Average entry price of the position
- `mark_price`: Current market price fetched from Hyperliquid API
- `unrealized_pnl`: Unrealized profit/loss in USD
- `position_value`: Total position value in USD
- `leverage`: Current leverage multiplier
- `liquidation_price`: Price at which position gets liquidated
- `margin_used`: Amount of margin allocated to this position
- `return_on_equity`: ROE percentage
- `position_type`: Position mode (e.g., "oneWay")

# Errors

* `HyperliquidToolError::NetworkError` - When API connection fails
* `HyperliquidToolError::Generic` - When signer context unavailable or address parsing fails

# Examples

```rust,ignore
use riglr_hyperliquid_tools::positions::get_hyperliquid_positions;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let positions = get_hyperliquid_positions().await?;

for position in positions {
println!("Position: {} {} contracts", position.symbol, position.size);
println!("Entry: ${}, PnL: ${}", position.entry_price, position.unrealized_pnl);
println!("Leverage: {}x, Margin: ${}", position.leverage, position.margin_used);
}
# Ok(())
# }
```

---

### close_hyperliquid_position

**Source**: `src/positions.rs`

```rust
pub async fn close_hyperliquid_position( symbol: String, size: Option<String>, ) -> Result<HyperliquidCloseResult, ToolError>
```

**Documentation:**

Close a position on Hyperliquid

This tool automatically closes an existing perpetual futures position by placing a market order
in the opposite direction. It can close the entire position or a partial amount, with automatic
side detection (sells long positions, buys short positions).

# Arguments

* `symbol` - Trading pair symbol of the position to close
* `size` - Optional specific amount to close (closes entire position if None)

# Returns

Returns `HyperliquidCloseResult` containing:
- `symbol`: The trading pair that was closed
- `closed_size`: Amount of the position that was closed
- `order_side`: Direction of the closing order ("buy" or "sell")
- `order_id`: Order ID for the closing transaction
- `status`: Order status from the API
- `message`: Human-readable confirmation message

# Errors

* `HyperliquidToolError::InvalidInput` - When no position exists or size is invalid
* `HyperliquidToolError::ApiError` - When the closing order fails
* `HyperliquidToolError::NetworkError` - When connection issues occur

# Examples

```rust,ignore
use riglr_hyperliquid_tools::positions::close_hyperliquid_position;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Close entire position
let result = close_hyperliquid_position(
"ETH-PERP".to_string(),
None, // Close entire position
).await?;

println!("Closed {} {} position", result.closed_size, result.symbol);
println!("Order ID: {:?}", result.order_id);

// Close partial position
let partial = close_hyperliquid_position(
"BTC-PERP".to_string(),
Some("0.05".to_string()), // Close 0.05 BTC worth
).await?;
# Ok(())
# }
```

---

### get_hyperliquid_position_details

**Source**: `src/positions.rs`

```rust
pub async fn get_hyperliquid_position_details( symbol: String, ) -> Result<Option<HyperliquidPosition>, ToolError>
```

**Documentation:**

Get position details for a specific symbol

This tool retrieves detailed information about a position for a specific trading pair
on Hyperliquid. It performs flexible symbol matching, supporting both base symbols
(e.g., "ETH") and full perpetual contract names (e.g., "ETH-PERP").

# Arguments

* `symbol` - Trading pair symbol to query (e.g., "ETH", "BTC", "ETH-PERP")

# Returns

Returns `Option<HyperliquidPosition>`:
- `Some(position)` - If an active position exists for the symbol
- `None` - If no position is found for the symbol

When a position is found, it contains:
- `symbol`: Full trading pair name (e.g., "ETH-PERP")
- `size`: Position size (positive for long, negative for short)
- `entry_price`: Average entry price
- `unrealized_pnl`: Current profit/loss in USD
- `leverage`: Current leverage multiplier
- `liquidation_price`: Price level where position gets liquidated
- Additional margin and return metrics

# Errors

* `HyperliquidToolError::NetworkError` - When API connection fails
* `HyperliquidToolError::Generic` - When signer context unavailable

# Examples

```rust,ignore
use riglr_hyperliquid_tools::positions::get_hyperliquid_position_details;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Check if we have an ETH position
let position = get_hyperliquid_position_details("ETH".to_string()).await?;

match position {
Some(pos) => {
println!("ETH Position Found:");
println!("  Size: {} contracts", pos.size);
println!("  Entry Price: ${}", pos.entry_price);
println!("  PnL: ${}", pos.unrealized_pnl);
println!("  Leverage: {}x", pos.leverage);
println!("  Liquidation: ${}", pos.liquidation_price);
}
None => {
println!("No ETH position found");
}
}

// Also works with full symbol names
let btc_position = get_hyperliquid_position_details("BTC-PERP".to_string()).await?;
# Ok(())
# }
```

---

### get_hyperliquid_portfolio_risk

**Source**: `src/positions.rs`

```rust
pub async fn get_hyperliquid_portfolio_risk() -> Result<HyperliquidRiskMetrics, ToolError>
```

**Documentation:**

Calculate position risk metrics

This tool performs comprehensive risk analysis of the entire Hyperliquid portfolio,
calculating exposure metrics, margin utilization, and identifying positions at risk.
Essential for risk management and portfolio monitoring in leveraged trading.

# Returns

Returns `HyperliquidRiskMetrics` containing:
- `total_positions`: Number of active positions
- `total_position_value`: Combined value of all positions in USD
- `total_unrealized_pnl`: Sum of unrealized P&L across all positions
- `margin_utilization_percent`: Percentage of available margin being used
- `max_leverage`: Highest leverage across all positions
- `positions_at_risk`: Number of positions with high leverage or negative P&L
- `risk_level`: Overall risk assessment ("LOW", "MEDIUM", "HIGH")

# Risk Level Calculation

Risk level is determined by:
- **HIGH**: >80% margin utilization, >50x max leverage, or >75% positions at risk
- **MEDIUM**: >50% margin utilization, >20x max leverage, or >50% positions at risk
- **LOW**: All other scenarios

# Errors

* `HyperliquidToolError::NetworkError` - When API connection fails
* `HyperliquidToolError::Generic` - When signer context unavailable

# Examples

```rust,ignore
use riglr_hyperliquid_tools::positions::get_hyperliquid_portfolio_risk;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let risk = get_hyperliquid_portfolio_risk().await?;

println!("Portfolio Risk Analysis:");
println!("  Risk Level: {}", risk.risk_level);
println!("  Total Positions: {}", risk.total_positions);
println!("  Total Exposure: ${}", risk.total_position_value);
println!("  Unrealized P&L: ${}", risk.total_unrealized_pnl);
println!("  Margin Utilization: {:.1}%", risk.margin_utilization_percent);
println!("  Max Leverage: {}x", risk.max_leverage);
println!("  Positions at Risk: {}", risk.positions_at_risk);

// Risk management alerts
match risk.risk_level.as_str() {
"HIGH" => println!("⚠️  HIGH RISK: Consider reducing positions"),
"MEDIUM" => println!("🔶 MEDIUM RISK: Monitor closely"),
"LOW" => println!("✅ LOW RISK: Portfolio within safe parameters"),
_ => {}
}

// Margin utilization warning
if risk.margin_utilization_percent > 70.0 {
println!("💡 Consider adding more margin or reducing position sizes");
}
# Ok(())
# }
```

---

### place_hyperliquid_order

**Source**: `src/trading.rs`

```rust
pub async fn place_hyperliquid_order( symbol: String, side: String, size: String, order_type: String, price: Option<String>, reduce_only: Option<bool>, time_in_force: Option<String>, ) -> Result<HyperliquidOrderResult, ToolError>
```

**Documentation:**

Place a perpetual futures order on Hyperliquid

This tool places market or limit orders for perpetual futures on the Hyperliquid DEX.
It supports both long and short positions with configurable leverage, time-in-force options,
and risk management features like reduce-only orders.

# Arguments

* `symbol` - Trading pair symbol (e.g., "ETH-PERP", "BTC", "SOL-PERP")
* `side` - Order side: "buy"/"long" or "sell"/"short"
* `size` - Position size as decimal string (e.g., "0.1" for 0.1 contracts)
* `order_type` - "market" for immediate execution or "limit" for price-specific order
* `price` - Limit price (required for limit orders, ignored for market orders)
* `reduce_only` - If true, order can only reduce existing position size
* `time_in_force` - "gtc" (good-till-cancel), "ioc" (immediate-or-cancel), or "alo" (add-liquidity-only)

# Returns

Returns `HyperliquidOrderResult` containing:
- Order details (symbol, side, size, type, price)
- `status`: API response status
- `order_id`: Unique order identifier for tracking
- `message`: Human-readable result description

# Errors

* `HyperliquidToolError::InvalidInput` - When parameters are invalid (negative size, unknown symbol)
* `HyperliquidToolError::ApiError` - When Hyperliquid API rejects the order
* `HyperliquidToolError::NetworkError` - When connection issues occur

# Examples

```rust,ignore
use riglr_hyperliquid_tools::trading::place_hyperliquid_order;
use riglr_core::SignerContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Place a limit buy order for 0.1 ETH-PERP at $2000
let result = place_hyperliquid_order(
"ETH-PERP".to_string(),
"buy".to_string(),
"0.1".to_string(),
"limit".to_string(),
Some("2000.0".to_string()),
Some(false), // Not reduce-only
Some("gtc".to_string()), // Good till cancel
).await?;

println!("Order placed! ID: {:?}", result.order_id);
println!("Status: {}", result.status);
# Ok(())
# }
```

---

### cancel_hyperliquid_order

**Source**: `src/trading.rs`

```rust
pub async fn cancel_hyperliquid_order( symbol: String, order_id: String, ) -> Result<HyperliquidCancelResult, ToolError>
```

**Documentation:**

Cancel an existing order on Hyperliquid

This tool cancels a previously placed order that is still active (not filled or expired).
Both the symbol and order ID are required to ensure the correct order is canceled.

# Arguments

* `symbol` - Trading pair symbol that the order belongs to
* `order_id` - Unique order identifier returned from place_hyperliquid_order

# Returns

Returns `HyperliquidCancelResult` containing cancellation confirmation details.

# Errors

* `HyperliquidToolError::InvalidInput` - When order_id format is invalid or symbol unknown
* `HyperliquidToolError::ApiError` - When order doesn't exist or already filled
* `HyperliquidToolError::NetworkError` - When connection issues occur

# Examples

```rust,ignore
use riglr_hyperliquid_tools::trading::cancel_hyperliquid_order;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let result = cancel_hyperliquid_order(
"ETH-PERP".to_string(),
"12345678".to_string(), // Order ID from previous order
).await?;

println!("Order canceled: {}", result.message);
# Ok(())
# }
```

---

### get_hyperliquid_account_info

**Source**: `src/trading.rs`

```rust
pub async fn get_hyperliquid_account_info() -> Result<HyperliquidAccountResult, ToolError>
```

**Documentation:**

Get account information from Hyperliquid

This tool retrieves comprehensive account information including balance, margin usage,
and position counts. Essential for monitoring account health and available trading capital.

# Returns

Returns `HyperliquidAccountResult` containing:
- `user_address`: The account's Ethereum address
- `withdrawable_balance`: Available balance that can be withdrawn
- `cross_margin_used`: Amount of balance used for cross-margin positions
- `cross_maintenance_margin_used`: Maintenance margin requirements
- `positions_count`: Number of active positions

# Errors

* `HyperliquidToolError::NetworkError` - When API connection fails
* `HyperliquidToolError::Generic` - When signer context is unavailable

# Examples

```rust,ignore
use riglr_hyperliquid_tools::trading::get_hyperliquid_account_info;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let account = get_hyperliquid_account_info().await?;

println!("Account: {}", account.user_address);
println!("Withdrawable: ${}", account.withdrawable_balance);
println!("Margin used: ${}", account.cross_margin_used);
println!("Active positions: {}", account.positions_count);
# Ok(())
# }
```

---

### set_leverage

**Source**: `src/trading.rs`

```rust
pub async fn set_leverage( symbol: String, leverage: u32, ) -> Result<HyperliquidLeverageResult, ToolError>
```

**Documentation:**

Set leverage for a trading pair on Hyperliquid

This tool sets the leverage multiplier for a specific asset on Hyperliquid.
The leverage determines the maximum position size relative to your margin.

# Arguments

* `symbol` - Trading pair symbol (e.g., "ETH", "BTC", "SOL")
* `leverage` - Leverage multiplier (1-100x)

# Returns

Returns `HyperliquidLeverageResult` containing the updated leverage settings.

# Errors

* `ToolError::permanent` - Invalid leverage value or symbol
* `ToolError::retriable` - Network or API errors

# Examples

```rust,ignore
use riglr_hyperliquid_tools::trading::set_leverage;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Set 10x leverage for ETH perpetual futures
let result = set_leverage(
"ETH".to_string(),
10,
).await?;

println!("Leverage updated: {}", result.message);
# Ok(())
# }
```

---


---

*This documentation was automatically generated from the source code.*