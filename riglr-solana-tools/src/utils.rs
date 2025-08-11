//! Transaction utilities for enhanced Solana transaction handling
//!
//! This module provides centralized transaction sending functionality with
//! robust retry logic, exponential backoff, and comprehensive error handling.

use riglr_core::{ToolError, SignerContext, signer::SignerError};
use crate::error::{classify_transaction_error, TransactionErrorType, RetryableError, PermanentError};
use solana_sdk::transaction::Transaction;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn, error};

/// Configuration for transaction retry behavior
#[derive(Debug, Clone)]
pub struct TransactionConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub base_delay_ms: u64,
    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to use jitter to avoid thundering herd
    pub use_jitter: bool,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000, // Start with 1 second
            max_delay_ms: 30_000, // Cap at 30 seconds
            backoff_multiplier: 2.0, // Double each time
            use_jitter: true,
        }
    }
}

/// Result of a transaction submission
#[derive(Debug, Clone)]
pub struct TransactionSubmissionResult {
    /// Transaction signature
    pub signature: String,
    /// Number of attempts made
    pub attempts: u32,
    /// Total time taken for all attempts
    pub total_duration_ms: u64,
    /// Whether transaction was confirmed (false for non-blocking sending)
    pub confirmed: bool,
}

// Error classification is now handled by structured types in crate::error module

/// Helper function to classify SignerError into TransactionErrorType
fn classify_signer_error(signer_error: &SignerError) -> TransactionErrorType {
    match signer_error {
        SignerError::SolanaTransaction(client_error) => {
            classify_transaction_error(client_error)
        },
        SignerError::NoSignerContext => {
            TransactionErrorType::Permanent(PermanentError::InvalidTransaction)
        },
        SignerError::Configuration(_) => {
            TransactionErrorType::Permanent(PermanentError::InvalidTransaction)
        },
        SignerError::Signing(_) => {
            TransactionErrorType::Permanent(PermanentError::InvalidSignature)
        },
        _ => {
            // For other error types, default to retriable for safety
            TransactionErrorType::Retryable(RetryableError::TemporaryRpcFailure)
        }
    }
}

/// Calculate delay for retry with exponential backoff and optional jitter
fn calculate_retry_delay(
    attempt: u32,
    config: &TransactionConfig,
) -> Duration {
    let base_delay = config.base_delay_ms as f64;
    let backoff_factor = config.backoff_multiplier.powf(attempt as f64);
    let mut delay_ms = base_delay * backoff_factor;
    
    // Cap at max delay
    delay_ms = delay_ms.min(config.max_delay_ms as f64);
    
    // Add jitter if enabled (±25% randomization)
    if config.use_jitter {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter_factor = rng.gen_range(0.75..=1.25);
        delay_ms *= jitter_factor;
    }
    
    Duration::from_millis(delay_ms as u64)
}

/// Send a Solana transaction with retry logic and exponential backoff
///
/// This function centralizes all Solana transaction sending logic with robust
/// error handling, retry logic, and comprehensive logging. It automatically
/// classifies errors and applies appropriate retry strategies.
///
/// # Arguments
///
/// * `transaction` - The transaction to send (will be mutably borrowed for signing)
/// * `config` - Configuration for retry behavior
/// * `operation_name` - Human-readable operation name for logging
///
/// # Returns
///
/// Returns `TransactionSubmissionResult` containing signature and attempt metadata
///
/// # Error Handling
///
/// Automatically retries on:
/// - Network timeouts and connection issues
/// - RPC rate limiting (with longer backoff)
/// - Temporary blockchain congestion
///
/// Does NOT retry on:
/// - Insufficient funds
/// - Invalid signatures or accounts
/// - Program execution errors
/// - Invalid transaction structure
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::utils::{send_transaction_with_retry, TransactionConfig};
/// use solana_sdk::{transaction::Transaction, system_instruction};
/// use riglr_core::SignerContext;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let signer = SignerContext::current().await?;
/// let from = signer.pubkey().unwrap().parse()?;
/// let to = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".parse()?;
/// 
/// let instruction = system_instruction::transfer(&from, &to, 1000000);
/// let mut tx = Transaction::new_with_payer(&[instruction], Some(&from));
///
/// let config = TransactionConfig::default();
/// let result = send_transaction_with_retry(
///     &mut tx, 
///     &config, 
///     "SOL Transfer"
/// ).await?;
///
/// println!("Transaction sent: {} (attempts: {})", 
///          result.signature, result.attempts);
/// # Ok(())
/// # }
/// ```
pub async fn send_transaction_with_retry(
    transaction: &mut Transaction,
    config: &TransactionConfig,
    operation_name: &str,
) -> Result<TransactionSubmissionResult, ToolError> {
    let start_time = std::time::Instant::now();
    
    debug!(
        "Sending transaction for operation '{}' with retry config: max_retries={}, base_delay={}ms",
        operation_name, config.max_retries, config.base_delay_ms
    );

    // Get signer context
    let signer_context = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;

    let mut last_error: Option<String> = None;
    
    for attempt in 0..=config.max_retries {
        debug!("Transaction attempt {} for '{}'", attempt + 1, operation_name);
        
        // Attempt to send the transaction
        match signer_context.sign_and_send_solana_transaction(transaction).await {
            Ok(signature) => {
                let total_duration = start_time.elapsed().as_millis() as u64;
                
                info!(
                    "Transaction successful for '{}': signature={}, attempts={}, duration={}ms",
                    operation_name, signature, attempt + 1, total_duration
                );
                
                return Ok(TransactionSubmissionResult {
                    signature,
                    attempts: attempt + 1,
                    total_duration_ms: total_duration,
                    confirmed: false, // Non-blocking - transaction is sent but not confirmed
                });
            }
            Err(signer_error) => {
                let error_msg = signer_error.to_string();
                last_error = Some(error_msg.clone());
                
                let error_type = classify_signer_error(&signer_error);
                
                debug!(
                    "Transaction attempt {} failed for '{}': {} (classified as: {:?})",
                    attempt + 1, operation_name, error_msg, error_type
                );
                
                // Don't retry permanent errors
                if matches!(error_type, TransactionErrorType::Permanent(_)) {
                    error!(
                        "Permanent transaction error for '{}' (attempt {}): {}",
                        operation_name, attempt + 1, error_msg
                    );
                    return Err(ToolError::permanent(format!(
                        "Transaction failed for '{}': {}", operation_name, error_msg
                    )));
                }
                
                // If this was the last attempt, fail
                if attempt >= config.max_retries {
                    error!(
                        "Transaction exhausted all {} attempts for '{}', last error: {}",
                        config.max_retries + 1, operation_name, error_msg
                    );
                    break;
                }
                
                // Calculate delay for retry
                let mut delay = calculate_retry_delay(attempt, config);
                
                // Use longer delay for rate limiting
                if error_type.is_rate_limited() {
                    delay = Duration::from_millis((delay.as_millis() as u64 * 3).min(config.max_delay_ms));
                    warn!(
                        "Rate limited for '{}', using extended delay: {}ms",
                        operation_name, delay.as_millis()
                    );
                }
                
                debug!(
                    "Retrying transaction for '{}' in {}ms (attempt {}/{})",
                    operation_name, delay.as_millis(), attempt + 1, config.max_retries + 1
                );
                
                sleep(delay).await;
            }
        }
    }
    
    // All attempts failed
    let final_error = last_error.unwrap_or_else(|| "Unknown error".to_string());
    let total_duration = start_time.elapsed().as_millis() as u64;
    
    error!(
        "Transaction failed for '{}' after {} attempts in {}ms, final error: {}",
        operation_name, config.max_retries + 1, total_duration, final_error
    );
    
    Err(ToolError::permanent(format!(
        "Transaction failed for '{}' after {} attempts: {}", 
        operation_name, config.max_retries + 1, final_error
    )))
}

/// Send a transaction with default retry configuration
///
/// Convenience function that uses the default `TransactionConfig` for standard
/// retry behavior. Suitable for most transaction sending scenarios.
///
/// # Arguments
///
/// * `transaction` - The transaction to send
/// * `operation_name` - Human-readable operation name for logging
///
/// # Returns
///
/// Returns the transaction signature on success
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::utils::send_transaction;
/// use solana_sdk::{transaction::Transaction, system_instruction};
/// use riglr_core::SignerContext;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let signer = SignerContext::current().await?;
/// let from = signer.pubkey().unwrap().parse()?;
/// let to = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".parse()?;
/// 
/// let instruction = system_instruction::transfer(&from, &to, 1000000);
/// let mut tx = Transaction::new_with_payer(&[instruction], Some(&from));
///
/// let signature = send_transaction(&mut tx, "SOL Transfer").await?;
/// println!("Transaction sent: {}", signature);
/// # Ok(())
/// # }
/// ```
pub async fn send_transaction(
    transaction: &mut Transaction,
    operation_name: &str,
) -> Result<String, ToolError> {
    let config = TransactionConfig::default();
    let result = send_transaction_with_retry(transaction, &config, operation_name).await?;
    Ok(result.signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: Error classification tests moved to error.rs module with structured error types
    // The old string-based classification has been replaced with structured ClientError classification
    
    #[test]
    fn test_retry_delay_calculation() {
        let config = TransactionConfig::default();
        
        // Test exponential backoff
        let delay0 = calculate_retry_delay(0, &config);
        let delay1 = calculate_retry_delay(1, &config);
        let delay2 = calculate_retry_delay(2, &config);
        
        // Base delay should be close to configured value (accounting for jitter)
        assert!(delay0.as_millis() >= 750 && delay0.as_millis() <= 1250);
        
        // Should increase exponentially
        assert!(delay1.as_millis() > delay0.as_millis());
        assert!(delay2.as_millis() > delay1.as_millis());
        
        // Test max delay cap
        let config_with_low_cap = TransactionConfig {
            max_delay_ms: 2000,
            ..Default::default()
        };
        let long_delay = calculate_retry_delay(10, &config_with_low_cap);
        assert!(long_delay.as_millis() <= 2500); // Allow for jitter
    }
    
    #[test]
    fn test_config_defaults() {
        let config = TransactionConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30_000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.use_jitter);
    }
}