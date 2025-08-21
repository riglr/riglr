//! EVM blockchain streaming capabilities
//!
//! This module provides streaming components for EVM-compatible blockchains,
//! including multi-chain management and WebSocket-based event streaming.

pub mod multi_chain;
pub mod websocket;

pub use multi_chain::MultiChainEvmManager;
pub use websocket::{ChainId, EvmEventType, EvmStreamConfig, EvmStreamEvent, EvmWebSocketStream};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_chain_manager_re_export_is_accessible() {
        // Test that MultiChainEvmManager is properly re-exported
        // This ensures the re-export statement is valid and accessible
        let type_name = std::any::type_name::<MultiChainEvmManager>();
        assert!(type_name.contains("MultiChainEvmManager"));
    }

    #[test]
    fn test_websocket_types_re_exports_are_accessible() {
        // Test that all websocket types are properly re-exported
        // This ensures all re-export statements are valid and accessible

        let chain_id_type = std::any::type_name::<ChainId>();
        assert!(chain_id_type.contains("ChainId"));

        let event_type_type = std::any::type_name::<EvmEventType>();
        assert!(event_type_type.contains("EvmEventType"));

        let config_type = std::any::type_name::<EvmStreamConfig>();
        assert!(config_type.contains("EvmStreamConfig"));

        let event_type = std::any::type_name::<EvmStreamEvent>();
        assert!(event_type.contains("EvmStreamEvent"));

        let stream_type = std::any::type_name::<EvmWebSocketStream>();
        assert!(stream_type.contains("EvmWebSocketStream"));
    }

    #[test]
    fn test_module_compilation() {
        // Test that the module compiles and all imports are valid
        // This test ensures that the module structure is correct
        // and all declared modules exist and can be compiled
        // If this test runs, the module compiled successfully
    }
}
