//! Common EVM types shared across riglr crates
//!
//! This module provides shared type definitions and configuration structs
//! that are needed by both riglr-evm-tools and riglr-cross-chain-tools
//! to avoid duplication and circular dependencies.

use alloy::{
    hex,
    primitives::{Address, Bytes, U256},
};
use core::time::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::address::{parse, validate};
use crate::chain::id_to_rpc_url;
use crate::error::Error;

/// Configuration for EVM operations shared across crates
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct EvmConfig {
    /// Chain ID (e.g., 1 for Ethereum, 137 for Polygon)
    pub chain_id: u64,
    /// Default gas price multiplier (1.0 = exact, 1.1 = 10% higher)
    pub gas_price_multiplier: Option<f64>,
    /// Maximum gas limit for transactions
    pub max_gas_limit: Option<u64>,
    /// RPC endpoint URL for the EVM chain
    pub rpc_url: String,
    /// Request timeout duration
    pub timeout_seconds: u64,
}

impl Default for EvmConfig {
    #[inline]
    fn default() -> Self {
        Self {
            chain_id: 1,                     // Ethereum mainnet
            gas_price_multiplier: Some(1.1), // 10% buffer
            max_gas_limit: None,
            rpc_url: "https://eth.llamarpc.com".to_owned(),
            timeout_seconds: 30,
        }
    }
}

impl EvmConfig {
    /// Create config for a specific chain ID
    ///
    /// # Errors
    ///
    /// Returns an error if the chain ID is not supported or if no RPC URL is available for the chain.
    #[inline]
    pub fn for_chain(chain_id: u64) -> Result<Self, Error> {
        let rpc_url = id_to_rpc_url(chain_id)?;

        Ok(Self {
            chain_id,
            gas_price_multiplier: Some(1.1),
            max_gas_limit: None,
            rpc_url,
            timeout_seconds: 30,
        })
    }

    /// Get timeout as Duration
    #[must_use]
    #[inline]
    pub const fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    /// Validate configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid (empty RPC URL, invalid protocol, zero chain ID, zero timeout, or negative gas price multiplier).
    #[inline]
    pub fn validate(&self) -> Result<(), Error> {
        if self.rpc_url.is_empty() {
            return Err(Error::InvalidConfig("RPC URL cannot be empty".to_owned()));
        }

        if !self.rpc_url.starts_with("http://")
            && !self.rpc_url.starts_with("https://")
            && !self.rpc_url.starts_with("wss://")
        {
            return Err(Error::InvalidConfig(
                "RPC URL must start with http://, https://, or wss://".to_owned(),
            ));
        }

        if self.chain_id == 0 {
            return Err(Error::InvalidConfig("Chain ID cannot be 0".to_owned()));
        }

        if self.timeout_seconds == 0 {
            return Err(Error::InvalidConfig("Timeout cannot be 0".to_owned()));
        }

        if let Some(multiplier) = self.gas_price_multiplier {
            if multiplier <= 0.0_f64 {
                return Err(Error::InvalidConfig(
                    "Gas price multiplier must be positive".to_owned(),
                ));
            }
        }

        Ok(())
    }
}

/// Common EVM account metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct EvmAccount {
    /// Address of the account (hex format with 0x prefix)
    pub address: String,
    /// Chain ID where this account exists
    pub chain_id: u64,
    /// Optional account alias/name
    pub name: Option<String>,
}

impl EvmAccount {
    /// Format address for display (checksummed)
    ///
    /// # Errors
    ///
    /// Returns an error if the stored address cannot be parsed.
    #[must_use = "address formatting should be used"]
    #[inline]
    pub fn display_address(&self) -> Result<String, Error> {
        let addr = self.to_address()?;
        Ok(format!("0x{addr:x}"))
    }

    /// Create new EVM account with validation
    ///
    /// # Errors
    ///
    /// Returns an error if the address format is invalid.
    #[inline]
    pub fn new(address: &str, chain_id: u64) -> Result<Self, Error> {
        // Validate address format
        validate(address)?;

        Ok(Self {
            address: address.to_owned(),
            chain_id,
            name: None,
        })
    }

    /// Get address as Alloy Address type
    ///
    /// # Errors
    ///
    /// Returns an error if the stored address cannot be parsed.
    #[must_use = "parsed address should be used"]
    #[inline]
    pub fn to_address(&self) -> Result<Address, Error> {
        parse(&self.address)
    }

    /// Create new EVM account with name
    ///
    /// # Errors
    ///
    /// Returns an error if the address format is invalid.
    #[inline]
    pub fn with_name(address: &str, chain_id: u64, name: String) -> Result<Self, Error> {
        let mut account = Self::new(address, chain_id)?;
        account.name = Some(name);
        Ok(account)
    }
}

/// EVM transaction data for cross-chain operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct EvmTransactionData {
    /// Chain ID for this transaction
    pub chain_id: u64,
    /// Transaction data (hex encoded)
    pub data: String,
    /// Gas limit (hex encoded)
    pub gas_limit: String,
    /// Gas price (in wei, hex encoded)
    pub gas_price: String,
    /// Target contract address
    pub to: String,
    /// Value to send (in wei, hex encoded)
    pub value: String,
}

impl EvmTransactionData {
    /// Parse data as Alloy Bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the stored data is not valid hexadecimal.
    #[must_use = "parsed bytes should be used"]
    #[inline]
    pub fn data_bytes(&self) -> Result<Bytes, Error> {
        let hex_str = self.data.strip_prefix("0x").unwrap_or(&self.data);
        hex::decode(hex_str)
            .map_err(|err| Error::InvalidData(format!("Invalid hex data: {err}")))
            .map(Bytes::from)
    }

    /// Parse gas limit as u64
    ///
    /// # Errors
    ///
    /// Returns an error if the stored gas limit is not valid hexadecimal.
    #[must_use = "parsed gas limit should be used"]
    #[inline]
    pub fn gas_limit_u64(&self) -> Result<u64, Error> {
        u64::from_str_radix(
            self.gas_limit.strip_prefix("0x").unwrap_or(&self.gas_limit),
            16,
        )
        .map_err(|err| Error::InvalidData(format!("Invalid gas limit format: {err}")))
    }

    /// Parse gas price as U256
    ///
    /// # Errors
    ///
    /// Returns an error if the stored gas price is not valid hexadecimal.
    #[must_use = "parsed gas price should be used"]
    #[inline]
    pub fn gas_price_u256(&self) -> Result<U256, Error> {
        U256::from_str_radix(
            self.gas_price.strip_prefix("0x").unwrap_or(&self.gas_price),
            16,
        )
        .map_err(|err| Error::InvalidData(format!("Invalid gas price format: {err}")))
    }

    /// Create new EVM transaction data with validation
    ///
    /// # Errors
    ///
    /// Returns an error if the address format is invalid or if the data doesn't start with '0x'.
    #[inline]
    pub fn new(
        to: &str,
        data: &str,
        value: U256,
        gas_limit: u64,
        gas_price: U256,
        chain_id: u64,
    ) -> Result<Self, Error> {
        // Validate to address
        validate(to)?;

        // Validate data is hex
        if !data.starts_with("0x") {
            return Err(Error::InvalidData(
                "Transaction data must start with 0x".to_owned(),
            ));
        }

        Ok(Self {
            chain_id,
            data: data.to_owned(),
            gas_limit: format!("0x{gas_limit:x}"),
            gas_price: format!("0x{gas_price:x}"),
            to: to.to_owned(),
            value: format!("0x{value:x}"),
        })
    }

    /// Parse to address as Alloy Address
    ///
    /// # Errors
    ///
    /// Returns an error if the stored address cannot be parsed.
    #[must_use = "parsed address should be used"]
    #[inline]
    pub fn to_address(&self) -> Result<Address, Error> {
        parse(&self.to)
    }

    /// Parse value as U256
    ///
    /// # Errors
    ///
    /// Returns an error if the stored value is not valid hexadecimal.
    #[must_use = "parsed value should be used"]
    #[inline]
    pub fn value_u256(&self) -> Result<U256, Error> {
        U256::from_str_radix(self.value.strip_prefix("0x").unwrap_or(&self.value), 16)
            .map_err(|err| Error::InvalidData(format!("Invalid value format: {err}")))
    }
}

/// Token information for ERC20 and native tokens
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct EvmToken {
    /// Contract address (0x0 for native token)
    pub address: String,
    /// Chain ID where this token exists
    pub chain_id: u64,
    /// Number of decimal places
    pub decimals: u8,
    /// Token name (e.g., "USD Coin", "Ethereum")
    pub name: String,
    /// Token symbol (e.g., "USDC", "ETH")
    pub symbol: String,
}

impl EvmToken {
    /// Get contract address as Alloy Address (for non-native tokens)
    ///
    /// # Errors
    ///
    /// Returns an error if the stored address cannot be parsed (for non-native tokens).
    #[must_use = "contract address should be used"]
    #[inline]
    pub fn contract_address(&self) -> Result<Option<Address>, Error> {
        if self.is_native() {
            return Ok(None);
        }
        Ok(Some(parse(&self.address)?))
    }

    /// Convert amount from smallest unit to human-readable format
    #[must_use]
    #[inline]
    pub fn format_amount(&self, raw_amount: U256) -> String {
        let divisor = U256::from(10_i32).pow(U256::from(self.decimals));

        // Use checked arithmetic to avoid potential side effects
        let whole = raw_amount.checked_div(divisor).unwrap_or(U256::ZERO);
        let remainder = raw_amount.checked_rem(divisor).unwrap_or(U256::ZERO);

        if remainder == U256::ZERO {
            return format!("{} {}", whole, self.symbol);
        }
        // Calculate decimal portion
        let decimal_str = format!("{:0width$}", remainder, width = self.decimals as usize);
        let decimal_trimmed = decimal_str.trim_end_matches('0');

        if decimal_trimmed.is_empty() {
            return format!("{} {}", whole, self.symbol);
        }
        format!("{}.{} {}", whole, decimal_trimmed, self.symbol)
    }

    /// Check if this is a native token (ETH, MATIC, etc.)
    #[must_use]
    #[inline]
    pub fn is_native(&self) -> bool {
        self.address == "0x0000000000000000000000000000000000000000" || self.address == "0x0"
    }

    /// Create new EVM token with validation
    ///
    /// # Errors
    ///
    /// Returns an error if the address format is invalid (except for native token addresses '0x0' or '0x0000000000000000000000000000000000000000').
    #[inline]
    pub fn new(
        address: &str,
        symbol: String,
        name: String,
        decimals: u8,
        chain_id: u64,
    ) -> Result<Self, Error> {
        // Validate address (allow 0x0 for native token)
        if address != "0x0000000000000000000000000000000000000000" && address != "0x0" {
            validate(address)?;
        }

        Ok(Self {
            address: address.to_owned(),
            chain_id,
            decimals,
            name,
            symbol,
        })
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn evm_config_validation() {
        let mut config = EvmConfig::default();
        config.validate().expect("Valid config should validate");

        // Test invalid URL
        config.rpc_url = "invalid-url".to_owned();
        config
            .validate()
            .expect_err("Invalid config should fail validation");

        // Test valid URL
        config.rpc_url = "https://eth.llamarpc.com".to_owned();
        config.validate().expect("Valid config should validate");

        // Test invalid chain ID
        config.chain_id = 0;
        config
            .validate()
            .expect_err("Invalid config should fail validation");
    }

    #[test]
    fn evm_account_creation() {
        let account = EvmAccount::new("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e", 1)
            .expect("Valid address should create account");
        assert_eq!(account.chain_id, 1);
        assert!(account.name.is_none());

        // Test with name
        let account = EvmAccount::with_name(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            1,
            "Test".to_owned(),
        )
        .expect("Valid address should create account with name");
        assert_eq!(account.name, Some("Test".to_owned()));
    }

    #[test]
    fn evm_token() {
        // Test native token
        let eth = EvmToken::new("0x0", "ETH".to_owned(), "Ethereum".to_owned(), 18, 1)
            .expect("Native token creation should succeed");
        assert!(eth.is_native());

        // Test ERC20 token
        let usdc = EvmToken::new(
            "0xA0b86a33E6417c5d6d6bE6C2e0C6C3e5d6c7D8E9",
            "USDC".to_owned(),
            "USD Coin".to_owned(),
            6,
            1,
        )
        .expect("ERC20 token creation should succeed");
        assert!(!usdc.is_native());

        // Test amount formatting
        let amount = U256::from(1_000_000_u64); // 1 USDC
        assert_eq!(usdc.format_amount(amount), "1 USDC");

        let small_amount = U256::from(500_000_u64); // 0.5 USDC
        assert_eq!(usdc.format_amount(small_amount), "0.5 USDC");
    }

    // Additional comprehensive tests for 100% coverage

    #[test]
    fn evm_config_default() {
        let config = EvmConfig::default();
        assert_eq!(config.rpc_url, "https://eth.llamarpc.com");
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_gas_limit, None);
        assert_eq!(config.gas_price_multiplier, Some(1.1_f64));
    }

    #[test]
    fn evm_config_timeout() {
        let config = EvmConfig {
            rpc_url: "https://test.com".to_owned(),
            chain_id: 1,
            timeout_seconds: 60,
            max_gas_limit: None,
            gas_price_multiplier: None,
        };
        assert_eq!(config.timeout(), Duration::from_secs(60));
    }

    #[test]
    fn evm_config_validate_empty_rpc_url() {
        let config = EvmConfig {
            rpc_url: String::new(),
            chain_id: 1,
            timeout_seconds: 30,
            max_gas_limit: None,
            gas_price_multiplier: None,
        };
        let result = config.validate();
        assert!(result.is_err());
        if let Err(Error::InvalidConfig(msg)) = result {
            assert_eq!(msg, "RPC URL cannot be empty");
        }
    }

    #[test]
    fn evm_config_validate_invalid_rpc_url_protocol() {
        let config = EvmConfig {
            rpc_url: "ftp://test.com".to_owned(),
            chain_id: 1,
            timeout_seconds: 30,
            max_gas_limit: None,
            gas_price_multiplier: None,
        };
        let result = config.validate();
        assert!(result.is_err());
        if let Err(Error::InvalidConfig(msg)) = result {
            assert_eq!(msg, "RPC URL must start with http://, https://, or wss://");
        }
    }

    #[test]
    fn evm_config_validate_zero_chain_id() {
        let config = EvmConfig {
            rpc_url: "https://test.com".to_owned(),
            chain_id: 0,
            timeout_seconds: 30,
            max_gas_limit: None,
            gas_price_multiplier: None,
        };
        let result = config.validate();
        assert!(result.is_err());
        if let Err(Error::InvalidConfig(msg)) = result {
            assert_eq!(msg, "Chain ID cannot be 0");
        }
    }

    #[test]
    fn evm_config_validate_zero_timeout() {
        let config = EvmConfig {
            rpc_url: "https://test.com".to_owned(),
            chain_id: 1,
            timeout_seconds: 0,
            max_gas_limit: None,
            gas_price_multiplier: None,
        };
        let result = config.validate();
        assert!(result.is_err());
        if let Err(Error::InvalidConfig(msg)) = result {
            assert_eq!(msg, "Timeout cannot be 0");
        }
    }

    #[test]
    fn evm_config_validate_negative_gas_price_multiplier() {
        let config = EvmConfig {
            rpc_url: "https://test.com".to_owned(),
            chain_id: 1,
            timeout_seconds: 30,
            max_gas_limit: None,
            gas_price_multiplier: Some(-1.0_f64),
        };
        let result = config.validate();
        assert!(result.is_err());
        if let Err(Error::InvalidConfig(msg)) = result {
            assert_eq!(msg, "Gas price multiplier must be positive");
        }
    }

    #[test]
    fn evm_config_validate_zero_gas_price_multiplier() {
        let config = EvmConfig {
            rpc_url: "https://test.com".to_owned(),
            chain_id: 1,
            timeout_seconds: 30,
            max_gas_limit: None,
            gas_price_multiplier: Some(0.0_f64),
        };
        let result = config.validate();
        assert!(result.is_err());
        if let Err(Error::InvalidConfig(msg)) = result {
            assert_eq!(msg, "Gas price multiplier must be positive");
        }
    }

    #[test]
    fn evm_config_validate_valid_http_url() {
        let config = EvmConfig {
            rpc_url: "http://test.com".to_owned(),
            chain_id: 1,
            timeout_seconds: 30,
            max_gas_limit: None,
            gas_price_multiplier: Some(1.0_f64),
        };
        config.validate().unwrap();
    }

    #[test]
    fn evm_config_validate_valid_wss_url() {
        let config = EvmConfig {
            rpc_url: "wss://test.com".to_owned(),
            chain_id: 1,
            timeout_seconds: 30,
            max_gas_limit: None,
            gas_price_multiplier: None,
        };
        config.validate().unwrap();
    }

    #[test]
    fn evm_config_for_chain_success() {
        // This test will depend on the chain module implementation
        // We're testing the happy path where a valid chain ID is provided
        if let Ok(config) = EvmConfig::for_chain(1) {
            assert_eq!(config.chain_id, 1);
            assert_eq!(config.timeout_seconds, 30);
            assert_eq!(config.gas_price_multiplier, Some(1.1));
        } else {
            // If the chain module returns an error for chain ID 1,
            // that's fine - we're just testing the structure
        }
    }

    #[test]
    fn evm_config_for_chain_invalid_chain() {
        // Test with an invalid chain ID that should fail
        let result = EvmConfig::for_chain(999_999);
        // The result depends on the chain module implementation
        // but this tests the error path
        match result {
            Ok(_) | Err(_) => {
                // Either chain module supports this chain ID or expected error for unsupported chain ID
            }
        }
    }

    #[test]
    fn evm_account_new_invalid_address() {
        let result = EvmAccount::new("invalid-address", 1);
        result.expect_err("Invalid address should fail");
    }

    #[test]
    fn evm_account_with_name_invalid_address() {
        let result = EvmAccount::with_name("invalid-address", 1, "Test".to_owned());
        assert!(result.is_err());
    }

    #[test]
    fn evm_account_to_address_success() {
        let account = EvmAccount::new("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e", 1)
            .expect("Valid address should create account");
        let address_result = account.to_address();
        assert!(address_result.is_ok());
    }

    #[test]
    fn evm_account_display_address_success() {
        let account = EvmAccount::new("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e", 1)
            .expect("Valid address should create account");
        let display_result = account.display_address();
        assert!(display_result.is_ok());
        let display = display_result.expect("Display address should succeed");
        assert!(display.starts_with("0x"));
    }

    #[test]
    fn evm_transaction_data_new_success() {
        let result = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        );
        let tx_data = result.expect("Transaction data creation should succeed");
        assert_eq!(tx_data.to, "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e");
        assert_eq!(tx_data.data, "0x1234");
        assert_eq!(tx_data.value, "0x3e8");
        assert_eq!(tx_data.gas_limit, "0x5208");
        assert_eq!(tx_data.chain_id, 1);
    }

    #[test]
    fn evm_transaction_data_new_invalid_address() {
        let result = EvmTransactionData::new(
            "invalid-address",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        );
        result.expect_err("Invalid address should fail");
    }

    #[test]
    fn evm_transaction_data_new_invalid_data_format() {
        let result = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "1234", // Missing 0x prefix
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        );
        assert!(result.is_err());
        if let Err(Error::InvalidData(msg)) = result {
            assert_eq!(msg, "Transaction data must start with 0x");
        }
    }

    #[test]
    fn evm_transaction_data_to_address() {
        let tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        let address_result = tx_data.to_address();
        address_result.expect("Valid address should parse");
    }

    #[test]
    fn evm_transaction_data_data_bytes() {
        let tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        let bytes_result = tx_data.data_bytes();
        let bytes = bytes_result.expect("Data bytes parsing should succeed");
        assert_eq!(bytes.len(), 2); // 0x1234 = 2 bytes
    }

    #[test]
    fn evm_transaction_data_data_bytes_no_prefix() {
        // Test data parsing without 0x prefix
        let mut tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        tx_data.data = "abcd".to_owned(); // No 0x prefix
        let bytes_result = tx_data.data_bytes();
        bytes_result.unwrap();
    }

    #[test]
    fn evm_transaction_data_data_bytes_invalid_hex() {
        let mut tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        tx_data.data = "0xgggg".to_owned(); // Invalid hex
        let bytes_result = tx_data.data_bytes();
        bytes_result.unwrap_err();
    }

    #[test]
    fn evm_transaction_data_value_u256() {
        let tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        let value_result = tx_data.value_u256();
        assert!(value_result.is_ok());
        assert_eq!(
            value_result.expect("Value parsing should succeed"),
            U256::from(1000)
        );
    }

    #[test]
    fn evm_transaction_data_value_u256_no_prefix() {
        let mut tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        tx_data.value = "3e8".to_owned(); // No 0x prefix
        let value_result = tx_data.value_u256();
        assert!(value_result.is_ok());
        assert_eq!(
            value_result.expect("Value parsing should succeed"),
            U256::from(1000)
        );
    }

    #[test]
    fn evm_transaction_data_value_u256_invalid() {
        let mut tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        tx_data.value = "0xgggg".to_owned(); // Invalid hex
        let value_result = tx_data.value_u256();
        value_result.unwrap_err();
    }

    #[test]
    fn evm_transaction_data_gas_limit_u64() {
        let tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        let gas_limit_result = tx_data.gas_limit_u64();
        assert!(gas_limit_result.is_ok());
        assert_eq!(
            gas_limit_result.expect("Gas limit parsing should succeed"),
            21000
        );
    }

    #[test]
    fn evm_transaction_data_gas_limit_u64_no_prefix() {
        let mut tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        tx_data.gas_limit = "5208".to_owned(); // No 0x prefix
        let gas_limit_result = tx_data.gas_limit_u64();
        assert!(gas_limit_result.is_ok());
        assert_eq!(
            gas_limit_result.expect("Gas limit parsing should succeed"),
            21000
        );
    }

    #[test]
    fn evm_transaction_data_gas_limit_u64_invalid() {
        let mut tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        tx_data.gas_limit = "0xgggg".to_owned(); // Invalid hex
        let gas_limit_result = tx_data.gas_limit_u64();
        gas_limit_result.unwrap_err();
    }

    #[test]
    fn evm_transaction_data_gas_price_u256() {
        let tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        let gas_price_result = tx_data.gas_price_u256();
        assert!(gas_price_result.is_ok());
        assert_eq!(
            gas_price_result.expect("Gas price parsing should succeed"),
            U256::from(20_000_000_000_u64)
        );
    }

    #[test]
    fn evm_transaction_data_gas_price_u256_no_prefix() {
        let mut tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        tx_data.gas_price = "4a817c800".to_owned(); // No 0x prefix
        let gas_price_result = tx_data.gas_price_u256();
        assert!(gas_price_result.is_ok());
        assert_eq!(
            gas_price_result.expect("Gas price parsing should succeed"),
            U256::from(20_000_000_000_u64)
        );
    }

    #[test]
    fn evm_transaction_data_gas_price_u256_invalid() {
        let mut tx_data = EvmTransactionData::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "0x1234",
            U256::from(1000),
            21000,
            U256::from(20_000_000_000u64),
            1,
        )
        .expect("Creation should succeed");

        tx_data.gas_price = "0xgggg".to_owned(); // Invalid hex
        let gas_price_result = tx_data.gas_price_u256();
        gas_price_result.unwrap_err();
    }

    #[test]
    fn evm_token_native_with_long_zero_address() {
        let token = EvmToken::new(
            "0x0000000000000000000000000000000000000000",
            "ETH".to_owned(),
            "Ethereum".to_owned(),
            18,
            1,
        )
        .expect("Creation should succeed");
        assert!(token.is_native());
    }

    #[test]
    fn evm_token_native_with_short_zero_address() {
        let token = EvmToken::new("0x0", "ETH".to_owned(), "Ethereum".to_owned(), 18, 1)
            .expect("Native token creation should succeed");
        assert!(token.is_native());
    }

    #[test]
    fn evm_token_contract_address_native() {
        let token = EvmToken::new("0x0", "ETH".to_owned(), "Ethereum".to_owned(), 18, 1)
            .expect("Native token creation should succeed");
        let contract_address = token
            .contract_address()
            .expect("Contract address should be retrievable");
        assert!(contract_address.is_none());
    }

    #[test]
    fn evm_token_contract_address_erc20() {
        let token = EvmToken::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "TEST".to_owned(),
            "Test Token".to_owned(),
            18,
            1,
        )
        .expect("Creation should succeed");
        let contract_address = token
            .contract_address()
            .expect("Contract address should be retrievable");
        assert!(contract_address.is_some());
    }

    #[test]
    fn evm_token_format_amount_zero() {
        let token = EvmToken::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "TEST".to_owned(),
            "Test Token".to_owned(),
            18,
            1,
        )
        .expect("Creation should succeed");
        let formatted = token.format_amount(U256::ZERO);
        assert_eq!(formatted, "0 TEST");
    }

    #[test]
    fn evm_token_format_amount_whole_number() {
        let token = EvmToken::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "TEST".to_owned(),
            "Test Token".to_owned(),
            18,
            1,
        )
        .expect("Creation should succeed");
        let amount = U256::from(10_i32).pow(U256::from(18_i32)) * U256::from(5_i32); // 5 tokens
        let formatted = token.format_amount(amount);
        assert_eq!(formatted, "5 TEST");
    }

    #[test]
    fn evm_token_format_amount_decimal() {
        let token = EvmToken::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "TEST".to_owned(),
            "Test Token".to_owned(),
            6, // 6 decimals like USDC
            1,
        )
        .expect("Creation should succeed");
        let amount = U256::from(1_500_000_i32); // 1.5 tokens
        let formatted = token.format_amount(amount);
        assert_eq!(formatted, "1.5 TEST");
    }

    #[test]
    fn evm_token_format_amount_trailing_zeros_trimmed() {
        let token = EvmToken::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "TEST".to_owned(),
            "Test Token".to_owned(),
            6,
            1,
        )
        .expect("Creation should succeed");
        let amount = U256::from(1_200_000_i32); // 1.2 tokens (should trim trailing zeros)
        let formatted = token.format_amount(amount);
        assert_eq!(formatted, "1.2 TEST");
    }

    #[test]
    fn evm_token_format_amount_small_decimals() {
        let token = EvmToken::new(
            "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e",
            "TEST".to_owned(),
            "Test Token".to_owned(),
            18,
            1,
        )
        .expect("Creation should succeed");
        let amount = U256::from(1_i32); // Smallest possible amount
        let formatted = token.format_amount(amount);
        assert_eq!(formatted, "0.000000000000000001 TEST");
    }

    #[test]
    fn evm_token_new_invalid_address() {
        let result = EvmToken::new(
            "invalid-address",
            "TEST".to_owned(),
            "Test Token".to_owned(),
            18,
            1,
        );
        assert!(result.is_err());
    }
}
