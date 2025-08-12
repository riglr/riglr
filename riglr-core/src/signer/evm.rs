//! EVM transaction signer implementation using Alloy
//! 
//! This module provides complete EVM transaction signing and sending capabilities
//! using the Alloy library with proper wallet management.

use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;

use crate::signer::{SignerError, TransactionSigner};

/// Local EVM signer with private key management
pub struct LocalEvmSigner {
    wallet: EthereumWallet,
    provider_url: String,
    chain_id: u64,
}

impl LocalEvmSigner {
    /// Create a new EVM signer from a private key
    pub fn new(private_key: String, provider_url: String, chain_id: u64) -> Result<Self, SignerError> {
        let signer = PrivateKeySigner::from_str(&private_key)
            .map_err(|e| SignerError::InvalidPrivateKey(format!("Invalid EVM private key: {}", e)))?;
        
        let wallet = EthereumWallet::from(signer);
        
        Ok(Self {
            wallet,
            provider_url,
            chain_id,
        })
    }
    
    /// Get the address of this signer
    pub fn get_address(&self) -> Address {
        self.wallet.default_signer().address()
    }
    
    /// Create a provider with this wallet attached
    async fn get_provider(&self) -> Result<impl Provider, SignerError> {
        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .connect(&self.provider_url)
            .await
            .map_err(|e| SignerError::ProviderError(format!("Failed to create provider: {}", e)))?;
        
        Ok(provider)
    }
}

impl std::fmt::Debug for LocalEvmSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalEvmSigner")
            .field("address", &self.get_address().to_string())
            .field("chain_id", &self.chain_id)
            .finish()
    }
}

#[async_trait]
impl TransactionSigner for LocalEvmSigner {
    fn address(&self) -> Option<String> {
        Some(self.get_address().to_string())
    }
    
    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation(
            "Solana transactions not supported by EVM signer".to_string()
        ))
    }
    
    async fn sign_and_send_evm_transaction(
        &self,
        mut tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        // Ensure chain ID is set
        if tx.chain_id.is_none() {
            tx.chain_id = Some(self.chain_id);
        }
        
        // Get provider with wallet
        let provider = self.get_provider().await?;
        
        // Send the transaction
        let pending_tx = provider
            .send_transaction(tx)
            .await
            .map_err(|e| SignerError::TransactionFailed(format!("Failed to send transaction: {}", e)))?;
        
        // Get the transaction hash
        let tx_hash = pending_tx.tx_hash().to_string();
        
        // Optionally wait for confirmation (1 block)
        let _receipt = pending_tx
            .get_receipt()
            .await
            .map_err(|e| SignerError::TransactionFailed(format!("Failed to get receipt: {}", e)))?;
        
        Ok(tx_hash)
    }
    
    fn solana_client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
        panic!("Solana client not available for EVM signer")
    }
    
    fn evm_client(&self) -> Result<Arc<dyn std::any::Any + Send + Sync>, SignerError> {
        // Return the provider URL as a simple client representation
        Ok(Arc::new(self.provider_url.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_evm_signer_creation() {
        // Test private key (DO NOT USE IN PRODUCTION)
        let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let provider_url = "https://eth.llamarpc.com".to_string();
        let chain_id = 1;
        
        let signer = LocalEvmSigner::new(private_key.to_string(), provider_url, chain_id);
        assert!(signer.is_ok());
        
        let signer = signer.unwrap();
        let address = signer.address();
        assert!(address.is_some());
        assert!(address.unwrap().starts_with("0x"));
    }
}