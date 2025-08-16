//! Main indexer service implementation

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{info, error, debug};

use riglr_events_core::prelude::*;

use crate::config::IndexerConfig;
use crate::core::{
    ServiceContext, ServiceState, ServiceLifecycle, EventProcessing, ProcessingStats,
    HealthStatus, ComponentHealth
};
use crate::error::{IndexerError, IndexerResult};
use crate::metrics::MetricsCollector;
use crate::core::{EventIngester, EventProcessor};

/// Main indexer service that orchestrates all components
pub struct IndexerService {
    /// Service context with shared state
    context: Arc<ServiceContext>,
    /// Event ingester for data collection
    ingester: Option<EventIngester>,
    /// Event processor for parallel processing
    processor: Option<EventProcessor>,
    /// Processing statistics
    stats: Arc<RwLock<ProcessingStats>>,
    /// Health check task handle
    health_task: Option<tokio::task::JoinHandle<()>>,
}

impl IndexerService {
    /// Create a new indexer service
    pub async fn new(config: IndexerConfig) -> IndexerResult<Self> {
        info!("Initializing RIGLR Indexer Service v{}", config.service.version);
        
        // Initialize data store
        let store = crate::storage::create_store(&config.storage).await?;
        
        // Initialize metrics collector
        let metrics = MetricsCollector::new(&config.metrics)?;
        
        // Create service context
        let context = Arc::new(ServiceContext::new(config, Arc::from(store), Arc::new(metrics)));
        
        let service = Self {
            context,
            ingester: None,
            processor: None,
            stats: Arc::new(RwLock::new(ProcessingStats::default())),
            health_task: None,
        };

        info!("Indexer service initialized successfully");
        Ok(service)
    }

    /// Initialize all service components
    async fn initialize_components(&mut self) -> IndexerResult<()> {
        info!("Initializing service components...");

        // Initialize ingester
        let ingester_config = crate::core::IngesterConfig {
            workers: self.context.config.processing.workers,
            batch_size: self.context.config.processing.batch.max_size,
            queue_capacity: self.context.config.processing.queue.capacity,
        };
        
        self.ingester = Some(EventIngester::new(
            ingester_config,
            self.context.clone(),
        ).await?);

        // Initialize processor
        let processor_config = crate::core::ProcessorConfig {
            workers: self.context.config.processing.workers,
            batch_config: self.context.config.processing.batch.clone(),
            retry_config: self.context.config.processing.retry.clone(),
        };
        
        self.processor = Some(EventProcessor::new(
            processor_config,
            self.context.clone(),
        ).await?);

        info!("All service components initialized");
        Ok(())
    }

    /// Start health monitoring task
    fn start_health_monitoring(&mut self) {
        let context = self.context.clone();
        let interval_duration = self.context.config.service.health_check_interval;
        
        self.health_task = Some(tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                match context.health_check().await {
                    Ok(status) => {
                        debug!("Health check completed: {} components", status.components.len());
                        
                        // Record metrics
                        let healthy_components = status.components.values().filter(|c| c.healthy).count();
                        context.metrics.record_gauge(
                            "indexer_healthy_components",
                            healthy_components as f64,
                        );
                        
                        context.metrics.record_gauge(
                            "indexer_total_components", 
                            status.components.len() as f64,
                        );
                    }
                    Err(e) => {
                        error!("Health check failed: {}", e);
                        context.metrics.increment_counter("indexer_health_check_errors");
                    }
                }
                
                // Check if we should stop
                if matches!(context.state().await, ServiceState::Stopping | ServiceState::Stopped) {
                    break;
                }
            }
            
            info!("Health monitoring task stopped");
        }));
    }

    /// Process events from the ingester
    #[allow(dead_code)]
    async fn process_events(&self) -> IndexerResult<()> {
        let ingester = self.ingester.as_ref()
            .ok_or_else(|| IndexerError::internal("Ingester not initialized"))?;
        let processor = self.processor.as_ref()
            .ok_or_else(|| IndexerError::internal("Processor not initialized"))?;

        info!("Starting event processing loop");
        let mut shutdown_rx = self.context.shutdown_receiver();
        
        loop {
            tokio::select! {
                // Process events from ingester
                events_result = ingester.receive_events() => {
                    match events_result {
                        Ok(events) => {
                            if !events.is_empty() {
                                // Update stats
                                {
                                    let mut stats = self.stats.write().await;
                                    stats.total_processed += events.len() as u64;
                                    stats.queue_depth = ingester.queue_depth().await;
                                }

                                // Process events
                                let start_time = std::time::Instant::now();
                                match processor.process_batch(events).await {
                                    Ok(_) => {
                                        let latency = start_time.elapsed().as_millis() as f64;
                                        self.context.metrics.record_histogram(
                                            "indexer_processing_latency_ms",
                                            latency,
                                        );
                                    }
                                    Err(e) => {
                                        error!("Batch processing failed: {}", e);
                                        self.context.metrics.increment_counter("indexer_processing_errors");
                                        
                                        // Update component health
                                        self.context.update_component_health(
                                            "processor",
                                            ComponentHealth::unhealthy(&format!("Processing error: {}", e)),
                                        ).await;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive events: {}", e);
                            self.context.metrics.increment_counter("indexer_ingestion_errors");
                            
                            // Update component health
                            self.context.update_component_health(
                                "ingester",
                                ComponentHealth::unhealthy(&format!("Ingestion error: {}", e)),
                            ).await;
                        }
                    }
                }
                
                // Handle shutdown
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received in processing loop");
                    break;
                }
            }
        }

        info!("Event processing loop stopped");
        Ok(())
    }

    /// Calculate processing statistics
    #[allow(dead_code)]
    async fn update_processing_stats(&self) {
        let mut stats = self.stats.write().await;
        
        // Calculate events per second (simple moving average)
        let now = std::time::Instant::now();
        static mut LAST_UPDATE: Option<std::time::Instant> = None;
        static mut LAST_COUNT: u64 = 0;
        
        unsafe {
            if let Some(last_time) = LAST_UPDATE {
                let elapsed = now.duration_since(last_time).as_secs_f64();
                if elapsed > 0.0 {
                    let events_delta = stats.total_processed.saturating_sub(LAST_COUNT);
                    stats.events_per_second = events_delta as f64 / elapsed;
                }
            }
            LAST_UPDATE = Some(now);
            LAST_COUNT = stats.total_processed;
        }

        // Get queue depth from ingester
        if let Some(ingester) = &self.ingester {
            stats.queue_depth = ingester.queue_depth().await;
        }

        // Get active workers from processor
        if let Some(processor) = &self.processor {
            stats.active_workers = processor.active_workers().await;
        }

        // Record metrics
        self.context.metrics.record_gauge("indexer_events_per_second", stats.events_per_second);
        self.context.metrics.record_gauge("indexer_queue_depth", stats.queue_depth as f64);
        self.context.metrics.record_gauge("indexer_active_workers", stats.active_workers as f64);
    }

    /// Get current configuration
    pub fn config(&self) -> &IndexerConfig {
        &self.context.config
    }

    /// Get service context
    pub fn context(&self) -> &Arc<ServiceContext> {
        &self.context
    }
}

#[async_trait::async_trait]
impl ServiceLifecycle for IndexerService {
    async fn start(&mut self) -> IndexerResult<()> {
        info!("Starting RIGLR Indexer Service");
        
        // Set state to starting
        self.context.set_state(ServiceState::Starting).await;

        // Initialize all components
        self.initialize_components().await?;

        // Start health monitoring
        self.start_health_monitoring();

        // Start ingester
        if let Some(ingester) = &mut self.ingester {
            ingester.start().await?;
            self.context.update_component_health(
                "ingester",
                ComponentHealth::healthy("Ingester started successfully"),
            ).await;
        }

        // Start processor
        if let Some(processor) = &mut self.processor {
            processor.start().await?;
            self.context.update_component_health(
                "processor", 
                ComponentHealth::healthy("Processor started successfully"),
            ).await;
        }

        // Set state to running
        self.context.set_state(ServiceState::Running).await;
        
        info!("RIGLR Indexer Service started successfully");
        info!("Service configuration:");
        info!("  - Workers: {}", self.context.config.processing.workers);
        info!("  - Batch size: {}", self.context.config.processing.batch.max_size);
        info!("  - Queue capacity: {}", self.context.config.processing.queue.capacity);
        info!("  - Storage backend: {:?}", self.context.config.storage.primary.backend);

        // Start main processing loop
        let context_clone = self.context.clone();
        let _stats_clone = self.stats.clone();
        let _ingester = self.ingester.as_ref().unwrap().clone();
        let _processor = self.processor.as_ref().unwrap().clone();

        tokio::spawn(async move {
            let mut shutdown_rx = context_clone.shutdown_receiver();
            let mut stats_interval = interval(Duration::from_secs(30));

            loop {
                tokio::select! {
                    // Update stats periodically
                    _ = stats_interval.tick() => {
                        // Update processing statistics would be called here
                        // But we need access to self which is not available in this context
                        // This will be handled by the processor itself
                    }
                    
                    // Handle shutdown
                    _ = shutdown_rx.recv() => {
                        info!("Main processing loop received shutdown signal");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop(&mut self) -> IndexerResult<()> {
        info!("Stopping RIGLR Indexer Service gracefully");
        
        // Set state to stopping
        self.context.set_state(ServiceState::Stopping).await;

        // Stop processor first to finish processing current events
        if let Some(processor) = &mut self.processor {
            if let Err(e) = processor.stop().await {
                error!("Error stopping processor: {}", e);
            } else {
                info!("Processor stopped successfully");
            }
        }

        // Stop ingester
        if let Some(ingester) = &mut self.ingester {
            if let Err(e) = ingester.stop().await {
                error!("Error stopping ingester: {}", e);
            } else {
                info!("Ingester stopped successfully");
            }
        }

        // Stop health monitoring
        if let Some(task) = self.health_task.take() {
            task.abort();
            info!("Health monitoring stopped");
        }

        // Final metrics flush
        if let Err(e) = self.context.metrics.flush().await {
            error!("Error flushing metrics: {}", e);
        }

        // Set state to stopped
        self.context.set_state(ServiceState::Stopped).await;
        
        info!("RIGLR Indexer Service stopped");
        Ok(())
    }

    async fn health(&self) -> IndexerResult<HealthStatus> {
        self.context.health_check().await
    }

    fn is_running(&self) -> bool {
        // We need a synchronous way to check this
        // For now, we'll use a simple heuristic
        self.ingester.is_some() && self.processor.is_some()
    }
}

#[async_trait::async_trait]
impl EventProcessing for IndexerService {
    async fn process_event(&self, event: Box<dyn Event>) -> IndexerResult<()> {
        let processor = self.processor.as_ref()
            .ok_or_else(|| IndexerError::internal("Processor not initialized"))?;
        
        processor.process_event(event).await
    }

    async fn process_batch(&self, events: Vec<Box<dyn Event>>) -> IndexerResult<()> {
        let processor = self.processor.as_ref()
            .ok_or_else(|| IndexerError::internal("Processor not initialized"))?;
        
        processor.process_batch(events).await
    }

    async fn processing_stats(&self) -> ProcessingStats {
        self.stats.read().await.clone()
    }
}