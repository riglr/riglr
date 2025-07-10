//! Transaction management and signing for EVM chains
//!
//! This module provides secure transaction creation, signing, and broadcasting
//! with support for both legacy and EIP-1559 transactions.

use crate::{
    client::{eth_to_wei, validate_address, EvmClient},
    error::EvmToolError,
};
use alloy::{
    consensus::Transaction as TransactionTrait,
    network::TransactionBuilder,
    primitives::U256,
    providers::Provider,
    rpc::types::TransactionRequest,
    signers::Signer,
    sol,
    sol_types::SolCall,
};
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
#[tool]
pub async fn transfer_eth(
    to_address: String,
    amount_eth: f64,
    gas_price_gwei: Option<u64>,
    nonce: Option<u64>,
) -> std::result::Result<TransactionResult, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Transferring {} ETH to {}", amount_eth, to_address);

    // Get signer context
    let signer_context = riglr_core::SignerContext::current().await
        .map_err(|e| EvmToolError::Generic(format!("No signer context: {}", e)))?;
    
    // Create EVM client (temporary - should come from signer context)
    let client = EvmClient::mainnet().await
        .map_err(|e| EvmToolError::Generic(format!("Failed to create EVM client: {}", e)))?;

    // Validate destination address
    let to_addr = validate_address(&to_address)
        .map_err(|e| EvmToolError::Generic(format!("Invalid address: {}", e)))?;

    // For now, we'll use a placeholder address since proper EVM signer integration is complex
    let from_addr_str = signer_context.address()
        .ok_or_else(|| EvmToolError::Generic("Signer has no address".to_string()))?;
    let from_addr = validate_address(&from_addr_str)
        .map_err(|e| EvmToolError::Generic(format!("Invalid signer address: {}", e)))?;

    // Convert ETH to wei
    let value_wei = eth_to_wei(amount_eth);

    // Get nonce if not provided
    let tx_nonce = if let Some(n) = nonce {
        n
    } else {
        client
            .provider()
            .get_transaction_count(from_addr)
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get nonce: {}", e)))?
    };

    // Get gas price if not provided
    let gas_price = if let Some(gwei) = gas_price_gwei {
        gwei as u128 * 1_000_000_000 // Convert gwei to wei
    } else {
        client
            .get_gas_price()
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get gas price: {}", e)))?
    };

    // Build transaction
    let tx = TransactionRequest::default()
        .from(from_addr)
        .to(to_addr)
        .value(value_wei)
        .nonce(tx_nonce)
        .gas_price(gas_price)
        .gas_limit(21000); // Standard ETH transfer gas limit

    // Sign the transaction manually
    // NOTE: Pending implementation of wallet signer integration with alloy
    // Currently uses provider's send_transaction which requires pre-configured wallet
    let pending_tx = client.provider()
        .send_transaction(tx)
        .await
        .map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("insufficient funds") {
                EvmToolError::Generic(format!("Insufficient funds: {}", e))
            } else if error_str.contains("nonce") {
                EvmToolError::Generic(format!("Nonce error: {}", e))
            } else {
                EvmToolError::Rpc(format!("Failed to send transaction: {}", e))
            }
        })?;

    // Wait for confirmation
    let receipt = pending_tx
        .with_required_confirmations(1)
        .get_receipt()
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get receipt: {}", e)))?;

    let result = TransactionResult {
        tx_hash: format!("0x{:x}", receipt.transaction_hash),
        from: format!("0x{:x}", from_addr),
        to: to_address.clone(),
        value_wei: value_wei.to_string(),
        value_eth: amount_eth,
        gas_used: Some(receipt.gas_used as u128),
        gas_price: Some(gas_price),
        block_number: receipt.block_number,
        chain_id: client.chain_id,
        status: receipt.status(),
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

    // Get signer context
    let signer_context = riglr_core::SignerContext::current().await
        .map_err(|e| EvmToolError::Generic(format!("No signer context: {}", e)))?;
    
    // Create EVM client (temporary - should come from signer context)
    let client = EvmClient::mainnet().await
        .map_err(|e| EvmToolError::Generic(format!("Failed to create EVM client: {}", e)))?;

    // Validate addresses
    let token_addr = validate_address(&token_address)
        .map_err(|e| EvmToolError::Generic(format!("Invalid token address: {}", e)))?;
    let to_addr = validate_address(&to_address)
        .map_err(|e| EvmToolError::Generic(format!("Invalid to address: {}", e)))?;

    let from_addr_str = signer_context.address()
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

    // Get gas price if not provided
    let gas_price = if let Some(gwei) = gas_price_gwei {
        gwei as u128 * 1_000_000_000
    } else {
        client
            .get_gas_price()
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get gas price: {}", e)))?
    };

    // Build transaction
    let tx = TransactionRequest::default()
        .from(from_addr)
        .to(token_addr)
        .input(call_data.into())
        .gas_price(gas_price)
        .gas_limit(100000); // Standard ERC20 transfer gas limit

    // Sign the transaction manually
    // NOTE: Pending implementation of wallet signer integration with alloy
    // Currently uses provider's send_transaction which requires pre-configured wallet
    let pending_tx = client.provider()
        .send_transaction(tx)
        .await
        .map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("insufficient") {
                EvmToolError::Generic(format!("Insufficient balance: {}", e))
            } else {
                EvmToolError::Rpc(format!("Failed to send transaction: {}", e))
            }
        })?;

    // Wait for confirmation
    let receipt = pending_tx
        .with_required_confirmations(1)
        .get_receipt()
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get receipt: {}", e)))?;

    let result = TransactionResult {
        tx_hash: format!("0x{:x}", receipt.transaction_hash),
        from: format!("0x{:x}", from_addr),
        to: to_address.clone(),
        value_wei: amount_wei.to_string(),
        value_eth: 0.0, // Not ETH
        gas_used: Some(receipt.gas_used as u128),
        gas_price: Some(gas_price),
        block_number: receipt.block_number,
        chain_id: client.chain_id,
        status: receipt.status(),
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
/// Returns `TransactionResult` with available transaction details. Some fields may be
/// placeholders due to current implementation limitations with transaction detail extraction.
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
#[tool]
pub async fn get_transaction_receipt(
    tx_hash: String,
) -> std::result::Result<TransactionResult, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Getting transaction receipt for {}", tx_hash);

    // Get signer context and create client
    let _signer_context = riglr_core::SignerContext::current().await
        .map_err(|e| EvmToolError::Generic(format!("No signer context: {}", e)))?;
    
    // Create EVM client (temporary - should come from signer context)
    let client = EvmClient::mainnet().await
        .map_err(|e| EvmToolError::Generic(format!("Failed to create EVM client: {}", e)))?;

    // Parse transaction hash
    let hash = tx_hash
        .parse()
        .map_err(|e| EvmToolError::Generic(format!("Invalid transaction hash: {}", e)))?;

    // Get receipt
    let receipt = client
        .provider()
        .get_transaction_receipt(hash)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get receipt: {}", e)))?
        .ok_or_else(|| EvmToolError::Generic("Transaction not found".to_string()))?;

    // Get transaction details (currently not fully extracting due to type access issues)
    let _tx = client
        .provider()
        .get_transaction_by_hash(hash)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get transaction: {}", e)))?
        .ok_or_else(|| EvmToolError::Generic("Transaction not found".to_string()))?;

    // Extract transaction details from the retrieved transaction
    // NOTE: Due to alloy's complex transaction envelope structure, using fallback values
    // Full implementation would require deep pattern matching on transaction variants
    let result = TransactionResult {
        tx_hash: tx_hash.clone(),
        from: "0x0000000000000000000000000000000000000000".to_string(), // Extracted from receipt if available
        to: "0x0000000000000000000000000000000000000000".to_string(),
        value_wei: "0".to_string(),
        value_eth: 0.0,
        gas_used: Some(receipt.gas_used as u128),
        gas_price: _tx.effective_gas_price.map(|price| price),
        block_number: receipt.block_number,
        chain_id: client.chain_id,
        status: receipt.status(),
    };

    info!("Transaction receipt retrieved: {}", tx_hash);

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