//! {{project-name}}: {{description}}
//!
//! This is a riglr-powered AI agent that demonstrates how to build sophisticated
//! on-chain applications using the riglr ecosystem.
//!
//! Generated with create-riglr-app - https://github.com/riglr-project/create-riglr-app

use anyhow::Result;
use clap::Parser;
use rig_core::{Agent, Provider};
use std::env;
use tracing::{info, warn, error};

mod config;
use config::Config as EnvConfig;

// Import riglr tools based on template configuration
{% if primary-chain == "solana" or primary-chain == "both" -%}
use riglr_solana_tools::{get_sol_balance, get_spl_token_balance, transfer_sol};
{% endif %}

{% if primary-chain == "ethereum" or primary-chain == "both" -%}
use riglr_evm_tools::{get_eth_balance, get_erc20_balance, transfer_eth};
{% endif %}

{% if include-web-tools -%}
use riglr_web_tools::{
    twitter::search_tweets,
    dexscreener::get_token_info,
    web_search::search_web,
    news::get_crypto_news,
};
{% endif %}

{% if include-graph-memory -%}
use riglr_graph_memory::GraphMemory;
{% endif %}

use riglr_core::{Job, JobQueue, ToolWorker};

/// Command-line configuration for the {{agent-name}} agent
#[derive(Debug, Parser)]
#[command(name = "{{project-name}}")]
#[command(about = "{{description}}")]
#[command(version)]
pub struct CliConfig {
    /// Run in interactive mode
    #[arg(short, long)]
    interactive: bool,

    /// Primary task to execute
    #[arg(short, long)]
    task: Option<String>,

    {% if primary-chain == "solana" or primary-chain == "both" -%}
    /// Solana RPC endpoint
    #[arg(long, default_value = "{{solana-rpc-url}}")]
    solana_rpc: String,
    {% endif %}

    {% if primary-chain == "ethereum" or primary-chain == "both" -%}
    /// Ethereum RPC endpoint
    #[arg(long, default_value = "{{ethereum-rpc-url}}")]
    ethereum_rpc: String,
    {% endif %}

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

/// Main application state
pub struct {{agent-name | snake_case | title_case}}Agent {
    cli_config: CliConfig,
    env_config: EnvConfig,
    agent: Agent<Provider>,
    {% if include-graph-memory -%}
    memory: GraphMemory,
    {% endif %}
    job_queue: Box<dyn JobQueue>,
    tool_worker: ToolWorker,
}

impl {{agent-name | snake_case | title_case}}Agent {
    /// Initialize the agent with all configured tools and services
    pub async fn new(cli_config: CliConfig, env_config: EnvConfig) -> Result<Self> {
        info!("Initializing {{agent-name}} agent...");

        // Initialize the AI provider (you'll need to set up your preferred LLM provider)
        let provider = Provider::new("your-provider-config")?;
        
        // Create the base agent with system prompt
        let mut agent = Agent::builder(&provider)
            .preamble(Self::system_prompt())
            .temperature(0.7);

        // Add blockchain tools
        {% if primary-chain == "solana" or primary-chain == "both" -%}
        agent = agent
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(transfer_sol);
        {% endif %}

        {% if primary-chain == "ethereum" or primary-chain == "both" -%}
        agent = agent
            .tool(get_eth_balance)
            .tool(get_erc20_balance)
            .tool(transfer_eth);
        {% endif %}

        {% if include-web-tools -%}
        // Add web data tools
        agent = agent
            .tool(search_tweets)
            .tool(get_token_info)
            .tool(search_web)
            .tool(get_crypto_news);
        {% endif %}

        let agent = agent.build();

        {% if include-graph-memory -%}
        // Initialize graph memory for knowledge storage
        let memory = GraphMemory::with_defaults("neo4j://localhost:7687").await?;
        {% endif %}

        // Initialize job queue and worker for async task execution
        let job_queue = riglr_core::create_redis_queue(&env_config.redis_url).await?;
        let tool_worker = ToolWorker::new(job_queue.clone()).await?;

        Ok(Self {
            cli_config,
            env_config,
            agent,
            {% if include-graph-memory -%}
            memory,
            {% endif %}
            job_queue,
            tool_worker,
        })
    }

    /// Get the system prompt based on the agent type
    fn system_prompt() -> String {
        match "{{agent-type}}" {
            "trading-bot" => {
                r#"You are {{agent-name}}, an intelligent cryptocurrency trading bot.

Your capabilities include:
{% if primary-chain == "solana" or primary-chain == "both" -%}
- Checking Solana wallet balances and token holdings
- Executing SOL and SPL token transfers
- Analyzing Solana DeFi opportunities
{% endif %}
{% if primary-chain == "ethereum" or primary-chain == "both" -%}
- Checking Ethereum wallet balances and ERC-20 token holdings
- Executing ETH and ERC-20 token transfers
- Analyzing Ethereum DeFi opportunities
{% endif %}
{% if include-web-tools -%}
- Monitoring social media sentiment on Twitter/X
- Getting real-time market data from DexScreener
- Searching the web for relevant information
- Aggregating and analyzing cryptocurrency news
{% endif %}

You should:
1. Always verify wallet balances before suggesting trades
2. Consider market sentiment and news when making decisions
3. Use proper risk management principles
4. Explain your reasoning clearly
5. Never execute trades without explicit user confirmation

Remember: You're handling real money. Be cautious, thorough, and transparent."#
            },
            "market-analyst" => {
                r#"You are {{agent-name}}, a sophisticated cryptocurrency market analyst.

Your capabilities include:
{% if include-web-tools -%}
- Analyzing social media sentiment and trends
- Gathering comprehensive market data from multiple sources  
- Searching for relevant news and developments
- Aggregating information from various crypto news sources
{% endif %}
{% if primary-chain == "solana" or primary-chain == "both" -%}
- Analyzing Solana ecosystem developments
- Monitoring on-chain activity and wallet movements
{% endif %}
{% if primary-chain == "ethereum" or primary-chain == "both" -%}
- Analyzing Ethereum and DeFi protocol developments
- Monitoring on-chain activity and smart contract interactions
{% endif %}

You should:
1. Provide data-driven analysis based on multiple sources
2. Identify trends and patterns in market behavior
3. Explain the reasoning behind your analysis
4. Highlight both opportunities and risks
5. Stay updated on regulatory and technological developments

Focus on providing actionable insights while maintaining analytical objectivity."#
            },
            "news-monitor" => {
                r#"You are {{agent-name}}, a cryptocurrency news monitoring and analysis agent.

Your capabilities include:
{% if include-web-tools -%}
- Monitoring breaking cryptocurrency news from multiple sources
- Analyzing sentiment and market impact of news events
- Tracking social media discussions and trends
- Searching for specific information and developments
{% endif %}

You should:
1. Prioritize breaking news and high-impact events
2. Analyze the potential market implications of news
3. Identify trends and recurring themes
4. Provide context and background information
5. Alert users to important developments quickly

Stay vigilant for market-moving events and provide timely, accurate reporting."#
            },
            _ => {
                r#"You are {{agent-name}}, a versatile cryptocurrency AI agent.

You have access to a comprehensive suite of tools for blockchain interaction,
market analysis, and information gathering. Use these tools wisely to assist
users with their cryptocurrency and DeFi needs.

Always prioritize security, accuracy, and user education in your responses."#
            }
        }.to_string()
    }

    /// Run the agent in interactive mode
    pub async fn run_interactive(&mut self) -> Result<()> {
        info!("Starting {{agent-name}} in interactive mode...");
        info!("Type 'help' for available commands or 'quit' to exit");

        let mut input = String::new();
        loop {
            print!("{{agent-name}}> ");
            std::io::Write::flush(&mut std::io::stdout())?;
            
            input.clear();
            std::io::stdin().read_line(&mut input)?;
            let command = input.trim();

            match command {
                "quit" | "exit" => {
                    info!("Shutting down {{agent-name}}...");
                    break;
                },
                "help" => {
                    self.show_help();
                },
                "status" => {
                    self.show_status().await?;
                },
                {% if agent-type == "trading-bot" -%}
                "portfolio" => {
                    self.check_portfolio().await?;
                },
                {% endif %}
                _ => {
                    // Process user query with the AI agent
                    match self.agent.prompt(command).await {
                        Ok(response) => {
                            println!("🤖 {}", response);
                            
                            {% if include-graph-memory -%}
                            // Store the interaction in memory
                            self.store_interaction(command, &response).await?;
                            {% endif %}
                        },
                        Err(e) => {
                            warn!("Error processing query: {}", e);
                            println!("❌ Sorry, I encountered an error processing your request.");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a specific task
    pub async fn execute_task(&mut self, task: &str) -> Result<()> {
        info!("Executing task: {}", task);

        let response = self.agent.prompt(task).await?;
        println!("Task Result: {}", response);

        {% if include-graph-memory -%}
        // Store the task execution in memory
        self.store_interaction(task, &response).await?;
        {% endif %}

        Ok(())
    }

    /// Show available commands and capabilities
    fn show_help(&self) {
        println!("{{agent-name}} - Available Commands:");
        println!("  help       - Show this help message");
        println!("  status     - Show agent status and configuration");
        println!("  quit/exit  - Shutdown the agent");
        {% if agent-type == "trading-bot" -%}
        println!("  portfolio  - Check current portfolio balances");
        {% endif %}
        println!("");
        println!("You can also ask me questions about:");
        {% if primary-chain == "solana" or primary-chain == "both" -%}
        println!("  • Solana blockchain and wallet operations");
        {% endif %}
        {% if primary-chain == "ethereum" or primary-chain == "both" -%}
        println!("  • Ethereum and EVM blockchain operations");
        {% endif %}
        {% if include-web-tools -%}
        println!("  • Market data and price information");
        println!("  • Social media sentiment analysis");
        println!("  • Cryptocurrency news and developments");
        println!("  • Web search and research tasks");
        {% endif %}
        println!("  • General cryptocurrency and DeFi questions");
    }

    /// Show current agent status
    async fn show_status(&self) -> Result<()> {
        println!("{{agent-name}} Status:");
        println!("  Agent Type: {{agent-type}}");
        println!("  Primary Chain: {{primary-chain}}");
        {% if include-web-tools -%}
        println!("  Web Tools: Enabled");
        {% endif %}
        {% if include-graph-memory -%}
        println!("  Graph Memory: Enabled");
        
        // Show memory statistics
        let stats = self.memory.get_stats().await?;
        println!("  Memory Stats: {} documents, {} entities", 
                 stats.document_count, stats.entity_count);
        {% endif %}
        
        // Check job queue status
        println!("  Job Queue: Connected");
        
        Ok(())
    }

    {% if agent-type == "trading-bot" -%}
    /// Check portfolio balances
    async fn check_portfolio(&mut self) -> Result<()> {
        println!("Checking portfolio balances...");
        
        // This would typically check configured wallet addresses
        let query = "Check my current portfolio balances and provide a summary";
        let response = self.agent.prompt(query).await?;
        println!("Portfolio Summary:\n{}", response);
        
        Ok(())
    }
    {% endif %}

    {% if include-graph-memory -%}
    /// Store interaction in graph memory for future reference
    async fn store_interaction(&self, query: &str, response: &str) -> Result<()> {
        use riglr_graph_memory::{RawTextDocument, DocumentSource, DocumentMetadata};
        
        let doc = RawTextDocument {
            id: uuid::Uuid::new_v4().to_string(),
            content: format!("Query: {}\nResponse: {}", query, response),
            metadata: Some(DocumentMetadata {
                title: Some("Agent Interaction".to_string()),
                source_type: Some("conversation".to_string()),
                ..Default::default()
            }),
            embedding: None,
            created_at: chrono::Utc::now(),
            source: DocumentSource::User,
        };

        self.memory.add_documents(vec![doc]).await?;
        Ok(())
    }
    {% endif %}
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Parse command line arguments
    let cli_config = CliConfig::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(&cli_config.log_level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting {{project-name}} v{}", env!("CARGO_PKG_VERSION"));

    // Load and validate environment configuration
    info!("Loading and validating environment configuration...");
    let env_config = EnvConfig::from_env();
    info!("✅ Configuration validated successfully");

    // Initialize the agent
    let mut agent = {{agent-name | snake_case | title_case}}Agent::new(cli_config, env_config).await?;

    // Run the agent
    if agent.cli_config.interactive {
        agent.run_interactive().await?;
    } else if let Some(task) = &agent.cli_config.task {
        agent.execute_task(task).await?;
    } else {
        // Default behavior
        println!("{{agent-name}} is ready!");
        println!("Use --interactive for interactive mode or --task 'your task' to execute a specific task");
        println!("Use --help for more options");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_initialization() {
        let config = Config {
            interactive: false,
            task: None,
            {% if primary-chain == "solana" or primary-chain == "both" -%}
            solana_rpc: "https://api.devnet.solana.com".to_string(),
            {% endif %}
            {% if primary-chain == "ethereum" or primary-chain == "both" -%}
            ethereum_rpc: "https://eth-sepolia.g.alchemy.com/v2/test".to_string(),
            {% endif %}
            log_level: "info".to_string(),
        };

        // This test would need proper API keys to fully initialize
        // For now, just test the configuration parsing
        assert_eq!(config.log_level, "info");
        {% if primary-chain == "solana" or primary-chain == "both" -%}
        assert!(config.solana_rpc.contains("solana.com"));
        {% endif %}
    }

    #[test]
    fn test_system_prompt_generation() {
        let prompt = {{agent-name | snake_case | title_case}}Agent::system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("{{agent-name}}"));
    }
}