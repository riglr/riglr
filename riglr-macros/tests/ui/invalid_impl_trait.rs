// Test that impl Trait parameters are rejected by the macro

use riglr_macros::tool;

// This should fail to compile because impl Trait parameters are not supported
#[tool]
pub async fn impl_trait(
    displayable: impl std::fmt::Display
) -> Result<String, anyhow::Error> {
    Ok(displayable.to_string())
}

fn main() {}