use riglr_macros::tool;
use anyhow::Result;

/// A simple tool that greets someone
#[tool]
pub async fn greet(name: String) -> Result<String> {
    Ok(format!("Hello, {}!", name))
}

fn main() {
    // This file just needs to compile successfully
}
