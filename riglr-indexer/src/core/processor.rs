//! Event processor for parallel processing with worker pools

use backoff::{backoff::Backoff, ExponentialBackoff};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::config::{BatchConfig, RetryConfig};
use crate::core::{
    ComponentHealth, EventProcessing, HealthStatus, ProcessingStats, ServiceContext,
    ServiceLifecycle,
};
use crate::error::{IndexerError, IndexerResult, ProcessingError};
use crate::utils::BatchProcessor;
use riglr_events_core::prelude::*;

/// Configuration for the event processor
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Number of worker threads
    pub workers: usize,
    /// Batch processing configuration
    pub batch_config: BatchConfig,
    /// Retry configuration
    pub retry_config: RetryConfig,
}

/// Event processor that handles parallel processing with worker pools
#[derive(Clone)]
pub struct EventProcessor {
    /// Processor configuration
    config: ProcessorConfig,
    /// Shared service context
    context: Arc<ServiceContext>,
    /// Worker pool semaphore for limiting concurrent operations
    worker_semaphore: Arc<Semaphore>,
    /// Processing statistics
    stats: Arc<RwLock<ProcessorStats>>,
    /// Active workers counter
    active_workers: Arc<RwLock<usize>>,
    /// Batch processor
    #[allow(dead_code)]
    batch_processor: Arc<BatchProcessor<Box<dyn Event>>>,
}

/// Internal processor statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessorStats {
    /// Total events processed
    total_processed: u64,
    /// Total errors encountered
    total_errors: u64,
    /// Events processed per second
    events_per_second: f64,
    /// Average processing latency in milliseconds
    avg_latency_ms: f64,
    /// Current batch size
    #[allow(dead_code)]
    current_batch_size: usize,
    /// Last processing time
    last_processing_time: Option<Instant>,
}

impl EventProcessor {
    /// Create a new event processor
    pub async fn new(config: ProcessorConfig, context: Arc<ServiceContext>) -> IndexerResult<Self> {
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
            stats: Arc::new(RwLock::new(ProcessorStats::default())),
            active_workers: Arc::new(RwLock::new(0)),
            batch_processor: Arc::new(batch_processor),
        };

        info!("Event processor initialized successfully");
        Ok(processor)
    }

    /// Start background statistics collection
    fn start_stats_collection(&self) {
        let stats = self.stats.clone();
        let context = self.context.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            let mut last_count = 0u64;
            let mut last_time = Instant::now();

            loop {
                interval.tick().await;

                let mut stats_guard = stats.write().await;
                let now = Instant::now();
                let elapsed = now.duration_since(last_time).as_secs_f64();

                if elapsed > 0.0 {
                    let events_delta = stats_guard.total_processed.saturating_sub(last_count);
                    stats_guard.events_per_second = events_delta as f64 / elapsed;

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
                        stats_guard.total_errors as f64 / stats_guard.total_processed as f64
                    } else {
                        0.0
                    };
                    context
                        .metrics
                        .record_gauge("indexer_processing_error_rate", error_rate);
                }

                last_count = stats_guard.total_processed;
                last_time = now;
                drop(stats_guard);

                // Check for shutdown
                if matches!(
                    context.state().await,
                    crate::core::ServiceState::Stopping | crate::core::ServiceState::Stopped
                ) {
                    break;
                }
            }
        });
    }

    /// Process a single event with retry logic
    async fn process_single_event(&self, event: Box<dyn Event>) -> IndexerResult<()> {
        let start_time = Instant::now();

        // Acquire worker permit
        let _permit = self
            .worker_semaphore
            .acquire()
            .await
            .map_err(|_| IndexerError::internal("Failed to acquire worker permit"))?;

        // Increment active workers
        {
            let mut workers = self.active_workers.write().await;
            *workers += 1;
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

        match &result {
            Ok(_) => {
                stats.total_processed += 1;
                stats.last_processing_time = Some(start_time);

                // Update average latency with exponential moving average
                let latency_ms = processing_time.as_millis() as f64;
                if stats.avg_latency_ms == 0.0 {
                    stats.avg_latency_ms = latency_ms;
                } else {
                    stats.avg_latency_ms = stats.avg_latency_ms * 0.9 + latency_ms * 0.1;
                }
            }
            Err(_) => {
                stats.total_errors += 1;
            }
        }

        result
    }

    /// Process event with exponential backoff retry
    async fn process_with_retry(&self, event: Box<dyn Event>) -> IndexerResult<()> {
        let mut backoff = ExponentialBackoff {
            initial_interval: self.config.retry_config.base_delay,
            max_interval: self.config.retry_config.max_delay,
            multiplier: self.config.retry_config.backoff_multiplier,
            max_elapsed_time: Some(Duration::from_secs(300)), // 5 minutes max
            ..Default::default()
        };

        let mut attempts = 0;
        let max_attempts = self.config.retry_config.max_attempts;

        loop {
            attempts += 1;

            match self.store_event(&*event).await {
                Ok(_) => {
                    if attempts > 1 {
                        info!("Event processed successfully after {} attempts", attempts);
                    }
                    return Ok(());
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        error!("Failed to process event after {} attempts: {}", attempts, e);
                        return Err(e);
                    }

                    if !e.is_retriable() {
                        error!("Non-retriable error processing event: {}", e);
                        return Err(e);
                    }

                    if let Some(delay) = backoff.next_backoff() {
                        warn!(
                            "Retrying event processing in {:?} (attempt {}/{})",
                            delay, attempts, max_attempts
                        );
                        sleep(delay).await;
                    } else {
                        error!(
                            "Backoff timer expired, giving up after {} attempts",
                            attempts
                        );
                        return Err(IndexerError::Processing(ProcessingError::PipelineStalled {
                            seconds: 300,
                        }));
                    }
                }
            }
        }
    }

    /// Store event using the data store
    async fn store_event(&self, event: &dyn Event) -> IndexerResult<()> {
        debug!("Storing event: {} (type: {:?})", event.id(), event.kind());

        // Convert event to JSON for storage
        let event_json = event.to_json().map_err(|_e| {
            IndexerError::Processing(ProcessingError::SerializationFailed {
                event_id: event.id().to_string(),
            })
        })?;

        // Create storage event data
        let storage_event = crate::storage::StoredEvent {
            id: event.id().to_string(),
            event_type: format!("{:?}", event.kind()),
            source: event.source().to_string(),
            data: event_json,
            timestamp: chrono::DateTime::from(event.timestamp()),
            block_height: event
                .metadata()
                .custom
                .get("block_height")
                .and_then(|v| v.as_u64()),
            transaction_hash: event
                .metadata()
                .custom
                .get("transaction_hash")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        // Store in primary storage
        self.context.store.insert_event(&storage_event).await?;

        // If caching is enabled, also cache the event
        if matches!(
            self.context.config.storage.cache.backend,
            crate::config::CacheBackend::Redis
        ) {
            if let Err(e) = self
                .context
                .store
                .cache_event(event.id(), &storage_event, Some(Duration::from_secs(3600)))
                .await
            {
                warn!("Failed to cache event {}: {}", event.id(), e);
                // Don't fail the entire operation for cache failures
            }
        }

        debug!("Event {} stored successfully", event.id());
        Ok(())
    }

    /// Get current number of active workers
    pub async fn active_workers(&self) -> usize {
        *self.active_workers.read().await
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
                stats.total_errors as f64 / stats.total_processed as f64
            } else {
                0.0
            },
            queue_depth: 0, // Not applicable for processor
            active_workers,
        }
    }
}

#[async_trait::async_trait]
impl ServiceLifecycle for EventProcessor {
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
                warn!(
                    "Timeout waiting for workers to complete, {} workers still active",
                    *self.active_workers.read().await
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
                && (stats.total_errors as f64 / stats.total_processed as f64) < 0.1);

        let mut components = std::collections::HashMap::new();

        let message = if healthy {
            format!(
                "Processor healthy: {:.2} events/sec, {:.1}ms avg latency",
                stats.events_per_second, stats.avg_latency_ms
            )
        } else {
            format!(
                "Processor unhealthy: {:.2}% error rate",
                if stats.total_processed > 0 {
                    stats.total_errors as f64 / stats.total_processed as f64 * 100.0
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

        Ok(HealthStatus {
            healthy,
            components,
            timestamp: chrono::Utc::now(),
        })
    }

    fn is_running(&self) -> bool {
        // Processor is considered running if we have capacity to process events
        self.worker_semaphore.available_permits() > 0
    }
}

#[async_trait::async_trait]
impl EventProcessing for EventProcessor {
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
                tokio::spawn(async move { processor.process_single_event(event).await })
            })
            .collect();

        // Wait for all tasks to complete
        let mut success_count = 0;
        let mut error_count = 0;

        for (index, task) in tasks.into_iter().enumerate() {
            match task.await {
                Ok(Ok(_)) => {
                    success_count += 1;
                }
                Ok(Err(e)) => {
                    error!("Event {} in batch failed: {}", index, e);
                    error_count += 1;
                }
                Err(e) => {
                    error!("Task {} in batch panicked: {}", index, e);
                    error_count += 1;
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
            processing_time.as_millis() as f64,
        );

        self.context
            .metrics
            .record_histogram("indexer_batch_size", (success_count + error_count) as f64);

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
                            success_count + error_count
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
pub struct ProcessingPipeline {
    stages: Vec<Box<dyn PipelineStage + Send + Sync>>,
}

impl Default for ProcessingPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessingPipeline {
    /// Create a new processing pipeline
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Add a stage to the pipeline
    pub fn add_stage(mut self, stage: Box<dyn PipelineStage + Send + Sync>) -> Self {
        self.stages.push(stage);
        self
    }

    /// Process an event through all stages
    pub async fn process(&self, mut event: Box<dyn Event>) -> IndexerResult<Box<dyn Event>> {
        for (index, stage) in self.stages.iter().enumerate() {
            match stage.process(event).await {
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
    /// Process an event and return the modified event
    async fn process(&self, event: Box<dyn Event>) -> IndexerResult<Box<dyn Event>>;

    /// Get stage name for debugging
    fn name(&self) -> &str;
}
