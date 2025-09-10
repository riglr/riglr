# riglr-events-core

{{#include ../../../riglr-events-core/README.md}}

## API Reference

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Type Aliases](#type-aliases)

### Structs

> Core data structures and types.

#### `AndFilter`

Composite filter that combines multiple filters with AND logic

---

#### `BinaryEventParser`

Binary data parser for handling raw bytes.

---

#### `EventBatcher`

Batching utility for accumulating events before processing.

---

#### `EventDeduplicator`

Event deduplication utility to prevent processing duplicate events.

---

#### `EventIdGenerator`

Utility for generating unique event IDs.

---

#### `EventMetadata`

Metadata associated with an event, providing context and timing information.

---

#### `EventPerformanceMetrics`

Performance metrics collector for event processing

---

#### `GenericEvent`

Generic event implementation suitable for most use cases.

---

#### `HandlerInfo`

Information about an event handler

---

#### `JsonEventParser`

JSON-based event parser implementation.

---

#### `KindFilter`

Filter that matches events by kind

---

#### `MetricsSummary`

Summary of event processing metrics

---

#### `MultiFormatParser`

Multi-format parser that can handle different data formats.

---

#### `OrFilter`

Composite filter that combines multiple filters with OR logic

---

#### `ParserInfo`

Information about an event parser

---

#### `ParsingConfig`

Configuration for parsing events from different data sources.

---

#### `RateLimiter`

Rate limiting utility for controlling event processing speed.

---

#### `SourceFilter`

Filter that matches events by source

---

#### `StreamInfo`

Stream information for tracking event stream sources and their health.

---

#### `StreamMetrics`

Stream health and performance metrics.

---

#### `StreamUtils`

Stream transformation utilities

---

#### `ValidationConfig`

Validation configuration for parsed events.

---

### Enums

> Enumeration types for representing variants.

#### `ChainData`

Blockchain-specific data that can be attached to events.

**Variants:**

- `Solana`
  - Solana-specific data
- `Evm`
  - EVM-specific data
- `Generic`
  - Generic chain data for unsupported chains

---

#### `EventError`

Main error type for event processing operations.

**Variants:**

- `ParseError`
  - Event parsing failed
- `StreamError`
  - Event stream error
- `FilterError`
  - Event filtering error
- `HandlerError`
  - Event handler error
- `Serialization`
  - Serialization/deserialization error
- `Io`
  - I/O error
- `InvalidConfig`
  - Invalid configuration
- `NotFound`
  - Resource not found
- `Timeout`
  - Operation timeout
- `Generic`
  - Generic event error

---

#### `EventKind`

Event classification for different types of blockchain and system events.

**Variants:**

- `Transaction`
  - Blockchain transaction events
- `Block`
  - Block-level events (new blocks, reorganizations)
- `Contract`
  - Smart contract events and logs
- `Transfer`
  - Token transfer events
- `Swap`
  - Swap/DEX events
- `Liquidity`
  - Liquidity provision events
- `Price`
  - Price update events
- `External`
  - External system events (APIs, websockets)
- `Custom`
  - Custom event type with string identifier

---

### Traits

> Trait definitions for implementing common behaviors.

#### `Event`

Core event trait that all events must implement.

This trait provides the minimal interface that all events share,
regardless of their source or specific data format.

**Methods:**

- `id()`
  - Get the unique event identifier
- `kind()`
  - Get the event kind/classification
- `metadata()`
  - Get event metadata
- `metadata_mut()`
  - Get mutable access to event metadata
- `timestamp()`
  - Get the event timestamp
- `source()`
  - Get the source that generated this event
- `as_any()`
  - Convert event to Any for downcasting
- `as_any_mut()`
  - Convert event to mutable Any for downcasting
- `clone_boxed()`
  - Clone the event as a boxed trait object
- `to_json()`
  - Serialize the event to JSON
- `matches_filter()`
  - Check if this event matches a given filter criteria

---

#### `EventBatchProcessor`

Batch processor for handling multiple events efficiently.

**Methods:**

- `process_batch()`
  - Process a batch of events
- `optimal_batch_size()`
  - Get the optimal batch size for this processor
- `batch_timeout()`
  - Get maximum time to wait for batch to fill

---

#### `EventFilter`

Filter trait for event routing and selection.

Filters are used to determine which events should be processed
by specific handlers or forwarded to particular destinations.

**Methods:**

- `matches()`
  - Check if an event matches this filter
- `description()`
  - Get a description of what this filter does

---

#### `EventHandler`

Handler trait for processing events.

Handlers contain the business logic for responding to specific
types of events, such as updating databases, sending notifications,
or triggering other actions.

**Methods:**

- `handle()`
  - Handle an event asynchronously
- `can_handle()`
  - Check if this handler can process the given event
- `info()`
  - Get handler information
- `initialize()`
  - Initialize the handler (called before first use)
- `shutdown()`
  - Shutdown the handler (cleanup resources)

---

#### `EventMetrics`

Metrics collector trait for gathering event processing statistics.

**Methods:**

- `record_event()`
  - Record an event being processed
- `record_error()`
  - Record an error during event processing
- `snapshot()`
  - Get current metrics snapshot
- `reset()`
  - Reset all metrics

---

#### `EventParser`

Parser trait for extracting events from raw data.

Parsers are responsible for converting raw bytes, JSON, or other
data formats into structured Event objects.

**Methods:**

- `parse()`
  - Parse events from input data
- `can_parse()`
  - Check if this parser can handle the given input
- `info()`
  - Get parser configuration information

---

#### `EventRouter`

Event routing trait for directing events to appropriate handlers.

**Methods:**

- `route()`
  - Route an event to appropriate handlers
- `add_handler()`
  - Add a handler to the routing table
- `remove_handlers()`
  - Remove handlers matching a filter

---

#### `EventStream`

Stream trait for producing events asynchronously.

Event streams are the primary way to receive events from various
sources like websockets, message queues, or blockchain nodes.

**Methods:**

- `start()`
  - Start the stream and return a stream of events
- `stop()`
  - Stop the stream
- `is_active()`
  - Check if the stream is currently active
- `info()`
  - Get stream information and health metrics
- `info_mut()`
  - Get mutable stream information
- `restart()`
  - Restart the stream (stop then start)

---

#### `EventTransformer`

Event transformation trait for converting events between formats.

**Methods:**

- `transform()`
  - Transform an event into a different format
- `can_transform()`
  - Check if this transformer can process the given event

---

### Type Aliases

#### `EventBatchStream`

Type alias for event batch streams

**Type:** `<<_>>`

---

#### `EventResult`

Specialized result type for event operations

**Type:** `<T, >`

---
