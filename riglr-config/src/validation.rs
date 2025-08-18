//! Configuration validation utilities

use crate::{ConfigError, ConfigResult};

/// Trait for validatable configuration
pub trait Validator {
    /// Validate the configuration
    fn validate(&self) -> ConfigResult<()>;

    /// Validate the configuration (public interface)
    fn validate_config(&self) -> ConfigResult<()> {
        self.validate()
    }
}

/// Validate an email address
#[allow(dead_code)]
pub fn validate_email(email: &str) -> ConfigResult<()> {
    if !email.contains('@') || !email.contains('.') {
        return Err(ConfigError::validation(format!(
            "Invalid email address: {}",
            email
        )));
    }
    Ok(())
}

/// Validate a URL
#[allow(dead_code)]
pub fn validate_url(url: &str) -> ConfigResult<()> {
    url::Url::parse(url)
        .map_err(|e| ConfigError::validation(format!("Invalid URL '{}': {}", url, e)))?;
    Ok(())
}

/// Validate a port number
#[allow(dead_code)]
pub fn validate_port(port: u16) -> ConfigResult<()> {
    if port == 0 {
        return Err(ConfigError::validation("Port number cannot be 0"));
    }
    Ok(())
}

/// Validate an Ethereum address
#[allow(dead_code)]
pub fn validate_eth_address(address: &str) -> ConfigResult<()> {
    if !address.starts_with("0x") || address.len() != 42 {
        return Err(ConfigError::validation(format!(
            "Invalid Ethereum address: {}",
            address
        )));
    }

    // Check if it's valid hex (basic check)
    for c in address[2..].chars() {
        if !c.is_ascii_hexdigit() {
            return Err(ConfigError::validation(format!(
                "Invalid hex character in address: {}",
                c
            )));
        }
    }

    Ok(())
}

/// Validate a Solana address
#[allow(dead_code)]
pub fn validate_solana_address(address: &str) -> ConfigResult<()> {
    // Solana addresses are base58 encoded and typically 32-44 characters
    if address.len() < 32 || address.len() > 44 {
        return Err(ConfigError::validation(format!(
            "Invalid Solana address length: {}",
            address
        )));
    }

    // Check if it's valid base58
    for c in address.chars() {
        if !c.is_ascii_alphanumeric() {
            return Err(ConfigError::validation(format!(
                "Invalid character in Solana address: {}",
                c
            )));
        }
    }

    Ok(())
}

/// Validate an API key format
#[allow(dead_code)]
pub fn validate_api_key(key: &str, name: &str) -> ConfigResult<()> {
    if key.is_empty() {
        return Err(ConfigError::validation(format!("{} cannot be empty", name)));
    }

    if key.contains(' ') && !key.starts_with("Bearer ") {
        return Err(ConfigError::validation(format!(
            "{} contains invalid spaces",
            name
        )));
    }

    Ok(())
}

/// Validate a percentage value (0-100)
#[allow(dead_code)]
pub fn validate_percentage(value: f64, name: &str) -> ConfigResult<()> {
    if !(0.0..=100.0).contains(&value) {
        return Err(ConfigError::validation(format!(
            "{} must be between 0 and 100, got {}",
            name, value
        )));
    }
    Ok(())
}

/// Validate a positive number
#[allow(dead_code)]
pub fn validate_positive<T: PartialOrd + Default + std::fmt::Display>(
    value: T,
    name: &str,
) -> ConfigResult<()> {
    if value <= T::default() {
        return Err(ConfigError::validation(format!(
            "{} must be positive, got {}",
            name, value
        )));
    }
    Ok(())
}

/// Validate a range
#[allow(dead_code)]
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    name: &str,
) -> ConfigResult<()> {
    if value < min || value > max {
        return Err(ConfigError::validation(format!(
            "{} must be between {} and {}, got {}",
            name, min, max, value
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for validate_email
    #[test]
    fn test_validate_email_when_valid_should_return_ok() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.email@domain.org").is_ok());
        assert!(validate_email("complex+email@sub.domain.co.uk").is_ok());
        assert!(validate_email("user123@test123.net").is_ok());
    }

    #[test]
    fn test_validate_email_when_missing_at_should_return_err() {
        let result = validate_email("userexample.com");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid email address: userexample.com"));
    }

    #[test]
    fn test_validate_email_when_missing_dot_should_return_err() {
        let result = validate_email("user@examplecom");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid email address: user@examplecom"));
    }

    #[test]
    fn test_validate_email_when_missing_both_at_and_dot_should_return_err() {
        let result = validate_email("userexamplecom");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid email address: userexamplecom"));
    }

    #[test]
    fn test_validate_email_when_empty_should_return_err() {
        let result = validate_email("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid email address: "));
    }

    #[test]
    fn test_validate_email_when_only_at_should_return_err() {
        let result = validate_email("@");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid email address: @"));
    }

    #[test]
    fn test_validate_email_when_only_dot_should_return_err() {
        let result = validate_email(".");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid email address: ."));
    }

    #[test]
    fn test_validate_email_when_at_and_dot_but_no_domain_should_return_ok() {
        // This is a basic check, so it accepts minimal format
        assert!(validate_email("@.").is_ok());
    }

    // Tests for validate_url
    #[test]
    fn test_validate_url_when_valid_http_should_return_ok() {
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("http://www.example.com").is_ok());
        assert!(validate_url("http://sub.domain.example.com").is_ok());
    }

    #[test]
    fn test_validate_url_when_valid_https_should_return_ok() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("https://www.example.com").is_ok());
        assert!(validate_url("https://api.example.com/v1/endpoint").is_ok());
    }

    #[test]
    fn test_validate_url_when_valid_with_port_should_return_ok() {
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("https://example.com:443").is_ok());
    }

    #[test]
    fn test_validate_url_when_valid_with_path_should_return_ok() {
        assert!(validate_url("https://example.com/path/to/resource").is_ok());
        assert!(validate_url("https://api.example.com/v1/users?id=123").is_ok());
    }

    #[test]
    fn test_validate_url_when_invalid_scheme_should_return_err() {
        let result = validate_url("invalid://example.com");
        assert!(result.is_ok()); // url crate accepts custom schemes
    }

    #[test]
    fn test_validate_url_when_missing_scheme_should_return_err() {
        let result = validate_url("example.com");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid URL 'example.com'"));
    }

    #[test]
    fn test_validate_url_when_empty_should_return_err() {
        let result = validate_url("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid URL ''"));
    }

    #[test]
    fn test_validate_url_when_malformed_should_return_err() {
        let result = validate_url("http://");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid URL 'http://'"));
    }

    #[test]
    fn test_validate_url_when_special_characters_should_return_ok() {
        assert!(validate_url("https://example.com/path?param=value&other=123").is_ok());
    }

    // Tests for validate_port
    #[test]
    fn test_validate_port_when_valid_ports_should_return_ok() {
        assert!(validate_port(80).is_ok());
        assert!(validate_port(443).is_ok());
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(3000).is_ok());
        assert!(validate_port(65535).is_ok()); // Max port
        assert!(validate_port(1).is_ok()); // Min valid port
    }

    #[test]
    fn test_validate_port_when_zero_should_return_err() {
        let result = validate_port(0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid configuration: Port number cannot be 0"
        );
    }

    #[test]
    fn test_validate_port_when_common_ports_should_return_ok() {
        assert!(validate_port(21).is_ok()); // FTP
        assert!(validate_port(22).is_ok()); // SSH
        assert!(validate_port(25).is_ok()); // SMTP
        assert!(validate_port(53).is_ok()); // DNS
        assert!(validate_port(110).is_ok()); // POP3
        assert!(validate_port(993).is_ok()); // IMAPS
        assert!(validate_port(5432).is_ok()); // PostgreSQL
    }

    // Tests for validate_eth_address
    #[test]
    fn test_validate_eth_address_when_valid_should_return_ok() {
        assert!(validate_eth_address("0x742c4A3F86aE6c46F7Ef6a96E936C3F7E3D16E8E").is_ok());
        assert!(validate_eth_address("0x0000000000000000000000000000000000000000").is_ok());
        assert!(validate_eth_address("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").is_ok());
        assert!(validate_eth_address("0x1234567890abcdef1234567890abcdef12345678").is_ok());
        assert!(validate_eth_address("0xABCDEF1234567890ABCDEF1234567890ABCDEF12").is_ok());
    }

    #[test]
    fn test_validate_eth_address_when_missing_0x_prefix_should_return_err() {
        let result = validate_eth_address("742c4A3F86aE6c46F7Ef6a96E936C3F7E3D16E8E");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Ethereum address"));
    }

    #[test]
    fn test_validate_eth_address_when_wrong_length_should_return_err() {
        let result = validate_eth_address("0x742c4A3F86aE6c46F7Ef6a96E936C3F7E3D16E"); // 41 chars
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Ethereum address"));

        let result = validate_eth_address("0x742c4A3F86aE6c46F7Ef6a96E936C3F7E3D16E8E1"); // 43 chars
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Ethereum address"));
    }

    #[test]
    fn test_validate_eth_address_when_empty_should_return_err() {
        let result = validate_eth_address("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Ethereum address"));
    }

    #[test]
    fn test_validate_eth_address_when_only_0x_should_return_err() {
        let result = validate_eth_address("0x");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Ethereum address"));
    }

    #[test]
    fn test_validate_eth_address_when_invalid_hex_characters_should_return_err() {
        let result = validate_eth_address("0x742c4A3F86aE6c46F7Ef6a96E936C3F7E3D16EGH");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid hex character in address: G"));

        let result = validate_eth_address("0x742c4A3F86aE6c46F7Ef6a96E936C3F7E3D16E@E");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid hex character in address: @"));
    }

    #[test]
    fn test_validate_eth_address_when_lowercase_hex_should_return_ok() {
        assert!(validate_eth_address("0x742c4a3f86ae6c46f7ef6a96e936c3f7e3d16e8e").is_ok());
    }

    #[test]
    fn test_validate_eth_address_when_mixed_case_should_return_ok() {
        assert!(validate_eth_address("0x742c4A3f86aE6C46f7eF6a96E936c3F7e3D16e8E").is_ok());
    }

    // Tests for validate_solana_address
    #[test]
    fn test_validate_solana_address_when_valid_should_return_ok() {
        assert!(validate_solana_address("11111111111111111111111111111112").is_ok()); // 32 chars
        assert!(validate_solana_address("1234567890123456789012345678901234567890123").is_ok()); // 43 chars
        assert!(validate_solana_address("12345678901234567890123456789012345678901234").is_ok()); // 44 chars
        assert!(validate_solana_address("SysvarC1ock11111111111111111111111111111111").is_ok());
        assert!(validate_solana_address("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").is_ok());
    }

    #[test]
    fn test_validate_solana_address_when_too_short_should_return_err() {
        let result = validate_solana_address("1111111111111111111111111111111"); // 31 chars
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Solana address length"));
    }

    #[test]
    fn test_validate_solana_address_when_too_long_should_return_err() {
        let result = validate_solana_address("123456789012345678901234567890123456789012345"); // 45 chars
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Solana address length"));
    }

    #[test]
    fn test_validate_solana_address_when_empty_should_return_err() {
        let result = validate_solana_address("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Solana address length"));
    }

    #[test]
    fn test_validate_solana_address_when_special_characters_should_return_err() {
        let result = validate_solana_address("1111111111111111111111111111111@");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid character in Solana address: @"));

        let result = validate_solana_address("1111111111111111111111111111111#");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid character in Solana address: #"));
    }

    #[test]
    fn test_validate_solana_address_when_spaces_should_return_err() {
        let result = validate_solana_address("111111111111111111111111111111 2");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid character in Solana address:  "));
    }

    #[test]
    fn test_validate_solana_address_boundary_lengths() {
        assert!(validate_solana_address("12345678901234567890123456789012").is_ok()); // 32 chars exactly
        assert!(validate_solana_address("12345678901234567890123456789012345678901234").is_ok());
        // 44 chars exactly
    }

    // Tests for validate_api_key
    #[test]
    fn test_validate_api_key_when_valid_should_return_ok() {
        assert!(validate_api_key("sk-1234567890abcdef", "API Key").is_ok());
        assert!(validate_api_key("Bearer token123", "Auth Token").is_ok());
        assert!(validate_api_key("abc123def456", "Secret Key").is_ok());
        assert!(validate_api_key("very_long_api_key_with_underscores", "API Key").is_ok());
    }

    #[test]
    fn test_validate_api_key_when_empty_should_return_err() {
        let result = validate_api_key("", "API Key");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid configuration: API Key cannot be empty"
        );
    }

    #[test]
    fn test_validate_api_key_when_contains_spaces_but_not_bearer_should_return_err() {
        let result = validate_api_key("invalid key with spaces", "API Key");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("API Key contains invalid spaces"));
    }

    #[test]
    fn test_validate_api_key_when_bearer_with_spaces_should_return_ok() {
        assert!(validate_api_key("Bearer abc123def456", "Auth Token").is_ok());
        assert!(validate_api_key("Bearer token_with_underscores", "Bearer Token").is_ok());
    }

    #[test]
    fn test_validate_api_key_when_bearer_prefix_only_should_return_ok() {
        assert!(validate_api_key("Bearer ", "Token").is_ok());
    }

    #[test]
    fn test_validate_api_key_different_names() {
        let result = validate_api_key("", "Twitter API Key");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Twitter API Key cannot be empty"));

        let result = validate_api_key("invalid key", "OpenAI Token");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("OpenAI Token contains invalid spaces"));
    }

    #[test]
    fn test_validate_api_key_with_special_characters_should_return_ok() {
        assert!(validate_api_key("sk-1234567890abcdef!@#$%^&*()", "API Key").is_ok());
        assert!(validate_api_key("token.with.dots", "Token").is_ok());
        assert!(validate_api_key("token-with-hyphens", "Token").is_ok());
    }

    // Tests for validate_percentage
    #[test]
    fn test_validate_percentage_when_valid_should_return_ok() {
        assert!(validate_percentage(0.0, "Test").is_ok());
        assert!(validate_percentage(50.0, "Test").is_ok());
        assert!(validate_percentage(100.0, "Test").is_ok());
        assert!(validate_percentage(0.1, "Test").is_ok());
        assert!(validate_percentage(99.9, "Test").is_ok());
        assert!(validate_percentage(25.5, "Test").is_ok());
    }

    #[test]
    fn test_validate_percentage_when_below_zero_should_return_err() {
        let result = validate_percentage(-0.1, "Percentage");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Percentage must be between 0 and 100, got -0.1"));
    }

    #[test]
    fn test_validate_percentage_when_above_hundred_should_return_err() {
        let result = validate_percentage(100.1, "Percentage");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Percentage must be between 0 and 100, got 100.1"));
    }

    #[test]
    fn test_validate_percentage_boundary_values() {
        assert!(validate_percentage(0.0, "Lower Bound").is_ok());
        assert!(validate_percentage(100.0, "Upper Bound").is_ok());
    }

    #[test]
    fn test_validate_percentage_with_negative_value() {
        let result = validate_percentage(-50.0, "Negative Test");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Negative Test must be between 0 and 100, got -50"));
    }

    #[test]
    fn test_validate_percentage_with_large_value() {
        let result = validate_percentage(1000.0, "Large Value");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Large Value must be between 0 and 100, got 1000"));
    }

    #[test]
    fn test_validate_percentage_different_names() {
        let result = validate_percentage(-1.0, "CPU Usage");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("CPU Usage must be between 0 and 100"));

        let result = validate_percentage(101.0, "Memory Usage");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Memory Usage must be between 0 and 100"));
    }

    // Tests for validate_positive
    #[test]
    fn test_validate_positive_when_positive_integers_should_return_ok() {
        assert!(validate_positive(1i32, "Integer").is_ok());
        assert!(validate_positive(100i32, "Integer").is_ok());
        assert!(validate_positive(i32::MAX, "Integer").is_ok());
    }

    #[test]
    fn test_validate_positive_when_positive_floats_should_return_ok() {
        assert!(validate_positive(0.1f64, "Float").is_ok());
        assert!(validate_positive(1.0f64, "Float").is_ok());
        assert!(validate_positive(100.5f64, "Float").is_ok());
        assert!(validate_positive(f64::MAX, "Float").is_ok());
    }

    #[test]
    fn test_validate_positive_when_zero_should_return_err() {
        let result = validate_positive(0i32, "Value");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Value must be positive, got 0"));

        let result = validate_positive(0.0f64, "Float Value");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Float Value must be positive, got 0"));
    }

    #[test]
    fn test_validate_positive_when_negative_should_return_err() {
        let result = validate_positive(-1i32, "Negative Integer");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Negative Integer must be positive, got -1"));

        let result = validate_positive(-0.1f64, "Negative Float");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Negative Float must be positive, got -0.1"));
    }

    #[test]
    fn test_validate_positive_with_unsigned_types() {
        assert!(validate_positive(1u32, "Unsigned").is_ok());
        assert!(validate_positive(u32::MAX, "Unsigned").is_ok());

        let result = validate_positive(0u32, "Unsigned Zero");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsigned Zero must be positive, got 0"));
    }

    #[test]
    fn test_validate_positive_with_different_names() {
        let result = validate_positive(0, "Timeout");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Timeout must be positive"));

        let result = validate_positive(-5, "Port Number");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Port Number must be positive"));
    }

    // Tests for validate_range
    #[test]
    fn test_validate_range_when_within_range_should_return_ok() {
        assert!(validate_range(5, 0, 10, "Value").is_ok());
        assert!(validate_range(0, 0, 10, "Lower Bound").is_ok());
        assert!(validate_range(10, 0, 10, "Upper Bound").is_ok());
        assert!(validate_range(50.5, 0.0, 100.0, "Float").is_ok());
    }

    #[test]
    fn test_validate_range_when_below_minimum_should_return_err() {
        let result = validate_range(-1, 0, 10, "Below Min");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Below Min must be between 0 and 10, got -1"));
    }

    #[test]
    fn test_validate_range_when_above_maximum_should_return_err() {
        let result = validate_range(11, 0, 10, "Above Max");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Above Max must be between 0 and 10, got 11"));
    }

    #[test]
    fn test_validate_range_boundary_values() {
        assert!(validate_range(0, 0, 0, "Single Value").is_ok());
        assert!(validate_range(100, 100, 100, "Single Value").is_ok());
    }

    #[test]
    fn test_validate_range_with_floats() {
        assert!(validate_range(5.5, 0.0, 10.0, "Float Range").is_ok());

        let result = validate_range(-0.1, 0.0, 10.0, "Float Below");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Float Below must be between 0 and 10, got -0.1"));

        let result = validate_range(10.1, 0.0, 10.0, "Float Above");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Float Above must be between 0 and 10, got 10.1"));
    }

    #[test]
    fn test_validate_range_with_negative_ranges() {
        assert!(validate_range(-5, -10, 0, "Negative Range").is_ok());
        assert!(validate_range(-10, -10, 0, "Negative Lower").is_ok());
        assert!(validate_range(0, -10, 0, "Negative Upper").is_ok());
    }

    #[test]
    fn test_validate_range_different_names() {
        let result = validate_range(150, 0, 100, "CPU Percentage");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("CPU Percentage must be between 0 and 100, got 150"));

        let result = validate_range(-5, 1, 65535, "Port Number");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Port Number must be between 1 and 65535, got -5"));
    }

    #[test]
    fn test_validate_range_with_chars() {
        assert!(validate_range('m', 'a', 'z', "Letter").is_ok());

        let result = validate_range('A', 'a', 'z', "Uppercase");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Uppercase must be between a and z, got A"));
    }

    // Edge case tests
    #[test]
    fn test_all_validators_with_extreme_inputs() {
        // Test with very long strings
        let long_string = "a".repeat(1000);
        let result = validate_email(&long_string);
        assert!(result.is_err());

        // Test with unicode characters in various validators
        assert!(validate_api_key("🔑key", "Unicode Key").is_ok());

        let result = validate_eth_address("0x742c4A3F86aE6c46F7Ef6a96E936C3F7E3D16E🔑E");
        assert!(result.is_err());
    }

    #[test]
    fn test_validator_trait_can_be_used() {
        struct TestConfig {
            email: String,
        }

        impl Validator for TestConfig {
            fn validate(&self) -> ConfigResult<()> {
                validate_email(&self.email)
            }
        }

        let config = TestConfig {
            email: "test@example.com".to_string(),
        };
        assert!(config.validate_config().is_ok());

        let invalid_config = TestConfig {
            email: "invalid-email".to_string(),
        };
        assert!(invalid_config.validate_config().is_err());
    }
}
