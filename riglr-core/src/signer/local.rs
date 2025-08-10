use solana_sdk::signer::Signer;
use solana_client::rpc_client::RpcClient;
use async_trait::async_trait;
use std::sync::Arc;

use super::traits::TransactionSigner;
use super::error::SignerError;

/// A local Solana signer implementation that holds a keypair and RPC client.
/// This implementation is suitable for development, testing, and scenarios where
/// private keys can be safely managed in memory.
pub struct LocalSolanaSigner {
    keypair: solana_sdk::signature::Keypair,
    rpc_url: String,
    client: Arc<RpcClient>,
}

impl std::fmt::Debug for LocalSolanaSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalSolanaSigner")
            .field("pubkey", &self.keypair.pubkey().to_string())
            .field("rpc_url", &self.rpc_url)
            .finish_non_exhaustive() // Don't expose the keypair or client details
    }
}

impl LocalSolanaSigner {
    /// Create a new LocalSolanaSigner with the given keypair and RPC URL
    pub fn new(keypair: solana_sdk::signature::Keypair, rpc_url: String) -> Self {
        let client = Arc::new(RpcClient::new(rpc_url.clone()));
        Self { 
            keypair, 
            rpc_url, 
            client 
        }
    }
    
    /// Create a new LocalSolanaSigner from a seed phrase
    pub fn from_seed_phrase(seed_phrase: &str, rpc_url: String) -> Result<Self, SignerError> {
        let keypair = solana_sdk::signature::keypair_from_seed_phrase_and_passphrase(
            seed_phrase, 
            ""
        )
        .map_err(|e| SignerError::Configuration(format!("Invalid seed phrase: {}", e)))?;
        
        Ok(Self::new(keypair, rpc_url))
    }
    
    /// Get the keypair (for advanced use cases)
    pub fn keypair(&self) -> &solana_sdk::signature::Keypair {
        &self.keypair
    }
    
    /// Get the RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }
}

#[async_trait]
impl TransactionSigner for LocalSolanaSigner {
    fn address(&self) -> Option<String> {
        Some(self.keypair.pubkey().to_string())
    }
    
    fn pubkey(&self) -> Option<String> {
        Some(self.keypair.pubkey().to_string())
    }
    
    async fn sign_and_send_solana_transaction(
        &self,
        tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String, SignerError> {
        // Sign the transaction with our keypair
        tx.try_sign(&[&self.keypair], self.client.get_latest_blockhash().map_err(|e| SignerError::SolanaTransaction(Box::new(e)))?)
            .map_err(|e| SignerError::Signing(e.to_string()))?;
        
        // Send and confirm the transaction
        let signature = self.client.send_and_confirm_transaction(tx).map_err(|e| SignerError::SolanaTransaction(Box::new(e)))?;
        Ok(signature.to_string())
    }
    
    async fn sign_and_send_evm_transaction(
        &self,
        _tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String, SignerError> {
        // LocalSolanaSigner doesn't support EVM transactions
        Err(SignerError::Configuration(
            "LocalSolanaSigner does not support EVM transactions".to_string()
        ))
    }
    
    fn solana_client(&self) -> Arc<RpcClient> {
        self.client.clone()
    }
    
    fn evm_client(&self) -> Result<Box<dyn std::any::Any + Send + Sync>, SignerError> {
        // LocalSolanaSigner doesn't provide EVM client
        Err(SignerError::Configuration(
            "LocalSolanaSigner does not support EVM clients".to_string()
        ))
    }
}