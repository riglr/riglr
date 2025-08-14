// Test that complex parameter patterns are rejected by the macro

use riglr_macros::tool;

// This should fail to compile because complex patterns in parameters are not supported
#[tool]
pub async fn complex_patterns(
    /// Simple pattern (this should work)
    simple: String,
    /// Tuple pattern (this should fail)
    (first, second): (String, i32),
) -> Result<String, anyhow::Error> {
    Ok(format!("{}-{}-{}", simple, first, second))
}

fn main() {}