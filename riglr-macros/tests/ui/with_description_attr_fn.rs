use riglr_macros::tool;
use anyhow::Result;

/// Fallback doc comment that should not be used when attribute is present
#[tool(description = "Explicit add tool description.")]
pub async fn add(a: i32, b: i32) -> Result<i32> { Ok(a + b) }

fn main() {}
