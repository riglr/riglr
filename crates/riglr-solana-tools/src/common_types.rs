//! Common Solana types shared across riglr crates
//!
//! This module provides shared type definitions and utilities that are needed
//! by both riglr-solana-tools and riglr-cross-chain-tools to avoid duplication
//! and circular dependencies.

use core::str::FromStr as _;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Error types for Solana operations shared across crates
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SolanaCommonError {
    #[error("Client error: {0}")]
    /// Solana RPC client communication error
    ClientError(String),

    #[error("Invalid public key: {0}")]
    /// Invalid Solana public key format or encoding
    InvalidPubkey(String),

    #[error("Parse error: {0}")]
    /// Failed to parse Solana-related data
    ParseError(String),
}

/// Shared configuration for Solana operations
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SolanaConfig {
    /// Transaction commitment level (processed, confirmed, finalized)
    pub commitment: String,
    /// Solana RPC endpoint URL
    pub rpc_url: String,
    /// RPC request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for SolanaConfig {
    #[inline]
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_owned(),
            commitment: "confirmed".to_owned(),
            timeout_seconds: 30,
        }
    }
}

/// Common Solana account metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SolanaAccount {
    /// Whether this account must sign the transaction
    pub is_signer: bool,
    /// Whether this account can be modified by the transaction
    pub is_writable: bool,
    /// Base58-encoded Solana public key
    pub pubkey: String,
}

impl SolanaAccount {
    /// Create a new Solana account with validation
    ///
    /// # Errors
    ///
    /// Returns `SolanaCommonError::InvalidPubkey` if the provided pubkey string
    /// is not a valid Solana public key format.
    #[inline]
    pub fn new(
        pubkey: &str,
        is_signer: bool,
        is_writable: bool,
    ) -> Result<Self, SolanaCommonError> {
        // Validate pubkey format
        Pubkey::from_str(pubkey)
            .map_err(|_original_error| SolanaCommonError::InvalidPubkey(pubkey.to_owned()))?;

        Ok(Self {
            pubkey: pubkey.to_owned(),
            is_signer,
            is_writable,
        })
    }

    /// Convert the string pubkey to a Solana Pubkey type
    ///
    /// # Errors
    ///
    /// Returns `SolanaCommonError::InvalidPubkey` if the stored pubkey string
    /// is not a valid Solana public key format.
    #[inline]
    pub fn to_pubkey(&self) -> Result<Pubkey, SolanaCommonError> {
        Pubkey::from_str(&self.pubkey)
            .map_err(|_original_error| SolanaCommonError::InvalidPubkey(self.pubkey.clone()))
    }
}

/// Solana transaction metadata shared between crates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SolanaTransactionData {
    /// Accounts involved in the transaction
    pub accounts: Vec<SolanaAccount>,
    /// Compute unit limit (optional)
    pub compute_unit_limit: Option<u32>,
    /// Compute unit price in micro-lamports (optional)
    pub compute_unit_price: Option<u64>,
    /// Fee payer account (base58 encoded)
    pub fee_payer: String,
    /// Transaction instructions data (base64 encoded)
    pub instructions_data: String,
    /// Recent blockhash for the transaction
    pub recent_blockhash: String,
}

impl SolanaTransactionData {
    /// Get total number of accounts
    #[must_use]
    #[inline]
    pub const fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Decode instructions data from base64
    ///
    /// # Errors
    ///
    /// Returns `SolanaCommonError::ParseError` if the `instructions_data` string
    /// is not valid base64 encoded data.
    #[inline]
    pub fn decode_instructions(&self) -> Result<Vec<u8>, SolanaCommonError> {
        use base64::{engine::general_purpose, Engine as _};
        general_purpose::STANDARD
            .decode(&self.instructions_data)
            .map_err(|decode_error| {
                SolanaCommonError::ParseError(format!(
                    "Failed to decode instructions: {decode_error}"
                ))
            })
    }

    /// Get fee payer as Pubkey
    ///
    /// # Errors
    ///
    /// Returns `SolanaCommonError::InvalidPubkey` if the `fee_payer` string
    /// is not a valid Solana public key format.
    #[inline]
    pub fn fee_payer_pubkey(&self) -> Result<Pubkey, SolanaCommonError> {
        validate_solana_address(&self.fee_payer)
    }

    /// Create new Solana transaction data
    ///
    /// # Errors
    ///
    /// Returns `SolanaCommonError::InvalidPubkey` if the `fee_payer` string
    /// is not a valid Solana public key format.
    #[inline]
    pub fn new(
        recent_blockhash: String,
        fee_payer: String,
        instructions_data: String,
        accounts: Vec<SolanaAccount>,
    ) -> Result<Self, SolanaCommonError> {
        // Validate fee payer address
        validate_solana_address(&fee_payer)?;

        Ok(Self {
            recent_blockhash,
            fee_payer,
            instructions_data,
            accounts,
            compute_unit_limit: None,
            compute_unit_price: None,
        })
    }

    /// Get signer accounts
    #[must_use]
    #[inline]
    pub fn signers(&self) -> Vec<&SolanaAccount> {
        self.accounts.iter().filter(|acc| acc.is_signer).collect()
    }

    /// Add compute unit configuration
    #[must_use]
    #[inline]
    pub const fn with_compute_units(mut self, limit: u32, price: u64) -> Self {
        self.compute_unit_limit = Some(limit);
        self.compute_unit_price = Some(price);
        self
    }

    /// Get writable accounts
    #[must_use]
    #[inline]
    pub fn writable_accounts(&self) -> Vec<&SolanaAccount> {
        self.accounts.iter().filter(|acc| acc.is_writable).collect()
    }
}

/// Helper function to validate Solana addresses
///
/// # Errors
///
/// Returns `SolanaCommonError::InvalidPubkey` if the address string
/// is not a valid Solana public key format.
#[inline]
pub fn validate_solana_address(address: &str) -> Result<Pubkey, SolanaCommonError> {
    Pubkey::from_str(address)
        .map_err(|_original_error| SolanaCommonError::InvalidPubkey(address.to_owned()))
}

/// Format a Solana address for display
#[must_use]
#[inline]
pub fn format_solana_address(pubkey: &Pubkey) -> String {
    pubkey.to_string()
}

/// Parse a commitment level string
#[must_use]
#[inline]
pub fn parse_commitment(commitment: &str) -> solana_commitment_config::CommitmentLevel {
    match commitment.to_lowercase().as_str() {
        "processed" => solana_commitment_config::CommitmentLevel::Processed,
        "finalized" => solana_commitment_config::CommitmentLevel::Finalized,
        _ => solana_commitment_config::CommitmentLevel::Confirmed, // default (includes "confirmed")
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use solana_commitment_config::CommitmentLevel;

    #[test]
    fn test_solana_config_default() {
        let config = SolanaConfig::default();
        assert_eq!(config.rpc_url, "https://api.mainnet-beta.solana.com");
        assert_eq!(config.commitment, "confirmed");
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_solana_config_debug_clone() {
        let config = SolanaConfig::default();
        let cloned = config.clone();
        assert_eq!(config.rpc_url, cloned.rpc_url);
        assert_eq!(config.commitment, cloned.commitment);
        assert_eq!(config.timeout_seconds, cloned.timeout_seconds);

        // Test Debug trait
        let debug_output = format!("{config:?}");
        assert!(debug_output.contains("SolanaConfig"));
    }

    #[test]
    fn test_solana_account_new_valid_pubkey() {
        let valid_pubkey = "11111111111111111111111111111112"; // System program
        let account = SolanaAccount::new(valid_pubkey, true, false)
            .expect("Valid pubkey should create account");

        assert_eq!(account.pubkey, valid_pubkey);
        assert!(account.is_signer);
        assert!(!account.is_writable);
    }

    #[test]
    fn test_solana_account_new_invalid_pubkey() {
        let invalid_pubkey = "invalid_pubkey";
        let result = SolanaAccount::new(invalid_pubkey, false, true);

        assert!(result.is_err());
        match result.expect_err("Should fail with invalid pubkey") {
            SolanaCommonError::InvalidPubkey(addr) => assert_eq!(addr, invalid_pubkey),
            _ => panic!("Expected InvalidPubkey error"),
        }
    }

    #[test]
    fn test_solana_account_to_pubkey_valid() {
        let valid_pubkey_str = "11111111111111111111111111111112";
        let account = SolanaAccount::new(valid_pubkey_str, false, false)
            .expect("Valid pubkey should create account");
        let pubkey = account
            .to_pubkey()
            .expect("Valid account should convert to pubkey");
        assert_eq!(pubkey.to_string(), valid_pubkey_str);
    }

    #[test]
    fn test_solana_account_to_pubkey_invalid() {
        // Create account with valid pubkey first, then corrupt it
        let account = SolanaAccount {
            pubkey: "invalid_pubkey".to_string(),
            is_signer: false,
            is_writable: false,
        };

        let result = account.to_pubkey();
        assert!(result.is_err());
        match result.expect_err("Should fail with invalid pubkey") {
            SolanaCommonError::InvalidPubkey(addr) => assert_eq!(addr, "invalid_pubkey"),
            _ => panic!("Expected InvalidPubkey error"),
        }
    }

    #[test]
    fn test_solana_account_serde() {
        let account = SolanaAccount::new("11111111111111111111111111111112", true, true)
            .expect("Valid pubkey should create account");

        // Test serialization
        let serialized = serde_json::to_string(&account).expect("Account should serialize to JSON");
        assert!(serialized.contains("11111111111111111111111111111112"));

        // Test deserialization
        let deserialized: SolanaAccount = serde_json::from_str(&serialized)
            .expect("Serialized JSON should deserialize to account");
        assert_eq!(deserialized.pubkey, account.pubkey);
        assert_eq!(deserialized.is_signer, account.is_signer);
        assert_eq!(deserialized.is_writable, account.is_writable);
    }

    #[test]
    fn test_solana_transaction_data_new_valid() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false)
            .expect("Valid pubkey should create account")];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .expect("Valid data should create transaction");

        assert_eq!(tx_data.recent_blockhash, "blockhash123");
        assert_eq!(tx_data.fee_payer, valid_pubkey);
        assert_eq!(tx_data.instructions_data, "base64data");
        assert_eq!(tx_data.accounts.len(), 1);
        assert!(tx_data.compute_unit_limit.is_none());
        assert!(tx_data.compute_unit_price.is_none());
    }

    #[test]
    fn test_solana_transaction_data_new_invalid_fee_payer() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false)
            .expect("Valid pubkey should create account")];

        let result = SolanaTransactionData::new(
            "blockhash123".to_string(),
            "invalid_fee_payer".to_string(),
            "base64data".to_string(),
            accounts,
        );

        assert!(result.is_err());
        match result.expect_err("Should fail with invalid fee payer") {
            SolanaCommonError::InvalidPubkey(addr) => assert_eq!(addr, "invalid_fee_payer"),
            _ => panic!("Expected InvalidPubkey error"),
        }
    }

    #[test]
    fn test_solana_transaction_data_with_compute_units() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false)
            .expect("Valid pubkey should create account")];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .expect("Valid data should create transaction")
        .with_compute_units(200_000, 1000);

        assert_eq!(tx_data.compute_unit_limit, Some(200_000));
        assert_eq!(tx_data.compute_unit_price, Some(1000));
    }

    #[test]
    fn test_solana_transaction_data_fee_payer_pubkey_valid() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false)
            .expect("Valid pubkey should create account")];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .expect("Valid data should create transaction");

        let pubkey = tx_data
            .fee_payer_pubkey()
            .expect("Valid fee payer should convert to pubkey");
        assert_eq!(pubkey.to_string(), valid_pubkey);
    }

    #[test]
    fn test_solana_transaction_data_decode_instructions_valid() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false)
            .expect("Valid pubkey should create account")];
        let base64_data = "SGVsbG8gV29ybGQ="; // "Hello World" in base64

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            base64_data.to_string(),
            accounts,
        )
        .expect("Valid data should create transaction");

        let decoded = tx_data
            .decode_instructions()
            .expect("Valid base64 should decode");
        assert_eq!(decoded, b"Hello World");
    }

    #[test]
    fn test_solana_transaction_data_decode_instructions_invalid() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false)
            .expect("Valid pubkey should create account")];
        let invalid_base64 = "invalid_base64!@#";

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            invalid_base64.to_string(),
            accounts,
        )
        .expect("Valid data should create transaction");

        let result = tx_data.decode_instructions();
        assert!(result.is_err());
        match result.expect_err("Should fail to decode invalid base64") {
            SolanaCommonError::ParseError(msg) => {
                assert!(msg.contains("Failed to decode instructions"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_solana_transaction_data_account_count() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![
            SolanaAccount::new(valid_pubkey, true, false)
                .expect("Valid pubkey should create account"),
            SolanaAccount::new(valid_pubkey, false, true)
                .expect("Valid pubkey should create account"),
        ];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .expect("Valid data should create transaction");

        assert_eq!(tx_data.account_count(), 2);
    }

    #[test]
    fn test_solana_transaction_data_signers() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![
            SolanaAccount::new(valid_pubkey, true, false)
                .expect("Valid pubkey should create account"), // signer
            SolanaAccount::new(valid_pubkey, false, true)
                .expect("Valid pubkey should create account"), // not signer
            SolanaAccount::new(valid_pubkey, true, true)
                .expect("Valid pubkey should create account"), // signer
        ];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .expect("Valid data should create transaction");

        let signers = tx_data.signers();
        assert_eq!(signers.len(), 2);
        assert!(signers.first().expect("Should have first signer").is_signer);
        assert!(signers.get(1).expect("Should have second signer").is_signer);
    }

    #[test]
    fn test_solana_transaction_data_writable_accounts() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![
            SolanaAccount::new(valid_pubkey, true, false)
                .expect("Valid pubkey should create account"), // not writable
            SolanaAccount::new(valid_pubkey, false, true)
                .expect("Valid pubkey should create account"), // writable
            SolanaAccount::new(valid_pubkey, true, true)
                .expect("Valid pubkey should create account"), // writable
        ];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .expect("Valid data should create transaction");

        let writable = tx_data.writable_accounts();
        assert_eq!(writable.len(), 2);
        assert!(
            writable
                .first()
                .expect("Should have first writable")
                .is_writable
        );
        assert!(
            writable
                .get(1)
                .expect("Should have second writable")
                .is_writable
        );
    }

    #[test]
    fn test_solana_transaction_data_empty_accounts() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .expect("Valid data should create transaction");

        assert_eq!(tx_data.account_count(), 0);
        assert_eq!(tx_data.signers().len(), 0);
        assert_eq!(tx_data.writable_accounts().len(), 0);
    }

    #[test]
    fn test_solana_transaction_data_serde() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false)
            .expect("Valid pubkey should create account")];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .expect("Valid data should create transaction")
        .with_compute_units(200_000, 1000);

        // Test serialization
        let serialized =
            serde_json::to_string(&tx_data).expect("Transaction data should serialize to JSON");
        assert!(serialized.contains("blockhash123"));

        // Test deserialization
        let deserialized: SolanaTransactionData = serde_json::from_str(&serialized)
            .expect("Serialized JSON should deserialize to transaction data");
        assert_eq!(deserialized.recent_blockhash, tx_data.recent_blockhash);
        assert_eq!(deserialized.fee_payer, tx_data.fee_payer);
        assert_eq!(deserialized.instructions_data, tx_data.instructions_data);
        assert_eq!(deserialized.compute_unit_limit, tx_data.compute_unit_limit);
        assert_eq!(deserialized.compute_unit_price, tx_data.compute_unit_price);
    }

    #[test]
    fn test_validate_solana_address_valid() {
        let valid_address = "11111111111111111111111111111112";
        let pubkey = validate_solana_address(valid_address).expect("Valid address should parse");
        assert_eq!(pubkey.to_string(), valid_address);
    }

    #[test]
    fn test_validate_solana_address_invalid() {
        let invalid_address = "invalid_address";
        let result = validate_solana_address(invalid_address);
        assert!(result.is_err());
        match result.expect_err("Should fail with invalid address") {
            SolanaCommonError::InvalidPubkey(addr) => assert_eq!(addr, invalid_address),
            _ => panic!("Expected InvalidPubkey error"),
        }
    }

    #[test]
    fn test_format_solana_address() {
        let pubkey = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid pubkey string should parse");
        let formatted = format_solana_address(&pubkey);
        assert_eq!(formatted, "11111111111111111111111111111112");
    }

    #[test]
    fn test_parse_commitment_processed() {
        assert_eq!(parse_commitment("processed"), CommitmentLevel::Processed);
        assert_eq!(parse_commitment("PROCESSED"), CommitmentLevel::Processed);
        assert_eq!(parse_commitment("Processed"), CommitmentLevel::Processed);
    }

    #[test]
    fn test_parse_commitment_confirmed() {
        assert_eq!(parse_commitment("confirmed"), CommitmentLevel::Confirmed);
        assert_eq!(parse_commitment("CONFIRMED"), CommitmentLevel::Confirmed);
        assert_eq!(parse_commitment("Confirmed"), CommitmentLevel::Confirmed);
    }

    #[test]
    fn test_parse_commitment_finalized() {
        assert_eq!(parse_commitment("finalized"), CommitmentLevel::Finalized);
        assert_eq!(parse_commitment("FINALIZED"), CommitmentLevel::Finalized);
        assert_eq!(parse_commitment("Finalized"), CommitmentLevel::Finalized);
    }

    #[test]
    fn test_parse_commitment_default() {
        // Test various invalid/unknown commitment levels that should default to Confirmed
        assert_eq!(parse_commitment("unknown"), CommitmentLevel::Confirmed);
        assert_eq!(parse_commitment(""), CommitmentLevel::Confirmed);
        assert_eq!(parse_commitment("invalid"), CommitmentLevel::Confirmed);
        assert_eq!(parse_commitment("123"), CommitmentLevel::Confirmed);
        assert_eq!(parse_commitment("recent"), CommitmentLevel::Confirmed);
    }

    #[test]
    fn test_solana_common_error_display() {
        let invalid_pubkey_err = SolanaCommonError::InvalidPubkey("test_key".to_string());
        assert_eq!(
            format!("{invalid_pubkey_err}"),
            "Invalid public key: test_key"
        );

        let client_err = SolanaCommonError::ClientError("connection failed".to_string());
        assert_eq!(format!("{client_err}"), "Client error: connection failed");

        let parse_err = SolanaCommonError::ParseError("invalid format".to_string());
        assert_eq!(format!("{parse_err}"), "Parse error: invalid format");
    }

    #[test]
    fn test_solana_common_error_debug() {
        let error = SolanaCommonError::InvalidPubkey("test".to_string());
        let debug_output = format!("{error:?}");
        assert!(debug_output.contains("InvalidPubkey"));
        assert!(debug_output.contains("test"));
    }
}
