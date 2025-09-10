# riglr-cross-chain-tools

{{#include ../../../riglr-cross-chain-tools/README.md}}

## API Reference

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Functions](#functions)
- [Constants](#constants)

### Structs

> Core data structures and types.

#### `Args`

Arguments structure for the tool

---

#### `BridgeExecutionResult`

Result of executing a cross-chain bridge

---

#### `BridgeFeeEstimate`

Result of bridge fee estimation

---

#### `BridgeStatusResponse`

Bridge transaction status response

---

#### `BridgeStatusResult`

Result of checking bridge status

---

#### `Chain`

Chain information from LiFi

---

#### `ChainInfo`

Chain information for supported networks

---

#### `CrossChainRoute`

A cross-chain route option from LiFi

---

#### `FeeBreakdown`

Fee breakdown information

---

#### `LiFiClient`

LiFi Protocol API client

---

#### `RouteDiscoveryResult`

Result of a cross-chain route discovery

---

#### `RouteFee`

Fee information for a route

---

#### `RouteInfo`

Simplified route information for tools

---

#### `RouteRequest`

Request parameters for getting cross-chain routes

---

#### `RouteResponse`

Response from the routes API

---

#### `RouteStep`

A step within a cross-chain route

---

#### `SolanaAccountMeta`

Solana account metadata for building instructions

---

#### `StepAction`

Action details for a route step

---

#### `StepEstimate`

Execution estimate for a step

---

#### `Token`

Token information

---

#### `TokenInfo`

Simplified token information

---

#### `Tool`

Tool implementation structure

---

#### `TransactionRequest`

Transaction request data for executing cross-chain bridges

---

### Enums

> Enumeration types for representing variants.

#### `BridgeStatus`

Bridge transaction status

**Variants:**

- `NotFound`
  - Transaction not found
- `Pending`
  - Transaction is pending execution
- `Done`
  - Transaction completed successfully
- `Failed`
  - Transaction failed

---

#### `ChainType`

Supported blockchain networks

**Variants:**

- `Evm`
  - Ethereum Virtual Machine based blockchain
- `Solana`
  - Solana blockchain

---

#### `CrossChainError`

Errors that can occur during cross-chain operations

**Variants:**

- `ToolError`
  - Core tool error
- `LifiApiError`
  - Li.fi API error
- `QuoteFetchError`
  - Quote fetch failed
- `InvalidRoute`
  - Invalid route configuration
- `BridgeExecutionError`
  - Bridge operation failed
- `UnsupportedChainPair`
  - Unsupported chain pair
- `InsufficientLiquidity`
  - Insufficient liquidity for amount

---

#### `LiFiError`

Errors that can occur during LiFi API operations

**Variants:**

- `Request`
  - HTTP request failed
- `InvalidResponse`
  - Invalid response format from API
- `ApiError`
  - API returned an error response
- `UnsupportedChain`
  - Chain is not supported by LiFi
- `RouteNotFound`
  - No route found between chains
- `Configuration`
  - Configuration error
- `UrlParse`
  - URL parsing error

---

### Functions

> Standalone functions and utilities.

#### `chain_id_to_name`

Helper function to convert chain ID to chain name

---

#### `chain_name_to_id`

Helper function to convert chain name to chain ID

---

#### `estimate_bridge_fees`

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

#### `estimate_bridge_fees_tool`

Factory function to create a new instance of the tool

---

#### `execute_cross_chain_bridge`

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

#### `execute_cross_chain_bridge_tool`

Factory function to create a new instance of the tool

---

#### `get_bridge_status`

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

#### `get_bridge_status_tool`

Factory function to create a new instance of the tool

---

#### `get_cross_chain_routes`

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

#### `get_cross_chain_routes_tool`

Factory function to create a new instance of the tool

---

#### `get_supported_chains`

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

#### `get_supported_chains_tool`

Factory function to create a new instance of the tool

---

### Constants

#### `VERSION`

Version information for the riglr-cross-chain-tools crate.

**Type:** `&str`

---
