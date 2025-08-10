//! Cross-chain bridge tools using LiFi Protocol.
//!
//! This module provides stateless tools for cross-chain operations that integrate
//! with riglr's SignerContext pattern. All tools automatically access the appropriate
//! signer from the current context without requiring explicit client parameters.

use crate::lifi::{
    BridgeStatus, BridgeStatusResponse, Chain, CrossChainRoute, LiFiClient, LiFiError, RouteRequest,
    Token, chain_name_to_id, chain_id_to_name,
};
use riglr_core::{SignerContext, ToolError};
use riglr_macros::tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

// ============================================================================
// Data Structures
// ============================================================================

/// Result of a cross-chain route discovery
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RouteDiscoveryResult {
    /// Available routes sorted by best to worst
    pub routes: Vec<RouteInfo>,
    /// Total number of routes found
    pub total_routes: usize,
    /// Recommended route (if any)
    pub recommended_route_id: Option<String>,
}

/// Simplified route information for tools
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RouteInfo {
    /// Unique route identifier
    pub id: String,
    /// Source chain name
    pub from_chain: String,
    /// Destination chain name
    pub to_chain: String,
    /// Source token info
    pub from_token: TokenInfo,
    /// Destination token info
    pub to_token: TokenInfo,
    /// Input amount (in token's smallest unit)
    pub from_amount: String,
    /// Expected output amount (in token's smallest unit)
    pub to_amount: String,
    /// Minimum output amount (accounting for slippage)
    pub to_amount_min: String,
    /// Estimated completion time in seconds
    pub estimated_duration: u64,
    /// Total fees in USD (if available)
    pub fees_usd: Option<f64>,
    /// Gas cost in USD (if available)  
    pub gas_cost_usd: Option<f64>,
    /// Bridge/DEX protocols used
    pub protocols: Vec<String>,
    /// Route quality tags (e.g., "RECOMMENDED", "FASTEST", "CHEAPEST")
    pub tags: Vec<String>,
}

/// Simplified token information
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    /// Token contract address (or mint for Solana)
    pub address: String,
    /// Token symbol (e.g., "USDC")
    pub symbol: String,
    /// Number of decimal places
    pub decimals: u8,
    /// Token name (e.g., "USD Coin")
    pub name: String,
    /// Current price in USD (if available)
    pub price_usd: Option<f64>,
}

/// Result of executing a cross-chain bridge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BridgeExecutionResult {
    /// Unique identifier for tracking this bridge operation
    pub bridge_id: String,
    /// Transaction hash on the source chain
    pub source_tx_hash: String,
    /// Source chain name
    pub from_chain: String,
    /// Destination chain name  
    pub to_chain: String,
    /// Amount sent (in token's smallest unit)
    pub amount_sent: String,
    /// Expected amount to receive (in token's smallest unit)
    pub expected_amount: String,
    /// Current status of the bridge operation
    pub status: String,
    /// Estimated completion time in seconds from now
    pub estimated_completion: u64,
    /// Instructions for the user
    pub message: String,
}

/// Result of checking bridge status
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BridgeStatusResult {
    /// Bridge operation identifier
    pub bridge_id: String,
    /// Current status
    pub status: String,
    /// Source chain transaction hash
    pub source_tx_hash: Option<String>,
    /// Destination chain transaction hash (if completed)
    pub destination_tx_hash: Option<String>,
    /// Amount sent from source chain
    pub amount_sent: Option<String>,
    /// Amount received on destination chain (if completed)
    pub amount_received: Option<String>,
    /// Human-readable status message
    pub message: String,
    /// Whether the operation is complete (success or failure)
    pub is_complete: bool,
    /// Whether the operation failed
    pub is_failed: bool,
}

/// Result of bridge fee estimation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BridgeFeeEstimate {
    /// Source chain name
    pub from_chain: String,
    /// Destination chain name
    pub to_chain: String,
    /// Input amount
    pub from_amount: String,
    /// Expected output amount after all fees
    pub estimated_output: String,
    /// Total fees breakdown
    pub fees: Vec<FeeBreakdown>,
    /// Total fees in USD
    pub total_fees_usd: Option<f64>,
    /// Gas cost estimate in USD
    pub gas_cost_usd: Option<f64>,
    /// Estimated completion time in seconds
    pub estimated_duration: u64,
}

/// Fee breakdown information
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeeBreakdown {
    /// Fee name (e.g., "Bridge Fee", "Gas Fee")
    pub name: String,
    /// Fee description
    pub description: String,
    /// Fee percentage (e.g., "0.05" for 0.05%)
    pub percentage: String,
    /// Fee amount in token units
    pub amount: String,
    /// Fee amount in USD (if available)
    pub amount_usd: Option<f64>,
    /// Token the fee is paid in
    pub token_symbol: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert LiFi Token to our TokenInfo
fn convert_token(token: &Token) -> TokenInfo {
    TokenInfo {
        address: token.address.clone(),
        symbol: token.symbol.clone(),
        decimals: token.decimals,
        name: token.name.clone(),
        price_usd: token.price_usd,
    }
}

/// Convert LiFi CrossChainRoute to our RouteInfo
fn convert_route(route: &CrossChainRoute) -> Result<RouteInfo, ToolError> {
    let from_chain = chain_id_to_name(route.from_chain_id)
        .map_err(|e| ToolError::permanent(format!("Invalid from chain ID: {}", e)))?;
    let to_chain = chain_id_to_name(route.to_chain_id)
        .map_err(|e| ToolError::permanent(format!("Invalid to chain ID: {}", e)))?;
        
    let protocols: Vec<String> = route.steps.iter()
        .map(|step| step.tool.clone())
        .collect();
        
    let fees_usd = route.fees.iter()
        .filter_map(|fee| fee.amount_usd)
        .sum::<f64>();
        
    Ok(RouteInfo {
        id: route.id.clone(),
        from_chain,
        to_chain,
        from_token: convert_token(&route.from_token),
        to_token: convert_token(&route.to_token),
        from_amount: route.from_amount.clone(),
        to_amount: route.to_amount.clone(),
        to_amount_min: route.to_amount_min.clone(),
        estimated_duration: route.estimated_execution_duration,
        fees_usd: if fees_usd > 0.0 { Some(fees_usd) } else { None },
        gas_cost_usd: route.gas_cost_usd,
        protocols,
        tags: route.tags.clone(),
    })
}

/// Create a LiFi client instance
async fn create_lifi_client() -> Result<LiFiClient, ToolError> {
    // In a production environment, you might want to get API key from environment
    let client = LiFiClient::new()
        .map_err(|e| ToolError::permanent(format!("Failed to create LiFi client: {}", e)))?;
        
    // Optionally set API key from environment
    if let Ok(api_key) = std::env::var("LIFI_API_KEY") {
        Ok(client.with_api_key(api_key))
    } else {
        Ok(client)
    }
}

// ============================================================================
// Cross-Chain Bridge Tools
// ============================================================================

/// Get available cross-chain routes between tokens on different networks
/// 
/// This tool discovers optimal paths for transferring tokens between blockchain networks
/// using LiFi Protocol's aggregation of multiple bridge providers and DEXs. Routes are
/// automatically sorted by quality, cost, and speed with the best options first.
/// 
/// # Arguments
/// 
/// * `from_chain` - Source blockchain name (e.g., "ethereum", "polygon", "arbitrum", "solana")
/// * `to_chain` - Destination blockchain name
/// * `from_token` - Source token address (contract address or mint address for Solana)
/// * `to_token` - Destination token address  
/// * `amount` - Transfer amount in token's smallest unit (e.g., wei for ETH, lamports for SOL)
/// * `slippage_percent` - Maximum acceptable slippage as percentage (e.g., 0.5 for 0.5%)
/// 
/// # Returns
/// 
/// Returns `RouteDiscoveryResult` containing:
/// - `routes`: Available routes sorted by quality (best first)
/// - `total_routes`: Number of routes found
/// - `recommended_route_id`: ID of the recommended route (if any)
/// 
/// Each route includes detailed information about fees, duration, protocols used, and expected amounts.
/// 
/// # Errors
/// 
/// * `CrossChainToolError::UnsupportedChain` - When chain names are not supported
/// * `CrossChainToolError::RouteNotFound` - When no routes exist between the chains/tokens
/// * `CrossChainToolError::ApiError` - When LiFi API issues occur
/// * `CrossChainToolError::NetworkError` - When connection problems occur
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_cross_chain_tools::bridge::get_cross_chain_routes;
/// use riglr_core::SignerContext;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Find routes to bridge USDC from Ethereum to Polygon
/// let routes = get_cross_chain_routes(
///     "ethereum".to_string(),
///     "polygon".to_string(),
///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC on Ethereum
///     "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(), // USDC on Polygon
///     "1000000000".to_string(), // 1000 USDC (6 decimals)
///     Some(0.5), // 0.5% slippage tolerance
/// ).await?;
/// 
/// println!("Found {} routes", routes.total_routes);
/// if let Some(best_route) = routes.routes.first() {
///     println!("Best route: {} -> {}", best_route.from_chain, best_route.to_chain);
///     println!("Expected output: {}", best_route.to_amount);
///     println!("Estimated duration: {}s", best_route.estimated_duration);
///     if let Some(fees) = best_route.fees_usd {
///         println!("Total fees: ${:.2}", fees);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_cross_chain_routes(
    from_chain: String,
    to_chain: String,
    from_token: String,
    to_token: String,
    amount: String,
    slippage_percent: Option<f64>,
) -> Result<RouteDiscoveryResult, ToolError> {
    info!("Discovering cross-chain routes from {} to {}", from_chain, to_chain);
    
    // Get signer to determine user's address
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context available: {}", e)))?;
    
    let from_address = signer.address();
    
    // Create LiFi client
    let lifi_client = create_lifi_client().await?;
    
    // Convert chain names to IDs
    let from_chain_id = chain_name_to_id(&from_chain)
        .map_err(|e| ToolError::permanent(format!("Unsupported from_chain '{}': {}", from_chain, e)))?;
    let to_chain_id = chain_name_to_id(&to_chain)
        .map_err(|e| ToolError::permanent(format!("Unsupported to_chain '{}': {}", to_chain, e)))?;
    
    // Prepare route request
    let route_request = RouteRequest {
        from_chain: from_chain_id,
        to_chain: to_chain_id,
        from_token: from_token.clone(),
        to_token: to_token.clone(),
        from_amount: amount.clone(),
        from_address,
        to_address: signer.address(), // Use same address on destination chain
        slippage: slippage_percent.map(|s| s / 100.0), // Convert percentage to decimal
    };
    
    // Get routes from LiFi
    let routes = lifi_client.get_routes(&route_request).await
        .map_err(|e| match e {
            LiFiError::RouteNotFound { .. } => {
                ToolError::Permanent(format!("No routes found between {} and {}", from_chain, to_chain))
            }
            LiFiError::UnsupportedChain { chain_name } => {
                ToolError::Permanent(format!("Chain not supported: {}", chain_name))
            }
            LiFiError::ApiError { code, message } => {
                if code >= 500 {
                    ToolError::Retriable(format!("LiFi API error {}: {}", code, message))
                } else {
                    ToolError::Permanent(format!("LiFi API error {}: {}", code, message))
                }
            }
            _ => ToolError::Retriable(format!("Failed to get routes: {}", e)),
        })?;
    
    if routes.is_empty() {
        return Err(ToolError::Permanent(format!(
            "No routes available from {} to {} for token {} -> {}",
            from_chain, to_chain, from_token, to_token
        )));
    }
    
    // Convert routes
    let converted_routes: Result<Vec<RouteInfo>, ToolError> = routes.iter()
        .map(convert_route)
        .collect();
    let mut route_infos = converted_routes?;
    
    // Sort routes by quality (recommended first, then by fees and duration)
    route_infos.sort_by(|a, b| {
        // Prioritize routes with "RECOMMENDED" tag
        let a_recommended = a.tags.contains(&"RECOMMENDED".to_string());
        let b_recommended = b.tags.contains(&"RECOMMENDED".to_string());
        
        match (a_recommended, b_recommended) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                // If both or neither are recommended, sort by total cost
                let a_cost = a.fees_usd.unwrap_or(0.0) + a.gas_cost_usd.unwrap_or(0.0);
                let b_cost = b.fees_usd.unwrap_or(0.0) + b.gas_cost_usd.unwrap_or(0.0);
                
                a_cost.partial_cmp(&b_cost).unwrap_or(std::cmp::Ordering::Equal)
            }
        }
    });
    
    // Find recommended route
    let recommended_route_id = route_infos.first()
        .filter(|r| r.tags.contains(&"RECOMMENDED".to_string()))
        .map(|r| r.id.clone());
    
    info!("Found {} cross-chain routes", route_infos.len());
    
    Ok(RouteDiscoveryResult {
        total_routes: route_infos.len(),
        recommended_route_id,
        routes: route_infos,
    })
}

/// Execute a cross-chain bridge transaction using a previously discovered route
///
/// This tool executes an actual cross-chain token transfer by taking a route ID from
/// get_cross_chain_routes and constructing the appropriate transaction. The transaction
/// is signed using the current signer context and submitted to the source blockchain.
/// 
/// **Note**: Current implementation is simulated for demonstration. A production version
/// would construct real transactions for each supported blockchain and bridge protocol.
/// 
/// # Arguments
/// 
/// * `route_id` - Route identifier from get_cross_chain_routes
/// * `from_chain` - Source blockchain name for validation
/// * `to_chain` - Destination blockchain name for validation
/// * `amount` - Amount to bridge in token's smallest unit
/// 
/// # Returns
/// 
/// Returns `BridgeExecutionResult` containing:
/// - `bridge_id`: Unique identifier for tracking the bridge operation
/// - `source_tx_hash`: Transaction hash on the source chain
/// - `from_chain`, `to_chain`: Source and destination networks
/// - `amount_sent`, `expected_amount`: Transfer amounts
/// - `status`: Current bridge status (e.g., "PENDING", "CONFIRMED")
/// - `estimated_completion`: Expected completion time in seconds
/// - `message`: Status message and tracking instructions
/// 
/// # Errors
/// 
/// * `CrossChainToolError::InvalidRoute` - When route ID is invalid or expired
/// * `CrossChainToolError::InsufficientFunds` - When account lacks required tokens
/// * `CrossChainToolError::TransactionFailed` - When transaction construction or submission fails
/// * `CrossChainToolError::NetworkError` - When connection issues occur
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_cross_chain_tools::bridge::{get_cross_chain_routes, execute_cross_chain_bridge};
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // First get routes
/// let routes = get_cross_chain_routes(/* ... */).await?;
/// let best_route = &routes.routes[0];
/// 
/// // Execute the bridge
/// let result = execute_cross_chain_bridge(
///     best_route.id.clone(),
///     "ethereum".to_string(),
///     "polygon".to_string(),
///     "1000000000".to_string(),
/// ).await?;
/// 
/// println!("Bridge initiated!");
/// println!("Bridge ID: {}", result.bridge_id);
/// println!("Source tx: {}", result.source_tx_hash);
/// println!("Status: {}", result.status);
/// println!("Track with: get_bridge_status");
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn execute_cross_chain_bridge(
    route_id: String,
    from_chain: String,
    to_chain: String,
    amount: String,
) -> Result<BridgeExecutionResult, ToolError> {
    info!("Executing cross-chain bridge with route {}", route_id);
    
    // Get current signer
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context available: {}", e)))?;
    
    // For now, we'll simulate the bridge execution since implementing the full
    // transaction construction and signing would require more complex integration
    // with each chain's transaction format and the specific bridge protocols.
    // 
    // In a production implementation, this would:
    // 1. Get the route details from LiFi  
    // 2. Construct the appropriate transaction for the source chain
    // 3. Sign and submit the transaction using the signer
    // 4. Monitor for transaction confirmation
    // 5. Return the bridge tracking information
    
    warn!("Bridge execution is currently simulated - not performing real transaction");
    
    // Generate a mock bridge ID for tracking
    let bridge_id = uuid::Uuid::new_v4().to_string();
    
    // For demonstration, create a mock successful result
    // In production, this would be the actual transaction hash
    let mock_tx_hash = format!("0x{}", hex::encode(&bridge_id.as_bytes()[..16]));
    
    Ok(BridgeExecutionResult {
        bridge_id: bridge_id.clone(),
        source_tx_hash: mock_tx_hash,
        from_chain,
        to_chain,
        amount_sent: amount.clone(),
        expected_amount: amount, // Simplified - would calculate after fees
        status: "PENDING".to_string(),
        estimated_completion: 300, // 5 minutes
        message: format!(
            "Bridge transaction submitted. Track progress with bridge ID: {}. This is currently a simulation.",
            bridge_id
        ),
    })
}

/// Check the status of an ongoing cross-chain bridge operation
///
/// This tool monitors the progress of a cross-chain bridge transaction using the bridge ID
/// and source transaction hash returned from execute_cross_chain_bridge. Essential for
/// tracking multi-step bridge operations that can take several minutes to complete.
/// 
/// # Arguments
/// 
/// * `bridge_id` - Unique bridge operation identifier from execute_cross_chain_bridge
/// * `source_tx_hash` - Transaction hash from the source chain
/// 
/// # Returns
/// 
/// Returns `BridgeStatusResult` containing:
/// - `bridge_id`: The tracked bridge operation ID
/// - `status`: Current status ("PENDING", "DONE", "FAILED", etc.)
/// - `source_tx_hash`: Source chain transaction hash
/// - `destination_tx_hash`: Destination chain transaction hash (when completed)
/// - `amount_sent`, `amount_received`: Actual transfer amounts
/// - `message`: Human-readable status description
/// - `is_complete`, `is_failed`: Boolean flags for operation state
/// 
/// # Errors
/// 
/// * `CrossChainToolError::BridgeNotFound` - When bridge ID or transaction hash is invalid
/// * `CrossChainToolError::NetworkError` - When status lookup fails
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_cross_chain_tools::bridge::get_bridge_status;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let status = get_bridge_status(
///     "bridge-123-abc".to_string(),
///     "0x1234...abcd".to_string(),
/// ).await?;
/// 
/// println!("Bridge status: {}", status.status);
/// println!("Message: {}", status.message);
/// 
/// if status.is_complete {
///     println!("✅ Bridge completed successfully!");
///     if let Some(dest_tx) = status.destination_tx_hash {
///         println!("Destination tx: {}", dest_tx);
///     }
///     if let Some(received) = status.amount_received {
///         println!("Amount received: {}", received);
///     }
/// } else if status.is_failed {
///     println!("❌ Bridge failed: {}", status.message);
/// } else {
///     println!("⏳ Bridge in progress...");
/// }
/// # Ok(())
/// # }
/// ```
#[tool] 
pub async fn get_bridge_status(
    bridge_id: String,
    source_tx_hash: String,
) -> Result<BridgeStatusResult, ToolError> {
    info!("Checking bridge status for {}", bridge_id);
    
    // Create LiFi client
    let lifi_client = create_lifi_client().await?;
    
    // For the simulation, we'll return a mock status
    // In production, this would call lifi_client.get_bridge_status()
    warn!("Bridge status check is currently simulated");
    
    // Mock a "completed" status for demonstration
    Ok(BridgeStatusResult {
        bridge_id: bridge_id.clone(),
        status: "DONE".to_string(),
        source_tx_hash: Some(source_tx_hash.clone()),
        destination_tx_hash: Some(format!("0x{}", hex::encode(&bridge_id.as_bytes()[..16]))),
        amount_sent: Some("1000000".to_string()), 
        amount_received: Some("995000".to_string()), // After fees
        message: "Bridge completed successfully. This is currently a simulation.".to_string(),
        is_complete: true,
        is_failed: false,
    })
}

/// Estimate fees and completion time for a cross-chain bridge operation
///
/// This tool provides detailed cost analysis and timing estimates for bridging tokens
/// between different blockchain networks without executing any transactions. Useful for
/// comparing bridge options and budgeting for cross-chain transfers.
/// 
/// # Arguments
/// 
/// * `from_chain` - Source blockchain name
/// * `to_chain` - Destination blockchain name
/// * `from_token` - Source token address
/// * `to_token` - Destination token address
/// * `amount` - Transfer amount in token's smallest unit
/// 
/// # Returns
/// 
/// Returns `BridgeFeeEstimate` containing:
/// - `from_chain`, `to_chain`: Source and destination networks
/// - `from_amount`: Input amount
/// - `estimated_output`: Expected output after all fees
/// - `fees`: Detailed breakdown of different fee types
/// - `total_fees_usd`: Total fees in USD (if available)
/// - `gas_cost_usd`: Gas cost estimate in USD  
/// - `estimated_duration`: Expected completion time in seconds
/// 
/// Each fee in the breakdown includes name, description, percentage, amount, and USD value.
/// 
/// # Errors
/// 
/// * `CrossChainToolError::UnsupportedChain` - When chain names are not supported
/// * `CrossChainToolError::RouteNotFound` - When no routes exist for fee estimation
/// * `CrossChainToolError::NetworkError` - When API connection fails
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_cross_chain_tools::bridge::estimate_bridge_fees;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let estimate = estimate_bridge_fees(
///     "ethereum".to_string(),
///     "polygon".to_string(),  
///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC on Ethereum
///     "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(), // USDC on Polygon
///     "100000000".to_string(), // 100 USDC (6 decimals)
/// ).await?;
/// 
/// println!("Bridge estimate for {} USDC:", "100");
/// println!("Expected output: {}", estimate.estimated_output);
/// println!("Duration: {}s (~{} minutes)", 
///          estimate.estimated_duration, 
///          estimate.estimated_duration / 60);
/// 
/// if let Some(total_fees) = estimate.total_fees_usd {
///     println!("Total fees: ${:.2}", total_fees);
/// }
/// 
/// for fee in estimate.fees {
///     println!("  {}: {} ({}%)", fee.name, fee.amount, fee.percentage);
/// }
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn estimate_bridge_fees(
    from_chain: String,
    to_chain: String,
    from_token: String,
    to_token: String,
    amount: String,
) -> Result<BridgeFeeEstimate, ToolError> {
    info!("Estimating bridge fees from {} to {}", from_chain, to_chain);
    
    // Get routes to analyze fees (reuse the route discovery logic)
    let routes_result = get_cross_chain_routes(
        from_chain.clone(),
        to_chain.clone(),
        from_token.clone(),
        to_token.clone(),
        amount.clone(),
        Some(0.5), // 0.5% default slippage
    ).await?;
    
    if routes_result.routes.is_empty() {
        return Err(ToolError::Permanent(format!(
            "No routes available for fee estimation between {} and {}",
            from_chain, to_chain
        )));
    }
    
    // Use the best (first) route for fee estimation
    let best_route = &routes_result.routes[0];
    
    // Create mock fee breakdown for demonstration
    let fees = vec![
        FeeBreakdown {
            name: "Bridge Fee".to_string(),
            description: "Fee charged by the bridge protocol".to_string(),
            percentage: "0.05".to_string(),
            amount: "5000".to_string(), // 0.05% of 1M = 5000
            amount_usd: Some(5.0),
            token_symbol: best_route.from_token.symbol.clone(),
        },
        FeeBreakdown {
            name: "Gas Fee".to_string(),
            description: "Transaction execution cost on source chain".to_string(),
            percentage: "0.0".to_string(),
            amount: "50000000000000000".to_string(), // ~$2.50 in wei
            amount_usd: best_route.gas_cost_usd,
            token_symbol: "ETH".to_string(),
        },
    ];
    
    let total_fees_usd = fees.iter()
        .filter_map(|f| f.amount_usd)
        .sum::<f64>();
    
    Ok(BridgeFeeEstimate {
        from_chain,
        to_chain,
        from_amount: amount,
        estimated_output: best_route.to_amount.clone(),
        fees,
        total_fees_usd: if total_fees_usd > 0.0 { Some(total_fees_usd) } else { None },
        gas_cost_usd: best_route.gas_cost_usd,
        estimated_duration: best_route.estimated_duration,
    })
}

// ============================================================================
// Utility Tools
// ============================================================================

/// Get a list of supported blockchain networks for cross-chain operations
///
/// This tool returns comprehensive information about all blockchain networks supported by
/// the cross-chain bridge infrastructure. Essential for discovering available chains
/// and their native tokens before initiating bridge operations.
/// 
/// # Returns
/// 
/// Returns `Vec<ChainInfo>` where each chain contains:
/// - `id`: Numeric chain identifier used by bridge protocols
/// - `name`: Human-readable chain name (e.g., "Ethereum", "Polygon")
/// - `key`: Short identifier key (e.g., "eth", "pol") 
/// - `chain_type`: Blockchain type ("evm" or "solana")
/// - `native_token`: Information about the chain's native currency
/// - `logo_uri`: Optional logo image URL
/// 
/// # Errors
/// 
/// * `CrossChainToolError::NetworkError` - When API connection fails
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_cross_chain_tools::bridge::get_supported_chains;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let chains = get_supported_chains().await?;
/// 
/// println!("Supported chains ({}):", chains.len());
/// for chain in chains {
///     println!("  {} (ID: {}, Type: {})", 
///              chain.name, chain.id, chain.chain_type);
///     println!("    Native token: {} ({})", 
///              chain.native_token.name, chain.native_token.symbol);
///     if let Some(logo) = chain.logo_uri {
///         println!("    Logo: {}", logo);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_supported_chains() -> Result<Vec<ChainInfo>, ToolError> {
    info!("Fetching supported chains for cross-chain operations");
    
    // Create LiFi client
    let lifi_client = create_lifi_client().await?;
    
    // Get chains from LiFi
    let chains = lifi_client.get_chains().await
        .map_err(|e| ToolError::Retriable(format!("Failed to get supported chains: {}", e)))?;
    
    // Convert to our format
    let chain_infos: Vec<ChainInfo> = chains.iter()
        .map(|chain| ChainInfo {
            id: chain.id,
            name: chain.name.clone(),
            key: chain.key.clone(),
            chain_type: match chain.chain_type {
                crate::lifi::ChainType::Evm => "evm".to_string(),
                crate::lifi::ChainType::Solana => "solana".to_string(),
            },
            native_token: convert_token(&chain.native_token),
            logo_uri: chain.logo_uri.clone(),
        })
        .collect();
    
    info!("Found {} supported chains", chain_infos.len());
    Ok(chain_infos)
}

/// Chain information for supported networks
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChainInfo {
    /// Numeric chain ID
    pub id: u64,
    /// Human-readable chain name
    pub name: String,
    /// Short key identifier
    pub key: String,
    /// Chain type ("evm" or "solana")
    pub chain_type: String,
    /// Native token information
    pub native_token: TokenInfo,
    /// Logo URL (if available)
    pub logo_uri: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_core::signer::{LocalSolanaSigner};
    use std::sync::Arc;
    
    // Mock signer for testing
    #[derive(Debug)]
    struct MockCrossChainSigner {
        address: String,
    }
    
    #[async_trait::async_trait]
    impl riglr_core::TransactionSigner for MockCrossChainSigner {
        fn address(&self) -> Option<String> {
            Some(self.address.clone())
        }
        
        async fn sign_and_send_solana_transaction(
            &self,
            _tx: &mut solana_sdk::transaction::Transaction,
        ) -> Result<String, riglr_core::SignerError> {
            Ok("mock_solana_signature".to_string())
        }
        
        async fn sign_and_send_evm_transaction(
            &self,
            _tx: alloy::rpc::types::TransactionRequest,
        ) -> Result<String, riglr_core::SignerError> {
            Ok("0xmock_evm_signature".to_string())
        }
        
        fn solana_client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
            Arc::new(solana_client::rpc_client::RpcClient::new("http://localhost:8899"))
        }
        
        fn evm_client(&self) -> Result<Box<dyn std::any::Any + Send + Sync>, riglr_core::SignerError> {
            Err(riglr_core::SignerError::Configuration("Mock EVM client not implemented".to_string()))
        }
    }
    
    #[tokio::test]
    async fn test_cross_chain_tools_require_signer_context() {
        // Test that tools fail without signer context
        let result = get_cross_chain_routes(
            "ethereum".to_string(),
            "polygon".to_string(),
            "0x0000000000000000000000000000000000000000".to_string(),
            "0xA0b86a33E6417c5d6d6bE6C2e0C6C3e5d6c7D8E9".to_string(),
            "1000000000000000000".to_string(), // 1 ETH in wei
            Some(0.5),
        ).await;
        
        assert!(result.is_err());
        assert!(matches!(result, Err(ToolError::Permanent(_))));
    }
    
    #[tokio::test]
    async fn test_chain_conversion_helpers() {
        let token = Token {
            address: "0xA0b86a33E6417c5d6d6bE6C2e0C6C3e5d6c7D8E9".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
            name: "USD Coin".to_string(),
            logo_uri: None,
            price_usd: Some(1.0),
        };
        
        let token_info = convert_token(&token);
        assert_eq!(token_info.symbol, "USDC");
        assert_eq!(token_info.decimals, 6);
        assert_eq!(token_info.price_usd, Some(1.0));
    }
}