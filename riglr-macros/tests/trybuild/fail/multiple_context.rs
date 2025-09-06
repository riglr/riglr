use riglr_core::provider::ApplicationContext;
use riglr_macros::tool;

#[tool]
async fn multiple_context_tool(
    context1: &ApplicationContext,
    context2: &ApplicationContext,
    name: String,
) -> Result<String, String> {
    Ok(format!("Hello {}", name))
}

fn main() {}
