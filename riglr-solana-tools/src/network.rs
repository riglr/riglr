//! Network state and blockchain query tools

use riglr_core::{ToolError, SignerContext};
use riglr_macros::tool;
use tracing::{debug, info};

/// Get the current block height from the Solana blockchain
#[tool]
pub async fn get_block_height() -> Result<u64, ToolError> {
    debug!("Getting current block height");

    // Get signer context and client
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    let client = signer.solana_client();

    let height = client
        .get_block_height()
        .map_err(|e| ToolError::retriable(format!("Failed to get block height: {}", e)))?;

    info!("Current block height: {}", height);
    Ok(height)
}

/// Get transaction status by signature
#[tool]
pub async fn get_transaction_status(signature: String) -> Result<String, ToolError> {
    debug!("Getting transaction status for signature: {}", signature);

    // Get signer context and client
    let signer_context = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    let client = signer_context.solana_client();

    let signatures = vec![signature.parse().map_err(|e| ToolError::permanent(format!("Invalid signature: {}", e)))?];
    let statuses = client
        .get_signature_statuses(&signatures)
        .map_err(|e| ToolError::retriable(format!("Failed to get transaction status: {}", e)))?;

    if let Some(Some(status)) = statuses.value.first() {
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
