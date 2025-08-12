//! Common utilities and types shared across protocol parsers.
//!
//! This module provides core functionality used by all Solana event parsers including:
//! - Utility functions for parsing various data types from byte arrays
//! - Common type definitions and re-exports
//! - Shared helper functions for instruction decoding and data extraction

/// Re-exported types from the main crate
pub mod types;

/// Parsing utility functions for bytes, integers, pubkeys, and other common operations
pub mod utils;

pub use types::*;
pub use utils::*;
