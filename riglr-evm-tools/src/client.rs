//! EVM client for interacting with EVM-based blockchains using alloy-rs
//!
//! This module provides a production-grade client for interacting with
//! Ethereum and EVM-compatible blockchains using the alloy framework.

use crate::error::{EvmToolError, Result};
use alloy::network::{Ethereum, EthereumWallet};
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::rpc::client::RpcClient;
use alloy::transports::http::{Client as HttpClient, Http};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// A production-grade client for interacting with EVM-based blockchains
#[derive(Clone)]
pub struct EvmClient {
    /// Alloy provider for blockchain interactions
    provider: Arc<RootProvider<Http<HttpClient>>>,
    /// RPC URL for the blockchain
    pub rpc_url: String,
    /// Chain ID for the target blockchain
    pub chain_id: u64,
}

impl EvmClient {
    /// Create a new EVM client with the given RPC URL
    pub async fn new(rpc_url: String) -> Result<Self> {
        debug!("Connecting to EVM RPC: {}", rpc_url);

        // Create HTTP transport
        let http = Http::<HttpClient>::new(rpc_url.parse().map_err(|e| {
            EvmToolError::Generic(format!("Invalid RPC URL: {}", e))
        })?);

        // Build provider
        let provider = ProviderBuilder::new()
            .on_http(rpc_url.parse().map_err(|e| {
                EvmToolError::Generic(format!("Invalid RPC URL: {}", e))
            })?);

        // Get chain ID
        let chain_id = provider
            .get_chain_id()
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get chain ID: {}", e)))?;

        info!(
            "Connected to EVM blockchain: {} (chain_id: {})",
            rpc_url, chain_id
        );

        Ok(Self {
            provider: Arc::new(provider),
            rpc_url: rpc_url.clone(),
            chain_id,
        })
    }

    /// Create a new EVM client for Ethereum mainnet
    pub async fn mainnet() -> Result<Self> {
        Self::new("https://eth-mainnet.g.alchemy.com/v2/demo".to_string()).await
    }

    /// Create a new EVM client for Ethereum mainnet with API key
    pub async fn mainnet_with_api_key(api_key: &str) -> Result<Self> {
        let rpc_url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", api_key);
        Self::new(rpc_url).await
    }

    /// Create a new EVM client for Polygon
    pub async fn polygon() -> Result<Self> {
        Self::new("https://polygon-rpc.com".to_string()).await
    }

    /// Create a new EVM client for Polygon with API key
    pub async fn polygon_with_api_key(api_key: &str) -> Result<Self> {
        let rpc_url = format!("https://polygon-mainnet.g.alchemy.com/v2/{}", api_key);
        Self::new(rpc_url).await
    }

    /// Create a new EVM client for Arbitrum One
    pub async fn arbitrum() -> Result<Self> {
        Self::new("https://arb1.arbitrum.io/rpc".to_string()).await
    }

    /// Create a new EVM client for Optimism
    pub async fn optimism() -> Result<Self> {
        Self::new("https://mainnet.optimism.io".to_string()).await
    }

    /// Create a new EVM client for Base
    pub async fn base() -> Result<Self> {
        Self::new("https://mainnet.base.org".to_string()).await
    }

    /// Get the provider reference
    pub fn provider(&self) -> &Arc<RootProvider<Http<HttpClient>>> {
        &self.provider
    }

    /// Get the current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get block number: {}", e)))
    }

    /// Get the current gas price in wei
    pub async fn get_gas_price(&self) -> Result<U256> {
        self.provider
            .get_gas_price()
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get gas price: {}", e)))
    }

    /// Get ETH balance for an address
    pub async fn get_balance(&self, address: &str) -> Result<U256> {
        let addr = Address::from_str(address)
            .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid address: {}", e)))?;

        self.provider
            .get_balance(addr)
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get balance: {}", e)))
    }

    /// Get transaction count (nonce) for an address
    pub async fn get_transaction_count(&self, address: &str) -> Result<u64> {
        let addr = Address::from_str(address)
            .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid address: {}", e)))?;

        self.provider
            .get_transaction_count(addr)
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get transaction count: {}", e)))
    }

    /// Get transaction receipt by hash
    pub async fn get_transaction_receipt(&self, tx_hash: &str) -> Result<Option<alloy::rpc::types::TransactionReceipt>> {
        let hash = tx_hash.parse()
            .map_err(|e| EvmToolError::Generic(format!("Invalid transaction hash: {}", e)))?;

        self.provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get transaction receipt: {}", e)))
    }

    /// Estimate gas for a transaction
    pub async fn estimate_gas(
        &self,
        from: &str,
        to: &str,
        value: Option<U256>,
        data: Option<Vec<u8>>,
    ) -> Result<u128> {
        let from_addr = Address::from_str(from)
            .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid from address: {}", e)))?;
        let to_addr = Address::from_str(to)
            .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid to address: {}", e)))?;

        let mut tx = alloy::rpc::types::TransactionRequest::default()
            .from(from_addr)
            .to(to_addr);

        if let Some(val) = value {
            tx = tx.value(val);
        }

        if let Some(d) = data {
            tx = tx.input(d.into());
        }

        self.provider
            .estimate_gas(&tx)
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to estimate gas: {}", e)))
    }

    /// Call a smart contract (read-only)
    pub async fn call_contract(
        &self,
        to: &str,
        data: Vec<u8>,
    ) -> Result<Vec<u8>> {
        let to_addr = Address::from_str(to)
            .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid contract address: {}", e)))?;

        let tx = alloy::rpc::types::TransactionRequest::default()
            .to(to_addr)
            .input(data.into());

        let result = self.provider
            .call(&tx)
            .await
            .map_err(|e| EvmToolError::Contract(format!("Contract call failed: {}", e)))?;

        Ok(result.to_vec())
    }
}

/// Helper function to validate Ethereum address format
pub fn validate_address(address_str: &str) -> Result<Address> {
    Address::from_str(address_str)
        .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid address: {}", e)))
}

/// Helper function to validate transaction hash format  
pub fn validate_tx_hash(hash_str: &str) -> Result<alloy::primitives::TxHash> {
    hash_str.parse()
        .map_err(|e| EvmToolError::Generic(format!("Invalid transaction hash: {}", e)))
}

/// Helper function to format wei as ETH
pub fn wei_to_eth(wei: U256) -> f64 {
    let eth = wei.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;
    eth
}

/// Helper function to format ETH as wei
pub fn eth_to_wei(eth: f64) -> U256 {
    U256::from((eth * 1e18) as u128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_address() {
        let addr_str = "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123";
        let result = validate_address(addr_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_address() {
        let addr_str = "invalid_address";
        let result = validate_address(addr_str);
        assert!(result.is_err());

        let short_addr = "0x123";
        let result = validate_address(short_addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tx_hash() {
        let hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = validate_tx_hash(hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_wei_to_eth_conversion() {
        let wei = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
        let eth = wei_to_eth(wei);
        assert!((eth - 1.0).abs() < 0.000001);
    }

    #[test]
    fn test_eth_to_wei_conversion() {
        let eth = 1.5;
        let wei = eth_to_wei(eth);
        assert_eq!(wei, U256::from(1_500_000_000_000_000_000u128));
    }
}