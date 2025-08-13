//! Common utility functions for Solana operations
//!
//! This module provides shared utility functions that are used by both
//! riglr-solana-tools and riglr-cross-chain-tools for common Solana operations.

use crate::types::{SolanaCommonError, SolanaConfig};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;
use std::sync::Arc;

/// Create a Solana RPC client with the given configuration
pub fn create_solana_client(config: &SolanaConfig) -> RpcClient {
    let commitment = crate::types::parse_commitment(&config.commitment);
    let commitment_config = CommitmentConfig { commitment };
    
    RpcClient::new_with_commitment(config.rpc_url.clone(), commitment_config)
}

/// Create a shared Solana RPC client
pub fn create_shared_solana_client(config: &SolanaConfig) -> Arc<RpcClient> {
    Arc::new(create_solana_client(config))
}

/// Convert a string to a Solana Pubkey with better error handling
pub fn string_to_pubkey(s: &str) -> Result<Pubkey, SolanaCommonError> {
    Pubkey::from_str(s).map_err(|_| SolanaCommonError::InvalidPubkey(s.to_string()))
}

/// Get the default Solana configuration from environment or defaults
pub fn default_solana_config() -> SolanaConfig {
    SolanaConfig {
        rpc_url: std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        commitment: std::env::var("SOLANA_COMMITMENT")
            .unwrap_or_else(|_| "confirmed".to_string()),
        timeout_seconds: std::env::var("SOLANA_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30),
    }
}

/// Validate that an RPC URL is reachable
pub async fn validate_rpc_url(url: &str) -> Result<(), SolanaCommonError> {
    let client = RpcClient::new(url.to_string());
    client
        .get_version()
        .await
        .map_err(|e| SolanaCommonError::ClientError(format!("RPC validation failed: {}", e)))?;
    Ok(())
}

/// Convert lamports to SOL for display
pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000.0
}

/// Convert SOL to lamports
pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1_000_000_000.0) as u64
}

/// Format a balance for display with appropriate units
pub fn format_balance(lamports: u64) -> String {
    let sol = lamports_to_sol(lamports);
    if sol >= 1.0 {
        format!("{:.9} SOL", sol)
    } else if lamports >= 1_000_000 {
        format!("{:.6} SOL", sol)
    } else {
        format!("{} lamports", lamports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lamports_conversion() {
        assert_eq!(sol_to_lamports(1.0), 1_000_000_000);
        assert_eq!(lamports_to_sol(1_000_000_000), 1.0);
        assert_eq!(lamports_to_sol(500_000_000), 0.5);
    }

    #[test]
    fn test_format_balance() {
        assert!(format_balance(1_000_000_000).contains("SOL"));
        assert!(format_balance(500).contains("lamports"));
    }

    #[test]
    fn test_default_config() {
        let config = default_solana_config();
        assert!(!config.rpc_url.is_empty());
        assert!(!config.commitment.is_empty());
        assert!(config.timeout_seconds > 0);
    }

    #[test]
    fn test_string_to_pubkey() {
        // Test with well-known Solana address
        let result = string_to_pubkey("So11111111111111111111111111111111111111112");
        assert!(result.is_ok());
        
        // Test with invalid address
        let result = string_to_pubkey("invalid");
        assert!(result.is_err());
    }
}