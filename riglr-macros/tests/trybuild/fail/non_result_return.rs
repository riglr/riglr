use riglr_macros::tool;
use riglr_core::provider::ApplicationContext;

#[tool]
async fn non_result_tool(
    context: &ApplicationContext,
    name: String,
) -> String {
    format!("Hello {}", name)
}

fn main() {}