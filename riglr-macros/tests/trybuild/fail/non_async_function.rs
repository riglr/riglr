use riglr_macros::tool;
use riglr_core::provider::ApplicationContext;

#[tool]
fn non_async_tool(
    context: &ApplicationContext,
    name: String,
) -> Result<String, String> {
    Ok(format!("Hello {}", name))
}

fn main() {}