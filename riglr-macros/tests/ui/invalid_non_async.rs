use riglr_macros::tool;
use anyhow::Result;

/// This should fail because the function is not async
#[tool]
pub fn sync_function(name: String) -> Result<String> {
    Ok(format!("Hello, {}!", name))
}

fn main() {}