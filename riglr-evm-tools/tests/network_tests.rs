//! Comprehensive tests for network module

use riglr_evm_tools::network::{get_block_number, get_transaction_receipt};
use riglr_evm_tools::client::EvmClient;
use mockito;
use serde_json::json;

#[tokio::test]
async fn test_get_block_number_placeholder() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    // Mock chain ID for client creation
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let client = EvmClient::new(url, 1).await.unwrap();
    let result = get_block_number(&client).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // Placeholder returns 0
}

#[tokio::test]
async fn test_get_transaction_receipt_placeholder() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    // Mock chain ID for client creation
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let client = EvmClient::new(url, 1).await.unwrap();
    let result = get_transaction_receipt(&client, "0x1234567890abcdef").await;
    
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, json!({})); // Placeholder returns empty object
}

#[tokio::test]
async fn test_get_block_number_with_different_clients() {
    // Test with Ethereum client
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let eth_client = EvmClient::new(url.clone(), 1).await.unwrap();
    assert_eq!(get_block_number(&eth_client).await.unwrap(), 0);
    
    // Test with Polygon client
    let _m2 = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x89"}"#)
        .create_async()
        .await;
    
    let poly_client = EvmClient::new(url, 137).await.unwrap();
    assert_eq!(get_block_number(&poly_client).await.unwrap(), 0);
}

#[tokio::test]
async fn test_get_transaction_receipt_various_hashes() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let client = EvmClient::new(url, 1).await.unwrap();
    
    // Test with various transaction hash formats
    let hashes = vec![
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    ];
    
    for hash in hashes {
        let result = get_transaction_receipt(&client, hash).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!({}));
    }
}

#[tokio::test]
async fn test_network_functions_with_custom_config() {
    use riglr_evm_tools::client::EvmConfig;
    use std::time::Duration;
    
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let config = EvmConfig {
        timeout: Duration::from_secs(10),
        max_retries: 1,
        retry_delay: Duration::from_millis(100),
        headers: Default::default(),
    };
    
    let client = EvmClient::with_config(url, 1, config).await.unwrap();
    
    // Both functions should work with custom config client
    assert_eq!(get_block_number(&client).await.unwrap(), 0);
    assert_eq!(get_transaction_receipt(&client, "0xtest").await.unwrap(), json!({}));
}