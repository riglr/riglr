#[cfg(test)]
mod tests {
    use riglr_hyperliquid_tools::error::HyperliquidToolError;
    use riglr_core::ToolError;

    #[test]
    fn test_network_error_conversion() {
        let hl_error = HyperliquidToolError::Network("Connection timeout".to_string());
        let tool_error: ToolError = hl_error.into();
        
        match tool_error {
            ToolError::Retriable(msg) => assert!(msg.contains("Connection timeout")),
            _ => panic!("Expected Retriable error"),
        }
    }

    #[test]
    fn test_rate_limit_conversion() {
        let hl_error = HyperliquidToolError::RateLimit("Too many requests".to_string());
        let tool_error: ToolError = hl_error.into();
        
        match tool_error {
            ToolError::Retriable(msg) => assert!(msg.contains("Too many requests")),
            _ => panic!("Expected Retriable error"),
        }
    }

    #[test]
    fn test_insufficient_balance_conversion() {
        let hl_error = HyperliquidToolError::InsufficientBalance("Not enough margin".to_string());
        let tool_error: ToolError = hl_error.into();
        
        match tool_error {
            ToolError::Retriable(msg) => assert!(msg.contains("Not enough margin")),
            _ => panic!("Expected Retriable error"),
        }
    }

    #[test]
    fn test_auth_error_conversion() {
        let hl_error = HyperliquidToolError::Auth("Invalid API key".to_string());
        let tool_error: ToolError = hl_error.into();
        
        match tool_error {
            ToolError::Permanent(msg) => assert!(msg.contains("Invalid API key")),
            _ => panic!("Expected Permanent error"),
        }
    }

    #[test]
    fn test_invalid_symbol_conversion() {
        let hl_error = HyperliquidToolError::InvalidSymbol("Unknown symbol".to_string());
        let tool_error: ToolError = hl_error.into();
        
        match tool_error {
            ToolError::Permanent(msg) => assert!(msg.contains("Unknown symbol")),
            _ => panic!("Expected Permanent error"),
        }
    }

    #[test]
    fn test_order_failed_conversion() {
        let hl_error = HyperliquidToolError::OrderFailed("Order rejected".to_string());
        let tool_error: ToolError = hl_error.into();
        
        match tool_error {
            ToolError::Permanent(msg) => assert!(msg.contains("Order rejected")),
            _ => panic!("Expected Permanent error"),
        }
    }
}