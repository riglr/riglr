//! Network and blockchain query tools for EVM chains

use riglr_core::{provider::ApplicationContext, ToolError};
use riglr_macros::tool;
use std::sync::Arc;
use tracing::{debug, info};

/// Internal function to get block number using ApplicationContext
pub async fn get_block_number_with_context(
    _chain_id: Option<u64>,
    app_context: &ApplicationContext,
) -> Result<u64, ToolError> {
    debug!("Getting current block number");

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
#[tool]
pub async fn get_block_number(
    chain_id: Option<u64>,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<u64, ToolError> {
    get_block_number_with_context(chain_id, context).await
}

/// Internal function to get gas price using ApplicationContext
pub async fn get_gas_price_with_context(
    _chain_id: Option<u64>,
    app_context: &ApplicationContext,
) -> Result<String, ToolError> {
    debug!("Getting current gas price");

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
#[tool]
pub async fn get_gas_price(
    chain_id: Option<u64>,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<String, ToolError> {
    get_gas_price_with_context(chain_id, context).await
}
