/// Checks if a hex data string starts with a given hex discriminator
///
/// Both `data` and `discriminator` must be valid hex strings starting with "0x".
/// Returns `true` if the data string begins with the discriminator string.
///
/// # Arguments
///
/// * `data` - The hex data string to check
/// * `discriminator` - The hex discriminator to match against
///
/// # Returns
///
/// `true` if the data starts with the discriminator, `false` otherwise
pub fn discriminator_matches(data: &str, discriminator: &str) -> bool {
    if !data.starts_with("0x") || !discriminator.starts_with("0x") {
        return false;
    }
    let dlen = discriminator.len();
    data.len() >= dlen && &data[..dlen] == discriminator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discriminator_matches_when_valid_matching_should_return_true() {
        // Happy Path: Valid hex strings with matching discriminator
        assert!(discriminator_matches("0x123456789abc", "0x1234"));
        assert!(discriminator_matches("0xabcdef", "0xabcd"));
        assert!(discriminator_matches("0x0000", "0x00"));
    }

    #[test]
    fn test_discriminator_matches_when_exact_match_should_return_true() {
        // Edge Case: Exact match (same length)
        assert!(discriminator_matches("0x1234", "0x1234"));
        assert!(discriminator_matches("0x", "0x"));
    }

    #[test]
    fn test_discriminator_matches_when_valid_non_matching_should_return_false() {
        // Happy Path: Valid hex strings but discriminator doesn't match
        assert!(!discriminator_matches("0x123456789abc", "0x5678"));
        assert!(!discriminator_matches("0xabcdef", "0x1234"));
        assert!(!discriminator_matches("0x0000", "0x11"));
    }

    #[test]
    fn test_discriminator_matches_when_data_missing_0x_prefix_should_return_false() {
        // Error Path: Data doesn't start with "0x"
        assert!(!discriminator_matches("123456789abc", "0x1234"));
        assert!(!discriminator_matches("", "0x1234"));
        assert!(!discriminator_matches("x1234", "0x1234"));
        assert!(!discriminator_matches("1x1234", "0x1234"));
    }

    #[test]
    fn test_discriminator_matches_when_discriminator_missing_0x_prefix_should_return_false() {
        // Error Path: Discriminator doesn't start with "0x"
        assert!(!discriminator_matches("0x123456789abc", "1234"));
        assert!(!discriminator_matches("0x123456789abc", ""));
        assert!(!discriminator_matches("0x123456789abc", "x1234"));
        assert!(!discriminator_matches("0x123456789abc", "1x1234"));
    }

    #[test]
    fn test_discriminator_matches_when_both_missing_0x_prefix_should_return_false() {
        // Error Path: Both don't start with "0x"
        assert!(!discriminator_matches("123456789abc", "1234"));
        assert!(!discriminator_matches("", ""));
        assert!(!discriminator_matches("abcd", "ab"));
    }

    #[test]
    fn test_discriminator_matches_when_discriminator_longer_than_data_should_return_false() {
        // Edge Case: Discriminator is longer than data
        assert!(!discriminator_matches("0x12", "0x123456"));
        assert!(!discriminator_matches("0x", "0x1234"));
        assert!(!discriminator_matches("0x1", "0x12"));
    }

    #[test]
    fn test_discriminator_matches_when_case_sensitive_should_return_false() {
        // Edge Case: Case sensitivity
        assert!(!discriminator_matches("0x123abc", "0x123ABC"));
        assert!(!discriminator_matches("0xABCDEF", "0xabcdef"));
    }

    #[test]
    fn test_discriminator_matches_when_partial_match_at_start_should_return_true() {
        // Edge Case: Partial match at the beginning
        assert!(discriminator_matches("0x123456789", "0x123"));
        assert!(discriminator_matches("0xabcdefghij", "0xabcde"));
    }

    #[test]
    fn test_discriminator_matches_when_partial_match_not_at_start_should_return_false() {
        // Edge Case: Discriminator appears later in data but not at start
        assert!(!discriminator_matches("0x999123456", "0x123"));
        assert!(!discriminator_matches("0xfffabcdef", "0xabcd"));
    }

    #[test]
    fn test_discriminator_matches_when_empty_strings_after_0x_should_work() {
        // Edge Case: Just "0x" prefix
        assert!(discriminator_matches("0x", "0x"));
        assert!(discriminator_matches("0x123", "0x"));
        assert!(!discriminator_matches("0x", "0x1"));
    }

    #[test]
    fn test_discriminator_matches_when_special_hex_characters_should_work() {
        // Edge Case: Valid hex characters
        assert!(discriminator_matches("0xabcdef123456", "0xabcd"));
        assert!(discriminator_matches("0x0123456789abcdef", "0x0123"));
        assert!(discriminator_matches("0xABCDEF", "0xABCD"));
        assert!(!discriminator_matches("0xabcdef", "0xABCD")); // Case matters
    }
}
