//! Network and blockchain query tools for EVM chains

use crate::{client::EvmClient, error::Result};
use alloy::primitives::TxHash;
use tracing::{debug, info};

/// Get the current block number
pub async fn get_block_number(_client: &EvmClient) -> Result<u64> {
    debug!("Getting current block number");

    // Use the underlying provider via the client
    let block_number = _client.get_block_number().await?;

    info!("Current block number: {}", block_number);
    Ok(block_number)
}

/// Get a transaction receipt by hash
pub async fn get_transaction_receipt(
    _client: &EvmClient,
    tx_hash: &str,
) -> Result<serde_json::Value> {
    debug!("Getting transaction receipt for: {}", tx_hash);

    // Parse transaction hash
    let hash: TxHash = tx_hash
        .parse()
        .map_err(|e| crate::error::EvmToolError::Rpc(format!("Invalid tx hash: {}", e)))?;

    // Query provider for receipt
    let provider = _client.provider();
    let receipt_opt = provider
        .get_transaction_receipt(hash)
        .await
        .map_err(|e| crate::error::EvmToolError::Rpc(format!(
            "Failed to get transaction receipt: {}",
            e
        )))?;

    let json = match receipt_opt {
        Some(receipt) => {
            // Build a compact JSON with common fields; serialize the rest for completeness
            let status = receipt.status();
            let block_number = receipt.block_number;
            let gas_used = receipt.gas_used;
            let full = serde_json::to_value(&receipt)?;
            serde_json::json!({
                "transactionHash": tx_hash,
                "status": if status { "0x1" } else { "0x0" },
                "blockNumber": block_number,
                "gasUsed": gas_used,
                "raw": full
            })
        }
        None => serde_json::json!({
            "transactionHash": tx_hash,
            "status": null,
            "blockNumber": null,
            "gasUsed": null
        }),
    };

    info!("Transaction receipt retrieved for: {}", tx_hash);
    Ok(json)
}