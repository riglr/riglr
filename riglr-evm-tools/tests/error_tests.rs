//! Comprehensive tests for error module

use riglr_evm_tools::error::{EvmToolError, Result};
use riglr_core::CoreError;

#[test]
fn test_rpc_error() {
    let error = EvmToolError::Rpc("Connection timeout".to_string());
    assert_eq!(error.to_string(), "RPC error: Connection timeout");
    
    let error2 = EvmToolError::Rpc("Invalid response".to_string());
    assert_eq!(error2.to_string(), "RPC error: Invalid response");
}

#[test]
fn test_invalid_address_error() {
    let error = EvmToolError::InvalidAddress("Not a valid hex address".to_string());
    assert_eq!(error.to_string(), "Invalid address: Not a valid hex address");
    
    let error2 = EvmToolError::InvalidAddress("Missing 0x prefix".to_string());
    assert_eq!(error2.to_string(), "Invalid address: Missing 0x prefix");
}

#[test]
fn test_contract_error() {
    let error = EvmToolError::Contract("Contract not found".to_string());
    assert_eq!(error.to_string(), "Contract error: Contract not found");
    
    let error2 = EvmToolError::Contract("Execution reverted".to_string());
    assert_eq!(error2.to_string(), "Contract error: Execution reverted");
}

#[test]
fn test_transaction_error() {
    let error = EvmToolError::Transaction("Insufficient gas".to_string());
    assert_eq!(error.to_string(), "Transaction error: Insufficient gas");
    
    let error2 = EvmToolError::Transaction("Nonce too low".to_string());
    assert_eq!(error2.to_string(), "Transaction error: Nonce too low");
}

#[test]
fn test_generic_error() {
    let error = EvmToolError::Generic("Something went wrong".to_string());
    assert_eq!(error.to_string(), "EVM tool error: Something went wrong");
    
    let error2 = EvmToolError::Generic("Unexpected error".to_string());
    assert_eq!(error2.to_string(), "EVM tool error: Unexpected error");
}

#[test]
fn test_serialization_error_from_json() {
    let invalid_json = "{ invalid json }";
    let json_err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
    let evm_error = EvmToolError::from(json_err);
    assert!(evm_error.to_string().contains("Serialization error"));
}

#[test]
fn test_core_error_conversion() {
    let core_error = CoreError::Generic("Core failure".to_string());
    let evm_error = EvmToolError::from(core_error);
    assert!(evm_error.to_string().contains("Core error"));
}

#[test]
fn test_http_error_conversion() {
    // Create a reqwest error by trying to build an invalid client
    let client_result = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(0))
        .build();
    
    if let Ok(client) = client_result {
        // Make an invalid request to trigger an error
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            client.get("http://invalid-domain-that-does-not-exist-12345.com")
                .send()
                .await
        });
        
        if let Err(req_err) = result {
            let evm_error = EvmToolError::from(req_err);
            assert!(evm_error.to_string().contains("HTTP error"));
        }
    }
}

#[test]
fn test_result_type_alias() {
    fn returns_ok() -> Result<String> {
        Ok("success".to_string())
    }
    
    fn returns_err() -> Result<String> {
        Err(EvmToolError::Generic("test error".to_string()))
    }
    
    assert_eq!(returns_ok().unwrap(), "success");
    assert!(returns_err().is_err());
}

#[test]
fn test_error_debug_format() {
    let error = EvmToolError::Rpc("Debug test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Rpc"));
    assert!(debug_str.contains("Debug test"));
}

#[test]
fn test_error_chain() {
    fn operation_that_fails() -> Result<()> {
        Err(EvmToolError::Contract("Operation failed".to_string()))
    }
    
    fn wrapper_operation() -> Result<()> {
        operation_that_fails().map_err(|e| {
            EvmToolError::Generic(format!("Wrapped error: {}", e))
        })
    }
    
    let result = wrapper_operation();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Wrapped error"));
}

#[test]
fn test_error_variants_equality() {
    let err1 = EvmToolError::InvalidAddress("test".to_string());
    let err2 = EvmToolError::InvalidAddress("test".to_string());
    
    // Test that errors with same content produce same string representation
    assert_eq!(err1.to_string(), err2.to_string());
}

#[test]
fn test_all_error_variants() {
    let errors = vec![
        EvmToolError::Rpc("rpc".to_string()),
        EvmToolError::InvalidAddress("addr".to_string()),
        EvmToolError::Contract("contract".to_string()),
        EvmToolError::Transaction("tx".to_string()),
        EvmToolError::Generic("generic".to_string()),
    ];
    
    for error in errors {
        // Test that all errors can be converted to string
        let _ = error.to_string();
        // Test debug format
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_error_with_empty_messages() {
    let errors = vec![
        EvmToolError::Rpc("".to_string()),
        EvmToolError::InvalidAddress("".to_string()),
        EvmToolError::Contract("".to_string()),
        EvmToolError::Transaction("".to_string()),
        EvmToolError::Generic("".to_string()),
    ];
    
    for error in errors {
        // Empty messages should still work
        let error_str = error.to_string();
        assert!(!error_str.is_empty());
    }
}

#[test]
fn test_error_with_long_messages() {
    let long_msg = "x".repeat(10000);
    let errors = vec![
        EvmToolError::Rpc(long_msg.clone()),
        EvmToolError::InvalidAddress(long_msg.clone()),
        EvmToolError::Contract(long_msg.clone()),
        EvmToolError::Transaction(long_msg.clone()),
        EvmToolError::Generic(long_msg.clone()),
    ];
    
    for error in errors {
        let error_str = error.to_string();
        assert!(error_str.len() > 10000);
    }
}