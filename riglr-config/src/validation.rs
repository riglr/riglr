//! Configuration validation utilities

use crate::{ConfigError, ConfigResult};

/// Trait for validatable configuration
pub trait Validator {
    /// Validate the configuration
    fn validate(&self) -> ConfigResult<()>;
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
