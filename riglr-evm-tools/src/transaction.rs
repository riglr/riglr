//! Transaction tools for EVM chains

use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use riglr_core::{SignerContext, ToolError};
use riglr_macros::tool;
use std::str::FromStr;
use tracing::{debug, info};

/// Send ETH to an address (requires SignerContext for transaction signing)
#[tool]
pub async fn send_eth(
    to: String,
    amount: String,
    _chain_id: Option<u64>,
    _context: &riglr_core::provider::ApplicationContext,
) -> Result<String, ToolError> {
    debug!("Sending {} ETH to {}", amount, to);

    // Get the current EVM signer from context
    let signer_context = SignerContext::current_as_evm()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No EVM signer context: {}", e)))?;

    // Parse the destination address
    let to_address = Address::from_str(&to)
        .map_err(|e| ToolError::permanent_string(format!("Invalid destination address: {}", e)))?;

    // Parse the amount (in ETH) and convert to wei
    let amount_eth: f64 = amount
        .parse()
        .map_err(|e| ToolError::permanent_string(format!("Invalid amount: {}", e)))?;
    let amount_wei = U256::from((amount_eth * 1e18) as u128);

    // Build the transaction request
    let tx = TransactionRequest::default()
        .to(to_address)
        .value(amount_wei);

    // Send the transaction using the signer
    let tx_hash = signer_context
        .sign_and_send_transaction(tx)
        .await
        .map_err(|e| ToolError::retriable_string(format!("Failed to send transaction: {}", e)))?;

    info!("Transaction sent: {}", tx_hash);
    Ok(tx_hash)
}
