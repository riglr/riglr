//! Balance checking tools for EVM chains

use alloy::hex;
use alloy::primitives::Address;
use alloy::providers::Provider;
use riglr_core::{provider::ApplicationContext, signer::SignerContext, ToolError};
use riglr_evm_common::get_chain_info;
use riglr_macros::tool;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{debug, info};

use crate::error::EvmToolError;
use crate::provider::EvmProvider;

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
///
/// This function implements smart chain_id resolution:
/// 1. Uses the explicit chain_id parameter if provided
/// 2. Falls back to the chain_id from the active EVM SignerContext if available
/// 3. Defaults to Ethereum mainnet (chain_id 1) as a final fallback
pub(crate) async fn get_eth_balance_with_context(
    address: String,
    chain_id: Option<u64>,
    app_context: &ApplicationContext,
) -> Result<EthBalance, EvmToolError> {
    debug!("Getting ETH balance for address: {}", address);

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

    // Parse the address
    let addr = Address::from_str(&address)
        .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid address: {}", e)))?;

    // Get Provider from the ApplicationContext's extensions
    let provider = app_context
        .get_extension::<EvmProvider>()
        .ok_or_else(|| EvmToolError::ProviderError("Provider not found in context".to_string()))?;

    // Get balance from the blockchain
    let balance_result = (**provider).get_balance(addr);
    let balance = balance_result
        .await
        .map_err(|e| EvmToolError::ProviderError(format!("Failed to get balance: {}", e)))?;

    // Get current block number  
    let block_number_result = (**provider).get_block_number();
    let block_number = block_number_result
        .await
        .map_err(|e| EvmToolError::ProviderError(format!("Failed to get block number: {}", e)))?;

    // Format the balance (wei to ETH)
    let balance_in_eth = balance.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;
    let chain_name = get_chain_info(resolved_chain_id)
        .map(|info| info.name)
        .unwrap_or_else(|| "Unknown Chain".to_string());

    info!(
        "ETH balance for {} on chain {}: {} ETH",
        address, resolved_chain_id, balance_in_eth
    );

    Ok(EthBalance {
        balance_formatted: format!("{:.6} ETH", balance_in_eth),
        unit: "ETH".to_string(),
        raw_balance: balance.to_string(),
        balance_raw: balance.to_string(),
        address,
        chain_name,
        chain_id: resolved_chain_id,
        block_number,
    })
}

/// Get ETH balance for an address on an EVM-compatible chain
///
/// This tool implements smart chain ID resolution:
/// - If `chain_id` is provided, uses that specific chain
/// - If `chain_id` is None but there's an active EVM SignerContext, uses the signer's chain ID
/// - Otherwise defaults to Ethereum mainnet (chain_id = 1)
///
/// # Arguments
/// * `address` - The wallet address to check (hex format, with or without 0x prefix)
/// * `chain_id` - Optional chain ID. If None, attempts to resolve from SignerContext
/// * `context` - Application context containing provider and other extensions
///
/// # Examples
/// ```ignore
/// // Explicit chain ID
/// let balance = get_eth_balance("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb5".to_string(), Some(1), &context).await?;
///
/// // Auto-resolve from SignerContext
/// let balance = get_eth_balance("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb5".to_string(), None, &context).await?;
/// ```
#[tool]
pub async fn get_eth_balance(
    address: String,
    chain_id: Option<u64>,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<EthBalance, ToolError> {
    get_eth_balance_with_context(address, chain_id, context)
        .await
        .map_err(Into::into)
}

/// Internal function to get ERC20 token balance using ApplicationContext
///
/// This function implements smart chain_id resolution for future extensibility:
/// 1. Uses the explicit chain_id parameter if provided
/// 2. Falls back to the chain_id from the active EVM SignerContext if available
/// 3. Defaults to Ethereum mainnet (chain_id 1) as a final fallback
pub(crate) async fn get_erc20_balance_with_context(
    token_address: String,
    wallet_address: String,
    chain_id: Option<u64>,
    app_context: &ApplicationContext,
) -> Result<TokenBalance, EvmToolError> {
    debug!(
        "Getting ERC20 balance for wallet: {} token: {}",
        wallet_address, token_address
    );

    // Smart chain_id resolution with SignerContext fallback
    // Note: Currently resolved_chain_id is not used in the balance query,
    // but may be needed for future multi-chain token tracking or validation
    let _resolved_chain_id = if let Some(id) = chain_id {
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

    let _ = _resolved_chain_id; // Explicitly acknowledge unused variable for future use

    // Parse addresses
    let token_addr = Address::from_str(&token_address)
        .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid token address: {}", e)))?;
    let wallet_addr = Address::from_str(&wallet_address)
        .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid wallet address: {}", e)))?;

    // Get provider from ApplicationContext
    let provider = app_context
        .get_extension::<EvmProvider>()
        .ok_or_else(|| EvmToolError::ProviderError("Provider not found in context".to_string()))?;
    let client = &**provider;

    // Get token decimals first
    let decimals = get_token_decimals(client, token_addr).await?;

    // Get token symbol
    let token_symbol = get_token_symbol(client, token_addr).await.ok();

    // Build the balanceOf(address) call
    use alloy::dyn_abi::{DynSolType, DynSolValue};
    use alloy::primitives::Bytes;

    // ERC20 balanceOf function selector: 0x70a08231
    let function_selector = hex::decode("70a08231")
        .map_err(|e| EvmToolError::InvalidParameter(format!("Failed to decode selector: {}", e)))?;

    // Encode the wallet address as parameter
    let param = DynSolValue::Address(wallet_addr);
    let param_bytes = param.abi_encode_params();

    // Combine selector and parameters
    let mut calldata = Vec::with_capacity(4 + param_bytes.len());
    calldata.extend_from_slice(&function_selector);
    calldata.extend_from_slice(&param_bytes);

    // Make the eth_call
    let tx_request = <alloy::network::Ethereum as alloy::network::Network>::TransactionRequest::default()
        .to(token_addr)
        .input(Bytes::from(calldata).into());

    let result_future = client.call(&tx_request);
    let result = result_future
        .await
        .map_err(|e| EvmToolError::ProviderError(format!("Failed to call balanceOf: {}", e)))?;

    // Decode the result as uint256
    let balance_type = DynSolType::Uint(256);
    let decoded = balance_type
        .abi_decode(&result)
        .map_err(|e| EvmToolError::ContractError(format!("Failed to decode balance: {}", e)))?;

    let raw_balance = if let DynSolValue::Uint(value, _) = decoded {
        value
    } else {
        return Err(EvmToolError::ContractError(
            "Invalid balance format".to_string(),
        ));
    };

    // Format the balance with decimals
    let divisor = alloy::primitives::U256::from(10u64).pow(alloy::primitives::U256::from(decimals));
    let whole = raw_balance / divisor;
    let fraction = raw_balance % divisor;

    // Convert to human-readable format
    let balance_formatted = if fraction.is_zero() {
        whole.to_string()
    } else {
        // Format with appropriate decimal places
        let fraction_str = format!("{:0>width$}", fraction, width = decimals as usize);
        let trimmed_fraction = fraction_str.trim_end_matches('0');
        if trimmed_fraction.is_empty() {
            whole.to_string()
        } else {
            format!("{}.{}", whole, trimmed_fraction)
        }
    };

    info!(
        "ERC20 balance check for token {} wallet {}: {} (raw: {})",
        token_address, wallet_address, balance_formatted, raw_balance
    );

    Ok(TokenBalance {
        balance_formatted,
        token_symbol,
        raw_balance: raw_balance.to_string(),
        decimals,
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
    get_erc20_balance_with_context(token_address, wallet_address, chain_id, context)
        .await
        .map_err(Into::into)
}

/// Get token symbol from ERC20 contract  
pub async fn get_token_symbol(
    client: &alloy::providers::RootProvider<alloy::transports::http::Http<reqwest::Client>>,
    token_address: Address,
) -> Result<String, EvmToolError>
{
    debug!("Getting token symbol for: {:?}", token_address);

    use alloy::dyn_abi::{DynSolType, DynSolValue};
    use alloy::primitives::Bytes;

    // ERC20 symbol() function selector: 0x95d89b41
    let function_selector = hex::decode("95d89b41")
        .map_err(|e| EvmToolError::InvalidParameter(format!("Failed to decode selector: {}", e)))?;

    // No parameters for symbol()
    let calldata = function_selector;

    // Make the eth_call
    let tx_request = <alloy::network::Ethereum as alloy::network::Network>::TransactionRequest::default()
        .to(token_address)
        .input(Bytes::from(calldata).into());

    let result_future = client.call(&tx_request);
    let result = result_future
        .await
        .map_err(|e| EvmToolError::ProviderError(format!("Failed to call symbol: {}", e)))?;

    // Decode the result as string
    let string_type = DynSolType::String;
    let decoded = string_type
        .abi_decode(&result)
        .map_err(|e| EvmToolError::ContractError(format!("Failed to decode symbol: {}", e)))?;

    if let DynSolValue::String(symbol) = decoded {
        Ok(symbol)
    } else {
        Err(EvmToolError::ContractError(
            "Invalid symbol format".to_string(),
        ))
    }
}

/// Get token name from ERC20 contract
pub async fn get_token_name(
    client: &alloy::providers::RootProvider<alloy::transports::http::Http<reqwest::Client>>,
    token_address: Address,
) -> Result<String, EvmToolError>
{
    debug!("Getting token name for: {:?}", token_address);

    use alloy::dyn_abi::{DynSolType, DynSolValue};
    use alloy::primitives::Bytes;

    // ERC20 name() function selector: 0x06fdde03
    let function_selector = hex::decode("06fdde03")
        .map_err(|e| EvmToolError::InvalidParameter(format!("Failed to decode selector: {}", e)))?;

    // No parameters for name()
    let calldata = function_selector;

    // Make the eth_call
    let tx_request = <alloy::network::Ethereum as alloy::network::Network>::TransactionRequest::default()
        .to(token_address)
        .input(Bytes::from(calldata).into());

    let result_future = client.call(&tx_request);
    let result = result_future
        .await
        .map_err(|e| EvmToolError::ProviderError(format!("Failed to call name: {}", e)))?;

    // Decode the result as string
    let string_type = DynSolType::String;
    let decoded = string_type
        .abi_decode(&result)
        .map_err(|e| EvmToolError::ContractError(format!("Failed to decode name: {}", e)))?;

    if let DynSolValue::String(name) = decoded {
        Ok(name)
    } else {
        Err(EvmToolError::ContractError(
            "Invalid name format".to_string(),
        ))
    }
}

/// Get token decimals from ERC20 contract
pub async fn get_token_decimals(
    client: &alloy::providers::RootProvider<alloy::transports::http::Http<reqwest::Client>>,
    token_address: Address,
) -> Result<u8, EvmToolError>
{
    debug!("Getting token decimals for: {:?}", token_address);

    use alloy::dyn_abi::{DynSolType, DynSolValue};
    use alloy::primitives::Bytes;

    // ERC20 decimals() function selector: 0x313ce567
    let function_selector = hex::decode("313ce567")
        .map_err(|e| EvmToolError::InvalidParameter(format!("Failed to decode selector: {}", e)))?;

    // No parameters for decimals()
    let calldata = function_selector;

    // Make the eth_call
    let tx_request = <alloy::network::Ethereum as alloy::network::Network>::TransactionRequest::default()
        .to(token_address)
        .input(Bytes::from(calldata).into());

    let result_future = client.call(&tx_request);
    let result = result_future
        .await
        .map_err(|e| EvmToolError::ProviderError(format!("Failed to call decimals: {}", e)))?;

    // Decode the result as uint8
    let uint8_type = DynSolType::Uint(8);
    let decoded = uint8_type
        .abi_decode(&result)
        .map_err(|e| EvmToolError::ContractError(format!("Failed to decode decimals: {}", e)))?;

    if let DynSolValue::Uint(value, _) = decoded {
        // Check if value fits in u8
        if value <= alloy::primitives::U256::from(255u8) {
            Ok(value.to::<u8>())
        } else {
            Err(EvmToolError::ContractError(format!(
                "Decimals value {} too large for u8",
                value
            )))
        }
    } else {
        Err(EvmToolError::ContractError(
            "Invalid decimals format".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_config::ConfigBuilder;

    #[tokio::test]
    async fn test_get_eth_balance_with_fantom_chain() {
        // This tests that we correctly get chain name from common/chain.rs
        // Fantom (chain ID 250) was not in the original hardcoded list
        let config = ConfigBuilder::default().build().unwrap();
        let _context = ApplicationContext::from_config(&config);

        // Create a mock balance response structure
        let balance = EthBalance {
            balance_formatted: "1.0 ETH".to_string(),
            unit: "ETH".to_string(),
            raw_balance: "1000000000000000000".to_string(),
            balance_raw: "1000000000000000000".to_string(),
            address: "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e".to_string(),
            chain_name: get_chain_info(250)
                .map(|info| info.name)
                .unwrap_or_else(|| "Unknown Chain".to_string()),
            chain_id: 250,
            block_number: 12345678,
        };

        // Verify that Fantom chain name is correctly retrieved
        assert_eq!(balance.chain_name, "Fantom");
        assert_eq!(balance.chain_id, 250);
    }

    #[test]
    fn test_chain_name_resolution() {
        // Test various chain IDs to ensure they resolve correctly
        let test_cases = vec![
            (1, "Ethereum"),
            (137, "Polygon"),
            (56, "BNB Smart Chain"),
            (42161, "Arbitrum"),
            (10, "Optimism"),
            (250, "Fantom"),      // Not in original hardcoded list
            (43114, "Avalanche"), // Not in original hardcoded list
            (8453, "Base"),       // Not in original hardcoded list
        ];

        for (chain_id, expected_name) in test_cases {
            let chain_name = get_chain_info(chain_id)
                .map(|info| info.name)
                .unwrap_or_else(|| "Unknown Chain".to_string());

            assert_eq!(
                chain_name, expected_name,
                "Chain ID {} should map to {}",
                chain_id, expected_name
            );
        }
    }

    #[test]
    fn test_unknown_chain_fallback() {
        // Test that unknown chains fall back to "Unknown Chain"
        let chain_name = get_chain_info(999999)
            .map(|info| info.name)
            .unwrap_or_else(|| "Unknown Chain".to_string());

        assert_eq!(chain_name, "Unknown Chain");
    }
}
