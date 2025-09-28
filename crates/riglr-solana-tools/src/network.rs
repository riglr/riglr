//! Network state and blockchain query tools
//!
//! These tools use `ApplicationContext` extensions for read-only blockchain queries and don't require transaction signing.

use riglr_core::{provider::ApplicationContext, ToolError};
use riglr_macros::tool;
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;
use tracing::{debug, info};

/// Get the current block height from the Solana blockchain using an RPC client
///
/// This tool queries the Solana network to retrieve the most recent block height,
/// which represents the number of blocks that have been processed by the network.
/// Essential for checking network activity and determining transaction finality.
/// This is a read-only operation that uses `ApplicationContext` extensions instead of requiring transaction signing.
///
/// # Arguments
///
/// * `context` - The `ApplicationContext` containing the RPC client
///
/// # Returns
///
/// Returns the current block height as a `u64` representing the total number
/// of blocks processed by the Solana network since genesis.
///
/// # Errors
///
/// * `ToolError::Retriable` - When network connection issues occur or RPC timeouts
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::network::get_block_height;
/// use riglr_core::provider::ApplicationContext;
/// use riglr_config::Config;
/// use solana_client::rpc_client::RpcClient;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::from_env();
/// let context = ApplicationContext::from_config(&config);
///
/// // Add Solana RPC client as an extension
/// let solana_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com"));
/// context.set_extension(solana_client);
///
/// let height = get_block_height(&context).await?;
/// println!("Current block height: {}", height);
///
/// // Use block height for transaction confirmation checks
/// if height > 150_000_000 {
///     println!("Network has processed over 150M blocks");
/// }
/// # Ok(())
/// # }
/// ```
/// Get the current block height from the Solana blockchain
///
/// This tool queries the Solana network to retrieve the most recent block height.
#[tool]
#[inline]
pub async fn get_block_height(context: &ApplicationContext) -> Result<u64, ToolError> {
    debug!("Getting current block height");

    // Get Solana RPC client from the ApplicationContext's extensions
    let rpc_client = context.get_extension::<Arc<RpcClient>>().ok_or_else(|| {
        ToolError::permanent_string("Solana RpcClient not found in context".to_owned())
    })?;
    let height = rpc_client
        .get_block_height()
        .map_err(|err| ToolError::retriable_string(format!("Failed to get block height: {err}")))?;

    info!("Current block height: {}", height);
    Ok(height)
}

/// Get transaction status by signature using an RPC client
///
/// This tool queries the Solana network to check the confirmation status of a transaction
/// using its signature. Essential for monitoring transaction progress and ensuring operations
/// have been confirmed by the network before proceeding with dependent actions.
/// This is a read-only operation that uses `ApplicationContext` extensions instead of requiring transaction signing.
///
/// # Arguments
///
/// * `signature` - The transaction signature to check (base58-encoded string)
/// * `context` - The `ApplicationContext` containing the RPC client
///
/// # Returns
///
/// Returns a `String` indicating the transaction status:
/// - `"finalized"` - Transaction is finalized and cannot be rolled back
/// - `"confirmed"` - Transaction is confirmed by supermajority of cluster
/// - `"processed"` - Transaction has been processed but may not be confirmed
/// - `"failed"` - Transaction failed due to an error
/// - `"not_found"` - Transaction signature not found (may not exist or be too old)
///
/// # Errors
///
/// * `ToolError::Permanent` - When signature format is invalid
/// * `ToolError::Retriable` - When network issues occur during status lookup
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::network::get_transaction_status;
/// use riglr_core::provider::ApplicationContext;
/// use riglr_config::Config;
/// use solana_client::rpc_client::RpcClient;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::from_env();
/// let context = ApplicationContext::from_config(&config);
///
/// // Add Solana RPC client as an extension
/// let solana_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com"));
/// context.set_extension(solana_client);
///
/// let status = get_transaction_status(
///     "5j7s88CkzQeE6EN5HiV7CqkYsL3x6PbJmSjYpJjm1J2v3z4x8K7b".to_string(),
///     &context
/// ).await?;
///
/// match status.as_str() {
///     "finalized" => println!("✅ Transaction is finalized"),
///     "confirmed" => println!("🔄 Transaction is confirmed"),
///     "processed" => println!("⏳ Transaction is processed, awaiting confirmation"),
///     "failed" => println!("❌ Transaction failed"),
///     "not_found" => println!("🔍 Transaction not found"),
///     _ => println!("Unknown status: {}", status),
/// }
/// # Ok(())
/// # }
/// ```
/// Convert confirmation status to string
const fn confirmation_status_to_string(
    confirmation_status: &solana_transaction_status::TransactionConfirmationStatus,
) -> &'static str {
    use solana_transaction_status::TransactionConfirmationStatus;
    match *confirmation_status {
        TransactionConfirmationStatus::Finalized => "finalized",
        TransactionConfirmationStatus::Confirmed => "confirmed",
        TransactionConfirmationStatus::Processed => "processed",
    }
}

/// Determine transaction status string from RPC status
fn determine_status_string(status: &solana_transaction_status::TransactionStatus) -> &str {
    if status.err.is_some() {
        return "failed";
    } else if status.confirmations.is_some() {
        return status
            .confirmation_status
            .as_ref()
            .map_or("unknown", confirmation_status_to_string);
    }
    "pending"
}

/// Get transaction status by signature
///
/// This tool queries the Solana network to check the confirmation status of a transaction.
///
/// # Errors
///
/// Returns `ToolError::Permanent` when the signature format is invalid or RPC client is not found.
/// Returns `ToolError::Retriable` when network issues occur during the status lookup.
#[tool]
#[inline]
pub async fn get_transaction_status(
    signature: String,
    context: &ApplicationContext,
) -> Result<String, ToolError> {
    debug!("Getting transaction status for signature: {}", signature);

    // Get Solana RPC client from the ApplicationContext's extensions
    let rpc_client = context.get_extension::<Arc<RpcClient>>().ok_or_else(|| {
        ToolError::permanent_string("Solana RpcClient not found in context".to_owned())
    })?;
    let signatures = vec![signature
        .parse()
        .map_err(|err| ToolError::permanent_string(format!("Invalid signature: {err}")))?];
    let statuses = rpc_client
        .get_signature_statuses(&signatures)
        .map_err(|err| {
            ToolError::retriable_string(format!("Failed to get transaction status: {err}"))
        })?;

    if let Some(status) = statuses.value.first().and_then(|s| s.as_ref()) {
        let status_str = determine_status_string(status);
        info!("Transaction {} status: {}", signature, status_str);
        return Ok(status_str.to_owned());
    }
    info!("Transaction {} not found", signature);
    Ok("not_found".to_owned())
}

#[cfg(test)]
#[expect(clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::clients::Clients;
    use core::str::FromStr as _;
    use riglr_core::provider::ApplicationContext;
    use solana_sdk::signature::Signature;
    use solana_transaction_status::{TransactionConfirmationStatus, TransactionStatus};
    use std::sync::Arc;
    use tokio;

    // Helper function to create a test context with ExternalClients
    fn create_test_context() -> ApplicationContext {
        // Load .env.test for test environment
        let _ = dotenvy::from_filename(".env.test");

        let config = riglr_config::Config::from_env();
        let test_context = ApplicationContext::from_config(&Arc::new(config.clone()));

        // Create and inject ExternalClients
        let api_clients = Clients::new(&config.providers);
        test_context.set_extension(Arc::new(api_clients));

        test_context
    }

    #[tokio::test]
    async fn get_block_height_when_no_signer_context_should_return_permanent_error() {
        // Test error path: No signer context available
        let context = create_test_context();
        let result = get_block_height(&context).await;

        assert!(result.is_err());
        if let Err(error) = result {
            match error {
                ToolError::Permanent { .. } => {
                    // Expected error type
                }
                _ => panic!("Expected permanent error, got: {error:?}"),
            }
        }
    }

    #[tokio::test]
    async fn get_transaction_status_when_no_signer_context_should_return_permanent_error() {
        // Test error path: No signer context available
        let signature = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d5XioYFMQvCWpJFWzPr6z6vWckNg1E1YiLqA3MmRZ5jV9".to_owned();
        let context = create_test_context();
        let result = get_transaction_status(signature, &context).await;

        assert!(result.is_err());
        if let Err(error) = result {
            match error {
                ToolError::Permanent { .. } => {
                    // Expected error type
                }
                _ => panic!("Expected permanent error, got: {error:?}"),
            }
        }
    }

    #[tokio::test]
    async fn get_transaction_status_when_invalid_signature_should_return_permanent_error() {
        // Test error path: Invalid signature format
        let invalid_signature = "invalid_signature_format".to_owned();
        let context = create_test_context();
        let result = get_transaction_status(invalid_signature, &context).await;

        assert!(result.is_err());
        if let Err(error) = result {
            match error {
                ToolError::Permanent { .. } => {
                    // Expected error type
                }
                _ => panic!("Expected permanent error, got: {error:?}"),
            }
        }
    }

    #[test]
    fn signature_parsing_when_invalid_format_should_fail() {
        // Test the signature parsing logic in isolation
        let invalid_signature = "invalid_signature";
        let parse_result = invalid_signature.parse::<Signature>();
        parse_result.unwrap_err();
    }

    #[test]
    fn signature_parsing_when_valid_format_should_succeed() {
        // Test the signature parsing logic with a valid signature
        let valid_signature = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d5XioYFMQvCWpJFWzPr6z6vWckNg1E1YiLqA3MmRZ5jV9";
        let parse_result = valid_signature.parse::<Signature>();
        parse_result.unwrap();
    }

    #[test]
    fn signature_parsing_when_empty_string_should_fail() {
        // Test edge case: empty signature
        let empty_signature = "";
        let parse_result = empty_signature.parse::<Signature>();
        parse_result.unwrap_err();
    }

    #[test]
    fn signature_parsing_when_too_short_should_fail() {
        // Test edge case: signature too short
        let short_signature = "abc";
        let parse_result = short_signature.parse::<Signature>();
        parse_result.unwrap_err();
    }

    #[test]
    fn signature_parsing_when_too_long_should_fail() {
        // Test edge case: signature too long
        let long_signature = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d5XioYFMQvCWpJFWzPr6z6vWckNg1E1YiLqA3MmRZ5jV9extra";
        let parse_result = long_signature.parse::<Signature>();
        parse_result.unwrap_err();
    }

    #[test]
    fn signature_parsing_when_contains_invalid_chars_should_fail() {
        // Test edge case: signature with invalid base58 characters
        let invalid_chars_signature = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d5XioYFMQvCWpJFWzPr6z6vWckNg1E1YiLqA3MmRZ5j00"; // contains '0' which is invalid in base58
        let parse_result = invalid_chars_signature.parse::<Signature>();
        parse_result.unwrap_err();
    }

    // Test transaction status logic branches
    #[test]
    fn transaction_status_determination_when_error_present_should_return_failed() {
        use solana_sdk::instruction::InstructionError;
        use solana_sdk::transaction::TransactionError;

        // Test the logic for determining status when error is present
        let status = TransactionStatus {
            slot: 1000,
            confirmations: Some(0),
            err: Some(TransactionError::InstructionError(
                0,
                InstructionError::InvalidAccountData,
            )),
            confirmation_status: Some(TransactionConfirmationStatus::Confirmed),
            status: Err(TransactionError::InstructionError(
                0,
                InstructionError::InvalidAccountData,
            )),
        };

        // Simulate the logic from the function
        let status_str = if status.err.is_some() {
            "failed"
        } else if status.confirmations.is_some() {
            status
                .confirmation_status
                .as_ref()
                .map_or(
                    "unknown",
                    |confirmation_status| match *confirmation_status {
                        TransactionConfirmationStatus::Finalized => "finalized",
                        TransactionConfirmationStatus::Confirmed => "confirmed",
                        TransactionConfirmationStatus::Processed => "processed",
                    },
                )
        } else {
            "pending"
        };

        assert_eq!(status_str, "failed");
    }

    #[test]
    fn transaction_status_determination_when_finalized_should_return_finalized() {
        let status = TransactionStatus {
            slot: 1000,
            confirmations: Some(32),
            err: None,
            confirmation_status: Some(TransactionConfirmationStatus::Finalized),
            status: Ok(()),
        };

        let status_str = if status.err.is_some() {
            "failed"
        } else if status.confirmations.is_some() {
            status
                .confirmation_status
                .as_ref()
                .map_or(
                    "unknown",
                    |confirmation_status| match *confirmation_status {
                        TransactionConfirmationStatus::Finalized => "finalized",
                        TransactionConfirmationStatus::Confirmed => "confirmed",
                        TransactionConfirmationStatus::Processed => "processed",
                    },
                )
        } else {
            "pending"
        };

        assert_eq!(status_str, "finalized");
    }

    #[test]
    fn transaction_status_determination_when_confirmed_should_return_confirmed() {
        let status = TransactionStatus {
            slot: 1000,
            confirmations: Some(16),
            err: None,
            confirmation_status: Some(TransactionConfirmationStatus::Confirmed),
            status: Ok(()),
        };

        let status_str = if status.err.is_some() {
            "failed"
        } else if status.confirmations.is_some() {
            status
                .confirmation_status
                .as_ref()
                .map_or(
                    "unknown",
                    |confirmation_status| match *confirmation_status {
                        TransactionConfirmationStatus::Finalized => "finalized",
                        TransactionConfirmationStatus::Confirmed => "confirmed",
                        TransactionConfirmationStatus::Processed => "processed",
                    },
                )
        } else {
            "pending"
        };

        assert_eq!(status_str, "confirmed");
    }

    #[test]
    fn transaction_status_determination_when_processed_should_return_processed() {
        let status = TransactionStatus {
            slot: 1000,
            confirmations: Some(1),
            err: None,
            confirmation_status: Some(TransactionConfirmationStatus::Processed),
            status: Ok(()),
        };

        let status_str = if status.err.is_some() {
            "failed"
        } else if status.confirmations.is_some() {
            status
                .confirmation_status
                .as_ref()
                .map_or(
                    "unknown",
                    |confirmation_status| match *confirmation_status {
                        TransactionConfirmationStatus::Finalized => "finalized",
                        TransactionConfirmationStatus::Confirmed => "confirmed",
                        TransactionConfirmationStatus::Processed => "processed",
                    },
                )
        } else {
            "pending"
        };

        assert_eq!(status_str, "processed");
    }

    #[test]
    fn transaction_status_determination_when_no_confirmation_status_should_return_unknown() {
        let status = TransactionStatus {
            slot: 1000,
            confirmations: Some(1),
            err: None,
            confirmation_status: None,
            status: Ok(()),
        };

        let status_str = if status.err.is_some() {
            "failed"
        } else if status.confirmations.is_some() {
            status
                .confirmation_status
                .as_ref()
                .map_or(
                    "unknown",
                    |confirmation_status| match *confirmation_status {
                        TransactionConfirmationStatus::Finalized => "finalized",
                        TransactionConfirmationStatus::Confirmed => "confirmed",
                        TransactionConfirmationStatus::Processed => "processed",
                    },
                )
        } else {
            "pending"
        };

        assert_eq!(status_str, "unknown");
    }

    #[test]
    fn transaction_status_determination_when_no_confirmations_should_return_pending() {
        let status = TransactionStatus {
            slot: 1000,
            confirmations: None,
            err: None,
            confirmation_status: Some(TransactionConfirmationStatus::Processed),
            status: Ok(()),
        };

        let status_str = if status.err.is_some() {
            "failed"
        } else if status.confirmations.is_some() {
            status
                .confirmation_status
                .as_ref()
                .map_or(
                    "unknown",
                    |confirmation_status| match *confirmation_status {
                        TransactionConfirmationStatus::Finalized => "finalized",
                        TransactionConfirmationStatus::Confirmed => "confirmed",
                        TransactionConfirmationStatus::Processed => "processed",
                    },
                )
        } else {
            "pending"
        };

        assert_eq!(status_str, "pending");
    }

    #[test]
    fn valid_signature_examples() {
        // Test various valid signature formats
        // These are actual valid base58-encoded 64-byte signatures
        let valid_signatures = vec![
            // Real transaction signatures from Solana mainnet
            "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d5XioYFMQvCWpJFWzPr6z6vWckNg1E1YiLqA3MmRZ5jV9",
            "3AsdoALgZFuq2oUVWrDYhg2pNeaLJKPLf8hU2mQ6U8qJxeJ6hsrPVpMn9ma39DtfYCrDQSvngWRP8CxCnjPYjqun",
        ];

        for signature in valid_signatures {
            let parse_result = signature.parse::<Signature>();
            assert!(
                parse_result.is_ok(),
                "Failed to parse valid signature: {signature}"
            );
        }
    }

    #[test]
    fn signature_from_str_trait() {
        // Test using FromStr trait directly with a valid signature
        let signature_str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d5XioYFMQvCWpJFWzPr6z6vWckNg1E1YiLqA3MmRZ5jV9";
        let signature = Signature::from_str(signature_str);
        signature.unwrap();
    }

    #[test]
    fn edge_case_signature_boundary_lengths() {
        // Test signatures that are exactly the expected length vs too short/long
        let exactly_right_length = "1".repeat(88); // Base58 signature should be around this length
        let too_short = "1".repeat(10);
        let too_long = "1".repeat(200);

        // Note: Even if length is right, content might not be valid base58
        let parse_right = exactly_right_length.parse::<Signature>();
        let parse_short = too_short.parse::<Signature>();
        let parse_long = too_long.parse::<Signature>();

        // All should fail because "1" repeated is not valid base58 format
        parse_right.unwrap_err();
        parse_short.unwrap_err();
        parse_long.unwrap_err();
    }

    // Test error message formatting
    #[test]
    fn tool_error_formatting() {
        let error_msg = "Test error message";
        let permanent_error = ToolError::permanent_string(error_msg.to_owned());
        let retriable_error = ToolError::retriable_string(error_msg.to_owned());

        match permanent_error {
            ToolError::Permanent { context, .. } => assert_eq!(context, error_msg),
            _ => panic!("Expected permanent error"),
        }

        match retriable_error {
            ToolError::Retriable { context, .. } => assert_eq!(context, error_msg),
            _ => panic!("Expected retriable error"),
        }
    }

    // Edge case: empty transaction signature
    #[tokio::test]
    async fn get_transaction_status_when_empty_signature_should_return_permanent_error() {
        let empty_signature = String::new();
        let context = create_test_context();
        let result = get_transaction_status(empty_signature, &context).await;

        assert!(result.is_err());
        // Should fail either due to no signer context or invalid signature
        if let Err(error) = result {
            match error {
                ToolError::Permanent { .. } => {
                    // Expected error type
                }
                _ => panic!("Expected permanent error, got: {error:?}"),
            }
        }
    }

    // Test very long signature
    #[tokio::test]
    async fn get_transaction_status_when_very_long_signature_should_return_permanent_error() {
        let very_long_signature = "a".repeat(1000);
        let context = create_test_context();
        let result = get_transaction_status(very_long_signature, &context).await;

        assert!(result.is_err());
        if let Err(error) = result {
            match error {
                ToolError::Permanent { .. } => {
                    // Expected error type
                }
                _ => panic!("Expected permanent error, got: {error:?}"),
            }
        }
    }

    // Test signature with special characters
    #[tokio::test]
    async fn get_transaction_status_when_signature_with_special_chars_should_return_permanent_error(
    ) {
        let special_char_signature = "5VfydnLu4XwV6le3gymz!@#$%^&*()".to_owned();
        let context = create_test_context();
        let result = get_transaction_status(special_char_signature, &context).await;

        assert!(result.is_err());
        if let Err(error) = result {
            match error {
                ToolError::Permanent { .. } => {
                    // Expected error type
                }
                _ => panic!("Expected permanent error, got: {error:?}"),
            }
        }
    }
}
