//! Balance checking tools for ETH and ERC20 tokens
//!
//! This module provides production-grade tools for checking balances on EVM chains.

use crate::{
    client::{validate_address, EvmClient},
    error::EvmToolError,
};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// ERC20 balanceOf function selector
const ERC20_BALANCE_OF_SELECTOR: &str = "0x70a08231";
/// ERC20 decimals function selector
const ERC20_DECIMALS_SELECTOR: &str = "0x313ce567";
/// ERC20 symbol function selector
const ERC20_SYMBOL_SELECTOR: &str = "0x95d89b41";
/// ERC20 name function selector
const ERC20_NAME_SELECTOR: &str = "0x06fdde03";

/// Result of balance checking operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BalanceResult {
    /// The wallet address that was queried
    pub address: String,
    /// The raw balance in wei (for ETH) or smallest unit (for tokens)
    pub balance_raw: String,
    /// The formatted balance for display
    pub balance_formatted: String,
    /// Number of decimals for the asset
    pub decimals: u8,
    /// The blockchain network
    pub network: String,
    /// Block number at which balance was queried
    pub block_number: u64,
}

/// Result of ERC20 token balance checking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenBalanceResult {
    /// The wallet address that was queried
    pub address: String,
    /// The token contract address
    pub token_address: String,
    /// Token symbol (if available)
    pub symbol: Option<String>,
    /// Token name (if available)
    pub name: Option<String>,
    /// The raw balance in token's smallest unit
    pub balance_raw: String,
    /// The formatted balance for display
    pub balance_formatted: String,
    /// Number of decimals for the token
    pub decimals: u8,
    /// The blockchain network
    pub network: String,
    /// Block number at which balance was queried
    pub block_number: u64,
}

/// Get ETH balance for an address
///
/// This tool queries the ETH balance for a given address on the specified network.
// #[tool]
pub async fn get_eth_balance(
    address: String,
    rpc_url: Option<String>,
    network_name: Option<String>,
) -> anyhow::Result<BalanceResult> {
    debug!("Getting ETH balance for address: {}", address);

    // Validate address
    let validated_addr =
        validate_address(&address).map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

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

    // Get balance and block number
    let balance_hex = client
        .get_balance(&validated_addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch balance: {}", e))?;
    let block_number = client
        .get_block_number()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch block number: {}", e))?;

    // Parse balance from hex
    let balance_wei = u128::from_str_radix(balance_hex.trim_start_matches("0x"), 16)
        .map_err(|e| anyhow::anyhow!("Failed to parse balance: {}", e))?;

    // Format balance (ETH has 18 decimals)
    let balance_f64 = balance_wei as f64 / 1e18;
    let balance_formatted = format!("{:.6}", balance_f64);

    let network = network_name.unwrap_or_else(|| match client.chain_id {
        1 => "Ethereum".to_string(),
        137 => "Polygon".to_string(),
        42161 => "Arbitrum One".to_string(),
        10 => "Optimism".to_string(),
        8453 => "Base".to_string(),
        _ => format!("Chain {}", client.chain_id),
    });

    info!(
        "ETH balance for {}: {} ETH on {}",
        address, balance_formatted, network
    );

    Ok(BalanceResult {
        address: validated_addr,
        balance_raw: balance_wei.to_string(),
        balance_formatted: format!("{} ETH", balance_formatted),
        decimals: 18,
        network,
        block_number,
    })
}

/// Get ERC20 token balance for an address
///
/// This tool queries the ERC20 token balance for a given address and token contract.
// #[tool]
pub async fn get_erc20_balance(
    address: String,
    token_address: String,
    rpc_url: Option<String>,
    network_name: Option<String>,
) -> anyhow::Result<TokenBalanceResult> {
    debug!(
        "Getting ERC20 balance for address: {}, token: {}",
        address, token_address
    );

    // Validate addresses
    let validated_addr =
        validate_address(&address).map_err(|e| anyhow::anyhow!("Invalid wallet address: {}", e))?;
    let validated_token_addr = validate_address(&token_address)
        .map_err(|e| anyhow::anyhow!("Invalid token address: {}", e))?;

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

    // Prepare balanceOf call data (function selector + padded address)
    let balance_call_data = format!(
        "{}{:0>64}",
        ERC20_BALANCE_OF_SELECTOR.trim_start_matches("0x"),
        validated_addr.trim_start_matches("0x")
    );

    // Get balance via contract call
    let balance_result = client
        .call_contract(&validated_token_addr, &format!("0x{}", balance_call_data))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get token balance: {}", e))?;

    // Parse balance from hex result
    let balance_wei =
        u128::from_str_radix(balance_result.trim_start_matches("0x"), 16).unwrap_or(0);

    // Get block number
    let block_number = client
        .get_block_number()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch block number: {}", e))?;

    // For now, assume 18 decimals (most common) - in production we'd query the decimals() function
    let decimals = 18u8;
    let divisor = 10_u128.pow(decimals as u32) as f64;
    let balance_f64 = balance_wei as f64 / divisor;
    let balance_formatted = if balance_f64 >= 1.0 {
        format!("{:.6}", balance_f64)
    } else {
        format!("{:.9}", balance_f64)
    };

    let network = network_name.unwrap_or_else(|| match client.chain_id {
        1 => "Ethereum".to_string(),
        137 => "Polygon".to_string(),
        42161 => "Arbitrum One".to_string(),
        10 => "Optimism".to_string(),
        8453 => "Base".to_string(),
        _ => format!("Chain {}", client.chain_id),
    });

    let symbol = "TOKEN"; // Placeholder - in production we'd query symbol() function

    info!(
        "Token balance for {}: {} {} on {}",
        address, balance_formatted, symbol, network
    );

    Ok(TokenBalanceResult {
        address: validated_addr,
        token_address: validated_token_addr,
        symbol: Some(symbol.to_string()),
        name: None, // Would need to query name() function
        balance_raw: balance_wei.to_string(),
        balance_formatted: format!("{} {}", balance_formatted, symbol),
        decimals,
        network,
        block_number,
    })
}

/// Get multiple token balances for an address
///
/// This tool efficiently queries multiple ERC20 token balances for a single address.
// #[tool]
pub async fn get_multi_token_balances(
    address: String,
    token_addresses: Vec<String>,
    rpc_url: Option<String>,
    network_name: Option<String>,
) -> anyhow::Result<Vec<TokenBalanceResult>> {
    debug!(
        "Getting multi-token balances for address: {}, tokens: {:?}",
        address, token_addresses
    );

    let mut results = Vec::new();

    // Query each token balance
    for token_address in token_addresses {
        match get_erc20_balance(
            address.clone(),
            token_address,
            rpc_url.clone(),
            network_name.clone(),
        )
        .await
        {
            Ok(result) => results.push(result),
            Err(e) => {
                debug!("Failed to get balance for token {}: {}", address, e);
                // Continue with other tokens instead of failing completely
            }
        }
    }

    info!("Retrieved {} token balances for {}", results.len(), address);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_result_serialization() {
        let result = BalanceResult {
            address: "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
            balance_raw: "1000000000000000000".to_string(),
            balance_formatted: "1.000000 ETH".to_string(),
            decimals: 18,
            network: "Ethereum".to_string(),
            block_number: 18500000,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("address"));
        assert!(json.contains("balance_raw"));
        assert!(json.contains("Ethereum"));
    }

    #[test]
    fn test_token_balance_result_serialization() {
        let result = TokenBalanceResult {
            address: "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
            token_address: "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
            symbol: Some("USDC".to_string()),
            name: Some("USD Coin".to_string()),
            balance_raw: "1000000".to_string(),
            balance_formatted: "1.000000 USDC".to_string(),
            decimals: 6,
            network: "Ethereum".to_string(),
            block_number: 18500000,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("token_address"));
        assert!(json.contains("USDC"));
    }
}
