use riglr_macros::tool;
use riglr_core::provider::ApplicationContext;
use riglr_core::ToolError;

#[tool]
async fn basic_tool(
    _context: &ApplicationContext,
    name: String,
    age: u32,
) -> Result<String, ToolError> {
    Ok(format!("Hello {}, age {}", name, age))
}

fn main() {}