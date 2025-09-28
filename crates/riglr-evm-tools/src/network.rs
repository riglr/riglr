//! Network and blockchain query tools for EVM chains

use crate::provider::HttpClient;
use alloy::providers::Provider as _;
use riglr_core::{provider::ApplicationContext, signer::SignerContext, ToolError};
use riglr_macros::tool;
use tracing::{debug, info};

/// Resolve chain ID from optional parameter or signer context
pub(crate) fn resolve_chain_id(chain_id: Option<u64>) -> u64 {
    chain_id.map_or_else(
        || {
            if let Ok(signer) = SignerContext::current_as_evm() {
                let chain_id_value = signer.signer().chain_id();
                debug!("Using `chain_id` from EVM `SignerContext`: {chain_id_value}");
                return chain_id_value;
            }
            debug!(
                "No explicit `chain_id` or EVM signer context, defaulting to Ethereum mainnet (1)"
            );
            1_u64 // Fallback to Ethereum mainnet
        },
        |chain_id_value| {
            debug!("Using explicit `chain_id`: {chain_id_value}");
            chain_id_value
        },
    )
}

/// Get current block number
///
/// This tool implements smart chain ID resolution:
/// - If `chain_id` is provided, uses that specific chain
/// - If `chain_id` is None but there's an active EVM `SignerContext`, uses the signer's chain ID
/// - Otherwise defaults to Ethereum mainnet (`chain_id` = 1)
///
/// # Arguments
/// * `chain_id` - Optional chain ID. If None, attempts to resolve from `SignerContext`
/// * `context` - Application context containing provider and other extensions
///
/// # Errors
/// Returns error if provider is not found in context or blockchain query fails
#[inline]
#[tool]
pub async fn get_block_number(
    chain_id: Option<u64>,
    context: &ApplicationContext,
) -> Result<u64, ToolError> {
    let resolved_chain_id = resolve_chain_id(chain_id);

    debug!("Getting current block number for `chain_id`: {resolved_chain_id}");

    // Get Provider from the ApplicationContext's extensions
    let provider = context
        .get_extension::<HttpClient>()
        .ok_or_else(|| ToolError::permanent_string("Provider not found in context".to_owned()))?;

    let block_number = provider.get_block_number().await.map_err(|error| {
        ToolError::retriable_string(format!("Failed to get block number: {error}"))
    })?;

    info!("Current block number: {block_number}");
    Ok(block_number)
}

/// Get gas price
///
/// This tool implements smart chain ID resolution:
/// - If `chain_id` is provided, uses that specific chain
/// - If `chain_id` is None but there's an active EVM `SignerContext`, uses the signer's chain ID
/// - Otherwise defaults to Ethereum mainnet (`chain_id` = 1)
///
/// # Arguments
/// * `chain_id` - Optional chain ID. If None, attempts to resolve from `SignerContext`
/// * `context` - Application context containing provider and other extensions
///
/// # Errors
/// Returns error if provider is not found in context or blockchain query fails
#[inline]
#[tool]
pub async fn get_gas_price(
    chain_id: Option<u64>,
    context: &ApplicationContext,
) -> Result<String, ToolError> {
    let resolved_chain_id = resolve_chain_id(chain_id);

    debug!("Getting current gas price for `chain_id`: {resolved_chain_id}");

    // Get Provider from the ApplicationContext's extensions
    let provider = context
        .get_extension::<HttpClient>()
        .ok_or_else(|| ToolError::permanent_string("Provider not found in context".to_owned()))?;

    let gas_price = provider.get_gas_price().await.map_err(|error| {
        ToolError::retriable_string(format!("Failed to get gas price: {error}"))
    })?;

    // Convert to Gwei for readability
    let gas_price_gwei = gas_price.to_string().parse::<f64>().unwrap_or(0.0_f64) / 1e9_f64;
    let formatted_price = format!("{gas_price_gwei:.2} Gwei");

    info!("Current gas price: {formatted_price}");
    Ok(formatted_price)
}
