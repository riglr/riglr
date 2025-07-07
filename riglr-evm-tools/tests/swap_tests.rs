//! Comprehensive tests for swap module

use riglr_evm_tools::swap::*;
use riglr_evm_tools::transaction::TransactionStatus;
use serde_json::json;

#[test]
fn test_uniswap_config_ethereum() {
    let config = UniswapConfig::ethereum();
    
    assert_eq!(config.router_address, "0xE592427A0AEce92De3Edee1F18E0157C05861564");
    assert_eq!(config.quoter_address, "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6");
    assert_eq!(config.slippage_bps, 50);
    assert_eq!(config.deadline_seconds, 300);
}

#[test]
fn test_uniswap_config_polygon() {
    let config = UniswapConfig::polygon();
    
    assert_eq!(config.router_address, "0xE592427A0AEce92De3Edee1F18E0157C05861564");
    assert_eq!(config.quoter_address, "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6");
    assert_eq!(config.slippage_bps, 50);
    assert_eq!(config.deadline_seconds, 300);
}

#[test]
fn test_uniswap_config_arbitrum() {
    let config = UniswapConfig::arbitrum();
    
    assert_eq!(config.router_address, "0xE592427A0AEce92De3Edee1F18E0157C05861564");
    assert_eq!(config.quoter_address, "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6");
    assert_eq!(config.slippage_bps, 50);
    assert_eq!(config.deadline_seconds, 300);
}

#[test]
fn test_uniswap_config_default() {
    let config = UniswapConfig::default();
    let ethereum_config = UniswapConfig::ethereum();
    
    assert_eq!(config.router_address, ethereum_config.router_address);
    assert_eq!(config.quoter_address, ethereum_config.quoter_address);
    assert_eq!(config.slippage_bps, ethereum_config.slippage_bps);
    assert_eq!(config.deadline_seconds, ethereum_config.deadline_seconds);
}

#[test]
fn test_uniswap_config_clone() {
    let config = UniswapConfig::ethereum();
    let cloned = config.clone();
    
    assert_eq!(cloned.router_address, config.router_address);
    assert_eq!(cloned.quoter_address, config.quoter_address);
    assert_eq!(cloned.slippage_bps, config.slippage_bps);
    assert_eq!(cloned.deadline_seconds, config.deadline_seconds);
}

#[test]
fn test_uniswap_config_debug() {
    let config = UniswapConfig::ethereum();
    let debug_str = format!("{:?}", config);
    
    assert!(debug_str.contains("UniswapConfig"));
    assert!(debug_str.contains("router_address"));
    assert!(debug_str.contains("quoter_address"));
    assert!(debug_str.contains("slippage_bps"));
}

// Tests for private functions removed - these are tested indirectly through public API

// Tests for private helper functions removed - tested through public API

#[test]
fn test_swap_quote_creation() {
    let quote = SwapQuote {
        token_in: "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        token_out: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        amount_in: 1000000,
        amount_out: 950000,
        fee_tier: 3000,
        price_impact_pct: 0.5,
        router_address: "0xE592427A0AEce92De3Edee1F18E0157C05861564".to_string(),
        network: "Ethereum".to_string(),
    };
    
    assert_eq!(quote.amount_in, 1000000);
    assert_eq!(quote.amount_out, 950000);
    assert_eq!(quote.fee_tier, 3000);
    assert_eq!(quote.price_impact_pct, 0.5);
}

#[test]
fn test_swap_quote_serialization() {
    let quote = SwapQuote {
        token_in: "0xtoken_in".to_string(),
        token_out: "0xtoken_out".to_string(),
        amount_in: 123456,
        amount_out: 123000,
        fee_tier: 500,
        price_impact_pct: 0.37,
        router_address: "0xrouter".to_string(),
        network: "TestNet".to_string(),
    };
    
    let json = serde_json::to_string(&quote).unwrap();
    assert!(json.contains("\"token_in\":\"0xtoken_in\""));
    assert!(json.contains("\"amount_in\":123456"));
    assert!(json.contains("\"fee_tier\":500"));
    assert!(json.contains("\"price_impact_pct\":0.37"));
    
    // Test deserialization
    let deserialized: SwapQuote = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.token_in, quote.token_in);
    assert_eq!(deserialized.amount_in, quote.amount_in);
    assert_eq!(deserialized.price_impact_pct, quote.price_impact_pct);
}

#[test]
fn test_swap_quote_clone() {
    let quote = SwapQuote {
        token_in: "0x1".to_string(),
        token_out: "0x2".to_string(),
        amount_in: 100,
        amount_out: 95,
        fee_tier: 100,
        price_impact_pct: 5.0,
        router_address: "0x3".to_string(),
        network: "Test".to_string(),
    };
    
    let cloned = quote.clone();
    assert_eq!(cloned.token_in, quote.token_in);
    assert_eq!(cloned.amount_in, quote.amount_in);
    assert_eq!(cloned.price_impact_pct, quote.price_impact_pct);
}

#[test]
fn test_swap_quote_debug() {
    let quote = SwapQuote {
        token_in: "0xin".to_string(),
        token_out: "0xout".to_string(),
        amount_in: 42,
        amount_out: 40,
        fee_tier: 3000,
        price_impact_pct: 4.76,
        router_address: "0xrouter".to_string(),
        network: "Debug".to_string(),
    };
    
    let debug_str = format!("{:?}", quote);
    assert!(debug_str.contains("SwapQuote"));
    assert!(debug_str.contains("0xin"));
    assert!(debug_str.contains("0xout"));
    assert!(debug_str.contains("42"));
}

#[test]
fn test_swap_result_creation() {
    let result = SwapResult {
        tx_hash: "0x1234567890abcdef".to_string(),
        token_in: "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        token_out: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        amount_in: 1000000,
        amount_out_minimum: 950000,
        fee_tier: 3000,
        status: TransactionStatus::Pending,
        network: "Ethereum".to_string(),
        gas_price: 30000000000,
        idempotency_key: Some("swap_123".to_string()),
    };
    
    assert_eq!(result.tx_hash, "0x1234567890abcdef");
    assert_eq!(result.amount_in, 1000000);
    assert_eq!(result.amount_out_minimum, 950000);
    assert_eq!(result.gas_price, 30000000000);
}

#[test]
fn test_swap_result_serialization() {
    let result = SwapResult {
        tx_hash: "0xhash".to_string(),
        token_in: "0xin".to_string(),
        token_out: "0xout".to_string(),
        amount_in: 1000,
        amount_out_minimum: 900,
        fee_tier: 500,
        status: TransactionStatus::Confirmed,
        network: "Polygon".to_string(),
        gas_price: 50000000000,
        idempotency_key: None,
    };
    
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"tx_hash\":\"0xhash\""));
    assert!(json.contains("\"amount_in\":1000"));
    assert!(json.contains("\"fee_tier\":500"));
    assert!(json.contains("\"status\":\"Confirmed\""));
    assert!(json.contains("\"network\":\"Polygon\""));
    
    // Test deserialization
    let deserialized: SwapResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tx_hash, result.tx_hash);
    assert_eq!(deserialized.amount_in, result.amount_in);
    assert_eq!(deserialized.gas_price, result.gas_price);
}

#[test]
fn test_swap_result_clone() {
    let result = SwapResult {
        tx_hash: "0xc".to_string(),
        token_in: "0xc1".to_string(),
        token_out: "0xc2".to_string(),
        amount_in: 999,
        amount_out_minimum: 990,
        fee_tier: 10000,
        status: TransactionStatus::Failed("slippage".to_string()),
        network: "Arbitrum".to_string(),
        gas_price: 100000000,
        idempotency_key: Some("key".to_string()),
    };
    
    let cloned = result.clone();
    assert_eq!(cloned.tx_hash, result.tx_hash);
    assert_eq!(cloned.amount_in, result.amount_in);
    assert_eq!(cloned.idempotency_key, result.idempotency_key);
}

#[test]
fn test_swap_result_debug() {
    let result = SwapResult {
        tx_hash: "0xdbg".to_string(),
        token_in: "0xd1".to_string(),
        token_out: "0xd2".to_string(),
        amount_in: 1,
        amount_out_minimum: 0,
        fee_tier: 100,
        status: TransactionStatus::Pending,
        network: "Base".to_string(),
        gas_price: 1,
        idempotency_key: None,
    };
    
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("SwapResult"));
    assert!(debug_str.contains("0xdbg"));
    assert!(debug_str.contains("0xd1"));
}

#[test]
fn test_token_price_info_creation() {
    let info = TokenPriceInfo {
        base_token: "0xbase".to_string(),
        quote_token: "0xquote".to_string(),
        price: 1.5,
        fee_tier: 3000,
        price_impact_pct: 0.1,
        network: "Ethereum".to_string(),
    };
    
    assert_eq!(info.base_token, "0xbase");
    assert_eq!(info.quote_token, "0xquote");
    assert_eq!(info.price, 1.5);
    assert_eq!(info.fee_tier, 3000);
}

#[test]
fn test_token_price_info_serialization() {
    let info = TokenPriceInfo {
        base_token: "0xAAA".to_string(),
        quote_token: "0xBBB".to_string(),
        price: 0.95,
        fee_tier: 500,
        price_impact_pct: 0.05,
        network: "Optimism".to_string(),
    };
    
    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("\"base_token\":\"0xAAA\""));
    assert!(json.contains("\"quote_token\":\"0xBBB\""));
    assert!(json.contains("\"price\":0.95"));
    assert!(json.contains("\"fee_tier\":500"));
    
    // Test deserialization
    let deserialized: TokenPriceInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.base_token, info.base_token);
    assert_eq!(deserialized.price, info.price);
}

#[test]
fn test_token_price_info_clone() {
    let info = TokenPriceInfo {
        base_token: "0x1".to_string(),
        quote_token: "0x2".to_string(),
        price: 2.5,
        fee_tier: 10000,
        price_impact_pct: 0.25,
        network: "Test".to_string(),
    };
    
    let cloned = info.clone();
    assert_eq!(cloned.base_token, info.base_token);
    assert_eq!(cloned.price, info.price);
    assert_eq!(cloned.price_impact_pct, info.price_impact_pct);
}

#[test]
fn test_token_price_info_debug() {
    let info = TokenPriceInfo {
        base_token: "0xDBG1".to_string(),
        quote_token: "0xDBG2".to_string(),
        price: 99.99,
        fee_tier: 100,
        price_impact_pct: 0.01,
        network: "DebugNet".to_string(),
    };
    
    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("TokenPriceInfo"));
    assert!(debug_str.contains("0xDBG1"));
    assert!(debug_str.contains("99.99"));
}

#[tokio::test]
async fn test_get_uniswap_quote_invalid_addresses() {
    let result = get_uniswap_quote(
        "invalid_token_in".to_string(),
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        "1000000".to_string(),
        3000,
        None,
        None,
    ).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid input token"));
    
    let result2 = get_uniswap_quote(
        "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        "invalid_token_out".to_string(),
        "1000000".to_string(),
        3000,
        None,
        None,
    ).await;
    
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("Invalid output token"));
}

#[tokio::test]
async fn test_get_uniswap_quote_invalid_amount() {
    let result = get_uniswap_quote(
        "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        "not_a_number".to_string(),
        3000,
        None,
        None,
    ).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid amount"));
}

#[tokio::test]
async fn test_perform_uniswap_swap_invalid_addresses() {
    let result = perform_uniswap_swap(
        "invalid".to_string(),
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        "1000000".to_string(),
        "950000".to_string(),
        3000,
        None,
        None,
        None,
        None,
        None,
        None,
    ).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid input token"));
}

#[tokio::test]
async fn test_perform_uniswap_swap_invalid_amounts() {
    let result = perform_uniswap_swap(
        "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        "invalid_amount".to_string(),
        "950000".to_string(),
        3000,
        None,
        None,
        None,
        None,
        None,
        None,
    ).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid input amount"));
    
    let result2 = perform_uniswap_swap(
        "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        "1000000".to_string(),
        "invalid_min".to_string(),
        3000,
        None,
        None,
        None,
        None,
        None,
        None,
    ).await;
    
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("Invalid minimum output amount"));
}

#[tokio::test]
async fn test_get_uniswap_quote_network_configurations() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    // Test different network configurations
    let test_cases = vec![
        (137, Some("polygon"), "polygon"),
        (42161, Some("arbitrum"), "arbitrum"),  
        (1, None, "ethereum"),
        (137, None, "polygon"), // Auto-detect based on chain ID
        (42161, None, "arbitrum"), // Auto-detect based on chain ID
        (999, None, "ethereum"), // Unknown chain falls back to ethereum
    ];
    
    for (chain_id, network_config, expected_config_type) in test_cases {
        let _m1 = server.mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_chainId"
            })))
            .with_body(&format!(r#"{{"jsonrpc":"2.0","id":1,"result":"0x{:x}"}}"#, chain_id))
            .expect(1)
            .create_async()
            .await;
        
        let _m2 = server.mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_call"
            })))
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x0000000000000000000000000000000000000000000000000000000000000400"}"#)
            .expect(1)
            .create_async()
            .await;
        
        let result = get_uniswap_quote(
            "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            "1000000".to_string(),
            3000,
            Some(url.clone()),
            network_config.map(|s| s.to_string()),
        ).await;
        
        assert!(result.is_ok(), "Failed for config: {:?}", expected_config_type);
        let quote = result.unwrap();
        
        // Verify the quote was created properly
        assert_eq!(quote.token_in, "0xa0b86a33e6441c68e1a7e97c82b6baba4d45a9e3");
        assert_eq!(quote.token_out, "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
        assert_eq!(quote.amount_in, 1000000);
        assert_eq!(quote.fee_tier, 3000);
    }
}

#[tokio::test]
async fn test_get_uniswap_quote_network_name_generation() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    // Test network name generation for different chain IDs - covers lines 150-157
    let test_cases = vec![
        (1, "Ethereum"),
        (137, "Polygon"),
        (42161, "Arbitrum One"),
        (10, "Optimism"),
        (8453, "Base"),
        (123456, "Chain 123456"), // Unknown chain
    ];
    
    for (chain_id, expected_network) in test_cases {
        let _m1 = server.mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_chainId"
            })))
            .with_body(&format!(r#"{{"jsonrpc":"2.0","id":1,"result":"0x{:x}"}}"#, chain_id))
            .expect(1)
            .create_async()
            .await;
        
        let _m2 = server.mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "method": "eth_call"
            })))
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x0000000000000000000000000000000000000000000000000000000000000500"}"#)
            .expect(1)
            .create_async()
            .await;
        
        let result = get_uniswap_quote(
            "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            "1000000".to_string(),
            500,
            Some(url.clone()),
            None,
        ).await;
        
        assert!(result.is_ok());
        let quote = result.unwrap();
        assert_eq!(quote.network, expected_network);
    }
}

#[test]
fn test_calculate_price_impact_edge_cases() {
    // Test the calculate_price_impact function - covers lines 455-469
    
    // Test zero amounts
    assert_eq!(calculate_price_impact(0, 1000), 0.0);
    assert_eq!(calculate_price_impact(1000, 0), 0.0);
    assert_eq!(calculate_price_impact(0, 0), 0.0);
    
    // Test high ratio (minimal impact)
    let minimal_impact = calculate_price_impact(1000000, 999500);
    assert!(minimal_impact >= 0.01); // Should be at least minimum impact
    
    // Test low ratio (high impact)
    let high_impact = calculate_price_impact(1000000, 900000);
    assert!(high_impact > 0.01);
    assert!(high_impact < 100.0);
    
    // Test equal amounts
    let equal_impact = calculate_price_impact(1000000, 1000000);
    assert_eq!(equal_impact, 0.01); // Should be minimum impact
}

#[test]
fn test_build_quote_call_data_comprehensive() {
    // Test the build_quote_call_data function - covers lines 376-400
    
    let token_in = "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3";
    let token_out = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    
    let result = build_quote_call_data(token_in, token_out, 3000, 1000000, 0).unwrap();
    
    // Should start with quoteExactInputSingle selector
    assert!(result.starts_with("0xf7729d43"));
    
    // Should contain token addresses (without 0x prefix, padded to 64 chars)
    let result_lower = result.to_lowercase();
    assert!(result_lower.contains("000000000000000000000000a0b86a33e6441c68e1a7e97c82b6baba4d45a9e3"));
    assert!(result_lower.contains("000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"));
    
    // Should contain fee tier as hex
    assert!(result.contains("0000000000000000000000000000000000000000000000000000000000000bb8")); // 3000 in hex
    
    // Test different parameters
    let result2 = build_quote_call_data(token_in, token_out, 500, 2000000, 100).unwrap();
    assert!(result2.starts_with("0xf7729d43"));
    assert_ne!(result, result2); // Should be different
}

#[test]
fn test_build_swap_call_data_comprehensive() {
    // Test the build_swap_call_data function - covers lines 402-452
    
    let token_in = "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3";
    let token_out = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let recipient = "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123";
    
    let result = build_swap_call_data(
        token_in,
        token_out,
        3000,
        recipient,
        1000000,
        950000,
        1700000000,
    ).unwrap();
    
    // Should start with exactInputSingle selector
    assert!(result.starts_with("0x414bf389"));
    
    // Should contain struct offset
    assert!(result.contains("0000000000000000000000000000000000000000000000000000000000000020"));
    
    // Should contain all the parameters
    let result_lower = result.to_lowercase();
    assert!(result_lower.contains("000000000000000000000000a0b86a33e6441c68e1a7e97c82b6baba4d45a9e3")); // token_in
    assert!(result_lower.contains("000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")); // token_out
    assert!(result_lower.contains("000000000000000000000000742d35cc6634c0532925a3b8d8e41e5d3e4f8123")); // recipient
    
    // Test with different parameters
    let result2 = build_swap_call_data(
        token_in,
        token_out,
        500,
        recipient,
        2000000,
        1900000,
        1800000000,
    ).unwrap();
    
    assert!(result2.starts_with("0x414bf389"));
    assert_ne!(result, result2); // Should be different
}

#[tokio::test]
async fn test_get_token_price_functionality() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    // Mock the chain ID and quote response
    let _m1 = server.mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_chainId"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .expect(1)
        .create_async()
        .await;
    
    let _m2 = server.mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_call"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x0000000000000000000000000000000000000000000000000000000000200000"}"#) // 2097152 in hex
        .expect(1)
        .create_async()
        .await;
    
    let result = get_token_price(
        "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        Some(500),
        Some(url.clone()),
    ).await;
    
    assert!(result.is_ok());
    let price_info = result.unwrap();
    assert_eq!(price_info.base_token.to_lowercase(), "0xa0b86a33e6441c68e1a7e97c82b6baba4d45a9e3");
    assert_eq!(price_info.quote_token.to_lowercase(), "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
    assert_eq!(price_info.fee_tier, 500);
    assert!(price_info.price > 0.0);
}

#[tokio::test]
async fn test_get_token_price_default_fee_tier() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    // Test default fee tier (3000) - covers line 356
    let _m1 = server.mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_chainId"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#)
        .expect(1)
        .create_async()
        .await;
    
    let _m2 = server.mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(json!({
            "method": "eth_call"
        })))
        .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x0000000000000000000000000000000000000000000000000000000000100000"}"#)
        .expect(1)
        .create_async()
        .await;
    
    let result = get_token_price(
        "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        None, // Should default to 3000
        Some(url.clone()),
    ).await;
    
    assert!(result.is_ok());
    let price_info = result.unwrap();
    assert_eq!(price_info.fee_tier, 3000); // Should use default
}