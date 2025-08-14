//! Metrics collector implementation

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, error, debug};

use crate::config::MetricsConfig;
use crate::error::{IndexerError, IndexerResult};
use crate::metrics::{MetricsRegistry, IndexerMetrics, PerformanceMetrics};

/// Main metrics collector
pub struct MetricsCollector {
    /// Metrics registry
    registry: Arc<MetricsRegistry>,
    /// Configuration
    config: MetricsConfig,
    /// Current indexer metrics snapshot
    current_metrics: Arc<RwLock<IndexerMetrics>>,
    /// Collection start time
    start_time: std::time::Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: &MetricsConfig) -> IndexerResult<Self> {
        info!("Initializing metrics collector");

        let registry = Arc::new(MetricsRegistry::new(config.clone()));

        let collector = Self {
            registry,
            config: config.clone(),
            current_metrics: Arc::new(RwLock::new(IndexerMetrics::default())),
            start_time: std::time::Instant::now(),
        };

        // Initialize standard metrics
        collector.init_standard_metrics()?;

        info!("Metrics collector initialized successfully");
        Ok(collector)
    }

    /// Initialize standard indexer metrics
    fn init_standard_metrics(&self) -> IndexerResult<()> {
        // Initialize counters
        self.registry.set_gauge("indexer_events_processed_total", 0.0)?;
        self.registry.set_gauge("indexer_events_processing_rate", 0.0)?;
        self.registry.set_gauge("indexer_processing_latency_ms", 0.0)?;
        self.registry.set_gauge("indexer_queue_depth", 0.0)?;
        self.registry.set_gauge("indexer_active_workers", 0.0)?;
        self.registry.set_gauge("indexer_error_rate", 0.0)?;
        self.registry.set_gauge("indexer_storage_size_bytes", 0.0)?;
        self.registry.set_gauge("indexer_cache_hit_rate", 0.0)?;
        self.registry.set_gauge("indexer_uptime_seconds", 0.0)?;

        // Initialize component health gauges
        self.registry.set_gauge("indexer_component_health_ingester", 1.0)?;
        self.registry.set_gauge("indexer_component_health_processor", 1.0)?;
        self.registry.set_gauge("indexer_component_health_storage", 1.0)?;

        Ok(())
    }

    /// Record a counter increment
    pub fn increment_counter(&self, name: &str) {
        if let Err(e) = self.registry.increment_counter(name, 1) {
            error!("Failed to increment counter {}: {}", name, e);
        }
    }

    /// Record a counter value
    pub fn record_counter(&self, name: &str, value: u64) {
        if let Err(e) = self.registry.set_gauge(name, value as f64) {
            error!("Failed to record counter {}: {}", name, e);
        }
    }

    /// Record a gauge value
    pub fn record_gauge(&self, name: &str, value: f64) {
        if let Err(e) = self.registry.set_gauge(name, value) {
            error!("Failed to record gauge {}: {}", name, e);
        }
    }

    /// Record a histogram value
    pub fn record_histogram(&self, name: &str, value: f64) {
        if let Err(e) = self.registry.record_histogram(name, value) {
            error!("Failed to record histogram {}: {}", name, e);
        }
    }

    /// Update indexer-specific metrics
    pub async fn update_indexer_metrics(&self, metrics: IndexerMetrics) {
        let mut current = self.current_metrics.write().await;
        *current = metrics.clone();
        drop(current);

        // Update registry with current values
        self.record_gauge("indexer_events_processed_total", metrics.events_processed_total as f64);
        self.record_gauge("indexer_events_processing_rate", metrics.events_processing_rate);
        self.record_gauge("indexer_processing_latency_ms", metrics.processing_latency_ms);
        self.record_gauge("indexer_queue_depth", metrics.queue_depth as f64);
        self.record_gauge("indexer_active_workers", metrics.active_workers as f64);
        self.record_gauge("indexer_error_rate", metrics.error_rate);
        self.record_gauge("indexer_storage_size_bytes", metrics.storage_size_bytes as f64);
        self.record_gauge("indexer_cache_hit_rate", metrics.cache_hit_rate);
        
        // Record uptime
        let uptime_seconds = self.start_time.elapsed().as_secs() as f64;
        self.record_gauge("indexer_uptime_seconds", uptime_seconds);

        debug!("Updated indexer metrics: {} events processed, {:.2} events/sec", 
               metrics.events_processed_total, metrics.events_processing_rate);
    }

    /// Get current indexer metrics
    pub async fn get_indexer_metrics(&self) -> IndexerMetrics {
        self.current_metrics.read().await.clone()
    }

    /// Get performance metrics summary
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        let current = self.current_metrics.read().await;
        
        // This is a simplified implementation
        // In production, you'd calculate these from historical data
        PerformanceMetrics {
            throughput: crate::metrics::ThroughputMetrics {
                events_per_second: current.events_processing_rate,
                avg_events_per_second: current.events_processing_rate, // Simplified
                peak_events_per_second: current.events_processing_rate, // Simplified
                total_events: current.events_processed_total,
            },
            latency: crate::metrics::LatencyMetrics {
                avg_latency_ms: current.processing_latency_ms,
                p50_latency_ms: current.processing_latency_ms, // Simplified
                p95_latency_ms: current.processing_latency_ms * 1.5, // Simplified
                p99_latency_ms: current.processing_latency_ms * 2.0, // Simplified
                max_latency_ms: current.processing_latency_ms * 3.0, // Simplified
            },
            resources: crate::metrics::ResourceMetrics {
                cpu_usage_percent: 0.0, // Would need system metrics
                memory_usage_bytes: 0, // Would need system metrics
                memory_usage_percent: 0.0, // Would need system metrics
                db_connections_active: current.active_workers as u32, // Simplified
                db_connections_max: 20, // From config
            },
            errors: crate::metrics::ErrorMetrics {
                total_errors: (current.events_processed_total as f64 * current.error_rate) as u64,
                error_rate: current.error_rate,
                errors_by_category: std::collections::HashMap::new(), // Would need tracking
                recent_error_rate: current.error_rate, // Simplified
            },
        }
    }

    /// Start metrics collection background task
    pub fn start_collection_task(&self) -> tokio::task::JoinHandle<()> {
        let collector = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(collector.config.collection_interval);
            
            loop {
                interval.tick().await;
                
                // Collect system metrics here if needed
                // For now, we'll just log that collection is running
                debug!("Metrics collection tick");
                
                // In a real implementation, you might:
                // 1. Collect system metrics (CPU, memory, etc.)
                // 2. Calculate derived metrics
                // 3. Clean up old histogram data
                // 4. Export metrics to external systems
            }
        })
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        self.registry.export_prometheus()
    }

    /// Export metrics as JSON
    pub fn export_json(&self) -> IndexerResult<serde_json::Value> {
        let metrics = self.registry.get_all_metrics();
        serde_json::to_value(metrics)
            .map_err(|e| IndexerError::internal_with_source("Failed to serialize metrics", e))
    }

    /// Flush metrics (for graceful shutdown)
    pub async fn flush(&self) -> IndexerResult<()> {
        info!("Flushing metrics");
        
        // In a real implementation, this would:
        // 1. Send any pending metrics to external systems
        // 2. Persist metrics state
        // 3. Clean up resources
        
        Ok(())
    }

    /// Reset all metrics
    pub fn reset(&self) {
        info!("Resetting all metrics");
        self.registry.clear();
        if let Err(e) = self.init_standard_metrics() {
            error!("Failed to reinitialize standard metrics: {}", e);
        }
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            config: self.config.clone(),
            current_metrics: self.current_metrics.clone(),
            start_time: self.start_time,
        }
    }
}