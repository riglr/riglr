//! Common EVM utilities shared across Riglr crates
//!
//! This crate provides a single source of truth for common EVM functionality
//! including address validation, chain information, type conversions, and more.
//!
//! It eliminates code duplication between riglr-core and riglr-evm-tools by
//! centralizing all shared EVM utilities in one place.

#![warn(missing_docs)]
pub mod address;
pub mod chain;
pub mod conversion;
pub mod error;
pub mod formatting;
pub mod types;
pub mod validation;

// Re-export all public types and functions at the crate root
pub use address::{
    ensure_0x_prefix, format, format_string, is_checksummed, known_addresses, parse,
    strip_0x_prefix, validate,
};
pub use chain::{
    get_address_url, get_block_explorer_url, get_info, get_supported, get_transaction_url,
    id_to_name, id_to_rpc_url, is_supported, name_to_id, Info,
};
pub use conversion::{
    eth_to_wei, gwei_to_wei, smallest_unit_to_token, token_to_smallest_unit, wei_to_eth,
    wei_to_gwei,
};
pub use formatting::{
    format_gas_price_gwei, format_token_amount, format_wei_to_eth, truncate_address,
};
pub use types::{EvmAccount, EvmConfig, EvmToken, EvmTransactionData};
pub use validation::{validate_chain_id, validate_gas_params, EvmAddressValidator};

// Re-export commonly used types
pub use error::Error;

/// Backward compatibility alias - prefer `Error` in new code
pub use error::Error as EvmCommonError;

/// Result type alias using `Error`
pub type EvmResult<T> = Result<T, Error>;
