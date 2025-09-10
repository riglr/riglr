# riglr-streams

{{#include ../../../riglr-streams/README.md}}

## API Reference

## Key Components

> The most important types and functions in this crate.

### `StreamManager`

Manages multiple streams and routes events to handlers

[→ Full documentation](#structs)

### `Stream`

Core trait that all streams must implement

[→ Full documentation](#traits)

### `ComposableStream`

Trait for composable streams

[→ Full documentation](#traits)

---

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Functions](#functions)
- [Type Aliases](#type-aliases)

### Structs

> Core data structures and types.

#### `BackpressureConfig`

Backpressure configuration

---

#### `BatchConfig`

Batch processing configuration

---

#### `BatchProcessor`

Event batch processor with configurable batching

---

#### `BatchedStream`

Stream that batches events

---

#### `BinanceConfig`

Binance stream configuration

---

#### `BinanceStream`

Binance WebSocket stream implementation

---

#### `BinanceStreamEvent`

Binance streaming event

---

#### `BitcoinBlock`

Bitcoin block data

---

#### `BitcoinTransaction`

Bitcoin transaction data

---

#### `CircuitBreaker`

Circuit breaker for stream connections

---

#### `ComponentHealth`

Individual component health

---

#### `CompositeCondition`

Composite condition that combines multiple conditions

---

#### `ConnectionConfig`

Connection configuration

---

#### `ConnectionHealth`

Connection health metrics

---

#### `ConnectionManager`

Connection manager with automatic reconnection

---

#### `ConnectionPool`

Connection pool for managing multiple connections

---

#### `CustomCondition`

Custom condition with closure

---

#### `DebouncedStream`

Stream that debounces events

---

#### `DynamicStreamWrapper`

Wrapper to make any Stream implement DynamicStream

---

#### `DynamicStreamedEvent`

Dynamic event wrapper for type-erased Events with streaming metadata
This is a separate implementation that doesn't rely on the generic StreamedEvent<T>

---

#### `EventKindMatcher`

Event kind matcher

---

#### `EventTriggerBuilder`

Builder for event-triggered tools

---

#### `EventTriggeredTool`

Wrapper that makes any Tool event-triggered

---

#### `EvmStreamConfig`

EVM stream configuration

---

#### `EvmStreamEvent`

EVM streaming event

---

#### `EvmWebSocketStream`

EVM WebSocket stream implementation

---

#### `FeeEstimate`

Bitcoin fee estimate data

---

#### `FilteredStream`

Stream that filters events based on a predicate

---

#### `FlowController`

Flow control manager with backpressure handling

---

#### `GeyserConfig`

Geyser stream configuration

---

#### `GlobalMetrics`

Global metrics aggregator

---

#### `GroupedStream`

Stream that groups events by a key function

---

#### `HandlerMetrics`

Handler metrics

---

#### `HealthAlert`

Health alert

---

#### `HealthMonitor`

Health monitor for streams

---

#### `HealthSnapshot`

Health snapshot at a point in time

---

#### `HealthThresholds`

Health check thresholds

---

#### `KlineData`

Kline/candlestick stream data

---

#### `KlineDetails`

Detailed kline/candlestick data

---

#### `LoggingEventHandler`

Simple event handler for testing

---

#### `MappedStream`

Stream that maps events through a transformation

---

#### `Matcher`

Empty struct to implement EventMatcher

---

#### `MempoolConfig`

Mempool configuration

---

#### `MempoolSpaceStream`

Mempool.space WebSocket stream implementation

---

#### `MempoolStats`

Mempool statistics data

---

#### `MempoolStreamEvent`

Mempool streaming event

---

#### `MergedStream`

Stream that merges events from two streams

---

#### `MetricsCollector`

Metrics collector for the streaming system

---

#### `MetricsConfig`

Performance monitoring configuration

---

#### `MetricsTimer`

Metrics timer for measuring execution time

---

#### `MockConfig`

Configuration for mock stream

---

#### `MockStream`

Mock stream that generates protocol-specific events for testing

---

#### `MovingAverageStream`

Moving average calculator for financial streams

---

#### `MultiChainEvmManager`

Multi-chain EVM stream manager

---

#### `OrderBookData`

Order book depth update data

---

#### `PatternMatcher`

Complex event processing pattern matcher

---

#### `ResilientConfig`

Configuration for resilient WebSocket connections

---

#### `ResilientWebSocketBuilder`

Builder for creating resilient WebSocket connectors

---

#### `ResilientWebSocketConnector`

Resilient WebSocket connector that handles reconnection and error recovery

---

#### `RetryPolicy`

Retry policy for operations

---

#### `SkipStream`

Stream that skips N events

---

#### `SolanaGeyserStream`

Solana Geyser stream implementation using WebSocket

---

#### `SourceMatcher`

Source matcher (matches event source)

---

#### `StatefulProcessor`

Stateful event processor with checkpointing

---

#### `StreamClientConfig`

Comprehensive streaming client configuration

---

#### `StreamHealth`

Health status of a stream

Enhanced with events-core integration for better observability

---

#### `StreamManagerBuilder`

Builder for StreamManager with configuration loading

---

#### `StreamManagerConfig`

Configuration structure for loading from TOML

---

#### `StreamMetadata`

Temporary StreamMetadata for migration phase

---

#### `StreamMetrics`

Performance metrics for a stream

---

#### `StreamedEvent`

Wrapper that adds streaming metadata to any Event through composition

---

#### `StreamingToolWorker`

Extended worker that can process streaming events

---

#### `TakeStream`

Stream that takes only N events

---

#### `ThrottledStream`

Stream that throttles events

---

#### `TickerData`

24-hour ticker price change statistics data

---

#### `TimestampRangeMatcher`

Timestamp range matcher

---

#### `TradeData`

Individual trade execution data

---

#### `TransactionEvent`

Simple transaction event for fallback cases

---

#### `VWAPStream`

Volume-weighted average price (VWAP) stream

---

#### `Window`

Window state for managing event windows

---

#### `WindowManager`

Window manager for handling different window types

---

#### `WindowedStream`

Stream that windows events by time

---

### Enums

> Enumeration types for representing variants.

#### `AlertSeverity`

Alert severity levels

**Variants:**

- `Info`
  - Informational alerts for general status updates
- `Warning`
  - Warning alerts for potential issues that need attention
- `Error`
  - Error alerts for failures that occurred but system can continue
- `Critical`
  - Critical alerts for severe issues requiring immediate attention

---

#### `BackoffStrategy`

Backoff strategy for retries

**Variants:**

- `Fixed`
  - Fixed delay between all retry attempts
- `Linear`
  - Linear backoff with configurable increment
- `Exponential`
  - Exponential backoff with configurable factor

---

#### `BackpressureStrategy`

Backpressure handling strategies

**Variants:**

- `Block`
  - Block and wait (default) - guarantees delivery but may cause slowdowns
- `Drop`
  - Drop messages when buffer is full - fastest but may lose data
- `Retry`
  - Retry with exponential backoff, then drop
- `Adaptive`
  - Adaptive strategy that switches based on load

---

#### `BinanceEventData`

Binance event data types

**Variants:**

- `Ticker`
  - 24hr ticker statistics
- `OrderBook`
  - Order book update
- `Trade`
  - Individual trade
- `Kline`
  - K-line/candlestick
- `Unknown`
  - Unknown event type

---

#### `BitcoinNetwork`

Bitcoin network type

**Variants:**

- `Mainnet`
  - Bitcoin mainnet
- `Testnet`
  - Bitcoin testnet
- `Signet`
  - Bitcoin signet

---

#### `ChainId`

EVM Chain ID enumeration

**Variants:**

- `Ethereum`
  - Ethereum mainnet
- `Polygon`
  - Polygon (Matic) network
- `BSC`
  - Binance Smart Chain
- `Arbitrum`
  - Arbitrum layer 2
- `Optimism`
  - Optimism layer 2
- `Avalanche`
  - Avalanche C-Chain
- `Base`
  - Base layer 2

---

#### `ConditionCombinator`

How to combine multiple conditions

**Variants:**

- `All`
  - All conditions must match
- `Any`
  - At least one condition must match
- `None`
  - No conditions must match

---

#### `ConfigError`

Configuration errors

**Variants:**

- `Invalid`
  - Invalid configuration parameter or value
- `Environment`
  - Environment variable parsing or access error

---

#### `ConnectionState`

Connection state

**Variants:**

- `Connected`
  - Connection is healthy and active
- `Connecting`
  - Connection is being established
- `Reconnecting`
  - Connection failed, attempting to reconnect
- `Failed`
  - Connection is permanently failed (circuit breaker open)
- `Disconnected`
  - Connection is explicitly disconnected

---

#### `EventPattern`

Complex event processing pattern matcher

**Variants:**

- `Single`
  - Match a single event type
- `Sequence`
  - Match a sequence of events
- `Within`
  - Match events within a time window
- `Any`
  - Match any of the patterns
- `All`
  - Match all patterns

---

#### `EvmEventType`

EVM-specific event types

**Variants:**

- `PendingTransaction`
  - Pending transaction
- `NewBlock`
  - New block
- `ContractEvent`
  - Contract event log

---

#### `HandlerExecutionMode`

Execution mode for event handlers

**Variants:**

- `Sequential`
  - Execute handlers sequentially in order (preserves ordering guarantees)
- `Concurrent`
  - Execute handlers concurrently (better throughput, no ordering guarantees)
- `ConcurrentBounded`
  - Execute handlers concurrently with a limit on parallelism

---

#### `HealthStatus`

Health status levels

**Variants:**

- `Healthy`
  - Everything is working normally
- `Degraded`
  - Some issues but still functional
- `Unhealthy`
  - Critical issues, may not be functional
- `Down`
  - Not operational

---

#### `ManagerState`

Represents the current state of the StreamManager

**Variants:**

- `Idle`
  - Manager is idle and not processing any streams
- `Starting`
  - Manager is in the process of starting up
- `Running`
  - Manager is actively running and processing streams
- `Stopping`
  - Manager is in the process of stopping
- `Stopped`
  - Manager has stopped and is no longer processing streams

---

#### `MempoolEventData`

Mempool event data types

**Variants:**

- `Transaction`
  - Bitcoin transaction
- `Block`
  - Bitcoin block
- `FeeEstimate`
  - Fee estimate update
- `MempoolStats`
  - Mempool statistics
- `Unknown`
  - Unknown event

---

#### `MempoolEventType`

Mempool event types

**Variants:**

- `Block`
  - Block-related event
- `Transaction`
  - Transaction-related event
- `Stats`
  - Statistics update event

---

#### `StreamConfig`

Configuration for a stream

**Variants:**

- `SolanaGeyser`
  - Solana Geyser stream configuration for monitoring Solana blockchain events
- `EvmWebSocket`
  - EVM WebSocket stream configuration for monitoring EVM-compatible blockchain events
- `Binance`
  - Binance stream configuration for monitoring Binance exchange data
- `MempoolSpace`
  - Mempool.space stream configuration for monitoring Bitcoin mempool data

---

#### `StreamError`

Comprehensive error type for streaming operations

This extends EventError with streaming-specific error variants while
maintaining compatibility with the events-core error hierarchy.

**Variants:**

- `Connection`
  - Connection-related errors
- `Configuration`
  - Configuration errors
- `ConfigurationInvalid`
  - Invalid configuration
- `Authentication`
  - Authentication failures
- `RateLimit`
  - Rate limiting errors
- `Parse`
  - Data parsing errors
- `ResourceExhausted`
  - Resource exhaustion
- `AlreadyRunning`
  - Stream already running
- `NotRunning`
  - Stream not running
- `Channel`
  - Channel errors
- `Timeout`
  - Timeout errors
- `Processing`
  - Processing errors
- `Backpressure`
  - Backpressure errors
- `Internal`
  - Internal errors

---

#### `WindowType`

Window types for time-based processing

**Variants:**

- `Tumbling`
  - Fixed-size tumbling windows
- `Sliding`
  - Sliding windows with overlap
- `Session`
  - Session windows that close after inactivity
- `Count`
  - Count-based windows

---

### Traits

> Trait definitions for implementing common behaviors.

#### `DynamicStream`

Type-erased stream for heterogeneous stream management

**Methods:**

- `start_dynamic()`
  - Start the stream
- `stop_dynamic()`
  - Stop the stream
- `is_running_dynamic()`
  - Check if running
- `health_dynamic()`
  - Get health status
- `name_dynamic()`
  - Get stream name
- `subscribe_dynamic()`
  - Subscribe to events as type-erased values

---

#### `EventCondition`

Trait for event conditions

**Methods:**

- `matches()`
  - Check if an event matches this condition
- `description()`
  - Get condition description

---

#### `EventHandler`

Trait for handling stream events

**Methods:**

- `should_handle()`
  - Check if this handler should process the event
- `handle()`
  - Process the event
- `name()`
  - Get handler name for logging

---

#### `EventMatcher`

Helper trait for pattern matching

**Methods:**

- `event_kind()`
  - Match event kind
- `source()`
  - Match source
- `timestamp_range()`
  - Match timestamp range
- `all()`
  - Combine conditions with ALL
- `any()`
  - Combine conditions with ANY

---

#### `IntoDynamicStreamedEvent`

Helper trait for creating DynamicStreamedEvent from Box<dyn Event>

**Methods:**

- `with_stream_metadata()`
  - Wrap this boxed event with streaming metadata
- `with_default_stream_metadata()`
  - Wrap this boxed event with default streaming metadata

---

#### `IntoStreamedEvent`

Helper trait to easily wrap events with streaming metadata

**Methods:**

- `with_stream_metadata()`
  - Wrap this event with streaming metadata
- `with_default_stream_metadata()`
  - Wrap this event with default streaming metadata

---

#### `StreamEvent`

Extension trait for events with streaming context

**Methods:**

- `stream_metadata()`
  - Get stream-specific metadata
- `stream_source()`
  - Get the stream source identifier
- `received_at()`
  - Get when the event was received
- `sequence_number()`
  - Get the sequence number if available

---

#### `StreamingTool`

Generic tool trait for event-triggered execution

**Methods:**

- `execute()`
  - Execute the tool
- `name()`
  - Get tool name

---

### Functions

> Standalone functions and utilities.

#### `as_event`

Helper to convert Any to Event by trying all known event types

---

#### `from_config_file`

Helper to load and build a StreamManager from a TOML file

---

#### `from_env`

Helper to load and build a StreamManager from environment variables

---

### Type Aliases

#### `SolanaStreamEvent`

Type alias for streamed Solana events using protocol-specific parsing

**Type:** ``

---

#### `StreamResult`

Result type for streaming operations

**Type:** `<T, >`

---
