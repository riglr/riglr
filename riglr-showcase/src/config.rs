//! Centralized application configuration with fail-fast validation
use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub openai_api_key: String,
    pub redis_url: String,
    pub neo4j_url: String,
    pub twitter_bearer_token: Option<String>,
    pub exa_api_key: Option<String>,
    // New convention-based RPC URLs
    #[serde(rename = "RPC_URL_1")]
    pub ethereum_rpc_url: String,
    #[serde(rename = "RPC_URL_137")]
    pub polygon_rpc_url: String,
    #[serde(rename = "RPC_URL_42161")]
    pub arbitrum_rpc_url: String,
    #[serde(rename = "RPC_URL_8453")]
    pub base_rpc_url: String,
    pub solana_rpc_url: String,
}

impl Config {
    /// Loads configuration from environment variables.
    /// Panics immediately if any required variables are missing.
    pub fn from_env() -> Self {
        match envy::from_env::<Config>() {
            Ok(config) => {
                tracing::info!("✅ All required environment variables loaded successfully");
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

    /// Validates the configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate API keys are not empty
        if self.openai_api_key.is_empty() {
            return Err(ConfigError::EmptyValue("OPENAI_API_KEY".to_string()));
        }
        
        // Validate URLs are valid
        if !self.redis_url.starts_with("redis://") {
            return Err(ConfigError::InvalidFormat("REDIS_URL must start with redis://".to_string()));
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    EmptyValue(String),
    InvalidFormat(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::EmptyValue(key) => write!(f, "Environment variable {} is empty", key),
            ConfigError::InvalidFormat(msg) => write!(f, "Invalid configuration format: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}