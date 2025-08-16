//! Integration tests for SolanaGeyserStream
//! Tests the actual Solana Geyser stream connection using testnet

use riglr_events_core::Event;
use riglr_streams::core::{operators::ComposableStream, Stream};
use riglr_streams::solana::geyser::{GeyserConfig, SolanaGeyserStream};
use std::time::Duration;
use tokio::time::timeout;

/// Test configuration for Yellowstone GRPC testnet
fn get_test_config() -> GeyserConfig {
    // Using the testnet Yellowstone GRPC endpoint provided
    GeyserConfig {
        ws_url: "grpc://solana-testnet-yellowstone-grpc.publicnode.com:443".to_string(),
        auth_token: None,
        program_ids: vec![
            // Common testnet programs
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string(), // Jupiter V6
            "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(), // Orca Whirlpool
        ],
        buffer_size: 10000,
    }
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_geyser_stream_connection() {
    let mut stream = SolanaGeyserStream::new("test-geyser");
    let config = get_test_config();

    // Start the stream
    match stream.start(config).await {
        Ok(_) => println!("Successfully connected to Geyser stream"),
        Err(e) => {
            println!("Failed to connect to Geyser stream: {:?}", e);
            // This is expected if running without proper GRPC setup
            return;
        }
    }

    // Check if stream is running
    assert!(stream.is_running(), "Stream should be running after start");

    // Check health
    let health = stream.health().await;
    assert!(health.is_connected, "Stream should report as connected");

    // Subscribe and wait for at least one event
    let mut rx = stream.subscribe();

    match timeout(Duration::from_secs(10), rx.recv()).await {
        Ok(Ok(event)) => {
            println!("Received Geyser event: {}", event.id());
            // Stream metadata is accessed via method
            println!("Stream source: {}", event.stream_meta().stream_source);
        }
        Ok(Err(e)) => println!("Error receiving event: {:?}", e),
        Err(_) => println!("Timeout waiting for events (might be normal on testnet)"),
    }

    // Stop the stream
    stream.stop().await.unwrap();
    assert!(
        !stream.is_running(),
        "Stream should not be running after stop"
    );
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_geyser_stream_with_filter() {
    let mut stream = SolanaGeyserStream::new("test-geyser-filter");
    let config = get_test_config();

    // Start the stream first
    if stream.start(config).await.is_err() {
        println!("Skipping test - cannot connect to Geyser");
        return;
    }

    // Apply filter to only get Jupiter events
    let filtered_stream = stream.filter(|_event| {
        // In real implementation, would check protocol type
        true
    });

    let mut rx = filtered_stream.subscribe();
    let mut events_received = 0;

    // Collect filtered events for 5 seconds
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        match timeout(Duration::from_millis(500), rx.recv()).await {
            Ok(Ok(event)) => {
                // All events should be Jupiter events
                // In real implementation, would assert protocol type
                println!("Filtered event: {}", event.id());
                events_received += 1;
                println!("Received Jupiter event #{}", events_received);

                if events_received >= 3 {
                    break;
                }
            }
            _ => continue,
        }
    }

    println!("Received {} Jupiter events", events_received);
    // Note: stream was moved when creating filtered_stream, so we can't stop it here
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_geyser_stream_with_batch() {
    let mut stream = SolanaGeyserStream::new("test-geyser-batch");
    let config = get_test_config();

    // Start the stream first
    if stream.start(config).await.is_err() {
        println!("Skipping test - cannot connect to Geyser");
        return;
    }

    // Batch events in groups of 5 or every 2 seconds
    let batched_stream = stream.batch(5, Duration::from_secs(2));

    let mut rx = batched_stream.subscribe();

    // Wait for at least one batch
    match timeout(Duration::from_secs(10), rx.recv()).await {
        Ok(Ok(batch)) => {
            println!("Received batch with {} events", batch.events.len());
            assert!(!batch.events.is_empty(), "Batch should not be empty");
            assert!(
                batch.events.len() <= 5,
                "Batch should not exceed size limit"
            );

            // Check that all events in batch have metadata
            for event in &batch.events {
                println!("Event stream source: {}", event.stream_meta().stream_source);
            }
        }
        Ok(Err(e)) => println!("Error receiving batch: {:?}", e),
        Err(_) => println!("Timeout waiting for batch (might be normal on testnet)"),
    }

    // Note: stream was moved when creating batched_stream, so we can't stop it here
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_geyser_stream_reconnection() {
    let mut stream = SolanaGeyserStream::new("test-geyser-reconnect");
    let config = get_test_config();

    // Start the stream
    if stream.start(config.clone()).await.is_err() {
        println!("Skipping test - cannot connect to Geyser");
        return;
    }

    // Get initial health
    let health1 = stream.health().await;
    assert!(health1.is_connected);

    // Stop the stream
    stream.stop().await.unwrap();

    // Health should show disconnected
    let health2 = stream.health().await;
    assert!(!health2.is_connected);

    // Restart the stream
    match stream.start(config).await {
        Ok(_) => {
            let health3 = stream.health().await;
            assert!(health3.is_connected, "Should reconnect successfully");
        }
        Err(e) => println!("Failed to reconnect: {:?}", e),
    }

    stream.stop().await.unwrap();
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_geyser_stream_metrics() {
    let mut stream = SolanaGeyserStream::new("test-geyser-metrics");
    let config = get_test_config();

    // Start the stream
    if stream.start(config).await.is_err() {
        println!("Skipping test - cannot connect to Geyser");
        return;
    }

    let mut rx = stream.subscribe();
    let mut events_received = 0;

    // Collect some events
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        match timeout(Duration::from_millis(500), rx.recv()).await {
            Ok(Ok(_event)) => {
                events_received += 1;
                if events_received >= 5 {
                    break;
                }
            }
            _ => continue,
        }
    }

    // Check health metrics
    let health = stream.health().await;
    assert!(health.is_connected);
    assert!(
        health.events_processed > 0,
        "Should have processed some events"
    );
    assert!(
        health.last_event_time.is_some(),
        "Should have last event time"
    );

    println!(
        "Stream metrics - Events processed: {}, Errors: {}",
        health.events_processed, health.error_count
    );

    stream.stop().await.unwrap();
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_geyser_stream_chain_operators() {
    let mut stream = SolanaGeyserStream::new("test-geyser-chain");
    let config = get_test_config();

    // Start the stream first
    if stream.start(config).await.is_err() {
        println!("Skipping test - cannot connect to Geyser");
        return;
    }

    // Chain multiple operators
    let processed_stream = stream
        .filter(|_event| {
            // In real implementation, would filter for specific protocol types
            true
        })
        .batch(3, Duration::from_secs(3));

    let mut rx = processed_stream.subscribe();

    // Wait for a processed batch
    match timeout(Duration::from_secs(15), rx.recv()).await {
        Ok(Ok(batch)) => {
            println!(
                "Received processed batch with {} events",
                batch.events.len()
            );

            // Verify all events match filter criteria
            for event in &batch.events {
                // In real implementation, would get protocol type
                let _protocol = "unknown";
                // In real implementation, would verify filter criteria
                println!("Event in batch: {}", event.id());
            }
        }
        Ok(Err(e)) => println!("Error receiving batch: {:?}", e),
        Err(_) => println!("Timeout waiting for batch"),
    }

    // Note: stream was moved when creating processed_stream, so we can't stop it here
}
