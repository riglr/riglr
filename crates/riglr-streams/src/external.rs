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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports_are_accessible() {
        use core::any::type_name;
        // Test that all re-exported types are accessible
        // This ensures the pub use statements work correctly

        // We can't instantiate these types without their dependencies,
        // but we can verify they're in scope by referencing their type names
        let _binance_config_type = type_name::<BinanceConfig>();
        let _binance_event_data_type = type_name::<BinanceEventData>();
        let _binance_stream_type = type_name::<BinanceStream>();
        let _binance_stream_event_type = type_name::<BinanceStreamEvent>();

        let _bitcoin_network_type = type_name::<BitcoinNetwork>();
        let _mempool_config_type = type_name::<MempoolConfig>();
        let _mempool_event_data_type = type_name::<MempoolEventData>();
        let _mempool_space_stream_type = type_name::<MempoolSpaceStream>();
        let _mempool_stream_event_type = type_name::<MempoolStreamEvent>();

        // If we reach this point without compilation errors, all exports are accessible
    }

    #[test]
    fn test_binance_module_exists() {
        // Test that the binance module is properly declared
        // The module declaration should be accessible
        // Module compilation confirms this works
    }

    #[test]
    fn test_mempool_module_exists() {
        // Test that the mempool module is properly declared
        // The module declaration should be accessible
        // Module compilation confirms this works
    }

    #[test]
    fn test_all_binance_exports_have_correct_names() {
        use core::any::type_name;
        // Verify the exact type names of binance exports
        assert_eq!(
            type_name::<BinanceConfig>(),
            "riglr_streams::external::binance::BinanceConfig"
        );
        assert_eq!(
            type_name::<BinanceEventData>(),
            "riglr_streams::external::binance::BinanceEventData"
        );
        assert_eq!(
            type_name::<BinanceStream>(),
            "riglr_streams::external::binance::BinanceStream"
        );
        assert_eq!(
            type_name::<BinanceStreamEvent>(),
            "riglr_streams::external::binance::BinanceStreamEvent"
        );
    }

    #[test]
    fn test_all_mempool_exports_have_correct_names() {
        use core::any::type_name;
        // Verify the exact type names of mempool exports
        assert_eq!(
            type_name::<BitcoinNetwork>(),
            "riglr_streams::external::mempool::BitcoinNetwork"
        );
        assert_eq!(
            type_name::<MempoolConfig>(),
            "riglr_streams::external::mempool::MempoolConfig"
        );
        assert_eq!(
            type_name::<MempoolEventData>(),
            "riglr_streams::external::mempool::MempoolEventData"
        );
        assert_eq!(
            type_name::<MempoolSpaceStream>(),
            "riglr_streams::external::mempool::MempoolSpaceStream"
        );
        assert_eq!(
            type_name::<MempoolStreamEvent>(),
            "riglr_streams::external::mempool::MempoolStreamEvent"
        );
    }
}
