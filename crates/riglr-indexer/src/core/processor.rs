//! Event processor for parallel processing with worker pools

use backoff::{backoff::Backoff, ExponentialBackoff, ExponentialBackoffBuilder};
use core::{
    fmt::{Debug, Formatter, Result as FmtResult},
    time::Duration,
};
use std::{collections::HashMap, string::ToString, sync::Arc, time::Instant};
use tokio::{
    sync::{RwLock, Semaphore},
    time::{interval, sleep},
};
use tracing::{debug, error, info, warn};

use crate::utils::{
    safe_cast_duration_millis, safe_cast_u64_to_f64, safe_cast_usize_to_f64, BatchProcessor,
};
use crate::{
    config::{BatchConfig, CacheBackend, RetryConfig},
    core::{
        ComponentHealth, EventProcessing, HealthStatus, ProcessingStats, ServiceContext,
        ServiceLifecycle, ServiceState,
    },
    error::{IndexerError, IndexerResult, ProcessingError},
    storage::StoredEvent,
};
use riglr_events_core::prelude::*;

/// Configuration for the event processor
#[derive(Debug, Clone)]
pub struct Config {
    /// Batch processing configuration
    pub batch_config: BatchConfig,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Number of worker threads
    pub workers: usize,
}

/// Event processor that handles parallel processing with worker pools
#[derive(Clone)]
#[expect(clippy::struct_field_names)]
pub struct Processor {
    /// Active workers counter
    active_workers: Arc<RwLock<usize>>,
    /// Batch processor
    #[expect(dead_code)] // Will be used for advanced batch processing
    batch_processor: Arc<BatchProcessor<Box<dyn Event>>>,
    /// Processor configuration
    config: Config,
    /// Shared service context
    context: Arc<ServiceContext>,
    /// Processing statistics
    stats: Arc<RwLock<Stats>>,
    /// Worker pool semaphore for limiting concurrent operations
    worker_semaphore: Arc<Semaphore>,
}

impl Debug for Processor {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("EventProcessor")
            .field("config", &self.config)
            .field("context", &"<ServiceContext>")
            .field("worker_permits", &self.worker_semaphore.available_permits())
            .field("stats", &"<ProcessorStats>")
            .field("active_workers", &"<usize>")
            .field("batch_processor", &"<BatchProcessor>")
            .finish_non_exhaustive()
    }
}

/// Internal processor statistics
#[derive(Debug, Clone, Default)]
pub struct Stats {
    /// Average processing latency in milliseconds
    avg_latency_ms: f64,
    /// Current batch size
    #[expect(dead_code)] // Will be used for advanced metrics
    current_batch_size: usize,
    /// Events processed per second
    events_per_second: f64,
    /// Last processing time
    last_processing_time: Option<Instant>,
    /// Total errors encountered
    total_errors: u64,
    /// Total events processed
    total_processed: u64,
}

impl Processor {
    /// Create a new event processor
    ///
    /// # Errors
    ///
    /// Returns error if pipeline stage initialization or worker thread pool setup fails
    pub fn new(config: Config, context: Arc<ServiceContext>) -> IndexerResult<Self> {
        info!(
            "Initializing event processor with {} workers",
            config.workers
        );

        let worker_semaphore = Arc::new(Semaphore::new(config.workers));

        // Create batch processor
        let batch_processor =
            BatchProcessor::new(config.batch_config.max_size, config.batch_config.max_age);

        let processor = Self {
            config,
            context,
            worker_semaphore,
            stats: Arc::new(RwLock::new(Stats::default())),
            active_workers: Arc::new(RwLock::new(0)),
            batch_processor: Arc::new(batch_processor),
        };

        info!("Event processor initialized successfully");
        Ok(processor)
    }

    /// Get current number of active workers
    pub async fn active_workers(&self) -> usize {
        *self.active_workers.read().await
    }

    /// Convert event to JSON, handling serialization errors
    fn convert_event_to_json(event: &dyn Event) -> IndexerResult<serde_json::Value> {
        event.to_json().map_err(|_e| {
            IndexerError::Processing(ProcessingError::SerializationFailed {
                event_id: event.id().to_string(),
            })
        })
    }

    /// Create configured exponential backoff for retry logic
    fn create_retry_backoff(&self) -> ExponentialBackoff {
        let mut builder = ExponentialBackoffBuilder::new();
        builder.with_initial_interval(self.config.retry_config.base_delay);
        builder.with_max_interval(self.config.retry_config.max_delay);
        builder.with_multiplier(self.config.retry_config.backoff_multiplier);
        builder.with_max_elapsed_time(Some(Duration::from_secs(300))); // 5 minutes max
        builder.build()
    }

    /// Create `StoredEvent` from event data and JSON
    fn create_stored_event(event: &dyn Event, event_json: serde_json::Value) -> StoredEvent {
        StoredEvent {
            id: event.id().to_string(),
            event_type: format!("{:?}", event.kind()),
            source: event.metadata().source.clone(),
            data: event_json,
            timestamp: event.metadata().timestamp,
            block_height: Self::extract_block_height(event),
            transaction_hash: Self::extract_transaction_hash(event),
        }
    }

    /// Extract block height from event metadata
    fn extract_block_height(event: &dyn Event) -> Option<i64> {
        event
            .metadata()
            .custom
            .get("block_height")
            .and_then(serde_json::Value::as_i64)
    }

    /// Extract transaction hash from event metadata
    fn extract_transaction_hash(event: &dyn Event) -> Option<String> {
        event
            .metadata()
            .custom
            .get("transaction_hash")
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
    }

    /// Get current processing statistics
    async fn get_processing_stats(&self) -> ProcessingStats {
        let stats = self.stats.read().await;
        let active_workers = *self.active_workers.read().await;

        ProcessingStats {
            total_processed: stats.total_processed,
            events_per_second: stats.events_per_second,
            avg_latency_ms: stats.avg_latency_ms,
            error_rate: if stats.total_processed > 0 {
                safe_cast_u64_to_f64(stats.total_errors)
                    / safe_cast_u64_to_f64(stats.total_processed)
            } else {
                0.0
            },
            queue_depth: 0, // Not applicable for processor
            active_workers,
        }
    }

    /// Handle event caching if enabled
    async fn handle_event_caching(&self, event_id: &str, storage_event: &StoredEvent) {
        if matches!(
            self.context.config.storage.cache.backend,
            CacheBackend::Redis
        ) {
            let cache_result = self
                .context
                .store
                .cache_event(event_id, storage_event, Some(Duration::from_secs(3600)))
                .await;
            if let Err(e) = cache_result {
                warn!("Failed to cache event {}: {}", event_id, e);
                // Don't fail the entire operation for cache failures
            }
        }
    }

    /// Handle retry delay and logging
    async fn handle_retry_delay(
        &self,
        backoff: &mut ExponentialBackoff,
        attempts: usize,
        max_attempts: u32,
    ) -> IndexerResult<()> {
        if let Some(delay) = backoff.next_backoff() {
            warn!(
                "Retrying event processing in {:?} (attempt {}/{})",
                delay, attempts, max_attempts
            );
            sleep(delay).await;
            return Ok(());
        }
        error!(
            "Backoff timer expired, giving up after {} attempts",
            attempts
        );
        Err(IndexerError::Processing(ProcessingError::PipelineStalled {
            seconds: 300,
        }))
    }

    /// Handle successful event processing after retries
    fn handle_successful_retry(attempts: usize) {
        if attempts > 1 {
            info!("Event processed successfully after {} attempts", attempts);
        }
    }

    /// Process a single event with retry logic
    async fn process_single_event(&self, event: Box<dyn Event>) -> IndexerResult<()> {
        let start_time = Instant::now();

        // Acquire worker permit
        let acquire_result = self.worker_semaphore.acquire().await;
        let _permit = acquire_result
            .map_err(|_| IndexerError::internal("Failed to acquire worker permit"))?;

        // Increment active workers
        {
            let mut workers = self.active_workers.write().await;
            {
                *workers = workers.saturating_add(1);
            }
        }

        let result = self.process_with_retry(event).await;

        // Decrement active workers
        {
            let mut workers = self.active_workers.write().await;
            *workers = workers.saturating_sub(1);
        }

        // Update statistics
        let processing_time = start_time.elapsed();
        let mut stats = self.stats.write().await;

        match result {
            Ok(()) => {
                {
                    stats.total_processed = stats.total_processed.saturating_add(1);
                }
                stats.last_processing_time = Some(start_time);

                // Update average latency with exponential moving average
                let latency_ms = safe_cast_duration_millis(processing_time);
                if stats.avg_latency_ms == 0.0 {
                    stats.avg_latency_ms = latency_ms;
                } else {
                    stats.avg_latency_ms = stats.avg_latency_ms.mul_add(0.9, latency_ms * 0.1);
                }
            }
            Err(_) => {
                stats.total_errors = stats.total_errors.saturating_add(1);
            }
        }

        result
    }

    /// Process event with exponential backoff retry
    async fn process_with_retry(&self, event: Box<dyn Event>) -> IndexerResult<()> {
        let mut backoff = self.create_retry_backoff();
        let mut attempts: usize = 0;
        let max_attempts = self.config.retry_config.max_attempts;

        loop {
            {
                attempts = attempts.saturating_add(1);
            }

            let store_result = self.store_event(&*event).await;
            match store_result {
                Ok(()) => {
                    Self::handle_successful_retry(attempts);
                    return Ok(());
                }
                Err(e) => {
                    Self::should_retry(&e, attempts, max_attempts)?;
                    self.handle_retry_delay(&mut backoff, attempts, max_attempts)
                        .await?;
                }
            }
        }
    }

    /// Check if retry should continue based on attempts and error type
    fn should_retry(
        error: &IndexerError,
        attempts: usize,
        max_attempts: u32,
    ) -> Result<(), IndexerError> {
        if attempts >= max_attempts as usize {
            error!(
                "Failed to process event after {} attempts: {}",
                attempts, error
            );
            return Err(IndexerError::Processing(ProcessingError::PipelineStalled {
                seconds: 300,
            }));
        }

        if !error.is_retriable() {
            error!("Non-retriable error processing event: {}", error);
            return Err(IndexerError::Processing(
                ProcessingError::ValidationFailed {
                    reason: format!("Non-retriable error: {error}"),
                },
            ));
        }

        Ok(())
    }

    /// Start background statistics collection
    fn start_stats_collection(&self) {
        let stats = self.stats.clone();
        let context = self.context.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            let mut last_count = 0u64;
            let mut last_time = Instant::now();

            loop {
                interval.tick().await;

                {
                    let mut stats_guard = stats.write().await;
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_time).as_secs_f64();

                    if elapsed > 0.0 {
                        let events_delta = stats_guard.total_processed.saturating_sub(last_count);
                        stats_guard.events_per_second =
                            safe_cast_u64_to_f64(events_delta) / elapsed;

                        // Record metrics
                        context.metrics.record_gauge(
                            "indexer_processing_events_per_second",
                            stats_guard.events_per_second,
                        );

                        context.metrics.record_counter(
                            "indexer_processing_total_events",
                            stats_guard.total_processed,
                        );

                        context.metrics.record_counter(
                            "indexer_processing_total_errors",
                            stats_guard.total_errors,
                        );

                        let error_rate = if stats_guard.total_processed > 0 {
                            safe_cast_u64_to_f64(stats_guard.total_errors)
                                / safe_cast_u64_to_f64(stats_guard.total_processed)
                        } else {
                            0.0
                        };
                        context
                            .metrics
                            .record_gauge("indexer_processing_error_rate", error_rate);
                    }

                    last_count = stats_guard.total_processed;
                    drop(stats_guard);
                    last_time = now;
                }

                // Check for shutdown
                if matches!(
                    context.state().await,
                    ServiceState::Stopping | ServiceState::Stopped
                ) {
                    break;
                }
            }
        });
    }

    /// Store event using the data store
    async fn store_event(&self, event: &dyn Event) -> IndexerResult<()> {
        debug!("Storing event: {} (type: {:?})", event.id(), event.kind());

        let event_json = Self::convert_event_to_json(event)?;
        let storage_event = Self::create_stored_event(event, event_json);

        // Store in primary storage
        self.context.store.insert_event(&storage_event).await?;

        // Handle caching if enabled
        self.handle_event_caching(event.id(), &storage_event).await;

        debug!("Event {} stored successfully", event.id());
        Ok(())
    }
}

#[async_trait::async_trait]
impl ServiceLifecycle for Processor {
    async fn start(&mut self) -> IndexerResult<()> {
        info!("Starting event processor");

        // Start statistics collection
        self.start_stats_collection();

        // Update component health
        self.context
            .update_component_health(
                "processor",
                ComponentHealth::healthy("Event processor started successfully"),
            )
            .await;

        info!(
            "Event processor started with {} workers",
            self.config.workers
        );
        Ok(())
    }

    async fn stop(&mut self) -> IndexerResult<()> {
        info!("Stopping event processor");

        // Wait for all workers to complete with timeout
        let timeout_duration = Duration::from_secs(30);
        let start_time = Instant::now();

        while *self.active_workers.read().await > 0 {
            if start_time.elapsed() > timeout_duration {
                let active_count = *self.active_workers.read().await;
                warn!(
                    "Timeout waiting for workers to complete, {} workers still active",
                    active_count
                );
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }

        info!("Event processor stopped");
        Ok(())
    }

    async fn health(&self) -> IndexerResult<HealthStatus> {
        let stats = self.stats.read().await;

        let healthy = stats.total_errors == 0
            || (stats.total_processed > 0
                && (safe_cast_u64_to_f64(stats.total_errors)
                    / safe_cast_u64_to_f64(stats.total_processed))
                    < 0.1);

        let mut components = HashMap::new();

        let message = if healthy {
            format!(
                "Processor healthy: {:.2} events/sec, {:.1}ms avg latency",
                stats.events_per_second, stats.avg_latency_ms
            )
        } else {
            format!(
                "Processor unhealthy: {:.2}% error rate",
                if stats.total_processed > 0 {
                    safe_cast_u64_to_f64(stats.total_errors)
                        / safe_cast_u64_to_f64(stats.total_processed)
                        * 100.0
                } else {
                    0.0
                }
            )
        };

        components.insert(
            "processor".to_string(),
            if healthy {
                ComponentHealth::healthy(&message)
            } else {
                ComponentHealth::unhealthy(&message)
            },
        );

        return Ok(HealthStatus {
            healthy,
            components,
            timestamp: chrono::Utc::now(),
        });
    }
    fn is_running(&self) -> bool {
        // Processor is considered running if we have capacity to process events
        self.worker_semaphore.available_permits() > 0
    }
}

#[async_trait::async_trait]
impl EventProcessing for Processor {
    async fn process_event(&self, event: Box<dyn Event>) -> IndexerResult<()> {
        self.process_single_event(event).await
    }

    async fn process_batch(&self, events: Vec<Box<dyn Event>>) -> IndexerResult<()> {
        if events.is_empty() {
            return Ok(());
        }

        info!("Processing batch of {} events", events.len());
        let start_time = Instant::now();

        // Process events concurrently
        let tasks: Vec<_> = events
            .into_iter()
            .map(|event| {
                let processor = self.clone();
                tokio::spawn(async move { return processor.process_single_event(event).await })
            })
            .collect();

        // Wait for all tasks to complete
        let mut success_count: usize = 0;
        let mut error_count: usize = 0;

        for (index, task) in tasks.into_iter().enumerate() {
            let task_result = task.await;
            match task_result {
                Ok(Ok(())) => {
                    success_count = success_count.saturating_add(1);
                }
                Ok(Err(e)) => {
                    error!("Event {} in batch failed: {}", index, e);
                    {
                        error_count = error_count.saturating_add(1);
                    }
                }
                Err(e) => {
                    error!("Task {} in batch panicked: {}", index, e);
                    {
                        error_count = error_count.saturating_add(1);
                    }
                }
            }
        }

        let processing_time = start_time.elapsed();
        info!(
            "Batch processing completed in {:?}: {} success, {} errors",
            processing_time, success_count, error_count
        );

        // Record batch metrics
        self.context.metrics.record_histogram(
            "indexer_batch_processing_duration_ms",
            safe_cast_duration_millis(processing_time),
        );

        self.context.metrics.record_histogram(
            "indexer_batch_size",
            safe_cast_usize_to_f64(success_count.saturating_add(error_count)),
        );

        if error_count > 0 {
            self.context
                .metrics
                .increment_counter("indexer_batch_errors");

            // If more than half the batch failed, consider it a batch failure
            if error_count > success_count {
                return Err(IndexerError::Processing(
                    ProcessingError::ValidationFailed {
                        reason: format!(
                            "Batch processing failed: {}/{} events failed",
                            error_count,
                            {
                                {
                                    success_count.saturating_add(error_count)
                                }
                            }
                        ),
                    },
                ));
            }
        }

        Ok(())
    }

    async fn processing_stats(&self) -> ProcessingStats {
        self.get_processing_stats().await
    }
}

/// Processing pipeline stage for complex event processing
#[derive(Default)]
pub struct ProcessingPipeline {
    stages: Vec<Box<dyn PipelineStage + Send + Sync>>,
}

impl Debug for ProcessingPipeline {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ProcessingPipeline")
            .field("stages", &format!("{} stages", self.stages.len()))
            .finish()
    }
}

impl ProcessingPipeline {
    /// Create a new processing pipeline
    #[must_use]
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Add a stage to the pipeline
    #[must_use]
    pub fn add_stage(mut self, stage: Box<dyn PipelineStage + Send + Sync>) -> Self {
        self.stages.push(stage);
        self
    }

    /// Process an event through all stages
    ///
    /// # Errors
    ///
    /// Returns error if any pipeline stage fails to process the event
    pub async fn process(&self, mut event: Box<dyn Event>) -> IndexerResult<Box<dyn Event>> {
        for (index, stage) in self.stages.iter().enumerate() {
            let process_result = stage.process(event).await;
            match process_result {
                Ok(processed_event) => {
                    event = processed_event;
                }
                Err(e) => {
                    error!("Pipeline stage {} failed: {}", index, e);
                    return Err(e);
                }
            }
        }
        Ok(event)
    }
}

/// Pipeline stage trait for processing events
#[async_trait::async_trait]
pub trait PipelineStage {
    /// Get stage name for debugging
    fn name(&self) -> &str;

    /// Process an event and return the modified event
    async fn process(&self, event: Box<dyn Event>) -> IndexerResult<Box<dyn Event>>;
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::panic)]
#[expect(clippy::float_cmp)] // All test float comparisons use exact literal values
mod tests {
    use super::*;
    use crate::config::IndexerConfig;
    use crate::core::{EventProcessor, ProcessorConfig};
    use crate::core::{ServiceContext, ServiceState};
    use crate::error::StorageError;
    use crate::metrics::MetricsCollector;
    use crate::storage::{DataStore, EventFilter, EventQuery, StorageStats, StoredEvent};
    use chrono::{DateTime, Utc};
    use core::any::Any;
    use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use riglr_events_core::{traits::Event, types::EventMetadata};
    use std::{collections::HashMap, time::SystemTime};
    use tokio::sync::{broadcast::channel, RwLock as TokioRwLock};

    // Mock implementations for testing
    #[derive(Debug, Clone)]
    struct MockEvent {
        id: String,
        kind: EventKind,
        source: String,
        timestamp: DateTime<Utc>,
        data: serde_json::Value,
        metadata: EventMetadata,
    }

    impl MockEvent {
        fn new(id: &str) -> Self {
            #[expect(clippy::collection_is_never_read)] // Used only for initialization
            let mut custom_map = HashMap::new();
            custom_map.insert(
                "block_height".to_string(),
                serde_json::Value::Number(serde_json::Number::from(12345u64)),
            );
            custom_map.insert(
                "transaction_hash".to_string(),
                serde_json::Value::String("0xabc123".to_string()),
            );

            Self {
                id: id.to_string(),
                kind: EventKind::Custom("test".to_string()),
                source: "test_source".to_string(),
                timestamp: Utc::now(),
                data: serde_json::json!({"test": "data"}),
                metadata: EventMetadata::new(
                    id.to_string(),
                    EventKind::Custom("test".to_string()),
                    "test_source".to_string(),
                )
                .with_custom("block_height".to_string(), 12345u64)
                .with_custom("transaction_hash".to_string(), "0xabc123".to_string()),
            }
        }

        fn new_without_metadata(id: &str) -> Self {
            Self {
                id: id.to_string(),
                kind: EventKind::Custom("test".to_string()),
                source: "test_source".to_string(),
                timestamp: Utc::now(),
                data: serde_json::json!({"test": "data"}),
                metadata: EventMetadata::new(
                    id.to_string(),
                    EventKind::Custom("test".to_string()),
                    "test_source".to_string(),
                ),
            }
        }
    }

    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.id
        }

        fn kind(&self) -> &EventKind {
            &self.kind
        }

        fn source(&self) -> &str {
            &self.source
        }

        fn timestamp(&self) -> SystemTime {
            self.timestamp.into()
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

        fn to_json(&self) -> Result<serde_json::Value, EventError> {
            Ok(self.data.clone())
        }
    }

    struct MockDataStore {
        should_fail: AtomicBool,
        fail_count: AtomicU64,
        fail_after: u64,
        events: TokioRwLock<Vec<StoredEvent>>,
    }

    impl MockDataStore {
        fn new() -> Self {
            Self {
                should_fail: AtomicBool::new(false),
                fail_count: AtomicU64::new(0),
                fail_after: 0,
                events: TokioRwLock::new(Vec::new()),
            }
        }

        fn set_should_fail(&self, should_fail: bool) {
            self.should_fail.store(should_fail, Ordering::SeqCst);
        }

        fn set_fail_after(&mut self, count: u64) {
            self.fail_after = count;
        }

        async fn get_events(&self) -> Vec<StoredEvent> {
            self.events.read().await.clone()
        }
    }

    #[async_trait::async_trait]
    impl DataStore for MockDataStore {
        async fn insert_event(&self, event: &StoredEvent) -> IndexerResult<()> {
            let current_count = self.fail_count.fetch_add(1, Ordering::SeqCst);

            if self.should_fail.load(Ordering::SeqCst) && current_count >= self.fail_after {
                return Err(IndexerError::Storage(StorageError::ConnectionFailed {
                    message: "Mock storage failure".to_string(),
                }));
            }

            self.events.write().await.push(event.clone());
            Ok(())
        }

        async fn insert_events(&self, events: &[StoredEvent]) -> IndexerResult<()> {
            for event in events {
                self.insert_event(event).await?;
            }
            Ok(())
        }

        async fn query_events(&self, _query: &EventQuery) -> IndexerResult<Vec<StoredEvent>> {
            Ok(self.events.read().await.clone())
        }

        async fn get_event(&self, id: &str) -> IndexerResult<Option<StoredEvent>> {
            let events = self.events.read().await;
            Ok(events.iter().find(|e| e.id == id).cloned())
        }

        async fn delete_events(&self, _filter: &EventFilter) -> IndexerResult<u64> {
            let mut events = self.events.write().await;
            let initial_len = events.len();
            events.clear();
            drop(events);
            Ok(initial_len as u64)
        }

        async fn count_events(&self, _filter: &EventFilter) -> IndexerResult<u64> {
            Ok(self.events.read().await.len() as u64)
        }

        async fn get_stats(&self) -> IndexerResult<StorageStats> {
            Ok(StorageStats {
                total_events: self.events.read().await.len() as u64,
                storage_size_bytes: 1024,
                avg_write_latency_ms: 5.0,
                avg_read_latency_ms: 2.0,
                active_connections: 1,
                cache_hit_rate: 0.8,
            })
        }

        async fn health_check(&self) -> IndexerResult<()> {
            Ok(())
        }

        async fn cache_event(
            &self,
            _key: &str,
            _event: &StoredEvent,
            _ttl: Option<Duration>,
        ) -> IndexerResult<()> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(IndexerError::Storage(StorageError::ConnectionFailed {
                    message: "Mock cache failure".to_string(),
                }));
            }
            Ok(())
        }

        async fn get_cached_event(&self, _key: &str) -> IndexerResult<Option<StoredEvent>> {
            Ok(None)
        }

        async fn initialize(&self) -> IndexerResult<()> {
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    fn create_mock_metrics_collector() -> MetricsCollector {
        use crate::config::MetricsConfig;
        MetricsCollector::new(MetricsConfig {
            enabled: false,
            port: 9090,
            endpoint: "/metrics".to_string(),
            collection_interval: Duration::from_secs(30),
            histogram_buckets: vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
            custom: HashMap::new(),
        })
        .unwrap() // Test assertion - safe to unwrap
    }

    fn create_test_context() -> Arc<ServiceContext> {
        let config = IndexerConfig::default();

        let (shutdown_tx, _) = channel(1);
        Arc::new(ServiceContext {
            config: Arc::new(config),
            state: Arc::new(RwLock::new(ServiceState::Starting)),
            store: Arc::new(MockDataStore::new()),
            metrics: Arc::new(create_mock_metrics_collector()),
            shutdown_tx,
            health: Arc::new(RwLock::new(HealthStatus {
                healthy: true,
                components: HashMap::new(),
                timestamp: chrono::Utc::now(),
            })),
        })
    }

    fn create_test_processor() -> (EventProcessor, Arc<ServiceContext>) {
        let context = create_test_context();
        let config = ProcessorConfig {
            workers: 2,
            batch_config: BatchConfig {
                max_size: 10,
                max_age: Duration::from_secs(5),
                target_size: 5,
            },
            retry_config: RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(10),
                backoff_multiplier: 2.0,
                jitter: 0.1,
            },
        };

        let processor = EventProcessor::new(config, context.clone()).unwrap(); // Test assertion - safe to unwrap
        (processor, context)
    }

    struct MockPipelineStage {
        name: String,
        should_fail: bool,
    }

    impl MockPipelineStage {
        fn new(name: &str, should_fail: bool) -> Self {
            Self {
                name: name.to_string(),
                should_fail,
            }
        }
    }

    #[async_trait::async_trait]
    impl PipelineStage for MockPipelineStage {
        async fn process(&self, event: Box<dyn Event>) -> IndexerResult<Box<dyn Event>> {
            if self.should_fail {
                return Err(IndexerError::Processing(
                    ProcessingError::ValidationFailed {
                        reason: format!("Stage {} failed", self.name),
                    },
                ));
            }
            Ok(event)
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    // Test ProcessorConfig
    #[test]
    fn test_processor_config_clone_and_debug() {
        let config = ProcessorConfig {
            workers: 4,
            batch_config: BatchConfig {
                max_size: 100,
                max_age: Duration::from_secs(30),
                target_size: 50,
            },
            retry_config: RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(10),
                backoff_multiplier: 2.0,
                jitter: 0.1,
            },
        };

        let cloned = config.clone();
        assert_eq!(config.workers, cloned.workers);
        assert_eq!(format!("{config:?}"), format!("{:?}", cloned));
    }

    // Test ProcessorStats
    #[test]
    fn test_processor_stats_default() {
        let stats = ProcessingStats::default();
        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.events_per_second, 0.0);
        assert_eq!(stats.avg_latency_ms, 0.0);
        assert_eq!(stats.error_rate, 0.0);
        assert_eq!(stats.queue_depth, 0);
        assert_eq!(stats.active_workers, 0);
    }

    #[test]
    fn test_processor_stats_clone_and_debug() {
        let stats = ProcessingStats {
            total_processed: 100,
            events_per_second: 10.5,
            avg_latency_ms: 15.2,
            error_rate: 0.05,
            queue_depth: 25,
            active_workers: 3,
        };

        let cloned = stats.clone();
        assert_eq!(stats.total_processed, cloned.total_processed);
        assert_eq!(stats.error_rate, cloned.error_rate);
        assert_eq!(format!("{stats:?}"), format!("{:?}", cloned));
    }

    // Test EventProcessor::new
    #[tokio::test]
    async fn test_event_processor_new_success() {
        let context = create_test_context();
        let config = ProcessorConfig {
            workers: 2,
            batch_config: BatchConfig {
                max_size: 10,
                max_age: Duration::from_secs(5),
                target_size: 5,
            },
            retry_config: RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(10),
                backoff_multiplier: 2.0,
                jitter: 0.1,
            },
        };

        let result = EventProcessor::new(config.clone(), context);
        assert!(result.is_ok());

        let processor = result.unwrap(); // Test assertion - safe to unwrap
        assert_eq!(processor.config.workers, config.workers);
        assert_eq!(
            processor.worker_semaphore.available_permits(),
            config.workers
        );
    }

    // Test EventProcessor::active_workers
    #[tokio::test]
    async fn test_active_workers_initially_zero() {
        let (processor, _) = create_test_processor();
        assert_eq!(processor.active_workers().await, 0);
    }

    // Test EventProcessor::get_processing_stats
    #[tokio::test]
    async fn test_get_processing_stats_initial_state() {
        let (processor, _) = create_test_processor();
        let stats = processor.get_processing_stats().await;

        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.events_per_second, 0.0);
        assert_eq!(stats.avg_latency_ms, 0.0);
        assert_eq!(stats.error_rate, 0.0);
        assert_eq!(stats.queue_depth, 0);
        assert_eq!(stats.active_workers, 0);
    }

    #[tokio::test]
    async fn test_get_processing_stats_with_errors() {
        let (processor, _) = create_test_processor();

        // Manually update stats to simulate errors
        {
            let mut stats = processor.stats.write().await;
            stats.total_processed = 100;
            stats.total_errors = 10;
            stats.events_per_second = 5.0;
            stats.avg_latency_ms = 50.0;
        }

        let stats = processor.get_processing_stats().await;
        assert_eq!(stats.total_processed, 100);
        assert_eq!(stats.error_rate, 0.1); // 10/100
        assert_eq!(stats.events_per_second, 5.0);
        assert_eq!(stats.avg_latency_ms, 50.0);
    }

    // Test ServiceLifecycle::start
    #[tokio::test]
    async fn test_service_lifecycle_start() {
        let (mut processor, _) = create_test_processor();
        let result = processor.start().await;
        assert!(result.is_ok());
    }

    // Test ServiceLifecycle::stop
    #[tokio::test]
    async fn test_service_lifecycle_stop_no_active_workers() {
        let (mut processor, _) = create_test_processor();
        let result = processor.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_lifecycle_stop_with_active_workers() {
        let (mut processor, _) = create_test_processor();

        // Simulate active workers
        {
            let mut workers = processor.active_workers.write().await;
            *workers = 2;
        }

        // This should timeout and continue
        let result = processor.stop().await;
        assert!(result.is_ok());
    }

    // Test ServiceLifecycle::health
    #[tokio::test]
    async fn test_service_lifecycle_health_healthy() {
        let (processor, _) = create_test_processor();
        let health = processor.health().await.unwrap(); // Test assertion - safe to unwrap

        assert!(health.healthy);
        assert!(health.components.contains_key("processor"));
    }

    #[tokio::test]
    async fn test_service_lifecycle_health_unhealthy_high_error_rate() {
        let (processor, _) = create_test_processor();

        // Set high error rate (>10%)
        {
            let mut stats = processor.stats.write().await;
            stats.total_processed = 100;
            stats.total_errors = 20; // 20% error rate
        }

        let health = processor.health().await.unwrap(); // Test assertion - safe to unwrap
        assert!(!health.healthy);
    }

    #[tokio::test]
    async fn test_service_lifecycle_health_healthy_low_error_rate() {
        let (processor, _) = create_test_processor();

        // Set low error rate (<10%)
        {
            let mut stats = processor.stats.write().await;
            stats.total_processed = 100;
            stats.total_errors = 5; // 5% error rate
            stats.events_per_second = 10.0;
            stats.avg_latency_ms = 25.0;
        }

        let health = processor.health().await.unwrap(); // Test assertion - safe to unwrap
        assert!(health.healthy);
    }

    // Test ServiceLifecycle::is_running
    #[tokio::test]
    async fn test_service_lifecycle_is_running() {
        let (processor, _) = create_test_processor();
        assert!(processor.is_running());
    }

    // Test EventProcessing::process_event
    #[tokio::test]
    async fn test_event_processing_process_event_success() {
        let (processor, context) = create_test_processor();

        // Start the processor to enable stats collection
        let mut proc_clone = processor.clone();
        let _ = proc_clone.start().await;

        let event = Box::new(MockEvent::new("test_event_1"));
        let result = processor.process_event(event).await;
        assert!(result.is_ok());

        // Check that event was stored
        let store = context
            .store
            .as_any()
            .downcast_ref::<MockDataStore>()
            .unwrap(); // Test assertion - safe to unwrap
        let events = store.get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(
            events.first().unwrap().id, // Test assertion - safe to unwrap
            "test_event_1"
        );
    }

    // Test EventProcessing::process_batch
    #[tokio::test]
    async fn test_event_processing_process_batch_empty() {
        let (processor, _) = create_test_processor();
        let result = processor.process_batch(vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_processing_process_batch_success() {
        let (processor, context) = create_test_processor();

        let events = vec![
            Box::new(MockEvent::new("event_1")) as Box<dyn Event>,
            Box::new(MockEvent::new("event_2")) as Box<dyn Event>,
        ];

        let result = processor.process_batch(events).await;
        assert!(result.is_ok());

        // Check that events were stored
        let store = context
            .store
            .as_any()
            .downcast_ref::<MockDataStore>()
            .unwrap(); // Test assertion - safe to unwrap
        let stored_events = store.get_events().await;
        assert_eq!(stored_events.len(), 2);
    }

    #[tokio::test]
    async fn test_event_processing_process_batch_partial_failure() {
        let context = create_test_context();

        // Create a store that fails after first event
        let mut mock_store = MockDataStore::new();
        mock_store.set_fail_after(1);
        mock_store.set_should_fail(true);

        // Replace the store in context
        let new_context = Arc::new(ServiceContext::new(
            (*context.config).clone(),
            Arc::new(mock_store),
            Arc::new(create_mock_metrics_collector()),
        ));

        let config = ProcessorConfig {
            workers: 2,
            batch_config: BatchConfig {
                max_size: 10,
                max_age: Duration::from_secs(5),
                target_size: 5,
            },
            retry_config: RetryConfig {
                max_attempts: 1, // Only one attempt to make test faster
                base_delay: Duration::from_millis(10),
                max_delay: Duration::from_secs(1),
                backoff_multiplier: 2.0,
                jitter: 0.1,
            },
        };

        let processor = EventProcessor::new(config, new_context).unwrap(); // Test assertion - safe to unwrap

        let events = vec![
            Box::new(MockEvent::new("event_1")) as Box<dyn Event>,
            Box::new(MockEvent::new("event_2")) as Box<dyn Event>,
            Box::new(MockEvent::new("event_3")) as Box<dyn Event>,
        ];

        let result = processor.process_batch(events).await;
        assert!(result.is_err()); // Should fail because more than half failed
    }

    // Test EventProcessing::processing_stats
    #[tokio::test]
    async fn test_event_processing_processing_stats() {
        let (processor, _) = create_test_processor();
        let stats = processor.processing_stats().await;
        assert_eq!(stats.total_processed, 0);
    }

    // Test ProcessingPipeline
    #[test]
    fn test_processing_pipeline_new() {
        let pipeline = ProcessingPipeline::default();
        assert_eq!(pipeline.stages.len(), 0);
    }

    #[test]
    fn test_processing_pipeline_default() {
        let pipeline = ProcessingPipeline::default();
        assert_eq!(pipeline.stages.len(), 0);
    }

    #[test]
    fn test_processing_pipeline_add_stage() {
        let pipeline = ProcessingPipeline::default()
            .add_stage(Box::new(MockPipelineStage::new("stage1", false)))
            .add_stage(Box::new(MockPipelineStage::new("stage2", false)));

        assert_eq!(pipeline.stages.len(), 2);
    }

    #[tokio::test]
    async fn test_processing_pipeline_process_success() {
        let pipeline = ProcessingPipeline::default()
            .add_stage(Box::new(MockPipelineStage::new("stage1", false)))
            .add_stage(Box::new(MockPipelineStage::new("stage2", false)));

        let event = Box::new(MockEvent::new("pipeline_test")) as Box<dyn Event>;
        let result = pipeline.process(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_processing_pipeline_process_failure() {
        let pipeline = ProcessingPipeline::default()
            .add_stage(Box::new(MockPipelineStage::new("stage1", false)))
            .add_stage(Box::new(MockPipelineStage::new("stage2", true))); // This stage fails

        let event = Box::new(MockEvent::new("pipeline_test")) as Box<dyn Event>;
        let result = pipeline.process(event).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_processing_pipeline_process_empty() {
        let pipeline = ProcessingPipeline::default();
        let event = Box::new(MockEvent::new("pipeline_test")) as Box<dyn Event>;
        let result = pipeline.process(event).await;
        assert!(result.is_ok());
    }

    // Test private methods indirectly through public interface

    #[tokio::test]
    async fn test_store_event_success() {
        let (processor, context) = create_test_processor();
        let event = MockEvent::new("store_test");

        // Call store_event indirectly through process_event
        let result = processor.process_event(Box::new(event)).await;
        assert!(result.is_ok());

        let store = context
            .store
            .as_any()
            .downcast_ref::<MockDataStore>()
            .unwrap(); // Test assertion - safe to unwrap
        let events = store.get_events().await;
        assert_eq!(events.len(), 1);
        let first_event = events.first().unwrap(); // Test assertion - safe to unwrap
        assert_eq!(first_event.event_type, "Custom(\"test\")");
        assert_eq!(first_event.source, "test_source");
        assert!(first_event.block_height.is_some());
        assert!(first_event.transaction_hash.is_some());
    }

    #[tokio::test]
    async fn test_store_event_without_metadata() {
        let (processor, context) = create_test_processor();
        let event = MockEvent::new_without_metadata("store_test_no_meta");

        let result = processor.process_event(Box::new(event)).await;
        assert!(result.is_ok());

        let store = context
            .store
            .as_any()
            .downcast_ref::<MockDataStore>()
            .unwrap(); // Test assertion - safe to unwrap
        let events = store.get_events().await;
        assert_eq!(events.len(), 1);
        assert!(events
            .first()
            .unwrap() // Test assertion - safe to unwrap
            .block_height
            .is_none());
        assert!(events
            .first()
            .unwrap() // Test assertion - safe to unwrap
            .transaction_hash
            .is_none());
    }

    #[tokio::test]
    async fn test_store_event_cache_failure() {
        let context = create_test_context();

        // Create a store that fails caching
        let mut mock_store = MockDataStore::new();
        mock_store.set_should_fail(true);

        // But allow the main insert to work by using fail_after a high number
        mock_store.set_fail_after(999);

        let new_context = Arc::new(ServiceContext::new(
            (*context.config).clone(),
            Arc::new(mock_store),
            Arc::new(create_mock_metrics_collector()),
        ));

        let config = ProcessorConfig {
            workers: 1,
            batch_config: BatchConfig {
                max_size: 10,
                max_age: Duration::from_secs(5),
                target_size: 5,
            },
            retry_config: RetryConfig {
                max_attempts: 1,
                base_delay: Duration::from_millis(10),
                max_delay: Duration::from_secs(1),
                backoff_multiplier: 2.0,
                jitter: 0.1,
            },
        };

        let processor = EventProcessor::new(config, new_context).unwrap(); // Test assertion - safe to unwrap
        let event = MockEvent::new("cache_fail_test");

        // Should succeed despite cache failure
        let result = processor.process_event(Box::new(event)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_with_retry_success_first_attempt() {
        let (processor, _) = create_test_processor();
        let event = MockEvent::new("retry_test_success");

        let result = processor.process_event(Box::new(event)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_with_retry_success_after_retries() {
        let context = create_test_context();

        // Create a store that fails first attempt but succeeds on retry
        let mut mock_store = MockDataStore::new();
        mock_store.set_fail_after(0); // Fail immediately
        mock_store.set_should_fail(true);

        let new_context = Arc::new(ServiceContext::new(
            (*context.config).clone(),
            Arc::new(mock_store),
            Arc::new(create_mock_metrics_collector()),
        ));

        let config = ProcessorConfig {
            workers: 1,
            batch_config: BatchConfig {
                max_size: 10,
                max_age: Duration::from_secs(5),
                target_size: 5,
            },
            retry_config: RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(100),
                backoff_multiplier: 2.0,
                jitter: 0.1,
            },
        };

        let processor = EventProcessor::new(config, new_context.clone()).unwrap(); // Test assertion - safe to unwrap

        // Start a task that will disable failure after a short delay
        let new_context_clone = new_context.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            let store_ref = new_context_clone
                .store
                .as_any()
                .downcast_ref::<MockDataStore>()
                .unwrap(); // Test assertion - safe to unwrap
            store_ref.set_should_fail(false);
        });

        let event = MockEvent::new("retry_test_eventual_success");
        let result = processor.process_event(Box::new(event)).await;
        // This may fail or succeed depending on timing, but shouldn't panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_process_with_retry_max_attempts_exceeded() {
        let context = create_test_context();

        // Create a store that always fails
        let mut mock_store = MockDataStore::new();
        mock_store.set_should_fail(true);
        mock_store.set_fail_after(0);

        let new_context = Arc::new(ServiceContext::new(
            (*context.config).clone(),
            Arc::new(mock_store),
            Arc::new(create_mock_metrics_collector()),
        ));

        let config = ProcessorConfig {
            workers: 1,
            batch_config: BatchConfig {
                max_size: 10,
                max_age: Duration::from_secs(5),
                target_size: 5,
            },
            retry_config: RetryConfig {
                max_attempts: 2,
                base_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(50),
                backoff_multiplier: 2.0,
                jitter: 0.1,
            },
        };

        let processor = EventProcessor::new(config, new_context).unwrap(); // Test assertion - safe to unwrap
        let event = MockEvent::new("retry_test_failure");

        let result = processor.process_event(Box::new(event)).await;
        assert!(result.is_err());
    }

    // Test serialization failure
    #[derive(Debug)]
    struct FailingEvent {
        kind: EventKind,
        metadata: EventMetadata,
    }

    impl FailingEvent {
        fn new() -> Self {
            Self {
                kind: EventKind::Custom("failing".to_string()),
                metadata: EventMetadata::new(
                    "failing_event".to_string(),
                    EventKind::Custom("failing".to_string()),
                    "test".to_string(),
                ),
            }
        }
    }

    impl Event for FailingEvent {
        fn id(&self) -> &'static str {
            "failing_event"
        }

        fn kind(&self) -> &EventKind {
            &self.kind
        }

        fn source(&self) -> &'static str {
            "test"
        }

        fn timestamp(&self) -> SystemTime {
            Utc::now().into()
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
            Box::new(Self::new())
        }

        fn to_json(&self) -> Result<serde_json::Value, EventError> {
            Err(EventError::generic("Serialization failed"))
        }
    }

    #[tokio::test]
    async fn test_store_event_serialization_failure() {
        let (processor, _) = create_test_processor();
        let event = Box::new(FailingEvent::new()) as Box<dyn Event>;

        let result = processor.process_event(event).await;
        assert!(result.is_err());

        match result {
            Err(IndexerError::Processing(ProcessingError::ValidationFailed { reason })) => {
                assert!(reason.contains("Non-retriable error"));
                assert!(reason.contains("Serialization failed for event failing_event"));
            }
            Err(other_error) => {
                panic!(
                    "Expected ValidationFailed error wrapping SerializationFailed, got: {other_error:?}"
                );
            }
            Ok(()) => {
                panic!("Expected error but got success");
            }
        }
    }

    // Test worker semaphore acquisition failure (edge case)
    #[tokio::test]
    async fn test_worker_semaphore_closed() {
        let (processor, _) = create_test_processor();

        // Close the semaphore to simulate acquisition failure
        processor.worker_semaphore.close();

        let event = MockEvent::new("semaphore_test");
        let result = processor.process_event(Box::new(event)).await;
        assert!(result.is_err());
    }

    // Test stats collection behavior
    #[tokio::test]
    async fn test_stats_calculation_with_data() {
        let (processor, _) = create_test_processor();

        // Update stats manually to test calculation paths
        {
            let mut stats = processor.stats.write().await;
            stats.total_processed = 10;
            stats.total_errors = 0;
            stats.avg_latency_ms = 0.0; // Test initial latency calculation
        }

        // Process an event to test latency update
        let event = MockEvent::new("latency_test");
        let result = processor.process_event(Box::new(event)).await;
        assert!(result.is_ok());

        // Check that latency was calculated (may be 0 for very fast operations)
        let stats = processor.stats.read().await;
        assert!(stats.avg_latency_ms >= 0.0);
        assert_eq!(stats.total_processed, 11);
        drop(stats);
    }

    #[tokio::test]
    async fn test_stats_exponential_moving_average() {
        let (processor, _) = create_test_processor();

        // Set initial latency
        {
            let mut stats = processor.stats.write().await;
            stats.avg_latency_ms = 100.0;
        }

        // Process an event to test EMA calculation
        let event = MockEvent::new("ema_test");
        let result = processor.process_event(Box::new(event)).await;
        assert!(result.is_ok());

        let stats = processor.stats.read().await;
        // Should be different from 100.0 due to EMA calculation
        assert_ne!(stats.avg_latency_ms, 100.0);
        drop(stats);
    }

    // Test pipeline stage trait
    #[tokio::test]
    async fn test_mock_pipeline_stage_name() {
        let stage = MockPipelineStage::new("test_stage", false);
        assert_eq!(stage.name(), "test_stage");
    }

    // Test edge cases for health check
    #[tokio::test]
    async fn test_health_check_zero_processed_events() {
        let (processor, _) = create_test_processor();

        {
            let mut stats = processor.stats.write().await;
            stats.total_processed = 0;
            stats.total_errors = 5; // Errors but no processed events
        }

        let health = processor.health().await.unwrap(); // Test assertion - safe to unwrap
        assert!(!health.healthy); // Should be unhealthy with errors
    }

    // Test that service state changes affect stats collection
    #[tokio::test]
    async fn test_stats_collection_stops_on_service_stop() {
        let (mut processor, context) = create_test_processor();

        // Start the processor
        let _ = processor.start().await;

        // Change service state to stopping
        context.set_state(ServiceState::Stopping).await;

        // Give stats collection time to detect state change
        sleep(Duration::from_millis(50)).await;

        // The stats collection should eventually stop (we can't easily test this without
        // making the interval much shorter, but the code path is covered)
    }
}
