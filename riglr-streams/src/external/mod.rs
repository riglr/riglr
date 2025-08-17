//! External data source streaming capabilities
//!
//! This module provides streaming components for external data sources,
//! including cryptocurrency exchanges and blockchain infrastructure services.
//! 
//! ## Supported Sources
//! 
//! - **Binance**: Real-time cryptocurrency market data (tickers, order books, trades, klines)
//! - **Mempool.space**: Bitcoin blockchain data (transactions, blocks, fee estimates)

pub mod binance;
pub mod mempool;

pub use binance::{BinanceConfig, BinanceEventData, BinanceStream, BinanceStreamEvent};
pub use mempool::{
    BitcoinNetwork, MempoolConfig, MempoolEventData, MempoolSpaceStream, MempoolStreamEvent,
};
