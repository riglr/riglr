//! Tests for SolanaToolError to ToolError conversion

use riglr_core::error::ToolError;
use riglr_solana_tools::error::SolanaToolError;

#[test]
fn test_rpc_error_conversions() {
    // Test timeout error becomes retriable
    let timeout_error = SolanaToolError::Rpc("connection timeout occurred".to_string());
    let tool_error: ToolError = timeout_error.into();
    assert!(tool_error.is_retriable());
    assert!(!tool_error.is_rate_limited());
    assert_eq!(
        tool_error.to_string(),
        "Retriable error: connection timeout occurred"
    );

    // Test network error becomes retriable
    let network_error = SolanaToolError::Rpc("network connection failed".to_string());
    let tool_error: ToolError = network_error.into();
    assert!(tool_error.is_retriable());
    assert!(!tool_error.is_rate_limited());

    // Test 429 error becomes rate limited
    let rate_limit_error = SolanaToolError::Rpc("429 Too Many Requests".to_string());
    let tool_error: ToolError = rate_limit_error.into();
    assert!(tool_error.is_rate_limited());
    assert!(tool_error.is_retriable());
    assert_eq!(
        tool_error.to_string(),
        "Rate limited: 429 Too Many Requests"
    );

    // Test "rate limit" string becomes rate limited
    let rate_limit_error2 = SolanaToolError::Rpc("rate limit exceeded".to_string());
    let tool_error2: ToolError = rate_limit_error2.into();
    assert!(tool_error2.is_rate_limited());
    assert!(tool_error2.is_retriable());

    // Test "too many requests" becomes rate limited
    let rate_limit_error3 = SolanaToolError::Rpc("too many requests per minute".to_string());
    let tool_error3: ToolError = rate_limit_error3.into();
    assert!(tool_error3.is_rate_limited());
    assert!(tool_error3.is_retriable());

    // Test generic RPC error becomes retriable
    let generic_rpc = SolanaToolError::Rpc("RPC method failed".to_string());
    let tool_error: ToolError = generic_rpc.into();
    assert!(tool_error.is_retriable());
    assert!(!tool_error.is_rate_limited());
}

#[tokio::test]
async fn test_http_error_conversions() {
    // Test HTTP error conversion by creating actual reqwest errors
    // This tests the logic without mocking since reqwest errors are hard to construct

    // Test with an invalid URL to get a connection error
    let result = reqwest::get("http://invalid-domain-that-does-not-exist-12345.local").await;
    if let Err(http_error) = result {
        let solana_error = SolanaToolError::Http(http_error);
        let tool_error: ToolError = solana_error.into();

        // Connection errors should be retriable
        assert!(tool_error.is_retriable());
    }
}

#[test]
fn test_invalid_address_conversion() {
    let invalid_addr = SolanaToolError::InvalidAddress("Invalid base58 encoding".to_string());
    let tool_error: ToolError = invalid_addr.into();
    assert!(!tool_error.is_retriable());
    assert!(!tool_error.is_rate_limited());
    assert_eq!(
        tool_error.to_string(),
        "Permanent error: Invalid base58 encoding"
    );
}

#[test]
fn test_transaction_error_conversions() {
    // Test insufficient funds becomes permanent
    let insufficient_funds =
        SolanaToolError::Transaction("insufficient funds for transaction".to_string());
    let tool_error: ToolError = insufficient_funds.into();
    assert!(!tool_error.is_retriable());
    assert!(!tool_error.is_rate_limited());
    assert_eq!(
        tool_error.to_string(),
        "Permanent error: insufficient funds for transaction"
    );

    // Test invalid transaction becomes permanent
    let invalid_tx = SolanaToolError::Transaction("invalid transaction signature".to_string());
    let tool_error: ToolError = invalid_tx.into();
    assert!(!tool_error.is_retriable());
    assert!(!tool_error.is_rate_limited());

    // Test network transaction error becomes retriable
    let network_tx = SolanaToolError::Transaction("network error during broadcast".to_string());
    let tool_error: ToolError = network_tx.into();
    assert!(tool_error.is_retriable());
    assert!(!tool_error.is_rate_limited());

    // Test generic transaction error becomes retriable
    let generic_tx = SolanaToolError::Transaction("transaction processing failed".to_string());
    let tool_error: ToolError = generic_tx.into();
    assert!(tool_error.is_retriable());
    assert!(!tool_error.is_rate_limited());
}

#[test]
fn test_serialization_error_conversion() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let solana_error = SolanaToolError::Serialization(json_error);
    let tool_error: ToolError = solana_error.into();
    assert!(!tool_error.is_retriable());
    assert!(!tool_error.is_rate_limited());
    assert!(tool_error.to_string().contains("Permanent error"));
}

#[test]
fn test_core_error_conversion() {
    let core_error = riglr_core::CoreError::Generic("Core system failure".to_string());
    let solana_error = SolanaToolError::Core(core_error);
    let tool_error: ToolError = solana_error.into();
    assert!(tool_error.is_retriable());
    assert!(!tool_error.is_rate_limited());
    assert!(tool_error.to_string().contains("Retriable error"));
}

#[test]
fn test_generic_error_conversion() {
    let generic_error = SolanaToolError::Generic("Something went wrong".to_string());
    let tool_error: ToolError = generic_error.into();
    assert!(tool_error.is_retriable());
    assert!(!tool_error.is_rate_limited());
    assert_eq!(
        tool_error.to_string(),
        "Retriable error: Something went wrong"
    );
}

#[test]
fn test_case_insensitive_rate_limit_detection() {
    let test_cases = vec![
        "429 TOO MANY REQUESTS",
        "Rate Limit Exceeded",
        "TOO MANY REQUESTS",
        "rate limit error",
        "RATE LIMIT WARNING",
    ];

    for case in test_cases {
        let rpc_error = SolanaToolError::Rpc(case.to_string());
        let tool_error: ToolError = rpc_error.into();
        assert!(
            tool_error.is_rate_limited(),
            "Case '{}' should be rate limited",
            case
        );
        assert!(
            tool_error.is_retriable(),
            "Rate limited errors should also be retriable"
        );
    }
}

#[test]
fn test_case_insensitive_permanent_error_detection() {
    let test_cases = vec![
        "INSUFFICIENT FUNDS",
        "Invalid Address Format",
        "INVALID SIGNATURE",
        "invalid account",
    ];

    for case in test_cases {
        let tx_error = SolanaToolError::Transaction(case.to_string());
        let tool_error: ToolError = tx_error.into();
        assert!(
            !tool_error.is_retriable(),
            "Case '{}' should not be retriable",
            case
        );
        assert!(
            !tool_error.is_rate_limited(),
            "Case '{}' should not be rate limited",
            case
        );
    }
}

#[test]
fn test_retriable_error_detection() {
    let test_cases = vec![
        ("RPC", "connection failed"),
        ("RPC", "timeout occurred"),
        ("RPC", "network unreachable"),
        ("Transaction", "network error"),
        ("Transaction", "broadcast failed"),
        ("Generic", "temporary failure"),
    ];

    for (error_type, message) in test_cases {
        let solana_error = match error_type {
            "RPC" => SolanaToolError::Rpc(message.to_string()),
            "Transaction" => SolanaToolError::Transaction(message.to_string()),
            "Generic" => SolanaToolError::Generic(message.to_string()),
            _ => unreachable!(),
        };

        let tool_error: ToolError = solana_error.into();
        assert!(
            tool_error.is_retriable(),
            "{} error '{}' should be retriable",
            error_type,
            message
        );
        assert!(
            !tool_error.is_rate_limited(),
            "{} error '{}' should not be rate limited",
            error_type,
            message
        );
    }
}

#[test]
fn test_error_conversion_preserves_message() {
    let original_message = "This is a test error message";
    let solana_error = SolanaToolError::Generic(original_message.to_string());
    let tool_error: ToolError = solana_error.into();

    assert!(tool_error.to_string().contains(original_message));
}
