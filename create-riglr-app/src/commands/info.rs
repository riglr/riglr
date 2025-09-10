//! Info command implementation

use crate::templates::TemplateManager;
use anyhow::Result;
use console::style;

/// Show detailed information about a template
pub async fn run(template: &str) -> Result<()> {
    let manager = TemplateManager::default();
    let info = manager.get_template_info(template)?;

    println!("{}", style(&info.name).cyan().bold());
    println!("{}", style("─".repeat(40)).dim());
    println!();
    println!("{}", style("Description:").yellow());
    println!("  {}", info.description);
    println!();
    println!("{}", style("Features:").yellow());
    for feature in &info.features {
        println!("  • {}", feature);
    }
    println!();
    println!("{}", style("Default chains:").yellow());
    for chain in &info.default_chains {
        println!("  • {}", chain);
    }
    println!();
    println!("{}", style("Included tools:").yellow());
    for tool in &info.included_tools {
        println!("  • {}", tool);
    }

    Ok(())
}
