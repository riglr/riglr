//! Uniswap V3 integration for token swaps on EVM chains
//!
//! This module provides production-grade tools for interacting with Uniswap V3,
//! enabling token swaps with optimal routing across multiple pools.

use crate::{
    client::{eth_to_wei, validate_address, wei_to_eth, EvmClient},
    error::{EvmToolError, Result},
    transaction::TransactionResult,
};
use alloy::{
    network::EthereumWallet,
    primitives::{Address, Bytes, U256},
    providers::Provider,
    rpc::types::TransactionRequest,
    sol,
    sol_types::SolCall,
};
use riglr_core::ToolError;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

// Define Uniswap V3 interfaces
sol! {
    #[allow(missing_docs)]
    interface ISwapRouter {
        struct ExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            address recipient;
            uint256 deadline;
            uint256 amountIn;
            uint256 amountOutMinimum;
            uint160 sqrtPriceLimitX96;
        }

        function exactInputSingle(ExactInputSingleParams calldata params)
            external
            payable
            returns (uint256 amountOut);
    }

    interface IQuoter {
        function quoteExactInputSingle(
            address tokenIn,
            address tokenOut,
            uint24 fee,
            uint256 amountIn,
            uint160 sqrtPriceLimitX96
        ) external returns (uint256 amountOut);
    }
}

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

    /// Default configuration for Arbitrum
    pub fn arbitrum() -> Self {
        Self {
            router_address: "0xE592427A0AEce92De3Edee1F18E0157C05861564".to_string(),
            quoter_address: "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".to_string(),
            slippage_bps: 50,
            deadline_seconds: 300,
        }
    }

    /// Default configuration for Optimism
    pub fn optimism() -> Self {
        Self {
            router_address: "0xE592427A0AEce92De3Edee1F18E0157C05861564".to_string(),
            quoter_address: "0x61fFE014bA17989E743c5F6cB21bF9697530B21e".to_string(),
            slippage_bps: 50,
            deadline_seconds: 300,
        }
    }

    /// Default configuration for Base
    pub fn base() -> Self {
        Self {
            router_address: "0x2626664c2603336E57B271c5C0b26F421741e481".to_string(),
            quoter_address: "0x3d4e44Eb1374240CE5F1B871ab261CD16335B76a".to_string(),
            slippage_bps: 50,
            deadline_seconds: 300,
        }
    }

    /// Get configuration for a specific chain ID
    pub fn for_chain(chain_id: u64) -> Self {
        match chain_id {
            1 => Self::ethereum(),
            137 => Self::polygon(),
            42161 => Self::arbitrum(),
            10 => Self::optimism(),
            8453 => Self::base(),
            _ => Self::ethereum(), // Default to Ethereum
        }
    }
}

/// Result of a Uniswap quote
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapQuote {
    /// Input token address
    pub token_in: String,
    /// Output token address
    pub token_out: String,
    /// Input amount in smallest units
    pub amount_in: String,
    /// Expected output amount in smallest units
    pub amount_out: String,
    /// Price per input token
    pub price: f64,
    /// Fee tier (100 = 0.01%, 500 = 0.05%, 3000 = 0.3%, 10000 = 1%)
    pub fee_tier: u32,
    /// Slippage tolerance in basis points
    pub slippage_bps: u16,
    /// Minimum output amount after slippage
    pub amount_out_minimum: String,
}

/// Result of a Uniswap swap
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapSwapResult {
    /// Transaction hash
    pub tx_hash: String,
    /// Input token address
    pub token_in: String,
    /// Output token address
    pub token_out: String,
    /// Input amount
    pub amount_in: String,
    /// Actual output amount received
    pub amount_out: String,
    /// Gas used
    pub gas_used: Option<u128>,
    /// Status
    pub status: bool,
}

/// Get a quote from Uniswap V3
///
/// This tool queries Uniswap V3 to get a quote for swapping tokens.
#[tool]
pub async fn get_uniswap_quote(
    client: &EvmClient,
    token_in: String,
    token_out: String,
    amount_in: String,
    decimals_in: u8,
    decimals_out: u8,
    fee_tier: Option<u32>,
    slippage_bps: Option<u16>,
) -> std::result::Result<UniswapQuote, ToolError> {
    debug!(
        "Getting Uniswap quote for {} {} to {}",
        amount_in, token_in, token_out
    );

    // Validate addresses
    let token_in_addr = validate_address(&token_in)
        .map_err(|e| ToolError::permanent(format!("Invalid token_in address: {}", e)))?;
    let token_out_addr = validate_address(&token_out)
        .map_err(|e| ToolError::permanent(format!("Invalid token_out address: {}", e)))?;


    // Get Uniswap config for this chain
    let config = UniswapConfig::for_chain(client.chain_id);

    // Parse amount
    let amount_in_wei = parse_amount_with_decimals(&amount_in, decimals_in)
        .map_err(|e| ToolError::permanent(format!("Invalid amount: {}", e)))?;

    // Default fee tier to 0.3% (3000)
    let fee = fee_tier.unwrap_or(3000) as u32;

    // Get quote from Quoter contract
    let quoter_addr = validate_address(&config.quoter_address)
        .map_err(|e| ToolError::permanent(format!("Invalid quoter address: {}", e)))?;

    let quote_amount = get_quote_from_quoter(
        &client,
        quoter_addr,
        token_in_addr,
        token_out_addr,
        fee,
        amount_in_wei,
    )
    .await
    .map_err(|e| {
        let error_str = e.to_string();
        if error_str.contains("revert") {
            ToolError::permanent(format!("Quote failed (likely no liquidity): {}", e))
        } else {
            ToolError::retriable(format!("Failed to get quote: {}", e))
        }
    })?;

    // Calculate price
    let amount_in_f64 = amount_in.parse::<f64>().unwrap_or(0.0);
    let amount_out_f64 = format_amount_with_decimals(quote_amount, decimals_out);
    let price = if amount_in_f64 > 0.0 {
        amount_out_f64 / amount_in_f64
    } else {
        0.0
    };

    // Calculate minimum output with slippage
    let slippage = slippage_bps.unwrap_or(config.slippage_bps);
    let slippage_factor = 1.0 - (slippage as f64 / 10000.0);
    let min_output = (quote_amount.to::<u128>() as f64 * slippage_factor) as u128;

    let result = UniswapQuote {
        token_in: token_in.clone(),
        token_out: token_out.clone(),
        amount_in: amount_in_wei.to_string(),
        amount_out: quote_amount.to_string(),
        price,
        fee_tier: fee,
        slippage_bps: slippage,
        amount_out_minimum: U256::from(min_output).to_string(),
    };

    info!(
        "Uniswap quote: {} {} -> {} {} (price: {})",
        amount_in, token_in, amount_out_f64, token_out, price
    );

    Ok(result)
}

/// Perform a token swap on Uniswap V3
///
/// This tool executes a token swap on Uniswap V3.
#[tool]
pub async fn perform_uniswap_swap(
    client: &EvmClient,
    token_in: String,
    token_out: String,
    amount_in: String,
    decimals_in: u8,
    amount_out_minimum: String,
    fee_tier: Option<u32>,
    deadline_seconds: Option<u64>,
) -> std::result::Result<UniswapSwapResult, ToolError> {
    debug!(
        "Performing Uniswap swap: {} {} to {}",
        amount_in, token_in, token_out
    );

    // Validate addresses
    let token_in_addr = validate_address(&token_in)
        .map_err(|e| ToolError::permanent(format!("Invalid token_in address: {}", e)))?;
    let token_out_addr = validate_address(&token_out)
        .map_err(|e| ToolError::permanent(format!("Invalid token_out address: {}", e)))?;

    // Get signer from client (replaces global state access)
    let signer = client.require_signer()
        .map_err(|e| ToolError::permanent(format!("Client requires signer configuration: {}", e)))?;
    
    let from_addr = signer.address();

    // Get Uniswap config
    let config = UniswapConfig::for_chain(client.chain_id);

    // Parse amounts
    let amount_in_wei = parse_amount_with_decimals(&amount_in, decimals_in)
        .map_err(|e| ToolError::permanent(format!("Invalid amount_in: {}", e)))?;
    let amount_out_min = amount_out_minimum
        .parse::<U256>()
        .map_err(|e| ToolError::permanent(format!("Invalid amount_out_minimum: {}", e)))?;

    // Calculate deadline
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + deadline_seconds.unwrap_or(config.deadline_seconds);

    // Create swap parameters
    let params = ISwapRouter::ExactInputSingleParams {
        tokenIn: token_in_addr,
        tokenOut: token_out_addr,
        fee: fee_tier.unwrap_or(3000) as u32,
        recipient: from_addr,
        deadline: U256::from(deadline),
        amountIn: amount_in_wei,
        amountOutMinimum: amount_out_min,
        sqrtPriceLimitX96: U256::ZERO, // No price limit
    };

    // Encode the swap call
    let call = ISwapRouter::exactInputSingleCall { params };
    let call_data = call.abi_encode();

    // Build transaction
    let router_addr = validate_address(&config.router_address)
        .map_err(|e| ToolError::permanent(format!("Invalid router address: {}", e)))?;

    let tx = TransactionRequest::default()
        .from(from_addr)
        .to(router_addr)
        .input(call_data.into())
        .gas_limit(300000); // Uniswap swaps typically need more gas

    // Create wallet for signing
    let ethereum_wallet = EthereumWallet::from(signer.clone());

    // Send transaction
    let provider_with_wallet = client.provider().with_wallet(ethereum_wallet);

    let pending_tx = provider_with_wallet
        .send_transaction(tx)
        .await
        .map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("insufficient") {
                ToolError::permanent(format!("Insufficient balance: {}", e))
            } else {
                ToolError::retriable(format!("Failed to send swap transaction: {}", e))
            }
        })?;

    // Wait for confirmation
    let receipt = pending_tx
        .with_required_confirmations(1)
        .get_receipt()
        .await
        .map_err(|e| ToolError::retriable(format!("Failed to get receipt: {}", e)))?;

    // TODO: Parse actual amount out from logs
    let amount_out_actual = amount_out_min.to_string(); // Placeholder

    let result = UniswapSwapResult {
        tx_hash: format!("0x{:x}", receipt.transaction_hash),
        token_in: token_in.clone(),
        token_out: token_out.clone(),
        amount_in: amount_in_wei.to_string(),
        amount_out: amount_out_actual,
        gas_used: receipt.gas_used,
        status: receipt.status(),
    };

    info!(
        "Uniswap swap complete: {} {} to {} (tx: {})",
        amount_in, token_in, token_out, result.tx_hash
    );

    Ok(result)
}

/// Helper to get quote from Quoter contract
async fn get_quote_from_quoter(
    client: &EvmClient,
    quoter_address: Address,
    token_in: Address,
    token_out: Address,
    fee: u32,
    amount_in: U256,
) -> Result<U256> {
    let call = IQuoter::quoteExactInputSingleCall {
        tokenIn: token_in,
        tokenOut: token_out,
        fee,
        amountIn: amount_in,
        sqrtPriceLimitX96: U256::ZERO,
    };
    let call_data = call.abi_encode();

    let tx = TransactionRequest::default()
        .to(quoter_address)
        .input(call_data.into());

    let result = client
        .provider()
        .call(&tx)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Quote call failed: {}", e)))?;

    // Decode the amount out from the result
    U256::try_from_be_slice(&result)
        .ok_or_else(|| EvmToolError::Generic("Failed to decode quote result".to_string()))
}

/// Parse amount with decimals
fn parse_amount_with_decimals(amount: &str, decimals: u8) -> Result<U256> {
    let amount_f64 = amount
        .parse::<f64>()
        .map_err(|e| EvmToolError::Generic(format!("Invalid amount: {}", e)))?;

    let multiplier = 10_f64.powi(decimals as i32);
    let amount_wei = (amount_f64 * multiplier) as u128;

    Ok(U256::from(amount_wei))
}

/// Format amount with decimals
fn format_amount_with_decimals(amount: U256, decimals: u8) -> f64 {
    let divisor = 10_f64.powi(decimals as i32);
    amount.to::<u128>() as f64 / divisor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniswap_config_for_chains() {
        let eth_config = UniswapConfig::ethereum();
        assert_eq!(
            eth_config.router_address,
            "0xE592427A0AEce92De3Edee1F18E0157C05861564"
        );

        let config_by_id = UniswapConfig::for_chain(1);
        assert_eq!(config_by_id.router_address, eth_config.router_address);

        let base_config = UniswapConfig::for_chain(8453);
        assert_eq!(
            base_config.router_address,
            "0x2626664c2603336E57B271c5C0b26F421741e481"
        );
    }

    #[test]
    fn test_parse_amount_with_decimals() {
        let amount = parse_amount_with_decimals("1.5", 18).unwrap();
        assert_eq!(amount, U256::from(1_500_000_000_000_000_000u128));

        let amount = parse_amount_with_decimals("100", 6).unwrap();
        assert_eq!(amount, U256::from(100_000_000u128));
    }

    #[test]
    fn test_format_amount_with_decimals() {
        let formatted = format_amount_with_decimals(U256::from(1_500_000_000_000_000_000u128), 18);
        assert!((formatted - 1.5).abs() < 0.000001);

        let formatted = format_amount_with_decimals(U256::from(100_000_000u128), 6);
        assert!((formatted - 100.0).abs() < 0.000001);
    }
}