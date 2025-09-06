use riglr_core::provider::ApplicationContext;
use riglr_macros::tool;

#[tool]
async fn non_result_tool(context: &ApplicationContext, name: String) -> String {
    format!("Hello {}", name)
}

fn main() {}
