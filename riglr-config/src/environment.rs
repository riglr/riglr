//! Environment variable utilities

use std::env;
use crate::{ConfigError, ConfigResult};

/// Type alias for custom environment source
type CustomEnvSource = dyn Fn(&str) -> Option<String>;

/// Source for loading environment variables
pub enum EnvironmentSource {
    /// Load from system environment
    System,
    /// Load from .env file
    DotEnv(String),
    /// Load from custom source
    Custom(Box<CustomEnvSource>),
}

impl EnvironmentSource {
    /// Get an environment variable
    pub fn get(&self, key: &str) -> Option<String> {
        match self {
            EnvironmentSource::System => env::var(key).ok(),
            EnvironmentSource::DotEnv(_path) => {
                // This would need dotenv parsing logic
                // For now, fallback to system
                env::var(key).ok()
            }
            EnvironmentSource::Custom(f) => f(key),
        }
    }
    
    /// Get a required environment variable
    pub fn require(&self, key: &str) -> ConfigResult<String> {
        self.get(key)
            .ok_or_else(|| ConfigError::MissingEnvVar(key.to_string()))
    }
    
    /// Get an optional environment variable with default
    pub fn get_or(&self, key: &str, default: String) -> String {
        self.get(key).unwrap_or(default)
    }
    
    /// Check if an environment variable exists
    pub fn exists(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

/// Helper to extract values by prefix
#[allow(dead_code)]
pub fn extract_by_prefix(prefix: &str) -> Vec<(String, String)> {
    env::vars()
        .filter(|(k, _)| k.starts_with(prefix))
        .collect()
}

/// Helper to extract and parse chain IDs from RPC_URL_{CHAIN_ID} pattern
#[allow(dead_code)]
pub fn extract_chain_rpc_urls() -> Vec<(u64, String)> {
    extract_by_prefix("RPC_URL_")
        .into_iter()
        .filter_map(|(key, value)| {
            key.strip_prefix("RPC_URL_")
                .and_then(|chain_id| chain_id.parse::<u64>().ok())
                .map(|id| (id, value))
        })
        .collect()
}

/// Helper to extract and parse contract addresses
#[allow(dead_code)]
pub fn extract_contract_overrides(chain_id: u64) -> Vec<(String, String)> {
    let prefixes = ["ROUTER_", "QUOTER_", "FACTORY_", "WETH_", "USDC_", "USDT_"];
    
    prefixes
        .iter()
        .filter_map(|prefix| {
            let key = format!("{}{}", prefix, chain_id);
            env::var(&key).ok().map(|value| {
                (prefix.trim_end_matches('_').to_lowercase(), value)
            })
        })
        .collect()
}