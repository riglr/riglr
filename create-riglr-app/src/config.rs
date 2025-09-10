//! Configuration types and template definitions

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Project configuration for scaffolding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// The name of the project
    pub name: String,
    /// The template to use for project scaffolding
    pub template: Template,
    /// List of blockchain networks to support
    pub chains: Vec<Chain>,
    /// Optional web server framework to include
    pub server_framework: Option<ServerFramework>,
    /// List of features to enable in the project
    pub features: Vec<Feature>,
    /// Author's name for project metadata
    pub author_name: String,
    /// Author's email for project metadata
    pub author_email: String,
    /// Project description
    pub description: String,
    /// Whether to include example code
    pub include_examples: bool,
    /// Whether to include test files
    pub include_tests: bool,
    /// Whether to include documentation
    pub include_docs: bool,
}

/// Available project templates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Template {
    /// RESTful API service template with blockchain integration
    ApiServiceBackend,
    /// Real-time blockchain data analysis template
    DataAnalyticsBot,
    /// Event-driven automated trading engine template
    EventDrivenTradingEngine,
    /// Minimal API service with health check and single endpoint
    MinimalApi,
    /// Minimal custom template
    Custom,
}

impl Template {
    /// Parse a template from a string identifier
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "api-service" | "api" => Ok(Template::ApiServiceBackend),
            "analytics" | "data-analytics" => Ok(Template::DataAnalyticsBot),
            "event-driven" | "trading-engine" => Ok(Template::EventDrivenTradingEngine),
            "minimal-api" | "minimal" => Ok(Template::MinimalApi),
            "custom" => Ok(Template::Custom),
            _ => Err(anyhow!("Unknown template: {}", s)),
        }
    }

    /// Get the description for this template
    pub fn description(&self) -> &str {
        match self {
            Template::ApiServiceBackend => {
                "RESTful API service with blockchain integration and AI agents"
            }
            Template::DataAnalyticsBot => {
                "Real-time blockchain data analysis and insights generation"
            }
            Template::EventDrivenTradingEngine => {
                "Event-driven automated trading with complex strategies"
            }
            Template::MinimalApi => {
                "A barebones API service with a health check and a single agent endpoint"
            }
            Template::Custom => "Minimal template with basic structure",
        }
    }

    #[allow(dead_code)]
    /// Get the default features for this template
    pub fn default_features(&self) -> Vec<Feature> {
        match self {
            Template::ApiServiceBackend => vec![
                Feature::WebTools,
                Feature::Auth,
                Feature::Redis,
                Feature::Database,
                Feature::ApiDocs,
                Feature::Logging,
            ],
            Template::DataAnalyticsBot => vec![
                Feature::WebTools,
                Feature::GraphMemory,
                Feature::Streaming,
                Feature::Database,
                Feature::Redis,
                Feature::Logging,
            ],
            Template::EventDrivenTradingEngine => vec![
                Feature::WebTools,
                Feature::Streaming,
                Feature::Redis,
                Feature::Database,
                Feature::Logging,
            ],
            Template::MinimalApi => vec![Feature::Logging],
            Template::Custom => vec![Feature::Logging],
        }
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Template::ApiServiceBackend => "api-service",
            Template::DataAnalyticsBot => "data-analytics",
            Template::EventDrivenTradingEngine => "event-driven",
            Template::MinimalApi => "minimal-api",
            Template::Custom => "custom",
        };
        write!(f, "{}", s)
    }
}

/// Web server framework options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerFramework {
    /// Actix Web framework
    Actix,
    /// Axum framework
    Axum,
    /// Warp framework
    Warp,
    /// Rocket framework
    Rocket,
}

impl ServerFramework {
    #[allow(dead_code)]
    /// Get the dependencies for this server framework
    pub fn dependencies(&self) -> Vec<(&str, &str)> {
        match self {
            ServerFramework::Actix => vec![
                ("actix-web", "4"),
                ("actix-web-lab", "0.20"),
                ("actix-cors", "0.7"),
            ],
            ServerFramework::Axum => vec![("axum", "0.7"), ("tower", "0.5"), ("tower-http", "0.6")],
            ServerFramework::Warp => vec![("warp", "0.3"), ("tokio-stream", "0.1")],
            ServerFramework::Rocket => vec![("rocket", "0.5"), ("rocket_cors", "0.6")],
        }
    }
}

/// Supported blockchain networks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Chain {
    /// Solana blockchain
    Solana,
    /// Ethereum blockchain
    Ethereum,
    /// Polygon blockchain
    Polygon,
    /// Arbitrum blockchain
    Arbitrum,
    /// Base blockchain
    Base,
    /// Binance Smart Chain
    Bsc,
    /// Avalanche blockchain
    Avalanche,
}

impl Chain {
    /// Parse a chain from a string identifier
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "solana" | "sol" => Ok(Chain::Solana),
            "ethereum" | "eth" => Ok(Chain::Ethereum),
            "polygon" | "matic" => Ok(Chain::Polygon),
            "arbitrum" | "arb" => Ok(Chain::Arbitrum),
            "base" => Ok(Chain::Base),
            "bsc" | "binance" => Ok(Chain::Bsc),
            "avalanche" | "avax" => Ok(Chain::Avalanche),
            _ => Err(anyhow!("Unknown chain: {}", s)),
        }
    }

    /// Get the string representation of the chain
    pub fn as_str(&self) -> &str {
        match self {
            Chain::Solana => "solana",
            Chain::Ethereum => "ethereum",
            Chain::Polygon => "polygon",
            Chain::Arbitrum => "arbitrum",
            Chain::Base => "base",
            Chain::Bsc => "bsc",
            Chain::Avalanche => "avalanche",
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Available features for projects
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    /// Web interaction tools
    WebTools,
    /// Graph-based memory system
    GraphMemory,
    /// Cross-chain functionality
    CrossChain,
    /// Authentication system
    Auth,
    /// Real-time streaming capabilities
    Streaming,
    /// Database integration
    Database,
    /// Redis caching system
    Redis,
    /// API documentation generation
    ApiDocs,
    /// Continuous integration/deployment
    CiCd,
    /// Docker containerization
    Docker,
    /// Test framework integration
    Tests,
    /// Example code generation
    Examples,
    /// Documentation generation
    Docs,
    /// Logging and observability
    Logging,
}

impl Feature {
    /// Parse a feature from a string identifier
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().replace("_", "-").as_str() {
            "web-tools" | "web" => Ok(Feature::WebTools),
            "graph-memory" | "graph" => Ok(Feature::GraphMemory),
            "cross-chain" | "crosschain" => Ok(Feature::CrossChain),
            "auth" | "authentication" => Ok(Feature::Auth),
            "streaming" | "stream" => Ok(Feature::Streaming),
            "database" | "db" => Ok(Feature::Database),
            "redis" | "cache" => Ok(Feature::Redis),
            "api-docs" | "apidocs" | "openapi" => Ok(Feature::ApiDocs),
            "ci-cd" | "cicd" | "ci" => Ok(Feature::CiCd),
            "docker" | "container" => Ok(Feature::Docker),
            "tests" | "test" => Ok(Feature::Tests),
            "examples" | "example" => Ok(Feature::Examples),
            "docs" | "documentation" => Ok(Feature::Docs),
            "logging" | "logs" => Ok(Feature::Logging),
            _ => Err(anyhow!("Unknown feature: {}", s)),
        }
    }

    /// Get the string representation of the feature
    pub fn as_str(&self) -> &str {
        match self {
            Feature::WebTools => "web_tools",
            Feature::GraphMemory => "graph_memory",
            Feature::CrossChain => "cross_chain",
            Feature::Auth => "auth",
            Feature::Streaming => "streaming",
            Feature::Database => "database",
            Feature::Redis => "redis",
            Feature::ApiDocs => "api_docs",
            Feature::CiCd => "cicd",
            Feature::Docker => "docker",
            Feature::Tests => "tests",
            Feature::Examples => "examples",
            Feature::Docs => "docs",
            Feature::Logging => "logging",
        }
    }
}

impl fmt::Display for Feature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Template metadata for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// List of template features
    pub features: Vec<String>,
    /// Default blockchain networks for this template
    pub default_chains: Vec<String>,
    /// Tools included with this template
    pub included_tools: Vec<String>,
}

impl TemplateInfo {
    /// Create template info from a template
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_from_str_when_api_service_aliases_should_return_api_service_backend() {
        assert_eq!(
            Template::parse("api-service").unwrap(),
            Template::ApiServiceBackend
        );
        assert_eq!(Template::parse("api").unwrap(), Template::ApiServiceBackend);
    }

    #[test]
    fn test_template_from_str_when_analytics_aliases_should_return_data_analytics_bot() {
        assert_eq!(
            Template::parse("analytics").unwrap(),
            Template::DataAnalyticsBot
        );
        assert_eq!(
            Template::parse("data-analytics").unwrap(),
            Template::DataAnalyticsBot
        );
    }

    #[test]
    fn test_template_from_str_when_event_driven_aliases_should_return_event_driven_trading_engine()
    {
        assert_eq!(
            Template::parse("event-driven").unwrap(),
            Template::EventDrivenTradingEngine
        );
        assert_eq!(
            Template::parse("trading-engine").unwrap(),
            Template::EventDrivenTradingEngine
        );
    }

    #[test]
    fn test_template_from_str_when_custom_should_return_custom() {
        assert_eq!(Template::parse("custom").unwrap(), Template::Custom);
    }

    #[test]
    fn test_template_from_str_when_minimal_api_aliases_should_return_minimal_api() {
        assert_eq!(
            Template::parse("minimal-api").unwrap(),
            Template::MinimalApi
        );
        assert_eq!(Template::parse("minimal").unwrap(), Template::MinimalApi);
    }

    #[test]
    fn test_template_from_str_when_case_insensitive_should_work() {
        assert_eq!(
            Template::parse("API-SERVICE").unwrap(),
            Template::ApiServiceBackend
        );
        assert_eq!(Template::parse("Api").unwrap(), Template::ApiServiceBackend);
        assert_eq!(Template::parse("CUSTOM").unwrap(), Template::Custom);
    }

    #[test]
    fn test_template_from_str_when_unknown_template_should_return_error() {
        let result = Template::parse("unknown-template");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unknown template: unknown-template"
        );
    }

    #[test]
    fn test_template_from_str_when_empty_string_should_return_error() {
        let result = Template::parse("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Unknown template: ");
    }

    #[test]
    fn test_template_description_for_all_variants() {
        assert_eq!(
            Template::ApiServiceBackend.description(),
            "RESTful API service with blockchain integration and AI agents"
        );
        assert_eq!(
            Template::DataAnalyticsBot.description(),
            "Real-time blockchain data analysis and insights generation"
        );
        assert_eq!(
            Template::EventDrivenTradingEngine.description(),
            "Event-driven automated trading with complex strategies"
        );
        assert_eq!(
            Template::MinimalApi.description(),
            "A barebones API service with a health check and a single agent endpoint"
        );
        assert_eq!(
            Template::Custom.description(),
            "Minimal template with basic structure"
        );
    }

    #[test]
    fn test_template_default_features_api_service_backend() {
        let features = Template::ApiServiceBackend.default_features();
        let expected = vec![
            Feature::WebTools,
            Feature::Auth,
            Feature::Redis,
            Feature::Database,
            Feature::ApiDocs,
            Feature::Logging,
        ];
        assert_eq!(features, expected);
    }

    #[test]
    fn test_template_default_features_data_analytics_bot() {
        let features = Template::DataAnalyticsBot.default_features();
        let expected = vec![
            Feature::WebTools,
            Feature::GraphMemory,
            Feature::Streaming,
            Feature::Database,
            Feature::Redis,
            Feature::Logging,
        ];
        assert_eq!(features, expected);
    }

    #[test]
    fn test_template_default_features_event_driven_trading_engine() {
        let features = Template::EventDrivenTradingEngine.default_features();
        let expected = vec![
            Feature::WebTools,
            Feature::Streaming,
            Feature::Redis,
            Feature::Database,
            Feature::Logging,
        ];
        assert_eq!(features, expected);
    }

    #[test]
    fn test_template_default_features_minimal_api() {
        let features = Template::MinimalApi.default_features();
        let expected = vec![Feature::Logging];
        assert_eq!(features, expected);
    }

    #[test]
    fn test_template_default_features_custom() {
        let features = Template::Custom.default_features();
        let expected = vec![Feature::Logging];
        assert_eq!(features, expected);
    }

    #[test]
    fn test_template_display_for_all_variants() {
        assert_eq!(Template::ApiServiceBackend.to_string(), "api-service");
        assert_eq!(Template::DataAnalyticsBot.to_string(), "data-analytics");
        assert_eq!(
            Template::EventDrivenTradingEngine.to_string(),
            "event-driven"
        );
        assert_eq!(Template::MinimalApi.to_string(), "minimal-api");
        assert_eq!(Template::Custom.to_string(), "custom");
    }

    #[test]
    fn test_server_framework_dependencies_actix() {
        let deps = ServerFramework::Actix.dependencies();
        let expected = vec![
            ("actix-web", "4"),
            ("actix-web-lab", "0.20"),
            ("actix-cors", "0.7"),
        ];
        assert_eq!(deps, expected);
    }

    #[test]
    fn test_server_framework_dependencies_axum() {
        let deps = ServerFramework::Axum.dependencies();
        let expected = vec![("axum", "0.7"), ("tower", "0.5"), ("tower-http", "0.6")];
        assert_eq!(deps, expected);
    }

    #[test]
    fn test_server_framework_dependencies_warp() {
        let deps = ServerFramework::Warp.dependencies();
        let expected = vec![("warp", "0.3"), ("tokio-stream", "0.1")];
        assert_eq!(deps, expected);
    }

    #[test]
    fn test_server_framework_dependencies_rocket() {
        let deps = ServerFramework::Rocket.dependencies();
        let expected = vec![("rocket", "0.5"), ("rocket_cors", "0.6")];
        assert_eq!(deps, expected);
    }

    #[test]
    fn test_template_info_from_template_api_service_backend() {
        let template = Template::ApiServiceBackend;
        let info = TemplateInfo::from_template(&template);

        assert_eq!(info.name, "api-service");
        assert_eq!(
            info.description,
            "RESTful API service with blockchain integration and AI agents"
        );
        assert_eq!(
            info.features,
            vec![
                "RESTful API endpoints".to_string(),
                "OpenAPI documentation".to_string(),
                "Authentication middleware".to_string(),
                "Rate limiting".to_string(),
                "CORS support".to_string(),
            ]
        );
        assert_eq!(
            info.default_chains,
            vec!["solana".to_string(), "ethereum".to_string()]
        );
        assert_eq!(
            info.included_tools,
            vec![
                "Blockchain query tools".to_string(),
                "Transaction builders".to_string(),
                "Wallet management".to_string(),
            ]
        );
    }

    #[test]
    fn test_template_info_from_template_data_analytics_bot() {
        let template = Template::DataAnalyticsBot;
        let info = TemplateInfo::from_template(&template);

        assert_eq!(info.name, "data-analytics");
        assert_eq!(
            info.description,
            "Real-time blockchain data analysis and insights generation"
        );
        assert_eq!(
            info.features,
            vec![
                "Real-time data ingestion".to_string(),
                "Time-series analysis".to_string(),
                "Pattern recognition".to_string(),
                "Alert system".to_string(),
                "Data visualization API".to_string(),
            ]
        );
        assert_eq!(info.default_chains, vec!["solana".to_string()]);
        assert_eq!(
            info.included_tools,
            vec![
                "DexScreener integration".to_string(),
                "On-chain data parsers".to_string(),
                "Statistical analysis tools".to_string(),
            ]
        );
    }

    #[test]
    fn test_template_info_from_template_event_driven_trading_engine() {
        let template = Template::EventDrivenTradingEngine;
        let info = TemplateInfo::from_template(&template);

        assert_eq!(info.name, "event-driven");
        assert_eq!(
            info.description,
            "Event-driven automated trading with complex strategies"
        );
        assert_eq!(
            info.features,
            vec![
                "Event sourcing".to_string(),
                "CQRS pattern".to_string(),
                "Strategy backtesting".to_string(),
                "Risk management".to_string(),
                "Order management system".to_string(),
            ]
        );
        assert_eq!(
            info.default_chains,
            vec!["solana".to_string(), "ethereum".to_string()]
        );
        assert_eq!(
            info.included_tools,
            vec![
                "Jupiter integration".to_string(),
                "Uniswap integration".to_string(),
                "Price oracles".to_string(),
                "Position tracking".to_string(),
            ]
        );
    }

    #[test]
    fn test_project_config_serialization() {
        let config = ProjectConfig {
            name: "test-project".to_string(),
            template: Template::ApiServiceBackend,
            chains: vec![Chain::Solana, Chain::Ethereum],
            server_framework: Some(ServerFramework::Axum),
            features: vec![Feature::Auth, Feature::Redis],
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            description: "A test project".to_string(),
            include_examples: true,
            include_tests: true,
            include_docs: false,
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: ProjectConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.template, deserialized.template);
        assert_eq!(config.chains, deserialized.chains);
        assert_eq!(config.server_framework, deserialized.server_framework);
        assert_eq!(config.features, deserialized.features);
        assert_eq!(config.author_name, deserialized.author_name);
        assert_eq!(config.author_email, deserialized.author_email);
        assert_eq!(config.description, deserialized.description);
        assert_eq!(config.include_examples, deserialized.include_examples);
        assert_eq!(config.include_tests, deserialized.include_tests);
        assert_eq!(config.include_docs, deserialized.include_docs);
    }

    #[test]
    fn test_template_serialization() {
        let template = Template::ApiServiceBackend;
        let serialized = serde_json::to_string(&template).unwrap();
        let deserialized: Template = serde_json::from_str(&serialized).unwrap();
        assert_eq!(template, deserialized);
    }

    #[test]
    fn test_server_framework_serialization() {
        let framework = ServerFramework::Axum;
        let serialized = serde_json::to_string(&framework).unwrap();
        let deserialized: ServerFramework = serde_json::from_str(&serialized).unwrap();
        assert_eq!(framework, deserialized);
    }

    #[test]
    fn test_template_info_serialization() {
        let info = TemplateInfo {
            name: "test".to_string(),
            description: "Test template".to_string(),
            features: vec!["feature1".to_string()],
            default_chains: vec!["solana".to_string()],
            included_tools: vec!["tool1".to_string()],
        };

        let serialized = serde_json::to_string(&info).unwrap();
        let deserialized: TemplateInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.description, deserialized.description);
        assert_eq!(info.features, deserialized.features);
        assert_eq!(info.default_chains, deserialized.default_chains);
        assert_eq!(info.included_tools, deserialized.included_tools);
    }

    #[test]
    fn test_template_partial_eq() {
        assert_eq!(Template::ApiServiceBackend, Template::ApiServiceBackend);
        assert_ne!(Template::ApiServiceBackend, Template::DataAnalyticsBot);
    }

    #[test]
    fn test_server_framework_partial_eq() {
        assert_eq!(ServerFramework::Axum, ServerFramework::Axum);
        assert_ne!(ServerFramework::Axum, ServerFramework::Actix);
    }

    #[test]
    fn test_template_debug_fmt() {
        let template = Template::ApiServiceBackend;
        let debug_str = format!("{:?}", template);
        assert!(debug_str.contains("ApiServiceBackend"));
    }

    #[test]
    fn test_server_framework_debug_fmt() {
        let framework = ServerFramework::Axum;
        let debug_str = format!("{:?}", framework);
        assert!(debug_str.contains("Axum"));
    }

    #[test]
    fn test_project_config_debug_fmt() {
        let config = ProjectConfig {
            name: "test".to_string(),
            template: Template::Custom,
            chains: vec![],
            server_framework: None,
            features: vec![],
            author_name: "Author".to_string(),
            author_email: "email@test.com".to_string(),
            description: "Description".to_string(),
            include_examples: false,
            include_tests: false,
            include_docs: false,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ProjectConfig"));
    }

    #[test]
    fn test_template_info_debug_fmt() {
        let info = TemplateInfo {
            name: "test".to_string(),
            description: "desc".to_string(),
            features: vec![],
            default_chains: vec![],
            included_tools: vec![],
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("TemplateInfo"));
    }

    #[test]
    fn test_template_clone() {
        let template = Template::ApiServiceBackend;
        let cloned = template.clone();
        assert_eq!(template, cloned);
    }

    #[test]
    fn test_server_framework_clone() {
        let framework = ServerFramework::Axum;
        let cloned = framework.clone();
        assert_eq!(framework, cloned);
    }

    #[test]
    fn test_project_config_clone() {
        let config = ProjectConfig {
            name: "test".to_string(),
            template: Template::Custom,
            chains: vec![Chain::Solana],
            server_framework: Some(ServerFramework::Axum),
            features: vec![Feature::Auth],
            author_name: "Author".to_string(),
            author_email: "email@test.com".to_string(),
            description: "Description".to_string(),
            include_examples: true,
            include_tests: true,
            include_docs: true,
        };

        let cloned = config.clone();
        assert_eq!(config.name, cloned.name);
        assert_eq!(config.template, cloned.template);
        assert_eq!(config.chains, cloned.chains);
        assert_eq!(config.server_framework, cloned.server_framework);
    }

    #[test]
    fn test_template_info_clone() {
        let info = TemplateInfo {
            name: "test".to_string(),
            description: "desc".to_string(),
            features: vec!["feature".to_string()],
            default_chains: vec!["solana".to_string()],
            included_tools: vec!["tool".to_string()],
        };

        let cloned = info.clone();
        assert_eq!(info.name, cloned.name);
        assert_eq!(info.description, cloned.description);
        assert_eq!(info.features, cloned.features);
    }
}
