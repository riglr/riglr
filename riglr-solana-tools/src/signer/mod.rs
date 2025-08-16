//! Solana-specific signer implementations.
//!
//! This module contains signer implementations that are specific to the Solana blockchain,
//! providing concrete implementations of the `TransactionSigner` trait from `riglr-core`.

pub mod local;

pub use local::LocalSolanaSigner;
