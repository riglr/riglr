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
pub fn parse_commitment(commitment: &str) -> solana_sdk::commitment_config::CommitmentLevel {
    match commitment.to_lowercase().as_str() {
        "processed" => solana_sdk::commitment_config::CommitmentLevel::Processed,
        "confirmed" => solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        "finalized" => solana_sdk::commitment_config::CommitmentLevel::Finalized,
        _ => solana_sdk::commitment_config::CommitmentLevel::Confirmed, // default
    }
}
