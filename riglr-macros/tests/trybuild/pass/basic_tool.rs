use riglr_macros::tool;
use riglr_core::provider::ApplicationContext;

#[tool]
async fn basic_tool(
    context: &ApplicationContext,
    name: String,
    age: u32,
) -> Result<String, String> {
    Ok(format!("Hello {}, age {}", name, age))
}

fn main() {}