//! Comprehensive tests for transaction module

use riglr_solana_tools::client::SolanaClient;
use riglr_solana_tools::transaction::*;
use solana_sdk::signature::{Keypair, Signer};

#[test]
fn test_client_with_signer() {
    let keypair = Keypair::new();
    let client = SolanaClient::devnet().with_signer(keypair.insecure_clone());

    assert!(client.has_signer());
    let signer = client.require_signer().unwrap();
    assert_eq!(signer.pubkey(), keypair.pubkey());
}

#[test]
fn test_client_without_signer() {
    let client = SolanaClient::devnet();

    assert!(!client.has_signer());
    assert!(client.require_signer().is_err());
}

#[test]
fn test_client_with_signer_from_bytes() {
    let keypair = Keypair::new();
    let bytes = keypair.to_bytes();
    let client = SolanaClient::devnet().with_signer_from_bytes(&bytes).unwrap();

    assert!(client.has_signer());
    let signer = client.require_signer().unwrap();
    assert_eq!(signer.pubkey(), keypair.pubkey());
}

#[test]
fn test_client_with_invalid_signer_bytes() {
    let client = SolanaClient::devnet();
    let invalid_bytes = vec![0u8; 32]; // Wrong length
    
    let result = client.with_signer_from_bytes(&invalid_bytes);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_transfer_sol_without_signer() {
    let client = SolanaClient::devnet();
    
    let result = transfer_sol(
        &client,
        "11111111111111111111111111111111".to_string(),
        0.001,
        None,
        None,
    ).await;
    
    // Should fail because no signer is configured
    assert!(result.is_err());
}

#[tokio::test]
async fn test_transfer_spl_token_without_signer() {
    let client = SolanaClient::devnet();
    
    let result = transfer_spl_token(
        &client,
        "11111111111111111111111111111111".to_string(),
        "So11111111111111111111111111111111111111112".to_string(),
        1000,
        6,
        false,
    ).await;
    
    // Should fail because no signer is configured
    assert!(result.is_err());
}

#[test]
fn test_transaction_result_creation() {
    let result = TransactionResult {
        signature: "test_signature".to_string(),
        from: "from_address".to_string(),
        to: "to_address".to_string(),
        amount: 1_000_000_000,
        amount_display: "1.0 SOL".to_string(),
        status: TransactionStatus::Confirmed,
        memo: None,
        idempotency_key: None,
    };

    assert_eq!(result.signature, "test_signature");
    assert_eq!(result.amount, 1_000_000_000);
    assert!(matches!(result.status, TransactionStatus::Confirmed));
}

#[test]
fn test_transaction_status() {
    let status = TransactionStatus::Pending;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"Pending\"");

    let status = TransactionStatus::Failed("error".to_string());
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("Failed"));
}