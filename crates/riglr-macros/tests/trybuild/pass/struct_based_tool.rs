// Test struct-based tool implementation
use riglr_macros::tool;
use riglr_core::provider::ApplicationContext;
use riglr_core::{ToolError, JobResult};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[tool(description = "A tool implemented as a struct.")]
struct MyStructTool {
    config_value: String,
}

impl MyStructTool {
    pub async fn execute(&self, _context: &ApplicationContext) -> Result<String, ToolError> {
        Ok(format!("Executed with config: {}", self.config_value))
    }
}

fn main() {}