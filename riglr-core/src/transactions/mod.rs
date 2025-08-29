//! Enhanced transaction processing with retry logic and gas optimization
//!
//! This module provides production-ready transaction handling with:
//! - Automatic retry with exponential backoff
//! - Gas price optimization
//! - Multi-signature support
//! - Transaction status tracking

use async_trait::async_trait;

pub mod evm;
pub mod solana;

// Re-export RetryConfig from retry module
pub use crate::retry::RetryConfig;

/// Transaction status tracking
#[derive(Debug, Clone)]
pub enum TransactionStatus {
    /// Transaction is pending submission
    Pending,
    /// Transaction has been submitted to the network
    Submitted {
        /// Transaction hash from the network
        hash: String,
    },
    /// Transaction is being confirmed
    Confirming {
        /// Transaction hash from the network
        hash: String,
        /// Current number of confirmations received
        confirmations: u64,
    },
    /// Transaction has been confirmed
    Confirmed {
        /// Transaction hash from the network
        hash: String,
        /// Block number where the transaction was included
        block: u64,
    },
    /// Transaction failed
    Failed {
        /// Reason why the transaction failed
        reason: String,
    },
}

/// Trait for transaction processors
#[async_trait]
pub trait TransactionProcessor: Send + Sync {
    /// Process a transaction with retry logic
    async fn process_with_retry<T, F, Fut>(
        &self,
        operation: F,
        config: RetryConfig,
    ) -> Result<T, crate::error::ToolError>
    where
        T: Send,
        F: Fn() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, crate::error::ToolError>> + Send;

    /// Get current transaction status
    async fn get_status(&self, tx_hash: &str)
        -> Result<TransactionStatus, crate::error::ToolError>;

    /// Wait for transaction confirmation
    async fn wait_for_confirmation(
        &self,
        tx_hash: &str,
        required_confirmations: u64,
    ) -> Result<TransactionStatus, crate::error::ToolError>;
}

/// Builder for creating chain-specific transaction processors
///
/// This builder ensures that transaction processors are created with the appropriate
/// chain-specific implementation, enforcing compile-time safety and preventing
/// runtime errors from incorrect processor usage.
///
/// # Example
///
/// ```rust,no_run
/// use riglr_core::transactions::{TransactionProcessorBuilder, RetryConfig};
/// use std::sync::Arc;
/// use solana_client::rpc_client::RpcClient;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a Solana processor
/// let solana_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com".to_string()));
/// let solana_processor = TransactionProcessorBuilder::new(RetryConfig::default())
///     .solana(solana_client, Default::default());
///
/// // Create an EVM processor
/// # /*
/// let evm_provider = Arc::new(/* your provider */);
/// let evm_processor = TransactionProcessorBuilder::new(RetryConfig::default())
///     .evm(evm_provider, Default::default());
/// # */
/// # Ok(())
/// # }
/// ```
pub struct TransactionProcessorBuilder {
    retry_config: RetryConfig,
}

impl TransactionProcessorBuilder {
    /// Create a new transaction processor builder with the given retry configuration
    pub fn new(retry_config: RetryConfig) -> Self {
        Self { retry_config }
    }

    /// Build a Solana transaction processor
    pub fn solana(
        self,
        client: std::sync::Arc<solana_client::rpc_client::RpcClient>,
        priority_config: solana::PriorityFeeConfig,
    ) -> solana::SolanaTransactionProcessor {
        solana::SolanaTransactionProcessor::new(client, priority_config, self.retry_config)
    }

    /// Build an EVM transaction processor
    pub fn evm<P: alloy::providers::Provider>(
        self,
        provider: std::sync::Arc<P>,
        gas_config: evm::GasConfig,
    ) -> evm::EvmTransactionProcessor<P> {
        evm::EvmTransactionProcessor::new(provider, gas_config, self.retry_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;

    // Tests for RetryConfig (from retry module)
    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30_000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.use_jitter);
    }

    #[test]
    fn test_retry_config_custom() {
        let config = RetryConfig {
            max_retries: 5,
            base_delay_ms: 500,
            max_delay_ms: 60_000,
            backoff_multiplier: 1.5,
            use_jitter: false,
        };
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay_ms, 500);
        assert_eq!(config.max_delay_ms, 60_000);
        assert_eq!(config.backoff_multiplier, 1.5);
        assert!(!config.use_jitter);
    }

    #[test]
    fn test_retry_config_clone() {
        let config = RetryConfig::default();
        let cloned = config.clone();
        assert_eq!(config.max_retries, cloned.max_retries);
        assert_eq!(config.base_delay_ms, cloned.base_delay_ms);
    }

    // Tests for TransactionStatus enum
    #[test]
    fn test_transaction_status_pending() {
        let status = TransactionStatus::Pending;
        match status {
            TransactionStatus::Pending => (),
            _ => panic!("Expected Pending status"),
        }
    }

    #[test]
    fn test_transaction_status_submitted() {
        let hash = "0x123".to_string();
        let status = TransactionStatus::Submitted { hash: hash.clone() };
        match status {
            TransactionStatus::Submitted { hash: h } => assert_eq!(h, hash),
            _ => panic!("Expected Submitted status"),
        }
    }

    #[test]
    fn test_transaction_status_confirming() {
        let hash = "0x456".to_string();
        let confirmations = 2;
        let status = TransactionStatus::Confirming {
            hash: hash.clone(),
            confirmations,
        };
        match status {
            TransactionStatus::Confirming {
                hash: h,
                confirmations: c,
            } => {
                assert_eq!(h, hash);
                assert_eq!(c, confirmations);
            }
            _ => panic!("Expected Confirming status"),
        }
    }

    #[test]
    fn test_transaction_status_confirmed() {
        let hash = "0x789".to_string();
        let block = 12345;
        let status = TransactionStatus::Confirmed {
            hash: hash.clone(),
            block,
        };
        match status {
            TransactionStatus::Confirmed { hash: h, block: b } => {
                assert_eq!(h, hash);
                assert_eq!(b, block);
            }
            _ => panic!("Expected Confirmed status"),
        }
    }

    #[test]
    fn test_transaction_status_failed() {
        let reason = "Insufficient gas".to_string();
        let status = TransactionStatus::Failed {
            reason: reason.clone(),
        };
        match status {
            TransactionStatus::Failed { reason: r } => assert_eq!(r, reason),
            _ => panic!("Expected Failed status"),
        }
    }

    #[test]
    fn test_transaction_status_clone() {
        let status = TransactionStatus::Pending;
        let cloned = status.clone();
        match (status, cloned) {
            (TransactionStatus::Pending, TransactionStatus::Pending) => (),
            _ => panic!("Clone failed for Pending status"),
        }
    }

    // Note: GenericTransactionProcessor has been removed in favor of chain-specific implementations
    // Tests for transaction processing should be implemented in evm.rs and solana.rs

    // Tests for TransactionProcessor trait implementations using mock processors

    #[tokio::test]
    async fn test_wait_for_confirmation_when_confirmed_status_should_return_immediately() {
        // Create a custom processor that returns confirmed status
        struct MockConfirmedProcessor;

        #[async_trait]
        impl TransactionProcessor for MockConfirmedProcessor {
            async fn process_with_retry<T, F, Fut>(
                &self,
                _operation: F,
                _config: RetryConfig,
            ) -> Result<T, crate::error::ToolError>
            where
                T: Send,
                F: Fn() -> Fut + Send,
                Fut: std::future::Future<Output = Result<T, crate::error::ToolError>> + Send,
            {
                unimplemented!()
            }

            async fn get_status(
                &self,
                _tx_hash: &str,
            ) -> Result<TransactionStatus, crate::error::ToolError> {
                Ok(TransactionStatus::Confirmed {
                    hash: "0x123".to_string(),
                    block: 100,
                })
            }

            async fn wait_for_confirmation(
                &self,
                tx_hash: &str,
                _required_confirmations: u64,
            ) -> Result<TransactionStatus, crate::error::ToolError> {
                self.get_status(tx_hash).await
            }
        }

        let processor = MockConfirmedProcessor;
        let result = processor.wait_for_confirmation("0x123", 1).await;
        assert!(result.is_ok());
        match result.unwrap() {
            TransactionStatus::Confirmed { hash, block } => {
                assert_eq!(hash, "0x123");
                assert_eq!(block, 100);
            }
            _ => panic!("Expected confirmed status"),
        }
    }

    #[tokio::test]
    async fn test_wait_for_confirmation_when_failed_status_should_return_error() {
        // Create a custom processor that returns failed status
        struct MockFailedProcessor;

        #[async_trait]
        impl TransactionProcessor for MockFailedProcessor {
            async fn process_with_retry<T, F, Fut>(
                &self,
                _operation: F,
                _config: RetryConfig,
            ) -> Result<T, crate::error::ToolError>
            where
                T: Send,
                F: Fn() -> Fut + Send,
                Fut: std::future::Future<Output = Result<T, crate::error::ToolError>> + Send,
            {
                unimplemented!()
            }

            async fn get_status(
                &self,
                _tx_hash: &str,
            ) -> Result<TransactionStatus, crate::error::ToolError> {
                Ok(TransactionStatus::Failed {
                    reason: "Gas limit exceeded".to_string(),
                })
            }

            async fn wait_for_confirmation(
                &self,
                tx_hash: &str,
                _required_confirmations: u64,
            ) -> Result<TransactionStatus, crate::error::ToolError> {
                match self.get_status(tx_hash).await? {
                    TransactionStatus::Failed { reason } => {
                        Err(crate::error::ToolError::permanent_string(format!(
                            "Transaction failed: {}",
                            reason
                        )))
                    }
                    status => Ok(status),
                }
            }
        }

        let processor = MockFailedProcessor;
        let result = processor.wait_for_confirmation("0x123", 1).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::ToolError::Permanent { context, .. } => {
                assert!(context.contains("Transaction failed: Gas limit exceeded"));
            }
            _ => panic!("Expected permanent error for failed transaction"),
        }
    }

    #[tokio::test]
    async fn test_wait_for_confirmation_when_confirming_with_sufficient_confirmations_should_continue(
    ) {
        // Create a processor that simulates progressing confirmations
        struct MockConfirmingProcessor {
            calls: Arc<std::sync::Mutex<u32>>,
        }

        #[async_trait]
        impl TransactionProcessor for MockConfirmingProcessor {
            async fn process_with_retry<T, F, Fut>(
                &self,
                _operation: F,
                _config: RetryConfig,
            ) -> Result<T, crate::error::ToolError>
            where
                T: Send,
                F: Fn() -> Fut + Send,
                Fut: std::future::Future<Output = Result<T, crate::error::ToolError>> + Send,
            {
                unimplemented!()
            }

            async fn get_status(
                &self,
                _tx_hash: &str,
            ) -> Result<TransactionStatus, crate::error::ToolError> {
                let mut calls = self.calls.lock().unwrap();
                *calls += 1;

                if *calls >= 3 {
                    Ok(TransactionStatus::Confirmed {
                        hash: "0x123".to_string(),
                        block: 100,
                    })
                } else {
                    Ok(TransactionStatus::Confirming {
                        hash: "0x123".to_string(),
                        confirmations: *calls as u64,
                    })
                }
            }

            async fn wait_for_confirmation(
                &self,
                tx_hash: &str,
                required_confirmations: u64,
            ) -> Result<TransactionStatus, crate::error::ToolError> {
                let mut confirmations = 0;
                let check_interval = Duration::from_millis(1); // Fast for testing
                let max_wait = Duration::from_secs(1);
                let start = std::time::Instant::now();

                while confirmations < required_confirmations {
                    if start.elapsed() > max_wait {
                        return Err(crate::error::ToolError::permanent_string(format!(
                            "Transaction {} not confirmed after {:?}",
                            tx_hash, max_wait
                        )));
                    }

                    match self.get_status(tx_hash).await? {
                        TransactionStatus::Confirmed { hash, block } => {
                            return Ok(TransactionStatus::Confirmed { hash, block });
                        }
                        TransactionStatus::Failed { reason } => {
                            return Err(crate::error::ToolError::permanent_string(format!(
                                "Transaction failed: {}",
                                reason
                            )));
                        }
                        TransactionStatus::Confirming {
                            confirmations: conf,
                            ..
                        } => {
                            confirmations = conf;
                        }
                        _ => {
                            // Still pending or submitted
                        }
                    }

                    sleep(check_interval).await;
                }

                self.get_status(tx_hash).await
            }
        }

        let processor = MockConfirmingProcessor {
            calls: Arc::new(std::sync::Mutex::new(0)),
        };
        let result = processor.wait_for_confirmation("0x123", 2).await;
        assert!(result.is_ok());
        match result.unwrap() {
            TransactionStatus::Confirmed { hash, block } => {
                assert_eq!(hash, "0x123");
                assert_eq!(block, 100);
            }
            _ => panic!("Expected confirmed status"),
        }
    }

    #[tokio::test]
    async fn test_wait_for_confirmation_when_pending_status_should_continue_waiting() {
        // Create a processor that returns pending then confirmed
        struct MockPendingProcessor {
            calls: Arc<std::sync::Mutex<u32>>,
        }

        #[async_trait]
        impl TransactionProcessor for MockPendingProcessor {
            async fn process_with_retry<T, F, Fut>(
                &self,
                _operation: F,
                _config: RetryConfig,
            ) -> Result<T, crate::error::ToolError>
            where
                T: Send,
                F: Fn() -> Fut + Send,
                Fut: std::future::Future<Output = Result<T, crate::error::ToolError>> + Send,
            {
                unimplemented!()
            }

            async fn get_status(
                &self,
                _tx_hash: &str,
            ) -> Result<TransactionStatus, crate::error::ToolError> {
                let mut calls = self.calls.lock().unwrap();
                *calls += 1;

                if *calls >= 3 {
                    Ok(TransactionStatus::Confirmed {
                        hash: "0x123".to_string(),
                        block: 100,
                    })
                } else {
                    Ok(TransactionStatus::Pending)
                }
            }

            async fn wait_for_confirmation(
                &self,
                tx_hash: &str,
                required_confirmations: u64,
            ) -> Result<TransactionStatus, crate::error::ToolError> {
                let mut confirmations = 0;
                let check_interval = Duration::from_millis(1);
                let max_wait = Duration::from_secs(1);
                let start = std::time::Instant::now();

                while confirmations < required_confirmations {
                    if start.elapsed() > max_wait {
                        return Err(crate::error::ToolError::permanent_string(format!(
                            "Transaction {} not confirmed after {:?}",
                            tx_hash, max_wait
                        )));
                    }

                    match self.get_status(tx_hash).await? {
                        TransactionStatus::Confirmed { hash, block } => {
                            return Ok(TransactionStatus::Confirmed { hash, block });
                        }
                        TransactionStatus::Failed { reason } => {
                            return Err(crate::error::ToolError::permanent_string(format!(
                                "Transaction failed: {}",
                                reason
                            )));
                        }
                        TransactionStatus::Confirming {
                            confirmations: conf,
                            ..
                        } => {
                            confirmations = conf;
                        }
                        _ => {
                            // Still pending or submitted - continue waiting
                        }
                    }

                    sleep(check_interval).await;
                }

                self.get_status(tx_hash).await
            }
        }

        let processor = MockPendingProcessor {
            calls: Arc::new(std::sync::Mutex::new(0)),
        };
        let result = processor.wait_for_confirmation("0x123", 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_wait_for_confirmation_when_timeout_should_return_error() {
        // Create a processor that always returns pending (never confirms)
        struct MockTimeoutProcessor;

        #[async_trait]
        impl TransactionProcessor for MockTimeoutProcessor {
            async fn process_with_retry<T, F, Fut>(
                &self,
                _operation: F,
                _config: RetryConfig,
            ) -> Result<T, crate::error::ToolError>
            where
                T: Send,
                F: Fn() -> Fut + Send,
                Fut: std::future::Future<Output = Result<T, crate::error::ToolError>> + Send,
            {
                unimplemented!()
            }

            async fn get_status(
                &self,
                _tx_hash: &str,
            ) -> Result<TransactionStatus, crate::error::ToolError> {
                Ok(TransactionStatus::Pending)
            }

            async fn wait_for_confirmation(
                &self,
                tx_hash: &str,
                required_confirmations: u64,
            ) -> Result<TransactionStatus, crate::error::ToolError> {
                let mut confirmations = 0;
                let check_interval = Duration::from_millis(1);
                let max_wait = Duration::from_millis(10); // Very short timeout for testing
                let start = std::time::Instant::now();

                while confirmations < required_confirmations {
                    if start.elapsed() > max_wait {
                        return Err(crate::error::ToolError::permanent_string(format!(
                            "Transaction {} not confirmed after {:?}",
                            tx_hash, max_wait
                        )));
                    }

                    match self.get_status(tx_hash).await? {
                        TransactionStatus::Confirmed { hash, block } => {
                            return Ok(TransactionStatus::Confirmed { hash, block });
                        }
                        TransactionStatus::Failed { reason } => {
                            return Err(crate::error::ToolError::permanent_string(format!(
                                "Transaction failed: {}",
                                reason
                            )));
                        }
                        TransactionStatus::Confirming {
                            confirmations: conf,
                            ..
                        } => {
                            confirmations = conf;
                        }
                        _ => {
                            // Still pending or submitted
                        }
                    }

                    sleep(check_interval).await;
                }

                self.get_status(tx_hash).await
            }
        }

        let processor = MockTimeoutProcessor;
        let result = processor.wait_for_confirmation("0x123", 1).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::ToolError::Permanent { context, .. } => {
                assert!(context.contains("not confirmed after"));
            }
            _ => panic!("Expected timeout error"),
        }
    }
}
