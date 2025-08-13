# riglr-hyperliquid-tools API Reference

Comprehensive API documentation for the `riglr-hyperliquid-tools` crate.

## Table of Contents

### Enums

- [`HyperliquidToolError`](#hyperliquidtoolerror)

### Constants

- [`VERSION`](#version)

### Tools

- [`cancel_hyperliquid_order`](#cancel_hyperliquid_order)
- [`close_hyperliquid_position`](#close_hyperliquid_position)
- [`get_hyperliquid_account_info`](#get_hyperliquid_account_info)
- [`get_hyperliquid_portfolio_risk`](#get_hyperliquid_portfolio_risk)
- [`get_hyperliquid_position_details`](#get_hyperliquid_position_details)
- [`get_hyperliquid_positions`](#get_hyperliquid_positions)
- [`place_hyperliquid_order`](#place_hyperliquid_order)
- [`set_leverage`](#set_leverage)

### Structs

- [`AccountInfo`](#accountinfo)
- [`AssetInfo`](#assetinfo)
- [`CancelResponse`](#cancelresponse)
- [`CancelResult`](#cancelresult)
- [`ClearinghouseState`](#clearinghousestate)
- [`HyperliquidAccountResult`](#hyperliquidaccountresult)
- [`HyperliquidCancelResult`](#hyperliquidcancelresult)
- [`HyperliquidClient`](#hyperliquidclient)
- [`HyperliquidCloseResult`](#hyperliquidcloseresult)
- [`HyperliquidLeverageResult`](#hyperliquidleverageresult)
- [`HyperliquidOrderResult`](#hyperliquidorderresult)
- [`HyperliquidPosition`](#hyperliquidposition)
- [`HyperliquidRiskMetrics`](#hyperliquidriskmetrics)
- [`LeverageAction`](#leverageaction)
- [`LeverageRequest`](#leveragerequest)
- [`LeverageResponse`](#leverageresponse)
- [`LeverageResult`](#leverageresult)
- [`LimitOrderType`](#limitordertype)
- [`Meta`](#meta)
- [`OrderData`](#orderdata)
- [`OrderRequest`](#orderrequest)
- [`OrderResponse`](#orderresponse)
- [`OrderResult`](#orderresult)
- [`OrderStatus`](#orderstatus)
- [`OrderType`](#ordertype)
- [`Position`](#position)
- [`PositionData`](#positiondata)
- [`PositionLeverage`](#positionleverage)
- [`ResponseData`](#responsedata)
- [`RestingOrder`](#restingorder)

### Functions

- [`cancel_order`](#cancel_order)
- [`get`](#get)
- [`get_account_info`](#get_account_info)
- [`get_all_mids`](#get_all_mids)
- [`get_meta`](#get_meta)
- [`get_positions`](#get_positions)
- [`get_user_address`](#get_user_address)
- [`new`](#new)
- [`place_order`](#place_order)
- [`post`](#post)
- [`update_leverage`](#update_leverage)
- [`with_base_url`](#with_base_url)

## Enums

### HyperliquidToolError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum HyperliquidToolError { /// API error returned by Hyperliquid exchange #[error("API error: {0}")] ApiError(String), /// Invalid trading symbol provided #[error("Invalid symbol: {0}")] InvalidSymbol(String), /// Network connectivity error #[error("Network error: {0}")] NetworkError(String), /// Rate limit exceeded by Hyperliquid API #[error("Rate limited: {0}")] RateLimit(String), /// Authentication or authorization error #[error("Authentication error: {0}")] AuthError(String), /// Insufficient balance for requested operation #[error("Insufficient balance: {0}")] InsufficientBalance(String), /// Trading order related error #[error("Order error: {0}")] OrderError(String), /// Configuration or setup error #[error("Configuration error: {0}")] Configuration(String), }
```

Main error type for Hyperliquid tool operations

**Variants**:

- `ApiError(String)`
- `InvalidSymbol(String)`
- `NetworkError(String)`
- `RateLimit(String)`
- `AuthError(String)`
- `InsufficientBalance(String)`
- `OrderError(String)`
- `Configuration(String)`

---

## Constants

### VERSION

**Source**: `src/lib.rs`

```rust
const VERSION: &str
```

Current version of riglr-hyperliquid-tools

---

## Tools

### cancel_hyperliquid_order

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn cancel_hyperliquid_order( symbol: String, order_id: String, ) -> Result<HyperliquidCancelResult, ToolError>
```

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

### close_hyperliquid_position

**Source**: `src/positions.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn close_hyperliquid_position( symbol: String, size: Option<String>, ) -> Result<HyperliquidCloseResult, ToolError>
```

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

### get_hyperliquid_account_info

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_hyperliquid_account_info() -> Result<HyperliquidAccountResult, ToolError>
```

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

### get_hyperliquid_portfolio_risk

**Source**: `src/positions.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_hyperliquid_portfolio_risk() -> Result<HyperliquidRiskMetrics, ToolError>
```

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

### get_hyperliquid_position_details

**Source**: `src/positions.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_hyperliquid_position_details( symbol: String, ) -> Result<Option<HyperliquidPosition>, ToolError>
```

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

### get_hyperliquid_positions

**Source**: `src/positions.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_hyperliquid_positions() -> Result<Vec<HyperliquidPosition>, ToolError>
```

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

### place_hyperliquid_order

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn place_hyperliquid_order( symbol: String, side: String, size: String, order_type: String, price: Option<String>, reduce_only: Option<bool>, time_in_force: Option<String>, ) -> Result<HyperliquidOrderResult, ToolError>
```

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

### set_leverage

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn set_leverage( symbol: String, leverage: u32, ) -> Result<HyperliquidLeverageResult, ToolError>
```

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

## Structs

### AccountInfo

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct AccountInfo { /// List of asset positions held by the user #[serde(rename = "assetPositions")]
```

Account information response from Hyperliquid API

Contains the current state of a user's account including positions,
margin usage, and withdrawable funds.

---

### AssetInfo

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct AssetInfo { /// Name of the trading asset (e.g., "BTC", "ETH", "SOL")
```

Information about a tradeable asset on Hyperliquid

Specifies the asset name and precision details required
for proper order formatting and size calculations.

---

### CancelResponse

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct CancelResponse { /// Status of the cancellation API call ("ok" for success)
```

Response from order cancellation API call

Contains the status of the cancellation attempt
and result information indicating success or failure.

---

### CancelResult

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct CancelResult { /// Numeric status code indicating the result of the cancellation #[serde(rename = "statuses")]
```

Result of an order cancellation operation

Contains a status code indicating whether the
cancellation was successful or failed.

---

### ClearinghouseState

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct ClearinghouseState { /// List of asset positions held by the user #[serde(rename = "assetPositions")]
```

Clearinghouse state information from Hyperliquid API

Represents the current state of the clearinghouse for a user,
including positions and margin information.

---

### HyperliquidAccountResult

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidAccountResult { /// Account's Ethereum address pub user_address: String, /// Available balance that can be withdrawn pub withdrawable_balance: String, /// Amount of balance used for cross-margin positions pub cross_margin_used: String, /// Maintenance margin requirements for cross-margin pub cross_maintenance_margin_used: String, /// Number of active positions pub positions_count: usize, }
```

Account information retrieved from Hyperliquid

Contains balance, margin usage, and position information for the account.

---

### HyperliquidCancelResult

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidCancelResult { /// Trading pair symbol that the order belonged to pub symbol: String, /// Order ID that was canceled pub order_id: String, /// Exchange response status pub status: String, /// Human-readable result message pub message: String, }
```

Result returned after canceling an order on Hyperliquid

Contains confirmation details about the canceled order.

---

### HyperliquidClient

**Source**: `src/client.rs`

```rust
pub struct HyperliquidClient { client: Client, base_url: String, signer: Arc<dyn TransactionSigner>, }
```

Hyperliquid API client - Real implementation using HTTP API
This client makes REAL API calls to Hyperliquid - NO SIMULATION

---

### HyperliquidCloseResult

**Source**: `src/positions.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidCloseResult { /// Trading pair symbol that was closed pub symbol: String, /// Amount of the position that was closed pub closed_size: String, /// Direction of the closing order ("buy" or "sell")
```

Result of closing a position on Hyperliquid

Contains the outcome details when a position is closed,
including order information and execution status.

---

### HyperliquidLeverageResult

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidLeverageResult { /// Trading pair symbol that leverage was set for pub symbol: String, /// Leverage multiplier (1-100x)
```

Result returned after setting leverage on Hyperliquid

Contains confirmation details about the leverage update.

---

### HyperliquidOrderResult

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidOrderResult { /// Trading pair symbol (e.g., "ETH-PERP", "BTC")
```

Result returned after placing an order on Hyperliquid

Contains all relevant information about the placed order including
the original parameters and the exchange's response.

---

### HyperliquidPosition

**Source**: `src/positions.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidPosition { /// Trading pair symbol (e.g., "ETH-PERP", "BTC-PERP")
```

Represents a trading position on Hyperliquid

Contains comprehensive position data including size, prices, leverage,
profit/loss metrics, and risk information for a perpetual futures position.

---

### HyperliquidRiskMetrics

**Source**: `src/positions.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidRiskMetrics { /// Total number of active positions in the portfolio pub total_positions: usize, /// Combined value of all positions in USD pub total_position_value: String, /// Sum of unrealized profit/loss across all positions pub total_unrealized_pnl: String, /// Percentage of available margin currently being used (0-100)
```

Portfolio risk analysis metrics for Hyperliquid positions

Provides comprehensive risk assessment including exposure,
margin utilization, leverage analysis, and overall risk rating.

---

### LeverageAction

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct LeverageAction { /// Type of action ("updateLeverage" for leverage changes)
```

Action specification for leverage updates

Defines the specific leverage change to be made,
including the asset, margin type, and new leverage value.

---

### LeverageRequest

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct LeverageRequest { /// The leverage action to perform (update leverage)
```

Request to update leverage settings for a trading asset

Contains the leverage action to perform, a nonce for security,
and an optional signature for authentication.

---

### LeverageResponse

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct LeverageResponse { /// Status of the leverage update API call ("success" for successful updates)
```

Response from leverage update API call

Contains the status of the leverage update attempt
and result data if the update was successful.

---

### LeverageResult

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct LeverageResult { /// New leverage multiplier that was set pub leverage: u32, /// Asset symbol that had its leverage updated pub asset: String, }
```

Result of a successful leverage update operation

Contains the new leverage value and the asset
that was updated, confirming the change was applied.

---

### LimitOrderType

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct LimitOrderType { /// Time in force: "Gtc" (Good Till Cancelled), "Ioc" (Immediate or Cancel), "Alo" (Add Liquidity Only)
```

Limit order configuration with time-in-force settings

Specifies how long a limit order should remain active
in the order book before expiring or being cancelled.

---

### Meta

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct Meta { /// List of all available trading assets with their specifications pub universe: Vec<AssetInfo>, }
```

Market metadata from Hyperliquid API

Contains information about all available trading assets
and their specifications on the Hyperliquid exchange.

---

### OrderData

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderData { /// List of order statuses for each order in the request pub statuses: Vec<OrderStatus>, }
```

Order data containing status information for placed orders

Contains an array of order statuses, typically one per order
in the batch request (single orders will have one status).

---

### OrderRequest

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderRequest { /// Asset ID for the trading pair (numeric identifier)
```

Order placement request for Hyperliquid API

Represents a request to place a new trading order with all
necessary parameters including price, size, and order type.

---

### OrderResponse

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderResponse { /// Status of the API call ("ok" for success, error message for failure)
```

Response from order placement API call

Contains the status of the order placement attempt
and detailed result information if successful.

---

### OrderResult

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderResult { /// Numeric status code indicating the result of the operation #[serde(rename = "statuses")]
```

Detailed result of an order placement operation

Contains status codes and response data indicating
whether the order was successfully placed.

---

### OrderStatus

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderStatus { /// Information about the order if it's resting in the order book pub resting: RestingOrder, }
```

Status information for a single order

Indicates whether the order is resting in the order book
and provides access to the order identifier.

---

### OrderType

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderType { /// Limit order configuration, if this is a limit order #[serde(rename = "limit")]
```

Order type configuration for trading orders

Specifies the type of order and its execution parameters.
Currently supports limit orders with time-in-force options.

---

### Position

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct Position { /// Detailed position data including size, entry price, and P&L #[serde(rename = "position")]
```

Trading position information from Hyperliquid API

Represents a user's position in a specific trading asset,
containing detailed position data and type information.

---

### PositionData

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct PositionData { /// The trading symbol/coin for this position (e.g., "BTC", "ETH")
```

Detailed position data from Hyperliquid API

Contains comprehensive information about a trading position including
entry price, leverage, margin usage, and profit/loss metrics.

---

### PositionLeverage

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct PositionLeverage { /// Type of leverage (e.g., "cross" for cross margin, "isolated" for isolated margin)
```

Leverage configuration for a trading position

Specifies the leverage type and multiplier used for a position.

---

### ResponseData

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct ResponseData { /// Type of response (e.g., "order" for order placement responses)
```

Response data container for order operations

Wraps the actual order data with type information
to indicate the kind of response received.

---

### RestingOrder

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct RestingOrder { /// Order ID assigned by the exchange for this resting order pub oid: u64, }
```

Information about an order resting in the order book

Contains the order identifier that can be used to reference
the order for cancellation or modification operations.

---

## Functions

### cancel_order

**Source**: `src/client.rs`

```rust
pub async fn cancel_order(&self, order_id: u64, asset: u32) -> Result<CancelResponse>
```

Cancel an order using real Hyperliquid API

# Arguments
* `order_id` - The ID of the order to cancel
* `asset` - The asset ID for the order

# Warning
This performs REAL order cancellation - NO SIMULATION

---

### get

**Source**: `src/client.rs`

```rust
pub async fn get(&self, endpoint: &str) -> Result<Response>
```

Make a GET request to the Hyperliquid API

---

### get_account_info

**Source**: `src/client.rs`

```rust
pub async fn get_account_info(&self, user_address: &str) -> Result<AccountInfo>
```

Get account information

---

### get_all_mids

**Source**: `src/client.rs`

```rust
pub async fn get_all_mids(&self) -> Result<serde_json::Value>
```

Get all market mid prices (current market prices)

---

### get_meta

**Source**: `src/client.rs`

```rust
pub async fn get_meta(&self) -> Result<Meta>
```

Get market information

---

### get_positions

**Source**: `src/client.rs`

```rust
pub async fn get_positions(&self, user_address: &str) -> Result<Vec<Position>>
```

Get current positions for a user

---

### get_user_address

**Source**: `src/client.rs`

```rust
pub fn get_user_address(&self) -> Result<String>
```

Get the user's address from the signer

Returns the address associated with the current signer, which is used
for identifying the user in Hyperliquid API calls.

---

### new

**Source**: `src/client.rs`

```rust
pub fn new(signer: Arc<dyn TransactionSigner>) -> Result<Self>
```

Create a new Hyperliquid client

---

### place_order

**Source**: `src/client.rs`

```rust
pub async fn place_order(&self, order: &OrderRequest) -> Result<OrderResponse>
```

Place an order using real Hyperliquid API
CRITICAL: This is REAL order placement - NO SIMULATION

---

### post

**Source**: `src/client.rs`

```rust
pub async fn post<T: Serialize>(&self, endpoint: &str, payload: &T) -> Result<Response>
```

Make a POST request to the Hyperliquid API

---

### update_leverage

**Source**: `src/client.rs`

```rust
pub async fn update_leverage( &self, leverage: u32, coin: &str, is_cross: bool, asset_id: Option<u32>, ) -> Result<LeverageResponse>
```

Cancel an order using real Hyperliquid API
CRITICAL: This is REAL order cancellation - NO SIMULATION

---

### with_base_url

**Source**: `src/client.rs`

```rust
pub fn with_base_url(signer: Arc<dyn TransactionSigner>, base_url: String) -> Result<Self>
```

Create a new Hyperliquid client with custom base URL (for testing)

---


---

*This documentation was automatically generated from the source code.*