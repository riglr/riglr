//! Shared EVM types and utilities for RIGLR framework
//! 
//! This crate provides common functionality needed by both
//! riglr-evm-tools and riglr-cross-chain-tools to break
//! circular dependencies and eliminate code duplication.
//!
//! # Key Components
//! 
//! - **Types**: Shared EVM data structures and configuration
//! - **Chain Management**: Unified chain ID mapping and validation  
//! - **Address Utilities**: Common address validation and conversion
//! - **Error Types**: Standardized EVM error handling
//!
//! # Architectural Benefits
//! 
//! - ✅ Prevents circular dependencies between EVM and cross-chain tools
//! - ✅ Eliminates code duplication across crates
//! - ✅ Provides consistent EVM abstractions
//! - ✅ Creates stable foundation for EVM ecosystem

pub mod types;
pub mod chain;
pub mod address;
pub mod error;

// Re-export all public items for convenience
pub use types::*;
pub use chain::*;
pub use address::*;
pub use error::*;

/// Current version of riglr-evm-common
pub const VERSION: &str = env!("CARGO_PKG_VERSION");