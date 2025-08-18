//! EVM validation utilities

use crate::error::EvmToolError;
use alloy::primitives::Address;
use std::str::FromStr;

/// Validate EVM address format with EvmToolError
pub fn validate_address_format(address: &str) -> Result<Address, EvmToolError> {
    Address::from_str(address)
        .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid address format: {}", e)))
}

/// Validate chain ID is supported
pub fn validate_chain_id(chain_id: u64) -> Result<(), EvmToolError> {
    if chain_id == 0 {
        return Err(EvmToolError::UnsupportedChain(chain_id));
    }
    Ok(())
}

/// Validate gas parameters
pub fn validate_gas_params(gas_limit: u64, gas_price: u64) -> Result<(), EvmToolError> {
    if gas_limit == 0 {
        return Err(EvmToolError::InvalidParameter(
            "Gas limit cannot be zero".to_string(),
        ));
    }
    if gas_price == 0 {
        return Err(EvmToolError::InvalidParameter(
            "Gas price cannot be zero".to_string(),
        ));
    }
    Ok(())
}
