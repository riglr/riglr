use metrics::{counter, gauge, histogram};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Stream metrics collector
pub struct StreamMetrics {
    /// Start time
    start_time: SystemTime,
    /// Total events processed
    events_processed: Arc<RwLock<u64>>,
    /// Events by type
    events_by_type: Arc<RwLock<HashMap<String, u64>>>,
    /// Error count
    error_count: Arc<RwLock<u64>>,
    /// Performance stats
    performance_stats: Arc<RwLock<PerformanceStats>>,
}

/// Performance statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// Average events per second
    pub events_per_second: f64,
    /// Peak events per second
    pub peak_events_per_second: f64,
    /// Average processing latency (ms)
    pub avg_latency_ms: f64,
    /// P50 latency (ms)
    pub p50_latency_ms: f64,
    /// P95 latency (ms)
    pub p95_latency_ms: f64,
    /// P99 latency (ms)
    pub p99_latency_ms: f64,
}

impl Default for StreamMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamMetrics {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            events_processed: Arc::new(RwLock::new(0)),
            events_by_type: Arc::new(RwLock::new(HashMap::new())),
            error_count: Arc::new(RwLock::new(0)),
            performance_stats: Arc::new(RwLock::new(PerformanceStats::default())),
        }
    }

    /// Record an event
    pub async fn record_event(&self, event_type: &str, latency_ms: f64) {
        // Increment total counter
        let mut count = self.events_processed.write().await;
        *count += 1;
        counter!("riglr_streams_events_total").increment(1);

        // Increment by type
        let mut by_type = self.events_by_type.write().await;
        *by_type.entry(event_type.to_string()).or_insert(0) += 1;
        counter!("riglr_streams_events_by_type", "type" => event_type.to_string()).increment(1);

        // Record latency
        histogram!("riglr_streams_event_latency_ms").record(latency_ms);

        // Update performance stats
        let mut stats = self.performance_stats.write().await;
        // Simplified update - in production would use proper percentile tracking
        stats.avg_latency_ms = (stats.avg_latency_ms + latency_ms) / 2.0;
    }

    /// Record an error
    pub async fn record_error(&self, error_type: &str) {
        let mut count = self.error_count.write().await;
        *count += 1;
        counter!("riglr_streams_errors_total", "type" => error_type.to_string()).increment(1);
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> MetricsSnapshot {
        let uptime = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or(Duration::ZERO);

        let events_processed = *self.events_processed.read().await;
        let events_by_type = self.events_by_type.read().await.clone();
        let error_count = *self.error_count.read().await;
        let performance_stats = self.performance_stats.read().await.clone();

        // Calculate events per second
        let events_per_second = if uptime.as_secs() > 0 {
            events_processed as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };

        // Update gauge metrics
        gauge!("riglr_streams_events_per_second").set(events_per_second);
        gauge!("riglr_streams_uptime_seconds").set(uptime.as_secs() as f64);

        MetricsSnapshot {
            uptime,
            events_processed,
            events_by_type,
            error_count,
            events_per_second,
            performance_stats,
        }
    }
}

/// Metrics snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Uptime
    pub uptime: Duration,
    /// Total events processed
    pub events_processed: u64,
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
    /// Error count
    pub error_count: u64,
    /// Events per second
    pub events_per_second: f64,
    /// Performance stats
    pub performance_stats: PerformanceStats,
}

/// Metrics collector for all streams
pub struct MetricsCollector {
    /// Metrics by stream
    stream_metrics: Arc<RwLock<HashMap<String, StreamMetrics>>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create new collector
    pub fn new() -> Self {
        Self {
            stream_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create metrics for a stream
    pub async fn get_stream_metrics(&self, stream_name: &str) -> StreamMetrics {
        let mut metrics = self.stream_metrics.write().await;
        metrics
            .entry(stream_name.to_string())
            .or_insert_with(StreamMetrics::new)
            .clone()
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> HashMap<String, MetricsSnapshot> {
        let mut all_metrics = HashMap::new();
        let metrics = self.stream_metrics.read().await;

        for (name, stream_metrics) in metrics.iter() {
            all_metrics.insert(name.clone(), stream_metrics.get_metrics().await);
        }

        all_metrics
    }
}

impl Clone for StreamMetrics {
    fn clone(&self) -> Self {
        Self {
            start_time: self.start_time,
            events_processed: self.events_processed.clone(),
            events_by_type: self.events_by_type.clone(),
            error_count: self.error_count.clone(),
            performance_stats: self.performance_stats.clone(),
        }
    }
}
