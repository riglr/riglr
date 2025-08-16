use riglr_core::error::ToolError;
use riglr_hyperliquid_tools::error::HyperliquidToolError;

#[test]
fn test_rate_limit_error_conversion() {
    let err = HyperliquidToolError::RateLimit("API rate limited".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::RateLimited { .. } => {}
        _ => panic!("Expected RateLimited variant"),
    }
}

#[test]
fn test_network_error_conversion_retriable() {
    let err = HyperliquidToolError::NetworkError("connection timeout".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Retriable { .. } => {}
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_network_error_conversion_permanent() {
    let err = HyperliquidToolError::NetworkError("invalid host".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Permanent { .. } => {}
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_auth_error_conversion() {
    let err = HyperliquidToolError::AuthError("Invalid API key".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Permanent { .. } => {}
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_invalid_symbol_conversion() {
    let err = HyperliquidToolError::InvalidSymbol("INVALID".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Permanent { .. } => {}
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_insufficient_balance_conversion() {
    let err = HyperliquidToolError::InsufficientBalance("Need 100 USD".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Permanent { .. } => {}
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_order_error_conversion() {
    let err = HyperliquidToolError::OrderError("Order failed".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Retriable { .. } => {}
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_api_error_rate_limit_conversion() {
    let err = HyperliquidToolError::ApiError("Error 429: rate limit exceeded".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::RateLimited { .. } => {}
        _ => panic!("Expected RateLimited variant"),
    }
}

#[test]
fn test_api_error_service_unavailable_conversion() {
    let err = HyperliquidToolError::ApiError("Error 503: service unavailable".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Retriable { .. } => {}
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_api_error_permanent_conversion() {
    let err = HyperliquidToolError::ApiError("Error 400: bad request".to_string());
    let tool_err: ToolError = err.into();

    match tool_err {
        ToolError::Permanent { .. } => {}
        _ => panic!("Expected Permanent variant"),
    }
}
