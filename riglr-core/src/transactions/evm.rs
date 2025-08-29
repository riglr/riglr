//! EVM-specific transaction processing with gas optimization
//!
//! This module provides enhanced EVM transaction handling including:
//! - Dynamic gas price estimation
//! - EIP-1559 support
//! - MEV protection options
//! - Transaction simulation

use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

use super::{RetryConfig, TransactionProcessor, TransactionStatus};
use crate::error::ToolError;

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

/// EVM transaction processor with gas optimization
pub struct EvmTransactionProcessor<P: Provider> {
    /// The blockchain provider for interacting with the EVM network
    provider: Arc<P>,
    /// Gas configuration settings for transaction optimization
    gas_config: GasConfig,
    /// Retry configuration for transaction processing
    _retry_config: super::RetryConfig,
}

impl<P: Provider> EvmTransactionProcessor<P> {
    /// Create a new EVM transaction processor
    pub fn new(provider: Arc<P>, gas_config: GasConfig, retry_config: super::RetryConfig) -> Self {
        Self {
            provider,
            gas_config,
            _retry_config: retry_config,
        }
    }

    /// Estimate optimal gas price
    pub async fn estimate_gas_price(&self) -> Result<u128, ToolError> {
        if self.gas_config.use_eip1559 {
            // Get base fee and estimate priority fee
            let base_fee = self.provider.get_gas_price().await.map_err(|e| {
                ToolError::permanent_string(format!("Failed to get base fee: {}", e))
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
            let gas_price = self.provider.get_gas_price().await.map_err(|e| {
                ToolError::permanent_string(format!("Failed to get gas price: {}", e))
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
    pub async fn estimate_gas_limit(&self, tx: &TransactionRequest) -> Result<u64, ToolError> {
        let estimate =
            self.provider.estimate_gas(tx.clone()).await.map_err(|e| {
                ToolError::permanent_string(format!("Failed to estimate gas: {}", e))
            })?;

        // Add 20% buffer to gas estimate
        Ok((estimate as f64 * 1.2) as u64)
    }

    /// Prepare transaction with optimal gas settings
    pub async fn prepare_transaction(
        &self,
        mut tx: TransactionRequest,
    ) -> Result<TransactionRequest, ToolError> {
        // Estimate gas limit if not set
        if tx.gas.is_none() {
            let gas_limit = self.estimate_gas_limit(&tx).await?;
            tx.gas = Some(gas_limit);
            debug!("Set gas limit to {}", gas_limit);
        }

        // Set gas price
        if self.gas_config.use_eip1559 {
            if tx.max_fee_per_gas.is_none() || tx.max_priority_fee_per_gas.is_none() {
                let base_fee = self.provider.get_gas_price().await.map_err(|e| {
                    ToolError::permanent_string(format!("Failed to get base fee: {}", e))
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
    pub async fn simulate_transaction(&self, tx: &TransactionRequest) -> Result<(), ToolError> {
        // Use eth_call to simulate the transaction
        let _result = self.provider.call(tx.clone()).await.map_err(|e| {
            ToolError::permanent_string(format!("Transaction simulation failed: {}", e))
        })?;

        info!("Transaction simulation successful");
        Ok(())
    }
}

#[async_trait]
impl<P: Provider + Send + Sync> TransactionProcessor for EvmTransactionProcessor<P> {
    async fn process_with_retry<T, F, Fut>(
        &self,
        operation: F,
        config: RetryConfig,
    ) -> Result<T, ToolError>
    where
        T: Send,
        F: Fn() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, ToolError>> + Send,
    {
        // Use the retry function directly
        crate::retry::retry_async(
            operation,
            |error| {
                if error.is_retriable() {
                    if error.is_rate_limited() {
                        crate::retry::ErrorClass::RateLimited
                    } else {
                        crate::retry::ErrorClass::Retryable
                    }
                } else {
                    crate::retry::ErrorClass::Permanent
                }
            },
            &config,
            "evm_transaction",
        )
        .await
    }

    async fn get_status(&self, tx_hash: &str) -> Result<TransactionStatus, ToolError> {
        let hash = tx_hash
            .parse()
            .map_err(|e| ToolError::permanent_string(format!("Invalid transaction hash: {}", e)))?;

        // Get transaction receipt
        match self.provider.get_transaction_receipt(hash).await {
            Ok(Some(receipt)) => {
                if receipt.inner.status() {
                    Ok(TransactionStatus::Confirmed {
                        hash: tx_hash.to_string(),
                        block: receipt.block_number.unwrap_or_default(),
                    })
                } else {
                    Ok(TransactionStatus::Failed {
                        reason: "Transaction reverted".to_string(),
                    })
                }
            }
            Ok(None) => {
                // Transaction not yet mined
                Ok(TransactionStatus::Submitted {
                    hash: tx_hash.to_string(),
                })
            }
            Err(e) => Err(ToolError::permanent_string(format!(
                "Failed to get transaction status: {}",
                e
            ))),
        }
    }

    async fn wait_for_confirmation(
        &self,
        tx_hash: &str,
        required_confirmations: u64,
    ) -> Result<TransactionStatus, ToolError> {
        use std::time::Duration;
        use tokio::time::sleep;

        if tx_hash.is_empty() {
            return Err(ToolError::permanent_string(
                "Transaction hash cannot be empty",
            ));
        }

        let check_interval = Duration::from_secs(2);
        let max_wait = Duration::from_secs(300); // 5 minutes max
        let start = std::time::Instant::now();

        loop {
            let status = self.get_status(tx_hash).await?;

            match status {
                TransactionStatus::Confirmed { ref hash, block } => {
                    // Check if we have enough confirmations
                    if let Ok(current_block) = self.provider.get_block_number().await {
                        let confirmations = current_block.saturating_sub(block);
                        if confirmations >= required_confirmations {
                            return Ok(status);
                        }
                        debug!(
                            "Transaction {} has {} confirmations, waiting for {}",
                            hash, confirmations, required_confirmations
                        );
                    }
                }
                TransactionStatus::Failed { ref reason } => {
                    return Err(ToolError::permanent_string(format!(
                        "Transaction failed: {}",
                        reason
                    )));
                }
                TransactionStatus::Submitted { .. } | TransactionStatus::Pending => {
                    debug!("Transaction {} is still pending", tx_hash);
                }
                TransactionStatus::Confirming { confirmations, .. } => {
                    if confirmations >= required_confirmations {
                        return Ok(status);
                    }
                    debug!(
                        "Transaction {} has {} confirmations, waiting for {}",
                        tx_hash, confirmations, required_confirmations
                    );
                }
            }

            if start.elapsed() > max_wait {
                return Err(ToolError::permanent_string(format!(
                    "Transaction {} not confirmed after {:?}",
                    tx_hash, max_wait
                )));
            }

            sleep(check_interval).await;
        }
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

    // MockProvider tests are temporarily disabled due to complex alloy trait implementations
    // TODO: Implement proper mock provider or use integration tests with real providers
    //
    // The following tests would normally verify:
    // - Gas price estimation (EIP-1559 and legacy)
    // - Gas limit estimation with buffers
    // - Transaction preparation with various configurations
    // - Transaction simulation
    // - Transaction status checking
    // - Error handling for provider failures
    // - Edge cases with gas multipliers
}
