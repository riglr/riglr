//! Comprehensive tests for error module

use riglr_core::error::{CoreError, Result};

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
    assert_eq!(error.to_string(), "Job execution error: Failed to execute job");
    
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
        operation_that_fails().map_err(|e| {
            CoreError::Generic(format!("Wrapped error: {}", e))
        })
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