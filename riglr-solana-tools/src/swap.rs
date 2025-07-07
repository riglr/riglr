//! Jupiter DEX integration for token swaps on Solana
//!
//! This module provides tools for interacting with the Jupiter aggregator,
//! enabling token swaps with optimal routing across multiple DEXs.

use crate::client::SolanaClient;
use crate::transaction::{get_signer_context, TransactionStatus};
use anyhow::anyhow;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction};
use std::str::FromStr;
use tracing::{debug, info};

/// Jupiter API configuration
#[derive(Debug, Clone)]
pub struct JupiterConfig {
    /// Jupiter API base URL
    pub api_url: String,
    pub slippage_bps: u16,
    /// Whether to use only direct routes
    pub only_direct_routes: bool,
    pub max_accounts: Option<usize>,
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            api_url: "https://quote-api.jup.ag/v6".to_string(),
            slippage_bps: 50, // 0.5% default slippage
            only_direct_routes: false,
            max_accounts: Some(20),
        }
    }
}

/// Get a quote from Jupiter for swapping tokens
///
/// This tool queries the Jupiter aggregator for the best swap route
/// and returns the expected output amount.
// #[tool]
pub async fn get_jupiter_quote(
    input_mint: String,
    output_mint: String,
    amount: u64,
    slippage_bps: u16,
    only_direct_routes: bool,
    jupiter_api_url: Option<String>,
) -> anyhow::Result<SwapQuote> {
    debug!(
        "Getting Jupiter quote for {} -> {} (amount: {})",
        input_mint, output_mint, amount
    );

    // Validate mint addresses
    let _input_pubkey =
        Pubkey::from_str(&input_mint).map_err(|e| anyhow!("Invalid input mint: {}", e))?;
    let _output_pubkey =
        Pubkey::from_str(&output_mint).map_err(|e| anyhow!("Invalid output mint: {}", e))?;

    let api_url = jupiter_api_url.unwrap_or_else(|| JupiterConfig::default().api_url);

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
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to request quote: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("Jupiter API error: {}", error_text));
    }

    let quote_response: JupiterQuoteResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse quote response: {}", e))?;

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
/// This tool executes a swap using the Jupiter aggregator,
/// handling transaction construction and submission.
// #[tool]
pub async fn perform_jupiter_swap(
    client: &SolanaClient,
    input_mint: String,
    output_mint: String,
    amount: u64,
    slippage_bps: u16,
    signer_name: Option<String>,
    jupiter_api_url: Option<String>,
    use_versioned_transaction: bool,
) -> anyhow::Result<SwapResult> {
    debug!(
        "Executing Jupiter swap: {} {} -> {}",
        amount, input_mint, output_mint
    );

    // Get signer
    let signer_context =
        get_signer_context().map_err(|e| anyhow!("Failed to get signer context: {}", e))?;

    let signer = if let Some(name) = signer_name {
        signer_context
            .get_signer(&name)
            .map_err(|e| anyhow!("Failed to get signer '{}': {}", name, e))?
    } else {
        signer_context
            .get_default_signer()
            .map_err(|e| anyhow!("Failed to get default signer: {}", e))?
    };

    let api_url = jupiter_api_url.unwrap_or_else(|| JupiterConfig::default().api_url);

    // First get a quote
    let quote = get_jupiter_quote(
        input_mint.clone(),
        output_mint.clone(),
        amount,
        slippage_bps,
        false,
        Some(api_url.clone()),
    )
    .await?;

    // Build swap request
    let swap_request = json!({
        "userPublicKey": signer.pubkey().to_string(),
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
        .map_err(|e| anyhow!("Failed to request swap transaction: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("Jupiter swap API error: {}", error_text));
    }

    let swap_response: JupiterSwapResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse swap response: {}", e))?;

    // Deserialize and sign the transaction
    use base64::{engine::general_purpose, Engine as _};
    let transaction_bytes = general_purpose::STANDARD
        .decode(&swap_response.swap_transaction)
        .map_err(|e| anyhow!("Failed to decode transaction: {}", e))?;

    let mut transaction: Transaction = bincode::deserialize(&transaction_bytes)
        .map_err(|e| anyhow!("Failed to deserialize transaction: {}", e))?;

    // Sign the transaction
    let blockhash = transaction.message.recent_blockhash;
    transaction.partial_sign(&[signer.as_ref()], blockhash);

    // Send transaction
    let signature = client
        .send_and_confirm_transaction(&transaction)
        .await
        .map_err(|e| anyhow!("Failed to send swap transaction: {}", e))?;

    let sig_str = signature.to_string();
    info!(
        "Jupiter swap executed: {} {} -> {} {} (expected), signature: {}",
        quote.in_amount, input_mint, quote.out_amount, output_mint, sig_str
    );

    Ok(SwapResult {
        signature: sig_str,
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
/// This tool fetches the current price and liquidity information
// #[tool]
pub async fn get_token_price(
    base_mint: String,
    quote_mint: String,
    jupiter_api_url: Option<String>,
) -> anyhow::Result<PriceInfo> {
    debug!("Getting price for {} in terms of {}", base_mint, quote_mint);

    let api_url = jupiter_api_url.unwrap_or_else(|| JupiterConfig::default().api_url);

    // Get a small quote to determine price
    let amount = 1_000_000; // 1 token with 6 decimals
    let quote = get_jupiter_quote(
        base_mint.clone(),
        quote_mint.clone(),
        amount,
        50, // 0.5% slippage
        false,
        Some(api_url),
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
    if let Some(price_impact) = quote.price_impact_pct {
        price_impact
    } else {
        0.0
    }
}

fn default_slippage() -> u16 {
    50 // 0.5%
}

/// Default USDC mint address
fn default_usdc_mint() -> String {
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()
}

/// Default true value
fn default_true() -> bool {
    true
}

/// Jupiter quote response
#[derive(Debug, Clone, Serialize, Deserialize)]

struct JupiterQuoteResponse {
    pub in_amount: u64,
    pub out_amount: u64,
    pub other_amount_threshold: u64,
    pub route_plan: Vec<RoutePlanStep>,
    pub context_slot: Option<u64>,
    pub time_taken: Option<f64>,
    pub price_impact_pct: Option<f64>,
}

/// Jupiter swap response
#[derive(Debug, Clone, Serialize, Deserialize)]

struct JupiterSwapResponse {
    pub swap_transaction: String,
    pub last_valid_block_height: u64,
    pub prioritization_fee: Option<u64>,
}

/// Route plan step in Jupiter quote
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]

pub struct RoutePlanStep {
    pub swap_info: SwapInfo,
    pub percent: u8,
}

/// Swap information for a route step
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]

pub struct SwapInfo {
    pub amm_key: String,
    pub label: Option<String>,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
}

/// Result of a swap quote from Jupiter
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SwapQuote {
    pub input_mint: String,
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
    pub input_mint: String,
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
    pub base_mint: String,
    pub quote_mint: String,
    /// Price of base in terms of quote
    pub price: f64,
    /// Price impact for small trade
    pub price_impact_pct: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = JupiterConfig::default();
        assert_eq!(config.slippage_bps, 50);
        assert!(!config.only_direct_routes);
        assert!(config.api_url.contains("jup.ag"));
    }

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
    }
}
