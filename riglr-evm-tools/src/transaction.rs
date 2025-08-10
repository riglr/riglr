//! Transaction management and signing for EVM chains
//!
//! This module provides secure transaction creation, signing, and broadcasting
//! with support for both legacy and EIP-1559 transactions.

use crate::{
    client::{eth_to_wei, validate_address, EvmClient},
    error::{EvmToolError, Result},
};
use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::{Address, Bytes, TxKind, U256},
    providers::{Provider, PendingTransactionConfig},
    rpc::types::TransactionRequest,
    signers::{local::PrivateKeySigner, Signer},
    sol,
};
use riglr_core::ToolError;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

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
/// This tool transfers ETH from the signer's address to another address.
#[tool]
pub async fn transfer_eth(
    client: &EvmClient,
    to_address: String,
    amount_eth: f64,
    gas_price_gwei: Option<u64>,
    nonce: Option<u64>,
) -> std::result::Result<TransactionResult, ToolError> {
    debug!("Transferring {} ETH to {}", amount_eth, to_address);

    // Validate destination address
    let to_addr = validate_address(&to_address)
        .map_err(|e| ToolError::permanent(format!("Invalid address: {}", e)))?;

    // Get signer from client (replaces global state access)
    let signer = client.require_signer()
        .map_err(|e| ToolError::permanent(format!("Client requires signer configuration: {}", e)))?;
    
    let from_addr = signer.address();

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
            .map_err(|e| ToolError::retriable(format!("Failed to get nonce: {}", e)))?
    };

    // Get gas price if not provided
    let gas_price = if let Some(gwei) = gas_price_gwei {
        gwei as u128 * 1_000_000_000 // Convert gwei to wei
    } else {
        client
            .get_gas_price()
            .await
            .map_err(|e| ToolError::retriable(format!("Failed to get gas price: {}", e)))?
    };

    // Build transaction
    let tx = TransactionRequest::default()
        .from(from_addr)
        .to(to_addr)
        .value(value_wei)
        .nonce(tx_nonce)
        .gas_price(gas_price)
        .gas_limit(21000); // Standard ETH transfer gas limit

    // Create a wallet for signing
    let ethereum_wallet = EthereumWallet::from(signer.clone());

    // Send transaction with the wallet
    let provider_with_wallet = client.provider().with_wallet(ethereum_wallet);

    let pending_tx = provider_with_wallet
        .send_transaction(tx)
        .await
        .map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("insufficient funds") {
                ToolError::permanent(format!("Insufficient funds: {}", e))
            } else if error_str.contains("nonce") {
                ToolError::permanent(format!("Nonce error: {}", e))
            } else {
                ToolError::retriable(format!("Failed to send transaction: {}", e))
            }
        })?;

    // Wait for confirmation
    let receipt = pending_tx
        .with_required_confirmations(1)
        .get_receipt()
        .await
        .map_err(|e| ToolError::retriable(format!("Failed to get receipt: {}", e)))?;

    let result = TransactionResult {
        tx_hash: format!("0x{:x}", receipt.transaction_hash),
        from: format!("0x{:x}", from_addr),
        to: to_address.clone(),
        value_wei: value_wei.to_string(),
        value_eth: amount_eth,
        gas_used: receipt.gas_used,
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
/// This tool transfers ERC20 tokens from the signer's address to another address.
#[tool]
pub async fn transfer_erc20(
    client: &EvmClient,
    token_address: String,
    to_address: String,
    amount: String,
    decimals: u8,
    gas_price_gwei: Option<u64>,
) -> std::result::Result<TransactionResult, ToolError> {
    debug!(
        "Transferring {} tokens to {} (token: {})",
        amount, to_address, token_address
    );

    // Validate addresses
    let token_addr = validate_address(&token_address)
        .map_err(|e| ToolError::permanent(format!("Invalid token address: {}", e)))?;
    let to_addr = validate_address(&to_address)
        .map_err(|e| ToolError::permanent(format!("Invalid to address: {}", e)))?;

    // Get signer from client (replaces global state access)
    let signer = client.require_signer()
        .map_err(|e| ToolError::permanent(format!("Client requires signer configuration: {}", e)))?;
    
    let from_addr = signer.address();

    // Parse amount with decimals
    let amount_wei = parse_token_amount(&amount, decimals)
        .map_err(|e| ToolError::permanent(format!("Invalid amount: {}", e)))?;

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
            .map_err(|e| ToolError::retriable(format!("Failed to get gas price: {}", e)))?
    };

    // Build transaction
    let tx = TransactionRequest::default()
        .from(from_addr)
        .to(token_addr)
        .input(call_data.into())
        .gas_price(gas_price)
        .gas_limit(100000); // Standard ERC20 transfer gas limit

    // Create a wallet for signing
    let ethereum_wallet = EthereumWallet::from(signer.clone());

    // Send transaction with the wallet
    let provider_with_wallet = client.provider().with_wallet(ethereum_wallet);

    let pending_tx = provider_with_wallet
        .send_transaction(tx)
        .await
        .map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("insufficient") {
                ToolError::permanent(format!("Insufficient balance: {}", e))
            } else {
                ToolError::retriable(format!("Failed to send transaction: {}", e))
            }
        })?;

    // Wait for confirmation
    let receipt = pending_tx
        .with_required_confirmations(1)
        .get_receipt()
        .await
        .map_err(|e| ToolError::retriable(format!("Failed to get receipt: {}", e)))?;

    let result = TransactionResult {
        tx_hash: format!("0x{:x}", receipt.transaction_hash),
        from: format!("0x{:x}", from_addr),
        to: to_address.clone(),
        value_wei: amount_wei.to_string(),
        value_eth: 0.0, // Not ETH
        gas_used: receipt.gas_used,
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
/// This tool retrieves the receipt for a transaction hash.
#[tool]
pub async fn get_transaction_receipt(
    client: &EvmClient,
    tx_hash: String,
) -> std::result::Result<TransactionResult, ToolError> {
    debug!("Getting transaction receipt for {}", tx_hash);

    // Parse transaction hash
    let hash = tx_hash
        .parse()
        .map_err(|e| ToolError::permanent(format!("Invalid transaction hash: {}", e)))?;

    // Get receipt
    let receipt = client
        .provider()
        .get_transaction_receipt(hash)
        .await
        .map_err(|e| ToolError::retriable(format!("Failed to get receipt: {}", e)))?
        .ok_or_else(|| ToolError::permanent("Transaction not found"))?;

    // Get transaction details
    let tx = client
        .provider()
        .get_transaction_by_hash(hash)
        .await
        .map_err(|e| ToolError::retriable(format!("Failed to get transaction: {}", e)))?
        .ok_or_else(|| ToolError::permanent("Transaction not found"))?;

    let result = TransactionResult {
        tx_hash: tx_hash.clone(),
        from: format!("0x{:x}", tx.from),
        to: tx.to.map(|a| format!("0x{:x}", a)).unwrap_or_default(),
        value_wei: tx.value.to_string(),
        value_eth: crate::client::wei_to_eth(tx.value),
        gas_used: receipt.gas_used,
        gas_price: tx.gas_price,
        block_number: receipt.block_number,
        chain_id: client.chain_id,
        status: receipt.status(),
    };

    info!("Transaction receipt retrieved: {}", tx_hash);

    Ok(result)
}

/// Parse token amount with decimals
fn parse_token_amount(amount: &str, decimals: u8) -> Result<U256> {
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