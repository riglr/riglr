//! Project generation logic

use anyhow::Result;
use handlebars::Handlebars;
use std::fs;
use std::path::Path;

use crate::config::{Chain, Feature, ProjectConfig, ServerFramework, Template};
use crate::dependencies::get_dependency_version;
use crate::templates::TemplateManager;

/// Template context for Handlebars rendering
#[derive(serde::Serialize)]
struct TemplateContext {
    project_name: String,
    description: String,
    author_name: String,
    author_email: String,
    template: String,
    template_description: String,
    has_solana: bool,
    has_evm: bool,
    has_server: bool,
    server_framework: String,
    // Feature flags
    has_web_tools: bool,
    has_graph_memory: bool,
    has_cross_chain: bool,
    has_auth: bool,
    has_streaming: bool,
    has_database: bool,
    has_redis: bool,
    has_api_docs: bool,
    has_cicd: bool,
    has_docker: bool,
    has_tests: bool,
    has_examples: bool,
    has_docs: bool,
    has_logging: bool,
    // Additional flags
    include_examples: bool,
    include_tests: bool,
    include_docs: bool,
    // Chain list for iteration in templates
    chains: Vec<String>,
}

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

        if self.config.features.contains(&Feature::Database) {
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
            let content = self.handlebars.render_template(&template, &data)?;
            let file_path = output_dir.join(&path);
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

        // Generate Cargo.toml programmatically
        let cargo_base = self
            .handlebars
            .render_template(&template_content.cargo_toml, &data)?;
        let cargo_content = self.build_cargo_toml(cargo_base)?;
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
        if self.config.features.contains(&Feature::CiCd) {
            self.generate_cicd_config(output_dir)?;
        }

        if self.config.features.contains(&Feature::Docker) {
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
    fn prepare_template_data(&self) -> TemplateContext {
        let chain_strings: Vec<String> = self.config.chains.iter().map(|c| c.to_string()).collect();

        TemplateContext {
            project_name: self.config.name.clone(),
            description: self.config.description.clone(),
            author_name: self.config.author_name.clone(),
            author_email: self.config.author_email.clone(),
            template: self.config.template.to_string(),
            template_description: self.config.template.description().to_string(),
            has_solana: self.config.chains.contains(&Chain::Solana),
            has_evm: self.config.chains.iter().any(|c| *c != Chain::Solana),
            has_server: self.config.server_framework.is_some(),
            server_framework: self
                .config
                .server_framework
                .as_ref()
                .map(|f| format!("{:?}", f).to_lowercase())
                .unwrap_or_default(),
            // Feature flags
            has_web_tools: self.config.features.contains(&Feature::WebTools),
            has_graph_memory: self.config.features.contains(&Feature::GraphMemory),
            has_cross_chain: self.config.features.contains(&Feature::CrossChain),
            has_auth: self.config.features.contains(&Feature::Auth),
            has_streaming: self.config.features.contains(&Feature::Streaming),
            has_database: self.config.features.contains(&Feature::Database),
            has_redis: self.config.features.contains(&Feature::Redis),
            has_api_docs: self.config.features.contains(&Feature::ApiDocs),
            has_cicd: self.config.features.contains(&Feature::CiCd),
            has_docker: self.config.features.contains(&Feature::Docker),
            has_tests: self.config.features.contains(&Feature::Tests),
            has_examples: self.config.features.contains(&Feature::Examples),
            has_docs: self.config.features.contains(&Feature::Docs),
            has_logging: self.config.features.contains(&Feature::Logging),
            // Additional flags
            include_examples: self.config.include_examples,
            include_tests: self.config.include_tests,
            include_docs: self.config.include_docs,
            // Chain list for iteration in templates
            chains: chain_strings,
        }
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

    /// Programmatically build Cargo.toml with dynamic dependencies
    fn build_cargo_toml(&self, base_toml: String) -> Result<String> {
        use toml_edit::{Array, DocumentMut, InlineTable};

        // Parse the base template
        let mut doc: DocumentMut = base_toml.parse()?;

        // Get the dependencies table
        let deps = doc["dependencies"]
            .as_table_mut()
            .ok_or_else(|| anyhow::anyhow!("No dependencies table found"))?;

        // Add chain-specific dependencies
        if self.config.chains.contains(&Chain::Solana) {
            let mut workspace_table = InlineTable::new();
            workspace_table.insert("workspace", true.into());
            deps.insert(
                "riglr-solana-tools",
                toml_edit::Item::Value(workspace_table.into()),
            );
            let solana_client_version = get_dependency_version("solana-client");
            let solana_sdk_version = get_dependency_version("solana-sdk");
            deps.insert(
                "solana-client",
                toml_edit::Item::Value(solana_client_version.into()),
            );
            deps.insert(
                "solana-sdk",
                toml_edit::Item::Value(solana_sdk_version.into()),
            );
        }

        if self.config.chains.contains(&Chain::Ethereum)
            || self.config.chains.contains(&Chain::Polygon)
            || self.config.chains.contains(&Chain::Arbitrum)
            || self.config.chains.contains(&Chain::Base)
            || self.config.chains.contains(&Chain::Bsc)
            || self.config.chains.contains(&Chain::Avalanche)
        {
            let mut workspace_table = InlineTable::new();
            workspace_table.insert("workspace", true.into());
            deps.insert(
                "riglr-evm-tools",
                toml_edit::Item::Value(workspace_table.into()),
            );

            let alloy_version = get_dependency_version("alloy");
            let mut alloy_table = InlineTable::new();
            alloy_table.insert("version", alloy_version.into());
            let mut features = Array::new();
            features.push("full");
            alloy_table.insert("features", features.into());
            deps.insert("alloy", toml_edit::Item::Value(alloy_table.into()));
        }

        // Add feature-specific dependencies
        if self.config.features.contains(&Feature::WebTools) {
            let mut workspace_table = InlineTable::new();
            workspace_table.insert("workspace", true.into());
            deps.insert(
                "riglr-web-tools",
                toml_edit::Item::Value(workspace_table.into()),
            );
        }

        if self.config.features.contains(&Feature::Redis) {
            let redis_version = get_dependency_version("redis");
            let mut redis_table = InlineTable::new();
            redis_table.insert("version", redis_version.into());
            let mut features = Array::new();
            features.push("aio");
            features.push("tokio-comp");
            redis_table.insert("features", features.into());
            deps.insert("redis", toml_edit::Item::Value(redis_table.into()));
        }

        if self.config.features.contains(&Feature::Database) {
            let sqlx_version = get_dependency_version("sqlx");
            let mut sqlx_table = InlineTable::new();
            sqlx_table.insert("version", sqlx_version.into());
            let mut features = Array::new();
            features.push("runtime-tokio");
            features.push("postgres");
            sqlx_table.insert("features", features.into());
            deps.insert("sqlx", toml_edit::Item::Value(sqlx_table.into()));
        }

        // Add server framework dependencies
        if let Some(framework) = &self.config.server_framework {
            match framework {
                ServerFramework::Actix => {
                    let actix_version = get_dependency_version("actix-web");
                    let actix_cors_version = get_dependency_version("actix-cors");
                    deps.insert("actix-web", toml_edit::Item::Value(actix_version.into()));
                    deps.insert(
                        "actix-cors",
                        toml_edit::Item::Value(actix_cors_version.into()),
                    );
                }
                ServerFramework::Axum => {
                    let axum_version = get_dependency_version("axum");
                    let tower_version = get_dependency_version("tower");
                    let tower_http_version = get_dependency_version("tower-http");
                    deps.insert("axum", toml_edit::Item::Value(axum_version.into()));
                    deps.insert("tower", toml_edit::Item::Value(tower_version.into()));
                    let mut tower_http_table = InlineTable::new();
                    tower_http_table.insert("version", tower_http_version.into());
                    let mut features = Array::new();
                    features.push("cors");
                    tower_http_table.insert("features", features.into());
                    deps.insert(
                        "tower-http",
                        toml_edit::Item::Value(tower_http_table.into()),
                    );
                }
                ServerFramework::Warp => {
                    let warp_version = get_dependency_version("warp");
                    deps.insert("warp", toml_edit::Item::Value(warp_version.into()));
                }
                ServerFramework::Rocket => {
                    let rocket_version = get_dependency_version("rocket");
                    let rocket_cors_version = get_dependency_version("rocket_cors");
                    deps.insert("rocket", toml_edit::Item::Value(rocket_version.into()));
                    deps.insert(
                        "rocket_cors",
                        toml_edit::Item::Value(rocket_cors_version.into()),
                    );
                }
            }
        }

        Ok(doc.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Chain, Feature, ProjectConfig, ServerFramework, Template};
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
            chains: vec![Chain::Solana],
            features: vec![Feature::Database, Feature::CiCd, Feature::Docker],
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

        assert_eq!(data.project_name, "test_project");
        assert_eq!(data.description, "A test project");
        assert_eq!(data.author_name, "Test Author");
        assert_eq!(data.author_email, "test@example.com");
    }

    #[test]
    fn test_prepare_template_data_should_include_template_info() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert_eq!(data.template, "custom");
        assert!(!data.template_description.is_empty());
    }

    #[test]
    fn test_prepare_template_data_should_include_chain_info() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert_eq!(data.chains, vec!["solana"]);
        assert!(data.has_solana);
        assert!(!data.has_evm);
    }

    #[test]
    fn test_prepare_template_data_when_evm_chains_should_set_has_evm_true() {
        let mut config = create_test_config();
        config.chains = vec![Chain::Ethereum, Chain::Polygon];
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert!(data.has_evm);
        assert!(!data.has_solana);
    }

    #[test]
    fn test_prepare_template_data_when_mixed_chains_should_set_both_true() {
        let mut config = create_test_config();
        config.chains = vec![Chain::Solana, Chain::Ethereum];
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert!(data.has_evm);
        assert!(data.has_solana);
    }

    #[test]
    fn test_prepare_template_data_should_include_server_info() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert!(data.has_server);
        assert_eq!(data.server_framework, "axum");
    }

    #[test]
    fn test_prepare_template_data_when_no_server_should_set_has_server_false() {
        let config = create_minimal_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert!(!data.has_server);
        assert!(data.server_framework.is_empty());
    }

    #[test]
    fn test_prepare_template_data_should_include_feature_flags() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert!(data.has_database);
        assert!(data.has_cicd);
        assert!(data.has_docker);
    }

    #[test]
    fn test_prepare_template_data_should_include_boolean_flags() {
        let config = create_test_config();
        let generator = ProjectGenerator::new(config);

        let data = generator.prepare_template_data();

        assert!(data.include_examples);
        assert!(data.include_tests);
        assert!(data.include_docs);
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
