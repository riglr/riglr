use riglr_core::provider::ApplicationContext;
use riglr_macros::tool;

#[tool]
fn non_async_tool(context: &ApplicationContext, name: String) -> Result<String, String> {
    Ok(format!("Hello {}", name))
}

fn main() {}
