//! Core traits for event processing.

use crate::error::{EventError, EventResult};
use crate::types::{EventKind, EventMetadata, StreamInfo};
use async_trait::async_trait;
use core::any::Any;
use core::fmt::Debug;
use core::pin::Pin;
use core::time::Duration;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Core event trait that all events must implement.
///
/// This trait provides the minimal interface that all events share,
/// regardless of their source or specific data format.
pub trait Event: Debug + Send + Sync {
    /// Convert event to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Convert event to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Clone the event as a boxed trait object
    fn clone_boxed(&self) -> Box<dyn Event>;

    /// Get the unique event identifier
    fn id(&self) -> &str;

    /// Get the event kind/classification
    fn kind(&self) -> &EventKind;

    /// Check if this event matches a given filter criteria
    #[inline]
    fn matches_filter(&self, filter: &dyn EventFilter) -> bool
    where
        Self: Sized,
    {
        filter.matches(self)
    }

    /// Get event metadata
    fn metadata(&self) -> &EventMetadata;

    /// Get mutable access to event metadata
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata cannot be accessed mutably.
    fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata>;

    /// Get the source that generated this event
    #[inline]
    fn source(&self) -> &str {
        &self.metadata().source
    }

    /// Get the event timestamp
    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata().timestamp.into()
    }

    /// Serialize the event to JSON
    ///
    /// Events should implement `serde::Serialize` and use this method to provide
    /// a standardized JSON representation.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    fn to_json(&self) -> EventResult<serde_json::Value>;
}

// Implement Clone for Box<dyn Event>
impl Clone for Box<dyn Event> {
    #[inline]
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

/// Parser metadata and capability information
#[derive(Debug, Clone)]
pub struct ParserInfo {
    /// Parser name/identifier
    pub name: String,
    /// Input data formats this parser supports
    pub supported_formats: Vec<String>,
    /// Event kinds this parser can produce
    pub supported_kinds: Vec<EventKind>,
    /// Parser version
    pub version: String,
}

impl ParserInfo {
    /// Create new parser info
    #[must_use]
    #[inline]
    pub const fn new(name: String, version: String) -> Self {
        Self {
            name,
            supported_formats: Vec::new(),
            supported_kinds: Vec::new(),
            version,
        }
    }

    /// Add supported input format
    #[must_use]
    #[inline]
    pub fn with_format(mut self, format: String) -> Self {
        self.supported_formats.push(format);
        self
    }

    /// Add supported event kind
    #[must_use]
    #[inline]
    pub fn with_kind(mut self, kind: EventKind) -> Self {
        self.supported_kinds.push(kind);
        self
    }
}

/// Stream trait for producing events asynchronously.
///
/// Event streams are the primary way to receive events from various
/// sources like websockets, message queues, or blockchain nodes.
#[async_trait]
pub trait EventStream: Send + Sync {
    /// Get stream information and health metrics
    fn info(&self) -> &StreamInfo;

    /// Get mutable stream information
    fn info_mut(&mut self) -> &mut StreamInfo;

    /// Check if the stream is currently active
    fn is_active(&self) -> bool;

    /// Restart the stream (stop then start)
    #[inline]
    async fn restart(
        &mut self,
    ) -> EventResult<Pin<Box<dyn Stream<Item = EventResult<Box<dyn Event>>> + Send>>> {
        if self.is_active() {
            self.stop().await?;
        }
        return self.start().await;
    }

    /// Start the stream and return a stream of events
    async fn start(
        &mut self,
    ) -> EventResult<Pin<Box<dyn Stream<Item = EventResult<Box<dyn Event>>> + Send>>>;

    /// Stop the stream
    async fn stop(&mut self) -> EventResult<()>;
}

/// Filter trait for event routing and selection.
///
/// Filters are used to determine which events should be processed
/// by specific handlers or forwarded to particular destinations.
pub trait EventFilter: Send + Sync + Debug {
    /// Get a description of what this filter does
    fn description(&self) -> String;

    /// Check if an event matches this filter
    fn matches(&self, event: &dyn Event) -> bool;
}

/// Handler trait for processing events.
///
/// Handlers contain the business logic for responding to specific
/// types of events, such as updating databases, sending notifications,
/// or triggering other actions.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Check if this handler can process the given event
    fn can_handle(&self, event: &dyn Event) -> bool;

    /// Handle an event asynchronously
    async fn handle(&self, event: Box<dyn Event>) -> EventResult<()>;

    /// Get handler information
    fn info(&self) -> HandlerInfo;

    /// Initialize the handler (called before first use)
    #[inline]
    async fn initialize(&mut self) -> EventResult<()> {
        return Ok(());
    }

    /// Shutdown the handler (cleanup resources)
    #[inline]
    async fn shutdown(&mut self) -> EventResult<()> {
        return Ok(());
    }
}

/// Information about an event handler
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandlerInfo {
    /// Event kinds this handler processes
    pub handled_kinds: Vec<EventKind>,
    /// Handler name/identifier
    pub name: String,
    /// Maximum processing time before timeout
    pub timeout: Option<Duration>,
    /// Handler version
    pub version: String,
}

impl HandlerInfo {
    /// Create new handler info
    #[must_use]
    #[inline]
    pub const fn new(name: String, version: String) -> Self {
        Self {
            handled_kinds: Vec::new(),
            name,
            timeout: None,
            version,
        }
    }

    /// Add handled event kind
    #[must_use]
    #[inline]
    pub fn with_kind(mut self, kind: EventKind) -> Self {
        self.handled_kinds.push(kind);
        self
    }

    /// Set timeout duration
    #[must_use]
    #[inline]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Batch processor for handling multiple events efficiently.
#[async_trait]
pub trait EventBatchProcessor: Send + Sync {
    /// Get maximum time to wait for batch to fill
    #[inline]
    fn batch_timeout(&self) -> Duration {
        Duration::from_millis(100)
    }

    /// Get the optimal batch size for this processor
    #[inline]
    fn optimal_batch_size(&self) -> usize {
        10
    }

    /// Process a batch of events
    async fn process_batch(&self, events: Vec<Box<dyn Event>>)
        -> EventResult<Vec<EventResult<()>>>;
}

/// Event transformation trait for converting events between formats.
pub trait EventTransformer: Send + Sync {
    /// Check if this transformer can process the given event
    fn can_transform(&self, event: &dyn Event) -> bool;

    /// Transform an event into a different format
    ///
    /// # Errors
    ///
    /// Returns an error if transformation fails.
    fn transform(&self, event: Box<dyn Event>) -> EventResult<Box<dyn Event>>;
}

/// Event parsing trait for converting raw data into events.
#[async_trait]
pub trait EventParser: Send + Sync {
    /// Input data type this parser accepts
    type Input;

    /// Check if this parser can process the given input
    fn can_parse(&self, input: &Self::Input) -> bool;

    /// Get parser information
    fn info(&self) -> &ParserInfo;

    /// Parse input data into events
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>>;
}

/// Event routing trait for directing events to appropriate handlers.
#[async_trait]
pub trait EventRouter: Send + Sync {
    /// Add a handler to the routing table
    fn add_handler(&mut self, filter: Box<dyn EventFilter>, handler: Box<dyn EventHandler>);

    /// Remove handlers matching a filter
    fn remove_handlers(&mut self, filter: Box<dyn EventFilter>) -> usize;

    /// Route an event to appropriate handlers
    async fn route(&self, event: Box<dyn Event>) -> EventResult<Vec<Box<dyn EventHandler>>>;
}

/// Metrics collector trait for gathering event processing statistics.
pub trait EventMetrics: Send + Sync {
    /// Record an error during event processing
    fn record_error(&self, event: &dyn Event, error: &EventError);

    /// Record an event being processed
    fn record_event(&self, event: &dyn Event, processing_time: Duration);

    /// Reset all metrics
    fn reset(&self);

    /// Get current metrics snapshot
    ///
    /// # Errors
    ///
    /// Returns an error if snapshot creation fails.
    fn snapshot(&self) -> EventResult<serde_json::Value>;
}

// Common filter implementations

/// Filter that matches events by kind
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct KindFilter {
    /// Event kinds to match
    pub kinds: Vec<EventKind>,
}

impl KindFilter {
    /// Create a new kind filter
    #[must_use]
    #[inline]
    pub const fn new(kinds: Vec<EventKind>) -> Self {
        Self { kinds }
    }

    /// Create a filter for a single kind
    #[must_use]
    #[inline]
    pub fn single(kind: EventKind) -> Self {
        Self { kinds: vec![kind] }
    }
}

impl EventFilter for KindFilter {
    #[inline]
    fn description(&self) -> String {
        format!("Filter events by kind: {:?}", self.kinds)
    }

    #[inline]
    fn matches(&self, event: &dyn Event) -> bool {
        self.kinds.contains(event.kind())
    }
}

/// Filter that matches events by source
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SourceFilter {
    /// Sources to match (exact match)
    pub sources: Vec<String>,
}

impl SourceFilter {
    /// Create a new source filter
    #[must_use]
    #[inline]
    pub const fn new(sources: Vec<String>) -> Self {
        Self { sources }
    }

    /// Create a filter for a single source
    #[must_use]
    #[inline]
    pub fn single(source: String) -> Self {
        Self {
            sources: vec![source],
        }
    }
}

impl EventFilter for SourceFilter {
    #[inline]
    fn description(&self) -> String {
        format!("Filter events by source: {:?}", self.sources)
    }

    #[inline]
    fn matches(&self, event: &dyn Event) -> bool {
        self.sources.contains(&event.source().to_owned())
    }
}

/// Composite filter that combines multiple filters with AND logic
#[non_exhaustive]
#[derive(Debug)]
pub struct AndFilter {
    /// Filters to combine
    pub filters: Vec<Box<dyn EventFilter>>,
}

impl AndFilter {
    /// Create a new AND filter
    #[must_use]
    #[inline]
    pub fn new(filters: Vec<Box<dyn EventFilter>>) -> Self {
        Self { filters }
    }
}

impl EventFilter for AndFilter {
    #[inline]
    fn description(&self) -> String {
        let descriptions: Vec<String> = self.filters.iter().map(|f| f.description()).collect();
        format!("AND({})", descriptions.join(", "))
    }

    #[inline]
    fn matches(&self, event: &dyn Event) -> bool {
        self.filters.iter().all(|f| f.matches(event))
    }
}

/// Composite filter that combines multiple filters with OR logic
#[non_exhaustive]
#[derive(Debug)]
pub struct OrFilter {
    /// Filters to combine
    pub filters: Vec<Box<dyn EventFilter>>,
}

impl OrFilter {
    /// Create a new OR filter
    #[must_use]
    #[inline]
    pub fn new(filters: Vec<Box<dyn EventFilter>>) -> Self {
        Self { filters }
    }
}

impl EventFilter for OrFilter {
    #[inline]
    fn description(&self) -> String {
        let descriptions: Vec<String> = self.filters.iter().map(|f| f.description()).collect();
        format!("OR({})", descriptions.join(", "))
    }

    #[inline]
    fn matches(&self, event: &dyn Event) -> bool {
        self.filters.iter().any(|f| f.matches(event))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GenericEvent;
    use serde_json::json;

    // Mock implementations for testing
    #[expect(dead_code)]
    struct MockHandler {
        info: HandlerInfo,
    }

    #[async_trait]
    impl EventHandler for MockHandler {
        fn can_handle(&self, event: &dyn Event) -> bool {
            self.info.handled_kinds.contains(event.kind())
        }

        async fn handle(&self, _event: Box<dyn Event>) -> EventResult<()> {
            return Ok(());
        }

        fn info(&self) -> HandlerInfo {
            self.info.clone()
        }
    }

    #[tokio::test]
    async fn kind_filter() {
        let filter = KindFilter::single(EventKind::Transaction);

        let event = GenericEvent::new("test".to_owned(), EventKind::Transaction, json!({}));

        assert!(filter.matches(&event));

        let event2 = GenericEvent::new("test2".to_owned(), EventKind::Block, json!({}));

        assert!(!filter.matches(&event2));
    }

    #[tokio::test]
    async fn source_filter() {
        let filter = SourceFilter::single("test-source".to_owned());

        let mut event = GenericEvent::new("test".to_owned(), EventKind::Transaction, json!({}));
        event.metadata.source = "test-source".to_owned();

        assert!(filter.matches(&event));

        event.metadata.source = "other-source".to_owned();
        assert!(!filter.matches(&event));
    }

    #[tokio::test]
    async fn and_filter() {
        let kind_filter = Box::new(KindFilter::single(EventKind::Transaction));
        let source_filter = Box::new(SourceFilter::single("test-source".to_owned()));
        let and_filter = AndFilter::new(vec![kind_filter, source_filter]);

        let mut event = GenericEvent::new("test".to_owned(), EventKind::Transaction, json!({}));
        event.metadata.source = "test-source".to_owned();

        assert!(and_filter.matches(&event));

        // Change kind - should not match
        let mut event2 = event.clone();
        event2.metadata.kind = EventKind::Block;
        assert!(!and_filter.matches(&event2));

        // Change source - should not match
        let mut event3 = event;
        event3.metadata.source = "other-source".to_owned();
        assert!(!and_filter.matches(&event3));
    }

    #[tokio::test]
    async fn or_filter() {
        let kind_filter = Box::new(KindFilter::single(EventKind::Transaction));
        let source_filter = Box::new(SourceFilter::single("test-source".to_owned()));
        let or_filter = OrFilter::new(vec![kind_filter, source_filter]);

        // Event matches kind but not source
        let event1 = GenericEvent::new("test1".to_owned(), EventKind::Transaction, json!({}));
        assert!(or_filter.matches(&event1));

        // Event matches source but not kind
        let mut event2 = GenericEvent::new("test2".to_owned(), EventKind::Block, json!({}));
        event2.metadata.source = "test-source".to_owned();
        assert!(or_filter.matches(&event2));

        // Event matches neither
        let event3 = GenericEvent::new("test3".to_owned(), EventKind::Block, json!({}));
        assert!(!or_filter.matches(&event3));
    }

    #[tokio::test]
    async fn handler_info() {
        let info = HandlerInfo::new("test-handler".to_owned(), "1.0.0".to_owned())
            .with_kind(EventKind::Transaction)
            .with_kind(EventKind::Block)
            .with_timeout(Duration::from_secs(30));

        assert_eq!(info.name, "test-handler");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.handled_kinds.len(), 2);
        assert_eq!(info.timeout, Some(Duration::from_secs(30)));
    }

    #[tokio::test]
    async fn parser_info() {
        let info = ParserInfo::new("test-parser".to_owned(), "1.0.0".to_owned())
            .with_kind(EventKind::Transaction)
            .with_format("json".to_owned())
            .with_format("binary".to_owned());

        assert_eq!(info.name, "test-parser");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.supported_kinds.len(), 1);
        assert_eq!(info.supported_formats.len(), 2);
    }

    #[test]
    fn generic_event_trait_implementation() {
        let event = GenericEvent::new(
            "test-event".to_owned(),
            EventKind::Swap,
            json!({"amount": 100_i32}),
        );

        assert_eq!(event.id(), "test-event");
        assert_eq!(event.kind(), &EventKind::Swap);
        assert_eq!(event.source(), "generic");

        // Test clone
        let cloned = event.clone_boxed();
        assert_eq!(cloned.id(), event.id());
    }

    // Additional comprehensive tests for 100% coverage

    #[test]
    fn parser_info_new() {
        let info = ParserInfo::new("test-parser".to_owned(), "2.0.0".to_owned());

        assert_eq!(info.name, "test-parser");
        assert_eq!(info.version, "2.0.0");
        assert!(info.supported_kinds.is_empty());
        assert!(info.supported_formats.is_empty());
    }

    #[test]
    fn parser_info_with_multiple_kinds() {
        let info = ParserInfo::new("parser".to_owned(), "1.0".to_owned())
            .with_kind(EventKind::Transaction)
            .with_kind(EventKind::Block)
            .with_kind(EventKind::Swap);

        assert_eq!(info.supported_kinds.len(), 3);
        assert!(info.supported_kinds.contains(&EventKind::Transaction));
        assert!(info.supported_kinds.contains(&EventKind::Block));
        assert!(info.supported_kinds.contains(&EventKind::Swap));
    }

    #[test]
    fn parser_info_with_multiple_formats() {
        let info = ParserInfo::new("parser".to_owned(), "1.0".to_owned())
            .with_format("json".to_owned())
            .with_format("xml".to_owned())
            .with_format("protobuf".to_owned());

        assert_eq!(info.supported_formats.len(), 3);
        assert!(info.supported_formats.contains(&"json".to_owned()));
        assert!(info.supported_formats.contains(&"xml".to_owned()));
        assert!(info.supported_formats.contains(&"protobuf".to_owned()));
    }

    #[test]
    fn handler_info_new() {
        let info = HandlerInfo::new("test-handler".to_owned(), "3.0.0".to_owned());

        assert_eq!(info.name, "test-handler");
        assert_eq!(info.version, "3.0.0");
        assert!(info.handled_kinds.is_empty());
        assert_eq!(info.timeout, None);
    }

    #[test]
    fn handler_info_with_multiple_kinds() {
        let info = HandlerInfo::new("handler".to_owned(), "1.0".to_owned())
            .with_kind(EventKind::Transaction)
            .with_kind(EventKind::Block);

        assert_eq!(info.handled_kinds.len(), 2);
        assert!(info.handled_kinds.contains(&EventKind::Transaction));
        assert!(info.handled_kinds.contains(&EventKind::Block));
    }

    #[test]
    fn handler_info_with_timeout() {
        let timeout_duration = Duration::from_secs(60);
        let info =
            HandlerInfo::new("handler".to_owned(), "1.0".to_owned()).with_timeout(timeout_duration);

        assert_eq!(info.timeout, Some(timeout_duration));
    }

    #[test]
    fn kind_filter_new() {
        let kinds = vec![EventKind::Transaction, EventKind::Block];
        let filter = KindFilter::new(kinds.clone());

        assert_eq!(filter.kinds, kinds);
    }

    #[test]
    fn kind_filter_multiple_kinds() {
        let filter = KindFilter::new(vec![
            EventKind::Transaction,
            EventKind::Block,
            EventKind::Swap,
        ]);

        let tx_event = GenericEvent::new("tx".to_owned(), EventKind::Transaction, json!({}));
        let block_event = GenericEvent::new("block".to_owned(), EventKind::Block, json!({}));
        let swap_event = GenericEvent::new("swap".to_owned(), EventKind::Swap, json!({}));
        let other_event = GenericEvent::new(
            "other".to_owned(),
            EventKind::Custom("test".to_owned()),
            json!({}),
        );

        assert!(filter.matches(&tx_event));
        assert!(filter.matches(&block_event));
        assert!(filter.matches(&swap_event));
        assert!(!filter.matches(&other_event));
    }

    #[test]
    fn kind_filter_description() {
        let filter = KindFilter::new(vec![EventKind::Transaction, EventKind::Block]);
        let description = filter.description();

        assert!(description.contains("Filter events by kind"));
        assert!(description.contains("Transaction"));
        assert!(description.contains("Block"));
    }

    #[test]
    fn source_filter_new() {
        let sources = vec!["source1".to_owned(), "source2".to_owned()];
        let filter = SourceFilter::new(sources.clone());

        assert_eq!(filter.sources, sources);
    }

    #[test]
    fn source_filter_multiple_sources() {
        let filter = SourceFilter::new(vec!["source1".to_owned(), "source2".to_owned()]);

        let mut event1 = GenericEvent::new("test1".to_owned(), EventKind::Transaction, json!({}));
        event1.metadata.source = "source1".to_owned();

        let mut event2 = GenericEvent::new("test2".to_owned(), EventKind::Transaction, json!({}));
        event2.metadata.source = "source2".to_owned();

        let mut event3 = GenericEvent::new("test3".to_owned(), EventKind::Transaction, json!({}));
        event3.metadata.source = "source3".to_owned();

        assert!(filter.matches(&event1));
        assert!(filter.matches(&event2));
        assert!(!filter.matches(&event3));
    }

    #[test]
    fn source_filter_description() {
        let filter = SourceFilter::new(vec!["test-source".to_owned()]);
        let description = filter.description();

        assert!(description.contains("Filter events by source"));
        assert!(description.contains("test-source"));
    }

    #[test]
    fn and_filter_empty_filters() {
        let and_filter = AndFilter::new(vec![]);
        let event = GenericEvent::new("test".to_owned(), EventKind::Transaction, json!({}));

        // Empty AND filter should return true (vacuous truth)
        assert!(and_filter.matches(&event));
    }

    #[test]
    fn and_filter_description() {
        let kind_filter = Box::new(KindFilter::single(EventKind::Transaction));
        let source_filter = Box::new(SourceFilter::single("test".to_owned()));
        let and_filter = AndFilter::new(vec![kind_filter, source_filter]);

        let description = and_filter.description();
        assert!(description.contains("AND("));
        assert!(description.contains("Filter events by kind"));
        assert!(description.contains("Filter events by source"));
    }

    #[test]
    fn or_filter_empty_filters() {
        let or_filter = OrFilter::new(vec![]);
        let event = GenericEvent::new("test".to_owned(), EventKind::Transaction, json!({}));

        // Empty OR filter should return false
        assert!(!or_filter.matches(&event));
    }

    #[test]
    fn or_filter_description() {
        let kind_filter = Box::new(KindFilter::single(EventKind::Transaction));
        let source_filter = Box::new(SourceFilter::single("test".to_owned()));
        let or_filter = OrFilter::new(vec![kind_filter, source_filter]);

        let description = or_filter.description();
        assert!(description.contains("OR("));
        assert!(description.contains("Filter events by kind"));
        assert!(description.contains("Filter events by source"));
    }

    #[test]
    fn event_timestamp() {
        let event = GenericEvent::new("test".to_owned(), EventKind::Transaction, json!({}));
        let timestamp = event.timestamp();

        // Should be a valid SystemTime
        assert!(timestamp.duration_since(SystemTime::UNIX_EPOCH).is_ok());
    }

    #[test]
    fn event_matches_filter() {
        let event = GenericEvent::new("test".to_owned(), EventKind::Transaction, json!({}));
        let filter = KindFilter::single(EventKind::Transaction);

        assert!(event.matches_filter(&filter));

        let other_filter = KindFilter::single(EventKind::Block);
        assert!(!event.matches_filter(&other_filter));
    }

    // Test struct for EventBatchProcessor
    struct MockBatchProcessor;

    #[test]
    fn box_event_clone() {
        let event = GenericEvent::new("test".to_owned(), EventKind::Transaction, json!({}));
        let boxed: Box<dyn Event> = Box::new(event);
        let cloned = boxed.clone();

        assert_eq!(boxed.id(), cloned.id());
        assert_eq!(boxed.kind(), cloned.kind());
    }

    #[async_trait]
    impl EventBatchProcessor for MockBatchProcessor {
        async fn process_batch(
            &self,
            _events: Vec<Box<dyn Event>>,
        ) -> EventResult<Vec<EventResult<()>>> {
            return Ok(vec![]);
        }
    }

    // Test struct for EventHandler default methods
    struct MockHandlerForDefaults {
        info: HandlerInfo,
    }

    #[async_trait]
    impl EventHandler for MockHandlerForDefaults {
        fn can_handle(&self, _event: &dyn Event) -> bool {
            true
        }

        async fn handle(&self, _event: Box<dyn Event>) -> EventResult<()> {
            return Ok(());
        }

        fn info(&self) -> HandlerInfo {
            self.info.clone()
        }
    }

    // Test struct for EventStream restart method
    struct MockEventStream {
        active: bool,
        info: StreamInfo,
        start_called: bool,
        stop_called: bool,
    }

    #[test]
    fn event_batch_processor_defaults() {
        let processor = MockBatchProcessor;

        assert_eq!(processor.optimal_batch_size(), 10);
        assert_eq!(processor.batch_timeout(), Duration::from_millis(100));
    }

    #[tokio::test]
    async fn event_handler_default_initialize() {
        let mut handler = MockHandlerForDefaults {
            info: HandlerInfo::new("test".to_owned(), "1.0".to_owned()),
        };

        let result = handler.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn event_handler_default_shutdown() {
        let mut handler = MockHandlerForDefaults {
            info: HandlerInfo::new("test".to_owned(), "1.0".to_owned()),
        };

        let result = handler.shutdown().await;
        assert!(result.is_ok());
    }

    impl MockEventStream {
        fn new() -> Self {
            Self {
                active: false,
                info: StreamInfo::new(
                    "mock-id".to_owned(),
                    "mock".to_owned(),
                    "test".to_owned(),
                    "mock://test".to_owned(),
                ),
                start_called: false,
                stop_called: false,
            }
        }
    }

    #[async_trait]
    impl EventStream for MockEventStream {
        fn info(&self) -> &StreamInfo {
            &self.info
        }

        fn info_mut(&mut self) -> &mut StreamInfo {
            &mut self.info
        }

        fn is_active(&self) -> bool {
            self.active
        }

        async fn start(
            &mut self,
        ) -> EventResult<Pin<Box<dyn Stream<Item = EventResult<Box<dyn Event>>> + Send>>> {
            self.active = true;
            self.start_called = true;
            self.info.active = true;
            return Err(EventError::generic("Mock stream"));
        }

        async fn stop(&mut self) -> EventResult<()> {
            self.active = false;
            self.stop_called = true;
            self.info.active = false;
            return Ok(());
        }
    }

    #[tokio::test]
    async fn event_stream_restart_when_active() {
        let mut stream = MockEventStream::new();
        stream.active = true; // Set as active initially

        let result = stream.restart().await;

        // Should have called stop then start
        assert!(stream.stop_called);
        assert!(stream.start_called);
        assert!(result.is_err()); // Mock implementation returns error
    }

    #[tokio::test]
    async fn event_stream_restart_when_inactive() {
        let mut stream = MockEventStream::new();
        stream.active = false; // Set as inactive initially

        let result = stream.restart().await;

        // Should only have called start (not stop since it wasn't active)
        assert!(!stream.stop_called);
        assert!(stream.start_called);
        assert!(result.is_err()); // Mock implementation returns error
    }
}
