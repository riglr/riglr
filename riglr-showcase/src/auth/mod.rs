//! Authentication providers for riglr showcase
//!
//! This module contains concrete implementations of SignerFactory for various
//! authentication providers. These serve as examples of how to implement
//! custom authentication for the riglr web adapters.

pub mod privy;

#[cfg(feature = "web-server")]
pub use privy::{PrivySignerFactory, PrivyUserData};