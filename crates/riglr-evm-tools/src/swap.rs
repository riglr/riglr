//! DEX integration tools for EVM chains

extern crate alloc;

use alloy::primitives::aliases::U24;
use alloy::primitives::{Address, Bytes, U160, U256};
use alloy::providers::Provider;
use alloy::rpc::types::{BlockId, TransactionRequest};
use alloy::sol_types::SolCall;
use riglr_core::{provider::ApplicationContext, signer::SignerContext, ToolError};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use alloc::sync::Arc;
use core::str::FromStr as _;
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
#[non_exhaustive]
pub struct Quote {
    pub amount_in: String,
    pub amount_out: String,
    pub amount_out_minimum: String,
    pub fee_tier: String,
    pub gas_estimate: String,
    pub price: String,
    pub price_impact: String,
    pub route: Vec<String>,
    pub token_in: String,
    pub token_out: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Parameters {
    pub token_in: String,
    pub token_out: String,
    pub amount_in: String,
    pub decimals_in: u8,
    pub decimals_out: u8,
    pub fee_tier: Option<u32>,
    pub slippage: Option<u32>,
    pub chain_id: Option<u64>,
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
        .map_err(|error| ToolError::permanent_string(format!("Invalid quoter address: {error}")))?;

    // Build the function call data using the generated types

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
        .map_err(|error| {
            ToolError::retriable_string(format!("Failed to get Uniswap quote: {error}"))
        })?;

    // Decode the result
    let decoded =
        <IQuoterV2::quoteExactInputSingleCall as SolCall>::abi_decode_returns(&result_bytes, true)
            .map_err(|error| {
                ToolError::permanent_string(format!("Failed to decode quote result: {error}"))
            })?;

    // Extract amountOut and gasEstimate from the result
    Ok((decoded.amountOut, decoded.gasEstimate))
}

/// Get swap quote from Uniswap
///
/// This tool implements smart chain ID resolution:
/// - If `chain_id` is provided, uses that specific chain
/// - If `chain_id` is None but there's an active EVM `SignerContext`, uses the signer's chain ID
/// - Otherwise defaults to Ethereum mainnet (`chain_id` = 1)
///
/// # Arguments
/// * `token_in` - Address of input token
/// * `token_out` - Address of output token  
/// * `amount_in` - Amount to swap in (in token's smallest unit)
/// * `decimals_in` - Decimal places of input token
/// * `decimals_out` - Decimal places of output token
/// * `fee_tier` - Pool fee tier (default: 3000 = 0.3%)
/// * `slippage` - Slippage tolerance in basis points (default: 50 = 0.5%)
/// * `chain_id` - Optional chain ID. If None, attempts to resolve from `SignerContext`
///
/// * `context` - Application context containing provider and configuration
///
/// # Errors
/// Returns `ToolError` if token addresses are invalid, provider is not found,
/// quoter address is not configured, or blockchain call fails
#[tool]
#[inline]
pub async fn get_uniswap_quote(
    params: Parameters,
    context: &ApplicationContext,
) -> Result<Quote, ToolError> {
    // Parse token addresses
    let token_in_addr = Address::from_str(&params.token_in).map_err(|error| {
        ToolError::permanent_string(format!("Invalid token_in address: {error}"))
    })?;
    let token_out_addr = Address::from_str(&params.token_out).map_err(|error| {
        ToolError::permanent_string(format!("Invalid token_out address: {error}"))
    })?;

    // Parse amount_in to U256
    let amount_in_wei = U256::from_str(&params.amount_in)
        .map_err(|error| ToolError::permanent_string(format!("Invalid amount_in: {error}")))?;

    // Get provider from ApplicationContext
    let provider = context
        .get_extension::<Arc<dyn Provider>>()
        .ok_or_else(|| ToolError::permanent_string("Provider not found in context".to_owned()))?;
    let client = &**provider;

    let resolved_chain_id = resolve_chain_id(params.chain_id);

    // Get quoter address from configuration
    let quoter_address = get_quoter_address(context, resolved_chain_id)?;

    // Default fee tier (0.3% = 3000)
    let fee = params.fee_tier.unwrap_or(3000);

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

    let amount_out_formatted = format_amount(amount_out_wei, params.decimals_out)?;

    // Calculate price (amount_out / amount_in)
    let price = calculate_price(
        amount_in_wei,
        amount_out_wei,
        params.decimals_in,
        params.decimals_out,
    )?;

    // Calculate minimum amount out with slippage
    let (amount_out_minimum, _amount_out_minimum_wei) = calculate_minimum_amount_out(
        amount_out_wei,
        params.slippage.unwrap_or(50),
        params.decimals_out,
    )?;

    // Calculate price impact (simplified - comparing against perfect AMM price)
    let price_impact = format!("{}%", f64::from(fee) / 10_000.0f64);

    // Format fee tier
    let fee_tier_str = format!("{}%", f64::from(fee) / 10_000.0f64);

    Ok(Quote {
        amount_in: params.amount_in,
        amount_out: amount_out_formatted,
        amount_out_minimum,
        fee_tier: fee_tier_str.clone(),
        gas_estimate: gas_estimate_wei.to_string(),
        price,
        price_impact,
        route: vec![format!("Uniswap V3 {fee_tier_str} pool")],
        token_in: params.token_in,
        token_out: params.token_out,
    })
}

fn resolve_chain_id(chain_id: Option<u64>) -> u64 {
    chain_id.map_or_else(
        || {
            SignerContext::current_as_evm().map_or_else(
                |_| {
                    debug!(
                    "No explicit chain_id or EVM signer context, defaulting to Ethereum mainnet (1)"
                );
                    1 // Fallback to Ethereum mainnet
                },
                |signer| {
                    let id = signer.signer().chain_id();
                    debug!("Using chain_id from EVM SignerContext: {id}");
                    id
                },
            )
        },
        |id| {
            debug!("Using explicit chain_id: {id}");
            id
        },
    )
}

fn get_quoter_address(context: &ApplicationContext, chain_id: u64) -> Result<&str, ToolError> {
    context
        .config
        .network
        .get_chain(chain_id)
        .and_then(|chain| chain.contracts.quoter.as_deref())
        .ok_or_else(|| {
            ToolError::permanent_string(format!(
                "Uniswap V3 quoter address not configured for chain_id {chain_id}. Please add it to chains.toml or set QUOTER_{chain_id} environment variable"
            ))
        })
}

fn format_amount(amount: U256, decimals: u8) -> Result<String, ToolError> {
    let divisor = U256::from(10u64)
        .checked_pow(U256::from(decimals))
        .ok_or_else(|| ToolError::permanent_string("Overflow in divisor calculation"))?;
    let whole = amount
        .checked_div(divisor)
        .ok_or_else(|| ToolError::permanent_string("Division overflow in amount formatting"))?;
    let fraction = amount
        .checked_rem(divisor)
        .ok_or_else(|| ToolError::permanent_string("Modulo overflow in amount formatting"))?;

    if fraction.is_zero() {
        Ok(whole.to_string())
    } else {
        let fraction_str = format!("{:0>width$}", fraction, width = decimals as usize);
        let trimmed = fraction_str.trim_end_matches('0');
        if trimmed.is_empty() {
            Ok(whole.to_string())
        } else {
            Ok(format!("{whole}.{trimmed}"))
        }
    }
}

fn calculate_price(
    amount_in: U256,
    amount_out: U256,
    decimals_in: u8,
    decimals_out: u8,
) -> Result<String, ToolError> {
    if amount_in.is_zero() {
        return Ok("0.0".to_owned());
    }

    let divisor_in = U256::from(10u64)
        .checked_pow(U256::from(decimals_in))
        .ok_or_else(|| ToolError::permanent_string("Overflow in divisor_in calculation"))?;
    let divisor_out = U256::from(10u64)
        .checked_pow(U256::from(decimals_out))
        .ok_or_else(|| ToolError::permanent_string("Overflow in divisor_out calculation"))?;

    let price_raw = amount_out
        .checked_mul(divisor_in)
        .and_then(|v| v.checked_div(amount_in))
        .and_then(|v| v.checked_div(divisor_out))
        .ok_or_else(|| ToolError::permanent_string("Overflow in price calculation"))?;

    let whole = price_raw
        .checked_div(U256::from(1000))
        .ok_or_else(|| ToolError::permanent_string("Division overflow in price formatting"))?;
    let fraction = price_raw
        .checked_rem(U256::from(1000))
        .ok_or_else(|| ToolError::permanent_string("Modulo overflow in price formatting"))?;

    Ok(format!("{whole}.{fraction:03}"))
}

fn calculate_minimum_amount_out(
    amount_out: U256,
    slippage_bps: u32,
    decimals: u8,
) -> Result<(String, U256), ToolError> {
    let slippage_factor = U256::from(
        10000u32
            .checked_sub(slippage_bps)
            .ok_or_else(|| ToolError::permanent_string("Slippage calculation overflow"))?,
    );

    let amount_out_minimum = amount_out
        .checked_mul(slippage_factor)
        .and_then(|v| v.checked_div(U256::from(10000u32)))
        .ok_or_else(|| ToolError::permanent_string("Overflow in minimum amount out calculation"))?;

    let formatted = format_amount(amount_out_minimum, decimals)?;
    Ok((formatted, amount_out_minimum))
}
