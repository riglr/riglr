//! Metrics collection and monitoring

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::MetricsConfig;
use crate::error::{IndexerError, IndexerResult, MetricsError};

pub mod collector;
pub mod prometheus;

pub use collector::MetricsCollector;

/// Metric types
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MetricType {
    /// Counter metric that only increases
    Counter,
    /// Gauge metric that can go up or down
    Gauge,
    /// Histogram for observing value distributions
    Histogram,
    /// Summary for calculating quantiles
    Summary,
}

/// Metric value
#[derive(Debug, Clone, Serialize)]
pub enum MetricValue {
    /// Counter value
    Counter(u64),
    /// Gauge value
    Gauge(f64),
    /// Histogram observations
    Histogram(Vec<f64>),
    /// Summary statistics
    Summary(SummaryValue),
}

/// Summary metric value
#[derive(Debug, Clone, Serialize)]
pub struct SummaryValue {
    /// Number of observations
    pub count: u64,
    /// Sum of all observations
    pub sum: f64,
    /// Quantile values (e.g., "0.5" -> median)
    pub quantiles: HashMap<String, f64>,
}

/// Individual metric data
#[derive(Debug, Clone, Serialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Type of metric
    pub metric_type: MetricType,
    /// Current metric value
    pub value: MetricValue,
    /// Metric labels for dimensional data
    pub labels: HashMap<String, String>,
    /// Timestamp when metric was recorded
    pub timestamp: DateTime<Utc>,
    /// Help text describing the metric
    pub help: String,
}

/// Indexer-specific metrics
#[derive(Debug, Clone, Default)]
pub struct IndexerMetrics {
    /// Total events processed
    pub events_processed_total: u64,
    /// Events processing rate (events/second)
    pub events_processing_rate: f64,
    /// Average processing latency in milliseconds
    pub processing_latency_ms: f64,
    /// Current queue depth
    pub queue_depth: u64,
    /// Active workers
    pub active_workers: u64,
    /// Error rate
    pub error_rate: f64,
    /// Storage size in bytes
    pub storage_size_bytes: u64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
}

/// Performance metrics summary
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Throughput metrics
    pub throughput: ThroughputMetrics,
    /// Latency metrics
    pub latency: LatencyMetrics,
    /// Resource utilization metrics
    pub resources: ResourceMetrics,
    /// Error metrics
    pub errors: ErrorMetrics,
}

/// Throughput metrics
#[derive(Debug, Clone, Default)]
pub struct ThroughputMetrics {
    /// Events per second (current)
    pub events_per_second: f64,
    /// Events per second (average)
    pub avg_events_per_second: f64,
    /// Peak events per second
    pub peak_events_per_second: f64,
    /// Total events processed
    pub total_events: u64,
}

/// Latency metrics
#[derive(Debug, Clone, Default)]
pub struct LatencyMetrics {
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// 50th percentile latency
    pub p50_latency_ms: f64,
    /// 95th percentile latency
    pub p95_latency_ms: f64,
    /// 99th percentile latency
    pub p99_latency_ms: f64,
    /// Maximum latency
    pub max_latency_ms: f64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Default)]
pub struct ResourceMetrics {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Active database connections
    pub db_connections_active: u32,
    /// Maximum database connections
    pub db_connections_max: u32,
}

/// Error metrics
#[derive(Debug, Clone, Default)]
pub struct ErrorMetrics {
    /// Total error count
    pub total_errors: u64,
    /// Error rate (errors per total events)
    pub error_rate: f64,
    /// Errors by category
    pub errors_by_category: HashMap<String, u64>,
    /// Recent error rate (last 5 minutes)
    pub recent_error_rate: f64,
}

/// Metrics registry for storing and managing metrics
#[derive(Debug)]
pub struct MetricsRegistry {
    /// Stored metrics
    metrics: Arc<DashMap<String, Metric>>,
    /// Configuration
    #[allow(dead_code)]
    config: MetricsConfig,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            metrics: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Register a metric
    pub fn register_metric(&self, metric: Metric) -> IndexerResult<()> {
        self.metrics.insert(metric.name.clone(), metric);
        Ok(())
    }

    /// Get a metric by name
    pub fn get_metric(&self, name: &str) -> Option<Metric> {
        self.metrics.get(name).map(|entry| entry.value().clone())
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> Vec<Metric> {
        self.metrics
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Update counter metric
    pub fn increment_counter(&self, name: &str, value: u64) -> IndexerResult<()> {
        match self.metrics.get_mut(name) {
            Some(mut metric) => {
                if let MetricValue::Counter(ref mut counter) = metric.value {
                    *counter += value;
                    metric.timestamp = Utc::now();
                } else {
                    return Err(IndexerError::Metrics(MetricsError::InvalidValue {
                        value: format!("Expected counter, found {:?}", metric.metric_type),
                    }));
                }
            }
            None => {
                // Create new counter
                let metric = Metric {
                    name: name.to_string(),
                    metric_type: MetricType::Counter,
                    value: MetricValue::Counter(value),
                    labels: HashMap::new(),
                    timestamp: Utc::now(),
                    help: format!("Counter metric: {}", name),
                };
                self.metrics.insert(name.to_string(), metric);
            }
        }

        Ok(())
    }

    /// Update gauge metric
    pub fn set_gauge(&self, name: &str, value: f64) -> IndexerResult<()> {
        let metric = Metric {
            name: name.to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(value),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            help: format!("Gauge metric: {}", name),
        };

        self.metrics.insert(name.to_string(), metric);
        Ok(())
    }

    /// Record histogram value
    pub fn record_histogram(&self, name: &str, value: f64) -> IndexerResult<()> {
        match self.metrics.get_mut(name) {
            Some(mut metric) => {
                if let MetricValue::Histogram(ref mut values) = metric.value {
                    values.push(value);
                    // Keep only recent values to prevent unbounded growth
                    if values.len() > 10000 {
                        values.drain(0..5000);
                    }
                    metric.timestamp = Utc::now();
                } else {
                    return Err(IndexerError::Metrics(MetricsError::InvalidValue {
                        value: format!("Expected histogram, found {:?}", metric.metric_type),
                    }));
                }
            }
            None => {
                // Create new histogram
                let metric = Metric {
                    name: name.to_string(),
                    metric_type: MetricType::Histogram,
                    value: MetricValue::Histogram(vec![value]),
                    labels: HashMap::new(),
                    timestamp: Utc::now(),
                    help: format!("Histogram metric: {}", name),
                };
                self.metrics.insert(name.to_string(), metric);
            }
        }

        Ok(())
    }

    /// Clear all metrics
    pub fn clear(&self) {
        self.metrics.clear();
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::default();

        for entry in self.metrics.iter() {
            let metric = entry.value();
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
            output.push_str(&format!(
                "# TYPE {} {}\n",
                metric.name,
                match metric.metric_type {
                    MetricType::Counter => "counter",
                    MetricType::Gauge => "gauge",
                    MetricType::Histogram => "histogram",
                    MetricType::Summary => "summary",
                }
            ));

            match &metric.value {
                MetricValue::Counter(value) => {
                    output.push_str(&format!("{} {}\n", metric.name, value));
                }
                MetricValue::Gauge(value) => {
                    output.push_str(&format!("{} {}\n", metric.name, value));
                }
                MetricValue::Histogram(values) => {
                    // Simple histogram implementation
                    let buckets = &[
                        0.005,
                        0.01,
                        0.025,
                        0.05,
                        0.1,
                        0.25,
                        0.5,
                        1.0,
                        2.5,
                        5.0,
                        10.0,
                        f64::INFINITY,
                    ];
                    let mut bucket_counts = vec![0; buckets.len()];
                    let mut sum = 0.0;

                    for &value in values {
                        sum += value;
                        for (i, &bucket) in buckets.iter().enumerate() {
                            if value <= bucket {
                                bucket_counts[i] += 1;
                            }
                        }
                    }

                    for (i, &bucket) in buckets.iter().enumerate() {
                        let le = if bucket == f64::INFINITY {
                            "+Inf"
                        } else {
                            &bucket.to_string()
                        };
                        output.push_str(&format!(
                            "{}_bucket{{le=\"{}\"}} {}\n",
                            metric.name, le, bucket_counts[i]
                        ));
                    }
                    output.push_str(&format!("{}_count {}\n", metric.name, values.len()));
                    output.push_str(&format!("{}_sum {}\n", metric.name, sum));
                }
                MetricValue::Summary(_) => {
                    // Summary implementation would go here
                }
            }
            output.push('\n');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_config() -> MetricsConfig {
        use std::collections::HashMap;
        use std::time::Duration;

        MetricsConfig {
            enabled: true,
            port: 8080,
            endpoint: "/metrics".to_string(),
            collection_interval: Duration::from_secs(10),
            histogram_buckets: vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
            custom: HashMap::new(),
        }
    }

    #[test]
    fn test_metric_type_debug_clone_eq() {
        let counter = MetricType::Counter;
        let counter_clone = counter.clone();
        assert_eq!(counter, counter_clone);
        assert_eq!(format!("{:?}", counter), "Counter");

        let gauge = MetricType::Gauge;
        let histogram = MetricType::Histogram;
        let summary = MetricType::Summary;

        assert_ne!(counter, gauge);
        assert_ne!(gauge, histogram);
        assert_ne!(histogram, summary);
    }

    #[test]
    fn test_summary_value_creation() {
        let mut quantiles = HashMap::new();
        quantiles.insert("0.5".to_string(), 100.0);
        quantiles.insert("0.95".to_string(), 200.0);

        let summary = SummaryValue {
            count: 1000,
            sum: 50000.0,
            quantiles,
        };

        assert_eq!(summary.count, 1000);
        assert_eq!(summary.sum, 50000.0);
        assert_eq!(summary.quantiles.get("0.5"), Some(&100.0));
        assert_eq!(summary.quantiles.get("0.95"), Some(&200.0));
    }

    #[test]
    fn test_metric_value_variants() {
        let counter = MetricValue::Counter(42);
        let gauge = MetricValue::Gauge(3.14);
        let histogram = MetricValue::Histogram(vec![1.0, 2.0, 3.0]);
        let summary = MetricValue::Summary(SummaryValue {
            count: 10,
            sum: 55.0,
            quantiles: HashMap::new(),
        });

        // Test debug formatting
        assert!(format!("{:?}", counter).contains("Counter(42)"));
        assert!(format!("{:?}", gauge).contains("Gauge(3.14)"));
        assert!(format!("{:?}", histogram).contains("Histogram"));
        assert!(format!("{:?}", summary).contains("Summary"));
    }

    #[test]
    fn test_metric_creation() {
        let timestamp = Utc::now();
        let mut labels = HashMap::new();
        labels.insert("service".to_string(), "indexer".to_string());

        let metric = Metric {
            name: "test_counter".to_string(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(100),
            labels: labels.clone(),
            timestamp,
            help: "Test counter metric".to_string(),
        };

        assert_eq!(metric.name, "test_counter");
        assert_eq!(metric.metric_type, MetricType::Counter);
        assert_eq!(metric.labels.get("service"), Some(&"indexer".to_string()));
        assert_eq!(metric.help, "Test counter metric");
    }

    #[test]
    fn test_indexer_metrics_default() {
        let metrics = IndexerMetrics::default();
        assert_eq!(metrics.events_processed_total, 0);
        assert_eq!(metrics.events_processing_rate, 0.0);
        assert_eq!(metrics.processing_latency_ms, 0.0);
        assert_eq!(metrics.queue_depth, 0);
        assert_eq!(metrics.active_workers, 0);
        assert_eq!(metrics.error_rate, 0.0);
        assert_eq!(metrics.storage_size_bytes, 0);
        assert_eq!(metrics.cache_hit_rate, 0.0);
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.throughput.events_per_second, 0.0);
        assert_eq!(metrics.latency.avg_latency_ms, 0.0);
        assert_eq!(metrics.resources.cpu_usage_percent, 0.0);
        assert_eq!(metrics.errors.total_errors, 0);
    }

    #[test]
    fn test_throughput_metrics_default() {
        let metrics = ThroughputMetrics::default();
        assert_eq!(metrics.events_per_second, 0.0);
        assert_eq!(metrics.avg_events_per_second, 0.0);
        assert_eq!(metrics.peak_events_per_second, 0.0);
        assert_eq!(metrics.total_events, 0);
    }

    #[test]
    fn test_latency_metrics_default() {
        let metrics = LatencyMetrics::default();
        assert_eq!(metrics.avg_latency_ms, 0.0);
        assert_eq!(metrics.p50_latency_ms, 0.0);
        assert_eq!(metrics.p95_latency_ms, 0.0);
        assert_eq!(metrics.p99_latency_ms, 0.0);
        assert_eq!(metrics.max_latency_ms, 0.0);
    }

    #[test]
    fn test_resource_metrics_default() {
        let metrics = ResourceMetrics::default();
        assert_eq!(metrics.cpu_usage_percent, 0.0);
        assert_eq!(metrics.memory_usage_bytes, 0);
        assert_eq!(metrics.memory_usage_percent, 0.0);
        assert_eq!(metrics.db_connections_active, 0);
        assert_eq!(metrics.db_connections_max, 0);
    }

    #[test]
    fn test_error_metrics_default() {
        let metrics = ErrorMetrics::default();
        assert_eq!(metrics.total_errors, 0);
        assert_eq!(metrics.error_rate, 0.0);
        assert!(metrics.errors_by_category.is_empty());
        assert_eq!(metrics.recent_error_rate, 0.0);
    }

    #[test]
    fn test_metrics_registry_new() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);
        assert_eq!(registry.metrics.len(), 0);
    }

    #[test]
    fn test_register_metric_success() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        let metric = Metric {
            name: "test_metric".to_string(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(0),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            help: "Test metric".to_string(),
        };

        let result = registry.register_metric(metric.clone());
        assert!(result.is_ok());
        assert_eq!(registry.metrics.len(), 1);

        let retrieved = registry.get_metric("test_metric");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_metric");
    }

    #[test]
    fn test_get_metric_not_found() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        let result = registry.get_metric("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_all_metrics_empty() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        let metrics = registry.get_all_metrics();
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_get_all_metrics_with_data() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        let metric1 = Metric {
            name: "metric1".to_string(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(1),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            help: "Metric 1".to_string(),
        };

        let metric2 = Metric {
            name: "metric2".to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(2.0),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            help: "Metric 2".to_string(),
        };

        registry.register_metric(metric1).unwrap();
        registry.register_metric(metric2).unwrap();

        let metrics = registry.get_all_metrics();
        assert_eq!(metrics.len(), 2);
    }

    #[test]
    fn test_increment_counter_existing() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        // Create initial counter
        let result = registry.increment_counter("test_counter", 5);
        assert!(result.is_ok());

        // Increment existing counter
        let result = registry.increment_counter("test_counter", 3);
        assert!(result.is_ok());

        let metric = registry.get_metric("test_counter").unwrap();
        if let MetricValue::Counter(value) = metric.value {
            assert_eq!(value, 8);
        } else {
            panic!("Expected counter value");
        }
    }

    #[test]
    fn test_increment_counter_new() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        let result = registry.increment_counter("new_counter", 10);
        assert!(result.is_ok());

        let metric = registry.get_metric("new_counter").unwrap();
        assert_eq!(metric.name, "new_counter");
        assert_eq!(metric.metric_type, MetricType::Counter);
        if let MetricValue::Counter(value) = metric.value {
            assert_eq!(value, 10);
        } else {
            panic!("Expected counter value");
        }
        assert_eq!(metric.help, "Counter metric: new_counter");
    }

    #[test]
    fn test_increment_counter_wrong_type() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        // First register a gauge
        let gauge_metric = Metric {
            name: "gauge_metric".to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(1.0),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            help: "Test gauge".to_string(),
        };
        registry.register_metric(gauge_metric).unwrap();

        // Try to increment as counter
        let result = registry.increment_counter("gauge_metric", 1);
        assert!(result.is_err());

        if let Err(IndexerError::Metrics(MetricsError::InvalidValue { value })) = result {
            assert!(value.contains("Expected counter, found"));
        } else {
            panic!("Expected InvalidValue error");
        }
    }

    #[test]
    fn test_set_gauge() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        let result = registry.set_gauge("test_gauge", 42.5);
        assert!(result.is_ok());

        let metric = registry.get_metric("test_gauge").unwrap();
        assert_eq!(metric.name, "test_gauge");
        assert_eq!(metric.metric_type, MetricType::Gauge);
        if let MetricValue::Gauge(value) = metric.value {
            assert_eq!(value, 42.5);
        } else {
            panic!("Expected gauge value");
        }
        assert_eq!(metric.help, "Gauge metric: test_gauge");
    }

    #[test]
    fn test_set_gauge_overwrite() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        // Set initial value
        registry.set_gauge("test_gauge", 10.0).unwrap();

        // Overwrite with new value
        registry.set_gauge("test_gauge", 20.0).unwrap();

        let metric = registry.get_metric("test_gauge").unwrap();
        if let MetricValue::Gauge(value) = metric.value {
            assert_eq!(value, 20.0);
        } else {
            panic!("Expected gauge value");
        }
    }

    #[test]
    fn test_record_histogram_new() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        let result = registry.record_histogram("test_histogram", 1.5);
        assert!(result.is_ok());

        let metric = registry.get_metric("test_histogram").unwrap();
        assert_eq!(metric.name, "test_histogram");
        assert_eq!(metric.metric_type, MetricType::Histogram);
        if let MetricValue::Histogram(values) = metric.value {
            assert_eq!(values.len(), 1);
            assert_eq!(values[0], 1.5);
        } else {
            panic!("Expected histogram value");
        }
        assert_eq!(metric.help, "Histogram metric: test_histogram");
    }

    #[test]
    fn test_record_histogram_existing() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        // Record first value
        registry.record_histogram("test_histogram", 1.0).unwrap();

        // Record second value
        registry.record_histogram("test_histogram", 2.0).unwrap();

        let metric = registry.get_metric("test_histogram").unwrap();
        if let MetricValue::Histogram(values) = metric.value {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], 1.0);
            assert_eq!(values[1], 2.0);
        } else {
            panic!("Expected histogram value");
        }
    }

    #[test]
    fn test_record_histogram_wrong_type() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        // First register a counter
        let counter_metric = Metric {
            name: "counter_metric".to_string(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(1),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            help: "Test counter".to_string(),
        };
        registry.register_metric(counter_metric).unwrap();

        // Try to record histogram value
        let result = registry.record_histogram("counter_metric", 1.0);
        assert!(result.is_err());

        if let Err(IndexerError::Metrics(MetricsError::InvalidValue { value })) = result {
            assert!(value.contains("Expected histogram, found"));
        } else {
            panic!("Expected InvalidValue error");
        }
    }

    #[test]
    fn test_record_histogram_limit_growth() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        // Record many values to trigger growth limiting
        for i in 0..10001 {
            registry
                .record_histogram("large_histogram", i as f64)
                .unwrap();
        }

        let metric = registry.get_metric("large_histogram").unwrap();
        if let MetricValue::Histogram(values) = metric.value {
            // Should be trimmed to prevent unbounded growth
            assert!(values.len() <= 10000);
            // Should contain the most recent values
            assert!(values.contains(&10000.0));
        } else {
            panic!("Expected histogram value");
        }
    }

    #[test]
    fn test_clear() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        // Add some metrics
        registry.increment_counter("counter", 1).unwrap();
        registry.set_gauge("gauge", 1.0).unwrap();
        assert_eq!(registry.metrics.len(), 2);

        // Clear all metrics
        registry.clear();
        assert_eq!(registry.metrics.len(), 0);
        assert!(registry.get_all_metrics().is_empty());
    }

    #[test]
    fn test_export_prometheus_empty() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        let output = registry.export_prometheus();
        assert_eq!(output, "");
    }

    #[test]
    fn test_export_prometheus_counter() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        registry.increment_counter("test_counter", 42).unwrap();

        let output = registry.export_prometheus();
        assert!(output.contains("# HELP test_counter Counter metric: test_counter"));
        assert!(output.contains("# TYPE test_counter counter"));
        assert!(output.contains("test_counter 42"));
    }

    #[test]
    fn test_export_prometheus_gauge() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        registry.set_gauge("test_gauge", 3.14).unwrap();

        let output = registry.export_prometheus();
        assert!(output.contains("# HELP test_gauge Gauge metric: test_gauge"));
        assert!(output.contains("# TYPE test_gauge gauge"));
        assert!(output.contains("test_gauge 3.14"));
    }

    #[test]
    fn test_export_prometheus_histogram() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        registry.record_histogram("test_histogram", 0.1).unwrap();
        registry.record_histogram("test_histogram", 0.5).unwrap();
        registry.record_histogram("test_histogram", 1.0).unwrap();

        let output = registry.export_prometheus();
        assert!(output.contains("# HELP test_histogram Histogram metric: test_histogram"));
        assert!(output.contains("# TYPE test_histogram histogram"));
        assert!(output.contains("test_histogram_bucket{le=\"0.005\"}"));
        assert!(output.contains("test_histogram_bucket{le=\"+Inf\"}"));
        assert!(output.contains("test_histogram_count 3"));
        assert!(output.contains("test_histogram_sum 1.6"));
    }

    #[test]
    fn test_export_prometheus_histogram_buckets() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        // Add values to test different buckets
        registry.record_histogram("bucket_test", 0.001).unwrap(); // Should be in 0.005 bucket
        registry.record_histogram("bucket_test", 0.02).unwrap(); // Should be in 0.025 bucket
        registry.record_histogram("bucket_test", 0.1).unwrap(); // Should be in 0.1 bucket
        registry.record_histogram("bucket_test", 5.0).unwrap(); // Should be in 5.0 bucket
        registry.record_histogram("bucket_test", 15.0).unwrap(); // Should be in +Inf bucket

        let output = registry.export_prometheus();

        // Check that buckets are present
        assert!(output.contains("bucket_test_bucket{le=\"0.005\"} 1"));
        assert!(output.contains("bucket_test_bucket{le=\"0.025\"} 2"));
        assert!(output.contains("bucket_test_bucket{le=\"0.1\"} 3"));
        assert!(output.contains("bucket_test_bucket{le=\"5.0\"} 4"));
        assert!(output.contains("bucket_test_bucket{le=\"+Inf\"} 5"));
        assert!(output.contains("bucket_test_count 5"));
        assert!(output.contains("bucket_test_sum 20.121"));
    }

    #[test]
    fn test_export_prometheus_summary() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        let summary_metric = Metric {
            name: "test_summary".to_string(),
            metric_type: MetricType::Summary,
            value: MetricValue::Summary(SummaryValue {
                count: 100,
                sum: 500.0,
                quantiles: HashMap::new(),
            }),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            help: "Test summary".to_string(),
        };

        registry.register_metric(summary_metric).unwrap();

        let output = registry.export_prometheus();
        assert!(output.contains("# HELP test_summary Test summary"));
        assert!(output.contains("# TYPE test_summary summary"));
        // Summary implementation is empty in the current code, so we just verify the headers
    }

    #[test]
    fn test_export_prometheus_multiple_metrics() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        registry.increment_counter("requests_total", 100).unwrap();
        registry.set_gauge("memory_usage", 75.5).unwrap();
        registry.record_histogram("request_duration", 0.1).unwrap();

        let output = registry.export_prometheus();

        // Should contain all three metrics
        assert!(output.contains("requests_total 100"));
        assert!(output.contains("memory_usage 75.5"));
        assert!(output.contains("request_duration_count 1"));

        // Should have proper separators
        assert!(output.matches('\n').count() > 6); // Multiple newlines for separation
    }

    #[test]
    fn test_prometheus_export_metric_type_formatting() {
        let config = create_test_config();
        let registry = MetricsRegistry::new(config);

        // Test all metric type formats
        registry.increment_counter("counter_metric", 1).unwrap();
        registry.set_gauge("gauge_metric", 1.0).unwrap();
        registry.record_histogram("histogram_metric", 1.0).unwrap();

        let summary_metric = Metric {
            name: "summary_metric".to_string(),
            metric_type: MetricType::Summary,
            value: MetricValue::Summary(SummaryValue {
                count: 1,
                sum: 1.0,
                quantiles: HashMap::new(),
            }),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            help: "Summary".to_string(),
        };
        registry.register_metric(summary_metric).unwrap();

        let output = registry.export_prometheus();

        assert!(output.contains("# TYPE counter_metric counter"));
        assert!(output.contains("# TYPE gauge_metric gauge"));
        assert!(output.contains("# TYPE histogram_metric histogram"));
        assert!(output.contains("# TYPE summary_metric summary"));
    }
}
