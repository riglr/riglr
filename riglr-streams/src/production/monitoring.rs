//! Metrics collection and performance monitoring for stream systems.
//!
//! This module provides comprehensive metrics collection for streaming applications,
//! including event counters, latency tracking, performance statistics, and throughput
//! measurements. It integrates with the `metrics` crate to provide both internal
//! monitoring and external observability through standard metrics formats.

use dashmap::DashMap;
use metrics::{counter, gauge, histogram};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Stream metrics collector
pub struct StreamMetrics {
    /// Start time
    start_time: SystemTime,
    /// Total events processed
    events_processed: Arc<AtomicU64>,
    /// Events by type
    events_by_type: Arc<DashMap<String, u64>>,
    /// Error count
    error_count: Arc<AtomicU64>,
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
        Self {
            start_time: SystemTime::now(),
            events_processed: Arc::new(AtomicU64::new(0)),
            events_by_type: Arc::new(DashMap::new()),
            error_count: Arc::new(AtomicU64::new(0)),
            performance_stats: Arc::new(RwLock::new(PerformanceStats::default())),
        }
    }
}

impl StreamMetrics {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an event
    pub async fn record_event(&self, event_type: &str, latency_ms: f64) {
        // Increment total counter
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        counter!("riglr_streams_events_total").increment(1);

        // Increment by type
        let mut entry = self.events_by_type.entry(event_type.to_string()).or_insert(0);
        *entry += 1;
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
        self.error_count.fetch_add(1, Ordering::Relaxed);
        counter!("riglr_streams_errors_total", "type" => error_type.to_string()).increment(1);
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> MetricsSnapshot {
        let uptime = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or(Duration::ZERO);

        let events_processed = self.events_processed.load(Ordering::Relaxed);
        let events_by_type = self.events_by_type.iter().map(|entry| (entry.key().clone(), *entry.value())).collect();
        let error_count = self.error_count.load(Ordering::Relaxed);
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
    stream_metrics: Arc<DashMap<String, StreamMetrics>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self {
            stream_metrics: Arc::new(DashMap::new()),
        }
    }
}

impl MetricsCollector {
    /// Create new collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create metrics for a stream
    pub async fn get_stream_metrics(&self, stream_name: &str) -> StreamMetrics {
        self.stream_metrics
            .entry(stream_name.to_string())
            .or_insert_with(StreamMetrics::new)
            .clone()
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> HashMap<String, MetricsSnapshot> {
        let mut all_metrics = HashMap::new();

        for entry in self.stream_metrics.iter() {
            let name = entry.key().clone();
            let snapshot = entry.value().get_metrics().await;
            all_metrics.insert(name, snapshot);
        }

        all_metrics
    }
}

impl Clone for StreamMetrics {
    fn clone(&self) -> Self {
        Self {
            start_time: self.start_time,
            events_processed: Arc::new(AtomicU64::new(self.events_processed.load(Ordering::Relaxed))),
            events_by_type: self.events_by_type.clone(),
            error_count: Arc::new(AtomicU64::new(self.error_count.load(Ordering::Relaxed))),
            performance_stats: self.performance_stats.clone(),
        }
    }
}
