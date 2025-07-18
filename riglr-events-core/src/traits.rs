//! Core traits for event processing.

use crate::error::{EventError, EventResult};
use crate::types::{EventKind, EventMetadata, StreamInfo};
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::pin::Pin;
use std::time::{Duration, SystemTime};

/// Core event trait that all events must implement.
///
/// This trait provides the minimal interface that all events share,
/// regardless of their source or specific data format.
pub trait Event: Debug + Send + Sync {
    /// Get the unique event identifier
    fn id(&self) -> &str;

    /// Get the event kind/classification
    fn kind(&self) -> &EventKind;

    /// Get event metadata
    fn metadata(&self) -> &EventMetadata;

    /// Get mutable access to event metadata
    fn metadata_mut(&mut self) -> &mut EventMetadata;

    /// Get the event timestamp
    fn timestamp(&self) -> SystemTime {
        self.metadata().timestamp.into()
    }

    /// Get the source that generated this event
    fn source(&self) -> &str {
        &self.metadata().source
    }

    /// Convert event to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Convert event to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Clone the event as a boxed trait object
    fn clone_boxed(&self) -> Box<dyn Event>;

    /// Serialize the event to JSON
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Err(EventError::generic(
            "Event serialization not implemented",
        ))
    }

    /// Check if this event matches a given filter criteria
    fn matches_filter(&self, filter: &dyn EventFilter) -> bool
    where
        Self: Sized,
    {
        filter.matches(self)
    }
}

// Implement Clone for Box<dyn Event>
impl Clone for Box<dyn Event> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

/// Parser trait for extracting events from raw data.
///
/// Parsers are responsible for converting raw bytes, JSON, or other
/// data formats into structured Event objects.
#[async_trait]
pub trait EventParser: Send + Sync {
    /// The type of data this parser can handle
    type Input: Send + Sync;

    /// Parse events from input data
    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>>;

    /// Check if this parser can handle the given input
    fn can_parse(&self, input: &Self::Input) -> bool;

    /// Get parser configuration information
    fn info(&self) -> ParserInfo;
}

/// Information about an event parser
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserInfo {
    /// Parser name/identifier
    pub name: String,
    /// Parser version
    pub version: String,
    /// Event kinds this parser can produce
    pub supported_kinds: Vec<EventKind>,
    /// Input data formats this parser supports
    pub supported_formats: Vec<String>,
}

impl ParserInfo {
    /// Create new parser info
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            supported_kinds: Vec::new(),
            supported_formats: Vec::new(),
        }
    }

    /// Add supported event kind
    pub fn with_kind(mut self, kind: EventKind) -> Self {
        self.supported_kinds.push(kind);
        self
    }

    /// Add supported input format
    pub fn with_format(mut self, format: String) -> Self {
        self.supported_formats.push(format);
        self
    }
}

/// Stream trait for producing events asynchronously.
///
/// Event streams are the primary way to receive events from various
/// sources like websockets, message queues, or blockchain nodes.
#[async_trait]
pub trait EventStream: Send + Sync {
    /// Start the stream and return a stream of events
    async fn start(&mut self) -> EventResult<Pin<Box<dyn Stream<Item = EventResult<Box<dyn Event>>> + Send>>>;

    /// Stop the stream
    async fn stop(&mut self) -> EventResult<()>;

    /// Check if the stream is currently active
    fn is_active(&self) -> bool;

    /// Get stream information and health metrics
    fn info(&self) -> &StreamInfo;

    /// Get mutable stream information
    fn info_mut(&mut self) -> &mut StreamInfo;

    /// Restart the stream (stop then start)
    async fn restart(&mut self) -> EventResult<Pin<Box<dyn Stream<Item = EventResult<Box<dyn Event>>> + Send>>> {
        if self.is_active() {
            self.stop().await?;
        }
        self.start().await
    }
}

/// Filter trait for event routing and selection.
///
/// Filters are used to determine which events should be processed
/// by specific handlers or forwarded to particular destinations.
pub trait EventFilter: Send + Sync + std::fmt::Debug {
    /// Check if an event matches this filter
    fn matches(&self, event: &dyn Event) -> bool;

    /// Get a description of what this filter does
    fn description(&self) -> String;
}

/// Handler trait for processing events.
///
/// Handlers contain the business logic for responding to specific
/// types of events, such as updating databases, sending notifications,
/// or triggering other actions.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event asynchronously
    async fn handle(&self, event: Box<dyn Event>) -> EventResult<()>;

    /// Check if this handler can process the given event
    fn can_handle(&self, event: &dyn Event) -> bool;

    /// Get handler information
    fn info(&self) -> HandlerInfo;

    /// Initialize the handler (called before first use)
    async fn initialize(&mut self) -> EventResult<()> {
        Ok(())
    }

    /// Shutdown the handler (cleanup resources)
    async fn shutdown(&mut self) -> EventResult<()> {
        Ok(())
    }
}

/// Information about an event handler
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandlerInfo {
    /// Handler name/identifier
    pub name: String,
    /// Handler version
    pub version: String,
    /// Event kinds this handler processes
    pub handled_kinds: Vec<EventKind>,
    /// Maximum processing time before timeout
    pub timeout: Option<Duration>,
}

impl HandlerInfo {
    /// Create new handler info
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            handled_kinds: Vec::new(),
            timeout: None,
        }
    }

    /// Add handled event kind
    pub fn with_kind(mut self, kind: EventKind) -> Self {
        self.handled_kinds.push(kind);
        self
    }

    /// Set timeout duration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Batch processor for handling multiple events efficiently.
#[async_trait]
pub trait EventBatchProcessor: Send + Sync {
    /// Process a batch of events
    async fn process_batch(&self, events: Vec<Box<dyn Event>>) -> EventResult<Vec<EventResult<()>>>;

    /// Get the optimal batch size for this processor
    fn optimal_batch_size(&self) -> usize {
        10
    }

    /// Get maximum time to wait for batch to fill
    fn batch_timeout(&self) -> Duration {
        Duration::from_millis(100)
    }
}

/// Event transformation trait for converting events between formats.
pub trait EventTransformer: Send + Sync {
    /// Transform an event into a different format
    fn transform(&self, event: Box<dyn Event>) -> EventResult<Box<dyn Event>>;

    /// Check if this transformer can process the given event
    fn can_transform(&self, event: &dyn Event) -> bool;
}

/// Event routing trait for directing events to appropriate handlers.
#[async_trait]
pub trait EventRouter: Send + Sync {
    /// Route an event to appropriate handlers
    async fn route(&self, event: Box<dyn Event>) -> EventResult<Vec<Box<dyn EventHandler>>>;

    /// Add a handler to the routing table
    fn add_handler(&mut self, filter: Box<dyn EventFilter>, handler: Box<dyn EventHandler>);

    /// Remove handlers matching a filter
    fn remove_handlers(&mut self, filter: Box<dyn EventFilter>) -> usize;
}

/// Metrics collector trait for gathering event processing statistics.
pub trait EventMetrics: Send + Sync {
    /// Record an event being processed
    fn record_event(&self, event: &dyn Event, processing_time: Duration);

    /// Record an error during event processing
    fn record_error(&self, event: &dyn Event, error: &EventError);

    /// Get current metrics snapshot
    fn snapshot(&self) -> EventResult<serde_json::Value>;

    /// Reset all metrics
    fn reset(&self);
}

// Common filter implementations

/// Filter that matches events by kind
#[derive(Debug, Clone)]
pub struct KindFilter {
    /// Event kinds to match
    pub kinds: Vec<EventKind>,
}

impl KindFilter {
    /// Create a new kind filter
    pub fn new(kinds: Vec<EventKind>) -> Self {
        Self { kinds }
    }

    /// Create a filter for a single kind
    pub fn single(kind: EventKind) -> Self {
        Self { kinds: vec![kind] }
    }
}

impl EventFilter for KindFilter {
    fn matches(&self, event: &dyn Event) -> bool {
        self.kinds.contains(event.kind())
    }

    fn description(&self) -> String {
        format!("Filter events by kind: {:?}", self.kinds)
    }
}

/// Filter that matches events by source
#[derive(Debug, Clone)]
pub struct SourceFilter {
    /// Sources to match (exact match)
    pub sources: Vec<String>,
}

impl SourceFilter {
    /// Create a new source filter
    pub fn new(sources: Vec<String>) -> Self {
        Self { sources }
    }

    /// Create a filter for a single source
    pub fn single(source: String) -> Self {
        Self {
            sources: vec![source],
        }
    }
}

impl EventFilter for SourceFilter {
    fn matches(&self, event: &dyn Event) -> bool {
        self.sources.contains(&event.source().to_string())
    }

    fn description(&self) -> String {
        format!("Filter events by source: {:?}", self.sources)
    }
}

/// Composite filter that combines multiple filters with AND logic
#[derive(Debug)]
pub struct AndFilter {
    /// Filters to combine
    pub filters: Vec<Box<dyn EventFilter>>,
}

impl AndFilter {
    /// Create a new AND filter
    pub fn new(filters: Vec<Box<dyn EventFilter>>) -> Self {
        Self { filters }
    }
}

impl EventFilter for AndFilter {
    fn matches(&self, event: &dyn Event) -> bool {
        self.filters.iter().all(|f| f.matches(event))
    }

    fn description(&self) -> String {
        let descriptions: Vec<String> = self.filters.iter().map(|f| f.description()).collect();
        format!("AND({})", descriptions.join(", "))
    }
}

/// Composite filter that combines multiple filters with OR logic
#[derive(Debug)]
pub struct OrFilter {
    /// Filters to combine
    pub filters: Vec<Box<dyn EventFilter>>,
}

impl OrFilter {
    /// Create a new OR filter
    pub fn new(filters: Vec<Box<dyn EventFilter>>) -> Self {
        Self { filters }
    }
}

impl EventFilter for OrFilter {
    fn matches(&self, event: &dyn Event) -> bool {
        self.filters.iter().any(|f| f.matches(event))
    }

    fn description(&self) -> String {
        let descriptions: Vec<String> = self.filters.iter().map(|f| f.description()).collect();
        format!("OR({})", descriptions.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GenericEvent;
    use serde_json::json;

    // Mock implementations for testing
    struct MockHandler {
        info: HandlerInfo,
    }

    #[async_trait]
    impl EventHandler for MockHandler {
        async fn handle(&self, _event: Box<dyn Event>) -> EventResult<()> {
            Ok(())
        }

        fn can_handle(&self, event: &dyn Event) -> bool {
            self.info.handled_kinds.contains(event.kind())
        }

        fn info(&self) -> HandlerInfo {
            self.info.clone()
        }
    }

    #[tokio::test]
    async fn test_kind_filter() {
        let filter = KindFilter::single(EventKind::Transaction);

        let event = GenericEvent::new(
            "test".to_string(),
            EventKind::Transaction,
            json!({}),
        );

        assert!(filter.matches(&event));

        let event2 = GenericEvent::new(
            "test2".to_string(),
            EventKind::Block,
            json!({}),
        );

        assert!(!filter.matches(&event2));
    }

    #[tokio::test]
    async fn test_source_filter() {
        let filter = SourceFilter::single("test-source".to_string());

        let mut event = GenericEvent::new(
            "test".to_string(),
            EventKind::Transaction,
            json!({}),
        );
        event.metadata.source = "test-source".to_string();

        assert!(filter.matches(&event));

        event.metadata.source = "other-source".to_string();
        assert!(!filter.matches(&event));
    }

    #[tokio::test]
    async fn test_and_filter() {
        let kind_filter = Box::new(KindFilter::single(EventKind::Transaction));
        let source_filter = Box::new(SourceFilter::single("test-source".to_string()));
        let and_filter = AndFilter::new(vec![kind_filter, source_filter]);

        let mut event = GenericEvent::new(
            "test".to_string(),
            EventKind::Transaction,
            json!({}),
        );
        event.metadata.source = "test-source".to_string();

        assert!(and_filter.matches(&event));

        // Change kind - should not match
        let mut event2 = event.clone();
        event2.metadata.kind = EventKind::Block;
        assert!(!and_filter.matches(&event2));

        // Change source - should not match
        let mut event3 = event.clone();
        event3.metadata.source = "other-source".to_string();
        assert!(!and_filter.matches(&event3));
    }

    #[tokio::test]
    async fn test_or_filter() {
        let kind_filter = Box::new(KindFilter::single(EventKind::Transaction));
        let source_filter = Box::new(SourceFilter::single("test-source".to_string()));
        let or_filter = OrFilter::new(vec![kind_filter, source_filter]);

        // Event matches kind but not source
        let event1 = GenericEvent::new(
            "test1".to_string(),
            EventKind::Transaction,
            json!({}),
        );
        assert!(or_filter.matches(&event1));

        // Event matches source but not kind
        let mut event2 = GenericEvent::new(
            "test2".to_string(),
            EventKind::Block,
            json!({}),
        );
        event2.metadata.source = "test-source".to_string();
        assert!(or_filter.matches(&event2));

        // Event matches neither
        let event3 = GenericEvent::new(
            "test3".to_string(),
            EventKind::Block,
            json!({}),
        );
        assert!(!or_filter.matches(&event3));
    }

    #[tokio::test]
    async fn test_handler_info() {
        let info = HandlerInfo::new("test-handler".to_string(), "1.0.0".to_string())
            .with_kind(EventKind::Transaction)
            .with_kind(EventKind::Block)
            .with_timeout(Duration::from_secs(30));

        assert_eq!(info.name, "test-handler");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.handled_kinds.len(), 2);
        assert_eq!(info.timeout, Some(Duration::from_secs(30)));
    }

    #[tokio::test]
    async fn test_parser_info() {
        let info = ParserInfo::new("test-parser".to_string(), "1.0.0".to_string())
            .with_kind(EventKind::Transaction)
            .with_format("json".to_string())
            .with_format("binary".to_string());

        assert_eq!(info.name, "test-parser");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.supported_kinds.len(), 1);
        assert_eq!(info.supported_formats.len(), 2);
    }

    #[test]
    fn test_generic_event_trait_implementation() {
        let event = GenericEvent::new(
            "test-event".to_string(),
            EventKind::Swap,
            json!({"amount": 100}),
        );

        assert_eq!(event.id(), "test-event");
        assert_eq!(event.kind(), &EventKind::Swap);
        assert_eq!(event.source(), "generic");

        // Test clone
        let cloned = event.clone_boxed();
        assert_eq!(cloned.id(), event.id());
    }
}