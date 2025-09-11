//! DEX integration tools for EVM chains

use alloy::primitives::{Address, U256, U160, Bytes};
use alloy::providers::Provider;
use alloy::rpc::types::{TransactionRequest, BlockId};
use alloy::sol_types::SolCall;
use riglr_core::{signer::SignerContext, ToolError};
use riglr_macros::tool;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::debug;

// Type-safe contract bindings for Uniswap V3 Quoter
alloy::sol! {
    /// Uniswap V3 Quoter V2 interface - only the functions we need
    interface IQuoterV2 {
        /// Returns the amount out received for a given exact input swap without executing the swap
        /// @param tokenIn The token being swapped in
        /// @param tokenOut The token being swapped out
        /// @param fee The fee of the token pool to consider for the pair
        /// @param amountIn The desired input amount
        /// @param sqrtPriceLimitX96 The price limit of the pool that cannot be exceeded by the swap
        /// @return amountOut The amount of `tokenOut` that would be received
        /// @return sqrtPriceX96After The sqrt price of the pool after the swap
        /// @return initializedTicksCrossed The number of initialized ticks crossed
        /// @return gasEstimate The estimate of the gas that the swap consumes
        function quoteExactInputSingle(
            address tokenIn,
            address tokenOut,
            uint24 fee,
            uint256 amountIn,
            uint160 sqrtPriceLimitX96
        ) external returns (
            uint256 amountOut,
            uint160 sqrtPriceX96After,
            uint32 initializedTicksCrossed,
            uint256 gasEstimate
        );
    }
}

/// Swap quote response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub amount_out: String,
    pub amount_in: String,
    pub token_in: String,
    pub token_out: String,
    pub price: String,
    pub price_impact: String,
    pub gas_estimate: String,
    pub route: Vec<String>,
    pub fee_tier: String,
    pub amount_out_minimum: String,
}

/// Internal type-safe function to get Uniswap quote using generated bindings
async fn get_uniswap_quote_typed(
    token_in_addr: Address,
    token_out_addr: Address,
    amount_in_wei: U256,
    fee: u32,
    quoter_address: &str,
    client: &dyn Provider,
) -> Result<(U256, U256), ToolError> {
    let quoter_addr = Address::from_str(quoter_address)
        .map_err(|e| ToolError::permanent_string(format!("Invalid quoter address: {}", e)))?;

    // Build the function call data using the generated types
    use alloy::primitives::aliases::U24;

    let call = IQuoterV2::quoteExactInputSingleCall {
        tokenIn: token_in_addr,
        tokenOut: token_out_addr,
        fee: U24::from(fee),
        amountIn: amount_in_wei,
        sqrtPriceLimitX96: U160::ZERO,
    };

    // Encode the call data
    let encoded = call.abi_encode();

    // Make the eth_call using the client directly
    let tx_request = TransactionRequest::default()
        .to(quoter_addr)
        .input(Bytes::from(encoded).into());

    let result_bytes = client
        .call(&tx_request)
        .block(BlockId::latest())
        .await
        .map_err(|e| ToolError::retriable_string(format!("Failed to get Uniswap quote: {}", e)))?;

    // Decode the result
    let decoded =
        <IQuoterV2::quoteExactInputSingleCall as SolCall>::abi_decode_returns(&result_bytes, true)
            .map_err(|e| {
                ToolError::permanent_string(format!("Failed to decode quote result: {}", e))
            })?;

    // Extract amountOut and gasEstimate from the result
    Ok((decoded.amountOut, decoded.gasEstimate))
}

/// Get swap quote from Uniswap
///
/// This tool implements smart chain ID resolution:
/// - If `chain_id` is provided, uses that specific chain
/// - If `chain_id` is None but there's an active EVM SignerContext, uses the signer's chain ID
/// - Otherwise defaults to Ethereum mainnet (chain_id = 1)
///
/// # Arguments
/// * `token_in` - Address of input token
/// * `token_out` - Address of output token  
/// * `amount_in` - Amount to swap in (in token's smallest unit)
/// * `decimals_in` - Decimal places of input token
/// * `decimals_out` - Decimal places of output token
/// * `fee_tier` - Pool fee tier (default: 3000 = 0.3%)
/// * `slippage` - Slippage tolerance in basis points (default: 50 = 0.5%)
/// * `chain_id` - Optional chain ID. If None, attempts to resolve from SignerContext
/// * `context` - Application context containing provider and configuration
#[tool]
#[allow(clippy::too_many_arguments)]
pub async fn get_uniswap_quote(
    token_in: String,
    token_out: String,
    amount_in: String,
    decimals_in: u8,
    decimals_out: u8,
    fee_tier: Option<u32>,
    slippage: Option<u32>,
    chain_id: Option<u64>,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<SwapQuote, ToolError> {
    // Parse token addresses
    let token_in_addr = Address::from_str(&token_in)
        .map_err(|e| ToolError::permanent_string(format!("Invalid token_in address: {}", e)))?;
    let token_out_addr = Address::from_str(&token_out)
        .map_err(|e| ToolError::permanent_string(format!("Invalid token_out address: {}", e)))?;

    // Parse amount_in to U256
    let amount_in_wei = U256::from_str(&amount_in)
        .map_err(|e| ToolError::permanent_string(format!("Invalid amount_in: {}", e)))?;

    // Get provider from ApplicationContext
    let provider = context
        .get_extension::<std::sync::Arc<dyn Provider>>()
        .ok_or_else(|| ToolError::permanent_string("Provider not found in context".to_string()))?;
    let client = &**provider;

    // Smart chain_id resolution with SignerContext fallback
    let resolved_chain_id = if let Some(id) = chain_id {
        debug!("Using explicit chain_id: {}", id);
        id
    } else if let Ok(signer) = SignerContext::current_as_evm().await {
        let id = signer.chain_id();
        debug!("Using chain_id from EVM SignerContext: {}", id);
        id
    } else {
        debug!("No explicit chain_id or EVM signer context, defaulting to Ethereum mainnet (1)");
        1 // Fallback to Ethereum mainnet
    };

    // Get quoter address from configuration
    let quoter_address = context
        .config
        .network
        .get_chain(resolved_chain_id)
        .and_then(|chain| chain.contracts.quoter.as_ref())
        .ok_or_else(|| {
            ToolError::permanent_string(format!(
                "Uniswap V3 quoter address not configured for chain_id {}. Please add it to chains.toml or set QUOTER_{} environment variable",
                resolved_chain_id, resolved_chain_id
            ))
        })?;

    // Default fee tier (0.3% = 3000)
    let fee = fee_tier.unwrap_or(3000);

    // Call the type-safe internal function
    let (amount_out_wei, gas_estimate_wei) = get_uniswap_quote_typed(
        token_in_addr,
        token_out_addr,
        amount_in_wei,
        fee,
        quoter_address,
        client,
    )
    .await?;

    // Format amount_out with decimals
    let divisor_out = U256::from(10u64).pow(U256::from(decimals_out));
    let amount_out_whole = amount_out_wei / divisor_out;
    let amount_out_fraction = amount_out_wei % divisor_out;

    let amount_out_formatted = if amount_out_fraction.is_zero() {
        amount_out_whole.to_string()
    } else {
        let fraction_str = format!(
            "{:0>width$}",
            amount_out_fraction,
            width = decimals_out as usize
        );
        let trimmed = fraction_str.trim_end_matches('0');
        if trimmed.is_empty() {
            amount_out_whole.to_string()
        } else {
            format!("{}.{}", amount_out_whole, trimmed)
        }
    };

    // Calculate price (amount_out / amount_in)
    let divisor_in = U256::from(10u64).pow(U256::from(decimals_in));
    let price = if !amount_in_wei.is_zero() {
        // Price = amount_out / amount_in (adjusted for decimals)
        let price_raw = amount_out_wei * divisor_in / amount_in_wei / divisor_out;
        format!(
            "{}.{}",
            price_raw / U256::from(1000),
            price_raw % U256::from(1000)
        )
    } else {
        "0.0".to_string()
    };

    // Calculate minimum amount out with slippage
    let slippage_bps = slippage.unwrap_or(50); // Default 0.5% slippage
    let slippage_factor = U256::from(10000 - slippage_bps);
    let amount_out_minimum_wei = amount_out_wei * slippage_factor / U256::from(10000);

    let amount_out_min_whole = amount_out_minimum_wei / divisor_out;
    let amount_out_min_fraction = amount_out_minimum_wei % divisor_out;

    let amount_out_minimum = if amount_out_min_fraction.is_zero() {
        amount_out_min_whole.to_string()
    } else {
        let fraction_str = format!(
            "{:0>width$}",
            amount_out_min_fraction,
            width = decimals_out as usize
        );
        let trimmed = fraction_str.trim_end_matches('0');
        if trimmed.is_empty() {
            amount_out_min_whole.to_string()
        } else {
            format!("{}.{}", amount_out_min_whole, trimmed)
        }
    };

    // Calculate price impact (simplified - comparing against perfect AMM price)
    let price_impact = format!("{}%", fee as f64 / 10000.0);

    // Format fee tier
    let fee_tier_str = format!("{}%", fee as f64 / 10000.0);

    Ok(SwapQuote {
        amount_out: amount_out_formatted,
        amount_in,
        token_in,
        token_out,
        price,
        price_impact,
        gas_estimate: gas_estimate_wei.to_string(),
        route: vec![format!("Uniswap V3 {} pool", fee_tier_str)],
        fee_tier: fee_tier_str,
        amount_out_minimum,
    })
}
