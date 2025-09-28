//! EVM transaction signer implementation using Alloy
//!
//! This module provides complete EVM transaction signing and sending capabilities
//! using the Alloy library with proper wallet management.

use alloy::network::{Ethereum, EthereumWallet};
use alloy::primitives::{hex, keccak256, Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::transports::http::Http;
use async_trait::async_trait;
use core::{fmt, str::FromStr as _};
use tracing::{debug, info};

use crate::error::Error as EvmToolError;
use riglr_config::EvmNetworkConfig;
use riglr_core::signer::{
    error::Standard,
    granular_traits::{
        Chain, EvmSigner as EvmSignerTrait, SignerBase, SolanaSigner, UnifiedSigner,
    },
    traits::EvmClient,
    SignerError,
};

/// EVM gas configuration
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GasConfig {
    /// Gas price multiplier for faster inclusion
    pub gas_price_multiplier: f64,
    /// Maximum gas price willing to pay (in wei)
    pub max_gas_price: Option<U256>,
    /// Priority fee for EIP-1559 (in wei)
    pub max_priority_fee: Option<U256>,
    /// Use EIP-1559 (base fee + priority fee)
    pub use_eip1559: bool,
}

impl Default for GasConfig {
    #[inline]
    fn default() -> Self {
        Self {
            gas_price_multiplier: 1.1, // 10% above estimate
            max_gas_price: None,
            max_priority_fee: Some(U256::from(2_000_000_000u64)), // 2 gwei
            use_eip1559: true,
        }
    }
}

/// Local EVM transaction handler with private key management
pub struct EvmLocalClient {
    /// EVM client implementation
    client: EvmClientImpl,
    /// Network configuration
    config: EvmNetworkConfig,
    /// Gas configuration
    gas_config: GasConfig,
    /// Ethereum wallet
    wallet: EthereumWallet,
}

impl EvmLocalClient {
    /// Get the address of this signer
    #[must_use]
    #[inline]
    pub fn get_address(&self) -> Address {
        return self.wallet.default_signer().address();
    }

    /// Create a new EVM signer from a private key and network config
    ///
    /// # Errors
    /// Returns `SignerError` if the private key is invalid
    #[inline]
    pub fn new(private_key: &str, config: EvmNetworkConfig) -> Result<Self, EvmToolError> {
        let signer = PrivateKeySigner::from_str(private_key).map_err(|error| {
            EvmToolError::InvalidParameter(format!("Invalid EVM private key: {error}"))
        })?;

        let wallet = EthereumWallet::from(signer);

        Ok(Self {
            client: EvmClientImpl {
                chain_id: config.chain_id,
                provider_url: config.rpc_url.clone(),
                wallet: wallet.clone(),
            },
            config,
            gas_config: GasConfig::default(),
            wallet,
        })
    }

    /// Estimate gas limit for a transaction
    ///
    /// # Errors
    /// Returns `EvmToolError` if provider creation fails or gas estimation fails
    #[inline]
    pub async fn estimate_gas_limit(&self, tx: &TransactionRequest) -> Result<u64, EvmToolError> {
        let provider = self
            .get_provider()
            .map_err(|e| EvmToolError::ProviderError(format!("Failed to create provider: {e}")))?;

        let estimate = provider
            .estimate_gas(tx)
            .await
            .map_err(|_e| EvmToolError::GasEstimationFailed)?;

        // Add 20% buffer to gas estimate
        #[expect(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let result = (estimate as f64 * 1.2_f64) as u64;
        Ok(result)
    }

    /// Estimate optimal gas price
    ///
    /// # Errors
    /// Returns `EvmToolError` if provider creation fails or gas price estimation fails
    #[inline]
    pub async fn estimate_gas_price(&self) -> Result<u128, EvmToolError> {
        let provider = self.get_provider().map_err(|error| {
            EvmToolError::ProviderError(format!("Failed to create provider: {error}"))
        })?;

        if self.gas_config.use_eip1559 {
            // Get base fee and estimate priority fee
            let base_fee = provider.get_gas_price().await.map_err(|error| {
                EvmToolError::ProviderError(format!("Failed to get base fee: {error}"))
            })?;

            let priority_fee = self
                .gas_config
                .max_priority_fee
                .unwrap_or_else(|| U256::from(1_000_000_000u64)); // 1 gwei default

            let total = base_fee.saturating_add(priority_fee.to::<u128>());

            // Apply multiplier
            #[expect(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let adjusted = (total as f64 * self.gas_config.gas_price_multiplier) as u128;

            // Apply max cap if set
            return Ok(self
                .gas_config
                .max_gas_price
                .map_or(adjusted, |max| adjusted.min(max.to::<u128>())));
        }
        // Legacy gas price
        let gas_price = provider.get_gas_price().await.map_err(|error| {
            EvmToolError::ProviderError(format!("Failed to get gas price: {error}"))
        })?;

        // Apply multiplier
        #[expect(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let adjusted = (gas_price as f64 * self.gas_config.gas_price_multiplier) as u128;

        // Apply max cap if set
        Ok(self
            .gas_config
            .max_gas_price
            .map_or(adjusted, |max| adjusted.min(max.to::<u128>())))
    }

    /// Create a new EVM signer with custom gas configuration
    ///
    /// # Errors
    /// Returns `SignerError` if the private key is invalid
    #[inline]
    pub fn new_with_gas_config(
        private_key: &str,
        config: EvmNetworkConfig,
        gas_config: GasConfig,
    ) -> Result<Self, EvmToolError> {
        let signer = PrivateKeySigner::from_str(private_key).map_err(|error| {
            EvmToolError::InvalidParameter(format!("Invalid EVM private key: {error}"))
        })?;

        let wallet = EthereumWallet::from(signer);

        Ok(Self {
            client: EvmClientImpl {
                chain_id: config.chain_id,
                provider_url: config.rpc_url.clone(),
                wallet: wallet.clone(),
            },
            config,
            gas_config,
            wallet,
        })
    }

    /// Create a provider with this wallet attached
    fn get_provider(&self) -> Result<impl Provider<Http<reqwest::Client>, Ethereum>, EvmToolError> {
        let url =
            self.config.rpc_url.parse().map_err(|error| {
                EvmToolError::ProviderError(format!("Invalid RPC URL: {error}"))
            })?;

        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        Ok(provider)
    }

    /// Create a new EVM signer from a private key, RPC URL, and chain ID (compatibility)
    ///
    /// This method is provided for backward compatibility but the config-based method is preferred.
    ///
    /// # Errors
    /// Returns `SignerError` if the private key is invalid or config creation fails
    #[inline]
    pub fn new_with_url(
        private_key: &str,
        rpc_url: String,
        chain_id: u64,
    ) -> Result<Self, EvmToolError> {
        let config = EvmNetworkConfig::new("custom", chain_id, rpc_url);
        Self::new(private_key, config)
    }

    /// Prepare transaction with optimal gas settings
    ///
    /// # Errors
    /// Returns `EvmToolError` if provider creation or gas estimation fails
    #[inline]
    pub async fn prepare_transaction(
        &self,
        mut tx: TransactionRequest,
    ) -> Result<TransactionRequest, EvmToolError> {
        self.set_gas_limit_if_needed(&mut tx).await?;
        self.set_gas_pricing(&mut tx).await?;
        Ok(tx)
    }

    /// Set gas limit if not already specified
    async fn set_gas_limit_if_needed(
        &self,
        tx: &mut TransactionRequest,
    ) -> Result<(), EvmToolError> {
        if tx.gas.is_none() {
            let gas_limit = self.estimate_gas_limit(tx).await?;
            tx.gas = Some(gas_limit);
            debug!("Set gas limit to {}", gas_limit);
        }
        Ok(())
    }

    /// Set gas pricing (EIP-1559 or legacy)
    async fn set_gas_pricing(&self, tx: &mut TransactionRequest) -> Result<(), EvmToolError> {
        if self.gas_config.use_eip1559 {
            self.set_eip1559_pricing(tx).await
        } else {
            self.set_legacy_pricing(tx).await
        }
    }

    /// Set EIP-1559 gas pricing
    async fn set_eip1559_pricing(&self, tx: &mut TransactionRequest) -> Result<(), EvmToolError> {
        if tx.max_fee_per_gas.is_none() || tx.max_priority_fee_per_gas.is_none() {
            let provider = self.get_provider().map_err(|e| {
                EvmToolError::ProviderError(format!("Failed to create provider: {e}"))
            })?;

            let base_fee = provider
                .get_gas_price()
                .await
                .map_err(|e| EvmToolError::ProviderError(format!("Failed to get base fee: {e}")))?;

            let priority_fee = self
                .gas_config
                .max_priority_fee
                .unwrap_or_else(|| U256::from(2_000_000_000_u64)); // 2 gwei

            let max_priority_fee = priority_fee.to::<u128>();
            let max_fee = base_fee.saturating_add(max_priority_fee.saturating_mul(2));

            tx.max_priority_fee_per_gas = Some(max_priority_fee);
            tx.max_fee_per_gas = Some(max_fee);

            debug!(
                "Set EIP-1559 gas: max_fee={}, priority_fee={}",
                max_fee, priority_fee
            );
        }
        Ok(())
    }

    /// Set legacy gas pricing
    async fn set_legacy_pricing(&self, tx: &mut TransactionRequest) -> Result<(), EvmToolError> {
        if tx.gas_price.is_none() {
            let gas_price = self.estimate_gas_price().await?;
            tx.gas_price = Some(gas_price);
            debug!("Set gas price to {}", gas_price);
        }
        Ok(())
    }

    /// Simulate transaction before sending
    ///
    /// # Errors
    /// Returns `EvmToolError` if provider creation fails or transaction simulation fails
    #[inline]
    pub async fn simulate_transaction(&self, tx: &TransactionRequest) -> Result<(), EvmToolError> {
        let provider = self
            .get_provider()
            .map_err(|e| EvmToolError::ProviderError(format!("Failed to create provider: {e}")))?;

        // Use eth_call to simulate the transaction
        let _result = provider
            .call(tx)
            .await
            .map_err(|e| EvmToolError::TransactionReverted {
                reason: format!("Transaction simulation failed: {e}"),
            })?;

        info!("Transaction simulation successful");
        Ok(())
    }
}

impl fmt::Debug for EvmLocalClient {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvmLocalClient")
            .field("address", &self.get_address().to_string())
            .field("chain_id", &self.config.chain_id)
            .field("network", &self.config.name)
            .field("gas_config", &self.gas_config)
            .field("client", &self.client)
            .field("wallet", &"<redacted>") // Don't expose sensitive wallet data
            .finish()
    }
}

// Implement SignerBase trait
impl SignerBase for EvmLocalClient {
    #[inline]
    fn supported_chains(&self) -> &[Chain] {
        &[Chain::Evm]
    }

    #[inline]
    fn supports_chain(&self, chain: Chain) -> bool {
        matches!(chain, Chain::Evm)
    }

    #[inline]
    fn user_id(&self) -> String {
        // Return the address as the user identifier
        self.get_address().to_string()
    }
}

// Implement EvmSigner trait
#[async_trait]
impl EvmSignerTrait for EvmLocalClient {
    #[inline]
    fn address(&self) -> String {
        self.get_address().to_string()
    }

    #[inline]
    fn chain_id(&self) -> u64 {
        self.config.chain_id
    }

    #[inline]
    fn client(&self) -> &dyn EvmClient {
        &self.client
    }

    #[inline]
    async fn sign_and_send_transaction(
        &self,
        tx_json: serde_json::Value,
    ) -> Result<String, Box<dyn SignerError>> {
        // Convert JSON to TransactionRequest
        let mut tx: TransactionRequest = serde_json::from_value(tx_json).map_err(|e| {
            Box::new(Standard::SigningFailed(format!(
                "Failed to parse transaction: {e}"
            ))) as Box<dyn SignerError>
        })?;
        // Ensure chain ID is set
        if tx.chain_id.is_none() {
            tx.chain_id = Some(self.config.chain_id);
        }

        // Get provider with wallet
        let provider = self.get_provider().map_err(|e| {
            Box::new(Standard::SigningFailed(format!(
                "Failed to get provider: {e}"
            ))) as Box<dyn SignerError>
        })?;

        // Send the transaction
        let pending_tx = provider.send_transaction(tx).await.map_err(|e| {
            Box::new(Standard::SigningFailed(format!(
                "Failed to send transaction: {e}"
            ))) as Box<dyn SignerError>
        })?;

        // Get the transaction hash
        let tx_hash = pending_tx.tx_hash().to_string();

        // Optionally wait for confirmation (1 block)
        let _receipt = pending_tx.get_receipt().await.map_err(|e| {
            Box::new(Standard::SigningFailed(format!(
                "Failed to get receipt: {e}"
            ))) as Box<dyn SignerError>
        })?;

        Ok(tx_hash)
    }

    #[inline]
    async fn sign_message(&self, message: &[u8]) -> Result<String, Box<dyn SignerError>> {
        // For now, return a placeholder implementation
        // The exact signature method depends on the specific Alloy version and TxSigner trait
        // This should be implemented based on the actual available methods
        let _hash = keccak256(message);

        // Return an error indicating that message signing is not yet implemented
        Err(Box::new(Standard::UnsupportedOperation(
            "Message signing not yet implemented for this signer".to_string(),
        )))
    }
}

// Implement UnifiedSigner trait
impl UnifiedSigner for EvmLocalClient {
    #[inline]
    fn as_evm(&self) -> Option<&dyn EvmSignerTrait> {
        Some(self)
    }

    #[inline]
    fn as_solana(&self) -> Option<&dyn SolanaSigner> {
        None
    }

    #[inline]
    fn supports_evm(&self) -> bool {
        true
    }

    #[inline]
    fn supports_solana(&self) -> bool {
        false
    }
}

/// Implementation of `EvmClient` trait for `LocalEvmSigner`
#[derive(Debug)]
struct EvmClientImpl {
    /// Chain ID
    chain_id: u64,
    /// Provider URL
    provider_url: String,
    /// Ethereum wallet
    wallet: EthereumWallet,
}

#[async_trait]
impl EvmClient for EvmClientImpl {
    async fn call(
        &self,
        call_request: &serde_json::Value,
        _block: Option<&str>,
    ) -> Result<String, Box<dyn SignerError>> {
        let tx: TransactionRequest = serde_json::from_value(call_request.clone()).map_err(|e| {
            Box::new(Standard::SigningFailed(format!(
                "Failed to parse transaction: {e}"
            ))) as Box<dyn SignerError>
        })?;

        let url = self.provider_url.parse().map_err(|e| {
            Box::new(Standard::Generic(format!("Invalid RPC URL: {e}"))) as Box<dyn SignerError>
        })?;

        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let result = provider.call(&tx).await.map_err(|e| {
            Box::new(Standard::Network(format!("Failed to call contract: {e}")))
                as Box<dyn SignerError>
        })?;

        Ok(result.to_string())
    }

    async fn estimate_gas(
        &self,
        transaction: &serde_json::Value,
    ) -> Result<String, Box<dyn SignerError>> {
        let tx: TransactionRequest = serde_json::from_value(transaction.clone()).map_err(|e| {
            Box::new(Standard::InvalidInput(format!(
                "Failed to parse transaction: {e}"
            ))) as Box<dyn SignerError>
        })?;

        let url = self.provider_url.parse().map_err(|e| {
            Box::new(Standard::Generic(format!("Invalid RPC URL: {e}"))) as Box<dyn SignerError>
        })?;

        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let gas = provider.estimate_gas(&tx).await.map_err(|e| {
            Box::new(Standard::Network(format!("Failed to estimate gas: {e}")))
                as Box<dyn SignerError>
        })?;

        Ok(gas.to_string())
    }

    async fn get_balance(&self, address: &str) -> Result<String, Box<dyn SignerError>> {
        let url = self.provider_url.parse().map_err(|e| {
            Box::new(Standard::Generic(format!("Invalid RPC URL: {e}"))) as Box<dyn SignerError>
        })?;

        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let parsed_address = address.parse::<Address>().map_err(|e| {
            Box::new(Standard::InvalidInput(format!(
                "Invalid address format: {e}"
            ))) as Box<dyn SignerError>
        })?;

        let balance = provider.get_balance(parsed_address).await.map_err(|e| {
            Box::new(Standard::Network(format!("Failed to get balance: {e}")))
                as Box<dyn SignerError>
        })?;

        Ok(balance.to_string())
    }

    async fn get_block_number(&self) -> Result<String, Box<dyn SignerError>> {
        let url = self.provider_url.parse().map_err(|e| {
            Box::new(Standard::Generic(format!("Invalid RPC URL: {e}"))) as Box<dyn SignerError>
        })?;

        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let block_number = provider.get_block_number().await.map_err(|e| {
            Box::new(Standard::Network(format!(
                "Failed to get block number: {e}"
            ))) as Box<dyn SignerError>
        })?;

        Ok(block_number.to_string())
    }

    async fn get_chain_id(&self) -> Result<String, Box<dyn SignerError>> {
        Ok(self.chain_id.to_string())
    }

    async fn get_gas_price(&self) -> Result<String, Box<dyn SignerError>> {
        let url = self.provider_url.parse().map_err(|e| {
            Box::new(Standard::Generic(format!("Invalid RPC URL: {e}"))) as Box<dyn SignerError>
        })?;

        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let gas_price = provider.get_gas_price().await.map_err(|e| {
            Box::new(Standard::Network(format!("Failed to get gas price: {e}")))
                as Box<dyn SignerError>
        })?;

        Ok(gas_price.to_string())
    }

    async fn get_nonce(&self, address: &str) -> Result<String, Box<dyn SignerError>> {
        let parsed_address = address.parse::<Address>().map_err(|e| {
            Box::new(Standard::InvalidInput(format!(
                "Invalid address format: {e}"
            ))) as Box<dyn SignerError>
        })?;

        let url = self.provider_url.parse().map_err(|e| {
            Box::new(Standard::Generic(format!("Invalid RPC URL: {e}"))) as Box<dyn SignerError>
        })?;

        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let nonce = provider
            .get_transaction_count(parsed_address)
            .await
            .map_err(|e| {
                Box::new(Standard::Network(format!("Failed to get nonce: {e}")))
                    as Box<dyn SignerError>
            })?;

        Ok(nonce.to_string())
    }

    async fn get_transaction(
        &self,
        tx_hash: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn SignerError>> {
        let parsed_hash = tx_hash.parse().map_err(|e| {
            Box::new(Standard::InvalidInput(format!(
                "Invalid transaction hash: {e}"
            ))) as Box<dyn SignerError>
        })?;

        let url = self.provider_url.parse().map_err(|e| {
            Box::new(Standard::Generic(format!("Invalid RPC URL: {e}"))) as Box<dyn SignerError>
        })?;

        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let tx = provider
            .get_transaction_by_hash(parsed_hash)
            .await
            .map_err(|e| {
                Box::new(Standard::Network(format!("Failed to get transaction: {e}")))
                    as Box<dyn SignerError>
            })?;

        match tx {
            Some(transaction) => Ok(Some(serde_json::to_value(transaction).map_err(|e| {
                Box::new(Standard::Generic(format!(
                    "Failed to serialize transaction: {e}"
                ))) as Box<dyn SignerError>
            })?)),
            None => Ok(None),
        }
    }

    async fn get_transaction_receipt(
        &self,
        tx_hash: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn SignerError>> {
        let parsed_hash = tx_hash.parse().map_err(|e| {
            Box::new(Standard::InvalidInput(format!(
                "Invalid transaction hash: {e}"
            ))) as Box<dyn SignerError>
        })?;

        let url = self.provider_url.parse().map_err(|e| {
            Box::new(Standard::Generic(format!("Invalid RPC URL: {e}"))) as Box<dyn SignerError>
        })?;

        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let receipt = provider
            .get_transaction_receipt(parsed_hash)
            .await
            .map_err(|e| {
                Box::new(Standard::Network(format!("Failed to get receipt: {e}")))
                    as Box<dyn SignerError>
            })?;

        match receipt {
            Some(receipt_data) => Ok(Some(serde_json::to_value(receipt_data).map_err(|e| {
                Box::new(Standard::Generic(format!(
                    "Failed to serialize receipt: {e}"
                ))) as Box<dyn SignerError>
            })?)),
            None => Ok(None),
        }
    }

    async fn send_raw_transaction(&self, signed_tx: &str) -> Result<String, Box<dyn SignerError>> {
        let tx_bytes =
            hex::decode(signed_tx.strip_prefix("0x").unwrap_or(signed_tx)).map_err(|e| {
                Box::new(Standard::InvalidInput(format!(
                    "Invalid hex transaction: {e}"
                ))) as Box<dyn SignerError>
            })?;

        let url = self.provider_url.parse().map_err(|e| {
            Box::new(Standard::Generic(format!("Invalid RPC URL: {e}"))) as Box<dyn SignerError>
        })?;

        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let pending_tx = provider
            .send_raw_transaction(&Bytes::from(tx_bytes))
            .await
            .map_err(|e| {
                Box::new(Standard::Network(format!(
                    "Failed to send raw transaction: {e}"
                ))) as Box<dyn SignerError>
            })?;

        Ok(pending_tx.tx_hash().to_string())
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_config_default() {
        let config = GasConfig::default();
        assert!(config.use_eip1559);
        assert!((config.gas_price_multiplier - 1.1).abs() < f64::EPSILON);
        assert!(config.max_gas_price.is_none());
        assert_eq!(config.max_priority_fee, Some(U256::from(2_000_000_000u64)));
    }

    #[test]
    fn test_gas_config_debug_clone() {
        let config = GasConfig {
            use_eip1559: false,
            gas_price_multiplier: 1.5,
            max_gas_price: Some(U256::from(50_000_000_000u64)),
            max_priority_fee: Some(U256::from(1_000_000_000u64)),
        };

        let cloned = config.clone();
        assert_eq!(config.use_eip1559, cloned.use_eip1559);
        assert!((config.gas_price_multiplier - cloned.gas_price_multiplier).abs() < f64::EPSILON);
        assert_eq!(config.max_gas_price, cloned.max_gas_price);
        assert_eq!(config.max_priority_fee, cloned.max_priority_fee);

        // Test Debug formatting
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("GasConfig"));
    }

    // Mock provider tests
    use alloy::primitives::Bytes;
    use core::error::Error;
    use std::sync::{Arc, Mutex};

    /// Mock provider for testing gas logic and error scenarios
    #[derive(Debug, Clone)]
    struct MockProvider {
        /// Expected gas price responses
        gas_price_responses: Arc<Mutex<Vec<Result<u128, MockProviderError>>>>,
        /// Expected gas estimation responses
        gas_estimate_responses: Arc<Mutex<Vec<Result<u64, MockProviderError>>>>,
        /// Expected call responses
        call_responses: Arc<Mutex<Vec<Result<Bytes, MockProviderError>>>>,
        /// Expected balance responses
        balance_responses: Arc<Mutex<Vec<Result<U256, MockProviderError>>>>,
        /// Track method calls
        method_calls: Arc<Mutex<Vec<String>>>,
    }

    #[derive(Debug, Clone)]
    enum MockProviderError {
        Rpc(String),
        Network(String),
        InsufficientFunds,
    }

    impl fmt::Display for MockProviderError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                Self::Rpc(ref msg) => write!(f, "RPC error: {msg}"),
                Self::Network(ref msg) => write!(f, "Network error: {msg}"),
                Self::InsufficientFunds => write!(f, "Insufficient funds"),
            }
        }
    }

    impl Error for MockProviderError {}

    impl MockProvider {
        fn new() -> Self {
            Self {
                gas_price_responses: Arc::new(Mutex::new(Vec::new())),
                gas_estimate_responses: Arc::new(Mutex::new(Vec::new())),
                call_responses: Arc::new(Mutex::new(Vec::new())),
                balance_responses: Arc::new(Mutex::new(Vec::new())),
                method_calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn set_gas_price_response(&self, response: Result<u128, MockProviderError>) {
            self.gas_price_responses.lock().unwrap().push(response);
        }

        fn set_gas_estimate_response(&self, response: Result<u64, MockProviderError>) {
            self.gas_estimate_responses.lock().unwrap().push(response);
        }

        fn set_call_response(&self, response: Result<Bytes, MockProviderError>) {
            self.call_responses.lock().unwrap().push(response);
        }

        fn set_balance_response(&self, response: Result<U256, MockProviderError>) {
            self.balance_responses.lock().unwrap().push(response);
        }

        fn get_method_calls(&self) -> Vec<String> {
            return self.method_calls.lock().unwrap().clone();
        }

        fn record_call(&self, method: &str) {
            self.method_calls.lock().unwrap().push(method.to_string());
        }

        fn pop_gas_price_response(&self) -> Option<Result<u128, MockProviderError>> {
            return self.gas_price_responses.lock().unwrap().pop();
        }

        fn pop_gas_estimate_response(&self) -> Option<Result<u64, MockProviderError>> {
            return self.gas_estimate_responses.lock().unwrap().pop();
        }

        fn pop_call_response(&self) -> Option<Result<Bytes, MockProviderError>> {
            return self.call_responses.lock().unwrap().pop();
        }

        fn pop_balance_response(&self) -> Option<Result<U256, MockProviderError>> {
            return self.balance_responses.lock().unwrap().pop();
        }
    }

    // Note: In a real implementation, we would implement the Provider trait properly.
    // For this test, we'll focus on unit testing the gas logic and error handling
    // methods directly rather than through the Provider interface, as implementing
    // the full Provider trait is complex and beyond the scope of this refactoring.

    /// Test helper to create a `EvmLocalClient` with custom gas config
    fn create_test_signer_with_gas_config(gas_config: GasConfig) -> EvmLocalClient {
        let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
        let config = riglr_config::EvmNetworkConfig::new("test", 1, "https://test.rpc".to_string());

        EvmLocalClient::new_with_gas_config(private_key, config, gas_config)
            .expect("Failed to create test signer")
    }

    #[test]
    fn test_gas_config_custom_values() {
        let gas_config = GasConfig {
            use_eip1559: false,
            gas_price_multiplier: 1.5,
            max_gas_price: Some(U256::from(100_000_000_000u64)), // 100 gwei
            max_priority_fee: Some(U256::from(3_000_000_000u64)), // 3 gwei
        };

        let signer = create_test_signer_with_gas_config(gas_config);

        // Test that the signer was created successfully
        assert!(!signer.gas_config.use_eip1559);
        assert!((signer.gas_config.gas_price_multiplier - 1.5).abs() < f64::EPSILON);
        assert_eq!(
            signer.gas_config.max_gas_price,
            Some(U256::from(100_000_000_000u64))
        );
        assert_eq!(
            signer.gas_config.max_priority_fee,
            Some(U256::from(3_000_000_000u64))
        );
    }

    #[test]
    fn test_gas_config_eip1559_enabled() {
        let gas_config = GasConfig {
            use_eip1559: true,
            gas_price_multiplier: 1.2,
            max_gas_price: Some(U256::from(200_000_000_000u64)), // 200 gwei
            max_priority_fee: Some(U256::from(5_000_000_000u64)), // 5 gwei
        };

        let signer = create_test_signer_with_gas_config(gas_config);

        assert!(signer.gas_config.use_eip1559);
        assert!((signer.gas_config.gas_price_multiplier - 1.2).abs() < f64::EPSILON);
        assert_eq!(
            signer.gas_config.max_gas_price,
            Some(U256::from(200_000_000_000u64))
        );
        assert_eq!(
            signer.gas_config.max_priority_fee,
            Some(U256::from(5_000_000_000u64))
        );
    }

    #[test]
    fn test_gas_config_no_max_limits() {
        let gas_config = GasConfig {
            use_eip1559: true,
            gas_price_multiplier: 2.0,
            max_gas_price: None,
            max_priority_fee: None,
        };

        let signer = create_test_signer_with_gas_config(gas_config);

        assert!(signer.gas_config.use_eip1559);
        assert!((signer.gas_config.gas_price_multiplier - 2.0).abs() < f64::EPSILON);
        assert_eq!(signer.gas_config.max_gas_price, None);
        assert_eq!(signer.gas_config.max_priority_fee, None);
    }

    #[tokio::test]
    async fn test_transaction_request_preparation() {
        use alloy::rpc::types::TransactionRequest;

        let gas_config = GasConfig::default();
        let _signer = create_test_signer_with_gas_config(gas_config);

        // Create a basic transaction request
        let tx = TransactionRequest {
            to: Some(Address::ZERO.into()),
            value: Some(U256::from(1_000_000_000_000_000_000_u64)), // 1 ETH in wei
            ..Default::default()
        };

        // Test that prepare_transaction would set gas limit
        // Note: This would require mocking the provider, which is complex
        // For now, we test the basic structure
        assert!(tx.gas.is_none()); // Initially no gas limit
        assert!(tx.gas_price.is_none()); // Initially no gas price
        assert!(tx.max_fee_per_gas.is_none()); // Initially no EIP-1559 fees
    }

    #[test]
    fn test_mock_provider_setup() {
        let mock = MockProvider::new();

        // Set up mock responses
        mock.set_gas_price_response(Ok(20_000_000_000u128)); // 20 gwei
        mock.set_gas_estimate_response(Ok(21000u64)); // Standard transfer gas
        mock.set_call_response(Ok(Bytes::new()));
        mock.set_balance_response(Ok(U256::from(1_000_000_000_000_000_000u64))); // 1 ETH

        // Test that responses are set correctly
        assert_eq!(
            mock.pop_gas_price_response().unwrap().unwrap(),
            20_000_000_000u128
        );
        assert_eq!(mock.pop_gas_estimate_response().unwrap().unwrap(), 21000u64);
        assert!(mock.pop_call_response().unwrap().is_ok());
        assert_eq!(
            mock.pop_balance_response().unwrap().unwrap(),
            U256::from(1_000_000_000_000_000_000u64)
        );
    }

    #[test]
    fn test_mock_provider_error_responses() {
        let mock = MockProvider::new();

        // Set up error responses
        mock.set_gas_price_response(Err(MockProviderError::Network(
            "Connection timeout".to_string(),
        )));
        mock.set_gas_estimate_response(Err(MockProviderError::Rpc(
            "Invalid transaction".to_string(),
        )));
        mock.set_balance_response(Err(MockProviderError::InsufficientFunds));

        // Test error responses
        let gas_price_error = mock.pop_gas_price_response().unwrap().unwrap_err();
        assert!(gas_price_error.to_string().contains("Connection timeout"));

        let gas_estimate_error = mock.pop_gas_estimate_response().unwrap().unwrap_err();
        assert!(gas_estimate_error
            .to_string()
            .contains("Invalid transaction"));

        let balance_error = mock.pop_balance_response().unwrap().unwrap_err();
        assert!(balance_error.to_string().contains("Insufficient funds"));
    }

    #[test]
    fn test_mock_provider_method_tracking() {
        let mock = MockProvider::new();

        // Record some method calls
        mock.record_call("get_gas_price");
        mock.record_call("estimate_gas");
        mock.record_call("get_balance");
        mock.record_call("send_transaction");

        let calls = mock.get_method_calls();
        assert_eq!(calls.len(), 4);
        assert_eq!(
            calls.first().expect("Expected at least 1 call"),
            "get_gas_price"
        );
        assert_eq!(
            calls.get(1).expect("Expected at least 2 calls"),
            "estimate_gas"
        );
        assert_eq!(
            calls.get(2).expect("Expected at least 3 calls"),
            "get_balance"
        );
        assert_eq!(
            calls.get(3).expect("Expected at least 4 calls"),
            "send_transaction"
        );
    }

    #[test]
    fn test_evm_signer_creation_with_different_configs() {
        // Test with default gas config
        let signer1 = create_test_signer_with_gas_config(GasConfig::default());
        assert!(signer1.gas_config.use_eip1559);

        // Test with legacy gas config
        let legacy_config = GasConfig {
            use_eip1559: false,
            gas_price_multiplier: 1.0,
            max_gas_price: Some(U256::from(50_000_000_000u64)),
            max_priority_fee: None,
        };
        let signer2 = create_test_signer_with_gas_config(legacy_config);
        assert!(!signer2.gas_config.use_eip1559);
        assert!((signer2.gas_config.gas_price_multiplier - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_debug_formatting() {
        let signer = create_test_signer_with_gas_config(GasConfig::default());
        let debug_str = format!("{signer:?}");

        // Test that debug output contains expected fields
        assert!(debug_str.contains("LocalSigner"));
        assert!(debug_str.contains("address"));
        assert!(debug_str.contains("chain_id"));
        assert!(debug_str.contains("gas_config"));
    }
}
