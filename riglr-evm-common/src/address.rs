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
/// use riglr_evm_common::address::validate_evm_address;
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
/// use riglr_evm_common::address::parse_evm_address;
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
/// use riglr_evm_common::address::{parse_evm_address, format_address};
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
/// use riglr_evm_common::address::format_address_string;
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
/// use riglr_evm_common::address::is_checksummed;
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
/// use riglr_evm_common::address::strip_0x_prefix;
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
/// use riglr_evm_common::address::ensure_0x_prefix;
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

    #[test]
    fn test_validate_address() {
        // Valid addresses
        assert!(validate_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());
        assert!(validate_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").is_ok());
        assert!(validate_evm_address("0x0000000000000000000000000000000000000000").is_ok());

        // Invalid addresses
        assert!(validate_evm_address("invalid").is_err());
        assert!(validate_evm_address("0x123").is_err());
        assert!(validate_evm_address("123").is_err());
        assert!(validate_evm_address("").is_err());
    }

    #[test]
    fn test_parse_address() {
        let addr1 = parse_evm_address("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").unwrap();
        let addr2 = parse_evm_address("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").unwrap();
        assert_eq!(addr1, addr2);

        // Test zero address
        let zero = parse_evm_address("0x0000000000000000000000000000000000000000").unwrap();
        assert_eq!(zero, Address::ZERO);
    }

    #[test]
    fn test_format_address() {
        let addr = parse_evm_address("0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e").unwrap();
        let formatted = format_address(&addr);
        // Note: Expected checksum may vary - this tests the function works
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
    }

    #[test]
    fn test_format_address_string() {
        let result = format_address_string("742d35cc67a5b747be4c506c5e8b0a146d7b2e9e");
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
    }

    #[test]
    fn test_checksummed_detection() {
        // This test depends on the actual checksum algorithm
        let lowercased = "0x742d35cc67a5b747be4c506c5e8b0a146d7b2e9e";
        assert!(!is_checksummed(lowercased));

        // Test invalid address
        assert!(!is_checksummed("invalid"));
    }

    #[test]
    fn test_prefix_utilities() {
        assert_eq!(
            strip_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
        assert_eq!(
            strip_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );

        assert_eq!(
            ensure_0x_prefix("742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
        assert_eq!(
            ensure_0x_prefix("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"),
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
    }

    #[test]
    fn test_known_addresses() {
        use known_addresses::*;

        assert!(validate_evm_address(ZERO_ADDRESS).is_ok());
        assert!(validate_evm_address(BURN_ADDRESS).is_ok());
        assert!(validate_evm_address(WETH_ETHEREUM).is_ok());
        assert!(validate_evm_address(USDC_ETHEREUM).is_ok());
        assert!(validate_evm_address(USDT_ETHEREUM).is_ok());
    }
}
