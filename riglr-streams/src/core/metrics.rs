use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;

#[cfg(feature = "metrics-facade")]
use metrics::{counter, gauge, histogram};

/// Performance metrics for a stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetrics {
    /// Stream name
    pub stream_name: String,
    /// Total events received
    pub events_received: u64,
    /// Total events processed
    pub events_processed: u64,
    /// Total events dropped
    pub events_dropped: u64,
    /// Average event processing time (ms)
    pub avg_processing_time_ms: f64,
    /// P99 event processing time (ms)
    pub p99_processing_time_ms: f64,
    /// P95 event processing time (ms)
    pub p95_processing_time_ms: f64,
    /// P50 event processing time (ms)
    pub p50_processing_time_ms: f64,
    /// Events per second (calculated over last minute)
    pub events_per_second: f64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Bytes per second (calculated over last minute)
    pub bytes_per_second: f64,
    /// Connection uptime
    pub uptime: Duration,
    /// Total reconnections
    pub reconnection_count: u64,
    /// Last event time
    pub last_event_time: Option<SystemTime>,
    /// Stream start time
    pub start_time: SystemTime,
    /// Latency distribution
    #[serde(skip)]
    pub latency_histogram: Vec<f64>,
}

impl Default for StreamMetrics {
    fn default() -> Self {
        Self {
            stream_name: String::default(),
            events_received: 0,
            events_processed: 0,
            events_dropped: 0,
            avg_processing_time_ms: 0.0,
            p99_processing_time_ms: 0.0,
            p95_processing_time_ms: 0.0,
            p50_processing_time_ms: 0.0,
            events_per_second: 0.0,
            bytes_received: 0,
            bytes_per_second: 0.0,
            uptime: Duration::from_secs(0),
            reconnection_count: 0,
            last_event_time: None,
            start_time: SystemTime::now(),
            latency_histogram: Vec::default(),
        }
    }
}

/// Handler metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerMetrics {
    /// Handler name
    pub handler_name: String,
    /// Total invocations
    pub invocations: u64,
    /// Successful executions
    pub successes: u64,
    /// Failed executions
    pub failures: u64,
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
    /// P99 execution time (ms)
    pub p99_execution_time_ms: f64,
    /// P95 execution time (ms)
    pub p95_execution_time_ms: f64,
    /// P50 execution time (ms)
    pub p50_execution_time_ms: f64,
    /// Last execution time
    pub last_execution_time: Option<SystemTime>,
    /// Execution time histogram
    #[serde(skip)]
    pub execution_histogram: Vec<f64>,
}

impl Default for HandlerMetrics {
    fn default() -> Self {
        Self {
            handler_name: String::default(),
            invocations: 0,
            successes: 0,
            failures: 0,
            avg_execution_time_ms: 0.0,
            p99_execution_time_ms: 0.0,
            p95_execution_time_ms: 0.0,
            p50_execution_time_ms: 0.0,
            last_execution_time: None,
            execution_histogram: Vec::default(),
        }
    }
}

/// Global metrics aggregator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetrics {
    /// Total events across all streams
    pub total_events: u64,
    /// Total bytes across all streams
    pub total_bytes: u64,
    /// Global events per second
    pub global_events_per_second: f64,
    /// Global bytes per second
    pub global_bytes_per_second: f64,
    /// Active stream count
    pub active_streams: usize,
    /// Active handler count
    pub active_handlers: usize,
    /// System start time
    pub system_start_time: SystemTime,
    /// Last metrics update
    pub last_update: SystemTime,
}

impl Default for GlobalMetrics {
    fn default() -> Self {
        Self {
            total_events: 0,
            total_bytes: 0,
            global_events_per_second: 0.0,
            global_bytes_per_second: 0.0,
            active_streams: 0,
            active_handlers: 0,
            system_start_time: SystemTime::now(),
            last_update: SystemTime::now(),
        }
    }
}

/// Metrics collector for the streaming system
pub struct MetricsCollector {
    /// Stream metrics
    stream_metrics: Arc<DashMap<String, StreamMetrics>>,
    /// Handler metrics
    handler_metrics: Arc<DashMap<String, HandlerMetrics>>,
    /// Global metrics
    global_metrics: Arc<RwLock<GlobalMetrics>>,
    /// Metrics history for time-series data
    metrics_history: Arc<RwLock<Vec<(SystemTime, GlobalMetrics)>>>,
    /// Maximum history size
    max_history_size: usize,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a stream event
    pub async fn record_stream_event(
        &self,
        stream_name: &str,
        processing_time_ms: f64,
        bytes: u64,
    ) {
        // Export to metrics facade if enabled
        #[cfg(feature = "metrics-facade")]
        {
            counter!("riglr_streams_events_total", "stream" => stream_name.to_string())
                .increment(1);
            histogram!("riglr_streams_processing_time_ms", "stream" => stream_name.to_string())
                .record(processing_time_ms);
            counter!("riglr_streams_bytes_received", "stream" => stream_name.to_string())
                .increment(bytes);
        }
        let mut stream_metrics = self
            .stream_metrics
            .entry(stream_name.to_string())
            .or_insert_with(|| StreamMetrics {
                stream_name: stream_name.to_string(),
                ..StreamMetrics::default()
            });

        // Update basic counters
        stream_metrics.events_received += 1;
        stream_metrics.events_processed += 1;
        stream_metrics.bytes_received += bytes;
        stream_metrics.last_event_time = Some(SystemTime::now());

        // Update latency histogram and percentiles
        stream_metrics.latency_histogram.push(processing_time_ms);
        if stream_metrics.latency_histogram.len() > 10000 {
            // Keep only last 10k samples
            stream_metrics.latency_histogram.drain(0..5000);
        }

        // Calculate percentiles
        let mut histogram = stream_metrics.latency_histogram.clone();
        let (p50, p95, p99, avg) = self.calculate_percentiles(&mut histogram);
        stream_metrics.latency_histogram = histogram;
        stream_metrics.p50_processing_time_ms = p50;
        stream_metrics.p95_processing_time_ms = p95;
        stream_metrics.p99_processing_time_ms = p99;
        stream_metrics.avg_processing_time_ms = avg;

        // Update global metrics
        let mut global = self.global_metrics.write().await;
        global.total_events += 1;
        global.total_bytes += bytes;
        global.last_update = SystemTime::now();
    }

    /// Record a handler execution
    pub async fn record_handler_execution(
        &self,
        handler_name: &str,
        execution_time_ms: f64,
        success: bool,
    ) {
        // Export to metrics facade if enabled
        #[cfg(feature = "metrics-facade")]
        {
            counter!("riglr_streams_handler_invocations", "handler" => handler_name.to_string())
                .increment(1);
            histogram!("riglr_streams_handler_execution_time_ms", "handler" => handler_name.to_string()).record(execution_time_ms);
            if success {
                counter!("riglr_streams_handler_successes", "handler" => handler_name.to_string())
                    .increment(1);
            } else {
                counter!("riglr_streams_handler_failures", "handler" => handler_name.to_string())
                    .increment(1);
            }
        }
        let mut handler_metrics = self
            .handler_metrics
            .entry(handler_name.to_string())
            .or_insert_with(|| HandlerMetrics {
                handler_name: handler_name.to_string(),
                ..HandlerMetrics::default()
            });

        // Update counters
        handler_metrics.invocations += 1;
        if success {
            handler_metrics.successes += 1;
        } else {
            handler_metrics.failures += 1;
        }
        handler_metrics.last_execution_time = Some(SystemTime::now());

        // Update execution histogram
        handler_metrics.execution_histogram.push(execution_time_ms);
        if handler_metrics.execution_histogram.len() > 10000 {
            handler_metrics.execution_histogram.drain(0..5000);
        }

        // Calculate percentiles
        let mut histogram = handler_metrics.execution_histogram.clone();
        let (p50, p95, p99, avg) = self.calculate_percentiles(&mut histogram);
        handler_metrics.execution_histogram = histogram;
        handler_metrics.p50_execution_time_ms = p50;
        handler_metrics.p95_execution_time_ms = p95;
        handler_metrics.p99_execution_time_ms = p99;
        handler_metrics.avg_execution_time_ms = avg;
    }

    /// Record a stream reconnection
    pub async fn record_reconnection(&self, stream_name: &str) {
        if let Some(mut stream_metrics) = self.stream_metrics.get_mut(stream_name) {
            stream_metrics.reconnection_count += 1;
        }

        // Export to metrics facade if enabled
        #[cfg(feature = "metrics-facade")]
        {
            counter!("riglr_streams_reconnections", "stream" => stream_name.to_string())
                .increment(1);
        }
    }

    /// Record a dropped event
    pub async fn record_dropped_event(&self, stream_name: &str) {
        if let Some(mut stream_metrics) = self.stream_metrics.get_mut(stream_name) {
            stream_metrics.events_dropped += 1;
        }

        // Export to metrics facade if enabled
        #[cfg(feature = "metrics-facade")]
        {
            counter!("riglr_streams_events_dropped", "stream" => stream_name.to_string())
                .increment(1);
        }
    }

    /// Record an error event
    pub async fn record_error(&self, stream_name: &str, error_type: &str) {
        // Increment error count for the stream
        if let Some(mut stream_metrics) = self.stream_metrics.get_mut(stream_name) {
            stream_metrics.events_dropped += 1; // Using events_dropped as error counter
        }

        // Export to metrics facade if enabled
        #[cfg(feature = "metrics-facade")]
        {
            counter!("riglr_streams_errors_total", "stream" => stream_name.to_string(), "type" => error_type.to_string()).increment(1);
        }

        // If metrics-facade is not enabled, we still use error_type for future extensibility
        #[cfg(not(feature = "metrics-facade"))]
        let _ = error_type;
    }

    /// Get stream metrics
    pub async fn get_stream_metrics(&self, stream_name: &str) -> Option<StreamMetrics> {
        self.stream_metrics
            .get(stream_name)
            .map(|entry| entry.value().clone())
    }

    /// Get all stream metrics
    pub async fn get_all_stream_metrics(&self) -> Vec<StreamMetrics> {
        self.stream_metrics
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get handler metrics
    pub async fn get_handler_metrics(&self, handler_name: &str) -> Option<HandlerMetrics> {
        self.handler_metrics
            .get(handler_name)
            .map(|entry| entry.value().clone())
    }

    /// Get all handler metrics
    pub async fn get_all_handler_metrics(&self) -> Vec<HandlerMetrics> {
        self.handler_metrics
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get global metrics
    pub async fn get_global_metrics(&self) -> GlobalMetrics {
        let global = self.global_metrics.read().await;
        global.clone()
    }

    /// Update rates (should be called periodically)
    pub async fn update_rates(&self) {
        let mut global = self.global_metrics.write().await;

        // Calculate rates for each stream
        for mut entry in self.stream_metrics.iter_mut() {
            let metrics = entry.value_mut();
            let uptime = SystemTime::now()
                .duration_since(metrics.start_time)
                .unwrap_or_default();
            let uptime_secs = uptime.as_secs_f64();

            if uptime_secs > 0.0 {
                metrics.events_per_second = metrics.events_processed as f64 / uptime_secs;
                metrics.bytes_per_second = metrics.bytes_received as f64 / uptime_secs;
                metrics.uptime = uptime;
            }
        }

        // Calculate global rates
        let system_uptime_secs = SystemTime::now()
            .duration_since(global.system_start_time)
            .unwrap_or_default()
            .as_secs_f64();

        if system_uptime_secs > 0.0 {
            global.global_events_per_second = global.total_events as f64 / system_uptime_secs;
            global.global_bytes_per_second = global.total_bytes as f64 / system_uptime_secs;
        }

        global.active_streams = self.stream_metrics.len();
        global.active_handlers = self.handler_metrics.len();

        // Add to history
        let mut history = self.metrics_history.write().await;
        history.push((SystemTime::now(), global.clone()));
        if history.len() > self.max_history_size {
            history.drain(0..100);
        }
        drop(history); // Release the lock

        // Export current rates to metrics facade if enabled
        #[cfg(feature = "metrics-facade")]
        {
            gauge!("riglr_streams_events_per_second").set(global.global_events_per_second);
            gauge!("riglr_streams_bytes_per_second").set(global.global_bytes_per_second);
            gauge!("riglr_streams_active_streams").set(global.active_streams as f64);
            gauge!("riglr_streams_active_handlers").set(global.active_handlers as f64);
            gauge!("riglr_streams_total_events").set(global.total_events as f64);
            gauge!("riglr_streams_total_bytes").set(global.total_bytes as f64);

            let uptime = SystemTime::now()
                .duration_since(global.system_start_time)
                .unwrap_or_default();
            gauge!("riglr_streams_uptime_seconds").set(uptime.as_secs() as f64);
        }
    }

    /// Calculate percentiles from histogram
    fn calculate_percentiles(&self, histogram: &mut [f64]) -> (f64, f64, f64, f64) {
        if histogram.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        histogram.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = histogram.len();
        let p50 = histogram[len / 2];
        let p95 = histogram[len * 95 / 100];
        let p99 = histogram[len * 99 / 100];

        let sum: f64 = histogram.iter().sum();
        let avg = sum / len as f64;

        (p50, p95, p99, avg)
    }

    /// Export metrics as JSON
    pub async fn export_json(&self) -> String {
        let streams = self.get_all_stream_metrics().await;
        let handlers = self.get_all_handler_metrics().await;
        let global = self.get_global_metrics().await;

        let export = serde_json::json!({
            "streams": streams,
            "handlers": handlers,
            "global": global,
            "timestamp": SystemTime::now(),
        });

        serde_json::to_string_pretty(&export).unwrap_or_default()
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let mut output = String::default();

        // Stream metrics
        let streams = self.get_all_stream_metrics().await;
        for stream in streams {
            output.push_str(&format!(
                "stream_events_received{{stream=\"{}\"}} {}\n",
                stream.stream_name, stream.events_received
            ));
            output.push_str(&format!(
                "stream_events_processed{{stream=\"{}\"}} {}\n",
                stream.stream_name, stream.events_processed
            ));
            output.push_str(&format!(
                "stream_events_dropped{{stream=\"{}\"}} {}\n",
                stream.stream_name, stream.events_dropped
            ));
            output.push_str(&format!(
                "stream_processing_time_ms{{stream=\"{}\",percentile=\"50\"}} {}\n",
                stream.stream_name, stream.p50_processing_time_ms
            ));
            output.push_str(&format!(
                "stream_processing_time_ms{{stream=\"{}\",percentile=\"95\"}} {}\n",
                stream.stream_name, stream.p95_processing_time_ms
            ));
            output.push_str(&format!(
                "stream_processing_time_ms{{stream=\"{}\",percentile=\"99\"}} {}\n",
                stream.stream_name, stream.p99_processing_time_ms
            ));
            output.push_str(&format!(
                "stream_bytes_received{{stream=\"{}\"}} {}\n",
                stream.stream_name, stream.bytes_received
            ));
            output.push_str(&format!(
                "stream_reconnections{{stream=\"{}\"}} {}\n",
                stream.stream_name, stream.reconnection_count
            ));
        }

        // Handler metrics
        let handlers = self.get_all_handler_metrics().await;
        for handler in handlers {
            output.push_str(&format!(
                "handler_invocations{{handler=\"{}\"}} {}\n",
                handler.handler_name, handler.invocations
            ));
            output.push_str(&format!(
                "handler_successes{{handler=\"{}\"}} {}\n",
                handler.handler_name, handler.successes
            ));
            output.push_str(&format!(
                "handler_failures{{handler=\"{}\"}} {}\n",
                handler.handler_name, handler.failures
            ));
            output.push_str(&format!(
                "handler_execution_time_ms{{handler=\"{}\",percentile=\"50\"}} {}\n",
                handler.handler_name, handler.p50_execution_time_ms
            ));
            output.push_str(&format!(
                "handler_execution_time_ms{{handler=\"{}\",percentile=\"95\"}} {}\n",
                handler.handler_name, handler.p95_execution_time_ms
            ));
            output.push_str(&format!(
                "handler_execution_time_ms{{handler=\"{}\",percentile=\"99\"}} {}\n",
                handler.handler_name, handler.p99_execution_time_ms
            ));
        }

        // Global metrics
        let global = self.get_global_metrics().await;
        output.push_str(&format!("global_total_events {}\n", global.total_events));
        output.push_str(&format!("global_total_bytes {}\n", global.total_bytes));
        output.push_str(&format!(
            "global_events_per_second {}\n",
            global.global_events_per_second
        ));
        output.push_str(&format!(
            "global_bytes_per_second {}\n",
            global.global_bytes_per_second
        ));
        output.push_str(&format!("active_streams {}\n", global.active_streams));
        output.push_str(&format!("active_handlers {}\n", global.active_handlers));

        output
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self {
            stream_metrics: Arc::new(DashMap::default()),
            handler_metrics: Arc::new(DashMap::default()),
            global_metrics: Arc::new(RwLock::new(GlobalMetrics::default())),
            metrics_history: Arc::new(RwLock::new(Vec::default())),
            max_history_size: 1000,
        }
    }
}

/// Metrics timer for measuring execution time
pub struct MetricsTimer {
    start: Instant,
    name: String,
    collector: Option<Arc<MetricsCollector>>,
}

impl MetricsTimer {
    /// Start a new timer
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            name: name.into(),
            collector: None,
        }
    }

    /// Start a timer with a collector
    pub fn start_with_collector(name: impl Into<String>, collector: Arc<MetricsCollector>) -> Self {
        Self {
            start: Instant::now(),
            name: name.into(),
            collector: Some(collector),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    /// Stop timer and record metrics
    pub async fn stop(self, success: bool) {
        let elapsed = self.elapsed_ms();
        if let Some(collector) = self.collector {
            if self.name.starts_with("stream:") {
                let stream_name = self.name.strip_prefix("stream:").unwrap_or(&self.name);
                collector.record_stream_event(stream_name, elapsed, 0).await;
            } else if self.name.starts_with("handler:") {
                let handler_name = self.name.strip_prefix("handler:").unwrap_or(&self.name);
                collector
                    .record_handler_execution(handler_name, elapsed, success)
                    .await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[test]
    fn test_stream_metrics_default() {
        let metrics = StreamMetrics::default();
        assert_eq!(metrics.stream_name, String::default());
        assert_eq!(metrics.events_received, 0);
        assert_eq!(metrics.events_processed, 0);
        assert_eq!(metrics.events_dropped, 0);
        assert_eq!(metrics.avg_processing_time_ms, 0.0);
        assert_eq!(metrics.p99_processing_time_ms, 0.0);
        assert_eq!(metrics.p95_processing_time_ms, 0.0);
        assert_eq!(metrics.p50_processing_time_ms, 0.0);
        assert_eq!(metrics.events_per_second, 0.0);
        assert_eq!(metrics.bytes_received, 0);
        assert_eq!(metrics.bytes_per_second, 0.0);
        assert_eq!(metrics.uptime, Duration::from_secs(0));
        assert_eq!(metrics.reconnection_count, 0);
        assert_eq!(metrics.last_event_time, None);
        assert_eq!(metrics.latency_histogram.len(), 0);
    }

    #[test]
    fn test_handler_metrics_default() {
        let metrics = HandlerMetrics::default();
        assert_eq!(metrics.handler_name, String::default());
        assert_eq!(metrics.invocations, 0);
        assert_eq!(metrics.successes, 0);
        assert_eq!(metrics.failures, 0);
        assert_eq!(metrics.avg_execution_time_ms, 0.0);
        assert_eq!(metrics.p99_execution_time_ms, 0.0);
        assert_eq!(metrics.p95_execution_time_ms, 0.0);
        assert_eq!(metrics.p50_execution_time_ms, 0.0);
        assert_eq!(metrics.last_execution_time, None);
        assert_eq!(metrics.execution_histogram.len(), 0);
    }

    #[test]
    fn test_global_metrics_default() {
        let metrics = GlobalMetrics::default();
        assert_eq!(metrics.total_events, 0);
        assert_eq!(metrics.total_bytes, 0);
        assert_eq!(metrics.global_events_per_second, 0.0);
        assert_eq!(metrics.global_bytes_per_second, 0.0);
        assert_eq!(metrics.active_streams, 0);
        assert_eq!(metrics.active_handlers, 0);
    }

    #[test]
    fn test_metrics_collector_new() {
        let collector = MetricsCollector::default();
        assert_eq!(collector.max_history_size, 1000);
    }

    #[test]
    fn test_metrics_collector_default() {
        let collector = MetricsCollector::default();
        assert_eq!(collector.max_history_size, 1000);
    }

    #[test]
    fn test_calculate_percentiles_empty_histogram() {
        let collector = MetricsCollector::default();
        let mut empty_histogram = Vec::default();
        let (p50, p95, p99, avg) = collector.calculate_percentiles(&mut empty_histogram);
        assert_eq!(p50, 0.0);
        assert_eq!(p95, 0.0);
        assert_eq!(p99, 0.0);
        assert_eq!(avg, 0.0);
    }

    #[test]
    fn test_calculate_percentiles_single_value() {
        let collector = MetricsCollector::default();
        let mut histogram = vec![10.0];
        let (p50, p95, p99, avg) = collector.calculate_percentiles(&mut histogram);
        assert_eq!(p50, 10.0);
        assert_eq!(p95, 10.0);
        assert_eq!(p99, 10.0);
        assert_eq!(avg, 10.0);
    }

    #[test]
    fn test_calculate_percentiles_multiple_values() {
        let collector = MetricsCollector::default();
        let mut histogram = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let (p50, p95, p99, avg) = collector.calculate_percentiles(&mut histogram);
        assert_eq!(p50, 6.0); // median of 10 items is at index 5
        assert_eq!(p95, 10.0); // 95th percentile of 10 items is at index 9
        assert_eq!(p99, 10.0); // 99th percentile of 10 items is at index 9
        assert_eq!(avg, 5.5); // (1+2+3+4+5+6+7+8+9+10)/10 = 5.5
    }

    #[tokio::test]
    async fn test_record_stream_event() {
        let collector = MetricsCollector::default();

        collector
            .record_stream_event("test_stream", 15.5, 1024)
            .await;

        let metrics = collector.get_stream_metrics("test_stream").await.unwrap();
        assert_eq!(metrics.stream_name, "test_stream");
        assert_eq!(metrics.events_received, 1);
        assert_eq!(metrics.events_processed, 1);
        assert_eq!(metrics.bytes_received, 1024);
        assert!(metrics.last_event_time.is_some());
        assert_eq!(metrics.latency_histogram.len(), 1);
        assert_eq!(metrics.latency_histogram[0], 15.5);
        assert_eq!(metrics.p50_processing_time_ms, 15.5);
        assert_eq!(metrics.avg_processing_time_ms, 15.5);
    }

    #[tokio::test]
    async fn test_record_stream_event_multiple_events() {
        let collector = MetricsCollector::default();

        // Record multiple events
        collector
            .record_stream_event("test_stream", 10.0, 100)
            .await;
        collector
            .record_stream_event("test_stream", 20.0, 200)
            .await;
        collector
            .record_stream_event("test_stream", 30.0, 300)
            .await;

        let metrics = collector.get_stream_metrics("test_stream").await.unwrap();
        assert_eq!(metrics.events_received, 3);
        assert_eq!(metrics.events_processed, 3);
        assert_eq!(metrics.bytes_received, 600);
        assert_eq!(metrics.latency_histogram.len(), 3);
    }

    #[tokio::test]
    async fn test_record_stream_event_histogram_limit() {
        let collector = MetricsCollector::default();

        // Add more than 10000 entries to trigger histogram cleanup
        for i in 0..10001 {
            collector
                .record_stream_event("test_stream", i as f64, 1)
                .await;
        }

        let metrics = collector.get_stream_metrics("test_stream").await.unwrap();
        // Should drain 5000 entries when it exceeds 10000, so we should have ~5001 entries
        assert!(metrics.latency_histogram.len() <= 5001);
        assert_eq!(metrics.events_received, 10001);
    }

    #[tokio::test]
    async fn test_record_handler_execution_success() {
        let collector = MetricsCollector::default();

        collector
            .record_handler_execution("test_handler", 25.0, true)
            .await;

        let metrics = collector.get_handler_metrics("test_handler").await.unwrap();
        assert_eq!(metrics.handler_name, "test_handler");
        assert_eq!(metrics.invocations, 1);
        assert_eq!(metrics.successes, 1);
        assert_eq!(metrics.failures, 0);
        assert!(metrics.last_execution_time.is_some());
        assert_eq!(metrics.execution_histogram.len(), 1);
        assert_eq!(metrics.execution_histogram[0], 25.0);
    }

    #[tokio::test]
    async fn test_record_handler_execution_failure() {
        let collector = MetricsCollector::default();

        collector
            .record_handler_execution("test_handler", 50.0, false)
            .await;

        let metrics = collector.get_handler_metrics("test_handler").await.unwrap();
        assert_eq!(metrics.invocations, 1);
        assert_eq!(metrics.successes, 0);
        assert_eq!(metrics.failures, 1);
    }

    #[tokio::test]
    async fn test_record_handler_execution_histogram_limit() {
        let collector = MetricsCollector::default();

        // Add more than 10000 entries to trigger histogram cleanup
        for i in 0..10001 {
            collector
                .record_handler_execution("test_handler", i as f64, true)
                .await;
        }

        let metrics = collector.get_handler_metrics("test_handler").await.unwrap();
        // Should drain 5000 entries when it exceeds 10000
        assert!(metrics.execution_histogram.len() <= 5001);
        assert_eq!(metrics.invocations, 10001);
        assert_eq!(metrics.successes, 10001);
        assert_eq!(metrics.failures, 0);
    }

    #[tokio::test]
    async fn test_record_reconnection() {
        let collector = MetricsCollector::default();

        // First create a stream by recording an event
        collector
            .record_stream_event("test_stream", 10.0, 100)
            .await;

        // Then record a reconnection
        collector.record_reconnection("test_stream").await;

        let metrics = collector.get_stream_metrics("test_stream").await.unwrap();
        assert_eq!(metrics.reconnection_count, 1);
    }

    #[tokio::test]
    async fn test_record_reconnection_nonexistent_stream() {
        let collector = MetricsCollector::default();

        // Try to record reconnection for non-existent stream
        collector.record_reconnection("nonexistent_stream").await;

        // Should not create a new stream
        let metrics = collector.get_stream_metrics("nonexistent_stream").await;
        assert!(metrics.is_none());
    }

    #[tokio::test]
    async fn test_record_dropped_event() {
        let collector = MetricsCollector::default();

        // First create a stream
        collector
            .record_stream_event("test_stream", 10.0, 100)
            .await;

        // Then record a dropped event
        collector.record_dropped_event("test_stream").await;

        let metrics = collector.get_stream_metrics("test_stream").await.unwrap();
        assert_eq!(metrics.events_dropped, 1);
    }

    #[tokio::test]
    async fn test_record_dropped_event_nonexistent_stream() {
        let collector = MetricsCollector::default();

        // Try to record dropped event for non-existent stream
        collector.record_dropped_event("nonexistent_stream").await;

        // Should not create a new stream
        let metrics = collector.get_stream_metrics("nonexistent_stream").await;
        assert!(metrics.is_none());
    }

    #[tokio::test]
    async fn test_get_stream_metrics_nonexistent() {
        let collector = MetricsCollector::default();

        let metrics = collector.get_stream_metrics("nonexistent").await;
        assert!(metrics.is_none());
    }

    #[tokio::test]
    async fn test_get_all_stream_metrics() {
        let collector = MetricsCollector::default();

        collector.record_stream_event("stream1", 10.0, 100).await;
        collector.record_stream_event("stream2", 20.0, 200).await;

        let all_metrics = collector.get_all_stream_metrics().await;
        assert_eq!(all_metrics.len(), 2);
    }

    #[tokio::test]
    async fn test_get_all_stream_metrics_empty() {
        let collector = MetricsCollector::default();

        let all_metrics = collector.get_all_stream_metrics().await;
        assert_eq!(all_metrics.len(), 0);
    }

    #[tokio::test]
    async fn test_get_handler_metrics_nonexistent() {
        let collector = MetricsCollector::default();

        let metrics = collector.get_handler_metrics("nonexistent").await;
        assert!(metrics.is_none());
    }

    #[tokio::test]
    async fn test_get_all_handler_metrics() {
        let collector = MetricsCollector::default();

        collector
            .record_handler_execution("handler1", 10.0, true)
            .await;
        collector
            .record_handler_execution("handler2", 20.0, false)
            .await;

        let all_metrics = collector.get_all_handler_metrics().await;
        assert_eq!(all_metrics.len(), 2);
    }

    #[tokio::test]
    async fn test_get_all_handler_metrics_empty() {
        let collector = MetricsCollector::default();

        let all_metrics = collector.get_all_handler_metrics().await;
        assert_eq!(all_metrics.len(), 0);
    }

    #[tokio::test]
    async fn test_get_global_metrics() {
        let collector = MetricsCollector::default();

        let global_metrics = collector.get_global_metrics().await;
        assert_eq!(global_metrics.total_events, 0);
        assert_eq!(global_metrics.total_bytes, 0);
    }

    #[tokio::test]
    async fn test_update_rates() {
        let collector = MetricsCollector::default();

        // Create some streams and handlers
        collector.record_stream_event("stream1", 10.0, 100).await;
        collector
            .record_handler_execution("handler1", 10.0, true)
            .await;

        collector.update_rates().await;

        let global_metrics = collector.get_global_metrics().await;
        assert_eq!(global_metrics.active_streams, 1);
        assert_eq!(global_metrics.active_handlers, 1);

        let stream_metrics = collector.get_stream_metrics("stream1").await.unwrap();
        assert!(stream_metrics.events_per_second > 0.0);
        assert!(stream_metrics.bytes_per_second > 0.0);
    }

    #[tokio::test]
    async fn test_update_rates_with_history_overflow() {
        let collector = MetricsCollector::default();

        // Fill history beyond max_history_size
        for _ in 0..1001 {
            collector.update_rates().await;
        }

        let history = collector.metrics_history.read().await;
        // Should have drained 100 entries when exceeding 1000
        assert!(history.len() <= 901);
    }

    #[tokio::test]
    async fn test_export_json() {
        let collector = MetricsCollector::default();

        collector.record_stream_event("stream1", 10.0, 100).await;
        collector
            .record_handler_execution("handler1", 15.0, true)
            .await;

        let json_export = collector.export_json().await;
        assert!(json_export.contains("streams"));
        assert!(json_export.contains("handlers"));
        assert!(json_export.contains("global"));
        assert!(json_export.contains("timestamp"));
        assert!(json_export.contains("stream1"));
        assert!(json_export.contains("handler1"));
    }

    #[tokio::test]
    async fn test_export_prometheus() {
        let collector = MetricsCollector::default();

        collector.record_stream_event("stream1", 10.0, 100).await;
        collector
            .record_handler_execution("handler1", 15.0, true)
            .await;

        let prometheus_export = collector.export_prometheus().await;
        assert!(prometheus_export.contains("stream_events_received"));
        assert!(prometheus_export.contains("stream_events_processed"));
        assert!(prometheus_export.contains("stream_events_dropped"));
        assert!(prometheus_export.contains("stream_processing_time_ms"));
        assert!(prometheus_export.contains("stream_bytes_received"));
        assert!(prometheus_export.contains("stream_reconnections"));
        assert!(prometheus_export.contains("handler_invocations"));
        assert!(prometheus_export.contains("handler_successes"));
        assert!(prometheus_export.contains("handler_failures"));
        assert!(prometheus_export.contains("handler_execution_time_ms"));
        assert!(prometheus_export.contains("global_total_events"));
        assert!(prometheus_export.contains("global_total_bytes"));
        assert!(prometheus_export.contains("active_streams"));
        assert!(prometheus_export.contains("active_handlers"));
        assert!(prometheus_export.contains("stream1"));
        assert!(prometheus_export.contains("handler1"));
    }

    #[tokio::test]
    async fn test_export_prometheus_empty() {
        let collector = MetricsCollector::default();

        let prometheus_export = collector.export_prometheus().await;
        assert!(prometheus_export.contains("global_total_events 0"));
        assert!(prometheus_export.contains("global_total_bytes 0"));
        assert!(prometheus_export.contains("active_streams 0"));
        assert!(prometheus_export.contains("active_handlers 0"));
    }

    #[test]
    fn test_metrics_timer_start() {
        let timer = MetricsTimer::start("test_timer");
        assert_eq!(timer.name, "test_timer");
        assert!(timer.collector.is_none());
    }

    #[test]
    fn test_metrics_timer_start_with_collector() {
        let collector = Arc::new(MetricsCollector::default());
        let timer = MetricsTimer::start_with_collector("test_timer", collector.clone());
        assert_eq!(timer.name, "test_timer");
        assert!(timer.collector.is_some());
    }

    #[tokio::test]
    async fn test_metrics_timer_elapsed_ms() {
        let timer = MetricsTimer::start("test_timer");
        sleep(Duration::from_millis(10)).await;
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10.0);
    }

    #[tokio::test]
    async fn test_metrics_timer_stop_without_collector() {
        let timer = MetricsTimer::start("test_timer");
        timer.stop(true).await;
        // Should not panic when no collector is present
    }

    #[tokio::test]
    async fn test_metrics_timer_stop_with_stream_prefix() {
        let collector = Arc::new(MetricsCollector::default());
        let timer = MetricsTimer::start_with_collector("stream:test_stream", collector.clone());
        timer.stop(true).await;

        let metrics = collector.get_stream_metrics("test_stream").await.unwrap();
        assert_eq!(metrics.events_received, 1);
        assert_eq!(metrics.events_processed, 1);
    }

    #[tokio::test]
    async fn test_metrics_timer_stop_with_handler_prefix() {
        let collector = Arc::new(MetricsCollector::default());
        let timer = MetricsTimer::start_with_collector("handler:test_handler", collector.clone());
        timer.stop(true).await;

        let metrics = collector.get_handler_metrics("test_handler").await.unwrap();
        assert_eq!(metrics.invocations, 1);
        assert_eq!(metrics.successes, 1);
        assert_eq!(metrics.failures, 0);
    }

    #[tokio::test]
    async fn test_metrics_timer_stop_with_handler_prefix_failure() {
        let collector = Arc::new(MetricsCollector::default());
        let timer = MetricsTimer::start_with_collector("handler:test_handler", collector.clone());
        timer.stop(false).await;

        let metrics = collector.get_handler_metrics("test_handler").await.unwrap();
        assert_eq!(metrics.invocations, 1);
        assert_eq!(metrics.successes, 0);
        assert_eq!(metrics.failures, 1);
    }

    #[tokio::test]
    async fn test_metrics_timer_stop_with_unknown_prefix() {
        let collector = Arc::new(MetricsCollector::default());
        let timer = MetricsTimer::start_with_collector("unknown:test", collector.clone());
        timer.stop(true).await;

        // Should not create any metrics for unknown prefix
        let stream_metrics = collector.get_stream_metrics("test").await;
        let handler_metrics = collector.get_handler_metrics("test").await;
        assert!(stream_metrics.is_none());
        assert!(handler_metrics.is_none());
    }

    #[tokio::test]
    async fn test_metrics_timer_strip_prefix_edge_cases() {
        let collector = Arc::new(MetricsCollector::default());

        // Test with just the prefix (should use fallback)
        let timer = MetricsTimer::start_with_collector("stream:", collector.clone());
        timer.stop(true).await;

        let metrics = collector.get_stream_metrics("").await.unwrap();
        assert_eq!(metrics.events_received, 1);

        // Test with handler prefix only
        let timer = MetricsTimer::start_with_collector("handler:", collector.clone());
        timer.stop(true).await;

        let metrics = collector.get_handler_metrics("").await.unwrap();
        assert_eq!(metrics.invocations, 1);
    }

    #[test]
    fn test_stream_metrics_serialization() {
        let metrics = StreamMetrics::default();
        let serialized = serde_json::to_string(&metrics).unwrap();
        let deserialized: StreamMetrics = serde_json::from_str(&serialized).unwrap();
        assert_eq!(metrics.stream_name, deserialized.stream_name);
        assert_eq!(metrics.events_received, deserialized.events_received);
        // latency_histogram should be skipped during serialization
        assert_eq!(deserialized.latency_histogram.len(), 0);
    }

    #[test]
    fn test_handler_metrics_serialization() {
        let metrics = HandlerMetrics::default();
        let serialized = serde_json::to_string(&metrics).unwrap();
        let deserialized: HandlerMetrics = serde_json::from_str(&serialized).unwrap();
        assert_eq!(metrics.handler_name, deserialized.handler_name);
        assert_eq!(metrics.invocations, deserialized.invocations);
        // execution_histogram should be skipped during serialization
        assert_eq!(deserialized.execution_histogram.len(), 0);
    }

    #[test]
    fn test_global_metrics_serialization() {
        let metrics = GlobalMetrics::default();
        let serialized = serde_json::to_string(&metrics).unwrap();
        let deserialized: GlobalMetrics = serde_json::from_str(&serialized).unwrap();
        assert_eq!(metrics.total_events, deserialized.total_events);
        assert_eq!(metrics.active_streams, deserialized.active_streams);
    }
}
