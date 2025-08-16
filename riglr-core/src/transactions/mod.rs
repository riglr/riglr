//! Enhanced transaction processing with retry logic and gas optimization
//!
//! This module provides production-ready transaction handling with:
//! - Automatic retry with exponential backoff
//! - Gas price optimization
//! - Multi-signature support
//! - Transaction status tracking

use async_trait::async_trait;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub mod evm;
pub mod solana;

/// Transaction retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Transaction status tracking
#[derive(Debug, Clone)]
pub enum TransactionStatus {
    /// Transaction is pending submission
    Pending,
    /// Transaction has been submitted to the network
    Submitted { hash: String },
    /// Transaction is being confirmed
    Confirming { hash: String, confirmations: u64 },
    /// Transaction has been confirmed
    Confirmed { hash: String, block: u64 },
    /// Transaction failed
    Failed { reason: String },
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

/// Generic transaction processor implementation
pub struct GenericTransactionProcessor;

#[async_trait]
impl TransactionProcessor for GenericTransactionProcessor {
    async fn process_with_retry<T, F, Fut>(
        &self,
        operation: F,
        config: RetryConfig,
    ) -> Result<T, crate::error::ToolError>
    where
        T: Send,
        F: Fn() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, crate::error::ToolError>> + Send,
    {
        let mut attempt = 0;
        let mut delay = config.initial_delay;

        loop {
            attempt += 1;
            info!("Transaction attempt {}/{}", attempt, config.max_attempts);

            match operation().await {
                Ok(result) => {
                    info!("Transaction successful on attempt {}", attempt);
                    return Ok(result);
                }
                Err(err) if attempt >= config.max_attempts => {
                    error!(
                        "Transaction failed after {} attempts: {}",
                        config.max_attempts, err
                    );
                    return Err(err);
                }
                Err(err) => {
                    // Check if error is retriable
                    match &err {
                        crate::error::ToolError::Retriable { .. } => {
                            warn!("Retriable error on attempt {}: {}", attempt, err);
                            sleep(delay).await;

                            // Calculate next delay with exponential backoff
                            delay = Duration::from_secs_f64(
                                (delay.as_secs_f64() * config.backoff_multiplier)
                                    .min(config.max_delay.as_secs_f64()),
                            );
                        }
                        crate::error::ToolError::RateLimited { retry_after, .. } => {
                            let wait_time = retry_after.unwrap_or(delay);
                            warn!("Rate limited, waiting {:?} before retry", wait_time);
                            sleep(wait_time).await;
                        }
                        _ => {
                            error!("Non-retriable error: {}", err);
                            return Err(err);
                        }
                    }
                }
            }
        }
    }

    async fn get_status(
        &self,
        tx_hash: &str,
    ) -> Result<TransactionStatus, crate::error::ToolError> {
        // Generic implementation that returns a pending status
        // Chain-specific implementations should override this with actual status checking

        // Validate the transaction hash format (basic check)
        if tx_hash.is_empty() {
            return Err(crate::error::ToolError::permanent_string(
                "Transaction hash cannot be empty".to_string(),
            ));
        }

        // Return a pending status as a safe default
        // This indicates the transaction is known but status checking is not implemented
        warn!(
            "Generic transaction status check for {}. Chain-specific implementation recommended.",
            tx_hash
        );

        Ok(TransactionStatus::Pending)
    }

    async fn wait_for_confirmation(
        &self,
        tx_hash: &str,
        required_confirmations: u64,
    ) -> Result<TransactionStatus, crate::error::ToolError> {
        let mut confirmations = 0;
        let check_interval = Duration::from_secs(2);
        let max_wait = Duration::from_secs(300); // 5 minutes max wait
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
                    if confirmations < required_confirmations {
                        info!(
                            "Transaction {} has {}/{} confirmations",
                            tx_hash, confirmations, required_confirmations
                        );
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_logic() {
        let processor = GenericTransactionProcessor;
        let attempts = Arc::new(std::sync::Mutex::new(0));
        let attempts_clone = attempts.clone();

        let result = processor
            .process_with_retry(
                move || {
                    let attempts = attempts_clone.clone();
                    async move {
                        let mut count = attempts.lock().unwrap();
                        *count += 1;
                        if *count < 3 {
                            Err(crate::error::ToolError::Retriable {
                                source: Box::new(std::io::Error::other("test")),
                                context: "test error".to_string(),
                            })
                        } else {
                            Ok("success")
                        }
                    }
                },
                RetryConfig {
                    max_attempts: 3,
                    initial_delay: Duration::from_millis(10),
                    max_delay: Duration::from_secs(1),
                    backoff_multiplier: 2.0,
                },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(*attempts.lock().unwrap(), 3);
    }
}
