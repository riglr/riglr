use riglr_macros::tool;
use riglr_core::provider::ApplicationContext;

/// This tool demonstrates a tool with description
#[tool]
async fn tool_with_description(
    context: &ApplicationContext,
    input: String,
) -> Result<String, String> {
    Ok(format!("Processed: {}", input))
}

fn main() {}