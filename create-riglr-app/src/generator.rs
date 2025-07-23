//! Project generation logic

use anyhow::Result;
use handlebars::Handlebars;
use std::fs;
use std::path::Path;
use std::collections::HashMap;

use crate::config::{ProjectConfig, ServerFramework, Template};
use crate::templates::TemplateManager;

/// Project generator for creating new RIGLR projects
pub struct ProjectGenerator {
    config: ProjectConfig,
    handlebars: Handlebars<'static>,
}

impl ProjectGenerator {
    pub fn new(config: ProjectConfig) -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        
        Self {
            config,
            handlebars,
        }
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
        let manager = TemplateManager::new();
        let template_content = manager.get_template_content(&self.config.template)?;
        
        // Prepare template data
        let data = self.prepare_template_data();
        
        // Generate main.rs
        let main_content = self.handlebars.render_template(&template_content.main_rs, &data)?;
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
        let manager = TemplateManager::new();
        let template_content = manager.get_template_content(&self.config.template)?;
        
        let data = self.prepare_template_data();
        
        // Generate Cargo.toml
        let cargo_content = self.handlebars.render_template(&template_content.cargo_toml, &data)?;
        fs::write(output_dir.join("Cargo.toml"), cargo_content)?;
        
        // Generate .env.example
        let env_content = self.handlebars.render_template(&template_content.env_example, &data)?;
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
        data.insert("template".to_string(), json!(self.config.template.to_string()));
        data.insert("template_description".to_string(), json!(self.config.template.description()));
        
        // Blockchain configuration
        data.insert("chains".to_string(), json!(self.config.chains));
        data.insert("has_solana".to_string(), json!(self.config.chains.contains(&"solana".to_string())));
        data.insert("has_evm".to_string(), json!(self.config.chains.iter().any(|c| c != "solana")));
        
        // Server configuration
        data.insert("has_server".to_string(), json!(self.config.server_framework.is_some()));
        if let Some(framework) = &self.config.server_framework {
            data.insert("server_framework".to_string(), json!(format!("{:?}", framework).to_lowercase()));
        }
        
        // Features
        for feature in &self.config.features {
            data.insert(format!("has_{}", feature), json!(true));
        }
        
        // Additional flags
        data.insert("include_examples".to_string(), json!(self.config.include_examples));
        data.insert("include_tests".to_string(), json!(self.config.include_tests));
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
"#.to_string()
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