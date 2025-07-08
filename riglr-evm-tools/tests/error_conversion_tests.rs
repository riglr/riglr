#[cfg(test)]
mod tests {
    use riglr_evm_tools::error::EvmToolError;
    use riglr_core::ToolError;

    #[test]
    fn test_rpc_error_conversion() {
        let evm_error = EvmToolError::Rpc("Connection failed".to_string());
        let tool_error: ToolError = evm_error.into();
        
        match tool_error {
            ToolError::Retriable(msg) => assert!(msg.contains("Connection failed")),
            _ => panic!("Expected Retriable error"),
        }
    }

    #[test]
    fn test_http_error_conversion() {
        let evm_error = EvmToolError::Http(reqwest::Error::from(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused, 
            "Connection refused"
        )));
        let tool_error: ToolError = evm_error.into();
        
        match tool_error {
            ToolError::Retriable(msg) => assert!(msg.contains("HTTP error")),
            _ => panic!("Expected Retriable error"),
        }
    }

    #[test]
    fn test_invalid_address_conversion() {
        let evm_error = EvmToolError::InvalidAddress("Bad address".to_string());
        let tool_error: ToolError = evm_error.into();
        
        match tool_error {
            ToolError::Permanent(msg) => assert!(msg.contains("Bad address")),
            _ => panic!("Expected Permanent error"),
        }
    }

    #[test]
    fn test_contract_error_conversion() {
        let evm_error = EvmToolError::Contract("Contract call failed".to_string());
        let tool_error: ToolError = evm_error.into();
        
        match tool_error {
            ToolError::Permanent(msg) => assert!(msg.contains("Contract call failed")),
            _ => panic!("Expected Permanent error"),
        }
    }

    #[test]
    fn test_transaction_error_conversion() {
        let evm_error = EvmToolError::Transaction("Transaction reverted".to_string());
        let tool_error: ToolError = evm_error.into();
        
        match tool_error {
            ToolError::Permanent(msg) => assert!(msg.contains("Transaction reverted")),
            _ => panic!("Expected Permanent error"),
        }
    }
}