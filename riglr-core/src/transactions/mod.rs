//! Enhanced transaction processing with retry logic and gas optimization
//!
//! This module provides production-ready transaction handling with:
//! - Automatic retry with exponential backoff
//! - Gas price optimization
//! - Multi-signature support
//! - Transaction status tracking

// Note: Chain-specific transaction processing logic has been moved directly into
// the signer implementations to simplify the architecture:
// - Solana: riglr-solana-tools/src/signer/local.rs (LocalSolanaSigner)
// - EVM: riglr-evm-tools/src/signer.rs (LocalEvmSigner)

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

// Note: The TransactionProcessor trait has been removed as part of the refactoring.
// Transaction processing logic has been moved directly into the chain-specific signers:
// - EVM: riglr-evm-tools/src/signer.rs (LocalEvmSigner)
// - Solana: riglr-solana-tools/src/signer/local.rs (LocalSolanaSigner)

/// Configuration builder for transaction processing
///
/// This builder provides a foundation for creating chain-specific transaction
/// processors. The actual processor creation methods are provided by the
/// chain-specific tool crates (riglr-solana-tools, riglr-evm-tools) to maintain
/// the chain-agnostic nature of riglr-core.
///
/// # Example
///
/// ```rust,no_run
/// use riglr_core::transactions::RetryConfig;
///
/// // Create a configuration for transaction processing
/// let config = RetryConfig::default()
///     .with_max_retries(5)
///     .with_base_delay_ms(2000);
///
/// // The actual processor creation happens in chain-specific crates:
/// // - For Solana: use riglr_solana_tools::transactions::SolanaTransactionProcessor
/// // - For EVM: use riglr_evm_tools::transactions::EvmTransactionProcessor
/// ```
#[derive(Default)]
pub struct TransactionConfig {
    /// Retry configuration for transaction processing
    pub retry_config: RetryConfig,
}

impl TransactionConfig {
    /// Create a new transaction configuration with the given retry settings
    pub fn new(retry_config: RetryConfig) -> Self {
        Self { retry_config }
    }

    /// Get the retry configuration
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // Note: TransactionProcessor trait and its tests have been removed.
    // Transaction processing logic has been moved to chain-specific signers.
}
