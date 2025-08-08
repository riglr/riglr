use riglr_macros::tool;

#[tool]
pub async fn minimal_test() -> Result<i32, anyhow::Error> {
    Ok(42)
}