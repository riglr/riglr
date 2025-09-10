//! New command implementation

use crate::config::{ProjectConfig, ServerFramework, Template};
use crate::templates::TemplateManager;
use anyhow::Result;
use std::path::PathBuf;

/// Create a new project from a template
pub async fn run(template: &str, name: &str, output: Option<PathBuf>) -> Result<()> {
    let _manager = TemplateManager::default();
    let template_enum = Template::parse(template)?;

    // Create config with sensible defaults
    let config = ProjectConfig {
        name: name.to_string(),
        template: template_enum.clone(),
        chains: vec![],
        server_framework: if template_enum == Template::ApiServiceBackend {
            Some(ServerFramework::Actix)
        } else {
            None
        },
        features: template_enum.default_features(),
        author_name: whoami::realname(),
        author_email: format!("{}@example.com", whoami::username()),
        description: format!("{} built with RIGLR", name),
        include_examples: true,
        include_tests: true,
        include_docs: false,
    };

    // Use the generate_project function from main
    crate::generate_project(config, output).await
}
