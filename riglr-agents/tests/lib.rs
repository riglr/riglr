//! Integration test library for riglr-agents.
//!
//! This crate contains shared test utilities and integration tests for the riglr-agents
//! multi-agent coordination system.

/// Common test utilities and fixtures shared across integration tests.
pub mod common;

/// End-to-end tests that verify complete system behavior including blockchain interactions.
pub mod e2e;

/// Integration tests that verify component interactions within the riglr-agents system.
pub mod integration;
