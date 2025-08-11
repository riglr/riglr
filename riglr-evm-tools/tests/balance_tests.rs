//! Comprehensive tests for balance module

use riglr_evm_tools::balance::{BalanceResult, TokenBalanceResult};

#[test]
fn test_balance_result_creation() {
    let result = BalanceResult {
        address: "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123".to_string(),
        balance_raw: "1000000000000000000".to_string(),
        balance_formatted: "1.000000".to_string(),
        unit: "ETH".to_string(),
        chain_id: 1,
        chain_name: "Ethereum Mainnet".to_string(),
        block_number: Some(18500000),
    };

    assert_eq!(result.address, "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123");
    assert_eq!(result.balance_raw, "1000000000000000000");
    assert_eq!(result.balance_formatted, "1.000000");
    assert_eq!(result.unit, "ETH");
    assert_eq!(result.chain_id, 1);
    assert_eq!(result.chain_name, "Ethereum Mainnet");
    assert_eq!(result.block_number, Some(18500000));
}

#[test]
fn test_balance_result_serialization() {
    let result = BalanceResult {
        address: "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123".to_string(),
        balance_raw: "2000000000000000000".to_string(),
        balance_formatted: "2.000000".to_string(),
        unit: "ETH".to_string(),
        chain_id: 137,
        chain_name: "Polygon".to_string(),
        block_number: Some(50000000),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"address\""));
    assert!(json.contains("\"balance_raw\""));
    assert!(json.contains("\"balance_formatted\""));
    assert!(json.contains("\"unit\":\"ETH\""));
    assert!(json.contains("\"chain_id\":137"));
    assert!(json.contains("\"chain_name\":\"Polygon\""));
    assert!(json.contains("\"block_number\":50000000"));

    // Test deserialization
    let deserialized: BalanceResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.address, result.address);
    assert_eq!(deserialized.balance_raw, result.balance_raw);
    assert_eq!(deserialized.chain_name, result.chain_name);
}

#[test]
fn test_balance_result_clone() {
    let result = BalanceResult {
        address: "0x123".to_string(),
        balance_raw: "1000".to_string(),
        balance_formatted: "0.000001".to_string(),
        unit: "ETH".to_string(),
        chain_id: 1,
        chain_name: "Test".to_string(),
        block_number: Some(100),
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
        balance_formatted: "1.000000".to_string(),
        unit: "ETH".to_string(),
        chain_id: 1,
        chain_name: "Debug".to_string(),
        block_number: Some(1),
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
        token_symbol: Some("USDC".to_string()),
        token_name: Some("USD Coin".to_string()),
        decimals: 6,
        balance_raw: "1000000".to_string(),
        balance_formatted: "1.000000".to_string(),
        chain_id: 1,
        chain_name: "Ethereum Mainnet".to_string(),
    };

    assert_eq!(result.address, "0x742d35cc6634c0532925a3b8d8e41e5d3e4f8123");
    assert_eq!(
        result.token_address,
        "0xa0b86a33e6441c68e1a7e97c82b6baba4d45a9e3"
    );
    assert_eq!(result.token_symbol, Some("USDC".to_string()));
    assert_eq!(result.token_name, Some("USD Coin".to_string()));
    assert_eq!(result.decimals, 6);
}

#[test]
fn test_token_balance_result_with_none_values() {
    let result = TokenBalanceResult {
        address: "0x123".to_string(),
        token_address: "0x456".to_string(),
        token_symbol: None,
        token_name: None,
        decimals: 18,
        balance_raw: "0".to_string(),
        balance_formatted: "0".to_string(),
        chain_id: 1,
        chain_name: "Test".to_string(),
    };

    assert!(result.token_symbol.is_none());
    assert!(result.token_name.is_none());
}

#[test]
fn test_token_balance_result_serialization() {
    let result = TokenBalanceResult {
        address: "0xabc".to_string(),
        token_address: "0xdef".to_string(),
        token_symbol: Some("DAI".to_string()),
        token_name: Some("Dai Stablecoin".to_string()),
        decimals: 18,
        balance_raw: "5000000000000000000".to_string(),
        balance_formatted: "5.000000000000000000".to_string(),
        chain_id: 42161,
        chain_name: "Arbitrum One".to_string(),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"token_address\""));
    assert!(json.contains("\"token_symbol\":\"DAI\""));
    assert!(json.contains("\"token_name\":\"Dai Stablecoin\""));
    assert!(json.contains("\"decimals\":18"));

    // Test deserialization
    let deserialized: TokenBalanceResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.token_address, result.token_address);
    assert_eq!(deserialized.token_symbol, result.token_symbol);
    assert_eq!(deserialized.token_name, result.token_name);
}

#[test]
fn test_token_balance_result_clone() {
    let result = TokenBalanceResult {
        address: "0x1".to_string(),
        token_address: "0x2".to_string(),
        token_symbol: Some("TEST".to_string()),
        token_name: Some("Test Token".to_string()),
        decimals: 0,
        balance_raw: "100".to_string(),
        balance_formatted: "100".to_string(),
        chain_id: 1,
        chain_name: "TestNet".to_string(),
    };

    let cloned = result.clone();
    assert_eq!(cloned.address, result.address);
    assert_eq!(cloned.token_address, result.token_address);
    assert_eq!(cloned.token_symbol, result.token_symbol);
    assert_eq!(cloned.token_name, result.token_name);
}

#[test]
fn test_token_balance_result_debug() {
    let result = TokenBalanceResult {
        address: "0xdebug".to_string(),
        token_address: "0xtoken".to_string(),
        token_symbol: Some("DBG".to_string()),
        token_name: None,
        decimals: 0,
        balance_raw: "42".to_string(),
        balance_formatted: "42".to_string(),
        chain_id: 1,
        chain_name: "Debug".to_string(),
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("TokenBalanceResult"));
    assert!(debug_str.contains("token_address"));
    assert!(debug_str.contains("0xtoken"));
    assert!(debug_str.contains("DBG"));
}

// Note: Integration tests that require actual network calls would need to be implemented
// when proper test infrastructure is set up. The functions get_eth_balance and get_erc20_balance
// have signatures:
// - get_eth_balance(address: String, block_number: Option<u64>) -> Result<BalanceResult, EvmToolError>
// - get_erc20_balance(address: String, token_address: String, fetch_metadata: Option<bool>) -> Result<TokenBalanceResult, EvmToolError>
//
// These tests verify the data structures compile and serialize/deserialize correctly.
