//! Transaction utilities for enhanced Solana transaction handling
//!
//! This module provides centralized transaction sending functionality with
//! robust retry logic, exponential backoff, and comprehensive error handling.
//!
//! All transaction utilities follow the SignerContext pattern for secure multi-tenant operation.

use crate::error::{
    classify_transaction_error, RetryableError, SolanaToolError, TransactionErrorType,
};
use riglr_core::{
    retry::{retry_async, ErrorClass, RetryConfig},
    signer::SignerError,
    SignerContext, ToolError,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, transaction::Transaction,
};
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Configuration for transaction retry behavior
/// This is now a simple wrapper around riglr_core::retry::RetryConfig
pub type TransactionConfig = RetryConfig;

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

/// Helper function to classify SignerError into ErrorClass for retry_async
fn classify_signer_error_for_retry(signer_error: &SignerError) -> ErrorClass {
    match signer_error {
        SignerError::SolanaTransaction(client_error) => {
            match classify_transaction_error(client_error) {
                TransactionErrorType::Permanent(_) => ErrorClass::Permanent,
                TransactionErrorType::Retryable(RetryableError::NetworkCongestion) => {
                    ErrorClass::RateLimited
                }
                TransactionErrorType::Retryable(_) => ErrorClass::Retryable,
                TransactionErrorType::RateLimited(_) => ErrorClass::RateLimited,
                TransactionErrorType::Unknown(_) => ErrorClass::Retryable, // Conservative: treat unknown as retryable
            }
        }
        SignerError::NoSignerContext | SignerError::Configuration(_) | SignerError::Signing(_) => {
            ErrorClass::Permanent
        }
        _ => ErrorClass::Retryable,
    }
}

/// Send a Solana transaction with retry logic and exponential backoff
///
/// This function centralizes all Solana transaction sending logic with robust
/// error handling, retry logic, and comprehensive logging. It automatically
/// classifies errors and applies appropriate retry strategies.
///
/// Uses SignerContext for secure multi-tenant operation.
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
/// use riglr_solana_tools::utils::transaction::{send_transaction_with_retry, TransactionConfig};
/// use solana_sdk::transaction::Transaction;
/// use solana_system_interface::instruction as system_instruction;
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
) -> std::result::Result<TransactionSubmissionResult, ToolError> {
    let start_time = std::time::Instant::now();
    let mut attempts = 0u32;

    debug!(
        "Sending transaction for operation '{}' with retry config: max_retries={}, base_delay={}ms",
        operation_name, config.max_retries, config.base_delay_ms
    );

    // Clone transaction for use in closure
    let tx_clone = transaction.clone();

    // Use retry_async with proper error classification
    let result = retry_async(
        || {
            attempts += 1;
            let mut tx = tx_clone.clone();

            async move {
                // Get the current signer context
                let signer = SignerContext::current_as_solana().await?;
                signer.sign_and_send_transaction(&mut tx).await
            }
        },
        classify_signer_error_for_retry,
        config,
        operation_name,
    )
    .await;

    let total_duration = start_time.elapsed().as_millis() as u64;

    match result {
        Ok(signature) => {
            info!(
                "Transaction successful for '{}': signature={}, attempts={}, duration={}ms",
                operation_name, signature, attempts, total_duration
            );

            Ok(TransactionSubmissionResult {
                signature,
                attempts,
                total_duration_ms: total_duration,
                confirmed: false, // Non-blocking - transaction is sent but not confirmed
            })
        }
        Err(e) => {
            error!(
                "Transaction failed for '{}' after {} attempts in {}ms: {}",
                operation_name, attempts, total_duration, e
            );

            Err(ToolError::permanent_string(format!(
                "Transaction failed for '{}' after {} attempts: {}",
                operation_name, attempts, e
            )))
        }
    }
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
/// use riglr_solana_tools::utils::transaction::send_transaction;
/// use solana_sdk::transaction::Transaction;
/// use solana_system_interface::instruction as system_instruction;
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
) -> std::result::Result<String, ToolError> {
    let config = TransactionConfig::default();
    let result = send_transaction_with_retry(transaction, &config, operation_name).await?;
    Ok(result.signature)
}

/// Higher-order function to execute Solana transactions
///
/// Abstracts signer context retrieval and transaction signing, following the established
/// riglr pattern of using SignerContext for multi-tenant operation.
///
/// # Arguments
///
/// * `tx_creator` - Function that creates the transaction given a pubkey and RPC client
///
/// # Returns
///
/// Returns the transaction signature on success
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::utils::transaction::execute_solana_transaction;
/// use solana_sdk::transaction::Transaction;
/// use solana_system_interface::instruction as system_instruction;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let signature = execute_solana_transaction(|pubkey, client| async move {
///     let to = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".parse()?;
///     let instruction = system_instruction::transfer(&pubkey, &to, 1000000);
///
///     let recent_blockhash = client.get_latest_blockhash()?;
///     let mut tx = Transaction::new_with_payer(&[instruction], Some(&pubkey));
///     tx.sign(&[], recent_blockhash);
///
///     Ok(tx)
/// }).await?;
///
/// println!("Transaction sent: {}", signature);
/// # Ok(())
/// # }
/// ```
pub async fn execute_solana_transaction<F, Fut>(
    tx_creator: F,
) -> std::result::Result<String, SolanaToolError>
where
    F: FnOnce(Pubkey, Arc<RpcClient>) -> Fut + Send + 'static,
    Fut: Future<Output = std::result::Result<Transaction, SolanaToolError>> + Send + 'static,
{
    // Get signer from context
    let signer = SignerContext::current_as_solana()
        .await
        .map_err(SolanaToolError::SignerError)?;

    // Get Solana pubkey
    let pubkey_str = signer.pubkey();
    let pubkey = Pubkey::from_str(&pubkey_str)
        .map_err(|e| SolanaToolError::InvalidAddress(format!("Invalid pubkey format: {}", e)))?;

    // Get client from SignerContext
    let client = signer.client();

    // Execute transaction creator
    let mut tx = tx_creator(pubkey, client).await?;

    // Sign and send via signer context
    signer
        .sign_and_send_transaction(&mut tx)
        .await
        .map_err(SolanaToolError::SignerError)
}

/// Creates properly signed Solana transaction with mint keypair
///
/// This function handles the complex case where a transaction needs to be signed by both
/// the signer context (for fees) and a mint keypair (for token creation).
///
/// # Arguments
///
/// * `instructions` - The instructions to include in the transaction
/// * `mint_keypair` - The keypair for the mint account (must sign the transaction)
///
/// # Returns
///
/// Returns the transaction signature on success
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::utils::transaction::create_token_with_mint_keypair;
/// use solana_sdk::{instruction::Instruction, signature::Keypair};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mint_keypair = Keypair::new();
/// let instructions = vec![
///     // Token creation instructions here
/// ];
///
/// let signature = create_token_with_mint_keypair(instructions, &mint_keypair).await?;
/// println!("Token created with signature: {}", signature);
/// # Ok(())
/// # }
/// ```
pub async fn create_token_with_mint_keypair(
    instructions: Vec<Instruction>,
    mint_keypair: &Keypair,
) -> std::result::Result<String, SolanaToolError> {
    let signer = SignerContext::current_as_solana()
        .await
        .map_err(SolanaToolError::SignerError)?;
    let payer_pubkey = signer
        .pubkey()
        .parse()
        .map_err(|e| SolanaToolError::InvalidKey(format!("Invalid pubkey format: {}", e)))?;

    let mut tx = Transaction::new_with_payer(&instructions, Some(&payer_pubkey));

    // Get recent blockhash from SignerContext
    let client = signer.client();
    let recent_blockhash = client
        .get_latest_blockhash()
        .map_err(|e| SolanaToolError::SolanaClient(Box::new(e)))?;

    tx.partial_sign(&[mint_keypair], recent_blockhash);

    // Sign and send transaction via signer context
    let signature = signer
        .sign_and_send_transaction(&mut tx)
        .await
        .map_err(SolanaToolError::SignerError)?;

    Ok(signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

    #[test]
    fn test_config_defaults() {
        let config = TransactionConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30_000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.use_jitter);
    }

    // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

    // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

    // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

    #[test]
    fn test_classify_signer_error_solana_transaction() {
        use riglr_core::signer::SignerError;
        use solana_client::client_error::{ClientError, ClientErrorKind};

        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout")),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );
        let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

        let result = classify_signer_error_for_retry(&signer_error);
        // Should classify as retryable since it's a network timeout
        assert!(matches!(result, ErrorClass::Retryable));
    }

    #[test]
    fn test_classify_signer_error_no_signer_context() {
        use riglr_core::signer::SignerError;

        let signer_error = SignerError::NoSignerContext;
        let result = classify_signer_error_for_retry(&signer_error);

        assert!(matches!(result, ErrorClass::Permanent));
    }

    #[test]
    fn test_classify_signer_error_configuration() {
        use riglr_core::signer::SignerError;

        let signer_error = SignerError::Configuration("config error".to_string());
        let result = classify_signer_error_for_retry(&signer_error);

        assert!(matches!(result, ErrorClass::Permanent));
    }

    #[test]
    fn test_classify_signer_error_signing() {
        use riglr_core::signer::SignerError;

        let signer_error = SignerError::Signing("signing error".to_string());
        let result = classify_signer_error_for_retry(&signer_error);

        assert!(matches!(result, ErrorClass::Permanent));
    }

    #[test]
    fn test_classify_signer_error_other() {
        use riglr_core::signer::SignerError;

        // Test with a different variant that falls through to the default case
        let signer_error = SignerError::ProviderError("rpc error".to_string());
        let result = classify_signer_error_for_retry(&signer_error);

        assert!(matches!(result, ErrorClass::Retryable));
    }

    #[test]
    fn test_transaction_config_clone() {
        let config = TransactionConfig::default();
        let cloned = config.clone();

        assert_eq!(config.max_retries, cloned.max_retries);
        assert_eq!(config.base_delay_ms, cloned.base_delay_ms);
        assert_eq!(config.max_delay_ms, cloned.max_delay_ms);
        assert_eq!(config.backoff_multiplier, cloned.backoff_multiplier);
        assert_eq!(config.use_jitter, cloned.use_jitter);
    }

    #[test]
    fn test_transaction_config_debug() {
        let config = TransactionConfig::default();
        let debug_str = format!("{:?}", config);
        // TransactionConfig is now a type alias for RetryConfig
        assert!(debug_str.contains("RetryConfig"));
        assert!(debug_str.contains("max_retries"));
    }

    #[test]
    fn test_transaction_submission_result_clone() {
        let result = TransactionSubmissionResult {
            signature: "test_signature".to_string(),
            attempts: 2,
            total_duration_ms: 5000,
            confirmed: true,
        };

        let cloned = result.clone();
        assert_eq!(result.signature, cloned.signature);
        assert_eq!(result.attempts, cloned.attempts);
        assert_eq!(result.total_duration_ms, cloned.total_duration_ms);
        assert_eq!(result.confirmed, cloned.confirmed);
    }

    #[test]
    fn test_transaction_submission_result_debug() {
        let result = TransactionSubmissionResult {
            signature: "test_signature".to_string(),
            attempts: 2,
            total_duration_ms: 5000,
            confirmed: true,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("TransactionSubmissionResult"));
        assert!(debug_str.contains("test_signature"));
        assert!(debug_str.contains("attempts"));
    }

    #[test]
    fn test_transaction_config_custom_values() {
        let config = TransactionConfig {
            max_retries: 5,
            base_delay_ms: 500,
            max_delay_ms: 15_000,
            backoff_multiplier: 1.5,
            use_jitter: false,
        };

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay_ms, 500);
        assert_eq!(config.max_delay_ms, 15_000);
        assert_eq!(config.backoff_multiplier, 1.5);
        assert!(!config.use_jitter);
    }

    // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

    // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

    // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

    // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

    // Note: Integration tests for async functions like send_transaction_with_retry,
    // send_transaction, execute_solana_transaction, and create_token_with_mint_keypair
    // would require mocking SignerContext and RpcClient, which would be more appropriate
    // in integration tests or with a proper mocking framework. These functions primarily
    // orchestrate external dependencies and the core logic (retry calculation, error
    // classification) is already tested above.
}
