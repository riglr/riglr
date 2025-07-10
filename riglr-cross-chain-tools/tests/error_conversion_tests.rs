use riglr_cross_chain_tools::error::CrossChainToolError;
use riglr_core::error::ToolError;

#[test]
fn test_rate_limit_error_conversion() {
    let err = CrossChainToolError::RateLimit("Bridge API rate limited".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::RateLimited { .. } => {},
        _ => panic!("Expected RateLimited variant"),
    }
}

#[test]
fn test_network_error_conversion_retriable() {
    let err = CrossChainToolError::NetworkError("connection timeout".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Retriable { .. } => {},
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_network_error_conversion_permanent() {
    let err = CrossChainToolError::NetworkError("invalid endpoint".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Permanent { .. } => {},
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_bridge_error_conversion_retriable() {
    let err = CrossChainToolError::BridgeError("Error 503: bridge maintenance".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Retriable { .. } => {},
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_bridge_error_conversion_permanent() {
    let err = CrossChainToolError::BridgeError("Error 400: invalid parameters".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Permanent { .. } => {},
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_unsupported_chain_conversion() {
    let err = CrossChainToolError::UnsupportedChain("UNKNOWN_CHAIN".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Permanent { .. } => {},
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_route_not_found_conversion() {
    let err = CrossChainToolError::RouteNotFound("No route from ETH to SOL".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Retriable { .. } => {},
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_insufficient_liquidity_conversion() {
    let err = CrossChainToolError::InsufficientLiquidity("Need 1000 USDC liquidity".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Retriable { .. } => {},
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_transaction_error_conversion() {
    let err = CrossChainToolError::TransactionError("Transaction failed".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Retriable { .. } => {},
        _ => panic!("Expected Retriable variant"),
    }
}