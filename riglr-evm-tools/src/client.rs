//! EVM client for interacting with EVM-based blockchains

use crate::error::Result;
use reqwest::Client;
use std::collections::HashMap;

/// A client for interacting with EVM-based blockchains
#[derive(Debug, Clone)]
pub struct EvmClient {
    /// HTTP client for making requests
    pub http_client: Client,
    /// RPC endpoint URL
    pub rpc_url: String,
    /// Chain ID for the target blockchain
    pub chain_id: u64,
    /// Optional configuration
    pub config: HashMap<String, String>,
}

impl EvmClient {
    /// Create a new EVM client with the given RPC URL and chain ID
    pub fn new(rpc_url: String, chain_id: u64) -> Self {
        Self {
            http_client: Client::new(),
            rpc_url,
            chain_id,
            config: HashMap::new(),
        }
    }

    /// Create a new EVM client for Ethereum mainnet
    pub fn ethereum() -> Self {
        Self::new("https://eth-mainnet.g.alchemy.com/v2/demo".to_string(), 1)
    }

    /// Create a new EVM client for Polygon
    pub fn polygon() -> Self {
        Self::new("https://polygon-rpc.com".to_string(), 137)
    }

    /// Create a new EVM client for Arbitrum One
    pub fn arbitrum() -> Self {
        Self::new("https://arb1.arbitrum.io/rpc".to_string(), 42161)
    }

    /// Create a new EVM client for Optimism
    pub fn optimism() -> Self {
        Self::new("https://mainnet.optimism.io".to_string(), 10)
    }

    /// Create a new EVM client for Base
    pub fn base() -> Self {
        Self::new("https://mainnet.base.org".to_string(), 8453)
    }

    /// Set configuration option
    pub fn with_config(mut self, key: String, value: String) -> Self {
        self.config.insert(key, value);
        self
    }

    /// Placeholder method for future RPC calls
    pub async fn call_rpc(&self, _method: &str, _params: serde_json::Value) -> Result<serde_json::Value> {
        // TODO: Implement actual RPC call logic
        Ok(serde_json::json!({}))
    }
}

impl Default for EvmClient {
    fn default() -> Self {
        Self::ethereum()
    }
}