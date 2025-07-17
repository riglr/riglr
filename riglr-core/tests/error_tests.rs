//! Comprehensive tests for error module

use riglr_core::error::{CoreError, Result, ToolError};
use std::error::Error;

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
    assert_eq!(error.to_string(), "Operation can be retried");
    assert!(error.is_retriable());

    let error2 = ToolError::retriable("Network issue");
    assert_eq!(error2.to_string(), "Operation can be retried");
    assert!(error2.is_retriable());
}

#[test]
fn test_tool_error_permanent() {
    let error = ToolError::permanent("Invalid address");
    assert_eq!(error.to_string(), "Permanent error, do not retry");
    assert!(!error.is_retriable());

    let error2 = ToolError::permanent("Insufficient permissions");
    assert_eq!(
        error2.to_string(),
        "Permanent error, do not retry"
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
    assert_eq!(retriable.to_string(), "Rate limited, retry after delay");

    let permanent = ToolError::permanent("Invalid API key");
    assert_eq!(permanent.to_string(), "Permanent error, do not retry");
}

#[test]
fn test_core_error_redis() {
    #[cfg(feature = "redis")]
    {
        use redis::RedisError;
        let redis_err = RedisError::from((redis::ErrorKind::IoError, "Connection failed"));
        let core_error = CoreError::from(redis_err);
        assert!(core_error.to_string().contains("Redis error"));
    }
}

#[test]
fn test_worker_error_display() {
    use riglr_core::error::WorkerError;
    use std::io;
    
    // Test ToolNotFound
    let err = WorkerError::ToolNotFound { tool_name: "my_tool".to_string() };
    assert_eq!(err.to_string(), "Tool 'my_tool' not found in worker registry");
    
    // Test SemaphoreAcquisition
    let io_err = io::Error::new(io::ErrorKind::Other, "semaphore error");
    let err = WorkerError::SemaphoreAcquisition {
        tool_name: "test_tool".to_string(),
        source: Box::new(io_err),
    };
    assert!(err.to_string().contains("Failed to acquire semaphore for tool 'test_tool'"));
    
    // Test IdempotencyStore
    let io_err = io::Error::new(io::ErrorKind::Other, "store error");
    let err = WorkerError::IdempotencyStore {
        source: Box::new(io_err),
    };
    assert!(err.to_string().contains("Idempotency store operation failed"));
    
    // Test JobSerialization
    let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
    let err = WorkerError::JobSerialization {
        source: json_err,
    };
    assert!(err.to_string().contains("Job serialization error"));
    
    // Test ExecutionTimeout
    let err = WorkerError::ExecutionTimeout {
        timeout: std::time::Duration::from_secs(30),
    };
    assert!(err.to_string().contains("Tool execution timed out after"));
    
    // Test Internal
    let err = WorkerError::Internal {
        message: "internal failure".to_string(),
    };
    assert_eq!(err.to_string(), "Internal worker error: internal failure");
}

#[test]
fn test_tool_error_with_source_variants() {
    use std::io;
    
    // Test retriable_with_source
    let io_err = io::Error::new(io::ErrorKind::TimedOut, "connection timeout");
    let tool_err = ToolError::retriable_with_source(io_err, "Failed to connect");
    assert!(tool_err.is_retriable());
    assert!(!tool_err.is_rate_limited());
    assert!(tool_err.source().is_some());
    
    // Test permanent_with_source
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let tool_err = ToolError::permanent_with_source(io_err, "Insufficient permissions");
    assert!(!tool_err.is_retriable());
    assert!(!tool_err.is_rate_limited());
    assert!(tool_err.source().is_some());
    
    // Test rate_limited_with_source with retry_after
    let io_err = io::Error::new(io::ErrorKind::Other, "rate limit exceeded");
    let retry_duration = Some(std::time::Duration::from_secs(120));
    let tool_err = ToolError::rate_limited_with_source(io_err, "API rate limit", retry_duration);
    assert!(tool_err.is_retriable());
    assert!(tool_err.is_rate_limited());
    assert_eq!(tool_err.retry_after(), retry_duration);
    assert!(tool_err.source().is_some());
    
    // Test rate_limited_with_source without retry_after
    let io_err = io::Error::new(io::ErrorKind::Other, "rate limit");
    let tool_err = ToolError::rate_limited_with_source(io_err, "Rate limited", None);
    assert!(tool_err.retry_after().is_none());
    
    // Test invalid_input_with_source
    let io_err = io::Error::new(io::ErrorKind::InvalidInput, "bad format");
    let tool_err = ToolError::invalid_input_with_source(io_err, "Invalid address format");
    assert!(!tool_err.is_retriable());
    assert!(!tool_err.is_rate_limited());
    assert!(tool_err.source().is_some());
    assert!(tool_err.to_string().contains("Invalid input provided"));
}

#[test]
fn test_tool_error_from_msg_variants() {
    // Test retriable_from_msg
    let err = ToolError::retriable_from_msg("Network error");
    assert!(err.is_retriable());
    assert!(err.source().is_some()); // SimpleError is used as source
    
    // Test permanent_from_msg
    let err = ToolError::permanent_from_msg("Config error");
    assert!(!err.is_retriable());
    assert!(err.source().is_some());
    
    // Test rate_limited_from_msg
    let err = ToolError::rate_limited_from_msg("Too many requests");
    assert!(err.is_retriable());
    assert!(err.is_rate_limited());
    assert!(err.retry_after().is_none());
    assert!(err.source().is_some());
    
    // Test invalid_input_from_msg
    let err = ToolError::invalid_input_from_msg("Bad input");
    assert!(!err.is_retriable());
    assert!(err.to_string().contains("Invalid input provided"));
    assert!(err.source().is_some());
}

#[test]
fn test_tool_error_from_conversions() {
    // Test From<anyhow::Error>
    let anyhow_err = anyhow::anyhow!("Anyhow error");
    let tool_err: ToolError = anyhow_err.into();
    assert!(!tool_err.is_retriable());
    
    // Test From<Box<dyn Error>>
    let boxed_err: Box<dyn std::error::Error + Send + Sync> = 
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, "boxed error"));
    let tool_err: ToolError = boxed_err.into();
    assert!(!tool_err.is_retriable());
    
    // Test From<String>
    let string_err = "String error".to_string();
    let tool_err: ToolError = string_err.into();
    assert!(!tool_err.is_retriable());
    
    // Test From<&str>
    let str_err = "Static str error";
    let tool_err: ToolError = str_err.into();
    assert!(!tool_err.is_retriable());
}

#[test]
fn test_tool_error_signer_context() {
    use riglr_core::signer::SignerError;
    
    let signer_err = SignerError::NoSignerContext;
    let tool_err: ToolError = signer_err.into();
    assert_eq!(tool_err.to_string(), "Signer context error");
    assert!(!tool_err.is_retriable());
}

#[test]
fn test_deprecated_constructors() {
    // Test deprecated methods still work for backward compatibility
    #[allow(deprecated)]
    {
        let err = ToolError::retriable("Old style");
        assert!(err.is_retriable());
        
        let err = ToolError::rate_limited("Old rate limit");
        assert!(err.is_rate_limited());
        
        let err = ToolError::permanent("Old permanent");
        assert!(!err.is_retriable());
        
        let err = ToolError::invalid_input("Old invalid");
        assert!(!err.is_retriable());
    }
}

#[test]
fn test_core_error_from_str() {
    let err: CoreError = "test error".into();
    assert_eq!(err.to_string(), "Core error: test error");
}
