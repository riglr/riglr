# riglr-cross-chain-tools API Reference

Comprehensive API documentation for the `riglr-cross-chain-tools` crate.

## Table of Contents

### Enums

- [`BridgeStatus`](#bridgestatus)
- [`ChainType`](#chaintype)
- [`CrossChainError`](#crosschainerror)
- [`LiFiError`](#lifierror)

### Structs

- [`BridgeExecutionResult`](#bridgeexecutionresult)
- [`BridgeFeeEstimate`](#bridgefeeestimate)
- [`BridgeStatusResponse`](#bridgestatusresponse)
- [`BridgeStatusResult`](#bridgestatusresult)
- [`Chain`](#chain)
- [`ChainInfo`](#chaininfo)
- [`CrossChainRoute`](#crosschainroute)
- [`FeeBreakdown`](#feebreakdown)
- [`LiFiClient`](#lificlient)
- [`RouteDiscoveryResult`](#routediscoveryresult)
- [`RouteFee`](#routefee)
- [`RouteInfo`](#routeinfo)
- [`RouteRequest`](#routerequest)
- [`RouteResponse`](#routeresponse)
- [`RouteStep`](#routestep)
- [`SolanaAccountMeta`](#solanaaccountmeta)
- [`StepAction`](#stepaction)
- [`StepEstimate`](#stepestimate)
- [`Token`](#token)
- [`TokenInfo`](#tokeninfo)
- [`TransactionRequest`](#transactionrequest)

### Tools

- [`estimate_bridge_fees`](#estimate_bridge_fees)
- [`execute_cross_chain_bridge`](#execute_cross_chain_bridge)
- [`get_bridge_status`](#get_bridge_status)
- [`get_cross_chain_routes`](#get_cross_chain_routes)
- [`get_supported_chains`](#get_supported_chains)

### Functions (lifi)

- [`chain_id_to_name`](#chain_id_to_name)
- [`chain_name_to_id`](#chain_name_to_id)
- [`get_bridge_status`](#get_bridge_status)
- [`get_chains`](#get_chains)
- [`get_route_with_transaction`](#get_route_with_transaction)
- [`get_routes`](#get_routes)
- [`get_transaction_request_for_route`](#get_transaction_request_for_route)
- [`prepare_bridge_execution`](#prepare_bridge_execution)
- [`with_api_key`](#with_api_key)
- [`with_base_url`](#with_base_url)

### Constants

- [`VERSION`](#version)

## Enums

### BridgeStatus

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
```

```rust
pub enum BridgeStatus { /// Transaction not found NotFound, /// Transaction is pending execution Pending, /// Transaction completed successfully Done, /// Transaction failed Failed, }
```

Bridge transaction status

**Variants**:

- `NotFound`
- `Pending`
- `Done`
- `Failed`

---

### ChainType

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
```

```rust
pub enum ChainType { /// Ethereum Virtual Machine based blockchain #[serde(rename = "evm")] Evm, /// Solana blockchain #[serde(rename = "solana")] Solana, }
```

Supported blockchain networks

**Variants**:

- `Evm`
- `Solana`

---

### CrossChainError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum CrossChainError { /// Core tool error #[error("Core tool error: {0}")] ToolError(#[from] ToolError), /// Li.fi API error #[error("Li.fi API error: {0}")] LifiApiError(String), /// Quote fetch failed #[error("Quote fetch failed: {0}")] QuoteFetchError(String), /// Invalid route configuration #[error("Invalid route configuration: {0}")] InvalidRoute(String), /// Bridge operation failed #[error("Bridge operation failed: {0}")] BridgeExecutionError(String), /// Unsupported chain pair #[error("Unsupported chain pair: {from_chain} -> {to_chain}")] UnsupportedChainPair { /// Source chain identifier from_chain: String, /// Destination chain identifier to_chain: String, }, /// Insufficient liquidity for amount #[error("Insufficient liquidity for amount: {amount}")] InsufficientLiquidity { /// Amount that was requested but unavailable amount: String }, }
```

Errors that can occur during cross-chain operations

**Variants**:

- `ToolError(#[from] ToolError)`
- `LifiApiError(String)`
- `QuoteFetchError(String)`
- `InvalidRoute(String)`
- `BridgeExecutionError(String)`
- `UnsupportedChainPair`
- `from_chain`
- `to_chain`
- `InsufficientLiquidity`
- `amount`

---

### LiFiError

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum LiFiError { /// HTTP request failed #[error("HTTP request failed: {0}")] Request(#[from] reqwest::Error), /// Invalid response format from API #[error("Invalid response format: {0}")] InvalidResponse(String), /// API returned an error response #[error("API error: {code} - {message}")] ApiError { /// HTTP status code code: u16, /// Error message from API message: String }, /// Chain is not supported by LiFi #[error("Chain not supported: {chain_name}")] UnsupportedChain { /// Name of the unsupported chain chain_name: String }, /// No route found between chains #[error("Route not found for {from_chain} -> {to_chain}")] RouteNotFound { /// Source chain name from_chain: String, /// Destination chain name to_chain: String, }, /// Configuration error #[error("Configuration error: {0}")] Configuration(String), /// URL parsing error #[error("URL parsing error: {0}")] UrlParse(#[from] ParseError), }
```

Errors that can occur during LiFi API operations

**Variants**:

- `Request(#[from] reqwest::Error)`
- `InvalidResponse(String)`
- `ApiError`
- `code`
- `message`
- `UnsupportedChain`
- `chain_name`
- `RouteNotFound`
- `from_chain`
- `to_chain`
- `Configuration(String)`
- `UrlParse(#[from] ParseError)`

---

## Structs

### BridgeExecutionResult

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct BridgeExecutionResult { /// Unique identifier for tracking this bridge operation pub bridge_id: String, /// Transaction hash on the source chain pub source_tx_hash: String, /// Source chain name pub from_chain: String, /// Destination chain name pub to_chain: String, /// Amount sent (in token's smallest unit)
```

Result of executing a cross-chain bridge

---

### BridgeFeeEstimate

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct BridgeFeeEstimate { /// Source chain name pub from_chain: String, /// Destination chain name pub to_chain: String, /// Input amount pub from_amount: String, /// Expected output amount after all fees pub estimated_output: String, /// Total fees breakdown pub fees: Vec<FeeBreakdown>, /// Total fees in USD pub total_fees_usd: Option<f64>, /// Gas cost estimate in USD pub gas_cost_usd: Option<f64>, /// Estimated completion time in seconds pub estimated_duration: u64, }
```

Result of bridge fee estimation

---

### BridgeStatusResponse

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct BridgeStatusResponse { /// Current status of the bridge transaction pub status: BridgeStatus, /// Source chain ID pub from_chain_id: Option<u64>, /// Destination chain ID pub to_chain_id: Option<u64>, /// Tool/protocol used for bridging pub tool: Option<String>, /// Transaction hash on source chain pub sending_tx_hash: Option<String>, /// Transaction hash on destination chain pub receiving_tx_hash: Option<String>, /// Amount sent from source chain pub amount_sent: Option<String>, /// Amount received on destination chain pub amount_received: Option<String>, }
```

Bridge transaction status response

---

### BridgeStatusResult

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct BridgeStatusResult { /// Bridge operation identifier pub bridge_id: String, /// Current status pub status: String, /// Source chain transaction hash pub source_tx_hash: Option<String>, /// Destination chain transaction hash (if completed)
```

Result of checking bridge status

---

### Chain

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct Chain { /// Unique chain identifier pub id: u64, /// Human-readable chain name pub name: String, /// Chain key used by LiFi API pub key: String, /// Type of blockchain (EVM or Solana)
```

Chain information from LiFi

---

### ChainInfo

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct ChainInfo { /// Numeric chain ID pub id: u64, /// Human-readable chain name pub name: String, /// Short key identifier pub key: String, /// Chain type ("evm" or "solana")
```

Chain information for supported networks

---

### CrossChainRoute

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct CrossChainRoute { /// Unique route identifier pub id: String, /// Source chain ID pub from_chain_id: u64, /// Destination chain ID pub to_chain_id: u64, /// Token being sent from source chain pub from_token: Token, /// Token being received on destination chain pub to_token: Token, /// Amount to send (in token units)
```

A cross-chain route option from LiFi

---

### FeeBreakdown

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct FeeBreakdown { /// Fee name (e.g., "Bridge Fee", "Gas Fee")
```

Fee breakdown information

---

### LiFiClient

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct LiFiClient { /// HTTP client for API requests client: reqwest::Client, /// Base URL for LiFi API base_url: Url, /// Optional API key for authentication api_key: Option<String>, }
```

LiFi Protocol API client

---

### RouteDiscoveryResult

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct RouteDiscoveryResult { /// Available routes sorted by best to worst pub routes: Vec<RouteInfo>, /// Total number of routes found pub total_routes: usize, /// Recommended route (if any)
```

Result of a cross-chain route discovery

---

### RouteFee

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct RouteFee { /// Fee name/type pub name: String, /// Human-readable fee description pub description: String, /// Fee percentage (as string)
```

Fee information for a route

---

### RouteInfo

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct RouteInfo { /// Unique route identifier pub id: String, /// Source chain name pub from_chain: String, /// Destination chain name pub to_chain: String, /// Source token info pub from_token: TokenInfo, /// Destination token info pub to_token: TokenInfo, /// Input amount (in token's smallest unit)
```

Simplified route information for tools

---

### RouteRequest

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct RouteRequest { /// Source chain ID pub from_chain: u64, /// Destination chain ID pub to_chain: u64, /// Source token address pub from_token: String, /// Destination token address pub to_token: String, /// Amount to bridge (in token units)
```

Request parameters for getting cross-chain routes

---

### RouteResponse

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct RouteResponse { /// Available cross-chain routes pub routes: Vec<CrossChainRoute>, }
```

Response from the routes API

---

### RouteStep

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct RouteStep { /// Unique step identifier pub id: String, /// Step type (e.g., "lifi", "cross", "swap")
```

A step within a cross-chain route

---

### SolanaAccountMeta

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct SolanaAccountMeta { /// Public key of the account pub pubkey: String, /// Whether this account must sign the transaction pub is_signer: bool, /// Whether this account is writable pub is_writable: bool, }
```

Solana account metadata for building instructions

---

### StepAction

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct StepAction { /// Source chain ID for this step pub from_chain_id: u64, /// Destination chain ID for this step pub to_chain_id: u64, /// Input token for this step pub from_token: Token, /// Output token for this step pub to_token: Token, /// Input amount for this step pub from_amount: String, /// Expected output amount for this step pub to_amount: String, }
```

Action details for a route step

---

### StepEstimate

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct StepEstimate { /// Tool/protocol used for estimation pub tool: String, /// Contract address that needs approval (if any)
```

Execution estimate for a step

---

### Token

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct Token { /// Token contract address pub address: String, /// Token symbol (e.g., ETH, USDC)
```

Token information

---

### TokenInfo

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct TokenInfo { /// Token contract address (or mint for Solana)
```

Simplified token information

---

### TransactionRequest

**Source**: `src/lifi.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct TransactionRequest { /// Target contract address pub to: String, /// Transaction data (hex encoded)
```

Transaction request data for executing cross-chain bridges

---

## Tools

### estimate_bridge_fees

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn estimate_bridge_fees( from_chain: String, to_chain: String, from_token: String, to_token: String, amount: String, ) -> Result<BridgeFeeEstimate, ToolError>
```

Estimate fees and completion time for a cross-chain bridge operation

This tool provides detailed cost analysis and timing estimates for bridging tokens
between different blockchain networks without executing any transactions. Useful for
comparing bridge options and budgeting for cross-chain transfers.

# Arguments

* `from_chain` - Source blockchain name
* `to_chain` - Destination blockchain name
* `from_token` - Source token address
* `to_token` - Destination token address
* `amount` - Transfer amount in token's smallest unit

# Returns

Returns `BridgeFeeEstimate` containing:
- `from_chain`, `to_chain`: Source and destination networks
- `from_amount`: Input amount
- `estimated_output`: Expected output after all fees
- `fees`: Detailed breakdown of different fee types
- `total_fees_usd`: Total fees in USD (if available)
- `gas_cost_usd`: Gas cost estimate in USD
- `estimated_duration`: Expected completion time in seconds

Each fee in the breakdown includes name, description, percentage, amount, and USD value.

# Errors

* `CrossChainToolError::UnsupportedChain` - When chain names are not supported
* `CrossChainToolError::RouteNotFound` - When no routes exist for fee estimation
* `CrossChainToolError::NetworkError` - When API connection fails

# Examples

```rust,ignore
use riglr_cross_chain_tools::bridge::estimate_bridge_fees;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let estimate = estimate_bridge_fees(
"ethereum".to_string(),
"polygon".to_string(),
"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC on Ethereum
"0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(), // USDC on Polygon
"100000000".to_string(), // 100 USDC (6 decimals)
).await?;

println!("Bridge estimate for {} USDC:", "100");
println!("Expected output: {}", estimate.estimated_output);
println!("Duration: {}s (~{} minutes)",
estimate.estimated_duration,
estimate.estimated_duration / 60);

if let Some(total_fees) = estimate.total_fees_usd {
println!("Total fees: ${:.2}", total_fees);
}

for fee in estimate.fees {
println!("  {}: {} ({}%)", fee.name, fee.amount, fee.percentage);
}
# Ok(())
# }
```

---

### execute_cross_chain_bridge

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn execute_cross_chain_bridge( route_id: String, from_chain: String, to_chain: String, amount: String, ) -> Result<BridgeExecutionResult, ToolError>
```

Execute a cross-chain bridge transaction using a previously discovered route

This tool executes an actual cross-chain token transfer by taking a route ID from
get_cross_chain_routes and constructing the appropriate transaction. The transaction
is signed using the current signer context and submitted to the source blockchain.

# Arguments

* `route_id` - Route identifier from get_cross_chain_routes
* `from_chain` - Source blockchain name for validation
* `to_chain` - Destination blockchain name for validation
* `amount` - Amount to bridge in token's smallest unit

# Returns

Returns `BridgeExecutionResult` containing:
- `bridge_id`: Unique identifier for tracking the bridge operation
- `source_tx_hash`: Transaction hash on the source chain
- `from_chain`, `to_chain`: Source and destination networks
- `amount_sent`, `expected_amount`: Transfer amounts
- `status`: Current bridge status (e.g., "PENDING", "CONFIRMED")
- `estimated_completion`: Expected completion time in seconds
- `message`: Status message and tracking instructions

# Errors

* `CrossChainToolError::InvalidRoute` - When route ID is invalid or expired
* `CrossChainToolError::InsufficientFunds` - When account lacks required tokens
* `CrossChainToolError::TransactionFailed` - When transaction construction or submission fails
* `CrossChainToolError::NetworkError` - When connection issues occur

# Examples

```rust,ignore
use riglr_cross_chain_tools::bridge::{get_cross_chain_routes, execute_cross_chain_bridge};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// First get routes
let routes = get_cross_chain_routes(/* ... */).await?;
let best_route = &routes.routes[0];

// Execute the bridge
let result = execute_cross_chain_bridge(
best_route.id.clone(),
"ethereum".to_string(),
"polygon".to_string(),
"1000000000".to_string(),
).await?;

println!("Bridge initiated!");
println!("Bridge ID: {}", result.bridge_id);
println!("Source tx: {}", result.source_tx_hash);
println!("Status: {}", result.status);
println!("Track with: get_bridge_status");
# Ok(())
# }
```

---

### get_bridge_status

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_bridge_status( bridge_id: String, source_tx_hash: String, ) -> Result<BridgeStatusResult, ToolError>
```

Check the status of an ongoing cross-chain bridge operation

This tool monitors the progress of a cross-chain bridge transaction using the bridge ID
and source transaction hash returned from execute_cross_chain_bridge. Essential for
tracking multi-step bridge operations that can take several minutes to complete.

# Arguments

* `bridge_id` - Unique bridge operation identifier from execute_cross_chain_bridge
* `source_tx_hash` - Transaction hash from the source chain

# Returns

Returns `BridgeStatusResult` containing:
- `bridge_id`: The tracked bridge operation ID
- `status`: Current status ("PENDING", "DONE", "FAILED", etc.)
- `source_tx_hash`: Source chain transaction hash
- `destination_tx_hash`: Destination chain transaction hash (when completed)
- `amount_sent`, `amount_received`: Actual transfer amounts
- `message`: Human-readable status description
- `is_complete`, `is_failed`: Boolean flags for operation state

# Errors

* `CrossChainToolError::BridgeNotFound` - When bridge ID or transaction hash is invalid
* `CrossChainToolError::NetworkError` - When status lookup fails

# Examples

```rust,ignore
use riglr_cross_chain_tools::bridge::get_bridge_status;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let status = get_bridge_status(
"bridge-123-abc".to_string(),
"0x1234...abcd".to_string(),
).await?;

println!("Bridge status: {}", status.status);
println!("Message: {}", status.message);

if status.is_complete {
println!("✅ Bridge completed successfully!");
if let Some(dest_tx) = status.destination_tx_hash {
println!("Destination tx: {}", dest_tx);
}
if let Some(received) = status.amount_received {
println!("Amount received: {}", received);
}
} else if status.is_failed {
println!("❌ Bridge failed: {}", status.message);
} else {
println!("⏳ Bridge in progress...");
}
# Ok(())
# }
```

---

### get_cross_chain_routes

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_cross_chain_routes( from_chain: String, to_chain: String, from_token: String, to_token: String, amount: String, slippage_percent: Option<f64>, ) -> Result<RouteDiscoveryResult, ToolError>
```

Get available cross-chain routes between tokens on different networks

This tool discovers optimal paths for transferring tokens between blockchain networks
using LiFi Protocol's aggregation of multiple bridge providers and DEXs. Routes are
automatically sorted by quality, cost, and speed with the best options first.

# Arguments

* `from_chain` - Source blockchain name (e.g., "ethereum", "polygon", "arbitrum", "solana")
* `to_chain` - Destination blockchain name
* `from_token` - Source token address (contract address or mint address for Solana)
* `to_token` - Destination token address
* `amount` - Transfer amount in token's smallest unit (e.g., wei for ETH, lamports for SOL)
* `slippage_percent` - Maximum acceptable slippage as percentage (e.g., 0.5 for 0.5%)

# Returns

Returns `RouteDiscoveryResult` containing:
- `routes`: Available routes sorted by quality (best first)
- `total_routes`: Number of routes found
- `recommended_route_id`: ID of the recommended route (if any)

Each route includes detailed information about fees, duration, protocols used, and expected amounts.

# Errors

* `CrossChainToolError::UnsupportedChain` - When chain names are not supported
* `CrossChainToolError::RouteNotFound` - When no routes exist between the chains/tokens
* `CrossChainToolError::ApiError` - When LiFi API issues occur
* `CrossChainToolError::NetworkError` - When connection problems occur

# Examples

```rust,ignore
use riglr_cross_chain_tools::bridge::get_cross_chain_routes;
use riglr_core::SignerContext;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Find routes to bridge USDC from Ethereum to Polygon
let routes = get_cross_chain_routes(
"ethereum".to_string(),
"polygon".to_string(),
"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC on Ethereum
"0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(), // USDC on Polygon
"1000000000".to_string(), // 1000 USDC (6 decimals)
Some(0.5), // 0.5% slippage tolerance
).await?;

println!("Found {} routes", routes.total_routes);
if let Some(best_route) = routes.routes.first() {
println!("Best route: {} -> {}", best_route.from_chain, best_route.to_chain);
println!("Expected output: {}", best_route.to_amount);
println!("Estimated duration: {}s", best_route.estimated_duration);
if let Some(fees) = best_route.fees_usd {
println!("Total fees: ${:.2}", fees);
}
}
# Ok(())
# }
```

---

### get_supported_chains

**Source**: `src/bridge.rs`

**Attributes**:
```rust
#[tool]
```

```rust
pub async fn get_supported_chains() -> Result<Vec<ChainInfo>, ToolError>
```

Get a list of supported blockchain networks for cross-chain operations

This tool returns comprehensive information about all blockchain networks supported by
the cross-chain bridge infrastructure. Essential for discovering available chains
and their native tokens before initiating bridge operations.

# Returns

Returns `Vec<ChainInfo>` where each chain contains:
- `id`: Numeric chain identifier used by bridge protocols
- `name`: Human-readable chain name (e.g., "Ethereum", "Polygon")
- `key`: Short identifier key (e.g., "eth", "pol")
- `chain_type`: Blockchain type ("evm" or "solana")
- `native_token`: Information about the chain's native currency
- `logo_uri`: Optional logo image URL

# Errors

* `CrossChainToolError::NetworkError` - When API connection fails

# Examples

```rust,ignore
use riglr_cross_chain_tools::bridge::get_supported_chains;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let chains = get_supported_chains().await?;

println!("Supported chains ({}):", chains.len());
for chain in chains {
println!("  {} (ID: {}, Type: {})",
chain.name, chain.id, chain.chain_type);
println!("    Native token: {} ({})",
chain.native_token.name, chain.native_token.symbol);
if let Some(logo) = chain.logo_uri {
println!("    Logo: {}", logo);
}
}
# Ok(())
# }
```

---

## Functions (lifi)

### chain_id_to_name

**Source**: `src/lifi.rs`

```rust
pub fn chain_id_to_name(id: u64) -> Result<String, LiFiError>
```

Helper function to convert chain ID to chain name

---

### chain_name_to_id

**Source**: `src/lifi.rs`

```rust
pub fn chain_name_to_id(name: &str) -> Result<u64, LiFiError>
```

Helper function to convert chain name to chain ID

---

### get_bridge_status

**Source**: `src/lifi.rs`

```rust
pub async fn get_bridge_status( &self, bridge_id: &str, tx_hash: &str, ) -> Result<BridgeStatusResponse, LiFiError>
```

Get the status of a bridge transaction

---

### get_chains

**Source**: `src/lifi.rs`

```rust
pub async fn get_chains(&self) -> Result<Vec<Chain>, LiFiError>
```

Get available chains from LiFi

---

### get_route_with_transaction

**Source**: `src/lifi.rs`

```rust
pub async fn get_route_with_transaction( &self, request: &RouteRequest, ) -> Result<Vec<CrossChainRoute>, LiFiError>
```

Get a route with transaction request for bridge execution
This method gets routes and includes the transaction data needed for execution

---

### get_routes

**Source**: `src/lifi.rs`

```rust
pub async fn get_routes( &self, request: &RouteRequest, ) -> Result<Vec<CrossChainRoute>, LiFiError>
```

Get cross-chain routes for a given request

---

### get_transaction_request_for_route

**Source**: `src/lifi.rs`

```rust
pub async fn get_transaction_request_for_route( &self, route_id: &str, ) -> Result<TransactionRequest, LiFiError>
```

Get transaction request data for a specific route

---

### prepare_bridge_execution

**Source**: `src/lifi.rs`

```rust
pub async fn prepare_bridge_execution( &self, route: &CrossChainRoute, ) -> Result<TransactionRequest, LiFiError>
```

Execute a cross-chain bridge transaction (requires integration with wallet/signer)
This method prepares the transaction data but requires external signing

---

### with_api_key

**Source**: `src/lifi.rs`

```rust
pub fn with_api_key(mut self, api_key: String) -> Self
```

Set an API key for authenticated requests (optional)

---

### with_base_url

**Source**: `src/lifi.rs`

```rust
pub fn with_base_url(base_url: &str) -> Result<Self, LiFiError>
```

Create a new LiFi client with custom base URL

---

## Constants

### VERSION

**Source**: `src/lib.rs`

```rust
const VERSION: &str
```

Version information for the riglr-cross-chain-tools crate.

---


---

*This documentation was automatically generated from the source code.*