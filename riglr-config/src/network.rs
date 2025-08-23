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
    pub fn validate_config(&self) -> ConfigResult<()> {
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

/// Solana-specific network configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SolanaNetworkConfig {
    /// Network name (mainnet, devnet, testnet)
    pub name: String,

    /// RPC endpoint URL
    pub rpc_url: String,

    /// Optional WebSocket URL
    #[serde(default)]
    pub ws_url: Option<String>,

    /// Optional block explorer URL
    #[serde(default)]
    pub explorer_url: Option<String>,
}

impl SolanaNetworkConfig {
    /// Create a new Solana network configuration
    pub fn new(name: impl Into<String>, rpc_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rpc_url: rpc_url.into(),
            ws_url: None,
            explorer_url: None,
        }
    }

    /// Create mainnet configuration
    pub fn mainnet() -> Self {
        Self::new("mainnet", "https://api.mainnet-beta.solana.com")
    }

    /// Create devnet configuration
    pub fn devnet() -> Self {
        Self::new("devnet", "https://api.devnet.solana.com")
    }

    /// Create testnet configuration
    pub fn testnet() -> Self {
        Self::new("testnet", "https://api.testnet.solana.com")
    }
}

/// EVM-specific network configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvmNetworkConfig {
    /// Network name (ethereum, polygon, arbitrum, etc.)
    pub name: String,

    /// Chain ID
    pub chain_id: u64,

    /// RPC endpoint URL
    pub rpc_url: String,

    /// Optional block explorer URL
    #[serde(default)]
    pub explorer_url: Option<String>,

    /// Native token symbol
    #[serde(default)]
    pub native_token: Option<String>,
}

impl EvmNetworkConfig {
    /// Create a new EVM network configuration
    pub fn new(name: impl Into<String>, chain_id: u64, rpc_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            chain_id,
            rpc_url: rpc_url.into(),
            explorer_url: None,
            native_token: None,
        }
    }

    /// Create Ethereum mainnet configuration
    pub fn ethereum_mainnet() -> Self {
        let mut config = Self::new("ethereum", 1, "https://eth.llamarpc.com");
        config.native_token = Some("ETH".to_string());
        config.explorer_url = Some("https://etherscan.io".to_string());
        config
    }

    /// Create Polygon configuration
    pub fn polygon() -> Self {
        let mut config = Self::new("polygon", 137, "https://polygon-rpc.com");
        config.native_token = Some("MATIC".to_string());
        config.explorer_url = Some("https://polygonscan.com".to_string());
        config
    }

    /// Generate CAIP-2 identifier
    pub fn caip2(&self) -> String {
        format!("eip155:{}", self.chain_id)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    // Helper function to create a temporary test directory
    fn create_temp_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    // Helper function to create a test chains.toml content
    fn create_test_chains_toml() -> String {
        r#"
[chains.ethereum]
id = 1
name = "Ethereum Mainnet"
router = "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"
quoter = "0xb27308f9F90D607463bb33eA8e66e3e6e63a3f75"
factory = "0x1F98431c8aD98523631AE4a59f267346ea31F984"
weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
usdc = "0xA0b86a33E6417efE3CF1AA5bAdC34a6a2C2d0BE0"
usdt = "0xdAC17F958D2ee523a2206206994597C13D831ec7"
explorer_url = "https://etherscan.io"
native_token = "ETH"
is_testnet = false

[chains.polygon]
id = 137
name = "Polygon"
router = "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"
is_testnet = false
"#
        .to_string()
    }

    #[test]
    fn test_default_functions() {
        assert_eq!(default_chain_id(), 1);
        assert_eq!(default_rpc_timeout(), 30);
        assert_eq!(default_ws_timeout(), 60);
        assert_eq!(default_http_timeout(), 30);
    }

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();

        assert_eq!(config.solana_rpc_url, "https://api.mainnet-beta.solana.com");
        assert_eq!(config.solana_ws_url, None);
        assert!(config.rpc_urls.is_empty());
        assert!(config.chains.is_empty());
        assert_eq!(config.default_chain_id, 1);
        assert_eq!(config.timeouts.rpc_timeout_secs, 30);
        assert_eq!(config.timeouts.ws_timeout_secs, 60);
        assert_eq!(config.timeouts.http_timeout_secs, 30);
    }

    #[test]
    fn test_network_timeouts_default() {
        let timeouts = NetworkTimeouts::default();

        assert_eq!(timeouts.rpc_timeout_secs, 30);
        assert_eq!(timeouts.ws_timeout_secs, 60);
        assert_eq!(timeouts.http_timeout_secs, 30);
    }

    #[test]
    fn test_chain_contract_default() {
        let contract = ChainContract::default();

        assert_eq!(contract.router, None);
        assert_eq!(contract.quoter, None);
        assert_eq!(contract.factory, None);
        assert_eq!(contract.weth, None);
        assert_eq!(contract.usdc, None);
        assert_eq!(contract.usdt, None);
        assert!(contract.custom.is_empty());
    }

    #[test]
    fn test_chain_from_toml_conversion() {
        let toml_chain = ChainFromToml {
            id: 1,
            name: "Ethereum".to_string(),
            router: Some("0x123".to_string()),
            quoter: Some("0x456".to_string()),
            factory: Some("0x789".to_string()),
            weth: Some("0xabc".to_string()),
            usdc: Some("0xdef".to_string()),
            usdt: Some("0x012".to_string()),
            explorer_url: Some("https://etherscan.io".to_string()),
            native_token: Some("ETH".to_string()),
            is_testnet: false,
        };

        let chain_config: ChainConfig = toml_chain.into();

        assert_eq!(chain_config.id, 1);
        assert_eq!(chain_config.name, "Ethereum");
        assert_eq!(chain_config.rpc_url, None);
        assert_eq!(chain_config.contracts.router, Some("0x123".to_string()));
        assert_eq!(chain_config.contracts.quoter, Some("0x456".to_string()));
        assert_eq!(chain_config.contracts.factory, Some("0x789".to_string()));
        assert_eq!(chain_config.contracts.weth, Some("0xabc".to_string()));
        assert_eq!(chain_config.contracts.usdc, Some("0xdef".to_string()));
        assert_eq!(chain_config.contracts.usdt, Some("0x012".to_string()));
        assert_eq!(
            chain_config.explorer_url,
            Some("https://etherscan.io".to_string())
        );
        assert_eq!(chain_config.native_token, Some("ETH".to_string()));
        assert!(!chain_config.is_testnet);
        assert!(chain_config.contracts.custom.is_empty());
    }

    #[test]
    fn test_chain_from_toml_conversion_with_defaults() {
        let toml_chain = ChainFromToml {
            id: 2,
            name: "Test Chain".to_string(),
            router: None,
            quoter: None,
            factory: None,
            weth: None,
            usdc: None,
            usdt: None,
            explorer_url: None,
            native_token: None,
            is_testnet: true,
        };

        let chain_config: ChainConfig = toml_chain.into();

        assert_eq!(chain_config.id, 2);
        assert_eq!(chain_config.name, "Test Chain");
        assert_eq!(chain_config.rpc_url, None);
        assert_eq!(chain_config.contracts.router, None);
        assert_eq!(chain_config.contracts.quoter, None);
        assert_eq!(chain_config.contracts.factory, None);
        assert_eq!(chain_config.contracts.weth, None);
        assert_eq!(chain_config.contracts.usdc, None);
        assert_eq!(chain_config.contracts.usdt, None);
        assert_eq!(chain_config.explorer_url, None);
        assert_eq!(chain_config.native_token, None);
        assert!(chain_config.is_testnet);
    }

    #[test]
    fn test_validate_address_valid() {
        assert!(validate_address("0x1234567890123456789012345678901234567890", "test").is_ok());
        assert!(validate_address("0xA0b86a33E6417efE3CF1AA5bAdC34a6a2C2d0BE0", "router").is_ok());
    }

    #[test]
    fn test_validate_address_invalid_no_prefix() {
        let result = validate_address("1234567890123456789012345678901234567890", "test");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid test address"));
    }

    #[test]
    fn test_validate_address_invalid_length_too_short() {
        let result = validate_address("0x123", "test");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid test address"));
    }

    #[test]
    fn test_validate_address_invalid_length_too_long() {
        let result = validate_address("0x12345678901234567890123456789012345678901", "test");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid test address"));
    }

    #[test]
    fn test_validate_address_empty_string() {
        let result = validate_address("", "test");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid test address"));
    }

    #[test]
    fn test_extract_rpc_urls() {
        // Set up test environment variables
        env::set_var("RPC_URL_1", "https://mainnet.infura.io");
        env::set_var("RPC_URL_137", "https://polygon-rpc.com");
        env::set_var("RPC_URL_INVALID", "https://invalid.com"); // Should be ignored
        env::set_var("NOT_RPC_URL_1", "https://should-be-ignored.com"); // Should be ignored

        let mut config = NetworkConfig::default();
        config.extract_rpc_urls();

        assert_eq!(
            config.rpc_urls.get("1"),
            Some(&"https://mainnet.infura.io".to_string())
        );
        assert_eq!(
            config.rpc_urls.get("137"),
            Some(&"https://polygon-rpc.com".to_string())
        );
        assert!(!config.rpc_urls.contains_key("INVALID"));
        assert!(!config.rpc_urls.contains_key("NOT_RPC_URL_1"));

        // Clean up
        env::remove_var("RPC_URL_1");
        env::remove_var("RPC_URL_137");
        env::remove_var("RPC_URL_INVALID");
        env::remove_var("NOT_RPC_URL_1");
    }

    #[test]
    fn test_extract_rpc_urls_empty_environment() {
        let mut config = NetworkConfig::default();
        config.extract_rpc_urls();

        // Should not crash with empty environment and rpc_urls should remain empty
        // (assuming no RPC_URL_* vars are set in test environment)
    }

    #[test]
    fn test_get_rpc_url_from_chain_config() {
        let mut config = NetworkConfig::default();

        let chain_config = ChainConfig {
            id: 1,
            name: "Ethereum".to_string(),
            rpc_url: Some("https://custom-rpc.com".to_string()),
            contracts: ChainContract::default(),
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        config.chains.insert(1, chain_config);

        assert_eq!(
            config.get_rpc_url(1),
            Some("https://custom-rpc.com".to_string())
        );
    }

    #[test]
    fn test_get_rpc_url_from_rpc_urls() {
        let mut config = NetworkConfig::default();
        config
            .rpc_urls
            .insert("137".to_string(), "https://polygon-rpc.com".to_string());

        assert_eq!(
            config.get_rpc_url(137),
            Some("https://polygon-rpc.com".to_string())
        );
    }

    #[test]
    fn test_get_rpc_url_chain_config_overrides_rpc_urls() {
        let mut config = NetworkConfig::default();
        config
            .rpc_urls
            .insert("1".to_string(), "https://fallback-rpc.com".to_string());

        let chain_config = ChainConfig {
            id: 1,
            name: "Ethereum".to_string(),
            rpc_url: Some("https://priority-rpc.com".to_string()),
            contracts: ChainContract::default(),
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        config.chains.insert(1, chain_config);

        assert_eq!(
            config.get_rpc_url(1),
            Some("https://priority-rpc.com".to_string())
        );
    }

    #[test]
    fn test_get_rpc_url_not_found() {
        let config = NetworkConfig::default();
        assert_eq!(config.get_rpc_url(999), None);
    }

    #[test]
    fn test_get_rpc_url_chain_config_without_rpc_url() {
        let mut config = NetworkConfig::default();
        config
            .rpc_urls
            .insert("1".to_string(), "https://fallback-rpc.com".to_string());

        let chain_config = ChainConfig {
            id: 1,
            name: "Ethereum".to_string(),
            rpc_url: None, // No RPC URL in chain config
            contracts: ChainContract::default(),
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        config.chains.insert(1, chain_config);

        assert_eq!(
            config.get_rpc_url(1),
            Some("https://fallback-rpc.com".to_string())
        );
    }

    #[test]
    fn test_get_chain_exists() {
        let mut config = NetworkConfig::default();

        let chain_config = ChainConfig {
            id: 1,
            name: "Ethereum".to_string(),
            rpc_url: None,
            contracts: ChainContract::default(),
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        config.chains.insert(1, chain_config);

        let result = config.get_chain(1);
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, 1);
        assert_eq!(result.unwrap().name, "Ethereum");
    }

    #[test]
    fn test_get_chain_not_exists() {
        let config = NetworkConfig::default();
        assert!(config.get_chain(999).is_none());
    }

    #[test]
    fn test_get_supported_chains_from_rpc_urls_only() {
        let mut config = NetworkConfig::default();
        config
            .rpc_urls
            .insert("1".to_string(), "https://eth.com".to_string());
        config
            .rpc_urls
            .insert("137".to_string(), "https://polygon.com".to_string());
        config
            .rpc_urls
            .insert("invalid".to_string(), "https://invalid.com".to_string()); // Should be ignored

        let chains = config.get_supported_chains();
        assert_eq!(chains.len(), 2);
        assert!(chains.contains(&1));
        assert!(chains.contains(&137));
    }

    #[test]
    fn test_get_supported_chains_from_chains_only() {
        let mut config = NetworkConfig::default();

        let chain1 = ChainConfig {
            id: 1,
            name: "Ethereum".to_string(),
            rpc_url: None,
            contracts: ChainContract::default(),
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        let chain2 = ChainConfig {
            id: 56,
            name: "BSC".to_string(),
            rpc_url: None,
            contracts: ChainContract::default(),
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        config.chains.insert(1, chain1);
        config.chains.insert(56, chain2);

        let chains = config.get_supported_chains();
        assert_eq!(chains.len(), 2);
        assert!(chains.contains(&1));
        assert!(chains.contains(&56));
    }

    #[test]
    fn test_get_supported_chains_mixed_sources_with_duplicates() {
        let mut config = NetworkConfig::default();

        // Add RPC URLs
        config
            .rpc_urls
            .insert("1".to_string(), "https://eth.com".to_string());
        config
            .rpc_urls
            .insert("137".to_string(), "https://polygon.com".to_string());

        // Add chain configs (including duplicate chain ID 1)
        let chain1 = ChainConfig {
            id: 1,
            name: "Ethereum".to_string(),
            rpc_url: None,
            contracts: ChainContract::default(),
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        let chain56 = ChainConfig {
            id: 56,
            name: "BSC".to_string(),
            rpc_url: None,
            contracts: ChainContract::default(),
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        config.chains.insert(1, chain1);
        config.chains.insert(56, chain56);

        let chains = config.get_supported_chains();
        assert_eq!(chains.len(), 3); // 1, 56, 137 (deduplicated)
        assert!(chains.contains(&1));
        assert!(chains.contains(&56));
        assert!(chains.contains(&137));
    }

    #[test]
    fn test_get_supported_chains_empty() {
        let config = NetworkConfig::default();
        let chains = config.get_supported_chains();
        assert!(chains.is_empty());
    }

    #[test]
    fn test_validate_valid_config() {
        let mut config = NetworkConfig::default();
        config.solana_rpc_url = "https://api.mainnet-beta.solana.com".to_string();
        config
            .rpc_urls
            .insert("1".to_string(), "https://mainnet.infura.io".to_string());
        config
            .rpc_urls
            .insert("137".to_string(), "wss://polygon-rpc.com".to_string());

        let chain_config = ChainConfig {
            id: 1,
            name: "Ethereum".to_string(),
            rpc_url: None,
            contracts: ChainContract {
                router: Some("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45".to_string()),
                quoter: Some("0xb27308f9F90D607463bb33eA8e66e3e6e63a3f75".to_string()),
                factory: Some("0x1F98431c8aD98523631AE4a59f267346ea31F984".to_string()),
                ..ChainContract::default()
            },
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        config.chains.insert(1, chain_config);

        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_invalid_solana_rpc_url_no_protocol() {
        let mut config = NetworkConfig::default();
        config.solana_rpc_url = "api.mainnet-beta.solana.com".to_string();

        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("SOLANA_RPC_URL must be a valid HTTP(S) URL"));
    }

    #[test]
    fn test_validate_invalid_solana_rpc_url_ftp_protocol() {
        let mut config = NetworkConfig::default();
        config.solana_rpc_url = "ftp://api.mainnet-beta.solana.com".to_string();

        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("SOLANA_RPC_URL must be a valid HTTP(S) URL"));
    }

    #[test]
    fn test_validate_invalid_rpc_url() {
        let mut config = NetworkConfig::default();
        config
            .rpc_urls
            .insert("1".to_string(), "ftp://invalid-protocol.com".to_string());

        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid RPC URL for chain 1"));
    }

    #[test]
    fn test_validate_valid_rpc_url_protocols() {
        let mut config = NetworkConfig::default();
        config
            .rpc_urls
            .insert("1".to_string(), "http://localhost:8545".to_string());
        config
            .rpc_urls
            .insert("2".to_string(), "https://mainnet.infura.io".to_string());
        config
            .rpc_urls
            .insert("3".to_string(), "ws://localhost:8546".to_string());
        config
            .rpc_urls
            .insert("4".to_string(), "wss://mainnet.infura.io/ws".to_string());

        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_chain_id_mismatch() {
        let mut config = NetworkConfig::default();

        let chain_config = ChainConfig {
            id: 2, // Different from the key (1)
            name: "Ethereum".to_string(),
            rpc_url: None,
            contracts: ChainContract::default(),
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        config.chains.insert(1, chain_config);

        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Chain ID mismatch: 1 vs 2"));
    }

    #[test]
    fn test_validate_invalid_router_address() {
        let mut config = NetworkConfig::default();

        let chain_config = ChainConfig {
            id: 1,
            name: "Ethereum".to_string(),
            rpc_url: None,
            contracts: ChainContract {
                router: Some("invalid_address".to_string()),
                ..ChainContract::default()
            },
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        config.chains.insert(1, chain_config);

        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid router address"));
    }

    #[test]
    fn test_validate_invalid_quoter_address() {
        let mut config = NetworkConfig::default();

        let chain_config = ChainConfig {
            id: 1,
            name: "Ethereum".to_string(),
            rpc_url: None,
            contracts: ChainContract {
                quoter: Some("0x123".to_string()), // Too short
                ..ChainContract::default()
            },
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        config.chains.insert(1, chain_config);

        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid quoter address"));
    }

    #[test]
    fn test_validate_invalid_factory_address() {
        let mut config = NetworkConfig::default();

        let chain_config = ChainConfig {
            id: 1,
            name: "Ethereum".to_string(),
            rpc_url: None,
            contracts: ChainContract {
                factory: Some("0x12345678901234567890123456789012345678901".to_string()), // Too long
                ..ChainContract::default()
            },
            explorer_url: None,
            native_token: None,
            is_testnet: false,
        };

        config.chains.insert(1, chain_config);

        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid factory address"));
    }

    #[test]
    fn test_load_chain_contracts_file_not_exists() {
        // Test with non-existent file (should not error)
        env::set_var(RIGLR_CHAINS_CONFIG, "/non/existent/path/chains.toml");

        let mut config = NetworkConfig::default();
        let result = config.load_chain_contracts();

        assert!(result.is_ok());
        assert!(config.chains.is_empty());

        env::remove_var(RIGLR_CHAINS_CONFIG);
    }

    #[test]
    fn test_load_chain_contracts_default_path_not_exists() {
        // Test with default path when environment variable is not set
        env::remove_var(RIGLR_CHAINS_CONFIG);

        let mut config = NetworkConfig::default();
        let result = config.load_chain_contracts();

        // Should succeed even if chains.toml doesn't exist
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_chain_contracts_valid_file() {
        let temp_dir = create_temp_dir();
        let chains_path = temp_dir.path().join("chains.toml");

        // Write test chains.toml
        fs::write(&chains_path, create_test_chains_toml()).unwrap();

        env::set_var(RIGLR_CHAINS_CONFIG, chains_path.to_str().unwrap());

        let mut config = NetworkConfig::default();
        let result = config.load_chain_contracts();

        assert!(result.is_ok());
        assert_eq!(config.chains.len(), 2);

        let eth_chain = config.chains.get(&1).unwrap();
        assert_eq!(eth_chain.name, "Ethereum Mainnet");
        assert_eq!(
            eth_chain.contracts.router,
            Some("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45".to_string())
        );
        assert_eq!(eth_chain.native_token, Some("ETH".to_string()));
        assert!(!eth_chain.is_testnet);

        let polygon_chain = config.chains.get(&137).unwrap();
        assert_eq!(polygon_chain.name, "Polygon");
        assert_eq!(
            polygon_chain.contracts.router,
            Some("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45".to_string())
        );
        assert_eq!(polygon_chain.contracts.quoter, None); // Not specified in TOML
        assert!(!polygon_chain.is_testnet);

        env::remove_var(RIGLR_CHAINS_CONFIG);
    }

    #[test]
    fn test_load_chain_contracts_invalid_toml() {
        let temp_dir = create_temp_dir();
        let chains_path = temp_dir.path().join("chains.toml");

        // Write invalid TOML
        fs::write(&chains_path, "invalid toml content [[[").unwrap();

        env::set_var(RIGLR_CHAINS_CONFIG, chains_path.to_str().unwrap());

        let mut config = NetworkConfig::default();
        let result = config.load_chain_contracts();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse chains.toml"));

        env::remove_var(RIGLR_CHAINS_CONFIG);
    }

    #[test]
    fn test_load_chain_contracts_with_environment_overrides() {
        let temp_dir = create_temp_dir();
        let chains_path = temp_dir.path().join("chains.toml");

        // Write test chains.toml
        fs::write(&chains_path, create_test_chains_toml()).unwrap();

        // Set environment overrides
        env::set_var("ROUTER_1", "0x1111111111111111111111111111111111111111");
        env::set_var("QUOTER_1", "0x2222222222222222222222222222222222222222");
        env::set_var("FACTORY_137", "0x3333333333333333333333333333333333333333");

        env::set_var(RIGLR_CHAINS_CONFIG, chains_path.to_str().unwrap());

        let mut config = NetworkConfig::default();
        let result = config.load_chain_contracts();

        assert!(result.is_ok());

        let eth_chain = config.chains.get(&1).unwrap();
        assert_eq!(
            eth_chain.contracts.router,
            Some("0x1111111111111111111111111111111111111111".to_string())
        );
        assert_eq!(
            eth_chain.contracts.quoter,
            Some("0x2222222222222222222222222222222222222222".to_string())
        );
        // Factory should remain from TOML since no override for chain 1
        assert_eq!(
            eth_chain.contracts.factory,
            Some("0x1F98431c8aD98523631AE4a59f267346ea31F984".to_string())
        );

        let polygon_chain = config.chains.get(&137).unwrap();
        // Router should remain from TOML since no override for chain 137
        assert_eq!(
            polygon_chain.contracts.router,
            Some("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45".to_string())
        );
        assert_eq!(polygon_chain.contracts.quoter, None); // No quoter in TOML and no env override
        assert_eq!(
            polygon_chain.contracts.factory,
            Some("0x3333333333333333333333333333333333333333".to_string())
        );

        // Clean up
        env::remove_var("ROUTER_1");
        env::remove_var("QUOTER_1");
        env::remove_var("FACTORY_137");
        env::remove_var(RIGLR_CHAINS_CONFIG);
    }

    #[test]
    fn test_load_chain_contracts_read_error() {
        // Test with a directory instead of a file to trigger read error
        let temp_dir = create_temp_dir();
        let chains_path = temp_dir.path().join("chains_dir");
        fs::create_dir(&chains_path).unwrap();

        env::set_var(RIGLR_CHAINS_CONFIG, chains_path.to_str().unwrap());

        let mut config = NetworkConfig::default();
        let result = config.load_chain_contracts();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read chains config"));

        env::remove_var(RIGLR_CHAINS_CONFIG);
    }
}
