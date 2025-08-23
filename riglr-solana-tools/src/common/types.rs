//! Common Solana types shared across riglr crates
//!
//! This module provides shared type definitions and utilities that are needed
//! by both riglr-solana-tools and riglr-cross-chain-tools to avoid duplication
//! and circular dependencies.

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Error types for Solana operations shared across crates
#[derive(Debug, thiserror::Error)]
pub enum SolanaCommonError {
    #[error("Invalid public key: {0}")]
    /// Invalid Solana public key format or encoding
    InvalidPubkey(String),

    #[error("Client error: {0}")]
    /// Solana RPC client communication error
    ClientError(String),

    #[error("Parse error: {0}")]
    /// Failed to parse Solana-related data
    ParseError(String),
}

/// Shared configuration for Solana operations
#[derive(Debug, Clone)]
pub struct SolanaConfig {
    /// Solana RPC endpoint URL
    pub rpc_url: String,
    /// Transaction commitment level (processed, confirmed, finalized)
    pub commitment: String,
    /// RPC request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for SolanaConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            commitment: "confirmed".to_string(),
            timeout_seconds: 30,
        }
    }
}

/// Common Solana account metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaAccount {
    /// Base58-encoded Solana public key
    pub pubkey: String,
    /// Whether this account must sign the transaction
    pub is_signer: bool,
    /// Whether this account can be modified by the transaction
    pub is_writable: bool,
}

impl SolanaAccount {
    /// Create a new Solana account with validation
    pub fn new(
        pubkey: &str,
        is_signer: bool,
        is_writable: bool,
    ) -> Result<Self, SolanaCommonError> {
        // Validate pubkey format
        Pubkey::from_str(pubkey)
            .map_err(|_| SolanaCommonError::InvalidPubkey(pubkey.to_string()))?;

        Ok(Self {
            pubkey: pubkey.to_string(),
            is_signer,
            is_writable,
        })
    }

    /// Convert the string pubkey to a Solana Pubkey type
    pub fn to_pubkey(&self) -> Result<Pubkey, SolanaCommonError> {
        Pubkey::from_str(&self.pubkey)
            .map_err(|_| SolanaCommonError::InvalidPubkey(self.pubkey.clone()))
    }
}

/// Solana transaction metadata shared between crates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaTransactionData {
    /// Recent blockhash for the transaction
    pub recent_blockhash: String,
    /// Fee payer account (base58 encoded)
    pub fee_payer: String,
    /// Transaction instructions data (base64 encoded)
    pub instructions_data: String,
    /// Accounts involved in the transaction
    pub accounts: Vec<SolanaAccount>,
    /// Compute unit limit (optional)
    pub compute_unit_limit: Option<u32>,
    /// Compute unit price in micro-lamports (optional)
    pub compute_unit_price: Option<u64>,
}

impl SolanaTransactionData {
    /// Create new Solana transaction data
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

    /// Add compute unit configuration
    pub fn with_compute_units(mut self, limit: u32, price: u64) -> Self {
        self.compute_unit_limit = Some(limit);
        self.compute_unit_price = Some(price);
        self
    }

    /// Get fee payer as Pubkey
    pub fn fee_payer_pubkey(&self) -> Result<Pubkey, SolanaCommonError> {
        validate_solana_address(&self.fee_payer)
    }

    /// Decode instructions data from base64
    pub fn decode_instructions(&self) -> Result<Vec<u8>, SolanaCommonError> {
        use base64::{engine::general_purpose, Engine as _};
        general_purpose::STANDARD
            .decode(&self.instructions_data)
            .map_err(|e| {
                SolanaCommonError::ParseError(format!("Failed to decode instructions: {}", e))
            })
    }

    /// Get total number of accounts
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Get signer accounts
    pub fn signers(&self) -> Vec<&SolanaAccount> {
        self.accounts.iter().filter(|acc| acc.is_signer).collect()
    }

    /// Get writable accounts
    pub fn writable_accounts(&self) -> Vec<&SolanaAccount> {
        self.accounts.iter().filter(|acc| acc.is_writable).collect()
    }
}

/// Helper function to validate Solana addresses
pub fn validate_solana_address(address: &str) -> Result<Pubkey, SolanaCommonError> {
    Pubkey::from_str(address).map_err(|_| SolanaCommonError::InvalidPubkey(address.to_string()))
}

/// Format a Solana address for display
pub fn format_solana_address(pubkey: &Pubkey) -> String {
    pubkey.to_string()
}

/// Parse a commitment level string
pub fn parse_commitment(commitment: &str) -> solana_commitment_config::CommitmentLevel {
    match commitment.to_lowercase().as_str() {
        "processed" => solana_commitment_config::CommitmentLevel::Processed,
        "confirmed" => solana_commitment_config::CommitmentLevel::Confirmed,
        "finalized" => solana_commitment_config::CommitmentLevel::Finalized,
        _ => solana_commitment_config::CommitmentLevel::Confirmed, // default
    }
}

#[cfg(test)]
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
        let debug_output = format!("{:?}", config);
        assert!(debug_output.contains("SolanaConfig"));
    }

    #[test]
    fn test_solana_account_new_valid_pubkey() {
        let valid_pubkey = "11111111111111111111111111111112"; // System program
        let account = SolanaAccount::new(valid_pubkey, true, false).unwrap();

        assert_eq!(account.pubkey, valid_pubkey);
        assert!(account.is_signer);
        assert!(!account.is_writable);
    }

    #[test]
    fn test_solana_account_new_invalid_pubkey() {
        let invalid_pubkey = "invalid_pubkey";
        let result = SolanaAccount::new(invalid_pubkey, false, true);

        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaCommonError::InvalidPubkey(addr) => assert_eq!(addr, invalid_pubkey),
            _ => panic!("Expected InvalidPubkey error"),
        }
    }

    #[test]
    fn test_solana_account_to_pubkey_valid() {
        let valid_pubkey_str = "11111111111111111111111111111112";
        let account = SolanaAccount::new(valid_pubkey_str, false, false).unwrap();
        let pubkey = account.to_pubkey().unwrap();
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
        match result.unwrap_err() {
            SolanaCommonError::InvalidPubkey(addr) => assert_eq!(addr, "invalid_pubkey"),
            _ => panic!("Expected InvalidPubkey error"),
        }
    }

    #[test]
    fn test_solana_account_serde() {
        let account = SolanaAccount::new("11111111111111111111111111111112", true, true).unwrap();

        // Test serialization
        let serialized = serde_json::to_string(&account).unwrap();
        assert!(serialized.contains("11111111111111111111111111111112"));

        // Test deserialization
        let deserialized: SolanaAccount = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.pubkey, account.pubkey);
        assert_eq!(deserialized.is_signer, account.is_signer);
        assert_eq!(deserialized.is_writable, account.is_writable);
    }

    #[test]
    fn test_solana_transaction_data_new_valid() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false).unwrap()];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .unwrap();

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
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false).unwrap()];

        let result = SolanaTransactionData::new(
            "blockhash123".to_string(),
            "invalid_fee_payer".to_string(),
            "base64data".to_string(),
            accounts,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaCommonError::InvalidPubkey(addr) => assert_eq!(addr, "invalid_fee_payer"),
            _ => panic!("Expected InvalidPubkey error"),
        }
    }

    #[test]
    fn test_solana_transaction_data_with_compute_units() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false).unwrap()];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .unwrap()
        .with_compute_units(200000, 1000);

        assert_eq!(tx_data.compute_unit_limit, Some(200000));
        assert_eq!(tx_data.compute_unit_price, Some(1000));
    }

    #[test]
    fn test_solana_transaction_data_fee_payer_pubkey_valid() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false).unwrap()];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .unwrap();

        let pubkey = tx_data.fee_payer_pubkey().unwrap();
        assert_eq!(pubkey.to_string(), valid_pubkey);
    }

    #[test]
    fn test_solana_transaction_data_decode_instructions_valid() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false).unwrap()];
        let base64_data = "SGVsbG8gV29ybGQ="; // "Hello World" in base64

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            base64_data.to_string(),
            accounts,
        )
        .unwrap();

        let decoded = tx_data.decode_instructions().unwrap();
        assert_eq!(decoded, b"Hello World");
    }

    #[test]
    fn test_solana_transaction_data_decode_instructions_invalid() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false).unwrap()];
        let invalid_base64 = "invalid_base64!@#";

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            invalid_base64.to_string(),
            accounts,
        )
        .unwrap();

        let result = tx_data.decode_instructions();
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaCommonError::ParseError(msg) => {
                assert!(msg.contains("Failed to decode instructions"))
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_solana_transaction_data_account_count() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![
            SolanaAccount::new(valid_pubkey, true, false).unwrap(),
            SolanaAccount::new(valid_pubkey, false, true).unwrap(),
        ];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .unwrap();

        assert_eq!(tx_data.account_count(), 2);
    }

    #[test]
    fn test_solana_transaction_data_signers() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![
            SolanaAccount::new(valid_pubkey, true, false).unwrap(), // signer
            SolanaAccount::new(valid_pubkey, false, true).unwrap(), // not signer
            SolanaAccount::new(valid_pubkey, true, true).unwrap(),  // signer
        ];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .unwrap();

        let signers = tx_data.signers();
        assert_eq!(signers.len(), 2);
        assert!(signers[0].is_signer);
        assert!(signers[1].is_signer);
    }

    #[test]
    fn test_solana_transaction_data_writable_accounts() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![
            SolanaAccount::new(valid_pubkey, true, false).unwrap(), // not writable
            SolanaAccount::new(valid_pubkey, false, true).unwrap(), // writable
            SolanaAccount::new(valid_pubkey, true, true).unwrap(),  // writable
        ];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .unwrap();

        let writable = tx_data.writable_accounts();
        assert_eq!(writable.len(), 2);
        assert!(writable[0].is_writable);
        assert!(writable[1].is_writable);
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
        .unwrap();

        assert_eq!(tx_data.account_count(), 0);
        assert_eq!(tx_data.signers().len(), 0);
        assert_eq!(tx_data.writable_accounts().len(), 0);
    }

    #[test]
    fn test_solana_transaction_data_serde() {
        let valid_pubkey = "11111111111111111111111111111112";
        let accounts = vec![SolanaAccount::new(valid_pubkey, true, false).unwrap()];

        let tx_data = SolanaTransactionData::new(
            "blockhash123".to_string(),
            valid_pubkey.to_string(),
            "base64data".to_string(),
            accounts,
        )
        .unwrap()
        .with_compute_units(200000, 1000);

        // Test serialization
        let serialized = serde_json::to_string(&tx_data).unwrap();
        assert!(serialized.contains("blockhash123"));

        // Test deserialization
        let deserialized: SolanaTransactionData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.recent_blockhash, tx_data.recent_blockhash);
        assert_eq!(deserialized.fee_payer, tx_data.fee_payer);
        assert_eq!(deserialized.instructions_data, tx_data.instructions_data);
        assert_eq!(deserialized.compute_unit_limit, tx_data.compute_unit_limit);
        assert_eq!(deserialized.compute_unit_price, tx_data.compute_unit_price);
    }

    #[test]
    fn test_validate_solana_address_valid() {
        let valid_address = "11111111111111111111111111111112";
        let pubkey = validate_solana_address(valid_address).unwrap();
        assert_eq!(pubkey.to_string(), valid_address);
    }

    #[test]
    fn test_validate_solana_address_invalid() {
        let invalid_address = "invalid_address";
        let result = validate_solana_address(invalid_address);
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaCommonError::InvalidPubkey(addr) => assert_eq!(addr, invalid_address),
            _ => panic!("Expected InvalidPubkey error"),
        }
    }

    #[test]
    fn test_format_solana_address() {
        let pubkey = Pubkey::from_str("11111111111111111111111111111112").unwrap();
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
            format!("{}", invalid_pubkey_err),
            "Invalid public key: test_key"
        );

        let client_err = SolanaCommonError::ClientError("connection failed".to_string());
        assert_eq!(format!("{}", client_err), "Client error: connection failed");

        let parse_err = SolanaCommonError::ParseError("invalid format".to_string());
        assert_eq!(format!("{}", parse_err), "Parse error: invalid format");
    }

    #[test]
    fn test_solana_common_error_debug() {
        let error = SolanaCommonError::InvalidPubkey("test".to_string());
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("InvalidPubkey"));
        assert!(debug_output.contains("test"));
    }
}
