//! Comprehensive tests for contract module

use riglr_evm_tools::{call_contract_read, call_contract_write, read_erc20_info};
use riglr_showcase::config::Config;

#[tokio::test]
#[ignore] // Integration test requiring API keys and signer context
async fn test_call_contract_read_testnet() {
    let config = Config::from_env();
    config.validate().expect("Invalid configuration");
    
    // Use a well-known ERC20 token on Sepolia testnet for testing
    // USDC on Sepolia: 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238
    let usdc_sepolia = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238";
    let zero_address = "0x0000000000000000000000000000000000000000";
    
    // This test requires a signer context to be set up
    // In practice, this would be set up by the calling environment
    let result = call_contract_read(
        usdc_sepolia.to_string(),
        "balanceOf(address)".to_string(),
        vec![zero_address.to_string()],
    ).await;
    
    // Validate real response structure - should return a number (balance)
    assert!(result.is_ok());
    let response = result.unwrap();
    // Balance should be a numeric string or "0"
    assert!(response.parse::<u64>().is_ok() || response == "0");
}

#[tokio::test]
#[ignore] // Integration test requiring testnet configuration and gas
async fn test_call_contract_write_testnet() {
    let config = Config::from_env();
    config.validate().expect("Invalid configuration");
    
    // Test contract write function (this would need gas and signer)
    // Using a safe read-only function that can also be called as write for testing
    let usdc_sepolia = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238";
    
    // This test requires a funded signer context
    let result = call_contract_write(
        usdc_sepolia.to_string(),
        "decimals()".to_string(), // Safe function that doesn't change state
        vec![],
        Some(50000), // Gas limit
    ).await;
    
    // Should fail gracefully if no signer context or return tx hash if successful
    match result {
        Ok(tx_hash) => {
            // Valid transaction hash should be 64 hex chars + 0x prefix
            assert!(tx_hash.starts_with("0x"));
            assert_eq!(tx_hash.len(), 66);
        },
        Err(e) => {
            // Expected if no signer context is set up
            let error_msg = e.to_string();
            assert!(error_msg.contains("signer") || error_msg.contains("context"));
        }
    }
}

#[tokio::test]
#[ignore] // Integration test requiring API keys
async fn test_call_contract_read_various_functions() {
    let config = Config::from_env();
    config.validate().expect("Invalid configuration");

    let usdc_sepolia = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238";

    // Test various ERC20 function signatures
    let functions = vec![
        ("totalSupply()", vec![]),
        ("decimals()", vec![]),
        ("symbol()", vec![]),
        ("name()", vec![]),
    ];

    for (func, params) in functions {
        let result = call_contract_read(
            usdc_sepolia.to_string(),
            func.to_string(),
            params,
        ).await;
        
        // All these functions should return valid data
        assert!(result.is_ok(), "Function {} failed: {:?}", func, result.err());
        let response = result.unwrap();
        assert!(!response.is_empty(), "Function {} returned empty response", func);
    }
}

#[tokio::test]
#[ignore] // Integration test requiring testnet configuration, gas, and signer
async fn test_erc20_info_reading() {
    let config = Config::from_env();
    config.validate().expect("Invalid configuration");

    let usdc_sepolia = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238";

    // Test the read_erc20_info function which combines multiple contract reads
    let result = read_erc20_info(usdc_sepolia.to_string()).await;
    
    match result {
        Ok(token_info) => {
            // Validate the returned JSON structure
            assert!(token_info.get("address").is_some());
            assert!(token_info.get("decimals").is_some());
            
            // USDC should have 6 decimals
            if let Some(decimals) = token_info.get("decimals") {
                assert_eq!(decimals.as_u64().unwrap_or(0), 6);
            }
        },
        Err(e) => {
            // Expected if no signer context is set up
            let error_msg = e.to_string();
            assert!(error_msg.contains("signer") || error_msg.contains("context"));
        }
    }
}

#[tokio::test]
#[ignore] // Integration test requiring API keys
async fn test_contract_address_validation() {
    // Test that the functions properly validate contract addresses
    let invalid_address = "invalid_address";
    
    let result = call_contract_read(
        invalid_address.to_string(),
        "totalSupply()".to_string(),
        vec![],
    ).await;
    
    // Should fail with address validation error
    assert!(result.is_err());
    let error_msg = result.err().unwrap().to_string();
    assert!(error_msg.contains("address") || error_msg.contains("invalid"));
}

#[tokio::test]
#[ignore] // Integration test requiring API keys
async fn test_function_signature_validation() {
    let usdc_sepolia = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238";
    
    // Test with invalid function signature
    let result = call_contract_read(
        usdc_sepolia.to_string(),
        "invalidFunction(".to_string(), // Malformed signature
        vec![],
    ).await;
    
    // Should fail with function signature error
    assert!(result.is_err());
    let error_msg = result.err().unwrap().to_string();
    assert!(error_msg.contains("function") || error_msg.contains("signature") || error_msg.contains("Invalid"));
}

#[tokio::test] 
#[ignore] // Integration test requiring API keys
async fn test_parameter_type_validation() {
    let usdc_sepolia = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238";
    
    // Test balanceOf with invalid address parameter
    let result = call_contract_read(
        usdc_sepolia.to_string(),
        "balanceOf(address)".to_string(),
        vec!["not_an_address".to_string()],
    ).await;
    
    // Should fail with parameter validation error
    assert!(result.is_err());
    let error_msg = result.err().unwrap().to_string();
    assert!(error_msg.contains("address") || error_msg.contains("parameter") || error_msg.contains("Invalid"));
}
