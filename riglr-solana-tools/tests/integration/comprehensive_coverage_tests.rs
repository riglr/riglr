//! Comprehensive integration tests to fill remaining coverage gaps
//!
//! This file contains tests specifically designed to cover any remaining
//! untested code paths and integration scenarios.

use riglr_solana_tools::error::{SolanaToolError, Result};
use riglr_solana_tools::signer::local::LocalSolanaSigner;
use riglr_core::signer::{TransactionSigner, SignerContext};
use solana_sdk::{
    signature::Keypair,
    transaction::Transaction,
    system_instruction,
    pubkey::Pubkey,
    hash::Hash,
};
use serde_json::json;
use std::sync::Arc;

/// Test complete error result type usage
#[test]
fn test_result_type_comprehensive() {
    // Test Ok result
    let ok_result: Result<i32> = Ok(42);
    assert!(ok_result.is_ok());
    assert_eq!(ok_result.unwrap(), 42);
    
    // Test various error types in Result
    let rpc_error: Result<()> = Err(SolanaToolError::Rpc("RPC failure".to_string()));
    assert!(rpc_error.is_err());
    
    let addr_error: Result<()> = Err(SolanaToolError::InvalidAddress("Bad address".to_string()));
    assert!(addr_error.is_err());
    
    let key_error: Result<()> = Err(SolanaToolError::InvalidKey("Bad key".to_string()));
    assert!(key_error.is_err());
    
    let tx_error: Result<()> = Err(SolanaToolError::Transaction("TX failed".to_string()));
    assert!(tx_error.is_err());
    
    let generic_error: Result<()> = Err(SolanaToolError::Generic("Generic failure".to_string()));
    assert!(generic_error.is_err());
}

/// Test error display implementations comprehensively
#[test]
fn test_error_display_implementations() {
    let errors = vec![
        SolanaToolError::Rpc("Network timeout occurred".to_string()),
        SolanaToolError::InvalidAddress("Address format invalid".to_string()),
        SolanaToolError::InvalidKey("Key format invalid".to_string()),
        SolanaToolError::Transaction("Transaction execution failed".to_string()),
        SolanaToolError::Generic("Generic system error".to_string()),
    ];
    
    for error in errors {
        let display_str = error.to_string();
        let debug_str = format!("{:?}", error);
        
        // Display string should be user-friendly
        assert!(!display_str.is_empty());
        assert!(!display_str.contains("SolanaToolError"));
        
        // Debug string should contain type information
        assert!(debug_str.contains("SolanaToolError"));
        
        // Test error formatting consistency
        match error {
            SolanaToolError::Rpc(msg) => {
                assert!(display_str.contains("RPC error"));
                assert!(display_str.contains(&msg));
            },
            SolanaToolError::InvalidAddress(msg) => {
                assert!(display_str.contains("Invalid address"));
                assert!(display_str.contains(&msg));
            },
            SolanaToolError::InvalidKey(msg) => {
                assert!(display_str.contains("Invalid key"));
                assert!(display_str.contains(&msg));
            },
            SolanaToolError::Transaction(msg) => {
                assert!(display_str.contains("Transaction error"));
                assert!(display_str.contains(&msg));
            },
            SolanaToolError::Generic(msg) => {
                assert!(display_str.contains("Solana tool error"));
                assert!(display_str.contains(&msg));
            },
            _ => {}, // Handle other variants like Serialization, Http, Core
        }
    }
}

/// Test complex error chain scenarios
#[test]
fn test_error_chain_scenarios() {
    // Test nested error scenarios
    
    // JSON serialization error chain
    let invalid_json = "{ not valid json }";
    let json_error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
    let solana_error = SolanaToolError::Serialization(json_error);
    
    // Test error source chain
    let error_string = solana_error.to_string();
    assert!(error_string.contains("Serialization error"));
    
    // Test error conversion chain
    let tool_error: riglr_core::error::ToolError = solana_error.into();
    assert!(!tool_error.is_retriable()); // Serialization errors should be permanent
    
    // Test HTTP error chain
    let http_error = reqwest::Error::from(reqwest::ErrorKind::Timeout);
    let solana_http_error = SolanaToolError::Http(http_error);
    let tool_error2: riglr_core::error::ToolError = solana_http_error.into();
    assert!(tool_error2.is_retriable()); // HTTP timeout should be retriable
}

/// Test signer module integration paths
#[tokio::test]
async fn test_signer_module_integration() {
    // Test the complete signer module integration
    let keypair = Keypair::new();
    let signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
    
    // Test all TransactionSigner trait methods
    assert!(signer.address().is_some());
    assert!(signer.pubkey().is_some());
    assert_eq!(signer.address(), signer.pubkey());
    
    // Test Solana client access
    let client = signer.solana_client();
    assert!(Arc::strong_count(&client) >= 1);
    
    // Test EVM client failure
    let evm_result = signer.evm_client();
    assert!(evm_result.is_err());
    
    // Test EVM transaction failure
    let evm_tx = alloy::rpc::types::TransactionRequest::default();
    let evm_sign_result = signer.sign_and_send_evm_transaction(evm_tx).await;
    assert!(evm_sign_result.is_err());
    
    // Test Solana transaction (will fail due to no real network, but tests the path)
    let payer = signer.keypair().pubkey();
    let recipient = Pubkey::new_unique();
    let instruction = system_instruction::transfer(&payer, &recipient, 1_000_000);
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer));
    
    let result = signer.sign_and_send_solana_transaction(&mut transaction).await;
    // Should get some kind of error (network, signing, etc.) but not a panic
    match result {
        Ok(_) => {}, // Unexpected but possible
        Err(_) => {}, // Expected due to no network
    }
}

/// Test error conversion comprehensive scenarios
#[test]
fn test_error_conversion_comprehensive() {
    // Test all error type conversions to ToolError
    
    // 1. RPC errors with different content
    let rpc_errors = vec![
        "429 Too Many Requests",
        "rate limit exceeded",
        "Connection timeout",
        "Network unreachable",
        "Unknown RPC error",
    ];
    
    for rpc_msg in rpc_errors {
        let error = SolanaToolError::Rpc(rpc_msg.to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        
        if rpc_msg.contains("429") || rpc_msg.contains("rate limit") {
            assert!(tool_error.is_rate_limited());
        } else {
            assert!(tool_error.is_retriable());
        }
    }
    
    // 2. Transaction errors with different content
    let tx_errors = vec![
        "insufficient funds for transaction",
        "invalid signature provided",
        "Network connectivity issue",
        "Unknown transaction error",
    ];
    
    for tx_msg in tx_errors {
        let error = SolanaToolError::Transaction(tx_msg.to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        
        if tx_msg.contains("insufficient funds") || tx_msg.contains("invalid") {
            assert!(!tool_error.is_retriable()); // Should be permanent
        } else {
            assert!(tool_error.is_retriable()); // Should be retriable
        }
    }
    
    // 3. Test address and key validation errors
    let addr_error = SolanaToolError::InvalidAddress("Not a valid base58 address".to_string());
    let tool_error: riglr_core::error::ToolError = addr_error.into();
    assert!(!tool_error.is_retriable()); // Should be permanent
    
    let key_error = SolanaToolError::InvalidKey("Invalid keypair format".to_string());
    let tool_error: riglr_core::error::ToolError = key_error.into();
    assert!(!tool_error.is_retriable()); // Should be permanent
    
    // 4. Test generic error
    let generic_error = SolanaToolError::Generic("Something went wrong".to_string());
    let tool_error: riglr_core::error::ToolError = generic_error.into();
    assert!(tool_error.is_retriable()); // Generic errors default to retriable
}

/// Test all module public interfaces
#[test]
fn test_module_public_interfaces() {
    // Test that all public modules and types are accessible
    
    // Error module
    let _error = SolanaToolError::Generic("test".to_string());
    
    // Test error result alias
    fn test_function() -> Result<String> {
        Ok("test".to_string())
    }
    assert!(test_function().is_ok());
    
    // Signer module
    let keypair = Keypair::new();
    let _signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
}

/// Test concurrent access to error conversion
#[test]
fn test_concurrent_error_handling() {
    use std::thread;
    use std::sync::Arc;
    
    let errors = Arc::new(vec![
        SolanaToolError::Rpc("RPC error".to_string()),
        SolanaToolError::InvalidAddress("Invalid address".to_string()),
        SolanaToolError::Transaction("Transaction failed".to_string()),
        SolanaToolError::Generic("Generic error".to_string()),
    ]);
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let errors_clone = errors.clone();
        let handle = thread::spawn(move {
            for _ in 0..100 {
                let error = &errors_clone[i % errors_clone.len()];
                
                // Test error display
                let _display = error.to_string();
                let _debug = format!("{:?}", error);
                
                // Test error conversion (clone to avoid moving)
                let cloned_error = match error {
                    SolanaToolError::Rpc(msg) => SolanaToolError::Rpc(msg.clone()),
                    SolanaToolError::InvalidAddress(msg) => SolanaToolError::InvalidAddress(msg.clone()),
                    SolanaToolError::Transaction(msg) => SolanaToolError::Transaction(msg.clone()),
                    SolanaToolError::Generic(msg) => SolanaToolError::Generic(msg.clone()),
                    _ => SolanaToolError::Generic("fallback".to_string()),
                };
                
                let _tool_error: riglr_core::error::ToolError = cloned_error.into();
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}

/// Test integration with riglr-core systems
#[tokio::test]
async fn test_riglr_core_integration() {
    use riglr_core::jobs::{Job, JobResult};
    
    // Test that our errors work well with core job system
    let job = Job::new("test_solana_tool", &json!({"test": true}), 3).unwrap();
    
    // Simulate a tool that uses Solana errors
    let simulate_error = || -> Result<JobResult> {
        // Simulate various error conditions
        Err(SolanaToolError::Rpc("Connection failed".to_string()))
    };
    
    match simulate_error() {
        Ok(result) => {
            assert!(result.is_success());
        },
        Err(e) => {
            // Convert to tool error
            let tool_error: riglr_core::error::ToolError = e.into();
            
            // Should be retriable for RPC errors
            assert!(tool_error.is_retriable());
            
            // Can create JobResult from error
            let job_result = if tool_error.is_retriable() {
                JobResult::retriable_failure(&tool_error.to_string())
            } else {
                JobResult::permanent_failure(&tool_error.to_string())
            };
            
            assert!(!job_result.is_success());
            assert!(job_result.is_retriable());
        }
    }
}

/// Test edge cases in error handling
#[test]
fn test_error_edge_cases() {
    // Empty error messages
    let empty_rpc = SolanaToolError::Rpc("".to_string());
    assert_eq!(empty_rpc.to_string(), "RPC error: ");
    
    let empty_addr = SolanaToolError::InvalidAddress("".to_string());
    assert_eq!(empty_addr.to_string(), "Invalid address: ");
    
    // Very long error messages
    let long_message = "x".repeat(10000);
    let long_error = SolanaToolError::Generic(long_message.clone());
    assert!(long_error.to_string().contains(&long_message));
    
    // Unicode error messages
    let unicode_error = SolanaToolError::Transaction("交易失败".to_string());
    assert!(unicode_error.to_string().contains("交易失败"));
    
    // Special characters
    let special_error = SolanaToolError::Rpc("Error: \"Connection refused\" (code: 111)".to_string());
    assert!(special_error.to_string().contains("Connection refused"));
}

/// Test signer context integration edge cases
#[tokio::test]
async fn test_signer_context_edge_cases() {
    // Clear any existing context
    SignerContext::clear().await;
    
    // Test accessing signer when none is set
    assert!(!SignerContext::is_available().await);
    let result = SignerContext::current().await;
    assert!(result.is_err());
    
    // Set a signer and test various operations
    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string()));
    SignerContext::set_current(signer.clone()).await;
    
    // Verify context is now available
    assert!(SignerContext::is_available().await);
    
    let current = SignerContext::current().await.unwrap();
    assert_eq!(current.address(), signer.address());
    
    // Test rapid context changes
    for i in 0..10 {
        let new_keypair = Keypair::new();
        let new_signer = Arc::new(LocalSolanaSigner::new(
            new_keypair, 
            format!("https://api-{}.devnet.solana.com", i)
        ));
        SignerContext::set_current(new_signer.clone()).await;
        
        let current = SignerContext::current().await.unwrap();
        assert_eq!(current.address(), new_signer.address());
    }
    
    // Clear and verify
    SignerContext::clear().await;
    assert!(!SignerContext::is_available().await);
}

/// Test comprehensive signer functionality
#[tokio::test]
async fn test_comprehensive_signer_functionality() {
    // Test signer creation from different sources
    let keypair1 = Keypair::new();
    let signer1 = LocalSolanaSigner::new(keypair1, "https://api.devnet.solana.com".to_string());
    
    // Test valid seed phrase
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let signer2 = LocalSolanaSigner::from_seed_phrase(seed_phrase, "https://api.testnet.solana.com".to_string()).unwrap();
    
    // Test that both signers are functional
    for signer in [&signer1, &signer2] {
        assert!(signer.address().is_some());
        assert!(signer.pubkey().is_some());
        assert_eq!(signer.address(), signer.pubkey());
        
        // Test debug output doesn't expose sensitive data
        let debug_str = format!("{:?}", signer);
        assert!(debug_str.contains("LocalSolanaSigner"));
        assert!(debug_str.contains("pubkey"));
        assert!(!debug_str.contains("private"));
        assert!(!debug_str.contains("secret"));
        
        // Test keypair access
        let keypair = signer.keypair();
        assert_eq!(keypair.pubkey().to_string(), signer.address().unwrap());
        
        // Test client access
        let client = signer.solana_client();
        assert!(Arc::strong_count(&client) >= 1);
        
        // Test EVM operations fail appropriately
        assert!(signer.evm_client().is_err());
        
        let evm_tx = alloy::rpc::types::TransactionRequest::default();
        let evm_result = signer.sign_and_send_evm_transaction(evm_tx).await;
        assert!(evm_result.is_err());
        match evm_result {
            Err(riglr_core::signer::SignerError::Configuration(msg)) => {
                assert!(msg.contains("EVM"));
            }
            _ => panic!("Expected Configuration error for EVM operations"),
        }
    }
}

/// Final integration validation
#[tokio::test] 
async fn test_final_integration_validation() {
    // This test ensures all major components work together
    
    // 1. Error system
    let error = SolanaToolError::Rpc("Integration test error".to_string());
    let tool_error: riglr_core::error::ToolError = error.into();
    assert!(tool_error.is_retriable());
    
    // 2. Signer system
    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string()));
    SignerContext::set_current(signer.clone()).await;
    
    assert!(SignerContext::is_available().await);
    let current = SignerContext::current().await.unwrap();
    assert_eq!(current.address(), signer.address());
    
    // 3. Job integration  
    use riglr_core::jobs::Job;
    let job = Job::new("integration_test", &json!({"test": "integration"}), 3).unwrap();
    assert_eq!(job.tool_name, "integration_test");
    assert_eq!(job.max_retries, 3);
    
    // 4. Clean up
    SignerContext::clear().await;
    assert!(!SignerContext::is_available().await);
}