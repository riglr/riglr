//! Additional compile-time failure tests for the #[tool] macro

use riglr_macros::tool;

// Test: Invalid #[tool] attribute syntax - missing value
#[tool(description)]
async fn invalid_attr_syntax() -> Result<(), String> {
    Ok(())
}

// Test: Invalid attribute value type - number instead of string
#[tool(description = 123)]
async fn invalid_attr_value_type() -> Result<(), String> {
    Ok(())
}

// Test: Unknown attribute key
#[tool(name = "test")]
async fn unknown_attr_key() -> Result<(), String> {
    Ok(())
}

// Test: Function with receiver (self parameter)
struct TestStruct;

impl TestStruct {
    #[tool]
    async fn method_with_receiver(&self) -> Result<(), String> {
        Ok(())
    }
}

// Test: Apply #[tool] to unsupported item types

// Module
#[tool]
mod test_module {
    pub fn test() {}
}

// Use statement
#[tool]
use std::collections::HashMap;

// Static item
#[tool]
static TEST_STATIC: &str = "test";

// Test: Apply #[derive(IntoToolError)] to struct (should fail)
use riglr_macros::IntoToolError;

#[derive(IntoToolError)]
struct TestStruct2 {
    field: String,
}
