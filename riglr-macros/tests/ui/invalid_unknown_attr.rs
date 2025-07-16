use riglr_macros::tool;
use anyhow::Result;

/// Using an unsupported attribute key should fail to compile
#[tool(desc = "wrong key")]
pub async fn bad() -> Result<()> { Ok(()) }

fn main() {}
