//! Network state and blockchain query tools

use crate::{
    client::SolanaClient,
    error::{Result, SolanaToolError},
};
use tracing::{debug, info};

/// Get the current block height from the Solana blockchain
pub async fn get_block_height(client: &SolanaClient) -> Result<u64> {
    debug!("Getting current block height");

    let height = client
        .get_block_height()
        .await
        .map_err(|e| SolanaToolError::Rpc(format!("Failed to get block height: {}", e)))?;

    info!("Current block height: {}", height);
    Ok(height)
}

/// Get transaction status by signature
pub async fn get_transaction_status(client: &SolanaClient, signature: &str) -> Result<String> {
    debug!("Getting transaction status for signature: {}", signature);

    let signatures = vec![signature.to_string()];
    let statuses = client
        .get_signature_statuses(&signatures)
        .await
        .map_err(|e| SolanaToolError::Rpc(format!("Failed to get transaction status: {}", e)))?;

    if let Some(Some(status)) = statuses.first() {
        let status_str = if status.err.is_some() {
            "failed"
        } else if status.confirmations.is_some() {
            match &status.confirmation_status {
                Some(confirmation_status) => {
                    use solana_transaction_status::TransactionConfirmationStatus;
                    match confirmation_status {
                        TransactionConfirmationStatus::Finalized => "finalized",
                        TransactionConfirmationStatus::Confirmed => "confirmed",
                        TransactionConfirmationStatus::Processed => "processed",
                    }
                }
                None => "unknown",
            }
        } else {
            "pending"
        };

        info!("Transaction {} status: {}", signature, status_str);
        Ok(status_str.to_string())
    } else {
        info!("Transaction {} not found", signature);
        Ok("not_found".to_string())
    }
}
