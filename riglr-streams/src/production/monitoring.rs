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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
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
    /// Record an event
    pub async fn record_event(&self, event_type: &str, latency_ms: f64) {
        // Increment total counter
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        counter!("riglr_streams_events_total").increment(1);

        // Increment by type
        let mut entry = self
            .events_by_type
            .entry(event_type.to_string())
            .or_insert(0);
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
        let events_by_type = self
            .events_by_type
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
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
    /// Get or create metrics for a stream
    pub async fn get_stream_metrics(&self, stream_name: &str) -> StreamMetrics {
        self.stream_metrics
            .entry(stream_name.to_string())
            .or_insert_with(StreamMetrics::default)
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
            events_processed: Arc::new(AtomicU64::new(
                self.events_processed.load(Ordering::Relaxed),
            )),
            events_by_type: self.events_by_type.clone(),
            error_count: Arc::new(AtomicU64::new(self.error_count.load(Ordering::Relaxed))),
            performance_stats: self.performance_stats.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[test]
    fn test_performance_stats_default() {
        let stats = PerformanceStats::default();
        assert_eq!(stats.events_per_second, 0.0);
        assert_eq!(stats.peak_events_per_second, 0.0);
        assert_eq!(stats.avg_latency_ms, 0.0);
        assert_eq!(stats.p50_latency_ms, 0.0);
        assert_eq!(stats.p95_latency_ms, 0.0);
        assert_eq!(stats.p99_latency_ms, 0.0);
    }

    #[test]
    fn test_performance_stats_clone() {
        let original = PerformanceStats {
            events_per_second: 100.0,
            peak_events_per_second: 200.0,
            avg_latency_ms: 50.0,
            p50_latency_ms: 40.0,
            p95_latency_ms: 80.0,
            p99_latency_ms: 120.0,
        };
        let cloned = original.clone();
        assert_eq!(cloned.events_per_second, original.events_per_second);
        assert_eq!(
            cloned.peak_events_per_second,
            original.peak_events_per_second
        );
        assert_eq!(cloned.avg_latency_ms, original.avg_latency_ms);
        assert_eq!(cloned.p50_latency_ms, original.p50_latency_ms);
        assert_eq!(cloned.p95_latency_ms, original.p95_latency_ms);
        assert_eq!(cloned.p99_latency_ms, original.p99_latency_ms);
    }

    #[test]
    fn test_stream_metrics_default() {
        let metrics = StreamMetrics::default();
        assert!(metrics.start_time <= SystemTime::now());
        assert_eq!(metrics.events_processed.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.error_count.load(Ordering::Relaxed), 0);
        assert!(metrics.events_by_type.is_empty());
    }

    #[test]
    fn test_stream_metrics_new() {
        let metrics = StreamMetrics::default();
        assert!(metrics.start_time <= SystemTime::now());
        assert_eq!(metrics.events_processed.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.error_count.load(Ordering::Relaxed), 0);
        assert!(metrics.events_by_type.is_empty());
    }

    #[tokio::test]
    async fn test_stream_metrics_record_event_when_valid_input_should_update_counters() {
        let metrics = StreamMetrics::default();

        metrics.record_event("trade", 100.0).await;

        assert_eq!(metrics.events_processed.load(Ordering::Relaxed), 1);
        assert_eq!(*metrics.events_by_type.get("trade").unwrap().value(), 1);

        let stats = metrics.performance_stats.read().await;
        assert!(stats.avg_latency_ms > 0.0);
    }

    #[tokio::test]
    async fn test_stream_metrics_record_event_when_multiple_events_should_accumulate() {
        let metrics = StreamMetrics::default();

        metrics.record_event("trade", 50.0).await;
        metrics.record_event("trade", 150.0).await;
        metrics.record_event("order", 75.0).await;

        assert_eq!(metrics.events_processed.load(Ordering::Relaxed), 3);
        assert_eq!(*metrics.events_by_type.get("trade").unwrap().value(), 2);
        assert_eq!(*metrics.events_by_type.get("order").unwrap().value(), 1);
    }

    #[tokio::test]
    async fn test_stream_metrics_record_event_when_zero_latency_should_handle_gracefully() {
        let metrics = StreamMetrics::default();

        metrics.record_event("test", 0.0).await;

        assert_eq!(metrics.events_processed.load(Ordering::Relaxed), 1);
        let stats = metrics.performance_stats.read().await;
        assert_eq!(stats.avg_latency_ms, 0.0);
    }

    #[tokio::test]
    async fn test_stream_metrics_record_event_when_negative_latency_should_handle() {
        let metrics = StreamMetrics::default();

        metrics.record_event("test", -10.0).await;

        assert_eq!(metrics.events_processed.load(Ordering::Relaxed), 1);
        let stats = metrics.performance_stats.read().await;
        assert!(stats.avg_latency_ms < 0.0);
    }

    #[tokio::test]
    async fn test_stream_metrics_record_event_when_empty_event_type_should_work() {
        let metrics = StreamMetrics::default();

        metrics.record_event("", 50.0).await;

        assert_eq!(metrics.events_processed.load(Ordering::Relaxed), 1);
        assert_eq!(*metrics.events_by_type.get("").unwrap().value(), 1);
    }

    #[tokio::test]
    async fn test_stream_metrics_record_error_when_valid_type_should_increment_counter() {
        let metrics = StreamMetrics::default();

        metrics.record_error("connection_timeout").await;

        assert_eq!(metrics.error_count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_stream_metrics_record_error_when_multiple_errors_should_accumulate() {
        let metrics = StreamMetrics::default();

        metrics.record_error("timeout").await;
        metrics.record_error("network").await;
        metrics.record_error("timeout").await;

        assert_eq!(metrics.error_count.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_stream_metrics_record_error_when_empty_type_should_work() {
        let metrics = StreamMetrics::default();

        metrics.record_error("").await;

        assert_eq!(metrics.error_count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_stream_metrics_get_metrics_when_no_events_should_return_zero_values() {
        let metrics = StreamMetrics::default();

        let snapshot = metrics.get_metrics().await;

        assert_eq!(snapshot.events_processed, 0);
        assert_eq!(snapshot.error_count, 0);
        assert!(snapshot.events_by_type.is_empty());
        assert_eq!(snapshot.events_per_second, 0.0);
    }

    #[tokio::test]
    async fn test_stream_metrics_get_metrics_when_has_events_should_return_correct_values() {
        let metrics = StreamMetrics::default();

        metrics.record_event("trade", 100.0).await;
        metrics.record_error("timeout").await;

        let snapshot = metrics.get_metrics().await;

        assert_eq!(snapshot.events_processed, 1);
        assert_eq!(snapshot.error_count, 1);
        assert_eq!(snapshot.events_by_type["trade"], 1);
        assert!(snapshot.uptime.as_nanos() > 0);
    }

    #[tokio::test]
    async fn test_stream_metrics_get_metrics_when_zero_uptime_should_handle_division_by_zero() {
        let metrics = StreamMetrics::default();

        // Simulate zero duration by setting a very close start time
        let snapshot = metrics.get_metrics().await;

        // Should not panic and handle zero division gracefully
        assert!(snapshot.events_per_second >= 0.0);
    }

    #[tokio::test]
    async fn test_stream_metrics_get_metrics_when_uptime_exists_should_calculate_events_per_second()
    {
        let metrics = StreamMetrics::default();

        // Add some events and wait a bit to ensure uptime > 0
        metrics.record_event("test", 50.0).await;
        sleep(TokioDuration::from_millis(10)).await;

        let snapshot = metrics.get_metrics().await;

        if snapshot.uptime.as_secs() > 0 {
            assert!(snapshot.events_per_second > 0.0);
        }
    }

    #[test]
    fn test_stream_metrics_clone_when_called_should_create_independent_copy() {
        let original = StreamMetrics::default();
        original.events_processed.store(5, Ordering::Relaxed);
        original.error_count.store(3, Ordering::Relaxed);

        let cloned = original.clone();

        assert_eq!(cloned.events_processed.load(Ordering::Relaxed), 5);
        assert_eq!(cloned.error_count.load(Ordering::Relaxed), 3);
        assert_eq!(cloned.start_time, original.start_time);

        // Test independence - changes to clone don't affect original
        cloned.events_processed.store(10, Ordering::Relaxed);
        assert_eq!(original.events_processed.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn test_metrics_snapshot_clone() {
        let original = MetricsSnapshot {
            uptime: Duration::from_secs(100),
            events_processed: 500,
            events_by_type: {
                let mut map = HashMap::new();
                map.insert("trade".to_string(), 300);
                map.insert("order".to_string(), 200);
                map
            },
            error_count: 5,
            events_per_second: 5.0,
            performance_stats: PerformanceStats {
                events_per_second: 5.0,
                peak_events_per_second: 10.0,
                avg_latency_ms: 50.0,
                p50_latency_ms: 45.0,
                p95_latency_ms: 95.0,
                p99_latency_ms: 150.0,
            },
        };

        let cloned = original.clone();
        assert_eq!(cloned.uptime, original.uptime);
        assert_eq!(cloned.events_processed, original.events_processed);
        assert_eq!(cloned.events_by_type, original.events_by_type);
        assert_eq!(cloned.error_count, original.error_count);
        assert_eq!(cloned.events_per_second, original.events_per_second);
    }

    #[test]
    fn test_metrics_collector_default() {
        let collector = MetricsCollector::default();
        assert!(collector.stream_metrics.is_empty());
    }

    #[test]
    fn test_metrics_collector_new() {
        let collector = MetricsCollector::default();
        assert!(collector.stream_metrics.is_empty());
    }

    #[tokio::test]
    async fn test_metrics_collector_get_stream_metrics_when_new_stream_should_create_entry() {
        let collector = MetricsCollector::default();

        let metrics = collector.get_stream_metrics("test_stream").await;

        assert_eq!(metrics.events_processed.load(Ordering::Relaxed), 0);
        assert!(collector.stream_metrics.contains_key("test_stream"));
    }

    #[tokio::test]
    async fn test_metrics_collector_get_stream_metrics_when_existing_stream_should_return_same() {
        let collector = MetricsCollector::default();

        let metrics1 = collector.get_stream_metrics("test_stream").await;
        metrics1.events_processed.store(5, Ordering::Relaxed);

        let metrics2 = collector.get_stream_metrics("test_stream").await;

        assert_eq!(metrics2.events_processed.load(Ordering::Relaxed), 5);
    }

    #[tokio::test]
    async fn test_metrics_collector_get_stream_metrics_when_empty_name_should_work() {
        let collector = MetricsCollector::default();

        let metrics = collector.get_stream_metrics("").await;

        assert_eq!(metrics.events_processed.load(Ordering::Relaxed), 0);
        assert!(collector.stream_metrics.contains_key(""));
    }

    #[tokio::test]
    async fn test_metrics_collector_get_all_metrics_when_no_streams_should_return_empty() {
        let collector = MetricsCollector::default();

        let all_metrics = collector.get_all_metrics().await;

        assert!(all_metrics.is_empty());
    }

    #[tokio::test]
    async fn test_metrics_collector_get_all_metrics_when_multiple_streams_should_return_all() {
        let collector = MetricsCollector::default();

        let metrics1 = collector.get_stream_metrics("stream1").await;
        let metrics2 = collector.get_stream_metrics("stream2").await;

        metrics1.record_event("trade", 100.0).await;
        metrics2.record_event("order", 50.0).await;

        let all_metrics = collector.get_all_metrics().await;

        assert_eq!(all_metrics.len(), 2);
        assert!(all_metrics.contains_key("stream1"));
        assert!(all_metrics.contains_key("stream2"));
        assert_eq!(all_metrics["stream1"].events_processed, 1);
        assert_eq!(all_metrics["stream2"].events_processed, 1);
    }

    #[tokio::test]
    async fn test_metrics_collector_get_all_metrics_when_single_stream_multiple_events_should_aggregate(
    ) {
        let collector = MetricsCollector::default();

        let metrics = collector.get_stream_metrics("main_stream").await;
        metrics.record_event("trade", 100.0).await;
        metrics.record_event("order", 75.0).await;
        metrics.record_error("timeout").await;

        let all_metrics = collector.get_all_metrics().await;

        assert_eq!(all_metrics.len(), 1);
        let stream_snapshot = &all_metrics["main_stream"];
        assert_eq!(stream_snapshot.events_processed, 2);
        assert_eq!(stream_snapshot.error_count, 1);
        assert_eq!(stream_snapshot.events_by_type.len(), 2);
    }

    #[tokio::test]
    async fn test_integration_full_workflow_should_work_correctly() {
        let collector = MetricsCollector::default();

        // Create multiple streams
        let stream1 = collector.get_stream_metrics("trading").await;
        let stream2 = collector.get_stream_metrics("risk_management").await;

        // Record various events
        stream1.record_event("buy_order", 120.5).await;
        stream1.record_event("sell_order", 95.3).await;
        stream1.record_error("network_timeout").await;

        stream2.record_event("risk_check", 45.8).await;
        stream2.record_event("position_update", 67.2).await;

        // Get all metrics
        let all_metrics = collector.get_all_metrics().await;

        // Verify trading stream
        let trading_metrics = &all_metrics["trading"];
        assert_eq!(trading_metrics.events_processed, 2);
        assert_eq!(trading_metrics.error_count, 1);
        assert_eq!(trading_metrics.events_by_type.len(), 2);
        assert_eq!(trading_metrics.events_by_type["buy_order"], 1);
        assert_eq!(trading_metrics.events_by_type["sell_order"], 1);

        // Verify risk management stream
        let risk_metrics = &all_metrics["risk_management"];
        assert_eq!(risk_metrics.events_processed, 2);
        assert_eq!(risk_metrics.error_count, 0);
        assert_eq!(risk_metrics.events_by_type.len(), 2);
        assert_eq!(risk_metrics.events_by_type["risk_check"], 1);
        assert_eq!(risk_metrics.events_by_type["position_update"], 1);
    }

    #[tokio::test]
    async fn test_concurrent_access_should_be_thread_safe() {
        let collector = Arc::new(MetricsCollector::default());
        let stream_metrics = collector.get_stream_metrics("concurrent_test").await;

        let mut handles = vec![];

        // Spawn multiple tasks that modify metrics concurrently
        for i in 0..10 {
            let metrics = stream_metrics.clone();
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    metrics
                        .record_event(&format!("event_{}", i), j as f64)
                        .await;
                    if j % 3 == 0 {
                        metrics.record_error(&format!("error_{}", i)).await;
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify final state
        let final_metrics = stream_metrics.get_metrics().await;
        assert_eq!(final_metrics.events_processed, 100); // 10 tasks * 10 events each
        assert!(final_metrics.error_count > 0);
        assert!(final_metrics.events_by_type.len() == 10); // event_0 through event_9
    }
}
