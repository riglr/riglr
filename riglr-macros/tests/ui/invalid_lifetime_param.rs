// Test that lifetime parameters are rejected by the macro

use riglr_macros::tool;

// This should fail to compile because lifetime parameters are not supported
#[tool]
pub async fn with_lifetime<'a>(
    text: &'a str
) -> Result<&'a str, anyhow::Error> {
    Ok(text)
}

fn main() {}