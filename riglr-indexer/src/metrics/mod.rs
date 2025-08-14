//! Metrics collection and monitoring

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::config::MetricsConfig;
use crate::error::{IndexerError, IndexerResult, MetricsError};

pub mod collector;
pub mod prometheus;

pub use collector::MetricsCollector;

/// Metric types
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Metric value
#[derive(Debug, Clone, Serialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Summary(SummaryValue),
}

/// Summary metric value
#[derive(Debug, Clone, Serialize)]
pub struct SummaryValue {
    pub count: u64,
    pub sum: f64,
    pub quantiles: HashMap<f64, f64>,
}

/// Individual metric data
#[derive(Debug, Clone, Serialize)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub help: String,
}

/// Indexer-specific metrics
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
pub struct MetricsRegistry {
    /// Stored metrics
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
    /// Configuration
    config: MetricsConfig,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register a metric
    pub fn register_metric(&self, metric: Metric) -> IndexerResult<()> {
        let mut metrics = self.metrics.write();
        metrics.insert(metric.name.clone(), metric);
        Ok(())
    }

    /// Get a metric by name
    pub fn get_metric(&self, name: &str) -> Option<Metric> {
        let metrics = self.metrics.read();
        metrics.get(name).cloned()
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> Vec<Metric> {
        let metrics = self.metrics.read();
        metrics.values().cloned().collect()
    }

    /// Update counter metric
    pub fn increment_counter(&self, name: &str, value: u64) -> IndexerResult<()> {
        let mut metrics = self.metrics.write();
        
        match metrics.get_mut(name) {
            Some(metric) => {
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
                metrics.insert(name.to_string(), metric);
            }
        }

        Ok(())
    }

    /// Update gauge metric
    pub fn set_gauge(&self, name: &str, value: f64) -> IndexerResult<()> {
        let mut metrics = self.metrics.write();
        
        let metric = Metric {
            name: name.to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(value),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            help: format!("Gauge metric: {}", name),
        };
        
        metrics.insert(name.to_string(), metric);
        Ok(())
    }

    /// Record histogram value
    pub fn record_histogram(&self, name: &str, value: f64) -> IndexerResult<()> {
        let mut metrics = self.metrics.write();
        
        match metrics.get_mut(name) {
            Some(metric) => {
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
                metrics.insert(name.to_string(), metric);
            }
        }

        Ok(())
    }

    /// Clear all metrics
    pub fn clear(&self) {
        let mut metrics = self.metrics.write();
        metrics.clear();
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let metrics = self.metrics.read();
        let mut output = String::new();

        for metric in metrics.values() {
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
            output.push_str(&format!("# TYPE {} {}\n", metric.name, 
                match metric.metric_type {
                    MetricType::Counter => "counter",
                    MetricType::Gauge => "gauge",
                    MetricType::Histogram => "histogram",
                    MetricType::Summary => "summary",
                }));

            match &metric.value {
                MetricValue::Counter(value) => {
                    output.push_str(&format!("{} {}\n", metric.name, value));
                }
                MetricValue::Gauge(value) => {
                    output.push_str(&format!("{} {}\n", metric.name, value));
                }
                MetricValue::Histogram(values) => {
                    // Simple histogram implementation
                    let buckets = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, f64::INFINITY];
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
                        let le = if bucket == f64::INFINITY { "+Inf" } else { &bucket.to_string() };
                        output.push_str(&format!("{}_bucket{{le=\"{}\"}} {}\n", 
                                                metric.name, le, bucket_counts[i]));
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

impl Default for IndexerMetrics {
    fn default() -> Self {
        Self {
            events_processed_total: 0,
            events_processing_rate: 0.0,
            processing_latency_ms: 0.0,
            queue_depth: 0,
            active_workers: 0,
            error_rate: 0.0,
            storage_size_bytes: 0,
            cache_hit_rate: 0.0,
        }
    }
}