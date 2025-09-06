//! Service-oriented example showing a long-running ToolWorker with job queue
//!
//! This demonstrates the recommended pattern for production services:
//! - Long-running worker task with graceful shutdown
//! - Job queue for decoupling job submission from processing
//! - Background processing with cancellation support
//! - Proper resource cleanup

use async_trait::async_trait;
use riglr_config::Config;
use riglr_core::{
    idempotency::InMemoryIdempotencyStore,
    provider::ApplicationContext,
    queue::{InMemoryJobQueue, JobQueue},
    ExecutionConfig, Job, JobResult, Tool, ToolError, ToolWorker,
};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

/// Example analytics tool that might take some time to process
#[derive(Clone)]
struct AnalyticsTool;

#[async_trait]
impl Tool for AnalyticsTool {
    async fn execute(
        &self,
        params: serde_json::Value,
        _app_context: &ApplicationContext,
    ) -> Result<JobResult, ToolError> {
        let data_size = params["data_size"].as_u64().unwrap_or(100);
        let analysis_type = params["type"].as_str().unwrap_or("basic");
        
        info!("Starting {} analysis on {} data points", analysis_type, data_size);
        
        // Simulate processing time based on data size
        let processing_time = std::time::Duration::from_millis(data_size * 10);
        tokio::time::sleep(processing_time).await;
        
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
        
        Ok(JobResult::success(&result)
            .map_err(|e| ToolError::permanent_string(e.to_string()))?)
    }
    
    fn name(&self) -> &str {
        "analytics"
    }
    
    fn description(&self) -> &str {
        "Performs data analytics with simulated processing time"
    }
}

/// Example notification tool
#[derive(Clone)]
struct NotificationTool;

#[async_trait]
impl Tool for NotificationTool {
    async fn execute(
        &self,
        params: serde_json::Value,
        _app_context: &ApplicationContext,
    ) -> Result<JobResult, ToolError> {
        let recipient = params["recipient"].as_str().unwrap_or("admin");
        let message = params["message"].as_str().unwrap_or("Notification");
        let priority = params["priority"].as_str().unwrap_or("normal");
        
        info!("Sending {} priority notification to {}", priority, recipient);
        
        // Simulate notification sending
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        let result = serde_json::json!({
            "status": "sent",
            "recipient": recipient,
            "message": message,
            "priority": priority,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        Ok(JobResult::success(&result)
            .map_err(|e| ToolError::permanent_string(e.to_string()))?)
    }
    
    fn name(&self) -> &str {
        "notify"
    }
    
    fn description(&self) -> &str {
        "Sends notifications to users"
    }
}

/// Example report generation tool
#[derive(Clone)]
struct ReportTool;

#[async_trait]
impl Tool for ReportTool {
    async fn execute(
        &self,
        params: serde_json::Value,
        _app_context: &ApplicationContext,
    ) -> Result<JobResult, ToolError> {
        let report_type = params["report_type"].as_str().unwrap_or("summary");
        let period = params["period"].as_str().unwrap_or("daily");
        
        info!("Generating {} {} report", period, report_type);
        
        // Simulate report generation
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        
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
        
        Ok(JobResult::success(&result)
            .map_err(|e| ToolError::permanent_string(e.to_string()))?)
    }
    
    fn name(&self) -> &str {
        "report"
    }
    
    fn description(&self) -> &str {
        "Generates various types of reports"
    }
}

/// Service coordinator that manages the worker lifecycle
struct ServiceCoordinator {
    worker: Arc<ToolWorker<InMemoryIdempotencyStore>>,
    job_queue: Arc<InMemoryJobQueue>,
    cancellation_token: CancellationToken,
}

impl ServiceCoordinator {
    /// Create a new service coordinator
    fn new(config: Config) -> Self {
        let exec_config = ExecutionConfig {
            max_concurrency: 5,
            default_timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            initial_retry_delay: std::time::Duration::from_secs(1),
            max_retry_delay: std::time::Duration::from_secs(60),
            idempotency_ttl: std::time::Duration::from_secs(3600),
            enable_idempotency: true,
        };
        
        let app_context = ApplicationContext::from_config(&config);
        let worker = Arc::new(ToolWorker::<InMemoryIdempotencyStore>::new(
            exec_config,
            app_context,
        ));
        
        let job_queue = Arc::new(InMemoryJobQueue::new());
        let cancellation_token = CancellationToken::new();
        
        Self {
            worker,
            job_queue,
            cancellation_token,
        }
    }
    
    /// Register all tools with the worker
    async fn register_tools(&self) {
        self.worker.register_tool(Arc::new(AnalyticsTool)).await;
        self.worker.register_tool(Arc::new(NotificationTool)).await;
        self.worker.register_tool(Arc::new(ReportTool)).await;
        info!("✅ Registered all tools with worker");
    }
    
    /// Start the background worker task
    fn start_worker(&self) -> tokio::task::JoinHandle<()> {
        let worker = Arc::clone(&self.worker);
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
    async fn enqueue_job(&self, job: Job) -> Result<(), Box<dyn std::error::Error>> {
        let job_name = job.tool_name.clone();
        self.job_queue.enqueue(job).await?;
        info!("📥 Enqueued job: {}", job_name);
        Ok(())
    }
    
    /// Gracefully shutdown the service
    async fn shutdown(self, worker_handle: tokio::task::JoinHandle<()>) {
        info!("🛑 Initiating graceful shutdown...");
        
        // Signal the worker to stop
        self.cancellation_token.cancel();
        
        // Wait for the worker to finish processing current jobs
        match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            worker_handle
        ).await {
            Ok(Ok(())) => info!("✅ Worker shutdown cleanly"),
            Ok(Err(e)) => error!("Worker task panicked: {:?}", e),
            Err(_) => warn!("⚠️ Worker shutdown timed out after 10 seconds"),
        }
        
        // Check if there are any remaining jobs in the queue
        if let Ok(size) = self.job_queue.len().await {
            if size > 0 {
                warn!("⚠️ {} jobs remaining in queue at shutdown", size);
            }
        }
        
        info!("✅ Service shutdown complete");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better observability
    tracing_subscriber::fmt()
        .with_target(false)
        .init();
    
    println!("=== riglr-core Service Worker Example ===\n");
    
    // Create the service coordinator
    let config = Config::from_env();
    let coordinator = ServiceCoordinator::new((*config).clone());
    
    // Register tools
    coordinator.register_tools().await;
    
    // Start the background worker
    let worker_handle = coordinator.start_worker();
    info!("✅ Service started with background worker");
    
    // Simulate job submission from various sources
    info!("\n📋 Submitting jobs to the queue...");
    
    // Analytics jobs
    for i in 1..=3 {
        let job = Job::new(
            "analytics",
            &serde_json::json!({
                "data_size": i * 50,
                "type": if i % 2 == 0 { "advanced" } else { "basic" }
            }),
            3
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
            2
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
        "weekly_performance_report_2024"
    )?;
    coordinator.enqueue_job(report_job.clone()).await?;
    
    // Submit the same report job again (will be deduplicated by idempotency)
    coordinator.enqueue_job(report_job).await?;
    
    info!("📊 Enqueued 7 jobs (6 unique + 1 duplicate)");
    
    // Let the worker process jobs for a while
    info!("\n⏳ Processing jobs for 5 seconds...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    
    // Add more jobs while worker is running
    info!("\n📋 Adding more jobs while worker is running...");
    for i in 4..=5 {
        let job = Job::new(
            "analytics",
            &serde_json::json!({
                "data_size": i * 30,
                "type": "realtime"
            }),
            3
        )?;
        coordinator.enqueue_job(job).await?;
    }
    
    // Let it process a bit more
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    
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