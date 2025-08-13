//! Template management and storage

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use std::path::{Path, PathBuf};
use std::fs;

use crate::config::{Template, TemplateInfo};

// Embed template files at compile time
static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Template manager for handling template operations
pub struct TemplateManager {
    templates_path: PathBuf,
}

impl TemplateManager {
    pub fn new() -> Self {
        // Use local templates directory or embedded templates
        let templates_path = std::env::current_dir()
            .unwrap()
            .join("templates");
        
        Self { templates_path }
    }
    
    /// List all available templates
    pub fn list_templates(&self) -> Result<Vec<TemplateInfo>> {
        let templates = vec![
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
        
        Ok(templates.iter().map(|t| TemplateInfo::from_template(t)).collect())
    }
    
    /// Get detailed information about a specific template
    pub fn get_template_info(&self, name: &str) -> Result<TemplateInfo> {
        let template = Template::from_str(name)?;
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
                ("src/routes/mod.rs", include_str!("../templates/api_service/routes/mod.rs.hbs")),
                ("src/routes/health.rs", include_str!("../templates/api_service/routes/health.rs.hbs")),
                ("src/routes/agent.rs", include_str!("../templates/api_service/routes/agent.rs.hbs")),
                ("src/middleware/mod.rs", include_str!("../templates/api_service/middleware/mod.rs.hbs")),
                ("src/middleware/auth.rs", include_str!("../templates/api_service/middleware/auth.rs.hbs")),
                ("src/middleware/rate_limit.rs", include_str!("../templates/api_service/middleware/rate_limit.rs.hbs")),
            ],
        }
    }
    
    fn get_data_analytics_template(&self) -> TemplateContent {
        TemplateContent {
            main_rs: include_str!("../templates/data_analytics/main.rs.hbs").to_string(),
            lib_rs: Some(include_str!("../templates/data_analytics/lib.rs.hbs").to_string()),
            cargo_toml: include_str!("../templates/data_analytics/Cargo.toml.hbs").to_string(),
            env_example: include_str!("../templates/data_analytics/.env.example.hbs").to_string(),
            additional_files: vec![
                ("src/analytics/mod.rs", include_str!("../templates/data_analytics/analytics/mod.rs.hbs")),
                ("src/analytics/metrics.rs", include_str!("../templates/data_analytics/analytics/metrics.rs.hbs")),
                ("src/analytics/patterns.rs", include_str!("../templates/data_analytics/analytics/patterns.rs.hbs")),
                ("src/data/mod.rs", include_str!("../templates/data_analytics/data/mod.rs.hbs")),
                ("src/data/ingestion.rs", include_str!("../templates/data_analytics/data/ingestion.rs.hbs")),
                ("src/data/storage.rs", include_str!("../templates/data_analytics/data/storage.rs.hbs")),
            ],
        }
    }
    
    fn get_event_driven_template(&self) -> TemplateContent {
        TemplateContent {
            main_rs: include_str!("../templates/event_driven/main.rs.hbs").to_string(),
            lib_rs: Some(include_str!("../templates/event_driven/lib.rs.hbs").to_string()),
            cargo_toml: include_str!("../templates/event_driven/Cargo.toml.hbs").to_string(),
            env_example: include_str!("../templates/event_driven/.env.example.hbs").to_string(),
            additional_files: vec![
                ("src/events/mod.rs", include_str!("../templates/event_driven/events/mod.rs.hbs")),
                ("src/events/processor.rs", include_str!("../templates/event_driven/events/processor.rs.hbs")),
                ("src/events/handlers.rs", include_str!("../templates/event_driven/events/handlers.rs.hbs")),
                ("src/strategies/mod.rs", include_str!("../templates/event_driven/strategies/mod.rs.hbs")),
                ("src/strategies/base.rs", include_str!("../templates/event_driven/strategies/base.rs.hbs")),
                ("src/execution/mod.rs", include_str!("../templates/event_driven/execution/mod.rs.hbs")),
                ("src/execution/engine.rs", include_str!("../templates/event_driven/execution/engine.rs.hbs")),
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
    pub main_rs: String,
    pub lib_rs: Option<String>,
    pub cargo_toml: String,
    pub env_example: String,
    pub additional_files: Vec<(&'static str, &'static str)>,
}