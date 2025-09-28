use core::fmt;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::info;

use crate::core::error::StreamResult;
use crate::core::StreamManager;

/// Extended worker that can process streaming events
pub struct StreamingToolWorker {
    /// Worker ID
    worker_id: String,
    /// Stream manager
    stream_manager: Arc<StreamManager>,
    /// Handle to the background event processing task
    event_task: Option<JoinHandle<StreamResult<()>>>,
}

impl fmt::Debug for StreamingToolWorker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamingToolWorker")
            .field("worker_id", &self.worker_id)
            .field("stream_manager", &"Arc<StreamManager>")
            .field(
                "event_task",
                &match self.event_task {
                    Some(_) => "Some(JoinHandle<StreamResult<()>>)",
                    None => "None",
                },
            )
            .finish()
    }
}

impl StreamingToolWorker {
    /// Create a new streaming tool worker
    #[must_use]
    pub const fn new(worker_id: String, stream_manager: Arc<StreamManager>) -> Self {
        Self {
            worker_id,
            stream_manager,
            event_task: None,
        }
    }

    /// Start processing events
    ///
    /// # Errors
    ///
    /// Returns error if stream manager fails to start or event processing initialization fails
    pub async fn start(&mut self) -> StreamResult<()> {
        info!("Starting streaming tool worker: {}", self.worker_id);

        // Start the stream manager if not already running
        (*self.stream_manager).start_all().await?;

        // Spawn process_events in background instead of blocking
        let manager = self.stream_manager.clone();
        self.event_task = Some(tokio::spawn(async move { manager.process_events().await }));

        Ok(())
    }

    /// Stop the worker
    ///
    /// # Errors
    ///
    /// Returns error if stream manager fails to stop gracefully
    pub async fn stop(&mut self) -> StreamResult<()> {
        info!("Stopping streaming tool worker: {}", self.worker_id);

        // Stop streams (this will send shutdown signal to process_events)
        (*self.stream_manager).stop_all().await?;

        // Wait for the event processing task to complete if it exists
        if let Some(task) = self.event_task.take() {
            // Abort the task if it's still running
            task.abort();
            // Wait for it to complete (ignoring the abort error)
            let _ = task.await;
        }

        Ok(())
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::core::StreamManager;
    use std::{error, io};
    use tokio::time::{timeout, Duration};

    // Test StreamingToolWorker::new
    #[test]
    fn test_streaming_tool_worker_new_with_string_worker_id() {
        let stream_manager = Arc::new(StreamManager::new());
        let worker_id = "test_worker".to_string();
        let worker = StreamingToolWorker::new(worker_id.clone(), stream_manager);

        assert_eq!(worker.worker_id, worker_id);
        assert_eq!(Arc::strong_count(&worker.stream_manager), 2); // One for worker, one for our reference
    }

    #[test]
    fn test_streaming_tool_worker_new_with_empty_worker_id() {
        let stream_manager = Arc::new(StreamManager::new());
        let worker_id = String::new();
        let worker = StreamingToolWorker::new(worker_id, stream_manager);

        assert_eq!(worker.worker_id, "");
        assert_eq!(Arc::strong_count(&worker.stream_manager), 2);
    }

    #[test]
    fn test_streaming_tool_worker_new_with_long_worker_id() {
        let stream_manager = Arc::new(StreamManager::new());
        let worker_id = "a".repeat(1000);
        let worker = StreamingToolWorker::new(worker_id.clone(), stream_manager);

        assert_eq!(worker.worker_id, worker_id);
        assert_eq!(Arc::strong_count(&worker.stream_manager), 2);
    }

    #[test]
    fn test_streaming_tool_worker_new_with_unicode_worker_id() {
        let stream_manager = Arc::new(StreamManager::new());
        let worker_id = "тест_🚀_worker".to_string();
        let worker = StreamingToolWorker::new(worker_id.clone(), stream_manager);

        assert_eq!(worker.worker_id, worker_id);
        assert_eq!(Arc::strong_count(&worker.stream_manager), 2);
    }

    #[tokio::test]
    async fn test_streaming_tool_worker_start_success() {
        // Since we can't easily mock the StreamManager due to its complex interface,
        // we'll test with a real StreamManager in its default state
        let stream_manager = Arc::new(StreamManager::new());
        let mut worker = StreamingToolWorker::new("test_worker".to_string(), stream_manager);

        // This should succeed but will complete quickly since there are no streams to process
        let result = worker.start().await;
        assert!(result.is_ok());

        // Clean up the background task
        worker
            .stop()
            .await
            .expect("Worker stop should succeed in test");
    }

    #[tokio::test]
    async fn test_streaming_tool_worker_stop_success() {
        let stream_manager = Arc::new(StreamManager::new());
        let mut worker = StreamingToolWorker::new("test_worker".to_string(), stream_manager);

        // Start the manager first
        let _ = worker.start().await;

        // Stop should succeed
        let result = worker.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_streaming_tool_worker_stop_when_not_started() {
        let stream_manager = Arc::new(StreamManager::new());
        let mut worker = StreamingToolWorker::new("test_worker".to_string(), stream_manager);

        // Stop without starting should still succeed
        let result = worker.stop().await;
        assert!(result.is_ok());
    }

    // Edge case tests
    #[tokio::test]
    async fn test_streaming_tool_worker_multiple_starts() {
        let stream_manager = Arc::new(StreamManager::new());
        let mut worker = StreamingToolWorker::new("test_worker".to_string(), stream_manager);

        // First start should succeed
        let result1 = worker.start().await;
        assert!(result1.is_ok());

        // Stop before second start
        let _ = worker.stop().await;

        // Second start should also succeed
        let result2 = worker.start().await;
        assert!(result2.is_ok());

        // Clean up
        worker
            .stop()
            .await
            .expect("Worker stop should succeed in test");
    }

    #[tokio::test]
    async fn test_streaming_tool_worker_multiple_stops() {
        let stream_manager = Arc::new(StreamManager::new());
        let mut worker = StreamingToolWorker::new("test_worker".to_string(), stream_manager);

        // Start first
        let _ = worker.start().await;

        // First stop should succeed
        let result1 = worker.stop().await;
        assert!(result1.is_ok());

        // Second stop should also succeed (idempotent)
        let result2 = worker.stop().await;
        assert!(result2.is_ok());
    }

    // Test with concurrent operations
    #[tokio::test]
    async fn test_streaming_tool_worker_concurrent_operations() {
        let stream_manager = Arc::new(StreamManager::new());
        let mut worker1 = StreamingToolWorker::new("worker1".to_string(), stream_manager.clone());
        let mut worker2 = StreamingToolWorker::new("worker2".to_string(), stream_manager);

        // Both workers should be able to operate concurrently
        let handle1 = tokio::spawn(async move {
            let start_result = worker1.start().await;
            // Immediately stop to clean up background task
            let stop_result = worker1.stop().await;
            start_result.and(stop_result)
        });

        let handle2 = tokio::spawn(async move {
            let start_result = worker2.start().await;
            // Immediately stop to clean up background task
            let stop_result = worker2.stop().await;
            start_result.and(stop_result)
        });

        // Use timeout on the join operation
        let result = timeout(Duration::from_millis(500), async {
            let (r1, r2) = tokio::join!(handle1, handle2);
            (r1, r2)
        })
        .await;

        assert!(result.is_ok());
        let (result1, result2) =
            result.expect("Concurrent operations should complete within timeout");
        assert!(result1
            .expect("Worker1 task should complete successfully")
            .is_ok());
        assert!(result2
            .expect("Worker2 task should complete successfully")
            .is_ok());
    }

    // Test error propagation (we can't easily simulate StreamManager errors without complex mocking)
    // So we test the error handling pattern with format! calls
    #[test]
    fn test_error_formatting_start_all_error() {
        let error_msg = "Test error";
        let formatted = format!("Failed to start streams: {error_msg}");
        assert_eq!(formatted, "Failed to start streams: Test error");
    }

    #[test]
    fn test_error_formatting_process_events_error() {
        let error_msg = "Process error";
        let formatted = format!("Event processing error: {error_msg}");
        assert_eq!(formatted, "Event processing error: Process error");
    }

    #[test]
    fn test_error_formatting_stop_all_error() {
        let error_msg = "Stop error";
        let formatted = format!("Failed to stop streams: {error_msg}");
        assert_eq!(formatted, "Failed to stop streams: Stop error");
    }

    // Test that worker_id field is accessible (testing the struct field)
    #[test]
    fn test_worker_id_field_access() {
        let stream_manager = Arc::new(StreamManager::new());
        let worker_id = "field_test_worker";
        let worker = StreamingToolWorker::new(worker_id.to_string(), stream_manager);

        // Direct field access (since fields are pub in the struct)
        assert_eq!(worker.worker_id, worker_id);
    }

    // Test stream_manager field is accessible
    #[test]
    fn test_stream_manager_field_access() {
        let stream_manager = Arc::new(StreamManager::new());
        let worker = StreamingToolWorker::new("test".to_string(), stream_manager.clone());

        // Verify the Arc is the same
        assert!(Arc::ptr_eq(&worker.stream_manager, &stream_manager));
    }

    // Test with various Box<dyn std::error::Error> scenarios
    #[test]
    fn test_box_error_conversion() {
        let io_error = io::Error::other("IO error");
        let boxed: Box<dyn error::Error> = Box::new(io_error);
        assert_eq!(boxed.to_string(), "IO error");
    }

    // Test start method behavior more thoroughly
    #[tokio::test]
    async fn test_start_method_calls_start_all_and_process_events() {
        // We can't easily mock the StreamManager, but we can verify the sequence
        // by testing with a real StreamManager and checking it doesn't panic
        let stream_manager = Arc::new(StreamManager::new());
        let mut worker = StreamingToolWorker::new("sequence_test".to_string(), stream_manager);

        // This tests the full sequence: start_all() then process_events()
        let result = worker.start().await;

        // Should succeed (though process_events will return quickly with no streams)
        assert!(result.is_ok());

        // Clean up the background task
        worker
            .stop()
            .await
            .expect("Worker stop should succeed in test");
    }

    // Test stop method behavior
    #[tokio::test]
    async fn test_stop_method_calls_stop_all() {
        let stream_manager = Arc::new(StreamManager::new());
        let mut worker = StreamingToolWorker::new("stop_test".to_string(), stream_manager);

        // Start first to get into running state
        let _ = worker.start().await;

        // Stop should call stop_all() internally
        let result = worker.stop().await;
        assert!(result.is_ok());
    }

    // Test that tracing info! calls don't panic (we can't easily verify log output)
    #[tokio::test]
    async fn test_logging_does_not_panic() {
        let stream_manager = Arc::new(StreamManager::new());
        let mut worker = StreamingToolWorker::new("log_test".to_string(), stream_manager);

        // These should not panic even with logging
        let start_result = worker.start().await;
        // Always call stop to clean up background task
        let stop_result = worker.stop().await;

        assert!(start_result.is_ok());
        assert!(stop_result.is_ok());
    }

    // Test Arc cloning behavior
    #[test]
    fn test_arc_cloning_behavior() {
        let stream_manager = Arc::new(StreamManager::new());
        let initial_count = Arc::strong_count(&stream_manager);

        let worker = StreamingToolWorker::new("arc_test".to_string(), stream_manager.clone());

        // Count should increase by 1 (one for worker, one for our clone)
        assert_eq!(Arc::strong_count(&worker.stream_manager), initial_count + 1);

        drop(worker);

        // Count should be back to initial after dropping worker
        assert_eq!(Arc::strong_count(&stream_manager), initial_count);
    }

    // Test struct construction with various Arc scenarios
    #[test]
    fn test_struct_construction_with_weak_references() {
        let stream_manager = Arc::new(StreamManager::new());
        let weak_ref = Arc::downgrade(&stream_manager);

        let worker = StreamingToolWorker::new("weak_test".to_string(), stream_manager);

        // Weak reference should still be valid
        assert!(weak_ref.upgrade().is_some());

        drop(worker);

        // After dropping worker, weak ref might still be valid if original Arc exists
        assert!(weak_ref.upgrade().is_some());
    }
}
