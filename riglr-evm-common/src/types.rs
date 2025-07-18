//! Common EVM types shared across riglr crates
//!
//! This module provides shared type definitions and configuration structs
//! that are needed by both riglr-evm-tools and riglr-cross-chain-tools
//! to avoid duplication and circular dependencies.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use alloy::primitives::{Address, U256, Bytes};
use std::time::Duration;

/// Configuration for EVM operations shared across crates
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvmConfig {
    /// RPC endpoint URL for the EVM chain
    pub rpc_url: String,
    /// Chain ID (e.g., 1 for Ethereum, 137 for Polygon)
    pub chain_id: u64,
    /// Request timeout duration
    pub timeout_seconds: u64,
    /// Maximum gas limit for transactions
    pub max_gas_limit: Option<u64>,
    /// Default gas price multiplier (1.0 = exact, 1.1 = 10% higher)
    pub gas_price_multiplier: Option<f64>,
}

impl Default for EvmConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://eth.llamarpc.com".to_string(),
            chain_id: 1, // Ethereum mainnet
            timeout_seconds: 30,
            max_gas_limit: None,
            gas_price_multiplier: Some(1.1), // 10% buffer
        }
    }
}

impl EvmConfig {
    /// Create config for a specific chain ID
    pub fn for_chain(chain_id: u64) -> Result<Self, crate::EvmCommonError> {
        let rpc_url = crate::chain::chain_id_to_rpc_url(chain_id)?;
        
        Ok(Self {
            rpc_url,
            chain_id,
            timeout_seconds: 30,
            max_gas_limit: None,
            gas_price_multiplier: Some(1.1),
        })
    }
    
    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), crate::EvmCommonError> {
        if self.rpc_url.is_empty() {
            return Err(crate::EvmCommonError::InvalidConfig("RPC URL cannot be empty".to_string()));
        }
        
        if !self.rpc_url.starts_with("http://") 
            && !self.rpc_url.starts_with("https://") 
            && !self.rpc_url.starts_with("wss://") {
            return Err(crate::EvmCommonError::InvalidConfig(
                "RPC URL must start with http://, https://, or wss://".to_string()
            ));
        }
        
        if self.chain_id == 0 {
            return Err(crate::EvmCommonError::InvalidConfig("Chain ID cannot be 0".to_string()));
        }
        
        if self.timeout_seconds == 0 {
            return Err(crate::EvmCommonError::InvalidConfig("Timeout cannot be 0".to_string()));
        }
        
        if let Some(multiplier) = self.gas_price_multiplier {
            if multiplier <= 0.0 {
                return Err(crate::EvmCommonError::InvalidConfig("Gas price multiplier must be positive".to_string()));
            }
        }
        
        Ok(())
    }
}

/// Common EVM account metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvmAccount {
    /// Address of the account (hex format with 0x prefix)
    pub address: String,
    /// Chain ID where this account exists
    pub chain_id: u64,
    /// Optional account alias/name
    pub name: Option<String>,
}

impl EvmAccount {
    /// Create new EVM account with validation
    pub fn new(address: &str, chain_id: u64) -> Result<Self, crate::EvmCommonError> {
        // Validate address format
        crate::address::validate_evm_address(address)?;
        
        Ok(Self {
            address: address.to_string(),
            chain_id,
            name: None,
        })
    }
    
    /// Create new EVM account with name
    pub fn with_name(address: &str, chain_id: u64, name: String) -> Result<Self, crate::EvmCommonError> {
        let mut account = Self::new(address, chain_id)?;
        account.name = Some(name);
        Ok(account)
    }
    
    /// Get address as Alloy Address type
    pub fn to_address(&self) -> Result<Address, crate::EvmCommonError> {
        crate::address::parse_evm_address(&self.address)
    }
    
    /// Format address for display (checksummed)
    pub fn display_address(&self) -> Result<String, crate::EvmCommonError> {
        let addr = self.to_address()?;
        Ok(format!("0x{:x}", addr))
    }
}

/// EVM transaction data for cross-chain operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvmTransactionData {
    /// Target contract address
    pub to: String,
    /// Transaction data (hex encoded)
    pub data: String,
    /// Value to send (in wei, hex encoded)
    pub value: String,
    /// Gas limit (hex encoded)
    pub gas_limit: String,
    /// Gas price (in wei, hex encoded)  
    pub gas_price: String,
    /// Chain ID for this transaction
    pub chain_id: u64,
}

impl EvmTransactionData {
    /// Create new EVM transaction data with validation
    pub fn new(
        to: &str,
        data: &str,
        value: U256,
        gas_limit: u64,
        gas_price: U256,
        chain_id: u64,
    ) -> Result<Self, crate::EvmCommonError> {
        // Validate to address
        crate::address::validate_evm_address(to)?;
        
        // Validate data is hex
        if !data.starts_with("0x") {
            return Err(crate::EvmCommonError::InvalidData("Transaction data must start with 0x".to_string()));
        }
        
        Ok(Self {
            to: to.to_string(),
            data: data.to_string(),
            value: format!("0x{:x}", value),
            gas_limit: format!("0x{:x}", gas_limit),
            gas_price: format!("0x{:x}", gas_price),
            chain_id,
        })
    }
    
    /// Parse to address as Alloy Address
    pub fn to_address(&self) -> Result<Address, crate::EvmCommonError> {
        crate::address::parse_evm_address(&self.to)
    }
    
    /// Parse data as Alloy Bytes
    pub fn data_bytes(&self) -> Result<Bytes, crate::EvmCommonError> {
        let hex_str = self.data.strip_prefix("0x").unwrap_or(&self.data);
        hex::decode(hex_str)
            .map_err(|e| crate::EvmCommonError::InvalidData(format!("Invalid hex data: {}", e)))
            .map(Bytes::from)
    }
    
    /// Parse value as U256
    pub fn value_u256(&self) -> Result<U256, crate::EvmCommonError> {
        U256::from_str_radix(&self.value.strip_prefix("0x").unwrap_or(&self.value), 16)
            .map_err(|e| crate::EvmCommonError::InvalidData(format!("Invalid value format: {}", e)))
    }
    
    /// Parse gas limit as u64
    pub fn gas_limit_u64(&self) -> Result<u64, crate::EvmCommonError> {
        u64::from_str_radix(&self.gas_limit.strip_prefix("0x").unwrap_or(&self.gas_limit), 16)
            .map_err(|e| crate::EvmCommonError::InvalidData(format!("Invalid gas limit format: {}", e)))
    }
    
    /// Parse gas price as U256
    pub fn gas_price_u256(&self) -> Result<U256, crate::EvmCommonError> {
        U256::from_str_radix(&self.gas_price.strip_prefix("0x").unwrap_or(&self.gas_price), 16)
            .map_err(|e| crate::EvmCommonError::InvalidData(format!("Invalid gas price format: {}", e)))
    }
}

/// Token information for ERC20 and native tokens
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvmToken {
    /// Contract address (0x0 for native token)
    pub address: String,
    /// Token symbol (e.g., "USDC", "ETH")
    pub symbol: String,
    /// Token name (e.g., "USD Coin", "Ethereum")
    pub name: String,
    /// Number of decimal places
    pub decimals: u8,
    /// Chain ID where this token exists
    pub chain_id: u64,
}

impl EvmToken {
    /// Create new EVM token with validation
    pub fn new(
        address: &str,
        symbol: String,
        name: String,
        decimals: u8,
        chain_id: u64,
    ) -> Result<Self, crate::EvmCommonError> {
        // Validate address (allow 0x0 for native token)
        if address != "0x0000000000000000000000000000000000000000" && address != "0x0" {
            crate::address::validate_evm_address(address)?;
        }
        
        Ok(Self {
            address: address.to_string(),
            symbol,
            name,
            decimals,
            chain_id,
        })
    }
    
    /// Check if this is a native token (ETH, MATIC, etc.)
    pub fn is_native(&self) -> bool {
        self.address == "0x0000000000000000000000000000000000000000" 
            || self.address == "0x0"
    }
    
    /// Get contract address as Alloy Address (for non-native tokens)
    pub fn contract_address(&self) -> Result<Option<Address>, crate::EvmCommonError> {
        if self.is_native() {
            Ok(None)
        } else {
            Ok(Some(crate::address::parse_evm_address(&self.address)?))
        }
    }
    
    /// Convert amount from smallest unit to human-readable format
    pub fn format_amount(&self, raw_amount: U256) -> String {
        let divisor = U256::from(10).pow(U256::from(self.decimals));
        let whole = raw_amount / divisor;
        let remainder = raw_amount % divisor;
        
        if remainder == U256::ZERO {
            format!("{} {}", whole, self.symbol)
        } else {
            // Calculate decimal portion
            let decimal_str = format!("{:0width$}", remainder, width = self.decimals as usize);
            let decimal_trimmed = decimal_str.trim_end_matches('0');
            
            if decimal_trimmed.is_empty() {
                format!("{} {}", whole, self.symbol)
            } else {
                format!("{}.{} {}", whole, decimal_trimmed, self.symbol)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_evm_config_validation() {
        let mut config = EvmConfig::default();
        assert!(config.validate().is_ok());
        
        // Test invalid URL
        config.rpc_url = "invalid-url".to_string();
        assert!(config.validate().is_err());
        
        // Test valid URL
        config.rpc_url = "https://eth.llamarpc.com".to_string();
        assert!(config.validate().is_ok());
        
        // Test invalid chain ID
        config.chain_id = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_evm_account_creation() {
        let account = EvmAccount::new("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e", 1);
        assert!(account.is_ok());
        
        let account = account.unwrap();
        assert_eq!(account.chain_id, 1);
        assert!(account.name.is_none());
        
        // Test with name
        let account = EvmAccount::with_name("0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e", 1, "Test".to_string());
        assert!(account.is_ok());
        assert_eq!(account.unwrap().name, Some("Test".to_string()));
    }
    
    #[test]
    fn test_evm_token() {
        // Test native token
        let eth = EvmToken::new("0x0", "ETH".to_string(), "Ethereum".to_string(), 18, 1);
        assert!(eth.is_ok());
        let eth = eth.unwrap();
        assert!(eth.is_native());
        
        // Test ERC20 token
        let usdc = EvmToken::new(
            "0xA0b86a33E6417c5d6d6bE6C2e0C6C3e5d6c7D8E9", 
            "USDC".to_string(), 
            "USD Coin".to_string(), 
            6, 
            1
        );
        assert!(usdc.is_ok());
        let usdc = usdc.unwrap();
        assert!(!usdc.is_native());
        
        // Test amount formatting
        let amount = U256::from(1_000_000u64); // 1 USDC
        assert_eq!(usdc.format_amount(amount), "1 USDC");
        
        let small_amount = U256::from(500_000u64); // 0.5 USDC
        assert_eq!(usdc.format_amount(small_amount), "0.5 USDC");
    }
}