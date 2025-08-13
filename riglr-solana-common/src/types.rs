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
    InvalidPubkey(String),
    
    #[error("Client error: {0}")]
    ClientError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Shared configuration for Solana operations
#[derive(Debug, Clone)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub commitment: String,
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
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl SolanaAccount {
    pub fn new(pubkey: &str, is_signer: bool, is_writable: bool) -> Result<Self, SolanaCommonError> {
        // Validate pubkey format
        Pubkey::from_str(pubkey)
            .map_err(|_| SolanaCommonError::InvalidPubkey(pubkey.to_string()))?;
        
        Ok(Self {
            pubkey: pubkey.to_string(),
            is_signer,
            is_writable,
        })
    }
    
    pub fn to_pubkey(&self) -> Result<Pubkey, SolanaCommonError> {
        Pubkey::from_str(&self.pubkey)
            .map_err(|_| SolanaCommonError::InvalidPubkey(self.pubkey.clone()))
    }
}

/// Transaction metadata shared between crates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaTransactionData {
    pub to: String,
    pub data: String,
    pub value: String,
    pub gas_limit: String,
    pub gas_price: String,
    pub solana_accounts: Option<Vec<SolanaAccount>>,
}

/// Helper function to validate Solana addresses
pub fn validate_solana_address(address: &str) -> Result<Pubkey, SolanaCommonError> {
    Pubkey::from_str(address)
        .map_err(|_| SolanaCommonError::InvalidPubkey(address.to_string()))
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