//! Solana transaction signer implementation
//! 
//! This module provides complete Solana transaction signing and sending capabilities
//! with proper keypair management and blockhash handling.

use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

use crate::signer::{SignerError, TransactionSigner};

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
            timestamp: Instant::now() - Duration::from_secs(60), // Force initial fetch
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
            .map_err(|e| SignerError::BlockhashError(format!("RPC error fetching blockhash: {}", e)))?;
            
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
}

impl LocalSolanaSigner {
    /// Create a new Solana signer from a base58-encoded private key
    pub fn new(private_key: String, rpc_url: String) -> Result<Self, SignerError> {
        let keypair = Keypair::from_base58_string(&private_key);
        
        let client = Arc::new(RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        ));
        
        let blockhash_cache = Arc::new(RwLock::new(BlockhashCache::new(client.clone())));
        
        Ok(Self {
            keypair: Arc::new(keypair),
            client,
            blockhash_cache,
        })
    }
    
    /// Create a new Solana signer from a Keypair (for testing)
    pub fn from_keypair(keypair: Keypair, rpc_url: String) -> Self {
        let client = Arc::new(RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        ));
        
        let blockhash_cache = Arc::new(RwLock::new(BlockhashCache::new(client.clone())));
        
        Self {
            keypair: Arc::new(keypair),
            client,
            blockhash_cache,
        }
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
impl TransactionSigner for LocalSolanaSigner {
    fn pubkey(&self) -> Option<String> {
        Some(self.get_pubkey().to_string())
    }
    
    fn address(&self) -> Option<String> {
        // For Solana, address and pubkey are the same
        self.pubkey()
    }
    
    async fn sign_and_send_solana_transaction(
        &self,
        tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        // Get recent blockhash
        let recent_blockhash = self.get_recent_blockhash().await?;
        tx.message.recent_blockhash = recent_blockhash;
        
        // Sign the transaction
        tx.try_sign(&[&*self.keypair], recent_blockhash)
            .map_err(|e| SignerError::SigningError(format!("Failed to sign transaction: {}", e)))?;
        
        // Send the transaction
        let signature = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let tx = tx.clone();
            move || client.send_and_confirm_transaction(&tx)
        })
        .await
        .map_err(|e| SignerError::TransactionFailed(format!("Failed to send transaction: {}", e)))?
        .map_err(|e| SignerError::TransactionFailed(format!("RPC error sending transaction: {}", e)))?;
        
        Ok(signature.to_string())
    }
    
    async fn sign_and_send_evm_transaction(
        &self,
        _tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation(
            "EVM transactions not supported by Solana signer".to_string()
        ))
    }
    
    fn solana_client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
        self.client.clone()
    }
    
    fn evm_client(&self) -> Result<Arc<dyn std::any::Any + Send + Sync>, SignerError> {
        Err(SignerError::UnsupportedOperation(
            "EVM client not available for Solana signer".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_solana_signer_creation() {
        // Test keypair (DO NOT USE IN PRODUCTION)
        let keypair = Keypair::new();
        let private_key = keypair.to_base58_string();
        let rpc_url = "https://api.mainnet-beta.solana.com".to_string();
        
        let signer = LocalSolanaSigner::new(private_key, rpc_url);
        assert!(signer.is_ok());
        
        let signer = signer.unwrap();
        let pubkey = signer.pubkey();
        assert!(pubkey.is_some());
        assert_eq!(pubkey.unwrap(), keypair.pubkey().to_string());
    }
}