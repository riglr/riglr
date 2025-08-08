//! Solana client for interacting with the Solana blockchain

use crate::error::{Result, SolanaToolError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    transaction::Transaction,
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

/// Configuration for Solana RPC client
#[derive(Debug, Clone)]
pub struct SolanaConfig {
    /// RPC endpoint URL
    pub rpc_url: String,
    /// Commitment level for transactions
    pub commitment: CommitmentLevel,
    /// Request timeout
    pub timeout: Duration,
    /// Whether to skip preflight checks
    pub skip_preflight: bool,
}

impl Default for SolanaConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            commitment: CommitmentLevel::Confirmed,
            timeout: Duration::from_secs(30),
            skip_preflight: false,
        }
    }
}

/// A client for interacting with the Solana blockchain
#[derive(Clone)]
pub struct SolanaClient {
    /// Native Solana RPC client
    pub rpc_client: Arc<RpcClient>,
    /// HTTP client for custom requests
    pub http_client: Client,
    /// Configuration
    pub config: SolanaConfig,
}

impl SolanaClient {
    /// Create a new Solana client with the given configuration
    pub fn new(config: SolanaConfig) -> Self {
        let rpc_client = RpcClient::new_with_timeout_and_commitment(
            config.rpc_url.clone(),
            config.timeout,
            CommitmentConfig {
                commitment: config.commitment,
            },
        );

        Self {
            rpc_client: Arc::new(rpc_client),
            http_client: Client::builder()
                .timeout(config.timeout)
                .build()
                .unwrap_or_else(|_| Client::new()),
            config,
        }
    }

    /// Create a new Solana client with default mainnet configuration
    pub fn mainnet() -> Self {
        Self::new(SolanaConfig::default())
    }

    /// Create a new Solana client with devnet configuration
    pub fn devnet() -> Self {
        Self::new(SolanaConfig {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            ..Default::default()
        })
    }

    /// Create a new Solana client with testnet configuration
    pub fn testnet() -> Self {
        Self::new(SolanaConfig {
            rpc_url: "https://api.testnet.solana.com".to_string(),
            ..Default::default()
        })
    }

    /// Create a new Solana client with custom RPC URL
    pub fn with_rpc_url(rpc_url: impl Into<String>) -> Self {
        Self::new(SolanaConfig {
            rpc_url: rpc_url.into(),
            ..Default::default()
        })
    }

    /// Set commitment level
    pub fn with_commitment(mut self, commitment: CommitmentLevel) -> Self {
        self.config.commitment = commitment;
        // Recreate RPC client with new commitment
        self.rpc_client = Arc::new(RpcClient::new_with_timeout_and_commitment(
            self.config.rpc_url.clone(),
            self.config.timeout,
            CommitmentConfig { commitment },
        ));
        self
    }

    /// Get SOL balance for an address
    pub async fn get_balance(&self, address: &str) -> Result<u64> {
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| SolanaToolError::InvalidAddress(e.to_string()))?;

        debug!("Getting balance for address: {}", address);

        let balance = self
            .rpc_client
            .get_balance(&pubkey)
            .map_err(|e| SolanaToolError::Rpc(e.to_string()))?;

        info!("Balance for {}: {} lamports", address, balance);
        Ok(balance)
    }

    /// Get token balance for an address
    pub async fn get_token_balance(&self, address: &str, mint: &str) -> Result<u64> {
        let owner_pubkey = Pubkey::from_str(address).map_err(|e| {
            SolanaToolError::InvalidAddress(format!("Invalid owner address: {}", e))
        })?;

        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| SolanaToolError::InvalidAddress(format!("Invalid mint address: {}", e)))?;

        debug!(
            "Getting token balance for owner: {}, mint: {}",
            address, mint
        );

        // Get token accounts by owner
        let accounts = self
            .rpc_client
            .get_token_accounts_by_owner(
                &owner_pubkey,
                solana_client::rpc_request::TokenAccountsFilter::Mint(mint_pubkey),
            )
            .map_err(|e| SolanaToolError::Rpc(e.to_string()))?;

        if accounts.is_empty() {
            info!(
                "No token account found for owner: {}, mint: {}",
                address, mint
            );
            return Ok(0);
        }

        // Parse the first account's balance
        let account_data = &accounts[0].account.data;
        let token_amount = serde_json::from_str::<serde_json::Value>(&account_data)
            .map_err(|e| SolanaToolError::Serialization(e))?;

        let amount = token_amount["parsed"]["info"]["tokenAmount"]["amount"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        info!("Token balance for {} (mint: {}): {}", address, mint, amount);
        Ok(amount)
    }

    /// Get latest blockhash
    pub async fn get_latest_blockhash(&self) -> Result<String> {
        let blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .map_err(|e| SolanaToolError::Rpc(e.to_string()))?;

        Ok(blockhash.to_string())
    }

    /// Get transaction details
    pub async fn get_transaction(&self, signature: &str) -> Result<serde_json::Value> {
        let sig = Signature::from_str(signature)
            .map_err(|e| SolanaToolError::Generic(format!("Invalid signature: {}", e)))?;

        debug!("Getting transaction details for: {}", signature);

        let transaction = self
            .rpc_client
            .get_transaction(
                &sig,
                solana_transaction_status::UiTransactionEncoding::JsonParsed,
            )
            .map_err(|e| SolanaToolError::Rpc(e.to_string()))?;

        // Convert to JSON value
        let json =
            serde_json::to_value(transaction).map_err(|e| SolanaToolError::Serialization(e))?;

        Ok(json)
    }

    /// Send a transaction
    pub async fn send_transaction(&self, transaction: Transaction) -> Result<String> {
        debug!("Sending transaction");

        let signature = self
            .rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| {
                error!("Transaction failed: {}", e);
                SolanaToolError::Transaction(e.to_string())
            })?;

        let sig_str = signature.to_string();
        info!("Transaction sent successfully: {}", sig_str);
        Ok(sig_str)
    }

    /// Make a custom RPC call
    pub async fn call_rpc(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Making RPC call: {}", method);

        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = self
            .http_client
            .post(&self.config.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SolanaToolError::Http(e))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SolanaToolError::Http(e))?;

        if let Some(error) = result.get("error") {
            error!("RPC error: {:?}", error);
            return Err(SolanaToolError::Rpc(error.to_string()));
        }

        Ok(result.get("result").cloned().unwrap_or(json!(null)))
    }

    /// Check if the client is connected
    pub async fn is_connected(&self) -> bool {
        self.rpc_client.get_version().is_ok()
    }

    /// Get cluster info
    pub async fn get_cluster_info(&self) -> Result<serde_json::Value> {
        let version = self
            .rpc_client
            .get_version()
            .map_err(|e| SolanaToolError::Rpc(e.to_string()))?;

        let slot = self
            .rpc_client
            .get_slot()
            .map_err(|e| SolanaToolError::Rpc(e.to_string()))?;

        Ok(json!({
            "version": version,
            "slot": slot,
            "rpc_url": self.config.rpc_url,
            "commitment": format!("{:?}", self.config.commitment)
        }))
    }
}

impl Default for SolanaClient {
    fn default() -> Self {
        Self::mainnet()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = SolanaClient::mainnet();
        assert!(client.config.rpc_url.contains("mainnet"));

        let client = SolanaClient::devnet();
        assert!(client.config.rpc_url.contains("devnet"));

        let client = SolanaClient::testnet();
        assert!(client.config.rpc_url.contains("testnet"));
    }

    #[test]
    fn test_config() {
        let config = SolanaConfig {
            rpc_url: "https://custom.rpc.com".to_string(),
            commitment: CommitmentLevel::Finalized,
            timeout: Duration::from_secs(60),
            skip_preflight: true,
        };

        let client = SolanaClient::new(config.clone());
        assert_eq!(client.config.rpc_url, config.rpc_url);
        assert_eq!(client.config.commitment, config.commitment);
    }
}
