//! Centralized configuration management with validation
//!
//! This module provides a production-ready configuration system that:
//! - Loads from environment variables
//! - Validates all settings at startup
//! - Provides type-safe access to configuration
//! - Supports multiple environments

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;
use riglr_core::config::{RpcConfig, EvmNetworkConfig, SolanaNetworkConfig};

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // AI Provider Configuration
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    
    // Database Configuration
    pub redis_url: String,
    pub neo4j_url: Option<String>,
    
    // Blockchain RPC Configuration (convention-based)
    #[serde(flatten)]
    pub rpc_urls: HashMap<String, String>,
    
    // Solana specific
    pub solana_rpc_url: String,
    
    // API Keys
    pub alchemy_api_key: Option<String>,
    pub infura_api_key: Option<String>,
    pub lifi_api_key: Option<String>,
    
    // Web Data APIs
    pub twitter_bearer_token: Option<String>,
    pub dexscreener_api_key: Option<String>,
    pub exa_api_key: Option<String>,
    
    // Server Configuration
    pub port: u16,
    pub environment: Environment,
    pub log_level: String,
    
    // Feature Flags
    pub enable_trading: bool,
    pub enable_bridging: bool,
    pub enable_social_monitoring: bool,
    pub enable_graph_memory: bool,
    
    // Development Settings
    pub use_testnet: bool,
    
    // Transaction Settings
    pub max_gas_price_gwei: Option<u64>,
    pub priority_fee_gwei: Option<u64>,
    pub slippage_tolerance_percent: Option<f64>,
    
    // Retry Configuration
    pub max_retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub retry_backoff_multiplier: f64,
}

/// Application environment
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok(); // Load .env file if present
        
        // Use envy to deserialize from environment
        let mut config = envy::from_env::<Config>()
            .map_err(|e| ConfigError::LoadError(e.to_string()))?;
        
        // Extract RPC URLs using convention
        config.extract_rpc_urls();
        
        // Validate the configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Extract RPC URLs following the RPC_URL_{CHAIN_ID} convention
    fn extract_rpc_urls(&mut self) {
        for (key, value) in std::env::vars() {
            if key.starts_with("RPC_URL_") {
                if let Some(chain_id) = key.strip_prefix("RPC_URL_") {
                    if chain_id.parse::<u64>().is_ok() {
                        self.rpc_urls.insert(chain_id.to_string(), value);
                    }
                }
            }
        }
    }
    
    /// Get RPC URL for a specific chain ID
    pub fn get_rpc_url(&self, chain_id: u64) -> Option<String> {
        self.rpc_urls.get(&chain_id.to_string()).cloned()
    }
    
    /// Get all configured chain IDs
    pub fn get_supported_chains(&self) -> Vec<u64> {
        self.rpc_urls
            .keys()
            .filter_map(|k| k.parse::<u64>().ok())
            .collect()
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Check required fields based on environment
        if self.environment == Environment::Production {
            // Production requires certain fields
            if self.redis_url.starts_with("redis://localhost") {
                return Err(ConfigError::ValidationError(
                    "Production cannot use localhost Redis".to_string()
                ));
            }
            
            if self.use_testnet {
                return Err(ConfigError::ValidationError(
                    "Production cannot use testnet".to_string()
                ));
            }
        }
        
        // Validate URLs
        if !self.redis_url.starts_with("redis://") && !self.redis_url.starts_with("rediss://") {
            return Err(ConfigError::ValidationError(
                "Invalid Redis URL format".to_string()
            ));
        }
        
        // Validate RPC URLs
        for (chain_id, url) in &self.rpc_urls {
            if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("wss://") {
                return Err(ConfigError::ValidationError(
                    format!("Invalid RPC URL for chain {}: {}", chain_id, url)
                ));
            }
        }
        
        // Validate retry configuration
        if self.max_retry_attempts == 0 {
            return Err(ConfigError::ValidationError(
                "max_retry_attempts must be at least 1".to_string()
            ));
        }
        
        if self.retry_backoff_multiplier <= 1.0 {
            return Err(ConfigError::ValidationError(
                "retry_backoff_multiplier must be greater than 1".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Check if a specific feature is enabled
    pub fn is_feature_enabled(&self, feature: Feature) -> bool {
        match feature {
            Feature::Trading => self.enable_trading,
            Feature::Bridging => self.enable_bridging,
            Feature::SocialMonitoring => self.enable_social_monitoring,
            Feature::GraphMemory => self.enable_graph_memory,
        }
    }

    /// Load chain configurations from TOML file
    pub fn load_chains(&self) -> Result<ChainsConfig, ConfigError> {
        let chains_path = std::env::var("RIGLR_CHAINS_CONFIG")
            .unwrap_or_else(|_| "chains.toml".to_string());
            
        let content = std::fs::read_to_string(&chains_path)
            .map_err(|e| ConfigError::ChainConfigNotFound(chains_path, e))?;
            
        toml::from_str(&content)
            .map_err(ConfigError::InvalidChainConfig)
    }
    
    /// Get chain configuration for a specific chain ID with environment variable overrides
    pub fn get_chain_config(&self, chain_id: u64) -> Result<ChainConfig, ConfigError> {
        let chains = self.load_chains()?;
        
        let mut chain_config = chains.chains
            .values()
            .find(|config| config.id == chain_id)
            .cloned()
            .ok_or(ConfigError::ChainNotSupported(chain_id))?;
            
        // Override with environment variables if present
        if let Ok(router) = std::env::var(format!("ROUTER_{}", chain_id)) {
            chain_config.router = router;
        }
        if let Ok(quoter) = std::env::var(format!("QUOTER_{}", chain_id)) {
            chain_config.quoter = quoter;
        }
        if let Ok(factory) = std::env::var(format!("FACTORY_{}", chain_id)) {
            chain_config.factory = factory;
        }
        
        Ok(chain_config)
    }
}

/// Available features
#[derive(Debug, Clone, Copy)]
pub enum Feature {
    Trading,
    Bridging,
    SocialMonitoring,
    GraphMemory,
}

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    LoadError(String),
    ValidationError(String),
    ChainConfigNotFound(String, std::io::Error),
    InvalidChainConfig(toml::de::Error),
    ChainNotSupported(u64),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::LoadError(msg) => write!(f, "Failed to load configuration: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Configuration validation failed: {}", msg),
            ConfigError::ChainConfigNotFound(path, err) => write!(f, "Chain configuration file not found at {}: {}", path, err),
            ConfigError::InvalidChainConfig(err) => write!(f, "Invalid chain configuration: {}", err),
            ConfigError::ChainNotSupported(chain_id) => write!(f, "Chain {} is not supported", chain_id),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Chain-specific configuration for contract addresses
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainConfig {
    pub id: u64,
    pub name: String,
    pub router: String,
    pub quoter: String,
    pub factory: String,
}

/// Collection of all chain configurations
#[derive(Debug, Clone, Deserialize)]
pub struct ChainsConfig {
    pub chains: HashMap<String, ChainConfig>,
}

// RpcConfig, EvmNetworkConfig, and SolanaNetworkConfig are imported from riglr_core
// Their implementations (Default, methods) are also provided by riglr_core

/// Default configuration for development
impl Default for Config {
    fn default() -> Self {
        Self {
            anthropic_api_key: None,
            openai_api_key: None,
            redis_url: "redis://localhost:6379".to_string(),
            neo4j_url: None,
            rpc_urls: HashMap::new(),
            solana_rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            alchemy_api_key: None,
            infura_api_key: None,
            lifi_api_key: None,
            twitter_bearer_token: None,
            dexscreener_api_key: None,
            exa_api_key: None,
            port: 8080,
            environment: Environment::Development,
            log_level: "info".to_string(),
            enable_trading: true,
            enable_bridging: true,
            enable_social_monitoring: false,
            enable_graph_memory: false,
            use_testnet: false,
            max_gas_price_gwei: Some(100),
            priority_fee_gwei: Some(2),
            slippage_tolerance_percent: Some(0.5),
            max_retry_attempts: 3,
            retry_delay_ms: 1000,
            retry_backoff_multiplier: 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.environment, Environment::Development);
        assert_eq!(config.port, 8080);
        assert!(!config.use_testnet);
    }
    
    #[test]
    fn test_rpc_url_extraction() {
        std::env::set_var("RPC_URL_1", "https://eth.example.com");
        std::env::set_var("RPC_URL_137", "https://polygon.example.com");
        
        let mut config = Config::default();
        config.extract_rpc_urls();
        
        assert_eq!(config.get_rpc_url(1), Some("https://eth.example.com".to_string()));
        assert_eq!(config.get_rpc_url(137), Some("https://polygon.example.com".to_string()));
        
        // Cleanup
        std::env::remove_var("RPC_URL_1");
        std::env::remove_var("RPC_URL_137");
    }
    
    #[test]
    fn test_validation() {
        let mut config = Config { 
            environment: Environment::Production, 
            ..Default::default() 
        };
        
        // Should fail with localhost Redis in production
        assert!(config.validate().is_err());
        
        // Fix Redis URL
        config.redis_url = "redis://prod.example.com:6379".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_chain_configuration_loading() {
        // Create a temporary chains config for testing
        let test_config = r#"
[chains]

[chains.ethereum]
id = 1
name = "Ethereum Mainnet"
router = "0xE592427A0AEce92De3Edee1F18E0157C05861564"
quoter = "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6"
factory = "0x1F98431c8aD98523631AE4a59f267346ea31F984"

[chains.polygon]
id = 137
name = "Polygon"
router = "0xE592427A0AEce92De3Edee1F18E0157C05861564"
quoter = "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6"
factory = "0x1F98431c8aD98523631AE4a59f267346ea31F984"
"#;

        // Write test config to temporary file
        let temp_path = format!("/tmp/test_chains_{}.toml", std::process::id());
        std::fs::write(&temp_path, test_config).unwrap();

        // Set environment variable to use test config
        std::env::set_var("RIGLR_CHAINS_CONFIG", &temp_path);

        let config = Config::default();
        let chains = config.load_chains().unwrap();
        
        // Verify chain configurations are loaded correctly
        assert_eq!(chains.chains.len(), 2);
        
        let eth_config = config.get_chain_config(1).unwrap();
        assert_eq!(eth_config.id, 1);
        assert_eq!(eth_config.name, "Ethereum Mainnet");
        assert_eq!(eth_config.router, "0xE592427A0AEce92De3Edee1F18E0157C05861564");

        let polygon_config = config.get_chain_config(137).unwrap();
        assert_eq!(polygon_config.id, 137);
        assert_eq!(polygon_config.name, "Polygon");

        // Test unsupported chain
        assert!(config.get_chain_config(999).is_err());

        // Cleanup
        std::env::remove_var("RIGLR_CHAINS_CONFIG");
        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_environment_variable_overrides() {
        // Create a temporary chains config for testing
        let test_config = r#"
[chains]

[chains.ethereum]
id = 1
name = "Ethereum Mainnet"
router = "0xE592427A0AEce92De3Edee1F18E0157C05861564"
quoter = "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6"
factory = "0x1F98431c8aD98523631AE4a59f267346ea31F984"
"#;

        // Write test config to temporary file
        let temp_path = "/tmp/test_chains_override.toml";
        std::fs::write(temp_path, test_config).unwrap();

        // Set environment variables
        std::env::set_var("RIGLR_CHAINS_CONFIG", temp_path);
        std::env::set_var("ROUTER_1", "0x1234567890123456789012345678901234567890");
        std::env::set_var("QUOTER_1", "0x0987654321098765432109876543210987654321");

        let config = Config::default();
        let chain_config = config.get_chain_config(1).unwrap();

        // Verify environment variable overrides work
        assert_eq!(chain_config.router, "0x1234567890123456789012345678901234567890");
        assert_eq!(chain_config.quoter, "0x0987654321098765432109876543210987654321");
        assert_eq!(chain_config.factory, "0x1F98431c8aD98523631AE4a59f267346ea31F984"); // Not overridden

        // Cleanup
        std::env::remove_var("RIGLR_CHAINS_CONFIG");
        std::env::remove_var("ROUTER_1");
        std::env::remove_var("QUOTER_1");
        std::fs::remove_file(temp_path).ok();
    }
}