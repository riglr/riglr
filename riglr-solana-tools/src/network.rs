//! Network state and blockchain query tools

use riglr_core::{ToolError, SignerContext};
use riglr_macros::tool;
use tracing::{debug, info};

/// Get the current block height from the Solana blockchain
///
/// This tool queries the Solana network to retrieve the most recent block height,
/// which represents the number of blocks that have been processed by the network.
/// Essential for checking network activity and determining transaction finality.
/// 
/// # Returns
/// 
/// Returns the current block height as a `u64` representing the total number
/// of blocks processed by the Solana network since genesis.
/// 
/// # Errors
/// 
/// * `ToolError::Permanent` - When no signer context is available
/// * `ToolError::Retriable` - When network connection issues occur or RPC timeouts
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_solana_tools::network::get_block_height;
/// use riglr_core::SignerContext;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let height = get_block_height().await?;
/// println!("Current block height: {}", height);
/// 
/// // Use block height for transaction confirmation checks
/// if height > 150_000_000 {
///     println!("Network has processed over 150M blocks");
/// }
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_block_height() -> Result<u64, ToolError> {
    debug!("Getting current block height");

    // Get signer context and client
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    let client = signer.solana_client()
        .ok_or_else(|| ToolError::permanent("No Solana client available in signer".to_string()))?;

    let height = client
        .get_block_height()
        .map_err(|e| ToolError::retriable(format!("Failed to get block height: {}", e)))?;

    info!("Current block height: {}", height);
    Ok(height)
}

/// Get transaction status by signature
///
/// This tool queries the Solana network to check the confirmation status of a transaction
/// using its signature. Essential for monitoring transaction progress and ensuring operations
/// have been confirmed by the network before proceeding with dependent actions.
/// 
/// # Arguments
/// 
/// * `signature` - The transaction signature to check (base58-encoded string)
/// 
/// # Returns
/// 
/// Returns a `String` indicating the transaction status:
/// - `"finalized"` - Transaction is finalized and cannot be rolled back
/// - `"confirmed"` - Transaction is confirmed by supermajority of cluster
/// - `"processed"` - Transaction has been processed but may not be confirmed
/// - `"failed"` - Transaction failed due to an error
/// - `"not_found"` - Transaction signature not found (may not exist or be too old)
/// 
/// # Errors
/// 
/// * `ToolError::Permanent` - When signature format is invalid or signer context unavailable
/// * `ToolError::Retriable` - When network issues occur during status lookup
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_solana_tools::network::get_transaction_status;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let status = get_transaction_status(
///     "5j7s88CkzQeE6EN5HiV7CqkYsL3x6PbJmSjYpJjm1J2v3z4x8K7b".to_string()
/// ).await?;
/// 
/// match status.as_str() {
///     "finalized" => println!("✅ Transaction is finalized"),
///     "confirmed" => println!("🔄 Transaction is confirmed"),
///     "processed" => println!("⏳ Transaction is processed, awaiting confirmation"),
///     "failed" => println!("❌ Transaction failed"),
///     "not_found" => println!("🔍 Transaction not found"),
///     _ => println!("Unknown status: {}", status),
/// }
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_transaction_status(signature: String) -> Result<String, ToolError> {
    debug!("Getting transaction status for signature: {}", signature);

    // Get signer context and client
    let signer_context = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    let client = signer_context.solana_client()
        .ok_or_else(|| ToolError::permanent("No Solana client available in signer".to_string()))?;

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
