//! Basic tests for riglr-macros

use riglr_macros::tool;
use riglr_core::Tool;
use anyhow::Result;

// Test basic async function with tool macro
#[tool]
pub async fn basic_async_tool() -> Result<String> {
    Ok("success".to_string())
}

// Test async function with single parameter
#[tool]
pub async fn single_param_tool(value: i32) -> Result<i32> {
    Ok(value * 2)
}

// Test async function with multiple parameters
#[tool]
pub async fn multi_param_tool(a: i32, b: i32, c: String) -> Result<String> {
    Ok(format!("{} + {} = {}", a, b, c))
}

/// Doc comment used when no explicit description attribute is provided
#[tool]
pub async fn doc_only_tool() -> Result<&'static str> { Ok("ok") }

#[tool(description = "Explicit description.")]
pub async fn attr_tool() -> Result<&'static str> { Ok("ok") }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_tool() {
        let result = basic_async_tool().await.unwrap();
        assert_eq!(result, "success");
    }

    #[tokio::test]
    async fn test_single_param_tool() {
        let result = single_param_tool(5).await.unwrap();
        assert_eq!(result, 10);
    }

    #[tokio::test]
    async fn test_multi_param_tool() {
        let result = multi_param_tool(1, 2, "test".to_string()).await.unwrap();
        assert_eq!(result, "1 + 2 = test");
    }

    #[tokio::test]
    async fn test_description_priority() {
    // With namespacing, the generated tools are in modules named after the function
    // The tool struct is always named Tool inside the function's module
    let doc_tool = doc_only_tool::Tool::new();
    let attr_tool = attr_tool::Tool::new();

        assert_eq!(doc_tool.description(), "Doc comment used when no explicit description attribute is provided");
        assert_eq!(attr_tool.description(), "Explicit description.");
    }
}