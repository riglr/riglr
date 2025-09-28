use crate::evm::ChainEvent;
use crate::external::binance::BinanceStreamEvent;
use crate::external::mempool::MempoolStreamEvent;
use core::any::Any;
use riglr_events_core::prelude::Event;

/// Helper to convert Any to Event by trying all known event types
pub fn as_event(event: &(dyn Any + Send + Sync)) -> Option<&dyn Event> {
    // Try to downcast to various event types that implement Event
    if let Some(evm_event) = event.downcast_ref::<ChainEvent>() {
        return Some(evm_event);
    }
    if let Some(binance_event) = event.downcast_ref::<BinanceStreamEvent>() {
        return Some(binance_event);
    }
    if let Some(mempool_event) = event.downcast_ref::<MempoolStreamEvent>() {
        return Some(mempool_event);
    }
    // Try to downcast to GenericEvent from riglr-events-core
    if let Some(generic_event) = event.downcast_ref::<riglr_events_core::GenericEvent>() {
        return Some(generic_event);
    }

    None
}

/// Macro to simplify adding new event types
/// Usage: `register_event_types!(NewEventType1`, `NewEventType2`);
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
#[expect(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use core::any::Any;
    use riglr_events_core::prelude::*;

    // Mock event type for testing
    #[derive(Debug, Clone, serde::Serialize)]
    struct MockEvent {
        metadata: EventMetadata,
    }

    impl MockEvent {
        fn new(id: String) -> Self {
            Self {
                metadata: EventMetadata::new(
                    id,
                    EventKind::Custom("mock_event".to_string()),
                    "test".to_string(),
                ),
            }
        }
    }

    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }

        fn kind(&self) -> &EventKind {
            &self.metadata.kind
        }

        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
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

        fn to_json(&self) -> EventResult<serde_json::Value> {
            serde_json::to_value(self).map_err(|e| EventError::generic(e.to_string()))
        }
    }

    // Another mock event type for macro testing
    #[derive(Debug, Clone, serde::Serialize)]
    struct CustomEvent {
        metadata: EventMetadata,
    }

    impl CustomEvent {
        fn new(name: &str) -> Self {
            Self {
                metadata: EventMetadata::new(
                    format!("custom-{name}"),
                    EventKind::Custom("custom_event".to_string()),
                    "test".to_string(),
                ),
            }
        }
    }

    impl Event for CustomEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }

        fn kind(&self) -> &EventKind {
            &self.metadata.kind
        }

        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
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

        fn to_json(&self) -> EventResult<serde_json::Value> {
            serde_json::to_value(self).map_err(|e| EventError::generic(e.to_string()))
        }
    }

    #[test]
    fn test_as_event_when_evm_stream_event_should_return_some() {
        use crate::core::StreamMetadata;
        use crate::evm::{ChainEvent, ChainId, EventType};
        use serde_json::json;
        use std::time::SystemTime;

        let metadata = EventMetadata::new(
            "test-event".to_string(),
            EventKind::Block,
            "evm-ws-1".to_string(),
        );

        let stream_meta = StreamMetadata {
            stream_source: "evm-ws-1".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: Some(json!({"test": "data"})),
        };

        let evm_event = ChainEvent {
            metadata,
            event_type: EventType::NewBlock,
            stream_meta,
            chain_id: ChainId::Ethereum,
            block_number: Some(123),
            transaction_hash: Some("0x123".to_string()),
            data: json!({"test_data": "value"}),
        };

        let any_event: &(dyn Any + Send + Sync) = &evm_event;
        let result = as_event(any_event);

        assert!(result.is_some());
        let event_ref = result.expect("Expected as_event to return Some for ChainEvent");
        assert_eq!(event_ref.kind(), &EventKind::Block);
    }

    #[test]
    fn test_as_event_when_binance_stream_event_should_return_some() {
        use crate::core::StreamMetadata;
        use crate::external::binance::TickerData;
        use crate::external::{BinanceEventData, BinanceStreamEvent};
        use serde_json::json;
        use std::time::SystemTime;

        let metadata = EventMetadata::new(
            "test-binance".to_string(),
            EventKind::Price,
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
            event_time: 1_234_567_890,
        };

        let binance_event = BinanceStreamEvent {
            metadata,
            data: BinanceEventData::Ticker(ticker_data),
            stream_meta,
        };

        let any_event: &(dyn Any + Send + Sync) = &binance_event;
        let result = as_event(any_event);

        assert!(result.is_some());
        let event_ref = result.expect("Expected as_event to return Some for BinanceStreamEvent");
        assert_eq!(event_ref.kind(), &EventKind::Price);
    }

    #[test]
    fn test_as_event_when_mempool_stream_event_should_return_some() {
        use crate::core::StreamMetadata;
        use crate::external::mempool::MempoolEventType;
        use crate::external::{BitcoinNetwork, MempoolStreamEvent};
        use serde_json::json;
        use std::time::SystemTime;

        let metadata = EventMetadata::new(
            "test-mempool".to_string(),
            EventKind::Transaction,
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
            block_height: Some(750_000),
            transaction_count: Some(1),
        };

        let any_event: &(dyn Any + Send + Sync) = &mempool_event;
        let result = as_event(any_event);

        assert!(result.is_some());
        let event_ref = result.expect("Expected as_event to return Some for MempoolStreamEvent");
        assert_eq!(event_ref.kind(), &EventKind::Transaction);
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

        let custom_event = CustomEvent::new("test_custom");

        let any_event: &(dyn Any + Send + Sync) = &custom_event;
        let result = as_event_extended(any_event);

        assert!(result.is_some());
        let event_ref = result.expect("Expected as_event_extended to return Some for CustomEvent");
        if let EventKind::Custom(event_type) = event_ref.kind().clone() {
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
        let event_ref = result.expect("Expected as_event_extended to return Some for MockEvent");
        if let EventKind::Custom(event_type) = event_ref.kind().clone() {
            assert_eq!(event_type, "mock_event");
        } else {
            panic!("Expected Custom event kind");
        }

        // Test with CustomEvent
        let custom_event = CustomEvent::new("custom_name");

        let any_event: &(dyn Any + Send + Sync) = &custom_event;
        let result = as_event_extended(any_event);

        assert!(result.is_some());
        let event_ref = result.expect(
            "Expected as_event_extended to return Some for CustomEvent in multiple types test",
        );
        if let EventKind::Custom(event_type) = event_ref.kind().clone() {
            assert_eq!(event_type, "custom_event");
        } else {
            panic!("Expected Custom event kind");
        }
    }

    #[test]
    fn test_register_event_types_macro_fallback_to_builtin_types() {
        use crate::core::StreamMetadata;
        use crate::evm::{ChainEvent, ChainId, EventType};
        use serde_json::json;
        use std::time::SystemTime;

        // Generate the extended function
        register_event_types!(CustomEvent);

        let metadata = EventMetadata::new(
            "test-fallback".to_string(),
            EventKind::Block,
            "evm-ws-1".to_string(),
        );

        let stream_meta = StreamMetadata {
            stream_source: "evm-ws-1".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: Some(json!({"test": "fallback_test"})),
        };

        let evm_event = ChainEvent {
            metadata,
            event_type: EventType::NewBlock,
            stream_meta,
            chain_id: ChainId::Ethereum,
            block_number: Some(456),
            transaction_hash: Some("0x456".to_string()),
            data: json!({"fallback_test": "value"}),
        };

        let any_event: &(dyn Any + Send + Sync) = &evm_event;
        let result = as_event_extended(any_event);

        assert!(result.is_some());
        let event_ref = result
            .expect("Expected as_event_extended to return Some for ChainEvent in fallback test");
        assert_eq!(event_ref.kind(), &EventKind::Block);
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
