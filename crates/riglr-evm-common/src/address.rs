//! EVM address utilities shared across riglr crates
//!
//! This module provides common address validation, parsing, and formatting
//! functions to ensure consistent address handling across the workspace.

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

use crate::error::{Error, EvmResult};
use alloy::primitives::Address;
use core::str::FromStr as _;

/// Validate an EVM address string format
///
/// Accepts both checksummed and non-checksummed addresses.
/// Returns error for invalid formats or lengths.
///
/// # Arguments
/// * `address` - Address string to validate (with or without 0x prefix)
///
/// # Errors
/// Returns `Error::InvalidAddress` for invalid address formats or lengths.
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_tools::common::address::validate;
///
/// // Valid addresses
/// assert!(validate("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());
/// assert!(validate("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());
///
/// // Invalid addresses
/// assert!(validate("invalid").is_err());
/// assert!(validate("0x123").is_err());
/// ```
#[inline]
pub fn validate(address: &str) -> EvmResult<()> {
    parse(address).map(|_| ())
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
/// # Errors
/// Returns `Error::InvalidAddress` for invalid address formats or lengths.
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_tools::common::address::parse;
///
/// let addr = parse("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")?;
/// let addr2 = parse("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")?;
/// assert_eq!(addr, addr2);
/// ```
#[inline]
pub fn parse(address: &str) -> EvmResult<Address> {
    let clean_address = if address.starts_with("0x") {
        address.to_owned()
    } else if address.len() == 40 {
        format!("0x{address}")
    } else {
        return Err(Error::InvalidAddress(format!(
            "Invalid address length: {}. Expected 40 hex characters (optionally prefixed with 0x)",
            address.len()
        )));
    };

    Address::from_str(&clean_address).map_err(|error| {
        Error::InvalidAddress(format!("Failed to parse address '{address}': {error}"))
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
/// use riglr_evm_tools::common::address::{parse, format};
///
/// let addr = parse("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")?;
/// let formatted = format(&addr);
/// assert_eq!(formatted, "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
/// ```
#[must_use]
#[inline]
pub fn format(address: &Address) -> String {
    // Use Display implementation which provides EIP-55 checksummed format
    format!("{address}")
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
/// # Errors
/// Returns `Error::InvalidAddress` if the input address cannot be parsed.
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_tools::common::address::format_string;
///
/// let formatted = format_string("742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")?;
/// assert_eq!(formatted, "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
/// ```
#[inline]
pub fn format_string(address: &str) -> EvmResult<String> {
    let addr = parse(address)?;
    Ok(format(&addr))
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
#[must_use]
#[inline]
pub fn is_checksummed(address: &str) -> bool {
    parse(address).is_ok_and(|addr| {
        let checksummed = format(&addr);
        let normalized = if address.starts_with("0x") {
            address.to_owned()
        } else {
            format!("0x{address}")
        };
        checksummed == normalized
    })
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
#[must_use]
#[inline]
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
#[must_use]
#[inline]
pub fn ensure_0x_prefix(address: &str) -> String {
    if address.starts_with("0x") {
        return address.to_owned();
    }
    format!("0x{address}")
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // === validate tests ===
    #[test]
    fn validate_when_valid_with_prefix_should_return_ok() {
        validate("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").unwrap();
        validate("0x0000000000000000000000000000000000000000").unwrap();
        validate("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
    }

    #[test]
    fn validate_when_valid_without_prefix_should_return_ok() {
        validate("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").unwrap();
        validate("0000000000000000000000000000000000000000").unwrap();
        validate("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
    }

    #[test]
    fn validate_when_invalid_format_should_return_err() {
        assert!(validate("invalid").is_err());
        assert!(validate("0x123").is_err());
        assert!(validate("123").is_err());
        assert!(validate("").is_err());
        assert!(validate("0xGGGG").is_err());
        assert!(validate("GGGG").is_err());
    }

    #[test]
    fn validate_when_wrong_length_should_return_err() {
        assert!(validate("0x123456789").is_err());
        assert!(validate("123456789").is_err());
        assert!(validate("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e1234").is_err());
        assert!(validate("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e1234").is_err());
    }

    // === parse tests ===
    #[test]
    fn parse_when_valid_with_prefix_should_return_address() {
        let addr1 = parse("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")
            .expect("Valid test address should parse successfully");
        let addr2 = parse("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")
            .expect("Valid test address should parse successfully");
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn parse_when_zero_address_should_return_zero() {
        let zero = parse("0x0000000000000000000000000000000000000000")
            .expect("Zero address should parse successfully");
        assert_eq!(zero, Address::ZERO);
    }

    #[test]
    fn parse_when_valid_without_prefix_40_chars_should_add_prefix() {
        let addr = parse("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")
            .expect("Valid test address should parse successfully");
        let addr_with_prefix = parse("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")
            .expect("Valid test address should parse successfully");
        assert_eq!(addr, addr_with_prefix);
    }

    #[test]
    fn parse_when_invalid_length_without_prefix_should_return_err() {
        let result = parse("123456789");
        match result.unwrap_err() {
            Error::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address length: 9"));
                assert!(msg.contains("Expected 40 hex characters"));
            }
            _ => unreachable!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn parse_when_invalid_length_with_prefix_should_return_err() {
        let result = parse("0x123");
        match result.unwrap_err() {
            Error::InvalidAddress(msg) => {
                assert!(msg.contains("Failed to parse address '0x123'"));
            }
            _ => unreachable!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn parse_when_invalid_hex_should_return_err() {
        let result = parse("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG");
        match result.unwrap_err() {
            Error::InvalidAddress(msg) => {
                assert!(msg.contains("Failed to parse address"));
            }
            _ => unreachable!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn parse_when_empty_string_should_return_err() {
        let result = parse("");
        match result.unwrap_err() {
            Error::InvalidAddress(msg) => {
                assert!(msg.contains("Invalid address length: 0"));
            }
            _ => unreachable!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn parse_when_lowercase_should_work() {
        let addr = parse("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")
            .expect("Valid lowercase test address should parse successfully");
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn parse_when_uppercase_should_work() {
        let addr = parse("0x742D35CC67A5B747BE4C506C5E8B0A146D7B2E9E")
            .expect("Valid uppercase test address should parse successfully");
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn parse_when_mixed_case_should_work() {
        let addr = parse("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")
            .expect("Valid mixed case test address should parse successfully");
        assert_ne!(addr, Address::ZERO);
    }

    // === format tests ===
    #[test]
    fn format_when_valid_address_should_return_checksummed() {
        let addr = parse("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")
            .expect("Valid test address should parse successfully for formatting test");
        let formatted = format(&addr);
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
        // The result should be checksummed (some mix of upper and lower case)
        assert_ne!(formatted, formatted.to_lowercase());
    }

    #[test]
    fn format_when_zero_address_should_return_zeros() {
        let formatted = format(&Address::ZERO);
        assert_eq!(formatted, "0x0000000000000000000000000000000000000000");
    }

    #[test]
    fn format_when_max_address_should_format_correctly() {
        let max_addr = Address::from_str("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
            .expect("Max address should parse successfully");
        let formatted = format(&max_addr);
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
        assert!(formatted.contains('F') || formatted.contains('f'));
    }

    // === format_string tests ===
    #[test]
    fn format_string_when_valid_with_prefix_should_return_checksummed() {
        let result = format_string("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e");
        let formatted = result.unwrap();
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
    }

    #[test]
    fn format_string_when_valid_without_prefix_should_return_checksummed() {
        let result = format_string("742d35cc67a5b747be4c506c5e8b0a146d7b2e9e");
        let formatted = result.unwrap();
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
    }

    #[test]
    fn format_string_when_invalid_should_return_err() {
        let result = format_string("invalid");
        match result.unwrap_err() {
            Error::InvalidAddress(_) => {}
            _ => unreachable!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn format_string_when_wrong_length_should_return_err() {
        let result = format_string("0x123");
        result.unwrap_err();
    }

    #[test]
    fn format_string_when_empty_should_return_err() {
        let result = format_string("");
        result.unwrap_err();
    }

    // === is_checksummed tests ===
    #[test]
    fn is_checksummed_when_lowercase_should_return_false() {
        let lowercased = "0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e";
        assert!(!is_checksummed(lowercased));
    }

    #[test]
    fn is_checksummed_when_uppercase_should_return_false() {
        let uppercased = "0x742D35CC67A5B747BE4C506C5E8B0A146D7B2E9E";
        assert!(!is_checksummed(uppercased));
    }

    #[test]
    fn is_checksummed_when_properly_checksummed_should_return_true() {
        // First get a properly checksummed address
        let addr = parse("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")
            .expect("Test address should parse successfully for checksum test");
        let checksummed = format(&addr);
        assert!(is_checksummed(&checksummed));
    }

    #[test]
    fn is_checksummed_when_zero_address_should_return_true() {
        assert!(is_checksummed("0x0000000000000000000000000000000000000000"));
    }

    #[test]
    fn is_checksummed_when_invalid_address_should_return_false() {
        assert!(!is_checksummed("invalid"));
        assert!(!is_checksummed("0x123"));
        assert!(!is_checksummed(""));
        assert!(!is_checksummed(
            "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG"
        ));
    }

    #[test]
    fn is_checksummed_when_without_prefix_but_valid_should_check_with_prefix() {
        // Test that the function properly handles addresses without 0x prefix
        let without_prefix = "742d35cc67a5b747be4c506c5e8b0a146d7b2e9e";
        // Should return false since it's all lowercase
        assert!(!is_checksummed(without_prefix));
    }

    #[test]
    fn is_checksummed_when_without_prefix_checksummed_should_return_true() {
        // Get a checksummed address and remove the prefix
        let addr = parse("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e")
            .expect("Test address should parse successfully for prefix removal test");
        let checksummed = format(&addr);
        let without_prefix = strip_0x_prefix(&checksummed);
        assert!(is_checksummed(without_prefix));
    }

    // === strip_0x_prefix tests ===
    #[test]
    fn strip_0x_prefix_when_has_prefix_should_remove_it() {
        assert_eq!(
            strip_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
    }

    #[test]
    fn strip_0x_prefix_when_no_prefix_should_return_unchanged() {
        assert_eq!(
            strip_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
    }

    #[test]
    fn strip_0x_prefix_when_empty_string_should_return_empty() {
        assert_eq!(strip_0x_prefix(""), "");
    }

    #[test]
    fn strip_0x_prefix_when_only_0x_should_return_empty() {
        assert_eq!(strip_0x_prefix("0x"), "");
    }

    #[test]
    fn strip_0x_prefix_when_starts_with_0_but_not_0x_should_return_unchanged() {
        assert_eq!(strip_0x_prefix("0abc123"), "0abc123");
    }

    #[test]
    fn strip_0x_prefix_when_multiple_0x_should_remove_only_first() {
        assert_eq!(strip_0x_prefix("0x0x123"), "0x123");
    }

    // === ensure_0x_prefix tests ===
    #[test]
    fn ensure_0x_prefix_when_no_prefix_should_add_it() {
        assert_eq!(
            ensure_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
    }

    #[test]
    fn ensure_0x_prefix_when_has_prefix_should_return_unchanged() {
        assert_eq!(
            ensure_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
    }

    #[test]
    fn ensure_0x_prefix_when_empty_string_should_add_prefix() {
        assert_eq!(ensure_0x_prefix(""), "0x");
    }

    #[test]
    fn ensure_0x_prefix_when_only_0_should_add_x() {
        assert_eq!(ensure_0x_prefix("0"), "0x0");
    }

    #[test]
    fn ensure_0x_prefix_when_starts_with_0_but_not_0x_should_add_prefix() {
        assert_eq!(ensure_0x_prefix("0abc123"), "0x0abc123");
    }

    // === known_addresses tests ===
    #[test]
    fn known_addresses_when_all_constants_should_be_valid() {
        use known_addresses::*;

        validate(ZERO_ADDRESS).unwrap();
        validate(BURN_ADDRESS).unwrap();
        validate(WETH_ETHEREUM).unwrap();
        validate(USDC_ETHEREUM).unwrap();
        validate(USDT_ETHEREUM).unwrap();
    }

    #[test]
    fn known_addresses_when_parsed_should_be_correct() {
        use known_addresses::*;

        let zero = parse(ZERO_ADDRESS).expect("Zero address constant should parse successfully");
        assert_eq!(zero, Address::ZERO);

        let burn = parse(BURN_ADDRESS).expect("Burn address constant should parse successfully");
        assert_ne!(burn, Address::ZERO);

        let weth = parse(WETH_ETHEREUM).expect("WETH address constant should parse successfully");
        assert_ne!(weth, Address::ZERO);

        let usdc = parse(USDC_ETHEREUM).expect("USDC address constant should parse successfully");
        assert_ne!(usdc, Address::ZERO);

        let usdt_address =
            parse(USDT_ETHEREUM).expect("USDT address constant should parse successfully");
        assert_ne!(usdt_address, Address::ZERO);
    }

    #[test]
    fn known_addresses_when_formatted_should_maintain_format() {
        use known_addresses::*;

        let addresses = [
            ZERO_ADDRESS,
            BURN_ADDRESS,
            WETH_ETHEREUM,
            USDC_ETHEREUM,
            USDT_ETHEREUM,
        ];

        for addr_str in addresses {
            let addr = parse(addr_str).expect("Known address constant should parse successfully");
            let formatted = format(&addr);
            assert!(formatted.starts_with("0x"));
            assert_eq!(formatted.len(), 42);
        }
    }

    // === Additional edge case tests ===
    #[test]
    fn edge_cases_when_max_values_should_work() {
        // Test with maximum address value
        let max_addr_str = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";
        validate(max_addr_str).unwrap();
        let addr = parse(max_addr_str).expect("Max address should parse successfully");
        let formatted = format(&addr);
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
    }

    #[test]
    fn edge_cases_when_all_numeric_should_work() {
        let numeric_addr = "1234567890123456789012345678901234567890";
        validate(numeric_addr).unwrap();
        let addr = parse(numeric_addr).expect("Numeric address should parse successfully");
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn edge_cases_when_mixed_case_boundaries_should_work() {
        // Test with mixed case at boundaries
        let mixed_case = "0xaBcDeF1234567890aBcDeF1234567890aBcDeF12";
        validate(mixed_case).unwrap();
        let addr = parse(mixed_case).expect("Mixed case address should parse successfully");
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn integration_when_round_trip_should_preserve_value() {
        // Test round-trip: string -> Address -> checksummed string -> Address
        let original = "742d35cc67a5b747be4c506c5e8b0a146d7b2e9e";
        let addr1 = parse(original).expect("Original address should parse successfully");
        let checksummed = format(&addr1);
        let addr2 = parse(&checksummed).expect("Checksummed address should parse successfully");
        assert_eq!(addr1, addr2);
    }
}
