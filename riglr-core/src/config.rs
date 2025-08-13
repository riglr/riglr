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
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::LoadError(msg) => write!(f, "Failed to load configuration: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Configuration validation failed: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Type-safe RPC configuration for blockchain networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub evm_networks: HashMap<String, EvmNetworkConfig>,
    pub solana_networks: HashMap<String, SolanaNetworkConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmNetworkConfig {
    pub name: String,
    pub chain_id: u64,
    pub rpc_url: String,
    pub explorer_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaNetworkConfig {
    pub name: String,
    pub rpc_url: String,
    pub explorer_url: Option<String>,
}

impl Default for RpcConfig {
    fn default() -> Self {
        let mut evm_networks = HashMap::new();
        evm_networks.insert("ethereum".to_string(), EvmNetworkConfig {
            name: "Ethereum Mainnet".to_string(),
            chain_id: 1,
            rpc_url: "https://eth.llamarpc.com".to_string(),
            explorer_url: Some("https://etherscan.io".to_string()),
        });
        evm_networks.insert("polygon".to_string(), EvmNetworkConfig {
            name: "Polygon".to_string(),
            chain_id: 137,
            rpc_url: "https://polygon.llamarpc.com".to_string(),
            explorer_url: Some("https://polygonscan.com".to_string()),
        });
        // Common EVM L2/L1 networks
        evm_networks.insert("arbitrum".to_string(), EvmNetworkConfig {
            name: "Arbitrum One".to_string(),
            chain_id: 42161,
            rpc_url: "https://arbitrum.llamarpc.com".to_string(),
            explorer_url: Some("https://arbiscan.io".to_string()),
        });
        evm_networks.insert("optimism".to_string(), EvmNetworkConfig {
            name: "Optimism".to_string(),
            chain_id: 10,
            rpc_url: "https://optimism.llamarpc.com".to_string(),
            explorer_url: Some("https://optimistic.etherscan.io".to_string()),
        });
        evm_networks.insert("base".to_string(), EvmNetworkConfig {
            name: "Base".to_string(),
            chain_id: 8453,
            rpc_url: "https://base.llamarpc.com".to_string(),
            explorer_url: Some("https://basescan.org".to_string()),
        });
        evm_networks.insert("bsc".to_string(), EvmNetworkConfig {
            name: "BNB Smart Chain".to_string(),
            chain_id: 56,
            rpc_url: "https://bsc.llamarpc.com".to_string(),
            explorer_url: Some("https://bscscan.com".to_string()),
        });
        evm_networks.insert("avalanche".to_string(), EvmNetworkConfig {
            name: "Avalanche C-Chain".to_string(),
            chain_id: 43114,
            rpc_url: "https://avalanche.llamarpc.com".to_string(),
            explorer_url: Some("https://snowtrace.io".to_string()),
        });
        evm_networks.insert("gnosis".to_string(), EvmNetworkConfig {
            name: "Gnosis".to_string(),
            chain_id: 100,
            rpc_url: "https://gnosis.llamarpc.com".to_string(),
            explorer_url: Some("https://gnosisscan.io".to_string()),
        });
        evm_networks.insert("fantom".to_string(), EvmNetworkConfig {
            name: "Fantom".to_string(),
            chain_id: 250,
            rpc_url: "https://fantom.llamarpc.com".to_string(),
            explorer_url: Some("https://ftmscan.com".to_string()),
        });
        evm_networks.insert("linea".to_string(), EvmNetworkConfig {
            name: "Linea".to_string(),
            chain_id: 59144,
            rpc_url: "https://linea.blockpi.network/v1/rpc/public".to_string(),
            explorer_url: Some("https://lineascan.build".to_string()),
        });
        evm_networks.insert("scroll".to_string(), EvmNetworkConfig {
            name: "Scroll".to_string(),
            chain_id: 534352,
            rpc_url: "https://rpc.scroll.io".to_string(),
            explorer_url: Some("https://scrollscan.com".to_string()),
        });
        evm_networks.insert("blast".to_string(), EvmNetworkConfig {
            name: "Blast".to_string(),
            chain_id: 81457,
            rpc_url: "https://blast.blockpi.network/v1/rpc/public".to_string(),
            explorer_url: Some("https://blastscan.io".to_string()),
        });
        evm_networks.insert("mode".to_string(), EvmNetworkConfig {
            name: "Mode".to_string(),
            chain_id: 34443,
            rpc_url: "https://mainnet.mode.network".to_string(),
            explorer_url: Some("https://modescan.io".to_string()),
        });
        evm_networks.insert("mantle".to_string(), EvmNetworkConfig {
            name: "Mantle".to_string(),
            chain_id: 5000,
            rpc_url: "https://mantle.publicnode.com".to_string(),
            explorer_url: Some("https://explorer.mantle.xyz".to_string()),
        });
        evm_networks.insert("celo".to_string(), EvmNetworkConfig {
            name: "Celo".to_string(),
            chain_id: 42220,
            rpc_url: "https://forno.celo.org".to_string(),
            explorer_url: Some("https://celoscan.io".to_string()),
        });
        evm_networks.insert("cronos".to_string(), EvmNetworkConfig {
            name: "Cronos".to_string(),
            chain_id: 25,
            rpc_url: "https://node.cronos.org/rpc".to_string(),
            explorer_url: Some("https://cronoscan.com".to_string()),
        });

        let mut solana_networks = HashMap::new();
        solana_networks.insert("mainnet".to_string(), SolanaNetworkConfig {
            name: "Solana Mainnet".to_string(),
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            explorer_url: Some("https://explorer.solana.com".to_string()),
        });
        solana_networks.insert("devnet".to_string(), SolanaNetworkConfig {
            name: "Solana Devnet".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            explorer_url: Some("https://explorer.solana.com".to_string()),
        });

        RpcConfig {
            evm_networks,
            solana_networks,
        }
    }
}

impl EvmNetworkConfig {
    /// Return the CAIP-2 identifier for this EVM network, e.g. "eip155:1".
    pub fn caip2(&self) -> String {
        format!("eip155:{}", self.chain_id)
    }
}

impl RpcConfig {
    /// Get CAIP-2 for an EVM network by name (case-insensitive key lookup).
    pub fn evm_caip2_for(&self, name: &str) -> Option<String> {
        let key = name.to_lowercase();
        self.evm_networks.get(&key).map(|n| n.caip2())
    }

    /// Add or update an EVM network configuration dynamically.
    pub fn add_evm_network(
        &mut self,
        name: impl Into<String>,
        chain_id: u64,
        rpc_url: impl Into<String>,
        explorer_url: Option<String>,
    ) -> &mut Self {
    let key = name.into().to_lowercase();
    let display_name = key.clone();
    self.evm_networks.insert(key, EvmNetworkConfig { name: display_name, chain_id, rpc_url: rpc_url.into(), explorer_url });
        self
    }

    /// Override or extend EVM networks from env like RPC_URL_{CHAIN_ID}.
    /// If chain_id exists, updates rpc_url; otherwise adds as "chain_{id}".
    pub fn with_env_overrides(mut self) -> Self {
        for (k, v) in std::env::vars() {
            if let Some(cid_str) = k.strip_prefix("RPC_URL_") {
                if let Ok(cid) = cid_str.parse::<u64>() {
                    // Try to find an existing network with same chain_id
                    if let Some((existing_key, _)) = self
                        .evm_networks
                        .iter()
                        .find(|(_, cfg)| cfg.chain_id == cid)
                        .map(|(k, v)| (k.clone(), v.clone()))
                    {
                        if let Some(cfg) = self.evm_networks.get_mut(&existing_key) {
                            cfg.rpc_url = v.clone();
                        }
                    } else {
                        let name = format!("chain_{}", cid);
                        self.add_evm_network(name, cid, v.clone(), None);
                    }
                }
            }
        }
        self
    }
}

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
        let mut config = Config::default();
        config.environment = Environment::Production;
        
        // Should fail with localhost Redis in production
        assert!(config.validate().is_err());
        
        // Fix Redis URL
        config.redis_url = "redis://prod.example.com:6379".to_string();
        assert!(config.validate().is_ok());
    }
}