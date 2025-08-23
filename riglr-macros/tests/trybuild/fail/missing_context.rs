use riglr_macros::tool;

#[tool]
async fn missing_context_tool(
    name: String,
    age: u32,
) -> Result<String, String> {
    Ok(format!("Hello {}, age {}", name, age))
}

fn main() {}