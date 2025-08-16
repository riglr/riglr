//! Address validation utilities for Solana
//!
//! This module provides functions for validating Solana addresses and other inputs.

use crate::error::{Result, SolanaToolError};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

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
/// Returns the parsed `Pubkey` if valid, or a `SolanaToolError::InvalidAddress` if invalid.
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
pub fn validate_address(address: &str) -> Result<Pubkey> {
    Pubkey::from_str(address)
        .map_err(|e| SolanaToolError::InvalidAddress(format!("Invalid address: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_address() {
        // Valid address (system program)
        let valid = "11111111111111111111111111111111";
        assert!(validate_address(valid).is_ok());

        // Another valid address (native mint)
        let valid2 = "So11111111111111111111111111111111111111112";
        assert!(validate_address(valid2).is_ok());

        // Invalid address
        let invalid = "invalid";
        assert!(validate_address(invalid).is_err());

        // Empty string
        assert!(validate_address("").is_err());
    }
}
