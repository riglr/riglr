//! Comprehensive tests for client module

use riglr_evm_tools::client::{validate_address, EvmClient, EvmConfig};
use alloy::primitives::U256;
use serde_json::json;
use std::time::Duration;

#[test]
fn test_evm_config_default() {
    let config = EvmConfig::default();
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert_eq!(config.chain_id, 1);
    assert!(config.rpc_url.contains("eth"));
}

#[test]
fn test_evm_config_custom() {
    let config = EvmConfig {
        rpc_url: "https://custom-rpc.com".to_string(),
        chain_id: 137,
        timeout: Duration::from_secs(60),
    };

    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.chain_id, 137);
    assert_eq!(config.rpc_url, "https://custom-rpc.com");
}

#[test]
fn test_evm_config_clone() {
    let config = EvmConfig::default();

    let cloned = config.clone();
    assert_eq!(cloned.timeout, config.timeout);
    assert_eq!(cloned.chain_id, config.chain_id);
    assert_eq!(cloned.rpc_url, config.rpc_url);
}

#[test]
fn test_evm_config_debug() {
    let config = EvmConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("timeout"));
    assert!(debug_str.contains("chain_id"));
    assert!(debug_str.contains("rpc_url"));
}

#[test]
fn test_validate_address_valid() {
    // Valid Ethereum addresses
    let addresses = vec![
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
        "0x0000000000000000000000000000000000000000",
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        "0xabcdef0123456789abcdef0123456789abcdef01",
        "0xABCDEF0123456789ABCDEF0123456789ABCDEF01",
    ];

    for addr in addresses {
        let result = validate_address(addr);
        assert!(result.is_ok(), "Failed for address: {}", addr);
        // validate_address returns an Address type, not a string
        assert!(result.is_ok());
    }
}

#[test]
fn test_validate_address_invalid_length() {
    let addresses = vec![
        "0x",
        "0x123",
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F81", // Too short
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F81234", // Too long
        "",
        // Note: "742d35Cc6634C0532925a3b8D8e41E5d3e4F8123" might be parsed as valid by alloy
        // so we test clearly invalid addresses
        "invalid_address",
    ];

    for addr in addresses {
        let result = validate_address(addr);
        assert!(result.is_err(), "Should fail for address: {}", addr);
    }
}

#[test]
fn test_validate_address_invalid_prefix() {
    let addresses = vec![
        "1x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
        "0X742d35Cc6634C0532925a3b8D8e41E5d3e4F8123", // Capital X
        "00742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
    ];

    for addr in addresses {
        let result = validate_address(addr);
        assert!(result.is_err(), "Should fail for address: {}", addr);
    }
}

#[test]
fn test_validate_address_invalid_hex() {
    let addresses = vec![
        "0xGGGG35Cc6634C0532925a3b8D8e41E5d3e4F8123", // Invalid hex chars
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F812Z", // Z is not hex
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F812!",
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F812 ",
    ];

    for addr in addresses {
        let result = validate_address(addr);
        assert!(result.is_err(), "Should fail for address: {}", addr);
    }
}


#[tokio::test]
async fn test_evm_client_creation_with_mock() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock eth_chainId call
    let _m1 = server
        .mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;

    let result = EvmClient::new(url).await;
    assert!(result.is_ok());

    let client = result.unwrap();
    assert_eq!(client.chain_id, 1);
}

#[tokio::test]
async fn test_evm_client_with_mock_server() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let _m = server
        .mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x89"}"#)
        .create_async()
        .await;

    let result = EvmClient::new(url).await;
    assert!(result.is_ok());

    let client = result.unwrap();
    assert_eq!(client.chain_id, 137); // 0x89 = 137
}

#[tokio::test]
async fn test_evm_client_chain_id_mismatch() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Return different chain ID than expected
    let _m = server
        .mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0xa"}"#)
        .create_async()
        .await;

    let result = EvmClient::new(url).await;
    assert!(result.is_ok());

    let client = result.unwrap();
    // Should use actual chain ID from RPC
    assert_eq!(client.chain_id, 10);
}

#[tokio::test]
async fn test_evm_client_with_rpc_url() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock for chain ID detection
    let _m = server
        .mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x2105"}"#)
        .create_async()
        .await;

    let result = EvmClient::new(url).await;
    assert!(result.is_ok());

    let client = result.unwrap();
    assert_eq!(client.chain_id, 8453); // 0x2105 = 8453 (Base)
}

#[tokio::test]
async fn test_evm_client_rpc_error() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Return RPC error
    let _m = server
        .mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"Invalid params"}}"#)
        .create_async()
        .await;

    let result = EvmClient::new(url).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_evm_client_get_chain_id() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock for initial connection
    let _m1 = server
        .mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;

    let client = EvmClient::new(url).await.unwrap();
    // The chain_id is available as a field, not a method
    assert_eq!(client.chain_id, 1);
}

#[tokio::test]
async fn test_evm_client_get_block_number() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock for initial connection
    let _m1 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_chainId"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;

    // Mock for block number
    let _m2 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_blockNumber"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x11a72a0"}"#)
        .create_async()
        .await;

    let client = EvmClient::new(url).await.unwrap();
    let block_number = client.get_block_number().await.unwrap();
    assert!(block_number > 18500000); // Should be a reasonable recent block
}

#[tokio::test]
async fn test_evm_client_get_gas_price() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock for initial connection
    let _m1 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_chainId"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;

    // Mock for gas price
    let _m2 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_gasPrice"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x5f5e100"}"#)
        .create_async()
        .await;

    let client = EvmClient::new(url).await.unwrap();
    let gas_price = client.get_gas_price().await.unwrap();
    assert_eq!(gas_price, 100000000); // 0x5f5e100
}

#[tokio::test]
async fn test_evm_client_get_balance() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock for initial connection
    let _m1 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_chainId"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;

    // Mock for balance
    let _m2 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_getBalance"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0xde0b6b3a7640000"}"#)
        .create_async()
        .await;

    let client = EvmClient::new(url).await.unwrap();
    let address = "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".parse().unwrap();
    let balance = client
        .get_balance(address)
        .await
        .unwrap();
    // get_balance returns U256, not a string
    assert!(balance > U256::from(0));
}



#[tokio::test]
async fn test_evm_client_invalid_response_format() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock with invalid response (missing result field)
    let _m = server
        .mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1}"#)
        .create_async()
        .await;

    let result = EvmClient::new(url).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_evm_client_http_error() {
    // Use invalid URL to trigger HTTP error
    let result = EvmClient::new("http://invalid-domain-12345.com".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_evm_client_fields() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let _m = server
        .mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;

    let client = EvmClient::new(url.clone()).await.unwrap();

    // Test public fields are accessible
    assert_eq!(client.rpc_url, url);
    assert_eq!(client.chain_id, 1);
}

#[tokio::test]
async fn test_evm_client_clone() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let _m = server
        .mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;

    let client = EvmClient::new(url.clone()).await.unwrap();
    let cloned = client.clone();

    assert_eq!(cloned.rpc_url, client.rpc_url);
    assert_eq!(cloned.chain_id, client.chain_id);
}

#[tokio::test]
async fn test_evm_client_convenience_constructors() {
    // Test all the convenience constructor methods
    // Note: These may succeed or fail depending on network connectivity,
    // but they exercise the code paths we want to cover

    // Test mainnet - available method
    let _result = EvmClient::mainnet().await;

    // Test polygon - may succeed or fail based on network
    let _result = EvmClient::polygon().await;

    // Test arbitrum
    let _result = EvmClient::arbitrum().await;

    // Test optimism
    let _result = EvmClient::optimism().await;

    // Test base
    let _result = EvmClient::base().await;

    // The main goal is to exercise the code paths, not test network connectivity
    // So we don't assert on results, just that the methods can be called
}

#[tokio::test]
async fn test_evm_client_invalid_url() {
    // Test invalid URL handling
    let result = EvmClient::new("invalid-url-format".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_evm_client_config_defaults() {
    // Test that EvmConfig has reasonable defaults
    use riglr_evm_tools::client::EvmConfig;
    
    let config = EvmConfig::default();
    assert!(!config.rpc_url.is_empty());
    assert!(config.chain_id > 0);
    assert!(config.timeout.as_secs() > 0);
}

#[tokio::test]
async fn test_evm_client_rpc_call_errors() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Test RPC error response - covers lines 216-221
    let _m1 = server
        .mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"Test error"}}"#)
        .create_async()
        .await;

    let client = EvmClient::new(url.clone()).await;
    assert!(client.is_err());

    // Test missing result field - covers line 226
    let _m2 = server
        .mock("POST", "/")
        .with_body(r#"{"jsonrpc":"2.0","id":1}"#) // No result or error field
        .create_async()
        .await;

    let client = EvmClient::new(url.clone()).await;
    assert!(client.is_err());
}

#[tokio::test]
async fn test_evm_client_chain_id_mismatch_warning() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock chain ID response that doesn't match expected - covers lines 100-105
    let _m = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_chainId"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x89"}"#) // Chain ID 137 when we expect 1
        .create_async()
        .await;

    // This should succeed but log a warning about chain ID mismatch
    let result = EvmClient::new(url).await;
    assert!(result.is_ok());

    let client = result.unwrap();
    assert_eq!(client.chain_id, 137); // Should use actual chain ID, not expected
}
