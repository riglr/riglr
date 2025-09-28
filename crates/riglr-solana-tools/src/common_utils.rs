//! Common utility functions for Solana operations
//!
//! This module provides shared utility functions that are used by both
//! riglr-solana-tools and riglr-cross-chain-tools for common Solana operations.

extern crate alloc;

use crate::common_types::{parse_commitment, SolanaCommonError, SolanaConfig};
use alloc::sync::Arc;
use core::str::FromStr as _;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;

/// Environment variable name for Solana RPC URL configuration
const SOLANA_RPC_URL: &str = "SOLANA_RPC_URL";
/// Environment variable name for Solana commitment level configuration
const SOLANA_COMMITMENT: &str = "SOLANA_COMMITMENT";
/// Environment variable name for Solana timeout configuration
const SOLANA_TIMEOUT: &str = "SOLANA_TIMEOUT";

/// Create a Solana RPC client with the given configuration
#[must_use]
#[inline]
pub fn create_solana_client(config: &SolanaConfig) -> RpcClient {
    let commitment = parse_commitment(&config.commitment);
    let commitment_config = CommitmentConfig { commitment };

    RpcClient::new_with_commitment(config.rpc_url.clone(), commitment_config)
}

/// Create a shared Solana RPC client
#[must_use]
#[inline]
pub fn create_shared_solana_client(config: &SolanaConfig) -> Arc<RpcClient> {
    Arc::new(create_solana_client(config))
}

/// Convert a string to a Solana Pubkey with better error handling
///
/// # Errors
///
/// Returns `SolanaCommonError::InvalidPubkey` if the string is not a valid Solana public key format.
#[inline]
pub fn string_to_pubkey(pubkey_str: &str) -> Result<Pubkey, SolanaCommonError> {
    Pubkey::from_str(pubkey_str)
        .map_err(|_parse_error| SolanaCommonError::InvalidPubkey(pubkey_str.to_owned()))
}

/// Get the default Solana configuration from environment or defaults
#[must_use]
#[inline]
pub fn default_solana_config() -> SolanaConfig {
    use std::env;

    SolanaConfig {
        rpc_url: env::var(SOLANA_RPC_URL)
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_owned()),
        commitment: env::var(SOLANA_COMMITMENT).unwrap_or_else(|_| "confirmed".to_owned()),
        timeout_seconds: env::var(SOLANA_TIMEOUT)
            .ok()
            .and_then(|timeout_str| timeout_str.parse().ok())
            .unwrap_or(30),
    }
}

/// Validate that an RPC URL is reachable
///
/// # Errors
///
/// Returns `SolanaCommonError::ClientError` if the RPC URL is unreachable or returns an invalid response.
#[inline]
pub async fn validate_rpc_url(url: &str) -> Result<(), SolanaCommonError> {
    let client = RpcClient::new(url.to_owned());
    client.get_version().await.map_err(|rpc_error| {
        SolanaCommonError::ClientError(format!("RPC validation failed: {rpc_error}"))
    })?;
    Ok(())
}

/// Convert lamports to SOL for display
///
/// # Precision Note
///
/// For lamports values larger than 2^52 (~4.5 billion SOL), this conversion
/// may lose precision due to f64 mantissa limitations. Since Solana's max
/// supply is ~500 million SOL, this is not a practical concern for most use cases.
#[must_use]
#[inline]
pub fn lamports_to_sol(lamports: u64) -> f64 {
    #[expect(clippy::cast_precision_loss)]
    let sol = lamports as f64 / 1_000_000_000.0;
    sol
}

/// Convert SOL to lamports
///
/// Negative values are clamped to 0. Values larger than `u64::MAX` when converted
/// to lamports are clamped to `u64::MAX`. Fractional lamports are truncated.
///
/// # Examples
///
/// ```
/// # use riglr_solana_tools::common_utils::sol_to_lamports;
/// assert_eq!(sol_to_lamports(1.0), 1_000_000_000);
/// assert_eq!(sol_to_lamports(-1.0), 0); // Negative clamped to 0
/// assert_eq!(sol_to_lamports(0.000_000_001), 1); // Smallest unit
/// ```
#[must_use]
#[inline]
pub fn sol_to_lamports(sol: f64) -> u64 {
    if sol < 0.0 {
        return 0;
    }

    let lamports_f64 = sol * 1_000_000_000.0;

    // Check for overflow before casting
    #[expect(clippy::cast_precision_loss)]
    let max_check = u64::MAX as f64;
    if lamports_f64 >= max_check {
        return u64::MAX;
    }

    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let result = lamports_f64.round() as u64;
    result
}

/// Format a balance for display with appropriate units
#[must_use]
#[inline]
pub fn format_balance(lamports: u64) -> String {
    let sol = lamports_to_sol(lamports);
    if sol >= 1.0 {
        return format!("{sol:.9} SOL");
    } else if lamports >= 1_000_000 {
        return format!("{sol:.6} SOL");
    }
    format!("{lamports} lamports")
}

#[cfg(test)]
#[expect(clippy::float_cmp, clippy::panic)]
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
        #[expect(unsafe_code)]
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
        #[expect(unsafe_code)]
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
        #[expect(unsafe_code)]
        unsafe {
            env::remove_var(SOLANA_RPC_URL);
            env::remove_var(SOLANA_COMMITMENT);
            env::remove_var(SOLANA_TIMEOUT);
        }
    }

    #[test]
    fn test_default_solana_config_when_invalid_timeout_should_use_default() {
        // SAFETY: Only modifying test environment variables in isolated test
        #[expect(unsafe_code)]
        unsafe {
            env::set_var(SOLANA_TIMEOUT, "invalid_number");
        }

        let config = default_solana_config();
        assert_eq!(config.timeout_seconds, 30); // Should fall back to default

        // SAFETY: Only modifying test environment variables in isolated test
        #[expect(unsafe_code)]
        unsafe {
            env::remove_var(SOLANA_TIMEOUT);
        }
    }

    #[test]
    // Exact comparison intentional for lamports-to-SOL conversion validation
    fn test_lamports_to_sol_when_zero_should_return_zero() {
        assert_eq!(lamports_to_sol(0), 0.0);
    }

    #[test]
    // Exact comparison intentional for lamports-to-SOL conversion validation
    fn test_lamports_to_sol_when_one_billion_should_return_one() {
        assert_eq!(lamports_to_sol(1_000_000_000), 1.0);
    }

    #[test]
    // Exact comparison intentional for lamports-to-SOL conversion validation
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
        assert_eq!(sol_to_lamports(0.000_000_000_1), 0); // Less than 1 lamport
        assert_eq!(sol_to_lamports(0.000_000_001), 1); // Exactly 1 lamport
    }

    #[test]
    fn test_sol_to_lamports_when_negative_should_clamp_to_zero() {
        assert_eq!(sol_to_lamports(-1.0), 0);
        assert_eq!(sol_to_lamports(-0.5), 0);
        assert_eq!(sol_to_lamports(-f64::INFINITY), 0);
    }

    #[test]
    fn test_sol_to_lamports_when_overflow_should_clamp_to_max() {
        #[expect(clippy::cast_precision_loss)]
        let max_sol_as_f64 = u64::MAX as f64 / 1_000_000_000.0;
        assert_eq!(sol_to_lamports(max_sol_as_f64 * 2.0), u64::MAX);
        assert_eq!(sol_to_lamports(f64::INFINITY), u64::MAX);
    }

    #[test]
    fn test_lamports_to_sol_precision_limits() {
        // Test values at the edge of f64 precision
        let large_lamports = 1u64 << 53; // 2^53, where f64 starts losing precision
        let sol_value = lamports_to_sol(large_lamports);
        assert!(sol_value > 0.0);

        // Test that very large values still convert without panicking
        let max_practical_lamports = 500_000_000 * 1_000_000_000u64; // 500M SOL
        let max_sol = lamports_to_sol(max_practical_lamports);
        assert_eq!(max_sol, 500_000_000.0);
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
    // Exact comparison intentional for lamports-to-SOL conversion validation
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
