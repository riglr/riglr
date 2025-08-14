use std::sync::Arc;
use tracing::info;

use crate::core::StreamManager;

/// Extended worker that can process streaming events
pub struct StreamingToolWorker {
    /// Worker ID
    worker_id: String,
    /// Stream manager
    stream_manager: Arc<StreamManager>,
}

impl StreamingToolWorker {
    /// Create a new streaming tool worker
    pub fn new(
        worker_id: String,
        stream_manager: Arc<StreamManager>,
    ) -> Self {
        Self {
            worker_id,
            stream_manager,
        }
    }
    
    /// Start processing events
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting streaming tool worker: {}", self.worker_id);
        
        // Start the stream manager if not already running
        self.stream_manager.start_all().await
            .map_err(|e| format!("Failed to start streams: {}", e))?;
        
        // Process events
        self.stream_manager.process_events().await
            .map_err(|e| format!("Event processing error: {}", e))?;
        
        Ok(())
    }
    
    /// Stop the worker
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Stopping streaming tool worker: {}", self.worker_id);
        
        // Stop streams
        self.stream_manager.stop_all().await
            .map_err(|e| format!("Failed to stop streams: {}", e))?;
        
        Ok(())
    }
}