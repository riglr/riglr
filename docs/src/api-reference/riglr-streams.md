# riglr-streams API Reference

Comprehensive API documentation for the `riglr-streams` crate.

## Table of Contents

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

### Type Aliases

- [`SolanaStreamEvent`](#solanastreamevent)

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
fn with_enhancements( self, max_rate_per_second: u64, dedup_ttl: Duration, ) -> EnhancedStream<Self> where Self: 'static, {;
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
async fn handle( &self, event: Arc<dyn Any + Send + Sync>, ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
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
fn timestamp_range( min: Option<std::time::SystemTime>, max: Option<std::time::SystemTime>, ) -> Box<dyn EventCondition> {;
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
fn filter_map<FilterF, MapF, R>( self, filter: FilterF, map: MapF, ) -> FilterMapStream<Self, FilterF, MapF, R> where Self: Sized, FilterF: Fn(&Self::Event) -> bool + Send + Sync + 'static, MapF: Fn(Arc<Self::Event>) -> R + Send + Sync + 'static, R: Event + Clone + Send + Sync + 'static, {;
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
async fn execute( &self, event: &dyn Event, ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
```

#### `name`

```rust
fn name(&self) -> &str;
```

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
pub struct BatchEvent<E: Event> { /// The collection of events that are batched together pub events: Vec<E>, /// When the batch was created pub batch_timestamp: SystemTime, /// Unique identifier for this batch pub batch_id: String, /// Event metadata for the batch event pub metadata: EventMetadata, }
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
pub struct BatchEvent<E> { /// The events in this batch pub events: Vec<Arc<E>>, /// Unique identifier for this batch pub batch_id: String, /// Timestamp when the batch was created pub timestamp: std::time::SystemTime, }
```

Event wrapper for batched events

---

### BatchProcessor

**Source**: `core/processor.rs`

```rust
pub struct BatchProcessor<E> { /// Batching configuration settings config: BatchConfig, /// Buffer for accumulating events before batching batch_buffer: VecDeque<E>, /// When the last batch was flushed last_flush: Instant, }
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
pub struct BatchedStream<S> { /// The underlying stream inner: S, /// Maximum number of events per batch batch_size: usize, /// Maximum time to wait before emitting a batch timeout: Duration, /// Name of the batched stream name: String, }
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
pub struct BitcoinBlock { /// Block ID (hash)
```

Bitcoin block data

---

### BitcoinTransaction

**Source**: `external/mempool.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct BitcoinTransaction { /// Transaction ID pub txid: String, /// Transaction fee in satoshis pub fee: u64, /// Virtual size of the transaction pub vsize: u32, /// Total value in satoshis pub value: u64, /// Whether the transaction is confirmed #[serde(default)]
```

Bitcoin transaction data

---

### BollingerBands

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct BollingerBands { /// Upper band (mean + std_dev_multiplier * std_dev)
```

Bollinger Bands calculation result

---

### BollingerBandsStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct BollingerBandsStream<S> { /// The underlying stream to process inner: S, /// Number of periods for the moving average and standard deviation window: usize, /// Multiplier for standard deviation to create upper/lower bands std_dev_multiplier: f64, /// Sliding window of values for calculation values: Arc<RwLock<VecDeque<f64>>>, }
```

Bollinger Bands calculation stream

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
pub struct CircuitBreaker { /// Configuration for connection behavior config: ConnectionConfig, /// Current atomic state of the circuit breaker state: AtomicConnectionState, /// Atomic counter for consecutive failures failure_count: AtomicUsize, /// Timestamp of the last connection failure last_failure: Mutex<Option<Instant>>, /// Timestamp of the last successful connection last_success: Mutex<Option<Instant>>, }
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
pub struct ConnectionHealth { /// Current state of the connection pub state: ConnectionState, /// Timestamp when the connection was established pub connected_at: Option<Instant>, /// Timestamp of the last activity on this connection pub last_activity: Option<Instant>, /// Total number of reconnection attempts made pub total_reconnects: usize, /// Number of consecutive failures experienced pub consecutive_failures: usize, /// Current latency in milliseconds, if available pub latency_ms: Option<u64>, }
```

Connection health metrics

---

### ConnectionManager

**Source**: `core/connection.rs`

```rust
pub struct ConnectionManager<T> { /// Circuit breaker for connection failure handling circuit_breaker: Arc<CircuitBreaker>, /// The managed connection, if active connection: Arc<RwLock<Option<T>>>, /// Health metrics and status of the connection health: Arc<RwLock<ConnectionHealth>>, /// Flag indicating if reconnection monitoring is active reconnect_task: Arc<AtomicBool>, }
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
pub struct ConnectionPool<T> { /// Collection of managed connections in the pool connections: Vec<ConnectionManager<T>>, /// Configuration applied to all connections in the pool config: ConnectionConfig, /// Current index for round-robin connection selection current_index: AtomicUsize, }
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
pub struct DebouncedStream<S> { /// The underlying stream inner: S, /// Debounce duration duration: Duration, /// Name of the debounced stream name: String, }
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
pub struct EmaStream<S> { /// The underlying stream to process inner: S, /// Smoothing factor (alpha) for the EMA calculation
```

Exponential moving average stream

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
pub struct FeeEstimate { /// Fastest fee estimate in sat/vB #[serde(rename = "fastestFee")]
```

Bitcoin fee estimate data

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
pub struct FilteredStream<S, F> { /// The underlying stream inner: S, /// The filter predicate predicate: Arc<F>, }
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
pub struct FinancialEvent<T, E> { /// Event metadata containing ID, kind, and source information pub metadata: riglr_events_core::EventMetadata, /// The calculated indicator value (e.g., VWAP, RSI, etc.)
```

Financial indicator event wrapper

---

### FlowController

**Source**: `core/processor.rs`

```rust
pub struct FlowController { /// Backpressure configuration settings config: BackpressureConfig, /// Semaphore for controlling concurrent access semaphore: Arc<Semaphore>, /// Current queue size counter queue_size: Arc<tokio::sync::RwLock<usize>>, /// Number of dropped events counter drop_count: Arc<tokio::sync::RwLock<usize>>, }
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
pub struct GasPriceEstimate { /// 25th percentile gas price (slow transactions)
```

Gas price estimates at different priority levels

---

### GasPriceOracleStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct GasPriceOracleStream<S> { /// The underlying stream to process inner: S, /// Percentiles to calculate for gas price estimates percentiles: Vec<usize>, /// Historical gas prices for percentile calculation gas_prices: Arc<RwLock<VecDeque<f64>>>, /// Number of historical prices to maintain window_size: usize, }
```

Gas price oracle stream for tracking network gas prices

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
pub struct KlineData { /// Trading pair symbol (e.g., "BTCUSDT")
```

Kline/candlestick stream data

---

### KlineDetails

**Source**: `external/binance.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct KlineDetails { /// Kline start time timestamp #[serde(rename = "t")]
```

Detailed kline/candlestick data

---

### LiquidityPoolStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct LiquidityPoolStream<S> { /// The underlying stream to process inner: S, /// Map of pool ID to pool state using high-performance concurrent map pools: Arc<DashMap<String, PoolState>>, }
```

Liquidity pool balance tracker stream

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
pub struct MappedStream<S, F, R> { /// The underlying stream inner: S, /// The transformation function transform: Arc<F>, /// Phantom data for the result type _phantom: PhantomData<R>, }
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
pub struct MempoolStats { /// Number of transactions in mempool pub count: u32, /// Total virtual size of mempool transactions pub vsize: u64, /// Total fees in mempool pub total_fee: u64, /// Fee rate histogram pub fee_histogram: Vec<Vec<u64>>, }
```

Mempool statistics data

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
pub struct MergedStream<S1, S2> { /// First stream to merge stream1: S1, /// Second stream to merge stream2: S2, /// Name of the merged stream name: String, }
```

Stream that merges events from two streams

---

### MetricsCollector

**Source**: `core/metrics.rs`

```rust
pub struct MetricsCollector { /// Stream metrics stream_metrics: Arc<DashMap<String, StreamMetrics>>, /// Handler metrics handler_metrics: Arc<DashMap<String, HandlerMetrics>>, /// Global metrics global_metrics: Arc<RwLock<GlobalMetrics>>, /// Metrics history for time-series data metrics_history: Arc<RwLock<Vec<(SystemTime, GlobalMetrics)>>>,
```

Metrics collector for the streaming system

---

### MetricsCollector

**Source**: `production/monitoring.rs`

```rust
pub struct MetricsCollector { /// Metrics by stream stream_metrics: Arc<DashMap<String, StreamMetrics>>, }
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
pub struct MevDetectionStream<S> { /// The underlying stream to process inner: S, /// Time window for MEV pattern detection window: Duration, /// Transaction patterns within the detection window transactions: Arc<RwLock<VecDeque<TransactionPattern>>>, }
```

MEV (Maximum Extractable Value) detection stream

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
pub struct MomentumStream<S> { /// The underlying stream to process inner: S, /// Number of periods to look back for momentum calculation lookback_period: usize, /// Historical price data for momentum calculation price_history: Arc<RwLock<VecDeque<f64>>>, }
```

Price momentum indicator stream

---

### MovingAverageStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct MovingAverageStream<S> { /// The underlying stream to process inner: S, /// Number of values to include in the moving average window_size: usize, /// Sliding window of values for calculation values: Arc<RwLock<VecDeque<f64>>>, }
```

Moving average calculation stream

---

### MultiChainEvmManager

**Source**: `evm/multi_chain.rs`

**Attributes**:
```rust
#[derive(Default)]
```

```rust
pub struct MultiChainEvmManager { /// Streams by chain ID streams: Arc<DashMap<ChainId, EvmWebSocketStream>>, /// Stream manager reference stream_manager: Option<Arc<StreamManager>>, }
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
pub struct OrderBookData { /// Trading pair symbol (e.g., "BTCUSDT")
```

Order book depth update data

---

### OrderBookImbalanceStream

**Source**: `core/financial_operators.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct OrderBookImbalanceStream<S> { /// The underlying stream to process inner: S, /// Number of order book depth levels to consider depth_levels: usize, }
```

Order book imbalance calculator stream

---

### PatternMatcher

**Source**: `core/processor.rs`

```rust
pub struct PatternMatcher<E> { /// Patterns to match against events patterns: Vec<EventPattern<E>>, /// History of recent events with timestamps event_history: VecDeque<(E, Instant)>,
```

Complex event processing pattern matcher

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
pub struct PoolState { /// Reserve amount of token A pub token_a_reserve: f64, /// Reserve amount of token B pub token_b_reserve: f64, /// Constant product (k = x * y)
```

State of a liquidity pool

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
pub struct RetryPolicy { /// Maximum number of retries pub max_retries: u32, /// Initial delay between retry attempts pub initial_delay: Duration, /// Backoff strategy to use for calculating delays pub backoff: BackoffStrategy, /// Maximum delay between retry attempts pub max_delay: Duration, /// Whether to apply random jitter to delays pub jitter: bool, }
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
pub struct RsiStream<S> { /// The underlying stream to process inner: S, /// Period for RSI calculation (typically 14)
```

RSI (Relative Strength Index) calculation stream

---

### ScanEvent

**Source**: `core/operators.rs`

**Attributes**:
```rust
#[derive(Clone, Debug)]
```

```rust
pub struct ScanEvent<S, E> { /// Current state after transformation pub state: S, /// Original event that triggered the scan pub original_event: Arc<E>, /// Unique identifier for this scan event pub scan_id: String, /// Timestamp when the scan was performed pub timestamp: std::time::SystemTime, }
```

Event wrapper for scan results

---

### ScanStream

**Source**: `core/operators.rs`

```rust
pub struct ScanStream<S, St, F> { /// The underlying stream inner: S, /// Current state of the scan state: Arc<RwLock<St>>, /// State transformation function transform: Arc<F>, /// Name of the scan stream name: String, }
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
pub struct SkipStream<S> { /// The underlying stream inner: S, /// Number of events to skip count: usize, /// Counter of events skipped so far skipped: Arc<RwLock<usize>>, /// Name of the skip stream name: String, }
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
pub struct StatefulProcessor<K, S> where K: Hash + Eq + Clone, S: Clone, { /// Concurrent state storage using DashMap for better performance state_store: Arc<DashMap<K, S>>, /// How often to create checkpoints checkpoint_interval: Duration, /// When the last checkpoint was created last_checkpoint: Instant, }
```

Stateful event processor with checkpointing

---

### StreamClientConfig

**Source**: `core/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Default)]
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
pub struct StreamManager { /// Registered streams streams: Arc<DashMap<String, Box<dyn DynamicStream + 'static>>>, /// Running stream tasks running_streams: Arc<DashMap<String, JoinHandle<()>>>,
```

Manages multiple streams and routes events to handlers

---

### StreamManagerBuilder

**Source**: `core/builder.rs`

```rust
pub struct StreamManagerBuilder { /// Handler execution mode for processing stream events execution_mode: HandlerExecutionMode, /// Collection of configured streams to be managed streams: Vec<StreamConfig>, /// Flag to enable or disable metrics collection enable_metrics: bool, /// Optional custom metrics collector instance metrics_collector: Option<Arc<MetricsCollector>>, }
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
pub struct StreamManagerConfig { /// Optional handler execution mode override pub execution_mode: Option<HandlerExecutionMode>, /// Optional flag to enable or disable metrics collection pub enable_metrics: Option<bool>, /// Collection of stream configurations to load pub streams: Vec<StreamConfig>, }
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
pub struct StreamMetadata { /// Source identifier for the stream that generated this event pub stream_source: String, /// Timestamp when this event was received by the stream processor #[serde(with = "systemtime_as_millis")]
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
pub struct StreamMetrics { /// Start time start_time: SystemTime, /// Total events processed events_processed: Arc<AtomicU64>, /// Events by type events_by_type: Arc<DashMap<String, u64>>, /// Error count error_count: Arc<AtomicU64>, /// Performance stats performance_stats: Arc<RwLock<PerformanceStats>>, }
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
pub struct TakeStream<S> { /// The underlying stream inner: S, /// Maximum number of events to take count: usize, /// Counter of events taken so far taken: Arc<RwLock<usize>>, /// Name of the take stream name: String, }
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
pub struct ThrottledStream<S> { /// The underlying stream inner: S, /// Throttle interval duration: Duration, /// Name of the throttled stream name: String, }
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
pub struct TickerData { /// Trading pair symbol (e.g., "BTCUSDT")
```

24-hour ticker price change statistics data

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
pub struct TradeData { /// Trading pair symbol (e.g., "BTCUSDT")
```

Individual trade execution data

---

### TransactionEvent

**Source**: `solana/geyser.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct TransactionEvent { /// Transaction signature hash pub signature: String, /// Slot number where the transaction was confirmed pub slot: u64, }
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
pub struct VwapStream<S> { /// The underlying stream to process inner: S, /// Time window for VWAP calculation window: Duration, /// Sliding window of (price, volume, timestamp) tuples
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
pub struct Window<E> { /// Unique identifier for this window pub id: u64, /// When this window started pub start_time: Instant, /// When this window ended (if closed)
```

Window state for managing event windows

---

### WindowManager

**Source**: `core/processor.rs`

```rust
pub struct WindowManager<E> { /// Type of windowing to apply window_type: WindowType, /// Currently active windows indexed by ID active_windows: HashMap<u64, Window<E>>, /// Counter for generating unique window IDs next_window_id: u64, /// Last time cleanup was performed last_cleanup: Instant, }
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

Attempts to acquire a permit based on the configured backpressure strategy

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

Add a condition to this composite condition

---

### add_event

**Source**: `core/processor.rs`

```rust
pub fn add_event(&mut self, event: E) -> Vec<Window<E>>
```

Adds an event to the appropriate window(s) and returns any completed windows

---

### add_event

**Source**: `core/processor.rs`

```rust
pub fn add_event(&mut self, event: E) -> Option<Vec<E>>
```

Adds an event to the batch buffer and returns a completed batch if ready

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

Create a TimestampRangeMatcher that matches events after the given timestamp

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

Attempts to establish a connection using the provided function

Respects circuit breaker state and implements backoff logic

---

### batch_timeout

**Source**: `core/config.rs`

```rust
pub fn batch_timeout(&self) -> Duration
```

Convert batch timeout milliseconds to Duration

---

### before

**Source**: `tools/condition.rs`

```rust
pub fn before(timestamp: std::time::SystemTime) -> Self
```

Create a TimestampRangeMatcher that matches events before the given timestamp

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

Calculates order book imbalance from bid/ask data

---

### checkpoint

**Source**: `core/processor.rs`

```rust
pub async fn checkpoint(&self) -> StreamResult<()>
```

Creates a checkpoint of the current state if enough time has passed

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

Establishes a connection using the provided function

Starts background monitoring for automatic reconnection

---

### connect_all

**Source**: `core/connection.rs`

```rust
pub async fn connect_all<F, Fut>(&self, connect_fn: F) -> StreamResult<()> where F: Fn() -> Fut + Send + Sync + 'static + Clone, Fut: std::future::Future<Output = StreamResult<T>> + Send,
```

Attempts to connect all connections in the pool concurrently

---

### connect_timeout

**Source**: `core/config.rs`

```rust
pub fn connect_timeout(&self) -> Duration
```

Convert connection timeout seconds to Duration

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

Disconnects and stops monitoring the connection

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

Forces a flush of the current batch buffer

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
pub fn from_event(event: Box<dyn Event>, stream_metadata: StreamMetadata) -> Self
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

Returns a clone of the current connection, if available

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

Returns a healthy connection from the pool using round-robin selection

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

Retrieves the current state for the given key

---

### get_stats

**Source**: `core/processor.rs`

```rust
pub async fn get_stats(&self) -> (usize, usize)
```

Returns current queue size and drop count statistics

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

**Source**: `core/connection.rs`

```rust
pub async fn health(&self) -> ConnectionHealth
```

Returns the current health status of the connection

---

### health

**Source**: `core/manager.rs`

```rust
pub async fn health(&self) -> std::collections::HashMap<String, StreamHealth>
```

Get health status of all streams

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

Calculate the high watermark threshold in absolute terms

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

### inner

**Source**: `core/operators.rs`

```rust
pub fn inner(&self) -> &dyn Event
```

Get the inner event as a trait object

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

Convert keepalive interval seconds to Duration

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

Calculate the low watermark threshold in absolute terms

---

### mark_disconnected

**Source**: `core/connection.rs`

```rust
pub fn mark_disconnected(&self)
```

Marks the connection as explicitly disconnected

---

### match_event

**Source**: `core/processor.rs`

```rust
pub fn match_event(&mut self, event: E) -> Vec<usize>
```

Processes an event and returns indices of matching patterns

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

**Source**: `core/streamed_event.rs`

```rust
pub fn new(inner: T, stream_metadata: StreamMetadata) -> Self
```

Create a new streamed event by wrapping an existing event

---

### new

**Source**: `core/mock_stream.rs`

```rust
pub fn new(name: impl Into<String>) -> Self
```

Create a new mock stream

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
pub fn new(inner: S, _max_rate_per_second: u64, _dedup_ttl: Duration) -> Self
```

Create an enhanced stream with rate limiting and deduplication

---

### new

**Source**: `core/builder.rs`

```rust
pub fn new() -> Self
```

Create a new builder

---

### new

**Source**: `core/connection.rs`

```rust
pub fn new(config: ConnectionConfig) -> Self
```

Creates a new circuit breaker with the given configuration

---

### new

**Source**: `core/connection.rs`

```rust
pub fn new(config: ConnectionConfig) -> Self
```

Creates a new connection manager with the given configuration

---

### new

**Source**: `core/connection.rs`

```rust
pub fn new(config: ConnectionConfig, pool_size: usize) -> Self
```

Creates a new connection pool with the specified size and configuration

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, window: Duration) -> Self
```

Creates a new VWAP stream with the specified time window

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, window_size: usize) -> Self
```

Creates a new moving average stream with the specified window size

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, periods: usize) -> Self
```

Creates a new EMA stream with the specified number of periods

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, window: usize, std_dev_multiplier: f64) -> Self
```

Creates a new Bollinger Bands stream with specified window and standard deviation multiplier

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, period: usize) -> Self
```

Creates a new RSI stream with the specified period

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, depth_levels: usize) -> Self
```

Creates a new order book imbalance stream with specified depth levels

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, lookback_period: usize) -> Self
```

Creates a new momentum stream with specified lookback period

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S) -> Self
```

Creates a new liquidity pool tracker stream

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, window: Duration) -> Self
```

Creates a new MEV detection stream with specified time window

---

### new

**Source**: `core/financial_operators.rs`

```rust
pub fn new(inner: S, window_size: usize) -> Self
```

Creates a new gas price oracle stream with specified window size

---

### new

**Source**: `core/stream.rs`

```rust
pub fn new(stream: S) -> Self
```

Create a new dynamic stream wrapper around the given stream

---

### new

**Source**: `core/processor.rs`

```rust
pub fn new(window_type: WindowType) -> Self
```

Creates a new window manager with the specified window type

---

### new

**Source**: `core/processor.rs`

```rust
pub fn new(checkpoint_interval: Duration) -> Self
```

Creates a new stateful processor with the specified checkpoint interval

---

### new

**Source**: `core/processor.rs`

```rust
pub fn new(config: BackpressureConfig) -> Self
```

Creates a new flow controller with the specified backpressure configuration

---

### new

**Source**: `core/processor.rs`

```rust
pub fn new(config: BatchConfig) -> Self
```

Creates a new batch processor with the specified configuration

---

### new

**Source**: `core/processor.rs`

```rust
pub fn new(patterns: Vec<EventPattern<E>>, max_history: usize) -> Self
```

Creates a new pattern matcher with the specified patterns and history size

---

### new

**Source**: `core/manager.rs`

```rust
pub fn new() -> Self
```

Create a new StreamManager with default configuration

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

Create a new mapped stream

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, predicate: F) -> Self
```

Create a new filtered stream

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(stream1: S1, stream2: S2) -> Self
```

Create a new merged stream

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, batch_size: usize, timeout: Duration) -> Self
```

Create a new batched stream

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, duration: Duration) -> Self
```

Create a new debounced stream

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, duration: Duration) -> Self
```

Create a new throttled stream

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, count: usize) -> Self
```

Create a new take stream

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, count: usize) -> Self
```

Create a new skip stream

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, initial_state: St, transform: F) -> Self
```

Create a new scan stream

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, strategy: BufferStrategy) -> Self
```

Create a new buffered stream

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, filter: FilterF, map: MapF) -> Self
```

Create a new filter-map stream

---

### new

**Source**: `core/operators.rs`

```rust
pub fn new(inner: S, max_retries: usize, retry_delay: Duration) -> Self
```

Create a new resilient stream

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

Create a new type-erased event

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

**Source**: `production/resilience.rs`

```rust
pub fn new(name: impl Into<String>) -> Self
```

Create new circuit breaker

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

**Source**: `solana/geyser.rs`

```rust
pub fn new(name: impl Into<String>) -> Self
```

Create a new Solana Geyser stream

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
pub fn new(worker_id: String, stream_manager: Arc<StreamManager>) -> Self
```

Create a new streaming tool worker

---

### new

**Source**: `tools/condition.rs`

```rust
pub fn new(event_kinds: Vec<EventKind>) -> Self
```

Create a new EventKindMatcher with multiple event kinds

---

### new

**Source**: `tools/condition.rs`

```rust
pub fn new(sources: Vec<String>) -> Self
```

Create a new SourceMatcher with multiple sources

---

### new

**Source**: `tools/condition.rs`

```rust
pub fn new( min_timestamp: Option<std::time::SystemTime>, max_timestamp: Option<std::time::SystemTime>, ) -> Self
```

Create a new TimestampRangeMatcher with optional min and max timestamps

---

### new

**Source**: `tools/condition.rs`

```rust
pub fn new(predicate: F, description: impl Into<String>) -> Self
```

Create a new CustomCondition with a predicate function and description

---

### new

**Source**: `tools/condition.rs`

```rust
pub fn new(combinator: ConditionCombinator) -> Self
```

Create a new CompositeCondition with the specified combinator

---

### pending_count

**Source**: `core/processor.rs`

```rust
pub fn pending_count(&self) -> usize
```

Returns the number of events currently in the batch buffer

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

Returns health status for all connections in the pool

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
pub async fn register(self, manager: &StreamManager) -> Arc<EventTriggeredTool<T>> where T: 'static,
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

Releases a permit back to the semaphore pool

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

Removes and returns the state for the given key

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

Convert report interval seconds to Duration

---

### request_timeout

**Source**: `core/config.rs`

```rust
pub fn request_timeout(&self) -> Duration
```

Convert request timeout seconds to Duration

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

Convert base retry delay milliseconds to Duration

---

### retry_max_delay

**Source**: `core/config.rs`

```rust
pub fn retry_max_delay(&self) -> Duration
```

Convert maximum retry delay milliseconds to Duration

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

Create a new EventKindMatcher for a single event kind

---

### single

**Source**: `tools/condition.rs`

```rust
pub fn single(source: String) -> Self
```

Create a new SourceMatcher for a single source

---

### slow_threshold

**Source**: `core/config.rs`

```rust
pub fn slow_threshold(&self) -> Duration
```

Convert slow threshold milliseconds to Duration

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

**Source**: `core/connection.rs`

```rust
pub fn state(&self) -> ConnectionState
```

Returns the current state of the circuit breaker

---

### state

**Source**: `core/manager.rs`

```rust
pub async fn state(&self) -> ManagerState
```

Get manager state

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

Updates the queue size by the specified delta

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

Updates the state for the given key using the provided function

---

### validate

**Source**: `core/config.rs`

```rust
pub fn validate(&self) -> Result<(), ConfigError>
```

Validate batch configuration parameters

---

### validate

**Source**: `core/config.rs`

```rust
pub fn validate(&self) -> Result<(), ConfigError>
```

Validate backpressure configuration parameters

---

### validate

**Source**: `core/config.rs`

```rust
pub fn validate(&self) -> Result<(), ConfigError>
```

Validate connection configuration parameters

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

Convert metrics window seconds to Duration

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

Executes a function with the current connection

Updates activity tracking when the connection is accessed

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

## Enums

### AlertSeverity

**Source**: `production/health.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
```

```rust
pub enum AlertSeverity { /// Informational alerts for general status updates Info, /// Warning alerts for potential issues that need attention Warning, /// Error alerts for failures that occurred but system can continue Error, /// Critical alerts for severe issues requiring immediate attention Critical, }
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
pub enum BackoffStrategy { /// Fixed delay between all retry attempts Fixed, /// Linear backoff with configurable increment Linear { /// Duration to add per retry attempt increment: Duration, }, /// Exponential backoff with configurable factor Exponential { /// Multiplication factor for each retry attempt factor: f64, }, }
```

Backoff strategy for retries

**Variants**:

- `Fixed`
- `Linear`
- `increment`
- `Exponential`
- `factor`

---

### BackpressureStrategy

**Source**: `core/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
```

```rust
pub enum BackpressureStrategy { /// Block and wait (default) - guarantees delivery but may cause slowdowns Block, /// Drop messages when buffer is full - fastest but may lose data Drop, /// Retry with exponential backoff, then drop Retry { /// Maximum number of retry attempts before dropping max_attempts: usize, /// Base wait time in milliseconds between retries base_wait_ms: u64, }, /// Adaptive strategy that switches based on load Adaptive, }
```

Backpressure handling strategies

**Variants**:

- `Block`
- `Drop`
- `Retry`
- `max_attempts`
- `base_wait_ms`
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
pub enum BitcoinNetwork { /// Bitcoin mainnet Mainnet, /// Bitcoin testnet Testnet, /// Bitcoin signet Signet, }
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
pub enum ChainId { /// Ethereum mainnet Ethereum = 1, /// Polygon (Matic) network Polygon = 137, /// Binance Smart Chain BSC = 56, /// Arbitrum layer 2 Arbitrum = 42161, /// Optimism layer 2 Optimism = 10, /// Avalanche C-Chain Avalanche = 43114, /// Base layer 2 Base = 8453, }
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
pub enum ConfigError { /// Invalid configuration parameter or value #[error("Invalid configuration: {0}")] Invalid(String), /// Environment variable parsing or access error #[error("Environment variable error: {0}")] Environment(String), }
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
pub enum EventPattern<E> { /// Match a single event type Single(fn(&E) -> bool), /// Match a sequence of events Sequence(Vec<fn(&E) -> bool>), /// Match events within a time window Within { pattern: Box<EventPattern<E>>, duration: Duration, }, /// Match any of the patterns Any(Vec<EventPattern<E>>), /// Match all patterns All(Vec<EventPattern<E>>), }
```

Complex event processing pattern matcher

**Variants**:

- `Single(fn(&E)`
- `Sequence(Vec<fn(&E)`
- `Within`
- `pattern`
- `duration`
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
pub enum ManagerState { /// Manager is idle and not processing any streams Idle, /// Manager is in the process of starting up Starting, /// Manager is actively running and processing streams Running, /// Manager is in the process of stopping Stopping, /// Manager has stopped and is no longer processing streams Stopped, }
```

Represents the current state of the StreamManager

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
pub enum MempoolEventType { /// Block-related event Block, /// Transaction-related event Transaction, /// Statistics update event Stats, }
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
pub enum MergedEvent<E1, E2> { /// Event from the first stream First(E1), /// Event from the second stream Second(E2), }
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
pub enum MevType { /// Sandwich attack - placing transactions before and after a target Sandwich, /// Front-running - placing a transaction before a target transaction Frontrun, /// Back-running - placing a transaction after a target transaction Backrun, /// Arbitrage opportunity - price differences across markets Arbitrage, }
```

Types of MEV (Maximum Extractable Value) patterns

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
pub enum StreamConfig { /// Solana Geyser stream configuration for monitoring Solana blockchain events SolanaGeyser { /// Unique name identifier for the stream name: String, /// Geyser-specific configuration parameters config: GeyserConfig, }, /// EVM WebSocket stream configuration for monitoring EVM-compatible blockchain events EvmWebSocket { /// Unique name identifier for the stream name: String, /// EVM WebSocket-specific configuration parameters config: EvmStreamConfig, }, /// Binance stream configuration for monitoring Binance exchange data Binance { /// Unique name identifier for the stream name: String, /// Binance-specific configuration parameters config: BinanceConfig, }, /// Mempool.space stream configuration for monitoring Bitcoin mempool data MempoolSpace { /// Unique name identifier for the stream name: String, /// Mempool.space-specific configuration parameters config: MempoolConfig, }, }
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
pub enum StreamError { /// Connection-related errors #[error("Connection failed: {message}")] Connection { /// Error message describing the connection failure message: String, /// Whether this connection error can be retried retriable: bool }, /// Configuration errors #[error("Configuration error: {message}")] Configuration { /// Error message describing the configuration issue message: String }, /// Invalid configuration #[error("Invalid configuration: {reason}")] ConfigurationInvalid { /// Reason why the configuration is invalid reason: String }, /// Authentication failures #[error("Authentication failed: {message}")] Authentication { /// Error message describing the authentication failure message: String }, /// Rate limiting errors #[error("Rate limit exceeded: {message}")] RateLimit { /// Error message describing the rate limit message: String, /// Optional duration in seconds to wait before retrying retry_after: Option<u64>, }, /// Data parsing errors #[error("Parse error: {message}")] Parse { /// Error message describing the parsing failure message: String, /// The raw data that failed to parse data: String }, /// Resource exhaustion #[error("Resource exhausted: {message}")] ResourceExhausted { /// Error message describing the resource exhaustion message: String }, /// Stream already running #[error("Stream already running: {name}")] AlreadyRunning { /// Name of the stream that is already running name: String }, /// Stream not running #[error("Stream not running: {name}")] NotRunning { /// Name of the stream that is not running name: String }, /// Channel errors #[error("Channel error: {message}")] Channel { /// Error message describing the channel failure message: String }, /// Timeout errors #[error("Operation timed out: {message}")] Timeout { /// Error message describing the timeout message: String }, /// Processing errors #[error("Processing error: {message}")] Processing { /// Error message describing the processing failure message: String }, /// Backpressure errors #[error("Backpressure error: {message}")] Backpressure { /// Error message describing the backpressure issue message: String }, /// Internal errors #[error("Internal error: {source}")] Internal { /// The underlying error that caused this internal error #[source] source: Box<dyn std::error::Error + Send + Sync>, }, }
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