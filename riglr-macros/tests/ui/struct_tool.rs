use riglr_macros::tool;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use anyhow::Result;

/// A calculator tool that performs operations
#[derive(Serialize, Deserialize, JsonSchema)]
#[tool]
pub struct Calculator {
    /// The operation to perform
    operation: String,
    /// The operands
    operands: Vec<f64>,
}

impl Calculator {
    pub async fn execute(&self) -> Result<f64> {
        match self.operation.as_str() {
            "add" => Ok(self.operands.iter().sum()),
            "multiply" => Ok(self.operands.iter().product()),
            _ => Err(anyhow::anyhow!("Unknown operation")),
        }
    }
}

fn main() {
    // Test compilation
}