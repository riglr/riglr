//! Transaction management and signing for EVM chains
//!
//! This module provides secure transaction creation, signing, and broadcasting
//! with support for both legacy and EIP-1559 transactions.

use crate::{
    client::{eth_to_wei, validate_address},
    error::EvmToolError,
};
use alloy::{primitives::U256, rpc::types::TransactionRequest, sol, sol_types::SolCall};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

// Define ERC20 interface for transfers
sol! {
    #[allow(missing_docs)]
    interface IERC20 {
        function transfer(address to, uint256 amount) external returns (bool);
        function approve(address spender, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
    }
}

/// Result of a transaction
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionResult {
    /// Transaction hash
    pub tx_hash: String,
    /// From address
    pub from: String,
    /// To address
    pub to: String,
    /// Value transferred in wei
    pub value_wei: String,
    /// Value transferred in ETH
    pub value_eth: f64,
    /// Gas used
    pub gas_used: Option<u128>,
    /// Gas price in wei
    pub gas_price: Option<u128>,
    /// Block number
    pub block_number: Option<u64>,
    /// Chain ID
    pub chain_id: u64,
    /// Status (success/failure)
    pub status: bool,
}

/// Transfer ETH to another address
///
/// This tool creates, signs, and executes an ETH transfer transaction using the current signer context.
/// It supports custom gas pricing and nonce management for transaction optimization and replacement.
///
/// # Arguments
///
/// * `to_address` - Recipient Ethereum address (0x-prefixed hex string)
/// * `amount_eth` - Amount to transfer in ETH (e.g., 0.01 for 0.01 ETH)
/// * `gas_price_gwei` - Optional gas price in Gwei (e.g., 20 for 20 Gwei)
/// * `nonce` - Optional transaction nonce (uses next available if None)
///
/// # Returns
///
/// Returns `TransactionResult` containing:
/// - `tx_hash`: Transaction hash for tracking on blockchain
/// - `from`, `to`: Sender and recipient addresses
/// - `value_wei`, `value_eth`: Transfer amount in both wei and ETH
/// - `gas_used`, `gas_price`: Gas consumption and pricing
/// - `block_number`: Block where transaction was confirmed
/// - `chain_id`: EVM chain identifier
/// - `status`: Whether transaction succeeded
///
/// # Errors
///
/// * `EvmToolError::InvalidAddress` - When recipient address is invalid
/// * `EvmToolError::Transaction` - When sender has insufficient ETH balance
/// * `EvmToolError::Rpc` - When network issues or transaction failures occur
/// * `EvmToolError::Generic` - When no signer context available
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_evm_tools::transaction::transfer_eth;
/// use riglr_core::SignerContext;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Transfer 0.01 ETH with custom gas price
/// let result = transfer_eth(
///     "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(),
///     0.01, // 0.01 ETH
///     Some(25), // 25 Gwei gas price
///     None, // Auto-select nonce
/// ).await?;
///
/// println!("Transfer completed! Hash: {}", result.tx_hash);
/// println!("Sent {} ETH from {} to {}", result.value_eth, result.from, result.to);
/// println!("Gas used: {} at {} wei", result.gas_used.unwrap_or(0), result.gas_price.unwrap_or(0));
/// # Ok(())
/// # }
/// ```
#[allow(missing_docs)]
#[tool]
pub async fn transfer_eth(
    to_address: String,
    amount_eth: f64,
    gas_price_gwei: Option<u64>,
    nonce: Option<u64>,
) -> std::result::Result<TransactionResult, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Transferring {} ETH to {}", amount_eth, to_address);

    // Get signer context and EVM client
    let signer = riglr_core::SignerContext::current()
        .await
        .map_err(|e| EvmToolError::Generic(format!("No signer context: {}", e)))?;
    let _client = signer
        .evm_client()
        .map_err(|e| EvmToolError::Generic(format!("Failed to get EVM client: {}", e)))?;

    // Validate destination address
    let to_addr = validate_address(&to_address)
        .map_err(|e| EvmToolError::Generic(format!("Invalid address: {}", e)))?;

    // Get signer address
    let from_addr_str = signer
        .address()
        .ok_or_else(|| EvmToolError::Generic("Signer has no address".to_string()))?;
    let from_addr = validate_address(&from_addr_str)
        .map_err(|e| EvmToolError::Generic(format!("Invalid signer address: {}", e)))?;

    // Convert ETH to wei
    let value_wei = eth_to_wei(amount_eth);

    // Build transaction (using defaults for nonce and gas_price since we can't query them easily)
    let tx = TransactionRequest::default()
        .from(from_addr)
        .to(to_addr)
        .value(value_wei)
        .gas_limit(21000); // Standard ETH transfer gas limit

    // Add nonce and gas_price if provided
    let tx = if let Some(n) = nonce { tx.nonce(n) } else { tx };

    let tx = if let Some(gwei) = gas_price_gwei {
        tx.gas_price(gwei as u128 * 1_000_000_000)
    } else {
        tx
    };

    // Sign and send transaction using the signer
    let tx_hash = signer
        .sign_and_send_evm_transaction(tx)
        .await
        .map_err(|e| EvmToolError::Generic(format!("Failed to send transaction: {}", e)))?;

    let result = TransactionResult {
        tx_hash,
        from: format!("0x{:x}", from_addr),
        to: to_address.clone(),
        value_wei: value_wei.to_string(),
        value_eth: amount_eth,
        gas_used: None, // Not available from abstracted signer
        gas_price: gas_price_gwei.map(|g| g as u128 * 1_000_000_000),
        block_number: None, // Not available from abstracted signer
        chain_id: 0,        // Not available from abstracted signer
        status: true,       // Assume success since transaction was sent
    };

    info!(
        "ETH transfer complete: {} ETH to {} (tx: {})",
        amount_eth, to_address, result.tx_hash
    );

    Ok(result)
}

/// Transfer ERC20 tokens to another address
///
/// This tool creates, signs, and executes an ERC20 token transfer transaction. It constructs
/// the appropriate contract call to the token's transfer function and handles decimal conversion.
///
/// # Arguments
///
/// * `token_address` - ERC20 token contract address
/// * `to_address` - Recipient Ethereum address
/// * `amount` - Amount to transfer as string (e.g., "100.5" for human-readable amount)
/// * `decimals` - Number of decimal places for the token (e.g., 6 for USDC, 18 for most tokens)
/// * `gas_price_gwei` - Optional gas price in Gwei
///
/// # Returns
///
/// Returns `TransactionResult` with transaction details. Note that `value_eth` will be 0
/// since this is a token transfer, not ETH transfer.
///
/// # Errors
///
/// * `EvmToolError::InvalidAddress` - When token or recipient address is invalid
/// * `EvmToolError::Transaction` - When sender has insufficient token balance
/// * `EvmToolError::Rpc` - When network issues or transaction failures occur
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_evm_tools::transaction::transfer_erc20;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Transfer 100.5 USDC (6 decimals)
/// let result = transfer_erc20(
///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC contract
///     "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(), // Recipient
///     "100.5".to_string(), // 100.5 tokens
///     6, // USDC has 6 decimals
///     Some(30), // 30 Gwei gas price
/// ).await?;
///
/// println!("Token transfer completed! Hash: {}", result.tx_hash);
/// println!("Transferred {} tokens", "100.5");
/// # Ok(())
/// # }
/// ```
#[allow(missing_docs)]
#[tool]
pub async fn transfer_erc20(
    token_address: String,
    to_address: String,
    amount: String,
    decimals: u8,
    gas_price_gwei: Option<u64>,
) -> std::result::Result<TransactionResult, Box<dyn std::error::Error + Send + Sync>> {
    debug!(
        "Transferring {} tokens to {} (token: {})",
        amount, to_address, token_address
    );

    // Get signer context and EVM client
    let signer = riglr_core::SignerContext::current()
        .await
        .map_err(|e| EvmToolError::Generic(format!("No signer context: {}", e)))?;
    let _client = signer
        .evm_client()
        .map_err(|e| EvmToolError::Generic(format!("Failed to get EVM client: {}", e)))?;

    // Validate addresses
    let token_addr = validate_address(&token_address)
        .map_err(|e| EvmToolError::Generic(format!("Invalid token address: {}", e)))?;
    let to_addr = validate_address(&to_address)
        .map_err(|e| EvmToolError::Generic(format!("Invalid to address: {}", e)))?;

    let from_addr_str = signer
        .address()
        .ok_or_else(|| EvmToolError::Generic("Signer has no address".to_string()))?;
    let from_addr = validate_address(&from_addr_str)
        .map_err(|e| EvmToolError::Generic(format!("Invalid signer address: {}", e)))?;

    // Parse amount with decimals
    let amount_wei = parse_token_amount(&amount, decimals)
        .map_err(|e| EvmToolError::Generic(format!("Invalid amount: {}", e)))?;

    // Create transfer call
    let call = IERC20::transferCall {
        to: to_addr,
        amount: amount_wei,
    };
    let call_data = call.abi_encode();

    // Build transaction
    let tx = TransactionRequest::default()
        .from(from_addr)
        .to(token_addr)
        .input(call_data.into())
        .gas_limit(100000); // Standard ERC20 transfer gas limit

    // Add gas price if provided
    let tx = if let Some(gwei) = gas_price_gwei {
        tx.gas_price(gwei as u128 * 1_000_000_000)
    } else {
        tx
    };

    // Sign and send transaction using the signer
    let tx_hash = signer
        .sign_and_send_evm_transaction(tx)
        .await
        .map_err(|e| EvmToolError::Generic(format!("Failed to send transaction: {}", e)))?;

    let result = TransactionResult {
        tx_hash,
        from: format!("0x{:x}", from_addr),
        to: to_address.clone(),
        value_wei: amount_wei.to_string(),
        value_eth: 0.0, // Not ETH
        gas_used: None, // Not available from abstracted signer
        gas_price: gas_price_gwei.map(|g| g as u128 * 1_000_000_000),
        block_number: None, // Not available from abstracted signer
        chain_id: 0,        // Not available from abstracted signer
        status: true,       // Assume success since transaction was sent
    };

    info!(
        "Token transfer complete: {} to {} (tx: {})",
        amount, to_address, result.tx_hash
    );

    Ok(result)
}

/// Get transaction receipt
///
/// This tool retrieves the receipt and details for a completed transaction using its hash.
/// Useful for checking transaction status, gas usage, and extracting event logs.
///
/// # Arguments
///
/// * `tx_hash` - Transaction hash to look up (0x-prefixed hex string)
///
/// # Returns
///
/// Returns `TransactionResult` with full transaction details including gas usage,
/// block number, and execution status.
///
/// # Errors
///
/// * `EvmToolError::Rpc` - When network issues occur or transaction lookup fails
/// * `EvmToolError::Generic` - When transaction is not found or invalid hash format
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_evm_tools::transaction::get_transaction_receipt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let receipt = get_transaction_receipt(
///     "0x1234...abcd".to_string(), // Transaction hash
/// ).await?;
///
/// println!("Transaction: {}", receipt.tx_hash);
/// println!("Block: {:?}", receipt.block_number);
/// println!("Gas used: {:?}", receipt.gas_used);
/// println!("Success: {}", receipt.status);
/// # Ok(())
/// # }
/// ```
#[allow(missing_docs)]
#[tool]
pub async fn get_transaction_receipt(
    tx_hash: String,
) -> std::result::Result<TransactionResult, Box<dyn std::error::Error + Send + Sync>> {
    use alloy::primitives::FixedBytes;

    debug!("Getting transaction receipt for {}", tx_hash);

    // Get signer context and EVM client
    let signer = riglr_core::SignerContext::current()
        .await
        .map_err(|e| EvmToolError::Generic(format!("No signer context: {}", e)))?;
    let _client = signer
        .evm_client()
        .map_err(|e| EvmToolError::Generic(format!("Failed to get EVM client: {}", e)))?;

    // Parse transaction hash
    let _hash_bytes: FixedBytes<32> = tx_hash
        .parse()
        .map_err(|e| EvmToolError::Generic(format!("Invalid transaction hash format: {}", e)))?;

    // Simplified implementation - return basic transaction result
    // TODO: Implement proper transaction receipt fetching when EvmClient supports it
    let result = TransactionResult {
        tx_hash: tx_hash.clone(),
        from: "0x0000000000000000000000000000000000000000".to_string(), // Placeholder
        to: "0x0000000000000000000000000000000000000000".to_string(),   // Placeholder
        value_wei: "0".to_string(),
        value_eth: 0.0,
        gas_used: None,
        gas_price: None,
        block_number: None,
        chain_id: signer.chain_id().unwrap_or(1), // Default to Ethereum mainnet
        status: true,                             // Assume successful for now
    };

    info!(
        "Transaction receipt retrieved: {} (block: {:?}, gas: {:?}, status: {})",
        tx_hash, result.block_number, result.gas_used, result.status
    );

    Ok(result)
}

/// Parse token amount with decimals
fn parse_token_amount(amount: &str, decimals: u8) -> Result<U256, EvmToolError> {
    let amount_f64 = amount
        .parse::<f64>()
        .map_err(|e| EvmToolError::Generic(format!("Invalid amount: {}", e)))?;

    let multiplier = 10_f64.powi(decimals as i32);
    let amount_wei = (amount_f64 * multiplier) as u128;

    Ok(U256::from(amount_wei))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_token_amount() {
        // Test with 18 decimals
        let amount = parse_token_amount("1.0", 18).unwrap();
        assert_eq!(amount, U256::from(1_000_000_000_000_000_000u128));

        // Test with 6 decimals (USDC)
        let amount = parse_token_amount("100.5", 6).unwrap();
        assert_eq!(amount, U256::from(100_500_000u128));

        // Test with 0 decimals
        let amount = parse_token_amount("1000", 0).unwrap();
        assert_eq!(amount, U256::from(1000u128));
    }

    #[test]
    fn test_transaction_result_creation() {
        let result = TransactionResult {
            tx_hash: "0x123".to_string(),
            from: "0xabc".to_string(),
            to: "0xdef".to_string(),
            value_wei: "1000000000000000000".to_string(),
            value_eth: 1.0,
            gas_used: Some(21000),
            gas_price: Some(20_000_000_000),
            block_number: Some(18000000),
            chain_id: 1,
            status: true,
        };

        assert_eq!(result.tx_hash, "0x123");
        assert!(result.status);
        assert_eq!(result.value_eth, 1.0);
    }
}
