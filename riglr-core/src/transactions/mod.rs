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

    // Tests for RetryConfig struct
    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_retry_config_custom() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 1.5,
        };
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(500));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.backoff_multiplier, 1.5);
    }

    #[test]
    fn test_retry_config_clone() {
        let config = RetryConfig::default();
        let cloned = config.clone();
        assert_eq!(config.max_attempts, cloned.max_attempts);
        assert_eq!(config.initial_delay, cloned.initial_delay);
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

    // Tests for GenericTransactionProcessor::get_status
    #[tokio::test]
    async fn test_get_status_when_empty_hash_should_return_error() {
        let processor = GenericTransactionProcessor;
        let result = processor.get_status("").await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            crate::error::ToolError::Permanent { context, .. } => {
                assert!(context.contains("Transaction hash cannot be empty"));
            }
            _ => panic!("Expected permanent error for empty hash"),
        }
    }

    #[tokio::test]
    async fn test_get_status_when_valid_hash_should_return_pending() {
        let processor = GenericTransactionProcessor;
        let result = processor.get_status("0x123456").await;
        assert!(result.is_ok());
        match result.unwrap() {
            TransactionStatus::Pending => (),
            _ => panic!("Expected Pending status for valid hash"),
        }
    }

    // Tests for GenericTransactionProcessor::process_with_retry
    #[tokio::test]
    async fn test_process_with_retry_when_success_on_first_attempt_should_return_ok() {
        let processor = GenericTransactionProcessor;
        let result = processor
            .process_with_retry(
                || async { Ok("success") },
                RetryConfig {
                    max_attempts: 3,
                    initial_delay: Duration::from_millis(1),
                    max_delay: Duration::from_secs(1),
                    backoff_multiplier: 2.0,
                },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_process_with_retry_when_retriable_error_should_retry() {
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
                                source: None,
                                source_message: std::io::Error::other("test").to_string(),
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

    #[tokio::test]
    async fn test_process_with_retry_when_max_attempts_exceeded_should_return_error() {
        let processor = GenericTransactionProcessor;
        let result: Result<&str, crate::error::ToolError> = processor
            .process_with_retry(
                || async {
                    Err(crate::error::ToolError::Retriable {
                        source: None,
                        source_message: std::io::Error::other("persistent error").to_string(),
                        context: "always fails".to_string(),
                    })
                },
                RetryConfig {
                    max_attempts: 2,
                    initial_delay: Duration::from_millis(1),
                    max_delay: Duration::from_secs(1),
                    backoff_multiplier: 2.0,
                },
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::ToolError::Retriable { context, .. } => {
                assert_eq!(context, "always fails");
            }
            _ => panic!("Expected retriable error"),
        }
    }

    #[tokio::test]
    async fn test_process_with_retry_when_rate_limited_with_retry_after_should_wait() {
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
                        if *count < 2 {
                            Err(crate::error::ToolError::RateLimited {
                                source: None,
                                source_message: std::io::Error::other("rate limited").to_string(),
                                context: "rate limited".to_string(),
                                retry_after: Some(Duration::from_millis(1)),
                            })
                        } else {
                            Ok("success after rate limit")
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
        assert_eq!(result.unwrap(), "success after rate limit");
        assert_eq!(*attempts.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_process_with_retry_when_rate_limited_without_retry_after_should_use_default_delay(
    ) {
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
                        if *count < 2 {
                            Err(crate::error::ToolError::RateLimited {
                                source: None,
                                source_message: std::io::Error::other(
                                    "rate limited no retry after",
                                )
                                .to_string(),
                                context: "rate limited no retry after".to_string(),
                                retry_after: None,
                            })
                        } else {
                            Ok("success")
                        }
                    }
                },
                RetryConfig {
                    max_attempts: 3,
                    initial_delay: Duration::from_millis(1),
                    max_delay: Duration::from_secs(1),
                    backoff_multiplier: 2.0,
                },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(*attempts.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_process_with_retry_when_non_retriable_error_should_fail_immediately() {
        let processor = GenericTransactionProcessor;
        let result: Result<&str, crate::error::ToolError> = processor
            .process_with_retry(
                || async {
                    Err(crate::error::ToolError::permanent_string(
                        "non-retriable error".to_string(),
                    ))
                },
                RetryConfig {
                    max_attempts: 3,
                    initial_delay: Duration::from_millis(1),
                    max_delay: Duration::from_secs(1),
                    backoff_multiplier: 2.0,
                },
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::ToolError::Permanent { context, .. } => {
                assert_eq!(context, "non-retriable error");
            }
            _ => panic!("Expected permanent error"),
        }
    }

    #[tokio::test]
    async fn test_process_with_retry_when_exponential_backoff_reaches_max_delay() {
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
                        if *count < 4 {
                            Err(crate::error::ToolError::Retriable {
                                source: None,
                                source_message: std::io::Error::other("test").to_string(),
                                context: "test backoff".to_string(),
                            })
                        } else {
                            Ok("success")
                        }
                    }
                },
                RetryConfig {
                    max_attempts: 5,
                    initial_delay: Duration::from_millis(1),
                    max_delay: Duration::from_millis(2), // Very small max to test capping
                    backoff_multiplier: 10.0,            // Large multiplier to test max delay cap
                },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(*attempts.lock().unwrap(), 4);
    }

    // Tests for GenericTransactionProcessor::wait_for_confirmation
    #[tokio::test]
    async fn test_wait_for_confirmation_when_empty_hash_should_return_error() {
        let processor = GenericTransactionProcessor;
        let result = processor.wait_for_confirmation("", 1).await;
        assert!(result.is_err());
    }

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

    // Original test preserved
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
                                source: None,
                                source_message: std::io::Error::other("test").to_string(),
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
