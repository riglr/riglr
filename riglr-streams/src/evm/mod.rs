//! EVM blockchain streaming capabilities
//!
//! This module provides streaming components for EVM-compatible blockchains,
//! including multi-chain management and WebSocket-based event streaming.

pub mod multi_chain;
pub mod websocket;

pub use multi_chain::MultiChainEvmManager;
pub use websocket::{ChainId, EvmEventType, EvmStreamConfig, EvmStreamEvent, EvmWebSocketStream};
