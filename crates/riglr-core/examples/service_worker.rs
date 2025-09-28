//! Service-oriented example showing a long-running `ToolWorker` with job queue
//!
//! This demonstrates the recommended pattern for production services:
//! - Long-running worker task with graceful shutdown
//! - Job queue for decoupling job submission from processing
//! - Background processing with cancellation support
//! - Proper resource cleanup

use async_trait::async_trait;
use core::{error::Error, time::Duration};
use riglr_config::Config;
use riglr_core::{
    idempotency::InMemoryIdempotencyStore,
    provider::ApplicationContext,
    queue::{InMemory, Queue},
    tool::Worker as ToolWorker,
    ExecutionConfig, Job, JobResult, Tool, ToolError,
};
use std::sync::Arc;
use tokio::{
    task::JoinHandle,
    time::{sleep, timeout},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

/// Example analytics tool that might take some time to process
#[derive(Clone)]
struct AnalyticsTool;

#[async_trait]
impl Tool for AnalyticsTool {
    type Args = serde_json::Value;
    type Output = JobResult;
    type Error = ToolError;

    fn name(&self) -> &'static str {
        "analytics"
    }

    fn description(&self) -> &'static str {
        "Performs data analytics with simulated processing time"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "data_size": {"type": "integer"},
                "type": {"type": "string"}
            }
        })
    }

    async fn call(&self, params: Self::Args) -> Result<Self::Output, Self::Error> {
        let data_size = params
            .get("data_size")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(100);
        let analysis_type = params
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("basic");

        info!(
            "Starting {} analysis on {} data points",
            analysis_type, data_size
        );

        // Simulate processing time based on data size
        let processing_time = Duration::from_millis(data_size.saturating_mul(10));
        sleep(processing_time).await;

        // Return analysis results
        let result = serde_json::json!({
            "type": analysis_type,
            "data_points": data_size,
            "processing_time_ms": processing_time.as_millis(),
            "insights": vec![
                "Pattern A detected",
                "Anomaly in segment B",
                "Trend shows positive growth"
            ]
        });

        Ok(JobResult::success(&result).map_err(|e| ToolError::permanent_string(e.to_string()))?)
    }
}

/// Example notification tool
#[derive(Clone)]
struct NotificationTool;

#[async_trait]
impl Tool for NotificationTool {
    type Args = serde_json::Value;
    type Output = JobResult;
    type Error = ToolError;

    fn name(&self) -> &'static str {
        "notify"
    }

    fn description(&self) -> &'static str {
        "Sends notifications to users"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "recipient": {"type": "string"},
                "message": {"type": "string"},
                "priority": {"type": "string"}
            }
        })
    }

    async fn call(&self, params: Self::Args) -> Result<Self::Output, Self::Error> {
        let recipient = params
            .get("recipient")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("admin");
        let message = params
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Notification");
        let priority = params
            .get("priority")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("normal");

        info!(
            "Sending {} priority notification to {}",
            priority, recipient
        );

        // Simulate notification sending
        sleep(Duration::from_millis(100)).await;

        let result = serde_json::json!({
            "status": "sent",
            "recipient": recipient,
            "message": message,
            "priority": priority,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(JobResult::success(&result).map_err(|e| ToolError::permanent_string(e.to_string()))?)
    }
}

/// Example report generation tool
#[derive(Clone)]
struct ReportTool;

#[async_trait]
impl Tool for ReportTool {
    type Args = serde_json::Value;
    type Output = JobResult;
    type Error = ToolError;

    fn name(&self) -> &'static str {
        "report"
    }

    fn description(&self) -> &'static str {
        "Generates various types of reports"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "report_type": {"type": "string"},
                "period": {"type": "string"}
            }
        })
    }

    async fn call(&self, params: Self::Args) -> Result<Self::Output, Self::Error> {
        let report_type = params
            .get("report_type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("summary");
        let period = params
            .get("period")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("daily");

        info!("Generating {} {} report", period, report_type);

        // Simulate report generation
        sleep(Duration::from_secs(1)).await;

        let result = serde_json::json!({
            "report_type": report_type,
            "period": period,
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "sections": vec![
                "Executive Summary",
                "Key Metrics",
                "Detailed Analysis",
                "Recommendations"
            ],
            "page_count": 12
        });

        Ok(JobResult::success(&result).map_err(|e| ToolError::permanent_string(e.to_string()))?)
    }
}

/// Service coordinator that manages the worker lifecycle
struct ServiceCoordinator {
    worker: Arc<ToolWorker<InMemoryIdempotencyStore>>,
    job_queue: Arc<InMemory>,
    cancellation_token: CancellationToken,
}

impl ServiceCoordinator {
    /// Create a new service coordinator
    fn new(config: &Config) -> Self {
        let mut exec_config = ExecutionConfig::default();
        exec_config.max_concurrency = 5;
        exec_config.default_timeout = Duration::from_secs(30);
        exec_config.max_retries = 3;
        exec_config.initial_retry_delay = Duration::from_secs(1);
        exec_config.max_retry_delay = Duration::from_secs(60);
        exec_config.idempotency_ttl = Duration::from_secs(3600);
        exec_config.enable_idempotency = true;

        let app_context = ApplicationContext::from_config(config);
        let worker = Arc::new(ToolWorker::<InMemoryIdempotencyStore>::new(
            exec_config,
            app_context,
        ));

        let job_queue = Arc::new(InMemory::new());
        let cancellation_token = CancellationToken::new();

        Self {
            worker,
            job_queue,
            cancellation_token,
        }
    }

    /// Register all tools with the worker
    fn register_tools(&self) {
        self.worker.register_tool(Arc::new(AnalyticsTool));
        self.worker.register_tool(Arc::new(NotificationTool));
        self.worker.register_tool(Arc::new(ReportTool));
        info!("✅ Registered all tools with worker");
    }

    /// Start the background worker task
    fn start_worker(&self) -> JoinHandle<()> {
        let worker: Arc<ToolWorker<InMemoryIdempotencyStore>> = Arc::clone(&self.worker);
        let queue = Arc::clone(&self.job_queue);
        let token = self.cancellation_token.clone();

        tokio::spawn(async move {
            info!("🚀 Starting background worker task");

            // Run the worker with the queue and cancellation token
            if let Err(e) = worker.run(queue, token.clone()).await {
                error!("Worker error: {}", e);
            }

            info!("👋 Worker task completed");
        })
    }

    /// Enqueue a job for processing
    async fn enqueue_job(&self, job: Job) -> Result<(), Box<dyn Error>> {
        let job_name = job.tool_name.clone();
        self.job_queue.enqueue(job).await?;
        info!("📥 Enqueued job: {}", job_name);
        Ok(())
    }

    /// Gracefully shutdown the service
    async fn shutdown(self, worker_handle: JoinHandle<()>) {
        info!("🛑 Initiating graceful shutdown...");

        // Signal the worker to stop
        self.signal_shutdown();

        // Wait for the worker to finish processing current jobs
        self.wait_for_worker_shutdown(worker_handle).await;

        // Check for remaining jobs
        self.check_remaining_jobs().await;

        info!("✅ Service shutdown complete");
    }

    /// Signal the worker to begin shutdown
    fn signal_shutdown(&self) {
        self.cancellation_token.cancel();
    }

    /// Wait for the worker to complete shutdown with timeout
    async fn wait_for_worker_shutdown(&self, worker_handle: JoinHandle<()>) {
        match timeout(Duration::from_secs(10), worker_handle).await {
            Ok(Ok(())) => info!("✅ Worker shutdown cleanly"),
            Ok(Err(e)) => error!("Worker task panicked: {:?}", e),
            Err(_) => warn!("⚠️ Worker shutdown timed out after 10 seconds"),
        }
    }

    /// Check for any jobs remaining in the queue after shutdown
    async fn check_remaining_jobs(&self) {
        if let Ok(size) = self.job_queue.len().await {
            if size > 0 {
                warn!("⚠️ {} jobs remaining in queue at shutdown", size);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing for better observability
    tracing_subscriber::fmt().with_target(false).init();

    println!("=== riglr-core Service Worker Example ===\n");

    // Create the service coordinator
    let config = Config::from_env();
    let coordinator = ServiceCoordinator::new(&config);

    // Register tools
    coordinator.register_tools();

    // Start the background worker
    let worker_handle = coordinator.start_worker();
    info!("✅ Service started with background worker");

    // Simulate job submission from various sources
    info!("\n📋 Submitting jobs to the queue...");

    // Analytics jobs
    for i in 1_u64..=3 {
        let job = Job::new(
            "analytics",
            &serde_json::json!({
                "data_size": i.saturating_mul(50),
                "type": if i % 2 == 0 { "advanced" } else { "basic" }
            }),
            3,
        )?;
        coordinator.enqueue_job(job).await?;
    }

    // Notification jobs
    for i in 1..=2 {
        let job = Job::new(
            "notify",
            &serde_json::json!({
                "recipient": format!("user_{}", i),
                "message": format!("Task {} completed", i),
                "priority": if i == 1 { "high" } else { "normal" }
            }),
            2,
        )?;
        coordinator.enqueue_job(job).await?;
    }

    // Report generation job (idempotent)
    let report_job = Job::new_idempotent(
        "report",
        &serde_json::json!({
            "report_type": "performance",
            "period": "weekly"
        }),
        3,
        "weekly_performance_report_2024",
    )?;
    coordinator.enqueue_job(report_job.clone()).await?;

    // Submit the same report job again (will be deduplicated by idempotency)
    coordinator.enqueue_job(report_job).await?;

    info!("📊 Enqueued 7 jobs (6 unique + 1 duplicate)");

    // Let the worker process jobs for a while
    info!("\n⏳ Processing jobs for 5 seconds...");
    sleep(Duration::from_secs(5)).await;

    // Add more jobs while worker is running
    info!("\n📋 Adding more jobs while worker is running...");
    for i in 4_u64..=5 {
        let job = Job::new(
            "analytics",
            &serde_json::json!({
                "data_size": i.saturating_mul(30),
                "type": "realtime"
            }),
            3,
        )?;
        coordinator.enqueue_job(job).await?;
    }

    // Let it process a bit more
    sleep(Duration::from_secs(3)).await;

    // Shutdown gracefully
    info!("\n🛑 Initiating shutdown sequence...");
    coordinator.shutdown(worker_handle).await;

    println!("\n🎉 Service worker example completed!");
    println!("\n🔧 Key patterns demonstrated:");
    println!("   • Long-running worker with job queue");
    println!("   • Graceful shutdown with cancellation token");
    println!("   • Background job processing");
    println!("   • Job enqueuing from multiple sources");
    println!("   • Idempotent job processing");
    println!("   • Proper resource cleanup on shutdown");

    Ok(())
}
