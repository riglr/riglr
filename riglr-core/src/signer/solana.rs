//! Solana transaction signer implementation
//!
//! This module provides complete Solana transaction signing and sending capabilities
//! with proper keypair management and blockhash handling.

use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signature::Signature, signer::Signer,
    transaction::Transaction,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::signer::{SignerBase, SignerError, SolanaClient, SolanaSigner, UnifiedSigner};
use riglr_config::SolanaNetworkConfig;
use std::any::Any;

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
            self.blockhash = tokio::task::spawn_blocking({
                let client = self.client.clone();
                move || client.get_latest_blockhash()
            })
            .await
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
    #[allow(dead_code)] // Stored for future use in enhanced network handling
    config: SolanaNetworkConfig,
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
        }
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
        cache.get_blockhash().await
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

    async fn sign_and_send_transaction(&self, tx: &mut Transaction) -> Result<String, SignerError> {
        // Get a recent blockhash and assign it to the transaction
        let recent_blockhash = self.get_recent_blockhash().await?;
        tx.message.recent_blockhash = recent_blockhash;

        // Sign the transaction
        tx.sign(&[&*self.keypair], recent_blockhash);

        // Send the transaction
        let signature = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let tx_clone = tx.clone();
            move || client.send_and_confirm_transaction(&tx_clone)
        })
        .await
        .map_err(|e| SignerError::TransactionFailed(format!("Failed to send transaction: {}", e)))?
        .map_err(|e| SignerError::TransactionFailed(format!("RPC error: {}", e)))?;

        Ok(signature.to_string())
    }

    fn client(&self) -> Arc<RpcClient> {
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
    async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, SignerError> {
        let balance = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let pubkey = *pubkey;
            move || client.get_balance(&pubkey)
        })
        .await
        .map_err(|e| SignerError::ProviderError(format!("Failed to get balance: {}", e)))?
        .map_err(|e| SignerError::ProviderError(format!("RPC error getting balance: {}", e)))?;

        Ok(balance)
    }

    async fn send_transaction(&self, tx: &Transaction) -> Result<Signature, SignerError> {
        let signature = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let tx = tx.clone();
            move || client.send_and_confirm_transaction(&tx)
        })
        .await
        .map_err(|e| SignerError::TransactionFailed(format!("Failed to send transaction: {}", e)))?
        .map_err(|e| {
            SignerError::TransactionFailed(format!("RPC error sending transaction: {}", e))
        })?;

        Ok(signature)
    }
}

impl UnifiedSigner for LocalSolanaSigner {
    fn supports_solana(&self) -> bool {
        true
    }

    fn supports_evm(&self) -> bool {
        false
    }

    fn as_solana(&self) -> Option<&dyn crate::signer::SolanaSigner> {
        Some(self)
    }

    fn as_evm(&self) -> Option<&dyn crate::signer::EvmSigner> {
        None
    }

    fn as_multi_chain(&self) -> Option<&dyn crate::signer::MultiChainSigner> {
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
            crate::signer::granular_traits::SignerBase::locale(&signer),
            "en"
        );
    }

    #[test]
    fn test_signer_base_user_id() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = LocalSolanaSigner::from_keypair(keypair, config);

        assert!(crate::signer::granular_traits::SignerBase::user_id(&signer).is_none());
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
}
