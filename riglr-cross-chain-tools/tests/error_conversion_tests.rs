use riglr_core::error::ToolError;
use riglr_cross_chain_tools::error::CrossChainError;

#[test]
fn test_lifi_api_error_conversion() {
    let err = CrossChainError::LifiApiError("Bridge API rate limited".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Retriable { .. } => {}
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_quote_fetch_error_conversion() {
    let err = CrossChainError::QuoteFetchError("connection timeout".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Retriable { .. } => {}
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_invalid_route_error_conversion() {
    let err = CrossChainError::InvalidRoute("invalid endpoint".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Permanent { .. } => {}
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_bridge_execution_error_conversion() {
    let err = CrossChainError::BridgeExecutionError("Error 503: bridge maintenance".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Retriable { .. } => {}
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_unsupported_chain_pair_conversion() {
    let err = CrossChainError::UnsupportedChainPair {
        from_chain: "UNKNOWN_CHAIN".to_string(),
        to_chain: "ETH".to_string(),
    };
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Permanent { .. } => {}
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_insufficient_liquidity_conversion() {
    let err = CrossChainError::InsufficientLiquidity {
        amount: "1000 USDC".to_string(),
    };
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Retriable { .. } => {}
        _ => panic!("Expected Retriable variant"),
    }
}
