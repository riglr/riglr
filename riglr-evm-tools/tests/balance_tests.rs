//! Comprehensive tests for balance module

use mockito;
use riglr_evm_tools::balance::{
    get_erc20_balance, get_eth_balance, get_multi_token_balances, BalanceResult, TokenBalanceResult,
};
use serde_json::json;
use tokio_test;

#[test]
fn test_balance_result_creation() {
    let result = BalanceResult {
        address: "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123".to_string(),
        balance_raw: "1000000000000000000".to_string(),
        balance_formatted: "1.000000 ETH".to_string(),
        decimals: 18,
        network: "Ethereum".to_string(),
        block_number: 18500000,
    };

    assert_eq!(result.address, "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123");
    assert_eq!(result.balance_raw, "1000000000000000000");
    assert_eq!(result.balance_formatted, "1.000000 ETH");
    assert_eq!(result.decimals, 18);
    assert_eq!(result.network, "Ethereum");
    assert_eq!(result.block_number, 18500000);
}

#[test]
fn test_balance_result_serialization() {
    let result = BalanceResult {
        address: "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123".to_string(),
        balance_raw: "2000000000000000000".to_string(),
        balance_formatted: "2.000000 ETH".to_string(),
        decimals: 18,
        network: "Polygon".to_string(),
        block_number: 50000000,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"address\""));
    assert!(json.contains("\"balance_raw\""));
    assert!(json.contains("\"balance_formatted\""));
    assert!(json.contains("\"decimals\":18"));
    assert!(json.contains("\"network\":\"Polygon\""));
    assert!(json.contains("\"block_number\":50000000"));

    // Test deserialization
    let deserialized: BalanceResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.address, result.address);
    assert_eq!(deserialized.balance_raw, result.balance_raw);
    assert_eq!(deserialized.network, result.network);
}

#[test]
fn test_balance_result_clone() {
    let result = BalanceResult {
        address: "0x123".to_string(),
        balance_raw: "1000".to_string(),
        balance_formatted: "0.001 ETH".to_string(),
        decimals: 18,
        network: "Test".to_string(),
        block_number: 100,
    };

    let cloned = result.clone();
    assert_eq!(cloned.address, result.address);
    assert_eq!(cloned.balance_raw, result.balance_raw);
    assert_eq!(cloned.balance_formatted, result.balance_formatted);
}

#[test]
fn test_balance_result_debug() {
    let result = BalanceResult {
        address: "0xtest".to_string(),
        balance_raw: "1".to_string(),
        balance_formatted: "1 ETH".to_string(),
        decimals: 18,
        network: "Debug".to_string(),
        block_number: 1,
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("BalanceResult"));
    assert!(debug_str.contains("address"));
    assert!(debug_str.contains("0xtest"));
}

#[test]
fn test_token_balance_result_creation() {
    let result = TokenBalanceResult {
        address: "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123".to_string(),
        token_address: "0xa0b86a33e6441c68e1a7e97c82b6baba4d45a9e3".to_string(),
        symbol: Some("USDC".to_string()),
        name: Some("USD Coin".to_string()),
        balance_raw: "1000000".to_string(),
        balance_formatted: "1.000000 USDC".to_string(),
        decimals: 6,
        network: "Ethereum".to_string(),
        block_number: 18500000,
    };

    assert_eq!(result.address, "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123");
    assert_eq!(
        result.token_address,
        "0xa0b86a33e6441c68e1a7e97c82b6baba4d45a9e3"
    );
    assert_eq!(result.symbol, Some("USDC".to_string()));
    assert_eq!(result.name, Some("USD Coin".to_string()));
    assert_eq!(result.decimals, 6);
}

#[test]
fn test_token_balance_result_with_none_values() {
    let result = TokenBalanceResult {
        address: "0x123".to_string(),
        token_address: "0x456".to_string(),
        symbol: None,
        name: None,
        balance_raw: "0".to_string(),
        balance_formatted: "0 TOKEN".to_string(),
        decimals: 18,
        network: "Test".to_string(),
        block_number: 0,
    };

    assert!(result.symbol.is_none());
    assert!(result.name.is_none());
}

#[test]
fn test_token_balance_result_serialization() {
    let result = TokenBalanceResult {
        address: "0xabc".to_string(),
        token_address: "0xdef".to_string(),
        symbol: Some("DAI".to_string()),
        name: Some("Dai Stablecoin".to_string()),
        balance_raw: "5000000000000000000".to_string(),
        balance_formatted: "5.000000000 DAI".to_string(),
        decimals: 18,
        network: "Arbitrum".to_string(),
        block_number: 100000000,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"token_address\""));
    assert!(json.contains("\"symbol\":\"DAI\""));
    assert!(json.contains("\"name\":\"Dai Stablecoin\""));
    assert!(json.contains("\"decimals\":18"));

    // Test deserialization
    let deserialized: TokenBalanceResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.token_address, result.token_address);
    assert_eq!(deserialized.symbol, result.symbol);
    assert_eq!(deserialized.name, result.name);
}

#[test]
fn test_token_balance_result_clone() {
    let result = TokenBalanceResult {
        address: "0x1".to_string(),
        token_address: "0x2".to_string(),
        symbol: Some("TEST".to_string()),
        name: Some("Test Token".to_string()),
        balance_raw: "100".to_string(),
        balance_formatted: "100 TEST".to_string(),
        decimals: 0,
        network: "TestNet".to_string(),
        block_number: 1,
    };

    let cloned = result.clone();
    assert_eq!(cloned.address, result.address);
    assert_eq!(cloned.token_address, result.token_address);
    assert_eq!(cloned.symbol, result.symbol);
    assert_eq!(cloned.name, result.name);
}

#[test]
fn test_token_balance_result_debug() {
    let result = TokenBalanceResult {
        address: "0xdebug".to_string(),
        token_address: "0xtoken".to_string(),
        symbol: Some("DBG".to_string()),
        name: None,
        balance_raw: "42".to_string(),
        balance_formatted: "42 DBG".to_string(),
        decimals: 0,
        network: "Debug".to_string(),
        block_number: 999,
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("TokenBalanceResult"));
    assert!(debug_str.contains("token_address"));
    assert!(debug_str.contains("0xtoken"));
    assert!(debug_str.contains("DBG"));
}

// ERC20 selector constants are private - tested indirectly through function calls

#[tokio::test]
async fn test_get_eth_balance_success() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock chain ID
    let _m1 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_chainId"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;

    // Mock balance
    let _m2 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_getBalance"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0xde0b6b3a7640000"}"#)
        .create_async()
        .await;

    // Mock block number
    let _m3 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_blockNumber"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x11a72a0"}"#)
        .create_async()
        .await;

    let result = get_eth_balance(
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
        Some(url),
        Some("TestNet".to_string()),
    )
    .await;

    assert!(result.is_ok());
    let balance = result.unwrap();
    assert_eq!(
        balance.address,
        "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123"
    );
    assert_eq!(balance.balance_raw, "1000000000000000000");
    assert!(balance.balance_formatted.contains("ETH"));
    assert_eq!(balance.decimals, 18);
    assert_eq!(balance.network, "TestNet");
    assert!(balance.block_number > 18500000); // Should be a reasonable recent block
}

#[tokio::test]
async fn test_get_eth_balance_invalid_address() {
    let result = get_eth_balance("invalid_address".to_string(), None, None).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid address"));
}

#[tokio::test]
async fn test_get_eth_balance_zero_balance() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock chain ID
    let _m1 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_chainId"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x89"}"#)
        .create_async()
        .await;

    // Mock zero balance
    let _m2 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_getBalance"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x0"}"#)
        .create_async()
        .await;

    // Mock block number
    let _m3 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_blockNumber"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .create_async()
        .await;

    let result = get_eth_balance(
        "0x0000000000000000000000000000000000000000".to_string(),
        Some(url),
        None,
    )
    .await;

    assert!(result.is_ok());
    let balance = result.unwrap();
    assert_eq!(balance.balance_raw, "0");
    assert!(balance.balance_formatted.contains("0.000000"));
    assert_eq!(balance.network, "Polygon"); // Chain ID 137
}

#[tokio::test]
async fn test_get_erc20_balance_success() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock chain ID
    let _m1 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_chainId"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0xa4b1"}"#)
        .create_async()
        .await;

    // Mock contract call for balance
    let _m2 = server.mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_call"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x00000000000000000000000000000000000000000000000000000000000f4240"}"#)
        .create_async()
        .await;

    // Mock block number
    let _m3 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_blockNumber"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x100"}"#)
        .create_async()
        .await;

    let result = get_erc20_balance(
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
        "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        Some(url),
        None,
    )
    .await;

    assert!(result.is_ok());
    let balance = result.unwrap();
    assert_eq!(
        balance.address,
        "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123"
    );
    assert_eq!(
        balance.token_address,
        "0xa0b86a33e6441c68e1a7e97c82b6baba4d45a9e3"
    );
    assert_eq!(balance.balance_raw, "1000000");
    assert_eq!(balance.decimals, 18);
    assert_eq!(balance.network, "Arbitrum One"); // Chain ID 42161
}

#[tokio::test]
async fn test_get_erc20_balance_invalid_addresses() {
    let result = get_erc20_balance(
        "invalid".to_string(),
        "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        None,
        None,
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid wallet address"));

    let result2 = get_erc20_balance(
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
        "invalid".to_string(),
        None,
        None,
    )
    .await;

    assert!(result2.is_err());
    assert!(result2
        .unwrap_err()
        .to_string()
        .contains("Invalid token address"));
}

#[tokio::test]
async fn test_get_multi_token_balances() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock chain ID (called multiple times)
    let _m1 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_chainId"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0xa"}"#)
        .expect(3) // Once per token
        .create_async()
        .await;

    // Mock contract calls for each token
    let _m2 = server.mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_call"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x0000000000000000000000000000000000000000000000000000000000001000"}"#)
        .expect(3)
        .create_async()
        .await;

    // Mock block numbers
    let _m3 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_blockNumber"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x200"}"#)
        .expect(3)
        .create_async()
        .await;

    let tokens = vec![
        "0x1111111111111111111111111111111111111111".to_string(),
        "0x2222222222222222222222222222222222222222".to_string(),
        "0x3333333333333333333333333333333333333333".to_string(),
    ];

    let result = get_multi_token_balances(
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
        tokens,
        Some(url),
        Some("TestChain".to_string()),
    )
    .await;

    assert!(result.is_ok());
    let balances = result.unwrap();
    assert_eq!(balances.len(), 3);

    for balance in balances {
        assert_eq!(
            balance.address,
            "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123"
        );
        assert_eq!(balance.balance_raw, "4096");
        assert_eq!(balance.network, "TestChain");
        assert_eq!(balance.block_number, 512);
    }
}

#[tokio::test]
async fn test_get_multi_token_balances_with_failures() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Mock chain ID - first token succeeds, second fails
    let _m1 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_chainId"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .expect(1)
        .create_async()
        .await;

    // First token succeeds
    let _m2 = server.mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_call"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x0000000000000000000000000000000000000000000000000000000000000100"}"#)
        .expect(1)
        .create_async()
        .await;

    let _m3 = server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_blockNumber"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .expect(1)
        .create_async()
        .await;

    let tokens = vec![
        "0x1111111111111111111111111111111111111111".to_string(),
        "invalid_token_address".to_string(), // This will fail
    ];

    let result = get_multi_token_balances(
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
        tokens,
        Some(url),
        None,
    )
    .await;

    assert!(result.is_ok());
    let balances = result.unwrap();
    assert_eq!(balances.len(), 1); // Only successful balance returned
    assert_eq!(
        balances[0].token_address,
        "0x1111111111111111111111111111111111111111"
    );
}

#[tokio::test]
async fn test_get_multi_token_balances_empty_list() {
    let result = get_multi_token_balances(
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
        vec![],
        None,
        None,
    )
    .await;

    assert!(result.is_ok());
    let balances = result.unwrap();
    assert_eq!(balances.len(), 0);
}

#[tokio::test]
async fn test_get_eth_balance_network_chain_detection() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Test different chain IDs for network name detection
    let test_cases = vec![
        (137, "Polygon"),
        (42161, "Arbitrum One"),
        (10, "Optimism"),
        (8453, "Base"),
        (999999, "Chain 999999"), // Unknown chain
    ];

    for (chain_id, expected_network) in test_cases {
        let _m1 = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_chainId"
            })))
            .with_body(&format!(
                r#"{{"jsonrpc":"2.0","id":1,"result":"0x{:x}"}}"#,
                chain_id
            ))
            .expect(1)
            .create_async()
            .await;

        let _m2 = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_getBalance"
            })))
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1000"}"#)
            .expect(1)
            .create_async()
            .await;

        let _m3 = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_blockNumber"
            })))
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x100"}"#)
            .expect(1)
            .create_async()
            .await;

        let result = get_eth_balance(
            "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
            Some(url.clone()),
            None,
        )
        .await;

        assert!(result.is_ok());
        let balance = result.unwrap();
        assert_eq!(balance.network, expected_network);
    }
}

#[tokio::test]
async fn test_get_erc20_balance_network_chain_detection() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Test different chain IDs for network name detection in ERC20 balance
    let test_cases = vec![
        (137, "Polygon"),
        (42161, "Arbitrum One"),
        (10, "Optimism"),
        (8453, "Base"),
        (777777, "Chain 777777"), // Unknown chain
    ];

    for (chain_id, expected_network) in test_cases {
        let _m1 = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_chainId"
            })))
            .with_body(&format!(
                r#"{{"jsonrpc":"2.0","id":1,"result":"0x{:x}"}}"#,
                chain_id
            ))
            .expect(1)
            .create_async()
            .await;

        let _m2 = server.mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_call"
            })))
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x0000000000000000000000000000000000000000000000000000000000002000"}"#)
            .expect(1)
            .create_async()
            .await;

        let _m3 = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_blockNumber"
            })))
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x200"}"#)
            .expect(1)
            .create_async()
            .await;

        let result = get_erc20_balance(
            "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
            "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
            Some(url.clone()),
            None,
        )
        .await;

        assert!(result.is_ok());
        let balance = result.unwrap();
        assert_eq!(balance.network, expected_network);
    }
}

#[tokio::test]
async fn test_get_eth_balance_formatting_edge_cases() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Test balance formatting for different amounts
    let test_cases = vec![
        ("0x1", "0.000000 ETH"),                     // Very small balance
        ("0xde0b6b3a7640000", "1.000000 ETH"),       // Exactly 1 ETH
        ("0x1bc16d674ec80000", "2.000000 ETH"),      // 2 ETH
        ("0x3635c9adc5dea00000", "1000.000000 ETH"), // 1000 ETH
    ];

    for (hex_balance, expected_formatted) in test_cases {
        let _m1 = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_chainId"
            })))
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
            .expect(1)
            .create_async()
            .await;

        let _m2 = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_getBalance"
            })))
            .with_body(&format!(
                r#"{{"jsonrpc":"2.0","id":1,"result":"{}"}}"#,
                hex_balance
            ))
            .expect(1)
            .create_async()
            .await;

        let _m3 = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_blockNumber"
            })))
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x100"}"#)
            .expect(1)
            .create_async()
            .await;

        let result = get_eth_balance(
            "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
            Some(url.clone()),
            None,
        )
        .await;

        assert!(result.is_ok());
        let balance = result.unwrap();
        assert_eq!(balance.balance_formatted, expected_formatted);
    }
}

#[tokio::test]
async fn test_get_erc20_balance_formatting_edge_cases() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Test balance formatting for different amounts - covers lines 192-196
    let test_cases = vec![
        ("0xde0b6b3a7640000", "1.000000 TOKEN"), // >= 1.0 format: 1 ETH in wei
        ("0x16345785d8a0000", "0.100000000 TOKEN"), // < 1.0 format: 0.1 ETH in wei
        ("0x1", "0.000000000 TOKEN"),            // Very small balance
    ];

    for (hex_balance, expected_formatted) in test_cases {
        let _m1 = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_chainId"
            })))
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
            .expect(1)
            .create_async()
            .await;

        let _m2 = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_call"
            })))
            .with_body(&format!(
                r#"{{"jsonrpc":"2.0","id":1,"result":"{}"}}"#,
                hex_balance
            ))
            .expect(1)
            .create_async()
            .await;

        let _m3 = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_blockNumber"
            })))
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x100"}"#)
            .expect(1)
            .create_async()
            .await;

        let result = get_erc20_balance(
            "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
            "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
            Some(url.clone()),
            None,
        )
        .await;

        assert!(result.is_ok());
        let balance = result.unwrap();
        assert_eq!(balance.balance_formatted, expected_formatted);
    }
}
