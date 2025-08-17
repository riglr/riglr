use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;

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
            latency_histogram: Vec::new(),
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
            execution_histogram: Vec::new(),
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
        let mut stream_metrics =
            self.stream_metrics
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
        let mut handler_metrics =
            self.handler_metrics
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
    }

    /// Record a dropped event
    pub async fn record_dropped_event(&self, stream_name: &str) {
        if let Some(mut stream_metrics) = self.stream_metrics.get_mut(stream_name) {
            stream_metrics.events_dropped += 1;
        }
    }

    /// Get stream metrics
    pub async fn get_stream_metrics(&self, stream_name: &str) -> Option<StreamMetrics> {
        self.stream_metrics.get(stream_name).cloned()
    }

    /// Get all stream metrics
    pub async fn get_all_stream_metrics(&self) -> Vec<StreamMetrics> {
        self.stream_metrics.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Get handler metrics
    pub async fn get_handler_metrics(&self, handler_name: &str) -> Option<HandlerMetrics> {
        self.handler_metrics.get(handler_name).cloned()
    }

    /// Get all handler metrics
    pub async fn get_all_handler_metrics(&self) -> Vec<HandlerMetrics> {
        self.handler_metrics.iter().map(|entry| entry.value().clone()).collect()
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
            let uptime_secs = SystemTime::now()
                .duration_since(metrics.start_time)
                .unwrap_or_default()
                .as_secs() as f64;

            if uptime_secs > 0.0 {
                metrics.events_per_second = metrics.events_processed as f64 / uptime_secs;
                metrics.bytes_per_second = metrics.bytes_received as f64 / uptime_secs;
                metrics.uptime = Duration::from_secs(uptime_secs as u64);
            }
        }

        // Calculate global rates
        let system_uptime_secs = SystemTime::now()
            .duration_since(global.system_start_time)
            .unwrap_or_default()
            .as_secs() as f64;

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
