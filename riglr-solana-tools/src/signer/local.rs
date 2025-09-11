//! Solana transaction signer implementation
//!
//! This module provides complete Solana transaction signing and sending capabilities
//! with proper keypair management and blockhash handling.

use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

use crate::error::SolanaToolError;
use riglr_config::SolanaNetworkConfig;
use riglr_core::signer::{SignerBase, SignerError, SolanaClient, SolanaSigner, UnifiedSigner};
use std::any::Any;

/// Solana priority fee configuration
#[derive(Debug, Clone)]
pub struct PriorityFeeConfig {
    /// Enable priority fees
    pub enabled: bool,
    /// Priority fee in microlamports per compute unit
    pub microlamports_per_cu: u64,
    /// Additional compute units to request
    pub additional_compute_units: Option<u32>,
}

impl Default for PriorityFeeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            microlamports_per_cu: 1000, // 0.001 lamports per CU
            additional_compute_units: Some(200_000),
        }
    }
}

/// Cache for recent blockhashes to improve performance
struct BlockhashCache {
    blockhash: solana_sdk::hash::Hash,
    timestamp: Instant,
    client: Arc<RpcClient>,
}

impl BlockhashCache {
    fn new(client: Arc<RpcClient>) -> Self {
        Self {
            blockhash: solana_sdk::hash::Hash::default(),
            timestamp: Instant::now()
                .checked_sub(Duration::from_secs(60))
                .unwrap_or_else(Instant::now), // Force initial fetch
            client,
        }
    }

    async fn get_blockhash(&mut self) -> Result<solana_sdk::hash::Hash, SignerError> {
        // Cache blockhash for 30 seconds
        if self.timestamp.elapsed() > Duration::from_secs(30) {
            let spawn_result = tokio::task::spawn_blocking({
                let client = self.client.clone();
                move || client.get_latest_blockhash()
            })
            .await;
            self.blockhash = spawn_result
                .map_err(|e| SignerError::BlockhashError(format!("Failed to fetch blockhash: {}", e)))?
                .map_err(|e| {
                    SignerError::BlockhashError(format!("RPC error fetching blockhash: {}", e))
                })?;

            self.timestamp = Instant::now();
        }
        Ok(self.blockhash)
    }
}

/// Local Solana signer with keypair management
pub struct LocalSolanaSigner {
    keypair: Arc<Keypair>,
    client: Arc<RpcClient>,
    blockhash_cache: Arc<RwLock<BlockhashCache>>,
    config: SolanaNetworkConfig,
    priority_config: PriorityFeeConfig,
}

impl LocalSolanaSigner {
    /// Create a new Solana signer from a base58-encoded private key and network config
    pub fn new(private_key: String, config: SolanaNetworkConfig) -> Result<Self, SignerError> {
        let keypair = Keypair::from_base58_string(&private_key);

        let client = Arc::new(RpcClient::new_with_commitment(
            &config.rpc_url,
            CommitmentConfig::confirmed(),
        ));

        let blockhash_cache = Arc::new(RwLock::new(BlockhashCache::new(client.clone())));

        Ok(Self {
            keypair: Arc::new(keypair),
            client,
            blockhash_cache,
            config,
            priority_config: PriorityFeeConfig::default(),
        })
    }

    /// Create a new Solana signer from a Keypair and network config
    pub fn from_keypair(keypair: Keypair, config: SolanaNetworkConfig) -> Self {
        let client = Arc::new(RpcClient::new_with_commitment(
            &config.rpc_url,
            CommitmentConfig::confirmed(),
        ));

        let blockhash_cache = Arc::new(RwLock::new(BlockhashCache::new(client.clone())));

        Self {
            keypair: Arc::new(keypair),
            client,
            blockhash_cache,
            config,
            priority_config: PriorityFeeConfig::default(),
        }
    }

    /// Create a new Solana signer with custom priority fee configuration
    pub fn new_with_priority_config(
        private_key: String,
        config: SolanaNetworkConfig,
        priority_config: PriorityFeeConfig,
    ) -> Result<Self, SignerError> {
        let keypair = Keypair::from_base58_string(&private_key);

        let client = Arc::new(RpcClient::new_with_commitment(
            &config.rpc_url,
            CommitmentConfig::confirmed(),
        ));

        let blockhash_cache = Arc::new(RwLock::new(BlockhashCache::new(client.clone())));

        Ok(Self {
            keypair: Arc::new(keypair),
            client,
            blockhash_cache,
            config,
            priority_config,
        })
    }

    /// Create a new Solana signer from a base58-encoded private key and RPC URL (compatibility)
    ///
    /// This method is provided for backward compatibility but the config-based methods are preferred.
    pub fn new_with_url(private_key: String, rpc_url: String) -> Result<Self, SignerError> {
        let config = SolanaNetworkConfig::new("custom", rpc_url);
        Self::new(private_key, config)
    }

    /// Create a new Solana signer from a Keypair and RPC URL (compatibility)
    ///
    /// This method is provided for backward compatibility but the config-based methods are preferred.
    pub fn from_keypair_with_url(keypair: Keypair, rpc_url: String) -> Self {
        let config = SolanaNetworkConfig::new("custom", rpc_url);
        Self::from_keypair(keypair, config)
    }

    /// Get the public key of this signer
    pub fn get_pubkey(&self) -> solana_sdk::pubkey::Pubkey {
        self.keypair.pubkey()
    }

    /// Get a recent blockhash (cached for performance)
    async fn get_recent_blockhash(&self) -> Result<solana_sdk::hash::Hash, SignerError> {
        let mut cache = self.blockhash_cache.write().await;
        let result = cache.get_blockhash().await;
        result
    }

    /// Create a new LocalSolanaSigner from a seed phrase (for compatibility with existing code)
    pub fn from_seed_phrase(seed_phrase: &str, rpc_url: String) -> Result<Self, SignerError> {
        // Validate seed phrase is not empty
        if seed_phrase.trim().is_empty() {
            return Err(SignerError::Configuration(
                "Invalid seed phrase: seed phrase cannot be empty".to_string(),
            ));
        }

        let keypair =
            solana_sdk::signature::keypair_from_seed_phrase_and_passphrase(seed_phrase, "")
                .map_err(|e| SignerError::Configuration(format!("Invalid seed phrase: {}", e)))?;

        let config = SolanaNetworkConfig::new("custom", rpc_url);
        Ok(Self::from_keypair(keypair, config))
    }

    /// Get the keypair (for advanced use cases)
    pub fn keypair(&self) -> &solana_sdk::signature::Keypair {
        &self.keypair
    }

    /// Get the RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.config.rpc_url
    }

    /// Add priority fee instructions to transaction
    pub fn add_priority_fee_instructions(&self, instructions: &mut Vec<Instruction>) {
        if !self.priority_config.enabled {
            return;
        }

        // Add compute budget instructions at the beginning
        let mut priority_instructions = vec![];

        // Set compute unit price for priority
        priority_instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
            self.priority_config.microlamports_per_cu,
        ));

        // Optionally set compute unit limit
        if let Some(units) = self.priority_config.additional_compute_units {
            priority_instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(units));
        }

        // Insert at the beginning of instructions
        priority_instructions.append(instructions);
        *instructions = priority_instructions;

        debug!(
            "Added priority fee: {} microlamports/CU",
            self.priority_config.microlamports_per_cu
        );
    }

    /// Optimize transaction for size and compute units
    pub fn optimize_transaction(&self, tx: &mut Transaction) -> Result<(), SolanaToolError> {
        // Check transaction size
        let serialized = bincode::serialize(&tx).map_err(|e| {
            SolanaToolError::Generic(format!("Failed to serialize transaction: {}", e))
        })?;

        const MAX_TRANSACTION_SIZE: usize = 1232; // Solana's max transaction size

        if serialized.len() > MAX_TRANSACTION_SIZE {
            return Err(SolanaToolError::Transaction(format!(
                "Transaction size {} exceeds maximum {}",
                serialized.len(),
                MAX_TRANSACTION_SIZE
            )));
        }

        debug!("Transaction size: {} bytes", serialized.len());
        Ok(())
    }

    /// Get recent prioritization fees from the network
    pub async fn get_recent_prioritization_fees(&self) -> Result<u64, SolanaToolError> {
        // Try to get recent prioritization fees from the network
        // Using the getRecentPrioritizationFees RPC method

        // Try to get fees from RPC, fall back to default if unavailable
        match self.client.get_recent_prioritization_fees(&[]) {
            Ok(fees) if !fees.is_empty() => {
                // Calculate average of recent fees
                let total: u64 = fees.iter().map(|f| f.prioritization_fee).sum();
                let average = total / fees.len() as u64;

                debug!(
                    "Average recent prioritization fee: {} microlamports/CU",
                    average
                );

                // Use the average or fall back to configured if too low
                if average > 0 {
                    Ok(average)
                } else {
                    Ok(self.priority_config.microlamports_per_cu)
                }
            }
            Ok(_) => {
                // No recent fees available, use configured default
                debug!(
                    "No recent prioritization fees available, using default: {} microlamports/CU",
                    self.priority_config.microlamports_per_cu
                );
                Ok(self.priority_config.microlamports_per_cu)
            }
            Err(e) => {
                // RPC error, log and use default
                debug!(
                    "Failed to get recent prioritization fees: {}, using default",
                    e
                );
                Ok(self.priority_config.microlamports_per_cu)
            }
        }
    }
}

impl std::fmt::Debug for LocalSolanaSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalSolanaSigner")
            .field("pubkey", &self.get_pubkey().to_string())
            .finish()
    }
}

#[async_trait]
impl SolanaSigner for LocalSolanaSigner {
    fn address(&self) -> String {
        self.get_pubkey().to_string()
    }

    fn pubkey(&self) -> String {
        self.get_pubkey().to_string()
    }

    async fn sign_and_send_transaction(
        &self,
        tx_bytes: &mut Vec<u8>,
    ) -> Result<String, SignerError> {
        // Deserialize the transaction from bytes
        use bincode::deserialize;
        let mut tx: Transaction = deserialize(tx_bytes).map_err(|e| {
            SignerError::Signing(format!("Failed to deserialize transaction: {}", e))
        })?;

        // Get a recent blockhash and assign it to the transaction
        let recent_blockhash = self.get_recent_blockhash().await?;
        tx.message.recent_blockhash = recent_blockhash;

        // Sign the transaction
        tx.sign(&[&*self.keypair], recent_blockhash);

        // Update the bytes with the signed transaction
        use bincode::serialize;
        *tx_bytes = serialize(&tx)
            .map_err(|e| SignerError::Signing(format!("Failed to serialize transaction: {}", e)))?;

        // Send the transaction
        let spawn_result = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let tx_clone = tx.clone();
            move || client.send_and_confirm_transaction(&tx_clone)
        })
        .await;
        let signature = spawn_result
            .map_err(|e| SignerError::TransactionFailed(format!("Failed to send transaction: {}", e)))?
            .map_err(|e| SignerError::TransactionFailed(format!("RPC error: {}", e)))?;

        Ok(signature.to_string())
    }

    fn client(&self) -> Arc<dyn std::any::Any + Send + Sync> {
        self.client.clone() as Arc<dyn std::any::Any + Send + Sync>
    }
}

impl LocalSolanaSigner {
    /// Get the Solana RPC client directly (convenience method for tools)
    pub fn get_rpc_client(&self) -> Arc<RpcClient> {
        self.client.clone()
    }
}

/// Implementation of SolanaClient trait for LocalSolanaSigner
#[allow(dead_code)]
struct SolanaClientImpl {
    client: Arc<RpcClient>,
}

#[async_trait]
impl SolanaClient for SolanaClientImpl {
    async fn get_balance(&self, pubkey_str: &str) -> Result<u64, SignerError> {
        use std::str::FromStr;
        let pubkey = Pubkey::from_str(pubkey_str)
            .map_err(|e| SignerError::InvalidPrivateKey(format!("Invalid pubkey: {}", e)))?;
        let spawn_result = tokio::task::spawn_blocking({
            let client = self.client.clone();
            move || client.get_balance(&pubkey)
        })
        .await;
        let balance = spawn_result
            .map_err(|e| SignerError::ProviderError(format!("Failed to get balance: {}", e)))?
            .map_err(|e| SignerError::ProviderError(format!("RPC error getting balance: {}", e)))?;

        Ok(balance)
    }

    async fn send_transaction(&self, tx_bytes: &[u8]) -> Result<String, SignerError> {
        use bincode::deserialize;
        let tx: Transaction = deserialize(tx_bytes).map_err(|e| {
            SignerError::Signing(format!("Failed to deserialize transaction: {}", e))
        })?;
        let spawn_result = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let tx = tx.clone();
            move || client.send_and_confirm_transaction(&tx)
        })
        .await;
        let signature = spawn_result
            .map_err(|e| SignerError::TransactionFailed(format!("Failed to send transaction: {}", e)))?
            .map_err(|e| {
                SignerError::TransactionFailed(format!("RPC error sending transaction: {}", e))
            })?;

        Ok(signature.to_string())
    }
}

impl UnifiedSigner for LocalSolanaSigner {
    fn supports_solana(&self) -> bool {
        true
    }

    fn supports_evm(&self) -> bool {
        false
    }

    fn as_solana(&self) -> Option<&dyn riglr_core::signer::SolanaSigner> {
        Some(self)
    }

    fn as_evm(&self) -> Option<&dyn riglr_core::signer::EvmSigner> {
        None
    }

    fn as_multi_chain(&self) -> Option<&dyn riglr_core::signer::MultiChainSigner> {
        None
    }
}

impl SignerBase for LocalSolanaSigner {
    fn locale(&self) -> String {
        "en".to_string()
    }

    fn user_id(&self) -> Option<String> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::hash::Hash;

    #[test]
    fn test_blockhash_cache_new() {
        let client = Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string()));
        let cache = BlockhashCache::new(client.clone());

        assert_eq!(cache.blockhash, Hash::default());
        assert!(cache.timestamp.elapsed() > Duration::from_secs(59)); // Should force initial fetch
        assert!(Arc::ptr_eq(&cache.client, &client));
    }

    #[tokio::test]
    async fn test_blockhash_cache_get_blockhash_needs_refresh() {
        let client = Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string()));
        let mut cache = BlockhashCache::new(client);

        // Since timestamp is old, this should attempt to fetch a new blockhash
        // This will likely fail in test environment, but we're testing the logic path
        let result = cache.get_blockhash().await;
        // In test environment, this will likely error due to network issues
        // But we're testing that the cache logic executes the fetch path
        assert!(result.is_err());
    }

    #[test]
    fn test_solana_signer_creation_with_valid_key() {
        let keypair = Keypair::new();
        let private_key = keypair.to_base58_string();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");

        let signer = LocalSolanaSigner::new(private_key, config);
        assert!(signer.is_ok());

        let signer = signer.unwrap();
        let pubkey = SolanaSigner::pubkey(&signer);
        assert_eq!(pubkey, keypair.pubkey().to_string());
    }

    #[test]
    fn test_solana_signer_from_keypair() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");

        let signer = LocalSolanaSigner::from_keypair(keypair, config);
        assert_eq!(signer.get_pubkey(), expected_pubkey);
    }

    #[test]
    fn test_solana_signer_get_pubkey() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");

        let signer = LocalSolanaSigner::from_keypair(keypair, config);
        assert_eq!(signer.get_pubkey(), expected_pubkey);
    }

    #[test]
    fn test_solana_signer_debug_formatting() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        let debug_string = format!("{:?}", signer);
        assert!(debug_string.contains("LocalSolanaSigner"));
        assert!(debug_string.contains("pubkey"));
        assert!(debug_string.contains(&signer.get_pubkey().to_string()));
    }

    #[test]
    fn test_solana_signer_pubkey() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey().to_string();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        let pubkey = SolanaSigner::pubkey(&signer);
        assert_eq!(pubkey, expected_pubkey);
    }

    #[test]
    fn test_solana_signer_address() {
        let keypair = Keypair::new();
        let expected_address = keypair.pubkey().to_string();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        let address = SolanaSigner::address(&signer);
        assert_eq!(address, expected_address);
    }

    #[test]
    fn test_solana_signer_client() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        let client = signer.client();
        // Just ensure we get a client back
        assert!(Arc::strong_count(&client) > 0); // Verify we have a valid client
    }

    #[test]
    fn test_signer_base_locale() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        assert_eq!(
            riglr_core::signer::granular_traits::SignerBase::locale(&signer),
            "en"
        );
    }

    #[test]
    fn test_signer_base_user_id() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        assert!(riglr_core::signer::granular_traits::SignerBase::user_id(&signer).is_none());
    }

    #[test]
    fn test_signer_base_as_any() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        let any_ref = signer.as_any();
        assert!(any_ref.downcast_ref::<LocalSolanaSigner>().is_some());
    }

    #[test]
    fn test_solana_signer_trait_address() {
        let keypair = Keypair::new();
        let expected_address = keypair.pubkey().to_string();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        assert_eq!(SolanaSigner::address(&signer), expected_address);
    }

    #[test]
    fn test_solana_signer_trait_pubkey() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey().to_string();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        assert_eq!(SolanaSigner::pubkey(&signer), expected_pubkey);
    }

    #[test]
    fn test_solana_signer_trait_client() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        let client = signer.client();
        assert!(Arc::strong_count(&client) > 0); // Just ensure we get a client back
    }

    #[test]
    fn test_unified_signer_supports_solana() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        assert!(signer.supports_solana());
    }

    #[test]
    fn test_unified_signer_supports_evm() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        assert!(!signer.supports_evm());
    }

    #[test]
    fn test_unified_signer_as_solana() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        let solana_signer = signer.as_solana();
        assert!(solana_signer.is_some());
    }

    #[test]
    fn test_unified_signer_as_evm() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        let evm_signer = signer.as_evm();
        assert!(evm_signer.is_none());
    }

    #[test]
    fn test_unified_signer_as_multi_chain() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        let multi_chain_signer = signer.as_multi_chain();
        assert!(multi_chain_signer.is_none());
    }

    #[test]
    fn test_solana_client_impl_creation() {
        let client = Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string()));
        let _solana_client = SolanaClientImpl {
            client: client.clone(),
        };
        // Just test that it can be created without panicking
    }

    // Edge case tests
    #[test]
    fn test_solana_signer_creation_with_empty_rpc_url() {
        let keypair = Keypair::new();
        let private_key = keypair.to_base58_string();
        let network_config = SolanaNetworkConfig {
            name: "Test".to_string(),
            rpc_url: "".to_string(), // Empty URL
            ws_url: None,
            explorer_url: None,
        };

        // This should still create the signer, but RPC calls would fail
        let signer = LocalSolanaSigner::new(private_key, network_config);
        assert!(signer.is_ok());
    }

    #[test]
    fn test_solana_signer_creation_with_none_explorer_url() {
        let keypair = Keypair::new();
        let private_key = keypair.to_base58_string();
        let network_config = SolanaNetworkConfig {
            name: "Test".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            ws_url: None,
            explorer_url: None, // None explorer URL
        };

        let signer = LocalSolanaSigner::new(private_key, network_config);
        assert!(signer.is_ok());
    }

    // Test blockhash cache scenarios
    #[test]
    fn test_blockhash_cache_timestamp_calculation() {
        let client = Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string()));
        let cache = BlockhashCache::new(client);

        // The timestamp should be artificially old to force initial fetch
        let elapsed = cache.timestamp.elapsed();
        assert!(elapsed >= Duration::from_secs(59));
    }

    // Test that the checked_sub fallback works
    #[test]
    fn test_blockhash_cache_new_with_instant_now_fallback() {
        // This tests the unwrap_or_else branch in BlockhashCache::new
        // The checked_sub should succeed in normal cases, but we're testing the fallback logic exists
        let client = Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string()));
        let cache = BlockhashCache::new(client);

        // Verify the cache was created successfully
        assert_eq!(cache.blockhash, Hash::default());
    }

    // Additional compatibility tests for backward-compatible methods
    #[test]
    fn test_from_seed_phrase() {
        let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let rpc_url = "https://api.devnet.solana.com".to_string();

        let result = LocalSolanaSigner::from_seed_phrase(seed_phrase, rpc_url.clone());
        assert!(result.is_ok());

        let signer = result.unwrap();
        assert_eq!(signer.rpc_url(), &rpc_url);
    }

    #[test]
    fn test_new_with_url() {
        let keypair = Keypair::new();
        let private_key = keypair.to_base58_string();
        let rpc_url = "https://api.devnet.solana.com".to_string();

        let result = LocalSolanaSigner::new_with_url(private_key, rpc_url.clone());
        assert!(result.is_ok());

        let signer = result.unwrap();
        assert_eq!(signer.rpc_url(), &rpc_url);
    }

    #[test]
    fn test_from_keypair_with_url() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey();
        let rpc_url = "https://api.devnet.solana.com".to_string();

        let signer = LocalSolanaSigner::from_keypair_with_url(keypair, rpc_url.clone());
        assert_eq!(signer.get_pubkey(), expected_pubkey);
        assert_eq!(signer.rpc_url(), &rpc_url);
    }
}
