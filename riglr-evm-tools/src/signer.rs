//! EVM transaction signer implementation using Alloy
//!
//! This module provides complete EVM transaction signing and sending capabilities
//! using the Alloy library with proper wallet management.

use alloy::network::EthereumWallet;
use alloy::primitives::{Address, Bytes, TxHash, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;

use riglr_config::EvmNetworkConfig;
use riglr_core::signer::{
    EvmClient, EvmSigner as EvmSignerTrait, SignerBase, SignerError, UnifiedSigner,
};
use std::any::Any;

/// Local EVM signer with private key management
pub struct LocalEvmSigner {
    wallet: EthereumWallet,
    config: EvmNetworkConfig,
}

impl LocalEvmSigner {
    /// Create a new EVM signer from a private key and network config
    pub fn new(private_key: String, config: EvmNetworkConfig) -> Result<Self, SignerError> {
        let signer = PrivateKeySigner::from_str(&private_key).map_err(|e| {
            SignerError::InvalidPrivateKey(format!("Invalid EVM private key: {}", e))
        })?;

        let wallet = EthereumWallet::from(signer);

        Ok(Self { wallet, config })
    }

    /// Create a new EVM signer from a private key, RPC URL, and chain ID (compatibility)
    ///
    /// This method is provided for backward compatibility but the config-based method is preferred.
    pub fn new_with_url(
        private_key: String,
        rpc_url: String,
        chain_id: u64,
    ) -> Result<Self, SignerError> {
        let config = EvmNetworkConfig::new("custom", chain_id, rpc_url);
        Self::new(private_key, config)
    }

    /// Get the address of this signer
    pub fn get_address(&self) -> Address {
        self.wallet.default_signer().address()
    }

    /// Create a provider with this wallet attached
    async fn get_provider(&self) -> Result<impl Provider, SignerError> {
        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .connect(&self.config.rpc_url)
            .await
            .map_err(|e| SignerError::ProviderError(format!("Failed to create provider: {}", e)))?;

        Ok(provider)
    }
}

impl std::fmt::Debug for LocalEvmSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalEvmSigner")
            .field("address", &self.get_address().to_string())
            .field("chain_id", &self.config.chain_id)
            .field("network", &self.config.name)
            .finish()
    }
}

// Implement SignerBase trait
impl SignerBase for LocalEvmSigner {
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

// Implement EvmSigner trait
#[async_trait]
impl EvmSignerTrait for LocalEvmSigner {
    fn chain_id(&self) -> u64 {
        self.config.chain_id
    }

    fn address(&self) -> String {
        self.get_address().to_string()
    }

    async fn sign_and_send_transaction(
        &self,
        mut tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        // Ensure chain ID is set
        if tx.chain_id.is_none() {
            tx.chain_id = Some(self.config.chain_id);
        }

        // Get provider with wallet
        let provider = self.get_provider().await?;

        // Send the transaction
        let pending_tx = provider.send_transaction(tx).await.map_err(|e| {
            SignerError::TransactionFailed(format!("Failed to send transaction: {}", e))
        })?;

        // Get the transaction hash
        let tx_hash = pending_tx.tx_hash().to_string();

        // Optionally wait for confirmation (1 block)
        let _receipt = pending_tx
            .get_receipt()
            .await
            .map_err(|e| SignerError::TransactionFailed(format!("Failed to get receipt: {}", e)))?;

        Ok(tx_hash)
    }

    fn client(&self) -> Result<Arc<dyn EvmClient>, SignerError> {
        Ok(Arc::new(EvmClientImpl {
            wallet: self.wallet.clone(),
            provider_url: self.config.rpc_url.clone(),
            chain_id: self.config.chain_id,
        }))
    }
}

// Implement UnifiedSigner trait
impl UnifiedSigner for LocalEvmSigner {
    fn supports_solana(&self) -> bool {
        false
    }

    fn supports_evm(&self) -> bool {
        true
    }

    fn as_solana(&self) -> Option<&dyn riglr_core::signer::granular_traits::SolanaSigner> {
        None
    }

    fn as_evm(&self) -> Option<&dyn EvmSignerTrait> {
        Some(self)
    }

    fn as_multi_chain(&self) -> Option<&dyn riglr_core::signer::granular_traits::MultiChainSigner> {
        None
    }
}

/// Implementation of EvmClient trait for LocalEvmSigner
#[derive(Debug)]
struct EvmClientImpl {
    wallet: EthereumWallet,
    provider_url: String,
    chain_id: u64,
}

#[async_trait]
impl EvmClient for EvmClientImpl {
    async fn get_balance(&self, address: &str) -> Result<U256, SignerError> {
        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .connect(&self.provider_url)
            .await
            .map_err(|e| SignerError::ProviderError(format!("Failed to create provider: {}", e)))?;

        let address = address.parse::<Address>().map_err(|e| {
            SignerError::InvalidPrivateKey(format!("Invalid address format: {}", e))
        })?;

        let balance = provider
            .get_balance(address)
            .await
            .map_err(|e| SignerError::ProviderError(format!("Failed to get balance: {}", e)))?;

        Ok(balance)
    }

    async fn send_transaction(&self, tx: &TransactionRequest) -> Result<TxHash, SignerError> {
        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .connect(&self.provider_url)
            .await
            .map_err(|e| SignerError::ProviderError(format!("Failed to create provider: {}", e)))?;

        let mut tx_request = tx.clone();
        if tx_request.chain_id.is_none() {
            tx_request.chain_id = Some(self.chain_id);
        }

        let pending_tx = provider.send_transaction(tx_request).await.map_err(|e| {
            SignerError::TransactionFailed(format!("Failed to send transaction: {}", e))
        })?;

        Ok(*pending_tx.tx_hash())
    }

    async fn call(&self, tx: &TransactionRequest) -> Result<Bytes, SignerError> {
        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .connect(&self.provider_url)
            .await
            .map_err(|e| SignerError::ProviderError(format!("Failed to create provider: {}", e)))?;

        let result = provider
            .call(tx.clone())
            .await
            .map_err(|e| SignerError::ProviderError(format!("Failed to call contract: {}", e)))?;

        Ok(result)
    }
}
