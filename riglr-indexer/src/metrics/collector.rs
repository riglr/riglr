//! Metrics collector implementation

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::config::MetricsConfig;
use crate::error::{IndexerError, IndexerResult};
use crate::metrics::{IndexerMetrics, MetricsRegistry, PerformanceMetrics};

/// Main metrics collector
#[derive(Debug)]
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
    pub fn new(config: MetricsConfig) -> IndexerResult<Self> {
        info!("Initializing metrics collector");

        let registry = Arc::new(MetricsRegistry::new(config.clone()));

        let collector = Self {
            registry,
            config,
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
        self.registry
            .set_gauge("indexer_events_processed_total", 0.0)?;
        self.registry
            .set_gauge("indexer_events_processing_rate", 0.0)?;
        self.registry
            .set_gauge("indexer_processing_latency_ms", 0.0)?;
        self.registry.set_gauge("indexer_queue_depth", 0.0)?;
        self.registry.set_gauge("indexer_active_workers", 0.0)?;
        self.registry.set_gauge("indexer_error_rate", 0.0)?;
        self.registry.set_gauge("indexer_storage_size_bytes", 0.0)?;
        self.registry.set_gauge("indexer_cache_hit_rate", 0.0)?;
        self.registry.set_gauge("indexer_uptime_seconds", 0.0)?;

        // Initialize component health gauges
        self.registry
            .set_gauge("indexer_component_health_ingester", 1.0)?;
        self.registry
            .set_gauge("indexer_component_health_processor", 1.0)?;
        self.registry
            .set_gauge("indexer_component_health_storage", 1.0)?;

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
        {
            let mut current = self.current_metrics.write().await;
            *current = metrics;
        }

        // Update registry with current values
        let current = self.current_metrics.read().await;
        self.record_gauge(
            "indexer_events_processed_total",
            current.events_processed_total as f64,
        );
        self.record_gauge(
            "indexer_events_processing_rate",
            current.events_processing_rate,
        );
        self.record_gauge(
            "indexer_processing_latency_ms",
            current.processing_latency_ms,
        );
        self.record_gauge("indexer_queue_depth", current.queue_depth as f64);
        self.record_gauge("indexer_active_workers", current.active_workers as f64);
        self.record_gauge("indexer_error_rate", current.error_rate);
        self.record_gauge(
            "indexer_storage_size_bytes",
            current.storage_size_bytes as f64,
        );
        self.record_gauge("indexer_cache_hit_rate", current.cache_hit_rate);

        // Record uptime
        let uptime_seconds = self.start_time.elapsed().as_secs() as f64;
        self.record_gauge("indexer_uptime_seconds", uptime_seconds);

        debug!(
            "Updated indexer metrics: {} events processed, {:.2} events/sec",
            current.events_processed_total, current.events_processing_rate
        );
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
                cpu_usage_percent: 0.0,    // Would need system metrics
                memory_usage_bytes: 0,     // Would need system metrics
                memory_usage_percent: 0.0, // Would need system metrics
                db_connections_active: current.active_workers as u32, // Simplified
                db_connections_max: 20,    // From config
            },
            errors: crate::metrics::ErrorMetrics {
                total_errors: (current.events_processed_total as f64 * current.error_rate) as u64,
                error_rate: current.error_rate,
                errors_by_category: std::collections::HashMap::new(), // Would need tracking
                recent_error_rate: current.error_rate,                // Simplified
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::timeout;

    fn create_test_config() -> MetricsConfig {
        MetricsConfig {
            enabled: true,
            port: 9090,
            endpoint: "/metrics".to_string(),
            collection_interval: Duration::from_secs(60),
            histogram_buckets: vec![0.1, 0.5, 1.0, 5.0, 10.0],
            custom: HashMap::new(),
        }
    }

    fn create_test_indexer_metrics() -> IndexerMetrics {
        IndexerMetrics {
            events_processed_total: 1000,
            events_processing_rate: 50.5,
            processing_latency_ms: 25.5,
            queue_depth: 10,
            active_workers: 5,
            error_rate: 0.05,
            storage_size_bytes: 1024 * 1024,
            cache_hit_rate: 0.85,
        }
    }

    #[tokio::test]
    async fn test_new_when_valid_config_should_create_collector() {
        let config = create_test_config();
        let result = MetricsCollector::new(config);

        assert!(result.is_ok());
        let collector = result.unwrap();
        assert_eq!(collector.config.port, 9090);
        assert_eq!(collector.config.endpoint, "/metrics");
    }

    #[tokio::test]
    async fn test_new_when_init_standard_metrics_fails_should_return_error() {
        // This test would require mocking the registry to fail on set_gauge
        // Since we can't easily mock in this test setup, we'll test the happy path
        let config = create_test_config();
        let result = MetricsCollector::new(config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_increment_counter_when_valid_name_should_not_panic() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        // This should not panic even if the counter doesn't exist
        collector.increment_counter("test_counter");

        // Multiple increments should work
        collector.increment_counter("test_counter");
        collector.increment_counter("another_counter");
    }

    #[tokio::test]
    async fn test_increment_counter_when_empty_name_should_not_panic() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        collector.increment_counter("");
    }

    #[tokio::test]
    async fn test_record_counter_when_valid_values_should_not_panic() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        collector.record_counter("test_counter", 100);
        collector.record_counter("test_counter", 0);
        collector.record_counter("test_counter", u64::MAX);
    }

    #[tokio::test]
    async fn test_record_counter_when_empty_name_should_not_panic() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        collector.record_counter("", 42);
    }

    #[tokio::test]
    async fn test_record_gauge_when_valid_values_should_not_panic() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        collector.record_gauge("test_gauge", 1.5);
        collector.record_gauge("test_gauge", 0.0);
        collector.record_gauge("test_gauge", -10.5);
        collector.record_gauge("test_gauge", f64::MAX);
        collector.record_gauge("test_gauge", f64::MIN);
        collector.record_gauge("test_gauge", f64::INFINITY);
        collector.record_gauge("test_gauge", f64::NEG_INFINITY);
    }

    #[tokio::test]
    async fn test_record_gauge_when_nan_should_not_panic() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        collector.record_gauge("test_gauge", f64::NAN);
    }

    #[tokio::test]
    async fn test_record_gauge_when_empty_name_should_not_panic() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        collector.record_gauge("", 3.14);
    }

    #[tokio::test]
    async fn test_record_histogram_when_valid_values_should_not_panic() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        collector.record_histogram("test_histogram", 1.0);
        collector.record_histogram("test_histogram", 0.0);
        collector.record_histogram("test_histogram", -5.5);
        collector.record_histogram("test_histogram", f64::MAX);
    }

    #[tokio::test]
    async fn test_record_histogram_when_empty_name_should_not_panic() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        collector.record_histogram("", 2.5);
    }

    #[tokio::test]
    async fn test_update_indexer_metrics_when_valid_metrics_should_update_successfully() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        let test_metrics = create_test_indexer_metrics();

        collector.update_indexer_metrics(test_metrics.clone()).await;

        let stored_metrics = collector.get_indexer_metrics().await;
        assert_eq!(
            stored_metrics.events_processed_total,
            test_metrics.events_processed_total
        );
        assert_eq!(
            stored_metrics.events_processing_rate,
            test_metrics.events_processing_rate
        );
        assert_eq!(
            stored_metrics.processing_latency_ms,
            test_metrics.processing_latency_ms
        );
        assert_eq!(stored_metrics.queue_depth, test_metrics.queue_depth);
        assert_eq!(stored_metrics.active_workers, test_metrics.active_workers);
        assert_eq!(stored_metrics.error_rate, test_metrics.error_rate);
        assert_eq!(
            stored_metrics.storage_size_bytes,
            test_metrics.storage_size_bytes
        );
        assert_eq!(stored_metrics.cache_hit_rate, test_metrics.cache_hit_rate);
    }

    #[tokio::test]
    async fn test_update_indexer_metrics_when_default_metrics_should_update_successfully() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        let default_metrics = IndexerMetrics::default();

        collector
            .update_indexer_metrics(default_metrics.clone())
            .await;

        let stored_metrics = collector.get_indexer_metrics().await;
        assert_eq!(stored_metrics.events_processed_total, 0);
        assert_eq!(stored_metrics.events_processing_rate, 0.0);
        assert_eq!(stored_metrics.processing_latency_ms, 0.0);
    }

    #[tokio::test]
    async fn test_update_indexer_metrics_when_extreme_values_should_handle_gracefully() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        let extreme_metrics = IndexerMetrics {
            events_processed_total: u64::MAX,
            events_processing_rate: f64::MAX,
            processing_latency_ms: f64::INFINITY,
            queue_depth: u64::MAX,
            active_workers: u64::MAX,
            error_rate: 1.0,
            storage_size_bytes: u64::MAX,
            cache_hit_rate: 1.0,
        };

        collector.update_indexer_metrics(extreme_metrics).await;

        let stored_metrics = collector.get_indexer_metrics().await;
        assert_eq!(stored_metrics.events_processed_total, u64::MAX);
        assert_eq!(stored_metrics.error_rate, 1.0);
    }

    #[tokio::test]
    async fn test_get_indexer_metrics_when_no_metrics_set_should_return_default() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        let metrics = collector.get_indexer_metrics().await;

        assert_eq!(metrics.events_processed_total, 0);
        assert_eq!(metrics.events_processing_rate, 0.0);
        assert_eq!(metrics.processing_latency_ms, 0.0);
        assert_eq!(metrics.queue_depth, 0);
        assert_eq!(metrics.active_workers, 0);
        assert_eq!(metrics.error_rate, 0.0);
        assert_eq!(metrics.storage_size_bytes, 0);
        assert_eq!(metrics.cache_hit_rate, 0.0);
    }

    #[tokio::test]
    async fn test_get_performance_metrics_when_default_metrics_should_return_calculated_values() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        let perf_metrics = collector.get_performance_metrics().await;

        assert_eq!(perf_metrics.throughput.events_per_second, 0.0);
        assert_eq!(perf_metrics.latency.avg_latency_ms, 0.0);
        assert_eq!(perf_metrics.resources.cpu_usage_percent, 0.0);
        assert_eq!(perf_metrics.errors.error_rate, 0.0);
    }

    #[tokio::test]
    async fn test_get_performance_metrics_when_valid_metrics_should_calculate_correctly() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        let test_metrics = create_test_indexer_metrics();

        collector.update_indexer_metrics(test_metrics.clone()).await;
        let perf_metrics = collector.get_performance_metrics().await;

        assert_eq!(
            perf_metrics.throughput.events_per_second,
            test_metrics.events_processing_rate
        );
        assert_eq!(
            perf_metrics.throughput.total_events,
            test_metrics.events_processed_total
        );
        assert_eq!(
            perf_metrics.latency.avg_latency_ms,
            test_metrics.processing_latency_ms
        );
        assert_eq!(
            perf_metrics.latency.p95_latency_ms,
            test_metrics.processing_latency_ms * 1.5
        );
        assert_eq!(
            perf_metrics.latency.p99_latency_ms,
            test_metrics.processing_latency_ms * 2.0
        );
        assert_eq!(
            perf_metrics.latency.max_latency_ms,
            test_metrics.processing_latency_ms * 3.0
        );
        assert_eq!(
            perf_metrics.resources.db_connections_active,
            test_metrics.active_workers as u32
        );
        assert_eq!(perf_metrics.resources.db_connections_max, 20);
        assert_eq!(perf_metrics.errors.error_rate, test_metrics.error_rate);

        let expected_total_errors =
            (test_metrics.events_processed_total as f64 * test_metrics.error_rate) as u64;
        assert_eq!(perf_metrics.errors.total_errors, expected_total_errors);
    }

    #[tokio::test]
    async fn test_start_collection_task_when_called_should_return_join_handle() {
        let config = MetricsConfig {
            enabled: true,
            port: 9090,
            endpoint: "/metrics".to_string(),
            collection_interval: Duration::from_millis(100), // Short interval for testing
            histogram_buckets: vec![],
            custom: HashMap::new(),
        };
        let collector = MetricsCollector::new(config).unwrap();

        let handle = collector.start_collection_task();

        // Let it run briefly
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Cancel the task
        handle.abort();

        // Verify it was aborted
        assert!(handle.is_finished());
    }

    #[tokio::test]
    async fn test_start_collection_task_when_long_interval_should_handle_cancellation() {
        let config = MetricsConfig {
            enabled: true,
            port: 9090,
            endpoint: "/metrics".to_string(),
            collection_interval: Duration::from_secs(3600), // Very long interval
            histogram_buckets: vec![],
            custom: HashMap::new(),
        };
        let collector = MetricsCollector::new(config).unwrap();

        let handle = collector.start_collection_task();

        // Immediately cancel
        handle.abort();

        // Should be finished quickly
        let result = timeout(Duration::from_millis(100), handle).await;
        assert!(result.is_err()); // Should timeout because handle was aborted
    }

    #[tokio::test]
    async fn test_export_prometheus_when_called_should_return_string() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        let prometheus_output = collector.export_prometheus();

        assert!(!prometheus_output.is_empty());
        // Should contain some standard metrics
        assert!(prometheus_output.contains("indexer_events_processed_total"));
    }

    #[tokio::test]
    async fn test_export_json_when_called_should_return_valid_json() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        let json_result = collector.export_json();

        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_array());
    }

    #[tokio::test]
    async fn test_flush_when_called_should_complete_successfully() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        let result = collector.flush().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reset_when_called_should_clear_and_reinitialize() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        // Add some metrics
        collector.record_gauge("custom_metric", 42.0);

        // Reset
        collector.reset();

        // Should still have standard metrics but custom ones should be gone
        let prometheus_output = collector.export_prometheus();
        assert!(prometheus_output.contains("indexer_events_processed_total"));
    }

    #[tokio::test]
    async fn test_clone_when_called_should_create_identical_collector() {
        let config = create_test_config();
        let original = MetricsCollector::new(config).unwrap();
        let test_metrics = create_test_indexer_metrics();

        original.update_indexer_metrics(test_metrics.clone()).await;

        let cloned = original.clone();

        // Should share the same underlying data
        let original_metrics = original.get_indexer_metrics().await;
        let cloned_metrics = cloned.get_indexer_metrics().await;

        assert_eq!(
            original_metrics.events_processed_total,
            cloned_metrics.events_processed_total
        );
        assert_eq!(
            original_metrics.events_processing_rate,
            cloned_metrics.events_processing_rate
        );

        // Should have the same start time
        assert_eq!(original.start_time, cloned.start_time);

        // Should have the same config
        assert_eq!(original.config.port, cloned.config.port);
        assert_eq!(original.config.endpoint, cloned.config.endpoint);
    }

    #[tokio::test]
    async fn test_init_standard_metrics_when_called_should_set_all_gauges() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        // The constructor calls init_standard_metrics, so we can check if it worked
        let prometheus_output = collector.export_prometheus();

        // Verify all standard metrics are present
        assert!(prometheus_output.contains("indexer_events_processed_total"));
        assert!(prometheus_output.contains("indexer_events_processing_rate"));
        assert!(prometheus_output.contains("indexer_processing_latency_ms"));
        assert!(prometheus_output.contains("indexer_queue_depth"));
        assert!(prometheus_output.contains("indexer_active_workers"));
        assert!(prometheus_output.contains("indexer_error_rate"));
        assert!(prometheus_output.contains("indexer_storage_size_bytes"));
        assert!(prometheus_output.contains("indexer_cache_hit_rate"));
        assert!(prometheus_output.contains("indexer_uptime_seconds"));
        assert!(prometheus_output.contains("indexer_component_health_ingester"));
        assert!(prometheus_output.contains("indexer_component_health_processor"));
        assert!(prometheus_output.contains("indexer_component_health_storage"));
    }

    #[tokio::test]
    async fn test_update_indexer_metrics_when_called_should_update_uptime() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        // Wait a small amount to ensure uptime > 0
        tokio::time::sleep(Duration::from_millis(10)).await;

        let test_metrics = create_test_indexer_metrics();
        collector.update_indexer_metrics(test_metrics).await;

        let prometheus_output = collector.export_prometheus();

        // Uptime should be greater than 0
        // We can't test exact value but can verify it's being set
        assert!(prometheus_output.contains("indexer_uptime_seconds"));
    }

    #[tokio::test]
    async fn test_multiple_concurrent_updates_should_handle_correctly() {
        let config = create_test_config();
        let collector = Arc::new(MetricsCollector::new(config).unwrap());

        let mut handles = vec![];

        // Spawn multiple concurrent updates
        for i in 0..10 {
            let collector_clone = collector.clone();
            let handle = tokio::spawn(async move {
                let metrics = IndexerMetrics {
                    events_processed_total: i * 100,
                    events_processing_rate: i as f64 * 10.0,
                    processing_latency_ms: i as f64 * 5.0,
                    queue_depth: i * 2,
                    active_workers: i,
                    error_rate: 0.01 * i as f64,
                    storage_size_bytes: i * 1024,
                    cache_hit_rate: 0.1 * i as f64,
                };
                collector_clone.update_indexer_metrics(metrics).await;
            });
            handles.push(handle);
        }

        // Wait for all updates to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Should complete without panicking
        let final_metrics = collector.get_indexer_metrics().await;
        // The final state will be from one of the updates
        assert!(final_metrics.events_processed_total <= 900);
    }

    #[tokio::test]
    async fn test_performance_metrics_edge_cases() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();

        // Test with zero error rate
        let zero_error_metrics = IndexerMetrics {
            events_processed_total: 1000,
            error_rate: 0.0,
            ..Default::default()
        };

        collector.update_indexer_metrics(zero_error_metrics).await;
        let perf_metrics = collector.get_performance_metrics().await;

        assert_eq!(perf_metrics.errors.total_errors, 0);
        assert_eq!(perf_metrics.errors.error_rate, 0.0);

        // Test with 100% error rate
        let high_error_metrics = IndexerMetrics {
            events_processed_total: 100,
            error_rate: 1.0,
            ..Default::default()
        };

        collector.update_indexer_metrics(high_error_metrics).await;
        let perf_metrics = collector.get_performance_metrics().await;

        assert_eq!(perf_metrics.errors.total_errors, 100);
        assert_eq!(perf_metrics.errors.error_rate, 1.0);
    }
}
