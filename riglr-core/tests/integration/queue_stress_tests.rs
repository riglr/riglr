//! Comprehensive stress and performance tests for queue implementations
//!
//! These tests verify that queue implementations can handle high-concurrency,
//! high-throughput scenarios while maintaining correctness.

use riglr_core::jobs::Job;
use riglr_core::queue::{InMemoryJobQueue, JobQueue};
use serde_json::json;
use std::sync::{Arc, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use tokio::sync::Barrier;
use std::collections::HashMap;

/// Test high-volume producer-consumer patterns
#[tokio::test]
async fn test_high_volume_producer_consumer() {
    // Skip this test when running under tarpaulin to avoid timeout
    if std::env::var("TARPAULIN_RUN").is_ok() || std::env::var("SKIP_SLOW_TESTS").is_ok() {
        println!("Skipping high volume test in coverage run");
        return;
    }

    let queue = Arc::new(InMemoryJobQueue::new());
    let jobs_to_produce = 10000;
    let num_producers = 10;
    let num_consumers = 5;
    
    let produced_count = Arc::new(AtomicU64::new(0));
    let consumed_count = Arc::new(AtomicU64::new(0));
    let stop_consumers = Arc::new(AtomicBool::new(false));

    // Start producers
    let mut producer_handles = vec![];
    for producer_id in 0..num_producers {
        let queue_clone = queue.clone();
        let produced_count_clone = produced_count.clone();
        let handle = tokio::spawn(async move {
            let jobs_per_producer = jobs_to_produce / num_producers;
            for i in 0..jobs_per_producer {
                let job = Job::new(
                    format!("job_p{}_j{}", producer_id, i),
                    &json!({
                        "producer_id": producer_id,
                        "job_index": i,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                    }),
                    0,
                ).unwrap();
                
                queue_clone.enqueue(job).await.unwrap();
                produced_count_clone.fetch_add(1, Ordering::Relaxed);
                
                // Small yield to allow other tasks to run
                if i % 100 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        });
        producer_handles.push(handle);
    }

    // Start consumers
    let mut consumer_handles = vec![];
    for consumer_id in 0..num_consumers {
        let queue_clone = queue.clone();
        let consumed_count_clone = consumed_count.clone();
        let stop_consumers_clone = stop_consumers.clone();
        
        let handle = tokio::spawn(async move {
            let mut local_consumed = 0;
            while !stop_consumers_clone.load(Ordering::Relaxed) {
                match queue_clone.dequeue_with_timeout(Duration::from_millis(100)).await {
                    Ok(Some(_job)) => {
                        local_consumed += 1;
                        consumed_count_clone.fetch_add(1, Ordering::Relaxed);
                    }
                    Ok(None) => {
                        // Timeout, check if we should continue
                        tokio::task::yield_now().await;
                    }
                    Err(_) => break,
                }
            }
            local_consumed
        });
        consumer_handles.push(handle);
    }

    // Wait for all producers to finish
    for handle in producer_handles {
        handle.await.unwrap();
    }

    // Give consumers time to process remaining jobs
    let mut attempts = 0;
    while consumed_count.load(Ordering::Relaxed) < jobs_to_produce && attempts < 100 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }

    // Stop consumers
    stop_consumers.store(true, Ordering::Relaxed);

    // Wait for consumers to finish
    let mut total_consumed = 0;
    for handle in consumer_handles {
        total_consumed += handle.await.unwrap();
    }

    // Verify all jobs were produced and consumed
    assert_eq!(produced_count.load(Ordering::Relaxed), jobs_to_produce);
    assert_eq!(consumed_count.load(Ordering::Relaxed), jobs_to_produce);
    assert_eq!(total_consumed, jobs_to_produce);
    assert!(queue.is_empty().await.unwrap());
}

/// Test concurrent access patterns with mixed operations
#[tokio::test]
async fn test_concurrent_mixed_operations() {
    let queue = Arc::new(InMemoryJobQueue::new());
    let num_threads = 20;
    let operations_per_thread = 100;
    
    let enqueue_count = Arc::new(AtomicU64::new(0));
    let dequeue_count = Arc::new(AtomicU64::new(0));
    let len_check_count = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let queue_clone = queue.clone();
        let enqueue_count_clone = enqueue_count.clone();
        let dequeue_count_clone = dequeue_count.clone();
        let len_check_count_clone = len_check_count.clone();

        let handle = tokio::spawn(async move {
            for op_id in 0..operations_per_thread {
                match op_id % 4 {
                    0 => {
                        // Enqueue operation
                        let job = Job::new(
                            format!("thread_{}_op_{}", thread_id, op_id),
                            &json!({"thread": thread_id, "op": op_id}),
                            0,
                        ).unwrap();
                        
                        if queue_clone.enqueue(job).await.is_ok() {
                            enqueue_count_clone.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    1 => {
                        // Dequeue operation
                        if let Ok(Some(_)) = queue_clone.dequeue_with_timeout(Duration::from_millis(10)).await {
                            dequeue_count_clone.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    2 => {
                        // Length check operation
                        if queue_clone.len().await.is_ok() {
                            len_check_count_clone.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    3 => {
                        // Empty check operation
                        let _ = queue_clone.is_empty().await;
                    }
                    _ => unreachable!(),
                }
                
                // Small yield every few operations
                if op_id % 10 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify operations were executed
    assert!(enqueue_count.load(Ordering::Relaxed) > 0);
    assert!(len_check_count.load(Ordering::Relaxed) > 0);
    
    // Final queue state should be consistent
    let final_len = queue.len().await.unwrap();
    let enqueued = enqueue_count.load(Ordering::Relaxed);
    let dequeued = dequeue_count.load(Ordering::Relaxed);
    
    assert_eq!(final_len as u64, enqueued - dequeued);
}

/// Test queue behavior under memory pressure
#[tokio::test]
async fn test_memory_pressure_handling() {
    let queue = Arc::new(InMemoryJobQueue::new());
    let num_large_jobs = 1000;
    
    // Create jobs with large payloads
    let mut enqueue_handles = vec![];
    for i in 0..num_large_jobs {
        let queue_clone = queue.clone();
        let handle = tokio::spawn(async move {
            // Create a job with large data payload
            let large_data = (0..1000).map(|j| format!("data_item_{}_{}", i, j)).collect::<Vec<_>>();
            let job = Job::new(
                format!("large_job_{}", i),
                &json!({
                    "job_id": i,
                    "large_payload": large_data,
                    "metadata": {
                        "size": "large",
                        "test": "memory_pressure"
                    }
                }),
                0,
            ).unwrap();
            
            queue_clone.enqueue(job).await.unwrap();
        });
        enqueue_handles.push(handle);
    }

    // Wait for all enqueues
    for handle in enqueue_handles {
        handle.await.unwrap();
    }

    assert_eq!(queue.len().await.unwrap(), num_large_jobs);

    // Now dequeue all jobs concurrently
    let mut dequeue_handles = vec![];
    let successful_dequeues = Arc::new(AtomicU64::new(0));
    
    for _ in 0..10 { // 10 concurrent dequeuers
        let queue_clone = queue.clone();
        let successful_dequeues_clone = successful_dequeues.clone();
        let handle = tokio::spawn(async move {
            let mut local_count = 0;
            while let Ok(Some(job)) = queue_clone.dequeue_with_timeout(Duration::from_millis(100)).await {
                // Verify job integrity
                if job.params["job_id"].is_number() && 
                   job.params["large_payload"].is_array() &&
                   job.params["large_payload"].as_array().unwrap().len() == 1000 {
                    local_count += 1;
                    successful_dequeues_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
            local_count
        });
        dequeue_handles.push(handle);
    }

    // Wait for all dequeues
    let mut total_dequeued = 0;
    for handle in dequeue_handles {
        total_dequeued += handle.await.unwrap();
    }

    assert_eq!(total_dequeued, num_large_jobs);
    assert_eq!(successful_dequeues.load(Ordering::Relaxed), num_large_jobs as u64);
    assert!(queue.is_empty().await.unwrap());
}

/// Test queue ordering under high concurrency
#[tokio::test]
async fn test_fifo_ordering_under_concurrency() {
    let queue = Arc::new(InMemoryJobQueue::new());
    let num_producers = 5;
    let jobs_per_producer = 200;
    
    // Track job insertion order
    let insertion_order = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let barrier = Arc::new(Barrier::new(num_producers));

    // Start all producers simultaneously
    let mut producer_handles = vec![];
    for producer_id in 0..num_producers {
        let queue_clone = queue.clone();
        let insertion_order_clone = insertion_order.clone();
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            // Wait for all producers to be ready
            barrier_clone.wait().await;
            
            for job_id in 0..jobs_per_producer {
                let global_job_id = producer_id * jobs_per_producer + job_id;
                let job = Job::new(
                    format!("ordered_job_{}", global_job_id),
                    &json!({
                        "producer_id": producer_id,
                        "job_id": job_id,
                        "global_id": global_job_id
                    }),
                    0,
                ).unwrap();
                
                // Record insertion time
                {
                    let mut order = insertion_order_clone.lock().await;
                    order.push(global_job_id);
                }
                
                queue_clone.enqueue(job).await.unwrap();
            }
        });
        producer_handles.push(handle);
    }

    // Wait for all producers
    for handle in producer_handles {
        handle.await.unwrap();
    }

    // Now dequeue all jobs and verify FIFO ordering is approximately maintained
    let mut dequeued_order = vec![];
    while let Ok(Some(job)) = queue.dequeue_with_timeout(Duration::from_millis(100)).await {
        let global_id = job.params["global_id"].as_u64().unwrap();
        dequeued_order.push(global_id);
    }

    // Verify we got all jobs
    assert_eq!(dequeued_order.len(), num_producers * jobs_per_producer);

    // Verify no duplicates
    let mut sorted_dequeued = dequeued_order.clone();
    sorted_dequeued.sort();
    let expected: Vec<u64> = (0..(num_producers * jobs_per_producer) as u64).collect();
    assert_eq!(sorted_dequeued, expected);

    // While exact FIFO isn't guaranteed under high concurrency,
    // the general order should be approximately correct
    let insertion_order = insertion_order.lock().await;
    let mut out_of_order_count = 0;
    
    for i in 1..dequeued_order.len() {
        if dequeued_order[i] < dequeued_order[i-1] {
            out_of_order_count += 1;
        }
    }
    
    // Allow some reordering due to concurrency, but not too much
    let total_jobs = num_producers * jobs_per_producer;
    let max_acceptable_out_of_order = total_jobs / 10; // 10% tolerance
    assert!(out_of_order_count <= max_acceptable_out_of_order, 
            "Too many out-of-order jobs: {} out of {}", out_of_order_count, total_jobs);
}

/// Test timeout behavior under various load conditions
#[tokio::test]
async fn test_timeout_behavior_under_load() {
    let queue = Arc::new(InMemoryJobQueue::new());
    
    // Test 1: Timeout on empty queue
    let start = Instant::now();
    let result = queue.dequeue_with_timeout(Duration::from_millis(100)).await.unwrap();
    let elapsed = start.elapsed();
    
    assert!(result.is_none());
    assert!(elapsed >= Duration::from_millis(90)); // Allow some tolerance
    assert!(elapsed < Duration::from_millis(200));

    // Test 2: No timeout when jobs are available
    let job = Job::new("quick_job", &json!({}), 0).unwrap();
    queue.enqueue(job).await.unwrap();
    
    let start = Instant::now();
    let result = queue.dequeue_with_timeout(Duration::from_millis(1000)).await.unwrap();
    let elapsed = start.elapsed();
    
    assert!(result.is_some());
    assert!(elapsed < Duration::from_millis(100)); // Should be very fast

    // Test 3: Multiple timeouts concurrently
    let mut timeout_handles = vec![];
    let timeout_count = Arc::new(AtomicU64::new(0));
    
    for _ in 0..50 {
        let queue_clone = queue.clone();
        let timeout_count_clone = timeout_count.clone();
        let handle = tokio::spawn(async move {
            let start = Instant::now();
            let result = queue_clone.dequeue_with_timeout(Duration::from_millis(50)).await.unwrap();
            let elapsed = start.elapsed();
            
            if result.is_none() && elapsed >= Duration::from_millis(40) {
                timeout_count_clone.fetch_add(1, Ordering::Relaxed);
            }
        });
        timeout_handles.push(handle);
    }

    // Wait for all timeouts
    for handle in timeout_handles {
        handle.await.unwrap();
    }

    // Most (if not all) should have timed out since queue is empty
    assert!(timeout_count.load(Ordering::Relaxed) >= 45); // Allow some variance
}

/// Test queue recovery after errors
#[tokio::test]
async fn test_error_recovery() {
    let queue = Arc::new(InMemoryJobQueue::new());
    let error_count = Arc::new(AtomicU64::new(0));
    let success_count = Arc::new(AtomicU64::new(0));
    
    // Simulate operations that might fail and recover
    let mut handles = vec![];
    for thread_id in 0..10 {
        let queue_clone = queue.clone();
        let error_count_clone = error_count.clone();
        let success_count_clone = success_count.clone();
        
        let handle = tokio::spawn(async move {
            for op_id in 0..100 {
                // Simulate various operations
                if op_id % 10 == 0 {
                    // Sometimes try to dequeue from potentially empty queue
                    match queue_clone.dequeue_with_timeout(Duration::from_millis(1)).await {
                        Ok(Some(_)) => success_count_clone.fetch_add(1, Ordering::Relaxed),
                        Ok(None) => (), // Timeout is expected behavior
                        Err(_) => error_count_clone.fetch_add(1, Ordering::Relaxed),
                    }
                } else {
                    // Usually enqueue
                    let job = Job::new(
                        format!("recovery_job_{}_{}", thread_id, op_id),
                        &json!({"thread": thread_id, "op": op_id}),
                        0,
                    ).unwrap();
                    
                    match queue_clone.enqueue(job).await {
                        Ok(_) => success_count_clone.fetch_add(1, Ordering::Relaxed),
                        Err(_) => error_count_clone.fetch_add(1, Ordering::Relaxed),
                    }
                }
                
                // Small delay to allow other operations
                if op_id % 20 == 0 {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify the queue is still functional
    let final_len = queue.len().await.unwrap();
    assert!(final_len > 0); // Should have jobs remaining
    
    // Should have more successes than errors
    let successes = success_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);
    assert!(successes > errors);
    
    // Queue operations should still work
    let test_job = Job::new("final_test", &json!({"test": "recovery"}), 0).unwrap();
    assert!(queue.enqueue(test_job).await.is_ok());
    assert!(queue.dequeue_with_timeout(Duration::from_millis(100)).await.is_ok());
}

/// Test queue behavior with rapid size changes
#[tokio::test]
async fn test_rapid_size_changes() {
    let queue = Arc::new(InMemoryJobQueue::new());
    let size_samples = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let stop_sampling = Arc::new(AtomicBool::new(false));
    
    // Start size sampler
    let queue_sampler = queue.clone();
    let size_samples_clone = size_samples.clone();
    let stop_sampling_clone = stop_sampling.clone();
    let sampler_handle = tokio::spawn(async move {
        while !stop_sampling_clone.load(Ordering::Relaxed) {
            if let Ok(len) = queue_sampler.len().await {
                let mut samples = size_samples_clone.lock().await;
                samples.push(len);
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    });

    // Rapidly change queue size
    let mut operation_handles = vec![];
    for batch in 0..20 {
        let queue_clone = queue.clone();
        let handle = tokio::spawn(async move {
            // Batch enqueue
            for i in 0..50 {
                let job = Job::new(
                    format!("batch_{}_{}", batch, i),
                    &json!({"batch": batch, "index": i}),
                    0,
                ).unwrap();
                queue_clone.enqueue(job).await.unwrap();
            }
            
            // Small pause
            tokio::time::sleep(Duration::from_millis(10)).await;
            
            // Batch dequeue
            for _ in 0..25 {
                let _ = queue_clone.dequeue_with_timeout(Duration::from_millis(1)).await;
            }
        });
        operation_handles.push(handle);
    }

    // Wait for operations
    for handle in operation_handles {
        handle.await.unwrap();
    }

    // Stop sampling
    stop_sampling.store(true, Ordering::Relaxed);
    sampler_handle.await.unwrap();

    // Analyze size changes
    let samples = size_samples.lock().await;
    assert!(samples.len() > 10); // Should have collected multiple samples
    
    let max_size = samples.iter().max().unwrap();
    let min_size = samples.iter().min().unwrap();
    
    // Should have seen significant size variations
    assert!(*max_size > 100); // Should have grown significantly
    assert!(*max_size - *min_size > 50); // Should have seen substantial changes
}

/// Test queue consistency under cancellation scenarios
#[tokio::test]
async fn test_cancellation_consistency() {
    let queue = Arc::new(InMemoryJobQueue::new());
    let jobs_enqueued = Arc::new(AtomicU64::new(0));
    let jobs_dequeued = Arc::new(AtomicU64::new(0));
    
    // Start background operations that might be cancelled
    let mut handles = vec![];
    for worker_id in 0..20 {
        let queue_clone = queue.clone();
        let jobs_enqueued_clone = jobs_enqueued.clone();
        let jobs_dequeued_clone = jobs_dequeued.clone();
        
        let handle = tokio::spawn(async move {
            for op_id in 0..100 {
                // Randomly choose operation
                if op_id % 2 == 0 {
                    let job = Job::new(
                        format!("cancel_test_{}_{}", worker_id, op_id),
                        &json!({"worker": worker_id, "op": op_id}),
                        0,
                    ).unwrap();
                    
                    if queue_clone.enqueue(job).await.is_ok() {
                        jobs_enqueued_clone.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    // Long timeout to potentially get cancelled
                    match queue_clone.dequeue_with_timeout(Duration::from_millis(500)).await {
                        Ok(Some(_)) => jobs_dequeued_clone.fetch_add(1, Ordering::Relaxed),
                        _ => (), // Timeout or cancellation
                    }
                }
                
                // Yield to allow cancellation
                tokio::task::yield_now().await;
            }
        });
        handles.push(handle);
    }

    // Let operations run for a bit
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Cancel some operations
    for (i, handle) in handles.iter().enumerate() {
        if i % 3 == 0 { // Cancel every third handle
            handle.abort();
        }
    }

    // Wait for remaining handles
    for handle in handles {
        let _ = handle.await; // Ignore cancellation errors
    }

    // Queue should still be in a consistent state
    let final_len = queue.len().await.unwrap();
    let total_enqueued = jobs_enqueued.load(Ordering::Relaxed);
    let total_dequeued = jobs_dequeued.load(Ordering::Relaxed);
    
    // Basic consistency check
    assert_eq!(final_len as u64, total_enqueued - total_dequeued);
    
    // Queue operations should still work normally
    let test_job = Job::new("post_cancel_test", &json!({}), 0).unwrap();
    assert!(queue.enqueue(test_job).await.is_ok());
    assert!(queue.dequeue_with_timeout(Duration::from_millis(100)).await.unwrap().is_some());
}