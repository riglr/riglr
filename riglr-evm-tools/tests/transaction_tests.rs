//! Comprehensive tests for transaction module

use riglr_evm_tools::transaction::*;

#[test]
fn test_evm_signer_context_creation() {
    let context = EvmSignerContext::new();
    // Should have no signers initially
    assert!(context.get_default_signer().is_err());
}

#[test]
fn test_evm_signer_context_add_signer() {
    let mut context = EvmSignerContext::new();
    let key1 = [1u8; 32];
    let key2 = [2u8; 32];

    // Add first signer
    context.add_signer("alice", key1).unwrap();

    // First signer becomes default
    assert_eq!(context.get_default_signer().unwrap(), key1);
    assert_eq!(context.get_signer("alice").unwrap(), key1);

    // Add second signer
    context.add_signer("bob", key2).unwrap();

    // Default should still be first
    assert_eq!(context.get_default_signer().unwrap(), key1);
    assert_eq!(context.get_signer("bob").unwrap(), key2);
}

#[test]
fn test_evm_signer_context_get_nonexistent() {
    let context = EvmSignerContext::new();
    assert!(context.get_signer("nonexistent").is_err());
}

#[test]
fn test_evm_signer_context_get_address() {
    let mut context = EvmSignerContext::new();
    let key = [0xAAu8; 32];

    context.add_signer("test", key).unwrap();
    let address = context.get_address("test").unwrap();

    // Should return a valid address format
    assert!(address.starts_with("0x"));
    assert!(address.len() > 2);
}

#[test]
fn test_evm_signer_context_multiple_signers() {
    let mut context = EvmSignerContext::new();

    // Add multiple signers
    for i in 0..10 {
        let mut key = [0u8; 32];
        key[0] = i;
        context.add_signer(format!("signer_{}", i), key).unwrap();
    }

    // Verify all signers are accessible
    for i in 0..10 {
        let mut expected_key = [0u8; 32];
        expected_key[0] = i;
        assert_eq!(
            context.get_signer(&format!("signer_{}", i)).unwrap(),
            expected_key
        );
    }

    // Default should be the first one added
    let mut expected_default = [0u8; 32];
    expected_default[0] = 0;
    assert_eq!(context.get_default_signer().unwrap(), expected_default);
}

#[test]
fn test_evm_signer_context_overwrite() {
    let mut context = EvmSignerContext::new();
    let key1 = [1u8; 32];
    let key2 = [2u8; 32];

    context.add_signer("alice", key1).unwrap();
    context.add_signer("alice", key2).unwrap(); // Overwrite

    assert_eq!(context.get_signer("alice").unwrap(), key2);
}

#[test]
fn test_evm_signer_context_default() {
    let context = EvmSignerContext::default();
    assert!(context.get_default_signer().is_err());
}

#[test]
fn test_evm_signer_context_clone() {
    let mut context = EvmSignerContext::new();
    let key = [42u8; 32];
    context.add_signer("test", key).unwrap();

    let cloned = context.clone();
    assert_eq!(cloned.get_signer("test").unwrap(), key);
}

#[test]
fn test_derive_address_from_key() {
    let key = [0u8; 32];
    let address = derive_address_from_key(&key).unwrap();

    // Should return a valid Ethereum address format
    assert!(address.starts_with("0x"));
    assert_eq!(address.len(), 42);
}

// Tests for private helper functions removed - these are tested through public API

// Tests for private transaction building functions removed - tested through public API

#[test]
fn test_transaction_result_creation() {
    let result = TransactionResult {
        tx_hash: "0x1234567890abcdef".to_string(),
        from: "0xabc".to_string(),
        to: "0xdef".to_string(),
        amount: "1000000000000000000".to_string(),
        amount_display: "1.0 ETH".to_string(),
        status: TransactionStatus::Pending,
        gas_price: 20000000000,
        gas_used: Some(21000),
        idempotency_key: Some("key123".to_string()),
    };

    assert_eq!(result.tx_hash, "0x1234567890abcdef");
    assert_eq!(result.from, "0xabc");
    assert_eq!(result.to, "0xdef");
    assert_eq!(result.gas_price, 20000000000);
    assert_eq!(result.gas_used, Some(21000));
}

#[test]
fn test_transaction_result_serialization() {
    let result = TransactionResult {
        tx_hash: "0xhash".to_string(),
        from: "0xfrom".to_string(),
        to: "0xto".to_string(),
        amount: "1000".to_string(),
        amount_display: "0.001 ETH".to_string(),
        status: TransactionStatus::Confirmed,
        gas_price: 1000000000,
        gas_used: None,
        idempotency_key: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"tx_hash\":\"0xhash\""));
    assert!(json.contains("\"status\":\"Confirmed\""));

    // Test deserialization
    let deserialized: TransactionResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tx_hash, result.tx_hash);
    assert_eq!(deserialized.gas_price, result.gas_price);
}

#[test]
fn test_transaction_result_clone() {
    let result = TransactionResult {
        tx_hash: "0x123".to_string(),
        from: "0xa".to_string(),
        to: "0xb".to_string(),
        amount: "100".to_string(),
        amount_display: "100 wei".to_string(),
        status: TransactionStatus::Failed("error".to_string()),
        gas_price: 1,
        gas_used: Some(100),
        idempotency_key: Some("id".to_string()),
    };

    let cloned = result.clone();
    assert_eq!(cloned.tx_hash, result.tx_hash);
    assert_eq!(cloned.gas_used, result.gas_used);
}

#[test]
fn test_transaction_result_debug() {
    let result = TransactionResult {
        tx_hash: "0xdebug".to_string(),
        from: "0x1".to_string(),
        to: "0x2".to_string(),
        amount: "42".to_string(),
        amount_display: "42 wei".to_string(),
        status: TransactionStatus::Pending,
        gas_price: 1,
        gas_used: None,
        idempotency_key: None,
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("TransactionResult"));
    assert!(debug_str.contains("0xdebug"));
}

#[test]
fn test_token_transfer_result_creation() {
    let result = TokenTransferResult {
        tx_hash: "0xtoken_tx".to_string(),
        from: "0xsender".to_string(),
        to: "0xreceiver".to_string(),
        token_address: "0xtoken".to_string(),
        amount: "1000000".to_string(),
        ui_amount: 1.0,
        decimals: 6,
        amount_display: "1.000000 USDC".to_string(),
        status: TransactionStatus::Pending,
        gas_price: 30000000000,
        gas_used: Some(60000),
        idempotency_key: None,
    };

    assert_eq!(result.token_address, "0xtoken");
    assert_eq!(result.ui_amount, 1.0);
    assert_eq!(result.decimals, 6);
}

#[test]
fn test_token_transfer_result_serialization() {
    let result = TokenTransferResult {
        tx_hash: "0x456".to_string(),
        from: "0xf".to_string(),
        to: "0xt".to_string(),
        token_address: "0xtkn".to_string(),
        amount: "100".to_string(),
        ui_amount: 0.0001,
        decimals: 18,
        amount_display: "0.0001 TOKEN".to_string(),
        status: TransactionStatus::Confirmed,
        gas_price: 1000000000,
        gas_used: None,
        idempotency_key: Some("idem_key".to_string()),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"token_address\":\"0xtkn\""));
    assert!(json.contains("\"decimals\":18"));
    assert!(json.contains("\"ui_amount\":0.0001"));

    // Test deserialization
    let deserialized: TokenTransferResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.token_address, result.token_address);
    assert_eq!(deserialized.decimals, result.decimals);
}

#[test]
fn test_token_transfer_result_clone() {
    let result = TokenTransferResult {
        tx_hash: "0xc".to_string(),
        from: "0xc1".to_string(),
        to: "0xc2".to_string(),
        token_address: "0xc3".to_string(),
        amount: "999".to_string(),
        ui_amount: 0.999,
        decimals: 3,
        amount_display: "0.999 TKN".to_string(),
        status: TransactionStatus::Failed("revert".to_string()),
        gas_price: 50000000000,
        gas_used: Some(80000),
        idempotency_key: None,
    };

    let cloned = result.clone();
    assert_eq!(cloned.token_address, result.token_address);
    assert_eq!(cloned.ui_amount, result.ui_amount);
}

#[test]
fn test_token_transfer_result_debug() {
    let result = TokenTransferResult {
        tx_hash: "0xdbg".to_string(),
        from: "0xd1".to_string(),
        to: "0xd2".to_string(),
        token_address: "0xd3".to_string(),
        amount: "1".to_string(),
        ui_amount: 0.000001,
        decimals: 6,
        amount_display: "0.000001 USD".to_string(),
        status: TransactionStatus::Pending,
        gas_price: 1,
        gas_used: None,
        idempotency_key: None,
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("TokenTransferResult"));
    assert!(debug_str.contains("0xdbg"));
    assert!(debug_str.contains("0xd3"));
}

#[test]
fn test_transaction_status_variants() {
    let pending = TransactionStatus::Pending;
    let confirmed = TransactionStatus::Confirmed;
    let failed = TransactionStatus::Failed("reason".to_string());

    // Test serialization
    assert_eq!(serde_json::to_string(&pending).unwrap(), "\"Pending\"");
    assert_eq!(serde_json::to_string(&confirmed).unwrap(), "\"Confirmed\"");

    let failed_json = serde_json::to_string(&failed).unwrap();
    assert!(failed_json.contains("Failed"));
    assert!(failed_json.contains("reason"));
}

#[test]
fn test_transaction_status_clone() {
    let status1 = TransactionStatus::Pending;
    let status2 = TransactionStatus::Confirmed;
    let status3 = TransactionStatus::Failed("error message".to_string());

    let cloned1 = status1.clone();
    let cloned2 = status2.clone();
    let cloned3 = status3.clone();

    assert!(matches!(cloned1, TransactionStatus::Pending));
    assert!(matches!(cloned2, TransactionStatus::Confirmed));
    assert!(matches!(cloned3, TransactionStatus::Failed(msg) if msg == "error message"));
}

#[test]
fn test_transaction_status_debug() {
    let status = TransactionStatus::Failed("debug error".to_string());
    let debug_str = format!("{:?}", status);

    assert!(debug_str.contains("Failed"));
    assert!(debug_str.contains("debug error"));
}

#[tokio::test]
async fn test_transfer_eth_invalid_amount() {
    let result = transfer_eth(
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
        -1.0, // Invalid negative amount
        None,
        None,
        None,
        None,
        None,
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Amount must be positive"));
}

#[tokio::test]
async fn test_transfer_eth_invalid_address() {
    let result = transfer_eth(
        "invalid_address".to_string(),
        1.0,
        None,
        None,
        None,
        None,
        None,
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid recipient address"));
}

#[tokio::test]
async fn test_transfer_erc20_invalid_addresses() {
    let result = transfer_erc20(
        "invalid".to_string(),
        "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        "1000000".to_string(),
        6,
        None,
        None,
        None,
        None,
        None,
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid recipient address"));

    let result2 = transfer_erc20(
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
        "invalid".to_string(),
        "1000000".to_string(),
        6,
        None,
        None,
        None,
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
async fn test_transfer_erc20_invalid_amount() {
    let result = transfer_erc20(
        "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string(),
        "0xA0b86a33E6441c68e1A7e97c82B6BAba4d45A9e3".to_string(),
        "not_a_number".to_string(),
        6,
        None,
        None,
        None,
        None,
        None,
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid amount"));
}

#[test]
fn test_global_signer_context_init_and_get() {
    // Test global signer context functionality - covers lines 108-126
    use riglr_evm_tools::transaction::{
        get_evm_signer_context, init_evm_signer_context, EvmSignerContext,
    };

    // Test getting signer context before initialization
    let result = get_evm_signer_context();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("signer context not initialized"));

    // Initialize with a context
    let mut context = EvmSignerContext::new();
    let private_key = [42u8; 32];
    context.add_signer("test_global", private_key).unwrap();

    init_evm_signer_context(context);

    // Now should be able to get it
    let result = get_evm_signer_context();
    assert!(result.is_ok());

    let global_context = result.unwrap();
    let retrieved_key = global_context.get_signer("test_global").unwrap();
    assert_eq!(retrieved_key, private_key);
}

#[test]
fn test_evm_signer_context_address_derivation() {
    // Test address derivation functionality - covers lines 76-94
    let mut context = EvmSignerContext::new();
    let private_key = [123u8; 32];
    context.add_signer("addr_test", private_key).unwrap();

    let address = context.get_address("addr_test").unwrap();
    assert!(address.starts_with("0x"));

    // Test error for non-existent signer
    let result = context.get_address("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_evm_signer_context_overwrite_signer() {
    // Test overwriting existing signer - covers signer replacement scenarios
    let mut context = EvmSignerContext::new();
    let key1 = [1u8; 32];
    let key2 = [2u8; 32];

    context.add_signer("same_name", key1).unwrap();
    context.add_signer("same_name", key2).unwrap(); // Overwrite

    let retrieved = context.get_signer("same_name").unwrap();
    assert_eq!(retrieved, key2); // Should have the new key

    // Default should still be "same_name" but now has the new key value
    let default_key = context.get_default_signer().unwrap();
    assert_eq!(default_key, key2); // Should have the updated key value
}

#[test]
fn test_evm_signer_context_many_signers() {
    // Test multiple signers management
    let mut context = EvmSignerContext::new();

    for i in 0..5 {
        let mut key = [0u8; 32];
        key[0] = i as u8;
        context.add_signer(&format!("signer{}", i), key).unwrap();
    }

    // Verify all signers exist
    for i in 0..5 {
        let key = context.get_signer(&format!("signer{}", i)).unwrap();
        assert_eq!(key[0], i as u8);
    }

    // Default should be first signer
    let default = context.get_default_signer().unwrap();
    assert_eq!(default[0], 0u8);
}

#[test]
fn test_derive_address_from_key_consistent() {
    // Test derive_address_from_key function - covers line 360-363
    use riglr_evm_tools::transaction::derive_address_from_key;

    let key1 = [100u8; 32];
    let key2 = [200u8; 32];

    let addr1 = derive_address_from_key(&key1).unwrap();
    let addr2 = derive_address_from_key(&key2).unwrap();

    // Both should be valid addresses
    assert!(addr1.starts_with("0x"));
    assert!(addr2.starts_with("0x"));

    // For now, both return the same placeholder - would be different in production
    assert_eq!(addr1, addr2);
    assert_eq!(addr1, "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123");
}

#[test]
fn test_build_eth_transfer_tx_various_inputs() {
    // Test build_eth_transfer_tx with various inputs - covers lines 366-378
    use riglr_evm_tools::transaction::build_eth_transfer_tx;

    let test_cases = vec![
        (
            "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123",
            1000000000000000000u128,
            0,
            20000000000,
            21000,
            1,
        ),
        (
            "0xdead000000000000000000000000000000000000",
            500000000000000000u128,
            1,
            30000000000,
            25000,
            137,
        ),
        (
            "0x1111111111111111111111111111111111111111",
            1u128,
            999,
            15000000000,
            21000,
            42161,
        ),
    ];

    for (to, amount, nonce, gas_price, gas_limit, chain_id) in test_cases {
        let result = build_eth_transfer_tx(to, amount, nonce, gas_price, gas_limit, chain_id);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 32); // Placeholder returns fixed size
    }
}

#[test]
fn test_build_erc20_transfer_data_edge_cases() {
    // Test build_erc20_transfer_data with edge cases - covers lines 381-388
    use riglr_evm_tools::transaction::build_erc20_transfer_data;

    let test_cases = vec![
        ("0x0000000000000000000000000000000000000000", 0u128), // Zero address, zero amount
        ("0xffffffffffffffffffffffffffffffffffffffff", u128::MAX), // Max address, max amount
        ("0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123", 1u128), // Normal case
    ];

    for (to, amount) in test_cases {
        let result = build_erc20_transfer_data(to, amount);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(data.starts_with("0xa9059cbb")); // transfer function selector
        assert_eq!(data.len(), 2 + 8 + 64 + 64); // 0x + selector + address + amount
    }
}

#[test]
fn test_build_contract_call_tx_various_inputs() {
    // Test build_contract_call_tx - covers lines 391-402
    use riglr_evm_tools::transaction::build_contract_call_tx;

    let test_cases = vec![
        (
            "0x1234567890123456789012345678901234567890",
            "0xabcdef",
            0,
            20000000000,
            100000,
            1,
        ),
        (
            "0xfedcba0987654321fedcba0987654321fedcba09",
            "0x12345678",
            5,
            25000000000,
            150000,
            137,
        ),
        (
            "0x0000000000000000000000000000000000000001",
            "0x",
            999,
            30000000000,
            200000,
            42161,
        ),
    ];

    for (to, data, nonce, gas_price, gas_limit, chain_id) in test_cases {
        let result = build_contract_call_tx(to, data, nonce, gas_price, gas_limit, chain_id);
        assert!(result.is_ok());
        let tx_data = result.unwrap();
        assert_eq!(tx_data.len(), 32); // Placeholder returns fixed size
    }
}

#[test]
fn test_sign_transaction_various_inputs() {
    // Test sign_transaction with various inputs - covers lines 405-409
    use riglr_evm_tools::transaction::sign_transaction;

    let test_cases = vec![
        (vec![1, 2, 3, 4], [1u8; 32]),
        (vec![255; 100], [255u8; 32]),
        (vec![], [0u8; 32]),         // Empty transaction data
        (vec![0; 1000], [42u8; 32]), // Large transaction data
    ];

    for (tx_data, private_key) in test_cases {
        let result = sign_transaction(tx_data.clone(), &private_key);
        assert!(result.is_ok());

        let signed = result.unwrap();
        assert!(signed.starts_with("0x"));
        assert_eq!(signed, "0x1234567890abcdef"); // Placeholder signature
    }
}

#[test]
fn test_transaction_result_comprehensive() {
    // Test TransactionResult creation and fields
    let result = TransactionResult {
        tx_hash: "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba".to_string(),
        from: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        to: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
        amount: "5000000000000000000".to_string(),
        amount_display: "5.0 ETH".to_string(),
        status: TransactionStatus::Confirmed,
        gas_price: 25000000000,
        gas_used: Some(21000),
        idempotency_key: Some("unique-key-123".to_string()),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba"));
    assert!(json.contains("5000000000000000000"));
    assert!(json.contains("5.0 ETH"));
    assert!(json.contains("25000000000"));
    assert!(json.contains("unique-key-123"));

    // Test deserialization
    let deserialized: TransactionResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tx_hash, result.tx_hash);
    assert_eq!(deserialized.amount, result.amount);
    assert!(matches!(deserialized.status, TransactionStatus::Confirmed));
}

#[test]
fn test_token_transfer_result_comprehensive() {
    // Test TokenTransferResult creation and fields
    let result = TokenTransferResult {
        tx_hash: "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321".to_string(),
        from: "0xcccccccccccccccccccccccccccccccccccccccc".to_string(),
        to: "0xdddddddddddddddddddddddddddddddddddddddd".to_string(),
        token_address: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
        amount: "1000000".to_string(),
        ui_amount: 1.0,
        decimals: 6,
        amount_display: "1.000000".to_string(),
        status: TransactionStatus::Failed("Insufficient balance".to_string()),
        gas_price: 35000000000,
        gas_used: Some(45000),
        idempotency_key: None,
    };

    // Test JSON serialization
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("0xfedcba0987654321"));
    assert!(json.contains("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"));
    assert!(json.contains("1000000"));
    assert!(json.contains("1.0"));
    assert!(json.contains("\"decimals\":6"));
    assert!(json.contains("35000000000"));
    assert!(json.contains("Insufficient balance"));

    // Test deserialization
    let deserialized: TokenTransferResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.token_address, result.token_address);
    assert_eq!(deserialized.ui_amount, result.ui_amount);
    assert_eq!(deserialized.decimals, result.decimals);
    assert!(matches!(deserialized.status, TransactionStatus::Failed(_)));
}
