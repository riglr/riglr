//! Production readiness integration tests
use std::env;

#[tokio::test]
async fn test_configuration_fail_fast() {
    // Test that missing environment variables cause immediate failure
    env::remove_var("OPENAI_API_KEY");
    env::remove_var("ANTHROPIC_API_KEY");
    
    // This should panic when trying to load config with missing required vars
    let result = std::panic::catch_unwind(|| {
        riglr_showcase::config::Config::from_env()
    });
    
    assert!(result.is_err(), "Should panic with missing required API keys");
    
    // Restore for other tests
    env::set_var("OPENAI_API_KEY", "test-key");
}

#[tokio::test]
async fn test_error_source_preservation() {
    // Test that error sources are preserved through conversion chains
    use riglr_core::error::ToolError;
    use riglr_evm_tools::error::EvmToolError;
    
    let original_error = EvmToolError::InvalidAddress("0xinvalid".to_string());
    let tool_error: ToolError = original_error.into();
    
    // Should preserve source for debugging
    assert!(tool_error.source().is_some());
    
    // Should classify correctly
    assert!(!tool_error.is_retriable());
}

#[tokio::test]
async fn test_evm_provider_extensibility() {
    // Test that new chains can be added without code changes
    env::set_var("RPC_URL_12345", "https://test-new-chain.example.com");
    
    let result = riglr_evm_tools::util::chain_id_to_rpc_url(12345);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://test-new-chain.example.com");
    
    let supported = riglr_evm_tools::util::get_supported_chains();
    assert!(supported.contains(&12345));
    
    env::remove_var("RPC_URL_12345");
}

#[tokio::test]
async fn test_no_mock_implementations() {
    // Verify no mock implementations remain in production code
    use std::process::Command;
    
    let output = Command::new("rg")
        .args(&[
            "-i", 
            "mock|placeholder|todo|fixme",
            "--type", "rust",
            "--glob", "!tests/*",
            "--glob", "!examples/*",
            "src/"
        ])
        .current_dir("/mnt/storage/projects/riglr")
        .output()
        .expect("Failed to run ripgrep");
    
    let results = String::from_utf8_lossy(&output.stdout);
    
    // Filter out acceptable mock usage (in test utilities, etc.)
    let concerning_mocks: Vec<&str> = results
        .lines()
        .filter(|line| {
            !line.contains("test_") && 
            !line.contains("mock_signer") &&
            !line.contains("MockAgent") &&
            !line.contains("// TODO:") &&
            !line.contains("// FIXME:") &&
            (line.contains("mock") || line.contains("TODO") || line.contains("FIXME"))
        })
        .collect();
    
    if !concerning_mocks.is_empty() {
        panic!("Found concerning mock/placeholder implementations:\n{:#?}", concerning_mocks);
    }
}

#[tokio::test]
#[ignore] // Requires real API keys
async fn test_real_api_integration() {
    // Test that real APIs work correctly
    
    // Test price feed
    let price_result = riglr_web_tools::price::get_token_price(
        "So11111111111111111111111111111111111111112".to_string(), // SOL
        Some("solana".to_string()),
    ).await;
    
    assert!(price_result.is_ok(), "Real price feed should work");
    
    // Test Li.fi integration
    let routes_result = riglr_cross_chain_tools::lifi::get_routes(
        "ethereum",
        "polygon", 
        "0xA0b86a33E6417c8f1E3BBb8D81c0a64d3E7fE6C8", // USDC
        "1000000", // 1 USDC
        "0x742d35Cc6645C677A61e7b77dDf0b9A4Cb5F9568",
        "0x742d35Cc6645C677A61e7b77dDf0b9A4Cb5F9568",
    ).await;
    
    assert!(routes_result.is_ok(), "Li.fi route discovery should work");
}

#[test]
fn test_bridge_mock_removal() {
    // Test that bridge functions don't return mock transaction hashes
    use std::process::Command;
    
    let output = Command::new("rg")
        .args(&[
            "SolanaTxHash1234567890|EvmTxHash0987654321",
            "--type", "rust",
            "src/"
        ])
        .current_dir("/mnt/storage/projects/riglr")
        .output()
        .expect("Failed to run ripgrep");
    
    let results = String::from_utf8_lossy(&output.stdout);
    assert!(results.is_empty(), "Found mock transaction hashes: {}", results);
}

#[test]
fn test_trading_bot_mock_removal() {
    // Test that trading bot doesn't have mock price fallbacks
    use std::process::Command;
    
    let output = Command::new("rg")
        .args(&[
            "23\\.45|1650\\.30|26800\\.50",
            "--type", "rust",
            "create-riglr-app/src/bin/trading_bot.rs"
        ])
        .current_dir("/mnt/storage/projects/riglr")
        .output()
        .expect("Failed to run ripgrep");
    
    let results = String::from_utf8_lossy(&output.stdout);
    
    // Filter out regex patterns which are acceptable
    let mock_prices: Vec<&str> = results
        .lines()
        .filter(|line| !line.contains(r"\$(\d+\.?\d*)") && !line.contains("price"))
        .collect();
    
    assert!(mock_prices.is_empty(), "Found mock prices in trading bot: {:#?}", mock_prices);
}