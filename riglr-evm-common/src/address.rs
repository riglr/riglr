//! EVM address utilities shared across riglr crates
//!
//! This module provides common address validation, parsing, and formatting
//! functions to ensure consistent address handling across the workspace.

use crate::error::{EvmCommonError, EvmResult};
use alloy::primitives::Address;
use std::str::FromStr;

/// Validate an EVM address string format
///
/// Accepts both checksummed and non-checksummed addresses.
/// Returns error for invalid formats or lengths.
///
/// # Arguments
/// * `address` - Address string to validate (with or without 0x prefix)
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_tools::common::address::validate_evm_address;
///
/// // Valid addresses
/// assert!(validate_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());
/// assert!(validate_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());
///
/// // Invalid addresses
/// assert!(validate_evm_address("invalid").is_err());
/// assert!(validate_evm_address("0x123").is_err());
/// ```
pub fn validate_evm_address(address: &str) -> EvmResult<()> {
    parse_evm_address(address).map(|_| ())
}

/// Parse an EVM address string into an Alloy Address type
///
/// Handles both checksummed and non-checksummed addresses.
/// Automatically adds 0x prefix if missing.
///
/// # Arguments
/// * `address` - Address string to parse
///
/// # Returns
/// * `Address` - Parsed Alloy address type
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_tools::common::address::parse_evm_address;
///
/// let addr = parse_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")?;
/// let addr2 = parse_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")?;
/// assert_eq!(addr, addr2);
/// ```
pub fn parse_evm_address(address: &str) -> EvmResult<Address> {
    let clean_address = if address.starts_with("0x") {
        address.to_string()
    } else if address.len() == 40 {
        format!("0x{}", address)
    } else {
        return Err(EvmCommonError::InvalidAddress(format!(
            "Invalid address length: {}. Expected 40 hex characters (optionally prefixed with 0x)",
            address.len()
        )));
    };

    Address::from_str(&clean_address).map_err(|e| {
        EvmCommonError::InvalidAddress(format!("Failed to parse address '{}': {}", address, e))
    })
}

/// Format an address for display with EIP-55 checksumming
///
/// Converts any valid address to its checksummed representation
/// for safe display and copy-paste operations.
///
/// # Arguments
/// * `address` - Alloy Address to format
///
/// # Returns
/// * Checksummed address string with 0x prefix
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_tools::common::address::{parse_evm_address, format_address};
///
/// let addr = parse_evm_address("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")?;
/// let formatted = format_address(&addr);
/// assert_eq!(formatted, "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
/// ```
pub fn format_address(address: &Address) -> String {
    // Use Display implementation which provides EIP-55 checksummed format
    format!("{}", address)
}

/// Format an address string with EIP-55 checksumming
///
/// Convenience function that parses and formats in one step.
///
/// # Arguments
/// * `address` - Address string to format
///
/// # Returns
/// * Checksummed address string
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_tools::common::address::format_address_string;
///
/// let formatted = format_address_string("742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")?;
/// assert_eq!(formatted, "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
/// ```
pub fn format_address_string(address: &str) -> EvmResult<String> {
    let addr = parse_evm_address(address)?;
    Ok(format_address(&addr))
}

/// Check if an address string is already checksummed according to EIP-55
///
/// # Arguments
/// * `address` - Address string to check
///
/// # Returns
/// * `true` if address is properly checksummed, `false` if not checksummed or invalid
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_tools::common::address::is_checksummed;
///
/// assert!(is_checksummed("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"));
/// assert!(!is_checksummed("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e"));
/// assert!(!is_checksummed("invalid"));
/// ```
pub fn is_checksummed(address: &str) -> bool {
    match parse_evm_address(address) {
        Ok(addr) => {
            let checksummed = format_address(&addr);
            let normalized = if address.starts_with("0x") {
                address.to_string()
            } else {
                format!("0x{}", address)
            };
            checksummed == normalized
        }
        Err(_) => false,
    }
}

/// Extract the address portion without 0x prefix
///
/// # Arguments
/// * `address` - Address string (with or without 0x prefix)
///
/// # Returns
/// * Address hex string without 0x prefix
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_tools::common::address::strip_0x_prefix;
///
/// assert_eq!(strip_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
/// assert_eq!(strip_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
/// ```
pub fn strip_0x_prefix(address: &str) -> &str {
    address.strip_prefix("0x").unwrap_or(address)
}

/// Ensure address has 0x prefix
///
/// # Arguments
/// * `address` - Address string (with or without 0x prefix)
///
/// # Returns
/// * Address string guaranteed to have 0x prefix
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_tools::common::address::ensure_0x_prefix;
///
/// assert_eq!(ensure_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
/// assert_eq!(ensure_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"), "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
/// ```
pub fn ensure_0x_prefix(address: &str) -> String {
    if address.starts_with("0x") {
        address.to_string()
    } else {
        format!("0x{}", address)
    }
}

/// Common Ethereum addresses for reference
pub mod known_addresses {
    /// Zero address (0x0)
    pub const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

    /// Burn address (0xdead)
    pub const BURN_ADDRESS: &str = "0x000000000000000000000000000000000000dEaD";

    /// WETH address on Ethereum mainnet
    pub const WETH_ETHEREUM: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

    /// USDC address on Ethereum mainnet
    pub const USDC_ETHEREUM: &str = "0xA0b86a33E6417c5d6d6bE6C2e0C6C3e5d6c7D8E9";

    /// USDT address on Ethereum mainnet
    pub const USDT_ETHEREUM: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
}

#[cfg(test)]
mod tests {
    use super::*;

    // === validate_evm_address tests ===
    #[test]
    fn test_validate_evm_address_when_valid_with_prefix_should_return_ok() {
        assert!(validate_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());
        assert!(validate_evm_address("0x0000000000000000000000000000000000000000").is_ok());
        assert!(validate_evm_address("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").is_ok());
    }

    #[test]
    fn test_validate_evm_address_when_valid_without_prefix_should_return_ok() {
        assert!(validate_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());
        assert!(validate_evm_address("0000000000000000000000000000000000000000").is_ok());
        assert!(validate_evm_address("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").is_ok());
    }

    #[test]
    fn test_validate_evm_address_when_invalid_format_should_return_err() {
        assert!(validate_evm_address("invalid").is_err());
        assert!(validate_evm_address("0x123").is_err());
        assert!(validate_evm_address("123").is_err());
        assert!(validate_evm_address("").is_err());
        assert!(validate_evm_address("0xGGGG").is_err());
        assert!(validate_evm_address("GGGG").is_err());
    }

    #[test]
    fn test_validate_evm_address_when_wrong_length_should_return_err() {
        assert!(validate_evm_address("0x123456789").is_err());
        assert!(validate_evm_address("123456789").is_err());
        assert!(validate_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e1234").is_err());
        assert!(validate_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e1234").is_err());
    }

    // === parse_evm_address tests ===
    #[test]
    fn test_parse_evm_address_when_valid_with_prefix_should_return_address() {
        let addr1 = parse_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").unwrap();
        let addr2 = parse_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").unwrap();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_parse_evm_address_when_zero_address_should_return_zero() {
        let zero = parse_evm_address("0x0000000000000000000000000000000000000000").unwrap();
        assert_eq!(zero, Address::ZERO);
    }

    #[test]
    fn test_parse_evm_address_when_valid_without_prefix_40_chars_should_add_prefix() {
        let addr = parse_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").unwrap();
        let addr_with_prefix =
            parse_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").unwrap();
        assert_eq!(addr, addr_with_prefix);
    }

    #[test]
    fn test_parse_evm_address_when_invalid_length_without_prefix_should_return_err() {
        let result = parse_evm_address("123456789");
        assert!(result.is_err());
        match result {
            Err(EvmCommonError::InvalidAddress(msg)) => {
                assert!(msg.contains("Invalid address length: 9"));
                assert!(msg.contains("Expected 40 hex characters"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_parse_evm_address_when_invalid_length_with_prefix_should_return_err() {
        let result = parse_evm_address("0x123");
        assert!(result.is_err());
        match result {
            Err(EvmCommonError::InvalidAddress(msg)) => {
                assert!(msg.contains("Failed to parse address '0x123'"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_parse_evm_address_when_invalid_hex_should_return_err() {
        let result = parse_evm_address("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG");
        assert!(result.is_err());
        match result {
            Err(EvmCommonError::InvalidAddress(msg)) => {
                assert!(msg.contains("Failed to parse address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_parse_evm_address_when_empty_string_should_return_err() {
        let result = parse_evm_address("");
        assert!(result.is_err());
        match result {
            Err(EvmCommonError::InvalidAddress(msg)) => {
                assert!(msg.contains("Invalid address length: 0"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_parse_evm_address_when_lowercase_should_work() {
        let addr = parse_evm_address("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e").unwrap();
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn test_parse_evm_address_when_uppercase_should_work() {
        let addr = parse_evm_address("0x742D35CC67A5B747BE4C506C5E8B0A146D7B2E9E").unwrap();
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn test_parse_evm_address_when_mixed_case_should_work() {
        let addr = parse_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").unwrap();
        assert_ne!(addr, Address::ZERO);
    }

    // === format_address tests ===
    #[test]
    fn test_format_address_when_valid_address_should_return_checksummed() {
        let addr = parse_evm_address("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e").unwrap();
        let formatted = format_address(&addr);
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
        // The result should be checksummed (some mix of upper and lower case)
        assert_ne!(formatted, formatted.to_lowercase());
    }

    #[test]
    fn test_format_address_when_zero_address_should_return_zeros() {
        let formatted = format_address(&Address::ZERO);
        assert_eq!(formatted, "0x0000000000000000000000000000000000000000");
    }

    #[test]
    fn test_format_address_when_max_address_should_format_correctly() {
        let max_addr = Address::from_str("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
        let formatted = format_address(&max_addr);
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
        assert!(formatted.contains("F") || formatted.contains("f"));
    }

    // === format_address_string tests ===
    #[test]
    fn test_format_address_string_when_valid_with_prefix_should_return_checksummed() {
        let result = format_address_string("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e");
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
    }

    #[test]
    fn test_format_address_string_when_valid_without_prefix_should_return_checksummed() {
        let result = format_address_string("742d35cc67a5b747be4c506c5e8b0a146d7b2e9e");
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
    }

    #[test]
    fn test_format_address_string_when_invalid_should_return_err() {
        let result = format_address_string("invalid");
        assert!(result.is_err());
        match result {
            Err(EvmCommonError::InvalidAddress(_)) => {}
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_format_address_string_when_wrong_length_should_return_err() {
        let result = format_address_string("0x123");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_address_string_when_empty_should_return_err() {
        let result = format_address_string("");
        assert!(result.is_err());
    }

    // === is_checksummed tests ===
    #[test]
    fn test_is_checksummed_when_lowercase_should_return_false() {
        let lowercased = "0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e";
        assert!(!is_checksummed(lowercased));
    }

    #[test]
    fn test_is_checksummed_when_uppercase_should_return_false() {
        let uppercased = "0x742D35CC67A5B747BE4C506C5E8B0A146D7B2E9E";
        assert!(!is_checksummed(uppercased));
    }

    #[test]
    fn test_is_checksummed_when_properly_checksummed_should_return_true() {
        // First get a properly checksummed address
        let addr = parse_evm_address("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e").unwrap();
        let checksummed = format_address(&addr);
        assert!(is_checksummed(&checksummed));
    }

    #[test]
    fn test_is_checksummed_when_zero_address_should_return_true() {
        assert!(is_checksummed("0x0000000000000000000000000000000000000000"));
    }

    #[test]
    fn test_is_checksummed_when_invalid_address_should_return_false() {
        assert!(!is_checksummed("invalid"));
        assert!(!is_checksummed("0x123"));
        assert!(!is_checksummed(""));
        assert!(!is_checksummed(
            "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG"
        ));
    }

    #[test]
    fn test_is_checksummed_when_without_prefix_but_valid_should_check_with_prefix() {
        // Test that the function properly handles addresses without 0x prefix
        let without_prefix = "742d35cc67a5b747be4c506c5e8b0a146d7b2e9e";
        // Should return false since it's all lowercase
        assert!(!is_checksummed(without_prefix));
    }

    #[test]
    fn test_is_checksummed_when_without_prefix_checksummed_should_return_true() {
        // Get a checksummed address and remove the prefix
        let addr = parse_evm_address("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e").unwrap();
        let checksummed = format_address(&addr);
        let without_prefix = strip_0x_prefix(&checksummed);
        assert!(is_checksummed(without_prefix));
    }

    // === strip_0x_prefix tests ===
    #[test]
    fn test_strip_0x_prefix_when_has_prefix_should_remove_it() {
        assert_eq!(
            strip_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
    }

    #[test]
    fn test_strip_0x_prefix_when_no_prefix_should_return_unchanged() {
        assert_eq!(
            strip_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
    }

    #[test]
    fn test_strip_0x_prefix_when_empty_string_should_return_empty() {
        assert_eq!(strip_0x_prefix(""), "");
    }

    #[test]
    fn test_strip_0x_prefix_when_only_0x_should_return_empty() {
        assert_eq!(strip_0x_prefix("0x"), "");
    }

    #[test]
    fn test_strip_0x_prefix_when_starts_with_0_but_not_0x_should_return_unchanged() {
        assert_eq!(strip_0x_prefix("0abc123"), "0abc123");
    }

    #[test]
    fn test_strip_0x_prefix_when_multiple_0x_should_remove_only_first() {
        assert_eq!(strip_0x_prefix("0x0x123"), "0x123");
    }

    // === ensure_0x_prefix tests ===
    #[test]
    fn test_ensure_0x_prefix_when_no_prefix_should_add_it() {
        assert_eq!(
            ensure_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
    }

    #[test]
    fn test_ensure_0x_prefix_when_has_prefix_should_return_unchanged() {
        assert_eq!(
            ensure_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
    }

    #[test]
    fn test_ensure_0x_prefix_when_empty_string_should_add_prefix() {
        assert_eq!(ensure_0x_prefix(""), "0x");
    }

    #[test]
    fn test_ensure_0x_prefix_when_only_0_should_add_x() {
        assert_eq!(ensure_0x_prefix("0"), "0x0");
    }

    #[test]
    fn test_ensure_0x_prefix_when_starts_with_0_but_not_0x_should_add_prefix() {
        assert_eq!(ensure_0x_prefix("0abc123"), "0x0abc123");
    }

    // === known_addresses tests ===
    #[test]
    fn test_known_addresses_when_all_constants_should_be_valid() {
        use known_addresses::*;

        assert!(validate_evm_address(ZERO_ADDRESS).is_ok());
        assert!(validate_evm_address(BURN_ADDRESS).is_ok());
        assert!(validate_evm_address(WETH_ETHEREUM).is_ok());
        assert!(validate_evm_address(USDC_ETHEREUM).is_ok());
        assert!(validate_evm_address(USDT_ETHEREUM).is_ok());
    }

    #[test]
    fn test_known_addresses_when_parsed_should_be_correct() {
        use known_addresses::*;

        let zero = parse_evm_address(ZERO_ADDRESS).unwrap();
        assert_eq!(zero, Address::ZERO);

        let burn = parse_evm_address(BURN_ADDRESS).unwrap();
        assert_ne!(burn, Address::ZERO);

        let weth = parse_evm_address(WETH_ETHEREUM).unwrap();
        assert_ne!(weth, Address::ZERO);

        let usdc = parse_evm_address(USDC_ETHEREUM).unwrap();
        assert_ne!(usdc, Address::ZERO);

        let usdt = parse_evm_address(USDT_ETHEREUM).unwrap();
        assert_ne!(usdt, Address::ZERO);
    }

    #[test]
    fn test_known_addresses_when_formatted_should_maintain_format() {
        use known_addresses::*;

        let addresses = [
            ZERO_ADDRESS,
            BURN_ADDRESS,
            WETH_ETHEREUM,
            USDC_ETHEREUM,
            USDT_ETHEREUM,
        ];

        for addr_str in addresses {
            let addr = parse_evm_address(addr_str).unwrap();
            let formatted = format_address(&addr);
            assert!(formatted.starts_with("0x"));
            assert_eq!(formatted.len(), 42);
        }
    }

    // === Additional edge case tests ===
    #[test]
    fn test_edge_cases_when_max_values_should_work() {
        // Test with maximum address value
        let max_addr_str = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";
        assert!(validate_evm_address(max_addr_str).is_ok());
        let addr = parse_evm_address(max_addr_str).unwrap();
        let formatted = format_address(&addr);
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
    }

    #[test]
    fn test_edge_cases_when_all_numeric_should_work() {
        let numeric_addr = "1234567890123456789012345678901234567890";
        assert!(validate_evm_address(numeric_addr).is_ok());
        let addr = parse_evm_address(numeric_addr).unwrap();
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn test_edge_cases_when_mixed_case_boundaries_should_work() {
        // Test with mixed case at boundaries
        let mixed_case = "0xaBcDeF1234567890aBcDeF1234567890aBcDeF12";
        assert!(validate_evm_address(mixed_case).is_ok());
        let addr = parse_evm_address(mixed_case).unwrap();
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn test_integration_when_round_trip_should_preserve_value() {
        // Test round-trip: string -> Address -> checksummed string -> Address
        let original = "742d35cc67a5b747be4c506c5e8b0a146d7b2e9e";
        let addr1 = parse_evm_address(original).unwrap();
        let checksummed = format_address(&addr1);
        let addr2 = parse_evm_address(&checksummed).unwrap();
        assert_eq!(addr1, addr2);
    }
}
