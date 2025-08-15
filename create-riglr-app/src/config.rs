//! Configuration types and template definitions

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Project configuration for scaffolding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub template: Template,
    pub chains: Vec<String>,
    pub server_framework: Option<ServerFramework>,
    pub features: Vec<String>,
    pub author_name: String,
    pub author_email: String,
    pub description: String,
    pub include_examples: bool,
    pub include_tests: bool,
    pub include_docs: bool,
}

/// Available project templates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Template {
    // New templates
    ApiServiceBackend,
    DataAnalyticsBot,
    EventDrivenTradingEngine,
    
    // Existing templates
    TradingBot,
    MarketAnalyst,
    NewsMonitor,
    DexArbitrageBot,
    PortfolioTracker,
    
    // Additional templates
    BridgeMonitor,
    MevProtectionAgent,
    DaoGovernanceBot,
    NftTradingBot,
    YieldOptimizer,
    SocialTradingCopier,
    
    // Basic template
    Custom,
}

impl Template {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "api-service" | "api" => Ok(Template::ApiServiceBackend),
            "analytics" | "data-analytics" => Ok(Template::DataAnalyticsBot),
            "event-driven" | "trading-engine" => Ok(Template::EventDrivenTradingEngine),
            "trading-bot" | "trader" => Ok(Template::TradingBot),
            "market-analyst" | "analyst" => Ok(Template::MarketAnalyst),
            "news-monitor" | "news" => Ok(Template::NewsMonitor),
            "dex-arbitrage" | "arbitrage" => Ok(Template::DexArbitrageBot),
            "portfolio" | "portfolio-tracker" => Ok(Template::PortfolioTracker),
            "bridge-monitor" | "bridge" => Ok(Template::BridgeMonitor),
            "mev-protection" | "mev" => Ok(Template::MevProtectionAgent),
            "dao-governance" | "dao" => Ok(Template::DaoGovernanceBot),
            "nft-trading" | "nft" => Ok(Template::NftTradingBot),
            "yield-optimizer" | "yield" => Ok(Template::YieldOptimizer),
            "social-trading" | "copier" => Ok(Template::SocialTradingCopier),
            "custom" | "minimal" => Ok(Template::Custom),
            _ => Err(anyhow!("Unknown template: {}", s)),
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            Template::ApiServiceBackend => "api-service".to_string(),
            Template::DataAnalyticsBot => "data-analytics".to_string(),
            Template::EventDrivenTradingEngine => "event-driven".to_string(),
            Template::TradingBot => "trading-bot".to_string(),
            Template::MarketAnalyst => "market-analyst".to_string(),
            Template::NewsMonitor => "news-monitor".to_string(),
            Template::DexArbitrageBot => "dex-arbitrage".to_string(),
            Template::PortfolioTracker => "portfolio-tracker".to_string(),
            Template::BridgeMonitor => "bridge-monitor".to_string(),
            Template::MevProtectionAgent => "mev-protection".to_string(),
            Template::DaoGovernanceBot => "dao-governance".to_string(),
            Template::NftTradingBot => "nft-trading".to_string(),
            Template::YieldOptimizer => "yield-optimizer".to_string(),
            Template::SocialTradingCopier => "social-trading".to_string(),
            Template::Custom => "custom".to_string(),
        }
    }
    
    pub fn description(&self) -> &str {
        match self {
            Template::ApiServiceBackend => "RESTful API service with blockchain integration and AI agents",
            Template::DataAnalyticsBot => "Real-time blockchain data analysis and insights generation",
            Template::EventDrivenTradingEngine => "Event-driven automated trading with complex strategies",
            Template::TradingBot => "Advanced trading bot with risk management",
            Template::MarketAnalyst => "Comprehensive market analysis and reporting",
            Template::NewsMonitor => "Real-time news aggregation and sentiment analysis",
            Template::DexArbitrageBot => "Cross-DEX arbitrage opportunity finder",
            Template::PortfolioTracker => "Multi-chain portfolio management and tracking",
            Template::BridgeMonitor => "Cross-chain bridge activity monitoring",
            Template::MevProtectionAgent => "MEV protection and sandwich attack defense",
            Template::DaoGovernanceBot => "Automated DAO participation and voting",
            Template::NftTradingBot => "NFT market making and sniping bot",
            Template::YieldOptimizer => "Yield farming strategy automation",
            Template::SocialTradingCopier => "Copy trading from successful wallets",
            Template::Custom => "Minimal template with basic structure",
        }
    }
    
    pub fn default_features(&self) -> Vec<String> {
        match self {
            Template::ApiServiceBackend => vec![
                "web_tools".to_string(),
                "auth".to_string(),
                "redis".to_string(),
                "database".to_string(),
                "api_docs".to_string(),
                "logging".to_string(),
            ],
            Template::DataAnalyticsBot => vec![
                "web_tools".to_string(),
                "graph_memory".to_string(),
                "streaming".to_string(),
                "database".to_string(),
                "redis".to_string(),
                "logging".to_string(),
            ],
            Template::EventDrivenTradingEngine => vec![
                "web_tools".to_string(),
                "streaming".to_string(),
                "redis".to_string(),
                "database".to_string(),
                "logging".to_string(),
            ],
            Template::TradingBot => vec![
                "web_tools".to_string(),
                "redis".to_string(),
                "logging".to_string(),
            ],
            _ => vec!["logging".to_string()],
        }
    }
}

/// Web server framework options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerFramework {
    Actix,
    Axum,
    Warp,
    Rocket,
}

impl ServerFramework {
    pub fn dependencies(&self) -> Vec<(&str, &str)> {
        match self {
            ServerFramework::Actix => vec![
                ("actix-web", "4"),
                ("actix-web-lab", "0.20"),
                ("actix-cors", "0.7"),
            ],
            ServerFramework::Axum => vec![
                ("axum", "0.7"),
                ("tower", "0.5"),
                ("tower-http", "0.6"),
            ],
            ServerFramework::Warp => vec![
                ("warp", "0.3"),
                ("tokio-stream", "0.1"),
            ],
            ServerFramework::Rocket => vec![
                ("rocket", "0.5"),
                ("rocket_cors", "0.6"),
            ],
        }
    }
}

/// Template metadata for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub features: Vec<String>,
    pub default_chains: Vec<String>,
    pub included_tools: Vec<String>,
}

impl TemplateInfo {
    pub fn from_template(template: &Template) -> Self {
        let (features, chains, tools) = match template {
            Template::ApiServiceBackend => (
                vec![
                    "RESTful API endpoints".to_string(),
                    "OpenAPI documentation".to_string(),
                    "Authentication middleware".to_string(),
                    "Rate limiting".to_string(),
                    "CORS support".to_string(),
                ],
                vec!["solana".to_string(), "ethereum".to_string()],
                vec![
                    "Blockchain query tools".to_string(),
                    "Transaction builders".to_string(),
                    "Wallet management".to_string(),
                ],
            ),
            Template::DataAnalyticsBot => (
                vec![
                    "Real-time data ingestion".to_string(),
                    "Time-series analysis".to_string(),
                    "Pattern recognition".to_string(),
                    "Alert system".to_string(),
                    "Data visualization API".to_string(),
                ],
                vec!["solana".to_string()],
                vec![
                    "DexScreener integration".to_string(),
                    "On-chain data parsers".to_string(),
                    "Statistical analysis tools".to_string(),
                ],
            ),
            Template::EventDrivenTradingEngine => (
                vec![
                    "Event sourcing".to_string(),
                    "CQRS pattern".to_string(),
                    "Strategy backtesting".to_string(),
                    "Risk management".to_string(),
                    "Order management system".to_string(),
                ],
                vec!["solana".to_string(), "ethereum".to_string()],
                vec![
                    "Jupiter integration".to_string(),
                    "Uniswap integration".to_string(),
                    "Price oracles".to_string(),
                    "Position tracking".to_string(),
                ],
            ),
            _ => (vec![], vec![], vec![]),
        };
        
        TemplateInfo {
            name: template.to_string(),
            description: template.description().to_string(),
            features,
            default_chains: chains,
            included_tools: tools,
        }
    }
}