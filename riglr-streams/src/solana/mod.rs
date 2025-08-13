//! Solana blockchain streaming capabilities via Geyser
//!
//! This module provides streaming components for Solana blockchain data
//! using the Geyser protocol for real-time account and transaction monitoring.

/// Geyser WebSocket streaming implementation for real-time Solana data
pub mod geyser;

pub use geyser::{GeyserConfig, SolanaGeyserStream, SolanaStreamEvent, TransactionEvent};
