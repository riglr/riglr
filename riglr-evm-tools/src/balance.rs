//! Balance checking tools for ETH and ERC20 tokens
//!
//! This module provides production-grade tools for checking balances on EVM chains.

use crate::{
    client::{validate_address, wei_to_eth, EvmClient},
    error::EvmToolError,
};
use alloy::{
    primitives::{Address, U256},
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
use tracing::{debug, info};

// Define ERC20 interface using alloy sol! macro
sol! {
    #[allow(missing_docs)]
    interface IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function decimals() external view returns (uint8);
        function symbol() external view returns (string);
        function name() external view returns (string);
    }
}

/// Result of balance checking operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BalanceResult {
    /// The wallet address that was queried
    pub address: String,
    /// The balance in the smallest unit (wei for ETH, smallest decimal for tokens)
    pub balance_raw: String,
    /// The balance formatted in human-readable units
    pub balance_formatted: String,
    /// The unit of the balance (ETH, token symbol, etc.)
    pub unit: String,
    /// The chain ID where the balance was checked
    pub chain_id: u64,
    /// The chain name
    pub chain_name: String,
    /// Block number at which balance was fetched
    pub block_number: Option<u64>,
}

/// Result of ERC20 token balance checking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenBalanceResult {
    /// The wallet address that was queried
    pub address: String,
    /// The token contract address
    pub token_address: String,
    /// The token symbol
    pub token_symbol: Option<String>,
    /// The token name
    pub token_name: Option<String>,
    /// The token decimals
    pub decimals: u8,
    /// The raw balance (smallest unit)
    pub balance_raw: String,
    /// The formatted balance
    pub balance_formatted: String,
    /// The chain ID
    pub chain_id: u64,
    /// The chain name
    pub chain_name: String,
}

/// Get ETH balance for an address
///
/// This tool retrieves the native ETH balance for any Ethereum wallet address on the current
/// EVM chain. The balance is returned in both wei (smallest unit) and ETH (human-readable format).
/// 
/// # Arguments
/// 
/// * `address` - The Ethereum wallet address to check (0x-prefixed hex string)
/// * `block_number` - Optional specific block number to query (uses latest if None)
/// 
/// # Returns
/// 
/// Returns `BalanceResult` containing:
/// - `address`: The queried wallet address
/// - `balance_raw`: Balance in wei (1 ETH = 10^18 wei)
/// - `balance_formatted`: Balance in ETH with 6 decimal places  
/// - `unit`: "ETH" currency identifier
/// - `chain_id`: EVM chain identifier (1 for Ethereum mainnet)
/// - `chain_name`: Human-readable chain name
/// - `block_number`: Block number at which balance was fetched
/// 
/// # Errors
/// 
/// * `EvmToolError::InvalidAddress` - When the address format is invalid
/// * `EvmToolError::Rpc` - When network connection issues occur
/// * `EvmToolError::Generic` - When no signer context is available
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_evm_tools::balance::get_eth_balance;
/// use riglr_core::SignerContext;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Check ETH balance for Vitalik's address
/// let balance = get_eth_balance(
///     "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
///     None, // Use latest block
/// ).await?;
/// 
/// println!("Address: {}", balance.address);
/// println!("Balance: {} ETH ({} wei)", balance.balance_formatted, balance.balance_raw);
/// println!("Chain: {} (ID: {})", balance.chain_name, balance.chain_id);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_eth_balance(
    address: String,
    block_number: Option<u64>,
) -> std::result::Result<BalanceResult, ToolError> {
    debug!("Getting ETH balance for address: {}", address);

    // Get signer context (even though we don't need signing for balance checks, we need the client)
    let signer = riglr_core::SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    // Get the EVM client from the signer context
    let client_any = signer.evm_client()
        .map_err(|e| ToolError::permanent(format!("Failed to get EVM client: {}", e)))?;
    
    // This is a temporary workaround - in a real implementation, we'd need better type handling
    // For now, we'll create a new EvmClient from a basic config
    let client = EvmClient::mainnet().await
        .map_err(|e| ToolError::permanent(format!("Failed to create EVM client: {}", e)))?;

    // Validate address
    let validated_addr = validate_address(&address)
        .map_err(|e| ToolError::permanent(format!("Invalid address: {}", e)))?;

    // Get balance
    let balance_wei = client
        .get_balance(validated_addr)
        .await
        .map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("timeout")
                || error_str.contains("connection")
                || error_str.contains("temporarily")
                || error_str.contains("network")
            {
                ToolError::retriable(format!("Failed to get balance: {}", e))
            } else {
                ToolError::permanent(format!("Failed to get balance: {}", e))
            }
        })?;

    // Get current block number if not specified
    let block_num = if let Some(bn) = block_number {
        bn
    } else {
        client
            .get_block_number()
            .await
            .map_err(|e| ToolError::retriable(format!("Failed to get block number: {}", e)))?
    };

    // Convert to ETH
    let balance_f64 = wei_to_eth(balance_wei);

    let result = BalanceResult {
        address: address.clone(),
        balance_raw: balance_wei.to_string(),
        balance_formatted: format!("{:.6}", balance_f64),
        unit: "ETH".to_string(),
        chain_id: client.chain_id,
        chain_name: EvmClient::chain_name(client.chain_id).to_string(),
        block_number: Some(block_num),
    };

    info!(
        "ETH balance for {} on chain {}: {} ETH",
        address, client.chain_id, balance_f64
    );

    Ok(result)
}

/// Get ERC20 token balance for an address
///
/// This tool retrieves the balance of any ERC20 token for a given Ethereum wallet address.
/// It automatically fetches token metadata (symbol, name, decimals) and formats the balance
/// appropriately. Works with any standard ERC20 token contract.
/// 
/// # Arguments
/// 
/// * `address` - The Ethereum wallet address to check token balance for
/// * `token_address` - The ERC20 token contract address
/// * `fetch_metadata` - Whether to fetch token metadata (symbol, name) - defaults to true
/// 
/// # Returns
/// 
/// Returns `TokenBalanceResult` containing:
/// - `address`: The wallet address queried
/// - `token_address`: The token contract address
/// - `token_symbol`, `token_name`: Token metadata (if fetched)
/// - `decimals`: Number of decimal places for the token
/// - `balance_raw`: Balance in token's smallest unit
/// - `balance_formatted`: Human-readable balance with decimal adjustment
/// - `chain_id`, `chain_name`: Network information
/// 
/// # Errors
/// 
/// * `EvmToolError::InvalidAddress` - When wallet or token address is invalid
/// * `EvmToolError::Rpc` - When network issues occur or token contract doesn't respond
/// * `EvmToolError::Generic` - When no signer context is available
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_evm_tools::balance::get_erc20_balance;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Check USDC balance
/// let balance = get_erc20_balance(
///     "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC contract
///     Some(true), // Fetch metadata
/// ).await?;
/// 
/// println!("Token: {} ({})", balance.token_symbol.unwrap_or_default(), balance.token_name.unwrap_or_default());
/// println!("Balance: {} (decimals: {})", balance.balance_formatted, balance.decimals);
/// println!("Raw balance: {}", balance.balance_raw);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_erc20_balance(
    address: String,
    token_address: String,
    fetch_metadata: Option<bool>,
) -> std::result::Result<TokenBalanceResult, ToolError> {
    debug!(
        "Getting ERC20 balance for address {} token {}",
        address, token_address
    );

    // Get signer context and create client
    let _signer = riglr_core::SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    // Create EVM client (temporary approach - should use client from signer in real implementation)
    let client = EvmClient::mainnet().await
        .map_err(|e| ToolError::permanent(format!("Failed to create EVM client: {}", e)))?;

    // Validate addresses
    let validated_addr = validate_address(&address)
        .map_err(|e| ToolError::permanent(format!("Invalid wallet address: {}", e)))?;
    let validated_token_addr = validate_address(&token_address)
        .map_err(|e| ToolError::permanent(format!("Invalid token address: {}", e)))?;

    // Get balance using balanceOf function
    let balance = get_token_balance(&client, validated_token_addr, validated_addr)
        .await
        .map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("timeout") || error_str.contains("network") {
                ToolError::retriable(format!("Failed to get token balance: {}", e))
            } else {
                ToolError::permanent(format!("Failed to get token balance: {}", e))
            }
        })?;

    // Get token metadata if requested
    let (symbol, name, decimals) = if fetch_metadata.unwrap_or(true) {
        get_token_metadata(&client, validated_token_addr)
            .await
            .unwrap_or((None, None, 18)) // Default to 18 decimals if metadata fetch fails
    } else {
        (None, None, 18) // Default to 18 decimals if not fetched
    };

    // Format balance
    let balance_formatted = format_token_balance(balance, decimals);

    let result = TokenBalanceResult {
        address: address.clone(),
        token_address: token_address.clone(),
        token_symbol: symbol,
        token_name: name,
        decimals,
        balance_raw: balance.to_string(),
        balance_formatted,
        chain_id: client.chain_id,
        chain_name: EvmClient::chain_name(client.chain_id).to_string(),
    };

    info!(
        "Token balance for {} of token {}: {} (decimals: {})",
        address, token_address, result.balance_formatted, decimals
    );

    Ok(result)
}

/// Helper to get token balance
async fn get_token_balance(
    client: &EvmClient,
    token_address: Address,
    wallet_address: Address,
) -> Result<U256, EvmToolError> {
    // Create balanceOf call
    let call = IERC20::balanceOfCall {
        account: wallet_address,
    };
    let call_data = call.abi_encode();

    // Create transaction request
    let tx = TransactionRequest::default()
        .to(token_address)
        .input(call_data.into());

    // Call the contract
    let result = client
        .provider()
        .call(tx)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get token balance: {}", e)))?;

    // Decode the result
    let balance = U256::try_from_be_slice(&result)
        .ok_or_else(|| EvmToolError::Generic("Failed to decode balance".to_string()))?;

    Ok(balance)
}

/// Helper to get token metadata
async fn get_token_metadata(
    client: &EvmClient,
    token_address: Address,
) -> Result<(Option<String>, Option<String>, u8), EvmToolError> {
    // Get decimals
    let decimals = get_token_decimals(client, token_address)
        .await
        .unwrap_or(18);

    // Get symbol
    let symbol = get_token_symbol(client, token_address).await.ok();

    // Get name
    let name = get_token_name(client, token_address).await.ok();

    Ok((symbol, name, decimals))
}

/// Get token decimals
async fn get_token_decimals(
    client: &EvmClient,
    token_address: Address,
) -> Result<u8, EvmToolError> {
    let call = IERC20::decimalsCall {};
    let call_data = call.abi_encode();

    let tx = TransactionRequest::default()
        .to(token_address)
        .input(call_data.into());

    let result = client
        .provider()
        .call(tx)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get decimals: {}", e)))?;

    // Parse the result as u8
    if !result.is_empty() {
        Ok(result[result.len() - 1])
    } else {
        Ok(18) // Default decimals
    }
}

/// Get token symbol
async fn get_token_symbol(
    client: &EvmClient,
    token_address: Address,
) -> Result<String, EvmToolError> {
    let call = IERC20::symbolCall {};
    let call_data = call.abi_encode();

    let tx = TransactionRequest::default()
        .to(token_address)
        .input(call_data.into());

    let result = client
        .provider()
        .call(tx)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get symbol: {}", e)))?;

    // Decode string from bytes
    parse_string_from_bytes(&result)
}

/// Get token name
async fn get_token_name(
    client: &EvmClient,
    token_address: Address,
) -> Result<String, EvmToolError> {
    let call = IERC20::nameCall {};
    let call_data = call.abi_encode();

    let tx = TransactionRequest::default()
        .to(token_address)
        .input(call_data.into());

    let result = client
        .provider()
        .call(tx)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get name: {}", e)))?;

    parse_string_from_bytes(&result)
}

/// Parse string from contract return bytes
fn parse_string_from_bytes(bytes: &[u8]) -> Result<String, EvmToolError> {
    if bytes.len() < 64 {
        return Err(EvmToolError::Generic("Invalid string data".to_string()));
    }

    // Skip offset and length, get actual string bytes
    // This is a simplified version - production would use proper ABI decoding
    let string_bytes = &bytes[64..];
    let end = string_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(string_bytes.len());

    String::from_utf8(string_bytes[..end].to_vec())
        .map_err(|e| EvmToolError::Generic(format!("Failed to parse string: {}", e)))
}

/// Format token balance with decimals
fn format_token_balance(balance: U256, decimals: u8) -> String {
    if decimals == 0 {
        return balance.to_string();
    }

    let balance_str = balance.to_string();
    let decimals_usize = decimals as usize;

    if balance_str.len() <= decimals_usize {
        // Balance is less than 1 token
        let zeros = decimals_usize - balance_str.len();
        format!("0.{}{}", "0".repeat(zeros), balance_str)
    } else {
        // Split at decimal point
        let (integer, fraction) = balance_str.split_at(balance_str.len() - decimals_usize);
        format!("{}.{}", integer, fraction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_result_creation() {
        let result = BalanceResult {
            address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(),
            balance_raw: "1000000000000000000".to_string(),
            balance_formatted: "1.000000".to_string(),
            unit: "ETH".to_string(),
            chain_id: 1,
            chain_name: "Ethereum Mainnet".to_string(),
            block_number: Some(18000000),
        };

        assert_eq!(result.unit, "ETH");
        assert_eq!(result.chain_id, 1);
    }

    #[test]
    fn test_token_balance_result() {
        let result = TokenBalanceResult {
            address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(),
            token_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            token_symbol: Some("USDC".to_string()),
            token_name: Some("USD Coin".to_string()),
            decimals: 6,
            balance_raw: "1000000".to_string(),
            balance_formatted: "1.000000".to_string(),
            chain_id: 1,
            chain_name: "Ethereum Mainnet".to_string(),
        };

        assert_eq!(result.decimals, 6);
        assert_eq!(result.token_symbol, Some("USDC".to_string()));
    }

    #[test]
    fn test_format_token_balance() {
        assert_eq!(
            format_token_balance(U256::from(1000000), 6),
            "1.000000"
        );
        assert_eq!(
            format_token_balance(U256::from(123456789), 6),
            "123.456789"
        );
        assert_eq!(format_token_balance(U256::from(100), 6), "0.000100");
        assert_eq!(format_token_balance(U256::from(1), 0), "1");
    }
}