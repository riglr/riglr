//! Balance checking tools for EVM chains

use alloy::primitives::Address;
use riglr_core::{provider::ApplicationContext, ToolError};
use riglr_macros::tool;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, info};

/// Balance response for ETH
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthBalance {
    pub balance_formatted: String,
    pub unit: String,
    pub raw_balance: String,
    pub balance_raw: String, // Alias for raw_balance
    pub address: String,
    pub chain_name: String,
    pub chain_id: u64,
    pub block_number: u64,
}

/// Balance response for ERC20 tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub balance_formatted: String,
    pub token_symbol: Option<String>,
    pub raw_balance: String,
    pub decimals: u8,
    pub token_address: String,
}

/// Internal function to get ETH balance using ApplicationContext
pub async fn get_eth_balance_with_context(
    address: String,
    chain_id: Option<u64>,
    app_context: &ApplicationContext,
) -> Result<EthBalance, ToolError> {
    debug!("Getting ETH balance for address: {}", address);

    // Parse the address
    let addr = Address::from_str(&address)
        .map_err(|e| ToolError::permanent_string(format!("Invalid address: {}", e)))?;

    // Get Provider from the ApplicationContext's extensions
    let provider = app_context
        .get_extension::<Arc<dyn alloy::providers::Provider>>()
        .ok_or_else(|| ToolError::permanent_string("Provider not found in context".to_string()))?;

    // Get balance from the blockchain
    let balance = provider
        .get_balance(addr)
        .await
        .map_err(|e| ToolError::retriable_string(format!("Failed to get balance: {}", e)))?;

    // Get current block number
    let block_number = provider
        .get_block_number()
        .await
        .map_err(|e| ToolError::retriable_string(format!("Failed to get block number: {}", e)))?;

    // Format the balance (wei to ETH)
    let balance_in_eth = balance.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;

    // Default to chain_id 1 if not provided
    let chain_id = chain_id.unwrap_or(1);
    let chain_name = match chain_id {
        1 => "Ethereum Mainnet",
        137 => "Polygon",
        56 => "BSC",
        42161 => "Arbitrum",
        10 => "Optimism",
        _ => "Unknown Chain",
    }
    .to_string();

    info!("ETH balance for {}: {} ETH", address, balance_in_eth);

    Ok(EthBalance {
        balance_formatted: format!("{:.6} ETH", balance_in_eth),
        unit: "ETH".to_string(),
        raw_balance: balance.to_string(),
        balance_raw: balance.to_string(),
        address,
        chain_name,
        chain_id,
        block_number,
    })
}

/// Get ETH balance for an address
#[tool]
pub async fn get_eth_balance(
    address: String,
    chain_id: Option<u64>,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<EthBalance, ToolError> {
    get_eth_balance_with_context(address, chain_id, context).await
}

/// Internal function to get ERC20 token balance using ApplicationContext
pub async fn get_erc20_balance_with_context(
    token_address: String,
    wallet_address: String,
    _chain_id: Option<u64>,
    app_context: &ApplicationContext,
) -> Result<TokenBalance, ToolError> {
    debug!(
        "Getting ERC20 balance for wallet: {} token: {}",
        wallet_address, token_address
    );

    // Parse addresses
    let _token_addr = Address::from_str(&token_address)
        .map_err(|e| ToolError::permanent_string(format!("Invalid token address: {}", e)))?;
    let _wallet_addr = Address::from_str(&wallet_address)
        .map_err(|e| ToolError::permanent_string(format!("Invalid wallet address: {}", e)))?;

    // Get Provider from the ApplicationContext's extensions
    let _provider = app_context
        .get_extension::<Arc<dyn alloy::providers::Provider>>()
        .ok_or_else(|| ToolError::permanent_string("Provider not found in context".to_string()))?;

    // For now, return a placeholder. Full implementation would call the ERC20 contract
    // This would require building a contract call for balanceOf(address)

    info!(
        "ERC20 balance check for token {} wallet {}",
        token_address, wallet_address
    );

    Ok(TokenBalance {
        balance_formatted: "0.0".to_string(),
        token_symbol: Some("TOKEN".to_string()),
        raw_balance: "0".to_string(),
        decimals: 18,
        token_address,
    })
}

/// Get ERC20 token balance for an address
#[tool]
pub async fn get_erc20_balance(
    token_address: String,
    wallet_address: String,
    chain_id: Option<u64>,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<TokenBalance, ToolError> {
    get_erc20_balance_with_context(token_address, wallet_address, chain_id, context).await
}

/// Get token symbol from ERC20 contract
pub async fn get_token_symbol(
    _client: &dyn riglr_core::signer::EvmClient,
    token_address: Address,
) -> Result<String, ToolError> {
    debug!("Getting token symbol for: {:?}", token_address);

    // TODO: Implement actual ERC20 contract call to symbol() function
    // For now, return a placeholder
    Ok("SYMBOL".to_string())
}

/// Get token name from ERC20 contract
pub async fn get_token_name(
    _client: &dyn riglr_core::signer::EvmClient,
    token_address: Address,
) -> Result<String, ToolError> {
    debug!("Getting token name for: {:?}", token_address);

    // TODO: Implement actual ERC20 contract call to name() function
    // For now, return a placeholder
    Ok("Token Name".to_string())
}

/// Get token decimals from ERC20 contract
pub async fn get_token_decimals(
    _client: &dyn riglr_core::signer::EvmClient,
    token_address: Address,
) -> Result<u8, ToolError> {
    debug!("Getting token decimals for: {:?}", token_address);

    // TODO: Implement actual ERC20 contract call to decimals() function
    // For now, return standard 18 decimals
    Ok(18)
}
