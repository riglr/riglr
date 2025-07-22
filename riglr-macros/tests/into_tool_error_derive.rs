//! Comprehensive tests for the #[derive(IntoToolError)] macro
//! 
//! This test suite covers:
//! - Automatic classification based on variant names
//! - Explicit classification via attributes
//! - All variant field types (unit, tuple, struct)
//! - Override scenarios

use riglr_macros::IntoToolError;
use riglr_core::ToolError;
use thiserror::Error;

/// Test automatic classification based on naming conventions
#[derive(Error, Debug, IntoToolError)]
enum AutomaticError {
    // Should be classified as Retriable
    #[error("RPC connection failed")]
    RpcConnectionFailed,
    
    #[error("Network timeout")]
    NetworkTimeout,
    
    #[error("Connection lost")]
    ConnectionLost,
    
    #[error("Request timeout")]
    RequestTimeout,
    
    #[error("Too many requests")]
    TooManyRequests,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("API error")]
    ApiError,
    
    #[error("HTTP error")]
    HttpError,
    
    // Should be classified as Permanent
    #[error("Invalid address")]
    InvalidAddress,
    
    #[error("Parse error")]
    ParseError,
    
    #[error("Serialization failed")]
    SerializationFailed,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Unauthorized access")]
    UnauthorizedAccess,
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    // Should default to Permanent (unmatched)
    #[error("Unknown error")]
    UnknownError,
    
    #[error("Custom failure")]
    CustomFailure,
}

/// Test explicit classification via attributes
#[derive(Error, Debug, IntoToolError)]
enum ExplicitError {
    #[tool_error(retriable)]
    #[error("Custom error that should be retriable")]
    CustomRetriable,
    
    #[tool_error(permanent)]
    #[error("Network error that should be permanent")]
    NetworkPermanent, // Override default network = retriable
    
    #[tool_error(rate_limited)]
    #[error("API quota exceeded")]
    ApiQuotaExceeded,
    
    #[tool_error(permanent)]
    #[error("Invalid state: {state}")]
    InvalidState { state: String },
    
    #[tool_error(retriable)]
    #[error("Temporary failure with code: {0}")]
    TemporaryFailure(i32),
}

/// Test all variant field types
#[derive(Error, Debug, IntoToolError)]
enum FieldTypesError {
    // Unit variant
    #[error("Simple unit error")]
    UnitVariant,
    
    // Tuple variant
    #[error("Tuple error: {0}")]
    TupleVariant(String),
    
    #[error("Multiple tuple error: {0}, {1}")]
    MultipleTupleVariant(String, i32),
    
    // Struct variant
    #[error("Struct error: {message}")]
    StructVariant { message: String },
    
    #[error("Complex struct error: {msg}, code: {code}")]
    ComplexStructVariant { msg: String, code: i32 },
    
    // Mixed with explicit classification
    #[tool_error(retriable)]
    #[error("Retriable tuple: {0}")]
    RetriableTuple(String),
    
    #[tool_error(permanent)]
    #[error("Permanent struct: {reason}")]
    PermanentStruct { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 2.2.1: Test Automatic Classification

    #[test]
    fn test_automatic_retriable_classification() {
        let retriable_errors = vec![
            AutomaticError::RpcConnectionFailed,
            AutomaticError::NetworkTimeout,
            AutomaticError::ConnectionLost,
            AutomaticError::RequestTimeout,
            AutomaticError::TooManyRequests,
            AutomaticError::RateLimitExceeded,
            AutomaticError::ApiError,
            AutomaticError::HttpError,
        ];
        
        for error in retriable_errors {
            let tool_error: ToolError = error.into();
            match tool_error {
                ToolError::Retriable { context, .. } => {
                    // Verify the error message is preserved
                    assert!(!context.is_empty());
                }
                _ => panic!("Expected Retriable error, got {:?}", tool_error),
            }
        }
    }

    #[test]
    fn test_automatic_permanent_classification() {
        let permanent_errors = vec![
            AutomaticError::InvalidAddress,
            AutomaticError::ParseError,
            AutomaticError::SerializationFailed,
            AutomaticError::UserNotFound,
            AutomaticError::UnauthorizedAccess,
            AutomaticError::InsufficientBalance,
            AutomaticError::InsufficientFunds,
            AutomaticError::UnknownError,
            AutomaticError::CustomFailure,
        ];
        
        for error in permanent_errors {
            let tool_error: ToolError = error.into();
            match tool_error {
                ToolError::Permanent { context, .. } => {
                    // Verify the error message is preserved
                    assert!(!context.is_empty());
                }
                _ => panic!("Expected Permanent error, got {:?}", tool_error),
            }
        }
    }

    // Test 2.2.2: Test Explicit Classification

    #[test]
    fn test_explicit_retriable_classification() {
        let error = ExplicitError::CustomRetriable;
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::Retriable { context, .. } => {
                assert!(context.contains("Custom error that should be retriable"));
            }
            _ => panic!("Expected Retriable error, got {:?}", tool_error),
        }
    }

    #[test]
    fn test_explicit_permanent_override() {
        // Test that explicit permanent attribute overrides automatic retriable classification
        let error = ExplicitError::NetworkPermanent;
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Network error that should be permanent"));
            }
            _ => panic!("Expected Permanent error (overriding network default), got {:?}", tool_error),
        }
    }

    #[test]
    fn test_explicit_rate_limited_classification() {
        let error = ExplicitError::ApiQuotaExceeded;
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::RateLimited { context, .. } => {
                assert!(context.contains("API quota exceeded"));
            }
            _ => panic!("Expected RateLimited error, got {:?}", tool_error),
        }
    }

    #[test]
    fn test_explicit_classification_with_fields() {
        // Test struct variant with explicit permanent
        let error = ExplicitError::InvalidState { 
            state: "corrupted".to_string() 
        };
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Invalid state: corrupted"));
            }
            _ => panic!("Expected Permanent error, got {:?}", tool_error),
        }
        
        // Test tuple variant with explicit retriable
        let error = ExplicitError::TemporaryFailure(500);
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::Retriable { context, .. } => {
                assert!(context.contains("Temporary failure with code: 500"));
            }
            _ => panic!("Expected Retriable error, got {:?}", tool_error),
        }
    }

    // Test 2.2.3: Test All Variant Field Types

    #[test]
    fn test_unit_variant_conversion() {
        let error = FieldTypesError::UnitVariant;
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Simple unit error"));
            }
            _ => panic!("Expected Permanent error, got {:?}", tool_error),
        }
    }

    #[test]
    fn test_tuple_variant_conversion() {
        let error = FieldTypesError::TupleVariant("test message".to_string());
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Tuple error: test message"));
            }
            _ => panic!("Expected Permanent error, got {:?}", tool_error),
        }
        
        let error = FieldTypesError::MultipleTupleVariant("error".to_string(), 404);
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Multiple tuple error: error, 404"));
            }
            _ => panic!("Expected Permanent error, got {:?}", tool_error),
        }
    }

    #[test]
    fn test_struct_variant_conversion() {
        let error = FieldTypesError::StructVariant { 
            message: "test struct".to_string() 
        };
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Struct error: test struct"));
            }
            _ => panic!("Expected Permanent error, got {:?}", tool_error),
        }
        
        let error = FieldTypesError::ComplexStructVariant { 
            msg: "complex".to_string(),
            code: 500,
        };
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Complex struct error: complex, code: 500"));
            }
            _ => panic!("Expected Permanent error, got {:?}", tool_error),
        }
    }

    #[test]
    fn test_mixed_field_types_with_explicit_classification() {
        // Test tuple with explicit retriable
        let error = FieldTypesError::RetriableTuple("should retry".to_string());
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::Retriable { context, .. } => {
                assert!(context.contains("Retriable tuple: should retry"));
            }
            _ => panic!("Expected Retriable error, got {:?}", tool_error),
        }
        
        // Test struct with explicit permanent
        let error = FieldTypesError::PermanentStruct { 
            reason: "bad data".to_string() 
        };
        let tool_error: ToolError = error.into();
        
        match tool_error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Permanent struct: bad data"));
            }
            _ => panic!("Expected Permanent error, got {:?}", tool_error),
        }
    }

    #[test]
    fn test_error_message_preservation() {
        // Test that the original error messages are preserved in the ToolError context
        let test_cases = vec![
            (AutomaticError::NetworkTimeout, "Network timeout"),
            (AutomaticError::InvalidAddress, "Invalid address"),
        ];
        
        for (error, expected_message) in test_cases {
            let tool_error: ToolError = error.into();
            let context = match tool_error {
                ToolError::Retriable { context, .. } => context,
                ToolError::Permanent { context, .. } => context,
                ToolError::RateLimited { context, .. } => context,
                ToolError::InvalidInput { context, .. } => context,
                ToolError::SignerContext(_) => panic!("Unexpected SignerContext error"),
            };
            
            assert!(context.contains(expected_message), 
                   "Expected '{}' to be found in '{}'", expected_message, context);
        }
    }
}
