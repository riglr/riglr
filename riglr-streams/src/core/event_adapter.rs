//! Event adapter module - Legacy adapter functionality removed
//!
//! All events now implement the Event trait from riglr-events-core directly.
//! The UnifiedEvent trait has been deprecated and removed.

/// Placeholder trait for backward compatibility
pub trait EventConversion {
    /// Legacy conversion functionality removed
    fn to_event(self: Box<Self>) -> Box<dyn riglr_events_core::Event>;
}

// All events now implement the Event trait directly

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::{
        types::{EventKind, EventMetadata},
        Event,
    };
    use std::any::Any;

    // Mock implementation of the Event trait for testing
    #[derive(Debug, Clone)]
    struct MockEvent {
        metadata: EventMetadata,
        data: String,
    }

    impl MockEvent {
        fn new(id: String, data: String) -> Self {
            Self {
                metadata: EventMetadata::new(
                    id,
                    EventKind::Custom("mock_event".to_string()),
                    "test_source".to_string(),
                ),
                data,
            }
        }
    }

    impl riglr_events_core::Event for MockEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }

        fn kind(&self) -> &EventKind {
            &self.metadata.kind
        }

        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> &mut EventMetadata {
            &mut self.metadata
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn clone_boxed(&self) -> Box<dyn riglr_events_core::Event> {
            Box::new(self.clone())
        }
    }

    // Mock implementation of EventConversion trait for testing
    struct MockEventConverter {
        should_panic: bool,
        event_data: String,
    }

    impl EventConversion for MockEventConverter {
        fn to_event(self: Box<Self>) -> Box<dyn riglr_events_core::Event> {
            if self.should_panic {
                panic!("Simulated conversion failure");
            }

            Box::new(MockEvent::new(
                "converted_event".to_string(),
                self.event_data,
            ))
        }
    }

    // Another mock implementation with different behavior
    struct AnotherMockConverter {
        id: u32,
    }

    impl EventConversion for AnotherMockConverter {
        fn to_event(self: Box<Self>) -> Box<dyn riglr_events_core::Event> {
            Box::new(MockEvent::new(
                format!("event_{}", self.id),
                "another_mock_data".to_string(),
            ))
        }
    }

    // Empty converter for edge case testing
    struct EmptyConverter;

    impl EventConversion for EmptyConverter {
        fn to_event(self: Box<Self>) -> Box<dyn riglr_events_core::Event> {
            Box::new(MockEvent::new(String::from(""), String::from("")))
        }
    }

    #[test]
    fn test_event_conversion_trait_when_valid_converter_should_return_event() {
        // Happy Path: Test basic conversion functionality
        let converter = MockEventConverter {
            should_panic: false,
            event_data: "test_data".to_string(),
        };

        let boxed_converter: Box<dyn EventConversion> = Box::new(converter);
        let event = boxed_converter.to_event();

        assert_eq!(event.kind().to_string(), "mock_event");
        assert!(event
            .timestamp()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .is_ok());
        assert_eq!(event.source(), "test_source");
    }

    #[test]
    fn test_event_conversion_trait_when_another_converter_should_return_different_event() {
        // Test with different converter implementation
        let converter = AnotherMockConverter { id: 42 };
        let boxed_converter: Box<dyn EventConversion> = Box::new(converter);
        let event = boxed_converter.to_event();

        assert_eq!(event.kind().to_string(), "mock_event");
        assert_eq!(event.source(), "test_source");

        // Test that the event can be downcast to access specific data
        let any_event = event.as_any();
        if let Some(mock_event) = any_event.downcast_ref::<MockEvent>() {
            assert_eq!(mock_event.id(), "event_42");
            assert_eq!(mock_event.data, "another_mock_data");
        } else {
            panic!("Failed to downcast event");
        }
    }

    #[test]
    fn test_event_conversion_trait_when_empty_converter_should_return_empty_event() {
        // Edge Case: Test with empty data
        let converter = EmptyConverter;
        let boxed_converter: Box<dyn EventConversion> = Box::new(converter);
        let event = boxed_converter.to_event();

        assert_eq!(event.kind().to_string(), "mock_event");

        // Verify empty data handling
        let any_event = event.as_any();
        if let Some(mock_event) = any_event.downcast_ref::<MockEvent>() {
            assert_eq!(mock_event.id(), "");
            assert_eq!(mock_event.data, "");
        } else {
            panic!("Failed to downcast event");
        }
    }

    #[test]
    fn test_event_conversion_trait_when_zero_id_should_return_event_0() {
        // Edge Case: Test with zero value
        let converter = AnotherMockConverter { id: 0 };
        let boxed_converter: Box<dyn EventConversion> = Box::new(converter);
        let event = boxed_converter.to_event();

        let any_event = event.as_any();
        if let Some(mock_event) = any_event.downcast_ref::<MockEvent>() {
            assert_eq!(mock_event.id(), "event_0");
        }
    }

    #[test]
    fn test_event_conversion_trait_when_max_id_should_return_event_with_max_value() {
        // Edge Case: Test with maximum value
        let converter = AnotherMockConverter { id: u32::MAX };
        let boxed_converter: Box<dyn EventConversion> = Box::new(converter);
        let event = boxed_converter.to_event();

        let any_event = event.as_any();
        if let Some(mock_event) = any_event.downcast_ref::<MockEvent>() {
            assert_eq!(mock_event.id(), format!("event_{}", u32::MAX));
        }
    }

    #[test]
    #[should_panic(expected = "Simulated conversion failure")]
    fn test_event_conversion_trait_when_converter_panics_should_propagate_panic() {
        // Error Path: Test panic handling in conversion
        let converter = MockEventConverter {
            should_panic: true,
            event_data: "irrelevant".to_string(),
        };

        let boxed_converter: Box<dyn EventConversion> = Box::new(converter);
        let _event = boxed_converter.to_event(); // This should panic
    }

    #[test]
    fn test_event_conversion_trait_when_special_characters_should_handle_correctly() {
        // Edge Case: Test with special characters
        let converter = MockEventConverter {
            should_panic: false,
            event_data: "data_with_🚀_emoji_and_special_chars_!@#$%^&*()".to_string(),
        };

        let boxed_converter: Box<dyn EventConversion> = Box::new(converter);
        let event = boxed_converter.to_event();

        let any_event = event.as_any();
        if let Some(mock_event) = any_event.downcast_ref::<MockEvent>() {
            assert_eq!(
                mock_event.data,
                "data_with_🚀_emoji_and_special_chars_!@#$%^&*()"
            );
        }
    }

    #[test]
    fn test_event_conversion_trait_when_very_long_string_should_handle_correctly() {
        // Edge Case: Test with very long string
        let long_data = "a".repeat(10000);
        let converter = MockEventConverter {
            should_panic: false,
            event_data: long_data.clone(),
        };

        let boxed_converter: Box<dyn EventConversion> = Box::new(converter);
        let event = boxed_converter.to_event();

        let any_event = event.as_any();
        if let Some(mock_event) = any_event.downcast_ref::<MockEvent>() {
            assert_eq!(mock_event.data, long_data);
            assert_eq!(mock_event.data.len(), 10000);
        }
    }

    #[test]
    fn test_event_conversion_trait_multiple_conversions_should_work_independently() {
        // Test multiple conversions to ensure statelessness
        let converter1 = MockEventConverter {
            should_panic: false,
            event_data: "first_event".to_string(),
        };

        let converter2 = MockEventConverter {
            should_panic: false,
            event_data: "second_event".to_string(),
        };

        let event1 = Box::new(converter1).to_event();
        let event2 = Box::new(converter2).to_event();

        // Verify both events are created independently
        assert_eq!(event1.kind().to_string(), "mock_event");
        assert_eq!(event2.kind().to_string(), "mock_event");

        let any_event1 = event1.as_any();
        let any_event2 = event2.as_any();

        if let (Some(mock1), Some(mock2)) = (
            any_event1.downcast_ref::<MockEvent>(),
            any_event2.downcast_ref::<MockEvent>(),
        ) {
            assert_eq!(mock1.data, "first_event");
            assert_eq!(mock2.data, "second_event");
            assert_ne!(mock1.data, mock2.data);
        }
    }

    #[test]
    fn test_event_conversion_trait_clone_box_functionality() {
        // Test that the converted event can be cloned
        let converter = MockEventConverter {
            should_panic: false,
            event_data: "cloneable_data".to_string(),
        };

        let event = Box::new(converter).to_event();
        let cloned_event = event.clone_boxed();

        assert_eq!(event.kind().to_string(), cloned_event.kind().to_string());
        assert_eq!(event.timestamp(), cloned_event.timestamp());
        assert_eq!(event.source(), cloned_event.source());
    }
}
