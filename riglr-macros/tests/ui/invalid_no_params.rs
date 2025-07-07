use riglr_macros::tool;

/// This should fail because tool is applied to something that's not a function or struct
#[tool]
const INVALID: i32 = 42;

fn main() {}