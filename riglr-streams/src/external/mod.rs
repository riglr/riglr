//! External data source streaming capabilities
//!
//! This module provides streaming components for external data sources,
//! including cryptocurrency exchanges and blockchain infrastructure services.
//!
//! ## Supported Sources
//!
//! - **Binance**: Real-time cryptocurrency market data (tickers, order books, trades, klines)
//! - **Mempool.space**: Bitcoin blockchain data (transactions, blocks, fee estimates)

/// Binance WebSocket streaming module for real-time cryptocurrency market data
pub mod binance;
/// Mempool.space WebSocket streaming module for Bitcoin blockchain data
pub mod mempool;

pub use binance::{BinanceConfig, BinanceEventData, BinanceStream, BinanceStreamEvent};
pub use mempool::{
    BitcoinNetwork, MempoolConfig, MempoolEventData, MempoolSpaceStream, MempoolStreamEvent,
};
