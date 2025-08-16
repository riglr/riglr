//! Shared Solana types and utilities for RIGLR framework
//!
//! This crate provides common functionality needed by both
//! riglr-solana-tools and riglr-cross-chain-tools to break
//! circular dependencies.

pub mod types;
pub mod utils;

pub use types::*;
pub use utils::*;
