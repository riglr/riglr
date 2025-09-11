//! Common utility functions for Solana operations
//!
//! This module provides shared utility functions that are used by both
//! riglr-solana-tools and riglr-cross-chain-tools for common Solana operations.

use super::types::{SolanaCommonError, SolanaConfig};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;

const SOLANA_RPC_URL: &str = "SOLANA_RPC_URL";
const SOLANA_COMMITMENT: &str = "SOLANA_COMMITMENT";
const SOLANA_TIMEOUT: &str = "SOLANA_TIMEOUT";

/// Create a Solana RPC client with the given configuration
pub fn create_solana_client(config: &SolanaConfig) -> RpcClient {
    let commitment = super::types::parse_commitment(&config.commitment);
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
        rpc_url: std::env::var(SOLANA_RPC_URL)
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        commitment: std::env::var(SOLANA_COMMITMENT).unwrap_or_else(|_| "confirmed".to_string()),
        timeout_seconds: std::env::var(SOLANA_TIMEOUT)
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
#[allow(unsafe_code)] // Test functions use unsafe std::env operations for Rust 2024 compatibility with proper safety documentation
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_create_solana_client_when_valid_config_should_return_client() {
        let config = SolanaConfig {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            commitment: "confirmed".to_string(),
            timeout_seconds: 30,
        };
        let _client = create_solana_client(&config);
        // We can't easily test the internal state of RpcClient, but we can verify it was created
        // This test mainly ensures the function doesn't panic and executes the creation path
    }

    #[test]
    fn test_create_solana_client_when_different_commitment_should_create_client() {
        let config = SolanaConfig {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            commitment: "finalized".to_string(),
            timeout_seconds: 60,
        };
        let _client = create_solana_client(&config);
        // Client creation succeeded if we reach here
    }

    #[test]
    fn test_create_shared_solana_client_when_valid_config_should_return_arc_client() {
        let config = SolanaConfig {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            commitment: "confirmed".to_string(),
            timeout_seconds: 30,
        };
        let client = create_shared_solana_client(&config);
        // Verify it returns an Arc by checking reference count behavior
        let _client_clone = Arc::clone(&client);
        assert_eq!(Arc::strong_count(&client), 2);
    }

    #[test]
    fn test_string_to_pubkey_when_valid_address_should_return_ok() {
        // Test with well-known Solana native mint address
        let result = string_to_pubkey("So11111111111111111111111111111111111111112");
        assert!(result.is_ok());

        // Test with system program address
        let result = string_to_pubkey("11111111111111111111111111111111");
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_to_pubkey_when_invalid_address_should_return_err() {
        // Test with completely invalid string
        let result = string_to_pubkey("invalid");
        assert!(result.is_err());
        if let Err(SolanaCommonError::InvalidPubkey(addr)) = result {
            assert_eq!(addr, "invalid");
        } else {
            panic!("Expected InvalidPubkey error");
        }
    }

    #[test]
    fn test_string_to_pubkey_when_empty_string_should_return_err() {
        let result = string_to_pubkey("");
        assert!(result.is_err());
        if let Err(SolanaCommonError::InvalidPubkey(addr)) = result {
            assert_eq!(addr, "");
        } else {
            panic!("Expected InvalidPubkey error");
        }
    }

    #[test]
    fn test_string_to_pubkey_when_wrong_length_should_return_err() {
        // Too short
        let result = string_to_pubkey("123");
        assert!(result.is_err());

        // Too long
        let result = string_to_pubkey("So11111111111111111111111111111111111111112000");
        assert!(result.is_err());
    }

    #[test]
    fn test_string_to_pubkey_when_invalid_characters_should_return_err() {
        // Contains invalid base58 characters
        let result = string_to_pubkey("0OIl111111111111111111111111111111111111112");
        assert!(result.is_err());
    }

    #[test]
    fn test_default_solana_config_when_no_env_vars_should_use_defaults() {
        // Clear environment variables to test defaults
        // SAFETY: Only modifying test environment variables in isolated test
        unsafe {
            env::remove_var(SOLANA_RPC_URL);
            env::remove_var(SOLANA_COMMITMENT);
            env::remove_var(SOLANA_TIMEOUT);
        }

        let config = default_solana_config();
        assert_eq!(config.rpc_url, "https://api.mainnet-beta.solana.com");
        assert_eq!(config.commitment, "confirmed");
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_default_solana_config_when_env_vars_set_should_use_env_values() {
        // Set environment variables
        // SAFETY: Only modifying test environment variables in isolated test
        unsafe {
            env::set_var(SOLANA_RPC_URL, "https://custom-rpc.com");
            env::set_var(SOLANA_COMMITMENT, "finalized");
            env::set_var(SOLANA_TIMEOUT, "60");
        }

        let config = default_solana_config();
        assert_eq!(config.rpc_url, "https://custom-rpc.com");
        assert_eq!(config.commitment, "finalized");
        assert_eq!(config.timeout_seconds, 60);

        // Clean up
        // SAFETY: Only modifying test environment variables in isolated test
        unsafe {
            env::remove_var(SOLANA_RPC_URL);
            env::remove_var(SOLANA_COMMITMENT);
            env::remove_var(SOLANA_TIMEOUT);
        }
    }

    #[test]
    fn test_default_solana_config_when_invalid_timeout_should_use_default() {
        // SAFETY: Only modifying test environment variables in isolated test
        unsafe {
            env::set_var(SOLANA_TIMEOUT, "invalid_number");
        }

        let config = default_solana_config();
        assert_eq!(config.timeout_seconds, 30); // Should fall back to default

        // SAFETY: Only modifying test environment variables in isolated test
        unsafe {
            env::remove_var(SOLANA_TIMEOUT);
        }
    }

    #[test]
    fn test_lamports_to_sol_when_zero_should_return_zero() {
        assert_eq!(lamports_to_sol(0), 0.0);
    }

    #[test]
    fn test_lamports_to_sol_when_one_billion_should_return_one() {
        assert_eq!(lamports_to_sol(1_000_000_000), 1.0);
    }

    #[test]
    fn test_lamports_to_sol_when_half_billion_should_return_half() {
        assert_eq!(lamports_to_sol(500_000_000), 0.5);
    }

    #[test]
    fn test_lamports_to_sol_when_max_value_should_not_panic() {
        let result = lamports_to_sol(u64::MAX);
        assert!(result > 0.0);
    }

    #[test]
    fn test_sol_to_lamports_when_zero_should_return_zero() {
        assert_eq!(sol_to_lamports(0.0), 0);
    }

    #[test]
    fn test_sol_to_lamports_when_one_should_return_billion() {
        assert_eq!(sol_to_lamports(1.0), 1_000_000_000);
    }

    #[test]
    fn test_sol_to_lamports_when_half_should_return_half_billion() {
        assert_eq!(sol_to_lamports(0.5), 500_000_000);
    }

    #[test]
    fn test_sol_to_lamports_when_fractional_should_round_down() {
        assert_eq!(sol_to_lamports(0.0000000001), 0); // Less than 1 lamport
        assert_eq!(sol_to_lamports(0.000000001), 1); // Exactly 1 lamport
    }

    #[test]
    fn test_format_balance_when_one_sol_or_more_should_show_sol_with_nine_decimals() {
        assert_eq!(format_balance(1_000_000_000), "1.000000000 SOL");
        assert_eq!(format_balance(2_500_000_000), "2.500000000 SOL");
        assert_eq!(format_balance(10_000_000_000), "10.000000000 SOL");
    }

    #[test]
    fn test_format_balance_when_between_million_and_billion_lamports_should_show_sol_with_six_decimals(
    ) {
        assert_eq!(format_balance(500_000_000), "0.500000 SOL");
        assert_eq!(format_balance(1_000_000), "0.001000 SOL");
        // 999_999_999 lamports = 0.999999999 SOL, but with 6 decimals it rounds to 1.000000
        assert_eq!(format_balance(999_999_999), "1.000000 SOL");
    }

    #[test]
    fn test_format_balance_when_less_than_million_lamports_should_show_lamports() {
        assert_eq!(format_balance(999_999), "999999 lamports");
        assert_eq!(format_balance(500), "500 lamports");
        assert_eq!(format_balance(1), "1 lamports");
        assert_eq!(format_balance(0), "0 lamports");
    }

    #[test]
    fn test_format_balance_edge_cases() {
        // Test boundary at exactly 1 million lamports
        assert_eq!(format_balance(1_000_000), "0.001000 SOL");

        // Test boundary just under 1 million lamports
        assert_eq!(format_balance(999_999), "999999 lamports");

        // Test boundary at exactly 1 SOL
        assert_eq!(format_balance(1_000_000_000), "1.000000000 SOL");

        // Test boundary just under 1 SOL - 999_999_999 rounds to 1.000000 with 6 decimals
        assert_eq!(format_balance(999_999_999), "1.000000 SOL");
    }

    // Additional integration-style tests
    #[test]
    fn test_lamports_conversion_roundtrip() {
        let original_sol = 1.5;
        let lamports = sol_to_lamports(original_sol);
        let converted_back = lamports_to_sol(lamports);
        assert_eq!(converted_back, original_sol);
    }

    #[test]
    fn test_default_config_consistency() {
        let config = default_solana_config();
        assert!(!config.rpc_url.is_empty());
        assert!(!config.commitment.is_empty());
        assert!(config.timeout_seconds > 0);

        // Verify the config can be used to create a client
        let _client = create_solana_client(&config);
        let _shared_client = create_shared_solana_client(&config);
    }
}
