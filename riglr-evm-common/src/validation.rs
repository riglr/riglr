//! EVM validation utilities

use crate::{address::validate_evm_address, error::EvmCommonError};
use riglr_config::{AddressValidator, ConfigError, ConfigResult};

/// Validate chain ID is supported
pub fn validate_chain_id(chain_id: u64) -> Result<(), EvmCommonError> {
    if chain_id == 0 {
        return Err(EvmCommonError::UnsupportedChain(chain_id));
    }
    Ok(())
}

/// Validate gas parameters
pub fn validate_gas_params(gas_limit: u64, gas_price: u64) -> Result<(), EvmCommonError> {
    if gas_limit == 0 {
        return Err(EvmCommonError::InvalidConfig(
            "Gas limit cannot be zero".to_string(),
        ));
    }
    if gas_price == 0 {
        return Err(EvmCommonError::InvalidConfig(
            "Gas price cannot be zero".to_string(),
        ));
    }
    Ok(())
}

/// EVM address validator that implements the AddressValidator trait
///
/// This allows EVM address validation to be used with the riglr-config
/// validation system without creating tight coupling between the config
/// and blockchain-specific crates.
#[derive(Debug, Clone, Copy)]
pub struct EvmAddressValidator;

impl AddressValidator for EvmAddressValidator {
    fn validate(&self, address: &str, contract_name: &str) -> ConfigResult<()> {
        validate_evm_address(address).map_err(|e| {
            ConfigError::validation(format!(
                "Invalid {} address: {} - {}",
                contract_name, address, e
            ))
        })
    }
}
