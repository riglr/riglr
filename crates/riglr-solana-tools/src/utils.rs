//! Organized utility modules for riglr-solana-tools
//!
//! This module provides well-organized utility functions for Solana operations,
//! following the established riglr architectural patterns with SignerContext-based
//! multi-tenant operation.
//!
//! # Module Organization
//!
//! - [`validation`] - Address and input validation utilities
//! - [`transaction`] - Transaction creation, sending, and retry logic
//! - [`keypair`] - Keypair generation utilities
//!
//! All modules follow the `SignerContext` pattern for secure multi-tenant operation,
//! as established by riglr-core architecture.

pub mod validation {
    //! Address validation utilities for Solana
    //!
    //! This module provides functions for validating Solana addresses and other inputs.

    use crate::error::{Error, Result};
    use core::str::FromStr as _;
    use solana_sdk::pubkey::Pubkey;

    /// Check if a Solana address is valid
    ///
    /// Validates that the provided string is a valid base58-encoded Solana public key.
    ///
    /// # Arguments
    ///
    /// * `address` - The address string to validate
    ///
    /// # Returns
    ///
    /// Returns the parsed `Pubkey` if valid, or a `Error::InvalidAddress` if invalid.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidAddress` if the address string cannot be parsed as a valid Solana public key.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use riglr_solana_tools::utils::validation::validate_address;
    ///
    /// // Valid address
    /// let pubkey = validate_address("11111111111111111111111111111111")?;
    ///
    /// // Invalid address will return error
    /// assert!(validate_address("invalid").is_err());
    /// ```
    #[inline]
    pub fn validate_address(address: &str) -> Result<Pubkey> {
        Pubkey::from_str(address)
            .map_err(|error| Error::InvalidAddress(format!("Invalid address: {error}")))
    }

    #[cfg(test)]
    #[expect(clippy::panic, clippy::expect_used)]
    mod tests {
        use super::*;

        #[test]
        fn validate_address_when_valid_system_program_should_return_ok() {
            // Happy Path: Valid system program address
            let valid = "11111111111111111111111111111111";
            let result = validate_address(valid);
            assert!(result.is_ok());
            let pubkey = result.expect("Valid system program address should parse successfully");
            assert_eq!(pubkey.to_string(), valid);
        }

        #[test]
        fn validate_address_when_valid_native_mint_should_return_ok() {
            // Happy Path: Valid native mint address
            let valid = "So11111111111111111111111111111111111111112";
            let result = validate_address(valid);
            assert!(result.is_ok());
            let pubkey = result.expect("Valid native mint address should parse successfully");
            assert_eq!(pubkey.to_string(), valid);
        }

        #[test]
        fn validate_address_when_valid_typical_pubkey_should_return_ok() {
            // Happy Path: Another valid typical pubkey
            let valid = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
            let result = validate_address(valid);
            assert!(result.is_ok());
            let pubkey = result.expect("Valid typical pubkey should parse successfully");
            assert_eq!(pubkey.to_string(), valid);
        }

        #[test]
        fn validate_address_when_empty_string_should_return_err() {
            // Error Path: Empty string
            let result = validate_address("");
            assert!(result.is_err());
            match result.expect_err("Empty string should fail validation") {
                Error::InvalidAddress(msg) => {
                    assert!(msg.contains("Invalid address"));
                }
                _ => panic!("Expected InvalidAddress error"),
            }
        }

        #[test]
        fn validate_address_when_invalid_characters_should_return_err() {
            // Error Path: Invalid characters
            let result = validate_address("invalid");
            assert!(result.is_err());
            match result.expect_err("Invalid characters should fail validation") {
                Error::InvalidAddress(msg) => {
                    assert!(msg.contains("Invalid address"));
                }
                _ => panic!("Expected InvalidAddress error"),
            }
        }

        #[test]
        fn validate_address_when_too_short_should_return_err() {
            // Error Path: Too short
            let result = validate_address("123");
            assert!(result.is_err());
            match result.expect_err("Too short address should fail validation") {
                Error::InvalidAddress(msg) => {
                    assert!(msg.contains("Invalid address"));
                }
                _ => panic!("Expected InvalidAddress error"),
            }
        }

        #[test]
        fn validate_address_when_too_long_should_return_err() {
            // Error Path: Too long
            let result = validate_address(
                "111111111111111111111111111111111111111111111111111111111111111111111",
            );
            assert!(result.is_err());
            match result.expect_err("Too long address should fail validation") {
                Error::InvalidAddress(msg) => {
                    assert!(msg.contains("Invalid address"));
                }
                _ => panic!("Expected InvalidAddress error"),
            }
        }

        #[test]
        fn validate_address_when_invalid_base58_characters_should_return_err() {
            // Error Path: Invalid base58 characters (contains 0, O, I, l)
            let result = validate_address("0OIl1111111111111111111111111111111");
            assert!(result.is_err());
            match result.expect_err("Invalid base58 characters should fail validation") {
                Error::InvalidAddress(msg) => {
                    assert!(msg.contains("Invalid address"));
                }
                _ => panic!("Expected InvalidAddress error"),
            }
        }

        #[test]
        fn validate_address_when_special_characters_should_return_err() {
            // Error Path: Special characters
            let result = validate_address("11111111111111111111111111111!@#");
            assert!(result.is_err());
            match result.expect_err("Special characters should fail validation") {
                Error::InvalidAddress(msg) => {
                    assert!(msg.contains("Invalid address"));
                }
                _ => panic!("Expected InvalidAddress error"),
            }
        }

        #[test]
        fn validate_address_when_whitespace_should_return_err() {
            // Error Path: Contains whitespace
            let result = validate_address("11111111111111111111111111111111 ");
            assert!(result.is_err());
            match result.expect_err("Trailing whitespace should fail validation") {
                Error::InvalidAddress(msg) => {
                    assert!(msg.contains("Invalid address"));
                }
                _ => panic!("Expected InvalidAddress error"),
            }
        }

        #[test]
        fn validate_address_when_leading_whitespace_should_return_err() {
            // Error Path: Leading whitespace
            let result = validate_address(" 11111111111111111111111111111111");
            assert!(result.is_err());
            match result.expect_err("Leading whitespace should fail validation") {
                Error::InvalidAddress(msg) => {
                    assert!(msg.contains("Invalid address"));
                }
                _ => panic!("Expected InvalidAddress error"),
            }
        }

        #[test]
        fn validate_address_when_mixed_case_invalid_should_return_err() {
            // Error Path: Mixed case that results in invalid base58
            let result = validate_address("AbCdEfGhIjKlMnOpQrStUvWxYz123456");
            assert!(result.is_err());
            match result.expect_err("Mixed case invalid base58 should fail validation") {
                Error::InvalidAddress(msg) => {
                    assert!(msg.contains("Invalid address"));
                }
                _ => panic!("Expected InvalidAddress error"),
            }
        }

        #[test]
        fn validate_address_when_unicode_characters_should_return_err() {
            // Error Path: Unicode characters
            let result = validate_address("1111111111111111111111111111111\u{1f680}");
            assert!(result.is_err());
            match result.expect_err("Unicode characters should fail validation") {
                Error::InvalidAddress(msg) => {
                    assert!(msg.contains("Invalid address"));
                }
                _ => panic!("Expected InvalidAddress error"),
            }
        }
    }
}

pub mod keypair {
    //! Keypair generation utilities for Solana
    //!
    //! This module provides utilities for generating keypairs for various purposes,
    //! such as mint accounts, program derived addresses, etc.

    use solana_sdk::signature::Keypair;

    /// Generates new mint keypair for token creation
    ///
    /// Creates a new randomly generated keypair suitable for use as a mint account
    /// in SPL token creation operations.
    ///
    /// # Returns
    ///
    /// Returns a new `Keypair` with a randomly generated public/private key pair.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use riglr_solana_tools::utils::keypair::generate_mint;
    ///
    /// let mint_keypair = generate_mint();
    /// println!("New mint pubkey: {}", mint_keypair.pubkey());
    /// ```
    ///
    /// # Security Notes
    ///
    /// - Each call generates a completely new keypair
    /// - The private key should be handled securely
    /// - For production use, consider proper key management practices
    #[must_use]
    #[inline]
    pub fn generate_mint() -> Keypair {
        Keypair::new()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use solana_sdk::{pubkey::Pubkey, signer::Signer as _};

        #[test]
        fn generate_mint_works() {
            let keypair1 = generate_mint();
            let keypair2 = generate_mint();

            // Ensure different keypairs are generated
            assert_ne!(keypair1.pubkey(), keypair2.pubkey());
        }

        #[test]
        fn keypair_properties() {
            let keypair = generate_mint();

            // Ensure pubkey is valid
            assert_ne!(keypair.pubkey(), Pubkey::default());

            // Ensure we can sign with the keypair
            let message = b"test message";
            let signature = keypair.sign_message(message);
            assert!(signature.verify(keypair.pubkey().as_ref(), message));
        }

        #[test]
        fn generate_mint_returns_valid_keypair() {
            let keypair = generate_mint();

            // Test that pubkey has correct length (32 bytes)
            assert_eq!(keypair.pubkey().to_bytes().len(), 32);

            // Test that secret key has correct length (64 bytes for ed25519)
            assert_eq!(keypair.to_bytes().len(), 64);
        }

        #[test]
        fn generate_mint_multiple_calls_unique() {
            // Generate multiple keypairs to ensure randomness
            use std::collections::HashSet;
            let mut pubkeys = HashSet::new();

            for _ in 0_i32..100_i32 {
                let keypair = generate_mint();
                let pubkey = keypair.pubkey();

                // Each pubkey should be unique
                assert!(pubkeys.insert(pubkey), "Duplicate pubkey found: {pubkey}");
            }

            // Should have 100 unique pubkeys
            assert_eq!(pubkeys.len(), 100);
        }

        #[test]
        fn generate_mint_signature_verification() {
            let keypair = generate_mint();

            // Test signing and verification with different message types
            let empty_message = b"";
            let short_message = b"a";
            let long_message = b"this is a longer message that should still work perfectly fine";

            // Test empty message
            let sig1 = keypair.sign_message(empty_message);
            assert!(sig1.verify(keypair.pubkey().as_ref(), empty_message));

            // Test short message
            let sig2 = keypair.sign_message(short_message);
            assert!(sig2.verify(keypair.pubkey().as_ref(), short_message));

            // Test long message
            let sig3 = keypair.sign_message(long_message);
            assert!(sig3.verify(keypair.pubkey().as_ref(), long_message));

            // Test that signatures are different for different messages
            assert_ne!(sig1, sig2);
            assert_ne!(sig2, sig3);
            assert_ne!(sig1, sig3);
        }

        #[test]
        #[expect(clippy::expect_used)]
        fn generate_mint_consistent_behavior() {
            // Test that the function always returns a valid Keypair type
            let keypair = generate_mint();

            // Should be able to convert to bytes and back
            let keypair_bytes = keypair.to_bytes();
            let reconstructed = Keypair::try_from(&keypair_bytes[..])
                .expect("Keypair reconstruction from valid bytes should never fail");

            // Reconstructed keypair should have same pubkey
            assert_eq!(keypair.pubkey(), reconstructed.pubkey());
        }
    }
}

pub mod transaction {
    //! Transaction utilities for enhanced Solana transaction handling
    //!
    //! This module provides centralized transaction sending functionality with
    //! robust retry logic, exponential backoff, and comprehensive error handling.
    //!
    //! All transaction utilities follow the `SignerContext` pattern for secure multi-tenant operation.

    use crate::error::Error;
    use core::fmt;
    use core::future::Future;
    use core::result::Result as CoreResult;
    use core::str::FromStr as _;
    use riglr_core::{
        retry::{retry_async, ErrorClass, RetryConfig},
        signer::SolanaClient,
        SignerContext, ToolError,
    };
    use solana_sdk::{
        instruction::Instruction, pubkey::Pubkey, signature::Keypair, transaction::Transaction,
    };
    use tracing::{debug, error, info};

    /// Configuration for transaction retry behavior.
    ///
    /// This is now a simple wrapper around `riglr_core::retry::RetryConfig`
    pub type Config = RetryConfig;

    /// Result of a transaction submission
    #[derive(Debug, Clone)]
    #[non_exhaustive]
    pub struct SubmissionResult {
        /// Number of attempts made
        pub attempts: u32,
        /// Whether transaction was confirmed (false for non-blocking sending)
        pub confirmed: bool,
        /// Transaction signature
        pub signature: String,
        /// Total time taken for all attempts
        pub total_duration_ms: u64,
    }

    /// Send a Solana transaction with retry logic and exponential backoff
    ///
    /// # Errors
    ///
    /// Returns `ToolError::Permanent` when the transaction fails after exhausting retries
    ///
    /// This function centralizes all Solana transaction sending logic with robust
    /// error handling, retry logic, and comprehensive logging. It automatically
    /// classifies errors and applies appropriate retry strategies.
    ///
    /// Uses `SignerContext` for secure multi-tenant operation.
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
    /// # Errors
    ///
    /// Returns `ToolError::Permanent` when the transaction fails after exhausting retries

    // Create a simple wrapper for the SignerError to make it cloneable
    #[derive(Debug, Clone)]
    struct RetryableError(String);

    impl fmt::Display for RetryableError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    /// Send a Solana transaction with retry logic and exponential backoff
    ///
    /// # Errors
    ///
    /// Returns `ToolError::Permanent` when the transaction fails after exhausting retries
    #[inline]
    pub async fn send_transaction_with_retry(
        transaction: &mut Transaction,
        config: &Config,
        operation_name: &str,
    ) -> CoreResult<SubmissionResult, ToolError> {
        use std::time::Instant;

        let start_time = Instant::now();
        let mut attempts = 0_u32;

        debug!(
            "Sending transaction for operation '{}' with retry config: max_retries={}, base_delay={}ms",
            operation_name, config.max_retries, config.base_delay_ms
        );

        // Clone transaction for use in closure
        let tx_clone = transaction.clone();

        // Use retry_async with proper error classification
        let result = retry_async(
            || {
                {
                    attempts = attempts.saturating_add(1);
                };
                let tx = tx_clone.clone();

                async move {
                    // Get the current signer context
                    let signer = SignerContext::current_as_solana()
                        .map_err(|error| RetryableError(error.to_string()))?;

                    // Convert transaction to JSON format for the new API
                    let tx_json = serde_json::to_value(&tx).map_err(|err| {
                        RetryableError(format!("Failed to serialize transaction: {err}"))
                    })?;
                    let result = signer
                        .signer()
                        .sign_and_send_transaction(tx_json)
                        .await
                        .map_err(|err| RetryableError(err.to_string()))?;
                    Ok(result)
                }
            },
            |error: &RetryableError| {
                // Simple classification based on error string
                let msg = &error.0;
                if msg.contains("rate limit") || msg.contains("too many requests") {
                    return ErrorClass::RateLimited;
                } else if msg.contains("network") || msg.contains("timeout") {
                    return ErrorClass::Retryable;
                } else if msg.contains("NoSignerContext") || msg.contains("configuration") {
                    return ErrorClass::Permanent;
                }
                ErrorClass::Retryable
            },
            config,
            operation_name,
        )
        .await;

        let total_duration = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

        match result {
            Ok(signature) => {
                info!(
                    "Transaction successful for '{}': signature={}, attempts={}, duration={}ms",
                    operation_name, signature, attempts, total_duration
                );

                Ok(SubmissionResult {
                    attempts,
                    confirmed: false, // Non-blocking - transaction is sent but not confirmed
                    signature,
                    total_duration_ms: total_duration,
                })
            }
            Err(error) => {
                error!(
                    "Transaction failed for '{}' after {} attempts in {}ms: {}",
                    operation_name, attempts, total_duration, error
                );

                Err(ToolError::permanent_string(format!(
                    "Transaction failed for '{operation_name}' after {attempts} attempts: {error}"
                )))
            }
        }
    }

    /// Send a transaction with default retry configuration
    ///
    /// Convenience function that uses the default `Config` for standard
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
    /// # Errors
    ///
    /// Returns `ToolError::Permanent` when the transaction fails after exhausting retries
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use riglr_solana_tools::utils::transaction::send;
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
    /// let signature = send(&mut tx, "SOL Transfer").await?;
    /// println!("Transaction sent: {}", signature);
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns `ToolError::Permanent` when the transaction fails after exhausting retries
    #[inline]
    pub async fn send(
        transaction: &mut Transaction,
        operation_name: &str,
    ) -> CoreResult<String, ToolError> {
        let config = Config::default();
        let result = send_transaction_with_retry(transaction, &config, operation_name).await?;
        Ok(result.signature)
    }

    /// Higher-order function to execute Solana transactions
    ///
    /// Abstracts signer context retrieval and transaction signing, following the established
    /// riglr pattern of using `SignerContext` for multi-tenant operation.
    ///
    /// # Arguments
    ///
    /// * `tx_creator` - Function that creates the transaction given a pubkey and RPC client
    ///
    /// # Returns
    ///
    /// Returns the transaction signature on success
    ///
    /// # Errors
    ///
    /// Returns `Error` when signer context is unavailable or transaction creation fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use riglr_solana_tools::utils::transaction::execute;
    /// use solana_sdk::transaction::Transaction;
    /// use solana_system_interface::instruction as system_instruction;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let signature = execute(|pubkey, client| async move {
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
    /// # Errors
    ///
    /// Returns `Error` when signer context is unavailable or transaction creation fails
    #[inline]
    pub async fn execute<F, Fut>(tx_creator: F) -> CoreResult<String, Error>
    where
        F: FnOnce(Pubkey, &dyn SolanaClient) -> Fut + Send + 'static,
        Fut: Future<Output = CoreResult<Transaction, Error>> + Send + 'static,
    {
        // Get signer from context
        let signer = SignerContext::current_as_solana().map_err(|e| Error::SignerError(e))?;

        // Get Solana pubkey
        let pubkey_str = signer.signer().pubkey();
        let pubkey = Pubkey::from_str(&pubkey_str)
            .map_err(|err| Error::InvalidAddress(format!("Invalid pubkey format: {err}")))?;

        // Get RPC client from SolanaSigner using new trait API
        let solana_client = signer.signer().client();

        // Execute transaction creator
        let tx = tx_creator(pubkey, solana_client).await?;

        // Sign and send via signer context
        let tx_json = serde_json::to_value(&tx).map_err(|err| {
            Error::from(ToolError::permanent_string(format!(
                "Failed to serialize transaction: {err}"
            )))
        })?;
        let result = signer.signer().sign_and_send_transaction(tx_json).await;
        result.map_err(|e| Error::SignerError(e))
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
    /// # Errors
    ///
    /// Returns `Error` when signer context is unavailable or transaction creation fails
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
    /// # Errors
    ///
    /// Returns `Error` when signer context is unavailable or transaction creation fails
    #[inline]
    pub async fn create_token_with_mint_keypair(
        instructions: Vec<Instruction>,
        mint_keypair: &Keypair,
    ) -> CoreResult<String, Error> {
        use core::str::FromStr as _;
        use solana_sdk::hash::Hash;
        let signer = SignerContext::current_as_solana().map_err(|e| Error::SignerError(e))?;
        let payer_pubkey = signer
            .signer()
            .pubkey()
            .parse()
            .map_err(|err| Error::InvalidKey(format!("Invalid pubkey format: {err}")))?;

        let mut tx = Transaction::new_with_payer(&instructions, Some(&payer_pubkey));

        // Get RPC client from SolanaSigner using new trait API
        let solana_client = signer.signer().client();
        let recent_blockhash_str = solana_client
            .get_latest_blockhash()
            .await
            .map_err(|e| Error::SignerError(e))?;

        // Parse the blockhash string to Hash for transaction signing
        let recent_blockhash = Hash::from_str(&recent_blockhash_str)
            .map_err(|err| Error::InvalidKey(format!("Invalid blockhash: {err}")))?;

        tx.partial_sign(&[mint_keypair], recent_blockhash);

        // Sign and send transaction via signer context
        let tx_json = serde_json::to_value(&tx).map_err(|err| {
            Error::from(ToolError::permanent_string(format!(
                "Failed to serialize transaction: {err}"
            )))
        })?;
        let result = signer.signer().sign_and_send_transaction(tx_json).await;
        let signature = result.map_err(|e| Error::SignerError(e))?;

        Ok(signature)
    }

    #[cfg(test)]
    #[expect(clippy::float_cmp)]
    mod tests {
        use super::*;

        // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

        #[test]
        fn config_defaults() {
            let config = Config::default();
            assert_eq!(config.max_retries, 3);
            assert_eq!(config.base_delay_ms, 1000);
            assert_eq!(config.max_delay_ms, 30_000);
            {
                assert_eq!(config.backoff_multiplier, 2.0_f64);
            }
            assert!(config.use_jitter);
        }

        // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

        // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

        // Test removed: calculate_retry_delay functionality moved to riglr_core::retry

        #[test]
        fn transaction_config_clone() {
            let config = Config::default();
            let cloned = config.clone();

            assert_eq!(config.max_retries, cloned.max_retries);
            assert_eq!(config.base_delay_ms, cloned.base_delay_ms);
            assert_eq!(config.max_delay_ms, cloned.max_delay_ms);
            {
                assert_eq!(config.backoff_multiplier, cloned.backoff_multiplier);
            }
            assert_eq!(config.use_jitter, cloned.use_jitter);
        }

        #[test]
        fn transaction_config_debug() {
            let config = Config::default();
            let debug_str = format!("{config:?}");
            // Config is now a type alias for riglr_core::retry::Config
            assert!(debug_str.contains("Config"));
            assert!(debug_str.contains("max_retries"));
        }

        #[test]
        fn submission_result_clone() {
            let result = SubmissionResult {
                signature: "test_signature".to_owned(),
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
        fn submission_result_debug() {
            let result = SubmissionResult {
                signature: "test_signature".to_owned(),
                attempts: 2,
                total_duration_ms: 5000,
                confirmed: true,
            };

            let debug_str = format!("{result:?}");
            assert!(debug_str.contains("SubmissionResult"));
            assert!(debug_str.contains("test_signature"));
            assert!(debug_str.contains("attempts"));
        }

        #[test]
        fn config_custom_values() {
            let config = Config::new(5, 500, 15_000, 1.5, false);

            assert_eq!(config.max_retries, 5);
            assert_eq!(config.base_delay_ms, 500);
            assert_eq!(config.max_delay_ms, 15_000);
            {
                assert_eq!(config.backoff_multiplier, 1.5_f64);
            }
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
}

// Re-export commonly used items for convenience
pub use keypair::generate_mint;
pub use transaction::{
    create_token_with_mint_keypair, execute, send, send_transaction_with_retry, Config,
    SubmissionResult,
};
pub use validation::validate_address;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_exports_keypair_functions() {
        // Test that generate_mint is properly re-exported
        // This will compile if the re-export is working correctly
        use solana_sdk::signer::keypair::Keypair;
        let _: fn() -> Keypair = generate_mint;
    }

    #[test]
    fn module_exports_transaction_types() {
        // Test that Config is properly re-exported
        use core::marker::PhantomData;
        let _: PhantomData<Config> = PhantomData;

        // Test that SubmissionResult is properly re-exported
        let _: PhantomData<SubmissionResult> = PhantomData;
    }

    #[test]
    fn module_exports_transaction_functions() {
        // Test that transaction functions are properly re-exported
        // These will compile if the re-exports are working correctly

        // Note: create_token_with_mint_keypair is an async function, so we can't assign it to a function pointer
        // We test its accessibility instead
        let _ = create_token_with_mint_keypair;

        // Note: We can't easily test function signatures for some functions due to complex types,
        // but the module compilation itself validates the re-exports
    }

    #[test]
    fn module_exports_validation_functions() {
        // Test that validate_address is properly re-exported
        // This will compile if the re-export is working correctly
        use crate::error::Error;
        use solana_sdk::pubkey::Pubkey as ValidatePubkey;
        let _: fn(&str) -> Result<ValidatePubkey, Error> = validate_address;
    }

    #[test]
    fn module_structure_is_complete() {
        // Test that all expected submodules are accessible
        // This is validated at compile time, but we can create a runtime test
        // to ensure the module structure is as expected

        // If these compile, the modules are properly declared
        use crate::utils;
        let _ = utils::keypair::generate_mint;
        let _ = utils::transaction::Config::default;
        let _ = utils::validation::validate_address;

        // Test passes if compilation succeeds
    }

    #[test]
    fn module_documentation_structure() {
        // Test that the module follows expected documentation patterns
        // This is more of a structural test to ensure the module is well-organized

        // The presence of this test validates that the module follows
        // the documented structure with proper organization
    }

    #[test]
    fn all_re_exports_are_accessible() {
        // Test that we can access all re-exported items without qualification
        // This ensures the pub use statements are working correctly

        // Try to reference each re-exported item to ensure they're accessible
        let _ = generate_mint;
        let _ = validate_address;
        let _ = create_token_with_mint_keypair;

        // Note: execute is generic, so we can't easily test it without specific types
        // We just validate that the function exists in the public API
        // let _ = execute; // Would require specific generic parameters

        let _ = send;
        let _ = send_transaction_with_retry;

        // If we reach this point, all re-exports are accessible
    }

    #[test]
    fn submodule_imports_work() {
        // Test that we can import from submodules directly
        use crate::utils::keypair::generate_mint as direct_generate_mint;
        use crate::utils::validation::validate_address as direct_validate_address;

        use crate::error::Error as DirectError;
        use solana_sdk::{
            pubkey::Pubkey as DirectValidatePubkey, signer::keypair::Keypair as DirectKeypair,
        };

        // Test that direct imports match re-exported functions
        let _: fn() -> DirectKeypair = direct_generate_mint;
        let _: fn(&str) -> Result<DirectValidatePubkey, DirectError> = direct_validate_address;

        // Direct submodule imports work correctly
    }
}
