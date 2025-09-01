use riglr_events_core::prelude::Event;
use std::any::Any;

/// Helper to convert Any to Event by trying all known event types
pub fn as_event(event: &(dyn Any + Send + Sync)) -> Option<&dyn Event> {
    // Try to downcast to various event types that implement Event
    if let Some(evm_event) = event.downcast_ref::<crate::evm::EvmStreamEvent>() {
        return Some(evm_event);
    }
    if let Some(binance_event) = event.downcast_ref::<crate::external::BinanceStreamEvent>() {
        return Some(binance_event);
    }
    if let Some(mempool_event) = event.downcast_ref::<crate::external::MempoolStreamEvent>() {
        return Some(mempool_event);
    }

    None
}

/// Macro to simplify adding new event types
/// Usage: register_event_types!(NewEventType1, NewEventType2);
#[macro_export]
macro_rules! register_event_types {
    ($($event_type:ty),*) => {
        pub fn as_event_extended(event: &(dyn Any + Send + Sync)) -> Option<&dyn Event> {
            // First try the built-in types
            if let Some(event_ref) = as_event(event) {
                return Some(event_ref);
            }

            // Then try the extended types
            $(
                if let Some(typed_event) = event.downcast_ref::<$event_type>() {
                    return Some(typed_event);
                }
            )*

            None
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::prelude::Event;
    use std::any::Any;

    // Mock event type for testing
    #[derive(Debug, Clone, serde::Serialize)]
    struct MockEvent {
        metadata: riglr_events_core::EventMetadata,
    }

    impl MockEvent {
        fn new(id: String) -> Self {
            Self {
                metadata: riglr_events_core::EventMetadata::new(
                    id,
                    riglr_events_core::EventKind::Custom("mock_event".to_string()),
                    "test".to_string(),
                ),
            }
        }
    }

    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }

        fn kind(&self) -> &riglr_events_core::EventKind {
            &self.metadata.kind
        }

        fn metadata(&self) -> &riglr_events_core::EventMetadata {
            &self.metadata
        }

        fn metadata_mut(
            &mut self,
        ) -> riglr_events_core::error::EventResult<&mut riglr_events_core::EventMetadata> {
            Ok(&mut self.metadata)
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }

        fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
            serde_json::to_value(self)
                .map_err(|e| riglr_events_core::error::EventError::generic(e.to_string()))
        }
    }

    // Another mock event type for macro testing
    #[derive(Debug, Clone, serde::Serialize)]
    struct CustomEvent {
        metadata: riglr_events_core::EventMetadata,
    }

    impl CustomEvent {
        fn new(name: String) -> Self {
            Self {
                metadata: riglr_events_core::EventMetadata::new(
                    format!("custom-{}", name),
                    riglr_events_core::EventKind::Custom("custom_event".to_string()),
                    "test".to_string(),
                ),
            }
        }
    }

    impl Event for CustomEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }

        fn kind(&self) -> &riglr_events_core::EventKind {
            &self.metadata.kind
        }

        fn metadata(&self) -> &riglr_events_core::EventMetadata {
            &self.metadata
        }

        fn metadata_mut(
            &mut self,
        ) -> riglr_events_core::EventResult<&mut riglr_events_core::EventMetadata> {
            Ok(&mut self.metadata)
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }

        fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
            serde_json::to_value(self)
                .map_err(|e| riglr_events_core::error::EventError::generic(e.to_string()))
        }
    }

    #[test]
    fn test_as_event_when_evm_stream_event_should_return_some() {
        use crate::core::StreamMetadata;
        use crate::evm::EvmStreamEvent;
        use serde_json::json;
        use std::time::SystemTime;

        let metadata = riglr_events_core::EventMetadata::new(
            "test-event".to_string(),
            riglr_events_core::EventKind::Block,
            "evm-ws-1".to_string(),
        );

        let stream_meta = StreamMetadata {
            stream_source: "evm-ws-1".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: Some(json!({"test": "data"})),
        };

        let evm_event = EvmStreamEvent {
            metadata,
            event_type: crate::evm::EvmEventType::NewBlock,
            stream_meta,
            chain_id: crate::evm::ChainId::Ethereum,
            block_number: Some(123),
            transaction_hash: Some("0x123".to_string()),
            data: json!({"test_data": "value"}),
        };

        let any_event: &(dyn Any + Send + Sync) = &evm_event;
        let result = as_event(any_event);

        assert!(result.is_some());
        let event_ref = result.unwrap();
        assert_eq!(event_ref.kind(), &riglr_events_core::EventKind::Block);
    }

    #[test]
    fn test_as_event_when_binance_stream_event_should_return_some() {
        use crate::core::StreamMetadata;
        use crate::external::binance::TickerData;
        use crate::external::{BinanceEventData, BinanceStreamEvent};
        use serde_json::json;
        use std::time::SystemTime;

        let metadata = riglr_events_core::EventMetadata::new(
            "test-binance".to_string(),
            riglr_events_core::EventKind::Price,
            "binance-ws".to_string(),
        );

        let stream_meta = StreamMetadata {
            stream_source: "binance-ws".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: Some(json!({"stream": "btcusdt@ticker"})),
        };

        let ticker_data = TickerData {
            symbol: "BTCUSDT".to_string(),
            close_price: "50000.0".to_string(),
            volume: "123.45".to_string(),
            price_change_percent: "2.5".to_string(),
            event_time: 1234567890,
        };

        let binance_event = BinanceStreamEvent {
            metadata,
            data: BinanceEventData::Ticker(ticker_data),
            stream_meta,
        };

        let any_event: &(dyn Any + Send + Sync) = &binance_event;
        let result = as_event(any_event);

        assert!(result.is_some());
        let event_ref = result.unwrap();
        assert_eq!(event_ref.kind(), &riglr_events_core::EventKind::Price);
    }

    #[test]
    fn test_as_event_when_mempool_stream_event_should_return_some() {
        use crate::core::StreamMetadata;
        use crate::external::mempool::MempoolEventType;
        use crate::external::{BitcoinNetwork, MempoolStreamEvent};
        use serde_json::json;
        use std::time::SystemTime;

        let metadata = riglr_events_core::EventMetadata::new(
            "test-mempool".to_string(),
            riglr_events_core::EventKind::Transaction,
            "mempool-ws".to_string(),
        );

        let stream_meta = StreamMetadata {
            stream_source: "mempool-ws".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: Some(json!({"network": "bitcoin"})),
        };

        let mempool_event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Transaction,
            data: json!({
                "txid": "abc123",
                "size": 250,
                "fee": 1000
            }),
            stream_meta,
            network: BitcoinNetwork::Mainnet,
            block_height: Some(750000),
            transaction_count: Some(1),
        };

        let any_event: &(dyn Any + Send + Sync) = &mempool_event;
        let result = as_event(any_event);

        assert!(result.is_some());
        let event_ref = result.unwrap();
        assert_eq!(event_ref.kind(), &riglr_events_core::EventKind::Transaction);
    }

    #[test]
    fn test_as_event_when_unknown_type_should_return_none() {
        let mock_event = MockEvent::new("test_id".to_string());

        let any_event: &(dyn Any + Send + Sync) = &mock_event;
        let result = as_event(any_event);

        assert!(result.is_none());
    }

    #[test]
    fn test_as_event_when_string_type_should_return_none() {
        let string_event = "not_an_event".to_string();

        let any_event: &(dyn Any + Send + Sync) = &string_event;
        let result = as_event(any_event);

        assert!(result.is_none());
    }

    #[test]
    fn test_as_event_when_number_type_should_return_none() {
        let number_event = 42u32;

        let any_event: &(dyn Any + Send + Sync) = &number_event;
        let result = as_event(any_event);

        assert!(result.is_none());
    }

    // Test the macro functionality
    #[test]
    fn test_register_event_types_macro_with_single_type() {
        // Generate the extended function with one custom type
        register_event_types!(CustomEvent);

        let custom_event = CustomEvent::new("test_custom".to_string());

        let any_event: &(dyn Any + Send + Sync) = &custom_event;
        let result = as_event_extended(any_event);

        assert!(result.is_some());
        let event_ref = result.unwrap();
        if let riglr_events_core::EventKind::Custom(event_type) = event_ref.kind() {
            assert_eq!(event_type, "custom_event");
        } else {
            panic!("Expected Custom event kind");
        }
    }

    #[test]
    fn test_register_event_types_macro_with_multiple_types() {
        // Generate the extended function with multiple custom types
        register_event_types!(MockEvent, CustomEvent);

        // Test with MockEvent
        let mock_event = MockEvent::new("mock_id".to_string());

        let any_event: &(dyn Any + Send + Sync) = &mock_event;
        let result = as_event_extended(any_event);

        assert!(result.is_some());
        let event_ref = result.unwrap();
        if let riglr_events_core::EventKind::Custom(event_type) = event_ref.kind() {
            assert_eq!(event_type, "mock_event");
        } else {
            panic!("Expected Custom event kind");
        }

        // Test with CustomEvent
        let custom_event = CustomEvent::new("custom_name".to_string());

        let any_event: &(dyn Any + Send + Sync) = &custom_event;
        let result = as_event_extended(any_event);

        assert!(result.is_some());
        let event_ref = result.unwrap();
        if let riglr_events_core::EventKind::Custom(event_type) = event_ref.kind() {
            assert_eq!(event_type, "custom_event");
        } else {
            panic!("Expected Custom event kind");
        }
    }

    #[test]
    fn test_register_event_types_macro_fallback_to_builtin_types() {
        use crate::core::StreamMetadata;
        use crate::evm::EvmStreamEvent;
        use serde_json::json;
        use std::time::SystemTime;

        // Generate the extended function
        register_event_types!(CustomEvent);

        let metadata = riglr_events_core::EventMetadata::new(
            "test-fallback".to_string(),
            riglr_events_core::EventKind::Block,
            "evm-ws-1".to_string(),
        );

        let stream_meta = StreamMetadata {
            stream_source: "evm-ws-1".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: Some(json!({"test": "fallback_test"})),
        };

        let evm_event = EvmStreamEvent {
            metadata,
            event_type: crate::evm::EvmEventType::NewBlock,
            stream_meta,
            chain_id: crate::evm::ChainId::Ethereum,
            block_number: Some(456),
            transaction_hash: Some("0x456".to_string()),
            data: json!({"fallback_test": "value"}),
        };

        let any_event: &(dyn Any + Send + Sync) = &evm_event;
        let result = as_event_extended(any_event);

        assert!(result.is_some());
        let event_ref = result.unwrap();
        assert_eq!(event_ref.kind(), &riglr_events_core::EventKind::Block);
    }

    #[test]
    fn test_register_event_types_macro_with_unknown_type_should_return_none() {
        register_event_types!(CustomEvent);

        let string_event = "still_not_an_event".to_string();

        let any_event: &(dyn Any + Send + Sync) = &string_event;
        let result = as_event_extended(any_event);

        assert!(result.is_none());
    }

    #[test]
    fn test_register_event_types_macro_with_empty_types() {
        // Test the macro with no types (edge case)
        register_event_types!();

        let mock_event = MockEvent::new("empty_test".to_string());

        let any_event: &(dyn Any + Send + Sync) = &mock_event;
        let result = as_event_extended(any_event);

        // Should return None since MockEvent is not a built-in type
        // and no custom types were registered
        assert!(result.is_none());
    }
}
