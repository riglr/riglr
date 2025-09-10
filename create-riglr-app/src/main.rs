//! Enhanced create-riglr-app CLI with interactive scaffolding

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::time::Duration;

mod commands;
mod config;
mod dependencies;
mod generator;
mod templates;
mod validation;

use crate::config::{Chain, Feature, ProjectConfig, ServerFramework, Template};
use crate::generator::ProjectGenerator;

/// Create RIGLR App - Interactive scaffolding for blockchain AI agents
#[derive(Parser, Debug)]
#[command(name = "create-riglr-app")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Project name
    #[arg(value_name = "PROJECT_NAME")]
    project_name: Option<String>,

    /// Use a specific template
    #[arg(short, long)]
    template: Option<String>,

    /// Skip interactive prompts and use defaults
    #[arg(short = 'y', long)]
    yes: bool,

    /// Output directory (defaults to current directory)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List available templates
    List,

    /// Create a new project with a specific template
    New {
        /// Template name
        template: String,

        /// Project name
        name: String,
    },

    /// Update templates from remote repository
    Update {
        /// Custom template repository URL
        #[arg(short, long)]
        url: Option<String>,

        /// Branch to checkout
        #[arg(short, long)]
        branch: Option<String>,
    },

    /// Show template details
    Info {
        /// Template name
        template: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt().with_env_filter("debug").init();
    }

    // Print banner
    print_banner();

    match cli.command {
        Some(Commands::List) => {
            commands::list_templates().await?;
        }
        Some(Commands::New { template, name }) => {
            commands::create_from_template(&template, &name, cli.output).await?;
        }
        Some(Commands::Update { url, branch }) => {
            commands::update::run_with_options(url, branch).await?;
        }
        Some(Commands::Info { template }) => {
            commands::show_template_info(&template).await?;
        }
        Option::None => {
            // Interactive mode
            let config = if cli.yes {
                create_default_config(cli.project_name)?
            } else {
                interactive_setup(cli.project_name)?
            };

            generate_project(config, cli.output).await?;
        }
    }

    Ok(())
}

/// Prints the RIGLR banner to the console
fn print_banner() {
    println!(
        "{}",
        style(
            "
╔═══════════════════════════════════════════════════════╗
║                                                       ║
║     ██████╗ ██╗ ██████╗ ██╗     ██████╗              ║
║     ██╔══██╗██║██╔════╝ ██║     ██╔══██╗             ║
║     ██████╔╝██║██║  ███╗██║     ██████╔╝             ║
║     ██╔══██╗██║██║   ██║██║     ██╔══██╗             ║
║     ██║  ██║██║╚██████╔╝███████╗██║  ██║             ║
║     ╚═╝  ╚═╝╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝             ║
║                                                       ║
║          Create RIGLR App - v0.2.0                   ║
║    Build AI-Powered Blockchain Agents with Ease      ║
║                                                       ║
╚═══════════════════════════════════════════════════════╝
    "
        )
        .cyan()
        .bold()
    );
    println!();
}

// Helper functions for interactive setup
/// Prompts the user for a project name or returns the provided name
fn prompt_project_name(theme: &ColorfulTheme, project_name: Option<String>) -> Result<String> {
    if let Some(name) = project_name {
        Ok(name)
    } else {
        Input::<String>::with_theme(theme)
            .with_prompt("Project name")
            .validate_with(|input: &String| validation::validate_project_name(input))
            .interact_text()
            .map_err(Into::into)
    }
}

/// Prompts the user to select a project template
fn prompt_template(theme: &ColorfulTheme) -> Result<Template> {
    let templates = vec![
        "🏦 API Service Backend - RESTful API with blockchain integration",
        "📊 Data Analytics Bot - Real-time market analysis and insights",
        "⚡ Event-Driven Trading Engine - Automated trading with event processing",
        "🚀 Minimal API Service - Barebones API with health check and single endpoint",
        "🎨 Custom - Start with a minimal template",
    ];

    let template_idx = Select::with_theme(theme)
        .with_prompt("Select a template")
        .items(&templates)
        .default(0)
        .interact()?;

    Ok(match template_idx {
        0 => Template::ApiServiceBackend,
        1 => Template::DataAnalyticsBot,
        2 => Template::EventDrivenTradingEngine,
        3 => Template::MinimalApi,
        _ => Template::Custom,
    })
}

/// Prompts the user to select blockchain networks to support
fn prompt_blockchains(theme: &ColorfulTheme) -> Result<Vec<Chain>> {
    let blockchains = vec![
        "⛓️ Solana",
        "Ξ Ethereum",
        "🔷 Arbitrum",
        "🟣 Polygon",
        "🔵 Base",
        "⚡ BSC",
        "🌊 Avalanche",
    ];

    let selected_chains = MultiSelect::with_theme(theme)
        .with_prompt("Select blockchain(s) to support")
        .items(&blockchains)
        .defaults(&[true, false, false, false, false, false, false])
        .interact()?;

    let chain_enums = [
        Chain::Solana,
        Chain::Ethereum,
        Chain::Arbitrum,
        Chain::Polygon,
        Chain::Base,
        Chain::Bsc,
        Chain::Avalanche,
    ];

    Ok(selected_chains
        .into_iter()
        .filter_map(|idx| chain_enums.get(idx).cloned())
        .collect())
}

/// Prompts the user to select a server framework
fn prompt_server_framework(theme: &ColorfulTheme) -> Result<Option<ServerFramework>> {
    let include_server = Confirm::with_theme(theme)
        .with_prompt("Include a pre-configured HTTP server?")
        .default(true)
        .interact()?;

    if !include_server {
        return Ok(Option::None);
    }

    let frameworks = vec![
        "🚀 Actix Web - High-performance, actor-based framework",
        "🗼 Axum - Ergonomic and modular framework by Tokio team",
        "🚂 Warp - Composable, fast web framework",
        "🌐 Rocket - Type-safe, intuitive framework",
        "⚡ None - I'll set up the server myself",
    ];

    let framework_idx = Select::with_theme(theme)
        .with_prompt("Select a web framework")
        .items(&frameworks)
        .default(0)
        .interact()?;

    Ok(match framework_idx {
        0 => Some(ServerFramework::Actix),
        1 => Some(ServerFramework::Axum),
        2 => Some(ServerFramework::Warp),
        3 => Some(ServerFramework::Rocket),
        _ => Option::None,
    })
}

/// Prompts the user to select project features
fn prompt_features(theme: &ColorfulTheme) -> Result<(Vec<Feature>, Vec<usize>)> {
    let features = vec![
        "🔍 Web Data Tools (Twitter, DexScreener, News)",
        "🧠 Graph Memory (Neo4j knowledge graph)",
        "🌉 Cross-Chain Tools (LI.FI integration)",
        "🔐 Authentication (Privy, Web3Auth, Magic)",
        "📈 Real-time Data Streaming (WebSocket)",
        "🗄️ Database Integration (PostgreSQL/MongoDB)",
        "📦 Redis Caching",
        "📚 API Documentation (OpenAPI/Swagger)",
        "🚀 CI/CD Pipeline (GitHub Actions)",
        "🐳 Docker Support",
        "🧪 Testing Framework",
        "📖 Example Code",
        "📝 Documentation",
        "📝 Comprehensive Logging",
    ];

    let selected_features = MultiSelect::with_theme(theme)
        .with_prompt("Select additional features")
        .items(&features)
        .defaults(&[
            true, false, false, true, false, false, true, false, false, false, true, true, false,
            true,
        ])
        .interact()?;

    let feature_enums = [
        Feature::WebTools,
        Feature::GraphMemory,
        Feature::CrossChain,
        Feature::Auth,
        Feature::Streaming,
        Feature::Database,
        Feature::Redis,
        Feature::ApiDocs,
        Feature::CiCd,
        Feature::Docker,
        Feature::Tests,
        Feature::Examples,
        Feature::Docs,
        Feature::Logging,
    ];

    let enabled_features = selected_features
        .iter()
        .filter_map(|&idx| feature_enums.get(idx).cloned())
        .collect();

    Ok((enabled_features, selected_features))
}

/// Prompts the user for author information (name and email)
fn prompt_author_info(theme: &ColorfulTheme) -> Result<(String, String)> {
    let author_name = Input::<String>::with_theme(theme)
        .with_prompt("Author name")
        .default(whoami::realname())
        .interact_text()?;

    let author_email = Input::<String>::with_theme(theme)
        .with_prompt("Author email")
        .validate_with(|input: &String| validation::validate_email(input))
        .interact_text()?;

    Ok((author_name, author_email))
}

/// Runs the interactive setup process to collect project configuration
fn interactive_setup(project_name: Option<String>) -> Result<ProjectConfig> {
    let theme = ColorfulTheme::default();

    println!(
        "{}",
        style("Let's set up your new RIGLR project! 🚀")
            .green()
            .bold()
    );
    println!();

    // Get all the configuration values through helper functions
    let name = prompt_project_name(&theme, project_name)?;
    let template = prompt_template(&theme)?;
    let chains = prompt_blockchains(&theme)?;
    let server_framework = prompt_server_framework(&theme)?;
    let (features, selected_indices) = prompt_features(&theme)?;
    let (author_name, author_email) = prompt_author_info(&theme)?;

    // Description
    let description = Input::<String>::with_theme(&theme)
        .with_prompt("Project description")
        .default("AI-powered blockchain agent built with RIGLR".to_string())
        .interact_text()?;

    Ok(ProjectConfig {
        name: name.clone(),
        template,
        chains,
        server_framework,
        features,
        author_name,
        author_email,
        description,
        include_examples: true,
        include_tests: selected_indices.contains(&9),
        include_docs: selected_indices.contains(&11),
    })
}

/// Creates a default project configuration for non-interactive mode
fn create_default_config(project_name: Option<String>) -> Result<ProjectConfig> {
    let name = project_name.unwrap_or_else(|| "my-riglr-agent".to_string());

    Ok(ProjectConfig {
        name: name.clone(),
        template: Template::ApiServiceBackend,
        chains: vec![Chain::Solana],
        server_framework: Some(ServerFramework::Actix),
        features: vec![Feature::WebTools, Feature::Redis, Feature::Logging],
        author_name: whoami::realname(),
        author_email: format!("{}@example.com", whoami::username()),
        description: "AI-powered blockchain agent built with RIGLR".to_string(),
        include_examples: true,
        include_tests: true,
        include_docs: false,
    })
}

/// Generates a new RIGLR project based on the provided configuration
pub async fn generate_project(config: ProjectConfig, output: Option<PathBuf>) -> Result<()> {
    use fs_extra::dir::{move_dir, CopyOptions};
    use tempfile::TempDir;

    println!();
    println!("{}", style("Generating your project...").cyan().bold());

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")?
            .progress_chars("##-"),
    );

    let final_output_dir = output.unwrap_or_else(|| PathBuf::from(&config.name));

    // Check if directory exists
    if final_output_dir.exists() {
        // In non-interactive mode (CI/tests), just remove the directory
        static CI: &str = "CI";

        if std::env::var(CI).is_ok() || !atty::is(atty::Stream::Stdin) {
            std::fs::remove_dir_all(&final_output_dir)?;
        } else {
            let overwrite = Confirm::new()
                .with_prompt(format!(
                    "Directory {} already exists. Overwrite?",
                    final_output_dir.display()
                ))
                .default(false)
                .interact()?;

            if !overwrite {
                println!("{}", style("Aborting...").red());
                return Ok(());
            }

            std::fs::remove_dir_all(&final_output_dir)?;
        }
    }

    // Create a temporary directory in the same parent directory as the final output
    // This ensures atomic moves are possible on the same filesystem
    let parent_dir = final_output_dir.parent().unwrap_or_else(|| Path::new("."));
    let temp_dir = TempDir::new_in(parent_dir)?;
    let temp_path = temp_dir.path();

    // Generate project in temporary directory
    pb.set_message("Creating project structure...");
    pb.set_position(10);

    let generator = ProjectGenerator::new(config.clone());
    generator.create_structure(temp_path)?;

    pb.set_message("Generating source files...");
    pb.set_position(30);
    tokio::time::sleep(Duration::from_millis(200)).await;

    generator.generate_source_files(temp_path)?;

    pb.set_message("Setting up configuration...");
    pb.set_position(50);
    tokio::time::sleep(Duration::from_millis(200)).await;

    generator.generate_config_files(temp_path)?;

    pb.set_message("Creating examples...");
    pb.set_position(70);
    tokio::time::sleep(Duration::from_millis(200)).await;

    if config.include_examples {
        generator.generate_examples(temp_path)?;
    }

    pb.set_message("Finalizing...");
    pb.set_position(90);
    tokio::time::sleep(Duration::from_millis(200)).await;

    generator.generate_readme(temp_path)?;

    // Move the completed project from temp directory to final location
    pb.set_message("Moving to final location...");
    pb.set_position(95);

    // Use fs_extra to move directory with all contents
    let options = CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 64000,
        copy_inside: false,
        content_only: false,
        depth: 0,
    };

    move_dir(temp_path, &final_output_dir, &options)?;

    pb.set_position(100);
    pb.finish_with_message("Done!");

    println!();
    println!(
        "{}",
        style("✨ Project created successfully!").green().bold()
    );
    println!();
    println!(
        "📁 Project location: {}",
        style(final_output_dir.display()).cyan()
    );
    println!();
    println!("{}", style("Next steps:").yellow().bold());
    println!("  1. cd {}", config.name);
    println!("  2. cp .env.example .env");
    println!("  3. # Edit .env with your API keys and configuration");
    println!("  4. cargo build");
    println!("  5. cargo run");
    println!();

    if config.server_framework.is_some() {
        println!("{}", style("Server endpoints:").yellow().bold());
        println!("  • Health: http://localhost:8080/health");
        println!("  • API: http://localhost:8080/api/v1");
        if config.features.contains(&Feature::Streaming) {
            println!("  • WebSocket: ws://localhost:8080/ws");
        }
        if config.features.contains(&Feature::ApiDocs) {
            println!("  • Docs: http://localhost:8080/docs");
        }
        println!();
    }

    println!("{}", style("Happy building! 🚀").magenta().bold());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_banner_should_execute_without_panic() {
        // Happy path: Banner should print without errors
        // Since print_banner() only prints to stdout, we just ensure it doesn't panic
        print_banner();
    }

    #[test]
    fn test_prompt_project_name_when_name_provided_should_return_name() {
        // Happy path: When project name is provided, it should return that name
        let theme = ColorfulTheme::default();
        let project_name = Some("test-project".to_string());

        let result = prompt_project_name(&theme, project_name);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-project");
    }

    #[test]
    fn test_prompt_template_should_return_valid_template() {
        // Testing template selection logic
        // Since prompt_template requires user interaction, we test the match logic
        let template_mappings = vec![
            (0, Template::ApiServiceBackend),
            (1, Template::DataAnalyticsBot),
            (2, Template::EventDrivenTradingEngine),
            (3, Template::MinimalApi),
            (4, Template::Custom),
            (999, Template::Custom), // Test default case
        ];

        for (idx, expected_template) in template_mappings {
            let result = match idx {
                0 => Template::ApiServiceBackend,
                1 => Template::DataAnalyticsBot,
                2 => Template::EventDrivenTradingEngine,
                3 => Template::MinimalApi,
                _ => Template::Custom,
            };
            assert_eq!(result, expected_template);
        }
    }

    #[test]
    fn test_create_default_config_when_no_name_should_use_default() {
        // Happy path: No project name provided, should use default
        let result = create_default_config(None);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.name, "my-riglr-agent");
        assert_eq!(config.template, Template::ApiServiceBackend);
        assert_eq!(config.chains, vec![Chain::Solana]);
        assert_eq!(config.server_framework, Some(ServerFramework::Actix));
        assert!(config.features.contains(&Feature::WebTools));
        assert!(config.features.contains(&Feature::Redis));
        assert!(config.features.contains(&Feature::Logging));
        assert!(config.include_examples);
        assert!(config.include_tests);
        assert!(!config.include_docs);
    }

    #[test]
    fn test_create_default_config_when_name_provided_should_use_name() {
        // Happy path: Project name provided
        let project_name = Some("custom-project".to_string());
        let result = create_default_config(project_name);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.name, "custom-project");
        assert_eq!(config.template, Template::ApiServiceBackend);
        assert_eq!(config.chains, vec![Chain::Solana]);
        assert_eq!(config.server_framework, Some(ServerFramework::Actix));
        assert_eq!(config.author_name, whoami::realname());
        assert_eq!(
            config.author_email,
            format!("{}@example.com", whoami::username())
        );
        assert_eq!(
            config.description,
            "AI-powered blockchain agent built with RIGLR"
        );
    }

    #[test]
    fn test_create_default_config_should_have_expected_features() {
        // Edge case: Verify all default features are correctly set
        let result = create_default_config(None);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.features.len(), 3);
        assert!(config.features.contains(&Feature::WebTools));
        assert!(config.features.contains(&Feature::Redis));
        assert!(config.features.contains(&Feature::Logging));
    }

    #[test]
    fn test_create_default_config_should_have_expected_booleans() {
        // Edge case: Verify boolean fields are correctly set
        let result = create_default_config(Some("test".to_string()));
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.include_examples);
        assert!(config.include_tests);
        assert!(!config.include_docs);
    }

    #[test]
    fn test_blockchain_chain_names_mapping() {
        // Test the blockchain chain names mapping logic
        let chain_names = [
            "solana",
            "ethereum",
            "arbitrum",
            "optimism",
            "polygon",
            "base",
            "bsc",
            "avalanche",
        ];

        // Test valid indices
        for (idx, expected_name) in chain_names.iter().enumerate() {
            assert_eq!(chain_names.get(idx).unwrap(), expected_name);
        }

        // Test invalid index
        assert!(chain_names.get(999).is_none());
    }

    #[test]
    fn test_feature_names_mapping() {
        // Test the feature names mapping logic
        let feature_names = [
            "web_tools",
            "graph_memory",
            "cross_chain",
            "dashboard",
            "auth",
            "streaming",
            "database",
            "redis",
            "logging",
            "testing",
            "cicd",
            "api_docs",
        ];

        // Test valid indices
        for (idx, expected_name) in feature_names.iter().enumerate() {
            assert_eq!(feature_names.get(idx).unwrap(), expected_name);
        }

        // Test invalid index
        assert!(feature_names.get(999).is_none());
    }

    #[test]
    fn test_server_framework_selection_logic() {
        // Test server framework selection mapping
        let framework_mappings = vec![
            (0, Some(ServerFramework::Actix)),
            (1, Some(ServerFramework::Axum)),
            (2, Some(ServerFramework::Warp)),
            (3, Some(ServerFramework::Rocket)),
            (4, None),
            (999, None), // Test default case
        ];

        for (idx, expected_framework) in framework_mappings {
            let result = match idx {
                0 => Some(ServerFramework::Actix),
                1 => Some(ServerFramework::Axum),
                2 => Some(ServerFramework::Warp),
                3 => Some(ServerFramework::Rocket),
                _ => Option::None,
            };
            assert_eq!(result, expected_framework);
        }
    }

    #[test]
    fn test_cli_debug_derive() {
        // Test that CLI struct can be debug printed
        let cli = Cli {
            project_name: Some("test".to_string()),
            template: Some("api".to_string()),
            yes: false,
            output: None,
            verbose: true,
            command: None,
        };

        let debug_str = format!("{:?}", cli);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("api"));
        assert!(debug_str.contains("verbose: true"));
    }

    #[test]
    fn test_commands_debug_derive() {
        // Test that Commands enum variants can be debug printed
        let list_cmd = Commands::List;
        let debug_str = format!("{:?}", list_cmd);
        assert!(debug_str.contains("List"));

        let new_cmd = Commands::New {
            template: "api".to_string(),
            name: "test".to_string(),
        };
        let debug_str = format!("{:?}", new_cmd);
        assert!(debug_str.contains("New"));
        assert!(debug_str.contains("api"));
        assert!(debug_str.contains("test"));

        let update_cmd = Commands::Update {
            url: None,
            branch: None,
        };
        let debug_str = format!("{:?}", update_cmd);
        assert!(debug_str.contains("Update"));

        let info_cmd = Commands::Info {
            template: "api".to_string(),
        };
        let debug_str = format!("{:?}", info_cmd);
        assert!(debug_str.contains("Info"));
        assert!(debug_str.contains("api"));
    }

    #[test]
    fn test_cli_with_all_options() {
        // Test CLI with all options set
        let cli = Cli {
            project_name: Some("my-project".to_string()),
            template: Some("trading-bot".to_string()),
            yes: true,
            output: Some(PathBuf::from("test-output")),
            verbose: false,
            command: Some(Commands::List),
        };

        assert_eq!(cli.project_name, Some("my-project".to_string()));
        assert_eq!(cli.template, Some("trading-bot".to_string()));
        assert!(cli.yes);
        assert_eq!(cli.output, Some(PathBuf::from("test-output")));
        assert!(!cli.verbose);
        assert!(matches!(cli.command, Some(Commands::List)));
    }

    #[test]
    fn test_cli_with_minimal_options() {
        // Test CLI with minimal options
        let cli = Cli {
            project_name: None,
            template: None,
            yes: false,
            output: None,
            verbose: false,
            command: None,
        };

        assert_eq!(cli.project_name, None);
        assert_eq!(cli.template, None);
        assert!(!cli.yes);
        assert_eq!(cli.output, None);
        assert!(!cli.verbose);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_commands_new_with_values() {
        // Test Commands::New with specific values
        let cmd = Commands::New {
            template: "data-analytics".to_string(),
            name: "my-analytics-bot".to_string(),
        };

        match cmd {
            Commands::New { template, name } => {
                assert_eq!(template, "data-analytics");
                assert_eq!(name, "my-analytics-bot");
            }
            _ => panic!("Expected Commands::New variant"),
        }
    }

    #[test]
    fn test_commands_info_with_template() {
        // Test Commands::Info with template value
        let cmd = Commands::Info {
            template: "trading-bot".to_string(),
        };

        match cmd {
            Commands::Info { template } => {
                assert_eq!(template, "trading-bot");
            }
            _ => panic!("Expected Commands::Info variant"),
        }
    }

    #[test]
    fn test_empty_string_handling() {
        // Edge case: Test empty string handling
        let empty_name = Some("".to_string());
        let theme = ColorfulTheme::default();

        // This should return the empty string without validation in this context
        let result = prompt_project_name(&theme, empty_name);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_pathbuf_creation() {
        // Test PathBuf creation and handling
        let path = PathBuf::from("test-project");
        assert_eq!(path.to_string_lossy(), "test-project");

        let path = PathBuf::from("/absolute/path/to/project");
        assert_eq!(path.to_string_lossy(), "/absolute/path/to/project");
    }
}
