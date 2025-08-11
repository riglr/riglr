use riglr_macros::tool;
use anyhow::Result;

/// Calculate the sum of two numbers
#[tool]
pub async fn add_numbers(
    // The first number
    a: i32,
    // The second number  
    b: i32,
    // Whether to return absolute value
    absolute: bool,
) -> Result<i32> {
    let sum = a + b;
    if absolute {
        Ok(sum.abs())
    } else {
        Ok(sum)
    }
}

fn main() {
    // Test compilation
}
