//! Event ingester for high-throughput data collection

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{info, error};

use riglr_events_core::prelude::*;

use crate::core::{ServiceContext, ServiceLifecycle, ComponentHealth, HealthStatus};
use crate::error::{IndexerError, IndexerResult};
use crate::utils::RateLimiter;

/// Configuration for the event ingester
#[derive(Debug, Clone)]
pub struct IngesterConfig {
    /// Number of ingestion workers
    pub workers: usize,
    /// Batch size for processing
    pub batch_size: usize,
    /// Queue capacity
    pub queue_capacity: usize,
}

/// Type alias for event sender
type EventSender = mpsc::UnboundedSender<Box<dyn Event>>;
/// Type alias for event receiver
type EventReceiver = mpsc::UnboundedReceiver<Box<dyn Event>>;

/// Event ingester that collects events from multiple sources
#[derive(Clone)]
pub struct EventIngester {
    /// Ingester configuration
    config: IngesterConfig,
    /// Shared service context
    context: Arc<ServiceContext>,
    /// Event queue sender
    event_tx: Arc<RwLock<Option<EventSender>>>,
    /// Event queue receiver for external access
    event_rx: Arc<RwLock<Option<EventReceiver>>>,
    /// Stream manager for handling multiple data sources
    stream_manager: Arc<RwLock<Option<StreamManager>>>,
    /// Rate limiter for ingestion
    rate_limiter: Arc<RateLimiter>,
    /// Ingestion statistics
    stats: Arc<RwLock<IngestionStats>>,
}

/// Ingestion statistics
#[derive(Debug, Clone, Default)]
pub struct IngestionStats {
    /// Total events ingested
    pub total_events: u64,
    /// Events per second
    pub events_per_second: f64,
    /// Queue depth
    pub queue_depth: usize,
    /// Active streams
    pub active_streams: usize,
    /// Error count
    pub error_count: u64,
    /// Last successful ingestion
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
}

impl EventIngester {
    /// Create a new event ingester
    pub async fn new(
        config: IngesterConfig,
        context: Arc<ServiceContext>,
    ) -> IndexerResult<Self> {
        info!("Initializing event ingester with {} workers", config.workers);

        // Create event queue
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Create rate limiter
        let rate_config = &context.config.processing.rate_limit;
        let rate_limiter = if rate_config.enabled {
            RateLimiter::new(
                rate_config.max_events_per_second,
                rate_config.burst_capacity,
            )
        } else {
            RateLimiter::unlimited()
        };

        let ingester = Self {
            config,
            context,
            event_tx: Arc::new(RwLock::new(Some(event_tx))),
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            stream_manager: Arc::new(RwLock::new(None)),
            rate_limiter: Arc::new(rate_limiter),
            stats: Arc::new(RwLock::new(IngestionStats::default())),
        };

        info!("Event ingester initialized successfully");
        Ok(ingester)
    }

    /// Initialize stream sources
    async fn initialize_streams(&self) -> IndexerResult<()> {
        info!("Initializing event streams...");

        // Create stream manager
        let mut builder = StreamManagerBuilder::new();

        // Add Solana streams if configured
        if let Ok(rpc_url) = std::env::var("SOLANA_RPC_URL") {
            info!("Adding Solana stream source: {}", rpc_url);
            
            // Create a Solana event stream
            // This would integrate with riglr-streams to create actual streams
            // For now, we'll create a placeholder that can be extended
            builder = builder.with_config(StreamManagerConfig {
                max_concurrent_streams: 10,
                health_check_interval: Duration::from_secs(30),
                reconnect_delay: Duration::from_secs(5),
                max_reconnect_attempts: 3,
            });
        }

        // Add EVM streams if configured
        for (chain_id_str, rpc_url) in std::env::vars() {
            if chain_id_str.starts_with("RPC_URL_") {
                if let Some(chain_id_str) = chain_id_str.strip_prefix("RPC_URL_") {
                    if let Ok(chain_id) = chain_id_str.parse::<u64>() {
                        info!("Adding EVM stream source for chain {}: {}", chain_id, rpc_url);
                        // Add EVM stream configuration
                    }
                }
            }
        }

        let manager = builder.build();
        *self.stream_manager.write().await = Some(manager);

        info!("Event streams initialized");
        Ok(())
    }

    /// Start ingestion workers
    async fn start_workers(&self) -> IndexerResult<()> {
        info!("Starting {} ingestion workers", self.config.workers);

        let event_tx = self.event_tx.read().await;
        let tx = event_tx.as_ref()
            .ok_or_else(|| IndexerError::internal("Event sender not initialized"))?
            .clone();
        
        drop(event_tx);

        for worker_id in 0..self.config.workers {
            let context = self.context.clone();
            let stats = self.stats.clone();
            let rate_limiter = self.rate_limiter.clone();
            let tx = tx.clone();
            
            tokio::spawn(async move {
                info!("Starting ingestion worker {}", worker_id);
                
                let mut shutdown_rx = context.shutdown_receiver();
                let mut batch_interval = interval(Duration::from_millis(100));

                loop {
                    tokio::select! {
                        _ = batch_interval.tick() => {
                            // This is where we would pull events from external sources
                            // For now, we'll create a mock event for testing
                            if context.config.features.experimental {
                                if let Ok(()) = rate_limiter.check() {
                                    let mock_event = create_mock_solana_event();
                                    
                                    if let Err(e) = tx.send(mock_event) {
                                        error!("Worker {} failed to send event: {}", worker_id, e);
                                        break;
                                    }
                                    
                                    // Update stats
                                    let mut stats = stats.write().await;
                                    stats.total_events += 1;
                                    stats.last_success = Some(chrono::Utc::now());
                                }
                            }
                        }
                        
                        _ = shutdown_rx.recv() => {
                            info!("Worker {} received shutdown signal", worker_id);
                            break;
                        }
                    }
                }
                
                info!("Ingestion worker {} stopped", worker_id);
            });
        }

        Ok(())
    }

    /// Start statistics collection
    fn start_stats_collection(&self) {
        let stats = self.stats.clone();
        let context = self.context.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            let mut last_count = 0u64;
            let mut last_time = std::time::Instant::now();
            
            loop {
                interval.tick().await;
                
                let mut stats_guard = stats.write().await;
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(last_time).as_secs_f64();
                
                if elapsed > 0.0 {
                    let events_delta = stats_guard.total_events.saturating_sub(last_count);
                    stats_guard.events_per_second = events_delta as f64 / elapsed;
                    
                    // Record metrics
                    context.metrics.record_gauge(
                        "indexer_ingestion_events_per_second",
                        stats_guard.events_per_second,
                    );
                    context.metrics.record_counter(
                        "indexer_ingestion_total_events",
                        stats_guard.total_events,
                    );
                }
                
                last_count = stats_guard.total_events;
                last_time = now;
                
                drop(stats_guard);
                
                // Check for shutdown
                if matches!(context.state().await, crate::core::ServiceState::Stopping | crate::core::ServiceState::Stopped) {
                    break;
                }
            }
        });
    }

    /// Receive events from the ingestion queue
    pub async fn receive_events(&self) -> IndexerResult<Vec<Box<dyn Event>>> {
        let mut rx_guard = self.event_rx.write().await;
        let rx = rx_guard.as_mut()
            .ok_or_else(|| IndexerError::internal("Event receiver not initialized"))?;

        let mut events = Vec::new();
        let batch_size = self.config.batch_size;

        // Collect events up to batch size
        while events.len() < batch_size {
            match rx.try_recv() {
                Ok(event) => events.push(event),
                Err(mpsc::error::TryRecvError::Empty) => {
                    if events.is_empty() {
                        // Wait for at least one event
                        match rx.recv().await {
                            Some(event) => events.push(event),
                            None => break, // Channel closed
                        }
                    } else {
                        break; // Return partial batch
                    }
                }
                Err(mpsc::error::TryRecvError::Disconnected) => break,
            }
        }

        // Update queue depth statistics
        {
            let mut stats = self.stats.write().await;
            stats.queue_depth = rx.len();
        }

        Ok(events)
    }

    /// Get current queue depth
    pub async fn queue_depth(&self) -> usize {
        let stats = self.stats.read().await;
        stats.queue_depth
    }

    /// Get ingestion statistics
    pub async fn stats(&self) -> IngestionStats {
        self.stats.read().await.clone()
    }
}

#[async_trait::async_trait]
impl ServiceLifecycle for EventIngester {
    async fn start(&mut self) -> IndexerResult<()> {
        info!("Starting event ingester");

        // Initialize streams
        self.initialize_streams().await?;

        // Start workers
        self.start_workers().await?;

        // Start statistics collection
        self.start_stats_collection();

        // Update component health
        self.context.update_component_health(
            "ingester",
            ComponentHealth::healthy("Event ingester started successfully"),
        ).await;

        info!("Event ingester started with {} workers", self.config.workers);
        Ok(())
    }

    async fn stop(&mut self) -> IndexerResult<()> {
        info!("Stopping event ingester");

        // Close event sender to signal workers to stop
        {
            let mut tx_guard = self.event_tx.write().await;
            *tx_guard = None;
        }

        // Stop stream manager
        {
            let mut manager_guard = self.stream_manager.write().await;
            if let Some(_manager) = manager_guard.take() {
                // manager.stop().await?;
                // Manager is automatically dropped here
            }
        }

        info!("Event ingester stopped");
        Ok(())
    }

    async fn health(&self) -> IndexerResult<HealthStatus> {
        let stats = self.stats.read().await;
        
        let healthy = stats.error_count < 100 && // Less than 100 errors
                     stats.last_success.is_some_and(|t| {
                         chrono::Utc::now().signed_duration_since(t).num_minutes() < 5
                     });

        let mut components = std::collections::HashMap::new();
        
        let message = if healthy {
            format!("Ingester healthy: {} events/sec, {} errors", 
                   stats.events_per_second, stats.error_count)
        } else {
            format!("Ingester unhealthy: {} errors, last success: {:?}", 
                   stats.error_count, stats.last_success)
        };

        components.insert("ingester".to_string(), if healthy {
            ComponentHealth::healthy(&message)
        } else {
            ComponentHealth::unhealthy(&message)
        });

        Ok(HealthStatus {
            healthy,
            components,
            timestamp: chrono::Utc::now(),
        })
    }

    fn is_running(&self) -> bool {
        // Check if event sender is still active
        tokio::runtime::Handle::try_current()
            .ok()
            .and_then(|_| {
                let tx_guard = self.event_tx.try_read().ok()?;
                Some(tx_guard.is_some())
            })
            .unwrap_or(false)
    }
}

/// Create a mock Solana event for testing
fn create_mock_solana_event() -> Box<dyn Event> {
    // For now, create a minimal mock event that implements the Event trait
    // In a real implementation, this would use proper Solana event creation
    #[derive(Debug, Clone)]
    struct MockEvent {
        id: String,
        timestamp: std::time::SystemTime,
        metadata: EventMetadata,
        kind: EventKind,
    }
    
    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.id
        }
        
        fn kind(&self) -> &EventKind {
            &self.kind
        }
        
        fn source(&self) -> &str {
            "mock"
        }
        
        fn timestamp(&self) -> std::time::SystemTime {
            self.timestamp
        }
        
        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }
        
        fn metadata_mut(&mut self) -> &mut EventMetadata {
            &mut self.metadata
        }
        
        fn as_any(&self) -> &(dyn std::any::Any + 'static) {
            self
        }
        
        fn as_any_mut(&mut self) -> &mut (dyn std::any::Any + 'static) {
            self
        }
        
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
        
        fn to_json(&self) -> Result<serde_json::Value, riglr_events_core::EventError> {
            Ok(serde_json::json!({
                "id": self.id,
                "type": "swap",
                "source": "mock",
                "timestamp": self.timestamp.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
            }))
        }
    }
    
    let custom_data = std::collections::HashMap::new();
    let metadata = EventMetadata {
        id: "mock".to_string(),
        source: "mock".to_string(),
        kind: EventKind::Swap,
        chain_data: None,
        received_at: chrono::Utc::now(),
        timestamp: chrono::Utc::now(),
        custom: custom_data,
    };
    
    Box::new(MockEvent {
        id: format!("mock-{}", uuid::Uuid::new_v4()),
        timestamp: std::time::SystemTime::now(),
        metadata,
        kind: EventKind::Swap,
    })
}

// Placeholder types that would be implemented properly with riglr-streams integration
struct StreamManager;
struct StreamManagerBuilder;
#[allow(dead_code)]
struct StreamManagerConfig {
    max_concurrent_streams: usize,
    health_check_interval: Duration,
    reconnect_delay: Duration,
    max_reconnect_attempts: u32,
}

impl StreamManagerBuilder {
    fn new() -> Self {
        Self
    }

    fn with_config(self, _config: StreamManagerConfig) -> Self {
        self
    }

    fn build(self) -> StreamManager {
        StreamManager
    }
}