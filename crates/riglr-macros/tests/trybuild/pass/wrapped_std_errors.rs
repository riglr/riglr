//! Test that demonstrates the correct pattern of wrapping standard library errors
//! in custom error enums using the IntoToolError derive macro.

use riglr_core::provider::ApplicationContext;
use riglr_macros::{tool, IntoToolError};
use thiserror::Error;

/// Custom error enum that wraps standard library errors using IntoToolError derive
#[derive(Error, Debug, IntoToolError)]
enum FileOperationError {
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Invalid file format")]
    InvalidFormat,
    
    #[tool_error(retriable)]
    #[error("Network timeout during file upload")]
    NetworkTimeout,
}

/// A tool that uses wrapped standard library errors - should compile successfully
#[tool]
async fn read_and_parse_file(
    file_path: String,
    context: &ApplicationContext,
) -> Result<serde_json::Value, FileOperationError> {
    // Simulate file operations that could produce std::io::Error
    let content = std::fs::read_to_string(&file_path)?; // std::io::Error gets wrapped
    
    // Simulate JSON parsing that could produce serde_json::Error
    let parsed: serde_json::Value = serde_json::from_str(&content)?; // serde_json::Error gets wrapped
    
    // Manual error creation
    if !parsed.is_object() {
        return Err(FileOperationError::InvalidFormat);
    }
    
    Ok(parsed)
}

/// Another custom error demonstrating reqwest error wrapping
#[derive(Error, Debug, IntoToolError)]
enum HttpClientError {
    #[error("HTTP request failed: {0}")]
    #[tool_error(retriable)]  // Override default classification
    RequestError(String), // Wrapped reqwest::Error as String for simplicity
    
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    
    #[error("Authentication failed")]
    Unauthorized,
}

/// A tool that demonstrates HTTP error handling without direct reqwest::Error usage
#[tool]
async fn fetch_data(
    url: String,
    context: &ApplicationContext,
) -> Result<String, HttpClientError> {
    // Validate URL format
    if !url.starts_with("http") {
        return Err(HttpClientError::InvalidUrl(url));
    }
    
    // Simulate network request (in real code, you'd catch and wrap reqwest::Error)
    if url.contains("unauthorized") {
        return Err(HttpClientError::Unauthorized);
    }
    
    if url.contains("timeout") {
        return Err(HttpClientError::RequestError("Connection timeout".to_string()));
    }
    
    Ok("Mock response data".to_string())
}

/// Example showing how to manually wrap std::io::Error in business logic
fn read_config_file(path: &str) -> Result<String, FileOperationError> {
    // The ? operator automatically converts std::io::Error to FileOperationError
    // due to the #[from] attribute in the enum definition
    std::fs::read_to_string(path).map_err(FileOperationError::from)
}

/// Tool that uses the business logic function
#[tool]
async fn load_configuration(
    config_path: String,
    context: &ApplicationContext,
) -> Result<String, FileOperationError> {
    let config_content = read_config_file(&config_path)?;
    Ok(format!("Loaded config: {}", config_content))
}

fn main() {
    // This file should compile successfully because:
    // 1. FileOperationError wraps std::io::Error and serde_json::Error using #[from]
    // 2. HttpClientError demonstrates proper error wrapping patterns
    // 3. All custom error types derive IntoToolError for automatic ToolError conversion
    // 4. No direct usage of std::io::Error or reqwest::Error in tool return types
    println!("All tools using wrapped standard library errors compiled successfully!");
}