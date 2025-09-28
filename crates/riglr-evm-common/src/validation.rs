//! EVM validation utilities

use crate::{address::validate, error::Error};
use riglr_config::{AddressValidator, ConfigError, ConfigResult};

/// EVM address validator that implements the `AddressValidator` trait
///
/// This allows EVM address validation to be used with the riglr-config
/// validation system without creating tight coupling between the config
/// and blockchain-specific crates.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct EvmAddressValidator;

impl AddressValidator for EvmAddressValidator {
    #[inline]
    fn validate(&self, address: &str, contract_name: &str) -> ConfigResult<()> {
        validate(address).map_err(|validation_error| {
            ConfigError::validation(format!(
                "Invalid {contract_name} address: {address} - {validation_error}"
            ))
        })
    }
}

/// Validate chain ID is supported
///
/// # Errors
/// Returns `Error::UnsupportedChain` if the chain ID is 0
#[inline]
pub const fn validate_chain_id(chain_id: u64) -> Result<(), Error> {
    if chain_id == 0 {
        return Err(Error::UnsupportedChain(chain_id));
    }
    Ok(())
}

/// Validate gas parameters
///
/// # Errors
/// Returns `Error::InvalidConfig` if `gas_limit` or `gas_price` is 0
#[inline]
pub fn validate_gas_params(gas_limit: u64, gas_price: u64) -> Result<(), Error> {
    if gas_limit == 0 {
        return Err(Error::InvalidConfig("Gas limit cannot be zero".to_owned()));
    }
    if gas_price == 0 {
        return Err(Error::InvalidConfig("Gas price cannot be zero".to_owned()));
    }
    Ok(())
}
