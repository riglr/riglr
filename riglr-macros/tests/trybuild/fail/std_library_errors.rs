//! Test that std::io::Error and reqwest::Error now fail to compile with the #[tool] macro
//! This demonstrates the stricter error handling enforcement.

use riglr_core::provider::ApplicationContext;
use riglr_macros::tool;

/// This tool should FAIL to compile because std::io::Error doesn't implement Into<ToolError>
#[tool]
async fn read_file_direct_io_error(
    file_path: String,
    context: &ApplicationContext,
) -> Result<String, std::io::Error> {
    // This function returns std::io::Error directly, which is no longer supported
    std::fs::read_to_string(&file_path)
}

/// This tool should also FAIL to compile for the same reason
#[tool]
async fn create_directory(
    dir_path: String,
    context: &ApplicationContext,
) -> Result<(), std::io::Error> {
    // Another example of direct std::io::Error usage
    std::fs::create_dir_all(&dir_path)
}

// Simulate a reqwest::Error-like structure since we don't want to depend on reqwest in tests
#[derive(Debug)]
struct MockRequestError {
    message: String,
}

impl std::fmt::Display for MockRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request error: {}", self.message)
    }
}

impl std::error::Error for MockRequestError {}

/// This tool should FAIL to compile because MockRequestError doesn't implement Into<ToolError>
#[tool]
async fn fetch_url_mock_error(
    url: String,
    context: &ApplicationContext,
) -> Result<String, MockRequestError> {
    if url.is_empty() {
        Err(MockRequestError {
            message: "Empty URL".to_string(),
        })
    } else {
        Ok("Mock response".to_string())
    }
}

/// Another example with serde_json::Error
#[tool]
async fn parse_json_direct_error(
    json_string: String,
    context: &ApplicationContext,
) -> Result<serde_json::Value, serde_json::Error> {
    // This should FAIL because serde_json::Error doesn't implement Into<ToolError>
    serde_json::from_str(&json_string)
}

fn main() {
    // This file should FAIL to compile with clear error messages indicating that:
    // - std::io::Error doesn't implement Into<ToolError>
    // - MockRequestError doesn't implement Into<ToolError>
    // - serde_json::Error doesn't implement Into<ToolError>
    //
    // The error messages should guide users to wrap these errors in custom types
    // that implement Into<ToolError> via the IntoToolError derive macro.
}
