//! Comprehensive tests for riglr-macros

use riglr_macros::tool;
use anyhow::Result;
use serde::{Deserialize, Serialize};

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

// Test async function with optional parameters
#[tool]
pub async fn optional_param_tool(
    required: String,
    #[serde(default)]
    optional: Option<String>,
) -> Result<String> {
    match optional {
        Some(opt) => Ok(format!("{} - {}", required, opt)),
        None => Ok(required),
    }
}

// Test async function with complex types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexInput {
    pub field1: String,
    pub field2: i32,
    pub field3: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexOutput {
    pub result: String,
    pub count: usize,
}

#[tool]
pub async fn complex_types_tool(input: ComplexInput) -> Result<ComplexOutput> {
    Ok(ComplexOutput {
        result: format!("{}-{}", input.field1, input.field2),
        count: input.field3.len(),
    })
}

// Test async function with Result return type
#[tool]
pub async fn result_tool(should_succeed: bool) -> Result<String> {
    if should_succeed {
        Ok("success".to_string())
    } else {
        Err(anyhow::anyhow!("intentional failure"))
    }
}

// Test async function with generic Result
#[tool]
pub async fn generic_result_tool() -> anyhow::Result<i32> {
    Ok(42)
}

// Test non-async function (should work with the macro)
#[tool]
pub fn sync_tool(value: String) -> Result<String> {
    Ok(value.to_uppercase())
}

// Test function with lifetime parameters (if supported)
#[tool]
pub async fn string_ref_tool(value: String) -> Result<String> {
    Ok(format!("Received: {}", value))
}

// Test function with Vec parameters
#[tool]
pub async fn vec_tool(values: Vec<i32>) -> Result<i32> {
    Ok(values.iter().sum())
}

// Test function with HashMap parameters
use std::collections::HashMap;

#[tool]
pub async fn hashmap_tool(map: HashMap<String, String>) -> Result<usize> {
    Ok(map.len())
}

// Test function with nested optional
#[tool]
pub async fn nested_optional_tool(
    #[serde(default)]
    maybe_value: Option<Option<String>>,
) -> Result<String> {
    match maybe_value {
        Some(Some(val)) => Ok(format!("Value: {}", val)),
        Some(None) => Ok("Inner None".to_string()),
        None => Ok("Outer None".to_string()),
    }
}

// Test function with default values
fn default_timeout() -> u64 {
    30
}

#[tool]
pub async fn default_value_tool(
    #[serde(default = "default_timeout")]
    timeout: u64,
) -> Result<u64> {
    Ok(timeout)
}

// Test function with rename
#[tool]
pub async fn renamed_param_tool(
    #[serde(rename = "input_value")]
    value: String,
) -> Result<String> {
    Ok(value)
}

// Test function with skip_serializing_if
#[tool]
pub async fn skip_if_tool(
    required: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    optional: Option<String>,
) -> Result<String> {
    Ok(format!("{}{}", required, optional.unwrap_or_default()))
}

// Test function that returns a tuple
#[tool]
pub async fn tuple_tool() -> Result<(String, i32)> {
    Ok(("test".to_string(), 42))
}

// Test function with boolean parameter
#[tool]
pub async fn bool_tool(flag: bool) -> Result<String> {
    if flag {
        Ok("true".to_string())
    } else {
        Ok("false".to_string())
    }
}

// Test function with f64 parameter
#[tool]
pub async fn float_tool(value: f64) -> Result<f64> {
    Ok(value * 2.0)
}

// Test function with multiple optional parameters
#[tool]
pub async fn multi_optional_tool(
    #[serde(default)]
    opt1: Option<String>,
    #[serde(default)]
    opt2: Option<i32>,
    #[serde(default)]
    opt3: Option<bool>,
) -> Result<String> {
    let mut result = String::new();
    if let Some(s) = opt1 {
        result.push_str(&s);
    }
    if let Some(i) = opt2 {
        result.push_str(&i.to_string());
    }
    if let Some(b) = opt3 {
        result.push_str(&b.to_string());
    }
    Ok(result)
}

// Test struct with tool macro implementation
pub struct ToolStruct {
    pub name: String,
}

impl ToolStruct {
    pub async fn method_tool(&self, input: String) -> Result<String> {
        Ok(format!("{}: {}", self.name, input))
    }
}

// Tests to verify the macros compile correctly
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_async_tool() {
        let result = basic_async_tool().await.unwrap();
        assert_eq!(result, "success");
    }

    #[tokio::test]
    async fn test_single_param_tool() {
        let result = single_param_tool(21).await.unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_multi_param_tool() {
        let result = multi_param_tool(1, 2, "3".to_string()).await.unwrap();
        assert_eq!(result, "1 + 2 = 3");
    }

    #[tokio::test]
    async fn test_optional_param_tool() {
        let result1 = optional_param_tool("test".to_string(), None).await.unwrap();
        assert_eq!(result1, "test");
        
        let result2 = optional_param_tool("test".to_string(), Some("opt".to_string())).await.unwrap();
        assert_eq!(result2, "test - opt");
    }

    #[tokio::test]
    async fn test_complex_types_tool() {
        let input = ComplexInput {
            field1: "test".to_string(),
            field2: 42,
            field3: vec!["a".to_string(), "b".to_string()],
        };
        
        let result = complex_types_tool(input).await.unwrap();
        assert_eq!(result.result, "test-42");
        assert_eq!(result.count, 2);
    }

    #[tokio::test]
    async fn test_result_tool() {
        let success = result_tool(true).await;
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), "success");
        
        let failure = result_tool(false).await;
        assert!(failure.is_err());
    }

    #[tokio::test]
    async fn test_generic_result_tool() {
        let result = generic_result_tool().await.unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_sync_tool() {
        let result = sync_tool("hello".to_string()).unwrap();
        assert_eq!(result, "HELLO");
    }

    #[tokio::test]
    async fn test_string_ref_tool() {
        let result = string_ref_tool("test".to_string()).await.unwrap();
        assert_eq!(result, "Received: test");
    }

    #[tokio::test]
    async fn test_vec_tool() {
        let result = vec_tool(vec![1, 2, 3, 4, 5]).await.unwrap();
        assert_eq!(result, 15);
    }

    #[tokio::test]
    async fn test_hashmap_tool() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), "value1".to_string());
        map.insert("key2".to_string(), "value2".to_string());
        
        let result = hashmap_tool(map).await.unwrap();
        assert_eq!(result, 2);
    }

    #[tokio::test]
    async fn test_nested_optional_tool() {
        let result1 = nested_optional_tool(None).await.unwrap();
        assert_eq!(result1, "Outer None");
        
        let result2 = nested_optional_tool(Some(None)).await.unwrap();
        assert_eq!(result2, "Inner None");
        
        let result3 = nested_optional_tool(Some(Some("value".to_string()))).await.unwrap();
        assert_eq!(result3, "Value: value");
    }

    #[tokio::test]
    async fn test_default_value_tool() {
        let result = default_value_tool(60).await.unwrap();
        assert_eq!(result, 60);
    }

    #[tokio::test]
    async fn test_renamed_param_tool() {
        let result = renamed_param_tool("test".to_string()).await.unwrap();
        assert_eq!(result, "test");
    }

    #[tokio::test]
    async fn test_skip_if_tool() {
        let result1 = skip_if_tool("required".to_string(), None).await.unwrap();
        assert_eq!(result1, "required");
        
        let result2 = skip_if_tool("required".to_string(), Some("optional".to_string())).await.unwrap();
        assert_eq!(result2, "requiredoptional");
    }

    #[tokio::test]
    async fn test_tuple_tool() {
        let (s, i) = tuple_tool().await.unwrap();
        assert_eq!(s, "test");
        assert_eq!(i, 42);
    }

    #[tokio::test]
    async fn test_bool_tool() {
        let result1 = bool_tool(true).await.unwrap();
        assert_eq!(result1, "true");
        
        let result2 = bool_tool(false).await.unwrap();
        assert_eq!(result2, "false");
    }

    #[tokio::test]
    async fn test_float_tool() {
        let result = float_tool(21.0).await.unwrap();
        assert_eq!(result, 42.0);
    }

    #[tokio::test]
    async fn test_multi_optional_tool() {
        let result1 = multi_optional_tool(None, None, None).await.unwrap();
        assert_eq!(result1, "");
        
        let result2 = multi_optional_tool(
            Some("a".to_string()),
            Some(1),
            Some(true),
        ).await.unwrap();
        assert_eq!(result2, "a1true");
    }

    #[tokio::test]
    async fn test_tool_struct_method() {
        let tool = ToolStruct {
            name: "TestTool".to_string(),
        };
        
        let result = tool.method_tool("input".to_string()).await.unwrap();
        assert_eq!(result, "TestTool: input");
    }
}