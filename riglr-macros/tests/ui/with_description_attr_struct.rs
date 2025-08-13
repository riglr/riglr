use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Doc comment description fallback for Calculator
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[tool(description = "Explicit calculator description.")]
pub struct Calculator {
    a: i32,
    b: i32,
}

impl Calculator {
    pub async fn execute(&self) -> Result<i32> { Ok(self.a + self.b) }
}

fn main() {}
