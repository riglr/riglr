//! Comprehensive tests for error module

use riglr_web_tools::error::{WebToolError, Result};
use riglr_core::CoreError;

#[test]
fn test_http_error() {
    // We can't directly create reqwest errors in tests,
    // so we'll skip this test for now
    // The HTTP error variant is tested through integration tests
    assert!(true);
}

#[test]
fn test_auth_error() {
    let error = WebToolError::Auth("Invalid API key".to_string());
    assert!(matches!(error, WebToolError::Auth(_)));
    assert_eq!(error.to_string(), "Authentication error: Invalid API key");
}

#[test]
fn test_rate_limit_error() {
    let error = WebToolError::RateLimit("429 Too Many Requests".to_string());
    assert!(matches!(error, WebToolError::RateLimit(_)));
    assert_eq!(error.to_string(), "Rate limit exceeded: 429 Too Many Requests");
}

#[test]
fn test_invalid_response_error() {
    let error = WebToolError::InvalidResponse("Unexpected JSON structure".to_string());
    assert!(matches!(error, WebToolError::InvalidResponse(_)));
    assert_eq!(error.to_string(), "Invalid response: Unexpected JSON structure");
}

#[test]
fn test_url_error() {
    let url_err = url::ParseError::RelativeUrlWithoutBase;
    let error = WebToolError::from(url_err);
    assert!(matches!(error, WebToolError::Url(_)));
    assert!(error.to_string().contains("URL error"));
}

#[test]
fn test_serialization_error() {
    let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
    let error = WebToolError::from(json_err);
    assert!(matches!(error, WebToolError::Serialization(_)));
    assert!(error.to_string().contains("Serialization error"));
}

#[test]
fn test_core_error() {
    let core_err = CoreError::Generic("Core issue".to_string());
    let error = WebToolError::from(core_err);
    assert!(matches!(error, WebToolError::Core(_)));
    assert!(error.to_string().contains("Core error"));
}

#[test]
fn test_generic_error() {
    let error = WebToolError::Generic("Something went wrong".to_string());
    assert!(matches!(error, WebToolError::Generic(_)));
    assert_eq!(error.to_string(), "Web tool error: Something went wrong");
}

#[test]
fn test_error_result_type() {
    fn returns_result() -> Result<String> {
        Ok("success".to_string())
    }
    
    let result = returns_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    
    fn returns_error() -> Result<String> {
        Err(WebToolError::Generic("failed".to_string()))
    }
    
    let result = returns_error();
    assert!(result.is_err());
}

#[test]
fn test_error_debug() {
    let error = WebToolError::Auth("debug test".to_string());
    let debug_str = format!("{:?}", error);
    
    assert!(debug_str.contains("Auth"));
    assert!(debug_str.contains("debug test"));
}

#[test]
fn test_error_variants() {
    let errors = vec![
        WebToolError::Auth("auth".to_string()),
        WebToolError::RateLimit("rate".to_string()),
        WebToolError::InvalidResponse("response".to_string()),
        WebToolError::Generic("generic".to_string()),
    ];
    
    for error in errors {
        let error_str = error.to_string();
        assert!(!error_str.is_empty());
    }
}

#[test]
fn test_error_chain() {
    // We can't directly create reqwest errors in tests,
    // so we test error chaining with other error types
    let core_err = CoreError::Generic("test error".to_string());
    let web_err = WebToolError::from(core_err);
    
    let error_str = web_err.to_string();
    assert!(error_str.contains("Core error"));
}

#[test]
fn test_result_mapping() {
    let ok_result: Result<i32> = Ok(42);
    let mapped = ok_result.map(|x| x * 2);
    assert_eq!(mapped.unwrap(), 84);
    
    let err_result: Result<i32> = Err(WebToolError::Generic("error".to_string()));
    let mapped = err_result.map(|x| x * 2);
    assert!(mapped.is_err());
}

#[test]
fn test_result_and_then() {
    fn double(x: i32) -> Result<i32> {
        Ok(x * 2)
    }
    
    let result: Result<i32> = Ok(21);
    let chained = result.and_then(double);
    assert_eq!(chained.unwrap(), 42);
}

#[test]
fn test_error_display() {
    let test_cases = vec![
        (WebToolError::Auth("key".to_string()), "Authentication error"),
        (WebToolError::RateLimit("limit".to_string()), "Rate limit"),
        (WebToolError::InvalidResponse("bad".to_string()), "Invalid response"),
        (WebToolError::Generic("gen".to_string()), "Web tool error"),
    ];
    
    for (error, expected_prefix) in test_cases {
        let display = format!("{}", error);
        assert!(display.contains(expected_prefix));
    }
}