/// Macro to generate resilient WebSocket connection logic for any stream with metrics
#[macro_export]
macro_rules! impl_resilient_websocket {
    ($stream_name:expr, $url:expr, $running:expr, $health:expr, $event_tx:expr, $metrics:expr, $connect_fn:expr, $subscribe_fn:expr, $parse_fn:expr) => {{
        use std::sync::atomic::Ordering;
        use std::time::SystemTime;
        use tokio_tungstenite::tungstenite::Message;
        use futures::{StreamExt, SinkExt};
        use tracing::{info, warn, error, debug};
        
        let mut sequence_number = 0u64;
        let mut consecutive_failures = 0;
        const MAX_RETRIES: usize = 5;
        const MAX_CONSECUTIVE_FAILURES: usize = 10;
        const MAX_IDLE_TIMEOUTS: usize = 3;
        
        while $running.load(Ordering::Relaxed) {
            let mut retry_count = 0;
            let mut connected = false;
            
            // Retry loop for connection
            while retry_count < MAX_RETRIES && !connected {
                info!("Attempting to connect to {} (attempt {}/{})", 
                      $stream_name, retry_count + 1, MAX_RETRIES);
                
                match $connect_fn().await {
                    Ok(ws_stream) => {
                        let (mut write, mut read) = ws_stream.split();
                        
                        // Send subscription messages
                        match $subscribe_fn(&mut write).await {
                            Ok(()) => {
                                info!("Successfully connected and subscribed to {}", $stream_name);
                                connected = true;
                                consecutive_failures = 0;
                                
                                // Update health - connected
                                {
                                    let mut h = $health.write().await;
                                    h.is_connected = true;
                                    h.last_event_time = Some(SystemTime::now());
                                }
                                
                                // Message reading loop
                                let mut idle_counter = 0;
                                loop {
                                    match tokio::time::timeout(
                                        tokio::time::Duration::from_secs(60),
                                        read.next()
                                    ).await {
                                        Ok(Some(Ok(Message::Text(text)))) => {
                                            idle_counter = 0;
                                            sequence_number += 1;
                                            
                                            // Track processing time
                                            let start = std::time::Instant::now();
                                            
                                            // Parse the message
                                            if let Some(event) = $parse_fn(text.clone(), sequence_number) {
                                                // Send event
                                                if let Err(e) = $event_tx.send(std::sync::Arc::new(event)) {
                                                    warn!("Failed to send event: {}", e);
                                                    // Record dropped event metric
                                                    if let Some(ref metrics) = $metrics {
                                                        metrics.record_dropped_event(&$stream_name).await;
                                                    }
                                                }
                                                
                                                // Record successful event processing
                                                if let Some(ref metrics) = $metrics {
                                                    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                                                    metrics.record_stream_event(&$stream_name, elapsed_ms, text.len() as u64).await;
                                                }
                                                
                                                // Update health
                                                let mut h = $health.write().await;
                                                h.last_event_time = Some(SystemTime::now());
                                                h.events_processed += 1;
                                            }
                                        }
                                        Ok(Some(Ok(Message::Binary(data)))) => {
                                            idle_counter = 0;
                                            sequence_number += 1;
                                            
                                            // Handle binary message if needed
                                            debug!("Received binary message of {} bytes", data.len());
                                        }
                                        Ok(Some(Ok(Message::Ping(data)))) => {
                                            idle_counter = 0;
                                            // Send pong
                                            if let Err(e) = write.send(Message::Pong(data)).await {
                                                warn!("Failed to send pong: {}", e);
                                                break;
                                            }
                                        }
                                        Ok(Some(Ok(Message::Close(_)))) => {
                                            info!("WebSocket connection closed gracefully for {}", $stream_name);
                                            break;
                                        }
                                        Ok(Some(Err(e))) => {
                                            error!("WebSocket error for {}: {}", $stream_name, e);
                                            break;
                                        }
                                        Ok(None) => {
                                            warn!("WebSocket stream ended for {}", $stream_name);
                                            break;
                                        }
                                        Err(_) => {
                                            idle_counter += 1;
                                            if idle_counter > MAX_IDLE_TIMEOUTS {
                                                warn!("WebSocket idle timeout for {}. Reconnecting...", $stream_name);
                                                break;
                                            }
                                            // Send ping to keep connection alive
                                            if let Err(e) = write.send(Message::Ping(vec![])).await {
                                                warn!("Failed to send ping: {}", e);
                                                break;
                                            }
                                        }
                                        _ => {} // Ignore other message types
                                    }
                                }
                                
                                // Connection lost - will retry
                                let mut h = $health.write().await;
                                h.is_connected = false;
                                h.error_count += 1;
                            }
                            Err(e) => {
                                error!("Failed to subscribe: {}", e);
                                retry_count += 1;
                                if retry_count < MAX_RETRIES {
                                    let delay = tokio::time::Duration::from_secs(2_u64.pow(retry_count as u32));
                                    tokio::time::sleep(delay).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to connect to {}: {}", $stream_name, e);
                        retry_count += 1;
                        if retry_count < MAX_RETRIES {
                            // Record reconnection attempt metric
                            if retry_count > 1 {
                                if let Some(ref metrics) = $metrics {
                                    metrics.record_reconnection(&$stream_name).await;
                                }
                            }
                            
                            let delay = tokio::time::Duration::from_secs(2_u64.pow(retry_count as u32));
                            info!("Retrying {} in {:?}", $stream_name, delay);
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
            }
            
            if !connected {
                consecutive_failures += 1;
                error!("Failed to connect to {} after {} retries", $stream_name, MAX_RETRIES);
                
                // Update health
                let mut h = $health.write().await;
                h.is_connected = false;
                h.error_count += 1;
                
                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    error!("Too many consecutive failures for {}. Stopping stream.", $stream_name);
                    $running.store(false, Ordering::Relaxed);
                    break;
                }
                
                // Wait before next attempt
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
        }
        
        info!("WebSocket handler exiting for {}", $stream_name);
    }};
}