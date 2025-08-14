// Test that reference parameters are rejected by the macro

use riglr_macros::tool;

// This should fail to compile because reference parameters are not supported
// (they can't be deserialized from JSON)
#[tool]
pub async fn reference_param(
    /// Reference parameter (should fail)
    reference: &str,
) -> Result<String, anyhow::Error> {
    Ok(reference.to_string())
}

fn main() {}