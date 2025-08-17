# riglr-events-core API Reference

Comprehensive API documentation for the `riglr-events-core` crate.

## Table of Contents

### Enums

- [`ChainData`](#chaindata)
- [`EventError`](#eventerror)
- [`EventKind`](#eventkind)

### Functions (error)

- [`filter_error`](#filter_error)
- [`generic`](#generic)
- [`handler_error`](#handler_error)
- [`invalid_config`](#invalid_config)
- [`is_retriable`](#is_retriable)
- [`not_found`](#not_found)
- [`parse_error`](#parse_error)
- [`stream_error`](#stream_error)
- [`timeout`](#timeout)
- [`to_tool_error`](#to_tool_error)

### Structs

- [`AndFilter`](#andfilter)
- [`BinaryEventParser`](#binaryeventparser)
- [`EventBatcher`](#eventbatcher)
- [`EventDeduplicator`](#eventdeduplicator)
- [`EventIdGenerator`](#eventidgenerator)
- [`EventMetadata`](#eventmetadata)
- [`EventPerformanceMetrics`](#eventperformancemetrics)
- [`GenericEvent`](#genericevent)
- [`HandlerInfo`](#handlerinfo)
- [`JsonEventParser`](#jsoneventparser)
- [`KindFilter`](#kindfilter)
- [`MetricsSummary`](#metricssummary)
- [`MultiFormatParser`](#multiformatparser)
- [`OrFilter`](#orfilter)
- [`ParserInfo`](#parserinfo)
- [`ParsingConfig`](#parsingconfig)
- [`RateLimiter`](#ratelimiter)
- [`SourceFilter`](#sourcefilter)
- [`StreamInfo`](#streaminfo)
- [`StreamMetrics`](#streammetrics)
- [`StreamUtils`](#streamutils)
- [`ValidationConfig`](#validationconfig)

### Functions (parser)

- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`with_default`](#with_default)
- [`with_mapping`](#with_mapping)
- [`with_parser`](#with_parser)
- [`with_pattern`](#with_pattern)
- [`with_range`](#with_range)
- [`with_required`](#with_required)
- [`with_type`](#with_type)
- [`with_validation`](#with_validation)

### Traits

- [`Event`](#event)
- [`EventBatchProcessor`](#eventbatchprocessor)
- [`EventFilter`](#eventfilter)
- [`EventHandler`](#eventhandler)
- [`EventMetrics`](#eventmetrics)
- [`EventParser`](#eventparser)
- [`EventRouter`](#eventrouter)
- [`EventStream`](#eventstream)
- [`EventTransformer`](#eventtransformer)

### Functions (traits)

- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`single`](#single)
- [`single`](#single)
- [`with_format`](#with_format)
- [`with_kind`](#with_kind)
- [`with_kind`](#with_kind)
- [`with_timeout`](#with_timeout)

### Type Aliases

- [`EventBatchStream`](#eventbatchstream)
- [`EventStream`](#eventstream)

### Functions (utils)

- [`add`](#add)
- [`avg_processing_time`](#avg_processing_time)
- [`batch_events`](#batch_events)
- [`can_process`](#can_process)
- [`cleanup`](#cleanup)
- [`current_rate`](#current_rate)
- [`current_size`](#current_size)
- [`deduplicate`](#deduplicate)
- [`error_rate`](#error_rate)
- [`filter_events`](#filter_events)
- [`flush`](#flush)
- [`is_duplicate`](#is_duplicate)
- [`is_empty`](#is_empty)
- [`map_events`](#map_events)
- [`mark_seen`](#mark_seen)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`next`](#next)
- [`next_with_context`](#next_with_context)
- [`processing_time_percentiles`](#processing_time_percentiles)
- [`rate_limit`](#rate_limit)
- [`record_error`](#record_error)
- [`record_event`](#record_event)
- [`record_processing_time`](#record_processing_time)
- [`reset`](#reset)
- [`should_flush`](#should_flush)
- [`start_cleanup_task`](#start_cleanup_task)
- [`summary`](#summary)
- [`total_errors`](#total_errors)
- [`total_events`](#total_events)
- [`wait_for_capacity`](#wait_for_capacity)

### Functions (types)

- [`block_id`](#block_id)
- [`chain_id`](#chain_id)
- [`error_rate`](#error_rate)
- [`extract_data`](#extract_data)
- [`get_custom`](#get_custom)
- [`is_healthy`](#is_healthy)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`record_error`](#record_error)
- [`set_active`](#set_active)
- [`transaction_id`](#transaction_id)
- [`update_data`](#update_data)
- [`update_last_event`](#update_last_event)
- [`with_chain_data`](#with_chain_data)
- [`with_custom`](#with_custom)
- [`with_metadata`](#with_metadata)
- [`with_source`](#with_source)
- [`with_timestamp`](#with_timestamp)

## Enums

### ChainData

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "chain", content = "data")]
```

```rust
pub enum ChainData { /// Solana-specific data #[cfg(feature = "solana")] Solana { /// Slot number slot: u64, /// Transaction signature signature: Option<String>, /// Program ID program_id: Option<solana_sdk::pubkey::Pubkey>, /// Instruction index within transaction instruction_index: Option<usize>, /// Block time in Unix seconds block_time: Option<i64>, /// Custom protocol-specific data (e.g., protocol_type, event_type) protocol_data: Option<serde_json::Value>, }, /// EVM-specific data #[cfg(feature = "evm")] Evm { /// Block number block_number: u64, /// Transaction hash transaction_hash: Option<String>, /// Contract address contract_address: Option<String>, /// Event log index log_index: Option<usize>, /// Chain ID chain_id: u64, }, /// Generic chain data for unsupported chains Generic { /// Chain identifier chain_id: String, /// Generic block identifier block_id: String, /// Generic transaction identifier transaction_id: Option<String>, /// Additional data data: HashMap<String, serde_json::Value>, }, }
```

Blockchain-specific data that can be attached to events.

**Variants**:

- `Solana`
- `slot`
- `signature`
- `program_id`
- `instruction_index`
- `block_time`
- `protocol_data`
- `Evm`
- `block_number`
- `transaction_hash`
- `contract_address`
- `log_index`
- `chain_id`
- `Generic`
- `chain_id`
- `block_id`
- `transaction_id`
- `data`

---

### EventError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum EventError { /// Event parsing failed #[error("Event parsing error: {context}")] ParseError { /// Error context context: String, /// Source error #[source] source: Box<dyn std::error::Error + Send + Sync>, }, /// Event stream error #[error("Event stream error: {context}")] StreamError { /// Error context context: String, /// Source error #[source] source: Box<dyn std::error::Error + Send + Sync>, }, /// Event filtering error #[error("Event filtering error: {context}")] FilterError { /// Error context context: String, /// Source error #[source] source: Box<dyn std::error::Error + Send + Sync>, }, /// Event handler error #[error("Event handler error: {context}")] HandlerError { /// Error context context: String, /// Source error #[source] source: Box<dyn std::error::Error + Send + Sync>, }, /// Serialization/deserialization error #[error("Serialization error: {0}")] Serialization(#[from] serde_json::Error), /// I/O error #[error("I/O error: {0}")] Io(#[from] std::io::Error), /// Invalid configuration #[error("Invalid configuration: {message}")] InvalidConfig { /// Error message message: String, }, /// Resource not found #[error("Resource not found: {resource}")] NotFound { /// Resource identifier resource: String, }, /// Operation timeout #[error("Operation timed out after {duration:?}")] Timeout { /// Timeout duration duration: std::time::Duration, }, /// Generic event error #[error("Event error: {message}")] Generic { /// Error message message: String, }, }
```

Main error type for event processing operations.

**Variants**:

- `ParseError`
- `context`
- `source`
- `StreamError`
- `context`
- `source`
- `FilterError`
- `context`
- `source`
- `HandlerError`
- `context`
- `source`
- `Serialization(#[from] serde_json::Error)`
- `Io(#[from] std::io::Error)`
- `InvalidConfig`
- `message`
- `NotFound`
- `resource`
- `Timeout`
- `duration`
- `Generic`
- `message`

---

### EventKind

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
```

```rust
pub enum EventKind { /// Blockchain transaction events #[default] Transaction, /// Block-level events (new blocks, reorganizations) Block, /// Smart contract events and logs Contract, /// Token transfer events Transfer, /// Swap/DEX events Swap, /// Liquidity provision events Liquidity, /// Price update events Price, /// External system events (APIs, websockets) External, /// Custom event type with string identifier Custom(String), }
```

Event classification for different types of blockchain and system events.

**Variants**:

- `Transaction`
- `Block`
- `Contract`
- `Transfer`
- `Swap`
- `Liquidity`
- `Price`
- `External`
- `Custom(String)`

---

## Functions (error)

### filter_error

**Source**: `src/error.rs`

```rust
pub fn filter_error<E: std::error::Error + Send + Sync + 'static>( source: E, context: impl Into<String>, ) -> Self
```

Create a filter error with source preservation

---

### generic

**Source**: `src/error.rs`

```rust
pub fn generic(message: impl Into<String>) -> Self
```

Create a generic error

---

### handler_error

**Source**: `src/error.rs`

```rust
pub fn handler_error<E: std::error::Error + Send + Sync + 'static>( source: E, context: impl Into<String>, ) -> Self
```

Create a handler error with source preservation

---

### invalid_config

**Source**: `src/error.rs`

```rust
pub fn invalid_config(message: impl Into<String>) -> Self
```

Create an invalid configuration error

---

### is_retriable

**Source**: `src/error.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if the error is retriable (follows riglr-core patterns)

---

### not_found

**Source**: `src/error.rs`

```rust
pub fn not_found(resource: impl Into<String>) -> Self
```

Create a not found error

---

### parse_error

**Source**: `src/error.rs`

```rust
pub fn parse_error<E: std::error::Error + Send + Sync + 'static>( source: E, context: impl Into<String>, ) -> Self
```

Create a parse error with source preservation

---

### stream_error

**Source**: `src/error.rs`

```rust
pub fn stream_error<E: std::error::Error + Send + Sync + 'static>( source: E, context: impl Into<String>, ) -> Self
```

Create a stream error with source preservation

---

### timeout

**Source**: `src/error.rs`

```rust
pub fn timeout(duration: std::time::Duration) -> Self
```

Create a timeout error

---

### to_tool_error

**Source**: `src/error.rs`

```rust
pub fn to_tool_error(self) -> ToolError
```

Convert to ToolError for integration with riglr-core

---

## Structs

### AndFilter

**Source**: `src/traits.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct AndFilter { /// Filters to combine pub filters: Vec<Box<dyn EventFilter>>, }
```

Composite filter that combines multiple filters with AND logic

---

### BinaryEventParser

**Source**: `src/parser.rs`

```rust
pub struct BinaryEventParser { name: String, version: String, event_kind: EventKind, source_name: String, }
```

Binary data parser for handling raw bytes.

---

### EventBatcher

**Source**: `src/utils.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct EventBatcher { batch_size: usize, timeout: Duration, current_batch: Vec<Box<dyn Event>>, last_flush: Instant, }
```

Batching utility for accumulating events before processing.

---

### EventDeduplicator

**Source**: `src/utils.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct EventDeduplicator { seen_events: Arc<DashMap<String, SystemTime>>, ttl: Duration, cleanup_interval: Duration, }
```

Event deduplication utility to prevent processing duplicate events.

---

### EventIdGenerator

**Source**: `src/utils.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct EventIdGenerator { prefix: String, counter: Arc<AtomicU64>, }
```

Utility for generating unique event IDs.

---

### EventMetadata

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
```

```rust
pub struct EventMetadata { /// Unique event identifier pub id: String, /// Event classification pub kind: EventKind, /// When the event occurred (blockchain time or system time)
```

Metadata associated with an event, providing context and timing information.

---

### EventPerformanceMetrics

**Source**: `src/utils.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct EventPerformanceMetrics { total_events: Arc<AtomicU64>, total_errors: Arc<AtomicU64>, processing_times: Arc<RwLock<Vec<Duration>>>, last_reset: Arc<RwLock<SystemTime>>, }
```

Performance metrics collector for event processing

---

### GenericEvent

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
```

```rust
pub struct GenericEvent { /// Event metadata pub metadata: EventMetadata, /// Event payload data pub data: serde_json::Value, }
```

Generic event implementation suitable for most use cases.

---

### HandlerInfo

**Source**: `src/traits.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
```

```rust
pub struct HandlerInfo { /// Handler name/identifier pub name: String, /// Handler version pub version: String, /// Event kinds this handler processes pub handled_kinds: Vec<EventKind>, /// Maximum processing time before timeout pub timeout: Option<Duration>, }
```

Information about an event handler

---

### JsonEventParser

**Source**: `src/parser.rs`

```rust
pub struct JsonEventParser { config: ParsingConfig, event_kind: EventKind, source_name: String, }
```

JSON-based event parser implementation.

---

### KindFilter

**Source**: `src/traits.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct KindFilter { /// Event kinds to match pub kinds: Vec<EventKind>, }
```

Filter that matches events by kind

---

### MetricsSummary

**Source**: `src/utils.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
```

```rust
pub struct MetricsSummary { /// Total events processed pub total_events: u64, /// Total errors encountered pub total_errors: u64, /// Error rate as percentage pub error_rate: f64, /// Average processing time pub avg_processing_time: Duration, /// 95th percentile processing time pub p95_processing_time: Duration, /// 99th percentile processing time pub p99_processing_time: Duration, /// Time since metrics were last reset pub uptime: Duration, }
```

Summary of event processing metrics

---

### MultiFormatParser

**Source**: `src/parser.rs`

```rust
pub struct MultiFormatParser { parsers: HashMap<String, Arc<dyn EventParser<Input = Vec<u8>> + Send + Sync>>, default_format: String, }
```

Multi-format parser that can handle different data formats.

---

### OrFilter

**Source**: `src/traits.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct OrFilter { /// Filters to combine pub filters: Vec<Box<dyn EventFilter>>, }
```

Composite filter that combines multiple filters with OR logic

---

### ParserInfo

**Source**: `src/traits.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
```

```rust
pub struct ParserInfo { /// Parser name/identifier pub name: String, /// Parser version pub version: String, /// Event kinds this parser can produce pub supported_kinds: Vec<EventKind>, /// Input data formats this parser supports pub supported_formats: Vec<String>, }
```

Information about an event parser

---

### ParsingConfig

**Source**: `src/parser.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct ParsingConfig { /// Parser name pub name: String, /// Parser version pub version: String, /// Data format (e.g., "json", "binary", "xml")
```

Configuration for parsing events from different data sources.

---

### RateLimiter

**Source**: `src/utils.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct RateLimiter { max_rate: u64, // events per second window: Duration, events_in_window: Arc<RwLock<Vec<SystemTime>>>, }
```

Rate limiting utility for controlling event processing speed.

---

### SourceFilter

**Source**: `src/traits.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct SourceFilter { /// Sources to match (exact match)
```

Filter that matches events by source

---

### StreamInfo

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
```

```rust
pub struct StreamInfo { /// Unique stream identifier pub id: String, /// Human-readable stream name pub name: String, /// Stream source type (e.g., "websocket", "rpc", "kafka")
```

Stream information for tracking event stream sources and their health.

---

### StreamMetrics

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
```

```rust
pub struct StreamMetrics { /// Total number of events processed pub event_count: u64, /// Total number of errors encountered pub error_count: u64, /// Total number of connections made pub connection_count: u64, /// Average processing time in microseconds pub avg_processing_time_us: u64, /// Stream uptime percentage (0-100)
```

Stream health and performance metrics.

---

### StreamUtils

**Source**: `src/utils.rs`

```rust
pub struct StreamUtils;
```

Stream transformation utilities

---

### ValidationConfig

**Source**: `src/parser.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct ValidationConfig { /// Required fields that must be present pub required_fields: Vec<String>, /// Field type constraints pub field_types: HashMap<String, String>, /// Minimum values for numeric fields pub min_values: HashMap<String, f64>, /// Maximum values for numeric fields pub max_values: HashMap<String, f64>, /// Regex patterns for string fields pub patterns: HashMap<String, String>, }
```

Validation configuration for parsed events.

---

## Functions (parser)

### new

**Source**: `src/parser.rs`

```rust
pub fn new(name: String, version: String, format: String) -> Self
```

Create a new parsing configuration

---

### new

**Source**: `src/parser.rs`

```rust
pub fn new(config: ParsingConfig, event_kind: EventKind, source_name: String) -> Self
```

Create a new JSON event parser

---

### new

**Source**: `src/parser.rs`

```rust
pub fn new(default_format: String) -> Self
```

Create a new multi-format parser

---

### new

**Source**: `src/parser.rs`

```rust
pub fn new(name: String, version: String, event_kind: EventKind, source_name: String) -> Self
```

Create a new binary event parser

---

### with_default

**Source**: `src/parser.rs`

```rust
pub fn with_default<T: Serialize>(mut self, field: String, value: T) -> Self
```

Add a default value

---

### with_mapping

**Source**: `src/parser.rs`

```rust
pub fn with_mapping(mut self, field: String, source_path: String) -> Self
```

Add a field mapping

---

### with_parser

**Source**: `src/parser.rs`

```rust
pub fn with_parser<P>(mut self, format: String, parser: P) -> Self where P: EventParser<Input = Vec<u8>> + Send + Sync + 'static,
```

Add a parser for a specific format

---

### with_pattern

**Source**: `src/parser.rs`

```rust
pub fn with_pattern(mut self, field: String, pattern: String) -> Self
```

Add a regex pattern constraint

---

### with_range

**Source**: `src/parser.rs`

```rust
pub fn with_range(mut self, field: String, min: f64, max: f64) -> Self
```

Add a numeric range constraint

---

### with_required

**Source**: `src/parser.rs`

```rust
pub fn with_required(mut self, field: String) -> Self
```

Add a required field

---

### with_type

**Source**: `src/parser.rs`

```rust
pub fn with_type(mut self, field: String, type_name: String) -> Self
```

Add a field type constraint

---

### with_validation

**Source**: `src/parser.rs`

```rust
pub fn with_validation(mut self, validation: ValidationConfig) -> Self
```

Set validation configuration

---

## Traits

### Event

**Source**: `src/traits.rs`

```rust
pub trait Event: Debug + Send + Sync { ... }
```

Core event trait that all events must implement.

This trait provides the minimal interface that all events share,
regardless of their source or specific data format.

**Methods**:

#### `id`

```rust
fn id(&self) -> &str;
```

#### `kind`

```rust
fn kind(&self) -> &EventKind;
```

#### `metadata`

```rust
fn metadata(&self) -> &EventMetadata;
```

#### `metadata_mut`

```rust
fn metadata_mut(&mut self) -> &mut EventMetadata;
```

#### `timestamp`

```rust
fn timestamp(&self) -> SystemTime {
```

#### `source`

```rust
fn source(&self) -> &str {
```

#### `as_any`

```rust
fn as_any(&self) -> &dyn std::any::Any;
```

#### `as_any_mut`

```rust
fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
```

#### `clone_boxed`

```rust
fn clone_boxed(&self) -> Box<dyn Event>;
```

#### `to_json`

```rust
fn to_json(&self) -> EventResult<serde_json::Value> {
```

#### `matches_filter`

```rust
fn matches_filter(&self, filter: &dyn EventFilter) -> bool where Self: Sized, {;
```

---

### EventBatchProcessor

**Source**: `src/traits.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait EventBatchProcessor: Send + Sync { ... }
```

Batch processor for handling multiple events efficiently.

**Methods**:

#### `process_batch`

```rust
async fn process_batch(&self, events: Vec<Box<dyn Event>>) -> EventResult<Vec<EventResult<()>>>;
```

#### `optimal_batch_size`

```rust
fn optimal_batch_size(&self) -> usize {
```

#### `batch_timeout`

```rust
fn batch_timeout(&self) -> Duration {
```

---

### EventFilter

**Source**: `src/traits.rs`

```rust
pub trait EventFilter: Send + Sync + std::fmt::Debug { ... }
```

Filter trait for event routing and selection.

Filters are used to determine which events should be processed
by specific handlers or forwarded to particular destinations.

**Methods**:

#### `matches`

```rust
fn matches(&self, event: &dyn Event) -> bool;
```

#### `description`

```rust
fn description(&self) -> String;
```

---

### EventHandler

**Source**: `src/traits.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait EventHandler: Send + Sync { ... }
```

Handler trait for processing events.

Handlers contain the business logic for responding to specific
types of events, such as updating databases, sending notifications,
or triggering other actions.

**Methods**:

#### `handle`

```rust
async fn handle(&self, event: Box<dyn Event>) -> EventResult<()>;
```

#### `can_handle`

```rust
fn can_handle(&self, event: &dyn Event) -> bool;
```

#### `info`

```rust
fn info(&self) -> HandlerInfo;
```

#### `initialize`

```rust
async fn initialize(&mut self) -> EventResult<()> {
```

#### `shutdown`

```rust
async fn shutdown(&mut self) -> EventResult<()> {
```

---

### EventMetrics

**Source**: `src/traits.rs`

```rust
pub trait EventMetrics: Send + Sync { ... }
```

Metrics collector trait for gathering event processing statistics.

**Methods**:

#### `record_event`

```rust
fn record_event(&self, event: &dyn Event, processing_time: Duration);
```

#### `record_error`

```rust
fn record_error(&self, event: &dyn Event, error: &EventError);
```

#### `snapshot`

```rust
fn snapshot(&self) -> EventResult<serde_json::Value>;
```

#### `reset`

```rust
fn reset(&self);
```

---

### EventParser

**Source**: `src/traits.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait EventParser: Send + Sync { ... }
```

Parser trait for extracting events from raw data.

Parsers are responsible for converting raw bytes, JSON, or other
data formats into structured Event objects.

**Methods**:

#### `parse`

```rust
async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>>;
```

#### `can_parse`

```rust
fn can_parse(&self, input: &Self::Input) -> bool;
```

#### `info`

```rust
fn info(&self) -> ParserInfo;
```

---

### EventRouter

**Source**: `src/traits.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait EventRouter: Send + Sync { ... }
```

Event routing trait for directing events to appropriate handlers.

**Methods**:

#### `route`

```rust
async fn route(&self, event: Box<dyn Event>) -> EventResult<Vec<Box<dyn EventHandler>>>;
```

#### `add_handler`

```rust
fn add_handler(&mut self, filter: Box<dyn EventFilter>, handler: Box<dyn EventHandler>);
```

#### `remove_handlers`

```rust
fn remove_handlers(&mut self, filter: Box<dyn EventFilter>) -> usize;
```

---

### EventStream

**Source**: `src/traits.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait EventStream: Send + Sync { ... }
```

Stream trait for producing events asynchronously.

Event streams are the primary way to receive events from various
sources like websockets, message queues, or blockchain nodes.

**Methods**:

#### `start`

```rust
async fn start( &mut self, ) -> EventResult<Pin<Box<dyn Stream<Item = EventResult<Box<dyn Event>>> + Send>>>;
```

#### `stop`

```rust
async fn stop(&mut self) -> EventResult<()>;
```

#### `is_active`

```rust
fn is_active(&self) -> bool;
```

#### `info`

```rust
fn info(&self) -> &StreamInfo;
```

#### `info_mut`

```rust
fn info_mut(&mut self) -> &mut StreamInfo;
```

#### `restart`

```rust
async fn restart( &mut self, ) -> EventResult<Pin<Box<dyn Stream<Item = EventResult<Box<dyn Event>>> + Send>>> {;
```

---

### EventTransformer

**Source**: `src/traits.rs`

```rust
pub trait EventTransformer: Send + Sync { ... }
```

Event transformation trait for converting events between formats.

**Methods**:

#### `transform`

```rust
fn transform(&self, event: Box<dyn Event>) -> EventResult<Box<dyn Event>>;
```

#### `can_transform`

```rust
fn can_transform(&self, event: &dyn Event) -> bool;
```

---

## Functions (traits)

### new

**Source**: `src/traits.rs`

```rust
pub fn new(name: String, version: String) -> Self
```

Create new parser info

---

### new

**Source**: `src/traits.rs`

```rust
pub fn new(name: String, version: String) -> Self
```

Create new handler info

---

### new

**Source**: `src/traits.rs`

```rust
pub fn new(kinds: Vec<EventKind>) -> Self
```

Create a new kind filter

---

### new

**Source**: `src/traits.rs`

```rust
pub fn new(sources: Vec<String>) -> Self
```

Create a new source filter

---

### new

**Source**: `src/traits.rs`

```rust
pub fn new(filters: Vec<Box<dyn EventFilter>>) -> Self
```

Create a new AND filter

---

### new

**Source**: `src/traits.rs`

```rust
pub fn new(filters: Vec<Box<dyn EventFilter>>) -> Self
```

Create a new OR filter

---

### single

**Source**: `src/traits.rs`

```rust
pub fn single(kind: EventKind) -> Self
```

Create a filter for a single kind

---

### single

**Source**: `src/traits.rs`

```rust
pub fn single(source: String) -> Self
```

Create a filter for a single source

---

### with_format

**Source**: `src/traits.rs`

```rust
pub fn with_format(mut self, format: String) -> Self
```

Add supported input format

---

### with_kind

**Source**: `src/traits.rs`

```rust
pub fn with_kind(mut self, kind: EventKind) -> Self
```

Add supported event kind

---

### with_kind

**Source**: `src/traits.rs`

```rust
pub fn with_kind(mut self, kind: EventKind) -> Self
```

Add handled event kind

---

### with_timeout

**Source**: `src/traits.rs`

```rust
pub fn with_timeout(mut self, timeout: Duration) -> Self
```

Set timeout duration

---

## Type Aliases

### EventBatchStream

**Source**: `src/utils.rs`

```rust
type EventBatchStream = Pin<Box<dyn Stream<Item = EventResult<Vec<Box<dyn Event>>>> + Send>>
```

Type alias for event batch streams

---

### EventStream

**Source**: `src/utils.rs`

```rust
type EventStream = Pin<Box<dyn Stream<Item = EventResult<Box<dyn Event>>> + Send>>
```

Type alias for event streams to reduce complexity

---

## Functions (utils)

### add

**Source**: `src/utils.rs`

```rust
pub fn add(&mut self, event: Box<dyn Event>) -> Option<Vec<Box<dyn Event>>>
```

Add an event to the current batch

---

### avg_processing_time

**Source**: `src/utils.rs`

```rust
pub async fn avg_processing_time(&self) -> Duration
```

Get average processing time

---

### batch_events

**Source**: `src/utils.rs`

```rust
pub fn batch_events(stream: EventStream, batch_size: usize) -> EventBatchStream
```

Batch events into groups of specified size

---

### can_process

**Source**: `src/utils.rs`

```rust
pub async fn can_process(&self) -> bool
```

Check if we can process another event

---

### cleanup

**Source**: `src/utils.rs`

```rust
pub async fn cleanup(&self)
```

Clean up expired entries

---

### current_rate

**Source**: `src/utils.rs`

```rust
pub async fn current_rate(&self) -> f64
```

Get current rate (events per second)

---

### current_size

**Source**: `src/utils.rs`

```rust
pub fn current_size(&self) -> usize
```

Get the current batch size

---

### deduplicate

**Source**: `src/utils.rs`

```rust
pub fn deduplicate(stream: EventStream, deduplicator: Arc<EventDeduplicator>) -> EventStream
```

Deduplicate events in a stream

---

### error_rate

**Source**: `src/utils.rs`

```rust
pub fn error_rate(&self) -> f64
```

Get error rate as percentage

---

### filter_events

**Source**: `src/utils.rs`

```rust
pub fn filter_events<F>(stream: EventStream, predicate: F) -> EventStream where F: Fn(&dyn Event) -> bool + Send + Sync + Clone + 'static,
```

Filter events based on a predicate

---

### flush

**Source**: `src/utils.rs`

```rust
pub fn flush(&mut self) -> Option<Vec<Box<dyn Event>>>
```

Flush the current batch and return the events

---

### is_duplicate

**Source**: `src/utils.rs`

```rust
pub async fn is_duplicate(&self, event: &dyn Event) -> bool
```

Check if an event is a duplicate

---

### is_empty

**Source**: `src/utils.rs`

```rust
pub fn is_empty(&self) -> bool
```

Check if the batch is empty

---

### map_events

**Source**: `src/utils.rs`

```rust
pub fn map_events<F, T>( stream: EventStream, mapper: F, ) -> Pin<Box<dyn Stream<Item = EventResult<T>> + Send>> where F: Fn(Box<dyn Event>) -> EventResult<T> + Send + 'static, T: Send + 'static,
```

Transform a stream of events using a mapping function

---

### mark_seen

**Source**: `src/utils.rs`

```rust
pub async fn mark_seen(&self, event: &dyn Event)
```

Mark an event as seen

---

### new

**Source**: `src/utils.rs`

```rust
pub fn new(prefix: String) -> Self
```

Create a new ID generator with a prefix

---

### new

**Source**: `src/utils.rs`

```rust
pub fn new(batch_size: usize, timeout: Duration) -> Self
```

Create a new event batcher

---

### new

**Source**: `src/utils.rs`

```rust
pub fn new(ttl: Duration, cleanup_interval: Duration) -> Self
```

Create a new deduplicator with TTL for seen events

---

### new

**Source**: `src/utils.rs`

```rust
pub fn new(max_rate: u64, window: Duration) -> Self
```

Create a new rate limiter

---

### next

**Source**: `src/utils.rs`

```rust
pub fn next(&self) -> String
```

Generate a new unique ID

---

### next_with_context

**Source**: `src/utils.rs`

```rust
pub fn next_with_context(&self, context: &str) -> String
```

Generate an ID with additional context

---

### processing_time_percentiles

**Source**: `src/utils.rs`

```rust
pub async fn processing_time_percentiles(&self, percentiles: &[f64]) -> Vec<Duration>
```

Get processing time percentiles

---

### rate_limit

**Source**: `src/utils.rs`

```rust
pub fn rate_limit(stream: EventStream, rate_limiter: Arc<RateLimiter>) -> EventStream
```

Add rate limiting to an event stream

---

### record_error

**Source**: `src/utils.rs`

```rust
pub fn record_error(&self)
```

Record an error

---

### record_event

**Source**: `src/utils.rs`

```rust
pub async fn record_event(&self)
```

Record that an event was processed

---

### record_processing_time

**Source**: `src/utils.rs`

```rust
pub async fn record_processing_time(&self, duration: Duration)
```

Record an event processing time

---

### reset

**Source**: `src/utils.rs`

```rust
pub async fn reset(&self)
```

Reset all metrics

---

### should_flush

**Source**: `src/utils.rs`

```rust
pub fn should_flush(&self) -> bool
```

Check if the batch should be flushed

---

### start_cleanup_task

**Source**: `src/utils.rs`

```rust
pub fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()>
```

Start automatic cleanup task

---

### summary

**Source**: `src/utils.rs`

```rust
pub async fn summary(&self) -> MetricsSummary
```

Get metrics summary

---

### total_errors

**Source**: `src/utils.rs`

```rust
pub fn total_errors(&self) -> u64
```

Get total errors

---

### total_events

**Source**: `src/utils.rs`

```rust
pub fn total_events(&self) -> u64
```

Get total events processed

---

### wait_for_capacity

**Source**: `src/utils.rs`

```rust
pub async fn wait_for_capacity(&self)
```

Wait until we can process the next event

---

## Functions (types)

### block_id

**Source**: `src/types.rs`

```rust
pub fn block_id(&self) -> String
```

Get the block identifier

---

### chain_id

**Source**: `src/types.rs`

```rust
pub fn chain_id(&self) -> String
```

Get the chain identifier

---

### error_rate

**Source**: `src/types.rs`

```rust
pub fn error_rate(&self) -> f64
```

Calculate error rate as a percentage

---

### extract_data

**Source**: `src/types.rs`

```rust
pub fn extract_data<T>(&self) -> Result<T, serde_json::Error> where T: for<'de> Deserialize<'de>,
```

Extract typed data from the event

---

### get_custom

**Source**: `src/types.rs`

```rust
pub fn get_custom<T>(&self, key: &str) -> Option<T> where T: for<'de> Deserialize<'de>,
```

Get custom metadata value

---

### is_healthy

**Source**: `src/types.rs`

```rust
pub fn is_healthy(&self) -> bool
```

Check if stream is healthy (low error rate, recent activity)

---

### new

**Source**: `src/types.rs`

```rust
pub fn new(id: String, kind: EventKind, source: String) -> Self
```

Create new event metadata with minimal required fields

---

### new

**Source**: `src/types.rs`

```rust
pub fn new(id: String, kind: EventKind, data: serde_json::Value) -> Self
```

Create a new generic event

---

### new

**Source**: `src/types.rs`

```rust
pub fn new(id: String, name: String, source_type: String, endpoint: String) -> Self
```

Create new stream info

---

### record_error

**Source**: `src/types.rs`

```rust
pub fn record_error(&mut self)
```

Record an error

---

### set_active

**Source**: `src/types.rs`

```rust
pub fn set_active(&mut self, active: bool)
```

Mark stream as active

---

### transaction_id

**Source**: `src/types.rs`

```rust
pub fn transaction_id(&self) -> Option<String>
```

Get the transaction identifier if available

---

### update_data

**Source**: `src/types.rs`

```rust
pub fn update_data<T: Serialize>(&mut self, data: T) -> Result<(), serde_json::Error>
```

Update the event data

---

### update_last_event

**Source**: `src/types.rs`

```rust
pub fn update_last_event(&mut self)
```

Update last event timestamp

---

### with_chain_data

**Source**: `src/types.rs`

```rust
pub fn with_chain_data(mut self, chain_data: ChainData) -> Self
```

Add chain-specific data

---

### with_custom

**Source**: `src/types.rs`

```rust
pub fn with_custom<T: Serialize>(mut self, key: String, value: T) -> Self
```

Add custom metadata field

---

### with_metadata

**Source**: `src/types.rs`

```rust
pub fn with_metadata(metadata: EventMetadata, data: serde_json::Value) -> Self
```

Create a new generic event with metadata

---

### with_source

**Source**: `src/types.rs`

```rust
pub fn with_source( id: String, kind: EventKind, source: String, data: serde_json::Value, ) -> Self
```

Create a new generic event with source

---

### with_timestamp

**Source**: `src/types.rs`

```rust
pub fn with_timestamp( id: String, kind: EventKind, source: String, timestamp: DateTime<Utc>, ) -> Self
```

Create new event metadata with specified timestamp

---


---

*This documentation was automatically generated from the source code.*