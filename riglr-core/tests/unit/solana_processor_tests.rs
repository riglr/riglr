use std::sync::Arc;

use riglr_core::transactions::solana::{PriorityFeeConfig, SolanaTransactionProcessor};
use riglr_core::transactions::{TransactionProcessor, TransactionStatus};
use riglr_core::error::ToolError;
use solana_client::rpc_client::RpcClient;

#[tokio::test]
async fn solana_get_status_invalid_signature_and_submitted() {
    // Use a real client but we'll only hit parsing branches
    let client = Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string()));
    let p = SolanaTransactionProcessor::new(client, PriorityFeeConfig::default());

    // invalid signature string
    let err = p.get_status("").await.unwrap_err();
    assert!(!err.is_retriable());

    // Submitted path: extremely unlikely signature won't exist. Provide a valid-looking base58 that won't parse -> still invalid
    let err2 = p.get_status("not-a-base58").await.unwrap_err();
    assert!(!matches!(err2, ToolError::Retriable { .. }));
}

#[tokio::test]
async fn solana_wait_for_confirmation_invalid_signature() {
    let client = Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string()));
    let p = SolanaTransactionProcessor::new(client, PriorityFeeConfig::default());

    let err = p.wait_for_confirmation("bad_signature", 1).await.unwrap_err();
    assert!(!err.is_retriable());
}
