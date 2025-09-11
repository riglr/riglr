//! Batch processing utilities

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, warn};

/// Batch processor for collecting items and processing them in batches
#[derive(Debug)]
pub struct BatchProcessor<T> {
    /// Maximum batch size
    max_size: usize,
    /// Maximum age before forcing a batch
    max_age: Duration,
    /// Current batch
    current_batch: Arc<RwLock<VecDeque<(T, Instant)>>>,
    /// Sender for completed batches
    batch_sender: mpsc::UnboundedSender<Vec<T>>,
    /// Receiver for completed batches
    batch_receiver: RwLock<Option<mpsc::UnboundedReceiver<Vec<T>>>>,
}

impl<T: Send + Sync + std::fmt::Debug + 'static> BatchProcessor<T> {
    /// Create a new batch processor
    pub fn new(max_size: usize, max_age: Duration) -> Self {
        let (batch_sender, batch_receiver) = mpsc::unbounded_channel();

        Self {
            max_size,
            max_age,
            current_batch: Arc::new(RwLock::new(VecDeque::new())),
            batch_sender,
            batch_receiver: RwLock::new(Some(batch_receiver)),
        }
    }

    /// Add an item to the current batch
    pub async fn add_item(&self, item: T) -> Result<(), BatchProcessorError> {
        let mut batch = self.current_batch.write().await;
        let now = Instant::now();

        batch.push_back((item, now));

        // Check if we should flush due to size
        if batch.len() >= self.max_size {
            self.flush_batch(&mut batch).await?;
        }

        Ok(())
    }

    /// Start the age-based flushing task
    pub fn start_age_flusher(&self) {
        let current_batch = Arc::clone(&self.current_batch);
        let batch_sender = self.batch_sender.clone();
        let max_age = self.max_age;
        let check_interval = max_age / 4; // Check 4 times per max_age period

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);

            loop {
                interval.tick().await;

                let should_flush = {
                    let batch = current_batch.read().await;
                    if let Some((_, oldest_time)) = batch.front() {
                        oldest_time.elapsed() >= max_age
                    } else {
                        false
                    }
                };

                if should_flush {
                    let mut batch = current_batch.write().await;
                    if !batch.is_empty() {
                        let items: Vec<T> = batch.drain(..).map(|(item, _)| item).collect();
                        if !items.is_empty() {
                            debug!("Age-based flush: {} items", items.len());
                            let send_result = batch_sender.send(items);
                            if let Err(e) = send_result {
                                warn!("Failed to send age-based batch: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    /// Get the next batch (blocks until available)
    pub async fn next_batch(&self) -> Option<Vec<T>> {
        let mut receiver_guard = self.batch_receiver.write().await;
        if let Some(receiver) = receiver_guard.as_mut() {
            let result = receiver.recv().await;
            result
        } else {
            None
        }
    }

    /// Try to get the next batch (non-blocking)
    pub async fn try_next_batch(&self) -> Option<Vec<T>> {
        let mut receiver_guard = self.batch_receiver.write().await;
        if let Some(receiver) = receiver_guard.as_mut() {
            let result = receiver.try_recv().ok();
            result
        } else {
            None
        }
    }

    /// Flush the current batch immediately
    pub async fn flush(&self) -> Result<(), BatchProcessorError> {
        let mut batch = self.current_batch.write().await;
        let result = self.flush_batch(&mut batch).await;
        result
    }

    /// Internal method to flush a batch
    async fn flush_batch(
        &self,
        batch: &mut VecDeque<(T, Instant)>,
    ) -> Result<(), BatchProcessorError> {
        if batch.is_empty() {
            return Ok(());
        }

        let items: Vec<T> = batch.drain(..).map(|(item, _)| item).collect();
        debug!("Flushing batch of {} items", items.len());

        self.batch_sender
            .send(items)
            .map_err(|_| BatchProcessorError::SendFailed)?;

        Ok(())
    }

    /// Get current batch size
    pub async fn current_batch_size(&self) -> usize {
        self.current_batch.read().await.len()
    }

    /// Check if the current batch is empty
    pub async fn is_empty(&self) -> bool {
        self.current_batch.read().await.is_empty()
    }

    /// Get the age of the oldest item in the batch
    pub async fn oldest_item_age(&self) -> Option<Duration> {
        let batch = self.current_batch.read().await;
        batch.front().map(|(_, time)| time.elapsed())
    }
}

/// Errors that can occur during batch processing
#[derive(Debug, thiserror::Error)]
pub enum BatchProcessorError {
    /// Failed to send batch through channel
    #[error("Failed to send batch")]
    SendFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_batch_processor_size_based_flush() {
        let processor: BatchProcessor<String> = BatchProcessor::new(3, Duration::from_secs(10));

        // Add items one by one
        processor.add_item("item1".to_string()).await.unwrap();
        processor.add_item("item2".to_string()).await.unwrap();

        // Should not have flushed yet
        assert_eq!(processor.current_batch_size().await, 2);

        // This should trigger a flush
        processor.add_item("item3".to_string()).await.unwrap();

        // Batch should be empty now
        assert_eq!(processor.current_batch_size().await, 0);

        // Should receive the batch
        let batch = timeout(Duration::from_millis(100), processor.next_batch())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(batch.len(), 3);
        assert_eq!(batch, vec!["item1", "item2", "item3"]);
    }

    #[tokio::test]
    async fn test_batch_processor_age_based_flush() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_millis(100));
        processor.start_age_flusher();

        // Add an item
        processor.add_item("item1".to_string()).await.unwrap();

        // Wait for age-based flush
        let batch = timeout(Duration::from_millis(200), processor.next_batch())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(batch.len(), 1);
        assert_eq!(batch, vec!["item1"]);
    }

    #[tokio::test]
    async fn test_batch_processor_manual_flush() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_secs(10));

        // Add items
        processor.add_item("item1".to_string()).await.unwrap();
        processor.add_item("item2".to_string()).await.unwrap();

        // Manual flush
        processor.flush().await.unwrap();

        // Should receive the batch
        let batch = timeout(Duration::from_millis(100), processor.next_batch())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(batch.len(), 2);
        assert_eq!(batch, vec!["item1", "item2"]);
    }

    #[tokio::test]
    async fn test_flush_empty_batch() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_secs(10));

        // Flush empty batch should succeed
        processor.flush().await.unwrap();

        // Should not receive any batch
        let result = timeout(Duration::from_millis(100), processor.next_batch()).await;
        assert!(result.is_err()); // Should timeout since no batch was sent
    }

    #[tokio::test]
    async fn test_try_next_batch_with_items() {
        let processor: BatchProcessor<String> = BatchProcessor::new(2, Duration::from_secs(10));

        // Add items to trigger a flush
        processor.add_item("item1".to_string()).await.unwrap();
        processor.add_item("item2".to_string()).await.unwrap();

        // Should immediately have a batch available
        let batch = processor.try_next_batch().await.unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(batch, vec!["item1", "item2"]);
    }

    #[tokio::test]
    async fn test_try_next_batch_without_items() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_secs(10));

        // Should return None when no batch is available
        let batch = processor.try_next_batch().await;
        assert!(batch.is_none());
    }

    #[tokio::test]
    async fn test_oldest_item_age_empty_batch() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_secs(10));

        // Empty batch should return None
        let age = processor.oldest_item_age().await;
        assert!(age.is_none());
    }

    #[tokio::test]
    async fn test_oldest_item_age_with_items() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_secs(10));

        // Add item and check age
        processor.add_item("item1".to_string()).await.unwrap();

        // Small delay to ensure some age
        tokio::time::sleep(Duration::from_millis(10)).await;

        let age = processor.oldest_item_age().await;
        assert!(age.is_some());
        assert!(age.unwrap() >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_is_empty_method() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_secs(10));

        // Should be empty initially
        assert!(processor.is_empty().await);

        // Add item
        processor.add_item("item1".to_string()).await.unwrap();
        assert!(!processor.is_empty().await);

        // Flush and should be empty again
        processor.flush().await.unwrap();
        assert!(processor.is_empty().await);
    }

    #[tokio::test]
    async fn test_current_batch_size() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_secs(10));

        // Should be 0 initially
        assert_eq!(processor.current_batch_size().await, 0);

        // Add items
        processor.add_item("item1".to_string()).await.unwrap();
        assert_eq!(processor.current_batch_size().await, 1);

        processor.add_item("item2".to_string()).await.unwrap();
        assert_eq!(processor.current_batch_size().await, 2);

        // Flush and should be 0 again
        processor.flush().await.unwrap();
        assert_eq!(processor.current_batch_size().await, 0);
    }

    #[tokio::test]
    async fn test_age_flusher_with_empty_batch() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_millis(50));
        processor.start_age_flusher();

        // Wait longer than max_age but no items added
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should not receive any batch
        let result = timeout(Duration::from_millis(50), processor.next_batch()).await;
        assert!(result.is_err()); // Should timeout since no batch was sent
    }

    #[tokio::test]
    async fn test_age_flusher_multiple_items() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_millis(100));
        processor.start_age_flusher();

        // Add items at different times
        processor.add_item("item1".to_string()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        processor.add_item("item2".to_string()).await.unwrap();

        // Wait for age-based flush
        let batch = timeout(Duration::from_millis(200), processor.next_batch())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(batch.len(), 2);
        assert_eq!(batch, vec!["item1", "item2"]);
    }

    #[tokio::test]
    async fn test_next_batch_with_dropped_receiver() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_secs(10));

        // Simulate dropping the receiver by taking it out
        {
            let mut receiver_guard = processor.batch_receiver.write().await;
            let _ = receiver_guard.take();
        }

        // next_batch should return None when receiver is taken
        let result = processor.next_batch().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_try_next_batch_with_dropped_receiver() {
        let processor: BatchProcessor<String> = BatchProcessor::new(10, Duration::from_secs(10));

        // Simulate dropping the receiver by taking it out
        {
            let mut receiver_guard = processor.batch_receiver.write().await;
            let _ = receiver_guard.take();
        }

        // try_next_batch should return None when receiver is taken
        let result = processor.try_next_batch().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_batch_processor_new_initialization() {
        let max_size = 5;
        let max_age = Duration::from_secs(30);
        let processor: BatchProcessor<String> = BatchProcessor::new(max_size, max_age);

        // Verify initial state
        assert_eq!(processor.max_size, max_size);
        assert_eq!(processor.max_age, max_age);
        assert!(processor.is_empty().await);
        assert_eq!(processor.current_batch_size().await, 0);
        assert!(processor.oldest_item_age().await.is_none());
    }

    #[tokio::test]
    async fn test_size_based_flush_exact_boundary() {
        let processor: BatchProcessor<String> = BatchProcessor::new(1, Duration::from_secs(10));

        // Adding single item should trigger immediate flush
        processor.add_item("item1".to_string()).await.unwrap();

        // Batch should be empty immediately
        assert_eq!(processor.current_batch_size().await, 0);

        // Should receive the batch
        let batch = timeout(Duration::from_millis(100), processor.next_batch())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(batch.len(), 1);
        assert_eq!(batch, vec!["item1"]);
    }

    #[tokio::test]
    async fn test_multiple_size_based_flushes() {
        let processor: BatchProcessor<String> = BatchProcessor::new(2, Duration::from_secs(10));

        // First batch
        processor.add_item("item1".to_string()).await.unwrap();
        processor.add_item("item2".to_string()).await.unwrap();

        // Second batch
        processor.add_item("item3".to_string()).await.unwrap();
        processor.add_item("item4".to_string()).await.unwrap();

        // Should receive first batch
        let batch1 = timeout(Duration::from_millis(100), processor.next_batch())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(batch1, vec!["item1", "item2"]);

        // Should receive second batch
        let batch2 = timeout(Duration::from_millis(100), processor.next_batch())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(batch2, vec!["item3", "item4"]);
    }

    #[tokio::test]
    async fn test_age_flusher_check_interval_calculation() {
        let max_age = Duration::from_millis(400);
        let processor: BatchProcessor<String> = BatchProcessor::new(10, max_age);

        // The check interval should be max_age / 4
        let _expected_check_interval = max_age / 4; // 100ms

        processor.start_age_flusher();
        processor.add_item("item1".to_string()).await.unwrap();

        // Should flush within approximately check_interval * some_factor
        // Allow some margin for timing variability
        let batch = timeout(Duration::from_millis(500), processor.next_batch())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(batch.len(), 1);
        assert_eq!(batch, vec!["item1"]);
    }

    #[test]
    fn test_batch_processor_error_display() {
        let error = BatchProcessorError::SendFailed;
        assert_eq!(error.to_string(), "Failed to send batch");
    }

    #[test]
    fn test_batch_processor_error_debug() {
        let error = BatchProcessorError::SendFailed;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("SendFailed"));
    }
}
