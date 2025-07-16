//! Configuration types for authentication providers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Base configuration for authentication providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Provider-specific configuration
    pub provider_config: HashMap<String, String>,
    
    /// Cache configuration
    #[serde(default)]
    pub cache_config: CacheConfig,
    
    /// Network-specific overrides
    #[serde(default)]
    pub network_overrides: HashMap<String, NetworkOverride>,
}

/// Cache configuration for authentication providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching of user data
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    
    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl_seconds: u64,
    
    /// Maximum cache size
    #[serde(default = "default_cache_size")]
    pub max_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            ttl_seconds: default_cache_ttl(),
            max_size: default_cache_size(),
        }
    }
}

fn default_cache_enabled() -> bool { true }
fn default_cache_ttl() -> u64 { 300 } // 5 minutes
fn default_cache_size() -> usize { 1000 }

/// Network-specific configuration overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOverride {
    /// Override RPC URL for this network
    pub rpc_url: Option<String>,
    
    /// Override chain ID
    pub chain_id: Option<u64>,
    
    /// Custom parameters
    #[serde(default)]
    pub custom_params: HashMap<String, String>,
}

/// Common trait for provider-specific configurations
pub trait ProviderConfig: Send + Sync {
    /// Validate the configuration
    fn validate(&self) -> Result<(), crate::AuthError>;
    
    /// Get provider name
    fn provider_name(&self) -> &'static str;
    
    /// Create from environment variables
    fn from_env() -> Result<Self, crate::AuthError>
    where
        Self: Sized;
}