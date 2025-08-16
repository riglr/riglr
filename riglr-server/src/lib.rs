//! riglr-server: a turnkey, configurable HTTP server for riglr agents
//!
//! This crate provides a production-ready Axum/Actix server wiring around
//! `riglr-web-adapters`, exposing standardized endpoints for:
//! - SSE streaming: POST /v1/stream
//! - Completion:   POST /v1/completion
//! - Health:       GET  /health
//! - Info:         GET  /
//!
//! It integrates the SignerFactory pattern, structured logging, and room for metrics.

pub mod server;

pub use server::{start_axum, ServerConfig};

#[cfg(feature = "actix")]
pub use server::start_actix;
