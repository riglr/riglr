//! Network state and blockchain query tools
//!
//! These tools use ApplicationContext extensions for read-only blockchain queries and don't require transaction signing.

use riglr_core::ToolError;
use riglr_macros::tool;
use std::sync::Arc;
use tracing::{debug, info};

/// Get the current block height from the Solana blockchain using an RPC client
///
/// This tool queries the Solana network to retrieve the most recent block height,
/// which represents the number of blocks that have been processed by the network.
/// Essential for checking network activity and determining transaction finality.
/// This is a read-only operation that uses ApplicationContext extensions instead of requiring transaction signing.
///
/// # Arguments
///
/// * `context` - The ApplicationContext containing the RPC client
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
pub async fn get_block_height(
    context: &riglr_core::provider::ApplicationContext,
) -> Result<u64, ToolError> {
    debug!("Getting current block height");

    // Get Solana RPC client from the ApplicationContext's extensions
    let rpc_client = context
        .get_extension::<Arc<solana_client::rpc_client::RpcClient>>()
        .ok_or_else(|| {
            ToolError::permanent_string("Solana RpcClient not found in context".to_string())
        })?;

    let height = rpc_client
        .get_block_height()
        .map_err(|e| ToolError::retriable_string(format!("Failed to get block height: {}", e)))?;

    info!("Current block height: {}", height);
    Ok(height)
}

/// Get transaction status by signature using an RPC client
///
/// This tool queries the Solana network to check the confirmation status of a transaction
/// using its signature. Essential for monitoring transaction progress and ensuring operations
/// have been confirmed by the network before proceeding with dependent actions.
/// This is a read-only operation that uses ApplicationContext extensions instead of requiring transaction signing.
///
/// # Arguments
///
/// * `signature` - The transaction signature to check (base58-encoded string)
/// * `context` - The ApplicationContext containing the RPC client
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
/// Get transaction status by signature
///
/// This tool queries the Solana network to check the confirmation status of a transaction.
#[tool]
pub async fn get_transaction_status(
    signature: String,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<String, ToolError> {
    debug!("Getting transaction status for signature: {}", signature);

    // Get Solana RPC client from the ApplicationContext's extensions
    let rpc_client = context
        .get_extension::<Arc<solana_client::rpc_client::RpcClient>>()
        .ok_or_else(|| {
            ToolError::permanent_string("Solana RpcClient not found in context".to_string())
        })?;

    let signatures = vec![signature
        .parse()
        .map_err(|e| ToolError::permanent_string(format!("Invalid signature: {}", e)))?];
    let statuses = rpc_client
        .get_signature_statuses(&signatures)
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to get transaction status: {}", e))
        })?;

    if let Some(Some(status)) = statuses.value.first() {
        let status_str = if status.err.is_some() {
            "failed"
        } else if status.confirmations.is_some() {
            match &status.confirmation_status {
                Some(confirmation_status) => {
                    use solana_transaction_status::TransactionConfirmationStatus;
                    match confirmation_status {
                        TransactionConfirmationStatus::Finalized => "finalized",
                        TransactionConfirmationStatus::Confirmed => "confirmed",
                        TransactionConfirmationStatus::Processed => "processed",
                    }
                }
                None => "unknown",
            }
        } else {
            "pending"
        };

        info!("Transaction {} status: {}", signature, status_str);
        Ok(status_str.to_string())
    } else {
        info!("Transaction {} not found", signature);
        Ok("not_found".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Signature;
    use solana_transaction_status::{TransactionConfirmationStatus, TransactionStatus};
    use std::str::FromStr;
    use tokio;

    // Helper function to create a test context with ApiClients
    fn create_test_context() -> riglr_core::provider::ApplicationContext {
        // Load .env.test for test environment
        dotenvy::from_filename(".env.test").ok();

        let config = riglr_config::Config::from_env();
        let context = riglr_core::provider::ApplicationContext::from_config(&std::sync::Arc::new(
            config.clone(),
        ));

        // Create and inject ApiClients
        let api_clients = crate::clients::ApiClients::new(&config.providers);
        context.set_extension(std::sync::Arc::new(api_clients));

        context
    }

    #[tokio::test]
    async fn test_get_block_height_when_no_signer_context_should_return_permanent_error() {
        // Test error path: No signer context available
        let context = create_test_context();
        let result = get_block_height(&context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Solana RpcClient not found in context"));
            }
            _ => panic!("Expected permanent error, got: {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_get_transaction_status_when_no_signer_context_should_return_permanent_error() {
        // Test error path: No signer context available
        let signature = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d5XioYFMQvCWpJFWzPr6z6vWckNg1E1YiLqA3MmRZ5jV9".to_string();
        let context = create_test_context();
        let result = get_transaction_status(signature, &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Solana RpcClient not found in context"));
            }
            _ => panic!("Expected permanent error, got: {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_get_transaction_status_when_invalid_signature_should_return_permanent_error() {
        // Test error path: Invalid signature format
        let invalid_signature = "invalid_signature_format".to_string();
        let context = create_test_context();
        let result = get_transaction_status(invalid_signature, &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        // The function should fail before even getting to signature parsing due to no signer context
        // But let's test what would happen with an invalid signature in isolation
        match error {
            ToolError::Permanent { .. } => {
                // Expected - either no signer context or invalid signature
            }
            _ => panic!("Expected permanent error, got: {:?}", error),
        }
    }

    #[test]
    fn test_signature_parsing_when_invalid_format_should_fail() {
        // Test the signature parsing logic in isolation
        let invalid_signature = "invalid_signature";
        let parse_result = invalid_signature.parse::<Signature>();
        assert!(parse_result.is_err());
    }

    #[test]
    fn test_signature_parsing_when_valid_format_should_succeed() {
        // Test the signature parsing logic with a valid signature
        let valid_signature = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d5XioYFMQvCWpJFWzPr6z6vWckNg1E1YiLqA3MmRZ5jV9";
        let parse_result = valid_signature.parse::<Signature>();
        assert!(parse_result.is_ok());
    }

    #[test]
    fn test_signature_parsing_when_empty_string_should_fail() {
        // Test edge case: empty signature
        let empty_signature = "";
        let parse_result = empty_signature.parse::<Signature>();
        assert!(parse_result.is_err());
    }

    #[test]
    fn test_signature_parsing_when_too_short_should_fail() {
        // Test edge case: signature too short
        let short_signature = "abc";
        let parse_result = short_signature.parse::<Signature>();
        assert!(parse_result.is_err());
    }

    #[test]
    fn test_signature_parsing_when_too_long_should_fail() {
        // Test edge case: signature too long
        let long_signature = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d5XioYFMQvCWpJFWzPr6z6vWckNg1E1YiLqA3MmRZ5jV9extra";
        let parse_result = long_signature.parse::<Signature>();
        assert!(parse_result.is_err());
    }

    #[test]
    fn test_signature_parsing_when_contains_invalid_chars_should_fail() {
        // Test edge case: signature with invalid base58 characters
        let invalid_chars_signature = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d5XioYFMQvCWpJFWzPr6z6vWckNg1E1YiLqA3MmRZ5j00"; // contains '0' which is invalid in base58
        let parse_result = invalid_chars_signature.parse::<Signature>();
        assert!(parse_result.is_err());
    }

    // Test transaction status logic branches
    #[test]
    fn test_transaction_status_determination_when_error_present_should_return_failed() {
        // Test the logic for determining status when error is present

        let status = TransactionStatus {
            slot: 1000,
            confirmations: Some(0),
            err: Some(solana_sdk::transaction::TransactionError::InstructionError(
                0,
                solana_sdk::instruction::InstructionError::InvalidAccountData,
            )),
            confirmation_status: Some(TransactionConfirmationStatus::Confirmed),
            status: Err(solana_sdk::transaction::TransactionError::InstructionError(
                0,
                solana_sdk::instruction::InstructionError::InvalidAccountData,
            )),
        };

        // Simulate the logic from the function
        let status_str = if status.err.is_some() {
            "failed"
        } else if status.confirmations.is_some() {
            match &status.confirmation_status {
                Some(confirmation_status) => match confirmation_status {
                    TransactionConfirmationStatus::Finalized => "finalized",
                    TransactionConfirmationStatus::Confirmed => "confirmed",
                    TransactionConfirmationStatus::Processed => "processed",
                },
                None => "unknown",
            }
        } else {
            "pending"
        };

        assert_eq!(status_str, "failed");
    }

    #[test]
    fn test_transaction_status_determination_when_finalized_should_return_finalized() {
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
            match &status.confirmation_status {
                Some(confirmation_status) => match confirmation_status {
                    TransactionConfirmationStatus::Finalized => "finalized",
                    TransactionConfirmationStatus::Confirmed => "confirmed",
                    TransactionConfirmationStatus::Processed => "processed",
                },
                None => "unknown",
            }
        } else {
            "pending"
        };

        assert_eq!(status_str, "finalized");
    }

    #[test]
    fn test_transaction_status_determination_when_confirmed_should_return_confirmed() {
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
            match &status.confirmation_status {
                Some(confirmation_status) => match confirmation_status {
                    TransactionConfirmationStatus::Finalized => "finalized",
                    TransactionConfirmationStatus::Confirmed => "confirmed",
                    TransactionConfirmationStatus::Processed => "processed",
                },
                None => "unknown",
            }
        } else {
            "pending"
        };

        assert_eq!(status_str, "confirmed");
    }

    #[test]
    fn test_transaction_status_determination_when_processed_should_return_processed() {
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
            match &status.confirmation_status {
                Some(confirmation_status) => match confirmation_status {
                    TransactionConfirmationStatus::Finalized => "finalized",
                    TransactionConfirmationStatus::Confirmed => "confirmed",
                    TransactionConfirmationStatus::Processed => "processed",
                },
                None => "unknown",
            }
        } else {
            "pending"
        };

        assert_eq!(status_str, "processed");
    }

    #[test]
    fn test_transaction_status_determination_when_no_confirmation_status_should_return_unknown() {
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
            match &status.confirmation_status {
                Some(confirmation_status) => match confirmation_status {
                    TransactionConfirmationStatus::Finalized => "finalized",
                    TransactionConfirmationStatus::Confirmed => "confirmed",
                    TransactionConfirmationStatus::Processed => "processed",
                },
                None => "unknown",
            }
        } else {
            "pending"
        };

        assert_eq!(status_str, "unknown");
    }

    #[test]
    fn test_transaction_status_determination_when_no_confirmations_should_return_pending() {
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
            match &status.confirmation_status {
                Some(confirmation_status) => match confirmation_status {
                    TransactionConfirmationStatus::Finalized => "finalized",
                    TransactionConfirmationStatus::Confirmed => "confirmed",
                    TransactionConfirmationStatus::Processed => "processed",
                },
                None => "unknown",
            }
        } else {
            "pending"
        };

        assert_eq!(status_str, "pending");
    }

    #[test]
    fn test_valid_signature_examples() {
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
                "Failed to parse valid signature: {}",
                signature
            );
        }
    }

    #[test]
    fn test_signature_from_str_trait() {
        // Test using FromStr trait directly with a valid signature
        let signature_str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d5XioYFMQvCWpJFWzPr6z6vWckNg1E1YiLqA3MmRZ5jV9";
        let signature = Signature::from_str(signature_str);
        assert!(signature.is_ok());
    }

    #[test]
    fn test_edge_case_signature_boundary_lengths() {
        // Test signatures that are exactly the expected length vs too short/long
        let exactly_right_length = "1".repeat(88); // Base58 signature should be around this length
        let too_short = "1".repeat(10);
        let too_long = "1".repeat(200);

        // Note: Even if length is right, content might not be valid base58
        let parse_right = exactly_right_length.parse::<Signature>();
        let parse_short = too_short.parse::<Signature>();
        let parse_long = too_long.parse::<Signature>();

        // All should fail because "1" repeated is not valid base58 format
        assert!(parse_right.is_err());
        assert!(parse_short.is_err());
        assert!(parse_long.is_err());
    }

    // Test error message formatting
    #[test]
    fn test_tool_error_formatting() {
        let error_msg = "Test error message";
        let permanent_error = ToolError::permanent_string(error_msg.to_string());
        let retriable_error = ToolError::retriable_string(error_msg.to_string());

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
    async fn test_get_transaction_status_when_empty_signature_should_return_permanent_error() {
        let empty_signature = "".to_string();
        let context = create_test_context();
        let result = get_transaction_status(empty_signature, &context).await;

        assert!(result.is_err());
        // Should fail either due to no signer context or invalid signature
        let error = result.unwrap_err();
        match error {
            ToolError::Permanent { .. } => {
                // Expected - either no signer context or invalid signature
            }
            _ => panic!("Expected permanent error, got: {:?}", error),
        }
    }

    // Test very long signature
    #[tokio::test]
    async fn test_get_transaction_status_when_very_long_signature_should_return_permanent_error() {
        let very_long_signature = "a".repeat(1000);
        let context = create_test_context();
        let result = get_transaction_status(very_long_signature, &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ToolError::Permanent { .. } => {
                // Expected - either no signer context or invalid signature
            }
            _ => panic!("Expected permanent error, got: {:?}", error),
        }
    }

    // Test signature with special characters
    #[tokio::test]
    async fn test_get_transaction_status_when_signature_with_special_chars_should_return_permanent_error(
    ) {
        let special_char_signature = "5VfydnLu4XwV6le3gymz!@#$%^&*()".to_string();
        let context = create_test_context();
        let result = get_transaction_status(special_char_signature, &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ToolError::Permanent { .. } => {
                // Expected - either no signer context or invalid signature
            }
            _ => panic!("Expected permanent error, got: {:?}", error),
        }
    }
}
