# riglr-streams API Reference

Comprehensive API documentation for the `riglr-streams` crate.

## Table of Contents

### Enums

- [`AlertSeverity`](#alertseverity)
- [`BackoffStrategy`](#backoffstrategy)
- [`BackpressureStrategy`](#backpressurestrategy)
- [`BinanceEventData`](#binanceeventdata)
- [`BitcoinNetwork`](#bitcoinnetwork)
- [`BufferStrategy`](#bufferstrategy)
- [`ChainId`](#chainid)
- [`ConditionCombinator`](#conditioncombinator)
- [`ConfigError`](#configerror)
- [`ConnectionState`](#connectionstate)
- [`EventPattern`](#eventpattern)
- [`EvmEventType`](#evmeventtype)
- [`HandlerExecutionMode`](#handlerexecutionmode)
- [`HealthStatus`](#healthstatus)
- [`ManagerState`](#managerstate)
- [`MempoolEventData`](#mempooleventdata)
- [`MempoolEventType`](#mempooleventtype)
- [`MergedEvent`](#mergedevent)
- [`MevType`](#mevtype)
- [`StreamConfig`](#streamconfig)
- [`StreamError`](#streamerror)
- [`WindowType`](#windowtype)

### Structs

- [`BackpressureConfig`](#backpressureconfig)
- [`BatchConfig`](#batchconfig)
- [`BatchEvent`](#batchevent)
- [`BatchEvent`](#batchevent)
- [`BatchProcessor`](#batchprocessor)
- [`BatchedStream`](#batchedstream)
- [`BatchedStream`](#batchedstream)
- [`BinanceConfig`](#binanceconfig)
- [`BinanceStream`](#binancestream)
- [`BinanceStreamEvent`](#binancestreamevent)
- [`BitcoinBlock`](#bitcoinblock)
- [`BitcoinTransaction`](#bitcointransaction)
- [`BollingerBands`](#bollingerbands)
- [`BollingerBandsStream`](#bollingerbandsstream)
- [`BufferedStream`](#bufferedstream)
- [`CatchErrorStream`](#catcherrorstream)
- [`CircuitBreaker`](#circuitbreaker)
- [`CircuitBreaker`](#circuitbreaker)
- [`CombineLatestStream`](#combinelateststream)
- [`ComponentHealth`](#componenthealth)
- [`CompositeCondition`](#compositecondition)
- [`ConnectionConfig`](#connectionconfig)
- [`ConnectionHealth`](#connectionhealth)
- [`ConnectionManager`](#connectionmanager)
- [`ConnectionPool`](#connectionpool)
- [`CustomCondition`](#customcondition)
- [`DebouncedStream`](#debouncedstream)
- [`DynamicStreamWrapper`](#dynamicstreamwrapper)
- [`DynamicStreamedEvent`](#dynamicstreamedevent)
- [`EmaStream`](#emastream)
- [`EnhancedStream`](#enhancedstream)
- [`EventKindMatcher`](#eventkindmatcher)
- [`EventTriggerBuilder`](#eventtriggerbuilder)
- [`EventTriggeredTool`](#eventtriggeredtool)
- [`EvmStreamConfig`](#evmstreamconfig)
- [`EvmStreamEvent`](#evmstreamevent)
- [`EvmWebSocketStream`](#evmwebsocketstream)
- [`FeeEstimate`](#feeestimate)
- [`FilterMapStream`](#filtermapstream)
- [`FilteredStream`](#filteredstream)
- [`FinancialEvent`](#financialevent)
- [`FlowController`](#flowcontroller)
- [`GasPriceEstimate`](#gaspriceestimate)
- [`GasPriceOracleStream`](#gaspriceoraclestream)
- [`GeyserConfig`](#geyserconfig)
- [`GlobalMetrics`](#globalmetrics)
- [`GuaranteedDeliveryStream`](#guaranteeddeliverystream)
- [`HandlerMetrics`](#handlermetrics)
- [`HealthAlert`](#healthalert)
- [`HealthMonitor`](#healthmonitor)
- [`HealthSnapshot`](#healthsnapshot)
- [`HealthThresholds`](#healththresholds)
- [`KlineData`](#klinedata)
- [`KlineDetails`](#klinedetails)
- [`LiquidityPoolStream`](#liquiditypoolstream)
- [`LoggingEventHandler`](#loggingeventhandler)
- [`MappedStream`](#mappedstream)
- [`Matcher`](#matcher)
- [`MempoolConfig`](#mempoolconfig)
- [`MempoolSpaceStream`](#mempoolspacestream)
- [`MempoolStats`](#mempoolstats)
- [`MempoolStreamEvent`](#mempoolstreamevent)
- [`MergeAllStream`](#mergeallstream)
- [`MergedStream`](#mergedstream)
- [`MetricsCollector`](#metricscollector)
- [`MetricsCollector`](#metricscollector)
- [`MetricsConfig`](#metricsconfig)
- [`MetricsSnapshot`](#metricssnapshot)
- [`MetricsTimer`](#metricstimer)
- [`MevDetectionStream`](#mevdetectionstream)
- [`MockConfig`](#mockconfig)
- [`MockStream`](#mockstream)
- [`MomentumStream`](#momentumstream)
- [`MovingAverageStream`](#movingaveragestream)
- [`MultiChainEvmManager`](#multichainevmmanager)
- [`OrderBookData`](#orderbookdata)
- [`OrderBookImbalanceStream`](#orderbookimbalancestream)
- [`PatternMatcher`](#patternmatcher)
- [`PerformanceStats`](#performancestats)
- [`PoolState`](#poolstate)
- [`ResilientStream`](#resilientstream)
- [`RetryPolicy`](#retrypolicy)
- [`RsiStream`](#rsistream)
- [`ScanEvent`](#scanevent)
- [`ScanStream`](#scanstream)
- [`SkipStream`](#skipstream)
- [`SolanaGeyserStream`](#solanageyserstream)
- [`SourceMatcher`](#sourcematcher)
- [`StatefulProcessor`](#statefulprocessor)
- [`StreamClientConfig`](#streamclientconfig)
- [`StreamHealth`](#streamhealth)
- [`StreamManager`](#streammanager)
- [`StreamManagerBuilder`](#streammanagerbuilder)
- [`StreamManagerConfig`](#streammanagerconfig)
- [`StreamMetadata`](#streammetadata)
- [`StreamMetrics`](#streammetrics)
- [`StreamMetrics`](#streammetrics)
- [`StreamPipeline`](#streampipeline)
- [`StreamedEvent`](#streamedevent)
- [`StreamingToolWorker`](#streamingtoolworker)
- [`TakeStream`](#takestream)
- [`TestPriceEvent`](#testpriceevent)
- [`ThrottledStream`](#throttledstream)
- [`TickerData`](#tickerdata)
- [`TimestampRangeMatcher`](#timestamprangematcher)
- [`TradeData`](#tradedata)
- [`TransactionEvent`](#transactionevent)
- [`TypeErasedEvent`](#typeerasedevent)
- [`VwapStream`](#vwapstream)
- [`Window`](#window)
- [`WindowManager`](#windowmanager)
- [`ZipStream`](#zipstream)

### Functions

- [`acquire_permit`](#acquire_permit)
- [`active_chains`](#active_chains)
- [`add_chain`](#add_chain)
- [`add_condition`](#add_condition)
- [`add_event`](#add_event)
- [`add_event`](#add_event)
- [`add_event_handler`](#add_event_handler)
- [`add_stream`](#add_stream)
- [`add_stream`](#add_stream)
- [`after`](#after)
- [`as_event`](#as_event)
- [`as_event_extended`](#as_event_extended)
- [`attempt_connect`](#attempt_connect)
- [`batch_timeout`](#batch_timeout)
- [`before`](#before)
- [`build`](#build)
- [`build`](#build)
- [`build`](#build)
- [`calculate_delay`](#calculate_delay)
- [`calculate_imbalance`](#calculate_imbalance)
- [`checkpoint`](#checkpoint)
- [`combinator`](#combinator)
- [`combine_latest`](#combine_latest)
- [`condition`](#condition)
- [`connect`](#connect)
- [`connect_all`](#connect_all)
- [`connect_timeout`](#connect_timeout)
- [`current_health`](#current_health)
- [`development`](#development)
- [`disconnect`](#disconnect)
- [`downcast`](#downcast)
- [`elapsed_ms`](#elapsed_ms)
- [`execute`](#execute)
- [`execution_count`](#execution_count)
- [`export_json`](#export_json)
- [`export_prometheus`](#export_prometheus)
- [`flush_batch`](#flush_batch)
- [`from`](#from)
- [`from_config_file`](#from_config_file)
- [`from_env`](#from_env)
- [`from_env`](#from_env)
- [`from_env`](#from_env)
- [`from_event`](#from_event)
- [`from_event_error`](#from_event_error)
- [`from_toml_file`](#from_toml_file)
- [`get_all_handler_metrics`](#get_all_handler_metrics)
- [`get_all_metrics`](#get_all_metrics)
- [`get_all_stream_metrics`](#get_all_stream_metrics)
- [`get_connection`](#get_connection)
- [`get_global_metrics`](#get_global_metrics)
- [`get_handler_metrics`](#get_handler_metrics)
- [`get_healthy_connection`](#get_healthy_connection)
- [`get_metrics`](#get_metrics)
- [`get_metrics`](#get_metrics)
- [`get_state`](#get_state)
- [`get_stats`](#get_stats)
- [`get_stream_metrics`](#get_stream_metrics)
- [`get_stream_metrics`](#get_stream_metrics)
- [`health`](#health)
- [`health`](#health)
- [`health_history`](#health_history)
- [`high_performance`](#high_performance)
- [`high_watermark`](#high_watermark)
- [`inner`](#inner)
- [`inner`](#inner)
- [`inner`](#inner)
- [`inner_mut`](#inner_mut)
- [`into_inner`](#into_inner)
- [`is_chain_configured`](#is_chain_configured)
- [`is_open`](#is_open)
- [`is_retriable`](#is_retriable)
- [`is_stream_running`](#is_stream_running)
- [`keepalive_interval`](#keepalive_interval)
- [`list_streams`](#list_streams)
- [`low_latency`](#low_latency)
- [`low_watermark`](#low_watermark)
- [`mark_disconnected`](#mark_disconnected)
- [`match_event`](#match_event)
- [`merge_all`](#merge_all)
- [`metrics_collector`](#metrics_collector)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`pending_count`](#pending_count)
- [`permanent_connection`](#permanent_connection)
- [`pool_health`](#pool_health)
- [`process_events`](#process_events)
- [`rate_limited`](#rate_limited)
- [`record_dropped_event`](#record_dropped_event)
- [`record_error`](#record_error)
- [`record_event`](#record_event)
- [`record_failure`](#record_failure)
- [`record_handler_execution`](#record_handler_execution)
- [`record_reconnection`](#record_reconnection)
- [`record_stream_event`](#record_stream_event)
- [`record_success`](#record_success)
- [`register`](#register)
- [`register_all_configured_chains`](#register_all_configured_chains)
- [`release_permit`](#release_permit)
- [`reliable`](#reliable)
- [`remove_chain`](#remove_chain)
- [`remove_state`](#remove_state)
- [`remove_stream`](#remove_stream)
- [`report_interval`](#report_interval)
- [`request_timeout`](#request_timeout)
- [`reset_metrics`](#reset_metrics)
- [`retriable_connection`](#retriable_connection)
- [`retry_after`](#retry_after)
- [`retry_base_delay`](#retry_base_delay)
- [`retry_max_delay`](#retry_max_delay)
- [`serialize`](#serialize)
- [`set_execution_mode`](#set_execution_mode)
- [`single`](#single)
- [`single`](#single)
- [`slow_threshold`](#slow_threshold)
- [`source_stream`](#source_stream)
- [`start`](#start)
- [`start`](#start)
- [`start`](#start)
- [`start_all`](#start_all)
- [`start_stream`](#start_stream)
- [`start_with_collector`](#start_with_collector)
- [`state`](#state)
- [`state`](#state)
- [`stop`](#stop)
- [`stop`](#stop)
- [`stop_all`](#stop_all)
- [`stop_all`](#stop_all)
- [`stop_stream`](#stop_stream)
- [`stream_meta`](#stream_meta)
- [`stream_meta`](#stream_meta)
- [`to_event_error`](#to_event_error)
- [`update_queue_size`](#update_queue_size)
- [`update_rates`](#update_rates)
- [`update_state`](#update_state)
- [`validate`](#validate)
- [`validate`](#validate)
- [`validate`](#validate)
- [`validate`](#validate)
- [`window_duration`](#window_duration)
- [`with_check_interval`](#with_check_interval)
- [`with_combinator`](#with_combinator)
- [`with_condition`](#with_condition)
- [`with_connection`](#with_connection)
- [`with_execution_mode`](#with_execution_mode)
- [`with_execution_mode`](#with_execution_mode)
- [`with_failure_threshold`](#with_failure_threshold)
- [`with_metrics`](#with_metrics)
- [`with_metrics_collector`](#with_metrics_collector)
- [`with_stream_manager`](#with_stream_manager)
- [`with_success_threshold`](#with_success_threshold)
- [`with_thresholds`](#with_thresholds)
- [`with_timeout`](#with_timeout)
- [`zip`](#zip)

### Traits

- [`AsNumeric`](#asnumeric)
- [`ComposableStream`](#composablestream)
- [`DynamicStream`](#dynamicstream)
- [`EnhancedStreamExt`](#enhancedstreamext)
- [`EventCondition`](#eventcondition)
- [`EventConversion`](#eventconversion)
- [`EventHandler`](#eventhandler)
- [`EventMatcher`](#eventmatcher)
- [`FinancialStreamExt`](#financialstreamext)
- [`IntoDynamicStreamedEvent`](#intodynamicstreamedevent)
- [`IntoStreamedEvent`](#intostreamedevent)
- [`PerformanceStreamExt`](#performancestreamext)
- [`ResilienceStreamExt`](#resiliencestreamext)
- [`Stream`](#stream)
- [`StreamEvent`](#streamevent)
- [`StreamingTool`](#streamingtool)

### Type Aliases

- [`SolanaStreamEvent`](#solanastreamevent)

## Enums

### AlertSeverity

**Source**: `production/health.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
```

```rust
pub enum AlertSeverity { Info, Warning, Error, Critical, }
```

Alert severity levels

**Variants**:

- `Info`
- `Warning`
- `Error`
- `Critical`

---

### BackoffStrategy

**Source**: `production/resilience.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub enum BackoffStrategy { /// Fixed delay Fixed, /// Linear backoff Linear { increment: Duration }, /// Exponential backoff Exponential { factor: f64 }, }
```

Backoff strategy for retries

**Variants**:

- `Fixed`
- `Linear`
- `Exponential`

---

### BackpressureStrategy

**Source**: `core/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
```

```rust
pub enum BackpressureStrategy { /// Block and wait (default) - guarantees delivery but may cause slowdowns Block, /// Drop messages when buffer is full - fastest but may lose data Drop, /// Retry with exponential backoff, then drop Retry { max_attempts: usize, base_wait_ms: u64 }, /// Adaptive strategy that switches based on load Adaptive, }
```

Backpressure handling strategies

**Variants**:

- `Block`
- `Drop`
- `Retry`
- `Adaptive`

---

### BinanceEventData

**Source**: `external/binance.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize)]
```

```rust
pub enum BinanceEventData { /// 24hr ticker statistics Ticker(TickerData), /// Order book update OrderBook(OrderBookData), /// Individual trade Trade(TradeData), /// K-line/candlestick Kline(KlineData), /// Unknown event type Unknown(serde_json::Value), }
```

Binance event data types

**Variants**:

- `Ticker(TickerData)`
- `OrderBook(OrderBookData)`
- `Trade(TradeData)`
- `Kline(KlineData)`
- `Unknown(serde_json::Value)`

---

### BitcoinNetwork

**Source**: `external/mempool.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
```

```rust
pub enum BitcoinNetwork { Mainnet, Testnet, Signet, }
```

Bitcoin network type

**Variants**:

- `Mainnet`
- `Testnet`
- `Signet`

---

### BufferStrategy

**Source**: `core/operators.rs`

**Attributes**:
```rust
#[derive(Clone, Debug)]
```

```rust
pub enum BufferStrategy { /// Buffer up to N events Count(usize), /// Buffer for duration Time(Duration), /// Buffer up to N events or duration, whichever comes first CountOrTime(usize, Duration), /// Sliding window of N events SlidingWindow(usize), }
```

Buffer strategies for buffered streams

**Variants**:

- `Count(usize)`
- `Time(Duration)`
- `CountOrTime(usize, Duration)`
- `SlidingWindow(usize)`

---

### ChainId

**Source**: `evm/websocket.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
```

```rust
pub enum ChainId { Ethereum = 1, Polygon = 137, BSC = 56, Arbitrum = 42161, Optimism = 10, Avalanche = 43114, Base = 8453, }
```

EVM Chain ID enumeration

**Variants**:

- `Ethereum`
- `Polygon`
- `BSC`
- `Arbitrum`
- `Optimism`
- `Avalanche`
- `Base`

---

### ConditionCombinator

**Source**: `tools/condition.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
```

```rust
pub enum ConditionCombinator { /// All conditions must match All, /// At least one condition must match Any, /// No conditions must match None, }
```

How to combine multiple conditions

**Variants**:

- `All`
- `Any`
- `None`

---

### ConfigError

**Source**: `core/config.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum ConfigError { #[error("Invalid configuration: {0}")] Invalid(String), #[error("Environment variable error: {0}")] Environment(String), }
```

Configuration errors

**Variants**:

- `Invalid(String)`
- `Environment(String)`

---

### ConnectionState

**Source**: `core/connection.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum ConnectionState { /// Connection is healthy and active Connected, /// Connection is being established Connecting, /// Connection failed, attempting to reconnect Reconnecting, /// Connection is permanently failed (circuit breaker open) Failed, /// Connection is explicitly disconnected Disconnected, }
```

Connection state

**Variants**:

- `Connected`
- `Connecting`
- `Reconnecting`
- `Failed`
- `Disconnected`

---

### EventPattern

**Source**: `core/processor.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub enum EventPattern<E> { /// Match a single event type Single(fn(&E) -> bool), /// Match a sequence of events Sequence(Vec<fn(&E) -> bool>), /// Match events within a time window Within { pattern: Box<EventPattern<E>>, duration: Duration }, /// Match any of the patterns Any(Vec<EventPattern<E>>), /// Match all patterns All(Vec<EventPattern<E>>), }
```

Complex event processing pattern matcher

**Variants**:

- `Single(fn(&E)`
- `Sequence(Vec<fn(&E)`
- `Within`
- `Any(Vec<EventPattern<E>>)`
- `All(Vec<EventPattern<E>>)`

---

### EvmEventType

**Source**: `evm/websocket.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize)]
```

```rust
pub enum EvmEventType { /// Pending transaction PendingTransaction, /// New block NewBlock, /// Contract event log ContractEvent, }
```

EVM-specific event types

**Variants**:

- `PendingTransaction`
- `NewBlock`
- `ContractEvent`

---

### HandlerExecutionMode

**Source**: `core/manager.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
```

```rust
pub enum HandlerExecutionMode { /// Execute handlers sequentially in order (preserves ordering guarantees) Sequential, /// Execute handlers concurrently (better throughput, no ordering guarantees) Concurrent, /// Execute handlers concurrently with a limit on parallelism ConcurrentBounded(usize), }
```

Execution mode for event handlers

**Variants**:

- `Sequential`
- `Concurrent`
- `ConcurrentBounded(usize)`

---

### HealthStatus

**Source**: `production/health.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
```

```rust
pub enum HealthStatus { /// Everything is working normally Healthy, /// Some issues but still functional Degraded, /// Critical issues, may not be functional Unhealthy, /// Not operational Down, }
```

Health status levels

**Variants**:

- `Healthy`
- `Degraded`
- `Unhealthy`
- `Down`

---

### ManagerState

**Source**: `core/manager.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum ManagerState { Idle, Starting, Running, Stopping, Stopped, }
```

**Variants**:

- `Idle`
- `Starting`
- `Running`
- `Stopping`
- `Stopped`

---

### MempoolEventData

**Source**: `external/mempool.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub enum MempoolEventData { /// Bitcoin transaction Transaction(BitcoinTransaction), /// Bitcoin block Block(BitcoinBlock), /// Fee estimate update FeeEstimate(FeeEstimate), /// Mempool statistics MempoolStats(MempoolStats), /// Unknown event Unknown(serde_json::Value), }
```

Mempool event data types

**Variants**:

- `Transaction(BitcoinTransaction)`
- `Block(BitcoinBlock)`
- `FeeEstimate(FeeEstimate)`
- `MempoolStats(MempoolStats)`
- `Unknown(serde_json::Value)`

---

### MempoolEventType

**Source**: `external/mempool.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize)]
```

```rust
pub enum MempoolEventType { Block, Transaction, Stats, }
```

Mempool event types

**Variants**:

- `Block`
- `Transaction`
- `Stats`

---

### MergedEvent

**Source**: `core/operators.rs`

**Attributes**:
```rust
#[derive(Clone, Debug)]
```

```rust
pub enum MergedEvent<E1, E2> { First(E1), Second(E2), }
```

Unified event type for merged streams

**Variants**:

- `First(E1)`
- `Second(E2)`

---

### MevType

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub enum MevType { Sandwich, Frontrun, Backrun, Arbitrage, }
```

**Variants**:

- `Sandwich`
- `Frontrun`
- `Backrun`
- `Arbitrage`

---

### StreamConfig

**Source**: `core/builder.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
```

```rust
pub enum StreamConfig { SolanaGeyser { name: String, config: GeyserConfig, }, EvmWebSocket { name: String, config: EvmStreamConfig, }, Binance { name: String, config: BinanceConfig, }, MempoolSpace { name: String, config: MempoolConfig, }, }
```

Configuration for a stream

**Variants**:

- `SolanaGeyser`
- `name`
- `config`
- `EvmWebSocket`
- `name`
- `config`
- `Binance`
- `name`
- `config`
- `MempoolSpace`
- `name`
- `config`

---

### StreamError

**Source**: `core/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum StreamError { /// Connection-related errors #[error("Connection failed: {message}")] Connection { message: String, retriable: bool, }, /// Configuration errors #[error("Configuration error: {message}")] Configuration { message: String, }, /// Invalid configuration #[error("Invalid configuration: {reason}")] ConfigurationInvalid { reason: String, }, /// Authentication failures #[error("Authentication failed: {message}")] Authentication { message: String, }, /// Rate limiting errors #[error("Rate limit exceeded: {message}")] RateLimit { message: String, retry_after: Option<u64>, }, /// Data parsing errors #[error("Parse error: {message}")] Parse { message: String, data: String, }, /// Resource exhaustion #[error("Resource exhausted: {message}")] ResourceExhausted { message: String, }, /// Stream already running #[error("Stream already running: {name}")] AlreadyRunning { name: String, }, /// Stream not running #[error("Stream not running: {name}")] NotRunning { name: String, }, /// Channel errors #[error("Channel error: {message}")] Channel { message: String, }, /// Timeout errors #[error("Operation timed out: {message}")] Timeout { message: String, }, /// Processing errors #[error("Processing error: {message}")] Processing { message: String, }, /// Backpressure errors #[error("Backpressure error: {message}")] Backpressure { message: String, }, /// Internal errors #[error("Internal error: {source}")] Internal { #[source] source: Box<dyn std::error::Error + Send + Sync>, }, }
```

Comprehensive error type for streaming operations

This extends EventError with streaming-specific error variants while
maintaining compatibility with the events-core error hierarchy.

**Variants**:

- `Connection`
- `message`
- `retriable`
- `Configuration`
- `message`
- `ConfigurationInvalid`
- `reason`
- `Authentication`
- `message`
- `RateLimit`
- `message`
- `retry_after`
- `Parse`
- `message`
- `data`
- `ResourceExhausted`
- `message`
- `AlreadyRunning`
- `name`
- `NotRunning`
- `name`
- `Channel`
- `message`
- `Timeout`
- `message`
- `Processing`
- `message`
- `Backpressure`
- `message`
- `Internal`
- `source`

---

### WindowType

**Source**: `core/processor.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub enum WindowType { /// Fixed-size tumbling windows Tumbling { duration: Duration }, /// Sliding windows with overlap Sliding { size: Duration, step: Duration }, /// Session windows that close after inactivity Session { timeout: Duration }, /// Count-based windows Count { size: usize }, }
```

Window types for time-based processing

**Variants**:

- `Tumbling`
- `Sliding`
- `Session`
- `Count`

---

## Structs

### BackpressureConfig

**Source**: `core/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct BackpressureConfig { /// Channel buffer size (default: 1000)
```

Backpressure configuration

---

### BatchConfig

**Source**: `core/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct BatchConfig { /// Maximum batch size (default: 100)
```

Batch processing configuration

---

### BatchEvent

**Source**: `core/enhanced_operators.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct BatchEvent<E: Event> { pub events: Vec<E>, pub batch_timestamp: SystemTime, pub batch_id: String, pub metadata: EventMetadata, }
```

Wrapper for batched events that implements Event

---

### BatchEvent

**Source**: `core/operators.rs`

**Attributes**:
```rust
#[derive(Clone, Debug)]
```

```rust
pub struct BatchEvent<E> { pub events: Vec<Arc<E>>, pub batch_id: String, pub timestamp: std::time::SystemTime, }
```

Event wrapper for batched events

---

### BatchProcessor

**Source**: `core/processor.rs`

```rust
pub struct BatchProcessor<E> { config: BatchConfig, batch_buffer: VecDeque<E>, last_flush: Instant, }
```

Event batch processor with configurable batching

---

### BatchedStream

**Source**: `core/enhanced_operators.rs`

```rust
pub struct BatchedStream<S: Stream> { inner: S, name: String, event_tx: broadcast::Sender<Arc<BatchEvent<S::Event>>>, _batcher_handle: tokio::task::JoinHandle<()>,
```

Enhanced stream that includes batching capabilities

---

### BatchedStream

**Source**: `core/operators.rs`

```rust
pub struct BatchedStream<S> { inner: S, batch_size: usize, timeout: Duration, name: String, }
```

Stream that batches events

---

### BinanceConfig

**Source**: `external/binance.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
```

```rust
pub struct BinanceConfig { /// Streams to subscribe to (e.g., ["btcusdt@ticker", "ethusdt@depth"])
```

Binance stream configuration

---

### BinanceStream

**Source**: `external/binance.rs`

```rust
pub struct BinanceStream { /// Stream configuration config: BinanceConfig, /// Event broadcast channel event_tx: broadcast::Sender<Arc<BinanceStreamEvent>>, /// Running state running: Arc<AtomicBool>, /// Health metrics health: Arc<RwLock<StreamHealth>>, /// Stream name name: String, }
```

Binance WebSocket stream implementation

---

### BinanceStreamEvent

**Source**: `external/binance.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct BinanceStreamEvent { /// Event metadata for riglr-events-core compatibility pub metadata: EventMetadata, /// Event data pub data: BinanceEventData, /// Stream metadata (legacy)
```

Binance streaming event

---

### BitcoinBlock

**Source**: `external/mempool.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct BitcoinBlock { pub id: String, pub height: u64, pub timestamp: u64, pub tx_count: u32, pub size: u32, pub weight: u32, }
```

---

### BitcoinTransaction

**Source**: `external/mempool.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct BitcoinTransaction { pub txid: String, pub fee: u64, pub vsize: u32, pub value: u64, #[serde(default)]
```

---

### BollingerBands

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct BollingerBands { pub upper: f64, pub middle: f64, pub lower: f64, pub current_value: f64, }
```

---

### BollingerBandsStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct BollingerBandsStream<S> { inner: S, window: usize, std_dev_multiplier: f64, values: Arc<RwLock<VecDeque<f64>>>, }
```

Bollinger Bands calculation

---

### BufferedStream

**Source**: `core/operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct BufferedStream<S: Stream> { inner: S, strategy: BufferStrategy, buffer: Arc<RwLock<Vec<Arc<S::Event>>>>, name: String, }
```

Stream with buffering strategies

---

### CatchErrorStream

**Source**: `core/operators.rs`

```rust
pub struct CatchErrorStream<S, F> { inner: S, error_handler: Arc<F>, name: String, }
```

Stream that catches errors and provides fallback values

---

### CircuitBreaker

**Source**: `core/connection.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct CircuitBreaker { config: ConnectionConfig, state: AtomicConnectionState, failure_count: AtomicUsize, last_failure: Mutex<Option<Instant>>, last_success: Mutex<Option<Instant>>, }
```

Circuit breaker for connection management

---

### CircuitBreaker

**Source**: `production/resilience.rs`

```rust
pub struct CircuitBreaker { /// Name name: String, /// Failure threshold failure_threshold: u64, /// Success threshold to reset success_threshold: u64, /// Timeout duration timeout: Duration, /// Current failure count failure_count: Arc<AtomicU64>, /// Current success count success_count: Arc<AtomicU64>, /// Is open is_open: Arc<AtomicBool>, /// Last failure time last_failure_time: Arc<RwLock<Option<SystemTime>>>, }
```

Circuit breaker for stream connections

---

### CombineLatestStream

**Source**: `core/operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct CombineLatestStream<S1, S2> { stream1: S1, stream2: S2, name: String, }
```

Stream that combines latest values

---

### ComponentHealth

**Source**: `production/health.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct ComponentHealth { /// Component name pub name: String, /// Health status pub status: HealthStatus, /// Is connected pub is_connected: bool, /// Events processed pub events_processed: u64, /// Error count pub error_count: u64, /// Last event time pub last_event_time: Option<SystemTime>, /// Additional metrics pub metrics: HashMap<String, f64>, }
```

Individual component health

---

### CompositeCondition

**Source**: `tools/condition.rs`

```rust
pub struct CompositeCondition { conditions: Vec<Box<dyn EventCondition>>, combinator: ConditionCombinator, }
```

Composite condition that combines multiple conditions

---

### ConnectionConfig

**Source**: `core/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct ConnectionConfig { /// Connection timeout in seconds (default: 10)
```

Connection configuration

---

### ConnectionHealth

**Source**: `core/connection.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ConnectionHealth { pub state: ConnectionState, pub connected_at: Option<Instant>, pub last_activity: Option<Instant>, pub total_reconnects: usize, pub consecutive_failures: usize, pub latency_ms: Option<u64>, }
```

Connection health metrics

---

### ConnectionManager

**Source**: `core/connection.rs`

```rust
pub struct ConnectionManager<T> { circuit_breaker: Arc<CircuitBreaker>, connection: Arc<RwLock<Option<T>>>, health: Arc<RwLock<ConnectionHealth>>, reconnect_task: Arc<AtomicBool>, }
```

Connection manager with automatic reconnection

---

### ConnectionPool

**Source**: `core/connection.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct ConnectionPool<T> { connections: Vec<ConnectionManager<T>>, config: ConnectionConfig, current_index: AtomicUsize, }
```

Connection pool for managing multiple connections

---

### CustomCondition

**Source**: `tools/condition.rs`

```rust
pub struct CustomCondition<F> where F: Fn(&(dyn Any + Send + Sync)) -> bool + Send + Sync,
```

Custom condition with closure

---

### DebouncedStream

**Source**: `core/operators.rs`

```rust
pub struct DebouncedStream<S> { inner: S, duration: Duration, name: String, }
```

Stream that debounces events

---

### DynamicStreamWrapper

**Source**: `core/stream.rs`

```rust
pub struct DynamicStreamWrapper<S: Stream> { pub(crate) inner: S,
```

Wrapper to make any Stream implement DynamicStream

---

### DynamicStreamedEvent

**Source**: `core/streamed_event.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct DynamicStreamedEvent { /// The type-erased event pub inner: Box<dyn Event>, /// Streaming-specific metadata pub stream_metadata: StreamMetadata, }
```

Dynamic event wrapper for type-erased Events with streaming metadata
This is a separate implementation that doesn't rely on the generic StreamedEvent<T>

---

### EmaStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct EmaStream<S> { inner: S, alpha: f64, current_ema: Arc<RwLock<Option<f64>>>, }
```

Exponential moving average

---

### EnhancedStream

**Source**: `core/enhanced_operators.rs`

```rust
pub struct EnhancedStream<S: Stream> { inner: S, name: String, event_tx: broadcast::Sender<Arc<S::Event>>, metrics: Arc<EventPerformanceMetrics>, _processor_handle: tokio::task::JoinHandle<()>,
```

Enhanced stream with rate limiting and deduplication

---

### EventKindMatcher

**Source**: `tools/condition.rs`

```rust
pub struct EventKindMatcher { event_kinds: Vec<EventKind>, }
```

Event kind matcher

---

### EventTriggerBuilder

**Source**: `tools/event_triggered.rs`

```rust
pub struct EventTriggerBuilder<T: StreamingTool> { tool: T, name: String, conditions: Vec<Box<dyn EventCondition>>, combinator: ConditionCombinator, }
```

Builder for event-triggered tools

---

### EventTriggeredTool

**Source**: `tools/event_triggered.rs`

```rust
pub struct EventTriggeredTool<T: StreamingTool> { /// The underlying tool tool: Arc<T>, /// Event conditions that trigger this tool conditions: Vec<Box<dyn EventCondition>>, /// Combinator for multiple conditions combinator: ConditionCombinator, /// Tool name name: String, /// Execution count execution_count: Arc<RwLock<u64>>, }
```

Wrapper that makes any Tool event-triggered

---

### EvmStreamConfig

**Source**: `evm/websocket.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
```

```rust
pub struct EvmStreamConfig { /// WebSocket URL pub ws_url: String, /// Chain ID pub chain_id: ChainId, /// Subscribe to pending transactions pub subscribe_pending_transactions: bool, /// Subscribe to new blocks pub subscribe_new_blocks: bool, /// Contract addresses to monitor (as hex strings)
```

EVM stream configuration

---

### EvmStreamEvent

**Source**: `evm/websocket.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct EvmStreamEvent { /// Event metadata for riglr-events-core compatibility pub metadata: EventMetadata, /// Event type pub event_type: EvmEventType, /// Stream metadata (legacy)
```

EVM streaming event

---

### EvmWebSocketStream

**Source**: `evm/websocket.rs`

```rust
pub struct EvmWebSocketStream { /// Stream configuration config: EvmStreamConfig, /// Event broadcast channel event_tx: broadcast::Sender<Arc<EvmStreamEvent>>, /// Running state running: Arc<AtomicBool>, /// Health metrics health: Arc<RwLock<StreamHealth>>, /// Stream name name: String, }
```

EVM WebSocket stream implementation

---

### FeeEstimate

**Source**: `external/mempool.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct FeeEstimate { #[serde(rename = "fastestFee")]
```

---

### FilterMapStream

**Source**: `core/operators.rs`

```rust
pub struct FilterMapStream<S, FilterF, MapF, R> { inner: S, filter: Arc<FilterF>, map: Arc<MapF>, name: String, _phantom: PhantomData<R>, }
```

Stream that fuses filter and map operations for better performance

---

### FilteredStream

**Source**: `core/operators.rs`

```rust
pub struct FilteredStream<S, F> { inner: S, predicate: Arc<F>, }
```

Stream that filters events based on a predicate

---

### FinancialEvent

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[derive(Clone, Debug)]
```

```rust
pub struct FinancialEvent<T, E> { pub metadata: riglr_events_core::EventMetadata, pub indicator_value: T, pub original_event: Arc<E>, pub indicator_type: String, pub timestamp: SystemTime, }
```

Financial indicator event wrapper

---

### FlowController

**Source**: `core/processor.rs`

```rust
pub struct FlowController { config: BackpressureConfig, semaphore: Arc<Semaphore>, queue_size: Arc<tokio::sync::RwLock<usize>>, drop_count: Arc<tokio::sync::RwLock<usize>>, }
```

Flow control manager with backpressure handling

---

### GasPriceEstimate

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct GasPriceEstimate { pub slow: f64, pub standard: f64, pub fast: f64, pub instant: f64, }
```

---

### GasPriceOracleStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct GasPriceOracleStream<S> { inner: S, percentiles: Vec<usize>, gas_prices: Arc<RwLock<VecDeque<f64>>>, window_size: usize, }
```

Gas price oracle stream

---

### GeyserConfig

**Source**: `solana/geyser.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Deserialize, Serialize)]
```

```rust
pub struct GeyserConfig { /// WebSocket endpoint URL (e.g., wss://api.mainnet-beta.solana.com)
```

Geyser stream configuration

---

### GlobalMetrics

**Source**: `core/metrics.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct GlobalMetrics { /// Total events across all streams pub total_events: u64, /// Total bytes across all streams pub total_bytes: u64, /// Global events per second pub global_events_per_second: f64, /// Global bytes per second pub global_bytes_per_second: f64, /// Active stream count pub active_streams: usize, /// Active handler count pub active_handlers: usize, /// System start time pub system_start_time: SystemTime, /// Last metrics update pub last_update: SystemTime, }
```

Global metrics aggregator

---

### GuaranteedDeliveryStream

**Source**: `core/operators.rs`

```rust
pub struct GuaranteedDeliveryStream<S> { inner: S, buffer_size: usize, name: String, }
```

Stream wrapper that provides guaranteed delivery using mpsc channels

Unlike the default broadcast channels that drop old messages when buffers fill,
this wrapper uses tokio::sync::mpsc channels to ensure every message is delivered.
This comes at the cost of potential pipeline blocking if receivers are slow.

---

### HandlerMetrics

**Source**: `core/metrics.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct HandlerMetrics { /// Handler name pub handler_name: String, /// Total invocations pub invocations: u64, /// Successful executions pub successes: u64, /// Failed executions pub failures: u64, /// Average execution time (ms)
```

Handler metrics

---

### HealthAlert

**Source**: `production/health.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct HealthAlert { /// Alert severity pub severity: AlertSeverity, /// Component that triggered the alert pub component: String, /// Alert message pub message: String, /// When the alert was triggered pub triggered_at: SystemTime, }
```

Health alert

---

### HealthMonitor

**Source**: `production/health.rs`

```rust
pub struct HealthMonitor { /// Stream manager reference stream_manager: Arc<StreamManager>, /// Health check interval check_interval: Duration, /// Health history health_history: Arc<RwLock<Vec<HealthSnapshot>>>, /// Alert thresholds thresholds: HealthThresholds, }
```

Health monitor for streams

---

### HealthSnapshot

**Source**: `production/health.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct HealthSnapshot { /// Timestamp of the snapshot pub timestamp: SystemTime, /// Overall health status pub overall_status: HealthStatus, /// Individual component health pub components: HashMap<String, ComponentHealth>, /// Active alerts pub alerts: Vec<HealthAlert>, }
```

Health snapshot at a point in time

---

### HealthThresholds

**Source**: `production/health.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct HealthThresholds { /// Max time without events before considered unhealthy pub max_event_age: Duration, /// Max error rate (errors per minute)
```

Health check thresholds

---

### KlineData

**Source**: `external/binance.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct KlineData { #[serde(rename = "s")]
```

---

### KlineDetails

**Source**: `external/binance.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct KlineDetails { #[serde(rename = "t")]
```

---

### LiquidityPoolStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct LiquidityPoolStream<S> { inner: S, pools: Arc<RwLock<HashMap<String, PoolState>>>, }
```

Liquidity pool balance tracker

---

### LoggingEventHandler

**Source**: `core/manager.rs`

```rust
pub struct LoggingEventHandler { name: String, }
```

Simple event handler for testing

---

### MappedStream

**Source**: `core/operators.rs`

```rust
pub struct MappedStream<S, F, R> { inner: S, transform: Arc<F>, _phantom: PhantomData<R>, }
```

Stream that maps events through a transformation

---

### Matcher

**Source**: `tools/condition.rs`

```rust
pub struct Matcher;
```

Empty struct to implement EventMatcher

---

### MempoolConfig

**Source**: `external/mempool.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
```

```rust
pub struct MempoolConfig { /// Bitcoin network pub network: BitcoinNetwork, /// Subscribe to transactions pub subscribe_transactions: bool, /// Subscribe to blocks pub subscribe_blocks: bool, /// Subscribe to fee estimates pub subscribe_fees: bool, /// Buffer size for event channel pub buffer_size: usize, }
```

Mempool configuration

---

### MempoolSpaceStream

**Source**: `external/mempool.rs`

```rust
pub struct MempoolSpaceStream { /// Stream configuration config: MempoolConfig, /// Event broadcast channel event_tx: broadcast::Sender<Arc<MempoolStreamEvent>>, /// Running state running: Arc<AtomicBool>, /// Health metrics health: Arc<RwLock<StreamHealth>>, /// Stream name name: String, }
```

Mempool.space WebSocket stream implementation

---

### MempoolStats

**Source**: `external/mempool.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct MempoolStats { pub count: u32, pub vsize: u64, pub total_fee: u64, pub fee_histogram: Vec<Vec<u64>>, }
```

---

### MempoolStreamEvent

**Source**: `external/mempool.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct MempoolStreamEvent { /// Event metadata for riglr-events-core compatibility pub metadata: EventMetadata, /// Event type pub event_type: MempoolEventType, /// Event data pub data: serde_json::Value, /// Stream metadata (legacy)
```

Mempool streaming event

---

### MergeAllStream

**Source**: `core/operators.rs`

```rust
pub struct MergeAllStream<S> { streams: Vec<S>, name: String, }
```

Stream that merges multiple streams using type erasure

---

### MergedStream

**Source**: `core/operators.rs`

```rust
pub struct MergedStream<S1, S2> { stream1: S1, stream2: S2, name: String, }
```

Stream that merges events from two streams

---

### MetricsCollector

**Source**: `core/metrics.rs`

```rust
pub struct MetricsCollector { /// Stream metrics stream_metrics: Arc<RwLock<HashMap<String, StreamMetrics>>>, /// Handler metrics handler_metrics: Arc<RwLock<HashMap<String, HandlerMetrics>>>, /// Global metrics global_metrics: Arc<RwLock<GlobalMetrics>>, /// Metrics history for time-series data metrics_history: Arc<RwLock<Vec<(SystemTime, GlobalMetrics)>>>,
```

Metrics collector for the streaming system

---

### MetricsCollector

**Source**: `production/monitoring.rs`

```rust
pub struct MetricsCollector { /// Metrics by stream stream_metrics: Arc<RwLock<HashMap<String, StreamMetrics>>>, }
```

Metrics collector for all streams

---

### MetricsConfig

**Source**: `core/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MetricsConfig { /// Enable metrics collection pub enabled: bool, /// Metrics collection window in seconds pub window_secs: u64, /// Metrics reporting interval in seconds pub report_interval_secs: u64, /// Slow processing threshold in milliseconds pub slow_threshold_ms: u64, /// Include detailed latency histograms pub detailed_latency: bool, }
```

Performance monitoring configuration

---

### MetricsSnapshot

**Source**: `production/monitoring.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MetricsSnapshot { /// Uptime pub uptime: Duration, /// Total events processed pub events_processed: u64, /// Events by type pub events_by_type: HashMap<String, u64>, /// Error count pub error_count: u64, /// Events per second pub events_per_second: f64, /// Performance stats pub performance_stats: PerformanceStats, }
```

Metrics snapshot at a point in time

---

### MetricsTimer

**Source**: `core/metrics.rs`

```rust
pub struct MetricsTimer { start: Instant, name: String, collector: Option<Arc<MetricsCollector>>, }
```

Metrics timer for measuring execution time

---

### MevDetectionStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct MevDetectionStream<S> { inner: S, window: Duration, transactions: Arc<RwLock<VecDeque<TransactionPattern>>>, }
```

MEV (Maximum Extractable Value) detection

---

### MockConfig

**Source**: `core/mock_stream.rs`

**Attributes**:
```rust
#[derive(Clone, Debug)]
```

```rust
pub struct MockConfig { /// Events per second to generate pub events_per_second: f64, /// Event kinds to simulate pub event_kinds: Vec<EventKind>, /// Total events to generate (None for infinite)
```

Configuration for mock stream

---

### MockStream

**Source**: `core/mock_stream.rs`

```rust
pub struct MockStream { config: MockConfig, event_tx: broadcast::Sender<Arc<DynamicStreamedEvent>>, running: Arc<AtomicBool>, health: Arc<RwLock<StreamHealth>>, name: String, }
```

Mock stream that generates protocol-specific events for testing

---

### MomentumStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct MomentumStream<S> { inner: S, lookback_period: usize, price_history: Arc<RwLock<VecDeque<f64>>>, }
```

Price momentum indicator

---

### MovingAverageStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct MovingAverageStream<S> { inner: S, window_size: usize, values: Arc<RwLock<VecDeque<f64>>>, }
```

Moving average calculation

---

### MultiChainEvmManager

**Source**: `evm/multi_chain.rs`

```rust
pub struct MultiChainEvmManager { /// Streams by chain ID streams: Arc<RwLock<HashMap<ChainId, EvmWebSocketStream>>>, /// Stream manager reference stream_manager: Option<Arc<StreamManager>>, }
```

Multi-chain EVM stream manager

---

### OrderBookData

**Source**: `external/binance.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct OrderBookData { #[serde(rename = "s")]
```

---

### OrderBookImbalanceStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct OrderBookImbalanceStream<S> { inner: S, depth_levels: usize, }
```

Order book imbalance calculator

---

### PatternMatcher

**Source**: `core/processor.rs`

```rust
pub struct PatternMatcher<E> { patterns: Vec<EventPattern<E>>, event_history: VecDeque<(E, Instant)>,
```

---

### PerformanceStats

**Source**: `production/monitoring.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
```

```rust
pub struct PerformanceStats { /// Average events per second pub events_per_second: f64, /// Peak events per second pub peak_events_per_second: f64, /// Average processing latency (ms)
```

Performance statistics

---

### PoolState

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct PoolState { pub token_a_reserve: f64, pub token_b_reserve: f64, pub k_constant: f64, pub last_updated: SystemTime, }
```

---

### ResilientStream

**Source**: `core/operators.rs`

```rust
pub struct ResilientStream<S> { inner: S, max_retries: usize, retry_delay: Duration, name: String, }
```

Stream with error recovery and retry capabilities

---

### RetryPolicy

**Source**: `production/resilience.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct RetryPolicy { /// Maximum number of retries pub max_retries: u32, /// Initial delay pub initial_delay: Duration, /// Backoff strategy pub backoff: BackoffStrategy, /// Maximum delay pub max_delay: Duration, /// Jitter pub jitter: bool, }
```

Retry policy for operations

---

### RsiStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct RsiStream<S> { inner: S, period: usize, gains: Arc<RwLock<VecDeque<f64>>>, losses: Arc<RwLock<VecDeque<f64>>>, last_value: Arc<RwLock<Option<f64>>>, }
```

RSI (Relative Strength Index) calculation

---

### ScanEvent

**Source**: `core/operators.rs`

**Attributes**:
```rust
#[derive(Clone, Debug)]
```

```rust
pub struct ScanEvent<S, E> { pub state: S, pub original_event: Arc<E>, pub scan_id: String, pub timestamp: std::time::SystemTime, }
```

Event wrapper for scan results

---

### ScanStream

**Source**: `core/operators.rs`

```rust
pub struct ScanStream<S, St, F> { inner: S, state: Arc<RwLock<St>>, transform: Arc<F>, name: String, }
```

Stream with stateful transformation (scan)

---

### SkipStream

**Source**: `core/operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct SkipStream<S> { inner: S, count: usize, skipped: Arc<RwLock<usize>>, name: String, }
```

Stream that skips N events

---

### SolanaGeyserStream

**Source**: `solana/geyser.rs`

```rust
pub struct SolanaGeyserStream { /// Stream configuration config: GeyserConfig, /// Event broadcast channel for protocol-specific events event_tx: broadcast::Sender<Arc<DynamicStreamedEvent>>, /// Event parser for converting raw data to structured events _event_parser_placeholder: (),
```

Solana Geyser stream implementation using WebSocket

---

### SourceMatcher

**Source**: `tools/condition.rs`

```rust
pub struct SourceMatcher { sources: Vec<String>, }
```

Source matcher (matches event source)

---

### StatefulProcessor

**Source**: `core/processor.rs`

```rust
pub struct StatefulProcessor<K, S> where K: Hash + Eq + Clone, S: Clone, { state_store: Arc<RwLock<HashMap<K, S>>>, checkpoint_interval: Duration, last_checkpoint: Instant, }
```

Stateful event processor with checkpointing

---

### StreamClientConfig

**Source**: `core/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
```

```rust
pub struct StreamClientConfig { /// Connection settings pub connection: ConnectionConfig, /// Batch processing settings pub batch: BatchConfig, /// Backpressure handling settings pub backpressure: BackpressureConfig, /// Performance monitoring settings pub metrics: MetricsConfig, /// Enable debugging features pub debug: bool, }
```

Comprehensive streaming client configuration

---

### StreamHealth

**Source**: `core/stream.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
#[derive(Default)]
```

```rust
pub struct StreamHealth { /// Whether the stream is connected pub is_connected: bool, /// Last time an event was received pub last_event_time: Option<SystemTime>, /// Number of errors encountered pub error_count: u64, /// Total number of events processed pub events_processed: u64, /// Current backlog size (if applicable)
```

Health status of a stream

Enhanced with events-core integration for better observability

---

### StreamManager

**Source**: `core/manager.rs`

```rust
pub struct StreamManager { /// Registered streams streams: Arc<RwLock<HashMap<String, Box<dyn DynamicStream>>>>, /// Running stream tasks running_streams: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
```

Manages multiple streams and routes events to handlers

---

### StreamManagerBuilder

**Source**: `core/builder.rs`

```rust
pub struct StreamManagerBuilder { execution_mode: HandlerExecutionMode, streams: Vec<StreamConfig>, enable_metrics: bool, metrics_collector: Option<Arc<MetricsCollector>>, }
```

Builder for StreamManager with configuration loading

---

### StreamManagerConfig

**Source**: `core/builder.rs`

**Attributes**:
```rust
#[derive(Debug, Deserialize)]
```

```rust
pub struct StreamManagerConfig { pub execution_mode: Option<HandlerExecutionMode>, pub enable_metrics: Option<bool>, pub streams: Vec<StreamConfig>, }
```

Configuration structure for loading from TOML

---

### StreamMetadata

**Source**: `core/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, serde::Serialize)]
```

```rust
pub struct StreamMetadata { pub stream_source: String, #[serde(with = "systemtime_as_millis")]
```

Temporary StreamMetadata for migration phase

---

### StreamMetrics

**Source**: `core/metrics.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct StreamMetrics { /// Stream name pub stream_name: String, /// Total events received pub events_received: u64, /// Total events processed pub events_processed: u64, /// Total events dropped pub events_dropped: u64, /// Average event processing time (ms)
```

Performance metrics for a stream

---

### StreamMetrics

**Source**: `production/monitoring.rs`

```rust
pub struct StreamMetrics { /// Start time start_time: SystemTime, /// Total events processed events_processed: Arc<RwLock<u64>>, /// Events by type events_by_type: Arc<RwLock<HashMap<String, u64>>>, /// Error count error_count: Arc<RwLock<u64>>, /// Performance stats performance_stats: Arc<RwLock<PerformanceStats>>, }
```

Stream metrics collector

---

### StreamPipeline

**Source**: `core/operators.rs`

```rust
pub struct StreamPipeline<T> { stream: T, }
```

Builder for complex stream compositions

---

### StreamedEvent

**Source**: `core/streamed_event.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct StreamedEvent<T: Event + Clone> { /// The original event from any source pub inner: T, /// Streaming-specific metadata pub stream_metadata: StreamMetadata, }
```

Wrapper that adds streaming metadata to any Event through composition

---

### StreamingToolWorker

**Source**: `tools/worker_extension.rs`

```rust
pub struct StreamingToolWorker { /// Worker ID worker_id: String, /// Stream manager stream_manager: Arc<StreamManager>, }
```

Extended worker that can process streaming events

---

### TakeStream

**Source**: `core/operators.rs`

```rust
pub struct TakeStream<S> { inner: S, count: usize, taken: Arc<RwLock<usize>>, name: String, }
```

Stream that takes only N events

---

### TestPriceEvent

**Source**: `core/test_asnumeric.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct TestPriceEvent { pub metadata: EventMetadata, pub symbol: String, pub price: f64, pub volume: f64, pub timestamp: SystemTime, }
```

---

### ThrottledStream

**Source**: `core/operators.rs`

```rust
pub struct ThrottledStream<S> { inner: S, duration: Duration, name: String, }
```

Stream that throttles events

---

### TickerData

**Source**: `external/binance.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct TickerData { #[serde(rename = "s")]
```

---

### TimestampRangeMatcher

**Source**: `tools/condition.rs`

```rust
pub struct TimestampRangeMatcher { min_timestamp: Option<std::time::SystemTime>, max_timestamp: Option<std::time::SystemTime>, }
```

Timestamp range matcher

---

### TradeData

**Source**: `external/binance.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct TradeData { #[serde(rename = "s")]
```

---

### TransactionEvent

**Source**: `solana/geyser.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct TransactionEvent { pub signature: String, pub slot: u64, }
```

Simple transaction event for fallback cases

---

### TypeErasedEvent

**Source**: `core/operators.rs`

**Attributes**:
```rust
#[derive(Clone, Debug)]
```

```rust
pub struct TypeErasedEvent { inner: Arc<dyn Event>, source_stream: String, }
```

Type-erased event for merged streams to avoid complex nested types

---

### VwapStream

**Source**: `core/financial_operators.rs`

```rust
pub struct VwapStream<S> { inner: S, window: Duration, price_volume_pairs: Arc<RwLock<VecDeque<(f64, f64, SystemTime)>>>,
```

Volume-Weighted Average Price (VWAP) calculation

---

### Window

**Source**: `core/processor.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct Window<E> { pub id: u64, pub start_time: Instant, pub end_time: Option<Instant>, pub events: Vec<E>, pub is_closed: bool, }
```

Window state for managing event windows

---

### WindowManager

**Source**: `core/processor.rs`

```rust
pub struct WindowManager<E> { window_type: WindowType, active_windows: HashMap<u64, Window<E>>, next_window_id: u64, last_cleanup: Instant, }
```

Window manager for handling different window types

---

### ZipStream

**Source**: `core/operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct ZipStream<S1, S2> { stream1: S1, stream2: S2, name: String, }
```

Stream that zips two streams

---

## Functions

### acquire_permit

**Source**: `core/processor.rs`

```rust
pub async fn acquire_permit(&self) -> StreamResult<()>
```

---

### active_chains

**Source**: `evm/multi_chain.rs`

```rust
pub async fn active_chains(&self) -> Vec<ChainId>
```

Get list of active chains

---

### add_chain

**Source**: `evm/multi_chain.rs`

```rust
pub async fn add_chain(&self, chain_id: ChainId) -> Result<(), StreamError>
```

Add an EVM chain using the RPC_URL_{CHAIN_ID} pattern

---

### add_condition

**Source**: `tools/condition.rs`

```rust
pub fn add_condition(mut self, condition: Box<dyn EventCondition>) -> Self
```

---

### add_event

**Source**: `core/processor.rs`

```rust
pub fn add_event(&mut self, event: E) -> Vec<Window<E>>
```

---

### add_event

**Source**: `core/processor.rs`

```rust
pub fn add_event(&mut self, event: E) -> Option<Vec<E>>
```

---

### add_event_handler

**Source**: `core/manager.rs`

```rust
pub async fn add_event_handler(&self, handler: Arc<dyn EventHandler>)
```

Add an event handler

---

### add_stream

**Source**: `core/builder.rs`

```rust
pub fn add_stream(mut self, config: StreamConfig) -> Self
```

Add a stream configuration

---

### add_stream

**Source**: `core/manager.rs`

```rust
pub async fn add_stream<S>(&self, name: String, stream: S) -> StreamResult<()> where S: Stream + Send + Sync + 'static, S::Event: Send + Sync + 'static,
```

Add a stream to be managed (stream should already be configured and started)

---

### after

**Source**: `tools/condition.rs`

```rust
pub fn after(timestamp: std::time::SystemTime) -> Self
```

---

### as_event

**Source**: `tools/event_utils.rs`

```rust
pub fn as_event(event: &(dyn Any + Send + Sync)) -> Option<&dyn Event>
```

Helper to convert Any to Event by trying all known event types

---

### as_event_extended

**Source**: `tools/event_utils.rs`

```rust
pub fn as_event_extended(event: &(dyn Any + Send + Sync)) -> Option<&dyn Event>
```

---

### attempt_connect

**Source**: `core/connection.rs`

```rust
pub async fn attempt_connect<F, Fut, T>(&self, connect_fn: F) -> StreamResult<T> where F: Fn() -> Fut, Fut: std::future::Future<Output = StreamResult<T>>,
```

---

### batch_timeout

**Source**: `core/config.rs`

```rust
pub fn batch_timeout(&self) -> Duration
```

---

### before

**Source**: `tools/condition.rs`

```rust
pub fn before(timestamp: std::time::SystemTime) -> Self
```

---

### build

**Source**: `core/builder.rs`

```rust
pub async fn build(self) -> Result<StreamManager, Box<dyn std::error::Error>>
```

Build the StreamManager with all configured streams

---

### build

**Source**: `core/operators.rs`

```rust
pub fn build(self) -> T
```

Build the final stream

---

### build

**Source**: `tools/event_triggered.rs`

```rust
pub fn build(self) -> EventTriggeredTool<T>
```

Build the event-triggered tool

---

### calculate_delay

**Source**: `production/resilience.rs`

```rust
pub fn calculate_delay(&self, attempt: u32) -> Duration
```

Calculate delay for retry attempt

---

### calculate_imbalance

**Source**: `core/financial_operators.rs`

```rust
pub fn calculate_imbalance(bids: &[(f64, f64)], asks: &[(f64, f64)]) -> f64
```

---

### checkpoint

**Source**: `core/processor.rs`

```rust
pub async fn checkpoint(&self) -> StreamResult<()>
```

---

### combinator

**Source**: `tools/event_triggered.rs`

```rust
pub fn combinator(mut self, combinator: ConditionCombinator) -> Self
```

Set combinator

---

### combine_latest

**Source**: `core/operators.rs`

```rust
pub fn combine_latest<S1, S2>(stream1: S1, stream2: S2) -> CombineLatestStream<S1, S2> where S1: Stream, S2: Stream,
```

Combine latest values from two streams

---

### condition

**Source**: `tools/event_triggered.rs`

```rust
pub fn condition(mut self, condition: Box<dyn EventCondition>) -> Self
```

Add a condition

---

### connect

**Source**: `core/connection.rs`

```rust
pub async fn connect<F, Fut>(&self, connect_fn: F) -> StreamResult<()> where F: Fn() -> Fut + Send + Sync + 'static + Clone, Fut: std::future::Future<Output = StreamResult<T>> + Send,
```

---

### connect_all

**Source**: `core/connection.rs`

```rust
pub async fn connect_all<F, Fut>(&self, connect_fn: F) -> StreamResult<()> where F: Fn() -> Fut + Send + Sync + 'static + Clone, Fut: std::future::Future<Output = StreamResult<T>> + Send,
```

---

### connect_timeout

**Source**: `core/config.rs`

```rust
pub fn connect_timeout(&self) -> Duration
```

---

### current_health

**Source**: `production/health.rs`

```rust
pub async fn current_health(&self) -> HealthSnapshot
```

Get current health snapshot

---

### development

**Source**: `core/config.rs`

```rust
pub fn development() -> Self
```

Development configuration with debugging enabled

---

### disconnect

**Source**: `core/connection.rs`

```rust
pub async fn disconnect(&self)
```

---

### downcast

**Source**: `core/operators.rs`

```rust
pub fn downcast<T: Event + 'static>(&self) -> Option<&T>
```

Attempt to downcast to a specific event type

---

### elapsed_ms

**Source**: `core/metrics.rs`

```rust
pub fn elapsed_ms(&self) -> f64
```

Get elapsed time in milliseconds

---

### execute

**Source**: `production/resilience.rs`

```rust
pub async fn execute<F, T, E>(&self, mut operation: F) -> Result<T, E> where F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>, E: std::fmt::Display,
```

Execute with retry

---

### execution_count

**Source**: `tools/event_triggered.rs`

```rust
pub async fn execution_count(&self) -> u64
```

Get execution count

---

### export_json

**Source**: `core/metrics.rs`

```rust
pub async fn export_json(&self) -> String
```

Export metrics as JSON

---

### export_prometheus

**Source**: `core/metrics.rs`

```rust
pub async fn export_prometheus(&self) -> String
```

Export metrics in Prometheus format

---

### flush_batch

**Source**: `core/processor.rs`

```rust
pub fn flush_batch(&mut self) -> Option<Vec<E>>
```

---

### from

**Source**: `core/operators.rs`

```rust
pub fn from(stream: T) -> Self
```

Create a new pipeline from a stream

---

### from_config_file

**Source**: `core/builder.rs`

```rust
pub async fn from_config_file(path: &str) -> Result<StreamManager, Box<dyn std::error::Error>>
```

Helper to load and build a StreamManager from a TOML file

---

### from_env

**Source**: `core/builder.rs`

```rust
pub fn from_env(mut self) -> Result<Self, Box<dyn std::error::Error>>
```

Load configuration from environment variables

---

### from_env

**Source**: `core/builder.rs`

```rust
pub async fn from_env() -> Result<StreamManager, Box<dyn std::error::Error>>
```

Helper to load and build a StreamManager from environment variables

---

### from_env

**Source**: `core/config.rs`

```rust
pub fn from_env() -> Result<Self, ConfigError>
```

Load configuration from environment variables

---

### from_event

**Source**: `core/streamed_event.rs`

```rust
pub fn from_event( event: Box<dyn Event>, stream_metadata: StreamMetadata, ) -> Self
```

Create from any Event

---

### from_event_error

**Source**: `core/error.rs`

```rust
pub fn from_event_error(event_error: EventError) -> Self
```

Create a StreamError from an EventError

---

### from_toml_file

**Source**: `core/builder.rs`

```rust
pub async fn from_toml_file(mut self, path: &str) -> Result<Self, Box<dyn std::error::Error>>
```

Load configuration from a TOML file

---

### get_all_handler_metrics

**Source**: `core/metrics.rs`

```rust
pub async fn get_all_handler_metrics(&self) -> Vec<HandlerMetrics>
```

Get all handler metrics

---

### get_all_metrics

**Source**: `production/monitoring.rs`

```rust
pub async fn get_all_metrics(&self) -> HashMap<String, MetricsSnapshot>
```

Get all metrics

---

### get_all_stream_metrics

**Source**: `core/metrics.rs`

```rust
pub async fn get_all_stream_metrics(&self) -> Vec<StreamMetrics>
```

Get all stream metrics

---

### get_connection

**Source**: `core/connection.rs`

```rust
pub async fn get_connection(&self) -> Option<T> where T: Clone,
```

---

### get_global_metrics

**Source**: `core/metrics.rs`

```rust
pub async fn get_global_metrics(&self) -> GlobalMetrics
```

Get global metrics

---

### get_handler_metrics

**Source**: `core/metrics.rs`

```rust
pub async fn get_handler_metrics(&self, handler_name: &str) -> Option<HandlerMetrics>
```

Get handler metrics

---

### get_healthy_connection

**Source**: `core/connection.rs`

```rust
pub async fn get_healthy_connection(&self) -> StreamResult<T>
```

---

### get_metrics

**Source**: `core/enhanced_operators.rs`

```rust
pub async fn get_metrics(&self) -> MetricsSummary
```

Get performance metrics for this stream

---

### get_metrics

**Source**: `production/monitoring.rs`

```rust
pub async fn get_metrics(&self) -> MetricsSnapshot
```

Get current metrics

---

### get_state

**Source**: `core/processor.rs`

```rust
pub async fn get_state(&self, key: &K) -> Option<S>
```

---

### get_stats

**Source**: `core/processor.rs`

```rust
pub async fn get_stats(&self) -> (usize, usize)
```

---

### get_stream_metrics

**Source**: `core/metrics.rs`

```rust
pub async fn get_stream_metrics(&self, stream_name: &str) -> Option<StreamMetrics>
```

Get stream metrics

---

### get_stream_metrics

**Source**: `production/monitoring.rs`

```rust
pub async fn get_stream_metrics(&self, stream_name: &str) -> StreamMetrics
```

Get or create metrics for a stream

---

### health

**Source**: `core/manager.rs`

```rust
pub async fn health(&self) -> HashMap<String, StreamHealth>
```

Get health status of all streams

---

### health

**Source**: `core/connection.rs`

```rust
pub async fn health(&self) -> ConnectionHealth
```

---

### health_history

**Source**: `production/health.rs`

```rust
pub async fn health_history(&self) -> Vec<HealthSnapshot>
```

Get health history

---

### high_performance

**Source**: `core/config.rs`

```rust
pub fn high_performance() -> Self
```

High-performance configuration for production environments

---

### high_watermark

**Source**: `core/config.rs`

```rust
pub fn high_watermark(&self) -> usize
```

---

### inner

**Source**: `core/operators.rs`

```rust
pub fn inner(&self) -> &dyn Event
```

Get the inner event as a trait object

---

### inner

**Source**: `core/streamed_event.rs`

```rust
pub fn inner(&self) -> &T
```

Get a reference to the inner event

---

### inner

**Source**: `core/streamed_event.rs`

```rust
pub fn inner(&self) -> &dyn Event
```

Get a reference to the inner event

---

### inner_mut

**Source**: `core/streamed_event.rs`

```rust
pub fn inner_mut(&mut self) -> &mut T
```

Get a mutable reference to the inner event

---

### into_inner

**Source**: `core/streamed_event.rs`

```rust
pub fn into_inner(self) -> T
```

Unwrap to get the inner event

---

### is_chain_configured

**Source**: `evm/multi_chain.rs`

```rust
pub fn is_chain_configured(&self, chain_id: ChainId) -> bool
```

Check if a chain is configured

---

### is_open

**Source**: `production/resilience.rs`

```rust
pub async fn is_open(&self) -> bool
```

Check if circuit is open

---

### is_retriable

**Source**: `core/error.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if this error is retriable

---

### is_stream_running

**Source**: `core/manager.rs`

```rust
pub async fn is_stream_running(&self, name: &str) -> bool
```

Check if a stream is running

---

### keepalive_interval

**Source**: `core/config.rs`

```rust
pub fn keepalive_interval(&self) -> Duration
```

---

### list_streams

**Source**: `core/manager.rs`

```rust
pub async fn list_streams(&self) -> Vec<String>
```

Get list of stream names

---

### low_latency

**Source**: `core/config.rs`

```rust
pub fn low_latency() -> Self
```

Low-latency configuration for real-time applications

---

### low_watermark

**Source**: `core/config.rs`

```rust
pub fn low_watermark(&self) -> usize
```

---

### mark_disconnected

**Source**: `core/connection.rs`

```rust
pub fn mark_disconnected(&self)
```

---

### match_event

**Source**: `core/processor.rs`

```rust
pub fn match_event(&mut self, event: E) -> Vec<usize>
```

---

### merge_all

**Source**: `core/operators.rs`

```rust
pub fn merge_all<S, I>(streams: I) -> MergeAllStream<S> where S: Stream + 'static, I: IntoIterator<Item = S>,
```

Merge multiple streams into one

---

### metrics_collector

**Source**: `core/manager.rs`

```rust
pub fn metrics_collector(&self) -> Arc<MetricsCollector>
```

Get the metrics collector

---

### new

**Source**: `core/builder.rs`

```rust
pub fn new() -> Self
```

Create a new builder

---

### new

**Source**: `core/enhanced_operators.rs`

```rust
pub fn new(inner: S, _batch_size: usize, _timeout: Duration) -> Self
```

Create a new batched stream

---

### new

**Source**: `core/enhanced_operators.rs`

```rust
pub fn new( inner: S, _max_rate_per_second: u64, _dedup_ttl: Duration, ) -> Self
```

Create an enhanced stream with rate limiting and deduplication

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, window: Duration) -> Self
```

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, window_size: usize) -> Self
```

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, periods: usize) -> Self
```

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, window: usize, std_dev_multiplier: f64) -> Self
```

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, period: usize) -> Self
```

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, depth_levels: usize) -> Self
```

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, lookback_period: usize) -> Self
```

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S) -> Self
```

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, window: Duration) -> Self
```

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, window_size: usize) -> Self
```

---

### new

**Source**: `core/manager.rs`

```rust
pub fn new() -> Self
```

Create a new StreamManager

---

### new

**Source**: `core/manager.rs`

```rust
pub fn new(name: impl Into<String>) -> Self
```

---

### new

**Source**: `core/metrics.rs`

```rust
pub fn new() -> Self
```

Create a new metrics collector

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, transform: F) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, predicate: F) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(stream1: S1, stream2: S2) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, batch_size: usize, timeout: Duration) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, duration: Duration) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, duration: Duration) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, count: usize) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, count: usize) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, initial_state: St, transform: F) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, strategy: BufferStrategy) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, filter: FilterF, map: MapF) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, max_retries: usize, retry_delay: Duration) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, error_handler: F) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(event: Arc<dyn Event>, source_stream: String) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(streams: Vec<S>) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, buffer_size: usize) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(stream1: S1, stream2: S2) -> Self
```

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(stream1: S1, stream2: S2) -> Self
```

---

### new

**Source**: `core/stream.rs`

```rust
pub fn new(stream: S) -> Self
```

---

### new

**Source**: `core/streamed_event.rs`

```rust
pub fn new(inner: T, stream_metadata: StreamMetadata) -> Self
```

Create a new streamed event by wrapping an existing event

---

### new

**Source**: `core/connection.rs`

```rust
pub fn new(config: ConnectionConfig) -> Self
```

---

### new

**Source**: `core/connection.rs`

```rust
pub fn new(config: ConnectionConfig) -> Self
```

---

### new

**Source**: `core/connection.rs`

```rust
pub fn new(config: ConnectionConfig, pool_size: usize) -> Self
```

---

### new

**Source**: `core/processor.rs`

```rust
pub fn new(window_type: WindowType) -> Self
```

---

### new

**Source**: `core/processor.rs`

```rust
pub fn new(checkpoint_interval: Duration) -> Self
```

---

### new

**Source**: `core/processor.rs`

```rust
pub fn new(config: BackpressureConfig) -> Self
```

---

### new

**Source**: `core/processor.rs`

```rust
pub fn new(config: BatchConfig) -> Self
```

---

### new

**Source**: `core/processor.rs`

```rust
pub fn new(patterns: Vec<EventPattern<E>>, max_history: usize) -> Self
```

---

### new

**Source**: `core/mock_stream.rs`

```rust
pub fn new(name: impl Into<String>) -> Self
```

Create a new mock stream

---

### new

**Source**: `evm/multi_chain.rs`

```rust
pub fn new() -> Self
```

Create a new multi-chain EVM manager

---

### new

**Source**: `evm/websocket.rs`

```rust
pub fn new(name: impl Into<String>) -> Self
```

Create a new EVM WebSocket stream

---

### new

**Source**: `external/binance.rs`

```rust
pub fn new(name: impl Into<String>) -> Self
```

Create a new Binance stream

---

### new

**Source**: `external/mempool.rs`

```rust
pub fn new(name: impl Into<String>) -> Self
```

Create a new Mempool.space stream

---

### new

**Source**: `production/health.rs`

```rust
pub fn new(stream_manager: Arc<StreamManager>) -> Self
```

Create a new health monitor

---

### new

**Source**: `production/monitoring.rs`

```rust
pub fn new() -> Self
```

Create new metrics collector

---

### new

**Source**: `production/monitoring.rs`

```rust
pub fn new() -> Self
```

Create new collector

---

### new

**Source**: `production/resilience.rs`

```rust
pub fn new(name: impl Into<String>) -> Self
```

Create new circuit breaker

---

### new

**Source**: `solana/geyser.rs`

```rust
pub fn new(name: impl Into<String>) -> Self
```

Create a new Solana Geyser stream

---

### new

**Source**: `tools/condition.rs`

```rust
pub fn new(event_kinds: Vec<EventKind>) -> Self
```

---

### new

**Source**: `tools/condition.rs`

```rust
pub fn new(sources: Vec<String>) -> Self
```

---

### new

**Source**: `tools/condition.rs`

```rust
pub fn new(min_timestamp: Option<std::time::SystemTime>, max_timestamp: Option<std::time::SystemTime>) -> Self
```

---

### new

**Source**: `tools/condition.rs`

```rust
pub fn new(predicate: F, description: impl Into<String>) -> Self
```

---

### new

**Source**: `tools/condition.rs`

```rust
pub fn new(combinator: ConditionCombinator) -> Self
```

---

### new

**Source**: `tools/event_triggered.rs`

```rust
pub fn new(tool: T, name: impl Into<String>) -> Self
```

Create a new event-triggered tool

---

### new

**Source**: `tools/event_triggered.rs`

```rust
pub fn new(tool: T, name: impl Into<String>) -> Self
```

Create a new builder

---

### new

**Source**: `tools/worker_extension.rs`

```rust
pub fn new( worker_id: String, stream_manager: Arc<StreamManager>, ) -> Self
```

Create a new streaming tool worker

---

### pending_count

**Source**: `core/processor.rs`

```rust
pub fn pending_count(&self) -> usize
```

---

### permanent_connection

**Source**: `core/error.rs`

```rust
pub fn permanent_connection(message: impl Into<String>) -> Self
```

Create a permanent connection error

---

### pool_health

**Source**: `core/connection.rs`

```rust
pub async fn pool_health(&self) -> Vec<ConnectionHealth>
```

---

### process_events

**Source**: `core/manager.rs`

```rust
pub async fn process_events(&self) -> StreamResult<()>
```

Process events from all streams

---

### rate_limited

**Source**: `core/error.rs`

```rust
pub fn rate_limited(message: impl Into<String>, retry_after_seconds: u64) -> Self
```

Create a rate limit error with retry after duration

---

### record_dropped_event

**Source**: `core/metrics.rs`

```rust
pub async fn record_dropped_event(&self, stream_name: &str)
```

Record a dropped event

---

### record_error

**Source**: `production/monitoring.rs`

```rust
pub async fn record_error(&self, error_type: &str)
```

Record an error

---

### record_event

**Source**: `production/monitoring.rs`

```rust
pub async fn record_event(&self, event_type: &str, latency_ms: f64)
```

Record an event

---

### record_failure

**Source**: `production/resilience.rs`

```rust
pub async fn record_failure(&self)
```

Record failure

---

### record_handler_execution

**Source**: `core/metrics.rs`

```rust
pub async fn record_handler_execution( &self, handler_name: &str, execution_time_ms: f64, success: bool, )
```

Record a handler execution

---

### record_reconnection

**Source**: `core/metrics.rs`

```rust
pub async fn record_reconnection(&self, stream_name: &str)
```

Record a stream reconnection

---

### record_stream_event

**Source**: `core/metrics.rs`

```rust
pub async fn record_stream_event( &self, stream_name: &str, processing_time_ms: f64, bytes: u64, )
```

Record a stream event

---

### record_success

**Source**: `production/resilience.rs`

```rust
pub async fn record_success(&self)
```

Record success

---

### register

**Source**: `tools/event_triggered.rs`

```rust
pub async fn register(self, manager: &StreamManager) -> Arc<EventTriggeredTool<T>> where T: 'static
```

Register with a stream manager

---

### register_all_configured_chains

**Source**: `evm/multi_chain.rs`

```rust
pub async fn register_all_configured_chains(&self) -> Result<(), StreamError>
```

Register all configured chains

---

### release_permit

**Source**: `core/processor.rs`

```rust
pub async fn release_permit(&self)
```

---

### reliable

**Source**: `core/config.rs`

```rust
pub fn reliable() -> Self
```

Reliable configuration prioritizing data integrity

---

### remove_chain

**Source**: `evm/multi_chain.rs`

```rust
pub async fn remove_chain(&self, chain_id: ChainId) -> Result<(), StreamError>
```

Remove a chain

---

### remove_state

**Source**: `core/processor.rs`

```rust
pub async fn remove_state(&self, key: &K) -> Option<S>
```

---

### remove_stream

**Source**: `core/manager.rs`

```rust
pub async fn remove_stream(&self, name: &str) -> StreamResult<()>
```

Remove a stream

---

### report_interval

**Source**: `core/config.rs`

```rust
pub fn report_interval(&self) -> Duration
```

---

### request_timeout

**Source**: `core/config.rs`

```rust
pub fn request_timeout(&self) -> Duration
```

---

### reset_metrics

**Source**: `core/enhanced_operators.rs`

```rust
pub async fn reset_metrics(&self)
```

Reset performance metrics

---

### retriable_connection

**Source**: `core/error.rs`

```rust
pub fn retriable_connection(message: impl Into<String>) -> Self
```

Create a retriable connection error

---

### retry_after

**Source**: `core/error.rs`

```rust
pub fn retry_after(&self) -> Option<std::time::Duration>
```

Get retry after duration if available

---

### retry_base_delay

**Source**: `core/config.rs`

```rust
pub fn retry_base_delay(&self) -> Duration
```

---

### retry_max_delay

**Source**: `core/config.rs`

```rust
pub fn retry_max_delay(&self) -> Duration
```

---

### serialize

**Source**: `core/mod.rs`

```rust
pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer,
```

---

### set_execution_mode

**Source**: `core/manager.rs`

```rust
pub async fn set_execution_mode(&self, mode: HandlerExecutionMode)
```

Set the handler execution mode

---

### single

**Source**: `tools/condition.rs`

```rust
pub fn single(event_kind: EventKind) -> Self
```

---

### single

**Source**: `tools/condition.rs`

```rust
pub fn single(source: String) -> Self
```

---

### slow_threshold

**Source**: `core/config.rs`

```rust
pub fn slow_threshold(&self) -> Duration
```

---

### source_stream

**Source**: `core/operators.rs`

```rust
pub fn source_stream(&self) -> &str
```

Get the source stream name

---

### start

**Source**: `core/metrics.rs`

```rust
pub fn start(name: impl Into<String>) -> Self
```

Start a new timer

---

### start

**Source**: `production/health.rs`

```rust
pub async fn start(&self)
```

Start monitoring

---

### start

**Source**: `tools/worker_extension.rs`

```rust
pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>
```

Start processing events

---

### start_all

**Source**: `core/manager.rs`

```rust
pub async fn start_all(&self) -> StreamResult<()>
```

Start all registered streams

---

### start_stream

**Source**: `core/manager.rs`

```rust
pub async fn start_stream(&self, name: &str) -> StreamResult<()>
```

Start a specific stream (stream should already be configured)

---

### start_with_collector

**Source**: `core/metrics.rs`

```rust
pub fn start_with_collector(name: impl Into<String>, collector: Arc<MetricsCollector>) -> Self
```

Start a timer with a collector

---

### state

**Source**: `core/manager.rs`

```rust
pub async fn state(&self) -> ManagerState
```

Get manager state

---

### state

**Source**: `core/connection.rs`

```rust
pub fn state(&self) -> ConnectionState
```

---

### stop

**Source**: `core/metrics.rs`

```rust
pub async fn stop(self, success: bool)
```

Stop timer and record metrics

---

### stop

**Source**: `tools/worker_extension.rs`

```rust
pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>>
```

Stop the worker

---

### stop_all

**Source**: `core/manager.rs`

```rust
pub async fn stop_all(&self) -> StreamResult<()>
```

Stop all streams

---

### stop_all

**Source**: `evm/multi_chain.rs`

```rust
pub async fn stop_all(&self) -> Result<(), StreamError>
```

Stop all streams

---

### stop_stream

**Source**: `core/manager.rs`

```rust
pub async fn stop_stream(&self, name: &str) -> StreamResult<()>
```

Stop a specific stream

---

### stream_meta

**Source**: `core/streamed_event.rs`

```rust
pub fn stream_meta(&self) -> &StreamMetadata
```

Get the stream metadata

---

### stream_meta

**Source**: `core/streamed_event.rs`

```rust
pub fn stream_meta(&self) -> &StreamMetadata
```

Get the stream metadata

---

### to_event_error

**Source**: `core/error.rs`

```rust
pub fn to_event_error(self) -> EventError
```

Convert StreamError to EventError for integration with events-core

---

### update_queue_size

**Source**: `core/processor.rs`

```rust
pub async fn update_queue_size(&self, delta: i32)
```

---

### update_rates

**Source**: `core/metrics.rs`

```rust
pub async fn update_rates(&self)
```

Update rates (should be called periodically)

---

### update_state

**Source**: `core/processor.rs`

```rust
pub async fn update_state<F, R>(&self, key: K, update_fn: F) -> R where F: FnOnce(Option<&S>) -> (S, R),
```

---

### validate

**Source**: `core/config.rs`

```rust
pub fn validate(&self) -> Result<(), ConfigError>
```

---

### validate

**Source**: `core/config.rs`

```rust
pub fn validate(&self) -> Result<(), ConfigError>
```

---

### validate

**Source**: `core/config.rs`

```rust
pub fn validate(&self) -> Result<(), ConfigError>
```

---

### validate

**Source**: `core/config.rs`

```rust
pub fn validate(&self) -> Result<(), ConfigError>
```

Validate all configuration settings

---

### window_duration

**Source**: `core/config.rs`

```rust
pub fn window_duration(&self) -> Duration
```

---

### with_check_interval

**Source**: `production/health.rs`

```rust
pub fn with_check_interval(mut self, interval: Duration) -> Self
```

Set check interval

---

### with_combinator

**Source**: `tools/event_triggered.rs`

```rust
pub fn with_combinator(mut self, combinator: ConditionCombinator) -> Self
```

Set condition combinator

---

### with_condition

**Source**: `tools/event_triggered.rs`

```rust
pub fn with_condition(mut self, condition: Box<dyn EventCondition>) -> Self
```

Add a condition

---

### with_connection

**Source**: `core/connection.rs`

```rust
pub async fn with_connection<F, R>(&self, f: F) -> StreamResult<R> where F: FnOnce(&T) -> R, T: Clone,
```

---

### with_execution_mode

**Source**: `core/builder.rs`

```rust
pub fn with_execution_mode(mut self, mode: HandlerExecutionMode) -> Self
```

Set the handler execution mode

---

### with_execution_mode

**Source**: `core/manager.rs`

```rust
pub fn with_execution_mode(mode: HandlerExecutionMode) -> Self
```

Create a new StreamManager with specified execution mode

---

### with_failure_threshold

**Source**: `production/resilience.rs`

```rust
pub fn with_failure_threshold(mut self, threshold: u64) -> Self
```

Set failure threshold

---

### with_metrics

**Source**: `core/builder.rs`

```rust
pub fn with_metrics(mut self, enable: bool) -> Self
```

Enable or disable metrics collection

---

### with_metrics_collector

**Source**: `core/builder.rs`

```rust
pub fn with_metrics_collector(mut self, collector: Arc<MetricsCollector>) -> Self
```

Use a custom metrics collector

---

### with_stream_manager

**Source**: `evm/multi_chain.rs`

```rust
pub fn with_stream_manager(mut self, manager: Arc<StreamManager>) -> Self
```

Set the stream manager

---

### with_success_threshold

**Source**: `production/resilience.rs`

```rust
pub fn with_success_threshold(mut self, threshold: u64) -> Self
```

Set success threshold

---

### with_thresholds

**Source**: `production/health.rs`

```rust
pub fn with_thresholds(mut self, thresholds: HealthThresholds) -> Self
```

Set custom thresholds

---

### with_timeout

**Source**: `production/resilience.rs`

```rust
pub fn with_timeout(mut self, timeout: Duration) -> Self
```

Set timeout

---

### zip

**Source**: `core/operators.rs`

```rust
pub fn zip<S1, S2>(stream1: S1, stream2: S2) -> ZipStream<S1, S2> where S1: Stream, S2: Stream,
```

Zip two streams together

---

## Traits

### AsNumeric

**Source**: `core/financial_operators.rs`

```rust
pub trait AsNumeric { ... }
```

Trait for extracting numeric data from events for financial analysis

**Methods**:

#### `as_price`

```rust
fn as_price(&self) -> Option<f64>;
```

#### `as_volume`

```rust
fn as_volume(&self) -> Option<f64>;
```

#### `as_market_cap`

```rust
fn as_market_cap(&self) -> Option<f64> {
```

#### `as_timestamp_ms`

```rust
fn as_timestamp_ms(&self) -> Option<i64> {
```

#### `as_custom_numeric`

```rust
fn as_custom_numeric(&self, _field: &str) -> Option<f64> {
```

---

### ComposableStream

**Source**: `core/operators.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait ComposableStream: Stream { ... }
```

Trait for composable streams

**Methods**:

#### `map`

```rust
fn map<F, R>(self, f: F) -> MappedStream<Self, F, R> where Self: Sized, F: Fn(Arc<Self::Event>) -> R + Send + Sync + 'static, R: Event + Clone + Send + Sync + 'static, {;
```

#### `filter`

```rust
fn filter<F>(self, predicate: F) -> FilteredStream<Self, F> where Self: Sized, F: Fn(&Self::Event) -> bool + Send + Sync + 'static, {;
```

#### `merge`

```rust
fn merge<S>(self, other: S) -> MergedStream<Self, S> where Self: Sized, S: Stream, S::Event: Event + Clone + Send + Sync + 'static, {;
```

#### `batch`

```rust
fn batch(self, size: usize, timeout: Duration) -> BatchedStream<Self> where Self: Sized, {;
```

#### `debounce`

```rust
fn debounce(self, duration: Duration) -> DebouncedStream<Self> where Self: Sized, {;
```

#### `throttle`

```rust
fn throttle(self, duration: Duration) -> ThrottledStream<Self> where Self: Sized, {;
```

#### `with_guaranteed_delivery`

```rust
fn with_guaranteed_delivery(self, buffer_size: usize) -> GuaranteedDeliveryStream<Self> where Self: Sized, {;
```

#### `take`

```rust
fn take(self, count: usize) -> TakeStream<Self> where Self: Sized, {;
```

#### `skip`

```rust
fn skip(self, count: usize) -> SkipStream<Self> where Self: Sized, {;
```

#### `scan`

```rust
fn scan<S, F>(self, initial_state: S, f: F) -> ScanStream<Self, S, F> where Self: Sized, S: Clone + Send + Sync + 'static, F: Fn(S, Arc<Self::Event>) -> S + Send + Sync + 'static, {;
```

#### `buffer`

```rust
fn buffer(self, strategy: BufferStrategy) -> BufferedStream<Self> where Self: Sized, {;
```

---

### DynamicStream

**Source**: `core/stream.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait DynamicStream: Send + Sync { ... }
```

Type-erased stream for heterogeneous stream management

**Methods**:

#### `start_dynamic`

```rust
async fn start_dynamic(&mut self) -> Result<(), StreamError>;
```

#### `stop_dynamic`

```rust
async fn stop_dynamic(&mut self) -> Result<(), StreamError>;
```

#### `is_running_dynamic`

```rust
fn is_running_dynamic(&self) -> bool;
```

#### `health_dynamic`

```rust
async fn health_dynamic(&self) -> StreamHealth;
```

#### `name_dynamic`

```rust
fn name_dynamic(&self) -> &str;
```

#### `subscribe_dynamic`

```rust
fn subscribe_dynamic(&self) -> broadcast::Receiver<Arc<dyn Any + Send + Sync>>;
```

---

### EnhancedStreamExt

**Source**: `core/enhanced_operators.rs`

```rust
pub trait EnhancedStreamExt: Stream + Sized { ... }
```

Extension trait to add enhanced operators to existing streams

**Methods**:

#### `with_batching`

```rust
fn with_batching(self, batch_size: usize, timeout: Duration) -> BatchedStream<Self> where Self: 'static, Self::Event: Event + Clone, {;
```

#### `with_enhancements`

```rust
fn with_enhancements( self, max_rate_per_second: u64, dedup_ttl: Duration, ) -> EnhancedStream<Self> where Self: 'static {;
```

---

### EventCondition

**Source**: `tools/condition.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait EventCondition: Send + Sync { ... }
```

Trait for event conditions

**Methods**:

#### `matches`

```rust
async fn matches(&self, event: &(dyn Any + Send + Sync)) -> bool;
```

#### `description`

```rust
fn description(&self) -> String;
```

---

### EventConversion

**Source**: `core/event_adapter.rs`

```rust
pub trait EventConversion { ... }
```

Event adapter module - Legacy adapter functionality removed

All events now implement the Event trait from riglr-events-core directly.
The UnifiedEvent trait has been deprecated and removed.
Placeholder trait for backward compatibility

**Methods**:

#### `to_event`

```rust
fn to_event(self: Box<Self>) -> Box<dyn riglr_events_core::Event>;
```

---

### EventHandler

**Source**: `core/manager.rs`

**Attributes**:
```rust
#[async_trait::async_trait]
```

```rust
pub trait EventHandler: Send + Sync { ... }
```

Trait for handling stream events

**Methods**:

#### `should_handle`

```rust
async fn should_handle(&self, event: &(dyn Any + Send + Sync)) -> bool;
```

#### `handle`

```rust
async fn handle(&self, event: Arc<dyn Any + Send + Sync>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
```

#### `name`

```rust
fn name(&self) -> &str;
```

---

### EventMatcher

**Source**: `tools/condition.rs`

```rust
pub trait EventMatcher { ... }
```

Helper trait for pattern matching

**Methods**:

#### `event_kind`

```rust
fn event_kind(event_kind: EventKind) -> Box<dyn EventCondition> {
```

#### `source`

```rust
fn source(source: String) -> Box<dyn EventCondition> {
```

#### `timestamp_range`

```rust
fn timestamp_range(min: Option<std::time::SystemTime>, max: Option<std::time::SystemTime>) -> Box<dyn EventCondition> {
```

#### `all`

```rust
fn all(conditions: Vec<Box<dyn EventCondition>>) -> Box<dyn EventCondition> {
```

#### `any`

```rust
fn any(conditions: Vec<Box<dyn EventCondition>>) -> Box<dyn EventCondition> {
```

---

### FinancialStreamExt

**Source**: `core/financial_operators.rs`

```rust
pub trait FinancialStreamExt: ComposableStream { ... }
```

Extension trait for adding financial operators

**Methods**:

#### `vwap`

```rust
fn vwap(self, window: Duration) -> VwapStream<Self> where Self: Sized, Self::Event: AsNumeric, {;
```

#### `moving_average`

```rust
fn moving_average(self, window_size: usize) -> MovingAverageStream<Self> where Self: Sized, {;
```

#### `ema`

```rust
fn ema(self, periods: usize) -> EmaStream<Self> where Self: Sized, {;
```

#### `bollinger_bands`

```rust
fn bollinger_bands(self, window: usize, std_dev: f64) -> BollingerBandsStream<Self> where Self: Sized, {;
```

#### `rsi`

```rust
fn rsi(self, period: usize) -> RsiStream<Self> where Self: Sized, {;
```

#### `momentum`

```rust
fn momentum(self, lookback: usize) -> MomentumStream<Self> where Self: Sized, {;
```

#### `liquidity_pools`

```rust
fn liquidity_pools(self) -> LiquidityPoolStream<Self> where Self: Sized, {;
```

#### `mev_detection`

```rust
fn mev_detection(self, window: Duration) -> MevDetectionStream<Self> where Self: Sized, {;
```

#### `gas_oracle`

```rust
fn gas_oracle(self, window_size: usize) -> GasPriceOracleStream<Self> where Self: Sized, {;
```

---

### IntoDynamicStreamedEvent

**Source**: `core/streamed_event.rs`

```rust
pub trait IntoDynamicStreamedEvent { ... }
```

Helper trait for creating DynamicStreamedEvent from Box<dyn Event>

**Methods**:

#### `with_stream_metadata`

```rust
fn with_stream_metadata(self, metadata: StreamMetadata) -> DynamicStreamedEvent;
```

#### `with_default_stream_metadata`

```rust
fn with_default_stream_metadata(self, source: impl Into<String>) -> DynamicStreamedEvent;
```

---

### IntoStreamedEvent

**Source**: `core/streamed_event.rs`

```rust
pub trait IntoStreamedEvent: Event + Clone + Sized { ... }
```

Helper trait to easily wrap events with streaming metadata

**Methods**:

#### `with_stream_metadata`

```rust
fn with_stream_metadata(self, metadata: StreamMetadata) -> StreamedEvent<Self> {
```

#### `with_default_stream_metadata`

```rust
fn with_default_stream_metadata(self, source: impl Into<String>) -> StreamedEvent<Self> {
```

---

### PerformanceStreamExt

**Source**: `core/operators.rs`

```rust
pub trait PerformanceStreamExt: ComposableStream { ... }
```

Extension trait for performance optimizations

**Methods**:

#### `filter_map`

```rust
fn filter_map<FilterF, MapF, R>(self, filter: FilterF, map: MapF) -> FilterMapStream<Self, FilterF, MapF, R> where Self: Sized, FilterF: Fn(&Self::Event) -> bool + Send + Sync + 'static, MapF: Fn(Arc<Self::Event>) -> R + Send + Sync + 'static, R: Event + Clone + Send + Sync + 'static, {;
```

---

### ResilienceStreamExt

**Source**: `core/operators.rs`

```rust
pub trait ResilienceStreamExt: ComposableStream { ... }
```

Extension trait for resilience and error handling

**Methods**:

#### `with_retries`

```rust
fn with_retries(self, max_retries: usize, retry_delay: Duration) -> ResilientStream<Self> where Self: Sized, {;
```

#### `catch_errors`

```rust
fn catch_errors<F>(self, error_handler: F) -> CatchErrorStream<Self, F> where Self: Sized, F: Fn(StreamError) -> Option<Arc<Self::Event>> + Send + Sync + 'static, {;
```

---

### Stream

**Source**: `core/stream.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait Stream: Send + Sync { ... }
```

Core trait that all streams must implement

**Methods**:

#### `start`

```rust
async fn start(&mut self, config: Self::Config) -> Result<(), StreamError>;
```

#### `stop`

```rust
async fn stop(&mut self) -> Result<(), StreamError>;
```

#### `subscribe`

```rust
fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>>;
```

#### `is_running`

```rust
fn is_running(&self) -> bool;
```

#### `health`

```rust
async fn health(&self) -> StreamHealth;
```

#### `name`

```rust
fn name(&self) -> &str;
```

---

### StreamEvent

**Source**: `core/stream.rs`

```rust
pub trait StreamEvent: Send + Sync { ... }
```

Extension trait for events with streaming context

**Methods**:

#### `stream_metadata`

```rust
fn stream_metadata(&self) -> Option<&StreamMetadata>;
```

#### `stream_source`

```rust
fn stream_source(&self) -> Option<&str> {
```

#### `received_at`

```rust
fn received_at(&self) -> Option<SystemTime> {
```

#### `sequence_number`

```rust
fn sequence_number(&self) -> Option<u64> {
```

---

### StreamingTool

**Source**: `tools/event_triggered.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait StreamingTool: Send + Sync { ... }
```

Generic tool trait for event-triggered execution

**Methods**:

#### `execute`

```rust
async fn execute(&self, event: &dyn Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
```

#### `name`

```rust
fn name(&self) -> &str;
```

---

## Type Aliases

### SolanaStreamEvent

**Source**: `solana/geyser.rs`

```rust
type SolanaStreamEvent = DynamicStreamedEvent
```

Type alias for streamed Solana events using protocol-specific parsing

---


---

*This documentation was automatically generated from the source code.*