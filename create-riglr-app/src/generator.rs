//! Project generation logic

use anyhow::Result;
use handlebars::Handlebars;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::config::{ProjectConfig, ServerFramework, Template};
use crate::templates::TemplateManager;

/// Project generator for creating new RIGLR projects
pub struct ProjectGenerator {
    config: ProjectConfig,
    handlebars: Handlebars<'static>,
}

impl ProjectGenerator {
    /// Create a new ProjectGenerator with the given configuration
    pub fn new(config: ProjectConfig) -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        Self { config, handlebars }
    }

    /// Create the project directory structure
    pub fn create_structure(&self, output_dir: &Path) -> Result<()> {
        // Create main directories
        fs::create_dir_all(output_dir)?;
        fs::create_dir_all(output_dir.join("src"))?;
        fs::create_dir_all(output_dir.join("src/bin"))?;

        // Create template-specific directories
        match &self.config.template {
            Template::ApiServiceBackend => {
                fs::create_dir_all(output_dir.join("src/routes"))?;
                fs::create_dir_all(output_dir.join("src/middleware"))?;
                fs::create_dir_all(output_dir.join("src/handlers"))?;
            }
            Template::DataAnalyticsBot => {
                fs::create_dir_all(output_dir.join("src/analytics"))?;
                fs::create_dir_all(output_dir.join("src/data"))?;
                fs::create_dir_all(output_dir.join("src/reports"))?;
            }
            Template::EventDrivenTradingEngine => {
                fs::create_dir_all(output_dir.join("src/events"))?;
                fs::create_dir_all(output_dir.join("src/strategies"))?;
                fs::create_dir_all(output_dir.join("src/execution"))?;
            }
            _ => {}
        }

        // Create additional directories based on features
        if self.config.include_tests {
            fs::create_dir_all(output_dir.join("tests"))?;
        }

        if self.config.include_examples {
            fs::create_dir_all(output_dir.join("examples"))?;
        }

        if self.config.include_docs {
            fs::create_dir_all(output_dir.join("docs"))?;
        }

        if self.config.features.contains(&"database".to_string()) {
            fs::create_dir_all(output_dir.join("migrations"))?;
        }

        Ok(())
    }

    /// Generate source files
    pub fn generate_source_files(&self, output_dir: &Path) -> Result<()> {
        let manager = TemplateManager::default();
        let template_content = manager.get_template_content(&self.config.template)?;

        // Prepare template data
        let data = self.prepare_template_data();

        // Generate main.rs
        let main_content = self
            .handlebars
            .render_template(&template_content.main_rs, &data)?;
        fs::write(output_dir.join("src/main.rs"), main_content)?;

        // Generate lib.rs if present
        if let Some(lib_template) = template_content.lib_rs {
            let lib_content = self.handlebars.render_template(&lib_template, &data)?;
            fs::write(output_dir.join("src/lib.rs"), lib_content)?;
        }

        // Generate additional files
        for (path, template) in template_content.additional_files {
            let content = self.handlebars.render_template(template, &data)?;
            let file_path = output_dir.join(path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(file_path, content)?;
        }

        // Generate server files if needed
        if let Some(framework) = &self.config.server_framework {
            self.generate_server_files(output_dir, framework)?;
        }

        Ok(())
    }

    /// Generate configuration files
    pub fn generate_config_files(&self, output_dir: &Path) -> Result<()> {
        let manager = TemplateManager::default();
        let template_content = manager.get_template_content(&self.config.template)?;

        let data = self.prepare_template_data();

        // Generate Cargo.toml
        let cargo_content = self
            .handlebars
            .render_template(&template_content.cargo_toml, &data)?;
        fs::write(output_dir.join("Cargo.toml"), cargo_content)?;

        // Generate .env.example
        let env_content = self
            .handlebars
            .render_template(&template_content.env_example, &data)?;
        fs::write(output_dir.join(".env.example"), env_content)?;

        // Generate .gitignore
        let gitignore_content = self.generate_gitignore();
        fs::write(output_dir.join(".gitignore"), gitignore_content)?;

        // Generate additional config files
        if self.config.features.contains(&"cicd".to_string()) {
            self.generate_cicd_config(output_dir)?;
        }

        if self.config.features.contains(&"docker".to_string()) {
            self.generate_docker_files(output_dir)?;
        }

        Ok(())
    }

    /// Generate example files
    pub fn generate_examples(&self, output_dir: &Path) -> Result<()> {
        // Generate examples based on template type
        match &self.config.template {
            Template::ApiServiceBackend => {
                self.generate_api_examples(output_dir)?;
            }
            Template::TradingBot => {
                self.generate_trading_examples(output_dir)?;
            }
            _ => {
                self.generate_basic_example(output_dir)?;
            }
        }

        Ok(())
    }

    /// Generate README
    pub fn generate_readme(&self, output_dir: &Path) -> Result<()> {
        let readme_template = include_str!("../templates/README.md.hbs");
        let data = self.prepare_template_data();
        let readme_content = self.handlebars.render_template(readme_template, &data)?;
        fs::write(output_dir.join("README.md"), readme_content)?;

        Ok(())
    }

    /// Prepare template data for Handlebars
    fn prepare_template_data(&self) -> HashMap<String, serde_json::Value> {
        use serde_json::json;

        let mut data = HashMap::new();

        // Basic project info
        data.insert("project_name".to_string(), json!(self.config.name));
        data.insert("description".to_string(), json!(self.config.description));
        data.insert("author_name".to_string(), json!(self.config.author_name));
        data.insert("author_email".to_string(), json!(self.config.author_email));

        // Template info
        data.insert(
            "template".to_string(),
            json!(self.config.template.to_string()),
        );
        data.insert(
            "template_description".to_string(),
            json!(self.config.template.description()),
        );

        // Blockchain configuration
        data.insert("chains".to_string(), json!(self.config.chains));
        data.insert(
            "has_solana".to_string(),
            json!(self.config.chains.contains(&"solana".to_string())),
        );
        data.insert(
            "has_evm".to_string(),
            json!(self.config.chains.iter().any(|c| c != "solana")),
        );

        // Server configuration
        data.insert(
            "has_server".to_string(),
            json!(self.config.server_framework.is_some()),
        );
        if let Some(framework) = &self.config.server_framework {
            data.insert(
                "server_framework".to_string(),
                json!(format!("{:?}", framework).to_lowercase()),
            );
        }

        // Features
        for feature in &self.config.features {
            data.insert(format!("has_{}", feature), json!(true));
        }

        // Additional flags
        data.insert(
            "include_examples".to_string(),
            json!(self.config.include_examples),
        );
        data.insert(
            "include_tests".to_string(),
            json!(self.config.include_tests),
        );
        data.insert("include_docs".to_string(), json!(self.config.include_docs));

        data
    }

    /// Generate server-specific files
    fn generate_server_files(&self, output_dir: &Path, framework: &ServerFramework) -> Result<()> {
        match framework {
            ServerFramework::Actix => {
                self.generate_actix_server(output_dir)?;
            }
            ServerFramework::Axum => {
                self.generate_axum_server(output_dir)?;
            }
            ServerFramework::Warp => {
                self.generate_warp_server(output_dir)?;
            }
            ServerFramework::Rocket => {
                self.generate_rocket_server(output_dir)?;
            }
        }

        Ok(())
    }

    fn generate_actix_server(&self, output_dir: &Path) -> Result<()> {
        let server_template = include_str!("../templates/servers/actix.rs.hbs");
        let data = self.prepare_template_data();
        let server_content = self.handlebars.render_template(server_template, &data)?;
        fs::write(output_dir.join("src/server.rs"), server_content)?;
        Ok(())
    }

    fn generate_axum_server(&self, output_dir: &Path) -> Result<()> {
        let server_template = include_str!("../templates/servers/axum.rs.hbs");
        let data = self.prepare_template_data();
        let server_content = self.handlebars.render_template(server_template, &data)?;
        fs::write(output_dir.join("src/server.rs"), server_content)?;
        Ok(())
    }

    fn generate_warp_server(&self, output_dir: &Path) -> Result<()> {
        let server_template = include_str!("../templates/servers/warp.rs.hbs");
        let data = self.prepare_template_data();
        let server_content = self.handlebars.render_template(server_template, &data)?;
        fs::write(output_dir.join("src/server.rs"), server_content)?;
        Ok(())
    }

    fn generate_rocket_server(&self, output_dir: &Path) -> Result<()> {
        let server_template = include_str!("../templates/servers/rocket.rs.hbs");
        let data = self.prepare_template_data();
        let server_content = self.handlebars.render_template(server_template, &data)?;
        fs::write(output_dir.join("src/server.rs"), server_content)?;
        Ok(())
    }

    fn generate_api_examples(&self, output_dir: &Path) -> Result<()> {
        let example = r#"//! Example API client

use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Health check
    let response = client
        .get("http://localhost:8080/health")
        .send()
        .await?;
    println!("Health: {}", response.text().await?);

    // Agent query
    let response = client
        .post("http://localhost:8080/api/v1/agent/query")
        .json(&json!({
            "prompt": "What is the current SOL price?",
            "chain": "solana"
        }))
        .send()
        .await?;
    println!("Response: {}", response.text().await?);

    Ok(())
}
"#;
        fs::write(output_dir.join("examples/api_client.rs"), example)?;
        Ok(())
    }

    fn generate_trading_examples(&self, output_dir: &Path) -> Result<()> {
        let example = r#"//! Example trading strategy

use riglr_core::Job;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Trading example");
    // Add trading-specific example code
    Ok(())
}
"#;
        fs::write(output_dir.join("examples/trading_strategy.rs"), example)?;
        Ok(())
    }

    fn generate_basic_example(&self, output_dir: &Path) -> Result<()> {
        let example = r#"//! Basic usage example

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Basic RIGLR example");
    Ok(())
}
"#;
        fs::write(output_dir.join("examples/basic.rs"), example)?;
        Ok(())
    }

    fn generate_gitignore(&self) -> String {
        r#"# Rust
/target/
**/*.rs.bk
Cargo.lock

# Environment
.env
.env.local
.env.*.local

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Logs
*.log

# Dependencies
node_modules/

# Test coverage
*.profraw
*.profdata
/coverage/

# Database
*.db
*.sqlite

# Docker
.dockerignore
"#
        .to_string()
    }

    fn generate_cicd_config(&self, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir.join(".github/workflows"))?;

        let workflow = r#"name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    - uses: Swatinem/rust-cache@v2
    - name: Run tests
      run: cargo test --all-features
    - name: Run clippy
      run: cargo clippy -- -D warnings
"#;
        fs::write(output_dir.join(".github/workflows/ci.yml"), workflow)?;
        Ok(())
    }

    fn generate_docker_files(&self, output_dir: &Path) -> Result<()> {
        let dockerfile = r#"FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/{{project_name}} /usr/local/bin/

CMD ["{{project_name}}"]
"#;
        fs::write(output_dir.join("Dockerfile"), dockerfile)?;

        let dockerignore = r#"target/
Dockerfile
.dockerignore
.git
.gitignore
"#;
        fs::write(output_dir.join(".dockerignore"), dockerignore)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ProjectConfig, ServerFramework, Template};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_config() -> ProjectConfig {
        ProjectConfig {
            name: "test_project".to_string(),
            description: "A test project".to_string(),
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            template: Template::Custom,
            chains: vec!["solana".to_string()],
            features: vec![
                "database".to_string(),
                "cicd".to_string(),
                "docker".to_string(),
            ],
            server_framework: Some(ServerFramework::Axum),
            include_tests: true,
            include_examples: true,
            include_docs: true,
        }
    }

    fn create_minimal_config() -> ProjectConfig {
        ProjectConfig {
            name: "minimal_project".to_string(),
            description: "A minimal project".to_string(),
            author_name: "Minimal Author".to_string(),
            author_email: "minimal@example.com".to_string(),
            template: Template::Custom,
            chains: vec![],
            features: vec![],
            server_framework: None,
            include_tests: false,
            include_examples: false,
            include_docs: false,
        }
    }

    #[test]
    fn test_new_should_create_generator_with_config() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config.clone());

        assert_eq!(generator.config.name, config.name);
        assert_eq!(generator.config.description, config.description);
        assert_eq!(generator.config.author_name, config.author_name);
        assert_eq!(generator.config.author_email, config.author_email);
    }

    #[test]
    fn test_create_structure_when_default_template_should_create_basic_dirs() -> Result<()> {
        let config = create_minimal_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        generator.create_structure(output_path)?;

        assert!(output_path.join("src").exists());
        assert!(output_path.join("src/bin").exists());

        Ok(())
    }

    #[test]
    fn test_create_structure_when_api_service_template_should_create_api_dirs() -> Result<()> {
        let mut config = create_test_config();
        config.template = Template::ApiServiceBackend;
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        generator.create_structure(output_path)?;

        assert!(output_path.join("src/routes").exists());
        assert!(output_path.join("src/middleware").exists());
        assert!(output_path.join("src/handlers").exists());

        Ok(())
    }

    #[test]
    fn test_create_structure_when_data_analytics_template_should_create_analytics_dirs(
    ) -> Result<()> {
        let mut config = create_test_config();
        config.template = Template::DataAnalyticsBot;
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        generator.create_structure(output_path)?;

        assert!(output_path.join("src/analytics").exists());
        assert!(output_path.join("src/data").exists());
        assert!(output_path.join("src/reports").exists());

        Ok(())
    }

    #[test]
    fn test_create_structure_when_event_driven_template_should_create_trading_dirs() -> Result<()> {
        let mut config = create_test_config();
        config.template = Template::EventDrivenTradingEngine;
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        generator.create_structure(output_path)?;

        assert!(output_path.join("src/events").exists());
        assert!(output_path.join("src/strategies").exists());
        assert!(output_path.join("src/execution").exists());

        Ok(())
    }

    #[test]
    fn test_create_structure_when_include_tests_true_should_create_tests_dir() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        generator.create_structure(output_path)?;

        assert!(output_path.join("tests").exists());

        Ok(())
    }

    #[test]
    fn test_create_structure_when_include_examples_true_should_create_examples_dir() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        generator.create_structure(output_path)?;

        assert!(output_path.join("examples").exists());

        Ok(())
    }

    #[test]
    fn test_create_structure_when_include_docs_true_should_create_docs_dir() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        generator.create_structure(output_path)?;

        assert!(output_path.join("docs").exists());

        Ok(())
    }

    #[test]
    fn test_create_structure_when_database_feature_should_create_migrations_dir() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        generator.create_structure(output_path)?;

        assert!(output_path.join("migrations").exists());

        Ok(())
    }

    #[test]
    fn test_create_structure_when_minimal_config_should_not_create_optional_dirs() -> Result<()> {
        let config = create_minimal_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        generator.create_structure(output_path)?;

        assert!(!output_path.join("tests").exists());
        assert!(!output_path.join("examples").exists());
        assert!(!output_path.join("docs").exists());
        assert!(!output_path.join("migrations").exists());

        Ok(())
    }

    #[test]
    fn test_prepare_template_data_should_include_basic_project_info() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert_eq!(
            data.get("project_name").unwrap(),
            &serde_json::json!("test_project")
        );
        assert_eq!(
            data.get("description").unwrap(),
            &serde_json::json!("A test project")
        );
        assert_eq!(
            data.get("author_name").unwrap(),
            &serde_json::json!("Test Author")
        );
        assert_eq!(
            data.get("author_email").unwrap(),
            &serde_json::json!("test@example.com")
        );
    }

    #[test]
    fn test_prepare_template_data_should_include_template_info() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert_eq!(data.get("template").unwrap(), &serde_json::json!("custom"));
        assert!(data.contains_key("template_description"));
    }

    #[test]
    fn test_prepare_template_data_should_include_chain_info() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert_eq!(
            data.get("chains").unwrap(),
            &serde_json::json!(vec!["solana"])
        );
        assert_eq!(data.get("has_solana").unwrap(), &serde_json::json!(true));
        assert_eq!(data.get("has_evm").unwrap(), &serde_json::json!(false));
    }

    #[test]
    fn test_prepare_template_data_when_evm_chains_should_set_has_evm_true() {
        let mut config = create_test_config();
        config.chains = vec!["ethereum".to_string(), "polygon".to_string()];
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert_eq!(data.get("has_evm").unwrap(), &serde_json::json!(true));
        assert_eq!(data.get("has_solana").unwrap(), &serde_json::json!(false));
    }

    #[test]
    fn test_prepare_template_data_when_mixed_chains_should_set_both_true() {
        let mut config = create_test_config();
        config.chains = vec!["solana".to_string(), "ethereum".to_string()];
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert_eq!(data.get("has_evm").unwrap(), &serde_json::json!(true));
        assert_eq!(data.get("has_solana").unwrap(), &serde_json::json!(true));
    }

    #[test]
    fn test_prepare_template_data_should_include_server_info() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert_eq!(data.get("has_server").unwrap(), &serde_json::json!(true));
        assert_eq!(
            data.get("server_framework").unwrap(),
            &serde_json::json!("axum")
        );
    }

    #[test]
    fn test_prepare_template_data_when_no_server_should_set_has_server_false() {
        let config = create_minimal_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert_eq!(data.get("has_server").unwrap(), &serde_json::json!(false));
        assert!(!data.contains_key("server_framework"));
    }

    #[test]
    fn test_prepare_template_data_should_include_feature_flags() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert_eq!(data.get("has_database").unwrap(), &serde_json::json!(true));
        assert_eq!(data.get("has_cicd").unwrap(), &serde_json::json!(true));
        assert_eq!(data.get("has_docker").unwrap(), &serde_json::json!(true));
    }

    #[test]
    fn test_prepare_template_data_should_include_boolean_flags() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert_eq!(
            data.get("include_examples").unwrap(),
            &serde_json::json!(true)
        );
        assert_eq!(data.get("include_tests").unwrap(), &serde_json::json!(true));
        assert_eq!(data.get("include_docs").unwrap(), &serde_json::json!(true));
    }

    #[test]
    fn test_generate_gitignore_should_return_standard_content() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let gitignore = generator.generate_gitignore();

        assert!(gitignore.contains("/target/"));
        assert!(gitignore.contains(".env"));
        assert!(gitignore.contains(".DS_Store"));
        assert!(gitignore.contains("*.log"));
        assert!(gitignore.contains("node_modules/"));
    }

    #[test]
    fn test_generate_examples_when_api_service_should_call_api_examples() -> Result<()> {
        let mut config = create_test_config();
        config.template = Template::ApiServiceBackend;
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("examples"))?;

        generator.generate_examples(output_path)?;

        assert!(output_path.join("examples/api_client.rs").exists());
        let content = fs::read_to_string(output_path.join("examples/api_client.rs"))?;
        assert!(content.contains("Example API client"));

        Ok(())
    }

    #[test]
    fn test_generate_examples_when_trading_bot_should_call_trading_examples() -> Result<()> {
        let mut config = create_test_config();
        config.template = Template::TradingBot;
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("examples"))?;

        generator.generate_examples(output_path)?;

        assert!(output_path.join("examples/trading_strategy.rs").exists());
        let content = fs::read_to_string(output_path.join("examples/trading_strategy.rs"))?;
        assert!(content.contains("Example trading strategy"));

        Ok(())
    }

    #[test]
    fn test_generate_examples_when_default_template_should_call_basic_example() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("examples"))?;

        generator.generate_examples(output_path)?;

        assert!(output_path.join("examples/basic.rs").exists());
        let content = fs::read_to_string(output_path.join("examples/basic.rs"))?;
        assert!(content.contains("Basic usage example"));

        Ok(())
    }

    #[test]
    fn test_generate_api_examples_should_create_api_client_file() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("examples"))?;

        generator.generate_api_examples(output_path)?;

        assert!(output_path.join("examples/api_client.rs").exists());
        let content = fs::read_to_string(output_path.join("examples/api_client.rs"))?;
        assert!(content.contains("reqwest::Client"));
        assert!(content.contains("health"));
        assert!(content.contains("agent/query"));

        Ok(())
    }

    #[test]
    fn test_generate_trading_examples_should_create_trading_strategy_file() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("examples"))?;

        generator.generate_trading_examples(output_path)?;

        assert!(output_path.join("examples/trading_strategy.rs").exists());
        let content = fs::read_to_string(output_path.join("examples/trading_strategy.rs"))?;
        assert!(content.contains("riglr_core::Job"));
        assert!(content.contains("Trading example"));

        Ok(())
    }

    #[test]
    fn test_generate_basic_example_should_create_basic_file() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("examples"))?;

        generator.generate_basic_example(output_path)?;

        assert!(output_path.join("examples/basic.rs").exists());
        let content = fs::read_to_string(output_path.join("examples/basic.rs"))?;
        assert!(content.contains("Basic RIGLR example"));

        Ok(())
    }

    #[test]
    fn test_generate_cicd_config_should_create_workflow_file() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        generator.generate_cicd_config(output_path)?;

        assert!(output_path.join(".github/workflows").exists());
        assert!(output_path.join(".github/workflows/ci.yml").exists());
        let content = fs::read_to_string(output_path.join(".github/workflows/ci.yml"))?;
        assert!(content.contains("name: CI"));
        assert!(content.contains("cargo test"));
        assert!(content.contains("cargo clippy"));

        Ok(())
    }

    #[test]
    fn test_generate_docker_files_should_create_dockerfile_and_dockerignore() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        generator.generate_docker_files(output_path)?;

        assert!(output_path.join("Dockerfile").exists());
        assert!(output_path.join(".dockerignore").exists());

        let dockerfile_content = fs::read_to_string(output_path.join("Dockerfile"))?;
        assert!(dockerfile_content.contains("FROM rust:1.75"));
        assert!(dockerfile_content.contains("cargo build --release"));

        let dockerignore_content = fs::read_to_string(output_path.join(".dockerignore"))?;
        assert!(dockerignore_content.contains("target/"));
        assert!(dockerignore_content.contains(".git"));

        Ok(())
    }

    #[test]
    fn test_generate_server_files_when_actix_should_call_generate_actix() -> Result<()> {
        let mut config = create_test_config();
        config.server_framework = Some(ServerFramework::Actix);
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("src"))?;

        generator.generate_server_files(output_path, &ServerFramework::Actix)?;

        Ok(())
    }

    #[test]
    fn test_generate_server_files_when_axum_should_call_generate_axum() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("src"))?;

        generator.generate_server_files(output_path, &ServerFramework::Axum)?;

        Ok(())
    }

    #[test]
    fn test_generate_server_files_when_warp_should_call_generate_warp() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("src"))?;

        generator.generate_server_files(output_path, &ServerFramework::Warp)?;

        Ok(())
    }

    #[test]
    fn test_generate_server_files_when_rocket_should_call_generate_rocket() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("src"))?;

        generator.generate_server_files(output_path, &ServerFramework::Rocket)?;

        Ok(())
    }

    #[test]
    fn test_generate_actix_server_should_create_server_file() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("src"))?;

        generator.generate_actix_server(output_path)?;

        assert!(output_path.join("src/server.rs").exists());

        Ok(())
    }

    #[test]
    fn test_generate_axum_server_should_create_server_file() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("src"))?;

        generator.generate_axum_server(output_path)?;

        assert!(output_path.join("src/server.rs").exists());

        Ok(())
    }

    #[test]
    fn test_generate_warp_server_should_create_server_file() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("src"))?;

        generator.generate_warp_server(output_path)?;

        assert!(output_path.join("src/server.rs").exists());

        Ok(())
    }

    #[test]
    fn test_generate_rocket_server_should_create_server_file() -> Result<()> {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path();

        fs::create_dir_all(output_path.join("src"))?;

        generator.generate_rocket_server(output_path)?;

        assert!(output_path.join("src/server.rs").exists());

        Ok(())
    }

    #[test]
    fn test_create_structure_when_directory_creation_fails_should_propagate_error() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);
        let invalid_path = PathBuf::from("/this/path/does/not/exist/and/cannot/be/created");

        let result = generator.create_structure(&invalid_path);

        assert!(result.is_err());
    }
}
