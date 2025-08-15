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

use crate::signer::{
    EvmClient, EvmSigner as EvmSignerTrait, SignerBase, SignerError, UnifiedSigner,
};
use riglr_config::EvmNetworkConfig;
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

    fn as_solana(&self) -> Option<&dyn crate::signer::granular_traits::SolanaSigner> {
        None
    }

    fn as_evm(&self) -> Option<&dyn EvmSignerTrait> {
        Some(self)
    }

    fn as_multi_chain(&self) -> Option<&dyn crate::signer::granular_traits::MultiChainSigner> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_private_key() -> String {
        // Test private key (DO NOT USE IN PRODUCTION)
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
    }

    fn get_test_network_config() -> EvmNetworkConfig {
        EvmNetworkConfig {
            name: "Ethereum Mainnet".to_string(),
            chain_id: 1,
            rpc_url: "https://eth.llamarpc.com".to_string(),
            explorer_url: Some("https://etherscan.io".to_string()),
            native_token: Some("ETH".to_string()),
        }
    }

    // Tests for LocalEvmSigner::new()
    #[test]
    fn test_new_when_valid_private_key_should_return_ok() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();

        let result = LocalEvmSigner::new(private_key, network_config.clone());
        assert!(result.is_ok());

        let signer = result.unwrap();
        assert_eq!(signer.config.chain_id, network_config.chain_id);
        assert_eq!(signer.config.rpc_url, network_config.rpc_url);
    }

    #[test]
    fn test_new_when_invalid_private_key_should_return_err() {
        let invalid_private_key = "invalid_key".to_string();
        let network_config = get_test_network_config();

        let result = LocalEvmSigner::new(invalid_private_key, network_config);
        assert!(result.is_err());

        if let Err(SignerError::InvalidPrivateKey(msg)) = result {
            assert!(msg.contains("Invalid EVM private key"));
        } else {
            panic!("Expected InvalidPrivateKey error");
        }
    }

    #[test]
    fn test_new_when_empty_private_key_should_return_err() {
        let empty_private_key = "".to_string();
        let network_config = get_test_network_config();

        let result = LocalEvmSigner::new(empty_private_key, network_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_when_private_key_wrong_format_should_return_err() {
        let wrong_format_key = "abc123".to_string();
        let network_config = get_test_network_config();

        let result = LocalEvmSigner::new(wrong_format_key, network_config);
        assert!(result.is_err());
    }

    // Tests for LocalEvmSigner::get_address()
    #[test]
    fn test_get_address_should_return_valid_address() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        let address = signer.get_address();
        assert_eq!(
            address.to_string(),
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        );
    }

    // Tests for Debug implementation
    #[test]
    fn test_debug_format_should_contain_address_and_chain_id() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        let debug_str = format!("{:?}", signer);
        assert!(debug_str.contains("LocalEvmSigner"));
        assert!(debug_str.contains("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"));
        assert!(debug_str.contains("chain_id"));
        assert!(debug_str.contains("1"));
    }

    // Tests for EvmSigner trait
    #[test]
    fn test_evm_signer_chain_id_should_return_u64() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        let chain_id = EvmSignerTrait::chain_id(&signer);
        assert_eq!(chain_id, 1);
    }

    #[test]
    fn test_evm_signer_address_should_return_string() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        let address = EvmSignerTrait::address(&signer);
        assert_eq!(address, "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
    }

    #[test]
    fn test_evm_client_should_return_ok() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        let result = EvmSignerTrait::client(&signer);
        assert!(result.is_ok());
    }

    // Tests for SignerBase trait
    #[test]
    fn test_signer_base_locale_should_return_en() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        assert_eq!(
            crate::signer::granular_traits::SignerBase::locale(&signer),
            "en"
        );
    }

    #[test]
    fn test_signer_base_user_id_should_return_none() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        assert!(crate::signer::granular_traits::SignerBase::user_id(&signer).is_none());
    }

    #[test]
    fn test_signer_base_as_any_should_return_self() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        let any_ref = signer.as_any();
        assert!(any_ref.downcast_ref::<LocalEvmSigner>().is_some());
    }

    // Tests for UnifiedSigner trait
    #[test]
    fn test_unified_signer_supports_solana_should_return_false() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        assert!(!signer.supports_solana());
    }

    #[test]
    fn test_unified_signer_supports_evm_should_return_true() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        assert!(signer.supports_evm());
    }

    #[test]
    fn test_unified_signer_as_solana_should_return_none() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        assert!(signer.as_solana().is_none());
    }

    #[test]
    fn test_unified_signer_as_evm_should_return_some() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        assert!(signer.as_evm().is_some());
    }

    #[test]
    fn test_unified_signer_as_multi_chain_should_return_none() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        assert!(signer.as_multi_chain().is_none());
    }

    // Test different network configurations
    #[test]
    fn test_different_chain_ids() {
        let private_key = get_test_private_key();

        // Test with different chain IDs
        let configs = vec![(1, "mainnet"), (5, "goerli"), (137, "polygon"), (56, "bsc")];

        for (chain_id, name) in configs {
            let network_config = EvmNetworkConfig {
                name: name.to_string(),
                chain_id,
                rpc_url: format!("https://{}.example.com", name),
                explorer_url: Some(format!("https://{}.etherscan.io", name)),
                native_token: Some("ETH".to_string()),
            };

            let signer = LocalEvmSigner::new(private_key.clone(), network_config);
            assert!(signer.is_ok());

            let signer = signer.unwrap();
            assert_eq!(signer.config.chain_id, chain_id);
            assert_eq!(EvmSignerTrait::chain_id(&signer), chain_id);
        }
    }

    // Test edge cases for address parsing
    #[test]
    fn test_address_format_consistency() {
        let private_key = get_test_private_key();
        let network_config = get_test_network_config();
        let signer = LocalEvmSigner::new(private_key, network_config).unwrap();

        let address1 = signer.get_address().to_string();
        let address2 = EvmSignerTrait::address(&signer);
        let address3 = EvmSignerTrait::address(&signer);

        assert_eq!(address1, address2);
        assert_eq!(address2, address3);
        assert!(address1.starts_with("0x"));
        assert_eq!(address1.len(), 42); // 0x + 40 hex characters
    }

    // Test multiple signers don't interfere
    #[test]
    fn test_multiple_signers_independence() {
        let private_key1 = get_test_private_key();
        let private_key2 = "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6";

        let network_config1 = EvmNetworkConfig {
            name: "Network 1".to_string(),
            chain_id: 1,
            rpc_url: "https://rpc1.example.com".to_string(),
            explorer_url: None,
            native_token: None,
        };

        let network_config2 = EvmNetworkConfig {
            name: "Network 2".to_string(),
            chain_id: 2,
            rpc_url: "https://rpc2.example.com".to_string(),
            explorer_url: None,
            native_token: None,
        };

        let signer1 = LocalEvmSigner::new(private_key1, network_config1).unwrap();
        let signer2 = LocalEvmSigner::new(private_key2.to_string(), network_config2).unwrap();

        assert_ne!(signer1.get_address(), signer2.get_address());
        assert_ne!(signer1.config.chain_id, signer2.config.chain_id);
        assert_ne!(signer1.config.rpc_url, signer2.config.rpc_url);
    }
}
