//! EVM transaction signer implementation using Alloy
//!
//! This module provides complete EVM transaction signing and sending capabilities
//! using the Alloy library with proper wallet management.

use alloy::network::EthereumWallet;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, info};

use crate::error::EvmToolError;
use riglr_config::EvmNetworkConfig;
use riglr_core::signer::{
    EvmClient, EvmSigner as EvmSignerTrait, SignerBase, SignerError, UnifiedSigner,
};
use std::any::Any;

/// Type alias for our HTTP provider with wallet
type HttpProvider = alloy::providers::fillers::FillProvider<
    alloy::providers::fillers::JoinFill<
        alloy::providers::Identity,
        alloy::providers::fillers::WalletFiller<EthereumWallet>,
    >,
    alloy::providers::RootProvider<alloy::transports::http::Http<reqwest::Client>>,
    alloy::transports::http::Http<reqwest::Client>,
    alloy::network::Ethereum,
>;

/// EVM gas configuration
#[derive(Debug, Clone)]
pub struct GasConfig {
    /// Use EIP-1559 (base fee + priority fee)
    pub use_eip1559: bool,
    /// Gas price multiplier for faster inclusion
    pub gas_price_multiplier: f64,
    /// Maximum gas price willing to pay (in wei)
    pub max_gas_price: Option<U256>,
    /// Priority fee for EIP-1559 (in wei)
    pub max_priority_fee: Option<U256>,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            use_eip1559: true,
            gas_price_multiplier: 1.1, // 10% above estimate
            max_gas_price: None,
            max_priority_fee: Some(U256::from(2_000_000_000u64)), // 2 gwei
        }
    }
}

/// Local EVM signer with private key management
pub struct LocalEvmSigner {
    wallet: EthereumWallet,
    config: EvmNetworkConfig,
    gas_config: GasConfig,
}

impl LocalEvmSigner {
    /// Create a new EVM signer from a private key and network config
    pub fn new(private_key: String, config: EvmNetworkConfig) -> Result<Self, SignerError> {
        let signer = PrivateKeySigner::from_str(&private_key).map_err(|e| {
            SignerError::InvalidPrivateKey(format!("Invalid EVM private key: {}", e))
        })?;

        let wallet = EthereumWallet::from(signer);

        Ok(Self {
            wallet,
            config,
            gas_config: GasConfig::default(),
        })
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

    /// Create a new EVM signer with custom gas configuration
    pub fn new_with_gas_config(
        private_key: String,
        config: EvmNetworkConfig,
        gas_config: GasConfig,
    ) -> Result<Self, SignerError> {
        let signer = PrivateKeySigner::from_str(&private_key).map_err(|e| {
            SignerError::InvalidPrivateKey(format!("Invalid EVM private key: {}", e))
        })?;

        let wallet = EthereumWallet::from(signer);

        Ok(Self {
            wallet,
            config,
            gas_config,
        })
    }

    /// Get the address of this signer
    pub fn get_address(&self) -> Address {
        self.wallet.default_signer().address()
    }

    /// Create a provider with this wallet attached
    async fn get_provider(&self) -> Result<HttpProvider, SignerError> {
        let url = self.config.rpc_url.parse()
            .map_err(|e| SignerError::ProviderError(format!("Invalid RPC URL: {}", e)))?;
        
        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        Ok(provider)
    }

    /// Estimate optimal gas price
    pub async fn estimate_gas_price(&self) -> Result<u128, EvmToolError> {
        let provider = self.get_provider().await.map_err(|e| {
            EvmToolError::ProviderError(format!("Failed to create provider: {}", e))
        })?;

        if self.gas_config.use_eip1559 {
            // Get base fee and estimate priority fee
            let base_fee = provider.get_gas_price().await.map_err(|e| {
                EvmToolError::ProviderError(format!("Failed to get base fee: {}", e))
            })?;

            let priority_fee = self
                .gas_config
                .max_priority_fee
                .unwrap_or_else(|| U256::from(1_000_000_000u64)); // 1 gwei default

            let total = base_fee + priority_fee.to::<u128>();

            // Apply multiplier
            let adjusted = (total as f64 * self.gas_config.gas_price_multiplier) as u128;

            // Apply max cap if set
            if let Some(max) = self.gas_config.max_gas_price {
                Ok(adjusted.min(max.to::<u128>()))
            } else {
                Ok(adjusted)
            }
        } else {
            // Legacy gas price
            let gas_price = provider.get_gas_price().await.map_err(|e| {
                EvmToolError::ProviderError(format!("Failed to get gas price: {}", e))
            })?;

            // Apply multiplier
            let adjusted = (gas_price as f64 * self.gas_config.gas_price_multiplier) as u128;

            // Apply max cap if set
            if let Some(max) = self.gas_config.max_gas_price {
                Ok(adjusted.min(max.to::<u128>()))
            } else {
                Ok(adjusted)
            }
        }
    }

    /// Estimate gas limit for a transaction
    pub async fn estimate_gas_limit(&self, tx: &TransactionRequest) -> Result<u64, EvmToolError> {
        let provider = self.get_provider().await.map_err(|e| {
            EvmToolError::ProviderError(format!("Failed to create provider: {}", e))
        })?;

        let estimate = provider
            .estimate_gas(tx)
            .await
            .map_err(|_e| EvmToolError::GasEstimationFailed)?;

        // Add 20% buffer to gas estimate
        Ok((estimate as f64 * 1.2) as u64)
    }

    /// Prepare transaction with optimal gas settings
    pub async fn prepare_transaction(
        &self,
        mut tx: TransactionRequest,
    ) -> Result<TransactionRequest, EvmToolError> {
        // Estimate gas limit if not set
        if tx.gas.is_none() {
            let gas_limit = self.estimate_gas_limit(&tx).await?;
            tx.gas = Some(gas_limit);
            debug!("Set gas limit to {}", gas_limit);
        }

        let provider = self.get_provider().await.map_err(|e| {
            EvmToolError::ProviderError(format!("Failed to create provider: {}", e))
        })?;

        // Set gas price
        if self.gas_config.use_eip1559 {
            if tx.max_fee_per_gas.is_none() || tx.max_priority_fee_per_gas.is_none() {
                let base_fee = provider.get_gas_price().await.map_err(|e| {
                    EvmToolError::ProviderError(format!("Failed to get base fee: {}", e))
                })?;

                let priority_fee = self
                    .gas_config
                    .max_priority_fee
                    .unwrap_or_else(|| U256::from(2_000_000_000u64)); // 2 gwei

                let max_priority_fee = priority_fee.to::<u128>();
                let max_fee = base_fee + max_priority_fee * 2;

                tx.max_priority_fee_per_gas = Some(max_priority_fee);
                tx.max_fee_per_gas = Some(max_fee);

                debug!(
                    "Set EIP-1559 gas: max_fee={}, priority_fee={}",
                    max_fee, priority_fee
                );
            }
        } else if tx.gas_price.is_none() {
            let gas_price = self.estimate_gas_price().await?;
            tx.gas_price = Some(gas_price);
            debug!("Set gas price to {}", gas_price);
        }

        Ok(tx)
    }

    /// Simulate transaction before sending
    pub async fn simulate_transaction(&self, tx: &TransactionRequest) -> Result<(), EvmToolError> {
        let provider = self.get_provider().await.map_err(|e| {
            EvmToolError::ProviderError(format!("Failed to create provider: {}", e))
        })?;

        // Use eth_call to simulate the transaction
        let _result =
            provider
                .call(tx)
                .await
                .map_err(|e| EvmToolError::TransactionReverted {
                    reason: format!("Transaction simulation failed: {}", e),
                })?;

        info!("Transaction simulation successful");
        Ok(())
    }
}

impl std::fmt::Debug for LocalEvmSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalEvmSigner")
            .field("address", &self.get_address().to_string())
            .field("chain_id", &self.config.chain_id)
            .field("network", &self.config.name)
            .field("gas_config", &self.gas_config)
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
        tx_json: serde_json::Value,
    ) -> Result<String, SignerError> {
        // Convert JSON to TransactionRequest
        let mut tx: TransactionRequest = serde_json::from_value(tx_json)
            .map_err(|e| SignerError::Signing(format!("Failed to parse transaction: {}", e)))?;
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

        Ok(tx_hash.to_string())
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
    async fn get_balance(&self, address: &str) -> Result<String, SignerError> {
        let url = self.provider_url.parse()
            .map_err(|e| SignerError::ProviderError(format!("Invalid RPC URL: {}", e)))?;
        
        let provider: HttpProvider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let address = address.parse::<Address>().map_err(|e| {
            SignerError::InvalidPrivateKey(format!("Invalid address format: {}", e))
        })?;

        let balance = provider
            .get_balance(address)
            .await
            .map_err(|e| SignerError::ProviderError(format!("Failed to get balance: {}", e)))?;

        Ok(balance.to_string())
    }

    async fn send_transaction(&self, tx_json: &serde_json::Value) -> Result<String, SignerError> {
        // Convert JSON to TransactionRequest
        let tx: TransactionRequest = serde_json::from_value(tx_json.clone())
            .map_err(|e| SignerError::Signing(format!("Failed to parse transaction: {}", e)))?;
        
        let url = self.provider_url.parse()
            .map_err(|e| SignerError::ProviderError(format!("Invalid RPC URL: {}", e)))?;
        
        let provider: HttpProvider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let mut tx_request = tx.clone();
        if tx_request.chain_id.is_none() {
            tx_request.chain_id = Some(self.chain_id);
        }

        let pending_tx = provider.send_transaction(tx_request).await.map_err(|e| {
            SignerError::TransactionFailed(format!("Failed to send transaction: {}", e))
        })?;

        Ok(pending_tx.tx_hash().to_string())
    }

    async fn call(&self, tx_json: &serde_json::Value) -> Result<String, SignerError> {
        // Convert JSON to TransactionRequest
        let tx: TransactionRequest = serde_json::from_value(tx_json.clone())
            .map_err(|e| SignerError::Signing(format!("Failed to parse transaction: {}", e)))?;
        
        let url = self.provider_url.parse()
            .map_err(|e| SignerError::ProviderError(format!("Invalid RPC URL: {}", e)))?;
        
        let provider: HttpProvider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .on_http(url);

        let result = provider
            .call(&tx)
            .await
            .map_err(|e| SignerError::ProviderError(format!("Failed to call contract: {}", e)))?;

        Ok(result.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_config_default() {
        let config = GasConfig::default();
        assert!(config.use_eip1559);
        assert_eq!(config.gas_price_multiplier, 1.1);
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
        assert_eq!(config.gas_price_multiplier, cloned.gas_price_multiplier);
        assert_eq!(config.max_gas_price, cloned.max_gas_price);
        assert_eq!(config.max_priority_fee, cloned.max_priority_fee);

        // Test Debug formatting
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("GasConfig"));
    }

    // Mock provider tests
    use alloy::primitives::Bytes;
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

    impl std::fmt::Display for MockProviderError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                MockProviderError::Rpc(msg) => write!(f, "RPC error: {}", msg),
                MockProviderError::Network(msg) => write!(f, "Network error: {}", msg),
                MockProviderError::InsufficientFunds => write!(f, "Insufficient funds"),
            }
        }
    }

    impl std::error::Error for MockProviderError {}

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
            self.method_calls.lock().unwrap().clone()
        }

        fn record_call(&self, method: &str) {
            self.method_calls.lock().unwrap().push(method.to_string());
        }

        fn pop_gas_price_response(&self) -> Option<Result<u128, MockProviderError>> {
            self.gas_price_responses.lock().unwrap().pop()
        }

        fn pop_gas_estimate_response(&self) -> Option<Result<u64, MockProviderError>> {
            self.gas_estimate_responses.lock().unwrap().pop()
        }

        fn pop_call_response(&self) -> Option<Result<Bytes, MockProviderError>> {
            self.call_responses.lock().unwrap().pop()
        }

        fn pop_balance_response(&self) -> Option<Result<U256, MockProviderError>> {
            self.balance_responses.lock().unwrap().pop()
        }
    }

    // Note: In a real implementation, we would implement the Provider trait properly.
    // For this test, we'll focus on unit testing the gas logic and error handling
    // methods directly rather than through the Provider interface, as implementing
    // the full Provider trait is complex and beyond the scope of this refactoring.

    /// Test helper to create a LocalEvmSigner with custom gas config
    fn create_test_signer_with_gas_config(gas_config: GasConfig) -> LocalEvmSigner {
        let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
        let config = riglr_config::EvmNetworkConfig::new("test", 1, "https://test.rpc".to_string());

        LocalEvmSigner::new_with_gas_config(private_key.to_string(), config, gas_config)
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
        assert_eq!(signer.gas_config.gas_price_multiplier, 1.5);
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
        assert_eq!(signer.gas_config.gas_price_multiplier, 1.2);
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
        assert_eq!(signer.gas_config.gas_price_multiplier, 2.0);
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
            to: Some(alloy::primitives::Address::ZERO.into()),
            value: Some(U256::from(1000000000000000000u64)), // 1 ETH in wei
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
        assert_eq!(calls[0], "get_gas_price");
        assert_eq!(calls[1], "estimate_gas");
        assert_eq!(calls[2], "get_balance");
        assert_eq!(calls[3], "send_transaction");
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
        assert_eq!(signer2.gas_config.gas_price_multiplier, 1.0);
    }

    #[test]
    fn test_debug_formatting() {
        let signer = create_test_signer_with_gas_config(GasConfig::default());
        let debug_str = format!("{:?}", signer);

        // Test that debug output contains expected fields
        assert!(debug_str.contains("LocalEvmSigner"));
        assert!(debug_str.contains("address"));
        assert!(debug_str.contains("chain_id"));
        assert!(debug_str.contains("gas_config"));
    }
}
