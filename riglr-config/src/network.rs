//! Network and blockchain configuration

use crate::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const RIGLR_CHAINS_CONFIG: &str = "RIGLR_CHAINS_CONFIG";

/// Network configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConfig {
    /// Solana RPC URL
    pub solana_rpc_url: String,

    /// Solana WebSocket URL (optional)
    #[serde(default)]
    pub solana_ws_url: Option<String>,

    /// EVM RPC URLs using RPC_URL_{CHAIN_ID} convention
    /// This is populated dynamically from environment variables
    #[serde(default, skip_serializing)]
    pub rpc_urls: HashMap<String, String>,

    /// Chain-specific contract addresses
    #[serde(default, skip_serializing)]
    pub chains: HashMap<u64, ChainConfig>,

    /// Default chain ID to use
    #[serde(default = "default_chain_id")]
    pub default_chain_id: u64,

    /// Network timeouts
    #[serde(flatten)]
    pub timeouts: NetworkTimeouts,
}

/// Chain-specific configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainConfig {
    /// Chain ID
    pub id: u64,

    /// Human-readable chain name
    pub name: String,

    /// RPC URL (overrides global RPC_URL_{CHAIN_ID} if set)
    #[serde(default)]
    pub rpc_url: Option<String>,

    /// Contract addresses for this chain
    #[serde(flatten)]
    pub contracts: ChainContract,

    /// Block explorer URL
    #[serde(default)]
    pub explorer_url: Option<String>,

    /// Native token symbol
    #[serde(default)]
    pub native_token: Option<String>,

    /// Whether this is a testnet
    #[serde(default)]
    pub is_testnet: bool,
}

/// Contract addresses for a chain
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ChainContract {
    /// Uniswap V3 router address
    #[serde(default)]
    pub router: Option<String>,

    /// Uniswap V3 quoter address
    #[serde(default)]
    pub quoter: Option<String>,

    /// Uniswap V3 factory address
    #[serde(default)]
    pub factory: Option<String>,

    /// WETH/WNATIVE address
    #[serde(default)]
    pub weth: Option<String>,

    /// USDC address
    #[serde(default)]
    pub usdc: Option<String>,

    /// USDT address
    #[serde(default)]
    pub usdt: Option<String>,

    /// Additional custom contracts
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

/// Network timeout configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkTimeouts {
    /// RPC request timeout in seconds
    #[serde(default = "default_rpc_timeout")]
    pub rpc_timeout_secs: u64,

    /// WebSocket connection timeout in seconds
    #[serde(default = "default_ws_timeout")]
    pub ws_timeout_secs: u64,

    /// HTTP request timeout in seconds
    #[serde(default = "default_http_timeout")]
    pub http_timeout_secs: u64,
}

impl NetworkConfig {
    /// Extract RPC URLs from environment using RPC_URL_{CHAIN_ID} convention
    pub fn extract_rpc_urls(&mut self) {
        for (key, value) in std::env::vars() {
            if let Some(chain_id_str) = key.strip_prefix("RPC_URL_") {
                if chain_id_str.parse::<u64>().is_ok() {
                    self.rpc_urls.insert(chain_id_str.to_string(), value);
                }
            }
        }
    }

    /// Load chain contracts from chains.toml file
    pub fn load_chain_contracts(&mut self) -> ConfigResult<()> {
        let chains_path =
            std::env::var(RIGLR_CHAINS_CONFIG).unwrap_or_else(|_| "chains.toml".to_string());

        // Only try to load if file exists
        if !std::path::Path::new(&chains_path).exists() {
            tracing::debug!("chains.toml not found at {}, using defaults", chains_path);
            return Ok(());
        }

        let content = std::fs::read_to_string(&chains_path).map_err(|e| {
            ConfigError::io(format!(
                "Failed to read chains config from {}: {}",
                chains_path, e
            ))
        })?;

        let chains_file: ChainsFile = toml::from_str(&content)
            .map_err(|e| ConfigError::parse(format!("Failed to parse chains.toml: {}", e)))?;

        // Convert from TOML structure to our internal structure
        for (_name, toml_chain) in chains_file.chains {
            let mut chain: ChainConfig = toml_chain.into();

            // Apply environment variable overrides
            if let Ok(router) = std::env::var(format!("ROUTER_{}", chain.id)) {
                chain.contracts.router = Some(router);
            }
            if let Ok(quoter) = std::env::var(format!("QUOTER_{}", chain.id)) {
                chain.contracts.quoter = Some(quoter);
            }
            if let Ok(factory) = std::env::var(format!("FACTORY_{}", chain.id)) {
                chain.contracts.factory = Some(factory);
            }

            self.chains.insert(chain.id, chain);
        }

        Ok(())
    }

    /// Get RPC URL for a specific chain ID
    pub fn get_rpc_url(&self, chain_id: u64) -> Option<String> {
        // First check chain-specific config
        if let Some(chain) = self.chains.get(&chain_id) {
            if let Some(ref url) = chain.rpc_url {
                return Some(url.clone());
            }
        }

        // Then check dynamic RPC URLs
        self.rpc_urls.get(&chain_id.to_string()).cloned()
    }

    /// Get chain configuration
    pub fn get_chain(&self, chain_id: u64) -> Option<&ChainConfig> {
        self.chains.get(&chain_id)
    }

    /// Get all supported chain IDs
    pub fn get_supported_chains(&self) -> Vec<u64> {
        let mut chains: Vec<u64> = self
            .rpc_urls
            .keys()
            .filter_map(|k| k.parse::<u64>().ok())
            .collect();

        // Add chains from config
        chains.extend(self.chains.keys());

        // Deduplicate
        chains.sort_unstable();
        chains.dedup();

        chains
    }

    /// Validates the network configuration
    ///
    /// Checks that all URLs are properly formatted and contract addresses are valid
    pub fn validate(&self) -> ConfigResult<()> {
        // Validate Solana RPC URL
        if !self.solana_rpc_url.starts_with("http://")
            && !self.solana_rpc_url.starts_with("https://")
        {
            return Err(ConfigError::validation(
                "SOLANA_RPC_URL must be a valid HTTP(S) URL",
            ));
        }

        // Validate RPC URLs
        for (chain_id, url) in &self.rpc_urls {
            if !url.starts_with("http://")
                && !url.starts_with("https://")
                && !url.starts_with("wss://")
                && !url.starts_with("ws://")
            {
                return Err(ConfigError::validation(format!(
                    "Invalid RPC URL for chain {}: {}",
                    chain_id, url
                )));
            }
        }

        // Validate chain configs
        for (chain_id, chain) in &self.chains {
            if chain.id != *chain_id {
                return Err(ConfigError::validation(format!(
                    "Chain ID mismatch: {} vs {}",
                    chain_id, chain.id
                )));
            }

            // Validate contract addresses are valid hex
            if let Some(ref addr) = chain.contracts.router {
                validate_address(addr, "router")?;
            }
            if let Some(ref addr) = chain.contracts.quoter {
                validate_address(addr, "quoter")?;
            }
            if let Some(ref addr) = chain.contracts.factory {
                validate_address(addr, "factory")?;
            }
        }

        Ok(())
    }
}

/// Helper to validate Ethereum addresses
fn validate_address(addr: &str, name: &str) -> ConfigResult<()> {
    if !addr.starts_with("0x") || addr.len() != 42 {
        return Err(ConfigError::validation(format!(
            "Invalid {} address: {}",
            name, addr
        )));
    }
    Ok(())
}

/// Structure for parsing chains.toml file
#[derive(Debug, Deserialize)]
struct ChainsFile {
    chains: HashMap<String, ChainFromToml>,
}

/// Chain configuration as parsed from TOML
#[derive(Debug, Deserialize)]
struct ChainFromToml {
    id: u64,
    name: String,
    #[serde(default)]
    router: Option<String>,
    #[serde(default)]
    quoter: Option<String>,
    #[serde(default)]
    factory: Option<String>,
    #[serde(default)]
    weth: Option<String>,
    #[serde(default)]
    usdc: Option<String>,
    #[serde(default)]
    usdt: Option<String>,
    #[serde(default)]
    explorer_url: Option<String>,
    #[serde(default)]
    native_token: Option<String>,
    #[serde(default)]
    is_testnet: bool,
}

impl From<ChainFromToml> for ChainConfig {
    fn from(toml: ChainFromToml) -> Self {
        Self {
            id: toml.id,
            name: toml.name,
            rpc_url: None,
            contracts: ChainContract {
                router: toml.router,
                quoter: toml.quoter,
                factory: toml.factory,
                weth: toml.weth,
                usdc: toml.usdc,
                usdt: toml.usdt,
                custom: HashMap::new(),
            },
            explorer_url: toml.explorer_url,
            native_token: toml.native_token,
            is_testnet: toml.is_testnet,
        }
    }
}

// Default value functions
fn default_chain_id() -> u64 {
    1
} // Ethereum mainnet
fn default_rpc_timeout() -> u64 {
    30
}
fn default_ws_timeout() -> u64 {
    60
}
fn default_http_timeout() -> u64 {
    30
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            solana_rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            solana_ws_url: None,
            rpc_urls: HashMap::new(),
            chains: HashMap::new(),
            default_chain_id: default_chain_id(),
            timeouts: NetworkTimeouts::default(),
        }
    }
}

impl Default for NetworkTimeouts {
    fn default() -> Self {
        Self {
            rpc_timeout_secs: default_rpc_timeout(),
            ws_timeout_secs: default_ws_timeout(),
            http_timeout_secs: default_http_timeout(),
        }
    }
}
