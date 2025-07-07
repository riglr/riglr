//! Comprehensive tests for contract module

use riglr_evm_tools::contract::{call_contract_read, call_contract_write};
use riglr_evm_tools::client::EvmClient;
use mockito;
use serde_json::json;

#[tokio::test]
async fn test_call_contract_read_placeholder() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    // Mock chain ID for client creation
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let client = EvmClient::new(url, 1).await.unwrap();
    let result = call_contract_read(
        &client,
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
        "balanceOf",
        vec!["0x0000000000000000000000000000000000000000".to_string()],
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({})); // Placeholder returns empty object
}

#[tokio::test]
async fn test_call_contract_write_placeholder() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    // Mock chain ID for client creation
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let client = EvmClient::new(url, 1).await.unwrap();
    let result = call_contract_write(
        &client,
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
        "transfer",
        vec![
            "0x0000000000000000000000000000000000000001".to_string(),
            "1000000000000000000".to_string(),
        ],
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0xplaceholder_transaction_hash");
}

#[tokio::test]
async fn test_call_contract_read_various_functions() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let client = EvmClient::new(url, 1).await.unwrap();
    
    // Test various function names
    let functions = vec![
        "balanceOf",
        "totalSupply",
        "decimals",
        "symbol",
        "name",
        "allowance",
        "owner",
    ];
    
    for func in functions {
        let result = call_contract_read(
            &client,
            "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
            func,
            vec![],
        ).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!({}));
    }
}

#[tokio::test]
async fn test_call_contract_write_various_functions() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let client = EvmClient::new(url, 1).await.unwrap();
    
    // Test various function names
    let functions = vec![
        ("transfer", vec!["0x123".to_string(), "100".to_string()]),
        ("approve", vec!["0x456".to_string(), "200".to_string()]),
        ("transferFrom", vec!["0x789".to_string(), "0xabc".to_string(), "300".to_string()]),
        ("mint", vec!["0xdef".to_string(), "400".to_string()]),
        ("burn", vec!["500".to_string()]),
    ];
    
    for (func, params) in functions {
        let result = call_contract_write(
            &client,
            "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
            func,
            params,
        ).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0xplaceholder_transaction_hash");
    }
}

#[tokio::test]
async fn test_call_contract_read_empty_params() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let client = EvmClient::new(url, 1).await.unwrap();
    let result = call_contract_read(
        &client,
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
        "totalSupply",
        vec![],
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({}));
}

#[tokio::test]
async fn test_call_contract_write_empty_params() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let client = EvmClient::new(url, 1).await.unwrap();
    let result = call_contract_write(
        &client,
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
        "pause",
        vec![],
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0xplaceholder_transaction_hash");
}

#[tokio::test]
async fn test_call_contract_read_many_params() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let client = EvmClient::new(url, 1).await.unwrap();
    
    // Test with many parameters
    let params: Vec<String> = (0..10).map(|i| format!("param_{}", i)).collect();
    let result = call_contract_read(
        &client,
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
        "complexFunction",
        params,
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({}));
}

#[tokio::test]
async fn test_call_contract_with_different_addresses() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let client = EvmClient::new(url, 1).await.unwrap();
    
    let addresses = vec![
        "0x0000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000001",
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
    ];
    
    for addr in addresses {
        let read_result = call_contract_read(
            &client,
            addr,
            "test",
            vec![],
        ).await;
        assert!(read_result.is_ok());
        
        let write_result = call_contract_write(
            &client,
            addr,
            "test",
            vec![],
        ).await;
        assert!(write_result.is_ok());
    }
}

#[tokio::test]
async fn test_contract_functions_with_custom_config() {
    use riglr_evm_tools::client::EvmConfig;
    use std::time::Duration;
    
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    let _m = server.mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;
    
    let config = EvmConfig {
        timeout: Duration::from_secs(5),
        max_retries: 2,
        retry_delay: Duration::from_millis(500),
        headers: Default::default(),
    };
    
    let client = EvmClient::with_config(url, 1, config).await.unwrap();
    
    // Both functions should work with custom config client
    let read_result = call_contract_read(
        &client,
        "0x123",
        "test",
        vec![],
    ).await;
    assert!(read_result.is_ok());
    
    let write_result = call_contract_write(
        &client,
        "0x456",
        "test",
        vec![],
    ).await;
    assert!(write_result.is_ok());
}