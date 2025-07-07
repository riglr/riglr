//! Tests for error conversion between tool-specific errors and ToolError

use riglr_core::error::ToolError;

#[test]
fn test_tool_error_creation() {
    // Test retriable error
    let retriable = ToolError::retriable("Network timeout");
    assert!(retriable.is_retriable());
    assert!(!retriable.is_rate_limited());
    
    // Test rate limited error
    let rate_limited = ToolError::rate_limited("Too many requests");
    assert!(rate_limited.is_retriable()); // Rate limited is also retriable
    assert!(rate_limited.is_rate_limited());
    
    // Test permanent error
    let permanent = ToolError::permanent("Invalid address format");
    assert!(!permanent.is_retriable());
    assert!(!permanent.is_rate_limited());
}

#[test]
fn test_tool_error_display() {
    let retriable = ToolError::retriable("Network timeout");
    assert_eq!(retriable.to_string(), "Retriable error: Network timeout");
    
    let rate_limited = ToolError::rate_limited("Too many requests");
    assert_eq!(rate_limited.to_string(), "Rate limited: Too many requests");
    
    let permanent = ToolError::permanent("Invalid address format");
    assert_eq!(permanent.to_string(), "Permanent error: Invalid address format");
}

#[cfg(feature = "solana_tools_integration")]
mod solana_integration_tests {
    use super::*;
    use riglr_solana_tools::error::SolanaToolError;
    
    #[test]
    fn test_solana_error_to_tool_error_conversion() {
        // Test RPC error becomes retriable
        let rpc_error = SolanaToolError::Rpc("Connection timeout".to_string());
        let tool_error: ToolError = rpc_error.into();
        assert!(tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());

        // Test rate limit error becomes rate limited
        let rate_limit_error = SolanaToolError::Rpc("429 - too many requests".to_string());
        let tool_error: ToolError = rate_limit_error.into();
        assert!(tool_error.is_rate_limited());
        assert!(tool_error.is_retriable());

        // Test validation error becomes permanent
        let invalid_address = SolanaToolError::InvalidAddress("Not a valid Solana address".to_string());
        let tool_error: ToolError = invalid_address.into();
        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());

        // Test transaction error with insufficient funds becomes permanent
        let tx_error = SolanaToolError::Transaction("insufficient funds".to_string());
        let tool_error: ToolError = tx_error.into();
        assert!(!tool_error.is_retriable());
        
        // Test transaction error without specific keywords becomes retriable
        let tx_error = SolanaToolError::Transaction("network error".to_string());
        let tool_error: ToolError = tx_error.into();
        assert!(tool_error.is_retriable());
    }
}