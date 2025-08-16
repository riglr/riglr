//! riglr-showcase library
//!
//! This library exposes common functionality used by the riglr-showcase binary
//! and its tests.

pub mod agents;
pub mod auth;
pub mod commands;
pub mod processors;

// Re-export Config from riglr-config for backward compatibility
pub mod config {
    pub use riglr_config::*;
}
