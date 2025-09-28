use riglr_macros::tool;
use riglr_core::provider::ApplicationContext;
use std::sync::Mutex;

#[tool]
async fn non_serializable_tool(
    context: &ApplicationContext,
    data: Mutex<String>,  // Mutex is not serializable
) -> Result<String, String> {
    Ok("result".to_string())
}

fn main() {}