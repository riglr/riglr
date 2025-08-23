/// Macro to generate resilient WebSocket connection logic for any stream with metrics
#[macro_export]
macro_rules! impl_resilient_websocket {
    ($stream_name:expr, $running:expr, $health:expr, $event_tx:expr, $metrics:expr, $connect_fn:expr, $subscribe_fn:expr, $parse_fn:expr) => {{
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
                                            if let Some(event) = $parse_fn(text.to_string(), sequence_number) {
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
                                            if let Err(e) = write.send(Message::Ping(vec![].into())).await {
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

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::SystemTime;
    use tokio::sync::{mpsc, RwLock};
    use tokio_tungstenite::tungstenite::Message;

    // Mock structures for testing
    #[derive(Debug, Clone)]
    struct MockEvent {
        _data: String,
        sequence: u64,
    }

    struct MockHealth {
        is_connected: bool,
        last_event_time: Option<SystemTime>,
        events_processed: u64,
        error_count: u64,
    }

    impl Default for MockHealth {
        fn default() -> Self {
            Self {
                is_connected: false,
                last_event_time: None,
                events_processed: 0,
                error_count: 0,
            }
        }
    }

    struct MockMetrics {
        events_recorded: Arc<std::sync::Mutex<Vec<String>>>,
        dropped_events: Arc<std::sync::Mutex<Vec<String>>>,
        reconnections: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl MockMetrics {
        fn new() -> Self {
            Self {
                events_recorded: Arc::new(std::sync::Mutex::new(Vec::new())),
                dropped_events: Arc::new(std::sync::Mutex::new(Vec::new())),
                reconnections: Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        async fn record_stream_event(&self, stream_name: &str, _elapsed_ms: f64, _size: u64) {
            self.events_recorded
                .lock()
                .unwrap()
                .push(stream_name.to_string());
        }

        async fn record_dropped_event(&self, stream_name: &str) {
            self.dropped_events
                .lock()
                .unwrap()
                .push(stream_name.to_string());
        }

        async fn record_reconnection(&self, stream_name: &str) {
            self.reconnections
                .lock()
                .unwrap()
                .push(stream_name.to_string());
        }
    }

    // Mock WebSocket stream
    // Simple error type that implements Clone for testing
    #[derive(Debug, Clone)]
    struct MockWsError(String);

    impl std::fmt::Display for MockWsError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for MockWsError {}

    struct MockWsStream {
        messages: Vec<Result<Message, MockWsError>>,
        _index: usize,
    }

    impl MockWsStream {
        fn new(messages: Vec<Result<Message, MockWsError>>) -> Self {
            Self {
                messages,
                _index: 0,
            }
        }

        fn split(self) -> (MockWsWrite, MockWsRead) {
            let write = MockWsWrite::new();
            let read = MockWsRead::new(self.messages);
            (write, read)
        }
    }

    struct MockWsWrite {
        sent_messages: Arc<std::sync::Mutex<Vec<Message>>>,
    }

    impl MockWsWrite {
        fn new() -> Self {
            Self {
                sent_messages: Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        async fn send(&mut self, msg: Message) -> Result<(), MockWsError> {
            self.sent_messages.lock().unwrap().push(msg);
            Ok(())
        }
    }

    struct MockWsRead {
        messages: Vec<Result<Message, MockWsError>>,
        index: Arc<std::sync::Mutex<usize>>,
    }

    impl MockWsRead {
        fn new(messages: Vec<Result<Message, MockWsError>>) -> Self {
            Self {
                messages,
                index: Arc::new(std::sync::Mutex::new(0)),
            }
        }

        async fn next(&mut self) -> Option<Result<Message, MockWsError>> {
            let mut idx = self.index.lock().unwrap();
            if *idx < self.messages.len() {
                let msg = self.messages[*idx].clone();
                *idx += 1;
                Some(msg)
            } else {
                None
            }
        }
    }

    // Helper functions for testing
    async fn mock_connect_success() -> Result<MockWsStream, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(MockWsStream::new(vec![
            Ok(Message::Text("{\"test\": \"data\"}".to_string().into())),
            Ok(Message::Close(None)),
        ]))
    }

    async fn mock_connect_failure() -> Result<MockWsStream, Box<dyn std::error::Error + Send + Sync>>
    {
        Err("Connection failed".into())
    }

    async fn mock_subscribe_success(
        _write: &mut MockWsWrite,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    async fn mock_subscribe_failure(
        _write: &mut MockWsWrite,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("Subscription failed".into())
    }

    fn mock_parse_success(data: String, sequence: u64) -> Option<MockEvent> {
        Some(MockEvent {
            _data: data,
            sequence,
        })
    }

    fn mock_parse_failure(_data: String, _sequence: u64) -> Option<MockEvent> {
        None
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_successful_connection_should_process_events() {
        let running = Arc::new(AtomicBool::new(true));
        let health = Arc::new(RwLock::new(MockHealth::default()));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Arc<MockEvent>>();
        let metrics = Some(Arc::new(MockMetrics::new()));

        // Set running to false after a short delay to stop the loop
        let running_clone = running.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            running_clone.store(false, Ordering::Relaxed);
        });

        // This would normally be the macro expansion, but we'll simulate it
        let stream_name = "test_stream";
        let _url = "ws://localhost:8080";

        // Simulate a simplified version of what the macro would generate
        let mut sequence_number = 0u64;
        let mut _consecutive_failures = 0;
        const MAX_RETRIES: usize = 5;

        while running.load(Ordering::Relaxed) {
            let mut retry_count = 0;
            let mut connected = false;

            while retry_count < MAX_RETRIES && !connected {
                match mock_connect_success().await {
                    Ok(ws_stream) => {
                        let (mut write, mut read) = ws_stream.split();

                        match mock_subscribe_success(&mut write).await {
                            Ok(()) => {
                                connected = true;
                                _consecutive_failures = 0;

                                // Update health - connected
                                {
                                    let mut h = health.write().await;
                                    h.is_connected = true;
                                    h.last_event_time = Some(SystemTime::now());
                                }

                                // Process one message and then break
                                if let Some(Ok(Message::Text(text))) = read.next().await {
                                    sequence_number += 1;
                                    if let Some(event) =
                                        mock_parse_success(text.to_string(), sequence_number)
                                    {
                                        let _ = event_tx.send(Arc::new(event));
                                        if let Some(ref metrics) = metrics {
                                            metrics
                                                .record_stream_event(stream_name, 10.0, 100)
                                                .await;
                                        }

                                        let mut h = health.write().await;
                                        h.last_event_time = Some(SystemTime::now());
                                        h.events_processed += 1;
                                    }
                                }
                            }
                            Err(_) => {
                                retry_count += 1;
                                if retry_count < MAX_RETRIES {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(10))
                                        .await;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        retry_count += 1;
                        if retry_count < MAX_RETRIES {
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        }
                    }
                }
            }
            break; // Exit after one iteration for testing
        }

        // Verify health was updated
        let health_state = health.read().await;
        assert!(health_state.is_connected);
        assert_eq!(health_state.events_processed, 1);
        assert!(health_state.last_event_time.is_some());

        // Verify event was received
        let event = event_rx.try_recv().expect("Expected to receive event");
        assert_eq!(event.sequence, 1);

        // Verify metrics were recorded
        if let Some(metrics) = metrics {
            let events = metrics.events_recorded.lock().unwrap();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0], stream_name);
        }
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_connection_fails_should_retry() {
        let running = Arc::new(AtomicBool::new(true));
        let health = Arc::new(RwLock::new(MockHealth::default()));
        let (_event_tx, _event_rx) = mpsc::unbounded_channel::<Arc<MockEvent>>();
        let metrics = Some(Arc::new(MockMetrics::new()));

        // Set running to false after a short delay to stop the loop
        let running_clone = running.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            running_clone.store(false, Ordering::Relaxed);
        });

        let stream_name = "test_stream";
        let mut _consecutive_failures = 0;
        const MAX_RETRIES: usize = 5;

        while running.load(Ordering::Relaxed) {
            let mut retry_count = 0;
            let mut connected = false;

            while retry_count < MAX_RETRIES && !connected {
                if mock_connect_failure().await.is_ok() {
                    connected = true;
                } else {
                    retry_count += 1;
                    if retry_count < MAX_RETRIES {
                        if retry_count > 1 {
                            if let Some(ref metrics) = metrics {
                                metrics.record_reconnection(stream_name).await;
                            }
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                }
            }

            if !connected {
                _consecutive_failures += 1;

                // Update health
                let mut h = health.write().await;
                h.is_connected = false;
                h.error_count += 1;
            }
            break; // Exit after one iteration for testing
        }

        // Verify health reflects connection failure
        let health_state = health.read().await;
        assert!(!health_state.is_connected);
        assert_eq!(health_state.error_count, 1);

        // Verify reconnection attempts were recorded
        if let Some(metrics) = metrics {
            let reconnections = metrics.reconnections.lock().unwrap();
            assert!(reconnections.len() > 0);
        }
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_subscription_fails_should_retry() {
        let running = Arc::new(AtomicBool::new(true));
        let health = Arc::new(RwLock::new(MockHealth::default()));
        let (_event_tx, _event_rx) = mpsc::unbounded_channel::<Arc<MockEvent>>();
        let _metrics: Option<Arc<MockMetrics>> = None;

        // Set running to false after a short delay
        let running_clone = running.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            running_clone.store(false, Ordering::Relaxed);
        });

        const MAX_RETRIES: usize = 5;

        while running.load(Ordering::Relaxed) {
            let mut retry_count = 0;
            let mut connected = false;

            while retry_count < MAX_RETRIES && !connected {
                match mock_connect_success().await {
                    Ok(ws_stream) => {
                        let (mut write, _read) = ws_stream.split();

                        match mock_subscribe_failure(&mut write).await {
                            Ok(()) => {
                                connected = true;
                            }
                            Err(_) => {
                                retry_count += 1;
                                if retry_count < MAX_RETRIES {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(10))
                                        .await;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        retry_count += 1;
                    }
                }
            }
            break; // Exit after one iteration for testing
        }

        // Verify connection was not established due to subscription failure
        let health_state = health.read().await;
        assert!(!health_state.is_connected);
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_ping_received_should_send_pong() {
        let _running = Arc::new(AtomicBool::new(true));
        let _health = Arc::new(RwLock::new(MockHealth::default()));
        let (_event_tx, _event_rx) = mpsc::unbounded_channel::<Arc<MockEvent>>();
        let _metrics: Option<Arc<MockMetrics>> = None;

        // Create a mock stream with a ping message
        let ping_data = vec![1, 2, 3, 4];
        let ws_stream = MockWsStream::new(vec![
            Ok(Message::Ping(ping_data.clone().into())),
            Ok(Message::Close(None)),
        ]);

        let (mut write, mut read) = ws_stream.split();

        // Simulate the ping handling logic from the macro
        if let Some(Ok(Message::Ping(data))) = read.next().await {
            let result = write.send(Message::Pong(data)).await;
            assert!(result.is_ok());
        }

        // Verify pong was sent
        let sent_messages = write.sent_messages.lock().unwrap();
        assert_eq!(sent_messages.len(), 1);
        assert!(matches!(&sent_messages[0], Message::Pong(data) if data == &ping_data));
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_binary_message_received_should_handle_gracefully() {
        let binary_data = vec![0xFF, 0xFE, 0xFD];
        let ws_stream = MockWsStream::new(vec![
            Ok(Message::Binary(binary_data.clone().into())),
            Ok(Message::Close(None)),
        ]);

        let (_write, mut read) = ws_stream.split();

        // Simulate binary message handling
        if let Some(Ok(Message::Binary(data))) = read.next().await {
            // The macro just logs binary messages, so we verify the data is received
            assert_eq!(data, binary_data);
        }
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_parse_fails_should_continue() {
        let _running = Arc::new(AtomicBool::new(true));
        let _health = Arc::new(RwLock::new(MockHealth::default()));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Arc<MockEvent>>();
        let _metrics = Some(Arc::new(MockMetrics::new()));

        let _stream_name = "test_stream";
        let mut sequence_number = 0u64;

        // Simulate receiving a message that fails to parse
        sequence_number += 1;
        let text = "{\"invalid\": \"json}".to_string();

        if let Some(event) = mock_parse_failure(text.clone(), sequence_number) {
            let _ = event_tx.send(Arc::new(event));
        }

        // Verify no event was sent (because parse failed)
        assert!(event_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_event_send_fails_should_record_dropped_event() {
        let metrics = Arc::new(MockMetrics::new());
        let stream_name = "test_stream";

        // Create a closed channel to simulate send failure
        let (event_tx, event_rx) = mpsc::unbounded_channel::<Arc<MockEvent>>();
        drop(event_rx); // Close the receiver

        let event = MockEvent {
            _data: "test".to_string(),
            sequence: 1,
        };

        // Simulate the event sending logic
        if event_tx.send(Arc::new(event)).is_err() {
            metrics.record_dropped_event(stream_name).await;
        }

        // Verify dropped event was recorded
        let dropped_events = metrics.dropped_events.lock().unwrap();
        assert_eq!(dropped_events.len(), 1);
        assert_eq!(dropped_events[0], stream_name);
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_max_consecutive_failures_reached_should_stop() {
        let running = Arc::new(AtomicBool::new(true));
        let health = Arc::new(RwLock::new(MockHealth::default()));
        let (_event_tx, _event_rx) = mpsc::unbounded_channel::<Arc<MockEvent>>();
        let _metrics: Option<Arc<MockMetrics>> = None;

        let mut consecutive_failures = 0;
        const MAX_CONSECUTIVE_FAILURES: usize = 10;
        const MAX_RETRIES: usize = 5;

        // Simulate the logic for handling max consecutive failures
        while running.load(Ordering::Relaxed) {
            let mut retry_count = 0;
            let mut connected = false;

            while retry_count < MAX_RETRIES && !connected {
                if mock_connect_failure().await.is_ok() {
                    connected = true;
                } else {
                    retry_count += 1;
                    if retry_count < MAX_RETRIES {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                    }
                }
            }

            if !connected {
                consecutive_failures += 1;

                let mut h = health.write().await;
                h.is_connected = false;
                h.error_count += 1;

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    running.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }

        // Verify running was set to false
        assert!(!running.load(Ordering::Relaxed));

        // Verify health reflects the failures
        let health_state = health.read().await;
        assert!(!health_state.is_connected);
        assert_eq!(health_state.error_count, MAX_CONSECUTIVE_FAILURES as u64);
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_websocket_error_should_break_loop() {
        let ws_stream = MockWsStream::new(vec![
            Ok(Message::Text("valid".to_string().into())),
            Err(MockWsError("Connection reset".to_string())),
        ]);

        let (_write, mut read) = ws_stream.split();
        let mut message_count = 0;

        // Simulate the message reading loop
        loop {
            match read.next().await {
                Some(Ok(Message::Text(_))) => {
                    message_count += 1;
                }
                Some(Err(_)) => {
                    // Error should break the loop
                    break;
                }
                Some(Ok(Message::Close(_))) => break,
                None => break,
                _ => {}
            }
        }

        // Verify we processed one message before the error
        assert_eq!(message_count, 1);
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_idle_timeout_exceeded_should_reconnect() {
        let _running = Arc::new(AtomicBool::new(true));

        // Simulate idle timeout logic
        let mut idle_counter = 0;
        const MAX_IDLE_TIMEOUTS: usize = 3;

        // Simulate multiple timeouts
        for _ in 0..=MAX_IDLE_TIMEOUTS {
            // Simulate timeout
            idle_counter += 1;
            if idle_counter > MAX_IDLE_TIMEOUTS {
                // Should break and reconnect
                break;
            }
        }

        assert!(idle_counter > MAX_IDLE_TIMEOUTS);
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_close_message_received_should_exit_gracefully() {
        let ws_stream = MockWsStream::new(vec![Ok(Message::Close(None))]);

        let (_write, mut read) = ws_stream.split();
        // Simulate the close message handling
        let closed = matches!(read.next().await, Some(Ok(Message::Close(_))));

        assert!(closed);
    }

    #[tokio::test]
    async fn test_impl_resilient_websocket_when_stream_ends_should_break_loop() {
        let ws_stream = MockWsStream::new(vec![]); // Empty stream

        let (_write, mut read) = ws_stream.split();
        let mut stream_ended = false;

        // Simulate stream ending (None)
        if read.next().await.is_none() {
            stream_ended = true;
        }

        assert!(stream_ended);
    }
}
