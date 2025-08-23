use riglr_macros::tool;
use riglr_core::provider::ApplicationContext;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CustomData {
    value: String,
    count: i32,
}

#[tool]
async fn complex_tool(
    context: &ApplicationContext,
    data: CustomData,
    numbers: Vec<i32>,
    optional: Option<String>,
) -> Result<String, String> {
    Ok(format!("Processed {} items", numbers.len()))
}

fn main() {}