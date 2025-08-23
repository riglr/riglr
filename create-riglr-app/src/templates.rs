//! Template management and storage

use anyhow::Result;
use include_dir::{include_dir, Dir};
use std::path::PathBuf;

use crate::config::{Template, TemplateInfo};

// Embed template files at compile time
#[allow(dead_code)]
static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Template manager for handling template operations
pub struct TemplateManager {
    #[allow(dead_code)]
    templates_path: PathBuf,
}

impl Default for TemplateManager {
    fn default() -> Self {
        // Use local templates directory or embedded templates
        let templates_path = std::env::current_dir().unwrap().join("templates");

        Self { templates_path }
    }
}

impl TemplateManager {
    /// List all available templates
    pub fn list_templates(&self) -> Result<Vec<TemplateInfo>> {
        let templates = [
            Template::ApiServiceBackend,
            Template::DataAnalyticsBot,
            Template::EventDrivenTradingEngine,
            Template::TradingBot,
            Template::MarketAnalyst,
            Template::NewsMonitor,
            Template::DexArbitrageBot,
            Template::PortfolioTracker,
            Template::BridgeMonitor,
            Template::MevProtectionAgent,
            Template::DaoGovernanceBot,
            Template::NftTradingBot,
            Template::YieldOptimizer,
            Template::SocialTradingCopier,
            Template::Custom,
        ];

        Ok(templates.iter().map(TemplateInfo::from_template).collect())
    }

    /// Get detailed information about a specific template
    pub fn get_template_info(&self, name: &str) -> Result<TemplateInfo> {
        // Only accept exact template names, not aliases
        let valid_names = [
            "api-service",
            "data-analytics",
            "event-driven",
            "trading-bot",
            "market-analyst",
            "news-monitor",
            "dex-arbitrage",
            "portfolio-tracker",
            "bridge-monitor",
            "mev-protection",
            "dao-governance",
            "nft-trading",
            "yield-optimizer",
            "social-trading",
            "custom",
        ];

        if !valid_names.contains(&name) {
            return Err(anyhow::anyhow!("Invalid template name: {}", name));
        }

        let template = Template::parse(name)?;
        Ok(TemplateInfo::from_template(&template))
    }

    /// Get template content for a specific template
    pub fn get_template_content(&self, template: &Template) -> Result<TemplateContent> {
        match template {
            Template::ApiServiceBackend => Ok(self.get_api_service_template()),
            Template::DataAnalyticsBot => Ok(self.get_data_analytics_template()),
            Template::EventDrivenTradingEngine => Ok(self.get_event_driven_template()),
            _ => Ok(self.get_default_template()),
        }
    }

    /// Update templates from remote repository
    pub async fn update_templates(&self) -> Result<()> {
        // In a real implementation, this would fetch templates from a git repository
        // For now, we'll just return success
        Ok(())
    }

    fn get_api_service_template(&self) -> TemplateContent {
        TemplateContent {
            main_rs: include_str!("../templates/api_service/main.rs.hbs").to_string(),
            lib_rs: Some(include_str!("../templates/api_service/lib.rs.hbs").to_string()),
            cargo_toml: include_str!("../templates/api_service/Cargo.toml.hbs").to_string(),
            env_example: include_str!("../templates/api_service/.env.example.hbs").to_string(),
            additional_files: vec![
                (
                    "src/routes/mod.rs",
                    include_str!("../templates/api_service/routes/mod.rs.hbs"),
                ),
                (
                    "src/routes/health.rs",
                    include_str!("../templates/api_service/routes/health.rs.hbs"),
                ),
                (
                    "src/routes/agent.rs",
                    include_str!("../templates/api_service/routes/agent.rs.hbs"),
                ),
                (
                    "src/middleware/mod.rs",
                    include_str!("../templates/api_service/middleware/mod.rs.hbs"),
                ),
                (
                    "src/middleware/auth.rs",
                    include_str!("../templates/api_service/middleware/auth.rs.hbs"),
                ),
                (
                    "src/middleware/rate_limit.rs",
                    include_str!("../templates/api_service/middleware/rate_limit.rs.hbs"),
                ),
            ],
        }
    }

    fn get_data_analytics_template(&self) -> TemplateContent {
        TemplateContent {
            main_rs: r#"//! {{project_name}} - Data Analytics Bot

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("{{project_name}}=info")
        .init();

    info!("Starting {{project_name}} Data Analytics Bot");

    // Initialize analytics engine
    let analytics = analytics::Engine::new();
    
    // Start data ingestion
    let ingestion = data::Ingestion::new();
    
    // Run analytics
    analytics.run().await?;

    Ok(())
}"#
            .to_string(),
            lib_rs: Some(
                r#"//! {{project_name}} library

pub mod analytics;
pub mod data;
pub mod reports;

pub use analytics::Engine;
pub use data::Ingestion;"#
                    .to_string(),
            ),
            cargo_toml: r#"[package]
name = "{{name}}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1""#
                .to_string(),
            env_example: r#"# Data Analytics Bot Configuration
RUST_LOG=info
DATABASE_URL=postgresql://localhost/analytics"#
                .to_string(),
            additional_files: vec![
                (
                    "src/analytics/mod.rs",
                    include_str!("../templates/data_analytics/analytics/mod.rs.hbs"),
                ),
                (
                    "src/analytics/metrics.rs",
                    include_str!("../templates/data_analytics/analytics/metrics.rs.hbs"),
                ),
                (
                    "src/analytics/patterns.rs",
                    include_str!("../templates/data_analytics/analytics/patterns.rs.hbs"),
                ),
                (
                    "src/data/mod.rs",
                    include_str!("../templates/data_analytics/data/mod.rs.hbs"),
                ),
                (
                    "src/data/ingestion.rs",
                    include_str!("../templates/data_analytics/data/ingestion.rs.hbs"),
                ),
                (
                    "src/data/storage.rs",
                    include_str!("../templates/data_analytics/data/storage.rs.hbs"),
                ),
            ],
        }
    }

    fn get_event_driven_template(&self) -> TemplateContent {
        TemplateContent {
            main_rs: r#"//! {{project_name}} - Event-Driven Trading Engine

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("{{project_name}}=info")
        .init();

    info!("Starting {{project_name}} Event-Driven Trading Engine");

    // Initialize event processor
    let processor = events::Processor::new();
    
    // Initialize strategies
    let strategies = strategies::Base::new();
    
    // Initialize execution engine
    let engine = execution::Engine::new();
    
    // Start processing
    processor.run().await?;

    Ok(())
}"#
            .to_string(),
            lib_rs: Some(
                r#"//! {{project_name}} library

pub mod events;
pub mod strategies;
pub mod execution;"#
                    .to_string(),
            ),
            cargo_toml: r#"[package]
name = "{{name}}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
serde = { version = "1", features = ["derive"] }"#
                .to_string(),
            env_example: r#"# Event-Driven Trading Engine Configuration
RUST_LOG=info"#
                .to_string(),
            additional_files: vec![
                (
                    "src/events/mod.rs",
                    include_str!("../templates/event_driven/events/mod.rs.hbs"),
                ),
                (
                    "src/events/processor.rs",
                    include_str!("../templates/event_driven/events/processor.rs.hbs"),
                ),
                (
                    "src/events/handlers.rs",
                    include_str!("../templates/event_driven/events/handlers.rs.hbs"),
                ),
                (
                    "src/strategies/mod.rs",
                    include_str!("../templates/event_driven/strategies/mod.rs.hbs"),
                ),
                (
                    "src/strategies/base.rs",
                    include_str!("../templates/event_driven/strategies/base.rs.hbs"),
                ),
                (
                    "src/execution/mod.rs",
                    include_str!("../templates/event_driven/execution/mod.rs.hbs"),
                ),
                (
                    "src/execution/engine.rs",
                    include_str!("../templates/event_driven/execution/engine.rs.hbs"),
                ),
            ],
        }
    }

    fn get_default_template(&self) -> TemplateContent {
        // For now, return a basic template
        // In production, each template would have its own files
        TemplateContent {
            main_rs: include_str!("../templates/default/main.rs.hbs").to_string(),
            lib_rs: None,
            cargo_toml: include_str!("../templates/default/Cargo.toml.hbs").to_string(),
            env_example: include_str!("../templates/default/.env.example.hbs").to_string(),
            additional_files: vec![],
        }
    }
}

/// Template content structure
pub struct TemplateContent {
    /// Main Rust source file content
    pub main_rs: String,
    /// Optional library Rust source file content
    pub lib_rs: Option<String>,
    /// Cargo.toml configuration file content
    pub cargo_toml: String,
    /// Environment example file content
    pub env_example: String,
    /// Additional template files as (path, content) pairs
    pub additional_files: Vec<(&'static str, &'static str)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_manager_new_should_create_instance() {
        let manager = TemplateManager::default();
        // Verify the manager was created successfully
        assert!(manager.templates_path.ends_with("templates"));
    }

    #[test]
    fn test_list_templates_should_return_all_templates() {
        let manager = TemplateManager::default();
        let result = manager.list_templates();

        assert!(result.is_ok());
        let templates = result.unwrap();
        assert_eq!(templates.len(), 15); // All 15 templates

        // Verify specific templates are included
        let template_names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
        assert!(template_names.contains(&"api-service"));
        assert!(template_names.contains(&"data-analytics"));
        assert!(template_names.contains(&"event-driven"));
        assert!(template_names.contains(&"trading-bot"));
        assert!(template_names.contains(&"market-analyst"));
        assert!(template_names.contains(&"news-monitor"));
        assert!(template_names.contains(&"dex-arbitrage"));
        assert!(template_names.contains(&"portfolio-tracker"));
        assert!(template_names.contains(&"bridge-monitor"));
        assert!(template_names.contains(&"mev-protection"));
        assert!(template_names.contains(&"dao-governance"));
        assert!(template_names.contains(&"nft-trading"));
        assert!(template_names.contains(&"yield-optimizer"));
        assert!(template_names.contains(&"social-trading"));
        assert!(template_names.contains(&"custom"));
    }

    #[test]
    fn test_get_template_info_when_valid_name_should_return_ok() {
        let manager = TemplateManager::default();

        // Test all valid template names
        let valid_names = [
            "api-service",
            "data-analytics",
            "event-driven",
            "trading-bot",
            "market-analyst",
            "news-monitor",
            "dex-arbitrage",
            "portfolio-tracker",
            "bridge-monitor",
            "mev-protection",
            "dao-governance",
            "nft-trading",
            "yield-optimizer",
            "social-trading",
            "custom",
        ];

        for name in &valid_names {
            let result = manager.get_template_info(name);
            assert!(result.is_ok(), "Should succeed for valid name: {}", name);
            let template_info = result.unwrap();
            assert_eq!(template_info.name, *name);
        }
    }

    #[test]
    fn test_get_template_info_when_invalid_name_should_return_err() {
        let manager = TemplateManager::default();

        let invalid_names = [
            "invalid-template",
            "api_service", // underscore instead of dash
            "apiservice",  // no dash
            "API-SERVICE", // uppercase
            "",            // empty string
            "non-existent",
        ];

        for name in &invalid_names {
            let result = manager.get_template_info(name);
            assert!(result.is_err(), "Should fail for invalid name: {}", name);
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Invalid template name"));
        }
    }

    #[test]
    fn test_get_template_content_when_api_service_should_return_api_template() {
        let manager = TemplateManager::default();
        let result = manager.get_template_content(&Template::ApiServiceBackend);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Verify it contains API service specific content
        assert!(!content.main_rs.is_empty());
        assert!(content.lib_rs.is_some());
        assert!(!content.cargo_toml.is_empty());
        assert!(!content.env_example.is_empty());
        assert!(!content.additional_files.is_empty());

        // Verify specific API service files are included
        let file_paths: Vec<&str> = content
            .additional_files
            .iter()
            .map(|(path, _)| *path)
            .collect();
        assert!(file_paths.contains(&"src/routes/mod.rs"));
        assert!(file_paths.contains(&"src/routes/health.rs"));
        assert!(file_paths.contains(&"src/routes/agent.rs"));
        assert!(file_paths.contains(&"src/middleware/mod.rs"));
        assert!(file_paths.contains(&"src/middleware/auth.rs"));
        assert!(file_paths.contains(&"src/middleware/rate_limit.rs"));
    }

    #[test]
    fn test_get_template_content_when_data_analytics_should_return_analytics_template() {
        let manager = TemplateManager::default();
        let result = manager.get_template_content(&Template::DataAnalyticsBot);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Verify it contains data analytics specific content
        assert!(!content.main_rs.is_empty());
        assert!(content.lib_rs.is_some());
        assert!(!content.cargo_toml.is_empty());
        assert!(!content.env_example.is_empty());
        assert!(!content.additional_files.is_empty());

        // Verify content contains data analytics specific text
        assert!(content.main_rs.contains("Data Analytics Bot"));
        assert!(content.lib_rs.as_ref().unwrap().contains("analytics"));
        assert!(content.env_example.contains("DATABASE_URL"));

        // Verify specific data analytics files are included
        let file_paths: Vec<&str> = content
            .additional_files
            .iter()
            .map(|(path, _)| *path)
            .collect();
        assert!(file_paths.contains(&"src/analytics/mod.rs"));
        assert!(file_paths.contains(&"src/analytics/metrics.rs"));
        assert!(file_paths.contains(&"src/analytics/patterns.rs"));
        assert!(file_paths.contains(&"src/data/mod.rs"));
        assert!(file_paths.contains(&"src/data/ingestion.rs"));
        assert!(file_paths.contains(&"src/data/storage.rs"));
    }

    #[test]
    fn test_get_template_content_when_event_driven_should_return_event_template() {
        let manager = TemplateManager::default();
        let result = manager.get_template_content(&Template::EventDrivenTradingEngine);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Verify it contains event driven specific content
        assert!(!content.main_rs.is_empty());
        assert!(content.lib_rs.is_some());
        assert!(!content.cargo_toml.is_empty());
        assert!(!content.env_example.is_empty());
        assert!(!content.additional_files.is_empty());

        // Verify content contains event driven specific text
        assert!(content.main_rs.contains("Event-Driven Trading Engine"));
        assert!(content.lib_rs.as_ref().unwrap().contains("events"));

        // Verify specific event driven files are included
        let file_paths: Vec<&str> = content
            .additional_files
            .iter()
            .map(|(path, _)| *path)
            .collect();
        assert!(file_paths.contains(&"src/events/mod.rs"));
        assert!(file_paths.contains(&"src/events/processor.rs"));
        assert!(file_paths.contains(&"src/events/handlers.rs"));
        assert!(file_paths.contains(&"src/strategies/mod.rs"));
        assert!(file_paths.contains(&"src/strategies/base.rs"));
        assert!(file_paths.contains(&"src/execution/mod.rs"));
        assert!(file_paths.contains(&"src/execution/engine.rs"));
    }

    #[test]
    fn test_get_template_content_when_other_template_should_return_default() {
        let manager = TemplateManager::default();
        let templates = [
            Template::TradingBot,
            Template::MarketAnalyst,
            Template::NewsMonitor,
            Template::DexArbitrageBot,
            Template::PortfolioTracker,
            Template::BridgeMonitor,
            Template::MevProtectionAgent,
            Template::DaoGovernanceBot,
            Template::NftTradingBot,
            Template::YieldOptimizer,
            Template::SocialTradingCopier,
            Template::Custom,
        ];

        for template in &templates {
            let result = manager.get_template_content(template);
            assert!(result.is_ok());

            let content = result.unwrap();
            // Verify it returns default template content
            assert!(!content.main_rs.is_empty());
            assert!(content.lib_rs.is_none()); // Default template has no lib.rs
            assert!(!content.cargo_toml.is_empty());
            assert!(!content.env_example.is_empty());
            assert!(content.additional_files.is_empty()); // Default template has no additional files
        }
    }

    #[test]
    fn test_update_templates_should_return_ok() {
        let manager = TemplateManager::default();
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(manager.update_templates());
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_api_service_template_should_return_complete_content() {
        let manager = TemplateManager::default();
        let content = manager.get_api_service_template();

        // Verify all fields are populated
        assert!(!content.main_rs.is_empty());
        assert!(content.lib_rs.is_some());
        assert!(!content.lib_rs.as_ref().unwrap().is_empty());
        assert!(!content.cargo_toml.is_empty());
        assert!(!content.env_example.is_empty());
        assert_eq!(content.additional_files.len(), 6);

        // Verify file contents are not empty
        for (path, file_content) in &content.additional_files {
            assert!(!path.is_empty());
            assert!(!file_content.is_empty());
        }
    }

    #[test]
    fn test_get_data_analytics_template_should_return_complete_content() {
        let manager = TemplateManager::default();
        let content = manager.get_data_analytics_template();

        // Verify all fields are populated
        assert!(!content.main_rs.is_empty());
        assert!(content.lib_rs.is_some());
        assert!(!content.lib_rs.as_ref().unwrap().is_empty());
        assert!(!content.cargo_toml.is_empty());
        assert!(!content.env_example.is_empty());
        assert_eq!(content.additional_files.len(), 6);

        // Verify specific content patterns
        assert!(content.main_rs.contains("{{project_name}}"));
        assert!(content.cargo_toml.contains("{{name}}"));
        assert!(content.env_example.contains("DATABASE_URL"));
    }

    #[test]
    fn test_get_event_driven_template_should_return_complete_content() {
        let manager = TemplateManager::default();
        let content = manager.get_event_driven_template();

        // Verify all fields are populated
        assert!(!content.main_rs.is_empty());
        assert!(content.lib_rs.is_some());
        assert!(!content.lib_rs.as_ref().unwrap().is_empty());
        assert!(!content.cargo_toml.is_empty());
        assert!(!content.env_example.is_empty());
        assert_eq!(content.additional_files.len(), 7);

        // Verify specific content patterns
        assert!(content.main_rs.contains("{{project_name}}"));
        assert!(content.cargo_toml.contains("{{name}}"));
        assert!(content.lib_rs.as_ref().unwrap().contains("events"));
        assert!(content.lib_rs.as_ref().unwrap().contains("strategies"));
        assert!(content.lib_rs.as_ref().unwrap().contains("execution"));
    }

    #[test]
    fn test_get_default_template_should_return_basic_content() {
        let manager = TemplateManager::default();
        let content = manager.get_default_template();

        // Verify basic structure
        assert!(!content.main_rs.is_empty());
        assert!(content.lib_rs.is_none());
        assert!(!content.cargo_toml.is_empty());
        assert!(!content.env_example.is_empty());
        assert!(content.additional_files.is_empty());
    }

    #[test]
    fn test_template_content_structure() {
        // Test TemplateContent can be created with all field types
        let content = TemplateContent {
            main_rs: "test main".to_string(),
            lib_rs: Some("test lib".to_string()),
            cargo_toml: "test cargo".to_string(),
            env_example: "test env".to_string(),
            additional_files: vec![("path", "content")],
        };

        assert_eq!(content.main_rs, "test main");
        assert_eq!(content.lib_rs.unwrap(), "test lib");
        assert_eq!(content.cargo_toml, "test cargo");
        assert_eq!(content.env_example, "test env");
        assert_eq!(content.additional_files.len(), 1);
        assert_eq!(content.additional_files[0].0, "path");
        assert_eq!(content.additional_files[0].1, "content");
    }

    #[test]
    fn test_template_content_with_none_lib_rs() {
        // Test TemplateContent with None lib_rs
        let content = TemplateContent {
            main_rs: "test main".to_string(),
            lib_rs: None,
            cargo_toml: "test cargo".to_string(),
            env_example: "test env".to_string(),
            additional_files: vec![],
        };

        assert_eq!(content.main_rs, "test main");
        assert!(content.lib_rs.is_none());
        assert_eq!(content.cargo_toml, "test cargo");
        assert_eq!(content.env_example, "test env");
        assert!(content.additional_files.is_empty());
    }
}
