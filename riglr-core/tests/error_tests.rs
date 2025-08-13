//! Comprehensive tests for error module

use riglr_core::error::{CoreError, Result, ToolError};

#[test]
fn test_queue_error() {
    let error = CoreError::Queue("Queue is full".to_string());
    assert_eq!(error.to_string(), "Queue error: Queue is full");

    let error2 = CoreError::Queue("Connection lost".to_string());
    assert_eq!(error2.to_string(), "Queue error: Connection lost");
}

#[test]
fn test_job_execution_error() {
    let error = CoreError::JobExecution("Failed to execute job".to_string());
    assert_eq!(
        error.to_string(),
        "Job execution error: Failed to execute job"
    );

    let error2 = CoreError::JobExecution("Timeout occurred".to_string());
    assert_eq!(error2.to_string(), "Job execution error: Timeout occurred");
}

#[test]
fn test_generic_error() {
    let error = CoreError::Generic("Something went wrong".to_string());
    assert_eq!(error.to_string(), "Core error: Something went wrong");

    let error2 = CoreError::Generic("Unexpected state".to_string());
    assert_eq!(error2.to_string(), "Core error: Unexpected state");
}

#[test]
fn test_serialization_error() {
    let invalid_json = "{ invalid json";
    let result: std::result::Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());

    let core_error = CoreError::from(result.unwrap_err());
    assert!(core_error.to_string().contains("Serialization error"));
}

#[test]
fn test_result_type_alias() {
    fn returns_ok() -> Result<i32> {
        Ok(42)
    }

    fn returns_err() -> Result<i32> {
        Err(CoreError::Generic("test error".to_string()))
    }

    assert_eq!(returns_ok().unwrap(), 42);
    assert!(returns_err().is_err());
}

#[test]
fn test_error_debug_format() {
    let error = CoreError::Queue("Debug test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Queue"));
    assert!(debug_str.contains("Debug test"));
}

#[test]
fn test_error_chain() {
    fn operation_that_fails() -> Result<()> {
        Err(CoreError::JobExecution("Operation failed".to_string()))
    }

    fn wrapper_operation() -> Result<()> {
        operation_that_fails().map_err(|e| CoreError::Generic(format!("Wrapped error: {}", e)))
    }

    let result = wrapper_operation();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Wrapped error"));
}

#[test]
fn test_error_variants_equality() {
    let err1 = CoreError::Queue("test".to_string());
    let err2 = CoreError::Queue("test".to_string());

    // Test that errors with same content produce same string representation
    assert_eq!(err1.to_string(), err2.to_string());
}

#[test]
fn test_error_serialization_from_json_error() {
    // Create a JSON error by trying to parse invalid JSON
    let json_str = "not valid json";
    let parse_result: std::result::Result<serde_json::Value, _> = serde_json::from_str(json_str);

    assert!(parse_result.is_err());
    if let Err(e) = parse_result {
        let core_error = CoreError::from(e);
        assert!(core_error.to_string().contains("Serialization error"));
    }
}

#[test]
fn test_tool_error_retriable() {
    let error = ToolError::retriable("Connection timeout");
    assert_eq!(error.to_string(), "Operation can be retried: Connection timeout");
    assert!(error.is_retriable());

    let error2 = ToolError::retriable("Network issue");
    assert_eq!(error2.to_string(), "Operation can be retried: Network issue");
    assert!(error2.is_retriable());
}

#[test]
fn test_tool_error_permanent() {
    let error = ToolError::permanent("Invalid address");
    assert_eq!(error.to_string(), "Permanent error, do not retry: Invalid address");
    assert!(!error.is_retriable());

    let error2 = ToolError::permanent("Insufficient permissions");
    assert_eq!(
        error2.to_string(),
        "Permanent error, do not retry: Insufficient permissions"
    );
    assert!(!error2.is_retriable());
}

#[test]
fn test_tool_error_constructors() {
    // Test retriable constructor
    let retriable = ToolError::retriable("Temporary failure");
    assert!(matches!(retriable, ToolError::Retriable { .. }));
    assert!(retriable.is_retriable());

    // Test permanent constructor
    let permanent = ToolError::permanent("Configuration error");
    assert!(matches!(permanent, ToolError::Permanent { .. }));
    assert!(!permanent.is_retriable());
}

#[test]
fn test_tool_error_debug_format() {
    let error = ToolError::retriable("Debug test");
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Retriable"));
    assert!(debug_str.contains("Debug test"));

    let error2 = ToolError::permanent("Permanent debug");
    let debug_str2 = format!("{:?}", error2);
    assert!(debug_str2.contains("Permanent"));
    assert!(debug_str2.contains("Permanent debug"));
}

#[test]
fn test_tool_error_display() {
    let retriable = ToolError::rate_limited("Rate limited");
    assert_eq!(retriable.to_string(), "Operation can be retried: Rate limited");

    let permanent = ToolError::permanent("Invalid API key");
    assert_eq!(permanent.to_string(), "Permanent error, do not retry: Invalid API key");
}
