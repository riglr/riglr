//! Network and blockchain query tools for EVM chains

use riglr_core::{provider::ApplicationContext, signer::SignerContext, ToolError};
use riglr_macros::tool;
use std::sync::Arc;
use tracing::{debug, info};

/// Internal function to get block number using ApplicationContext
///
/// This function implements smart chain_id resolution for future extensibility:
/// 1. Uses the explicit chain_id parameter if provided
/// 2. Falls back to the chain_id from the active EVM SignerContext if available
/// 3. Defaults to Ethereum mainnet (chain_id 1) as a final fallback
pub(crate) async fn get_block_number_with_context(
    chain_id: Option<u64>,
    app_context: &ApplicationContext,
) -> Result<u64, ToolError> {
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

    debug!(
        "Getting current block number for chain_id: {}",
        resolved_chain_id
    );

    // Get Provider from the ApplicationContext's extensions
    let provider = app_context
        .get_extension::<Arc<dyn alloy::providers::Provider>>()
        .ok_or_else(|| ToolError::permanent_string("Provider not found in context".to_string()))?;

    let block_number = provider
        .get_block_number()
        .await
        .map_err(|e| ToolError::retriable_string(format!("Failed to get block number: {}", e)))?;

    info!("Current block number: {}", block_number);
    Ok(block_number)
}

/// Get current block number
///
/// This tool implements smart chain ID resolution:
/// - If `chain_id` is provided, uses that specific chain
/// - If `chain_id` is None but there's an active EVM SignerContext, uses the signer's chain ID
/// - Otherwise defaults to Ethereum mainnet (chain_id = 1)
///
/// # Arguments
/// * `chain_id` - Optional chain ID. If None, attempts to resolve from SignerContext
/// * `context` - Application context containing provider and other extensions
#[tool]
pub async fn get_block_number(
    chain_id: Option<u64>,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<u64, ToolError> {
    get_block_number_with_context(chain_id, context).await
}

/// Internal function to get gas price using ApplicationContext
///
/// This function implements smart chain_id resolution for future extensibility:
/// 1. Uses the explicit chain_id parameter if provided
/// 2. Falls back to the chain_id from the active EVM SignerContext if available
/// 3. Defaults to Ethereum mainnet (chain_id 1) as a final fallback
pub(crate) async fn get_gas_price_with_context(
    chain_id: Option<u64>,
    app_context: &ApplicationContext,
) -> Result<String, ToolError> {
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

    debug!(
        "Getting current gas price for chain_id: {}",
        resolved_chain_id
    );

    // Get Provider from the ApplicationContext's extensions
    let provider = app_context
        .get_extension::<Arc<dyn alloy::providers::Provider>>()
        .ok_or_else(|| ToolError::permanent_string("Provider not found in context".to_string()))?;

    let gas_price = provider
        .get_gas_price()
        .await
        .map_err(|e| ToolError::retriable_string(format!("Failed to get gas price: {}", e)))?;

    // Convert to Gwei for readability
    let gas_price_gwei = gas_price.to_string().parse::<f64>().unwrap_or(0.0) / 1e9;
    let formatted_price = format!("{:.2} Gwei", gas_price_gwei);

    info!("Current gas price: {}", formatted_price);
    Ok(formatted_price)
}

/// Get gas price
///
/// This tool implements smart chain ID resolution:
/// - If `chain_id` is provided, uses that specific chain
/// - If `chain_id` is None but there's an active EVM SignerContext, uses the signer's chain ID
/// - Otherwise defaults to Ethereum mainnet (chain_id = 1)
///
/// # Arguments
/// * `chain_id` - Optional chain ID. If None, attempts to resolve from SignerContext
/// * `context` - Application context containing provider and other extensions
#[tool]
pub async fn get_gas_price(
    chain_id: Option<u64>,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<String, ToolError> {
    get_gas_price_with_context(chain_id, context).await
}
