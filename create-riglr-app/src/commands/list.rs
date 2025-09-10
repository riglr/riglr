//! List command implementation

use crate::templates::TemplateManager;
use anyhow::Result;
use console::style;

/// List all available templates
pub async fn run() -> Result<()> {
    let manager = TemplateManager::default();
    let templates = manager.list_templates()?;

    println!("{}", style("Available Templates:").cyan().bold());
    println!();

    for template in templates {
        println!(
            "  {} {} - {}",
            style("•").green(),
            style(&template.name).yellow().bold(),
            template.description
        );
    }

    println!();
    println!(
        "Use {} to create a project with a specific template",
        style("create-riglr-app new <template> <name>").cyan()
    );

    Ok(())
}
