// Test file covering all features of the macro

use riglr_macros::tool;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use schemars::JsonSchema;

// Test function with no parameters - covers empty schema generation
#[tool]
pub async fn empty_params() -> Result<String> {
    Ok("empty".to_string())
}

// Test function with documentation
/// This function has documentation
/// spanning multiple lines
#[tool]
pub async fn with_docs(input: String) -> Result<String> {
    Ok(input)
}

// Test function without documentation
#[tool]
pub async fn without_docs(input: String) -> Result<String> {
    Ok(input)
}

// Test non-async function
#[tool]
pub fn sync_fn(input: String) -> Result<String> {
    Ok(input)
}

// Test function with multiple parameters and docs
#[tool]
pub async fn multiple_params(
    /// First parameter documentation
    param1: String,
    /// Second parameter documentation
    param2: i32,
    // Parameter without doc comment
    param3: bool,
) -> Result<String> {
    Ok(format!("{}-{}-{}", param1, param2, param3))
}

// Test function with serde attributes
#[tool]
pub async fn with_serde_attrs(
    /// Required parameter
    required: String,
    /// Optional parameter with default
    #[serde(default)]
    optional: Option<String>,
    /// Parameter with custom name
    #[serde(rename = "custom_name")]
    renamed: String,
) -> Result<String> {
    Ok(format!("{:?}-{:?}-{}", required, optional, renamed))
}

// Test struct with tool macro
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[tool]
pub struct TestStruct {
    field1: String,
    field2: i32,
}

impl TestStruct {
    pub async fn execute(&self) -> Result<String> {
        Ok(format!("{}-{}", self.field1, self.field2))
    }
}

// Test struct with documentation
/// This struct has documentation
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[tool]
pub struct DocumentedStruct {
    value: String,
}

impl DocumentedStruct {
    pub async fn execute(&self) -> Result<String> {
        Ok(self.value.clone())
    }
}

// Test function that returns an error with "timeout" (retriable)
#[tool]
pub async fn timeout_error() -> Result<String> {
    Err(anyhow::anyhow!("Connection timeout"))
}

// Test function that returns an error with "connection" (retriable)
#[tool]
pub async fn connection_error() -> Result<String> {
    Err(anyhow::anyhow!("Failed to establish connection"))
}

// Test function that returns an error with "temporarily" (retriable)
#[tool]
pub async fn temporary_error() -> Result<String> {
    Err(anyhow::anyhow!("Service temporarily unavailable"))
}

// Test function that returns a non-retriable error
#[tool]
pub async fn permanent_error() -> Result<String> {
    Err(anyhow::anyhow!("Invalid input"))
}

// Test function with simple parameter (complex patterns moved to compile_fail tests)
#[tool]
pub async fn simple_pattern(
    /// Simple pattern
    simple: String,
) -> Result<String> {
    Ok(format!("{}", simple))
}

// Test visibility modifiers
#[tool]
pub(crate) async fn crate_visible(input: String) -> Result<String> {
    Ok(input)
}

#[tool]
pub(super) async fn super_visible(input: String) -> Result<String> {
    Ok(input)
}

#[tool]
async fn private_function(input: String) -> Result<String> {
    Ok(input)
}

// Test with very long names
#[tool]
pub async fn extremely_long_function_name_that_tests_the_limits_of_pascal_case_conversion_and_identifier_generation_in_the_macro() -> Result<String> {
    Ok("long".to_string())
}

// Test with special characters in doc comments
/// This function has special characters: `code`, **bold**, _italic_
/// - List item 1
/// - List item 2
/// # Header
/// > Quote
#[tool]
pub async fn special_docs(input: String) -> Result<String> {
    Ok(input)
}

// Test with nested generics
#[tool]
pub async fn nested_generics(
    data: Vec<Option<String>>
) -> Result<Vec<String>> {
    Ok(data.into_iter().flatten().collect())
}

// Note: Lifetime parameters and impl Trait tests moved to compile_fail tests
// as they are not supported by the macro

// Test with where clause
#[tool]
pub async fn where_clause<T>(value: T) -> Result<String>
where
    T: ToString + Send + Sync,
{
    Ok(value.to_string())
}

// Test empty struct
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[tool]
pub struct EmptyStruct;

impl EmptyStruct {
    pub async fn execute(&self) -> Result<String> {
        Ok("empty".to_string())
    }
}

// Test unit struct
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[tool]
pub struct UnitStruct();

impl UnitStruct {
    pub async fn execute(&self) -> Result<String> {
        Ok("unit".to_string())
    }
}

// Test struct with many fields
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[tool]
pub struct ManyFields {
    field1: String,
    field2: i32,
    field3: bool,
    field4: Option<String>,
    field5: Vec<i32>,
    field6: std::collections::HashMap<String, String>,
}

impl ManyFields {
    pub async fn execute(&self) -> Result<String> {
        Ok("many".to_string())
    }
}

fn main() {
    // Test that convenience functions are generated
    let _tool1 = empty_params_tool();
    let _tool2 = with_docs_tool();

    // Test that structs have as_tool method
    let struct_instance = TestStruct {
        field1: "test".to_string(),
        field2: 42,
    };
    let _tool3 = struct_instance.as_tool();

    // Test that tool structs implement Default
    let _tool4 = EmptyParamsTool::default();
    let _tool5 = WithDocsTool::new();
}