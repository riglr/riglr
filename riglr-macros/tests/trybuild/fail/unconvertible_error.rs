//! Test that custom errors NOT implementing Into<ToolError> fail to compile with a clear error message.

use riglr_core::provider::ApplicationContext;
use riglr_macros::tool;

/// Custom error that does NOT implement Into<ToolError>
#[derive(Debug)]
struct UnconvertibleError {
    message: String,
}

impl std::fmt::Display for UnconvertibleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UnconvertibleError: {}", self.message)
    }
}

impl std::error::Error for UnconvertibleError {}

/// This tool should fail to compile because UnconvertibleError doesn't implement Into<ToolError>
#[tool]
async fn tool_with_unconvertible_error(
    data: String,
    context: &ApplicationContext,
) -> Result<String, UnconvertibleError> {
    if data.is_empty() {
        Err(UnconvertibleError {
            message: "Data cannot be empty".to_string(),
        })
    } else {
        Ok(format!("Processed: {}", data))
    }
}

// Another unconvertible error type (not a standard library type either)
struct CustomBusinessError {
    code: i32,
    details: String,
}

impl std::fmt::Display for CustomBusinessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BusinessError({}): {}", self.code, self.details)
    }
}

impl std::fmt::Debug for CustomBusinessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CustomBusinessError {{ code: {}, details: {} }}",
            self.code, self.details
        )
    }
}

/// This should also fail to compile
#[tool]
async fn another_failing_tool(
    operation_id: u64,
    context: &ApplicationContext,
) -> Result<bool, CustomBusinessError> {
    if operation_id == 0 {
        Err(CustomBusinessError {
            code: 400,
            details: "Invalid operation ID".to_string(),
        })
    } else {
        Ok(true)
    }
}

fn main() {
    // This file should FAIL to compile with a clear error message about
    // UnconvertibleError and CustomBusinessError not implementing Into<ToolError>
}
