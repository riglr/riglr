//! Common EVM functionality
//!
//! This module contains shared EVM utilities, types, and functions
//! previously in the riglr-evm-common crate.

pub mod address;
pub mod chain;
pub mod error;
pub mod types;

// Re-export all functionality at the common module root
pub use address::*;
pub use chain::*;
pub use error::*;
pub use types::*;
