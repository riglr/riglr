//! Test that custom errors implementing Into<ToolError> work correctly with the #[tool] macro.

use riglr_core::{provider::ApplicationContext, ToolError};
use riglr_macros::tool;
use thiserror::Error;

/// Custom error enum with From implementation for ToolError
#[derive(Error, Debug)]
enum CustomError {
    #[error("Network issue: {0}")]
    NetworkIssue(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Rate limit exceeded")]
    RateLimited,
}

// Implement From<CustomError> for ToolError
impl From<CustomError> for ToolError {
    fn from(err: CustomError) -> Self {
        match err {
            CustomError::NetworkIssue(_) => ToolError::retriable_string(err.to_string()),
            CustomError::InvalidData(_) => ToolError::permanent_string(err.to_string()),
            CustomError::RateLimited => ToolError::rate_limited_string(err.to_string()),
        }
    }
}

/// A tool that uses a custom error type with Into<ToolError> implementation
#[tool]
async fn process_with_custom_error(
    data: String,
    mode: String,
    context: &ApplicationContext,
) -> Result<String, CustomError> {
    // Simulate different error conditions based on mode
    match mode.as_str() {
        "network_fail" => Err(CustomError::NetworkIssue("Connection timeout".to_string())),
        "invalid" => Err(CustomError::InvalidData("Malformed input".to_string())),
        "rate_limit" => Err(CustomError::RateLimited),
        _ => Ok(format!("Processed: {}", data)),
    }
}

// Another custom error that also implements Into<ToolError>
#[derive(Error, Debug)]
struct SimpleCustomError {
    message: String,
}

impl std::fmt::Display for SimpleCustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SimpleCustomError: {}", self.message)
    }
}

impl From<SimpleCustomError> for ToolError {
    fn from(err: SimpleCustomError) -> Self {
        ToolError::permanent_string(err.to_string())
    }
}

/// Another tool using a different custom error type
#[tool]
async fn simple_tool_with_custom_error(
    input: i32,
    context: &ApplicationContext,
) -> Result<i32, SimpleCustomError> {
    if input < 0 {
        Err(SimpleCustomError {
            message: "Input must be non-negative".to_string(),
        })
    } else {
        Ok(input * 2)
    }
}

fn main() {
    // This file should compile successfully because both CustomError and SimpleCustomError
    // implement Into<ToolError> (via From implementations)
}
