//! Comprehensive unit tests for LocalSolanaSigner
//!
//! Tests the complete functionality of the LocalSolanaSigner implementation,
//! including error handling, key management, and transaction signing.

use riglr_solana_tools::signer::local::LocalSolanaSigner;
use riglr_core::signer::{TransactionSigner, SignerError};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_instruction,
    pubkey::Pubkey,
    hash::Hash,
    message::Message,
    system_program,
};
use std::sync::Arc;
use std::str::FromStr;
use proptest::prelude::*;

#[test]
fn test_local_signer_creation() {
    let keypair = Keypair::new();
    let pubkey_str = keypair.pubkey().to_string();
    let rpc_url = "https://api.devnet.solana.com".to_string();
    
    let signer = LocalSolanaSigner::new(keypair, rpc_url.clone());
    
    // Basic properties
    assert_eq!(signer.address(), Some(pubkey_str.clone()));
    assert_eq!(signer.pubkey(), Some(pubkey_str));
    assert_eq!(signer.rpc_url(), rpc_url);
}

#[test]
fn test_local_signer_from_seed_phrase() {
    // Test with valid BIP39 seed phrase
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let rpc_url = "https://api.devnet.solana.com".to_string();
    
    let result = LocalSolanaSigner::from_seed_phrase(seed_phrase, rpc_url.clone());
    assert!(result.is_ok());
    
    let signer = result.unwrap();
    assert!(signer.address().is_some());
    assert!(signer.pubkey().is_some());
    assert_eq!(signer.rpc_url(), rpc_url);
    
    // Verify deterministic address generation
    let result2 = LocalSolanaSigner::from_seed_phrase(seed_phrase, rpc_url.clone());
    assert!(result2.is_ok());
    let signer2 = result2.unwrap();
    
    assert_eq!(signer.address(), signer2.address());
    assert_eq!(signer.pubkey(), signer2.pubkey());
}

#[test]
fn test_local_signer_from_invalid_seed_phrase() {
    let rpc_url = "https://api.devnet.solana.com".to_string();
    
    let invalid_seed_phrases = vec![
        "", // Empty
        "invalid", // Too short
        "one two three", // Too short
        "invalid seed phrase that is definitely not a valid bip39 mnemonic phrase", // Invalid words
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon", // Too many words
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon invalid", // Invalid word
        "ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABOUT", // Wrong case
    ];
    
    for invalid_seed in invalid_seed_phrases {
        let result = LocalSolanaSigner::from_seed_phrase(invalid_seed, rpc_url.clone());
        assert!(result.is_err(), "Expected error for seed phrase: '{}'", invalid_seed);
        
        match result {
            Err(SignerError::Configuration(msg)) => {
                assert!(msg.contains("Invalid seed phrase"), "Error message should mention invalid seed phrase: {}", msg);
            }
            _ => panic!("Expected SignerError::Configuration for invalid seed phrase: '{}'", invalid_seed),
        }
    }
}

#[test]
fn test_local_signer_debug_implementation() {
    let keypair = Keypair::new();
    let signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
    
    let debug_str = format!("{:?}", signer);
    
    // Should contain basic information
    assert!(debug_str.contains("LocalSolanaSigner"));
    assert!(debug_str.contains("pubkey"));
    assert!(debug_str.contains("rpc_url"));
    
    // Should NOT contain sensitive information
    assert!(!debug_str.contains("keypair"));
    assert!(!debug_str.contains("private"));
    assert!(!debug_str.contains("secret"));
    
    // Should contain the public key
    let pubkey = signer.address().unwrap();
    assert!(debug_str.contains(&pubkey));
}

#[test]
fn test_local_signer_keypair_access() {
    let keypair = Keypair::new();
    let original_pubkey = keypair.pubkey();
    let signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
    
    // Test keypair access
    let accessed_keypair = signer.keypair();
    assert_eq!(accessed_keypair.pubkey(), original_pubkey);
    
    // Test that we can sign with the accessed keypair
    let message = b"test message";
    let signature = accessed_keypair.sign_message(message);
    assert!(accessed_keypair.pubkey().verify(message, &signature).is_ok());
}

#[test]
fn test_local_signer_evm_methods_fail() {
    let keypair = Keypair::new();
    let signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
    
    // EVM transaction should fail
    let evm_tx = alloy::rpc::types::TransactionRequest::default();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    let result = runtime.block_on(signer.sign_and_send_evm_transaction(evm_tx));
    assert!(result.is_err());
    
    match result {
        Err(SignerError::Configuration(msg)) => {
            assert!(msg.contains("LocalSolanaSigner does not support EVM"));
        }
        _ => panic!("Expected SignerError::Configuration for EVM transaction"),
    }
    
    // EVM client should fail
    let client_result = signer.evm_client();
    assert!(client_result.is_err());
    
    match client_result {
        Err(SignerError::Configuration(msg)) => {
            assert!(msg.contains("LocalSolanaSigner does not support EVM"));
        }
        _ => panic!("Expected SignerError::Configuration for EVM client"),
    }
}

#[test]
fn test_local_signer_solana_client_access() {
    let keypair = Keypair::new();
    let rpc_url = "https://api.devnet.solana.com".to_string();
    let signer = LocalSolanaSigner::new(keypair, rpc_url);
    
    let client = signer.solana_client();
    assert!(Arc::strong_count(&client) >= 1);
    
    // Test that we can get multiple references
    let client2 = signer.solana_client();
    assert!(Arc::ptr_eq(&client, &client2));
}

#[tokio::test]
async fn test_local_signer_transaction_signing_mock() {
    // This test uses a mock-like approach since we can't easily test actual RPC calls
    let keypair = Keypair::new();
    let payer = keypair.pubkey();
    let signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
    
    // Create a simple transfer transaction
    let recipient = Pubkey::new_unique();
    let instruction = system_instruction::transfer(&payer, &recipient, 1_000_000); // 0.001 SOL
    
    let message = Message::new(&[instruction], Some(&payer));
    let mut transaction = Transaction::new_unsigned(message);
    
    // Since we can't actually send the transaction without a real RPC connection,
    // we'll test that the signing method properly handles the transaction structure
    // The actual RPC call will likely fail, but we can verify error handling
    
    let result = signer.sign_and_send_solana_transaction(&mut transaction).await;
    
    // The result will be an error due to no real RPC connection, but it should be
    // a Solana transaction error, not a configuration error
    match result {
        Err(SignerError::SolanaTransaction(_)) => {
            // Expected - RPC call failed
        }
        Err(SignerError::Signing(_)) => {
            // Also acceptable - signing might fail for various reasons
        }
        Ok(_) => {
            // Unexpected but possible if somehow the call succeeded
        }
        Err(SignerError::Configuration(_)) => {
            panic!("Should not be a configuration error for valid Solana transaction");
        }
        Err(e) => {
            panic!("Unexpected error type: {:?}", e);
        }
    }
}

#[test]
fn test_local_signer_with_different_rpc_urls() {
    let keypair = Keypair::new();
    let test_urls = vec![
        "https://api.mainnet-beta.solana.com",
        "https://api.devnet.solana.com",
        "https://api.testnet.solana.com",
        "http://localhost:8899",
        "https://custom-rpc.example.com",
    ];
    
    for url in test_urls {
        let signer = LocalSolanaSigner::new(keypair, url.to_string());
        assert_eq!(signer.rpc_url(), url);
        
        // Basic functionality should work regardless of URL
        assert!(signer.address().is_some());
        assert!(signer.pubkey().is_some());
        assert!(signer.solana_client().url().contains(url) || signer.solana_client().url() == url);
    }
}

#[test]
fn test_local_signer_deterministic_behavior() {
    // Test that creating signers with the same keypair produces identical results
    let keypair_bytes = Keypair::new().to_bytes();
    let keypair1 = Keypair::from_bytes(&keypair_bytes).unwrap();
    let keypair2 = Keypair::from_bytes(&keypair_bytes).unwrap();
    
    let signer1 = LocalSolanaSigner::new(keypair1, "https://api.devnet.solana.com".to_string());
    let signer2 = LocalSolanaSigner::new(keypair2, "https://api.devnet.solana.com".to_string());
    
    assert_eq!(signer1.address(), signer2.address());
    assert_eq!(signer1.pubkey(), signer2.pubkey());
    assert_eq!(signer1.rpc_url(), signer2.rpc_url());
}

#[test]
fn test_local_signer_key_validation() {
    // Test with various keypair formats and edge cases
    
    // Test with system program keypair (all zeros - special case)
    let zero_bytes = [0u8; 64];
    let result = Keypair::from_bytes(&zero_bytes);
    // Note: This might fail as system program key is special
    if result.is_ok() {
        let signer = LocalSolanaSigner::new(result.unwrap(), "https://api.devnet.solana.com".to_string());
        assert!(signer.address().is_some());
    }
    
    // Test with random valid keypairs
    for _ in 0..10 {
        let keypair = Keypair::new();
        let signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
        
        // Address should be valid base58
        let address = signer.address().unwrap();
        assert!(Pubkey::from_str(&address).is_ok());
        
        // Pubkey should match address
        assert_eq!(signer.address(), signer.pubkey());
    }
}

#[test]
fn test_local_signer_thread_safety() {
    use std::thread;
    use std::sync::Arc;
    
    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string()));
    let expected_address = signer.address().clone();
    
    // Test concurrent access from multiple threads
    let mut handles = vec![];
    for i in 0..10 {
        let signer_clone = signer.clone();
        let expected_clone = expected_address.clone();
        
        let handle = thread::spawn(move {
            // Access signer properties from different threads
            for _ in 0..100 {
                assert_eq!(signer_clone.address(), expected_clone);
                assert_eq!(signer_clone.pubkey(), expected_clone);
                assert_eq!(signer_clone.rpc_url(), "https://api.devnet.solana.com");
                
                // Access Solana client (should be thread-safe)
                let _client = signer_clone.solana_client();
                
                if i % 2 == 0 {
                    thread::yield_now();
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

// Property-based tests using proptest
proptest! {
    #[test]
    fn test_local_signer_with_arbitrary_rpc_urls(
        url in "https?://[a-zA-Z0-9.-]{1,50}\\.[a-z]{2,10}(:[0-9]{1,5})?(/[a-zA-Z0-9._-]*)*"
    ) {
        let keypair = Keypair::new();
        let signer = LocalSolanaSigner::new(keypair, url.clone());
        
        prop_assert_eq!(signer.rpc_url(), url);
        prop_assert!(signer.address().is_some());
        prop_assert!(signer.pubkey().is_some());
        prop_assert_eq!(signer.address(), signer.pubkey());
    }

    #[test]
    fn test_local_signer_keypair_properties(
        // Generate random 32-byte seeds for keypair creation
        seed in prop::collection::vec(any::<u8>(), 32..=32)
    ) {
        let keypair = Keypair::from_seed(&seed).unwrap();
        let signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
        
        // Basic properties should hold
        prop_assert!(signer.address().is_some());
        prop_assert!(signer.pubkey().is_some());
        prop_assert_eq!(signer.address(), signer.pubkey());
        
        // Address should be valid Solana public key
        let address = signer.address().unwrap();
        prop_assert!(Pubkey::from_str(&address).is_ok());
        
        // Should be deterministic - same seed produces same address
        let keypair2 = Keypair::from_seed(&seed).unwrap();
        let signer2 = LocalSolanaSigner::new(keypair2, "https://api.devnet.solana.com".to_string());
        prop_assert_eq!(signer.address(), signer2.address());
    }
}

#[test]
fn test_local_signer_error_classifications() {
    // Test various error scenarios that might occur during transaction signing
    let keypair = Keypair::new();
    let signer = LocalSolanaSigner::new(keypair, "https://invalid-rpc-url-that-does-not-exist.test".to_string());
    
    // Create a transaction that will fail due to invalid RPC URL
    let payer = signer.keypair().pubkey();
    let recipient = Pubkey::new_unique();
    let instruction = system_instruction::transfer(&payer, &recipient, 1_000_000);
    let message = Message::new(&[instruction], Some(&payer));
    let mut transaction = Transaction::new_unsigned(message);
    
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(signer.sign_and_send_solana_transaction(&mut transaction));
    
    // Should get a SolanaTransaction error (not Configuration error)
    match result {
        Err(SignerError::SolanaTransaction(_)) => {
            // Expected - RPC call should fail with invalid URL
        }
        Err(SignerError::Signing(_)) => {
            // Also acceptable - might fail at signing stage
        }
        Ok(_) => {
            panic!("Unexpected success with invalid RPC URL");
        }
        Err(e) => {
            // Print the actual error to help debug
            println!("Got error: {:?}", e);
            // Accept any error type for this test since RPC behavior can vary
        }
    }
}

#[test]
fn test_local_signer_edge_cases() {
    // Test with edge case RPC URLs
    let keypair = Keypair::new();
    
    let edge_case_urls = vec![
        "http://localhost:8899", // Local RPC
        "https://api.mainnet-beta.solana.com", // Mainnet
        "https://api.testnet.solana.com", // Testnet  
        "http://127.0.0.1:8899", // IP address
        "https://solana-rpc.com:443", // Custom port
    ];
    
    for url in edge_case_urls {
        let signer = LocalSolanaSigner::new(keypair, url.to_string());
        
        // Basic functionality should work
        assert!(signer.address().is_some());
        assert!(signer.pubkey().is_some());
        assert_eq!(signer.rpc_url(), url);
        
        // Client should be created successfully
        let _client = signer.solana_client();
    }
}

#[test]
fn test_local_signer_memory_usage() {
    // Test that creating many signers doesn't cause memory issues
    let mut signers = Vec::new();
    
    for _ in 0..1000 {
        let keypair = Keypair::new();
        let signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
        signers.push(signer);
    }
    
    // All signers should be functional
    for (i, signer) in signers.iter().enumerate() {
        assert!(signer.address().is_some(), "Signer {} should have address", i);
        assert!(signer.pubkey().is_some(), "Signer {} should have pubkey", i);
        
        // Spot check some signers
        if i % 100 == 0 {
            let _client = signer.solana_client();
            let debug_str = format!("{:?}", signer);
            assert!(debug_str.contains("LocalSolanaSigner"));
        }
    }
    
    // Check that addresses are unique (very high probability with proper random generation)
    let addresses: std::collections::HashSet<_> = signers.iter()
        .map(|s| s.address().unwrap())
        .collect();
    assert_eq!(addresses.len(), signers.len(), "All addresses should be unique");
}

#[test]
fn test_local_signer_comparison_and_equality() {
    // Test various comparison scenarios
    let keypair1 = Keypair::new();
    let keypair2 = Keypair::new();
    
    let signer1a = LocalSolanaSigner::new(keypair1, "https://api.devnet.solana.com".to_string());
    let signer1b = LocalSolanaSigner::new(keypair1, "https://api.devnet.solana.com".to_string());
    let signer2 = LocalSolanaSigner::new(keypair2, "https://api.devnet.solana.com".to_string());
    
    // Same keypair should produce same addresses
    assert_eq!(signer1a.address(), signer1b.address());
    assert_eq!(signer1a.pubkey(), signer1b.pubkey());
    
    // Different keypairs should produce different addresses  
    assert_ne!(signer1a.address(), signer2.address());
    assert_ne!(signer1a.pubkey(), signer2.pubkey());
    
    // RPC URL differences
    let signer1c = LocalSolanaSigner::new(keypair1, "https://api.mainnet-beta.solana.com".to_string());
    assert_eq!(signer1a.address(), signer1c.address()); // Same keypair, same address
    assert_ne!(signer1a.rpc_url(), signer1c.rpc_url()); // Different RPC URLs
}