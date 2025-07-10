//! EVM client for interacting with Ethereum and EVM-compatible chains
//!
//! This module provides a production-grade client for EVM operations using alloy-rs.

use crate::error::{EvmToolError, Result};
use alloy::node_bindings::Anvil;
use alloy::primitives::{Address, U256};
use alloy::network::Ethereum;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

/// Configuration for EVM client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub timeout: Duration,
}

impl Default for EvmConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://eth.llamarpc.com".to_string(),
            chain_id: 1, // Ethereum mainnet
            timeout: Duration::from_secs(30),
        }
    }
}

/// Production-grade EVM client using alloy-rs
#[derive(Clone)]
pub struct EvmClient {
    provider: Arc<dyn Provider<Ethereum>>,
    signer: Option<PrivateKeySigner>,  // Add signer field
    config: EvmConfig,
    pub rpc_url: String,
    pub chain_id: u64,
}

impl EvmClient {
    /// Create a new EVM client
    pub async fn new(rpc_url: String) -> Result<Self> {
        debug!("Connecting to EVM RPC: {}", rpc_url);

        // Parse the URL
        let url = rpc_url
            .parse()
            .map_err(|e| EvmToolError::Generic(format!("Invalid RPC URL: {}", e)))?;

        // Create provider
        let provider = ProviderBuilder::new()
            .connect_http(url);

        // Get chain ID
        let chain_id = provider
            .get_chain_id()
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get chain ID: {}", e)))?;

        info!(
            "Connected to EVM chain {} (ID: {})",
            Self::chain_name(chain_id),
            chain_id
        );

        let config = EvmConfig {
            rpc_url: rpc_url.clone(),
            chain_id,
            timeout: Duration::from_secs(30),
        };

        Ok(Self {
            provider: Arc::new(provider) as Arc<dyn Provider<Ethereum>>,
            signer: None,  // Initialize as None
            config,
            rpc_url,
            chain_id,
        })
    }

    /// Create a mainnet client
    pub async fn mainnet() -> Result<Self> {
        Self::new("https://eth.llamarpc.com".to_string()).await
    }

    /// Create a Polygon client
    pub async fn polygon() -> Result<Self> {
        Self::new("https://polygon-rpc.com".to_string()).await
    }

    /// Create an Arbitrum client
    pub async fn arbitrum() -> Result<Self> {
        Self::new("https://arb1.arbitrum.io/rpc".to_string()).await
    }

    /// Create an Optimism client
    pub async fn optimism() -> Result<Self> {
        Self::new("https://mainnet.optimism.io".to_string()).await
    }

    /// Create a Base client
    pub async fn base() -> Result<Self> {
        Self::new("https://mainnet.base.org".to_string()).await
    }

    /// Create a local Anvil client for testing
    pub async fn anvil() -> Result<Self> {
        let anvil = Anvil::new().spawn();
        let url = anvil.endpoint();
        Self::new(url).await
    }

    /// Get chain name from chain ID
    pub fn chain_name(chain_id: u64) -> &'static str {
        match chain_id {
            1 => "Ethereum Mainnet",
            5 => "Goerli Testnet",
            11155111 => "Sepolia Testnet",
            137 => "Polygon",
            42161 => "Arbitrum One",
            10 => "Optimism",
            8453 => "Base",
            43114 => "Avalanche C-Chain",
            56 => "BNB Smart Chain",
            250 => "Fantom",
            _ => "Unknown Chain",
        }
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        debug!("Getting current block number");

        let block_number = self.provider
            .get_block_number()
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get block number: {}", e)))?;

        Ok(block_number)
    }

    /// Get ETH balance for an address
    pub async fn get_balance(&self, address: Address) -> Result<U256> {
        debug!("Getting balance for address: {}", address);

        let balance = self.provider
            .get_balance(address)
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get balance: {}", e)))?;

        Ok(balance)
    }

    /// Get gas price
    pub async fn get_gas_price(&self) -> Result<u128> {
        debug!("Getting current gas price");

        let gas_price = self.provider
            .get_gas_price()
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get gas price: {}", e)))?;

        Ok(gas_price)
    }

    /// Get the provider
    pub fn provider(&self) -> &Arc<dyn Provider<Ethereum>> {
        &self.provider
    }

    /// Get config
    pub fn config(&self) -> &EvmConfig {
        &self.config
    }

    /// Configure client with a private key signer
    pub fn with_signer(mut self, private_key: &str) -> Result<Self> {
        let signer = private_key.parse::<PrivateKeySigner>()
            .map_err(|e| EvmToolError::InvalidKey(format!("Invalid private key: {}", e)))?;
        self.signer = Some(signer);
        Ok(self)
    }
    
    /// Get reference to the signer if configured
    pub fn signer(&self) -> Option<&PrivateKeySigner> {
        self.signer.as_ref()
    }
    
    /// Check if client has a signer configured
    pub fn has_signer(&self) -> bool {
        self.signer.is_some()
    }
    
    /// Get signer or return error if not configured
    pub fn require_signer(&self) -> Result<&PrivateKeySigner> {
        self.signer.as_ref()
            .ok_or_else(|| EvmToolError::Generic("Client requires signer configuration".to_string()))
    }

    /// Create an EvmClient from a TransactionSigner
    pub async fn from_signer(signer: &dyn riglr_core::signer::TransactionSigner) -> Result<Self> {
        // Get the EVM client from the signer context
        let _client_any = signer.evm_client()
            .map_err(|e| EvmToolError::Generic(format!("Failed to get EVM client: {}", e)))?;
        
        // For now, create a basic mainnet client
        // In a real implementation, we'd extract the proper configuration from the signer
        Self::mainnet().await
    }
}

/// Validate an Ethereum address
pub fn validate_address(address: &str) -> Result<Address> {
    address
        .parse::<Address>()
        .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid address {}: {}", address, e)))
}

/// Convert wei to ETH
pub fn wei_to_eth(wei: U256) -> f64 {
    // Convert U256 to f64, dividing by 10^18
    let wei_f64 = wei.to_string().parse::<f64>().unwrap_or(0.0);
    wei_f64 / 1e18
}

/// Convert ETH to wei
pub fn eth_to_wei(eth: f64) -> U256 {
    let wei = (eth * 1e18) as u128;
    U256::from(wei)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EvmConfig::default();
        assert_eq!(config.chain_id, 1);
        assert!(config.rpc_url.contains("eth"));
    }

    #[test]
    fn test_chain_name() {
        assert_eq!(EvmClient::chain_name(1), "Ethereum Mainnet");
        assert_eq!(EvmClient::chain_name(137), "Polygon");
        assert_eq!(EvmClient::chain_name(42161), "Arbitrum One");
        assert_eq!(EvmClient::chain_name(999999), "Unknown Chain");
    }

    #[test]
    fn test_validate_address() {
        // Valid address
        let valid = "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B";
        assert!(validate_address(valid).is_ok());

        // Invalid address
        let invalid = "invalid_address";
        assert!(validate_address(invalid).is_err());
    }

    #[test]
    fn test_wei_conversions() {
        // Test wei to ETH
        let one_eth_in_wei = U256::from(1_000_000_000_000_000_000u128);
        let eth = wei_to_eth(one_eth_in_wei);
        assert!((eth - 1.0).abs() < 0.000001);

        // Test ETH to wei
        let wei = eth_to_wei(1.0);
        assert_eq!(wei, one_eth_in_wei);
    }
}