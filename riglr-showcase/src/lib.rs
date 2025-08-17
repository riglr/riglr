//! riglr-showcase library
//!
//! This library exposes common functionality used by the riglr-showcase binary
//! and its tests.

pub mod agents;
pub mod auth;
pub mod commands;
pub mod processors;

/// Configuration module providing backward compatibility.
/// 
/// Re-exports all items from riglr-config for backward compatibility.
pub mod config {
    pub use riglr_config::*;
}
