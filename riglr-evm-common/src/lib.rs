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
pub use address::*;
pub use chain::*;
pub use conversion::*;
pub use formatting::*;
pub use types::*;
pub use validation::*;

// Re-export commonly used types
pub use error::EvmCommonError;

/// Result type alias using EvmCommonError
pub type EvmResult<T> = Result<T, EvmCommonError>;
