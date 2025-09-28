//! Transaction tools for EVM chains

// Tool macro generates public fields with underscores for unused params

use alloy::primitives::Address;
use alloy::rpc::types::TransactionRequest;
use core::str::FromStr as _;
use riglr_core::{provider::ApplicationContext, SignerContext, ToolError};
use riglr_evm_common::eth_to_wei;
use riglr_macros::tool;
use tracing::{debug, info};

#[cfg(feature = "high-precision")]
use rust_decimal::{prelude::FromPrimitive, Decimal};

/// Parse destination address
fn parse_destination_address(to: &str) -> Result<Address, ToolError> {
    Address::from_str(to).map_err(|error| {
        ToolError::permanent_string(format!("Invalid destination address: {error}"))
    })
}

/// Build transaction request for ETH transfer
fn build_eth_transaction(to_address: Address, amount_eth: f64) -> TransactionRequest {
    // Convert ETH amount to wei
    #[cfg(feature = "high-precision")]
    let amount_wei = eth_to_wei(Decimal::from_f64(amount_eth).unwrap_or(Decimal::ZERO));

    #[cfg(not(feature = "high-precision"))]
    let amount_wei = eth_to_wei(amount_eth);

    TransactionRequest::default()
        .to(to_address)
        .value(amount_wei)
}

/// Send ETH to an address (requires `SignerContext` for transaction signing)
///
/// # Errors
///
/// Returns an error if:
/// - No EVM signer context is available
/// - The destination address is invalid
/// - Transaction serialization fails
/// - Transaction sending fails
#[tool]
#[inline]
pub async fn send_eth(
    to: String,
    amount_eth: f64,
    chain_id: Option<u64>,
    _context: &ApplicationContext,
) -> Result<String, ToolError> {
    debug!("Sending {} ETH to {}", amount_eth, to);

    // Log if chain_id is provided for debugging
    if let Some(id) = chain_id {
        debug!("Using explicit chain_id: {}", id);
    }

    // Get the current EVM signer from context
    let signer_context = SignerContext::current_as_evm()
        .map_err(|error| ToolError::permanent_string(format!("No EVM signer context: {error}")))?;

    // Parse the destination address
    let to_address = parse_destination_address(&to)?;

    // Build the transaction request
    let tx = build_eth_transaction(to_address, amount_eth);

    // Send the transaction using the signer
    let tx_json = serde_json::to_value(&tx).map_err(|error| {
        ToolError::retriable_string(format!("Failed to serialize transaction: {error}"))
    })?;

    let tx_hash = signer_context
        .signer()
        .sign_and_send_transaction(tx_json)
        .await
        .map_err(|error| {
            ToolError::retriable_string(format!("Failed to send transaction: {error}"))
        })?;

    info!("Transaction sent: {}", tx_hash);
    Ok(tx_hash)
}
