//! Comprehensive tests for error module

use riglr_core::CoreError;
use riglr_solana_tools::error::{Result, SolanaToolError};

#[test]
fn test_rpc_error() {
    let error = SolanaToolError::Rpc("Connection failed".to_string());
    assert_eq!(error.to_string(), "RPC error: Connection failed");

    let error2 = SolanaToolError::Rpc("Request timeout".to_string());
    assert_eq!(error2.to_string(), "RPC error: Request timeout");
}

#[test]
fn test_invalid_address_error() {
    let error = SolanaToolError::InvalidAddress("Invalid base58 string".to_string());
    assert_eq!(error.to_string(), "Invalid address: Invalid base58 string");

    let error2 = SolanaToolError::InvalidAddress("Wrong length".to_string());
    assert_eq!(error2.to_string(), "Invalid address: Wrong length");
}

#[test]
fn test_transaction_error() {
    let error = SolanaToolError::Transaction("Insufficient funds".to_string());
    assert_eq!(error.to_string(), "Transaction error: Insufficient funds");

    let error2 = SolanaToolError::Transaction("Account not found".to_string());
    assert_eq!(error2.to_string(), "Transaction error: Account not found");
}

#[test]
fn test_generic_error() {
    let error = SolanaToolError::Generic("Unknown error occurred".to_string());
    assert_eq!(
        error.to_string(),
        "Solana tool error: Unknown error occurred"
    );

    let error2 = SolanaToolError::Generic("Failed to parse response".to_string());
    assert_eq!(
        error2.to_string(),
        "Solana tool error: Failed to parse response"
    );
}

#[test]
fn test_serialization_error_from_json() {
    let invalid_json = "{ not valid json }";
    let json_err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
    let solana_error = SolanaToolError::from(json_err);
    assert!(solana_error.to_string().contains("Serialization error"));
}

#[test]
fn test_core_error_conversion() {
    let core_error = CoreError::Generic("Core system failure".to_string());
    let solana_error = SolanaToolError::from(core_error);
    assert!(solana_error.to_string().contains("Core error"));
}

#[test]
fn test_http_error_conversion() {
    // Create a reqwest error by attempting an invalid request
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result =
        runtime.block_on(async { reqwest::get("http://invalid-test-domain-12345.com").await });

    if let Err(req_err) = result {
        let solana_error = SolanaToolError::from(req_err);
        assert!(solana_error.to_string().contains("HTTP error"));
    }
}

#[test]
fn test_result_type_alias() {
    fn returns_ok() -> Result<i32> {
        Ok(42)
    }

    fn returns_err() -> Result<i32> {
        Err(SolanaToolError::Generic("test error".to_string()))
    }

    assert_eq!(returns_ok().unwrap(), 42);
    assert!(returns_err().is_err());
}

#[test]
fn test_error_debug_format() {
    let error = SolanaToolError::Transaction("Debug test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Transaction"));
    assert!(debug_str.contains("Debug test"));
}

#[test]
fn test_error_chain_propagation() {
    fn inner_operation() -> Result<()> {
        Err(SolanaToolError::Rpc("Inner failure".to_string()))
    }

    fn outer_operation() -> Result<()> {
        inner_operation().map_err(|e| SolanaToolError::Generic(format!("Outer wrapper: {}", e)))
    }

    let result = outer_operation();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Outer wrapper"));
    assert!(error.to_string().contains("RPC error"));
}

#[test]
fn test_all_error_variants() {
    let errors = vec![
        SolanaToolError::Rpc("rpc".to_string()),
        SolanaToolError::InvalidAddress("address".to_string()),
        SolanaToolError::Transaction("tx".to_string()),
        SolanaToolError::Generic("generic".to_string()),
    ];

    for error in errors {
        // Test string conversion
        let _ = error.to_string();
        // Test debug format
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_error_with_empty_messages() {
    let errors = vec![
        SolanaToolError::Rpc("".to_string()),
        SolanaToolError::InvalidAddress("".to_string()),
        SolanaToolError::Transaction("".to_string()),
        SolanaToolError::Generic("".to_string()),
    ];

    for error in errors {
        let error_str = error.to_string();
        assert!(!error_str.is_empty());
    }
}

#[test]
fn test_error_with_long_messages() {
    let long_msg = "x".repeat(10000);
    let errors = vec![
        SolanaToolError::Rpc(long_msg.clone()),
        SolanaToolError::InvalidAddress(long_msg.clone()),
        SolanaToolError::Transaction(long_msg.clone()),
        SolanaToolError::Generic(long_msg.clone()),
    ];

    for error in errors {
        let error_str = error.to_string();
        assert!(error_str.len() > 10000);
    }
}

#[test]
fn test_error_variants_display() {
    let rpc_err = SolanaToolError::Rpc("test".to_string());
    assert!(rpc_err.to_string().starts_with("RPC error:"));

    let addr_err = SolanaToolError::InvalidAddress("test".to_string());
    assert!(addr_err.to_string().starts_with("Invalid address:"));

    let tx_err = SolanaToolError::Transaction("test".to_string());
    assert!(tx_err.to_string().starts_with("Transaction error:"));

    let gen_err = SolanaToolError::Generic("test".to_string());
    assert!(gen_err.to_string().starts_with("Solana tool error:"));
}

#[test]
fn test_nested_errors() {
    let inner = SolanaToolError::InvalidAddress("bad address".to_string());
    let outer = SolanaToolError::Transaction(format!("Failed due to: {}", inner));

    assert!(outer.to_string().contains("Transaction error"));
    assert!(outer.to_string().contains("Invalid address"));
}
