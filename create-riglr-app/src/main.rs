//! Enhanced create-riglr-app CLI with interactive scaffolding

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Duration;

mod templates;
mod generator;
mod config;
mod validation;

use crate::config::{ProjectConfig, Template, ServerFramework};
use crate::generator::ProjectGenerator;
use crate::templates::TemplateManager;

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
    Update,
    
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
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    }

    // Print banner
    print_banner();

    match cli.command {
        Some(Commands::List) => {
            list_templates().await?;
        }
        Some(Commands::New { template, name }) => {
            create_from_template(&template, &name, cli.output).await?;
        }
        Some(Commands::Update) => {
            update_templates().await?;
        }
        Some(Commands::Info { template }) => {
            show_template_info(&template).await?;
        }
        None => {
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

fn print_banner() {
    println!("{}", style("
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
    ").cyan().bold());
    println!();
}

fn interactive_setup(project_name: Option<String>) -> Result<ProjectConfig> {
    let theme = ColorfulTheme::default();
    
    println!("{}", style("Let's set up your new RIGLR project! 🚀").green().bold());
    println!();

    // Project name
    let name = if let Some(name) = project_name {
        name
    } else {
        Input::<String>::with_theme(&theme)
            .with_prompt("Project name")
            .validate_with(|input: &String| validation::validate_project_name(input))
            .interact_text()?
    };

    // Template selection
    let templates = vec![
        "🏦 API Service Backend - RESTful API with blockchain integration",
        "📊 Data Analytics Bot - Real-time market analysis and insights",
        "⚡ Event-Driven Trading Engine - Automated trading with event processing",
        "🤖 Trading Bot - Advanced automated trading with risk management",
        "📈 Market Analyst - Comprehensive market analysis and reporting",
        "📰 News Monitor - Real-time news aggregation and sentiment analysis",
        "🔄 DEX Arbitrage Bot - Cross-DEX arbitrage opportunity finder",
        "💼 Portfolio Tracker - Multi-chain portfolio management",
        "🌉 Bridge Monitor - Cross-chain bridge activity tracker",
        "🎯 MEV Protection Agent - MEV protection and sandwich defense",
        "🏛️ DAO Governance Bot - Automated DAO participation",
        "🖼️ NFT Trading Bot - NFT market making and sniping",
        "🌾 Yield Optimizer - Yield farming strategy automation",
        "📱 Social Trading Copier - Copy trading from successful wallets",
        "🎨 Custom - Start with a minimal template",
    ];

    let template_idx = Select::with_theme(&theme)
        .with_prompt("Select a template")
        .items(&templates)
        .default(0)
        .interact()?;

    let template = match template_idx {
        0 => Template::ApiServiceBackend,
        1 => Template::DataAnalyticsBot,
        2 => Template::EventDrivenTradingEngine,
        3 => Template::TradingBot,
        4 => Template::MarketAnalyst,
        5 => Template::NewsMonitor,
        6 => Template::DexArbitrageBot,
        7 => Template::PortfolioTracker,
        8 => Template::BridgeMonitor,
        9 => Template::MevProtectionAgent,
        10 => Template::DaoGovernanceBot,
        11 => Template::NftTradingBot,
        12 => Template::YieldOptimizer,
        13 => Template::SocialTradingCopier,
        14 => Template::Custom,
        _ => Template::Custom,
    };

    // Blockchain selection
    let blockchains = vec![
        "⛓️ Solana",
        "Ξ Ethereum",
        "🔷 Arbitrum",
        "🔴 Optimism",
        "🟣 Polygon",
        "🔵 Base",
        "⚡ BSC",
        "🌊 Avalanche",
    ];

    let selected_chains = MultiSelect::with_theme(&theme)
        .with_prompt("Select blockchain(s) to support")
        .items(&blockchains)
        .defaults(&[true, false, false, false, false, false, false, false])
        .interact()?;

    let mut chains = vec![];
    for &selected_idx in selected_chains.iter() {
        chains.push(match selected_idx {
            0 => "solana".to_string(),
            1 => "ethereum".to_string(),
            2 => "arbitrum".to_string(),
            3 => "optimism".to_string(),
            4 => "polygon".to_string(),
            5 => "base".to_string(),
            6 => "bsc".to_string(),
            7 => "avalanche".to_string(),
            _ => continue,
        });
    }

    // Server framework selection
    let include_server = Confirm::with_theme(&theme)
        .with_prompt("Include a pre-configured HTTP server?")
        .default(true)
        .interact()?;

    let server_framework = if include_server {
        let frameworks = vec![
            "🚀 Actix Web - High-performance, actor-based framework",
            "🗼 Axum - Ergonomic and modular framework by Tokio team",
            "🚂 Warp - Composable, fast web framework",
            "🌐 Rocket - Type-safe, intuitive framework",
            "⚡ None - I'll set up the server myself",
        ];

        let framework_idx = Select::with_theme(&theme)
            .with_prompt("Select a web framework")
            .items(&frameworks)
            .default(0)
            .interact()?;

        match framework_idx {
            0 => Some(ServerFramework::Actix),
            1 => Some(ServerFramework::Axum),
            2 => Some(ServerFramework::Warp),
            3 => Some(ServerFramework::Rocket),
            _ => None,
        }
    } else {
        None
    };

    // Features selection
    let features = vec![
        "🔍 Web Data Tools (Twitter, DexScreener, News)",
        "🧠 Graph Memory (Neo4j knowledge graph)",
        "🌉 Cross-Chain Tools (LI.FI integration)",
        "📊 Analytics Dashboard (Web UI)",
        "🔐 Authentication (Privy, Web3Auth, Magic)",
        "📈 Real-time Data Streaming (WebSocket)",
        "🗄️ Database Integration (PostgreSQL/MongoDB)",
        "📦 Redis Caching",
        "📝 Comprehensive Logging",
        "🧪 Testing Framework",
        "🚀 CI/CD Pipeline (GitHub Actions)",
        "📚 API Documentation (OpenAPI/Swagger)",
    ];

    let selected_features = MultiSelect::with_theme(&theme)
        .with_prompt("Select additional features")
        .items(&features)
        .defaults(&[true, false, false, false, true, false, false, true, true, true, false, false])
        .interact()?;

    let mut enabled_features = vec![];
    for (idx, &selected) in selected_features.iter().enumerate() {
        if selected {
            enabled_features.push(match idx {
                0 => "web_tools",
                1 => "graph_memory",
                2 => "cross_chain",
                3 => "dashboard",
                4 => "auth",
                5 => "streaming",
                6 => "database",
                7 => "redis",
                8 => "logging",
                9 => "testing",
                10 => "cicd",
                11 => "api_docs",
                _ => continue,
            }.to_string());
        }
    }

    // Author information
    let author_name = Input::<String>::with_theme(&theme)
        .with_prompt("Author name")
        .default(whoami::realname())
        .interact_text()?;

    let author_email = Input::<String>::with_theme(&theme)
        .with_prompt("Author email")
        .validate_with(|input: &String| validation::validate_email(input))
        .interact_text()?;

    // Description
    let description = Input::<String>::with_theme(&theme)
        .with_prompt("Project description")
        .default(format!("AI-powered blockchain agent built with RIGLR"))
        .interact_text()?;

    Ok(ProjectConfig {
        name: name.clone(),
        template,
        chains,
        server_framework,
        features: enabled_features,
        author_name,
        author_email,
        description,
        include_examples: true,
        include_tests: selected_features.contains(&9),
        include_docs: selected_features.contains(&11),
    })
}

fn create_default_config(project_name: Option<String>) -> Result<ProjectConfig> {
    let name = project_name.unwrap_or_else(|| "my-riglr-agent".to_string());
    
    Ok(ProjectConfig {
        name: name.clone(),
        template: Template::ApiServiceBackend,
        chains: vec!["solana".to_string()],
        server_framework: Some(ServerFramework::Actix),
        features: vec!["web_tools".to_string(), "redis".to_string(), "logging".to_string()],
        author_name: whoami::realname(),
        author_email: format!("{}@example.com", whoami::username()),
        description: "AI-powered blockchain agent built with RIGLR".to_string(),
        include_examples: true,
        include_tests: true,
        include_docs: false,
    })
}

async fn generate_project(config: ProjectConfig, output: Option<PathBuf>) -> Result<()> {
    println!();
    println!("{}", style("Generating your project...").cyan().bold());
    
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")?
            .progress_chars("##-"),
    );
    
    let output_dir = output.unwrap_or_else(|| PathBuf::from(&config.name));
    
    // Check if directory exists
    if output_dir.exists() {
        let overwrite = Confirm::new()
            .with_prompt(format!("Directory {} already exists. Overwrite?", output_dir.display()))
            .default(false)
            .interact()?;
        
        if !overwrite {
            println!("{}", style("Aborting...").red());
            return Ok(());
        }
        
        std::fs::remove_dir_all(&output_dir)?;
    }
    
    pb.set_message("Creating project structure...");
    pb.set_position(10);
    
    let generator = ProjectGenerator::new(config.clone());
    generator.create_structure(&output_dir)?;
    
    pb.set_message("Generating source files...");
    pb.set_position(30);
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    generator.generate_source_files(&output_dir)?;
    
    pb.set_message("Setting up configuration...");
    pb.set_position(50);
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    generator.generate_config_files(&output_dir)?;
    
    pb.set_message("Creating examples...");
    pb.set_position(70);
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    if config.include_examples {
        generator.generate_examples(&output_dir)?;
    }
    
    pb.set_message("Finalizing...");
    pb.set_position(90);
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    generator.generate_readme(&output_dir)?;
    
    pb.set_position(100);
    pb.finish_with_message("Done!");
    
    println!();
    println!("{}", style("✨ Project created successfully!").green().bold());
    println!();
    println!("📁 Project location: {}", style(output_dir.display()).cyan());
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
        if config.features.contains(&"streaming".to_string()) {
            println!("  • WebSocket: ws://localhost:8080/ws");
        }
        if config.features.contains(&"api_docs".to_string()) {
            println!("  • Docs: http://localhost:8080/docs");
        }
        println!();
    }
    
    println!("{}", style("Happy building! 🚀").magenta().bold());
    
    Ok(())
}

async fn list_templates() -> Result<()> {
    let manager = TemplateManager::new();
    let templates = manager.list_templates()?;
    
    println!("{}", style("Available Templates:").cyan().bold());
    println!();
    
    for template in templates {
        println!("  {} {} - {}", 
            style("•").green(),
            style(&template.name).yellow().bold(),
            template.description
        );
    }
    
    println!();
    println!("Use {} to create a project with a specific template", 
        style("create-riglr-app new <template> <name>").cyan());
    
    Ok(())
}

async fn create_from_template(template: &str, name: &str, output: Option<PathBuf>) -> Result<()> {
    let manager = TemplateManager::new();
    let template_enum = Template::from_str(template)?;
    
    let config = ProjectConfig {
        name: name.to_string(),
        template: template_enum,
        chains: vec!["solana".to_string()],
        server_framework: Some(ServerFramework::Actix),
        features: vec!["web_tools".to_string(), "redis".to_string()],
        author_name: whoami::realname(),
        author_email: format!("{}@example.com", whoami::username()),
        description: format!("{} built with RIGLR", name),
        include_examples: true,
        include_tests: true,
        include_docs: false,
    };
    
    generate_project(config, output).await
}

async fn update_templates() -> Result<()> {
    println!("{}", style("Updating templates...").cyan());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner());
    pb.set_message("Fetching latest templates...");
    
    let manager = TemplateManager::new();
    manager.update_templates().await?;
    
    pb.finish_with_message("Templates updated successfully!");
    
    Ok(())
}

async fn show_template_info(template: &str) -> Result<()> {
    let manager = TemplateManager::new();
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