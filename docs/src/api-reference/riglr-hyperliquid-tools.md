# riglr-hyperliquid-tools API Reference

Comprehensive API documentation for the `riglr-hyperliquid-tools` crate.

## Table of Contents

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

### Enums

- [`HyperliquidToolError`](#hyperliquidtoolerror)

### Tools

- [`cancel_hyperliquid_order`](#cancel_hyperliquid_order)
- [`close_hyperliquid_position`](#close_hyperliquid_position)
- [`get_hyperliquid_account_info`](#get_hyperliquid_account_info)
- [`get_hyperliquid_portfolio_risk`](#get_hyperliquid_portfolio_risk)
- [`get_hyperliquid_position_details`](#get_hyperliquid_position_details)
- [`get_hyperliquid_positions`](#get_hyperliquid_positions)
- [`place_hyperliquid_order`](#place_hyperliquid_order)
- [`set_leverage`](#set_leverage)

### Constants

- [`VERSION`](#version)

## Structs

### AccountInfo

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct AccountInfo { #[serde(rename = "assetPositions")]
```

---

### AssetInfo

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct AssetInfo { pub name: String, #[serde(rename = "szDecimals")]
```

---

### CancelResponse

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct CancelResponse { pub status: String, pub data: CancelResult, }
```

---

### CancelResult

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct CancelResult { #[serde(rename = "statuses")]
```

---

### ClearinghouseState

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct ClearinghouseState { #[serde(rename = "assetPositions")]
```

---

### HyperliquidAccountResult

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidAccountResult { pub user_address: String, pub withdrawable_balance: String, pub cross_margin_used: String, pub cross_maintenance_margin_used: String, pub positions_count: usize, }
```

---

### HyperliquidCancelResult

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidCancelResult { pub symbol: String, pub order_id: String, pub status: String, pub message: String, }
```

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
pub struct HyperliquidCloseResult { pub symbol: String, pub closed_size: String, pub order_side: String, pub order_id: Option<String>, pub status: String, pub message: String, }
```

---

### HyperliquidLeverageResult

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidLeverageResult { pub symbol: String, pub leverage: u32, pub status: String, pub message: String, }
```

---

### HyperliquidOrderResult

**Source**: `src/trading.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidOrderResult { pub symbol: String, pub side: String, pub size: String, pub order_type: String, pub price: Option<String>, pub status: String, pub order_id: Option<String>, pub message: String, }
```

---

### HyperliquidPosition

**Source**: `src/positions.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidPosition { pub symbol: String, pub size: String, pub entry_price: String, pub mark_price: String, pub unrealized_pnl: String, pub position_value: String, pub leverage: u32, pub liquidation_price: String, pub margin_used: String, pub return_on_equity: String, pub position_type: String, }
```

---

### HyperliquidRiskMetrics

**Source**: `src/positions.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct HyperliquidRiskMetrics { pub total_positions: usize, pub total_position_value: String, pub total_unrealized_pnl: String, pub margin_utilization_percent: f64, pub max_leverage: u32, pub positions_at_risk: usize, pub risk_level: String, }
```

---

### LeverageAction

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct LeverageAction { #[serde(rename = "type")]
```

---

### LeverageRequest

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct LeverageRequest { pub action: LeverageAction, pub nonce: i64, pub signature: Option<String>, }
```

---

### LeverageResponse

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct LeverageResponse { pub status: String, pub data: Option<LeverageResult>, }
```

---

### LeverageResult

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct LeverageResult { pub leverage: u32, pub asset: String, }
```

---

### LimitOrderType

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct LimitOrderType { pub tif: String, // Time in force: "Gtc", "Ioc", "Alo" }
```

---

### Meta

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct Meta { pub universe: Vec<AssetInfo>, }
```

---

### OrderData

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderData { pub statuses: Vec<OrderStatus>, }
```

---

### OrderRequest

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderRequest { pub asset: u32, #[serde(rename = "isBuy")]
```

---

### OrderResponse

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderResponse { pub status: String, pub data: OrderResult, }
```

---

### OrderResult

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderResult { #[serde(rename = "statuses")]
```

---

### OrderStatus

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderStatus { pub resting: RestingOrder, }
```

---

### OrderType

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrderType { #[serde(rename = "limit")]
```

---

### Position

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct Position { #[serde(rename = "position")]
```

---

### PositionData

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct PositionData { pub coin: String, #[serde(rename = "entryPx")]
```

---

### PositionLeverage

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct PositionLeverage { #[serde(rename = "type")]
```

---

### ResponseData

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct ResponseData { #[serde(rename = "type")]
```

---

### RestingOrder

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct RestingOrder { pub oid: u64, }
```

---

## Functions

### cancel_order

**Source**: `src/client.rs`

```rust
pub async fn cancel_order(&self, order_id: u64, asset: u32) -> Result<CancelResponse>
```

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
pub async fn update_leverage(&self, leverage: u32, coin: &str, is_cross: bool, asset_id: Option<u32>) -> Result<LeverageResponse>
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

## Enums

### HyperliquidToolError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum HyperliquidToolError { #[error("API error: {0}")] ApiError(String), #[error("Invalid symbol: {0}")] InvalidSymbol(String), #[error("Network error: {0}")] NetworkError(String), #[error("Rate limited: {0}")] RateLimit(String), #[error("Authentication error: {0}")] AuthError(String), #[error("Insufficient balance: {0}")] InsufficientBalance(String), #[error("Order error: {0}")] OrderError(String), #[error("Configuration error: {0}")] Configuration(String), }
```

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

## Constants

### VERSION

**Source**: `src/lib.rs`

```rust
const VERSION: &str
```

Current version of riglr-hyperliquid-tools

---


---

*This documentation was automatically generated from the source code.*