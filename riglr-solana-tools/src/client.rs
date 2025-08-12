//! Solana client for interacting with the Solana blockchain

use crate::error::{Result, SolanaToolError};
use reqwest::Client;
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    transaction::Transaction,
};
use solana_transaction_status::EncodedTransactionWithStatusMeta;
use spl_token;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

/// Configuration for Solana RPC client
#[derive(Debug, Clone)]
pub struct SolanaConfig {
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
    /// Optional signer for transactions
    pub signer: Option<Arc<Keypair>>,
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
            signer: None,
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

    /// Configure client with a keypair signer for transactions
    pub fn with_signer(mut self, keypair: Keypair) -> Self {
        self.signer = Some(Arc::new(keypair));
        self
    }

    /// Configure client with a signer from private key bytes
    pub fn with_signer_from_bytes(self, private_key_bytes: &[u8]) -> Result<Self> {
        if private_key_bytes.len() != 64 {
            return Err(SolanaToolError::InvalidKey(
                "Private key must be 64 bytes".to_string(),
            ));
        }
        let keypair = Keypair::try_from(private_key_bytes)
            .map_err(|e| SolanaToolError::InvalidKey(format!("Invalid private key: {}", e)))?;
        Ok(self.with_signer(keypair))
    }

    /// Get reference to the signer if configured
    pub fn signer(&self) -> Option<&Arc<Keypair>> {
        self.signer.as_ref()
    }

    /// Check if client has a signer configured
    pub fn has_signer(&self) -> bool {
        self.signer.is_some()
    }

    /// Get signer or return error if not configured
    pub fn require_signer(&self) -> Result<&Arc<Keypair>> {
        self.signer
            .as_ref()
            .ok_or_else(|| SolanaToolError::Generic("Client requires signer configuration".to_string()))
    }

    /// Create a SolanaClient from a TransactionSigner
    pub fn from_signer(signer: &dyn riglr_core::signer::TransactionSigner) -> Result<Self> {
        let client = signer.solana_client();
        
        // Extract configuration from the RPC client
        // This is a simplified approach - in practice you might want to store config in the signer
        Ok(SolanaClient {
            rpc_client: client,
            http_client: reqwest::Client::new(),
            config: SolanaConfig::default(), // Use default config for now
            signer: None, // The signer is managed by the context
        })
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

        // Get the first token account's pubkey
        let token_account_pubkey = Pubkey::from_str(&accounts[0].pubkey)
            .map_err(|e| SolanaToolError::InvalidAddress(format!("Invalid token account pubkey: {}", e)))?;

        // Get the full account data
        let account_info = self
            .rpc_client
            .get_account(&token_account_pubkey)
            .map_err(|e| SolanaToolError::Rpc(format!("Failed to get token account: {}", e)))?;

        // Parse the token account data using spl_token
        let token_account = spl_token::state::Account::unpack(&account_info.data)
            .map_err(|e| SolanaToolError::Generic(format!("Failed to parse token account data: {}", e)))?;

        info!("Token balance for {} (mint: {}): {}", address, mint, token_account.amount);
        Ok(token_account.amount)
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
        let json = serde_json::to_value(transaction).map_err(SolanaToolError::Serialization)?;

        Ok(json)
    }

    /// Get the current block height
    pub async fn get_block_height(&self) -> Result<u64> {
        debug!("Getting current block height");

        let height = self
            .rpc_client
            .get_block_height()
            .map_err(|e| SolanaToolError::Rpc(e.to_string()))?;

        info!("Current block height: {}", height);
        Ok(height)
    }

    /// Get transaction status by signature
    pub async fn get_signature_statuses(
        &self,
        signatures: &[String],
    ) -> Result<Vec<Option<solana_transaction_status::TransactionStatus>>> {
        let sigs: Result<Vec<Signature>> = signatures
            .iter()
            .map(|s| {
                Signature::from_str(s)
                    .map_err(|e| SolanaToolError::Generic(format!("Invalid signature: {}", e)))
            })
            .collect();

        let sigs = sigs?;

        debug!("Getting status for {} signatures", sigs.len());

        let statuses = self
            .rpc_client
            .get_signature_statuses(&sigs)
            .map_err(|e| SolanaToolError::Rpc(e.to_string()))?
            .value;

        Ok(statuses)
    }

    /// Get token accounts owned by the given address
    pub async fn get_token_accounts_by_owner(
        &self,
        owner: &str,
        mint: Option<&str>,
    ) -> Result<Vec<solana_client::rpc_response::RpcKeyedAccount>> {
        let owner_pubkey = Pubkey::from_str(owner).map_err(|e| {
            SolanaToolError::InvalidAddress(format!("Invalid owner address: {}", e))
        })?;

        let filter = if let Some(mint_str) = mint {
            let mint_pubkey = Pubkey::from_str(mint_str).map_err(|e| {
                SolanaToolError::InvalidAddress(format!("Invalid mint address: {}", e))
            })?;
            solana_client::rpc_request::TokenAccountsFilter::Mint(mint_pubkey)
        } else {
            solana_client::rpc_request::TokenAccountsFilter::ProgramId(spl_token::id())
        };

        debug!("Getting token accounts for owner: {}", owner);

        let accounts = self
            .rpc_client
            .get_token_accounts_by_owner(&owner_pubkey, filter)
            .map_err(|e| SolanaToolError::Rpc(e.to_string()))?;

        info!("Found {} token accounts for {}", accounts.len(), owner);
        Ok(accounts)
    }

    /// Send and confirm a transaction with retries
    pub async fn send_and_confirm_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<Signature> {
        debug!("Sending and confirming transaction");

        let signature = self
            .rpc_client
            .send_and_confirm_transaction(transaction)
            .map_err(|e| {
                error!("Transaction failed: {}", e);
                SolanaToolError::Transaction(e.to_string())
            })?;

        info!("Transaction confirmed: {}", signature);
        Ok(signature)
    }

    /// Get token account balance
    pub async fn get_token_account_balance(
        &self,
        token_account: &str,
    ) -> Result<solana_account_decoder::parse_token::UiTokenAmount> {
        let pubkey = Pubkey::from_str(token_account).map_err(|e| {
            SolanaToolError::InvalidAddress(format!("Invalid token account address: {}", e))
        })?;

        debug!("Getting balance for token account: {}", token_account);

        let balance = self
            .rpc_client
            .get_token_account_balance(&pubkey)
            .map_err(|e| SolanaToolError::Rpc(e.to_string()))?;

        info!(
            "Token account {} balance: {} (decimals: {})",
            token_account,
            balance.ui_amount.unwrap_or(0.0),
            balance.decimals
        );
        Ok(balance)
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
            .map_err(SolanaToolError::Http)?;

        let result: serde_json::Value = response.json().await.map_err(SolanaToolError::Http)?;

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

    /// Get transaction with full metadata for event parsing
    pub async fn get_transaction_with_meta(
        &self,
        signature: &str,
    ) -> Result<EncodedTransactionWithStatusMeta> {
        let sig = Signature::from_str(signature)
            .map_err(|e| SolanaToolError::InvalidSignature(signature.to_string()))?;

        let transaction = self
            .rpc_client
            .get_transaction_with_config(
                &sig,
                solana_client::rpc_config::RpcTransactionConfig {
                    encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(0),
                }
            )
            .map_err(|e| SolanaToolError::Rpc(e.to_string()))?;

        Ok(transaction)
    }

    /// Get recent transactions for a token (simplified implementation)
    pub async fn get_recent_transactions_for_token(
        &self,
        _token_address: &str,
        limit: usize,
    ) -> Result<Vec<EncodedTransactionWithStatusMeta>> {
        // Note: This is a simplified implementation. In production, you would:
        // 1. Query token account changes
        // 2. Find transactions that modified the token account
        // 3. Filter by recency and limit
        
        // For now, return empty vec - this would be implemented based on specific requirements
        debug!("get_recent_transactions_for_token called with limit: {}", limit);
        Ok(vec![])
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
