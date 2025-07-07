//! EVM client for interacting with EVM-based blockchains
//!
//! This module provides a production-grade client for interacting with
//! Ethereum and EVM-compatible blockchains.

use crate::error::{EvmToolError, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Configuration for EVM client
#[derive(Debug, Clone)]
pub struct EvmConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Maximum number of retries for failed requests
    pub max_retries: usize,
    /// Retry delay
    pub retry_delay: Duration,
    /// Custom headers for RPC requests
    pub headers: HashMap<String, String>,
}

impl Default for EvmConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            headers: HashMap::new(),
        }
    }
}

/// A production-grade client for interacting with EVM-based blockchains
#[derive(Debug, Clone)]
pub struct EvmClient {
    /// HTTP client for JSON-RPC calls
    pub http_client: Arc<Client>,
    pub rpc_url: String,
    /// Chain ID for the target blockchain
    pub chain_id: u64,
    /// Client configuration
    pub config: EvmConfig,
}

impl EvmClient {
    /// Create a new EVM client with the given RPC URL and chain ID
    pub async fn new(rpc_url: String, chain_id: u64) -> Result<Self> {
        Self::with_config(rpc_url, chain_id, EvmConfig::default()).await
    }

    /// Create a new EVM client with custom configuration
    pub async fn with_config(rpc_url: String, chain_id: u64, config: EvmConfig) -> Result<Self> {
        debug!(
            "Connecting to EVM RPC: {} (chain_id: {})",
            rpc_url, chain_id
        );

        // Build HTTP client with custom configuration
        let mut client_builder = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent("riglr-evm-tools/0.1.0");

        // Add custom headers
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in &config.headers {
            let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| EvmToolError::Generic(format!("Invalid header name: {}", e)))?;
            let header_value = reqwest::header::HeaderValue::from_str(value)
                .map_err(|e| EvmToolError::Generic(format!("Invalid header value: {}", e)))?;
            headers.insert(header_name, header_value);
        }

        if !headers.is_empty() {
            client_builder = client_builder.default_headers(headers);
        }

        let http_client =
            Arc::new(client_builder.build().map_err(|e| {
                EvmToolError::Generic(format!("Failed to build HTTP client: {}", e))
            })?);

        // Verify connection by getting chain ID
        let client = Self {
            http_client: http_client.clone(),
            rpc_url: rpc_url.clone(),
            chain_id,
            config: config.clone(),
        };

        let actual_chain_id = client
            .get_chain_id()
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get chain ID: {}", e)))?;

        if actual_chain_id != chain_id {
            warn!(
                "Chain ID mismatch: expected {}, got {}",
                chain_id, actual_chain_id
            );
        }

        info!(
            "Connected to EVM blockchain: {} (chain_id: {})",
            rpc_url, actual_chain_id
        );

        Ok(Self {
            http_client,
            rpc_url,
            chain_id: actual_chain_id,
            config,
        })
    }

    /// Create a new EVM client for Ethereum mainnet
    pub async fn ethereum() -> Result<Self> {
        Self::new("https://eth-mainnet.g.alchemy.com/v2/demo".to_string(), 1).await
    }

    /// Create a new EVM client for Ethereum mainnet with API key
    pub async fn ethereum_with_api_key(api_key: &str) -> Result<Self> {
        let rpc_url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", api_key);
        Self::new(rpc_url, 1).await
    }

    /// Create a new EVM client for Polygon
    pub async fn polygon() -> Result<Self> {
        Self::new("https://polygon-rpc.com".to_string(), 137).await
    }

    /// Create a new EVM client for Polygon with API key
    pub async fn polygon_with_api_key(api_key: &str) -> Result<Self> {
        let rpc_url = format!("https://polygon-mainnet.g.alchemy.com/v2/{}", api_key);
        Self::new(rpc_url, 137).await
    }

    /// Create a new EVM client for Arbitrum One
    pub async fn arbitrum() -> Result<Self> {
        Self::new("https://arb1.arbitrum.io/rpc".to_string(), 42161).await
    }

    /// Create a new EVM client for Optimism
    pub async fn optimism() -> Result<Self> {
        Self::new("https://mainnet.optimism.io".to_string(), 10).await
    }

    /// Create a new EVM client for Base
    pub async fn base() -> Result<Self> {
        Self::new("https://mainnet.base.org".to_string(), 8453).await
    }

    /// Create a client with custom RPC URL (auto-detect chain ID)
    pub async fn with_rpc_url(rpc_url: String) -> Result<Self> {
        // Create temporary client to detect chain ID
        let temp_config = EvmConfig::default();
        let temp_client = Arc::new(
            reqwest::Client::builder()
                .timeout(temp_config.timeout)
                .build()
                .map_err(|e| {
                    EvmToolError::Generic(format!("Failed to build HTTP client: {}", e))
                })?,
        );

        let chain_id_hex =
            Self::rpc_call(&temp_client, &rpc_url, "eth_chainId", &json!([])).await?;

        let chain_id = u64::from_str_radix(
            chain_id_hex
                .as_str()
                .unwrap_or("0x1")
                .trim_start_matches("0x"),
            16,
        )
        .unwrap_or(1);

        Self::new(rpc_url, chain_id).await
    }

    /// Make a JSON-RPC call
    async fn rpc_call(
        client: &Client,
        rpc_url: &str,
        method: &str,
        params: &Value,
    ) -> Result<Value> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| EvmToolError::Http(e))?;

        if !response.status().is_success() {
            return Err(EvmToolError::Rpc(format!(
                "RPC request failed with status: {}",
                response.status()
            )));
        }

        let rpc_response: Value = response.json().await.map_err(|e| EvmToolError::Http(e))?;

        if let Some(error) = rpc_response.get("error") {
            return Err(EvmToolError::Rpc(format!(
                "RPC error: {}",
                error.get("message").unwrap_or(&json!("Unknown error"))
            )));
        }

        rpc_response
            .get("result")
            .cloned()
            .ok_or_else(|| EvmToolError::Rpc("Missing result in RPC response".to_string()))
    }

    /// Make an RPC call using this client's HTTP client
    pub async fn call_rpc(&self, method: &str, params: &Value) -> Result<Value> {
        Self::rpc_call(&self.http_client, &self.rpc_url, method, params).await
    }

    /// Get the chain ID
    pub async fn get_chain_id(&self) -> Result<u64> {
        let result = self.call_rpc("eth_chainId", &json!([])).await?;
        let chain_id_hex = result
            .as_str()
            .ok_or_else(|| EvmToolError::Rpc("Invalid chain ID format".to_string()))?;

        u64::from_str_radix(chain_id_hex.trim_start_matches("0x"), 16)
            .map_err(|e| EvmToolError::Rpc(format!("Failed to parse chain ID: {}", e)))
    }

    /// Get the current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        let result = self.call_rpc("eth_blockNumber", &json!([])).await?;
        let block_hex = result
            .as_str()
            .ok_or_else(|| EvmToolError::Rpc("Invalid block number format".to_string()))?;

        u64::from_str_radix(block_hex.trim_start_matches("0x"), 16)
            .map_err(|e| EvmToolError::Rpc(format!("Failed to parse block number: {}", e)))
    }

    /// Get the current gas price
    pub async fn get_gas_price(&self) -> Result<u64> {
        let result = self.call_rpc("eth_gasPrice", &json!([])).await?;
        let gas_price_hex = result
            .as_str()
            .ok_or_else(|| EvmToolError::Rpc("Invalid gas price format".to_string()))?;

        u64::from_str_radix(gas_price_hex.trim_start_matches("0x"), 16)
            .map_err(|e| EvmToolError::Rpc(format!("Failed to parse gas price: {}", e)))
    }

    /// Get ETH balance for an address
    pub async fn get_balance(&self, address: &str) -> Result<String> {
        let result = self
            .call_rpc("eth_getBalance", &json!([address, "latest"]))
            .await?;

        result
            .as_str()
            .ok_or_else(|| EvmToolError::Rpc("Invalid balance format".to_string()))
            .map(|s| s.to_string())
    }

    /// Get transaction count (nonce) for an address
    pub async fn get_transaction_count(&self, address: &str) -> Result<u64> {
        let result = self
            .call_rpc("eth_getTransactionCount", &json!([address, "latest"]))
            .await?;

        let nonce_hex = result
            .as_str()
            .ok_or_else(|| EvmToolError::Rpc("Invalid transaction count format".to_string()))?;

        u64::from_str_radix(nonce_hex.trim_start_matches("0x"), 16)
            .map_err(|e| EvmToolError::Rpc(format!("Failed to parse transaction count: {}", e)))
    }

    /// Make a contract call
    pub async fn call_contract(&self, to: &str, data: &str) -> Result<String> {
        let result = self
            .call_rpc(
                "eth_call",
                &json!([{
                "to": to,
                "data": data
            }, "latest"]),
            )
            .await?;

        result
            .as_str()
            .ok_or_else(|| EvmToolError::Contract("Invalid call result format".to_string()))
            .map(|s| s.to_string())
    }

    /// Send a raw transaction
    pub async fn send_raw_transaction(&self, tx_data: &str) -> Result<String> {
        let result = self
            .call_rpc("eth_sendRawTransaction", &json!([tx_data]))
            .await?;

        result
            .as_str()
            .ok_or_else(|| EvmToolError::Transaction("Invalid transaction hash format".to_string()))
            .map(|s| s.to_string())
    }
}

/// Helper function to validate Ethereum address format
pub fn validate_address(address_str: &str) -> Result<String> {
    // Basic validation: must be 42 chars, start with 0x, and be valid hex
    if address_str.len() != 42 {
        return Err(EvmToolError::InvalidAddress(format!(
            "Address must be 42 characters long, got {}",
            address_str.len()
        )));
    }

    if !address_str.starts_with("0x") {
        return Err(EvmToolError::InvalidAddress(
            "Address must start with '0x'".to_string(),
        ));
    }

    // Check if hex is valid
    if !address_str[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(EvmToolError::InvalidAddress(
            "Address contains invalid hex characters".to_string(),
        ));
    }

    Ok(address_str.to_lowercase())
}

/// Helper function to validate transaction hash format  
pub fn validate_tx_hash(hash_str: &str) -> Result<String> {
    if hash_str.len() != 66 {
        return Err(EvmToolError::Generic(format!(
            "Transaction hash must be 66 characters long, got {}",
            hash_str.len()
        )));
    }

    if !hash_str.starts_with("0x") {
        return Err(EvmToolError::Generic(
            "Transaction hash must start with '0x'".to_string(),
        ));
    }

    if !hash_str[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(EvmToolError::Generic(
            "Transaction hash contains invalid hex characters".to_string(),
        ));
    }

    Ok(hash_str.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_address() {
        let addr_str = "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123";
        let result = validate_address(addr_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), addr_str.to_lowercase());
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
    fn test_config_defaults() {
        let config = EvmConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
    }
}
