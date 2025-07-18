//! Batch processing utilities

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use tracing::{debug, warn};

/// Batch processor for collecting items and processing them in batches
pub struct BatchProcessor<T> {
    /// Maximum batch size
    max_size: usize,
    /// Maximum age before forcing a batch
    max_age: Duration,
    /// Current batch
    current_batch: RwLock<VecDeque<(T, Instant)>>,
    /// Sender for completed batches
    batch_sender: mpsc::UnboundedSender<Vec<T>>,
    /// Receiver for completed batches
    batch_receiver: RwLock<Option<mpsc::UnboundedReceiver<Vec<T>>>>,
}

impl<T: Send + 'static> BatchProcessor<T> {
    /// Create a new batch processor
    pub fn new(max_size: usize, max_age: Duration) -> Self {
        let (batch_sender, batch_receiver) = mpsc::unbounded_channel();

        Self {
            max_size,
            max_age,
            current_batch: RwLock::new(VecDeque::new()),
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
        let current_batch = self.current_batch.clone();
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
                            if let Err(e) = batch_sender.send(items) {
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
            receiver.recv().await
        } else {
            None
        }
    }

    /// Try to get the next batch (non-blocking)
    pub async fn try_next_batch(&self) -> Option<Vec<T>> {
        let mut receiver_guard = self.batch_receiver.write().await;
        if let Some(receiver) = receiver_guard.as_mut() {
            receiver.try_recv().ok()
        } else {
            None
        }
    }

    /// Flush the current batch immediately
    pub async fn flush(&self) -> Result<(), BatchProcessorError> {
        let mut batch = self.current_batch.write().await;
        self.flush_batch(&mut batch).await
    }

    /// Internal method to flush a batch
    async fn flush_batch(&self, batch: &mut VecDeque<(T, Instant)>) -> Result<(), BatchProcessorError> {
        if batch.is_empty() {
            return Ok(());
        }

        let items: Vec<T> = batch.drain(..).map(|(item, _)| item).collect();
        debug!("Flushing batch of {} items", items.len());
        
        self.batch_sender.send(items)
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
    #[error("Failed to send batch")]
    SendFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_batch_processor_size_based_flush() {
        let processor = BatchProcessor::new(3, Duration::from_secs(10));
        
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
        let processor = BatchProcessor::new(10, Duration::from_millis(100));
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
        let processor = BatchProcessor::new(10, Duration::from_secs(10));
        
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
}