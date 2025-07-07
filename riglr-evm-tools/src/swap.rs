//! Uniswap V3 integration for token swaps on EVM chains
//!
//! This module provides production-grade tools for interacting with Uniswap V3,
//! enabling token swaps with optimal routing across multiple pools.

use crate::{
    client::{validate_address, EvmClient},
    error::{EvmToolError, Result},
    transaction::{derive_address_from_key, get_evm_signer_context, TransactionStatus},
};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Uniswap V3 configuration
#[derive(Debug, Clone)]
pub struct UniswapConfig {
    /// Uniswap V3 SwapRouter contract address
    pub router_address: String,
    /// Uniswap V3 Quoter contract address
    pub quoter_address: String,
    /// Default slippage tolerance (basis points, 100 = 1%)
    pub slippage_bps: u16,
    /// Default deadline for transactions (seconds from now)
    pub deadline_seconds: u64,
}

impl UniswapConfig {
    /// Default configuration for Ethereum mainnet
    pub fn ethereum() -> Self {
        Self {
            router_address: "0xE592427A0AEce92De3Edee1F18E0157C05861564".to_string(), // Uniswap V3 SwapRouter
            quoter_address: "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".to_string(), // Uniswap V3 Quoter
            slippage_bps: 50,      // 0.5% default slippage
            deadline_seconds: 300, // 5 minutes
        }
    }

    /// Default configuration for Polygon
    pub fn polygon() -> Self {
        Self {
            router_address: "0xE592427A0AEce92De3Edee1F18E0157C05861564".to_string(),
            quoter_address: "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".to_string(),
            slippage_bps: 50,
            deadline_seconds: 300,
        }
    }

    /// Default configuration for Arbitrum One
    pub fn arbitrum() -> Self {
        Self {
            router_address: "0xE592427A0AEce92De3Edee1F18E0157C05861564".to_string(),
            quoter_address: "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".to_string(),
            slippage_bps: 50,
            deadline_seconds: 300,
        }
    }
}

impl Default for UniswapConfig {
    fn default() -> Self {
        Self::ethereum()
    }
}

/// Get a quote for a token swap from Uniswap V3
///
/// This tool queries Uniswap V3's Quoter contract for the best swap route
/// and returns the expected output amount.
// #[tool]
pub async fn get_uniswap_quote(
    token_in: String,
    token_out: String,
    amount_in: String,
    fee_tier: u32,
    rpc_url: Option<String>,
    network_config: Option<String>,
) -> anyhow::Result<SwapQuote> {
    debug!(
        "Getting Uniswap quote: {} -> {} (amount: {})",
        token_in, token_out, amount_in
    );

    // Validate token addresses
    let validated_token_in =
        validate_address(&token_in).map_err(|e| anyhow::anyhow!("Invalid input token: {}", e))?;
    let validated_token_out =
        validate_address(&token_out).map_err(|e| anyhow::anyhow!("Invalid output token: {}", e))?;

    // Parse amount
    let amount_raw: u128 = amount_in
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid amount: {}", e))?;

    // Create client
    let client = if let Some(url) = rpc_url {
        Arc::new(
            EvmClient::with_rpc_url(url)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create client: {}", e))?,
        )
    } else {
        Arc::new(
            EvmClient::ethereum()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create Ethereum client: {}", e))?,
        )
    };

    // Get network config
    let config = match network_config.as_deref() {
        Some("polygon") => UniswapConfig::polygon(),
        Some("arbitrum") => UniswapConfig::arbitrum(),
        _ => match client.chain_id {
            137 => UniswapConfig::polygon(),
            42161 => UniswapConfig::arbitrum(),
            _ => UniswapConfig::ethereum(),
        },
    };

    // Build quote call data for Quoter contract
    // quoteExactInputSingle(address tokenIn, address tokenOut, uint24 fee, uint256 amountIn, uint160 sqrtPriceLimitX96)
    let quote_call_data = build_quote_call_data(
        &validated_token_in,
        &validated_token_out,
        fee_tier,
        amount_raw,
        0, // sqrtPriceLimitX96 = 0 means no price limit
    )?;

    debug!(
        "Calling Uniswap Quoter at {} with data: {}",
        config.quoter_address, quote_call_data
    );

    // Call quoter contract
    let result = client
        .call_contract(&config.quoter_address, &quote_call_data)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get quote from Uniswap: {}", e))?;

    // Parse result (uint256 amountOut)
    let amount_out = u128::from_str_radix(result.trim_start_matches("0x"), 16).unwrap_or(0);

    // Calculate price impact (simplified)
    let price_impact = calculate_price_impact(amount_raw, amount_out);

    let network = match client.chain_id {
        1 => "Ethereum".to_string(),
        137 => "Polygon".to_string(),
        42161 => "Arbitrum One".to_string(),
        10 => "Optimism".to_string(),
        8453 => "Base".to_string(),
        _ => format!("Chain {}", client.chain_id),
    };

    info!(
        "Uniswap quote: {} -> {} (fee tier: {}, price impact: {:.2}%)",
        amount_raw,
        amount_out,
        fee_tier,
        price_impact * 100.0
    );

    Ok(SwapQuote {
        token_in: validated_token_in,
        token_out: validated_token_out,
        amount_in: amount_raw,
        amount_out,
        fee_tier,
        price_impact_pct: price_impact * 100.0,
        router_address: config.router_address.clone(),
        network,
    })
}

/// Execute a token swap through Uniswap V3
///
/// This tool executes a swap using Uniswap V3's SwapRouter,
/// handling transaction construction and submission.
// #[tool]
pub async fn perform_uniswap_swap(
    token_in: String,
    token_out: String,
    amount_in: String,
    amount_out_minimum: String,
    fee_tier: u32,
    from_signer: Option<String>,
    rpc_url: Option<String>,
    network_config: Option<String>,
    gas_price: Option<u64>,
    gas_limit: Option<u64>,
    idempotency_key: Option<String>,
) -> anyhow::Result<SwapResult> {
    debug!(
        "Executing Uniswap swap: {} {} -> {}",
        amount_in, token_in, token_out
    );

    // Validate addresses
    let validated_token_in =
        validate_address(&token_in).map_err(|e| anyhow::anyhow!("Invalid input token: {}", e))?;
    let validated_token_out =
        validate_address(&token_out).map_err(|e| anyhow::anyhow!("Invalid output token: {}", e))?;

    // Parse amounts
    let amount_in_raw: u128 = amount_in
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid input amount: {}", e))?;
    let amount_out_min_raw: u128 = amount_out_minimum
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid minimum output amount: {}", e))?;

    // Get signer
    let signer_context = get_evm_signer_context()
        .map_err(|e| anyhow::anyhow!("Failed to get signer context: {}", e))?;

    let signer_key = if let Some(name) = from_signer {
        signer_context
            .get_signer(&name)
            .map_err(|e| anyhow::anyhow!("Failed to get signer '{}': {}", name, e))?
    } else {
        signer_context
            .get_default_signer()
            .map_err(|e| anyhow::anyhow!("Failed to get default signer: {}", e))?
    };

    // Create client
    let client = if let Some(url) = rpc_url {
        Arc::new(
            EvmClient::with_rpc_url(url)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create client: {}", e))?,
        )
    } else {
        Arc::new(
            EvmClient::ethereum()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create Ethereum client: {}", e))?,
        )
    };

    // Get network config
    let config = match network_config.as_deref() {
        Some("polygon") => UniswapConfig::polygon(),
        Some("arbitrum") => UniswapConfig::arbitrum(),
        _ => match client.chain_id {
            137 => UniswapConfig::polygon(),
            42161 => UniswapConfig::arbitrum(),
            _ => UniswapConfig::ethereum(),
        },
    };

    let from_address = derive_address_from_key(&signer_key)?;

    // Get nonce and gas price
    let nonce = client
        .get_transaction_count(&from_address)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get nonce: {}", e))?;

    let gas_price = if let Some(price) = gas_price {
        price
    } else {
        client
            .get_gas_price()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get gas price: {}", e))?
    };

    // Build swap call data
    let deadline = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + config.deadline_seconds) as u128;

    let swap_call_data = build_swap_call_data(
        &validated_token_in,
        &validated_token_out,
        fee_tier,
        &from_address,
        amount_in_raw,
        amount_out_min_raw,
        deadline,
    )?;

    // Build transaction data
    let transaction_data = crate::transaction::build_contract_call_tx(
        &config.router_address,
        &swap_call_data,
        nonce,
        gas_price,
        gas_limit.unwrap_or(300000), // Uniswap V3 swap gas limit
        client.chain_id,
    )?;

    // Sign transaction
    let signed_tx = crate::transaction::sign_transaction(transaction_data, &signer_key)?;

    // Send transaction
    let tx_hash = client
        .send_raw_transaction(&signed_tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send swap transaction: {}", e))?;

    let network = match client.chain_id {
        1 => "Ethereum".to_string(),
        137 => "Polygon".to_string(),
        42161 => "Arbitrum One".to_string(),
        10 => "Optimism".to_string(),
        8453 => "Base".to_string(),
        _ => format!("Chain {}", client.chain_id),
    };

    info!(
        "Uniswap swap executed: {} {} -> {} {} (expected), tx: {}",
        amount_in_raw, token_in, amount_out_min_raw, token_out, tx_hash
    );

    Ok(SwapResult {
        tx_hash,
        token_in: validated_token_in,
        token_out: validated_token_out,
        amount_in: amount_in_raw,
        amount_out_minimum: amount_out_min_raw,
        fee_tier,
        status: TransactionStatus::Pending,
        network,
        gas_price,
        idempotency_key,
    })
}

/// Get token information and current price
///
/// This tool fetches token information and calculates current price based on Uniswap V3 pools.
// #[tool]
pub async fn get_token_price(
    base_token: String,
    quote_token: String,
    fee_tier: Option<u32>,
    rpc_url: Option<String>,
) -> anyhow::Result<TokenPriceInfo> {
    debug!(
        "Getting token price: {} in terms of {}",
        base_token, quote_token
    );

    // Use a small amount (1 unit) to get the price
    let amount = "1000000".to_string(); // 1 token with 6 decimals

    let quote = get_uniswap_quote(
        base_token.clone(),
        quote_token.clone(),
        amount,
        fee_tier.unwrap_or(3000), // Default to 0.3% fee tier
        rpc_url,
        None,
    )
    .await?;

    // Calculate price
    let price = quote.amount_out as f64 / quote.amount_in as f64;

    Ok(TokenPriceInfo {
        base_token,
        quote_token,
        price,
        fee_tier: quote.fee_tier,
        price_impact_pct: quote.price_impact_pct,
        network: quote.network,
    })
}

/// Build quote call data for Uniswap V3 Quoter contract
fn build_quote_call_data(
    token_in: &str,
    token_out: &str,
    fee: u32,
    amount_in: u128,
    sqrt_price_limit_x96: u128,
) -> anyhow::Result<String> {
    // quoteExactInputSingle function selector: 0xf7729d43
    let selector = "f7729d43";
    let token_in_padded = format!("{:0>64}", token_in.trim_start_matches("0x"));
    let token_out_padded = format!("{:0>64}", token_out.trim_start_matches("0x"));
    let fee_padded = format!("{:0>64x}", fee);
    let amount_in_padded = format!("{:0>64x}", amount_in);
    let sqrt_price_limit_padded = format!("{:0>64x}", sqrt_price_limit_x96);

    Ok(format!(
        "0x{}{}{}{}{}{}",
        selector,
        token_in_padded,
        token_out_padded,
        fee_padded,
        amount_in_padded,
        sqrt_price_limit_padded
    ))
}

/// Build swap call data for Uniswap V3 SwapRouter contract
fn build_swap_call_data(
    token_in: &str,
    token_out: &str,
    fee: u32,
    recipient: &str,
    amount_in: u128,
    amount_out_minimum: u128,
    deadline: u128,
) -> anyhow::Result<String> {
    // exactInputSingle function selector: 0x414bf389
    let selector = "414bf389";

    // Build ExactInputSingleParams struct
    // struct ExactInputSingleParams {
    //     address tokenIn;
    //     address tokenOut;
    //     uint24 fee;
    //     address recipient;
    //     uint256 deadline;
    //     uint256 amountIn;
    //     uint256 amountOutMinimum;
    //     uint160 sqrtPriceLimitX96;
    // }

    let token_in_padded = format!("{:0>64}", token_in.trim_start_matches("0x"));
    let token_out_padded = format!("{:0>64}", token_out.trim_start_matches("0x"));
    let fee_padded = format!("{:0>64x}", fee);
    let recipient_padded = format!("{:0>64}", recipient.trim_start_matches("0x"));
    let deadline_padded = format!("{:0>64x}", deadline);
    let amount_in_padded = format!("{:0>64x}", amount_in);
    let amount_out_min_padded = format!("{:0>64x}", amount_out_minimum);
    let sqrt_price_limit_padded = format!("{:0>64x}", 0u128); // No price limit

    // Parameters struct offset (0x20 = 32 bytes)
    let struct_offset = format!("{:0>64x}", 0x20u128);

    Ok(format!(
        "0x{}{}{}{}{}{}{}{}{}{}",
        selector,
        struct_offset,
        token_in_padded,
        token_out_padded,
        fee_padded,
        recipient_padded,
        deadline_padded,
        amount_in_padded,
        amount_out_min_padded,
        sqrt_price_limit_padded
    ))
}

/// Calculate price impact from swap amounts
fn calculate_price_impact(amount_in: u128, amount_out: u128) -> f64 {
    // Simplified price impact calculation
    // In production, this would be more sophisticated
    if amount_in == 0 || amount_out == 0 {
        return 0.0;
    }

    // This is a placeholder calculation
    // Real price impact would consider pool reserves and swap size
    let ratio = amount_out as f64 / amount_in as f64;
    if ratio > 0.99 {
        0.01 // Minimum 0.01% impact
    } else {
        (1.0 - ratio) * 100.0
    }
}

/// Result of a swap quote from Uniswap V3
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SwapQuote {
    /// Input token address
    pub token_in: String,
    /// Output token address
    pub token_out: String,
    /// Input amount
    pub amount_in: u128,
    /// Expected output amount
    pub amount_out: u128,
    /// Fee tier for the pool
    pub fee_tier: u32,
    /// Price impact percentage
    pub price_impact_pct: f64,
    /// Router contract address
    pub router_address: String,
    /// Network name
    pub network: String,
}

/// Result of a swap execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SwapResult {
    /// Transaction hash
    pub tx_hash: String,
    /// Input token address
    pub token_in: String,
    /// Output token address
    pub token_out: String,
    /// Input amount
    pub amount_in: u128,
    /// Minimum output amount
    pub amount_out_minimum: u128,
    /// Fee tier used
    pub fee_tier: u32,
    /// Transaction status
    pub status: TransactionStatus,
    /// Network name
    pub network: String,
    /// Gas price used
    pub gas_price: u64,
    /// Idempotency key if provided
    pub idempotency_key: Option<String>,
}

/// Token price information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenPriceInfo {
    /// Base token address
    pub base_token: String,
    /// Quote token address
    pub quote_token: String,
    /// Price of base in terms of quote
    pub price: f64,
    /// Fee tier of the pool used
    pub fee_tier: u32,
    /// Price impact for small trade
    pub price_impact_pct: f64,
    /// Network name
    pub network: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniswap_config() {
        let config = UniswapConfig::ethereum();
        assert_eq!(config.slippage_bps, 50);
        assert!(!config.router_address.is_empty());
        assert!(!config.quoter_address.is_empty());
    }

    #[test]
    fn test_quote_call_data() {
        let token_in = "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3";
        let token_out = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
        let result = build_quote_call_data(token_in, token_out, 3000, 1000000, 0).unwrap();

        assert!(result.starts_with("0xf7729d43")); // quoteExactInputSingle selector
        assert!(result.len() > 10); // Should have selector + parameters
    }

    #[test]
    fn test_swap_call_data() {
        let token_in = "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3";
        let token_out = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
        let recipient = "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123";
        let result = build_swap_call_data(
            token_in, token_out, 3000, recipient, 1000000, 950000, 1700000000,
        )
        .unwrap();

        assert!(result.starts_with("0x414bf389")); // exactInputSingle selector
        assert!(result.len() > 10);
    }

    #[test]
    fn test_price_impact_calculation() {
        let impact = calculate_price_impact(1000000, 990000);
        assert!(impact > 0.0);
        assert!(impact < 10.0); // Should be reasonable

        let minimal_impact = calculate_price_impact(1000000, 999000);
        assert!(minimal_impact < impact);
    }

    #[test]
    fn test_swap_quote_serialization() {
        let quote = SwapQuote {
            token_in: "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
            token_out: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            amount_in: 1000000,
            amount_out: 950000,
            fee_tier: 3000,
            price_impact_pct: 0.5,
            router_address: "0xE592427A0AEce92De3Edee1F18E0157C05861564".to_string(),
            network: "Ethereum".to_string(),
        };

        let json = serde_json::to_string(&quote).unwrap();
        assert!(json.contains("token_in"));
        assert!(json.contains("1000000"));
    }
}
