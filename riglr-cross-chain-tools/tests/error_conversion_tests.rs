#[cfg(test)]
mod tests {
    use riglr_cross_chain_tools::error::CrossChainToolError;
    use riglr_core::ToolError;

    #[test]
    fn test_network_error_conversion() {
        let cc_error = CrossChainToolError::Network("Connection timeout".to_string());
        let tool_error: ToolError = cc_error.into();
        
        match tool_error {
            ToolError::Retriable(msg) => assert!(msg.contains("Connection timeout")),
            _ => panic!("Expected Retriable error"),
        }
    }

    #[test]
    fn test_insufficient_balance_conversion() {
        let cc_error = CrossChainToolError::InsufficientBalance("Not enough funds".to_string());
        let tool_error: ToolError = cc_error.into();
        
        match tool_error {
            ToolError::Retriable(msg) => assert!(msg.contains("Not enough funds")),
            _ => panic!("Expected Retriable error"),
        }
    }

    #[test]
    fn test_unsupported_chain_conversion() {
        let cc_error = CrossChainToolError::UnsupportedChain("Unknown chain".to_string());
        let tool_error: ToolError = cc_error.into();
        
        match tool_error {
            ToolError::Permanent(msg) => assert!(msg.contains("Unknown chain")),
            _ => panic!("Expected Permanent error"),
        }
    }

    #[test]
    fn test_bridge_error_conversion() {
        let cc_error = CrossChainToolError::BridgeError("Bridge protocol failed".to_string());
        let tool_error: ToolError = cc_error.into();
        
        match tool_error {
            ToolError::Permanent(msg) => assert!(msg.contains("Bridge protocol failed")),
            _ => panic!("Expected Permanent error"),
        }
    }

    #[test]
    fn test_invalid_address_conversion() {
        let cc_error = CrossChainToolError::InvalidAddress("Bad address format".to_string());
        let tool_error: ToolError = cc_error.into();
        
        match tool_error {
            ToolError::Permanent(msg) => assert!(msg.contains("Bad address format")),
            _ => panic!("Expected Permanent error"),
        }
    }

    #[test]
    fn test_transaction_failed_conversion() {
        let cc_error = CrossChainToolError::TransactionFailed("Tx reverted".to_string());
        let tool_error: ToolError = cc_error.into();
        
        match tool_error {
            ToolError::Permanent(msg) => assert!(msg.contains("Tx reverted")),
            _ => panic!("Expected Permanent error"),
        }
    }
}