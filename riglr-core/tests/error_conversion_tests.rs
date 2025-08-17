//! Tests for error conversion and classification functionality

use riglr_core::error::ToolError;

#[test]
fn test_tool_error_retriable() {
    let retriable = ToolError::retriable_string("Network timeout");
    assert!(retriable.is_retriable());
    assert!(!retriable.is_rate_limited());
    assert_eq!(retriable.to_string(), "Operation can be retried");
}

#[test]
fn test_tool_error_rate_limited() {
    let rate_limited = ToolError::rate_limited_string("Too many requests");
    assert!(rate_limited.is_rate_limited());
    assert!(rate_limited.is_retriable()); // Rate limited errors are also retriable
    assert_eq!(rate_limited.to_string(), "Rate limited, retry after delay");
}

#[test]
fn test_tool_error_permanent() {
    let permanent = ToolError::permanent_string("Invalid configuration");
    assert!(!permanent.is_retriable());
    assert!(!permanent.is_rate_limited());
    assert_eq!(permanent.to_string(), "Permanent error, do not retry");
}

#[test]
fn test_tool_error_from_string() {
    let retriable = ToolError::retriable_string("Connection failed");
    assert_eq!(retriable.to_string(), "Operation can be retried");

    let permanent = ToolError::permanent_string("Invalid address format");
    assert_eq!(permanent.to_string(), "Permanent error, do not retry");
}
