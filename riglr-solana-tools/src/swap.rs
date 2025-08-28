//! Jupiter DEX integration for token swaps on Solana
//!
//! This module provides tools for interacting with the Jupiter aggregator,
//! enabling token swaps with optimal routing across multiple DEXs.

use crate::transaction::TransactionStatus;
use crate::utils::send_transaction;
use riglr_core::provider::ApplicationContext;
use riglr_core::{SignerContext, ToolError};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};
use std::str::FromStr;
use tracing::{debug, info};

/// Get a quote from Jupiter for swapping tokens
///
/// This tool queries the Jupiter aggregator to find the best swap route between two SPL tokens
/// and returns detailed pricing information without executing any transaction. Jupiter aggregates
/// liquidity from multiple DEXs to provide optimal pricing.
///
/// # Arguments
///
/// * `input_mint` - Source token mint address to swap from
/// * `output_mint` - Destination token mint address to swap to
/// * `amount` - Input amount in token's smallest unit (e.g., lamports for SOL)
/// * `slippage_bps` - Maximum acceptable slippage in basis points (e.g., 50 = 0.5%)
/// * `only_direct_routes` - If true, only consider direct swap routes (no intermediate tokens)
/// * `jupiter_api_url` - Optional custom Jupiter API endpoint URL
///
/// # Returns
///
/// Returns `SwapQuote` containing:
/// - `input_mint` and `output_mint`: Token addresses
/// - `in_amount` and `out_amount`: Expected input and output amounts
/// - `other_amount_threshold`: Minimum output after slippage
/// - `price_impact_pct`: Price impact as percentage
/// - `route_plan`: Detailed routing through DEXs
/// - `context_slot` and `time_taken`: Quote freshness metadata
///
/// # Errors
///
/// * `ToolError::Permanent` - When token addresses are invalid or no routes exist
/// * `ToolError::Retriable` - When Jupiter API is temporarily unavailable
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::swap::get_jupiter_quote;
/// use riglr_core::provider::ApplicationContext;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let context = ApplicationContext::from_env();
/// // Get quote for swapping 1 SOL to USDC
/// let quote = get_jupiter_quote(
///     "So11111111111111111111111111111111111111112".to_string(), // SOL mint
///     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
///     1_000_000_000, // 1 SOL in lamports
///     50, // 0.5% slippage
///     false, // Allow multi-hop routes
///     None, // Use default Jupiter API
///     &context,
/// ).await?;
///
/// println!("Quote: {} SOL -> {} USDC", quote.in_amount, quote.out_amount);
/// println!("Price impact: {:.2}%", quote.price_impact_pct);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_jupiter_quote(
    input_mint: String,
    output_mint: String,
    amount: u64,
    slippage_bps: u16,
    only_direct_routes: bool,
    jupiter_api_url: Option<String>,
    context: &ApplicationContext,
) -> Result<SwapQuote, ToolError> {
    debug!(
        "Getting Jupiter quote for {} {} -> {}",
        amount, input_mint, output_mint
    );

    // Validate mint addresses
    let _input_pubkey = Pubkey::from_str(&input_mint)
        .map_err(|e| ToolError::permanent_string(format!("Invalid input mint: {}", e)))?;
    let _output_pubkey = Pubkey::from_str(&output_mint)
        .map_err(|e| ToolError::permanent_string(format!("Invalid output mint: {}", e)))?;

    // Get API clients from context
    let api_clients = context
        .get_extension::<std::sync::Arc<crate::clients::ApiClients>>()
        .ok_or_else(|| ToolError::permanent_string("ApiClients not found in context"))?;

    let api_url =
        jupiter_api_url.unwrap_or_else(|| format!("{}/v6", api_clients.jupiter.api_url()));

    // Build quote request URL
    let mut url = format!("{}/quote", api_url);
    let mut params = vec![
        format!("inputMint={}", input_mint),
        format!("outputMint={}", output_mint),
        format!("amount={}", amount),
        format!("slippageBps={}", slippage_bps),
    ];

    if only_direct_routes {
        params.push("onlyDirectRoutes=true".to_string());
    }

    url = format!("{}?{}", url, params.join("&"));

    debug!("Requesting quote from: {}", url);

    // Make HTTP request to Jupiter API
    let response = api_clients
        .jupiter
        .http_client()
        .get(&url)
        .send()
        .await
        .map_err(|e| ToolError::retriable_string(format!("Failed to request quote: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::permanent_string(format!(
            "Jupiter API error: {}",
            error_text
        )));
    }

    let quote_response: JupiterQuoteResponse = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse quote response: {}", e))
    })?;

    // Calculate price impact
    let price_impact = calculate_price_impact(&quote_response);

    info!(
        "Jupiter quote: {} {} -> {} {} (price impact: {:.2}%)",
        amount,
        input_mint,
        quote_response.out_amount,
        output_mint,
        price_impact * 100.0
    );

    Ok(SwapQuote {
        input_mint,
        output_mint,
        in_amount: quote_response.in_amount,
        out_amount: quote_response.out_amount,
        other_amount_threshold: quote_response.other_amount_threshold,
        price_impact_pct: price_impact * 100.0,
        route_plan: quote_response.route_plan.clone(),
        context_slot: quote_response.context_slot,
        time_taken: quote_response.time_taken,
    })
}

/// Execute a token swap using Jupiter
///
/// This tool executes an actual token swap using the Jupiter aggregator. It automatically
/// gets a fresh quote, constructs the swap transaction, signs it with the current signer context,
/// and submits it to the Solana network. The swap uses optimal routing across multiple DEXs.
///
/// # Arguments
///
/// * `input_mint` - Source token mint address to swap from
/// * `output_mint` - Destination token mint address to swap to
/// * `amount` - Input amount in token's smallest unit
/// * `slippage_bps` - Maximum acceptable slippage in basis points (e.g., 50 = 0.5%)
/// * `jupiter_api_url` - Optional custom Jupiter API endpoint URL
/// * `use_versioned_transaction` - Whether to use versioned transactions (recommended for lower fees)
///
/// # Returns
///
/// Returns `SwapResult` containing:
/// - `signature`: Transaction signature for tracking
/// - `input_mint` and `output_mint`: Token addresses involved
/// - `in_amount` and `out_amount`: Actual swap amounts
/// - `price_impact_pct`: Price impact percentage experienced
/// - `status`: Current transaction status (initially Pending)
///
/// # Errors
///
/// * `ToolError::Permanent` - When addresses invalid, signer unavailable, or swap construction fails
/// * `ToolError::Retriable` - When Jupiter API unavailable or network issues occur
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::swap::perform_jupiter_swap;
/// use riglr_core::SignerContext;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Swap 0.1 SOL to USDC
/// let result = perform_jupiter_swap(
///     "So11111111111111111111111111111111111111112".to_string(), // SOL mint
///     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
///     100_000_000, // 0.1 SOL in lamports
///     100, // 1% slippage tolerance
///     None, // Use default Jupiter API
///     true, // Use versioned transactions
/// ).await?;
///
/// println!("Swap executed! Signature: {}", result.signature);
/// println!("Swapped {} for {} tokens", result.in_amount, result.out_amount);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn perform_jupiter_swap(
    input_mint: String,
    output_mint: String,
    amount: u64,
    slippage_bps: u16,
    jupiter_api_url: Option<String>,
    use_versioned_transaction: bool,
    context: &ApplicationContext,
) -> Result<SwapResult, ToolError> {
    debug!(
        "Executing Jupiter swap: {} {} -> {}",
        amount, input_mint, output_mint
    );

    // Get signer from context
    let signer_context = SignerContext::current_as_solana()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {}", e)))?;

    let signer_pubkey = signer_context.pubkey();

    // Get the API clients from context
    let api_clients = context
        .get_extension::<std::sync::Arc<crate::clients::ApiClients>>()
        .ok_or_else(|| ToolError::permanent_string("ApiClients not found in context"))?;

    let api_url =
        jupiter_api_url.unwrap_or_else(|| format!("{}/v6", api_clients.jupiter.api_url()));

    // First get a quote
    let quote = get_jupiter_quote(
        input_mint.clone(),
        output_mint.clone(),
        amount,
        slippage_bps,
        false,
        Some(api_url.clone()),
        context,
    )
    .await?;

    // Build swap request
    let swap_request = json!({
        "userPublicKey": signer_pubkey,
        "quoteResponse": {
            "inputMint": quote.input_mint,
            "outputMint": quote.output_mint,
            "inAmount": quote.in_amount.to_string(),
            "outAmount": quote.out_amount.to_string(),
            "otherAmountThreshold": quote.other_amount_threshold.to_string(),
            "routePlan": quote.route_plan,
            "contextSlot": quote.context_slot,
        },
        "wrapAndUnwrapSol": true,
        "useSharedAccounts": true,
        "prioritizationFeeLamports": "auto",
        "asLegacyTransaction": !use_versioned_transaction,
    });

    debug!("Requesting swap transaction from Jupiter");

    // Request swap transaction from Jupiter
    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(format!("{}/swap", api_url))
        .json(&swap_request)
        .send()
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to request swap transaction: {}", e))
        })?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::permanent_string(format!(
            "Jupiter swap API error: {}",
            error_text
        )));
    }

    let swap_response: JupiterSwapResponse = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse swap response: {}", e))
    })?;

    // Deserialize and sign the transaction
    use base64::{engine::general_purpose, Engine as _};
    let transaction_bytes = general_purpose::STANDARD
        .decode(&swap_response.swap_transaction)
        .map_err(|e| ToolError::permanent_string(format!("Failed to decode transaction: {}", e)))?;

    let mut transaction: Transaction = bincode::deserialize(&transaction_bytes).map_err(|e| {
        ToolError::permanent_string(format!("Failed to deserialize transaction: {}", e))
    })?;

    // Send transaction with retry logic
    let signature = send_transaction(
        &mut transaction,
        &format!("Jupiter Swap ({} -> {})", input_mint, output_mint),
    )
    .await?;

    info!(
        "Jupiter swap executed: {} {} -> {} {} (expected), signature: {}",
        quote.in_amount, input_mint, quote.out_amount, output_mint, signature
    );

    Ok(SwapResult {
        signature,
        input_mint,
        output_mint,
        in_amount: quote.in_amount,
        out_amount: quote.out_amount,
        price_impact_pct: quote.price_impact_pct,
        status: TransactionStatus::Pending,
        idempotency_key: None,
    })
}

/// Get the current price of a token pair
///
/// This tool fetches the current exchange rate between two SPL tokens by requesting
/// a small test quote from Jupiter. This provides real-time pricing without executing trades.
///
/// # Arguments
///
/// * `base_mint` - Token address to price (the token being quoted)
/// * `quote_mint` - Token address to price against (usually USDC or SOL)
/// * `jupiter_api_url` - Optional custom Jupiter API endpoint URL
///
/// # Returns
///
/// Returns `PriceInfo` containing:
/// - `base_mint` and `quote_mint`: Token addresses used
/// - `price`: Exchange rate (how much quote_mint per 1 base_mint)
/// - `price_impact_pct`: Price impact for small test trade
///
/// # Errors
///
/// * `ToolError::Permanent` - When tokens are invalid or no liquidity exists
/// * `ToolError::Retriable` - When Jupiter API is temporarily unavailable
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::swap::get_token_price;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get SOL price in USDC
/// let price_info = get_token_price(
///     "So11111111111111111111111111111111111111112".to_string(), // SOL mint
///     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
///     None, // Use default Jupiter API
/// ).await?;
///
/// println!("1 SOL = {} USDC", price_info.price);
/// println!("Price impact: {:.3}%", price_info.price_impact_pct);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_token_price(
    base_mint: String,
    quote_mint: String,
    jupiter_api_url: Option<String>,
    context: &ApplicationContext,
) -> Result<PriceInfo, ToolError> {
    debug!("Getting price for {} in terms of {}", base_mint, quote_mint);

    // Validate mint addresses first (for better error messages in tests)
    let _base_pubkey = Pubkey::from_str(&base_mint)
        .map_err(|e| ToolError::permanent_string(format!("Invalid input mint: {}", e)))?;
    let _quote_pubkey = Pubkey::from_str(&quote_mint)
        .map_err(|e| ToolError::permanent_string(format!("Invalid output mint: {}", e)))?;

    // Get the API clients from context
    let api_clients = context
        .get_extension::<std::sync::Arc<crate::clients::ApiClients>>()
        .ok_or_else(|| ToolError::permanent_string("ApiClients not found in context"))?;

    let api_url =
        jupiter_api_url.unwrap_or_else(|| format!("{}/v6", api_clients.jupiter.api_url()));

    // Get a small quote to determine price
    let amount = 1_000_000; // 1 token with 6 decimals
    let quote = get_jupiter_quote(
        base_mint.clone(),
        quote_mint.clone(),
        amount,
        50, // 0.5% slippage
        false,
        Some(api_url),
        context,
    )
    .await?;

    // Calculate price
    let price = quote.out_amount as f64 / quote.in_amount as f64;

    Ok(PriceInfo {
        base_mint,
        quote_mint,
        price,
        price_impact_pct: quote.price_impact_pct,
    })
}

/// Calculate price impact from Jupiter quote response
fn calculate_price_impact(quote: &JupiterQuoteResponse) -> f64 {
    // Jupiter provides price impact in the response
    // This is a simplified calculation
    quote.price_impact_pct.unwrap_or(0.0)
}

/// Jupiter quote response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JupiterQuoteResponse {
    /// Input amount in smallest token units
    pub in_amount: u64,
    /// Output amount in smallest token units
    pub out_amount: u64,
    /// Minimum output amount after slippage
    pub other_amount_threshold: u64,
    /// Detailed routing plan through DEXs
    pub route_plan: Vec<RoutePlanStep>,
    /// Solana slot context for quote freshness
    pub context_slot: Option<u64>,
    /// Time taken to compute the quote in seconds
    pub time_taken: Option<f64>,
    /// Price impact percentage for the swap
    pub price_impact_pct: Option<f64>,
}

/// Jupiter swap response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JupiterSwapResponse {
    /// Base64 encoded transaction ready for signing
    pub swap_transaction: String,
    /// Last valid block height for transaction expiry
    pub last_valid_block_height: u64,
    /// Optional prioritization fee in lamports
    pub prioritization_fee: Option<u64>,
}

/// Route plan step in Jupiter quote
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoutePlanStep {
    /// Detailed swap information for this step
    pub swap_info: SwapInfo,
    /// Percentage of input amount for this route step
    pub percent: u8,
}

/// Swap information for a route step
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SwapInfo {
    /// AMM program public key performing the swap
    pub amm_key: String,
    /// Human-readable DEX name (e.g., "Raydium", "Orca")
    pub label: Option<String>,
    /// Input token mint address for this step
    pub input_mint: String,
    /// Output token mint address for this step
    pub output_mint: String,
    /// Input amount for this swap step
    pub in_amount: String,
    /// Output amount for this swap step
    pub out_amount: String,
    /// Fee amount charged by the DEX
    pub fee_amount: String,
    /// Token mint address for the fee
    pub fee_mint: String,
}

/// Result of a swap quote from Jupiter
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SwapQuote {
    /// Source token mint address
    pub input_mint: String,
    /// Destination token mint address
    pub output_mint: String,
    /// Input amount
    pub in_amount: u64,
    /// Expected output amount
    pub out_amount: u64,
    /// Minimum output amount after slippage
    pub other_amount_threshold: u64,
    /// Price impact percentage
    pub price_impact_pct: f64,
    /// Detailed routing plan
    pub route_plan: Vec<RoutePlanStep>,
    /// Context slot for the quote
    pub context_slot: Option<u64>,
    /// Time taken to compute quote
    pub time_taken: Option<f64>,
}

/// Result of a swap execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SwapResult {
    /// Transaction signature
    pub signature: String,
    /// Source token mint address
    pub input_mint: String,
    /// Destination token mint address
    pub output_mint: String,
    /// Input amount
    pub in_amount: u64,
    /// Expected output amount
    pub out_amount: u64,
    /// Price impact percentage
    pub price_impact_pct: f64,
    /// Transaction status
    pub status: TransactionStatus,
    /// Idempotency key if provided
    pub idempotency_key: Option<String>,
}

/// Token price information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PriceInfo {
    /// Token being priced (the asset)
    pub base_mint: String,
    /// Token used for pricing (usually USDC or SOL)
    pub quote_mint: String,
    /// Price of base in terms of quote
    pub price: f64,
    /// Price impact for small trade
    pub price_impact_pct: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::JUPITER_API_URL;

    // Jupiter API URL tests
    #[test]
    fn test_default_jupiter_api_url() {
        let jupiter_api_url = std::env::var(JUPITER_API_URL)
            .unwrap_or_else(|_| "https://quote-api.jup.ag".to_string());
        assert!(jupiter_api_url.contains("jup.ag") || jupiter_api_url.contains("jupiter"));
    }

    // calculate_price_impact tests
    #[test]
    fn test_calculate_price_impact_with_price_impact() {
        let quote = JupiterQuoteResponse {
            in_amount: 1000000,
            out_amount: 990000,
            other_amount_threshold: 980000,
            route_plan: vec![],
            context_slot: Some(123456),
            time_taken: Some(0.1),
            price_impact_pct: Some(1.5),
        };
        let impact = calculate_price_impact(&quote);
        assert_eq!(impact, 1.5);
    }

    #[test]
    fn test_calculate_price_impact_with_none_returns_zero() {
        let quote = JupiterQuoteResponse {
            in_amount: 1000000,
            out_amount: 990000,
            other_amount_threshold: 980000,
            route_plan: vec![],
            context_slot: None,
            time_taken: None,
            price_impact_pct: None,
        };
        let impact = calculate_price_impact(&quote);
        assert_eq!(impact, 0.0);
    }

    // Serialization/Deserialization tests
    #[test]
    fn test_swap_quote_serialization() {
        let quote = SwapQuote {
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            in_amount: 1000000000,
            out_amount: 50000000,
            other_amount_threshold: 49500000,
            price_impact_pct: 0.5,
            route_plan: vec![],
            context_slot: Some(123456),
            time_taken: Some(0.123),
        };

        let json = serde_json::to_string(&quote).unwrap();
        assert!(json.contains("input_mint"));
        assert!(json.contains("1000000000"));

        let deserialized: SwapQuote = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.input_mint, quote.input_mint);
        assert_eq!(deserialized.out_amount, quote.out_amount);
    }

    #[test]
    fn test_swap_result_serialization() {
        let result = SwapResult {
            signature: "test_signature".to_string(),
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            in_amount: 1000000000,
            out_amount: 50000000,
            price_impact_pct: 0.5,
            status: TransactionStatus::Pending,
            idempotency_key: Some("test_key".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test_signature"));

        let deserialized: SwapResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.signature, result.signature);
        assert_eq!(deserialized.idempotency_key, result.idempotency_key);
    }

    #[test]
    fn test_price_info_serialization() {
        let price_info = PriceInfo {
            base_mint: "So11111111111111111111111111111111111111112".to_string(),
            quote_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            price: 50.5,
            price_impact_pct: 0.1,
        };

        let json = serde_json::to_string(&price_info).unwrap();
        assert!(json.contains("50.5"));

        let deserialized: PriceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.price, price_info.price);
        assert_eq!(deserialized.base_mint, price_info.base_mint);
    }

    #[test]
    fn test_route_plan_step_serialization() {
        let swap_info = SwapInfo {
            amm_key: "test_amm_key".to_string(),
            label: Some("Raydium".to_string()),
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            in_amount: "1000000".to_string(),
            out_amount: "50000".to_string(),
            fee_amount: "1000".to_string(),
            fee_mint: "So11111111111111111111111111111111111111112".to_string(),
        };

        let route_step = RoutePlanStep {
            swap_info,
            percent: 100,
        };

        let json = serde_json::to_string(&route_step).unwrap();
        assert!(json.contains("test_amm_key"));
        assert!(json.contains("Raydium"));

        let deserialized: RoutePlanStep = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.percent, 100);
        assert_eq!(deserialized.swap_info.label, Some("Raydium".to_string()));
    }

    #[test]
    fn test_swap_info_with_none_label() {
        let swap_info = SwapInfo {
            amm_key: "test_amm_key".to_string(),
            label: None,
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            in_amount: "1000000".to_string(),
            out_amount: "50000".to_string(),
            fee_amount: "1000".to_string(),
            fee_mint: "So11111111111111111111111111111111111111112".to_string(),
        };

        let json = serde_json::to_string(&swap_info).unwrap();
        let deserialized: SwapInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.label, None);
        assert_eq!(deserialized.amm_key, "test_amm_key");
    }

    #[test]
    fn test_jupiter_quote_response_serialization() {
        let response = JupiterQuoteResponse {
            in_amount: 1000000,
            out_amount: 990000,
            other_amount_threshold: 980000,
            route_plan: vec![],
            context_slot: Some(123456),
            time_taken: Some(0.5),
            price_impact_pct: Some(1.0),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: JupiterQuoteResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.in_amount, 1000000);
        assert_eq!(deserialized.price_impact_pct, Some(1.0));
    }

    #[test]
    fn test_jupiter_swap_response_serialization() {
        let response = JupiterSwapResponse {
            swap_transaction: "base64_transaction_data".to_string(),
            last_valid_block_height: 123456789,
            prioritization_fee: Some(5000),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: JupiterSwapResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.swap_transaction, "base64_transaction_data");
        assert_eq!(deserialized.last_valid_block_height, 123456789);
        assert_eq!(deserialized.prioritization_fee, Some(5000));
    }

    #[test]
    fn test_jupiter_swap_response_with_none_fee() {
        let response = JupiterSwapResponse {
            swap_transaction: "base64_transaction_data".to_string(),
            last_valid_block_height: 123456789,
            prioritization_fee: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: JupiterSwapResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.prioritization_fee, None);
    }

    // Debug and Clone trait tests
    #[test]
    fn test_struct_debug_implementations() {
        let swap_info = SwapInfo {
            amm_key: "test".to_string(),
            label: Some("Test".to_string()),
            input_mint: "input".to_string(),
            output_mint: "output".to_string(),
            in_amount: "1000".to_string(),
            out_amount: "900".to_string(),
            fee_amount: "10".to_string(),
            fee_mint: "fee".to_string(),
        };

        let route_step = RoutePlanStep {
            swap_info: swap_info.clone(),
            percent: 50,
        };

        let quote = SwapQuote {
            input_mint: "input".to_string(),
            output_mint: "output".to_string(),
            in_amount: 1000,
            out_amount: 900,
            other_amount_threshold: 890,
            price_impact_pct: 0.5,
            route_plan: vec![route_step.clone()],
            context_slot: Some(123),
            time_taken: Some(0.1),
        };

        let result = SwapResult {
            signature: "sig".to_string(),
            input_mint: "input".to_string(),
            output_mint: "output".to_string(),
            in_amount: 1000,
            out_amount: 900,
            price_impact_pct: 0.5,
            status: TransactionStatus::Pending,
            idempotency_key: None,
        };

        let price_info = PriceInfo {
            base_mint: "base".to_string(),
            quote_mint: "quote".to_string(),
            price: 1.5,
            price_impact_pct: 0.1,
        };

        // Test Debug implementations
        let debug_swap_info = format!("{:?}", swap_info);
        let debug_route_step = format!("{:?}", route_step);
        let debug_quote = format!("{:?}", quote);
        let debug_result = format!("{:?}", result);
        let debug_price_info = format!("{:?}", price_info);

        assert!(debug_swap_info.contains("SwapInfo"));
        assert!(debug_route_step.contains("RoutePlanStep"));
        assert!(debug_quote.contains("SwapQuote"));
        assert!(debug_result.contains("SwapResult"));
        assert!(debug_price_info.contains("PriceInfo"));

        // Test Clone implementations
        let cloned_swap_info = swap_info.clone();
        let cloned_route_step = route_step.clone();
        let cloned_quote = quote.clone();
        let cloned_result = result.clone();
        let cloned_price_info = price_info.clone();

        assert_eq!(cloned_swap_info.amm_key, swap_info.amm_key);
        assert_eq!(cloned_route_step.percent, route_step.percent);
        assert_eq!(cloned_quote.in_amount, quote.in_amount);
        assert_eq!(cloned_result.signature, result.signature);
        assert_eq!(cloned_price_info.price, price_info.price);
    }

    // Edge case tests for struct field variations
    #[test]
    fn test_swap_quote_with_empty_route_plan() {
        let quote = SwapQuote {
            input_mint: "input".to_string(),
            output_mint: "output".to_string(),
            in_amount: 1000,
            out_amount: 900,
            other_amount_threshold: 890,
            price_impact_pct: 0.0,
            route_plan: vec![],
            context_slot: None,
            time_taken: None,
        };

        assert!(quote.route_plan.is_empty());
        assert_eq!(quote.context_slot, None);
        assert_eq!(quote.time_taken, None);
        assert_eq!(quote.price_impact_pct, 0.0);
    }

    #[test]
    fn test_swap_result_with_none_idempotency_key() {
        let result = SwapResult {
            signature: "sig".to_string(),
            input_mint: "input".to_string(),
            output_mint: "output".to_string(),
            in_amount: 1000,
            out_amount: 900,
            price_impact_pct: 0.5,
            status: TransactionStatus::Confirmed,
            idempotency_key: None,
        };

        assert_eq!(result.idempotency_key, None);
        assert!(matches!(result.status, TransactionStatus::Confirmed));
    }

    #[test]
    fn test_route_plan_step_with_zero_percent() {
        let swap_info = SwapInfo {
            amm_key: "test".to_string(),
            label: None,
            input_mint: "input".to_string(),
            output_mint: "output".to_string(),
            in_amount: "0".to_string(),
            out_amount: "0".to_string(),
            fee_amount: "0".to_string(),
            fee_mint: "fee".to_string(),
        };

        let route_step = RoutePlanStep {
            swap_info,
            percent: 0,
        };

        assert_eq!(route_step.percent, 0);
        assert_eq!(route_step.swap_info.in_amount, "0");
    }

    #[test]
    fn test_price_info_with_zero_price() {
        let price_info = PriceInfo {
            base_mint: "base".to_string(),
            quote_mint: "quote".to_string(),
            price: 0.0,
            price_impact_pct: 0.0,
        };

        assert_eq!(price_info.price, 0.0);
        assert_eq!(price_info.price_impact_pct, 0.0);
    }

    #[test]
    fn test_price_info_with_negative_price_impact() {
        let price_info = PriceInfo {
            base_mint: "base".to_string(),
            quote_mint: "quote".to_string(),
            price: 1.0,
            price_impact_pct: -0.1, // Negative price impact (beneficial)
        };

        assert_eq!(price_info.price_impact_pct, -0.1);
    }

    #[test]
    fn test_jupiter_quote_response_with_all_none_optionals() {
        let response = JupiterQuoteResponse {
            in_amount: 1000,
            out_amount: 900,
            other_amount_threshold: 890,
            route_plan: vec![],
            context_slot: None,
            time_taken: None,
            price_impact_pct: None,
        };

        assert_eq!(response.context_slot, None);
        assert_eq!(response.time_taken, None);
        assert_eq!(response.price_impact_pct, None);
    }

    #[test]
    fn test_swap_info_with_empty_strings() {
        let swap_info = SwapInfo {
            amm_key: "".to_string(),
            label: Some("".to_string()),
            input_mint: "".to_string(),
            output_mint: "".to_string(),
            in_amount: "".to_string(),
            out_amount: "".to_string(),
            fee_amount: "".to_string(),
            fee_mint: "".to_string(),
        };

        assert_eq!(swap_info.amm_key, "");
        assert_eq!(swap_info.label, Some("".to_string()));
        assert_eq!(swap_info.input_mint, "");
    }

    #[test]
    fn test_large_numeric_values() {
        let quote = SwapQuote {
            input_mint: "input".to_string(),
            output_mint: "output".to_string(),
            in_amount: u64::MAX,
            out_amount: u64::MAX - 1,
            other_amount_threshold: u64::MAX - 2,
            price_impact_pct: f64::MAX,
            route_plan: vec![],
            context_slot: Some(u64::MAX),
            time_taken: Some(f64::MAX),
        };

        assert_eq!(quote.in_amount, u64::MAX);
        assert_eq!(quote.out_amount, u64::MAX - 1);
        assert_eq!(quote.price_impact_pct, f64::MAX);
    }

    // Test struct field access and manipulation
    #[test]
    fn test_mutable_struct_modifications() {
        let mut quote = SwapQuote {
            input_mint: "old".to_string(),
            output_mint: "old".to_string(),
            in_amount: 100,
            out_amount: 90,
            other_amount_threshold: 85,
            price_impact_pct: 1.0,
            route_plan: vec![],
            context_slot: Some(123),
            time_taken: Some(0.1),
        };

        quote.input_mint = "new".to_string();
        quote.in_amount = 200;
        quote.price_impact_pct = 2.0;

        assert_eq!(quote.input_mint, "new");
        assert_eq!(quote.in_amount, 200);
        assert_eq!(quote.price_impact_pct, 2.0);
    }

    // Helper function to create a test context with ApiClients
    fn create_test_context() -> riglr_core::provider::ApplicationContext {
        // Load .env.test for test environment
        dotenvy::from_filename(".env.test").ok();

        let config = riglr_config::Config::from_env();
        let context = riglr_core::provider::ApplicationContext::from_config(&std::sync::Arc::new(
            config.clone(),
        ));

        // Create and inject ApiClients
        let api_clients = crate::clients::ApiClients::new(&config.providers);
        context.set_extension(std::sync::Arc::new(api_clients));

        context
    }

    // Tests for async function input validation
    #[tokio::test]
    async fn test_get_jupiter_quote_invalid_input_mint() {
        let context = create_test_context();
        let result = get_jupiter_quote(
            "invalid_mint".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1000000,
            50,
            false,
            None,
            &context,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid input mint"));
    }

    #[tokio::test]
    async fn test_get_jupiter_quote_invalid_output_mint() {
        let context = create_test_context();
        let result = get_jupiter_quote(
            "So11111111111111111111111111111111111111112".to_string(),
            "invalid_mint".to_string(),
            1000000,
            50,
            false,
            None,
            &context,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid output mint"));
    }

    #[tokio::test]
    async fn test_get_jupiter_quote_empty_input_mint() {
        let context = create_test_context();
        let result = get_jupiter_quote(
            "".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1000000,
            50,
            false,
            None,
            &context,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_jupiter_quote_empty_output_mint() {
        let context = create_test_context();
        let result = get_jupiter_quote(
            "So11111111111111111111111111111111111111112".to_string(),
            "".to_string(),
            1000000,
            50,
            false,
            None,
            &context,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_jupiter_quote_zero_amount() {
        let context = create_test_context();
        let result = get_jupiter_quote(
            "So11111111111111111111111111111111111111112".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            0,
            50,
            false,
            None,
            &context,
        )
        .await;

        // Zero amount should pass validation but fail on Jupiter API
        // This tests that our validation allows zero (Jupiter will handle the business logic)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_jupiter_quote_max_slippage() {
        let context = create_test_context();
        let result = get_jupiter_quote(
            "So11111111111111111111111111111111111111112".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1000000,
            u16::MAX,
            false,
            None,
            &context,
        )
        .await;

        // Should pass validation but fail on Jupiter API due to network issues
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_jupiter_quote_with_custom_api_url() {
        let context = create_test_context();
        let result = get_jupiter_quote(
            "So11111111111111111111111111111111111111112".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1000000,
            50,
            false,
            Some("https://custom.api.com".to_string()),
            &context,
        )
        .await;

        // Should pass validation but fail on network request
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_jupiter_quote_only_direct_routes_true() {
        let context = create_test_context();
        let result = get_jupiter_quote(
            "So11111111111111111111111111111111111111112".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1000000,
            50,
            true, // only_direct_routes
            None,
            &context,
        )
        .await;

        // Should pass validation but fail on network request
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_token_price_invalid_base_mint() {
        let context = create_test_context();
        let result = get_token_price(
            "invalid_mint".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            None,
            &context,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        // The error message should indicate invalid mint format
        let error_str = error.to_string();
        assert!(
            error_str.contains("Invalid input mint") || error_str.contains("Invalid base58"),
            "Error was: {}",
            error_str
        );
    }

    #[tokio::test]
    async fn test_get_token_price_invalid_quote_mint() {
        let context = create_test_context();
        let result = get_token_price(
            "So11111111111111111111111111111111111111112".to_string(),
            "invalid_mint".to_string(),
            None,
            &context,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        // The error message should indicate invalid mint format
        let error_str = error.to_string();
        assert!(
            error_str.contains("Invalid output mint") || error_str.contains("Invalid base58"),
            "Error was: {}",
            error_str
        );
    }

    #[tokio::test]
    async fn test_get_token_price_with_custom_api() {
        let context = create_test_context();
        let result = get_token_price(
            "So11111111111111111111111111111111111111112".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            Some("https://custom.api.com".to_string()),
            &context,
        )
        .await;

        // Should pass validation but fail on network request
        assert!(result.is_err());
    }

    // Test perform_jupiter_swap function error paths
    #[tokio::test]
    async fn test_perform_jupiter_swap_invalid_input_mint() {
        let context = create_test_context();
        let result = perform_jupiter_swap(
            "invalid_mint".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1000000,
            50,
            None,
            true,
            &context,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        // Without a signer context, the error will be about missing signer
        assert!(error.to_string().contains("No signer context available"));
    }

    #[tokio::test]
    async fn test_perform_jupiter_swap_invalid_output_mint() {
        let context = create_test_context();
        let result = perform_jupiter_swap(
            "So11111111111111111111111111111111111111112".to_string(),
            "invalid_mint".to_string(),
            1000000,
            50,
            None,
            false,
            &context,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        // Without a signer context, the error will be about missing signer
        assert!(error.to_string().contains("No signer context available"));
    }

    #[tokio::test]
    async fn test_perform_jupiter_swap_versioned_transaction_false() {
        let context = create_test_context();
        let result = perform_jupiter_swap(
            "So11111111111111111111111111111111111111112".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1000000,
            50,
            None,
            false, // use_versioned_transaction = false
            &context,
        )
        .await;

        // Should pass validation but fail due to no signer context
        assert!(result.is_err());
    }

    // Additional edge case tests
    #[test]
    fn test_route_plan_step_max_percent() {
        let swap_info = SwapInfo {
            amm_key: "test".to_string(),
            label: Some("Test".to_string()),
            input_mint: "input".to_string(),
            output_mint: "output".to_string(),
            in_amount: "1000".to_string(),
            out_amount: "900".to_string(),
            fee_amount: "10".to_string(),
            fee_mint: "fee".to_string(),
        };

        let route_step = RoutePlanStep {
            swap_info,
            percent: 255, // u8::MAX
        };

        assert_eq!(route_step.percent, 255);
    }

    #[test]
    fn test_jupiter_quote_response_debug_clone() {
        let response = JupiterQuoteResponse {
            in_amount: 1000,
            out_amount: 900,
            other_amount_threshold: 890,
            route_plan: vec![],
            context_slot: Some(123),
            time_taken: Some(0.1),
            price_impact_pct: Some(1.0),
        };

        let cloned = response.clone();
        let debug_str = format!("{:?}", response);

        assert_eq!(cloned.in_amount, response.in_amount);
        assert!(debug_str.contains("JupiterQuoteResponse"));
    }

    #[test]
    fn test_jupiter_swap_response_debug_clone() {
        let response = JupiterSwapResponse {
            swap_transaction: "test".to_string(),
            last_valid_block_height: 123,
            prioritization_fee: Some(100),
        };

        let cloned = response.clone();
        let debug_str = format!("{:?}", response);

        assert_eq!(cloned.swap_transaction, response.swap_transaction);
        assert!(debug_str.contains("JupiterSwapResponse"));
    }

    // Test extreme numeric values
    #[test]
    fn test_jupiter_quote_response_with_zero_values() {
        let response = JupiterQuoteResponse {
            in_amount: 0,
            out_amount: 0,
            other_amount_threshold: 0,
            route_plan: vec![],
            context_slot: Some(0),
            time_taken: Some(0.0),
            price_impact_pct: Some(0.0),
        };

        assert_eq!(response.in_amount, 0);
        assert_eq!(response.out_amount, 0);
        assert_eq!(response.other_amount_threshold, 0);
    }

    #[test]
    fn test_jupiter_swap_response_with_zero_block_height() {
        let response = JupiterSwapResponse {
            swap_transaction: "".to_string(),
            last_valid_block_height: 0,
            prioritization_fee: Some(0),
        };

        assert_eq!(response.last_valid_block_height, 0);
        assert_eq!(response.prioritization_fee, Some(0));
    }

    // Test complex route plan
    #[test]
    fn test_complex_route_plan_serialization() {
        let swap_info1 = SwapInfo {
            amm_key: "key1".to_string(),
            label: Some("DEX1".to_string()),
            input_mint: "mint1".to_string(),
            output_mint: "mint2".to_string(),
            in_amount: "1000".to_string(),
            out_amount: "500".to_string(),
            fee_amount: "5".to_string(),
            fee_mint: "mint1".to_string(),
        };

        let swap_info2 = SwapInfo {
            amm_key: "key2".to_string(),
            label: None,
            input_mint: "mint2".to_string(),
            output_mint: "mint3".to_string(),
            in_amount: "500".to_string(),
            out_amount: "450".to_string(),
            fee_amount: "2".to_string(),
            fee_mint: "mint2".to_string(),
        };

        let route_plan = vec![
            RoutePlanStep {
                swap_info: swap_info1,
                percent: 60,
            },
            RoutePlanStep {
                swap_info: swap_info2,
                percent: 40,
            },
        ];

        let quote = SwapQuote {
            input_mint: "mint1".to_string(),
            output_mint: "mint3".to_string(),
            in_amount: 1000,
            out_amount: 450,
            other_amount_threshold: 440,
            price_impact_pct: 1.5,
            route_plan,
            context_slot: Some(12345),
            time_taken: Some(0.25),
        };

        let json = serde_json::to_string(&quote).unwrap();
        let deserialized: SwapQuote = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.route_plan.len(), 2);
        assert_eq!(deserialized.route_plan[0].percent, 60);
        assert_eq!(deserialized.route_plan[1].percent, 40);
        assert_eq!(
            deserialized.route_plan[0].swap_info.label,
            Some("DEX1".to_string())
        );
        assert_eq!(deserialized.route_plan[1].swap_info.label, None);
    }

    // Test struct field boundary values
    #[test]
    fn test_price_info_with_infinity_values() {
        let price_info = PriceInfo {
            base_mint: "base".to_string(),
            quote_mint: "quote".to_string(),
            price: f64::INFINITY,
            price_impact_pct: f64::NEG_INFINITY,
        };

        assert_eq!(price_info.price, f64::INFINITY);
        assert_eq!(price_info.price_impact_pct, f64::NEG_INFINITY);
    }

    #[test]
    fn test_price_info_with_nan_values() {
        let price_info = PriceInfo {
            base_mint: "base".to_string(),
            quote_mint: "quote".to_string(),
            price: f64::NAN,
            price_impact_pct: f64::NAN,
        };

        assert!(price_info.price.is_nan());
        assert!(price_info.price_impact_pct.is_nan());
    }

    // Test swap result with different transaction statuses
    #[test]
    fn test_swap_result_with_failed_status() {
        let result = SwapResult {
            signature: "failed_sig".to_string(),
            input_mint: "input".to_string(),
            output_mint: "output".to_string(),
            in_amount: 1000,
            out_amount: 0,           // No output due to failure
            price_impact_pct: 100.0, // High impact
            status: TransactionStatus::Failed("test failure".to_string()),
            idempotency_key: Some("retry_key".to_string()),
        };

        assert!(matches!(result.status, TransactionStatus::Failed(_)));
        assert_eq!(result.out_amount, 0);
        assert_eq!(result.idempotency_key, Some("retry_key".to_string()));
    }
}
