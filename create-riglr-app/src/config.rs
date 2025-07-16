//! Centralized application configuration.
//! 
//! This module provides a strongly-typed configuration struct that loads
//! and validates all environment variables at startup, implementing a
//! fail-fast pattern for production safety.

use serde::Deserialize;
use std::fmt;
use std::collections::HashMap;

/// Main application configuration loaded from environment variables
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    // AI Provider Configuration
    pub anthropic_api_key: String,
    
    // Database Configuration  
    pub redis_url: String,
    pub neo4j_url: String,
    
    // Blockchain API Configuration
    pub alchemy_api_key: String,
    pub lifi_api_key: String,
    
    // EVM RPC URLs using convention-based naming (RPC_URL_{CHAIN_ID})
    // This HashMap is populated dynamically from environment variables
    #[serde(flatten)]
    pub rpc_urls: HashMap<String, String>,
    
    // Optional Solana Configuration
    #[serde(default)]
    pub solana_rpc_url: Option<String>,
    
    // Optional API Keys
    #[serde(default)]
    pub dexscreener_api_key: Option<String>,
    
    #[serde(default)]
    pub pump_api_key: Option<String>,
}

impl Config {
    /// Loads configuration from environment variables, panicking if any required variables are missing.
    /// This implements a fail-fast pattern to prevent runtime configuration errors.
    pub fn from_env() -> Self {
        match envy::from_env::<Config>() {
            Ok(mut config) => {
                // Extract RPC URLs using the RPC_URL_{CHAIN_ID} convention
                config.extract_rpc_urls();
                
                tracing::info!("✅ All required environment variables loaded successfully");
                // Validate the configuration
                if let Err(e) = config.validate() {
                    eprintln!("❌ Configuration validation failed: {}", e);
                    eprintln!("   Please check your environment variables match the requirements.");
                    eprintln!("   See .env.example for the required format.");
                    std::process::exit(1);
                }
                config
            }
            Err(e) => {
                eprintln!("❌ FATAL: Failed to load required environment configuration:");
                eprintln!("   Error: {}", e);
                eprintln!("   Please ensure all required environment variables are set.");
                eprintln!("   See .env.example for required variables.");
                std::process::exit(1);
            }
        }
    }
    
    /// Extract RPC URLs following the RPC_URL_{CHAIN_ID} convention
    fn extract_rpc_urls(&mut self) {
        for (key, value) in std::env::vars() {
            if key.starts_with("RPC_URL_") {
                if let Some(chain_id) = key.strip_prefix("RPC_URL_") {
                    // Validate that it's a valid chain ID (numeric)
                    if chain_id.parse::<u64>().is_ok() {
                        self.rpc_urls.insert(chain_id.to_string(), value);
                    }
                }
            }
        }
    }
    
    /// Validates the configuration values to ensure they are properly formatted
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate API keys are not empty
        if self.anthropic_api_key.trim().is_empty() {
            return Err(ConfigError::EmptyValue("ANTHROPIC_API_KEY".to_string()));
        }
        
        if self.alchemy_api_key.trim().is_empty() {
            return Err(ConfigError::EmptyValue("ALCHEMY_API_KEY".to_string()));
        }
        
        if self.lifi_api_key.trim().is_empty() {
            return Err(ConfigError::EmptyValue("LIFI_API_KEY".to_string()));
        }
        
        // Validate URL formats
        if !self.redis_url.starts_with("redis://") && !self.redis_url.starts_with("rediss://") {
            return Err(ConfigError::InvalidFormat(
                "REDIS_URL must start with redis:// or rediss://".to_string()
            ));
        }
        
        if !self.neo4j_url.starts_with("neo4j://") && !self.neo4j_url.starts_with("bolt://") {
            return Err(ConfigError::InvalidFormat(
                "NEO4J_URL must start with neo4j:// or bolt://".to_string()
            ));
        }
        
        // Validate RPC URLs
        for (chain_id, url) in &self.rpc_urls {
            if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("wss://") {
                return Err(ConfigError::InvalidFormat(
                    format!("RPC_URL_{} must be a valid RPC URL starting with http://, https://, or wss://", chain_id)
                ));
            }
        }
        
        // Ensure at least one RPC URL is configured
        if self.rpc_urls.is_empty() {
            return Err(ConfigError::InvalidFormat(
                "At least one RPC_URL_{CHAIN_ID} must be configured".to_string()
            ));
        }
        
        // Validate optional Solana RPC URL if provided
        if let Some(ref solana_url) = self.solana_rpc_url {
            if !solana_url.is_empty() && !solana_url.starts_with("http://") && !solana_url.starts_with("https://") {
                return Err(ConfigError::InvalidFormat(
                    "SOLANA_RPC_URL must be a valid URL".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Get RPC URL for a specific chain ID using the convention-based pattern
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
}

/// Configuration validation errors
#[derive(Debug)]
pub enum ConfigError {
    EmptyValue(String),
    InvalidFormat(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::EmptyValue(key) => write!(f, "Environment variable {} cannot be empty", key),
            ConfigError::InvalidFormat(msg) => write!(f, "Invalid configuration format: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    fn set_test_env_vars() {
        env::set_var("ANTHROPIC_API_KEY", "test-anthropic-key");
        env::set_var("REDIS_URL", "redis://localhost:6379");
        env::set_var("NEO4J_URL", "bolt://localhost:7687");
        env::set_var("ALCHEMY_API_KEY", "test-alchemy-key");
        env::set_var("LIFI_API_KEY", "test-lifi-key");
        env::set_var("RPC_URL_1", "https://eth-mainnet.example.com");
        env::set_var("RPC_URL_137", "https://polygon-mainnet.example.com");
        env::set_var("RPC_URL_42161", "https://arbitrum-mainnet.example.com");
        env::set_var("RPC_URL_8453", "https://base-mainnet.example.com");
    }
    
    fn clear_test_env_vars() {
        env::remove_var("ANTHROPIC_API_KEY");
        env::remove_var("REDIS_URL");
        env::remove_var("NEO4J_URL");
        env::remove_var("ALCHEMY_API_KEY");
        env::remove_var("LIFI_API_KEY");
        env::remove_var("RPC_URL_1");
        env::remove_var("RPC_URL_137");
        env::remove_var("RPC_URL_42161");
        env::remove_var("RPC_URL_8453");
    }
    
    #[test]
    fn test_config_validation_success() {
        set_test_env_vars();
        let config = Config::from_env();
        assert!(config.validate().is_ok());
        clear_test_env_vars();
    }
    
    #[test]
    fn test_config_validation_empty_api_key() {
        set_test_env_vars();
        env::set_var("ANTHROPIC_API_KEY", "");
        let config = Config::from_env();
        assert!(matches!(config.validate(), Err(ConfigError::EmptyValue(_))));
        clear_test_env_vars();
    }
    
    #[test]
    fn test_config_validation_invalid_redis_url() {
        set_test_env_vars();
        env::set_var("REDIS_URL", "invalid-url");
        let config = Config::from_env();
        assert!(matches!(config.validate(), Err(ConfigError::InvalidFormat(_))));
        clear_test_env_vars();
    }
    
    #[test]
    fn test_get_rpc_url() {
        set_test_env_vars();
        let config = Config::from_env();
        
        // Test retrieving configured chains
        assert_eq!(config.get_rpc_url(1), Some("https://eth-mainnet.example.com".to_string()));
        assert_eq!(config.get_rpc_url(137), Some("https://polygon-mainnet.example.com".to_string()));
        assert_eq!(config.get_rpc_url(999), None);
        
        // Test get_supported_chains
        let supported = config.get_supported_chains();
        assert!(supported.contains(&1));
        assert!(supported.contains(&137));
        assert!(supported.contains(&42161));
        assert!(supported.contains(&8453));
        
        clear_test_env_vars();
    }
    
    #[test]
    fn test_dynamic_chain_support() {
        set_test_env_vars();
        
        // Add a new chain dynamically
        env::set_var("RPC_URL_10", "https://optimism.example.com");
        
        let config = Config::from_env();
        
        // Should be able to retrieve the dynamically added chain
        assert_eq!(config.get_rpc_url(10), Some("https://optimism.example.com".to_string()));
        
        // Clean up
        env::remove_var("RPC_URL_10");
        clear_test_env_vars();
    }
}