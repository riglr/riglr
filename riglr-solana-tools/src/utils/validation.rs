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
    fn test_validate_address_when_valid_system_program_should_return_ok() {
        // Happy Path: Valid system program address
        let valid = "11111111111111111111111111111111";
        let result = validate_address(valid);
        assert!(result.is_ok());
        let pubkey = result.unwrap();
        assert_eq!(pubkey.to_string(), valid);
    }

    #[test]
    fn test_validate_address_when_valid_native_mint_should_return_ok() {
        // Happy Path: Valid native mint address
        let valid = "So11111111111111111111111111111111111111112";
        let result = validate_address(valid);
        assert!(result.is_ok());
        let pubkey = result.unwrap();
        assert_eq!(pubkey.to_string(), valid);
    }

    #[test]
    fn test_validate_address_when_valid_typical_pubkey_should_return_ok() {
        // Happy Path: Another valid typical pubkey
        let valid = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
        let result = validate_address(valid);
        assert!(result.is_ok());
        let pubkey = result.unwrap();
        assert_eq!(pubkey.to_string(), valid);
    }

    #[test]
    fn test_validate_address_when_empty_string_should_return_err() {
        // Error Path: Empty string
        let result = validate_address("");
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaToolError::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_validate_address_when_invalid_characters_should_return_err() {
        // Error Path: Invalid characters
        let result = validate_address("invalid");
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaToolError::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_validate_address_when_too_short_should_return_err() {
        // Error Path: Too short
        let result = validate_address("123");
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaToolError::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_validate_address_when_too_long_should_return_err() {
        // Error Path: Too long
        let result = validate_address(
            "111111111111111111111111111111111111111111111111111111111111111111111",
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaToolError::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_validate_address_when_invalid_base58_characters_should_return_err() {
        // Error Path: Invalid base58 characters (contains 0, O, I, l)
        let result = validate_address("0OIl1111111111111111111111111111111");
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaToolError::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_validate_address_when_special_characters_should_return_err() {
        // Error Path: Special characters
        let result = validate_address("11111111111111111111111111111!@#");
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaToolError::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_validate_address_when_whitespace_should_return_err() {
        // Error Path: Contains whitespace
        let result = validate_address("11111111111111111111111111111111 ");
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaToolError::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_validate_address_when_leading_whitespace_should_return_err() {
        // Error Path: Leading whitespace
        let result = validate_address(" 11111111111111111111111111111111");
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaToolError::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_validate_address_when_mixed_case_invalid_should_return_err() {
        // Error Path: Mixed case that results in invalid base58
        let result = validate_address("AbCdEfGhIjKlMnOpQrStUvWxYz123456");
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaToolError::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_validate_address_when_unicode_characters_should_return_err() {
        // Error Path: Unicode characters
        let result = validate_address("1111111111111111111111111111111🚀");
        assert!(result.is_err());
        match result.unwrap_err() {
            SolanaToolError::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }
}
